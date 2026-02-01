use ammonia::Builder;
use std::collections::HashSet;

/// Sanitize HTML for safe display in notifications.
///
/// Allowed tags: b, i, u, a, br, p
/// Allowed attributes: href (on a tags only)
/// Allowed URL schemes: http, https, mailto
///
/// All dangerous content is stripped:
/// - script, style, iframe, object, embed, img, video, audio tags
/// - event handlers (onclick, onerror, onload, etc.)
/// - dangerous URL schemes (javascript:, data:, vbscript:)
///
/// Links automatically get rel="noopener noreferrer" for security.
pub fn sanitize_html(html: &str) -> String {
  let mut allowed_tags = HashSet::new();
  allowed_tags.insert("b");
  allowed_tags.insert("i");
  allowed_tags.insert("u");
  allowed_tags.insert("a");
  allowed_tags.insert("br");
  allowed_tags.insert("p");

  let mut allowed_attrs = HashSet::new();
  allowed_attrs.insert("href");

  let mut url_schemes = HashSet::new();
  url_schemes.insert("http");
  url_schemes.insert("https");
  url_schemes.insert("mailto");

  Builder::default()
    .tags(allowed_tags)
    .link_rel(Some("noopener noreferrer"))
    .url_schemes(url_schemes)
    .generic_attributes(HashSet::new()) // No global attributes allowed
    .tag_attributes(std::iter::once(("a", allowed_attrs)).collect())
    .clean(html)
    .to_string()
}

/// Check if text contains HTML markup that would be rendered.
///
/// Returns true if the text contains actual HTML tags like <b>, <i>, <u>, <a>, etc.
/// Returns false for plain text or escaped entities.
pub fn has_rich_content(text: &str) -> bool {
  // Match actual HTML tags like <b>, <i>, <u>, <a>, <p>, <br>
  // Don't match escaped entities like &lt;b&gt; or math operators like 5 < 10
  let tag_pattern = regex::Regex::new(r"<\s*/?(?:b|i|u|a|p|br)(?:\s+[^>]*)?>").unwrap();
  tag_pattern.is_match(text)
}

/// Strip all HTML tags, returning plain text.
///
/// This converts HTML entities and removes all markup,
/// leaving only the text content.
pub fn strip_html(html: &str) -> String {
  // FIRST decode HTML entities so we can recognize entity-encoded tags
  // (e.g., &lt;a href=...&gt; becomes <a href=...>)
  let decoded = decode_entities(html);

  // Remove all HTML tags with regex
  let tag_regex = regex::Regex::new(r"<[^>]*>").unwrap();
  let without_tags = tag_regex.replace_all(&decoded, "");

  without_tags.into_owned()
}

/// Extract URLs from href attributes in anchor tags.
///
/// This parses `<a href="...">` tags and extracts the URL from the href attribute.
/// Returns a vector of (url, link_text) tuples.
pub fn extract_hrefs(html: &str) -> Vec<(String, String)> {
  // First decode HTML entities so we can recognize entity-encoded anchor tags
  // (e.g., &lt;a href=&quot;...&quot;&gt; becomes <a href="...">)
  let decoded = decode_entities(html);

  let href_regex = regex::Regex::new(
    r#"<a\s+[^>]*href\s*=\s*["']([^"']+)["'][^>]*>([^<]*)</a>"#
  ).unwrap();

  href_regex
    .captures_iter(&decoded)
    .filter_map(|cap| {
      let url = cap.get(1)?.as_str().to_string();
      let text = cap.get(2)?.as_str().to_string();
      // Only include safe URLs
      if url.starts_with("https://") || url.starts_with("http://") || url.starts_with("mailto:") {
        Some((url, text))
      } else {
        None
      }
    })
    .collect()
}

/// Decode common HTML entities to their character equivalents
fn decode_entities(text: &str) -> String {
  text
    .replace("&lt;", "<")
    .replace("&gt;", ">")
    .replace("&quot;", "\"")
    .replace("&#39;", "'")
    .replace("&#x2F;", "/")
    .replace("&#x27;", "'")
    .replace("&#47;", "/")
    .replace("&#32;", " ")
    .replace("&#61;", "=")
    .replace("&amp;", "&") // Must be last to avoid double-decoding
}

#[cfg(test)]
mod tests {
  use super::*;

  // Tests for sanitize_html

  #[test]
  fn test_preserves_allowed_tags() {
    let input = "<b>bold</b> <i>italic</i> <u>underline</u>";
    let output = sanitize_html(input);
    assert_eq!(output, input, "Should preserve b, i, u tags");
  }

