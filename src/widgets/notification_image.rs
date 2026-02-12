#![allow(dead_code)]

use cosmic::iced::Length;
use cosmic::widget::{container, icon};
use cosmic::Element;
use cosmic_ext_notifications_util::ProcessedImage;
use std::time::Instant;

/// Size variants for notification images
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageSize {
  Icon,      // 32x32 - for app icons
  Thumbnail, // 64x64 - inline in notification
  Expanded,  // 128x128 - larger view
}

impl ImageSize {
  pub fn dimensions(&self) -> (u16, u16) {
    match self {
      Self::Icon => (32, 32),
      Self::Thumbnail => (64, 64),
      Self::Expanded => (128, 128),
    }
  }
}

/// Create an image element from processed notification image data
pub fn notification_image<'a, Message: 'a>(
  image: &ProcessedImage,
  size: ImageSize,
) -> Element<'a, Message> {
  let (width, height) = size.dimensions();

  if image.data.is_empty() || image.width == 0 || image.height == 0 {
    return placeholder_image(width, height);
  }

  // Use cosmic's icon::from_raster_pixels for RGBA data
  // Apply .size() on the Icon widget (after .icon()) to scale to desired display size
  let icon_widget = icon::from_raster_pixels(image.width, image.height, image.data.clone())
    .icon()
    .size(width);  // Scale icon to target size

  container(icon_widget)
    .width(Length::Fixed(width as f32))
    .height(Length::Fixed(height as f32))
    .center_x(Length::Fixed(width as f32))
    .center_y(Length::Fixed(height as f32))
    .into()
}

/// Create a placeholder when image is not available
pub fn placeholder_image<'a, Message: 'a>(width: u16, height: u16) -> Element<'a, Message> {
  container(cosmic::widget::Space::new(width, height))
    .width(Length::Fixed(width as f32))
    .height(Length::Fixed(height as f32))
    .into()
}

/// Create app icon from icon name or fallback to processed image
pub fn app_icon<'a, Message: 'a>(
  icon_name: &str,
  fallback: Option<&ProcessedImage>,
) -> Element<'a, Message> {
  if !icon_name.is_empty() {
    // Try to use icon name from theme
    let icon_widget = icon::from_name(icon_name).size(32).icon();
    return container(icon_widget)
      .width(Length::Fixed(32.0))
      .height(Length::Fixed(32.0))
      .into();
  }

  // Use fallback image if provided
  if let Some(img) = fallback {
    return notification_image(img, ImageSize::Icon);
  }

  // Return placeholder
  placeholder_image(32, 32)
}

/// Animation state for image fade-in effect
///
/// Tracks the fade-in animation for notification images to provide
/// a smooth visual transition when images are loaded.
///
/// # Performance Note
/// The fade-in animation duration is kept short (200ms) to maintain
/// a snappy UI feel while still providing visual polish.
#[derive(Debug, Clone)]
pub struct ImageFadeInState {
  /// When the image started displaying
  start_time: Instant,
  /// Duration of fade-in animation in milliseconds
  duration_ms: u64,
}

impl ImageFadeInState {
  /// Create a new fade-in animation state
  ///
  /// # Arguments
  /// * `duration_ms` - Duration of the fade-in effect (default: 200ms recommended)
  pub fn new(duration_ms: u64) -> Self {
    Self {
      start_time: Instant::now(),
      duration_ms,
    }
  }

  /// Get current opacity value (0.0 to 1.0)
  ///
  /// Returns a value that smoothly interpolates from 0.0 to 1.0
  /// over the animation duration using linear interpolation.
  pub fn opacity(&self) -> f32 {
    let elapsed = self.start_time.elapsed().as_millis() as u64;
    if elapsed >= self.duration_ms {
      1.0
    } else {
      (elapsed as f32 / self.duration_ms as f32).clamp(0.0, 1.0)
    }
  }

  /// Check if animation is complete
  pub fn is_complete(&self) -> bool {
    self.start_time.elapsed().as_millis() as u64 >= self.duration_ms
  }
}

impl Default for ImageFadeInState {
  fn default() -> Self {
    Self::new(200) // 200ms default for snappy feel
  }
}
