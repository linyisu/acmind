// ACMind Importer — Shared protocol constants.

export const MSG_SOURCE_CONTENT = "acmind-content-script";
export const MSG_SOURCE_BRIDGE = "acmind-scraper-bridge";

// content <-> bridge (window.postMessage in main world)
export const BRIDGE_READY = "ready";
export const BRIDGE_SCRAPE_REQUEST = "scrape-request";
export const BRIDGE_SCRAPE_PROGRESS = "scrape-progress";
export const BRIDGE_SCRAPE_DATA = "scrape-data";
export const BRIDGE_SCRAPE_ERROR = "scrape-error";
export const BRIDGE_DESCRIBE_PAGE_REQUEST = "describe-page-request";
export const BRIDGE_DESCRIBE_PAGE_RESPONSE = "describe-page-response";

// background -> content (chrome.tabs.sendMessage)
export const CMD_START_IMPORT = "acmind:start-import";

// content -> background (chrome.runtime.sendMessage)
export const BG_IMPORT = "import-to-acmind";
export const BG_CHECK_CONNECTION = "check-connection";
export const BG_GET_TOKEN_STATUS = "get-token-status";
export const BG_DESCRIBE_PAGE = "describe-page"; // background asks content "what is this tab?"

export const SCRAPER_READY_TIMEOUT_MS = 3000;