  #[test]
  fn test_preserves_links() {
    let input = r#"<a href="https://example.com">link</a>"#;
    let output = sanitize_html(input);
    assert!(output.contains("<a"), "Should preserve a tag");
    assert!(output.contains("href="), "Should preserve href attribute");
    assert!(output.contains("example.com"), "Should preserve URL");
  }

  #[test]
  fn test_preserves_paragraph_and_br() {
    let input = "<p>paragraph</p>line<br>break";
    let output = sanitize_html(input);
    assert!(output.contains("<p>"), "Should preserve p tag");
    assert!(output.contains("<br>"), "Should preserve br tag");
  }

  #[test]
  fn test_removes_script_tags() {
    let input = r#"Safe text<script>alert('XSS')</script>more text"#;
    let output = sanitize_html(input);
    assert!(!output.contains("<script"), "Should remove script tag");
    assert!(!output.contains("alert"), "Should remove script content");
    assert!(output.contains("Safe text"), "Should keep safe content");
  }

  #[test]
  fn test_removes_style_tags() {
    let input = r#"<style>body { background: red; }</style>Text"#;
    let output = sanitize_html(input);
    assert!(!output.contains("<style"), "Should remove style tag");
    assert!(!output.contains("background"), "Should remove style content");
  }

  #[test]
  fn test_removes_iframe_tags() {
    let input = r#"<iframe src="evil.com"></iframe>Text"#;
    let output = sanitize_html(input);
    assert!(!output.contains("<iframe"), "Should remove iframe tag");
    assert!(!output.contains("evil.com"), "Should remove iframe content");
  }

  #[test]
  fn test_removes_object_and_embed_tags() {
    let input = r#"<object data="evil.swf"></object><embed src="bad.swf">Text"#;
    let output = sanitize_html(input);
    assert!(!output.contains("<object"), "Should remove object tag");
    assert!(!output.contains("<embed"), "Should remove embed tag");
  }

  #[test]
  fn test_removes_img_tags() {
    let input = r#"<img src="image.png" alt="test">Text"#;
    let output = sanitize_html(input);
    assert!(!output.contains("<img"), "Should remove img tag");
  }

  #[test]
  fn test_removes_video_and_audio_tags() {
    let input = r#"<video src="v.mp4"></video><audio src="a.mp3"></audio>Text"#;
    let output = sanitize_html(input);
    assert!(!output.contains("<video"), "Should remove video tag");
    assert!(!output.contains("<audio"), "Should remove audio tag");
  }

  #[test]
  fn test_removes_onclick_handler() {
    let input = r#"<b onclick="alert('XSS')">click me</b>"#;
    let output = sanitize_html(input);
    assert!(!output.contains("onclick"), "Should remove onclick attribute");
    assert!(!output.contains("alert"), "Should remove event handler code");
    assert!(output.contains("<b>"), "Should preserve b tag");
    assert!(output.contains("click me"), "Should preserve text content");
  }

  #[test]
  fn test_removes_onerror_handler() {
    let input = r#"<i onerror="alert('XSS')">text</i>"#;
    let output = sanitize_html(input);
    assert!(!output.contains("onerror"), "Should remove onerror attribute");
  }

  #[test]
  fn test_removes_onload_handler() {
    let input = r#"<p onload="alert('XSS')">text</p>"#;
    let output = sanitize_html(input);
    assert!(!output.contains("onload"), "Should remove onload attribute");
  }

  #[test]
  fn test_blocks_javascript_urls() {
    let input = r#"<a href="javascript:alert('XSS')">click</a>"#;
    let output = sanitize_html(input);
    assert!(!output.contains("javascript:"), "Should block javascript: URLs");
  }

  #[test]
  fn test_blocks_data_urls() {
    let input = r#"<a href="data:text/html,<script>alert('XSS')</script>">click</a>"#;
    let output = sanitize_html(input);
    assert!(!output.contains("data:"), "Should block data: URLs");
  }

  #[test]
  fn test_blocks_vbscript_urls() {
    let input = r#"<a href="vbscript:msgbox('XSS')">click</a>"#;
    let output = sanitize_html(input);
    assert!(!output.contains("vbscript:"), "Should block vbscript: URLs");
  }

  #[test]
  fn test_allows_http_urls() {
    let input = r#"<a href="http://example.com">link</a>"#;
    let output = sanitize_html(input);
    assert!(output.contains("http://example.com"), "Should allow http: URLs");
  }

  #[test]
  fn test_allows_https_urls() {
    let input = r#"<a href="https://example.com">link</a>"#;
    let output = sanitize_html(input);
    assert!(output.contains("https://example.com"), "Should allow https: URLs");
  }

