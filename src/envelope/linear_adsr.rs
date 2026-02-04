use crate::envelope::{AdsrStage, Envelope};

/// Minimum time in seconds for attack, decay, and release to prevent division by zero.
const MIN_TIME: f32 = 0.001;
/// Minimum envelope level (silent).
const MIN_LEVEL: f32 = 0.0;
/// Maximum envelope level (full amplitude).
const MAX_LEVEL: f32 = 1.0;

/// Linear ADSR (Attack, Decay, Sustain, Release) envelope generator.
///
/// Attempt to traverse from one level to the next in a straight line over time.
/// The envelope output ranges from `0.0` (silent) to `1.0` (full amplitude).
///
/// # Timing behavior
///
/// The time parameters represent the duration to traverse the full `0.0` to `1.0` range:
/// - **Attack**: Time to ramp from `0.0` to `1.0`
/// - **Decay**: Time to ramp from `1.0` to `0.0` (not to sustain level)
/// - **Release**: Time to ramp from `1.0` to `0.0` (not from current level)
///
/// This means actual decay/release times depend on the distance traveled.
/// For example, with `sustain = 0.8`, decay only covers 20% of the range,
/// so it completes in 20% of the decay time parameter.
///
/// # Example
///
/// ```
/// use soundlab::envelope::{Envelope, LinearAdsr};
///
/// let mut env = LinearAdsr::new(44100.0, 0.01, 0.1, 0.7, 0.3);
/// env.gate_on();
///
/// // Process samples in your audio callback
/// for _ in 0..44100 {
///     let amplitude = env.next_sample();
///     // Multiply your oscillator output by amplitude
/// }
///
/// env.gate_off(); // Start release phase
/// ```
pub struct LinearAdsr {
    sample_rate: f32,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    attack_rate: f32,
    decay_rate: f32,
    release_rate: f32,
    stage: AdsrStage,
    level: f32,
    retrigger: bool,
}

impl LinearAdsr {
    /// Creates a new linear ADSR envelope.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - Sample rate in Hz (e.g., 44100.0)
    /// * `attack` - Attack time in seconds
    /// * `decay` - Decay time in seconds
    /// * `sustain` - Sustain level from 0.0 to 1.0
    /// * `release` - Release time in seconds
    ///
    /// Time values are clamped to a minimum of 1ms to prevent division by zero.
    /// Sustain is clamped to the 0.0–1.0 range.
    pub fn new(sample_rate: f32, attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        let attack = attack.max(MIN_TIME);
        let decay = decay.max(MIN_TIME);
        let release = release.max(MIN_TIME);

        Self {
            sample_rate,
            attack,
            decay,
            sustain: sustain.clamp(MIN_LEVEL, MAX_LEVEL),
            release,
            attack_rate: 1.0 / (attack * sample_rate),
            decay_rate: 1.0 / (decay * sample_rate),
            release_rate: 1.0 / (release * sample_rate),
            stage: AdsrStage::Idle,
            level: MIN_LEVEL,
            retrigger: true,
        }
    }

    /// Creates a pad preset with slow attack and long release.
    pub fn pad(sample_rate: f32) -> Self {
        Self::new(sample_rate, 0.01, 0.1, 0.7, 0.3)
    }

    /// Creates a pluck preset with instant attack and no sustain.
    pub fn pluck(sample_rate: f32) -> Self {
        Self::new(sample_rate, 0.001, 0.3, 0.0, 0.1)
    }

    /// Creates a percussion preset with instant attack and fast decay.
    pub fn percussion(sample_rate: f32) -> Self {
        Self::new(sample_rate, 0.001, 0.1, 0.0, 0.05)
    }

    /// Sets the attack time in seconds.
    pub fn set_attack(&mut self, seconds: f32) {
        self.attack = seconds.max(MIN_TIME);
        self.attack_rate = 1.0 / (self.attack * self.sample_rate);
    }

    /// Sets the decay time in seconds.
    pub fn set_decay(&mut self, seconds: f32) {
        self.decay = seconds.max(MIN_TIME);
        self.decay_rate = 1.0 / (self.decay * self.sample_rate);
    }

    /// Sets the sustain level (0.0 to 1.0).
    pub fn set_sustain(&mut self, level: f32) {
        self.sustain = level.clamp(MIN_LEVEL, MAX_LEVEL);
    }

    /// Sets the release time in seconds.
    pub fn set_release(&mut self, seconds: f32) {
        self.release = seconds.max(MIN_TIME);
        self.release_rate = 1.0 / (self.release * self.sample_rate);
    }

    /// Returns the attack time in seconds.
    pub fn attack(&self) -> f32 {
        self.attack
    }

    /// Returns the decay time in seconds.
    pub fn decay(&self) -> f32 {
        self.decay
    }

    /// Returns the sustain level.
    pub fn sustain(&self) -> f32 {
        self.sustain
    }

    /// Returns the release time in seconds.
    pub fn release(&self) -> f32 {
        self.release
    }

    /// Returns whether retrigger mode is enabled.
    pub fn retrigger(&self) -> bool {
        self.retrigger
    }

    /// Sets the retrigger behavior.
    ///
    /// When `true` (default), `gate_on` resets the level to zero before starting attack.
    /// When `false`, the envelope continues from its current level (legato behavior).
    pub fn set_retrigger(&mut self, retrigger: bool) {
        self.retrigger = retrigger;
    }

    /// Returns the current envelope stage.
    pub fn stage(&self) -> AdsrStage {
        self.stage
    }

    /// Returns the current envelope level (0.0 to 1.0).
    pub fn level(&self) -> f32 {
        self.level
    }
}

impl Envelope for LinearAdsr {
    fn gate_on(&mut self) {
        if self.retrigger {
            self.level = MIN_LEVEL;
        }
        self.stage = if self.level >= MAX_LEVEL {
            AdsrStage::Decay
        } else {
            AdsrStage::Attack
        };
    }

    fn gate_off(&mut self) {
        if self.stage != AdsrStage::Idle {
            self.stage = AdsrStage::Release;
        }
    }

    fn next_sample(&mut self) -> f32 {
        match self.stage {
            AdsrStage::Idle => {}

            AdsrStage::Attack => {
                self.level += self.attack_rate;
                if self.level >= MAX_LEVEL {
                    self.level = MAX_LEVEL;
                    self.stage = AdsrStage::Decay;
                }
            }

            AdsrStage::Decay => {
                self.level -= self.decay_rate;
                if self.level <= self.sustain {
                    self.level = self.sustain;
                    self.stage = AdsrStage::Sustain;
                }
            }

            AdsrStage::Sustain => {}

            AdsrStage::Release => {
                self.level -= self.release_rate;
                if self.level <= MIN_LEVEL {
                    self.level = MIN_LEVEL;
                    self.stage = AdsrStage::Idle;
                }
            }
        }

        self.level
    }

    fn is_active(&self) -> bool {
        self.stage != AdsrStage::Idle
    }

    fn reset(&mut self) {
        self.level = MIN_LEVEL;
        self.stage = AdsrStage::Idle;
    }
}
