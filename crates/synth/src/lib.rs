pub mod waveform;

use std::collections::BTreeMap;
use waveform::Waveform;

fn note_to_freq(note: i8) -> f32 {
    440.0 * 2f32.powf((note - 69) as f32 / 12.0)
}

pub struct Synth {
    notes_ringing: BTreeMap<i8, f32>,
    sampling_rate: f32,
    waveforms: Vec<Box<dyn Waveform>>,
}

impl Synth {
    pub fn new(sampling_rate: f32) -> Self {
        Synth {
            notes_ringing: BTreeMap::new(),
            sampling_rate,
            waveforms: Vec::new(),
        }
    }

    pub fn sample(&mut self) -> f32 {
        let sample = self
            .notes_ringing
            .iter()
            .flat_map(|(&note, &phase)| {
                let freq = note_to_freq(note);
                self.waveforms
                    .iter()
                    .map(move |wave| wave.sample(phase, freq))
            })
            .sum();
        for (&note, phase) in &mut self.notes_ringing {
            let freq = note_to_freq(note);
            *phase += 1.0 / self.sampling_rate * freq;
        }
        sample
    }

    pub fn note_on(&mut self, note: i8) {
        self.notes_ringing.insert(note, 0.0);
    }

    pub fn note_off(&mut self, note: i8) {
        self.notes_ringing.remove(&note);
    }

    pub fn add_waveform<T: Waveform + 'static>(&mut self, wave: T) {
        self.waveforms.push(Box::new(wave));
    }
}

#[cfg(test)]
mod tests {
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
                Event::NoteOn(note) => {
                    synth.note_on(*note);
                }
                Event::NoteOff(note) => {
                    synth.note_off(*note);
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
        use player::play;

        play(|sampling_rate| {
            let mut synth = Synth::new(sampling_rate);
            synth.add_waveform(fm_wave(2.0, 3.0));

            let mut events_queue = VecDeque::from(vec![
                (0.0, Event::NoteOn(A_4)),
                (0.2, Event::NoteOn(A_4 + 4)),
                (0.4, Event::NoteOn(A_4 + 7)),
                (1.6, Event::NoteOff(A_4)),
                (1.7, Event::NoteOff(A_4 + 4)),
                (1.8, Event::NoteOff(A_4 + 7)),
            ]);

            (0..(sampling_rate as usize) * 2)
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
    fn test_note_to_freq() {
        use super::note_to_freq;

        approx::assert_relative_eq!(note_to_freq(A_4), 440.0);
        approx::assert_relative_eq!(note_to_freq(A_4 + 7), 659.255113); // E5
        approx::assert_relative_eq!(note_to_freq(A_4 + 12), 880.0); // A5
    }
}
