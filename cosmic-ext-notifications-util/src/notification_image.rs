/// Image processing utilities for notification images.
///
/// This module provides functionality to process notification images from various sources:
/// - Raw pixel data with rowstride and alpha channel information
/// - File paths to image files
///
/// All images are normalized to RGBA format and resized to fit within maximum dimensions
/// while preserving aspect ratio.

use fast_image_resize as fr;
use image::ImageError;

/// Maximum width for notification images in pixels
pub const MAX_IMAGE_WIDTH: u32 = 128;

/// Maximum height for notification images in pixels
pub const MAX_IMAGE_HEIGHT: u32 = 128;

/// Processed notification image ready for display
#[derive(Debug, Clone)]
pub struct ProcessedImage {
  /// Raw RGBA pixel data
  pub data: Vec<u8>,
  /// Image width in pixels
  pub width: u32,
  /// Image height in pixels
  pub height: u32,
}

/// Notification image processor
pub struct NotificationImage;

impl NotificationImage {
  /// Create a ProcessedImage from raw pixel data.
  ///
  /// # Arguments
  ///
  /// * `data` - Raw pixel data
  /// * `width` - Image width in pixels
  /// * `height` - Image height in pixels
  /// * `rowstride` - Number of bytes per row (may include padding)
  /// * `has_alpha` - Whether the data contains an alpha channel (RGBA vs RGB)
  ///
  /// # Returns
  ///
  /// A `ProcessedImage` with RGBA data, resized if necessary to fit within max dimensions.
  ///
  /// # Errors
  ///
  /// Returns `ImageError` if the image data is invalid or processing fails.
  pub fn from_raw_data(
    data: &[u8],
    width: i32,
    height: i32,
    rowstride: i32,
    has_alpha: bool,
  ) -> Result<ProcessedImage, ImageError> {
    if width <= 0 || height <= 0 {
      return Err(ImageError::Limits(
        image::error::LimitError::from_kind(
          image::error::LimitErrorKind::DimensionError,
        ),
      ));
    }

    let width = width as u32;
    let height = height as u32;
    let channels = if has_alpha { 4 } else { 3 };

    // Validate data length
    if data.len() < (rowstride * height as i32) as usize {
      return Err(ImageError::Limits(
        image::error::LimitError::from_kind(
          image::error::LimitErrorKind::InsufficientMemory,
        ),
      ));
    }

    // Extract pixel data handling rowstride
    let mut pixel_data = Vec::with_capacity((width * height * channels) as usize);
    for y in 0..height {
      let row_start = (y as i32 * rowstride) as usize;
      let row_data = &data[row_start..row_start + (width * channels) as usize];
      pixel_data.extend_from_slice(row_data);
    }

    // Convert RGB to RGBA if necessary
    let rgba_data = if has_alpha {
      pixel_data
    } else {
      let mut rgba = Vec::with_capacity((width * height * 4) as usize);
      for chunk in pixel_data.chunks_exact(3) {
        rgba.extend_from_slice(chunk);
        rgba.push(255); // Add alpha channel
      }
      rgba
    };

    // Resize if necessary
    let (final_width, final_height, final_data) =
      Self::resize_if_needed(width, height, rgba_data)?;

    Ok(ProcessedImage {
      data: final_data,
      width: final_width,
      height: final_height,
    })
  }

  /// Load and process an image from a file path.
  ///
  /// # Arguments
  ///
  /// * `path` - Path to the image file
  ///
  /// # Returns
  ///
  /// A `ProcessedImage` with RGBA data, resized if necessary to fit within max dimensions.
  ///
  /// # Errors
  ///
  /// Returns `ImageError` if the file cannot be read or is not a valid image.
  pub fn from_path(path: &str) -> Result<ProcessedImage, ImageError> {
    // Load image from file
    let img = image::open(path)?;

    // Convert to RGBA
    let rgba_img = img.to_rgba8();
    let width = rgba_img.width();
    let height = rgba_img.height();
    let data = rgba_img.into_raw();

    // Resize if necessary
    let (final_width, final_height, final_data) = Self::resize_if_needed(width, height, data)?;

    Ok(ProcessedImage {
      data: final_data,
      width: final_width,
      height: final_height,
    })
  }

