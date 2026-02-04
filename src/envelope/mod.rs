//! Envelope generators for amplitude shaping.
//!
//! This module provides traits and implementations for audio envelopes,
//! which control how a sound's amplitude changes over time.

mod linear_adsr;

pub use linear_adsr::LinearAdsr;

/// Trait for envelope generators.
///
/// An envelope controls amplitude over time, typically triggered by note events.
/// Implementations should output values in the `0.0` to `1.0` range.
pub trait Envelope {
    /// Triggers the envelope (note on).
    fn gate_on(&mut self);

    /// Releases the envelope (note off).
    fn gate_off(&mut self);

    /// Advances the envelope by one sample and returns the current level.
    ///
    /// Call this once per sample in your audio processing loop.
    fn next_sample(&mut self) -> f32;

    /// Returns `true` if the envelope is currently producing non-zero output.
    fn is_active(&self) -> bool;

    /// Resets the envelope to its initial idle state.
    fn reset(&mut self);
}

/// The current stage of an ADSR envelope.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum AdsrStage {
    /// Envelope is inactive and outputting zero.
    Idle,
    /// Envelope is ramping up to peak level.
    Attack,
    /// Envelope is ramping down from peak to sustain level.
    Decay,
    /// Envelope is holding at sustain level.
    Sustain,
    /// Envelope is ramping down to zero after gate off.
    Release,
}
