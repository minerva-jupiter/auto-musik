use crate::data::{Bar, Beat, Chord, NoteEvent, Tonality};
use rand::Rng;
use std::collections::HashMap;

pub struct MusicGenerator {
    previous_state: Chord,
    transition_model: HashMap<Chord, Vec<(Chord, u32)>>,
}

impl MusicGenerator {
    pub fn new() -> Self {
        use Chord::*;
        let mut model = HashMap::new();

        model.insert(
            First,
            vec![
                (Fourth, 5), // I -> IV (F) に行きやすい
                (Fifth, 3),  // I -> V (G) もあり
                (Sixth, 2),  // I -> VI (Am) もあり
                (First, 1),  // I のまま留まる
            ],
        );

        model.insert(
            Fourth,
            vec![
                (Fifth, 8), // IV -> V (G) に行きやすい
                (First, 2), // IV -> I (C) に戻る
            ],
        );

        model.insert(
            Fifth,
            vec![
                (First, 10), // V -> I (C) に戻るのが基本
            ],
        );

        model.insert(Sixth, vec![(Fourth, 5), (First, 5)]); // Am -> F or C
        model.insert(Second, vec![(Fifth, 10)]); // Dm -> G

        MusicGenerator {
            previous_state: First, // 初期状態は I (C)
            transition_model: model,
        }
    }

    pub fn generate_next_bar(&mut self) -> Bar {
        let next_chord = self.choose_next_state();

        let next_bar = self.generate_events_for_chord(next_chord);

        self.previous_state = next_chord;
        next_bar
    }

    fn choose_next_state(&self) -> Chord {
        let options = self
            .transition_model
            .get(&self.previous_state)
            .unwrap_or_else(|| panic!("there no code {:?} on this model", self.previous_state));

        let total_weight: u32 = options.iter().map(|(_, w)| w).sum();
        let mut rng = rand::rng();
        let mut target = rng.random_range(0..total_weight);

        for (state, weight) in options {
            if target < *weight {
                return *state;
            }
            target -= weight;
        }
        options[0].0
    }

    fn generate_events_for_chord(&self, chord: Chord) -> Bar {
        let root_note = match chord {
            Chord::First => 60,   // C4
            Chord::Second => 62,  // D4
            Chord::Third => 64,   // E4
            Chord::Fourth => 65,  // F4
            Chord::Fifth => 67,   // G4
            Chord::Sixth => 69,   // A4
            Chord::Seventh => 71, // B4
        };

        let mut events = Vec::new();
        const NOTE_DURATION: u64 = 480; // 500ms - 20ms の隙間

        for i in 0..4 {
            let time_ms = (i as u64) * 500;

            events.push((
                time_ms,
                NoteEvent::NoteOn {
                    note: root_note,
                    velocity: 90,
                },
            ));
            events.push((
                time_ms + NOTE_DURATION,
                NoteEvent::NoteOff { note: root_note },
            ));
        }

        Bar {
            beat: Beat::FourFourth,
            tonality: Tonality::CM,
            chord,
            events,
        }
    }
}
