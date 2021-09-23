use crate::waveform::Waveform;

pub struct Oscillator {
    waveform: Box<dyn Waveform>,
    voices: usize,
    detune: f32,
}

impl Oscillator {
    pub fn new<T: Waveform + 'static>(waveform: T) -> Self {
        Oscillator::from_boxed(Box::new(waveform))
    }

    pub fn from_boxed(waveform: Box<dyn Waveform>) -> Self {
        Oscillator {
            waveform,
            voices: 1,
            detune: 1.0,
        }
    }

    pub fn voices(mut self, voices: usize) -> Self {
        self.voices = voices;
        self
    }

    pub fn detune(mut self, detune: f32) -> Self {
        self.detune = detune;
        self
    }
}

impl Waveform for Oscillator {
    fn sample(&self, phase: f32, freq: f32) -> f32 {
        (0..self.voices)
            .map(|x| {
                let multiplier = self.detune.powi(x as i32);
                self.waveform.sample(phase * multiplier, freq * multiplier)
            })
            .sum()
    }
}
