pub trait Waveform {
    fn sample(&self, phase: f32, frequency: f32) -> f32;
}

impl<F> Waveform for F
where
    F: Fn(f32, f32) -> f32,
{
    fn sample(&self, phase: f32, frequency: f32) -> f32 {
        self(phase, frequency)
    }
}
