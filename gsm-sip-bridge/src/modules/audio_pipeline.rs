use alsa::pcm::{Access, Format, HwParams, PCM};
use alsa::{Direction, ValueOr};
use crossbeam_queue::ArrayQueue;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

pub const FRAME_SIZE: usize = 160; // 20ms at 8kHz mono
const TEST_RING_CAPACITY: usize = 50;
const SAMPLE_RATE: u32 = 8000;
const CHANNELS: u32 = 1;
const PERIOD_FRAMES: u32 = 160;

pub type AudioFrame = [i16; FRAME_SIZE];

pub struct AudioPipeline {
    pub capture_ring: Arc<ArrayQueue<AudioFrame>>,
    pub playback_ring: Arc<ArrayQueue<AudioFrame>>,
    running: Arc<AtomicBool>,
    threads: Vec<thread::JoinHandle<()>>,
}

impl AudioPipeline {
    pub fn with_capacity(ring_capacity: usize) -> Self {
        Self {
            capture_ring: Arc::new(ArrayQueue::new(ring_capacity)),
            playback_ring: Arc::new(ArrayQueue::new(ring_capacity)),
            running: Arc::new(AtomicBool::new(false)),
            threads: Vec::new(),
        }
    }

    pub fn new() -> Self {
        Self::with_capacity(TEST_RING_CAPACITY)
    }

    pub fn start(&mut self, audio_device: &str) -> Result<(), String> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }
        self.running.store(true, Ordering::SeqCst);

        let dev = audio_device.to_string();
        let thread_name_prefix = dev.replace(':', "_");
        let running = self.running.clone();
        let capture_ring = self.capture_ring.clone();
        let playback_ring = self.playback_ring.clone();

        let cap_dev = dev.clone();
        let cap_running = running.clone();
        let cap_thread = thread::Builder::new()
            .name(format!("alsa-cap-{}", thread_name_prefix))
            .spawn(move || {
                run_capture(&cap_dev, &cap_running, &capture_ring);
            })
            .map_err(|e| format!("failed to spawn capture thread: {e}"))?;

        let play_running = running;
        let play_thread = thread::Builder::new()
            .name(format!("alsa-play-{}", thread_name_prefix))
            .spawn(move || {
                run_playback(&dev, &play_running, &playback_ring);
            })
            .map_err(|e| format!("failed to spawn playback thread: {e}"))?;

        self.threads.push(cap_thread);
        self.threads.push(play_thread);

        tracing::info!(device = %audio_device, "audio pipeline ALSA threads started");
        Ok(())
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        let threads = std::mem::take(&mut self.threads);
        for t in threads {
            let _ = t.join();
        }
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

fn configure_pcm(pcm: &PCM, label: &str) -> Result<(), String> {
    let hwp = HwParams::any(pcm).map_err(|e| format!("{label}: HwParams::any: {e}"))?;
    hwp.set_access(Access::RWInterleaved)
        .map_err(|e| format!("{label}: set_access: {e}"))?;
    hwp.set_format(Format::s16())
        .map_err(|e| format!("{label}: set_format: {e}"))?;
    hwp.set_channels(CHANNELS)
        .map_err(|e| format!("{label}: set_channels: {e}"))?;
    hwp.set_rate(SAMPLE_RATE, ValueOr::Nearest)
        .map_err(|e| format!("{label}: set_rate: {e}"))?;
    hwp.set_period_size(PERIOD_FRAMES as alsa::pcm::Frames, ValueOr::Nearest)
        .map_err(|e| format!("{label}: set_period_size: {e}"))?;
    hwp.set_buffer_size((PERIOD_FRAMES * 4) as alsa::pcm::Frames)
        .map_err(|e| format!("{label}: set_buffer_size: {e}"))?;
    pcm.hw_params(&hwp)
        .map_err(|e| format!("{label}: hw_params apply: {e}"))?;
    Ok(())
}

fn run_capture(device: &str, running: &AtomicBool, ring: &ArrayQueue<AudioFrame>) {
    let pcm = match PCM::new(device, Direction::Capture, false) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(device, error = %e, "capture: failed to open PCM");
            return;
        }
    };
    if let Err(e) = configure_pcm(&pcm, "capture") {
        tracing::error!(device, error = %e, "capture: configure failed");
        return;
    }
    let io = match pcm.io_i16() {
        Ok(i) => i,
        Err(e) => {
            tracing::error!(device, error = %e, "capture: io_i16 failed");
            return;
        }
    };

    let mut buf = [0i16; FRAME_SIZE];
    while running.load(Ordering::SeqCst) {
        match io.readi(&mut buf) {
            Ok(_) => {
                let _ = ring.push(buf);
            }
            Err(e) if e.errno() == libc::EPIPE => {
                tracing::warn!(device, "capture overrun, recovering");
                let _ = pcm.prepare();
            }
            Err(e) if e.errno() == libc::EINTR => continue,
            Err(e) => {
                tracing::error!(device, error = %e, "capture ALSA read error");
                break;
            }
        }
    }
}

fn run_playback(device: &str, running: &AtomicBool, ring: &ArrayQueue<AudioFrame>) {
    let pcm = match PCM::new(device, Direction::Playback, false) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(device, error = %e, "playback: failed to open PCM");
            return;
        }
    };
    if let Err(e) = configure_pcm(&pcm, "playback") {
        tracing::error!(device, error = %e, "playback: configure failed");
        return;
    }
    let io = match pcm.io_i16() {
        Ok(i) => i,
        Err(e) => {
            tracing::error!(device, error = %e, "playback: io_i16 failed");
            return;
        }
    };

    // Pre-fill with silence to prevent initial underrun
    let silence = [0i16; FRAME_SIZE];
    for _ in 0..3 {
        let _ = io.writei(&silence);
    }

    while running.load(Ordering::SeqCst) {
        let frame = ring.pop().unwrap_or(silence);
        match io.writei(&frame) {
            Ok(_) => {}
            Err(e) if e.errno() == libc::EPIPE => {
                tracing::warn!(device, "playback underrun, recovering");
                let _ = pcm.prepare();
                for _ in 0..2 {
                    let _ = io.writei(&silence);
                }
                let _ = io.writei(&frame);
            }
            Err(e) if e.errno() == libc::EINTR => {}
            Err(e) => {
                tracing::error!(device, error = %e, "playback ALSA write error");
                break;
            }
        }
    }
}
