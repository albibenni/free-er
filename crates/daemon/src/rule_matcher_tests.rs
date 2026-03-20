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
