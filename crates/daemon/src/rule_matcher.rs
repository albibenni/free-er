/// Matches a URL against an allowed pattern.
///
/// Pattern syntax:
/// - `"github.com"`                    → exact host, any path
/// - `"*.rust-lang.org"`               → any subdomain, any path
/// - `"*.com"`                         → any `.com` host (not `.com.br`)
/// - `"github.com/torvalds"`           → host + exact path-prefix (sub-paths also match)
/// - `"youtube.com/watch*"`            → host + path glob (matches `/watch`, `/watch?v=x`, `/watches`)
/// - `"www.youtube.com/watch?v=abc"`   → host + path-prefix + required query param
/// - `"*"`                             → matches everything
///
/// Query params in the pattern are treated as required subsets:
/// the URL must contain all the pattern's query params (extra params are allowed).
#[allow(dead_code)]
pub fn matches(pattern: &str, host: &str, path: &str, query: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    // Split pattern into host+path part and optional query string
    let (host_path, pattern_query) = match pattern.split_once('?') {
        Some((hp, q)) => (hp, Some(q)),
        None => (pattern, None),
    };

    // Split host+path into host part and optional path prefix
    let (host_pat, path_prefix) = match host_path.split_once('/') {
        Some((h, p)) => (h, Some(p)),
        None => (host_path, None),
    };

    // Match host
    let host_ok = if let Some(suffix) = host_pat.strip_prefix("*.") {
        host == suffix || host.ends_with(&format!(".{suffix}"))
    } else {
        host_pat == host
    };
    if !host_ok {
        return false;
    }

    // Match path prefix
    if let Some(p) = path_prefix {
        if let Some(prefix) = p.strip_suffix('*') {
            // Glob: path must start with the prefix (e.g. "watch*" matches "/watch", "/watches", "/watch?v=x")
            if !path.starts_with(&format!("/{prefix}")) {
                return false;
            }
        } else {
            // Exact prefix: "/torvalds" or "/torvalds/anything" — but NOT "/torvalds-fork"
            let full_prefix = format!("/{p}");
            if path != full_prefix && !path.starts_with(&format!("{full_prefix}/")) {
                return false;
            }
        }
    }

    // Match query params (pattern params must all be present in the URL)
    if let Some(pq) = pattern_query {
        for pair in pq.split('&') {
            if !query.split('&').any(|u| u == pair) {
                return false;
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_host() {
        assert!(matches("github.com", "github.com", "/", ""));
        assert!(!matches("github.com", "gitlab.com", "/", ""));
    }

    #[test]
    fn wildcard_subdomain() {
        assert!(matches("*.rust-lang.org", "doc.rust-lang.org", "/", ""));
        assert!(matches("*.rust-lang.org", "rust-lang.org", "/", ""));
        assert!(!matches("*.rust-lang.org", "notrust-lang.org", "/", ""));
    }

    #[test]
    fn wildcard_all() {
        assert!(matches("*", "anything.com", "/foo", ""));
    }

    #[test]
    fn path_prefix() {
        assert!(matches("github.com/torvalds/linux", "github.com", "/torvalds/linux", ""));
        assert!(matches("github.com/torvalds/linux", "github.com", "/torvalds/linux/commits", ""));
        assert!(!matches("github.com/torvalds/linux", "github.com", "/torvalds", ""));
        assert!(!matches("github.com/torvalds/linux", "github.com", "/torvalds/linux-next", ""));
    }

    #[test]
    fn wildcard_tld() {
        // *.com matches any .com host but not .com.br
        assert!(matches("*.com", "example.com", "/", ""));
        assert!(matches("*.com", "foo.bar.com", "/", ""));
        assert!(!matches("*.com", "example.com.br", "/", ""));
        assert!(!matches("*.com", "example.net", "/", ""));
    }

    #[test]
    fn path_glob() {
        // youtube.com/watch* matches /watch, /watches, /watch/anything, /watch?v=x
        assert!(matches("youtube.com/watch*", "youtube.com", "/watch", ""));
        assert!(matches("youtube.com/watch*", "youtube.com", "/watch", "v=abc"));
        assert!(matches("youtube.com/watch*", "youtube.com", "/watches", ""));
        assert!(matches("youtube.com/watch*", "youtube.com", "/watch/later", ""));
        assert!(!matches("youtube.com/watch*", "youtube.com", "/channel", ""));
        // plain prefix (no *) still requires exact segment boundary
        assert!(matches("github.com/torvalds", "github.com", "/torvalds", ""));
        assert!(matches("github.com/torvalds", "github.com", "/torvalds/linux", ""));
        assert!(!matches("github.com/torvalds", "github.com", "/torvalds-fork", ""));
    }

    #[test]
    fn query_params() {
        assert!(matches("www.youtube.com/watch?v=abc", "www.youtube.com", "/watch", "v=abc"));
        // extra params in URL are fine
        assert!(matches("www.youtube.com/watch?v=abc", "www.youtube.com", "/watch", "v=abc&feature=share"));
        // wrong video id
        assert!(!matches("www.youtube.com/watch?v=abc", "www.youtube.com", "/watch", "v=xyz"));
        // no query at all
        assert!(!matches("www.youtube.com/watch?v=abc", "www.youtube.com", "/watch", ""));
    }
}
