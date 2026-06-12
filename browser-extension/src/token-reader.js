// ACMind Importer — Token reader.
// Runs on ACMind localhost pages, mirrors the JWT into extension storage.

const TOKEN_KEY = "acmind_token";
const DEFAULT_API_URL = "http://localhost:8080";

async function syncToken() {
  const token = localStorage.getItem(TOKEN_KEY);
  if (!token) return;
  const { token: stored, apiUrl } = await chrome.storage.local.get(["token", "apiUrl"]);
  const updates = {};
  if (stored !== token) updates.token = token;
  if (!apiUrl) updates.apiUrl = DEFAULT_API_URL;
  if (Object.keys(updates).length > 0) await chrome.storage.local.set(updates);
}

syncToken().catch((e) => console.warn("[ACMind] token sync failed:", e));
