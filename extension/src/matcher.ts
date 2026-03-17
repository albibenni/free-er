/**
 * Matches a URL against an allowed pattern (mirrors rule_matcher.rs).
 *
 * Pattern syntax:
 *   "github.com"              → exact host (any path)
 *   "*.rust-lang.org"         → any subdomain (any path)
 *   "github.com/torvalds"     → host + path-prefix
 *   "*.youtube.com/watch"     → subdomain + path-prefix
 *   "*"                       → matches everything
 */
export function patternMatches(pattern: string, host: string, pathname: string): boolean {
  if (pattern === "*") return true;

  const slashIdx = pattern.indexOf("/");
  const hostPat = slashIdx === -1 ? pattern : pattern.slice(0, slashIdx);
  const pathPrefix = slashIdx === -1 ? null : pattern.slice(slashIdx + 1);

  // Match host
  let hostOk: boolean;
  if (hostPat.startsWith("*.")) {
    const suffix = hostPat.slice(2);
    hostOk = host === suffix || host.endsWith("." + suffix);
  } else {
    hostOk = hostPat === host;
  }

  if (!hostOk) return false;

  // Match path prefix if present
  if (pathPrefix === null) return true;
  const full = "/" + pathPrefix;
  return pathname === full || pathname.startsWith(full + "/");
}

export function isAllowed(url: string, patterns: string[]): boolean {
  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    return false;
  }
  const { hostname, pathname } = parsed;
  return patterns.some((p) => patternMatches(p, hostname, pathname));
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
