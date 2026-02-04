# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2026-02-04

### Added

- `envelope` module
  - `Envelope` trait for envelope generators
  - `LinearAdsr` - linear ADSR envelope with configurable attack, decay, sustain, release
  - `AdsrStage` enum for tracking envelope state
  - Presets: `pad`, `pluck`, `percussion`
  - Retrigger/legato mode support

- `voice` module
  - `Voice` struct combining oscillator and amplitude envelope

- `polyphony` module
  - `Polyphony` struct for managing multiple voices
