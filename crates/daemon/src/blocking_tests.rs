use super::*;

#[tokio::test]
async fn apply_blocked_domains_with_empty_list_returns_ok() {
    assert!(apply_blocked_domains(&[]).await.is_ok());
}

#[tokio::test]
async fn apply_blocked_domains_with_entries_returns_ok() {
    assert!(apply_blocked_domains(&["example.com".to_string(), "social.net".to_string()])
        .await
        .is_ok());
}

#[tokio::test]
async fn clear_blocked_domains_returns_ok() {
    assert!(clear_blocked_domains().await.is_ok());
}
