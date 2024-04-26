use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("File system error: {0}")] FileSystemError(#[from] io::Error),

    #[error("Metadata retrieval failed")] MetadataError(#[source] io::Error),

    #[error("Failed to process audio data")]
    AudioProcessingError,

    #[error("Invalid file format")]
    UnsupportedFormat,

    #[error("Notification error: {0}")] NotifyError(#[from] notify::Error),
}
