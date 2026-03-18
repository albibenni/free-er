/**
 * Matches a URL against an allowed pattern (mirrors rule_matcher.rs).
 *
 * Pattern syntax:
 *   "github.com"                  → exact host, any path
 *   "*.rust-lang.org"             → any subdomain, any path
 *   "github.com/torvalds"         → host + path-prefix
 *   "www.youtube.com/watch?v=abc" → host + path-prefix + required query param
 *   "*"                           → matches everything
 *
 * Query params in the pattern are required subsets of the URL's query string.
 */
export function patternMatches(
  pattern: string,
  host: string,
  pathname: string,
  search: string, // e.g. "?v=abc&feature=share" or ""
): boolean {
  if (pattern === "*") return true;

  // Split off query string from pattern
  const qIdx = pattern.indexOf("?");
  const hostPath = qIdx === -1 ? pattern : pattern.slice(0, qIdx);
  const patternQuery = qIdx === -1 ? null : pattern.slice(qIdx + 1);

  // Split host+path
  const slashIdx = hostPath.indexOf("/");
  const hostPat = slashIdx === -1 ? hostPath : hostPath.slice(0, slashIdx);
  const pathPrefix = slashIdx === -1 ? null : hostPath.slice(slashIdx + 1);

  // Match host
  let hostOk: boolean;
  if (hostPat.startsWith("*.")) {
    const suffix = hostPat.slice(2);
    hostOk = host === suffix || host.endsWith("." + suffix);
  } else {
    hostOk = hostPat === host;
  }
  if (!hostOk) return false;

  // Match path prefix
  if (pathPrefix !== null) {
    const full = "/" + pathPrefix;
    if (pathname !== full && !pathname.startsWith(full + "/")) return false;
  }

  // Match query params (all pattern params must appear in the URL)
  if (patternQuery !== null) {
    const urlParams = new URLSearchParams(search);
    const patParams = new URLSearchParams(patternQuery);
    for (const [key, val] of patParams.entries()) {
      if (urlParams.get(key) !== val) return false;
    }
  }

  return true;
}

export function isAllowed(url: string, patterns: string[]): boolean {
  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    return false;
  }
  const { hostname, pathname, search } = parsed;
  return patterns.some((p) => patternMatches(p, hostname, pathname, search));
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
