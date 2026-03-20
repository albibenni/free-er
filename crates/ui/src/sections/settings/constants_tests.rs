use super::*;

#[test]
fn contains_any_matches_and_misses() {
    let urls = vec!["discord.com".to_string(), "github.com".to_string()];
    assert!(contains_any(&urls, &[DISCORD]));
    assert!(!contains_any(&urls, &[WHATSAPP, TELEGRAM]));
}
