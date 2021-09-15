extern crate cpal;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub fn play<F: FnOnce(f32) -> Vec<f32> + Send + Sync>(func: F) -> bool {
    let host = cpal::default_host();

    let device = host.default_output_device().unwrap();
    let config = device.default_output_config().unwrap();

    match config.sample_format() {
        cpal::SampleFormat::F32 => play_with_depth::<f32, F>(&device, &config.into(), func),
        cpal::SampleFormat::I16 => play_with_depth::<i16, F>(&device, &config.into(), func),
        cpal::SampleFormat::U16 => play_with_depth::<u16, F>(&device, &config.into(), func),
    }
    .is_some()
}

fn play_with_depth<T, F>(device: &cpal::Device, config: &cpal::StreamConfig, func: F) -> Option<()>
where
    T: cpal::Sample,
    F: FnOnce(f32) -> Vec<f32>,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    let samples = Arc::new(Mutex::new(VecDeque::from(func(sample_rate))));
    let samples_shared = samples.clone();

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _| write_samples(data, channels, &samples_shared),
            |err| panic!("Error playing stream: {}", err),
        )
        .ok()?;

    stream.play().ok()?;

    while !samples.lock().unwrap().is_empty() {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    Some(())
}

fn write_samples<T: cpal::Sample>(
    data: &mut [T],
    channels: usize,
    samples: &Arc<Mutex<VecDeque<f32>>>,
) {
    let mut samples = samples.lock().unwrap();
    for frame in data.chunks_mut(channels) {
        let sample = samples.pop_front().unwrap_or(0.0);
        let value: T = cpal::Sample::from(&sample);
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

#[cfg(test)]
mod test {
    use std::f32::consts::PI;
    #[test]
    fn test_play() {
        use super::play;

        fn sine(frequency: f32, duration: f32) -> impl Fn(f32) -> Option<f32> {
            move |phase: f32| {
                if phase < duration {
                    Some(f32::sin(phase * PI * 2.0 * frequency))
                } else {
                    None
                }
            }
        }

        assert!(play(|sampling_rate| {
            let osc = sine(440.0, 2.0);
            (0..)
                .scan((), |_, i| {
                    let phase = i as f32 / sampling_rate;
                    osc(phase)
                })
                .collect()
        }));
    }
}
