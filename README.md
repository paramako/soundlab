# soundlab

[![Crates.io](https://img.shields.io/crates/v/soundlab)](https://crates.io/crates/soundlab)
[![Docs.rs](https://docs.rs/soundlab/badge.svg)](https://docs.rs/soundlab)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Changelog](https://img.shields.io/badge/changelog-md-blue)](CHANGELOG.md)

A Rust library for building synthesizers: envelopes, voices, and polyphony.

## Components

| Component | Description |
|-----------|-------------|
| `Envelope` | Trait for envelope generators. `LinearAdsr` included, more to come. |
| `Voice` | Combines an oscillator and envelope into a playable voice with MIDI support. |
| `Polyphony` | Manages multiple voices for chords with configurable voice stealing. |

## Usage

### Envelope
```rust
use soundlab::envelope::{Envelope, LinearAdsr};

let mut env = LinearAdsr::new(44100.0, 0.01, 0.1, 0.7, 0.3);

env.gate_on();  // Note pressed

for _ in 0..44100 {
    let amplitude = env.next_sample();
    // Multiply your oscillator output by amplitude
}

env.gate_off();  // Note released
```

Presets for common sounds:
```rust
let pad = LinearAdsr::pad(44100.0);         // Slow attack, long release
let pluck = LinearAdsr::pluck(44100.0);     // Instant attack, no sustain
let perc = LinearAdsr::percussion(44100.0); // Instant attack, fast decay
```

### Voice

A voice combines an oscillator with an amplitude envelope:
```rust
use soundlab::voice::Voice;
use soundlab::envelope::LinearAdsr;
use oscy::{poly_blep::PolyBlepOsc, Waveform};

let osc = PolyBlepOsc::new(44100.0, 440.0, Waveform::Saw);
let env = LinearAdsr::pad(44100.0);
let mut voice = Voice::new(osc, env);

// MIDI note on (C4, velocity 0.8)
voice.note_on(60, 0.8);

for _ in 0..44100 {
    let sample = voice.next_sample();
}

voice.note_off();
```

### Polyphony

Manage multiple voices for playing chords:
```rust
use soundlab::polyphony::{Polyphony, VoiceStealStrategy};
use soundlab::voice::Voice;
use soundlab::envelope::LinearAdsr;
use oscy::{poly_blep::PolyBlepOsc, Waveform};

// Create 8-voice polyphony
let mut poly = Polyphony::<_, _, 8>::from_factory(
    VoiceStealStrategy::Oldest,
    || Voice::new(
        PolyBlepOsc::new(44100.0, 440.0, Waveform::Saw),
        LinearAdsr::pad(44100.0),
    ),
);

// Play a chord
poly.note_on(60, 0.8); // C4
poly.note_on(64, 0.8); // E4
poly.note_on(67, 0.8); // G4

for _ in 0..44100 {
    let sample = poly.next_sample();
}

// Release
poly.note_off(60);
poly.note_off(64);
poly.note_off(67);
```

### With nih-plug
```rust
fn process(&mut self, buffer: &mut Buffer, context: &mut impl ProcessContext<Self>) {
    let mut next_event = context.next_event();

    for (sample_idx, channel_samples) in buffer.iter_samples().enumerate() {
        // Process events at their exact sample timing
        while let Some(event) = next_event {
            if event.timing() > sample_idx as u32 {
                break;
            }

            match event {
                NoteEvent::NoteOn { note, velocity, .. } => {
                    self.poly.note_on(note, velocity);
                }
                NoteEvent::NoteOff { note, .. } => {
                    self.poly.note_off(note);
                }
                _ => {}
            }

            next_event = context.next_event();
        }

        // Generate audio
        let out = self.poly.next_sample();
        for sample in channel_samples {
            *sample = out;
        }
    }
}
```

## License

MIT License - see [LICENSE](LICENSE) for details.