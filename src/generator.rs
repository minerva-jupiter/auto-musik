use crate::data::{Bar, Chord, NoteEvent, Tonality};
use rand::Rng;
use rand::prelude::IndexedRandom;
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
        let root_midi_note = match chord {
            Chord::First => 60,   // C (I)
            Chord::Second => 62,  // D (ii)
            Chord::Third => 64,   // E (iii)
            Chord::Fourth => 65,  // F (IV)
            Chord::Fifth => 67,   // G (V)
            Chord::Sixth => 69,   // A (vi)
            Chord::Seventh => 71, // B (vii°) - (今回は単純化のためルート音のみ)
        };

        // Cメジャースケール内での三和音（トニックからのインターバル）
        let (third_interval, fifth_interval) = match chord {
            // メジャーコード: I (C), IV (F), V (G) -> 4半音 (長3度), 7半音 (完全5度)
            Chord::First | Chord::Fourth | Chord::Fifth => (4, 7),
            // マイナーコード: ii (Dm), iii (Em), vi (Am) -> 3半音 (短3度), 7半音 (完全5度)
            Chord::Second | Chord::Third | Chord::Sixth => (3, 7),
            // ディミニッシュ: vii° (Bdim) -> 3半音 (短3度), 6半音 (減5度)
            Chord::Seventh => (3, 6),
        };

        let c_major_scale: [u8; 7] = [60, 62, 64, 65, 67, 69, 71];

        // コードトーンの絶対音程 (オクターブを含めて定義)
        let chord_midi_tones: [u8; 3] = [
            root_midi_note,
            root_midi_note + third_interval,
            root_midi_note + fifth_interval,
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

            // ----------------------------------------------------
            // 3. 重みに基づいてノートカテゴリを選択
            // ----------------------------------------------------
            let selected_note: u8;
            let mut current_sum = 0;

            if target_roll < root_weight {
                // グループ A: ルート音 (ルート音は 1オクターブ下も選択肢に入れる)
                let available_roots = [root_midi_note, root_midi_note - 12];
                selected_note = *available_roots.choose(&mut rng).unwrap();
            } else {
                current_sum += root_weight;
                if target_roll < current_sum + chord_weight {
                    // グループ B: コードトーン (ルート除く 3度, 5度)
                    let available_chords = &chord_midi_tones[1..];
                    selected_note = *available_chords.choose(&mut rng).unwrap();
                } else {
                    current_sum += chord_weight;

                    // スケールトーン (ノン・コードトーン) のリストを作成
                    let non_chord_tones: Vec<u8> = c_major_scale
                        .iter()
                        .filter(|&n| !chord_midi_tones.contains(n))
                        .cloned()
                        .collect();

                    if target_roll < current_sum + scale_weight && !non_chord_tones.is_empty() {
                        // グループ C: ノン・コードトーン (スケール内)
                        selected_note = *non_chord_tones.choose(&mut rng).unwrap();
                    } else {
                        // グループ D: スケール外の音 (一時的にルート音の半音下・上を候補とする)
                        // TODO: 実際のスケール外の音を定義する必要があります
                        let available_others = [root_midi_note + 1, root_midi_note - 1];
                        selected_note = *available_others.choose(&mut rng).unwrap();
                    }
                }
            }

            // ノートイベントの生成は変更なし
            const NOTE_DURATION: u16 = 250;
            const NOTE_INTERVAL: u16 = 500;
            let time_ms = (i as u16) * NOTE_INTERVAL;

            // Note On/Off
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
}
struct PositionWeights {
    // 0, 1, 2, 3 は 4分音符内の 4つの音の位置を示す (N1, N2, N3, N4)
    // 値は重み、またはパーセンテージ
    root: [u8; 4],
    chord: [u8; 4],
    scale: [u8; 4], // ノン・コードトーン (スケール内)
    other: [u8; 4], // スケール外の音
}

// ハ長調の例を実装に使う
const MELODY_WEIGHTS: PositionWeights = PositionWeights {
    // N1, N2, N3, N4
    root: [15, 20, 20, 65],  // ルート音
    chord: [45, 50, 50, 20], // 3度、5度
    scale: [25, 20, 20, 10], // スケール内の非コード音
    other: [15, 10, 10, 5],  // スケール外の音
};
