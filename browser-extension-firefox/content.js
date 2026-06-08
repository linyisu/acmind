// ACMind VJudge Importer — Content script for vjudge.net.
// Only injects import button on problem pages.
// Scrapes problem info + all user submissions for that problem.

// ---- Detect current page type ----
function detectPageType() {
  const href = window.location.href;
  if (href.includes("/problem/")) return "problem";
  return "other";
}

// ---- Inject page-context scraper script ----
function injectScraper() {
  const script = document.createElement("script");
  script.src = chrome.runtime.getURL("vjudge-scraper.js");
  script.onload = () => script.remove();
  (document.head || document.documentElement).appendChild(script);
}

// ---- Listen for messages from scraper ----
window.addEventListener("message", (event) => {
  if (event.source !== window) return;
  const msg = event.data;
  if (!msg || msg.source !== "acmind-vjudge-scraper") return;

  if (msg.type === "scraped-data") {
    setStatusText("⏳ Uploading to ACMind...");
    chrome.runtime.sendMessage({
      type: "import-to-acmind",
      payload: msg.payload,
    }).then((resp) => {
      if (resp && resp.success) {
        const subs = resp.submissions_imported || 0;
        setStatusText(`✅ Done — ${subs} submissions imported`);
      } else {
        setStatusText(`❌ ${resp?.error || "Import failed"}`);
      }
    }).catch((err) => {
      setStatusText(`❌ ${err.message || "Connection failed"}`);
    });
  } else if (msg.type === "scraper-progress") {
    setStatusText(`⏳ ${msg.message}`);
  } else if (msg.type === "scraper-error") {
    setStatusText(`❌ ${msg.message}`);
  } else if (msg.type === "scraper-ready") {
    console.log("[ACMind] Scraper ready");
  }
});

// ---- Listen for background responses ----
chrome.runtime.onMessage.addListener((msg) => {
  if (msg.type === "import-complete") {
    setStatusText(`✅ ${msg.message}`);
  } else if (msg.type === "import-error") {
    setStatusText(`❌ ${msg.message}`);
  }
});

// ---- Add import button on problem pages ----
function addImportButton() {
  if (detectPageType() !== "problem") return;

  const titleEl = document.querySelector("h1, h2, .problem-title, #problem-title");
  const insertAfter = titleEl || document.querySelector(".container, main");
  if (!insertAfter) return;

  const container = document.createElement("div");
  container.id = "acmind-import-bar";
  container.style.cssText = `
    display: flex; gap: 8px; align-items: center;
    padding: 8px 0; margin: 8px 0;
    font-family: -apple-system, BlinkMacSystemFont, sans-serif;
  `;

  const btn = document.createElement("button");
  btn.textContent = "🧠 Import to ACMind";
  btn.className = "acmind-btn acmind-btn-primary";
  btn.addEventListener("click", () => {
    setStatusText("⏳ Scraping...");
    window.postMessage({
      source: "acmind-content-script",
      type: "scrape-request",
      mode: "problem-full",
    }, "*");
  });
  container.appendChild(btn);

  const statusText = document.createElement("span");
  statusText.id = "acmind-status-text";
  statusText.style.cssText = "font-size: 12px; color: #888; margin-left: 8px;";
  container.appendChild(statusText);

  insertAfter.parentNode.insertBefore(container, insertAfter.nextSibling);
}

function setStatusText(text) {
  const el = document.getElementById("acmind-status-text");
  if (el) el.textContent = text;
}

// ---- Init ----
injectScraper();

window.addEventListener("message", function onReady(event) {
  if (event.source !== window) return;
  const msg = event.data;
  if (msg && msg.source === "acmind-vjudge-scraper" && msg.type === "scraper-ready") {
    window.removeEventListener("message", onReady);
    addImportButton();
  }
});
