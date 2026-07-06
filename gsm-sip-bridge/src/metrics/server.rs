use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Router};
use prometheus::TextEncoder;
use std::net::SocketAddr;
use std::time::Instant;

use super::web_state::{SharedSlots, WebSlotInfo};

static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

pub fn record_start_time() {
    START_TIME.get_or_init(Instant::now);
}

async fn metrics_handler() -> impl IntoResponse {
    if let Some(start) = START_TIME.get() {
        super::UPTIME_SECONDS.set(start.elapsed().as_secs_f64());
    }

    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();

    match encoder.encode_to_string(&metric_families) {
        Ok(output) => (
            StatusCode::OK,
            [("Content-Type", "text/plain; version=0.0.4")],
            output,
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "failed to encode metrics");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn dashboard_handler(State(slots): State<SharedSlots>) -> impl IntoResponse {
    let uptime_secs = match START_TIME.get() {
        Some(start) => start.elapsed().as_secs_f64(),
        None => 0.0,
    };
    let sip_ok = super::SIP_REGISTERED.get() > 0.5;

    let rows = {
        let guard = slots.read().unwrap();
        build_rows(&guard)
    };

    let html = render_page(&rows, uptime_secs, sip_ok);

    (
        StatusCode::OK,
        [("Content-Type", "text/html; charset=utf-8")],
        html,
    )
}

fn build_rows(slots: &[WebSlotInfo]) -> String {
    if slots.is_empty() {
        return "<tr><td colspan=\"6\" style=\"text-align:center;color:#64748b;padding:24px;\">No cards found</td></tr>".into();
    }
    let mut rows = String::new();
    for s in slots {
        let state_class = match s.state.as_str() {
            "Ready" => "state-ready",
            "Initializing" => "state-initializing",
            "Recovering" => "state-recovering",
            "GivenUp" => "state-given-up",
            _ => "",
        };
        rows.push_str(&format!(
            r#"<tr><td>{}</td><td style="font-family:monospace;font-size:0.8rem;">{}</td><td>{}</td><td class="{}">{}</td><td>{}</td><td>{}</td></tr>"#,
            s.slot,
            s.imei,
            s.phone,
            state_class,
            s.state,
            s.network,
            if s.active_call { "🔊 Active" } else { "Idle" },
        ));
    }
    rows
}

fn render_page(rows: &str, uptime_secs: f64, sip_ok: bool) -> String {
    let uptime = format_uptime(uptime_secs);
    let sip_status = if sip_ok { "Registered" } else { "Unregistered" };
    let sip_badge = if sip_ok { "badge-green" } else { "badge-red" };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>GSM-SIP Bridge</title>
<style>
  *{{margin:0;padding:0;box-sizing:border-box;}}
  body{{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,Oxygen,Ubuntu,Cantarell,sans-serif;padding:24px;background:#0f172a;color:#e2e8f0;}}
  h1{{font-size:1.35rem;font-weight:600;}}
  .header{{display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:20px;flex-wrap:wrap;gap:12px;}}
  .subtitle{{font-size:0.8rem;color:#64748b;margin-top:2px;}}
  .status-bar{{display:flex;gap:24px;margin-bottom:20px;padding:12px 16px;background:#1e293b;border-radius:8px;font-size:0.85rem;flex-wrap:wrap;}}
  .stat-item{{white-space:nowrap;}}
  .badge{{display:inline-block;padding:1px 10px;border-radius:999px;font-size:0.75rem;font-weight:600;}}
  .badge-green{{background:#166534;color:#bbf7d0;}}
  .badge-red{{background:#991b1b;color:#fecaca;}}
  table{{width:100%;border-collapse:collapse;background:#1e293b;border-radius:8px;overflow:hidden;}}
  th{{text-align:left;padding:10px 12px;font-size:0.7rem;text-transform:uppercase;letter-spacing:0.08em;color:#64748b;font-weight:600;border-bottom:1px solid #334155;}}
  td{{padding:10px 12px;border-bottom:1px solid #334155;font-size:0.85rem;}}
  tr:last-child td{{border-bottom:none;}}
  .state-ready{{color:#bbf7d0;}}
  .state-recovering{{color:#fef08a;}}
  .state-given-up{{color:#fecaca;}}
  .state-initializing{{color:#93c5fd;}}
  .refresh{{color:#64748b;font-size:0.75rem;text-align:right;}}
</style>
</head>
<body>
<div class="header">
  <div>
    <h1>GSM-SIP Bridge</h1>
    <div class="subtitle">Dashboard</div>
  </div>
  <div class="refresh" id="lastUpdate">Loading…</div>
</div>
<div class="status-bar">
  <span class="stat-item">SIP: <span class="badge {sip_badge}">{sip_status}</span></span>
  <span class="stat-item">Uptime: {uptime}</span>
</div>
<table>
  <thead><tr><th>Slot</th><th>IMEI</th><th>Phone</th><th>State</th><th>Network</th><th>Call</th></tr></thead>
  <tbody>{rows}</tbody>
</table>
<script>
document.getElementById('lastUpdate').textContent = 'Updated: ' + new Date().toLocaleTimeString();
setInterval(function(){{location.reload();}},5000);
</script>
</body>
</html>"#,
        rows = rows,
        sip_badge = sip_badge,
        sip_status = sip_status,
        uptime = uptime,
    )
}

fn format_uptime(secs: f64) -> String {
    let total = secs as u64;
    let days = total / 86400;
    let hours = (total % 86400) / 3600;
    let minutes = (total % 3600) / 60;
    let seconds = total % 60;
    if days > 0 {
        format!("{days}d {hours}h {minutes}m {seconds}s")
    } else if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}

pub async fn serve(
    port: u16,
    slots: SharedSlots,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/", get(dashboard_handler))
        .with_state(slots);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::info!(port = port, "metrics server starting");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
