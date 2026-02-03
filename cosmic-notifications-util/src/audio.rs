//! Audio playback for notification sounds
//!
//! Supports playing sound files and XDG sound theme sounds.

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::thread;

use rodio::{Decoder, OutputStream, Sink};
use tracing::{debug, error};

/// Play a sound file
///
/// Supports common audio formats: WAV, OGG, MP3, FLAC
/// Sound is played in a background thread to avoid blocking.
pub fn play_sound_file(path: &Path) -> Result<(), AudioError> {
    if !path.exists() {
        return Err(AudioError::FileNotFound(path.to_path_buf()));
    }

    let path = path.to_path_buf();

    // Spawn a thread to play the sound so we don't block
    thread::spawn(move || {
        if let Err(e) = play_sound_file_blocking(&path) {
            error!("Failed to play sound file {:?}: {}", path, e);
        }
    });

    Ok(())
}

/// Play a sound file (blocking)
fn play_sound_file_blocking(path: &Path) -> Result<(), AudioError> {
    // Create a new output stream for this playback
    let (_stream, handle) = OutputStream::try_default()
        .map_err(|_| AudioError::NoAudioDevice)?;

    let file = File::open(path).map_err(|e| AudioError::IoError(e.to_string()))?;
    let reader = BufReader::new(file);

    let source = Decoder::new(reader).map_err(|e| AudioError::DecodeError(e.to_string()))?;

    let sink = Sink::try_new(&handle).map_err(|e| AudioError::PlaybackError(e.to_string()))?;
    sink.append(source);
    sink.sleep_until_end();

    Ok(())
}

/// Play a sound from the XDG sound theme
///
/// Looks up the sound name in the freedesktop.org sound theme.
/// Common sound names: "message-new-instant", "bell", "dialog-warning"
pub fn play_sound_name(name: &str) -> Result<(), AudioError> {
    // Look up the sound file in XDG sound theme directories
    let sound_path = find_sound_theme_file(name)?;
    play_sound_file(&sound_path)
}

/// Find a sound file from the XDG sound theme
fn find_sound_theme_file(name: &str) -> Result<PathBuf, AudioError> {
    // XDG sound theme directories
    let search_dirs = get_sound_theme_dirs();

    // Common extensions for sound files
    let extensions = ["oga", "ogg", "wav", "mp3"];

    for dir in &search_dirs {
        for ext in &extensions {
            let path = dir.join(format!("{}.{}", name, ext));
            if path.exists() {
                debug!("Found sound theme file: {:?}", path);
                return Ok(path);
            }

            // Also check stereo subdirectory
            let stereo_path = dir.join("stereo").join(format!("{}.{}", name, ext));
            if stereo_path.exists() {
                debug!("Found sound theme file: {:?}", stereo_path);
                return Ok(stereo_path);
            }
        }
    }

    Err(AudioError::SoundNotFound(name.to_string()))
}

/// Get XDG sound theme directories
fn get_sound_theme_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // User sound themes
    if let Some(data_home) = std::env::var_os("XDG_DATA_HOME") {
        dirs.push(PathBuf::from(&data_home).join("sounds/freedesktop/stereo"));
        dirs.push(PathBuf::from(data_home).join("sounds"));
    } else if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(&home).join(".local/share/sounds/freedesktop/stereo"));
        dirs.push(PathBuf::from(home).join(".local/share/sounds"));
    }

    // System sound themes
    let system_dirs = [
        "/usr/share/sounds/freedesktop/stereo",
        "/usr/share/sounds/freedesktop",
        "/usr/share/sounds",
        "/usr/local/share/sounds/freedesktop/stereo",
        "/usr/local/share/sounds/freedesktop",
        "/usr/local/share/sounds",
    ];

    for dir in &system_dirs {
        dirs.push(PathBuf::from(dir));
    }

    dirs
}

/// Audio playback errors
#[derive(Debug, Clone)]
pub enum AudioError {
    /// No audio output device available
    NoAudioDevice,
    /// Sound file not found
    FileNotFound(PathBuf),
    /// Sound theme entry not found
    SoundNotFound(String),
    /// IO error reading file
    IoError(String),
    /// Error decoding audio file
    DecodeError(String),
    /// Error during playback
    PlaybackError(String),
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioError::NoAudioDevice => write!(f, "No audio output device available"),
            AudioError::FileNotFound(path) => write!(f, "Sound file not found: {:?}", path),
            AudioError::SoundNotFound(name) => {
                write!(f, "Sound '{}' not found in theme", name)
            }
            AudioError::IoError(e) => write!(f, "IO error: {}", e),
            AudioError::DecodeError(e) => write!(f, "Audio decode error: {}", e),
            AudioError::PlaybackError(e) => write!(f, "Playback error: {}", e),
        }
    }
}

impl std::error::Error for AudioError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_sound_theme_dirs() {
        let dirs = get_sound_theme_dirs();
        assert!(!dirs.is_empty());
    }

    #[test]
    fn test_audio_error_display() {
        let err = AudioError::NoAudioDevice;
        assert!(!err.to_string().is_empty());
    }
}
