//! Audio playback for notification sounds
//!
//! Supports playing sound files and XDG sound theme sounds.
//!
//! # Security
//!
//! Sound file paths are validated to prevent path traversal attacks.
//! Only files in allowed system and user sound directories can be played:
//! - `/usr/share/sounds/**`
//! - `/usr/local/share/sounds/**`
//! - `$XDG_DATA_HOME/sounds/**` (or `$HOME/.local/share/sounds/**`)

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use rodio::{Decoder, OutputStream, Sink};
use tracing::{debug, error, warn};

/// Maximum number of concurrent sounds that can be played simultaneously.
/// This prevents DoS attacks from malicious apps spawning unlimited audio threads.
const MAX_CONCURRENT_SOUNDS: usize = 4;

/// Tracks the current number of active sound playback threads.
static ACTIVE_SOUNDS: AtomicUsize = AtomicUsize::new(0);

/// Check if a sound file path is in an allowed directory.
///
/// This prevents path traversal attacks where a malicious notification
/// could try to access arbitrary files like `/etc/passwd` or
/// `/usr/share/sounds/../../etc/shadow`.
///
/// # Allowed directories
///
/// - `/usr/share/sounds/**`
/// - `/usr/local/share/sounds/**`
/// - `$XDG_DATA_HOME/sounds/**`
/// - `$HOME/.local/share/sounds/**`
///
/// # Security notes
///
/// - Uses canonicalization to resolve symlinks and `..` components
/// - Rejects paths that cannot be canonicalized (e.g., broken symlinks)
/// - OWASP reference: Path Traversal (CWE-22)
fn is_allowed_sound_path(path: &Path) -> bool {
    // Canonicalize to resolve symlinks and .. components
    // This prevents attacks like /usr/share/sounds/../../etc/passwd
    let canonical = match path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to canonicalize sound path {:?}: {}", path, e);
            return false;
        }
    };

    // System sound directories - canonicalize for robust comparison
    let system_dirs = ["/usr/share/sounds", "/usr/local/share/sounds"];
    for dir in &system_dirs {
        if let Ok(dir_canonical) = Path::new(dir).canonicalize() {
            if canonical.starts_with(&dir_canonical) {
                return true;
            }
        }
    }

    // User sound directories - check XDG_DATA_HOME first
    if let Some(data_home) = std::env::var_os("XDG_DATA_HOME") {
        let user_sounds = PathBuf::from(&data_home).join("sounds");
        if let Ok(user_canonical) = user_sounds.canonicalize() {
            if canonical.starts_with(&user_canonical) {
                return true;
            }
        }
    }

    // Fallback to $HOME/.local/share/sounds
    if let Some(home) = std::env::var_os("HOME") {
        let user_sounds = PathBuf::from(&home).join(".local/share/sounds");
        if let Ok(user_canonical) = user_sounds.canonicalize() {
            if canonical.starts_with(&user_canonical) {
                return true;
            }
        }
    }

    warn!(
        "Sound file path {:?} (canonical: {:?}) is not in an allowed directory",
        path, canonical
    );
    false
}

