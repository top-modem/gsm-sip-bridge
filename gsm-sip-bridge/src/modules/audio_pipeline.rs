use crossbeam_queue::ArrayQueue;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const FRAME_SIZE: usize = 160; // 20ms at 8kHz mono
const RING_CAPACITY: usize = 50; // ~1 second of buffering

pub type AudioFrame = [i16; FRAME_SIZE];

pub struct AudioPipeline {
    pub capture_ring: Arc<ArrayQueue<AudioFrame>>,
    pub playback_ring: Arc<ArrayQueue<AudioFrame>>,
    running: Arc<AtomicBool>,
}

impl AudioPipeline {
    pub fn new() -> Self {
        Self {
            capture_ring: Arc::new(ArrayQueue::new(RING_CAPACITY)),
            playback_ring: Arc::new(ArrayQueue::new(RING_CAPACITY)),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&self, _audio_device: &str) -> Result<(), String> {
        self.running.store(true, Ordering::SeqCst);
        tracing::info!("audio pipeline started (ALSA threads not yet spawned)");
        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn push_capture_frame(&self, frame: AudioFrame) -> bool {
        self.capture_ring.push(frame).is_ok()
    }

    pub fn pop_capture_frame(&self) -> Option<AudioFrame> {
        self.capture_ring.pop()
    }

    pub fn push_playback_frame(&self, frame: AudioFrame) -> bool {
        self.playback_ring.push(frame).is_ok()
    }

    pub fn pop_playback_frame(&self) -> Option<AudioFrame> {
        self.playback_ring.pop()
    }
}

impl Default for AudioPipeline {
    fn default() -> Self {
        Self::new()
    }
}
