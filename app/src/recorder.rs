//! Microphone capture. Audio stays in memory and is dropped after processing.

use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};

#[derive(Default)]
struct Captured {
    samples: Vec<f32>,
    peak: f32,
}

pub struct Recording {
    pub samples: Vec<f32>,
    pub channels: usize,
    pub rate: u32,
}

pub struct Recorder {
    captured: Arc<Mutex<Captured>>,
    channels: usize,
    rate: u32,
    stop: Sender<()>,
    thread: JoinHandle<()>,
}

impl Recorder {
    pub fn start() -> Result<Self, String> {
        let captured = Arc::new(Mutex::new(Captured::default()));
        let (stop, stopped) = mpsc::channel::<()>();
        let (ready, started) = mpsc::channel::<Result<(usize, u32), String>>();

        // The stream is not `Send` on every platform, so it lives on its own thread.
        let sink = captured.clone();
        let thread = std::thread::spawn(move || match open_stream(sink) {
            Ok((stream, channels, rate)) => {
                let _ = ready.send(Ok((channels, rate)));
                let _ = stopped.recv();
                drop(stream);
            }
            Err(e) => {
                let _ = ready.send(Err(e));
            }
        });

        let (channels, rate) = started
            .recv()
            .map_err(|_| "microphone thread stopped".to_string())??;
        Ok(Self {
            captured,
            channels,
            rate,
            stop,
            thread,
        })
    }

    /// Seconds recorded so far and the peak level since the previous call.
    pub fn progress(&self) -> (f32, f32) {
        let mut captured = self.captured.lock().unwrap();
        let seconds = captured.samples.len() as f32 / (self.channels as f32 * self.rate as f32);
        (seconds, std::mem::take(&mut captured.peak))
    }

    pub fn finish(self) -> Recording {
        let _ = self.stop.send(());
        let _ = self.thread.join();
        let samples = std::mem::take(&mut self.captured.lock().unwrap().samples);
        Recording {
            samples,
            channels: self.channels,
            rate: self.rate,
        }
    }
}

fn open_stream(sink: Arc<Mutex<Captured>>) -> Result<(Stream, usize, u32), String> {
    let device = cpal::default_host()
        .default_input_device()
        .ok_or("no microphone found")?;
    let config = device.default_input_config().map_err(|e| e.to_string())?;
    let channels = config.channels() as usize;
    let rate = config.sample_rate();
    let format = config.sample_format();
    let on_error = |e| eprintln!("microphone error: {e}");

    let push = move |frames: &mut dyn Iterator<Item = f32>| {
        let mut captured = sink.lock().unwrap();
        for sample in frames {
            captured.peak = captured.peak.max(sample.abs());
            captured.samples.push(sample);
        }
    };
    let stream = match format {
        SampleFormat::F32 => device.build_input_stream(
            config.into(),
            move |data: &[f32], _: &_| push(&mut data.iter().copied()),
            on_error,
            None,
        ),
        SampleFormat::I16 => device.build_input_stream(
            config.into(),
            move |data: &[i16], _: &_| push(&mut data.iter().map(|s| *s as f32 / 32768.0)),
            on_error,
            None,
        ),
        other => return Err(format!("unsupported microphone sample format: {other:?}")),
    }
    .map_err(|e| e.to_string())?;
    stream.play().map_err(|e| e.to_string())?;
    Ok((stream, channels, rate))
}
