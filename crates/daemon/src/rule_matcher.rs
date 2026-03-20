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
#[path = "rule_matcher_tests.rs"]
mod tests;
