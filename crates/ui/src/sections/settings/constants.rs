pub const LOCALHOST_URLS: &[&str] = &["localhost", "127.0.0.1", "[::1]", "192.168.*", "10.*"];

pub(super) const WHATSAPP: &str = "web.whatsapp.com";
pub(super) const TELEGRAM: &str = "web.telegram.org";
pub(super) const DISCORD: &str = "discord.com";
pub(super) const SPOTIFY: &str = "open.spotify.com";

pub const SEARCH_ENGINES: &[&str] = &[
    "google.com",
    "bing.com",
    "duckduckgo.com",
    "search.yahoo.com",
    "ecosia.org",
    "startpage.com",
    "search.brave.com",
    "kagi.com",
    "yandex.com",
];

pub const AI_SITES: &[&str] = &[
    "chat.openai.com",
    "claude.ai",
    "gemini.google.com",
    "copilot.microsoft.com",
    "perplexity.ai",
    "grok.com",
    "poe.com",
    "you.com",
    "mistral.ai",
    "huggingface.co",
];

pub(super) fn contains_any(urls: &[String], patterns: &[&str]) -> bool {
    patterns.iter().any(|p| urls.iter().any(|u| u == p))
}

#[cfg(test)]
#[path = "constants_tests.rs"]
mod tests;
