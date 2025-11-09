mod data;
mod generator;
mod midi_io;

use data::{Bar, NoteEvent};
use generator::MusicGenerator;
use midi_io::MidiTransmitter;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target_port_name = "Microsoft GS Wavetable Synth";
    let mut transmitter = MidiTransmitter::new(target_port_name)?;

    const CHANNEL_CAPACITY: usize = 10;
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Bar>(CHANNEL_CAPACITY);

    tokio::spawn(async move {
        let mut generator = MusicGenerator::new();
        loop {
            let next_bar: Bar = generator.generate_next_bar();

            if tx.send(next_bar).await.is_err() {
                eprintln!("Generator: Receiver dropped. Exiting generator task.");
                break;
            }
        }
    });

    const QUARTER_NOTE_DURATION_MS: u16 = 500;
    let mut current_bar_start_time = std::time::Instant::now();

    let mut current_bar = rx.recv().await.expect("Failed to receive first bar.");

    loop {
        let next_bar_future = rx.recv();

        println!("Playing Chord: {:?}", current_bar.chord);
        println!("currnt_bar is {:?}", current_bar);

        let bar_duration_ms: u16 = current_bar.beat * QUARTER_NOTE_DURATION_MS;

        for (event_time_offset, event) in current_bar.events.iter() {
            let elapsed_in_bar_ms: u32 = current_bar_start_time.elapsed().as_millis() as u32;
            let target_relative_time_ms: u32 = (*event_time_offset).into();

            if target_relative_time_ms > elapsed_in_bar_ms {
                let wait_time_ms = target_relative_time_ms - elapsed_in_bar_ms;

                sleep(Duration::from_millis(wait_time_ms.into())).await;
            }

            if let Some(message) = note_event_to_midi_message(*event)
                && let Err(e) = transmitter.send_message(&message)
            {
                eprintln!("Midi sending error: {}", e);
            }
        }

        // wait until the last for bar
        let elapsed_in_bar_ms: u32 = current_bar_start_time.elapsed().as_millis() as u32;
        let bar_duration_u32: u32 = bar_duration_ms.into();

        if bar_duration_u32 > elapsed_in_bar_ms {
            let wait_time_ms = bar_duration_u32 - elapsed_in_bar_ms;
            sleep(Duration::from_millis(wait_time_ms.into())).await;
        }

        current_bar_start_time = std::time::Instant::now();

        current_bar = next_bar_future
            .await
            .expect("Generator task closed unexpectedly.");
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
