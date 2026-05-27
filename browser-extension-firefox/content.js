// ACMind VJudge Importer — Firefox content injector.
// Mirrors the Chrome content script behavior.

function detectPageType() {
	const href = window.location.href;
	if (href.includes("/status")) return "status";
	if (href.includes("/solution/")) return "solution";
	if (href.includes("/problem/")) return "problem";
	return "other";
}

function injectScraper() {
	const script = document.createElement("script");
	script.src = chrome.runtime.getURL("vjudge-scraper.js");
	script.onload = () => script.remove();
	(document.head || document.documentElement).appendChild(script);
}

async function sendRuntimeMessage(message) {
	return chrome.runtime.sendMessage(message);
}

function setStatusText(text) {
	const el = document.getElementById("acmind-status-text");
	if (el) el.textContent = text;
}

function formatImportSuccess(payload, result) {
	if (result?.message) return result.message;
	if (payload.type === "status-page" || payload.type === "status-all") {
		return `Imported ${payload.items?.length || 0} submissions`;
	}
	if (payload.type === "problem-page") {
		const subCount = payload.submissions?.length || 0;
		return (
			`Imported problem${payload.title ? ": " + payload.title : ""}` +
			(subCount > 0 ? ` + ${subCount} submissions` : "")
		);
	}
	if (payload.type === "solution-page") {
		return `Imported submission #${payload.runId || "?"}`;
	}
	return "Import complete";
}

async function importPayloadToAcmind(payload) {
	setStatusText("⏳ Uploading to ACMind...");

	try {
		const result = await sendRuntimeMessage({
			type: "import-to-acmind",
			payload,
		});

		if (!result?.success) {
			throw new Error(result?.error || result?.message || "Import failed");
		}

		setStatusText(`✅ ${formatImportSuccess(payload, result)}`);
	} catch (error) {
		const message = error instanceof Error ? error.message : String(error);
		setStatusText(`❌ ${message}`);
		console.error("[ACMind] Import failed:", error);
	}
}

function createButton(text, className, onClick) {
	const btn = document.createElement("button");
	btn.textContent = text;
	btn.className = className;
	btn.addEventListener("click", onClick);
	return btn;
}

function requestScrape(mode) {
	setStatusText("⏳ Scraping...");
	window.postMessage(
		{
			source: "acmind-content-script",
			type: "scrape-request",
			mode,
		},
		"*",
	);
}

function addStatusPageButtons() {
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

	container.appendChild(
		createButton("Import this page", "acmind-btn acmind-btn-primary", () =>
			requestScrape("status-page"),
		),
	);
	container.appendChild(
		createButton("Import all submissions", "acmind-btn acmind-btn-accent", () =>
			requestScrape("status-all"),
		),
	);

	const statusText = document.createElement("span");
	statusText.id = "acmind-status-text";
	statusText.style.cssText = "margin-left: auto; font-size: 12px; color: #888;";
	container.appendChild(statusText);

	const table = document.querySelector("#status-root, .vjudge_table, table");
	if (table) {
		table.parentNode.insertBefore(container, table);
	} else {
		document.body.prepend(container);
	}
}

function addProblemPageButtons() {
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

	container.appendChild(
		createButton("🧠 Import to ACMind", "acmind-btn acmind-btn-primary", () =>
			requestScrape("problem-page"),
		),
	);

	const statusText = document.createElement("span");
	statusText.id = "acmind-status-text";
	statusText.style.cssText = "font-size: 12px; color: #888; margin-left: 8px;";
	container.appendChild(statusText);

	insertAfter.parentNode.insertBefore(container, insertAfter.nextSibling);
}

function addSolutionPageButtons() {
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

	container.appendChild(
		createButton(
			"🧠 Import this submission",
			"acmind-btn acmind-btn-primary",
			() => requestScrape("solution-page"),
		),
	);

	const statusText = document.createElement("span");
	statusText.id = "acmind-status-text";
	statusText.style.cssText = "font-size: 12px; color: #888; margin-left: 8px;";
	container.appendChild(statusText);

	insertAfter.parentNode.insertBefore(container, insertAfter.nextSibling);
}

function addImportButtons() {
	const pageType = detectPageType();
	if (pageType === "status") addStatusPageButtons();
	else if (pageType === "problem") addProblemPageButtons();
	else if (pageType === "solution") addSolutionPageButtons();
}

window.addEventListener("message", (event) => {
	if (event.source !== window) return;
	const msg = event.data;
	if (!msg || msg.source !== "acmind-vjudge-scraper") return;

	if (msg.type === "scraped-data") {
		void importPayloadToAcmind(msg.payload);
	} else if (msg.type === "scraper-ready") {
		console.log("[ACMind] Scraper injected and ready");
	} else if (msg.type === "scraper-progress") {
		setStatusText(`⏳ ${msg.message}`);
	} else if (msg.type === "scraper-error") {
		setStatusText(`❌ ${msg.message || "Scraping failed"}`);
	}
});

chrome.runtime.onMessage.addListener((msg) => {
	if (msg.type === "acmind-connection-change") {
		setStatusText(
			msg.available ? "✅ ACMind connected" : "❌ ACMind disconnected",
		);
	}
});

injectScraper();

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
