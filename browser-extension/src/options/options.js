// ACMind Importer — Options page.

import { BG_CHECK_CONNECTION, BG_GET_TOKEN_STATUS } from "../protocol.js";

const $ = (sel) => document.querySelector(sel);

const apiInput = $("#api-url");
const saveBtn = $("#save");
const statusEl = $("#status");
const connDot = $("#conn-dot");
const connText = $("#conn-text");

async function loadSettings() {
  const { apiUrl = "" } = await chrome.storage.local.get(["apiUrl"]);
  apiInput.value = apiUrl;

  const tokenStatus = await chrome.runtime.sendMessage({ type: BG_GET_TOKEN_STATUS });
  if (tokenStatus?.hasToken && !apiUrl && tokenStatus.apiUrl) {
    apiInput.value = tokenStatus.apiUrl;
  }

  refreshConnection();
}

async function refreshConnection() {
  connDot.className = "dot";
  connText.textContent = "正在检测...";
  const resp = await chrome.runtime.sendMessage({ type: BG_CHECK_CONNECTION });
  const ok = !!resp?.available;
  connDot.className = `dot ${ok ? "connected" : "disconnected"}`;
  connText.textContent = ok ? "已连接" : "未连接";
}

saveBtn.addEventListener("click", async () => {
  const url = (apiInput.value ?? "").replace(/\/+$/, "");
  await chrome.storage.local.set({ apiUrl: url });
  statusEl.textContent = "已保存 ✓";
  statusEl.classList.add("visible");
  setTimeout(() => statusEl.classList.remove("visible"), 1500);
  refreshConnection();
});

apiInput.addEventListener("keydown", (e) => {
  if (e.key === "Enter") saveBtn.click();
});

loadSettings();
