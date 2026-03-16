/** Matches a URL hostname against a wildcard pattern (mirrors rule_matcher.rs). */
export function patternMatches(pattern: string, host: string): boolean {
  if (pattern === "*") return true;
  if (pattern.startsWith("*.")) {
    const suffix = pattern.slice(2);
    return host === suffix || host.endsWith("." + suffix);
  }
  return pattern === host;
}

export function isAllowed(url: string, patterns: string[]): boolean {
  let host: string;
  try {
    host = new URL(url).hostname;
  } catch {
    return false;
  }
  return patterns.some((p) => patternMatches(p, host));
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
