#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoteEvent {
    // Note On:
    NoteOn { note: u8, velocity: u8 },
    // Note Off:
    NoteOff { note: u8 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bar {
    pub beat: u16,
    pub tonality: Tonality,
    pub chord: Chord,
    pub events: Vec<(u16, NoteEvent)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tonality {
    CM,
    GM,
    DM,
    AM,
    EM,
    BM,
    GFM,
    DFM,
    AFM,
    EFM,
    BFM,
    FM,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Chord {
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
    Seventh,
}