  /// Resize image if it exceeds maximum dimensions, preserving aspect ratio.
  ///
  /// Uses Lanczos3 algorithm for high-quality downscaling.
  fn resize_if_needed(
    width: u32,
    height: u32,
    data: Vec<u8>,
  ) -> Result<(u32, u32, Vec<u8>), ImageError> {
    // Check if resize is needed
    if width <= MAX_IMAGE_WIDTH && height <= MAX_IMAGE_HEIGHT {
      return Ok((width, height, data));
    }

    // Calculate new dimensions preserving aspect ratio
    let aspect_ratio = width as f32 / height as f32;
    let (new_width, new_height) = if width > height {
      let new_width = MAX_IMAGE_WIDTH;
      let new_height = (new_width as f32 / aspect_ratio) as u32;
      (new_width, new_height.max(1))
    } else {
      let new_height = MAX_IMAGE_HEIGHT;
      let new_width = (new_height as f32 * aspect_ratio) as u32;
      (new_width.max(1), new_height)
    };

    // Use fast_image_resize for high-quality resizing
    let mut src = fr::images::Image::from_vec_u8(width, height, data, fr::PixelType::U8x4)
      .map_err(|_| {
        ImageError::Limits(image::error::LimitError::from_kind(
          image::error::LimitErrorKind::DimensionError,
        ))
      })?;

    let mut dst = fr::images::Image::new(new_width, new_height, fr::PixelType::U8x4);

    // Multiply alpha for proper blending during resize
    fr::MulDiv::default()
      .multiply_alpha_inplace(&mut src)
      .map_err(|_| {
        ImageError::Limits(image::error::LimitError::from_kind(
          image::error::LimitErrorKind::DimensionError,
        ))
      })?;

    // Resize with Lanczos3 algorithm
    let mut resizer = fr::Resizer::new();
    let resize_options = fr::ResizeOptions::new()
      .resize_alg(fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3));

    resizer
      .resize(&src, &mut dst, Some(&resize_options))
      .map_err(|_| {
        ImageError::Limits(image::error::LimitError::from_kind(
          image::error::LimitErrorKind::DimensionError,
        ))
      })?;

    // Divide alpha back
    fr::MulDiv::default()
      .divide_alpha_inplace(&mut dst)
      .map_err(|_| {
        ImageError::Limits(image::error::LimitError::from_kind(
          image::error::LimitErrorKind::DimensionError,
        ))
      })?;

    Ok((new_width, new_height, dst.into_vec()))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use image::RgbaImage;

  /// Test that images larger than max dimensions are resized
  #[test]
  fn test_resize_large_image() {
    // Create a 256x256 RGBA image (exceeds max 128x128)
    let width = 256;
    let height = 256;
    let channels = 4; // RGBA
    let data = vec![255u8; (width * height * channels) as usize];

    let result = NotificationImage::from_raw_data(&data, width, height, width * channels, true);

    assert!(result.is_ok());
    let processed = result.unwrap();

    // Should be resized to fit within 128x128
    assert!(processed.width <= MAX_IMAGE_WIDTH);
    assert!(processed.height <= MAX_IMAGE_HEIGHT);

    // Should maintain aspect ratio (square image stays square)
    assert_eq!(processed.width, processed.height);
    assert_eq!(processed.width, MAX_IMAGE_WIDTH);

    // RGBA data should have 4 bytes per pixel
    assert_eq!(
      processed.data.len(),
      (processed.width * processed.height * 4) as usize
    );
  }

  /// Test that aspect ratio is preserved when resizing
  #[test]
  fn test_aspect_ratio_preservation() {
    // Create a 200x100 RGBA image (2:1 aspect ratio)
    let width = 200;
    let height = 100;
    let channels = 4;
    let data = vec![128u8; (width * height * channels) as usize];

    let result = NotificationImage::from_raw_data(&data, width, height, width * channels, true);

    assert!(result.is_ok());
    let processed = result.unwrap();

    // Should be resized to fit within 128x128
    assert!(processed.width <= MAX_IMAGE_WIDTH);
    assert!(processed.height <= MAX_IMAGE_HEIGHT);

    // Should maintain 2:1 aspect ratio
    let aspect_ratio = processed.width as f32 / processed.height as f32;
    assert!((aspect_ratio - 2.0).abs() < 0.01, "Aspect ratio not preserved");

    // Width should be at max, height should be half
    assert_eq!(processed.width, MAX_IMAGE_WIDTH);
    assert_eq!(processed.height, MAX_IMAGE_HEIGHT / 2);
  }

