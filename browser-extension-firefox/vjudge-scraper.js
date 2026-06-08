// ACMind VJudge Scraper — Page-context script injected into vjudge.net.
// Runs in the page's main world with same-origin access to vjudge's AJAX APIs.
// Communicates with the content script via window.postMessage.

(function () {
	"use strict";

	const VJUDGE_ORIGIN = "https://vjudge.net";
	const PAGE_SIZE = 20;

	// ---- Messaging ----
	function postToContent(msg) {
		window.postMessage({ source: "acmind-vjudge-scraper", ...msg }, "*");
	}

	function progress(message) {
		postToContent({ type: "scraper-progress", message });
	}

	function sendData(payload) {
		postToContent({ type: "scraped-data", payload });
	}

	// ---- Fetch helpers ----
	async function fetchJSON(url, options = {}) {
		const resp = await fetch(url, {
			credentials: "include",
			headers: {
				"X-Requested-With": "XMLHttpRequest",
				Accept: "application/json, text/javascript, */*; q=0.01",
			},
			...options,
		});
		if (!resp.ok) {
			throw new Error(`HTTP ${resp.status} for ${url}`);
		}
		return resp.json();
	}

	async function fetchText(url) {
		const resp = await fetch(url, {
			credentials: "include",
			headers: {
				"X-Requested-With": "XMLHttpRequest",
				Accept: "text/html, */*",
			},
		});
		if (!resp.ok) {
			throw new Error(`HTTP ${resp.status} for ${url}`);
		}
		return resp.text();
	}

	// ---- Extract metadata from page ----
	function extractPageJson(html, marker) {
		const markerIdx = html.indexOf(marker);
		if (markerIdx === -1) return null;
		const textareaStart = html.lastIndexOf("<textarea", markerIdx);
		if (textareaStart === -1) return null;
		const openEnd = html.indexOf(">", textareaStart) + 1;
		const closeStart = html.indexOf("</textarea>", openEnd);
		if (closeStart === -1) return null;
		const raw = html.substring(openEnd, closeStart).trim();
		try {
			return JSON.parse(
				raw
					.replace(/&lt;/g, "<")
					.replace(/&gt;/g, ">")
					.replace(/&amp;/g, "&")
					.replace(/&quot;/g, '"')
					.replace(/&#39;/g, "'"),
			);
		} catch {
			return null;
		}
	}

	function extractTitle(html, sourceId) {
		const titleTag = html.match(/<title>([^<]*)<\/title>/);
		if (!titleTag) return sourceId;
		const title = titleTag[1]
			.replace(/&lt;/g, "<")
			.replace(/&gt;/g, ">")
			.replace(/&amp;/g, "&")
			.replace(/&quot;/g, '"');
		return title.split(` - ${sourceId}`)[0].trim() || sourceId;
	}

	// ---- Scrape: status page (current visible page) ----
	async function scrapeStatusPage() {
		const username = extractUsernameFromPage();
		if (!username) {
			throw new Error("Could not determine VJudge username from page");
		}

		// Try to get current page params from the DataTable
		const pageIdx = getCurrentStatusPage();
		const start = pageIdx * PAGE_SIZE;

		progress(`Fetching submissions page ${pageIdx + 1}...`);

		const params = new URLSearchParams({
			draw: "1",
			start: String(start),
			length: String(PAGE_SIZE),
			un: username,
			OJId: "All",
			probNum: "",
			res: "all",
			language: "",
			onlyFollowee: "false",
			orderBy: "run_id",
			// Also get the hidden columns
		});

		const url = `${VJUDGE_ORIGIN}/status/data?${params.toString()}`;
		const resp = await fetchJSON(url);

		if (!resp.data || !Array.isArray(resp.data)) {
			throw new Error("Unexpected response from VJudge status API");
		}

		const items = resp.data.map(parseSubmissionItem);
		return {
			type: "status-page",
			username,
			items,
			page: pageIdx,
			total: resp.recordsTotal || items.length,
		};
	}

	// ---- Scrape: all status pages ----
	async function scrapeStatusAll() {
		const username = extractUsernameFromPage();
		if (!username) {
			throw new Error("Could not determine VJudge username from page");
		}

		const allItems = [];
		for (let page = 0; page < 200; page++) {
			const start = page * PAGE_SIZE;
			progress(`Fetching submissions page ${page + 1}...`);
			const params = new URLSearchParams({
				draw: String(page + 1),
				start: String(start),
				length: String(PAGE_SIZE),
				un: username,
				OJId: "All",
				probNum: "",
				res: "all",
				language: "",
				onlyFollowee: "false",
				orderBy: "run_id",
			});

			const url = `${VJUDGE_ORIGIN}/status/data?${params.toString()}`;
			const resp = await fetchJSON(url);

			if (!resp.data || !Array.isArray(resp.data) || resp.data.length === 0) {
				break;
			}

			const items = resp.data.map(parseSubmissionItem);
			allItems.push(...items);

			// Delay to avoid rate limiting
			if (page < 199) {
				await sleep(300);
			}
		}

		return {
			type: "status-all",
			username,
			items: allItems,
			total: allItems.length,
		};
	}

	// ---- Scrape: problem page (description + metadata) ----
	async function scrapeProblemPage() {
		const sourceProblemId = extractSourceProblemId();
		if (!sourceProblemId) {
			throw new Error("Could not determine problem ID from URL");
		}

		progress(`Fetching problem ${sourceProblemId}...`);

		// Fetch the problem page HTML
		const html = await fetchText(`${VJUDGE_ORIGIN}/problem/${sourceProblemId}`);

		const data = extractPageJson(html, "dataJson");
		const title = extractTitle(html, sourceProblemId);
		const [oj, probNum] = sourceProblemId.split("-");

		// Try to fetch description
		let statement = null;
		if (data && data.descBriefs && data.descBriefs.length > 0) {
			const desc = data.descBriefs[0];
			try {
				const descHtml = await fetchText(
					`${VJUDGE_ORIGIN}/problem/description/${desc.key}?${desc.version}`,
				);
				const descData = extractPageJson(descHtml, "data-json-container");
				if (descData && descData.sections) {
					statement = descData.sections
						.map((sec) => {
							const title = sec.title ? `## ${sec.title}\n\n` : "";
							const content = stripHtml(sec.value?.content || "");
							return title + content;
						})
						.join("\n\n");
				}
			} catch (e) {
				// Description might be in a different format — try alternative path
				progress(`Description fetch failed: ${e.message}, trying fallback...`);
				const descPathMatch = html.match(/\/problem\/description\/\d+/);
				if (descPathMatch) {
					try {
						const descHtml = await fetchText(
							`${VJUDGE_ORIGIN}${descPathMatch[0]}`,
						);
						const descData = extractPageJson(descHtml, "data-json-container");
						if (descData && descData.sections) {
							statement = descData.sections
								.map((sec) => {
									const title = sec.title ? `## ${sec.title}\n\n` : "";
									const content = stripHtml(sec.value?.content || "");
									return title + content;
								})
								.join("\n\n");
						}
					} catch {}
				}
			}
		}

		return {
			type: "problem-page",
			sourceProblemId,
			oj: data?.oj || oj,
			probNum: data?.prob || probNum,
			title: title,
			url: `${VJUDGE_ORIGIN}/problem/${sourceProblemId}`,
			statement: statement,
			tags: data?.oj ? [data.oj] : [oj],
			rawData: data,
		};
	}

	// ---- Scrape: solution page (source code + submission metadata) ----
	async function scrapeSolutionPage() {
		const runId = extractRunId();
		if (!runId) {
			throw new Error("Could not determine run ID from URL");
		}

		progress(`Fetching solution #${runId}...`);

		// POST to solution/data to get source code and full metadata
		const url = `${VJUDGE_ORIGIN}/solution/data/${runId}?inPage=true`;
		const resp = await fetch(url, {
			method: "POST",
			credentials: "include",
			headers: {
				"Content-Type": "application/x-www-form-urlencoded; charset=UTF-8",
				"X-Requested-With": "XMLHttpRequest",
				Accept: "application/json, text/javascript, */*; q=0.01",
				Origin: VJUDGE_ORIGIN,
				Referer: `${VJUDGE_ORIGIN}/solution/${runId}`,
			},
			body: "shareCode=",
		});

		if (!resp.ok) {
			throw new Error(`HTTP ${resp.status} for solution/${runId}`);
		}

		const data = await resp.json();

		if (!data.code && data.codeAccessInfo) {
			throw new Error(
				`Source code not accessible: ${data.codeAccessInfo.i18nKey || "unknown"}`,
			);
		}

		return {
			type: "solution-page",
			runId: data.runId || runId,
			oj: data.oj || "",
			probNum: data.probNum || "",
			status: data.status || "",
			language: data.language || "",
			code: data.code || "",
			runtime: data.runtime || null,
			memory: data.memory || null,
			submitTime: data.submitTime || null,
		};
	}

	// ---- Scrape: problem submissions (for a specific problem) ----
	async function scrapeProblemSubmissions(oj, probNum, username) {
		const items = [];
		for (let page = 0; page < 200; page++) {
			const start = page * PAGE_SIZE;
			progress(`Fetching submissions for ${oj}-${probNum} page ${page + 1}...`);
			const params = new URLSearchParams({
				draw: String(page + 1),
				start: String(start),
				length: String(PAGE_SIZE),
				un: username || "",
				OJId: oj,
				probNum: probNum,
				res: "all",
				language: "",
				onlyFollowee: "false",
				orderBy: "run_id",
			});

			const url = `${VJUDGE_ORIGIN}/status/data?${params.toString()}`;
			const resp = await fetchJSON(url);

			if (!resp.data || !Array.isArray(resp.data) || resp.data.length === 0) {
				break;
			}

			const parsed = resp.data
				.map(parseSubmissionItem)
				.filter((item) => item.oj === oj && item.probNum === probNum);
			items.push(...parsed);

			if (resp.data.length < PAGE_SIZE) break;
			await sleep(400);
		}
		return items;
	}

	// ---- Scrape: source code for a submission ----
	async function scrapeSourceCode(runId) {
		const url = `${VJUDGE_ORIGIN}/solution/data/${runId}?inPage=true`;
		const resp = await fetch(url, {
			method: "POST",
			credentials: "include",
			headers: {
				"Content-Type": "application/x-www-form-urlencoded; charset=UTF-8",
				"X-Requested-With": "XMLHttpRequest",
				Accept: "application/json, text/javascript, */*; q=0.01",
				Origin: VJUDGE_ORIGIN,
				Referer: `${VJUDGE_ORIGIN}/solution/${runId}`,
			},
			body: "shareCode=",
		});

		if (!resp.ok) return null;
		const data = await resp.json();
		return data.code || null;
	}

	// ---- Parsers ----
	function parseSubmissionItem(item) {
		return {
			runId: item.runId || item.run_id,
			oj: item.oj || item.OJ || "",
			probNum: item.probNum || item.prob_num || "",
			status: item.status || "",
			language: item.language || "",
			runtime: item.runtime || null,
			memory: item.memory || null,
			time: item.time || 0,
			// Include sourceProblemId for easier lookup
			sourceProblemId: `${item.oj || item.OJ || ""}-${item.probNum || item.prob_num || ""}`,
		};
	}

	// ---- Page info extractors ----
	function extractUsernameFromPage() {
		// Try multiple ways to extract username
		// 1. From the status page URL query param
		const urlParams = new URLSearchParams(window.location.search);
		if (urlParams.get("un")) return urlParams.get("un");

		// 2. From the page content (user dropdown / navbar)
		const userEl = document.querySelector(
			'a[href*="/user/"], .username, #user-menu, [data-username]',
		);
		if (userEl) {
			const href = userEl.getAttribute("href") || "";
			const match = href.match(/\/user\/([^/?]+)/);
			if (match) return match[1];
			const text = userEl.textContent?.trim();
			if (text && !text.includes("Login")) return text;
		}

		// 3. From the DataTable's ajax config (embedded in page JS)
		const scripts = document.querySelectorAll("script");
		for (const s of scripts) {
			const text = s.textContent || "";
			const match = text.match(/"un"\s*:\s*"([^"]+)"/);
			if (match) return match[1];
		}

		// 4. From the page title or header
		const header = document.querySelector(
			"h1, h2, #header-username, .profile-username",
		);
		if (header) {
			const text = header.textContent?.trim();
			if (text && text.length < 50) return text;
		}

		return null;
	}

	function getCurrentStatusPage() {
		// vjudge DataTable stores page info
		const pageInfo = document.querySelector(".dataTables_info, #status_info");
		if (pageInfo) {
			const text = pageInfo.textContent || "";
			const match = text.match(/page\s+(\d+)/i);
			if (match) return parseInt(match[1]) - 1;
		}
		return 0;
	}

	function extractSourceProblemId() {
		const match = window.location.pathname.match(/\/problem\/([^/?]+)/);
		if (match) return decodeURIComponent(match[1]);
		return null;
	}

	function extractRunId() {
		const match = window.location.pathname.match(/\/solution\/(\d+)/);
		if (match) return parseInt(match[1]);
		return null;
	}

	// ---- HTML utilities ----
	function stripHtml(html) {
		if (!html) return "";
		return html
			.replace(/<pre[^>]*>/g, "\n```text\n")
			.replace(/<\/pre>/g, "\n```\n")
			.replace(/<br\s*\/?>/g, "\n")
			.replace(/<[^>]+>/g, "")
			.replace(/&lt;/g, "<")
			.replace(/&gt;/g, ">")
			.replace(/&amp;/g, "&")
			.replace(/&quot;/g, '"')
			.replace(/&#39;/g, "'")
			.replace(/&nbsp;/g, " ")
			.replace(/\n{3,}/g, "\n\n")
			.trim();
	}

	function sleep(ms) {
		return new Promise((resolve) => setTimeout(resolve, ms));
	}

	// ---- Message handler ----
	window.addEventListener("message", async (event) => {
		if (event.source !== window) return;
		const msg = event.data;
		if (!msg || msg.source !== "acmind-content-script") return;
		if (msg.type !== "scrape-request") return;

		const mode = msg.mode;

		try {
			let payload;
			switch (mode) {
				case "problem-full":
					payload = await scrapeProblemFull();
					break;
				case "problem-page":
					payload = await scrapeProblemPage();
					break;
				case "solution-page":
					payload = await scrapeSolutionPage();
					break;
				default:
					throw new Error(`Unknown scrape mode: ${mode}`);
			}

			sendData(payload);
		} catch (err) {
			postToContent({
				type: "scraper-error",
				message: err.message || String(err),
				mode,
			});
		}
	});

	// ---- Scrape: problem page + all user submissions for this problem ----
	async function scrapeProblemFull() {
		// 1. Scrape problem info (title, statement, tags)
		progress("Fetching problem info...");
		const problem = await scrapeProblemPage();

		// 2. Scrape all user submissions for this problem
		const username = extractUsernameFromPage();
		progress(`Fetching submissions for ${problem.oj}-${problem.probNum}...`);
		const submissions = await scrapeProblemSubmissions(
			problem.oj,
			problem.probNum,
			username,
		);

		// 3. For each submission, try to fetch source code
		progress(`Fetching source code for ${submissions.length} submissions...`);
		for (let i = 0; i < submissions.length; i++) {
			progress(`Fetching code ${i + 1}/${submissions.length}...`);
			try {
				const code = await scrapeSourceCode(submissions[i].runId);
				if (code) submissions[i].code = code;
			} catch {
				// Skip if code not accessible
			}
		}

		return {
			type: "problem-full",
			problem,
			submissions,
		};
	}

	// ---- Signal ready ----
	postToContent({ type: "scraper-ready" });
})();
