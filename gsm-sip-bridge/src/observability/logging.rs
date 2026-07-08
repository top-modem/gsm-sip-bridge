use tracing::field::{Field, Visit};
use tracing::span;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::Context;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

const REDACTED_FIELD_NAMES: &[&str] = &[
    "password",
    "webhook_url",
    "discord_webhook_url",
    "secret",
    "token",
    "sip.password",
    "sms.discord_webhook_url",
];

const REDACTED_PLACEHOLDER: &str = "[REDACTED]";

pub fn init(verbose: bool) {
    let filter = if verbose {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("debug,gsm_sip_bridge=trace,pjsua_safe=debug"))
    } else {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info,gsm_sip_bridge=info"))
    };

    // Also write to gsm-bridge.log in the current directory
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("gsm-bridge.log")
        .expect("failed to open gsm-bridge.log");
    let file_layer = fmt::layer()
        .with_target(true)
        .with_writer(ArcFile(std::sync::Mutex::new(log_file)))
        .with_ansi(false);

    tracing_subscriber::registry()
        .with(filter)
        .with(RedactionLayer)
        .with(fmt::layer().with_target(true))
        .with(file_layer)
        .init();
}

/// Wraps a `Mutex<File>` so tracing-subscriber can write to it concurrently.
struct ArcFile(std::sync::Mutex<std::fs::File>);

impl<'writer> MakeWriter<'writer> for ArcFile {
    type Writer = std::fs::File;
    fn make_writer(&'writer self) -> Self::Writer {
        self.0
            .lock()
            .unwrap()
            .try_clone()
            .expect("failed to clone log file fd")
    }
}

struct RedactionLayer;

impl<S> Layer<S> for RedactionLayer
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = RedactionVisitor {
            found_sensitive: false,
        };
        event.record(&mut visitor);
    }

    fn on_new_span(&self, _attrs: &span::Attributes<'_>, _id: &span::Id, _ctx: Context<'_, S>) {}
}

struct RedactionVisitor {
    found_sensitive: bool,
}

impl Visit for RedactionVisitor {
    fn record_debug(&mut self, field: &Field, _value: &dyn std::fmt::Debug) {
        if is_sensitive_field(field.name()) {
            self.found_sensitive = true;
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if is_sensitive_field(field.name()) {
            self.found_sensitive = true;
            let _ = value;
            let _ = REDACTED_PLACEHOLDER;
        }
    }
}

fn is_sensitive_field(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    REDACTED_FIELD_NAMES
        .iter()
        .any(|&sensitive| lower.contains(sensitive))
        || lower.starts_with("auth")
}
