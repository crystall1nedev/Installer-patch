pub mod patch;
pub mod paths;
pub mod update;

use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("Windows moved directory unexpectedly")]
    ErrWindowsMovedDirectory,
    #[error("Location already patched")]
    ErrLocationPatched,
    #[error("Location already unpatched")]
    ErrLocationNotPatched,
    #[error("Location invalid")]
    ErrLocationInvalid,
    #[error("Network error")]
    ErrNetwork(#[from] reqwest::Error),
    #[error("Serde json error: {0}")]
    ErrSerdeJson(#[from] serde_json::Error),
    #[error("Fs error: {0}")]
    ErrIo(#[from] std::io::Error),
}
