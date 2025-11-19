use crate::data::{Bar, Chord, NoteEvent, Tonality};
use rand::Rng;
use rand::prelude::IndexedRandom;
use std::collections::HashMap;

pub struct MusicGenerator {
    previous_tonality: Tonality,
    previous_chord: Chord,
    tonality_transition_model: HashMap<Tonality, Vec<(Tonality, u32)>>,
    chord_transition_model: HashMap<Chord, Vec<(Chord, u32)>>,
}

impl MusicGenerator {
    pub fn new() -> Self {
        use Chord::*;
        use Tonality::*;
        let mut chord_model = HashMap::new();

        chord_model.insert(
            First,
            vec![
                (Fourth, 5), // I -> IV (F) に行きやすい
                (Fifth, 3),  // I -> V (G) もあり
                (Sixth, 2),  // I -> VI (Am) もあり
                (First, 1),  // I のまま留まる
            ],
        );

        chord_model.insert(
            Fourth,
            vec![
                (Fifth, 8), // IV -> V (G) に行きやすい
                (First, 2), // IV -> I (C) に戻る
            ],
        );

        chord_model.insert(
            Fifth,
            vec![
                (First, 10), // V -> I (C) に戻るのが基本
            ],
        );

        chord_model.insert(Sixth, vec![(Fourth, 5), (First, 5)]); // Am -> F or C
        chord_model.insert(Second, vec![(Fifth, 10)]); // Dm -> G

        let mut tonality_model = HashMap::new();
        tonality_model.insert(
            CM,
            vec![
                (CM, 80), // 留まる確率を高く
                (GM, 10), // ドミナント調 (G)
                (FM, 5),  // サブドミナント調 (F)
            ],
        );

        tonality_model.insert(GM, vec![(CM, 90), (GM, 10)]);

        tonality_model.insert(
            FM,
            vec![
                (CM, 70),
                (GM, 20), // V of V を経由するような遷移
                (FM, 10),
            ],
        );

        MusicGenerator {
            previous_tonality: CM,
            previous_chord: First,
            tonality_transition_model: tonality_model,
            chord_transition_model: chord_model,
        }
    }

    fn generate_events_for_chord(&self, tonality: Tonality, chord: Chord) -> Bar {
        let root_midi_note = Self::get_root_midi_note(tonality, chord);
        let (third_interval, fifth_interval) = match chord {
            Chord::First | Chord::Fourth | Chord::Fifth => (4, 7),
            Chord::Second | Chord::Third | Chord::Sixth => (3, 7),
            Chord::Seventh => (3, 6),
        };

        let c_major_scale: [u8; 7] = [60, 62, 64, 65, 67, 69, 71];

        let chord_midi_tones: [u8; 4] = [
            root_midi_note,
            root_midi_note + third_interval,
            root_midi_note + fifth_interval,
            root_midi_note + 12,
        ];

        let mut events = Vec::new();

        for i in 0..4 {
            let position_index = i as usize; // 0, 1, 2, 3

            let root_weight = MELODY_WEIGHTS.root[position_index];
            let chord_weight = MELODY_WEIGHTS.chord[position_index];
            let scale_weight = MELODY_WEIGHTS.scale[position_index];
            let other_weight = MELODY_WEIGHTS.other[position_index];

            let total_weight = root_weight + chord_weight + scale_weight + other_weight;

            let mut rng = rand::rng();

            let target_roll = rng.random_range(0..total_weight);

            let selected_note: u8;
            let mut current_sum = 0;

            if target_roll < root_weight {
                let available_roots = [root_midi_note, root_midi_note - 12];
                selected_note = *available_roots.choose(&mut rng).unwrap();
            } else {
                current_sum += root_weight;
                if target_roll < current_sum + chord_weight {
                    let available_chords = &chord_midi_tones[1..];
                    selected_note = *available_chords.choose(&mut rng).unwrap();
                } else {
                    current_sum += chord_weight;

                    let non_chord_tones: Vec<u8> = c_major_scale
                        .iter()
                        .filter(|&n| !chord_midi_tones.contains(n))
                        .cloned()
                        .collect();

                    if target_roll < current_sum + scale_weight && !non_chord_tones.is_empty() {
                        selected_note = *non_chord_tones.choose(&mut rng).unwrap();
                    } else {
                        let available_others = [root_midi_note + 1, root_midi_note - 1];
                        selected_note = *available_others.choose(&mut rng).unwrap();
                    }
                }
            }

            const NOTE_DURATION: u16 = 250;
            const NOTE_INTERVAL: u16 = 500;
            let time_ms = (i as u16) * NOTE_INTERVAL;

            events.push((
                time_ms,
                NoteEvent::NoteOn {
                    note: selected_note,
                    velocity: 80,
                },
            ));
            events.push((
                time_ms + NOTE_DURATION,
                NoteEvent::NoteOff {
                    note: selected_note,
                },
            ));
        }

        Bar {
            beat: 4,
            tonality: Tonality::CM,
            chord,
            events,
        }
    }

    pub fn generate_next_bar(&mut self) -> Bar {
        let next_tonality = self.choose_next_tonality();

        let next_chord = self.choose_next_chord();

        let next_bar = self.generate_events_for_chord(next_tonality, next_chord);

        self.previous_tonality = next_tonality;
        self.previous_chord = next_chord; // (必要に応じて)

        next_bar
    }

    fn choose_next_tonality(&self) -> Tonality {
        let options = self
            .tonality_transition_model
            .get(&self.previous_tonality)
            .unwrap_or_else(|| {
                panic!(
                    "遷移モデルに調性 {:?} が定義されていません",
                    self.previous_tonality
                )
            });

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

    fn choose_next_chord(&self) -> Chord {
        let options = self
            .chord_transition_model
            .get(&self.previous_chord)
            .unwrap_or_else(|| {
                panic!(
                    "遷移モデルにコード {:?} が定義されていません",
                    self.previous_chord
                )
            });

        // 重み付きランダム選択ロジック（前回と同じ）
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

    fn get_root_midi_note(tonality: Tonality, chord: Chord) -> u8 {
        let root_of_tonality = match tonality {
            Tonality::CM => 60,
            Tonality::GM => 67,
            Tonality::DM => 62,
            Tonality::AM => 69,
            Tonality::EM => 64,
            Tonality::BM => 71,
            Tonality::GFM => 66,
            Tonality::DFM => 61,
            Tonality::AFM => 68,
            Tonality::EFM => 63,
            Tonality::BFM => 70,
            Tonality::FM => 65,
        };

        let interval_from_tonality_root = match chord {
            Chord::First => 0,    // I (C)
            Chord::Second => 2,   // ii (D)
            Chord::Third => 4,    // iii (E)
            Chord::Fourth => 5,   // IV (F)
            Chord::Fifth => 7,    // V (G)
            Chord::Sixth => 9,    // vi (A)
            Chord::Seventh => 11, // vii° (B)
        };

        root_of_tonality + interval_from_tonality_root
    }
}
struct PositionWeights {
    root: [u8; 4],
    chord: [u8; 4],
    scale: [u8; 4], // ノン・コードトーン (スケール内)
    other: [u8; 4], // スケール外の音
}

const MELODY_WEIGHTS: PositionWeights = PositionWeights {
    root: [15, 20, 20, 65],  // ルート音
    chord: [45, 50, 50, 20], // 3度、5度
    scale: [25, 20, 20, 10], // スケール内の非コード音
    other: [15, 10, 10, 5],  // スケール外の音
};
