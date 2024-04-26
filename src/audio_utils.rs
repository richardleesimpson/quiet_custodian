use std::{ ffi::OsStr, path::Path };
use crate::{ debug, error::AppError };

pub enum AudioFormat {
    MP3,
    MP4,
}

impl AudioFormat {
    fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "mp3" => Some(Self::MP3),
            "mp4" => Some(Self::MP4),
            _ => None,
        }
    }

    fn is_supported(ext: &str) -> bool {
        Self::from_extension(ext).is_some()
    }
}

pub fn is_supported_audio_format(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .map(|ext| AudioFormat::is_supported(ext))
        .unwrap_or(false)
}

pub fn transcribe_audio(input: &Path, output: &Path) -> Result<(), AppError> {
    // TODO: Actual call to audio processing library
    debug!("Transcribing file from {:?} to {:?}", input, output);
    Ok(())
}
