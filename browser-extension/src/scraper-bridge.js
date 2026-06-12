// ACMind Importer — Scraper bridge.
// Injected into the page main world by content.js. Runs the matching parser
// when asked. All parsers auto-register via build-time index.

import {
  MSG_SOURCE_BRIDGE,
  MSG_SOURCE_CONTENT,
  BRIDGE_READY,
  BRIDGE_SCRAPE_REQUEST,
  BRIDGE_SCRAPE_PROGRESS,
  BRIDGE_SCRAPE_DATA,
  BRIDGE_SCRAPE_ERROR,
  BRIDGE_DESCRIBE_PAGE_REQUEST,
  BRIDGE_DESCRIBE_PAGE_RESPONSE,
} from "./protocol.js";
import { findParser, describePage } from "./parsers/registry.js";

const post = (payload) =>
  window.postMessage({ source: MSG_SOURCE_BRIDGE, ...payload }, "*");

function normalizeProgress(input) {
  if (typeof input === "string") return { message: input };
  if (typeof input === "object" && input !== null) {
    const out = {};
    if (typeof input.message === "string") out.message = input.message;
    if (typeof input.pct === "number") out.pct = Math.max(0, Math.min(100, input.pct));
    return out;
  }
  return {};
}

async function handleScrape({ requestId, mode }) {
  const url = window.location.href;
  const parser = findParser(url);
  if (!parser) {
    post({ type: BRIDGE_SCRAPE_ERROR, requestId, message: "当前页面没有可用的解析器" });
    return;
  }
  const modeSpec = parser.modes[mode];
  if (!modeSpec) {
    post({ type: BRIDGE_SCRAPE_ERROR, requestId, message: `未知的抓取模式: ${mode}` });
    return;
  }

  const progress = (input) =>
    post({ type: BRIDGE_SCRAPE_PROGRESS, requestId, ...normalizeProgress(input) });

  try {
    const payload = await modeSpec.scrape({ progress, url, document });
    post({ type: BRIDGE_SCRAPE_DATA, requestId, payload });
  } catch (err) {
    post({
      type: BRIDGE_SCRAPE_ERROR,
      requestId,
      message: err?.message ?? String(err),
    });
  }
}

window.addEventListener("message", (event) => {
  if (event.source !== window) return;
  const msg = event.data;
  if (!msg || msg.source !== MSG_SOURCE_CONTENT) return;

  if (msg.type === BRIDGE_SCRAPE_REQUEST) handleScrape(msg);
  else if (msg.type === BRIDGE_DESCRIBE_PAGE_REQUEST) {
    post({
      type: BRIDGE_DESCRIBE_PAGE_RESPONSE,
      requestId: msg.requestId,
      description: describePage(window.location.href),
    });
  }
});

post({ type: BRIDGE_READY });
