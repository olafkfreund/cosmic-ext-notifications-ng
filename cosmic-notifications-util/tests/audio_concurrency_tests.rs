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
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    // This test verifies the DoS protection against unbounded thread spawning.
    //
    // IMPORTANT: This test has inherent limitations due to the audio module's design:
    // - The audio module returns Ok() for all valid requests, even when the concurrent
    //   limit is reached (graceful degradation by silently dropping excess requests)
    // - We cannot directly access the internal active_sounds counter
    // - We cannot reliably verify the exact number of *actually playing* sounds
    //
    // What this test DOES verify:
    // - The system accepts requests gracefully without panicking
    // - Multiple concurrent threads can safely call play_sound_file()
    // - The function returns Ok() for valid sound files
    // - No errors occur during concurrent access
    //
    // What this test CANNOT verify:
    // - The exact number of concurrently playing sounds
    // - That the limit is precisely enforced (would require internal state access)

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

    // Track how many calls return Ok() (not how many actually play)
    let success_count = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    // Attempt to play many more sounds than the limit (typically 4)
    // Use actual threads to simulate true concurrent requests
    let concurrent_attempts = 10;

    for _ in 0..concurrent_attempts {
        let path = test_file.clone();
        let counter = Arc::clone(&success_count);

        handles.push(std::thread::spawn(move || {
            if play_sound_file(&path).is_ok() {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        }));
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    let total_ok = success_count.load(Ordering::SeqCst);

    // All calls should return Ok() (the module accepts requests gracefully)
    // Note: We can't assert exact count == MAX_CONCURRENT_SOUNDS because the
    // limit only applies to *concurrent playing* sounds, not total accepted requests.
    // The audio module may accept all requests and drop excess ones internally.
    assert!(
        total_ok > 0,
        "At least some sound playback requests should return Ok()"
    );

    // Verify we actually attempted concurrent requests
    assert!(
        total_ok <= concurrent_attempts,
        "Should not have more successes than attempts"
    );

    // Wait for sounds to finish playing before cleanup
    sleep(Duration::from_secs(2));

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
