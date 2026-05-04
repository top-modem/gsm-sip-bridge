# Operations Guide

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
