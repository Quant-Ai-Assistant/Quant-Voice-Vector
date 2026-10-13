use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cannot read {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot decode audio: {0}")]
    Decode(String),
    #[error("no audio track in the file")]
    NoAudioTrack,
    #[error("model file {path} has sha256 {actual}, expected {expected}")]
    ModelChecksum {
        path: PathBuf,
        expected: &'static str,
        actual: String,
    },
    #[error("cannot initialise {0}")]
    Engine(&'static str),
    #[error("not enough speech: {have} usable segments, need at least {need}")]
    NotEnoughSpeech { have: usize, need: usize },
    #[error("invalid voiceprint file: {0}")]
    InvalidFile(String),
}
