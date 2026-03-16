// Injected only on http://127.0.0.1:10000 — the block page.
// Currently a no-op; can be extended to show the blocked URL or remaining time.

const params = new URLSearchParams(window.location.search);
const blockedUrl = params.get("url");

if (blockedUrl) {
  const el = document.querySelector<HTMLParagraphElement>("p");
  if (el) {
    el.textContent = `${new URL(blockedUrl).hostname} is blocked during your focus session.`;
  }
}
