use crate::modules::audio_pipeline::AudioPipeline;
use pjsua_safe::MediaPortHandle;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardState {
    Idle,
    Ringing,
    Answering,
    Bridged,
    Cleanup,
}

pub struct CardInstance {
    pub id: String,
    pub serial_port: std::path::PathBuf,
    pub audio_device: String,
    pub state: CardState,
    pub pipeline: Arc<AudioPipeline>,
    pub port_handle: Option<MediaPortHandle>,
}

impl CardInstance {
    pub fn new(
        id: String,
        serial_port: std::path::PathBuf,
        audio_device: String,
        ring_capacity: usize,
    ) -> Self {
        Self {
            id,
            serial_port,
            audio_device,
            state: CardState::Idle,
            pipeline: Arc::new(AudioPipeline::with_capacity(ring_capacity)),
            port_handle: None,
        }
    }

    pub fn register_media_port(&mut self) -> Result<i32, String> {
        let pipeline = self.pipeline.clone();
        struct AlsaMediaPort {
            pipeline: Arc<AudioPipeline>,
        }
        impl pjsua_safe::AudioMediaPort for AlsaMediaPort {
            fn read_frame(&mut self, buf: &mut [i16]) {
                let frame = self.pipeline.pop_capture_frame().unwrap_or([0i16; 160]);
                let n = buf.len().min(160);
                buf[..n].copy_from_slice(&frame[..n]);
            }
            fn write_frame(&mut self, buf: &[i16]) {
                let mut frame = [0i16; 160];
                let n = buf.len().min(160);
                frame[..n].copy_from_slice(&buf[..n]);
                let _ = self.pipeline.push_playback_frame(frame);
            }
        }

        let port = AlsaMediaPort { pipeline };
        let handle = MediaPortHandle::register_to_conf_bridge(Box::new(port))
            .map_err(|e| format!("failed to register GSM media port: {e}"))?;
        let slot = handle.slot_id();
        tracing::info!(slot = slot, device = %self.audio_device, "GSM media port registered");
        self.port_handle = Some(handle);
        Ok(slot)
    }
}
