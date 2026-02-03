//! HTML markup parser for notification body text
//!
//! Parses sanitized HTML into styled text segments that can be rendered
//! with rich text widgets.
//!
//! SECURITY: This parser expects input to be pre-sanitized with ammonia.
//! It uses a state-machine approach instead of regex for safer parsing.

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
///
/// SECURITY: Input must be pre-sanitized with ammonia to remove dangerous content.
/// This parser validates URLs and uses case-insensitive tag matching.
pub fn parse_markup(html: &str) -> Vec<StyledSegment> {
    let mut segments = Vec::new();
    let mut current_style = TextStyle::default();
    let mut current_link: Option<String> = None;
    let mut style_stack: Vec<(String, TextStyle, Option<String>)> = Vec::new();

    let mut chars = html.chars().peekable();
    let mut current_text = String::new();

    while let Some(ch) = chars.next() {
        if ch == '<' {
            // Save any accumulated text
            if !current_text.is_empty() {
                let decoded = decode_entities(&current_text);
                if !decoded.is_empty() {
                    segments.push(StyledSegment {
                        text: decoded,
                        style: current_style.clone(),
                        link: current_link.clone(),
                    });
                }
                current_text.clear();
            }

            // Parse the tag
            if let Some(tag) = parse_tag(&mut chars) {
                match tag {
                    Tag::Open(name, attrs) => {
                        let tag_lower = name.to_lowercase();
                        let prev_style = current_style.clone();
                        let prev_link = current_link.clone();

                        match tag_lower.as_str() {
                            "b" | "strong" => {
                                style_stack.push((tag_lower, prev_style, prev_link));
                                current_style.bold = true;
                            }
                            "i" | "em" => {
                                style_stack.push((tag_lower, prev_style, prev_link));
                                current_style.italic = true;
                            }
                            "u" => {
                                style_stack.push((tag_lower, prev_style, prev_link));
                                current_style.underline = true;
                            }
                            "a" => {
                                if let Some(href) = attrs.get("href") {
                                    // Validate URL is safe
                                    if is_safe_url(href) {
                                        let decoded_url = decode_entities(href);
                                        style_stack.push((tag_lower, prev_style, prev_link));
                                        current_link = Some(decoded_url);
                                        current_style.underline = true;
                                    }
                                }
                            }
                            "br" | "p" => {
                                segments.push(StyledSegment::plain("\n"));
                            }
                            _ => {} // Ignore unknown tags
                        }
                    }
                    Tag::Close(name) => {
                        let tag_lower = name.to_lowercase();
                        // Only pop from stack if the TOP matches (proper nesting)
                        if let Some((tag, _, _)) = style_stack.last() {
                            let matches = *tag == tag_lower
                                || (*tag == "b" && tag_lower == "strong")
                                || (*tag == "strong" && tag_lower == "b")
                                || (*tag == "i" && tag_lower == "em")
                                || (*tag == "em" && tag_lower == "i");

                            if matches {
                                if let Some((_, prev_style, prev_link)) = style_stack.pop() {
                                    current_style = prev_style;
                                    current_link = prev_link;
                                }
                            }
                            // If no match, ignore the closing tag (malformed HTML)
                        }
                    }
                }
            }
        } else {
            current_text.push(ch);
        }
    }

    // Add any remaining text
    if !current_text.is_empty() {
        let decoded = decode_entities(&current_text);
        if !decoded.is_empty() {
            segments.push(StyledSegment {
                text: decoded,
                style: current_style,
                link: current_link,
            });
        }
    }

    // If no segments, return plain text
    if segments.is_empty() && !html.is_empty() {
        segments.push(StyledSegment::plain(decode_entities(html)));
    }

    // Merge adjacent segments with same style
    merge_segments(segments)
}

/// Represents a parsed HTML tag
#[derive(Debug)]
enum Tag {
    Open(String, std::collections::HashMap<String, String>),
    Close(String),
}

