import type { DaemonStatus } from "./types";
import { isAllowed, isInternalUrl, isNewTabUrl } from "./matcher";

const API_URL = "http://127.0.0.1:10000/api/status";
const BLOCK_URL = "http://127.0.0.1:10000/";
const POLL_INTERVAL_MS = 2000;

let focusActive = false;
let allowedPatterns: string[] = [];
let allowNewTab = true;

// ── Daemon polling ──────────────────────────────────────────────────────────

async function pollDaemon(): Promise<void> {
    try {
        const res = await fetch(API_URL);
        if (!res.ok) return;
        const data: DaemonStatus = await res.json();
        focusActive = data.focus_active ?? false;
        allowedPatterns = data.allowed_urls ?? [];
        allowNewTab = data.allow_new_tab ?? true;
    } catch {
        // Daemon not running — treat as focus inactive
        focusActive = false;
        allowedPatterns = [];
        allowNewTab = true;
    }
}

// ── Tab auditing ────────────────────────────────────────────────────────────

async function auditOpenTabs(): Promise<void> {
    if (!focusActive) return;
    const tabs = await chrome.tabs.query({});
    for (const tab of tabs) {
        if (!tab.url || !tab.id) continue;
        if (!allowNewTab && isNewTabUrl(tab.url)) {
            chrome.tabs.update(tab.id, { url: BLOCK_URL });
            continue;
        }
        if (isInternalUrl(tab.url) || isAllowed(tab.url, allowedPatterns))
            continue;
        chrome.tabs.update(tab.id, { url: BLOCK_URL });
    }
}

// ── Navigation interception ─────────────────────────────────────────────────

chrome.webNavigation.onBeforeNavigate.addListener((details) => {
    if (details.frameId !== 0) return;
    if (!focusActive) return;
    if (!allowNewTab && isNewTabUrl(details.url)) {
        chrome.tabs.update(details.tabId, { url: BLOCK_URL });
        return;
    }
    if (isInternalUrl(details.url)) return;
    if (isAllowed(details.url, allowedPatterns)) return;

    chrome.tabs.update(details.tabId, { url: BLOCK_URL });
});

// ── Poll loop ───────────────────────────────────────────────────────────────

setInterval(async () => {
    await pollDaemon();
    await auditOpenTabs();
}, POLL_INTERVAL_MS);

pollDaemon();
