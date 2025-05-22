use fundsp::prelude::*;
use std::f64::consts::TAU;

/// Custom oscillator with multiple waveforms
#[derive(Clone)]
pub struct MultiOscillator {
    frequency: f64,
    waveform: i32,
    phase: f64,
    sample_rate: f64,
}

impl MultiOscillator {
    pub fn new(frequency: f64, waveform: i32, sample_rate: f64) -> Self {
        Self {
            frequency,
            waveform,
            phase: 0.0,
            sample_rate,
        }
    }

    pub fn set_frequency(&mut self, freq: f64) {
        self.frequency = freq;
    }

    pub fn set_waveform(&mut self, waveform: i32) {
        self.waveform = waveform;
    }
}

impl AudioNode for MultiOscillator {
    const ID: u64 = 12345; // Unique ID for this node type
    type Inputs = typenum::U0;
    type Outputs = typenum::U1;

    fn tick(&mut self, _input: &Frame<f64, Self::Inputs>) -> Frame<f64, Self::Outputs> {
        let phase_increment = self.frequency * TAU / self.sample_rate;
        self.phase += phase_increment;

        if self.phase >= TAU {
            self.phase -= TAU;
        }

        let output = match self.waveform {
            0 => self.phase.sin(), // Sine
            1 => {
                // Sawtooth
                2.0 * (self.phase / TAU) - 1.0
            }
            2 => {
                // Square
                if self.phase < std::f64::consts::PI {
                    1.0
                } else {
                    -1.0
                }
            }
            3 => {
                // Triangle
                let normalized = self.phase / TAU;
                if normalized < 0.5 {
                    4.0 * normalized - 1.0
                } else {
                    3.0 - 4.0 * normalized
                }
            }
            _ => self.phase.sin(), // Default to sine
        };

        [output].into()
    }

    fn reset(&mut self) {
        self.phase = 0.0;
    }
}

/// Enhanced filter with multiple types
#[derive(Clone)]
pub struct MultiFilter {
    filter_type: i32,
    cutoff: f64,
    resonance: f64,
    sample_rate: f64,
    // State variables for the filter
    low: f64,
    band: f64,
    high: f64,
}

impl MultiFilter {
    pub fn new(filter_type: i32, cutoff: f64, resonance: f64, sample_rate: f64) -> Self {
        Self {
            filter_type,
            cutoff,
            resonance,
            sample_rate,
            low: 0.0,
            band: 0.0,
            high: 0.0,
        }
    }

    pub fn set_cutoff(&mut self, cutoff: f64) {
        self.cutoff = cutoff;
    }

    pub fn set_resonance(&mut self, resonance: f64) {
        self.resonance = resonance;
    }

    pub fn set_type(&mut self, filter_type: i32) {
        self.filter_type = filter_type;
    }
}

impl AudioNode for MultiFilter {
    const ID: u64 = 12346;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;

    fn tick(&mut self, input: &Frame<f64, Self::Inputs>) -> Frame<f64, Self::Outputs> {
        let input_sample = input[0];

        // State variable filter implementation
        let f = 2.0 * (self.cutoff / self.sample_rate).sin();
        let q = self.resonance.max(0.01); // Prevent division by zero

        self.low += f * self.band;
        self.high = input_sample - self.low - q * self.band;
        self.band += f * self.high;

        let output = match self.filter_type {
            0 => self.low,  // Lowpass
            1 => self.high, // Highpass
            2 => self.band, // Bandpass
            _ => self.low,  // Default to lowpass
        };

        [output].into()
    }

    fn reset(&mut self) {
        self.low = 0.0;
        self.band = 0.0;
        self.high = 0.0;
    }
}

/// Simple delay line with feedback
#[derive(Clone)]
pub struct SimpleDelay {
    buffer: Vec<f64>,
    write_pos: usize,
    delay_samples: usize,
    feedback: f64,
    wet: f64,
}

impl SimpleDelay {
    pub fn new(delay_time: f64, feedback: f64, wet: f64, sample_rate: f64) -> Self {
        let delay_samples = (delay_time * sample_rate) as usize;
        let buffer_size = delay_samples.max(1);

        Self {
            buffer: vec![0.0; buffer_size],
            write_pos: 0,
            delay_samples,
            feedback,
            wet,
        }
    }

    pub fn set_delay_time(&mut self, delay_time: f64, sample_rate: f64) {
        let new_delay_samples = (delay_time * sample_rate) as usize;
        if new_delay_samples != self.delay_samples {
            self.delay_samples = new_delay_samples;
            if new_delay_samples >= self.buffer.len() {
                self.buffer.resize(new_delay_samples + 1, 0.0);
            }
        }
    }

    pub fn set_feedback(&mut self, feedback: f64) {
        self.feedback = feedback.clamp(0.0, 0.95); // Prevent runaway feedback
    }

    pub fn set_wet(&mut self, wet: f64) {
        self.wet = wet.clamp(0.0, 1.0);
    }
}

impl AudioNode for SimpleDelay {
    const ID: u64 = 12347;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;

