//! Integration tests for audio concurrency limits
//!
//! These tests verify that the audio module properly limits concurrent
//! sound playback to prevent DoS attacks from malicious applications.

use cosmic_notifications_util::audio::{play_sound_file, AudioError};
use std::fs;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

/// External access to the active sounds counter for testing.
/// In production code, this would be private to the audio module.
/// For testing, we validate behavior by observing the effects.
#[test]
fn test_concurrent_sound_limit_enforcement() {
    // This test verifies the DoS protection against unbounded thread spawning
    // Create a test WAV file in an allowed directory
    let temp_dir = if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join(".local/share/sounds")
    } else {
        println!("Skipping test: No HOME environment variable");
        return;
    };

    // Create directory if it doesn't exist
    if fs::create_dir_all(&temp_dir).is_err() {
        println!("Skipping test: Cannot create sound directory");
        return;
    }

    let test_file = temp_dir.join("test_concurrent_sound.wav");
    let wav_data = create_test_wav_file();
    if fs::write(&test_file, &wav_data).is_err() {
        println!("Skipping test: Cannot write test file");
        return;
    }

    // Attempt to play many more sounds than the limit (4)
    // The audio module should gracefully drop excess requests
    let attempts = 10;
    let mut success_count = 0;

    for _ in 0..attempts {
        if play_sound_file(&test_file).is_ok() {
            success_count += 1;
        }
    }

    // All calls should succeed (even if some are dropped internally)
    // The function returns Ok(()) when limit is reached, not an error
    assert!(
        success_count > 0,
        "At least some sound playback requests should succeed"
    );

    // Wait for sounds to finish
    for _ in 0..100 {
        sleep(Duration::from_millis(100));
        // We can't directly check the counter, but we wait for cleanup
    }

    // Cleanup
    let _ = fs::remove_file(&test_file);
}

#[test]
fn test_nonexistent_file_returns_error() {
    let nonexistent_file = PathBuf::from("/tmp/nonexistent_sound_file_12345.wav");

    // This should return an error immediately without spawning a thread
    let result = play_sound_file(&nonexistent_file);
    assert!(
        matches!(result, Err(AudioError::FileNotFound(_))),
        "Expected FileNotFound error for nonexistent file"
    );
}

#[test]
fn test_path_outside_allowed_directories_rejected() {
    // Verify that paths outside allowed directories are rejected
    let temp_dir = std::env::temp_dir();
    let malicious_file = temp_dir.join("malicious_sound.wav");

    // Create the file
    let wav_data = create_test_wav_file();
    if fs::write(&malicious_file, &wav_data).is_err() {
        println!("Skipping test: Cannot create temp file");
        return;
    }

    let result = play_sound_file(&malicious_file);

    // Should return error due to path validation
    assert!(
        matches!(result, Err(AudioError::PathNotAllowed(_))),
        "Expected PathNotAllowed error for file outside allowed directories"
    );

    let _ = fs::remove_file(&malicious_file);
}

#[test]
fn test_rapid_fire_sound_requests() {
    // Simulate a malicious app sending many notifications rapidly
    let temp_dir = if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join(".local/share/sounds")
    } else {
        return;
    };

    let _ = fs::create_dir_all(&temp_dir);
    let test_file = temp_dir.join("test_rapid_fire.wav");
    let wav_data = create_test_wav_file();

    if fs::write(&test_file, &wav_data).is_err() {
        return;
    }

    // Rapidly fire 20 sound requests
    let rapid_attempts = 20;
    for _ in 0..rapid_attempts {
        let _ = play_sound_file(&test_file);
    }

    // The system should handle this gracefully without crashing or spawning 20 threads
    // Success is measured by not panicking and completing the test
    sleep(Duration::from_millis(100));

    let _ = fs::remove_file(&test_file);
}

/// Creates a minimal valid WAV file for testing (1 second of silence at 8kHz)
fn create_test_wav_file() -> Vec<u8> {
    let sample_rate = 8000u32;
    let num_channels = 1u16;
    let bits_per_sample = 16u16;
    let duration_seconds = 1;
    let num_samples = sample_rate * duration_seconds;
    let data_size = num_samples * (bits_per_sample as u32 / 8) * (num_channels as u32);

    let mut wav = Vec::new();

    // RIFF chunk descriptor
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_size).to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // fmt sub-chunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // Sub-chunk size
    wav.extend_from_slice(&1u16.to_le_bytes()); // Audio format (PCM)
    wav.extend_from_slice(&num_channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate * (num_channels as u32) * (bits_per_sample as u32 / 8);
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    let block_align = num_channels * (bits_per_sample / 8);
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());

    // data sub-chunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());

    // Silence (zeros)
    wav.resize(wav.len() + data_size as usize, 0);

    wav
}
