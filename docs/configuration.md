# Configuration Reference

The bridge reads a single TOML configuration file specified via `--config`.

## Sections

### `[sip]` (required)

| Key | Type | Default | Description |
|---|---|---|---|
| `server` | string | (required) | PBX hostname or IP |
| `port` | integer | 5060 | SIP port |
| `username` | string | (required) | SIP account |
| `password` | string | (required) | Supports `env:VAR` syntax |
| `transport` | enum | `udp` | `udp`, `tcp`, or `tls` |
| `local_port` | integer | 5060 | Fixed local port |
| `display_name` | string | username | Callee display |
| `tls_verify` | enum | `strict` | `strict` or `skip` |

### `[bridge]`

| Key | Type | Default | Description |
|---|---|---|---|
| `sip_destination` | string | `""` | Empty = DID passthrough |
| `sip_dial_timeout_sec` | integer | 30 | Range: 5-120 |

### `[sms]`

| Key | Type | Default | Description |
|---|---|---|---|
| `enabled` | boolean | true | Disable SMS monitoring |
| `discord_webhook_url` | string | `""` | Supports `env:VAR` syntax |
| `db_path` | string | `/var/lib/gsm-sip-bridge/store.db` | Store path |

### `[metrics]`

| Key | Type | Default | Description |
|---|---|---|---|
| `port` | integer | 9091 | `METRICS_PORT` env var wins |

### `[modules]`

| Key | Type | Default | Description |
|---|---|---|---|
| `retry_interval_sec` | integer | 30 | Range: 5-600 |
| `max_concurrent` | integer | 8 | Range: 1-8 |

## Examples

### Single-card development

```toml
[sip]
server = "127.0.0.1"
port = 5060
username = "test"
password = "test"
transport = "udp"

[sms]
enabled = false
```

### Production multi-card with TLS

```toml
[sip]
server = "pbx.example.com"
port = 5061
username = "gsm-bridge"
password = "env:SIP_PASSWORD"
transport = "tls"
tls_verify = "strict"
display_name = "GSM Bridge"

[bridge]
sip_destination = ""
sip_dial_timeout_sec = 30

[sms]
enabled = true
discord_webhook_url = "env:DISCORD_WEBHOOK_URL"
db_path = "/data/store.db"

[metrics]
port = 9091

[modules]
retry_interval_sec = 30
max_concurrent = 8
```
