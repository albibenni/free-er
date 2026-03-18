import { describe, it, expect } from "vitest";
import { matchesUrl, isAllowed } from "./matcher";

describe("exact_host", () => {
  it("matches same host", () => expect(matchesUrl("github.com", "github.com")).toBe(true));
  it("does not match different host", () =>
    expect(matchesUrl("github.com", "gitlab.com")).toBe(false));
});

describe("wildcard_subdomain", () => {
  it("matches subdomain", () =>
    expect(matchesUrl("*.rust-lang.org", "doc.rust-lang.org")).toBe(true));
  it("matches root domain", () =>
    expect(matchesUrl("*.rust-lang.org", "rust-lang.org")).toBe(true));
  it("does not match wrong domain", () =>
    expect(matchesUrl("*.rust-lang.org", "notrust-lang.org")).toBe(false));
});

describe("wildcard_all", () => {
  it("* matches anything", () => expect(matchesUrl("*", "anything.com")).toBe(true));
});

describe("path_prefix", () => {
  it("exact match", () =>
    expect(matchesUrl("github.com/torvalds/linux", "github.com/torvalds/linux")).toBe(true));
  it("sub-paths require explicit *", () =>
    expect(matchesUrl("github.com/torvalds/linux", "github.com/torvalds/linux/commits")).toBe(false));
  it("shorter path does not match", () =>
    expect(matchesUrl("github.com/torvalds/linux", "github.com/torvalds")).toBe(false));
  it("sibling path does not match", () =>
    expect(matchesUrl("github.com/torvalds/linux", "github.com/torvalds/linux-next")).toBe(false));
});

describe("www_normalization", () => {
  it("bare host matches with www.", () =>
    expect(matchesUrl("netflix.com", "www.netflix.com")).toBe(true));
  it("bare host matches without www.", () =>
    expect(matchesUrl("netflix.com", "netflix.com")).toBe(true));
  it("path glob works through www.", () =>
    expect(matchesUrl("netflix.com/*", "www.netflix.com/watch/81522188/trackId=x")).toBe(true));
  it("subdomain wildcard still works", () =>
    expect(matchesUrl("*.netflix.com", "api.netflix.com")).toBe(true));
  it("www. on non-www host is not fabricated", () =>
    expect(matchesUrl("api.netflix.com", "www.netflix.com")).toBe(false));
});

describe("wildcard_tld", () => {
  it("*.com matches .com host", () =>
    expect(matchesUrl("*.com", "example.com")).toBe(true));
  it("*.com matches subdomain.com", () =>
    expect(matchesUrl("*.com", "foo.bar.com")).toBe(true));
  it("*.com does not match .com.br", () =>
    expect(matchesUrl("*.com", "example.com.br")).toBe(false));
  it("*.com does not match .net", () =>
    expect(matchesUrl("*.com", "example.net")).toBe(false));
});

describe("wildcard_trailing_tld", () => {
  it("calendar.google.* matches .com", () =>
    expect(matchesUrl("calendar.google.*", "calendar.google.com")).toBe(true));
  it("calendar.google.* matches .co.uk", () =>
    expect(matchesUrl("calendar.google.*", "calendar.google.co.uk")).toBe(true));
  it("calendar.google.* does not match mail.google.com", () =>
    expect(matchesUrl("calendar.google.*", "mail.google.com")).toBe(false));
});

describe("wildcard_edge_cases", () => {
  it("app path glob matches with query", () =>
    expect(
      matchesUrl("app.todoist.com/app*", "app.todoist.com/app/upcoming?cdn_fallback=1"),
    ).toBe(true));
  it("query glob matches", () =>
    expect(
      matchesUrl("app.todoist.com/app/upcoming?*", "app.todoist.com/app/upcoming?cdn_fallback=1"),
    ).toBe(true));
  it("app path glob does not match different path", () =>
    expect(matchesUrl("app.todoist.com/app*", "app.todoist.com/dashboard")).toBe(false));
});

describe("path_glob", () => {
  it("watch* matches /watch", () =>
    expect(matchesUrl("youtube.com/watch*", "youtube.com/watch")).toBe(true));
  it("watch* matches /watch?v=abc", () =>
    expect(matchesUrl("youtube.com/watch*", "youtube.com/watch?v=abc")).toBe(true));
  it("watch* matches /watches", () =>
    expect(matchesUrl("youtube.com/watch*", "youtube.com/watches")).toBe(true));
  it("watch* matches /watch/later", () =>
    expect(matchesUrl("youtube.com/watch*", "youtube.com/watch/later")).toBe(true));
  it("watch* does not match /channel", () =>
    expect(matchesUrl("youtube.com/watch*", "youtube.com/channel")).toBe(false));
  it("no * = exact match", () =>
    expect(matchesUrl("github.com/torvalds", "github.com/torvalds")).toBe(true));
  it("no * does not match sub-path", () =>
    expect(matchesUrl("github.com/torvalds", "github.com/torvalds/linux")).toBe(false));
  it("no * does not match sibling", () =>
    expect(matchesUrl("github.com/torvalds", "github.com/torvalds-fork")).toBe(false));
});

describe("query_params", () => {
  it("exact query match", () =>
    expect(matchesUrl("www.youtube.com/watch?v=abc", "youtube.com/watch?v=abc")).toBe(true));
  it("* after ? matches any trailing params", () =>
    expect(
      matchesUrl("youtube.com/watch?v=abc*", "youtube.com/watch?v=abc&feature=share"),
    ).toBe(true));
  it("wrong video id does not match", () =>
    expect(matchesUrl("www.youtube.com/watch?v=abc", "youtube.com/watch?v=xyz")).toBe(false));
  it("no query does not match pattern with query", () =>
    expect(matchesUrl("www.youtube.com/watch?v=abc", "youtube.com/watch")).toBe(false));
});

describe("isAllowed", () => {
  it("returns true when any pattern matches", () =>
    expect(isAllowed("https://github.com/torvalds", ["gitlab.com", "github.com"])).toBe(true));
  it("returns false when no pattern matches", () =>
    expect(isAllowed("https://evil.com", ["github.com", "gitlab.com"])).toBe(false));
  it("full https URL is matched correctly", () =>
    expect(matchesUrl("netflix.com/*", "https://www.netflix.com/watch/123")).toBe(true));
});
