pub mod envelope;
pub mod oscillator;
pub mod waveform;

use envelope::Envelope;
use oscillator::Oscillator;
use std::collections::BTreeMap;

fn key_to_freq(key: i8) -> f32 {
    440.0 * 2f32.powf((key - 69) as f32 / 12.0)
}

#[derive(Clone, Copy, Default)]
struct Note {
    phase: f32,
    elapsed: f32,
}

pub struct Synth {
    envelope: Option<Envelope>,
    notes_ringing: BTreeMap<i8, Note>,
    notes_releasing: Vec<(i8, Note)>,
    sampling_rate: f32,
    oscillators: Vec<Oscillator>,
}

impl Synth {
    pub fn new(sampling_rate: f32) -> Self {
        Synth {
            envelope: None,
            notes_ringing: BTreeMap::new(),
            notes_releasing: Vec::new(),
            sampling_rate,
            oscillators: Vec::new(),
        }
    }

    pub fn sample(&mut self) -> f32 {
        let sample: f32 = self
            .notes_ringing
            .iter()
            .flat_map(|(&key, &Note { phase, elapsed })| {
                let freq = key_to_freq(key);
                let multiplier = self.envelope.map(|x| x.multiplier(elapsed)).unwrap_or(1.0);
                self.oscillators
                    .iter()
                    .map(move |osc| osc.sample(phase, freq) * multiplier)
            })
            .sum();

        let sample_releasing: f32 = self
            .notes_releasing
            .iter()
            .flat_map(|&(key, Note { phase, elapsed })| {
                let freq = key_to_freq(key);
                let multiplier = self
                    .envelope
                    .and_then(|x| x.release_multiplier(elapsed))
                    .unwrap_or(0.0);
                self.oscillators
                    .iter()
                    .map(move |osc| osc.sample(phase, freq) * multiplier)
            })
            .sum();

        for (&note, Note { phase, elapsed }) in &mut self.notes_ringing {
            let freq = key_to_freq(note);
            *phase += 1.0 / self.sampling_rate * freq;
            *elapsed += 1.0 / self.sampling_rate;
        }

        for (note, Note { phase, elapsed }) in &mut self.notes_releasing {
            let freq = key_to_freq(*note);
            *phase += 1.0 / self.sampling_rate * freq;
            *elapsed += 1.0 / self.sampling_rate;
        }

        let envelope = self.envelope;

        self.notes_releasing.retain(|&(_, note)| {
            envelope
                .map(|env| env.is_releasing(note.elapsed))
                .unwrap_or(false)
        });

        sample + sample_releasing
    }

    pub fn note_on(&mut self, key: i8) {
        self.notes_ringing.insert(key, Default::default());
    }

    pub fn note_off(&mut self, key: i8) {
        if let Some(note) = self.notes_ringing.remove(&key) {
            if self.envelope.is_some() {
                self.notes_releasing.push((
                    key,
                    Note {
                        elapsed: 0.0,
                        ..note
                    },
                ));
            }
        }
    }

    pub fn set_envelope(&mut self, envelope: Option<Envelope>) {
        self.envelope = envelope;
    }

    pub fn push_oscillator(&mut self, oscillator: Oscillator) {
        self.oscillators.push(oscillator);
    }
}

#[cfg(test)]
mod tests {
    use super::oscillator::Oscillator;
    use super::Synth;
    use std::collections::VecDeque;
    use std::f32::consts::PI;

    const A_4: i8 = 69;

    #[derive(Copy, Clone)]
    enum Event {
        NoteOn(i8),
        NoteOff(i8),
    }

    impl Event {
        fn apply_to_synth(&self, synth: &mut Synth) {
            match self {
                Event::NoteOn(key) => {
                    synth.note_on(*key);
                }
                Event::NoteOff(key) => {
                    synth.note_off(*key);
                }
            }
        }
    }

    fn fm_wave(mod_amp: f32, mod_freq_ratio: f32) -> impl Fn(f32, f32) -> f32 {
        fn sinusoid(phase: f32) -> f32 {
            f32::sin(2.0 * PI * phase)
        }

        move |phase, _| sinusoid(phase + mod_amp * sinusoid(phase * mod_freq_ratio))
    }

    #[test]
    fn test_synth() {
        use super::envelope::Envelope;
        use player::play;

        play(|sampling_rate| {
            let mut synth = Synth::new(sampling_rate);
            synth.push_oscillator(Oscillator::new(fm_wave(2.0, 3.0)).voices(3).detune(1.01));
            synth.set_envelope(Some(Envelope::new(0.0, 0.1, 0.2, 0.5)));

            let mut events_queue = VecDeque::from(vec![
                (0.0, Event::NoteOn(A_4)),
                (0.2, Event::NoteOn(A_4 + 4)),
                (0.4, Event::NoteOn(A_4 + 7)),
                (1.6, Event::NoteOff(A_4)),
                (1.7, Event::NoteOff(A_4 + 4)),
                (1.8, Event::NoteOff(A_4 + 7)),
            ]);

            (0..(sampling_rate as usize) * 3)
                .map(move |sample| {
                    let pos = sample as f32 / sampling_rate;
                    while let Some(&(event_at, event)) = events_queue.front() {
                        if pos >= event_at {
                            event.apply_to_synth(&mut synth);
                            events_queue.pop_front();
                        } else {
                            break;
                        }
                    }
                    synth.sample()
                })
                .collect()
        });
    }

    #[test]
    fn test_key_to_freq() {
        use super::key_to_freq;

        approx::assert_relative_eq!(key_to_freq(A_4), 440.0);
        approx::assert_relative_eq!(key_to_freq(A_4 + 7), 659.255113); // E5
        approx::assert_relative_eq!(key_to_freq(A_4 + 12), 880.0); // A5
    }
}
