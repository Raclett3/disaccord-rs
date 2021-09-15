pub struct Synth {
    sampling_rate: f32,
    phase: f32,
}

impl Synth {
    pub fn new(sampling_rate: f32) -> Self {
        Synth {
            sampling_rate,
            phase: 0.0,
        }
    }

    pub fn sample(&mut self) -> f32 {
        self.phase += 1.0 / self.sampling_rate * 440.0;
        self.phase %= 1.0;
        f32::sin(self.phase * 2.0 * std::f32::consts::PI)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_synth() {
        use super::Synth;
        use player::play;

        play(|sampling_rate| {
            let mut synth = Synth::new(sampling_rate as f32);
            (0..(sampling_rate as usize) * 2)
                .map(move |_| synth.sample())
                .collect()
        });
    }
}
