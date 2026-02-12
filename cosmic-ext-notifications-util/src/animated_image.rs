use std::time::Duration;

/// Maximum frames to store per animation (memory protection)
pub const MAX_FRAMES: usize = 100;

/// Maximum animation duration
pub const MAX_ANIMATION_DURATION: Duration = Duration::from_secs(30);

/// Single frame of an animation
#[derive(Clone)]
pub struct AnimationFrame {
    pub data: Vec<u8>,      // RGBA pixels
    pub width: u32,
    pub height: u32,
    pub delay_ms: u32,      // Delay before next frame
}

/// Animated image with frame data
#[derive(Clone)]
pub struct AnimatedImage {
    frames: Vec<AnimationFrame>,
    total_duration_ms: u32,
}

impl AnimatedImage {
    /// Create from a vector of frames
    pub fn new(frames: Vec<AnimationFrame>) -> Self {
        let total_duration_ms = frames.iter().map(|f| f.delay_ms).sum();
        Self { frames, total_duration_ms }
    }

    /// Check if image data might be animated (basic check)
    pub fn might_be_animated(data: &[u8]) -> bool {
        // Check for GIF signature
        if data.len() >= 6 && &data[0..6] == b"GIF89a" {
            return true;
        }
        if data.len() >= 6 && &data[0..6] == b"GIF87a" {
            return true;
        }
        // Check for PNG signature (APNG detection would need more parsing)
        if data.len() >= 8 && &data[0..8] == b"\x89PNG\r\n\x1a\n" {
            return true; // Could be APNG
        }
        // Check for WebP signature
        if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
            return true; // Could be animated WebP
        }
        false
    }

    /// Try to decode animated image from data
    /// Returns None if not animated or decoding fails
    pub fn from_data(data: &[u8]) -> Option<Self> {
        use image::codecs::gif::GifDecoder;
        use image::AnimationDecoder;
        use std::io::Cursor;

        // Try GIF first
        if let Ok(decoder) = GifDecoder::new(Cursor::new(data)) {
            let frames: Vec<_> = decoder
                .into_frames()
                .filter_map(|f| f.ok())
                .take(MAX_FRAMES)
                .map(|frame| {
                    let (numer, denom) = frame.delay().numer_denom_ms();
                    let delay_ms = ((numer as u64 * 1000) / denom as u64) as u32;
                    let buffer = frame.into_buffer();
                    let (width, height) = buffer.dimensions();

                    AnimationFrame {
                        data: buffer.into_raw(),
                        width,
                        height,
                        delay_ms: delay_ms.max(10), // Minimum 10ms delay
                    }
                })
                .collect();

            if frames.len() > 1 {
                return Some(Self::new(frames));
            }
        }

        None
    }

    /// Get number of frames
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Check if this is actually animated (more than 1 frame)
    pub fn is_animated(&self) -> bool {
        self.frames.len() > 1
    }

    /// Get frame at specific time offset (loops)
    pub fn frame_at(&self, elapsed_ms: u32) -> Option<&AnimationFrame> {
        if self.frames.is_empty() || self.total_duration_ms == 0 {
            return self.frames.first();
        }

        let looped_time = elapsed_ms % self.total_duration_ms;
        let mut accumulated = 0u32;

        for frame in &self.frames {
            accumulated += frame.delay_ms;
            if accumulated > looped_time {
                return Some(frame);
            }
        }

        self.frames.first()
    }

    /// Get first frame (for static fallback)
    pub fn first_frame(&self) -> Option<&AnimationFrame> {
        self.frames.first()
    }

    /// Get total animation duration
    pub fn total_duration(&self) -> Duration {
        Duration::from_millis(self.total_duration_ms as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_might_be_animated_gif() {
        let gif_data = b"GIF89a...";
        assert!(AnimatedImage::might_be_animated(gif_data));
    }

    #[test]
    fn test_might_be_animated_png() {
        let png_data = b"\x89PNG\r\n\x1a\n...";
        assert!(AnimatedImage::might_be_animated(png_data));
    }

    #[test]
    fn test_might_be_animated_webp() {
        let webp_data = b"RIFF....WEBP...";
        assert!(AnimatedImage::might_be_animated(webp_data));
    }

    #[test]
    fn test_not_animated_random_data() {
        let random_data = b"random data";
        assert!(!AnimatedImage::might_be_animated(random_data));
    }

    #[test]
    fn test_animation_frame_at() {
        let frames = vec![
            AnimationFrame { data: vec![], width: 10, height: 10, delay_ms: 100 },
            AnimationFrame { data: vec![], width: 10, height: 10, delay_ms: 100 },
            AnimationFrame { data: vec![], width: 10, height: 10, delay_ms: 100 },
        ];
        let anim = AnimatedImage::new(frames);

        // At time 0, should be first frame
        assert!(anim.frame_at(0).is_some());

        // At time 150, should be second frame
        assert!(anim.frame_at(150).is_some());

        // At time 350, should loop back
        assert!(anim.frame_at(350).is_some());
    }

    #[test]
    fn test_is_animated() {
        let single = AnimatedImage::new(vec![
            AnimationFrame { data: vec![], width: 10, height: 10, delay_ms: 100 },
        ]);
        assert!(!single.is_animated());

        let multi = AnimatedImage::new(vec![
            AnimationFrame { data: vec![], width: 10, height: 10, delay_ms: 100 },
            AnimationFrame { data: vec![], width: 10, height: 10, delay_ms: 100 },
        ]);
        assert!(multi.is_animated());
    }
}
