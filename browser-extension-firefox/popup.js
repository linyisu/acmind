// ACMind VJudge Importer — Popup Script.

const $ = (sel) => document.querySelector(sel);

function setStatus(state, text) {
  const dot = $("#status-dot");
  const label = $("#status-label");
  dot.className = "status-dot";
  if (state === "connected") dot.classList.add("connected");
  else if (state === "disconnected") dot.classList.add("disconnected");
  label.textContent = text;
}

(async () => {
  const { apiUrl, token } = await chrome.storage.local.get(["apiUrl", "token"]);

  if (!apiUrl || !token) {
    setStatus("disconnected", "Not connected — login to ACMind first");
    return;
  }

  try {
    const resp = await fetch(`${apiUrl}/api/v1/auth/me`, {
      headers: { Authorization: `Bearer ${token}` },
      signal: AbortSignal.timeout(5000),
    });
    if (resp.ok) {
      const user = await resp.json();
      setStatus("connected", `Ready — ${user.username}`);
      $("#hint-text").textContent =
        "Go to vjudge.net/status and click Import on the page.";
    } else {
      setStatus("disconnected", "Session expired — re-login to ACMind");
    }
  } catch {
    setStatus("disconnected", `Cannot reach ${apiUrl}`);
  }
})();
