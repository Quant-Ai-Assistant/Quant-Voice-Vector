//! Decoding of audio files into mono 16 kHz samples.

use std::io::ErrorKind;
use std::path::Path;

use sherpa_onnx::LinearResampler;
use symphonia::core::codecs::CodecParameters;
use symphonia::core::codecs::registry::CodecRegistry;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::TrackType;
use symphonia::core::io::MediaSourceStream;
use symphonia_adapter_libopus::OpusDecoder;

use crate::error::{Error, Result};
use crate::model::SAMPLE_RATE;

/// Decodes any supported file (wav, mp3, m4a, flac, ogg, opus, webm) to mono 16 kHz.
pub fn load_file(path: &Path) -> Result<Vec<f32>> {
    let (samples, rate) = decode_mono(path)?;
    Ok(to_16k(&samples, rate))
}

/// Mixes interleaved samples down to mono and resamples them to 16 kHz.
pub fn interleaved_to_16k(samples: &[f32], channels: usize, rate: u32) -> Vec<f32> {
    to_16k(&downmix(samples, channels), rate)
}

fn decode_mono(path: &Path) -> Result<(Vec<f32>, u32)> {
    let file = std::fs::File::open(path)
        .map_err(|source| Error::Io { path: path.to_path_buf(), source })?;
    let stream = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }
    let decode = |e: SymphoniaError| Error::Decode(e.to_string());

    let mut reader = symphonia::default::get_probe()
        .probe(&hint, stream, Default::default(), Default::default())
        .map_err(decode)?;
    let track = reader.default_track(TrackType::Audio).ok_or(Error::NoAudioTrack)?;
    let track_id = track.id;
    let Some(CodecParameters::Audio(params)) = track.codec_params.clone() else {
        return Err(Error::NoAudioTrack);
    };

    let mut codecs = CodecRegistry::new();
    symphonia::default::register_enabled_codecs(&mut codecs);
    codecs.register_audio_decoder::<OpusDecoder>();
    let mut decoder = codecs.make_audio_decoder(&params, &Default::default()).map_err(decode)?;

    let mut mono = Vec::new();
    let mut interleaved: Vec<f32> = Vec::new();
    let mut rate = params.sample_rate.unwrap_or(0);
    loop {
        let packet = match reader.next_packet() {
            Ok(Some(packet)) => packet,
            Ok(None) => break,
            // Some containers report the end of the stream as an I/O error.
            Err(SymphoniaError::IoError(e)) if e.kind() == ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(decode(e)),
        };
        if packet.track_id != track_id {
            continue;
        }
        let buffer = match decoder.decode(&packet) {
            Ok(buffer) => buffer,
            // A damaged packet is skipped, the rest of the file is still usable.
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(e) => return Err(decode(e)),
        };
        rate = buffer.spec().rate();
        let channels = buffer.spec().channels().count();
        interleaved.clear();
        buffer.copy_to_vec_interleaved(&mut interleaved);
        mono.extend(downmix(&interleaved, channels));
    }
    if rate == 0 || mono.is_empty() {
        return Err(Error::NoAudioTrack);
    }
    Ok((mono, rate))
}

fn downmix(interleaved: &[f32], channels: usize) -> Vec<f32> {
    if channels <= 1 {
        return interleaved.to_vec();
    }
    interleaved
        .chunks_exact(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect()
}

fn to_16k(samples: &[f32], rate: u32) -> Vec<f32> {
    if rate == SAMPLE_RATE {
        return samples.to_vec();
    }
    // sherpa-onnx's own resampler: Quant-Core can use the very same one.
    match LinearResampler::create(rate as i32, SAMPLE_RATE as i32) {
        Some(resampler) => resampler.resample(samples, true),
        None => Vec::new(),
    }
}
