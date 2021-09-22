#[derive(Debug, Clone, Copy)]
pub struct Envelope {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
}

fn linear_interpolation((x1, y1): (f32, f32), (x2, y2): (f32, f32), x: f32) -> Option<f32> {
    if x1 <= x && x < x2 {
        let relative_position = (x - x1) / (x2 - x1);
        Some(y1 * (1.0 - relative_position) + y2 * relative_position)
    } else {
        None
    }
}

impl Envelope {
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32) -> Envelope {
        Envelope {
            attack,
            decay,
            sustain,
            release,
        }
    }

    pub fn multiplier(&self, position: f32) -> f32 {
        let start = (0.0, 0.0);
        let summit = (self.attack, 1.0);
        let bottom = (self.attack + self.decay, self.sustain);
        linear_interpolation(start, summit, position)
            .or_else(|| linear_interpolation(summit, bottom, position))
            .unwrap_or(self.sustain)
    }

    pub fn release_multiplier(&self, position: f32) -> Option<f32> {
        linear_interpolation((0.0, self.sustain), (self.release, 0.0), position)
    }

    pub fn is_releasing(&self, position: f32) -> bool {
        position < self.release
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_interpolation() {
        approx::assert_relative_eq!(
            linear_interpolation((0.0, 1.0), (2.0, 3.0), 0.0).unwrap(),
            1.0
        );
        approx::assert_relative_eq!(
            linear_interpolation((0.0, 1.0), (2.0, 3.0), 1.0).unwrap(),
            2.0
        );
        assert_eq!(linear_interpolation((0.0, 1.0), (2.0, 3.0), 2.0), None);
        assert_eq!(linear_interpolation((0.0, 1.0), (2.0, 3.0), 2.5), None);
    }

    #[test]
    fn test_envelope() {
        let envelope = Envelope::new(1.0, 0.5, 0.5, 1.0);
        approx::assert_relative_eq!(envelope.multiplier(0.0), 0.0);
        approx::assert_relative_eq!(envelope.multiplier(0.2), 0.2);
        approx::assert_relative_eq!(envelope.multiplier(0.4), 0.4);
        approx::assert_relative_eq!(envelope.multiplier(0.6), 0.6);
        approx::assert_relative_eq!(envelope.multiplier(0.8), 0.8);
        approx::assert_relative_eq!(envelope.multiplier(1.0), 1.0);
        approx::assert_relative_eq!(envelope.multiplier(1.1), 0.9);
        approx::assert_relative_eq!(envelope.multiplier(1.2), 0.8);
        approx::assert_relative_eq!(envelope.multiplier(1.3), 0.7);
        approx::assert_relative_eq!(envelope.multiplier(1.4), 0.6);
        approx::assert_relative_eq!(envelope.multiplier(1.5), 0.5);
        approx::assert_relative_eq!(envelope.multiplier(2.0), 0.5);
        approx::assert_relative_eq!(envelope.multiplier(3.0), 0.5);
        approx::assert_relative_eq!(envelope.multiplier(200.0), 0.5);

        approx::assert_relative_eq!(envelope.release_multiplier(0.0).unwrap(), 0.5);
        approx::assert_relative_eq!(envelope.release_multiplier(0.2).unwrap(), 0.4);
        approx::assert_relative_eq!(envelope.release_multiplier(0.4).unwrap(), 0.3);
        approx::assert_relative_eq!(envelope.release_multiplier(0.6).unwrap(), 0.2);
        approx::assert_relative_eq!(envelope.release_multiplier(0.8).unwrap(), 0.1);
        assert_eq!(envelope.release_multiplier(1.0), None);
        assert_eq!(envelope.release_multiplier(1.1), None);
    }
}
