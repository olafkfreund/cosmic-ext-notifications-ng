//! HTML markup parser for notification body text
//!
//! Parses sanitized HTML into styled text segments that can be rendered
//! with rich text widgets.

use regex::Regex;

/// Style flags for text segments
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

/// A segment of styled text
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyledSegment {
    pub text: String,
    pub style: TextStyle,
    pub link: Option<String>,
}

impl StyledSegment {
    /// Create a plain text segment
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: TextStyle::default(),
            link: None,
        }
    }

    /// Create a styled text segment
    pub fn styled(text: impl Into<String>, style: TextStyle) -> Self {
        Self {
            text: text.into(),
            style,
            link: None,
        }
    }

    /// Create a link segment
    pub fn link(text: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: TextStyle::default(),
            link: Some(url.into()),
        }
    }
}

/// Parse sanitized HTML into styled text segments
///
/// Supports: <b>, <i>, <u>, <a href="...">
/// Nested tags are supported (e.g., <b><i>bold italic</i></b>)
pub fn parse_markup(html: &str) -> Vec<StyledSegment> {
    let mut segments = Vec::new();
    let mut current_style = TextStyle::default();
    let mut current_link: Option<String> = None;
    let mut style_stack: Vec<(&str, TextStyle, Option<String>)> = Vec::new();

    // Regex patterns for tag matching
    let tag_pattern = Regex::new(r"<(/?)(\w+)(?:\s+[^>]*)?>").unwrap();
    let href_pattern = Regex::new(r#"href=["']([^"']+)["']"#).unwrap();

    let mut last_end = 0;

    for cap in tag_pattern.captures_iter(html) {
        let full_match = cap.get(0).unwrap();
        let is_closing = &cap[1] == "/";
        let tag_name = cap[2].to_lowercase();

        // Add text before this tag
        let text_before = &html[last_end..full_match.start()];
        if !text_before.is_empty() {
            let decoded = decode_entities(text_before);
            if !decoded.is_empty() {
                segments.push(StyledSegment {
                    text: decoded,
                    style: current_style.clone(),
                    link: current_link.clone(),
                });
            }
        }

        last_end = full_match.end();

        if is_closing {
            // Pop style from stack
            if let Some((expected_tag, prev_style, prev_link)) = style_stack.pop() {
                if expected_tag == tag_name {
                    current_style = prev_style;
                    current_link = prev_link;
                }
            }
        } else {
            // Push current style and apply new
            let prev_style = current_style.clone();
            let prev_link = current_link.clone();

            match tag_name.as_str() {
                "b" | "strong" => {
                    style_stack.push(("b", prev_style, prev_link));
                    current_style.bold = true;
                }
                "i" | "em" => {
                    style_stack.push(("i", prev_style, prev_link));
                    current_style.italic = true;
                }
                "u" => {
                    style_stack.push(("u", prev_style, prev_link));
                    current_style.underline = true;
                }
                "a" => {
                    // Extract href
                    let tag_content = full_match.as_str();
                    if let Some(href_cap) = href_pattern.captures(tag_content) {
                        let url = decode_entities(&href_cap[1]);
                        style_stack.push(("a", prev_style, prev_link));
                        current_link = Some(url);
                        current_style.underline = true; // Links are underlined
                    }
                }
                "br" | "p" => {
                    // Line breaks - add newline
                    segments.push(StyledSegment::plain("\n"));
                }
                _ => {}
            }
        }
    }

    // Add remaining text after last tag
    let remaining = &html[last_end..];
    if !remaining.is_empty() {
        let decoded = decode_entities(remaining);
        if !decoded.is_empty() {
            segments.push(StyledSegment {
                text: decoded,
                style: current_style,
                link: current_link,
            });
        }
    }

    // If no tags were found, return the whole text as plain
    if segments.is_empty() && !html.is_empty() {
        segments.push(StyledSegment::plain(decode_entities(html)));
    }

    // Merge adjacent segments with same style
    merge_segments(segments)
}

/// Decode HTML entities
fn decode_entities(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
        .replace("&#58;", ":")
        .replace("&#x3A;", ":")
        .replace("&nbsp;", " ")
}

/// Merge adjacent segments with the same style
fn merge_segments(segments: Vec<StyledSegment>) -> Vec<StyledSegment> {
    let mut merged: Vec<StyledSegment> = Vec::new();

    for segment in segments {
        if let Some(last) = merged.last_mut() {
            if last.style == segment.style && last.link == segment.link {
                last.text.push_str(&segment.text);
                continue;
            }
        }
        merged.push(segment);
    }

    merged
}

/// Convert segments back to plain text (for fallback)
pub fn segments_to_plain_text(segments: &[StyledSegment]) -> String {
    segments.iter().map(|s| s.text.as_str()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text() {
        let segments = parse_markup("Hello World");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "Hello World");
        assert!(!segments[0].style.bold);
    }

    #[test]
    fn test_bold_text() {
        let segments = parse_markup("Hello <b>Bold</b> World");
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].text, "Hello ");
        assert!(!segments[0].style.bold);
        assert_eq!(segments[1].text, "Bold");
        assert!(segments[1].style.bold);
        assert_eq!(segments[2].text, " World");
        assert!(!segments[2].style.bold);
    }

    #[test]
    fn test_italic_text() {
        let segments = parse_markup("Hello <i>Italic</i> World");
        assert_eq!(segments.len(), 3);
        assert!(segments[1].style.italic);
    }

    #[test]
    fn test_underline_text() {
        let segments = parse_markup("Hello <u>Underline</u> World");
        assert_eq!(segments.len(), 3);
        assert!(segments[1].style.underline);
    }

    #[test]
    fn test_nested_tags() {
        let segments = parse_markup("<b><i>Bold Italic</i></b>");
        assert_eq!(segments.len(), 1);
        assert!(segments[0].style.bold);
        assert!(segments[0].style.italic);
    }

    #[test]
    fn test_link() {
        let segments = parse_markup(r#"Click <a href="https://example.com">here</a>"#);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[1].text, "here");
        assert_eq!(segments[1].link, Some("https://example.com".to_string()));
        assert!(segments[1].style.underline);
    }

    #[test]
    fn test_entity_decoding() {
        let segments = parse_markup("&lt;script&gt; &amp; &quot;test&quot;");
        assert_eq!(segments[0].text, "<script> & \"test\"");
    }

    #[test]
    fn test_strong_tag() {
        let segments = parse_markup("<strong>Strong</strong>");
        assert!(segments[0].style.bold);
    }

    #[test]
    fn test_em_tag() {
        let segments = parse_markup("<em>Emphasis</em>");
        assert!(segments[0].style.italic);
    }

    #[test]
    fn test_br_tag() {
        let segments = parse_markup("Line 1<br>Line 2");
        let text = segments_to_plain_text(&segments);
        assert!(text.contains('\n'));
    }

    #[test]
    fn test_empty_string() {
        let segments = parse_markup("");
        assert!(segments.is_empty());
    }

    #[test]
    fn test_complex_markup() {
        let html = r#"New message from <b>John</b>: <i>"Hello <u>there</u>!"</i>"#;
        let segments = parse_markup(html);
        assert!(!segments.is_empty());
        // Verify we can convert back to text
        let plain = segments_to_plain_text(&segments);
        assert!(plain.contains("John"));
        assert!(plain.contains("Hello"));
    }
}
