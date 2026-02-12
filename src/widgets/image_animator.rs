#![allow(unexpected_cfgs, dead_code)]

use std::time::Instant;

#[cfg(feature = "image")]
use cosmic_ext_notifications_util::AnimatedImage;

/// Controls playback of an animated image
pub struct ImageAnimator {
    #[cfg(feature = "image")]
    animation: Option<AnimatedImage>,
    start_time: Instant,
    paused: bool,
}

impl ImageAnimator {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "image")]
            animation: None,
            start_time: Instant::now(),
            paused: false,
        }
    }

    #[cfg(feature = "image")]
    pub fn with_animation(animation: AnimatedImage) -> Self {
        Self {
            animation: Some(animation),
            start_time: Instant::now(),
            paused: false,
        }
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u32 {
        if self.paused {
            0
        } else {
            self.start_time.elapsed().as_millis() as u32
        }
    }

    /// Check if animator is playing
    pub fn is_playing(&self) -> bool {
        !self.paused
    }

    /// Pause animation
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume animation
    pub fn resume(&mut self) {
        self.paused = false;
        self.start_time = Instant::now();
    }

    /// Reset animation to start
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
    }

    #[cfg(feature = "image")]
    pub fn is_animated(&self) -> bool {
        self.animation.as_ref().map(|a| a.is_animated()).unwrap_or(false)
    }

    #[cfg(not(feature = "image"))]
    pub fn is_animated(&self) -> bool {
        false
    }
}

impl Default for ImageAnimator {
    fn default() -> Self {
        Self::new()
    }
}
