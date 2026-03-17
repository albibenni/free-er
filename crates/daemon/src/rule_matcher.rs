/// Matches a full URL against an allowed pattern.
///
/// Pattern syntax:
/// - `"github.com"`            → exact host match (any path)
/// - `"*.rust-lang.org"`       → any subdomain (any path)
/// - `"github.com/torvalds"`   → host + path-prefix match
/// - `"*.youtube.com/watch"`   → subdomain + path-prefix match
/// - `"*"`                     → matches everything
#[allow(dead_code)]
pub fn matches(pattern: &str, host: &str, path: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    // Split pattern into host part and optional path prefix
    let (host_pat, path_prefix) = match pattern.split_once('/') {
        Some((h, p)) => (h, Some(p)),
        None => (pattern, None),
    };

    // Match the host portion
    let host_ok = if let Some(suffix) = host_pat.strip_prefix("*.") {
        host == suffix || host.ends_with(&format!(".{suffix}"))
    } else {
        host_pat == host
    };

    if !host_ok {
        return false;
    }

    // If the pattern has a path prefix, the URL path must start with it
    match path_prefix {
        None => true,
        Some(p) => {
            // path always starts with "/"; pattern prefix does not include the leading "/"
            let full_prefix = format!("/{p}");
            path == full_prefix || path.starts_with(&format!("{full_prefix}/"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_host() {
        assert!(matches("github.com", "github.com", "/"));
        assert!(!matches("github.com", "gitlab.com", "/"));
    }

    #[test]
    fn wildcard_subdomain() {
        assert!(matches("*.rust-lang.org", "doc.rust-lang.org", "/"));
        assert!(matches("*.rust-lang.org", "rust-lang.org", "/"));
        assert!(!matches("*.rust-lang.org", "notrust-lang.org", "/"));
    }

    #[test]
    fn wildcard_all() {
        assert!(matches("*", "anything.com", "/foo"));
    }

    #[test]
    fn path_prefix() {
        assert!(matches("github.com/torvalds/linux", "github.com", "/torvalds/linux"));
        assert!(matches("github.com/torvalds/linux", "github.com", "/torvalds/linux/commits"));
        assert!(!matches("github.com/torvalds/linux", "github.com", "/torvalds"));
        assert!(!matches("github.com/torvalds/linux", "github.com", "/torvalds/linux-next"));
    }

    #[test]
    fn subdomain_with_path() {
        assert!(matches("*.youtube.com/watch", "www.youtube.com", "/watch"));
        assert!(matches("*.youtube.com/watch", "www.youtube.com", "/watch/somesubpath"));
        assert!(!matches("*.youtube.com/watch", "www.youtube.com", "/shorts"));
    }
}
