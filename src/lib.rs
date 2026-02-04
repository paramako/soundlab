//! A toolkit for building software synthesizers and audio plugins.
//!
//! `soundlab` provides reusable components for audio development, designed to be
//! composed into VST plugins, standalone synthesizers, and other audio applications.
//!
//! # Goals
//!
//! - Provide building blocks for common synthesis tasks
//! - Support polyphonic voice management
//! - Keep components decoupled and reusable
//! - Maintain performance suitable for real-time audio
//!
//! # Modules
//!
//! - [`envelope`] - Envelope generators (ADSR, etc.) for amplitude and modulation shaping
//! - [`voice`] - Synthesizer voice combining oscillator and envelope
//! - [`polyphony`] - Polyphonic voice allocation and management

pub mod envelope;
pub mod voice;
pub mod polyphony;
