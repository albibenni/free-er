import type { DaemonStatus } from "../types";

const API_URL = "http://127.0.0.1:10000/api/status";

async function render(): Promise<void> {
  const statusEl = document.getElementById("status")!;
  const listEl = document.getElementById("allowed-list")!;

  try {
    const res = await fetch(API_URL);
    if (!res.ok) throw new Error("bad response");
    const data: DaemonStatus = await res.json();

    statusEl.textContent = data.focus_active ? "Focus active" : "Focus inactive";
    statusEl.className = data.focus_active ? "active" : "inactive";

    listEl.innerHTML = "";
    if (data.focus_active && data.allowed_urls.length > 0) {
      for (const url of data.allowed_urls) {
        const li = document.createElement("li");
        li.textContent = url;
        listEl.appendChild(li);
      }
    } else if (data.focus_active) {
      listEl.innerHTML = "<li><em>All sites blocked</em></li>";
    }
  } catch {
    statusEl.textContent = "Daemon not running";
    statusEl.className = "inactive";
  }
}

render();
