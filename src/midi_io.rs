use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};

pub struct MidiTransmitter {
    conn: MidiOutputConnection,
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

impl MidiTransmitter {
    pub fn new(port_name: &str) -> Result<Self> {
        let midi_out = MidiOutput::new("Rust Midi Generator")?;

        let out_port = Self::find_port(&midi_out, port_name)?;
        let actual_port_name = midi_out.port_name(&out_port)?;

        println!("We use to connect port: {}", actual_port_name);

        let conn = midi_out.connect(&out_port, "midir-output-port")?;

        Ok(MidiTransmitter { conn })
    }

    fn find_port(midi_out: &MidiOutput, search_name: &str) -> Result<MidiOutputPort> {
        for port in midi_out.ports().iter() {
            let name = midi_out.port_name(port)?;
            if name.contains(search_name) {
                return Ok(port.clone());
            }
        }

        Err(format!("Midi port named '{}' was not found ", search_name).into())
    }

    pub fn send_message(&mut self, message: &[u8]) -> std::result::Result<(), midir::SendError> {
        self.conn.send(message)
    }

    pub fn get_port_names(midi_out: &MidiOutput) -> Vec<String> {
        midi_out
            .ports()
            .iter()
            .filter_map(|p| midi_out.port_name(p).ok())
            .collect()
    }
}