  #[test]
  fn test_allows_mailto_urls() {
    let input = r#"<a href="mailto:test@example.com">email</a>"#;
    let output = sanitize_html(input);
    assert!(output.contains("mailto:test@example.com"), "Should allow mailto: URLs");
  }

  #[test]
  fn test_adds_noopener_noreferrer() {
    let input = r#"<a href="https://example.com">link</a>"#;
    let output = sanitize_html(input);
    assert!(
      output.contains("rel=\"noopener noreferrer\""),
      "Should add rel=\"noopener noreferrer\" to links"
    );
  }

  #[test]
  fn test_only_href_on_links() {
    let input = r#"<b href="bad">bold</b><a href="https://example.com" class="test">link</a>"#;
    let output = sanitize_html(input);
    assert!(!output.contains("href=\"bad\""), "Should not allow href on non-link tags");
    assert!(!output.contains("class="), "Should not allow class attribute on links");
  }

  #[test]
  fn test_complex_attack_vectors() {
    let input = r#"
      <b>Safe</b>
      <script>alert('XSS')</script>
      <a href="javascript:void(0)" onclick="steal()">Bad Link</a>
      <img src=x onerror="alert('XSS')">
      <iframe src="evil.com"></iframe>
      <i>More safe</i>
    "#;
    let output = sanitize_html(input);
    assert!(output.contains("<b>Safe</b>"), "Should preserve safe content");
    assert!(output.contains("<i>More safe</i>"), "Should preserve safe content");
    assert!(!output.contains("<script"), "Should remove all scripts");
    assert!(!output.contains("javascript:"), "Should remove javascript: URLs");
    assert!(!output.contains("onclick"), "Should remove event handlers");
    assert!(!output.contains("<img"), "Should remove images");
    assert!(!output.contains("<iframe"), "Should remove iframes");
  }

  // Tests for has_rich_content

  #[test]
  fn test_has_rich_content_with_bold() {
    assert!(has_rich_content("<b>text</b>"), "Should detect <b> tag");
  }

  #[test]
  fn test_has_rich_content_with_italic() {
    assert!(has_rich_content("<i>text</i>"), "Should detect <i> tag");
  }

  #[test]
  fn test_has_rich_content_with_underline() {
    assert!(has_rich_content("<u>text</u>"), "Should detect <u> tag");
  }

