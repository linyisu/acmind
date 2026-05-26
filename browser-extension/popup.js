// ACMind VJudge Importer — Popup script.
// Shows connection status and quick actions.

const statusDot = document.getElementById("status-dot");
const statusLabel = document.getElementById("status-label");
const btnCheck = document.getElementById("btn-check");
const btnSettings = document.getElementById("btn-settings");

async function checkConnection() {
  statusDot.className = "status-dot";
  statusLabel.textContent = "Checking...";

  try {
    const resp = await chrome.runtime.sendMessage({ type: "check-connection" });
    if (resp && resp.available) {
      statusDot.className = "status-dot connected";
      statusLabel.textContent = "ACMind connected ✓";
    } else {
      statusDot.className = "status-dot disconnected";
      statusLabel.textContent = "ACMind not running";
    }
  } catch {
    statusDot.className = "status-dot disconnected";
    statusLabel.textContent = "Cannot reach ACMind";
  }

  btnCheck.disabled = false;
}

btnCheck.addEventListener("click", () => {
  btnCheck.disabled = true;
  checkConnection();
});

btnSettings.addEventListener("click", () => {
  // Open the ACMind app - since we can't directly launch it,
  // we can try to open a deep link or just show info
  chrome.tabs.create({
    url: "https://github.com/mengh04/acmind",
    active: true,
  });
});

// Check on popup open
checkConnection();

// Listen for connection changes
chrome.runtime.onMessage.addListener((msg) => {
  if (msg.type === "acmind-connection-change") {
    if (msg.available) {
      statusDot.className = "status-dot connected";
      statusLabel.textContent = "ACMind connected ✓";
    } else {
      statusDot.className = "status-dot disconnected";
      statusLabel.textContent = "ACMind disconnected";
    }
  }
});
