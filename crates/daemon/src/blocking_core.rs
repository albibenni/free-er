/// Placeholder for the blocking coordinator.
///
/// Phase 1 default is browser-extension-only blocking (Option B from the plan),
/// so this module is a no-op stub. Option A (/etc/hosts) will be implemented later.
use anyhow::Result;

#[allow(dead_code)]
pub async fn apply_blocked_domains(_domains: &[String]) -> Result<()> {
    // TODO: write managed section to /etc/hosts (Option A)
    Ok(())
}

#[allow(dead_code)]
pub async fn clear_blocked_domains() -> Result<()> {
    // TODO: remove managed section from /etc/hosts
    Ok(())
}
