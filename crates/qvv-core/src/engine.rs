//! Speech detection and speaker embeddings.

use std::path::Path;

use sherpa_onnx::{
    SileroVadModelConfig, SpeakerEmbeddingExtractor, SpeakerEmbeddingExtractorConfig,
    VadModelConfig, VoiceActivityDetector,
};

use crate::error::{Error, Result};
use crate::model::{self, SAMPLE_RATE};

/// Length of one piece of speech that gets its own vector.
pub const CHUNK_SECONDS: f32 = 8.0;
/// A trailing piece shorter than this is dropped.
pub const MIN_CHUNK_SECONDS: f32 = 3.0;

const VAD_WINDOW: usize = 512;

/// A piece of pure speech (silence removed), mono 16 kHz.
pub struct SpeechChunk {
    pub samples: Vec<f32>,
}

impl SpeechChunk {
    pub fn seconds(&self) -> f32 {
        self.samples.len() as f32 / SAMPLE_RATE as f32
    }
}

pub struct Engine {
    extractor: SpeakerEmbeddingExtractor,
    vad_config: VadModelConfig,
}

impl Engine {
    /// Loads the pinned models from `models_dir`, refusing files with a wrong checksum.
    pub fn load(models_dir: &Path) -> Result<Self> {
        let embedding = model::verified(
            models_dir,
            model::EMBEDDING_MODEL_FILE,
            model::EMBEDDING_MODEL_SHA256,
        )?;
        let vad = model::verified(models_dir, model::VAD_MODEL_FILE, model::VAD_MODEL_SHA256)?;

        let extractor = SpeakerEmbeddingExtractor::create(&SpeakerEmbeddingExtractorConfig {
            model: Some(embedding.to_string_lossy().into_owned()),
            num_threads: 1,
            debug: false,
            provider: Some("cpu".into()),
        })
        .ok_or(Error::Engine("speaker embedding model"))?;
        if extractor.dim() as usize != model::EMBEDDING_DIM {
            return Err(Error::Engine(
                "speaker embedding model: unexpected dimension",
            ));
        }

        let vad_config = VadModelConfig {
            silero_vad: SileroVadModelConfig {
                model: Some(vad.to_string_lossy().into_owned()),
                threshold: 0.5,
                min_silence_duration: 0.5,
                min_speech_duration: 0.25,
                window_size: VAD_WINDOW as i32,
                max_speech_duration: 20.0,
            },
            sample_rate: SAMPLE_RATE as i32,
            num_threads: 1,
            provider: Some("cpu".into()),
            debug: false,
            ..Default::default()
        };
        // Fail early if the VAD model cannot be loaded.
        VoiceActivityDetector::create(&vad_config, 1.0).ok_or(Error::Engine("VAD model"))?;

        Ok(Self {
            extractor,
            vad_config,
        })
    }

    /// Removes silence and noise from mono 16 kHz audio, returning only speech.
    pub fn speech_only(&self, samples: &[f32]) -> Result<Vec<f32>> {
        let vad = VoiceActivityDetector::create(&self.vad_config, 60.0)
            .ok_or(Error::Engine("VAD model"))?;
        let mut speech = Vec::new();
        let mut drain = |vad: &VoiceActivityDetector| {
            while let Some(segment) = vad.front() {
                speech.extend_from_slice(segment.samples());
                vad.pop();
            }
        };
        for window in samples.as_chunks::<VAD_WINDOW>().0 {
            vad.accept_waveform(window);
            drain(&vad);
        }
        vad.flush();
        drain(&vad);
        Ok(speech)
    }

    /// Cuts mono 16 kHz audio into speech chunks ready for [`Engine::embed`].
    pub fn chunks(&self, samples: &[f32]) -> Result<Vec<SpeechChunk>> {
        let speech = self.speech_only(samples)?;
        let chunk_len = (CHUNK_SECONDS * SAMPLE_RATE as f32) as usize;
        let min_len = (MIN_CHUNK_SECONDS * SAMPLE_RATE as f32) as usize;
        Ok(speech
            .chunks(chunk_len)
            .filter(|c| c.len() >= min_len)
            .map(|c| SpeechChunk {
                samples: c.to_vec(),
            })
            .collect())
    }

    /// Computes the L2-normalized speaker vector of mono 16 kHz speech.
    pub fn embed(&self, samples: &[f32]) -> Result<Vec<f32>> {
        let stream = self
            .extractor
            .create_stream()
            .ok_or(Error::Engine("embedding stream"))?;
        stream.accept_waveform(SAMPLE_RATE as i32, samples);
        // Quant-Core must call inputFinished() too: without it the last frames are not
        // flushed and the vector differs (cosine ~0.999 instead of 1).
        stream.input_finished();
        if !self.extractor.is_ready(&stream) {
            return Err(Error::NotEnoughSpeech { have: 0, need: 1 });
        }
        let mut vector = self
            .extractor
            .compute(&stream)
            .ok_or(Error::Engine("embedding computation"))?;
        normalize(&mut vector);
        Ok(vector)
    }
}

pub(crate) fn normalize(vector: &mut [f32]) {
    let norm = vector.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > 0.0 {
        vector.iter_mut().for_each(|v| *v /= norm);
    }
}

/// Cosine similarity of two L2-normalized vectors.
pub fn similarity(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
