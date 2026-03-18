/**
 * Matches a URL against an allowed pattern (mirrors rule_matcher.rs :: matches_url).
 *
 * Pattern syntax:
 *   "github.com"                  → exact host, any path
 *   "*.rust-lang.org"             → any subdomain, any path (also matches root)
 *   "*.com"                       → any .com host (not .com.br)
 *   "calendar.google.*"           → any TLD
 *   "youtube.com/watch*"          → host + path glob
 *   "www.youtube.com/watch?v=abc" → host + exact query string
 *   "*"                           → matches everything
 *
 * "*" is a glob wildcard: matches any sequence of characters (including none),
 * just like the "*" wildcard in bash. Query matching is string-glob based.
 */
export function matchesUrl(pattern: string, url: string): boolean {
  // Strip scheme
  url = url.replace(/^https?:\/\//, "");
  // Normalize www.
  url = url.replace(/^www\./, "");
  pattern = pattern.replace(/^www\./, "");

  // Host-only pattern (no "/" and no "*"): implicitly match any path
  if (!pattern.includes("/") && !pattern.includes("*")) {
    pattern = pattern + "*";
  }

  // Special: "*.domain" matches the root domain AND any subdomain.
  // Pure glob fails for the root-domain case ("*" won't match empty prefix before ".").
  if (pattern.startsWith("*.")) {
    const rest = pattern.slice(2);
    return globMatch(rest, url) || globMatch(pattern, url);
  }

  return globMatch(pattern, url);
}

/** Returns true if pattern (which may contain "*" wildcards) matches s. */
function globMatch(pattern: string, s: string): boolean {
  const starIdx = pattern.indexOf("*");
  if (starIdx === -1) {
    return pattern === s;
  }
  const before = pattern.slice(0, starIdx);
  const after = pattern.slice(starIdx + 1);
  if (!s.startsWith(before)) {
    return false;
  }
  const rest = s.slice(before.length);
  if (after === "") {
    return true; // trailing * matches everything remaining
  }
  for (let i = 0; i <= rest.length; i++) {
    if (globMatch(after, rest.slice(i))) {
      return true;
    }
  }
  return false;
}

export function isAllowed(url: string, patterns: string[]): boolean {
  return patterns.some((p) => matchesUrl(p, url));
}

export function isInternalUrl(url: string): boolean {
  return (
    url.startsWith("chrome://") ||
    url.startsWith("chrome-extension://") ||
    url.startsWith("moz-extension://") ||
    url.startsWith("about:") ||
    url.startsWith("http://127.0.0.1:10000")
  );
}

export function isNewTabUrl(url: string): boolean {
  return (
    url.startsWith("chrome://newtab") ||
    url.startsWith("edge://newtab") ||
    url === "about:newtab"
  );
}
