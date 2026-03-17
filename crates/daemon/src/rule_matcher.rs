/// Matches a URL hostname against a wildcard pattern.
///
/// Supported syntax:
/// - `"github.com"`        → exact match
/// - `"*.rust-lang.org"`   → any subdomain
/// - `"*"`                 → matches everything
#[allow(dead_code)]
pub fn matches(pattern: &str, host: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(suffix) = pattern.strip_prefix("*.") {
        // *.rust-lang.org matches rust-lang.org and foo.rust-lang.org
        return host == suffix || host.ends_with(&format!(".{suffix}"));
    }
    pattern == host
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match() {
        assert!(matches("github.com", "github.com"));
        assert!(!matches("github.com", "gitlab.com"));
    }

    #[test]
    fn wildcard_subdomain() {
        assert!(matches("*.rust-lang.org", "doc.rust-lang.org"));
        assert!(matches("*.rust-lang.org", "rust-lang.org"));
        assert!(!matches("*.rust-lang.org", "notrust-lang.org"));
    }

    #[test]
    fn wildcard_all() {
        assert!(matches("*", "anything.com"));
    }
}
