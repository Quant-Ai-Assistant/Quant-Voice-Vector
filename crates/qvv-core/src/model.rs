//! The pinned models. Changing anything here invalidates every existing profile.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::error::{Error, Result};

pub const SAMPLE_RATE: u32 = 16_000;

/// Speaker embedding model, written into every profile as `model.id`.
pub const EMBEDDING_MODEL_ID: &str = "3dspeaker_speech_campplus_sv_zh_en_16k-common_advanced";
pub const EMBEDDING_MODEL_FILE: &str =
    "3dspeaker_speech_campplus_sv_zh_en_16k-common_advanced.onnx";
pub const EMBEDDING_MODEL_SHA256: &str =
    "aa3cfc16963a10586a9393f5035d6d6b57e98d358b347f80c2a30bf4f00ceba2";
pub const EMBEDDING_DIM: usize = 192;

pub const VAD_MODEL_FILE: &str = "silero_vad.onnx";
pub const VAD_MODEL_SHA256: &str =
    "9e2449e1087496d8d4caba907f23e0bd3f78d91fa552479bb9c23ac09cbb1fd6";

pub fn sha256_file(path: &Path) -> Result<String> {
    let io = |source| Error::Io { path: path.to_path_buf(), source };
    let mut file = std::fs::File::open(path).map_err(io)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(io)?;
    Ok(hasher.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// Returns the path of `file` inside `dir` after checking its checksum.
pub(crate) fn verified(dir: &Path, file: &str, expected: &'static str) -> Result<PathBuf> {
    let path = dir.join(file);
    let actual = sha256_file(&path)?;
    if actual != expected {
        return Err(Error::ModelChecksum { path, expected, actual });
    }
    Ok(path)
}
