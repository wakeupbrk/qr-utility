use url::Url;

pub struct UrlValidator;

impl UrlValidator {
    /// Validates whether a string is a valid URL with http or https scheme.
    /// Returns Ok(normalized_url_string) if valid, or Err(reason) if invalid.
    pub fn validate(input: &str) -> Result<String, String> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err("URL cannot be empty.".to_string());
        }

        // Auto-prepend https:// if missing scheme and looks like domain
        let target = if !trimmed.contains("://") {
            format!("https://{}", trimmed)
        } else {
            trimmed.to_string()
        };

        match Url::parse(&target) {
            Ok(parsed) => {
                let scheme = parsed.scheme();
                if scheme != "http" && scheme != "https" {
                    return Err(format!(
                        "Unsupported protocol '{}'. Only http:// and https:// are supported.",
                        scheme
                    ));
                }
                if parsed.host_str().is_none() {
                    return Err("URL must include a valid domain name or IP host.".to_string());
                }
                Ok(parsed.to_string())
            }
            Err(e) => Err(format!("Invalid URL syntax: {}", e)),
        }
    }

    /// Truncate long URL for compact display.
    pub fn truncate(url: &str, max_len: usize) -> String {
        if url.len() <= max_len {
            url.to_string()
        } else {
            format!("{}...", &url[..max_len.saturating_sub(3)])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        assert!(UrlValidator::validate("https://google.com").is_ok());
        assert!(UrlValidator::validate("http://example.org/path?q=1").is_ok());
        assert!(UrlValidator::validate("example.com").is_ok());
    }

    #[test]
    fn test_invalid_urls() {
        assert!(UrlValidator::validate("").is_err());
        assert!(UrlValidator::validate("   ").is_err());
        assert!(UrlValidator::validate("ftp://files.example.com").is_err());
    }
}
