//! Polyphonic voice management.
//!
//! This module provides voice allocation and management for polyphonic synthesizers.
//! It handles note-on/off events, voice stealing when all voices are busy, and
//! mixing multiple voices into a single output.

use crate::envelope::Envelope;
use crate::voice::Voice;
use oscy::Oscillator;

/// Polyphonic voice manager.
///
/// Manages a fixed number of voices for playing chords and handling note allocation.
/// When all voices are busy, a voice is stolen based on the configured strategy.
///
/// # Type parameters
///
/// * `O` - Oscillator type
/// * `E` - Envelope type
/// * `N` - Number of voices (const generic)
///
/// # Example
///
/// ```
/// use soundlab::polyphony::{Polyphony, VoiceStealStrategy};
/// use soundlab::voice::Voice;
/// use soundlab::envelope::LinearAdsr;
/// use oscy::{poly_blep::PolyBlepOsc, Waveform};
///
/// // Create 8-voice polyphony
/// let mut poly = Polyphony::<_, _, 8>::from_factory(
///     VoiceStealStrategy::default(),
///     || {
///         Voice::new(
///             PolyBlepOsc::new(44100.0, 440.0, Waveform::Saw),
///             LinearAdsr::pad(44100.0),
///         )
///     },
/// );
///
/// // Play a chord
/// poly.note_on(60, 0.8); // C4
/// poly.note_on(64, 0.8); // E4
/// poly.note_on(67, 0.8); // G4
///
/// // Process samples
/// for _ in 0..44100 {
///     let sample = poly.next_sample();
/// }
///
/// // Release all
/// poly.note_off(60);
/// poly.note_off(64);
/// poly.note_off(67);
/// ```
pub struct Polyphony<O: Oscillator, E: Envelope, const N: usize> {
    voices: [Voice<O, E>; N],
    /// Tracks order of note-ons for voice stealing (oldest first).
    ages: [u64; N],
    /// Counter incremented on each note_on.
    counter: u64,
    steal_strategy: VoiceStealStrategy,
}

impl<O: Oscillator, E: Envelope, const N: usize> Polyphony<O, E, N> {
    /// Creates a new polyphonic voice manager with the given voices and steal strategy.
    pub fn new(voices: [Voice<O, E>; N], steal_strategy: VoiceStealStrategy) -> Self {
        Self {
            voices,
            ages: [0; N],
            counter: 0,
            steal_strategy,
        }
    }

    /// Creates a new polyphonic voice manager using a factory function.
    ///
    /// The factory is called `N` times to create each voice.
    pub fn from_factory<F>(steal_strategy: VoiceStealStrategy, mut factory: F) -> Self
    where
        F: FnMut() -> Voice<O, E>,
    {
        Self {
            voices: std::array::from_fn(|_| factory()),
            ages: [0; N],
            counter: 0,
            steal_strategy,
        }
    }

    /// Triggers a note on a free voice, or steals one if all are busy.
    pub fn note_on(&mut self, midi_note: u8, velocity: f32) {
        let voice_idx = self
            .find_free_voice()
            .unwrap_or_else(|| self.find_voice_to_steal());

        self.counter += 1;
        self.ages[voice_idx] = self.counter;
        self.voices[voice_idx].note_on(midi_note, velocity);
    }

    /// Releases a note by finding the voice playing it.
    ///
    /// Does nothing if no voice is playing the given note.
    pub fn note_off(&mut self, midi_note: u8) {
        if let Some(voice) = self
            .voices
            .iter_mut()
            .find(|v| v.note() == Some(midi_note))
        {
            voice.note_off();
        }
    }

    /// Generates the next audio sample by summing all active voices.
    pub fn next_sample(&mut self) -> f32 {
        self.voices
            .iter_mut()
            .filter(|v| v.is_active())
            .map(|v| v.next_sample())
            .sum()
    }

    /// Returns the number of currently active voices.
    pub fn active_count(&self) -> usize {
        self.voices.iter().filter(|v| v.is_active()).count()
    }

    /// Resets all voices to idle state immediately.
    ///
    /// Useful for panic/all-notes-off MIDI messages.
    pub fn reset(&mut self) {
        for voice in &mut self.voices {
            voice.reset();
        }
        self.ages = [0; N];
        self.counter = 0;
    }

    /// Returns a reference to the voice array.
    pub fn voices(&self) -> &[Voice<O, E>; N] {
        &self.voices
    }

    /// Returns a mutable reference to the voice array.
    ///
    /// Use this to modify oscillator or envelope parameters on all voices.
    pub fn voices_mut(&mut self) -> &mut [Voice<O, E>; N] {
        &mut self.voices
    }

    fn find_voice_to_steal(&self) -> usize {
        match self.steal_strategy {
            VoiceStealStrategy::Oldest => self
                .ages
                .iter()
                .enumerate()
                .min_by_key(|(_, age)| *age)
                .map(|(idx, _)| idx)
                .unwrap_or(0),
        }
    }

    fn find_free_voice(&self) -> Option<usize> {
        self.voices.iter().position(|v| !v.is_active())
    }
    
    /// Returns the total number of voices.
    pub const fn capacity(&self) -> usize {
        N
    }
}

/// Strategy for selecting which voice to steal when all voices are busy.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum VoiceStealStrategy {
    /// Steal the voice that has been playing the longest.
    #[default]
    Oldest,
}
