use std::sync::OnceLock;
use std::time::Duration;

use rodio::{Source, Sink};

static SINK: OnceLock<Sink> = OnceLock::new();

/// Runs the M thread.
/// 
/// If an error occurs, the thread just stops silently.
pub fn init() {
    let Ok((stream, handle)) = rodio::OutputStream::try_default() else { return };
    std::mem::forget(stream);
    let Ok(sink) = Sink::try_new(&handle) else { return };
    sink.set_volume(0.8);
    SINK.set(sink).ok().unwrap();
}

struct Note {
    duration: f64,
    frequency: f64,
}

impl Note {
    pub fn from_hash(hash: u64) -> Self {
        const NOTE_COUNT: u64 = 36;
        const BASE_FREQ: f64 = 100.0;
        
        let frequency = BASE_FREQ * 2f64.powf((hash % NOTE_COUNT) as f64 / 12.0);

        Self {
            duration: frequency / 500.0,
            frequency,
        }
    }
}

/// Play a note with the provided hash.
pub fn play_note(hash: u64) {
    let Some(sink) = SINK.get() else { return };
    
    let note = Note::from_hash(hash);
    let source = rodio::source::SineWave::new(note.frequency as f32);

    sink.clear();
    sink.play();
    sink.append(source);
}
