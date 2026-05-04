mod common;

use gsm_sip_bridge::metrics;
use std::sync::Once;

static INIT: Once = Once::new();

fn init_all_metrics() {
    INIT.call_once(|| {});
    metrics::CALLS_TOTAL.with_label_values(&["test", "answered"]).inc();
    metrics::SIP_CALLS_TOTAL.with_label_values(&["test", "success"]).inc();
    metrics::CALL_DURATION_SECONDS.with_label_values(&["test"]).observe(1.0);
    metrics::ACTIVE_CALLS.with_label_values(&["test"]).set(0.0);
    metrics::SIP_REGISTRATIONS_TOTAL.with_label_values(&["success"]).inc();
    metrics::SIP_REGISTERED.set(0.0);
    metrics::MODULE_INIT_TOTAL.with_label_values(&["test", "success", "none"]).inc();
    metrics::MODULE_RETRIES_TOTAL.with_label_values(&["test"]).inc();
    metrics::MODULES_ACTIVE.set(0.0);
    metrics::MODULES_FAILED.set(0.0);
    metrics::AUDIO_ERRORS_TOTAL.with_label_values(&["test", "underrun"]).inc();
    metrics::SMS_RECEIVED_TOTAL.with_label_values(&["test"]).inc();
    metrics::SMS_FORWARDED_TOTAL.with_label_values(&["test", "sent"]).inc();
    metrics::SMS_DB_WRITES_TOTAL.with_label_values(&["success"]).inc();
    metrics::UPTIME_SECONDS.set(1.0);
    metrics::BUILD_INFO.with_label_values(&["5.0.0", "test", "2.16", "1.80.0"]).set(1.0);
}

#[test]
fn test_build_info_metric_has_labels() {
    init_all_metrics();
    metrics::BUILD_INFO.reset();
    metrics::BUILD_INFO
        .with_label_values(&["5.0.0", "abc1234", "2.16", "1.80.0"])
        .set(1.0);

    let encoder = prometheus::TextEncoder::new();
    let families = prometheus::gather();
    let output = encoder.encode_to_string(&families).unwrap();

    assert!(output.contains("gsm_sip_bridge_build_info"));
    assert!(output.contains("version=\"5.0.0\""));
    assert!(output.contains("git_sha=\"abc1234\""));
}

#[test]
fn test_all_metrics_registered() {
    init_all_metrics();

    let encoder = prometheus::TextEncoder::new();
    let families = prometheus::gather();
    let output = encoder.encode_to_string(&families).unwrap();

    let expected_metrics = [
        "gsm_sip_bridge_calls_total",
        "gsm_sip_bridge_sip_calls_total",
        "gsm_sip_bridge_sip_registrations_total",
        "gsm_sip_bridge_module_init_total",
        "gsm_sip_bridge_module_retries_total",
        "gsm_sip_bridge_audio_errors_total",
        "gsm_sip_bridge_sip_registered",
        "gsm_sip_bridge_modules_active",
        "gsm_sip_bridge_modules_failed",
        "gsm_sip_bridge_active_calls",
        "gsm_sip_bridge_uptime_seconds",
        "gsm_sip_bridge_sms_received_total",
        "gsm_sip_bridge_sms_forwarded_total",
        "gsm_sip_bridge_sms_db_writes_total",
        "gsm_sip_bridge_call_duration_seconds",
        "gsm_sip_bridge_build_info",
    ];

    for metric in &expected_metrics {
        assert!(
            output.contains(metric),
            "missing metric: {metric}\n\nActual output:\n{output}"
        );
    }
}
