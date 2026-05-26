# ACMind VJudge Browser Extension

One-click import of VJudge problems, submissions, and source code into ACMind.

## How it works

1. The extension injects an "Import to ACMind" button bar onto vjudge.net pages
2. When clicked, it scrapes data from vjudge's internal APIs (same-origin, no CORS)
3. Scraped data is sent to the ACMind desktop app via `http://127.0.0.1:18921`
4. The ACMind app stores everything in its local SQLite database

## Installation

### Chrome / Edge / Brave

1. Open `chrome://extensions` in your browser
2. Enable **Developer mode** (toggle in top-right)
3. Click **Load unpacked** and select the `browser-extension/` directory
4. The extension icon appears in your toolbar

### Firefox

Firefox support coming soon. Chrome Manifest V3 is largely compatible.

## Usage

1. Launch the ACMind desktop app (starts the import server on port 18921)
2. Open [vjudge.net](https://vjudge.net) and log in
3. Navigate to any of these pages:
   - **Status page** (`/status`) — click "Import this page" or "Import all submissions"
   - **Problem page** (`/problem/OJ-ID`) — click "Import to ACMind"
   - **Solution page** (`/solution/12345`) — click "Import this submission" (includes source code)
4. Data appears in ACMind immediately

## Architecture

```
┌──────────────────────┐     ┌─────────────────────┐     ┌─────────────────┐
│  vjudge.net page     │     │  Browser Extension  │     │  ACMind App     │
│                      │     │                     │     │                 │
│  ┌────────────────┐  │     │  content.js         │     │  import_server  │
│  │ vjudge-scraper │◄─┼─────┤  (injects scraper,  │     │  (tiny_http on  │
│  │ (page-context) │──┼─────┤   relays messages)   │────►│   127.0.0.1:    │
│  │                │  │     │                     │     │   18921)        │
│  │ fetch() to     │  │     │  background.js      │     │                 │
│  │ vjudge APIs    │  │     │  (POSTs to ACMind)  │     │  SQLite + FS    │
│  └────────────────┘  │     │                     │     │                 │
└──────────────────────┘     └─────────────────────┘     └─────────────────┘
```

## Troubleshooting

- **"ACMind not running"**: Make sure the ACMind desktop app is launched
- **Import button not appearing**: Refresh the vjudge page; check extension is enabled
- **Source code not imported**: Source code requires viewing a solution page directly
