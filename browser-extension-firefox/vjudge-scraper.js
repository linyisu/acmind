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
		const username = requireUsername();
		const pageIdx = getCurrentStatusPage();

		progress(`Fetching submissions page ${pageIdx + 1}...`);

		const resp = await fetchStatusPage({
			page: pageIdx,
			username,
			oj: "All",
			probNum: "",
			draw: 1,
		});

		if (!Array.isArray(resp.data)) {
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
		const username = requireUsername();
		const allItems = [];

		for (let page = 0; page < 200; page++) {
			progress(`Fetching submissions page ${page + 1}...`);
			const resp = await fetchStatusPage({
				page,
				username,
				oj: "All",
				probNum: "",
				draw: page + 1,
			});

			if (!Array.isArray(resp.data) || resp.data.length === 0) {
				break;
			}

			allItems.push(...resp.data.map(parseSubmissionItem));

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

	async function fetchStatusPage({ page, username, oj, probNum, draw }) {
		const params = new URLSearchParams({
			draw: String(draw),
			start: String(page * PAGE_SIZE),
			length: String(PAGE_SIZE),
			un: username || "",
			OJId: oj,
			probNum,
			res: "all",
			language: "",
			onlyFollowee: "false",
			orderBy: "run_id",
		});

		return fetchJSON(`${VJUDGE_ORIGIN}/status/data?${params.toString()}`);
	}

	// ---- Scrape: problem page (description + metadata + submissions + source code) ----
	async function scrapeProblemPage() {
		const sourceProblemId = extractSourceProblemId();
		if (!sourceProblemId) {
			throw new Error("Could not determine problem ID from URL");
		}

		const problem = await scrapeProblemMetadata(sourceProblemId);
		const username = extractUsernameFromPage();
		const submissions = await scrapeProblemSubmissionsWithSource(
			problem,
			username,
		);

		return {
			type: "problem-page",
			sourceProblemId,
			oj: problem.oj,
			probNum: problem.probNum,
			title: problem.title,
			url: `${VJUDGE_ORIGIN}/problem/${sourceProblemId}`,
			statement: problem.statement,
			tags: problem.tags,
			rawData: problem.rawData,
			username,
			submissions,
		};
	}

	async function scrapeProblemMetadata(sourceProblemId) {
		progress(`Fetching problem ${sourceProblemId}...`);

		const html = await fetchText(`${VJUDGE_ORIGIN}/problem/${sourceProblemId}`);
		const rawData = extractPageJson(html, "dataJson");
		const title = extractTitle(html, sourceProblemId);
		const [fallbackOj, fallbackProbNum] = sourceProblemId.split("-");
		const oj = rawData?.oj || fallbackOj;
		const probNum = rawData?.prob || fallbackProbNum;

		return {
			sourceProblemId,
			oj,
			probNum,
			title,
			statement: await scrapeProblemStatement(html, rawData),
			tags: rawData?.oj ? [rawData.oj] : [fallbackOj],
			rawData,
		};
	}

	async function scrapeProblemStatement(problemHtml, rawData) {
		const desc = rawData?.descBriefs?.[0];
		if (desc) {
			try {
				const descHtml = await fetchText(
					`${VJUDGE_ORIGIN}/problem/description/${desc.key}?${desc.version}`,
				);
				const statement = parseStatementFromDescription(descHtml);
				if (statement) return statement;
			} catch (e) {
				progress(`Description fetch failed: ${e.message}, trying fallback...`);
			}
		}

		const descPathMatch = problemHtml.match(/\/problem\/description\/\d+/);
		if (!descPathMatch) return null;

		try {
			const descHtml = await fetchText(`${VJUDGE_ORIGIN}${descPathMatch[0]}`);
			return parseStatementFromDescription(descHtml);
		} catch {
			return null;
		}
	}

	function parseStatementFromDescription(descHtml) {
		const descData = extractPageJson(descHtml, "data-json-container");
		if (!descData?.sections) return null;

		return descData.sections
			.map((sec) => {
				const title = sec.title ? `## ${sec.title}\n\n` : "";
				const content = stripHtml(sec.value?.content || "");
				return title + content;
			})
			.join("\n\n");
	}

	async function scrapeProblemSubmissionsWithSource(problem, username) {
		if (!username) return [];

		progress(
			`Fetching your submissions for ${problem.oj}-${problem.probNum}...`,
		);

		let submissions = [];
		try {
			submissions = await scrapeProblemSubmissions(
				problem.oj,
				problem.probNum,
				username,
			);
		} catch (e) {
			console.warn("[ACMind] Failed to scrape submissions:", e.message);
			return [];
		}

		return attachSourceCodeToSubmissions(submissions);
	}

	async function attachSourceCodeToSubmissions(submissions) {
		for (let i = 0; i < submissions.length; i++) {
			const sub = submissions[i];
			progress(
				`Fetching source code ${i + 1}/${submissions.length} (run #${sub.runId})...`,
			);
			try {
				const code = await scrapeSourceCode(sub.runId);
				if (code) sub.code = code;
			} catch (e) {
				console.warn(
					`[ACMind] Failed to fetch source for run #${sub.runId}:`,
					e.message,
				);
			}
			if (i < submissions.length - 1) {
				await sleep(400);
			}
		}

		return submissions;
	}

	// ---- Scrape: solution page (source code + submission metadata) ----
	async function scrapeSolutionPage() {
		const runId = extractRunId();
		if (!runId) {
			throw new Error("Could not determine run ID from URL");
		}

		progress(`Fetching solution #${runId}...`);

		const data = await fetchSolutionData(runId);
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
			progress(`Fetching submissions for ${oj}-${probNum} page ${page + 1}...`);
			const resp = await fetchStatusPage({
				page,
				username,
				oj,
				probNum,
				draw: page + 1,
			});

			if (!Array.isArray(resp.data) || resp.data.length === 0) {
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
		const data = await fetchSolutionData(runId);
		return data.code || null;
	}

	async function fetchSolutionData(runId) {
		const resp = await fetch(
			`${VJUDGE_ORIGIN}/solution/data/${runId}?inPage=true`,
			{
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
			},
		);

		if (!resp.ok) {
			throw new Error(`HTTP ${resp.status} for solution/${runId}`);
		}

		return resp.json();
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
		const urlParams = new URLSearchParams(window.location.search);
		const usernameFromQuery = cleanUsername(urlParams.get("un"));
		if (usernameFromQuery) return usernameFromQuery;

		const usernameFromUserLink = extractUsernameFromUserLink();
		if (usernameFromUserLink) return usernameFromUserLink;

		const usernameFromVisitorInput = cleanUsername(
			document
				.querySelector('input[name="sup"], input[id="visitor_sup"]')
				?.getAttribute("value"),
		);
		if (usernameFromVisitorInput) return usernameFromVisitorInput;

		const usernameFromScripts = extractUsernameFromScripts();
		if (usernameFromScripts) return usernameFromScripts;

		const usernameFromProfileHeader = cleanUsername(
			document.querySelector("#header-username, .profile-username")
				?.textContent,
		);
		return usernameFromProfileHeader;
	}

	function extractUsernameFromUserLink() {
		const userEl = document.querySelector(
			'a[href*="/user/"], .username, #user-menu, [data-username]',
		);
		if (!userEl) return null;

		const dataUsername = cleanUsername(userEl.getAttribute("data-username"));
		if (dataUsername) return dataUsername;

		const href = userEl.getAttribute("href") || "";
		const match = href.match(/\/user\/([^/?]+)/);
		if (match) return cleanUsername(decodeURIComponent(match[1]));

		const text = cleanUsername(userEl.textContent);
		return text && !text.includes("Login") ? text : null;
	}

	function extractUsernameFromScripts() {
		const scripts = document.querySelectorAll("script");
		for (const s of scripts) {
			const text = s.textContent || "";
			const match = text.match(/"un"\s*:\s*"([^"]+)"/);
			if (match) return cleanUsername(match[1]);

			const gaMatch = text.match(/'user_id'\s*:\s*'([^']+)'/);
			if (gaMatch) return cleanUsername(gaMatch[1]);
		}
		return null;
	}

	function cleanUsername(value) {
		const username = value?.trim();
		if (!username || username.length >= 50) return null;
		return username;
	}

	function requireUsername() {
		const username = extractUsernameFromPage();
		if (!username) {
			throw new Error("Could not determine VJudge username from page");
		}
		return username;
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
				case "status-page":
					payload = await scrapeStatusPage();
					break;
				case "status-all":
					payload = await scrapeStatusAll();
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

	// ---- Signal ready ----
	postToContent({ type: "scraper-ready" });
})();
