use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};
use prometheus::TextEncoder;
use std::net::SocketAddr;
use std::time::Instant;

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

pub async fn serve(port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = Router::new().route("/metrics", get(metrics_handler));
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::info!(port = port, "metrics server starting");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
