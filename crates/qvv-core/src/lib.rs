//! Core of Quant-Voice-Vector: audio in, `.qvoice.json` out.
//!
//! The same model and the same preprocessing must be used by Quant-Core when it
//! computes a vector for an incoming utterance, otherwise vectors are not comparable.
//! Everything that affects the vector lives in [`engine`] and [`model`].

pub mod audio;
pub mod engine;
pub mod error;
pub mod model;
pub mod profile;

pub use engine::{Engine, SpeechChunk};
pub use error::{Error, Result};
pub use profile::{Match, ProfileBuilder, Sample, Source, VoiceprintFile};