  /// Test RGB to RGBA conversion
  #[test]
  fn test_rgb_to_rgba_conversion() {
    // Create a small 4x4 RGB image
    let width = 4;
    let height = 4;
    let channels = 3; // RGB
    let data = vec![200u8; (width * height * channels) as usize];

    let result = NotificationImage::from_raw_data(&data, width, height, width * channels, false);

    assert!(result.is_ok());
    let processed = result.unwrap();

    // Should have RGBA format (4 bytes per pixel)
    assert_eq!(
      processed.data.len(),
      (processed.width * processed.height * 4) as usize
    );

    // Check that alpha channel was added (every 4th byte should be 255)
    for i in 0..(processed.width * processed.height) as usize {
      let alpha_byte = processed.data[i * 4 + 3];
      assert_eq!(alpha_byte, 255, "Alpha channel not properly set");
    }
  }

  /// Test handling of rowstride padding
  #[test]
  fn test_rowstride_handling() {
    // Create a 4x4 RGB image with rowstride padding
    let width = 4;
    let height = 4;
    let channels = 3; // RGB
    let rowstride = width * channels + 2; // Extra 2 bytes padding per row

    // Build data with padding
    let mut data = Vec::new();
    for _ in 0..height {
      // Add actual pixel data for the row
      for _ in 0..width {
        data.extend_from_slice(&[100u8, 150u8, 200u8]); // RGB values
      }
      // Add padding
      data.extend_from_slice(&[0u8, 0u8]);
    }

    assert_eq!(data.len(), (rowstride * height) as usize);

    let result = NotificationImage::from_raw_data(&data, width, height, rowstride, false);

    assert!(result.is_ok());
    let processed = result.unwrap();

    // Should produce valid RGBA image
    assert_eq!(processed.width, width as u32);
    assert_eq!(processed.height, height as u32);
    assert_eq!(
      processed.data.len(),
      (processed.width * processed.height * 4) as usize
    );

    // Verify first pixel was correctly extracted (ignoring padding)
    assert_eq!(processed.data[0], 100); // R
    assert_eq!(processed.data[1], 150); // G
    assert_eq!(processed.data[2], 200); // B
    assert_eq!(processed.data[3], 255); // A (added)
  }

  /// Test loading an image from a file path
  #[test]
  fn test_from_path() {
    // Create a temporary test image
    use std::fs;

    let temp_dir = std::env::temp_dir();
    let test_image_path = temp_dir.join("test_notification.png");

    // Create a simple 64x64 RGBA test image
    let img = RgbaImage::from_fn(64, 64, |x, y| {
      image::Rgba([
        (x * 4) as u8,
        (y * 4) as u8,
        128u8,
        255u8,
      ])
    });

    img.save(&test_image_path).expect("Failed to save test image");

    // Test loading it
    let result = NotificationImage::from_path(test_image_path.to_str().unwrap());

    assert!(result.is_ok());
    let processed = result.unwrap();

    // Should load successfully
    assert_eq!(processed.width, 64);
    assert_eq!(processed.height, 64);
    assert_eq!(
      processed.data.len(),
      (processed.width * processed.height * 4) as usize
    );

    // Cleanup
    fs::remove_file(&test_image_path).ok();
  }

  /// Test that images within max dimensions are not resized
  #[test]
  fn test_small_image_not_resized() {
    // Create a 64x64 RGBA image (within max 128x128)
    let width = 64;
    let height = 64;
    let channels = 4;
    let data = vec![128u8; (width * height * channels) as usize];

    let result = NotificationImage::from_raw_data(&data, width, height, width * channels, true);

    assert!(result.is_ok());
    let processed = result.unwrap();

    // Should NOT be resized
    assert_eq!(processed.width, width as u32);
    assert_eq!(processed.height, height as u32);
  }

  /// Test handling invalid data
  #[test]
  fn test_invalid_data() {
    // Too little data for the specified dimensions
    let width = 100;
    let height = 100;
    let channels = 4;
    let data = vec![0u8; 10]; // Way too small

    let result = NotificationImage::from_raw_data(&data, width, height, width * channels, true);

    assert!(result.is_err(), "Should fail with insufficient data");
  }

  /// Test loading non-existent file
  #[test]
  fn test_nonexistent_file() {
    let result = NotificationImage::from_path("/nonexistent/path/to/image.png");
    assert!(result.is_err(), "Should fail for non-existent file");
  }
}
