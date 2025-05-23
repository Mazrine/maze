// src/audio/synth.rs
use log::{error, info, warn};
use rodio::{OutputStream, Sink, buffer::SamplesBuffer}; // Import logging macros

pub fn play_sine_wave(freq: f32, duration_secs: u32) {
    info!(
        "Attempting to play sine wave at {}Hz for {} seconds...",
        freq, duration_secs
    );

    let sample_rate = 44100;
    let total_samples = sample_rate * duration_secs;

    let samples: Vec<i16> = (0..total_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            let amplitude = (2.0 * std::f32::consts::PI * freq * t).sin();
            (amplitude * i16::MAX as f32) as i16
        })
        .collect();

    match OutputStream::try_default() {
        Ok((_stream, stream_handle)) => {
            info!("Obtained audio output stream.");
            match Sink::try_new(&stream_handle) {
                Ok(sink) => {
                    let source = SamplesBuffer::new(1, sample_rate as u32, samples);
                    sink.append(source);
                    info!("Audio appended to sink. Waiting for playback to finish...");
                    sink.sleep_until_end();
                    info!("Sine wave playback finished.");
                }
                Err(e) => {
                    error!("Failed to create audio sink: {}", e);
                }
            }
        }
        Err(e) => {
            error!("Failed to get default audio output stream: {}", e);
            warn!(
                "This might be why the pipewire warning appears externally if rodio/cpal failed to initialize properly due to missing audio devices or permissions."
            );
        }
    }
}
