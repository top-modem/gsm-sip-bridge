use crate::error::PjsipError;
#[cfg(feature = "pjsip-linked")]
use crate::error::PJ_SUCCESS;
use crate::log_bridge;

#[derive(Debug, Clone)]
pub struct EndpointConfig {
    pub transport: TransportType,
    pub local_port: u16,
    pub tls_verify: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportType {
    Udp,
    Tcp,
    Tls,
}

pub struct Endpoint {
    #[allow(dead_code)]
    config: EndpointConfig,
    started: bool,
}

impl Endpoint {
    pub fn create(config: EndpointConfig) -> Result<Self, PjsipError> {
        #[cfg(feature = "pjsip-linked")]
        {
            // SAFETY: pjsua_create must be called exactly once before any other pjsua API.
            // We ensure this by taking ownership in Endpoint::create which can only succeed once.
            unsafe {
                let status = pjsua_sys::pjsua_create();
                if status != PJ_SUCCESS {
                    return Err(PjsipError::InitFailed(format!(
                        "pjsua_create returned {status}"
                    )));
                }

                let mut cfg: pjsua_sys::pjsua_config = std::mem::zeroed();
                pjsua_sys::pjsua_config_default(&mut cfg);

                let mut log_cfg: pjsua_sys::pjsua_logging_config = std::mem::zeroed();
                pjsua_sys::pjsua_logging_config_default(&mut log_cfg);
                log_cfg.level = 4;
                log_cfg.console_level = 0;

                let mut media_cfg: pjsua_sys::pjsua_media_config = std::mem::zeroed();
                pjsua_sys::pjsua_media_config_default(&mut media_cfg);
                media_cfg.clock_rate = 8000;
                media_cfg.snd_clock_rate = 8000;
                media_cfg.channel_count = 1;
                media_cfg.no_vad = 1;

                let status = pjsua_sys::pjsua_init(&cfg, &log_cfg, &media_cfg);
                if status != PJ_SUCCESS {
                    pjsua_sys::pjsua_destroy();
                    return Err(PjsipError::InitFailed(format!(
                        "pjsua_init returned {status}"
                    )));
                }

                let mut tp_cfg: pjsua_sys::pjsua_transport_config = std::mem::zeroed();
                pjsua_sys::pjsua_transport_config_default(&mut tp_cfg);
                tp_cfg.port = config.local_port as u32;

                let tp_type = match config.transport {
                    TransportType::Udp => pjsua_sys::pjsip_transport_type_e_PJSIP_TRANSPORT_UDP,
                    TransportType::Tcp => pjsua_sys::pjsip_transport_type_e_PJSIP_TRANSPORT_TCP,
                    TransportType::Tls => pjsua_sys::pjsip_transport_type_e_PJSIP_TRANSPORT_TLS,
                };

                let mut tp_id: pjsua_sys::pjsua_transport_id = -1;
                let status = pjsua_sys::pjsua_transport_create(tp_type, &tp_cfg, &mut tp_id);
                if status != PJ_SUCCESS {
                    pjsua_sys::pjsua_destroy();
                    return Err(PjsipError::TransportCreate(format!(
                        "pjsua_transport_create returned {status}"
                    )));
                }

                let status = pjsua_sys::pjsua_start();
                if status != PJ_SUCCESS {
                    pjsua_sys::pjsua_destroy();
                    return Err(PjsipError::InitFailed(format!(
                        "pjsua_start returned {status}"
                    )));
                }
            }
        }

        #[cfg(not(feature = "pjsip-linked"))]
        {
            log_bridge::install_log_bridge();
            tracing::info!(
                transport = ?config.transport,
                port = config.local_port,
                "PJSIP endpoint created (stub mode - no real PJSIP linked)"
            );
        }

        Ok(Self {
            config,
            started: true,
        })
    }

    pub fn is_started(&self) -> bool {
        self.started
    }

    pub fn conf_slot_count(&self) -> u32 {
        #[cfg(feature = "pjsip-linked")]
        unsafe {
            pjsua_sys::pjsua_conf_get_active_ports() as u32
        }
        #[cfg(not(feature = "pjsip-linked"))]
        0
    }
}

impl Drop for Endpoint {
    fn drop(&mut self) {
        if self.started {
            #[cfg(feature = "pjsip-linked")]
            {
                // SAFETY: pjsua_destroy is the cleanup counterpart to pjsua_create.
                // We own the Endpoint, so this is guaranteed to run exactly once.
                unsafe {
                    pjsua_sys::pjsua_destroy();
                }
            }
            tracing::info!("PJSIP endpoint destroyed");
            self.started = false;
        }
    }
}
