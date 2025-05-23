use rodio::{OutputStream, Sink, buffer::SamplesBuffer};

pub fn play_sine_wave(freq: f32, duration_secs: u32) {
    let sample_rate = 44100;
    let total_samples = sample_rate * duration_secs;

    let samples: Vec<i16> = (0..total_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            let amplitude = (2.0 * std::f32::consts::PI * freq * t).sin();
            (amplitude * i16::MAX as f32) as i16
        })
        .collect();

    if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
        if let Ok(sink) = Sink::try_new(&stream_handle) {
            let source = SamplesBuffer::new(1, sample_rate as u32, samples);
            sink.append(source);
            sink.sleep_until_end();
        }
    }
}
