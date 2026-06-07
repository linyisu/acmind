// ACMind VJudge Importer — Popup Script.

const $ = (sel) => document.querySelector(sel);

async function loadSettings() {
  const result = await chrome.storage.local.get(["apiUrl", "token"]);
  if (result.apiUrl) $("#api-url").value = result.apiUrl;
  if (result.token) $("#token").value = result.token;
}

async function saveSettings() {
  const apiUrl = $("#api-url").value.trim();
  const token = $("#token").value.trim();
  await chrome.storage.local.set({ apiUrl, token });
  setStatus("checking", "Settings saved, checking...");
  setTimeout(checkConnection, 500);
}

function setStatus(state, text) {
  const dot = $("#status-dot");
  const label = $("#status-label");
  dot.className = "status-dot";
  if (state === "connected") dot.classList.add("connected");
  else if (state === "disconnected") dot.classList.add("disconnected");
  label.textContent = text;
}

async function checkConnection() {
  setStatus("checking", "Checking...");

  const { apiUrl } = await chrome.storage.local.get(["apiUrl"]);
  if (!apiUrl) {
    setStatus("disconnected", "No API URL configured");
    return;
  }

  try {
    const resp = await fetch(`${apiUrl.replace(/\/+$/, "")}/health`, {
      method: "GET",
      signal: AbortSignal.timeout(5000),
    });
    if (resp.ok) {
      setStatus("connected", "Connected to ACMind");
    } else {
      setStatus("disconnected", `Server returned ${resp.status}`);
    }
  } catch {
    setStatus("disconnected", "Cannot reach ACMind API");
  }
}

// ---- Init ----
loadSettings();
checkConnection();

$("#btn-save").addEventListener("click", saveSettings);
$("#btn-check").addEventListener("click", checkConnection);
