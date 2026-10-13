//! The `.qvoice.json` file, version 1. The format is a contract with Quant-Core:
//! see `schema/qvoice.v1.schema.json`.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::engine::{normalize, similarity};
use crate::error::{Error, Result};
use crate::model;

pub const FORMAT: &str = "quant-voiceprint";
pub const VERSION: u32 = 1;
pub const MIN_EMBEDDINGS: usize = 5;
pub const MAX_EMBEDDINGS: usize = 20;

/// A segment whose median similarity to the other segments is below this is treated as
/// noise or another voice and dropped. The median keeps the rule working while the
/// foreign voice is a minority of the segments. Starting value, tune on real data.
pub const OUTLIER_MIN_SIMILARITY: f32 = 0.4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Microphone,
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Sample {
    pub vector: Vec<f32>,
    pub speech_seconds: f32,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileInfo {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelInfo {
    pub id: String,
    pub sha256: String,
    pub dim: usize,
    pub sample_rate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VoiceprintFile {
    pub format: String,
    pub version: u32,
    pub profile: ProfileInfo,
    pub model: ModelInfo,
    pub normalized: bool,
    pub centroid: Vec<f32>,
    pub embeddings: Vec<Sample>,
}

/// How close a new utterance is to a profile.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Match {
    pub centroid: f32,
    pub best_segment: f32,
}

/// Collects segment vectors until there are enough of them for a profile.
#[derive(Default)]
pub struct ProfileBuilder {
    samples: Vec<Sample>,
}

impl ProfileBuilder {
    pub fn add(&mut self, sample: Sample) {
        self.samples.push(sample);
    }

    pub fn clear(&mut self) {
        self.samples.clear();
    }

    /// Samples that survive outlier filtering.
    pub fn accepted(&self) -> Vec<&Sample> {
        if self.samples.len() < 3 {
            return self.samples.iter().collect();
        }
        self.samples
            .iter()
            .enumerate()
            .filter(|(i, sample)| {
                let mut others: Vec<f32> = self
                    .samples
                    .iter()
                    .enumerate()
                    .filter(|(j, _)| j != i)
                    .map(|(_, other)| similarity(&sample.vector, &other.vector))
                    .collect();
                others.sort_by(f32::total_cmp);
                others[others.len() / 2] >= OUTLIER_MIN_SIMILARITY
            })
            .map(|(_, sample)| sample)
            .collect()
    }

    pub fn rejected(&self) -> usize {
        self.samples.len() - self.accepted().len()
    }

    pub fn speech_seconds(&self) -> f32 {
        self.accepted().iter().map(|s| s.speech_seconds).sum()
    }

    pub fn build(&self, name: &str) -> Result<VoiceprintFile> {
        let accepted = self.accepted();
        if accepted.len() < MIN_EMBEDDINGS {
            return Err(Error::NotEnoughSpeech { have: accepted.len(), need: MIN_EMBEDDINGS });
        }
        // Too many segments: keep an evenly spread subset so every recording is represented.
        let kept: Vec<Sample> = if accepted.len() > MAX_EMBEDDINGS {
            (0..MAX_EMBEDDINGS)
                .map(|i| accepted[i * accepted.len() / MAX_EMBEDDINGS].clone())
                .collect()
        } else {
            accepted.into_iter().cloned().collect()
        };

        let mut created_at = OffsetDateTime::now_utc();
        created_at = created_at.replace_nanosecond(0).unwrap_or(created_at);
        Ok(VoiceprintFile {
            format: FORMAT.into(),
            version: VERSION,
            profile: ProfileInfo {
                id: uuid::Uuid::new_v4().to_string(),
                name: name.to_string(),
                created_at: created_at.format(&Rfc3339).unwrap_or_default(),
            },
            model: ModelInfo {
                id: model::EMBEDDING_MODEL_ID.into(),
                sha256: model::EMBEDDING_MODEL_SHA256.into(),
                dim: model::EMBEDDING_DIM,
                sample_rate: model::SAMPLE_RATE,
            },
            normalized: true,
            centroid: centroid(kept.iter().map(|s| &s.vector)),
            embeddings: kept,
        })
    }
}

impl VoiceprintFile {
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("voiceprint is always serializable")
    }

    /// Parses a file and checks it was made by the model this build uses.
    pub fn from_json(json: &str) -> Result<Self> {
        let file: Self =
            serde_json::from_str(json).map_err(|e| Error::InvalidFile(e.to_string()))?;
        let invalid = |reason: &str| Err(Error::InvalidFile(reason.to_string()));
        if file.format != FORMAT || file.version != VERSION {
            return invalid("unknown format or version");
        }
        if file.model.id != model::EMBEDDING_MODEL_ID
            || file.model.sha256 != model::EMBEDDING_MODEL_SHA256
        {
            return invalid("made by a different model");
        }
        let dim = file.model.dim;
        if dim != model::EMBEDDING_DIM
            || file.centroid.len() != dim
            || file.embeddings.iter().any(|s| s.vector.len() != dim)
        {
            return invalid("vector length does not match model.dim");
        }
        Ok(file)
    }

    pub fn compare(&self, vector: &[f32]) -> Match {
        Match {
            centroid: similarity(&self.centroid, vector),
            best_segment: self
                .embeddings
                .iter()
                .map(|s| similarity(&s.vector, vector))
                .fold(f32::MIN, f32::max),
        }
    }
}

fn centroid<'a>(vectors: impl Iterator<Item = &'a Vec<f32>>) -> Vec<f32> {
    let mut sum: Vec<f32> = Vec::new();
    for vector in vectors {
        if sum.is_empty() {
            sum = vec![0.0; vector.len()];
        }
        sum.iter_mut().zip(vector).for_each(|(s, v)| *s += v);
    }
    normalize(&mut sum);
    sum
}
