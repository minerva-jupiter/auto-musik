mod data;
mod generator;
mod midi_io;

use data::{Bar, NoteEvent};
use generator::MusicGenerator;
use midi_io::MidiTransmitter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target_port_name = "Microsoft GS Wavetable Synth";
    let mut transmitter = MidiTransmitter::new(target_port_name)?;

    let mut generator = MusicGenerator::new();

    // 1分(60,000ms) / 120 = 500ms が四分音符の長さ
    const QUARTER_NOTE_DURATION_MS: u64 = 500;

    let start_time = std::time::Instant::now();
    let mut current_time_ms: u64 = 0;

    loop {
        let next_bar: Bar = generator.generate_next_bar();
        println!("Generated Chord is: {:?}", next_bar.chord); // どのコードが選ばれたか表示

        let bar_duration_ms = 4 * QUARTER_NOTE_DURATION_MS;

        for (event_time_offset, event) in next_bar.events.iter() {
            let target_absolute_time_ms = current_time_ms + event_time_offset;

            let elapsed_time = start_time.elapsed().as_millis() as u64;

            if target_absolute_time_ms > elapsed_time {
                let wait_time_ms = target_absolute_time_ms - elapsed_time;
                std::thread::sleep(std::time::Duration::from_millis(wait_time_ms));
            }

            if let Some(message) = note_event_to_midi_message(*event)
                && let Err(e) = transmitter.send_message(&message)
            {
                eprintln!("Midi sending error: {}", e);
            }
        }

        current_time_ms += bar_duration_ms;

        let elapsed_time = start_time.elapsed().as_millis() as u64;
        if current_time_ms > elapsed_time {
            let wait_time_ms = current_time_ms - elapsed_time;
            std::thread::sleep(std::time::Duration::from_millis(wait_time_ms));
        }
    }
}

fn note_event_to_midi_message(event: NoteEvent) -> Option<[u8; 3]> {
    match event {
        NoteEvent::NoteOn { note, velocity } => Some([0x90, note, velocity]),
        NoteEvent::NoteOff { note } => Some([0x80, note, 0]),
    }
}

// ----------------------------------------------------
// 3. generator.rs の調整（data.rs に合わせて）
// ----------------------------------------------------
// generator.rs の MusicGenerator::new() で、HashMapのキーとして
// data::Chord を使うように修正します。
// 例: model.insert(Chord::First, vec![(Chord::Fourth, 7), ...]);
// また、generate_events_for_state() も Chord に対応させてください。

/* this is test for midi_io.ts
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("avalable Midi ports");
    if let Ok(midi_out) = MidiOutput::new("temp_lister") {
        for name in MidiTransmitter::get_port_names(&midi_out) {
            println!("  - {}", name);
        }
    } else {
        println!("Failed to initialize Midi output");
    }
    println!("---------------------------------");

    let target_port_name = "Synth";

    let mut transmitter = MidiTransmitter::new(target_port_name)?;

    // Note On: 0x9n (n=チャンネル), ノート番号(60=C4), ベロシティ(100)
    let note_on_c4: [u8; 3] = [0x90, 60, 100];
    // Note Off: 0x8n (n=チャンネル), ノート番号(60=C4), ベロシティ(0)
    let note_off_c4: [u8; 3] = [0x80, 60, 0];

    println!("Sending C4 vel 100 on channel");
    transmitter.send_message(&note_on_c4)?;

    std::thread::sleep(std::time::Duration::from_millis(500));

    println!("Sending C4 vel 100 off channel");
    transmitter.send_message(&note_off_c4)?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    println!("Done");
    Ok(())
}
*/
