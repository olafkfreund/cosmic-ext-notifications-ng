use linkify::{LinkFinder, LinkKind};
use crate::NotificationLink;

/// Detect URLs and emails in text
pub fn detect_links(text: &str) -> Vec<NotificationLink> {
  let finder = LinkFinder::new();

  finder.links(text)
    .filter_map(|link| {
      let url = match link.kind() {
        LinkKind::Url => link.as_str().to_string(),
        LinkKind::Email => format!("mailto:{}", link.as_str()),
        _ => return None,
      };

      if !is_safe_url(&url) {
        return None;
      }

      Some(NotificationLink {
        url,
        title: None,
        start: link.start(),
        length: link.end() - link.start(),
      })
    })
    .collect()
}

/// Check if URL is safe to open (http, https, mailto only)
pub fn is_safe_url(url: &str) -> bool {
  let url_lower = url.to_lowercase();
  url_lower.starts_with("https://") ||
  url_lower.starts_with("http://") ||
  url_lower.starts_with("mailto:")
}

/// Open a URL in the default browser/handler
pub fn open_link(url: &str) -> Result<(), std::io::Error> {
  if !is_safe_url(url) {
    return Err(std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      "Unsafe URL scheme"
    ));
  }
  open::that(url)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_detect_https_url() {
    let text = "Check out https://example.com for more";
    let links = detect_links(text);
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].url, "https://example.com");
  }

  #[test]
  fn test_detect_multiple_urls() {
    let text = "Visit https://a.com and https://b.com";
    let links = detect_links(text);
    assert_eq!(links.len(), 2);
  }

  #[test]
  fn test_detect_email() {
    let text = "Contact user@example.com";
    let links = detect_links(text);
    assert_eq!(links.len(), 1);
    assert!(links[0].url.starts_with("mailto:"));
  }

  #[test]
  fn test_is_safe_url() {
    assert!(is_safe_url("https://example.com"));
    assert!(is_safe_url("http://example.com"));
    assert!(is_safe_url("mailto:user@example.com"));
    assert!(!is_safe_url("javascript:alert('xss')"));
    assert!(!is_safe_url("file:///etc/passwd"));
  }

  #[test]
  fn test_no_links_in_plain_text() {
    let text = "Just plain text without any links";
    let links = detect_links(text);
    assert!(links.is_empty());
  }
}
