/// Matches a full URL string against a glob pattern.
///
/// - Strips `https://` / `http://` scheme before matching.
/// - Strips leading `www.` from both pattern and URL.
/// - `*` matches any sequence of characters (including none), like shell globbing.
/// - A pattern with no `/` (host only) implicitly matches any path: `github.com` → `github.com*`.
///
/// Examples:
/// ```
/// assert!(matches_url("netflix.com/*",                    "https://www.netflix.com/watch/123"));
/// assert!(matches_url("app.todoist.com/app*",             "app.todoist.com/app/upcoming?cdn_fallback=1"));
/// assert!(matches_url("app.todoist.com/app/upcoming?*",   "app.todoist.com/app/upcoming?cdn_fallback=1"));
/// assert!(matches_url("calendar.google.*",                "calendar.google.com/calendar/r/week"));
/// ```
#[allow(dead_code)]
pub fn matches_url(pattern: &str, url: &str) -> bool {
    // Strip scheme
    let url = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);

    // Normalize www. on both sides
    let url = url.strip_prefix("www.").unwrap_or(url);
    let pattern = pattern.strip_prefix("www.").unwrap_or(pattern);

    // If the pattern has no path component and no wildcard, any path should match.
    let pattern = if !pattern.contains('/') && !pattern.contains('*') {
        std::borrow::Cow::Owned(format!("{pattern}*"))
    } else {
        std::borrow::Cow::Borrowed(pattern)
    };

    // Special: "*.domain" means "the domain itself OR any subdomain".
    // Pure glob fails for the root-domain case because "*" won't match an empty prefix
    // before the dot (e.g. "*.rust-lang.org" vs "rust-lang.org").
    if let Some(rest) = pattern.strip_prefix("*.") {
        return glob_match(rest, url) || glob_match(&pattern, url);
    }

    glob_match(&pattern, url)
}

/// Returns true if `pattern` (which may contain `*` wildcards) matches `s`.
fn glob_match(pattern: &str, s: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == s,
        Some((before, after)) => {
            if !s.starts_with(before) {
                return false;
            }
            let rest = &s[before.len()..];
            if after.is_empty() {
                return true; // trailing * matches everything remaining
            }
            // Try matching `after` against every suffix of `rest`
            for i in 0..=rest.len() {
                if rest.is_char_boundary(i) && glob_match(after, &rest[i..]) {
                    return true;
                }
            }
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_host() {
        assert!(matches_url("github.com", "github.com"));
        assert!(!matches_url("github.com", "gitlab.com"));
    }

    #[test]
    fn wildcard_subdomain() {
        assert!(matches_url("*.rust-lang.org", "doc.rust-lang.org"));
        assert!(matches_url("*.rust-lang.org", "rust-lang.org"));
        assert!(!matches_url("*.rust-lang.org", "notrust-lang.org"));
    }

    #[test]
    fn wildcard_all() {
        assert!(matches_url("*", "anything.com"));
    }

    #[test]
    fn path_prefix() {
        // exact match
        assert!(matches_url(
            "github.com/torvalds/linux",
            "github.com/torvalds/linux"
        ));
        // sub-paths require an explicit * (e.g. "github.com/torvalds/linux*")
        assert!(!matches_url(
            "github.com/torvalds/linux",
            "github.com/torvalds/linux/commits"
        ));
        // shorter path doesn't match
        assert!(!matches_url(
            "github.com/torvalds/linux",
            "github.com/torvalds"
        ));
        // sibling path doesn't match
        assert!(!matches_url(
            "github.com/torvalds/linux",
            "github.com/torvalds/linux-next"
        ));
    }

    #[test]
    fn www_normalization() {
        // bare host matches_url with or without www.
        assert!(matches_url("netflix.com", "www.netflix.com"));
        assert!(matches_url("netflix.com", "netflix.com"));
        // path glob works through www.
        assert!(matches_url(
            "netflix.com/*",
            "www.netflix.com/watch/81522188/trackId=x"
        ));
        // subdomain wildcard still works
        assert!(matches_url("*.netflix.com", "api.netflix.com"));
        // www. on a non-www host is not fabricated
        assert!(!matches_url("api.netflix.com", "www.netflix.com"));
    }

    #[test]
    fn wildcard_tld() {
        // *.com matches_url any .com host but not .com.br
        assert!(matches_url("*.com", "example.com"));
        assert!(matches_url("*.com", "foo.bar.com"));
        assert!(!matches_url("*.com", "example.com.br"));
        assert!(!matches_url("*.com", "example.net"));
    }

    #[test]
    fn wildcard_trailing_tld() {
        // calendar.google.* matches_url any TLD
        assert!(matches_url("calendar.google.*", "calendar.google.com",));
        assert!(matches_url("calendar.google.*", "calendar.google.co.uk",));
        assert!(!matches_url("calendar.google.*", "mail.google.com",));
    }

    #[test]
    fn wildcard_edge_cases() {
        // calendar.google.* matches_url any TLD
        assert!(matches_url(
            "app.todoist.com/app*",
            "app.todoist.com/app/upcoming?cdn_fallback=1",
        ));
        assert!(matches_url(
            "app.todoist.com/app/upcoming?*",
            "app.todoist.com/app/upcoming?cdn_fallback=1",
        ));
        assert!(!matches_url(
            "app.todoist.com/app*",
            "app.todoist.com/dashboard",
        ));
    }
    #[test]
    fn path_glob() {
        // "watch*" matches /watch, /watches, /watch/anything, /watch?v=x
        assert!(matches_url("youtube.com/watch*", "youtube.com/watch"));
        assert!(matches_url("youtube.com/watch*", "youtube.com/watch?v=abc"));
        assert!(matches_url("youtube.com/watch*", "youtube.com/watches"));
        assert!(matches_url("youtube.com/watch*", "youtube.com/watch/later"));
        assert!(!matches_url("youtube.com/watch*", "youtube.com/channel"));
        // no * = exact match only
        assert!(matches_url("github.com/torvalds", "github.com/torvalds"));
        assert!(!matches_url(
            "github.com/torvalds",
            "github.com/torvalds/linux"
        ));
        assert!(!matches_url(
            "github.com/torvalds",
            "github.com/torvalds-fork"
        ));
    }

    #[test]
    fn query_params() {
        // exact query match
        assert!(matches_url(
            "www.youtube.com/watch?v=abc",
            "youtube.com/watch?v=abc"
        ));
        // * after ? matches any trailing params
        assert!(matches_url(
            "youtube.com/watch?v=abc*",
            "youtube.com/watch?v=abc&feature=share"
        ));
        // wrong video id
        assert!(!matches_url(
            "www.youtube.com/watch?v=abc",
            "youtube.com/watch?v=xyz"
        ));
        // no query at all
        assert!(!matches_url(
            "www.youtube.com/watch?v=abc",
            "youtube.com/watch"
        ));
    }
}
