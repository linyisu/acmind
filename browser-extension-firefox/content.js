// ACMind VJudge Importer — Page-context injector.
// This script runs in the content script isolated world. It injects a
// page-context script (vjudge-scraper.js) that has same-origin access to
// vjudge's internal AJAX APIs, then relays scraped data to the extension
// background worker via chrome.runtime.sendMessage.

const ACMIND_PORT = 18921;

// ---- Detect current page type ----
function detectPageType() {
	const href = window.location.href;
	if (href.includes("/status")) return "status";
	if (href.includes("/solution/")) return "solution";
	if (href.includes("/problem/")) return "problem";
	return "other";
}

// ---- Inject page-context script ----
function injectScraper() {
	const script = document.createElement("script");
	script.src = chrome.runtime.getURL("vjudge-scraper.js");
	script.onload = () => script.remove();
	(document.head || document.documentElement).appendChild(script);
}

// ---- Listen for messages from page-context script ----
window.addEventListener("message", (event) => {
	if (event.source !== window) return;
	const msg = event.data;
	if (!msg || msg.source !== "acmind-vjudge-scraper") return;

	if (msg.type === "scraped-data") {
		// Forward to background worker for POST to ACMind app
		chrome.runtime
			.sendMessage({
				type: "import-to-acmind",
				payload: msg.payload,
			})
			.catch(() => {
				// Background might not be ready; retry once
				setTimeout(() => {
					chrome.runtime
						.sendMessage({
							type: "import-to-acmind",
							payload: msg.payload,
						})
						.catch(() => {});
				}, 500);
			});
	} else if (msg.type === "scraper-ready") {
		console.log("[ACMind] Scraper injected and ready");
	} else if (msg.type === "scraper-progress") {
		// Forward progress to popup if open
		chrome.runtime
			.sendMessage({
				type: "import-progress",
				...msg,
			})
			.catch(() => {});
	}
});

// ---- Add UI buttons to the page ----
function addImportButtons() {
	const pageType = detectPageType();

	if (pageType === "status") {
		addStatusPageButtons();
	} else if (pageType === "problem") {
		addProblemPageButtons();
	} else if (pageType === "solution") {
		addSolutionPageButtons();
	}
}

function createButton(text, className, onClick) {
	const btn = document.createElement("button");
	btn.textContent = text;
	btn.className = className;
	btn.addEventListener("click", onClick);
	return btn;
}

function addStatusPageButtons() {
	// Add a floating import bar at the top of the status table
	const container = document.createElement("div");
	container.id = "acmind-import-bar";
	container.style.cssText = `
    display: flex; gap: 8px; align-items: center;
    padding: 8px 16px; margin: 8px 0;
    background: #1a1a2e; border-radius: 8px;
    font-family: -apple-system, BlinkMacSystemFont, sans-serif;
    font-size: 13px; color: #e0e0e0;
  `;

	const label = document.createElement("span");
	label.textContent = "🧠 ACMind Import:";
	label.style.cssText = "font-weight: 600; margin-right: 4px;";
	container.appendChild(label);

	const importCurrentBtn = createButton(
		"Import this page",
		"acmind-btn acmind-btn-primary",
		() => requestScrape("status-page"),
	);
	container.appendChild(importCurrentBtn);

	const importAllBtn = createButton(
		"Import all submissions",
		"acmind-btn acmind-btn-accent",
		() => requestScrape("status-all"),
	);
	container.appendChild(importAllBtn);

	const statusText = document.createElement("span");
	statusText.id = "acmind-status-text";
	statusText.style.cssText = "margin-left: auto; font-size: 12px; color: #888;";
	container.appendChild(statusText);

	// Insert before the status table
	const table = document.querySelector("#status-root, .vjudge_table, table");
	if (table) {
		table.parentNode.insertBefore(container, table);
	} else {
		// Fallback: prepend to body
		document.body.prepend(container);
	}
}

