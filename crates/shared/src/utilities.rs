use url::Url;

/// URL validation utilities
pub struct UrlValidator;

impl UrlValidator {
    /// Validates the URL for HTTPS.
    /// Helps validate if the URL is a valid HTTPS URL.
    pub fn is_valid_url_https(url: &str) -> bool {
        match Url::parse(url) {
            Ok(parsed_url) => {
                parsed_url.scheme() == "https" && parsed_url.port_or_known_default() == Some(443)
            }
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_https_url() {
        assert!(UrlValidator::is_valid_url_https("https://example.com"));
        assert!(UrlValidator::is_valid_url_https("https://example.com:443"));
    }

    #[test]
    fn test_invalid_https_url() {
        assert!(!UrlValidator::is_valid_url_https("http://example.com"));
        assert!(!UrlValidator::is_valid_url_https("https://example.com:8080"));
        assert!(!UrlValidator::is_valid_url_https("not-a-url"));
    }
}

