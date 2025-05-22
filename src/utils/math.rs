use std::f64::consts::{PI, TAU};

/// Convert frequency to MIDI note number
pub fn freq_to_midi(freq: f64) -> f64 {
    69.0 + 12.0 * (freq / 440.0).log2()
}

/// Convert MIDI note number to frequency
pub fn midi_to_freq(midi: f64) -> f64 {
    440.0 * 2.0_f64.powf((midi - 69.0) / 12.0)
}

/// Linear interpolation
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + t * (b - a)
}

/// Convert linear amplitude to dB
pub fn amp_to_db(amp: f64) -> f64 {
    if amp <= 0.0 {
        -60.0 // -60dB floor
    } else {
        20.0 * amp.log10()
    }
}

/// Convert dB to linear amplitude
pub fn db_to_amp(db: f64) -> f64 {
    10.0_f64.powf(db / 20.0)
}

/// Clamp value between min and max
pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Normalize value from one range to another
pub fn normalize_range(value: f64, from_min: f64, from_max: f64, to_min: f64, to_max: f64) -> f64 {
    let normalized = (value - from_min) / (from_max - from_min);
    to_min + normalized * (to_max - to_min)
}

/// Simple low-pass filter coefficient calculation
pub fn lowpass_coeff(cutoff_freq: f64, sample_rate: f64) -> f64 {
    let rc = 1.0 / (cutoff_freq * TAU);
    let dt = 1.0 / sample_rate;
    dt / (rc + dt)
}

/// High-pass filter coefficient calculation
pub fn highpass_coeff(cutoff_freq: f64, sample_rate: f64) -> f64 {
    let rc = 1.0 / (cutoff_freq * TAU);
    let dt = 1.0 / sample_rate;
    rc / (rc + dt)
}

/// Calculate delay time in samples
pub fn delay_time_to_samples(delay_time: f64, sample_rate: f64) -> usize {
    (delay_time * sample_rate).round() as usize
}

/// Convert BPM to samples per beat
pub fn bpm_to_samples_per_beat(bpm: f64, sample_rate: f64) -> f64 {
    (60.0 / bpm) * sample_rate
}

/// Generate a window function (Hanning window)
pub fn hanning_window(size: usize) -> Vec<f64> {
    (0..size)
        .map(|i| {
            let phase = TAU * i as f64 / (size - 1) as f64;
            0.5 * (1.0 - phase.cos())
        })
        .collect()
}

/// Calculate RMS (Root Mean Square) of a signal
pub fn rms(samples: &[f64]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }

    let sum_squares: f64 = samples.iter().map(|&x| x * x).sum();
    (sum_squares / samples.len() as f64).sqrt()
}

/// Calculate peak amplitude of a signal
pub fn peak(samples: &[f64]) -> f64 {
    samples.iter().map(|&x| x.abs()).fold(0.0, f64::max)
}

/// Simple envelope generator states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// ADSR envelope generator
#[derive(Debug, Clone)]
pub struct ADSREnvelope {
    pub attack_time: f64,   // in seconds
    pub decay_time: f64,    // in seconds
    pub sustain_level: f64, // 0.0 to 1.0
    pub release_time: f64,  // in seconds

    state: EnvelopeState,
    current_level: f64,
    sample_rate: f64,
    samples_in_state: u64,
}

impl ADSREnvelope {
    pub fn new(attack: f64, decay: f64, sustain: f64, release: f64, sample_rate: f64) -> Self {
        Self {
            attack_time: attack,
            decay_time: decay,
            sustain_level: sustain.clamp(0.0, 1.0),
            release_time: release,
            state: EnvelopeState::Idle,
            current_level: 0.0,
            sample_rate,
            samples_in_state: 0,
        }
    }

    pub fn note_on(&mut self) {
        self.state = EnvelopeState::Attack;
        self.samples_in_state = 0;
    }

    pub fn note_off(&mut self) {
        self.state = EnvelopeState::Release;
        self.samples_in_state = 0;
    }

    pub fn tick(&mut self) -> f64 {
        let samples_per_second = self.sample_rate;

        match self.state {
            EnvelopeState::Idle => {
                self.current_level = 0.0;
            }
            EnvelopeState::Attack => {
                let attack_samples = (self.attack_time * samples_per_second) as u64;
                if attack_samples == 0 {
                    self.current_level = 1.0;
                    self.state = EnvelopeState::Decay;
                    self.samples_in_state = 0;
                } else if self.samples_in_state >= attack_samples {
                    self.current_level = 1.0;
                    self.state = EnvelopeState::Decay;
                    self.samples_in_state = 0;
                } else {
                    self.current_level = self.samples_in_state as f64 / attack_samples as f64;
                }
            }
            EnvelopeState::Decay => {
                let decay_samples = (self.decay_time * samples_per_second) as u64;
                if decay_samples == 0 {
                    self.current_level = self.sustain_level;
                    self.state = EnvelopeState::Sustain;
                    self.samples_in_state = 0;
                } else if self.samples_in_state >= decay_samples {
                    self.current_level = self.sustain_level;
                    self.state = EnvelopeState::Sustain;
                    self.samples_in_state = 0;
                } else {
                    let progress = self.samples_in_state as f64 / decay_samples as f64;
                    self.current_level = 1.0 + progress * (self.sustain_level - 1.0);
                }
            }
            EnvelopeState::Sustain => {
                self.current_level = self.sustain_level;
            }
            EnvelopeState::Release => {
                let release_samples = (self.release_time * samples_per_second) as u64;
                if release_samples == 0 {
                    self.current_level = 0.0;
                    self.state = EnvelopeState::Idle;
                    self.samples_in_state = 0;
                } else if self.samples_in_state >= release_samples {
                    self.current_level = 0.0;
                    self.state = EnvelopeState::Idle;
                    self.samples_in_state = 0;
                } else {
                    let progress = self.samples_in_state as f64 / release_samples as f64;
                    let release_start_level =
                        if self.state == EnvelopeState::Release && self.samples_in_state == 0 {
                            self.current_level
                        } else {
                            self.sustain_level
                        };
                    self.current_level = release_start_level * (1.0 - progress);
                }
            }
        }

        self.samples_in_state += 1;
        self.current_level
    }

    pub fn is_active(&self) -> bool {
        !matches!(self.state, EnvelopeState::Idle)
    }

    pub fn get_state(&self) -> EnvelopeState {
        self.state
    }
}

/// Scale a value using different curve types
#[derive(Debug, Clone, Copy)]
pub enum ScaleCurve {
    Linear,
    Exponential(f64), // exponent
    Logarithmic,
}

pub fn scale_with_curve(value: f64, curve: ScaleCurve) -> f64 {
    let normalized = value.clamp(0.0, 1.0);

    match curve {
        ScaleCurve::Linear => normalized,
        ScaleCurve::Exponential(exp) => normalized.powf(exp),
        ScaleCurve::Logarithmic => {
            if normalized <= 0.0 {
                0.0
            } else {
                normalized.ln() / (-1.0_f64).ln() + 1.0
            }
        }
    }
}
