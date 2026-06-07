// ACMind VJudge Importer — Firefox Background Script (Event Page).
// Receives scraped data from the content script and POSTs it to the
// ACMind Web API with JWT authentication.
//
// Firefox uses background.scripts (event page) instead of service_worker.
// Both chrome.* and browser.* APIs are available.

const MAX_RETRIES = 3;
const RETRY_DELAY_MS = 1000;

let acmindAvailable = false;
let lastCheck = 0;

// ---- Settings ----
async function getSettings() {
  const result = await chrome.storage.local.get(["apiUrl", "token"]);
  return {
    apiUrl: (result.apiUrl || "").replace(/\/+$/, ""),
    token: result.token || "",
  };
}

// ---- Connection check ----
async function checkAcmindAvailable() {
  const now = Date.now();
  if (now - lastCheck < 5000) return acmindAvailable;

  const { apiUrl } = await getSettings();
  if (!apiUrl) {
    acmindAvailable = false;
    lastCheck = now;
    return false;
  }

  try {
    const resp = await fetch(`${apiUrl}/health`, {
      method: "GET",
      signal: AbortSignal.timeout(5000),
    });
    acmindAvailable = resp.ok;
  } catch {
    acmindAvailable = false;
  }
  lastCheck = now;
  return acmindAvailable;
}

// ---- API request helper ----
async function postToAcmind(endpoint, data, retries = MAX_RETRIES) {
  const { apiUrl, token } = await getSettings();

  if (!apiUrl) {
    throw new Error("ACMind API URL not configured. Open the extension popup and set it.");
  }
  if (!token) {
    throw new Error("JWT token not configured. Open the extension popup and set it.");
  }

  for (let attempt = 0; attempt < retries; attempt++) {
    try {
      const resp = await fetch(`${apiUrl}${endpoint}`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Authorization": `Bearer ${token}`,
        },
        body: JSON.stringify(data),
        signal: AbortSignal.timeout(30000),
      });

      if (resp.ok) {
        const result = await resp.json().catch(() => ({}));
        return { success: true, ...result };
      }

      if (resp.status === 401) {
        throw new Error("Authentication failed. Please check your JWT token in the extension settings.");
      }

      const errorText = await resp.text().catch(() => "Unknown error");
      console.error(`[ACMind] Server error ${resp.status}: ${errorText}`);
      throw new Error(`Server error: ${resp.status}`);
    } catch (err) {
      console.error(`[ACMind] POST attempt ${attempt + 1} failed:`, err.message);
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
    handleImport(msg.payload).then(sendResponse).catch((err) => {
      sendResponse({ success: false, error: err.message });
    });
    return true;
  }

  if (msg.type === "check-connection") {
    checkAcmindAvailable()
      .then((available) => sendResponse({ available }))
      .catch(() => sendResponse({ available: false }));
    return true;
  }

  return false;
});

async function handleImport(payload) {
  let endpoint, data;

  switch (payload.type) {
    case "status-page":
    case "status-all":
      endpoint = "/api/v1/import/vjudge/submissions";
      data = {
        username: payload.username,
        items: payload.items,
      };
      break;

    case "problem-page":
      endpoint = "/api/v1/import/vjudge/problem";
      data = {
        source_problem_id: payload.sourceProblemId,
        oj: payload.oj,
        prob_num: payload.probNum,
        title: payload.title,
        url: payload.url,
        statement: payload.statement,
        tags: payload.tags,
      };
      break;

    case "solution-page":
      endpoint = "/api/v1/import/vjudge/submission";
      data = {
        run_id: payload.runId,
        oj: payload.oj,
        prob_num: payload.probNum,
        status: payload.status,
        language: payload.language,
        code: payload.code,
        runtime: payload.runtime,
        memory: payload.memory,
        submit_time: payload.submitTime,
      };
      break;

    default:
      throw new Error(`Unknown payload type: ${payload.type}`);
  }

  return postToAcmind(endpoint, data);
}

// ---- Notify content scripts about connection changes ----
setInterval(async () => {
  const wasAvailable = acmindAvailable;
  await checkAcmindAvailable();
  if (wasAvailable !== acmindAvailable) {
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

console.log("[ACMind] Firefox background event page started.");
