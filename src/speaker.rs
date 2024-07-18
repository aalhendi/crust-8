pub struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl SquareWave {
    pub fn new(phase_inc: f32, phase:f32, volume: f32) -> Self {
        Self {
            phase_inc,
            phase,
            volume,
        }
    }
}

impl sdl2::audio::AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}