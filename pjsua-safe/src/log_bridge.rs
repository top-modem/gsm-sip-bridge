/// Bridges PJSIP internal logging to the `tracing` framework.
///
/// When PJSIP is compiled in, this installs a log callback that routes all
/// PJSIP log lines to `tracing` under the `sip` target.
pub fn install_log_bridge() {
    // SAFETY: pjsua_logging_config_default + setting the callback is safe
    // as long as pjsua_create has been called first. The caller (Endpoint::create)
    // guarantees this ordering.
    #[cfg(feature = "pjsip-linked")]
    unsafe {
        use pjsua_sys::*;
        let mut log_cfg: pjsua_logging_config = std::mem::zeroed();
        pjsua_logging_config_default(&mut log_cfg);
        log_cfg.cb = Some(pjsip_log_callback);
        log_cfg.level = 4;
        log_cfg.console_level = 0;
    }

    #[cfg(not(feature = "pjsip-linked"))]
    {
        tracing::debug!(target: "sip", "PJSIP log bridge installed (stub mode)");
    }
}

#[cfg(feature = "pjsip-linked")]
unsafe extern "C" fn pjsip_log_callback(level: i32, data: *const i8, len: i32) {
    if data.is_null() || len <= 0 {
        return;
    }
    let slice = std::slice::from_raw_parts(data as *const u8, len as usize);
    let msg = String::from_utf8_lossy(slice);
    let msg = msg.trim();

    match level {
        0 | 1 => tracing::error!(target: "sip", "{}", msg),
        2 => tracing::warn!(target: "sip", "{}", msg),
        3 => tracing::info!(target: "sip", "{}", msg),
        4 => tracing::debug!(target: "sip", "{}", msg),
        _ => tracing::trace!(target: "sip", "{}", msg),
    }
}
