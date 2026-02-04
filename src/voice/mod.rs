//! Synthesizer voice management.
//!
//! A voice represents a single sound-producing unit, combining an oscillator
//! with an amplitude envelope. Voices handle MIDI note events and generate
//! audio samples.

use crate::envelope::Envelope;
use oscy::Oscillator;
use pitchy::Pitch;

/// A single synthesizer voice combining an oscillator and amplitude envelope.
///
/// The voice converts MIDI note numbers to frequencies, applies velocity scaling,
/// and shapes the output with an amplitude envelope. Use multiple voices with a
/// voice allocator for polyphonic playback.
///
/// # Type parameters
///
/// * `O` - Oscillator type (e.g., `PolyBlepOsc`, `WavetableOsc`)
/// * `E` - Envelope type (e.g., `LinearAdsr`)
///
/// # Example
///
/// ```
/// use soundlab::voice::Voice;
/// use soundlab::envelope::LinearAdsr;
/// use oscy::{poly_blep::PolyBlepOsc, Waveform};
///
/// let osc = PolyBlepOsc::new(44100.0, 440.0, Waveform::Saw);
/// let env = LinearAdsr::pad(44100.0);
/// let mut voice = Voice::new(osc, env);
///
/// // MIDI note on
/// voice.note_on(60, 0.8); // C4, velocity 0.8
///
/// // Process samples
/// for _ in 0..44100 {
///     let sample = voice.next_sample();
///     // Output sample to audio buffer
/// }
///
/// // MIDI note off
/// voice.note_off();
/// ```
pub struct Voice<O: Oscillator, E: Envelope> {
    osc: O,
    amp_env: E,
    velocity: f32,
    note: Option<u8>,
}

impl<O: Oscillator, E: Envelope> Voice<O, E> {
    /// Creates a new voice with the given oscillator and envelope.
    ///
    /// The voice starts in an idle state with no note playing.
    pub fn new(osc: O, amp_env: E) -> Self {
        Self {
            osc,
            amp_env,
            velocity: 0.0,
            note: None,
        }
    }

    /// Triggers the voice with a MIDI note and velocity.
    ///
    /// Converts the MIDI note number to a frequency and starts the envelope.
    /// Silently ignores invalid MIDI note numbers (outside 0-127 range supported by pitchy).
    ///
    /// Use [`Self::try_note_on`] if you need error handling.
    pub fn note_on(&mut self, midi_note: u8, velocity: f32) {
        let _ = self.try_note_on(midi_note, velocity);
    }

    /// Triggers the voice with a MIDI note and velocity, returning any error.
    ///
    /// Converts the MIDI note number to a frequency and starts the envelope.
    /// Returns an error if the MIDI note number cannot be converted to a pitch.
    ///
    /// # Arguments
    ///
    /// * `midi_note` - MIDI note number (0-127)
    /// * `velocity` - Note velocity, clamped to 0.0-1.0
    ///
    /// # Errors
    ///
    /// Returns `pitchy::PitchyError` if the MIDI note number is invalid.
    pub fn try_note_on(&mut self, midi_note: u8, velocity: f32) -> Result<(), pitchy::PitchyError> {
        let pitch = Pitch::try_from_midi_number(midi_note)?;

        self.note = Some(midi_note);
        self.velocity = velocity.clamp(0.0, 1.0);
        self.osc.set_frequency(pitch.frequency() as f32);
        self.amp_env.gate_on();

        Ok(())
    }

    /// Releases the voice, starting the envelope release phase.
    ///
    /// The voice continues producing sound until the envelope completes its release.
    /// Check [`Self::is_active`] to know when the voice has finished.
    pub fn note_off(&mut self) {
        self.amp_env.gate_off();
        self.note = None;
    }

    /// Generates and returns the next audio sample.
    ///
    /// Call this once per sample in your audio processing loop.
    /// Returns 0.0 when the voice is idle.
    pub fn next_sample(&mut self) -> f32 {
        let osc_out = self.osc.next_sample();
        let env_out = self.amp_env.next_sample();

        osc_out * env_out * self.velocity
    }

    /// Returns `true` if the voice is currently producing sound.
    ///
    /// A voice is active from `note_on` until the envelope finishes its release phase.
    /// Use this to determine when a voice can be reused for a new note.
    pub fn is_active(&self) -> bool {
        self.amp_env.is_active()
    }

    /// Resets the voice to its initial idle state.
    ///
    /// Stops any sound immediately without going through the release phase.
    /// Useful for voice stealing or panic/all-notes-off handling.
    pub fn reset(&mut self) {
        self.amp_env.reset();
        self.osc.reset();
        self.velocity = 0.0;
        self.note = None;
    }

    /// Returns the currently playing MIDI note, or `None` if idle.
    pub fn note(&self) -> Option<u8> {
        self.note
    }

    /// Returns the current velocity (0.0 to 1.0).
    pub fn velocity(&self) -> f32 {
        self.velocity
    }

    /// Returns a reference to the oscillator.
    pub fn osc(&self) -> &O {
        &self.osc
    }

    /// Returns a mutable reference to the oscillator.
    ///
    /// Use this to change waveform or other oscillator parameters in real-time.
    pub fn osc_mut(&mut self) -> &mut O {
        &mut self.osc
    }

    /// Returns a reference to the amplitude envelope.
    pub fn amp_env(&self) -> &E {
        &self.amp_env
    }

    /// Returns a mutable reference to the amplitude envelope.
    ///
    /// Use this to change ADSR parameters in real-time.
    pub fn amp_env_mut(&mut self) -> &mut E {
        &mut self.amp_env
    }
}
