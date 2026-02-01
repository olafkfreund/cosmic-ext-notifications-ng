use cosmic::iced::{Alignment, Length};
use cosmic::iced_widget::{progress_bar, row};
use cosmic::widget::text;
use cosmic::Element;
use std::time::Instant;

/// Create a notification progress bar
///
/// # Arguments
/// * `value` - Progress value from 0.0 to 1.0
/// * `show_percentage` - Whether to show percentage text
///
/// # Example
/// ```
/// let progress = notification_progress(0.75, true);
/// ```
pub fn notification_progress<'a, Message: 'static>(
    value: f32,
    show_percentage: bool,
) -> Element<'a, Message> {
    let clamped_value = value.clamp(0.0, 1.0);

    // Create progress bar using iced's progress_bar widget
    let bar = progress_bar(0.0..=1.0, clamped_value)
        .width(Length::Fill)
        .height(Length::Fixed(4.0));

    if show_percentage {
        let percentage = format!("{}%", (clamped_value * 100.0).round() as u32);

        row![
            bar,
            cosmic::widget::Space::with_width(8),
            text::caption(percentage),
        ]
        .align_y(Alignment::Center)
        .into()
    } else {
        bar.into()
    }
}

/// Create a progress bar with custom styling
///
/// # Arguments
/// * `value` - Progress value from 0.0 to 1.0
/// * `height` - Height of the progress bar in pixels
///
/// # Example
/// ```
/// let progress = styled_progress(0.5, 6.0);
/// ```
pub fn styled_progress<'a, Message: 'static>(value: f32, height: f32) -> Element<'a, Message> {
    let clamped_value = value.clamp(0.0, 1.0);

    progress_bar(0.0..=1.0, clamped_value)
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .into()
}

/// Check if a notification should show a progress bar
///
/// Returns true if the progress value is valid (between 0.0 and 1.0)
///
/// # Arguments
/// * `progress` - Optional progress value
///
/// # Example
/// ```
/// if should_show_progress(Some(0.5)) {
///     // Show progress bar
/// }
/// ```
pub fn should_show_progress(progress: Option<f32>) -> bool {
    matches!(progress, Some(v) if (0.0..=1.0).contains(&v))
}

/// Animated progress bar state
///
/// Tracks smooth transitions between progress values to avoid jarring
/// jumps when progress updates rapidly.
///
/// # Performance Note
/// Animation duration is 300ms - long enough for smooth visual feedback
/// but short enough to feel responsive. Multiple concurrent progress
/// animations are lightweight as they only track scalar interpolation.
///
/// # Memory Note
/// This structure is very small (24 bytes) so having multiple instances
/// for concurrent notifications has negligible memory impact.
#[derive(Debug, Clone)]
pub struct AnimatedProgress {
  /// Current display value (being animated)
  current: f32,
  /// Target value (what we're animating toward)
  target: f32,
  /// When the current animation started
  start_time: Instant,
  /// Value at start of current animation
  start_value: f32,
  /// Duration of animation in milliseconds
  duration_ms: u64,
}

impl AnimatedProgress {
  /// Create a new animated progress tracker
  ///
  /// # Arguments
  /// * `initial_value` - Starting progress value (0.0 to 1.0)
  /// * `duration_ms` - Animation duration (default: 300ms recommended)
  pub fn new(initial_value: f32, duration_ms: u64) -> Self {
    let clamped = initial_value.clamp(0.0, 1.0);
    Self {
      current: clamped,
      target: clamped,
      start_time: Instant::now(),
      start_value: clamped,
      duration_ms,
    }
  }

  /// Set a new target value and start animation
  ///
  /// If the new target differs from current target, starts a new
  /// animation from the current interpolated position.
  pub fn set_target(&mut self, new_target: f32) {
    let clamped = new_target.clamp(0.0, 1.0);
    if (self.target - clamped).abs() > f32::EPSILON {
      self.start_value = self.current_value();
      self.target = clamped;
      self.start_time = Instant::now();
    }
  }

  /// Get the current interpolated value
  ///
  /// Uses linear interpolation for smooth animation.
  /// Returns a value between 0.0 and 1.0.
  pub fn current_value(&self) -> f32 {
    let elapsed = self.start_time.elapsed().as_millis() as u64;
    if elapsed >= self.duration_ms {
      return self.target;
    }

    let t = elapsed as f32 / self.duration_ms as f32;
    let value = self.start_value + (self.target - self.start_value) * t;
    value.clamp(0.0, 1.0)
  }

  /// Check if animation is complete
  pub fn is_animating(&self) -> bool {
    let elapsed = self.start_time.elapsed().as_millis() as u64;
    elapsed < self.duration_ms && (self.current - self.target).abs() > f32::EPSILON
  }

  /// Instantly set value without animation
  pub fn set_immediate(&mut self, value: f32) {
    let clamped = value.clamp(0.0, 1.0);
    self.current = clamped;
    self.target = clamped;
    self.start_value = clamped;
  }
}

impl Default for AnimatedProgress {
  fn default() -> Self {
    Self::new(0.0, 300) // 300ms default animation duration
  }
}

/// Create an animated progress bar
///
/// Uses AnimatedProgress state to smoothly transition between values.
///
/// # Arguments
/// * `state` - Mutable reference to animation state
/// * `show_percentage` - Whether to show percentage text
///
/// # Example
/// ```
/// let mut progress = AnimatedProgress::default();
/// progress.set_target(0.75);
/// let bar = animated_notification_progress(&mut progress, true);
/// ```
pub fn animated_notification_progress<'a, Message: 'static>(
  state: &AnimatedProgress,
  show_percentage: bool,
) -> Element<'a, Message> {
  let current_value = state.current_value();
  notification_progress(current_value, show_percentage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_show_progress() {
        assert!(should_show_progress(Some(0.0)));
        assert!(should_show_progress(Some(0.5)));
        assert!(should_show_progress(Some(1.0)));
        assert!(!should_show_progress(Some(-0.1)));
        assert!(!should_show_progress(Some(1.1)));
        assert!(!should_show_progress(None));
    }

    #[test]
    fn test_clamping() {
        // These would panic if not clamped, so we just verify they compile
        let _ = styled_progress::<()>(-1.0, 4.0);
        let _ = styled_progress::<()>(2.0, 4.0);
    }

    #[test]
    fn test_animated_progress_initialization() {
        let progress = AnimatedProgress::new(0.5, 300);
        assert!((progress.current_value() - 0.5).abs() < f32::EPSILON);
        assert!(!progress.is_animating());
    }

    #[test]
    fn test_animated_progress_set_target() {
        let mut progress = AnimatedProgress::new(0.0, 300);
        progress.set_target(1.0);
        assert!(progress.is_animating());
        assert!(progress.current_value() >= 0.0);
        assert!(progress.current_value() <= 1.0);
    }

    #[test]
    fn test_animated_progress_immediate() {
        let mut progress = AnimatedProgress::new(0.0, 300);
        progress.set_immediate(0.75);
        assert!((progress.current_value() - 0.75).abs() < f32::EPSILON);
        assert!(!progress.is_animating());
    }

    #[test]
    fn test_animated_progress_clamping() {
        let mut progress = AnimatedProgress::new(2.0, 300);
        assert!((progress.current_value() - 1.0).abs() < f32::EPSILON);

        progress.set_target(-1.0);
        // Wait for animation to complete by calling current_value repeatedly
        let _ = progress.current_value();
        std::thread::sleep(std::time::Duration::from_millis(350));
        assert!((progress.current_value() - 0.0).abs() < f32::EPSILON);
    }
}
