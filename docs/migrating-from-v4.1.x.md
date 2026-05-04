# Migrating from v4.1.x to v5.0.0

This guide covers all steps needed to transition from the C++ gsm-sip-bridge v4.1.x to the Rust rewrite v5.0.0.

## Overview

v5.0.0 is a clean break: new binary, new config format (TOML), new database schema, renamed metrics, and a new Docker Compose stack. The old deployment is left untouched so you can roll back.

## Configuration Mapping (INI to TOML)

| v4.1.x (`config.ini`) | v5.0.0 (`config.toml`) | Notes |
|---|---|---|
| `[sip] server` | `[sip] server` | Unchanged |
| `[sip] port` | `[sip] port` | Unchanged |
| `[sip] username` | `[sip] username` | Unchanged |
| `[sip] password` | `[sip] password` | Now supports `env:VAR_NAME` syntax |
| `[sip] transport` | `[sip] transport` | Unchanged |
| `[sip] local_port` | `[sip] local_port` | Unchanged |
| `[sip] display_name` | `[sip] display_name` | Unchanged |
| `[bridge] sip_destination` | `[bridge] sip_destination` | Empty string = DID passthrough |
| `[bridge] sip_dial_timeout_sec` | `[bridge] sip_dial_timeout_sec` | Unchanged |
| `[sms] enabled` | `[sms] enabled` | Unchanged |
| `[sms] discord_webhook_url` | `[sms] discord_webhook_url` | Supports `env:` references |
| `[sms] db_path` | `[sms] db_path` | Default changed to `/var/lib/gsm-sip-bridge/store.db` |
| (new) | `[sip] tls_verify` | `strict` (default) or `skip` |
| (new) | `[metrics] port` | Default 9091 |
| (new) | `[modules] retry_interval_sec` | Default 30 |
| (new) | `[modules] max_concurrent` | Default 8 |

## Database Conversion

The v5.0.0 store uses a new schema. Run this SQL to migrate your existing `sms.db`:

```sql
-- Create the new store.db alongside the old sms.db
-- Run with: sqlite3 /var/lib/gsm-sip-bridge/store.db < migrate.sql

ATTACH DATABASE '/path/to/old/sms.db' AS old;

CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
INSERT OR IGNORE INTO meta(key, value) VALUES ('schema_version', '1');

CREATE TABLE IF NOT EXISTS calls (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    module_id TEXT NOT NULL,
    caller_id TEXT NOT NULL DEFAULT '',
    started_at TEXT NOT NULL,
    duration_seconds REAL NOT NULL DEFAULT 0.0,
    status TEXT NOT NULL CHECK (status IN ('answered','missed','failed')),
    sip_destination TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS sms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    module_id TEXT NOT NULL,
    sender TEXT NOT NULL,
    body TEXT NOT NULL,
    received_at TEXT NOT NULL,
    forwarding_status TEXT NOT NULL CHECK (forwarding_status IN ('pending','sent','failed','skipped')),
    forwarded_at TEXT,
    discord_status_code INTEGER
);

INSERT INTO sms (module_id, sender, body, received_at, forwarding_status, forwarded_at, discord_status_code)
SELECT
    COALESCE(module_id, 'ec20-LEGACY'),
    sender,
    body,
    received_at,
    CASE
        WHEN discord_status_code BETWEEN 200 AND 299 THEN 'sent'
        WHEN discord_status_code IS NOT NULL THEN 'failed'
        ELSE 'skipped'
    END,
    forwarded_at,
    discord_status_code
FROM old.sms;

DETACH DATABASE old;
VACUUM;
```

The original `sms.db` is left untouched.

## Metrics Rename Mapping

| v4.1.x | v5.0.0 |
|---|---|
| `gsm_bridge_calls_total` | `gsm_sip_bridge_calls_total` |
| `gsm_bridge_sip_calls_total` | `gsm_sip_bridge_sip_calls_total` |
| `gsm_bridge_sip_registrations_total` | `gsm_sip_bridge_sip_registrations_total` |
| `gsm_bridge_module_init_total` | `gsm_sip_bridge_module_init_total` |
| `gsm_bridge_module_retries_total` | `gsm_sip_bridge_module_retries_total` |
| `gsm_bridge_audio_errors_total` | `gsm_sip_bridge_audio_errors_total` |
| `gsm_bridge_sip_registered` | `gsm_sip_bridge_sip_registered` |
| `gsm_bridge_modules_active` | `gsm_sip_bridge_modules_active` |
| `gsm_bridge_modules_failed` | `gsm_sip_bridge_modules_failed` |
| `gsm_bridge_active_calls` | `gsm_sip_bridge_active_calls` |
| `gsm_bridge_uptime_seconds` | `gsm_sip_bridge_uptime_seconds` |
| `gsm_bridge_sms_received_total` | `gsm_sip_bridge_sms_received_total` |
| `gsm_bridge_sms_forwarded_total` | `gsm_sip_bridge_sms_forwarded_total` |
| `gsm_bridge_sms_db_writes_total` | `gsm_sip_bridge_sms_db_writes_total` |
| `gsm_bridge_call_duration_seconds` | `gsm_sip_bridge_call_duration_seconds` |

For Grafana panels, find-and-replace `gsm_bridge_` with `gsm_sip_bridge_`.

## CLI Flag Mapping

| v4.1.x | v5.0.0 |
|---|---|
| `--config config.ini` | `--config config.toml` |
| `--verbose` | `--verbose` |
| `-s /dev/ttyUSB3 -a hw:2,0` | `-s /dev/ttyUSB3 -a hw:2,0` |

## Docker Compose

Replace your existing `docker-compose.yml` with `docker/docker-compose.yml` from the v5.0.0 release.

## Roll-back

1. Stop the v5.0.0 container: `docker compose down`
2. Restore your v4.1.x `docker-compose.yml`
3. Start: `docker compose up -d`
4. The v4.1.x binary, `config.ini`, and `sms.db` are unmodified.