  #[test]
  fn test_has_rich_content_with_link() {
    assert!(
      has_rich_content(r#"<a href="https://example.com">link</a>"#),
      "Should detect <a> tag"
    );
  }

  #[test]
  fn test_has_rich_content_with_paragraph() {
    assert!(has_rich_content("<p>text</p>"), "Should detect <p> tag");
  }

  #[test]
  fn test_has_rich_content_with_br() {
    assert!(has_rich_content("line<br>break"), "Should detect <br> tag");
  }

  #[test]
  fn test_has_rich_content_plain_text() {
    assert!(
      !has_rich_content("Just plain text"),
      "Plain text should not be rich content"
    );
  }

  #[test]
  fn test_has_rich_content_escaped_entities() {
    assert!(
      !has_rich_content("&lt;b&gt;escaped&lt;/b&gt;"),
      "Escaped HTML should not be rich content"
    );
  }

  #[test]
  fn test_has_rich_content_angle_brackets_in_text() {
    assert!(
      !has_rich_content("5 < 10 and 10 > 5"),
      "Math operators should not be rich content"
    );
  }

  // Tests for strip_html

  #[test]
  fn test_strip_html_removes_all_tags() {
    let input = "<b>bold</b> <i>italic</i> <u>underline</u>";
    let output = strip_html(input);
    assert_eq!(output, "bold italic underline", "Should remove all HTML tags");
  }

  #[test]
  fn test_strip_html_with_links() {
    let input = r#"<a href="https://example.com">link text</a>"#;
    let output = strip_html(input);
    assert_eq!(output, "link text", "Should remove link tag but keep text");
  }

  #[test]
  fn test_strip_html_decodes_and_strips_entity_encoded_tags() {
    // Entity-encoded HTML tags should be decoded then stripped
    let input = "&lt;b&gt;text&lt;/b&gt; &amp; more";
    let output = strip_html(input);
    assert_eq!(
      output, "text & more",
      "Should decode entities then strip tags, preserving non-tag text"
    );
  }

  #[test]
  fn test_strip_html_plain_text() {
    let input = "Just plain text";
    let output = strip_html(input);
    assert_eq!(output, input, "Plain text should pass through unchanged");
  }

  #[test]
  fn test_strip_html_complex() {
    let input = r#"<p>Para 1</p><p>Para 2</p><br><b>bold</b>"#;
    let output = strip_html(input);
    assert!(!output.contains("<"), "Should have no HTML tags");
    assert!(output.contains("Para 1"), "Should preserve text content");
    assert!(output.contains("Para 2"), "Should preserve text content");
    assert!(output.contains("bold"), "Should preserve text content");
  }

  // Tests for extract_hrefs

  #[test]
  fn test_extract_hrefs_simple() {
    let input = r#"<a href="https://example.com">link text</a>"#;
    let hrefs = extract_hrefs(input);
    assert_eq!(hrefs.len(), 1);
    assert_eq!(hrefs[0].0, "https://example.com");
    assert_eq!(hrefs[0].1, "link text");
  }

  #[test]
  fn test_extract_hrefs_with_rel_attribute() {
    let input = r#"<a href="https://www.youtube.com/" rel="noopener noreferrer">www.youtube.com</a>"#;
    let hrefs = extract_hrefs(input);
    assert_eq!(hrefs.len(), 1);
    assert_eq!(hrefs[0].0, "https://www.youtube.com/");
    assert_eq!(hrefs[0].1, "www.youtube.com");
  }

  #[test]
  fn test_extract_hrefs_multiple_links() {
    let input = r#"<a href="https://a.com">A</a> and <a href="https://b.com">B</a>"#;
    let hrefs = extract_hrefs(input);
    assert_eq!(hrefs.len(), 2);
    assert_eq!(hrefs[0].0, "https://a.com");
    assert_eq!(hrefs[1].0, "https://b.com");
  }

  #[test]
  fn test_extract_hrefs_filters_unsafe_urls() {
    let input = r#"<a href="javascript:alert('xss')">bad</a> <a href="https://safe.com">good</a>"#;
    let hrefs = extract_hrefs(input);
    assert_eq!(hrefs.len(), 1);
    assert_eq!(hrefs[0].0, "https://safe.com");
  }

  #[test]
  fn test_extract_hrefs_no_links() {
    let input = "Plain text without any links";
    let hrefs = extract_hrefs(input);
    assert!(hrefs.is_empty());
  }

  #[test]
  fn test_extract_hrefs_mailto() {
    let input = r#"<a href="mailto:test@example.com">email us</a>"#;
    let hrefs = extract_hrefs(input);
    assert_eq!(hrefs.len(), 1);
    assert_eq!(hrefs[0].0, "mailto:test@example.com");
  }

  // Tests for entity-encoded HTML (Chrome sends this)

  #[test]
  fn test_strip_html_entity_encoded_anchor() {
    // Chrome sends HTML like this with entity-encoded tags
    let input = "&lt;a href=&quot;https://www.youtube.com/&quot; rel=&quot;noopener noreferrer&quot;&gt;www.youtube.com&lt;/a&gt;";
    let output = strip_html(input);
    assert_eq!(output, "www.youtube.com", "Should decode entities then strip tags");
  }

  #[test]
  fn test_strip_html_entity_encoded_with_text() {
    // Chrome notification body with entity-encoded anchor and text
    let input = "&lt;a href=&quot;https://www.youtube.com/&quot;&gt;www.youtube.com&lt;/a&gt;Video Title Here";
    let output = strip_html(input);
    assert_eq!(output, "www.youtube.comVideo Title Here", "Should decode and strip, preserving text");
  }

  #[test]
  fn test_extract_hrefs_entity_encoded() {
    // Chrome sends HTML with entity-encoded attributes
    let input = "&lt;a href=&quot;https://www.youtube.com/&quot; rel=&quot;noopener noreferrer&quot;&gt;www.youtube.com&lt;/a&gt;";
    let hrefs = extract_hrefs(input);
    assert_eq!(hrefs.len(), 1, "Should find entity-encoded anchor");
    assert_eq!(hrefs[0].0, "https://www.youtube.com/");
    assert_eq!(hrefs[0].1, "www.youtube.com");
  }

  #[test]
  fn test_extract_hrefs_mixed_regular_and_encoded() {
    // Mix of regular and entity-encoded anchors
    let input = r#"<a href="https://a.com">A</a> and &lt;a href=&quot;https://b.com&quot;&gt;B&lt;/a&gt;"#;
    let hrefs = extract_hrefs(input);
    assert_eq!(hrefs.len(), 2, "Should find both regular and encoded anchors");
    assert_eq!(hrefs[0].0, "https://a.com");
    assert_eq!(hrefs[1].0, "https://b.com");
  }
}
