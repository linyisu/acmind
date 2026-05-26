(() => {
	function text(selector, root = document) {
		const element = root.querySelector(selector);
		return element ? normalizeText(element.textContent) : "";
	}

	function normalizeText(value) {
		return (value || "")
			.replace(/\u00a0/g, " ")
			.replace(/[ \t]+\n/g, "\n")
			.replace(/\n{3,}/g, "\n\n")
			.trim();
	}

	function extractLastText(selector, root = document) {
		const element = root.querySelector(selector);
		if (!element) return "";

		const styled = element.querySelector(
			".tex-font-style-sl, .tex-font-style-bf",
		);
		return normalizeText(
			styled
				? styled.textContent
				: element.lastChild?.textContent || element.textContent,
		);
	}

	function parseTimeLimitMs(value) {
		const match = value.match(/([0-9.]+)/);
		return match ? Math.floor(Number.parseFloat(match[1]) * 1000) : null;
	}

	function parseMemoryLimitMb(value) {
		const match = value.match(/([0-9]+)/);
		return match ? Number.parseInt(match[1], 10) : null;
	}

	function parseProblemId(url) {
		const parsed = new URL(url);
		const parts = parsed.pathname.split("/").filter(Boolean);
		const problemsetIndex = parts.indexOf("problemset");

		if (
			parts[problemsetIndex + 1] === "problem" &&
			parts[problemsetIndex + 2] &&
			parts[problemsetIndex + 3]
		) {
			return `${parts[problemsetIndex + 2]}${parts[problemsetIndex + 3]}`;
		}

		const problemIndex = parts.indexOf("problem");
		if (problemIndex > 0 && parts[problemIndex + 1]) {
			return `${parts[problemIndex - 1]}${parts[problemIndex + 1]}`;
		}

		return "";
	}

	function extractSampleBlock(pre) {
		const lines = Array.from(pre.querySelectorAll(".test-example-line"));
		if (lines.length === 0) {
			return normalizeText(pre.textContent);
		}

		return lines
			.map((line) => normalizeText(line.textContent))
			.join("\n")
			.trim();
	}

	function extractSamples(statement) {
		const inputs = Array.from(statement.querySelectorAll(".input pre"));
		const outputs = Array.from(statement.querySelectorAll(".output pre"));
		const count = Math.min(inputs.length, outputs.length);
		const samples = [];

		for (let index = 0; index < count; index += 1) {
			samples.push({
				input: extractSampleBlock(inputs[index]),
				output: extractSampleBlock(outputs[index]),
			});
		}

		return samples;
	}

	function cloneStatementForMarkdown(statement) {
		const clone = statement.cloneNode(true);

		clone.querySelector(".header")?.remove();
		clone
			.querySelectorAll(".sample-tests, .sample-test")
			.forEach((element) => element.remove());
		clone
			.querySelectorAll(
				".MathJax, .MathJax_Preview, .MJX_Assistive_MathML, style",
			)
			.forEach((element) => element.remove());

		clone.querySelectorAll('script[type="math/tex"]').forEach((script) => {
			const tex = normalizeText(script.textContent);
			script.replaceWith(document.createTextNode(tex ? ` $${tex}$ ` : ""));
		});

		clone.querySelectorAll(".section-title").forEach((element) => {
			element.replaceWith(
				document.createTextNode(
					`\n\n## ${normalizeText(element.textContent)}\n\n`,
				),
			);
		});

		clone.querySelectorAll("p, center, li").forEach((element) => {
			element.appendChild(document.createTextNode("\n\n"));
		});

		clone.querySelectorAll("br").forEach((element) => {
			element.replaceWith(document.createTextNode("\n"));
		});

		clone.querySelectorAll("img").forEach((image) => {
			const src = image.src || image.getAttribute("src");
			image.replaceWith(
				document.createTextNode(src ? `\n\n![题面图片](${src})\n\n` : ""),
			);
		});

		return clone;
	}

	function extractStatementMarkdown(statement) {
		return normalizeText(cloneStatementForMarkdown(statement).textContent)
			.replace(/\n[ \t]+/g, "\n")
			.replace(/[ \t]+\n/g, "\n")
			.replace(/\n{3,}/g, "\n\n");
	}

	function markdownEscapeFence(value) {
		return value.replace(/```/g, "`\u200b``");
	}

	function buildMarkdown(problem) {
		const lines = [
			`# ${problem.title}`,
			"",
			`- 来源：${problem.source}`,
			`- 题号：${problem.sourceProblemId || "未知"}`,
			`- URL：${problem.url}`,
		];

		if (problem.timeLimitMs)
			lines.push(`- 时间限制：${problem.timeLimitMs} ms`);
		if (problem.memoryLimitMb)
			lines.push(`- 内存限制：${problem.memoryLimitMb} MB`);

		lines.push(
			"",
			"## 题面",
			"",
			problem.statementText || "（未能提取题面正文）",
		);

		if (problem.samples.length > 0) {
			lines.push("", "## 样例");
			problem.samples.forEach((sample, index) => {
				lines.push(
					"",
					`### 样例 ${index + 1}`,
					"",
					"输入：",
					"```text",
					markdownEscapeFence(sample.input),
					"```",
					"",
					"输出：",
					"```text",
					markdownEscapeFence(sample.output),
					"```",
				);
			});
		}

		return `${lines.join("\n")}\n`;
	}

	function extractCodeforcesProblem() {
		const statement = document.querySelector(".problem-statement");
		if (!statement) {
			throw new Error("当前页面没有找到 Codeforces 题面区域。");
		}

		const title = text(".problem-statement > .header > .title");
		if (!title) {
			throw new Error("当前页面没有找到题目标题。");
		}

		const statementMarkdown = extractStatementMarkdown(statement);
		const problem = {
			source: "Codeforces",
			sourceProblemId: parseProblemId(window.location.href),
			title,
			url: window.location.href,
			timeLimitMs: parseTimeLimitMs(
				extractLastText(".problem-statement > .header > .time-limit"),
			),
			memoryLimitMb: parseMemoryLimitMb(
				extractLastText(".problem-statement > .header > .memory-limit"),
			),
			statementHtml: statement.innerHTML,
			statementText: statementMarkdown,
			statementMarkdown,
			samples: extractSamples(statement),
		};

		return {
			problem,
			markdown: buildMarkdown(problem),
		};
	}

	browser.runtime.onMessage.addListener((message) => {
		if (message?.type !== "ACMIND_EXTRACT_CODEFORCES") return undefined;

		try {
			return Promise.resolve({ ok: true, data: extractCodeforcesProblem() });
		} catch (error) {
			return Promise.resolve({
				ok: false,
				error: error.message || String(error),
			});
		}
	});
})();
