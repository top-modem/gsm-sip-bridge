# Operations Guide

## CLI Card Management

The `card` subcommands talk to the running daemon over its control socket (default: `/tmp/gsm-sip-bridge.sock`). The daemon must be running.

```bash
# Show all known slots: state, phone number, network type
gsm-sip-bridge --config config.toml card list

# Restart a slot (safe to run while other cards are active; resets give-up state)
gsm-sip-bridge --config config.toml card restart --slot 0

# Set network preference for a slot and persist it
# Valid modes: 2g, 3g, 4g, auto
gsm-sip-bridge --config config.toml card set-mode --slot 0 --mode 4g

# Query the stored network mode preference
gsm-sip-bridge --config config.toml card get-mode --slot 0
```

Network mode preferences survive daemon restarts and are re-applied automatically whenever a card initialises (cold start or after recovery).

## Querying the Store

Connect to the SQLite store directly:

```bash
sqlite3 /var/lib/gsm-sip-bridge/store.db
```

Useful queries:

```sql
-- Recent calls
SELECT * FROM recent_calls;

-- Recent SMS
SELECT * FROM recent_sms;

-- Calls by module
SELECT * FROM calls WHERE module_id = 'ec20-A1B2C3' ORDER BY id DESC LIMIT 20;

-- Failed SMS forwards
SELECT * FROM sms WHERE forwarding_status = 'failed';

-- IMEI → slot assignments
SELECT slot, imei, assigned_at FROM card_slots ORDER BY slot;

-- Stored network mode preferences
SELECT slot, mode, updated_at FROM card_mode_prefs ORDER BY slot;
```

## Manual Prune

The bridge does not auto-prune. Run periodically:

```sql
DELETE FROM calls WHERE started_at < datetime('now', '-365 days');
DELETE FROM sms WHERE received_at < datetime('now', '-365 days');
VACUUM;
```

## WAL Checkpoint

SQLite WAL files grow during writes. Force a checkpoint:

```sql
PRAGMA wal_checkpoint(TRUNCATE);
```

## Backup

```bash
sqlite3 /var/lib/gsm-sip-bridge/store.db ".backup /backup/store-$(date +%Y%m%d).db"
```

## Troubleshooting

### Module shows FAILED at startup

Check:
1. USB device connected: `lsusb | grep 2c7c:0125`
2. Serial port accessible: `ls -la /dev/ttyUSB*`
3. ModemManager not interfering: `systemctl is-active ModemManager`
4. Permissions: user must be in `dialout` group

### Card is in GivenUp state (stopped retrying)

A slot stops retrying after `[resilience] max_retries` consecutive failures and emits a `CRITICAL` log. To re-enable it:

```bash
gsm-sip-bridge --config config.toml card restart --slot <N>
```

This resets the give-up counter and triggers a fresh initialization attempt.

### Card recovery not triggering after USB re-plug

The bridge detects USB disconnect via a serial read error on the AT port. If the device re-enumerates but the slot stays in `Recovering`:
1. Check that the IMEI in `card_slots` matches the re-plugged modem (`sqlite3 store.db "SELECT * FROM card_slots;"`).
2. Verify no other process holds the ttyUSB port: `fuser /dev/ttyUSB*`
3. Force a restart: `gsm-sip-bridge --config config.toml card restart --slot <N>`

### Control socket not reachable

```
error: daemon not running or socket unreachable: /tmp/gsm-sip-bridge.sock
```

1. Verify the daemon is running: `ps aux | grep gsm-sip-bridge`
2. Check the configured socket path matches: `[control] socket_path` in `config.toml`
3. Check filesystem permissions on the socket directory

### SIP registration failing

Check:
1. PBX reachable: `nc -zuv <server> <port>`
2. Credentials correct in config.toml
3. Transport matches PBX (udp/tcp/tls)
4. If TLS: check `tls_verify` setting

### Discord forwarding failing

Check:
1. Webhook URL valid (test with curl)
2. Network connectivity from bridge host
3. Check `sms` table for `forwarding_status = 'failed'` with `discord_status_code`

### Metrics endpoint returns 5xx

Check:
1. Port not in use: `ss -tlnp | grep 9091`
2. Bridge process running: `ps aux | grep gsm-sip-bridge`

### Store.db corrupt

1. Stop the bridge
2. Run: `sqlite3 /var/lib/gsm-sip-bridge/store.db "PRAGMA integrity_check;"`
3. If corrupt, restore from backup
4. Restart the bridge (it will create a fresh DB if needed)
