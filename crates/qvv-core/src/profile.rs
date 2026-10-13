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
            return Err(Error::NotEnoughSpeech {
                have: accepted.len(),
                need: MIN_EMBEDDINGS,
            });
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A unit vector pointing mostly along `axis`, slightly tilted by `tilt`.
    fn voice(axis: usize, tilt: usize) -> Sample {
        let mut vector = vec![0.0; model::EMBEDDING_DIM];
        vector[axis] = 1.0;
        vector[10 + tilt] = 0.3;
        normalize(&mut vector);
        Sample {
            vector,
            speech_seconds: 8.0,
            source: Source::Microphone,
        }
    }

    fn builder(samples: impl IntoIterator<Item = Sample>) -> ProfileBuilder {
        let mut builder = ProfileBuilder::default();
        samples.into_iter().for_each(|sample| builder.add(sample));
        builder
    }

    #[test]
    fn too_few_segments_do_not_make_a_profile() {
        let result = builder((0..4).map(|i| voice(0, i))).build("Alexey");
        assert!(matches!(
            result,
            Err(Error::NotEnoughSpeech {
                have: 4,
                need: MIN_EMBEDDINGS
            })
        ));
    }

    #[test]
    fn segments_of_another_voice_are_dropped() {
        let own = (0..6).map(|i| voice(0, i));
        let foreign = (0..2).map(|i| voice(1, i));
        let builder = builder(own.chain(foreign));
        assert_eq!(builder.rejected(), 2);
        assert_eq!(builder.speech_seconds(), 48.0);
    }

    #[test]
    fn profile_keeps_at_most_the_maximum_number_of_vectors() {
        let profile = builder((0..30).map(|i| voice(0, i)))
            .build("Alexey")
            .unwrap();
        assert_eq!(profile.embeddings.len(), MAX_EMBEDDINGS);
    }

    #[test]
    fn saved_file_reads_back_and_matches_its_own_voice() {
        let profile = builder((0..6).map(|i| voice(0, i)))
            .build("Alexey")
            .unwrap();
        let restored = VoiceprintFile::from_json(&profile.to_json()).unwrap();

        assert_eq!(restored.profile.id, profile.profile.id);
        assert_eq!(restored.centroid.len(), model::EMBEDDING_DIM);
        let norm = restored.centroid.iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5);
        assert!(restored.compare(&voice(0, 0).vector).centroid > 0.9);
        assert!(restored.compare(&voice(1, 0).vector).centroid < 0.1);
    }

    #[test]
    fn file_made_by_another_model_is_refused() {
        let mut profile = builder((0..6).map(|i| voice(0, i)))
            .build("Alexey")
            .unwrap();
        profile.model.sha256 = "0".repeat(64);
        assert!(matches!(
            VoiceprintFile::from_json(&profile.to_json()),
            Err(Error::InvalidFile(_))
        ));
    }
}