/// Play a sound file
///
/// Supports common audio formats: WAV, OGG, MP3, FLAC
/// Sound is played in a background thread to avoid blocking.
///
/// To prevent resource exhaustion from malicious apps, this function limits
/// the number of concurrent sound playbacks to [`MAX_CONCURRENT_SOUNDS`].
/// If the limit is reached, the sound request is silently dropped.
pub fn play_sound_file(path: &Path) -> Result<(), AudioError> {
    if !path.exists() {
        return Err(AudioError::FileNotFound(path.to_path_buf()));
    }

    // Security: Validate path is in an allowed sound directory
    // This prevents path traversal attacks (CWE-22)
    //
    // Note: There is a small TOCTOU (Time-of-Check-Time-of-Use) window between
    // validation and file open. For sound files this is acceptable risk because:
    // 1. Sound directories are typically system-owned with limited write access
    // 2. Attack requires local file system access
    // 3. Worst case is playing wrong sound, not code execution
    // 4. The audio decoder (rodio) is memory-safe Rust
    if !is_allowed_sound_path(path) {
        return Err(AudioError::PathNotAllowed(path.to_path_buf()));
    }

    // Atomically check and increment the active sound counter
    // Using compare_exchange prevents race condition where multiple threads
    // could pass the limit check simultaneously
    loop {
        let current = ACTIVE_SOUNDS.load(Ordering::SeqCst);
        if current >= MAX_CONCURRENT_SOUNDS {
            warn!(
                "Maximum concurrent sounds ({}) reached, dropping sound request for {:?}",
                MAX_CONCURRENT_SOUNDS, path
            );
            return Ok(());
        }

        // Try to atomically increment if counter hasn't changed
        match ACTIVE_SOUNDS.compare_exchange(
            current,
            current + 1,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(_) => break, // Successfully incremented, proceed to spawn thread
            Err(_) => continue, // Counter changed, retry the check
        }
    }

    let path = path.to_path_buf();

    // Spawn a thread to play the sound so we don't block
    let spawn_result = thread::Builder::new()
        .name("audio-playback".into())
        .spawn(move || {
            let result = play_sound_file_blocking(&path);

            // Always decrement the counter when done, even on error
            ACTIVE_SOUNDS.fetch_sub(1, Ordering::SeqCst);

            if let Err(e) = result {
                error!("Failed to play sound file {:?}: {}", path, e);
            }
        });

    // Handle spawn failure - must decrement counter if thread creation failed
    if let Err(e) = spawn_result {
        ACTIVE_SOUNDS.fetch_sub(1, Ordering::SeqCst);
        warn!("Failed to spawn audio thread: {}", e);
    }

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
    /// Sound file path is not in an allowed directory (security violation)
    PathNotAllowed(PathBuf),
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
            AudioError::PathNotAllowed(path) => {
                write!(f, "Sound file path not in allowed directory: {:?}", path)
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

    #[test]
    fn test_path_not_allowed_error_display() {
        let err = AudioError::PathNotAllowed(PathBuf::from("/etc/passwd"));
        let msg = err.to_string();
        assert!(msg.contains("not in allowed directory"));
        assert!(msg.contains("/etc/passwd"));
    }

    // Security tests for path traversal prevention (CWE-22)
    mod security {
        use super::*;

        #[test]
        fn test_rejects_etc_passwd() {
            // Direct path to sensitive system file
            let path = Path::new("/etc/passwd");
            assert!(
                !is_allowed_sound_path(path),
                "/etc/passwd should be rejected"
            );
        }

        #[test]
        fn test_rejects_path_traversal_from_sounds_dir() {
            // Path traversal attack: start in allowed dir, escape with ..
            let path = Path::new("/usr/share/sounds/../../etc/passwd");
            assert!(
                !is_allowed_sound_path(path),
                "Path traversal via .. should be rejected"
            );
        }

        #[test]
        fn test_rejects_arbitrary_home_file() {
            // Attempting to access arbitrary file in home directory
            if let Some(home) = std::env::var_os("HOME") {
                let path = PathBuf::from(&home).join(".bashrc");
                assert!(
                    !is_allowed_sound_path(&path),
                    "~/.bashrc should be rejected"
                );
            }
        }

        #[test]
        fn test_rejects_tmp_file() {
            // Temporary files should not be playable
            let path = Path::new("/tmp/malicious.wav");
            assert!(
                !is_allowed_sound_path(path),
                "/tmp files should be rejected"
            );
        }

        #[test]
        fn test_rejects_var_file() {
            let path = Path::new("/var/log/messages");
            assert!(
                !is_allowed_sound_path(path),
                "/var files should be rejected"
            );
        }

        #[test]
        fn test_rejects_root_file() {
            let path = Path::new("/root/.ssh/id_rsa");
            assert!(
                !is_allowed_sound_path(path),
                "/root files should be rejected"
            );
        }

        #[test]
        fn test_rejects_proc_file() {
            // /proc filesystem should never be accessible
            let path = Path::new("/proc/self/environ");
            assert!(
                !is_allowed_sound_path(path),
                "/proc files should be rejected"
            );
        }

        #[test]
        fn test_rejects_dev_file() {
            // Device files should never be accessible
            let path = Path::new("/dev/random");
            assert!(
                !is_allowed_sound_path(path),
                "/dev files should be rejected"
            );
        }

        #[test]
        fn test_allows_system_sounds_dir() {
            // Valid system sound path (note: file doesn't need to exist for path check)
            // We test the path pattern, actual file existence is checked separately
            let path = Path::new("/usr/share/sounds/freedesktop/stereo/message.oga");

            // This test only passes if the file actually exists (due to canonicalize)
            // So we check the inverse: if it exists, it should be allowed
            if path.exists() {
                assert!(
                    is_allowed_sound_path(path),
                    "Valid system sound file should be allowed"
                );
            }
        }

        #[test]
        fn test_allows_usr_local_sounds_dir() {
            let path = Path::new("/usr/local/share/sounds/custom/alert.wav");

            // Same as above - only test if file exists
            if path.exists() {
                assert!(
                    is_allowed_sound_path(path),
                    "Valid /usr/local sound file should be allowed"
                );
            }
        }

        #[test]
        fn test_rejects_double_encoded_traversal() {
            // Some path traversal attempts use URL encoding or double dots
            // Rust's canonicalize handles these, but let's verify
            let path = Path::new("/usr/share/sounds/../sounds/../../etc/passwd");
            assert!(
                !is_allowed_sound_path(path),
                "Double traversal should be rejected"
            );
        }

        #[test]
        fn test_play_sound_file_returns_path_not_allowed() {
            // Test that play_sound_file returns the correct error type
            // We need a file that exists but is not allowed
            let path = Path::new("/etc/passwd");
            if path.exists() {
                let result = play_sound_file(path);
                assert!(result.is_err());
                match result.unwrap_err() {
                    AudioError::PathNotAllowed(p) => {
                        assert_eq!(p, PathBuf::from("/etc/passwd"));
                    }
                    other => panic!("Expected PathNotAllowed, got {:?}", other),
                }
            }
        }

        #[test]
        fn test_play_sound_file_file_not_found_before_path_check() {
            // Non-existent file should return FileNotFound, not PathNotAllowed
            let path = Path::new("/nonexistent/path/to/sound.wav");
            let result = play_sound_file(path);
            assert!(result.is_err());
            match result.unwrap_err() {
                AudioError::FileNotFound(_) => {}
                other => panic!("Expected FileNotFound, got {:?}", other),
            }
        }
    }
}
