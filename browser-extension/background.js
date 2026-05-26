// ACMind VJudge Importer — Background Service Worker.
// Receives scraped data from the content script and POSTs it to the
// ACMind desktop app's local HTTP server (127.0.0.1:18921).

const ACMIND_BASE = "http://127.0.0.1:18921";
const MAX_RETRIES = 3;
const RETRY_DELAY_MS = 1000;

// Track active connections
let acmindAvailable = false;
let lastCheck = 0;

async function checkAcmindAvailable() {
	const now = Date.now();
	if (now - lastCheck < 5000) return acmindAvailable; // cache for 5s

	try {
		const resp = await fetch(`${ACMIND_BASE}/health`, {
			method: "GET",
			signal: AbortSignal.timeout(2000),
		});
		acmindAvailable = resp.ok;
	} catch {
		acmindAvailable = false;
	}
	lastCheck = now;
	return acmindAvailable;
}

async function postToAcmind(endpoint, data, retries = MAX_RETRIES) {
	for (let attempt = 0; attempt < retries; attempt++) {
		try {
			const resp = await fetch(`${ACMIND_BASE}${endpoint}`, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(data),
				signal: AbortSignal.timeout(30000),
			});

			if (resp.ok) {
				const result = await resp.json().catch(() => ({}));
				return { success: true, ...result };
			}

			const errorText = await resp.text().catch(() => "Unknown error");
			console.error(`[ACMind] Server error ${resp.status}: ${errorText}`);
			throw new Error(`Server error: ${resp.status}`);
		} catch (err) {
			console.error(
				`[ACMind] POST attempt ${attempt + 1} failed:`,
				err.message,
			);
			if (attempt < retries - 1) {
				await sleep(RETRY_DELAY_MS * (attempt + 1));
			} else {
				throw err;
			}
		}
	}
}

function sleep(ms) {
	return new Promise((resolve) => setTimeout(resolve, ms));
}

// ---- Message handlers ----
chrome.runtime.onMessage.addListener((msg, sender, sendResponse) => {
	if (msg.type === "import-to-acmind") {
		handleImport(msg.payload)
			.then(sendResponse)
			.catch((err) => {
				sendResponse({ success: false, error: err.message });
			});
		return true; // keep channel open for async response
	}

	if (msg.type === "check-connection") {
		checkAcmindAvailable()
			.then((available) => {
				sendResponse({ available });
			})
			.catch(() => {
				sendResponse({ available: false });
			});
		return true;
	}

	return false;
});

async function handleImport(payload) {
	const available = await checkAcmindAvailable();
	if (!available) {
		throw new Error(
			"ACMind app is not running or the import server is not started. " +
				"Please launch ACMind and ensure the import server is active.",
		);
	}

	let endpoint, data;

	switch (payload.type) {
		case "status-page":
		case "status-all":
			endpoint = "/vjudge/import/submissions";
			data = {
				username: payload.username,
				items: payload.items,
				includeSource: false, // source code fetched separately
			};
			break;

		case "problem-page":
			endpoint = "/vjudge/import/problem";
			data = {
				sourceProblemId: payload.sourceProblemId,
				oj: payload.oj,
				probNum: payload.probNum,
				title: payload.title,
				url: payload.url,
				statement: payload.statement,
				tags: payload.tags,
			};
			break;

		case "solution-page":
			endpoint = "/vjudge/import/submission";
			data = {
				runId: payload.runId,
				oj: payload.oj,
				probNum: payload.probNum,
				status: payload.status,
				language: payload.language,
				code: payload.code,
				runtime: payload.runtime,
				memory: payload.memory,
				submitTime: payload.submitTime,
			};
			break;

		default:
			throw new Error(`Unknown payload type: ${payload.type}`);
	}

	return postToAcmind(endpoint, data);
}

// ---- Notify content scripts about ACMind availability changes ----
// Periodically check and broadcast status
setInterval(async () => {
	const wasAvailable = acmindAvailable;
	await checkAcmindAvailable();
	if (wasAvailable !== acmindAvailable) {
		// Broadcast to all vjudge tabs
		const tabs = await chrome.tabs.query({ url: "https://vjudge.net/*" });
		for (const tab of tabs) {
			chrome.tabs
				.sendMessage(tab.id, {
					type: "acmind-connection-change",
					available: acmindAvailable,
				})
				.catch(() => {});
		}
	}
}, 10000);

console.log("[ACMind] Background service worker started.");
