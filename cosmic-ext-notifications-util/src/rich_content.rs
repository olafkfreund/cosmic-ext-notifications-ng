use crate::{Hint, Image};

#[cfg(feature = "image")]
use crate::{NotificationImage, ProcessedImage};

/// Extracted rich content from notification hints
#[derive(Debug, Clone, Default)]
pub struct RichContent {
    #[cfg(feature = "image")]
    pub processed_image: Option<ProcessedImage>,
    pub progress: Option<f32>,
    pub urgency: u8,
    pub category: Option<String>,
}

impl RichContent {
    /// Extract rich content from notification hints
    pub fn from_hints(hints: &[Hint]) -> Self {
        let mut content = Self::default();

        // Extract urgency (default to 1 = Normal)
        content.urgency = hints.iter().find_map(|h| match h {
            Hint::Urgency(u) => Some(*u),
            _ => None,
        }).unwrap_or(1);

        // Extract category
        content.category = hints.iter().find_map(|h| match h {
            Hint::Category(c) => Some(c.clone()),
            _ => None,
        });

        // Extract and process image (priority: image-data > image-path)
        #[cfg(feature = "image")]
        {
            content.processed_image = Self::extract_image(hints);
        }

        // Extract progress value (0-100)
        content.progress = hints.iter().find_map(|h| match h {
            Hint::Value(v) => {
                // Clamp value between 0 and 100, convert to 0.0-1.0 range
                let clamped = (*v).max(0).min(100);
                Some(clamped as f32 / 100.0)
            },
            _ => None,
        });

        content
    }

    #[cfg(feature = "image")]
    fn extract_image(hints: &[Hint]) -> Option<ProcessedImage> {
        // Try to find Image::Data first (raw pixel data - highest priority)
        for hint in hints {
            if let Hint::Image(Image::Data { width, height, data }) = hint {
                if let Ok(img) = NotificationImage::from_raw_data(
                    data, *width as i32, *height as i32, (*width * 4) as i32, true
                ) {
                    return Some(img);
                }
            }
        }

        // Try Image::File path (second priority)
        for hint in hints {
            if let Hint::Image(Image::File(path)) = hint {
                if let Some(path_str) = path.to_str() {
                    if let Ok(img) = NotificationImage::from_path(path_str) {
                        return Some(img);
                    }
                }
            }
        }

        // Try Image::Name (icon name - would need icon theme lookup)
        // For now, skip icon names as they need additional infrastructure
        // In the future, this could use icon theme lookup to resolve icon names

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_extract_urgency_low() {
        let hints = vec![Hint::Urgency(0)];
        let content = RichContent::from_hints(&hints);
        assert_eq!(content.urgency, 0);
    }

    #[test]
    fn test_extract_urgency_normal() {
        let hints = vec![Hint::Urgency(1)];
        let content = RichContent::from_hints(&hints);
        assert_eq!(content.urgency, 1);
    }

    #[test]
    fn test_extract_urgency_critical() {
        let hints = vec![Hint::Urgency(2)];
        let content = RichContent::from_hints(&hints);
        assert_eq!(content.urgency, 2);
    }

    #[test]
    fn test_default_urgency_when_missing() {
        let hints = vec![Hint::Category("test".to_string())];
        let content = RichContent::from_hints(&hints);
        assert_eq!(content.urgency, 1); // Default to Normal
    }

    #[test]
    fn test_extract_category() {
        let hints = vec![Hint::Category("email".to_string())];
        let content = RichContent::from_hints(&hints);
        assert_eq!(content.category, Some("email".to_string()));
    }

    #[test]
    fn test_category_missing() {
        let hints = vec![Hint::Urgency(1)];
        let content = RichContent::from_hints(&hints);
        assert_eq!(content.category, None);
    }

    #[test]
    fn test_extract_progress_value() {
        let hints = vec![Hint::Value(50)];
        let content = RichContent::from_hints(&hints);
        assert_eq!(content.progress, Some(0.5));
    }

    #[test]
    fn test_progress_value_clamping_low() {
        let hints = vec![Hint::Value(-10)];
        let content = RichContent::from_hints(&hints);
        assert_eq!(content.progress, Some(0.0));
    }

    #[test]
    fn test_progress_value_clamping_high() {
        let hints = vec![Hint::Value(150)];
        let content = RichContent::from_hints(&hints);
        assert_eq!(content.progress, Some(1.0));
    }

    #[test]
    fn test_progress_value_zero() {
        let hints = vec![Hint::Value(0)];
        let content = RichContent::from_hints(&hints);
        assert_eq!(content.progress, Some(0.0));
    }

    #[test]
    fn test_progress_value_max() {
        let hints = vec![Hint::Value(100)];
        let content = RichContent::from_hints(&hints);
        assert_eq!(content.progress, Some(1.0));
    }

    #[test]
    fn test_progress_missing() {
        let hints = vec![Hint::Urgency(1)];
        let content = RichContent::from_hints(&hints);
        assert_eq!(content.progress, None);
    }

    #[test]
    fn test_multiple_hints() {
        let hints = vec![
            Hint::Urgency(2),
            Hint::Category("download".to_string()),
            Hint::Value(75),
        ];
        let content = RichContent::from_hints(&hints);
        assert_eq!(content.urgency, 2);
        assert_eq!(content.category, Some("download".to_string()));
        assert_eq!(content.progress, Some(0.75));
    }

    #[test]
    fn test_empty_hints() {
        let hints = vec![];
        let content = RichContent::from_hints(&hints);
        assert_eq!(content.urgency, 1); // Default
        assert_eq!(content.category, None);
        assert_eq!(content.progress, None);
    }

    #[test]
    #[cfg(feature = "image")]
    fn test_image_priority_data_over_file() {
        // Create minimal RGBA data (1x1 red pixel)
        let data = vec![255, 0, 0, 255];

        let hints = vec![
            Hint::Image(Image::File(PathBuf::from("/path/to/icon.png"))),
            Hint::Image(Image::Data {
                width: 1,
                height: 1,
                data: std::sync::Arc::new(data.clone()),
            }),
        ];

        let content = RichContent::from_hints(&hints);
        // Image::Data should be preferred over Image::File
        assert!(content.processed_image.is_some());
    }

    #[test]
    #[cfg(feature = "image")]
    fn test_image_file_fallback() {
        // When only file path is provided
        let hints = vec![
            Hint::Image(Image::File(PathBuf::from("/nonexistent/path.png"))),
        ];

        let content = RichContent::from_hints(&hints);
        // Will be None because file doesn't exist, but extraction was attempted
        // In real usage, valid file paths would return Some(ProcessedImage)
        assert!(content.processed_image.is_none());
    }

    #[test]
    #[cfg(feature = "image")]
    fn test_image_name_not_supported() {
        // Icon names are not yet supported
        let hints = vec![
            Hint::Image(Image::Name("dialog-information".to_string())),
        ];

        let content = RichContent::from_hints(&hints);
        assert!(content.processed_image.is_none());
    }

    #[test]
    #[cfg(feature = "image")]
    fn test_no_image_hints() {
        let hints = vec![
            Hint::Urgency(1),
            Hint::Category("test".to_string()),
        ];

        let content = RichContent::from_hints(&hints);
        assert!(content.processed_image.is_none());
    }
}