    fn tick(&mut self, input: &Frame<f64, Self::Inputs>) -> Frame<f64, Self::Outputs> {
        let input_sample = input[0];

        // Calculate read position
        let read_pos = if self.write_pos >= self.delay_samples {
            self.write_pos - self.delay_samples
        } else {
            self.buffer.len() + self.write_pos - self.delay_samples
        };

        // Read delayed sample
        let delayed_sample = self.buffer[read_pos];

        // Write new sample with feedback
        self.buffer[self.write_pos] = input_sample + delayed_sample * self.feedback;

        // Advance write position
        self.write_pos = (self.write_pos + 1) % self.buffer.len();

        // Mix dry and wet signals
        let output = input_sample * (1.0 - self.wet) + delayed_sample * self.wet;

        [output].into()
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
    }
}

/// Simple reverb using multiple delays
#[derive(Clone)]
pub struct SimpleReverb {
    delays: Vec<SimpleDelay>,
    wet: f64,
}

impl SimpleReverb {
    pub fn new(room_size: f64, damping: f64, wet: f64, sample_rate: f64) -> Self {
        // Create multiple delay lines with different lengths for reverb effect
        let delay_times = [
            0.03 * room_size,
            0.05 * room_size,
            0.07 * room_size,
            0.11 * room_size,
        ];

        let delays = delay_times
            .iter()
            .map(|&time| SimpleDelay::new(time, damping * 0.7, 1.0, sample_rate))
            .collect();

        Self { delays, wet }
    }

    pub fn set_room_size(&mut self, room_size: f64, sample_rate: f64) {
        let delay_times = [
            0.03 * room_size,
            0.05 * room_size,
            0.07 * room_size,
            0.11 * room_size,
        ];

        for (delay, &time) in self.delays.iter_mut().zip(delay_times.iter()) {
            delay.set_delay_time(time, sample_rate);
        }
    }

    pub fn set_damping(&mut self, damping: f64) {
        for delay in &mut self.delays {
            delay.set_feedback(damping * 0.7);
        }
    }

    pub fn set_wet(&mut self, wet: f64) {
        self.wet = wet.clamp(0.0, 1.0);
    }
}

impl AudioNode for SimpleReverb {
    const ID: u64 = 12348;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;

    fn tick(&mut self, input: &Frame<f64, Self::Inputs>) -> Frame<f64, Self::Outputs> {
        let input_sample = input[0];

        // Process through all delay lines and sum
        let mut reverb_sum = 0.0;
        for delay in &mut self.delays {
            let delayed = delay.tick(&[input_sample].into())[0];
            reverb_sum += delayed;
        }

        // Average the reverb signals
        let reverb_output = reverb_sum / self.delays.len() as f64;

        // Mix dry and wet
        let output = input_sample * (1.0 - self.wet) + reverb_output * self.wet;

        [output].into()
    }

    fn reset(&mut self) {
        for delay in &mut self.delays {
            delay.reset();
        }
    }
}

/// Stereo panner and output
#[derive(Clone)]
pub struct StereoOutput {
    volume: f64,
    pan: f64,
}

impl StereoOutput {
    pub fn new(volume: f64, pan: f64) -> Self {
        Self {
            volume: volume.clamp(0.0, 1.0),
            pan: pan.clamp(0.0, 1.0),
        }
    }

    pub fn set_volume(&mut self, volume: f64) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    pub fn set_pan(&mut self, pan: f64) {
        self.pan = pan.clamp(0.0, 1.0);
    }
}

impl AudioNode for StereoOutput {
    const ID: u64 = 12349;
    type Inputs = typenum::U1;
    type Outputs = typenum::U2;

    fn tick(&mut self, input: &Frame<f64, Self::Inputs>) -> Frame<f64, Self::Outputs> {
        let input_sample = input[0] * self.volume;

        // Calculate left and right channels based on pan
        let left = input_sample * (1.0 - self.pan);
        let right = input_sample * self.pan;

        [left, right].into()
    }

    fn reset(&mut self) {
        // No state to reset for a simple panner
    }
}

/// Utility functions for creating nodes from modules
pub fn create_oscillator_node(
    frequency: f64,
    waveform: i32,
    amplitude: f64,
    sample_rate: f64,
) -> An<impl AudioNode<Inputs = U0, Outputs = U1>> {
    An(MultiOscillator::new(frequency, waveform, sample_rate)) * amplitude
}

pub fn create_filter_node(
    filter_type: i32,
    cutoff: f64,
    resonance: f64,
    sample_rate: f64,
) -> An<impl AudioNode<Inputs = U1, Outputs = U1>> {
    An(MultiFilter::new(
        filter_type,
        cutoff,
        resonance,
        sample_rate,
    ))
}

pub fn create_delay_node(
    delay_time: f64,
    feedback: f64,
    wet: f64,
    sample_rate: f64,
) -> An<impl AudioNode<Inputs = U1, Outputs = U1>> {
    An(SimpleDelay::new(delay_time, feedback, wet, sample_rate))
}

pub fn create_reverb_node(
    room_size: f64,
    damping: f64,
    wet: f64,
    sample_rate: f64,
) -> An<impl AudioNode<Inputs = U1, Outputs = U1>> {
    An(SimpleReverb::new(room_size, damping, wet, sample_rate))
}

pub fn create_output_node(volume: f64, pan: f64) -> An<impl AudioNode<Inputs = U1, Outputs = U2>> {
    An(StereoOutput::new(volume, pan))
}
