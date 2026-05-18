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

### `[resilience]`

Controls automatic card recovery behavior. All keys are optional; defaults cover typical homelab use.

| Key | Type | Default | Description |
|---|---|---|---|
| `initial_backoff_sec` | integer | 5 | Delay before the first recovery retry (seconds). Range: 1-600 |
| `max_backoff_sec` | integer | 120 | Maximum backoff delay after repeated failures (seconds). Range: 1-3600 |
| `max_retries` | integer | 10 | Give-up threshold: stop retrying a slot after this many consecutive failures. Range: 1-1000 |
| `network_loss_timeout_sec` | integer | 60 | Seconds of failed network registration before recovery is triggered. Range: 10-600 |
| `network_poll_interval_sec` | integer | 30 | How often to poll the modem for network registration status (seconds). Range: 5-300 |

### `[control]`

Configures the Unix domain socket used by `card` CLI subcommands to communicate with the running daemon.

| Key | Type | Default | Description |
|---|---|---|---|
| `socket_path` | string | `/tmp/gsm-sip-bridge.sock` | Filesystem path for the control socket. Must be writable by the bridge process and readable by CLI users. |

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

[resilience]
initial_backoff_sec = 5
max_backoff_sec = 120
max_retries = 10
network_loss_timeout_sec = 60
network_poll_interval_sec = 30

[control]
socket_path = "/run/gsm-sip-bridge/control.sock"
```
