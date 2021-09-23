use synth::Synth;

const A_4: i8 = 69;

#[derive(Debug, Clone)]
enum EventType {
    NoteOn(i8),
    NoteOff(i8),
}

impl EventType {
    fn at(self, position: f32) -> Event {
        Event {
            position,
            event_type: self,
        }
    }

    fn apply_to_synth(&self, synth: &mut Synth) {
        match self {
            EventType::NoteOn(key) => {
                synth.note_on(*key);
            }
            EventType::NoteOff(key) => {
                synth.note_off(*key);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Event {
    position: f32,
    event_type: EventType,
}

impl Event {
    pub fn apply_to_synth(&self, synth: &mut Synth) {
        self.event_type.apply_to_synth(synth);
    }

    pub fn is_fired_at(&self, position: f32) -> bool {
        position >= self.position
    }
}

#[derive(Debug, Clone)]
pub enum MMLError {
    UnknownToken(char),
}

fn note_name_to_relative_key(name: char) -> i8 {
    match name.to_ascii_lowercase() {
        'c' => -9,
        'd' => -7,
        'e' => -5,
        'f' => -4,
        'g' => -2,
        'a' => 0,
        'b' => 2,
        _ => unreachable!(),
    }
}

pub fn mml_to_sorted_events(mml: &str) -> Result<Vec<Event>, MMLError> {
    let tempo = 120.0;
    let mut elapsed = 0.0;
    let mut events = Vec::new();

    for ch in mml.chars() {
        let length = 240.0 / tempo / 8.0;

        match ch {
            ch @ 'a'..='g' => {
                let key = note_name_to_relative_key(ch) + A_4;
                events.push(EventType::NoteOn(key).at(elapsed));
                events.push(EventType::NoteOff(key).at(elapsed + length));
                elapsed += length;
            }
            'r' => {
                elapsed += length;
            }
            ';' => {
                elapsed = 0.0;
            }
            ch => return Err(MMLError::UnknownToken(ch)),
        }
    }

    events.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap());

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::mml_to_sorted_events;
    use player::play;
    use std::collections::VecDeque;
    use synth::oscillator::Oscillator;
    use synth::Synth;

    fn sinusoid(phase: f32, _: f32) -> f32 {
        f32::sin(phase * std::f32::consts::PI * 2.0) * 0.5
    }

    #[test]
    fn test_mml() {
        let mml = "cdefgab;rrrrcde";

        play(|sampling_rate| {
            let mut synth = Synth::new(sampling_rate);
            synth.push_oscillator(Oscillator::new(sinusoid));

            let mut events = VecDeque::from(mml_to_sorted_events(mml).unwrap());

            (0..(sampling_rate as usize) * 3)
                .map(move |sample| {
                    let pos = sample as f32 / sampling_rate;
                    while let Some(event) = events.front().cloned() {
                        if !event.is_fired_at(pos) {
                            break;
                        }
                        event.apply_to_synth(&mut synth);
                        events.pop_front();
                    }
                    synth.sample()
                })
                .collect()
        });
    }
}
