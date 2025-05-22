use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use fundsp::prelude::*;
use std::sync::{Arc, Mutex};

pub struct AudioEngine {
    pub sample_rate: f32,
    // Simple parameters for now - we'll make this more sophisticated later
    pub current_frequency: Arc<Mutex<f32>>,
    pub current_amplitude: Arc<Mutex<f32>>,
}

impl AudioEngine {
    pub fn new() -> Result<Self> {
        Ok(Self {
            sample_rate: 44100.0,
            current_frequency: Arc::new(Mutex::new(440.0)),
            current_amplitude: Arc::new(Mutex::new(0.5)),
        })
    }

    pub fn start_audio_stream(&self) -> Result<cpal::Stream> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow::anyhow!("No output device available"))?;

        let config = device.default_output_config()?;

        let freq = Arc::clone(&self.current_frequency);
        let amp = Arc::clone(&self.current_amplitude);

        let stream = device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let frequency = *freq.lock().unwrap();
                let amplitude = *amp.lock().unwrap();

                // Simple sine wave for now
                static mut PHASE: f32 = 0.0;
                for sample in data.iter_mut() {
                    unsafe {
                        *sample = (PHASE * frequency * 2.0 * std::f32::consts::PI / 44100.0).sin()
                            * amplitude;
                        PHASE += 1.0;
                        if PHASE >= 44100.0 {
                            PHASE = 0.0;
                        }
                    }
                }
            },
            |err| eprintln!("Audio stream error: {}", err),
            None,
        )?;

        stream.play()?;
        Ok(stream)
    }

    pub fn update_frequency(&self, freq: f32) {
        *self.current_frequency.lock().unwrap() = freq;
    }

    pub fn update_amplitude(&self, amp: f32) {
        *self.current_amplitude.lock().unwrap() = amp;
    }
}
