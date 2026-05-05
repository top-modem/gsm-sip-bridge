use std::f32::consts::PI;

const SAMPLE_RATE: u32 = 8000;
const FREQUENCY: f32 = 400.0;
const AMPLITUDE: f32 = 16000.0;

pub struct BeepGenerator {
    phase: f32,
    active: bool,
}

impl BeepGenerator {
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            active: false,
        }
    }

    pub fn start(&mut self) {
        self.active = true;
        self.phase = 0.0;
    }

    pub fn stop(&mut self) {
        self.active = false;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn fill_buffer(&mut self, buf: &mut [i16]) {
        if !self.active {
            buf.fill(0);
            return;
        }
        let phase_increment = 2.0 * PI * FREQUENCY / SAMPLE_RATE as f32;
        for sample in buf.iter_mut() {
            *sample = (self.phase.sin() * AMPLITUDE) as i16;
            self.phase += phase_increment;
            if self.phase >= 2.0 * PI {
                self.phase -= 2.0 * PI;
            }
        }
    }
}

impl Default for BeepGenerator {
    fn default() -> Self {
        Self::new()
    }
}