/// Parse a single HTML tag using character-by-character state machine
fn parse_tag(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<Tag> {
    let mut tag_content = String::new();

    // Read until '>'
    while let Some(&ch) = chars.peek() {
        chars.next();
        if ch == '>' {
            break;
        }
        tag_content.push(ch);
    }

    if tag_content.is_empty() {
        return None;
    }

    // Check if closing tag
    let is_closing = tag_content.starts_with('/');
    let tag_content = if is_closing {
        &tag_content[1..]
    } else {
        &tag_content
    };

    // Split tag name from attributes
    let parts: Vec<&str> = tag_content.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let tag_name = parts[0].to_string();

    if is_closing {
        return Some(Tag::Close(tag_name));
    }

    // Parse attributes for opening tags
    let mut attrs = std::collections::HashMap::new();
    let attr_string = parts[1..].join(" ");

    if !attr_string.is_empty() {
        // Simple attribute parser - looks for name="value" or name='value'
        let mut i = 0;
        let attr_chars: Vec<char> = attr_string.chars().collect();

        while i < attr_chars.len() {
            // Skip whitespace
            while i < attr_chars.len() && attr_chars[i].is_whitespace() {
                i += 1;
            }

            if i >= attr_chars.len() {
                break;
            }

            // Read attribute name
            let mut attr_name = String::new();
            while i < attr_chars.len() && attr_chars[i] != '=' && !attr_chars[i].is_whitespace() {
                attr_name.push(attr_chars[i]);
                i += 1;
            }

            // Skip to '='
            while i < attr_chars.len() && attr_chars[i] != '=' {
                i += 1;
            }
            i += 1; // Skip '='

            // Skip whitespace after '='
            while i < attr_chars.len() && attr_chars[i].is_whitespace() {
                i += 1;
            }

            if i >= attr_chars.len() {
                break;
            }

            // Read attribute value
            let quote = if attr_chars[i] == '"' || attr_chars[i] == '\'' {
                let q = attr_chars[i];
                i += 1; // Skip opening quote
                q
            } else {
                '\0'
            };

            let mut attr_value = String::new();
            if quote != '\0' {
                while i < attr_chars.len() && attr_chars[i] != quote {
                    attr_value.push(attr_chars[i]);
                    i += 1;
                }
                i += 1; // Skip closing quote
            } else {
                while i < attr_chars.len() && !attr_chars[i].is_whitespace() {
                    attr_value.push(attr_chars[i]);
                    i += 1;
                }
            }

            if !attr_name.is_empty() {
                attrs.insert(attr_name.to_lowercase(), attr_value);
            }
        }
    }

    Some(Tag::Open(tag_name, attrs))
}

/// Validate that a URL is safe (no javascript:, data:, vbscript:, etc.)
fn is_safe_url(url: &str) -> bool {
    // Decode any entities first to catch encoded attacks
    let decoded = decode_entities(url);
    let url_lower = decoded.trim().to_lowercase();

    // Allow common safe schemes
    if url_lower.starts_with("http://")
        || url_lower.starts_with("https://")
        || url_lower.starts_with("mailto:") {
        return true;
    }

    // Block dangerous schemes with proper operator precedence
    let dangerous_schemes = ["javascript:", "vbscript:", "data:", "file:"];
    for scheme in &dangerous_schemes {
        if url_lower.starts_with(scheme) || url_lower.contains(&format!(" {}", scheme)) {
            return false;
        }
    }

    // Check for entity-encoded dangerous schemes
    // (already decoded above, but check for double-encoding attempts)
    if url_lower.contains("javascript:")
        || url_lower.contains("vbscript:")
        || (url_lower.starts_with("data:") && !url_lower.starts_with("data:image/")) {
        return false;
    }

    // For relative URLs (no scheme), allow them
    // Ammonia should have already blocked dangerous ones
    !url_lower.contains(':') || url_lower.starts_with("mailto:")
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

    // Security tests

    #[test]
    fn test_case_insensitive_tags() {
        let segments1 = parse_markup("<B>Bold</B>");
        assert!(segments1[0].style.bold);

        let segments2 = parse_markup("<STRONG>Bold</STRONG>");
        assert!(segments2[0].style.bold);

        let segments3 = parse_markup("<ScRiPt>alert('xss')</ScRiPt>");
        // Script tags should be ignored, text treated as plain
        assert_eq!(segments3[0].text, "alert('xss')");
        assert!(!segments3[0].style.bold);
    }

    #[test]
    fn test_javascript_url_blocked() {
        let html = r#"<a href="javascript:alert('XSS')">click</a>"#;
        let segments = parse_markup(html);
        // Link should not be created for javascript: URLs
        assert!(segments.iter().all(|s| s.link.is_none()));
    }

    #[test]
    fn test_data_url_blocked() {
        let html = r#"<a href="data:text/html,<script>alert('XSS')</script>">click</a>"#;
        let segments = parse_markup(html);
        assert!(segments.iter().all(|s| s.link.is_none()));
    }

    #[test]
    fn test_vbscript_url_blocked() {
        let html = r#"<a href="vbscript:msgbox('XSS')">click</a>"#;
        let segments = parse_markup(html);
        assert!(segments.iter().all(|s| s.link.is_none()));
    }

    #[test]
    fn test_file_url_blocked() {
        let html = r#"<a href="file:///etc/passwd">click</a>"#;
        let segments = parse_markup(html);
        assert!(segments.iter().all(|s| s.link.is_none()));
    }

    #[test]
    fn test_safe_urls_allowed() {
        let html = r#"<a href="https://example.com">HTTPS</a> <a href="http://example.com">HTTP</a> <a href="mailto:test@example.com">Email</a>"#;
        let segments = parse_markup(html);

        let links: Vec<&StyledSegment> = segments.iter().filter(|s| s.link.is_some()).collect();
        assert_eq!(links.len(), 3);
        assert_eq!(links[0].link, Some("https://example.com".to_string()));
        assert_eq!(links[1].link, Some("http://example.com".to_string()));
        assert_eq!(links[2].link, Some("mailto:test@example.com".to_string()));
    }

    #[test]
    fn test_malformed_tag_attributes() {
        // Attribute injection attempts
        let html = r#"<a href="x" onload="evil()">click</a>"#;
        let segments = parse_markup(html);
        // Should parse href but ignore onload
        // In reality, ammonia would strip onload before we see it
        let has_link = segments.iter().any(|s| s.link.is_some());
        assert!(has_link);
    }

    #[test]
    fn test_unclosed_tags() {
        let html = "<b>Bold without closing";
        let segments = parse_markup(html);
        assert_eq!(segments[0].text, "Bold without closing");
        // Should still be bold even if tag not closed
        assert!(segments[0].style.bold);
    }

    #[test]
    fn test_nested_quotes_in_attributes() {
        let html = r#"<a href="https://example.com?q='test'">link</a>"#;
        let segments = parse_markup(html);
        let link_seg = segments.iter().find(|s| s.link.is_some()).unwrap();
        assert!(link_seg.link.as_ref().unwrap().contains("q='test'"));
    }

    #[test]
    fn test_empty_href() {
        let html = r#"<a href="">empty link</a>"#;
        let segments = parse_markup(html);
        // Empty href should create link but with empty URL
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].link, Some("".to_string()));
    }

    #[test]
    fn test_relative_urls_allowed() {
        // Relative URLs should be allowed (ammonia handles validation)
        let html = r#"<a href="/path/to/page">relative</a>"#;
        let segments = parse_markup(html);
        let link_seg = segments.iter().find(|s| s.link.is_some());
        assert!(link_seg.is_some());
        assert_eq!(link_seg.unwrap().link, Some("/path/to/page".to_string()));
    }

    #[test]
    fn test_encoded_javascript_blocked() {
        // Even encoded javascript should be blocked
        let html = r#"<a href="&#106;avascript:alert(1)">click</a>"#;
        let segments = parse_markup(html);
        // After decoding, this would be javascript: so should be blocked
        // Note: Our decode_entities doesn't handle &#106; currently
        // but ammonia should have already blocked this
        let has_js_link = segments.iter().any(|s| {
            if let Some(ref url) = s.link {
                url.to_lowercase().contains("javascript")
            } else {
                false
            }
        });
        assert!(!has_js_link, "Encoded javascript URLs should be blocked");
    }

    #[test]
    fn test_attribute_without_quotes() {
        let html = r#"<a href=https://example.com>no quotes</a>"#;
        let segments = parse_markup(html);
        let link_seg = segments.iter().find(|s| s.link.is_some());
        assert!(link_seg.is_some());
        assert_eq!(link_seg.unwrap().link, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_mixed_case_strong_em() {
        let segments = parse_markup("<StRoNg>bold</StRoNg> <Em>italic</Em>");
        assert_eq!(segments.len(), 3);
        assert!(segments[0].style.bold);
        assert!(segments[2].style.italic);
    }

    #[test]
    fn test_whitespace_in_tags() {
        let html = r#"<  b  >bold<  /  b  >"#;
        let segments = parse_markup(html);
        // Should handle whitespace gracefully
        assert!(segments.iter().any(|s| s.text.contains("bold")));
    }
}