function addProblemPageButtons() {
	// Add import button near the problem title
	const titleEl = document.querySelector(
		"h1, h2, .problem-title, #problem-title",
	);
	const insertAfter = titleEl || document.querySelector(".container, main");
	if (!insertAfter) return;

	const container = document.createElement("div");
	container.id = "acmind-import-bar";
	container.style.cssText = `
    display: flex; gap: 8px; align-items: center;
    padding: 8px 0; margin: 8px 0;
  `;

	const importBtn = createButton(
		"🧠 Import to ACMind",
		"acmind-btn acmind-btn-primary",
		() => requestScrape("problem-page"),
	);
	container.appendChild(importBtn);

	const statusText = document.createElement("span");
	statusText.id = "acmind-status-text";
	statusText.style.cssText = "font-size: 12px; color: #888; margin-left: 8px;";
	container.appendChild(statusText);

	insertAfter.parentNode.insertBefore(container, insertAfter.nextSibling);
}

function addSolutionPageButtons() {
	// Add import button near the solution header
	const headerEl = document.querySelector(
		"#solution-header, .solution-header, h1, h2",
	);
	const insertAfter = headerEl || document.querySelector(".container, main");
	if (!insertAfter) return;

	const container = document.createElement("div");
	container.id = "acmind-import-bar";
	container.style.cssText = `
    display: flex; gap: 8px; align-items: center;
    padding: 8px 0; margin: 8px 0;
  `;

	const importBtn = createButton(
		"🧠 Import this submission",
		"acmind-btn acmind-btn-primary",
		() => requestScrape("solution-page"),
	);
	container.appendChild(importBtn);

	const statusText = document.createElement("span");
	statusText.id = "acmind-status-text";
	statusText.style.cssText = "font-size: 12px; color: #888; margin-left: 8px;";
	container.appendChild(statusText);

	insertAfter.parentNode.insertBefore(container, insertAfter.nextSibling);
}

function requestScrape(mode) {
	setStatusText("⏳ Scraping...");

	window.postMessage(
		{
			source: "acmind-content-script",
			type: "scrape-request",
			mode: mode,
		},
		"*",
	);
}

function setStatusText(text) {
	const el = document.getElementById("acmind-status-text");
	if (el) el.textContent = text;
}

// ---- Listen for scraper responses to update UI ----
window.addEventListener("message", (event) => {
	if (event.source !== window) return;
	const msg = event.data;
	if (!msg || msg.source !== "acmind-vjudge-scraper") return;

	if (msg.type === "scraper-progress") {
		setStatusText(`⏳ ${msg.message}`);
	} else if (msg.type === "scraped-data") {
		const p = msg.payload;
		if (p.type === "status-page" || p.type === "status-all") {
			setStatusText(`✅ Imported ${p.items?.length || 0} submissions`);
		} else if (p.type === "problem-page") {
			setStatusText(`✅ Imported problem${p.title ? ": " + p.title : ""}`);
		} else if (p.type === "solution-page") {
			setStatusText(`✅ Imported submission #${p.runId || "?"}`);
		}
	}
});

// ---- Also handle import-progress from background ----
chrome.runtime.onMessage.addListener((msg) => {
	if (msg.type === "import-progress") {
		setStatusText(`⏳ ${msg.message}`);
	} else if (msg.type === "import-complete") {
		setStatusText(`✅ ${msg.message}`);
	} else if (msg.type === "import-error") {
		setStatusText(`❌ ${msg.message}`);
	}
});

// ---- Init ----
injectScraper();

// Wait for scraper-ready before adding buttons
window.addEventListener("message", function onReady(event) {
	if (event.source !== window) return;
	const msg = event.data;
	if (
		msg &&
		msg.source === "acmind-vjudge-scraper" &&
		msg.type === "scraper-ready"
	) {
		window.removeEventListener("message", onReady);
		addImportButtons();
	}
});
