# ACMind VJudge Browser Extension (Firefox)

One-click import of VJudge problems, submissions, and source code into ACMind.

## How it works

1. The extension injects "Import" buttons onto vjudge.net pages
2. When clicked, it scrapes data from VJudge's internal APIs (same-origin)
3. Scraped data is POSTed to the ACMind Web API with JWT authentication
4. ACMind stores the data in PostgreSQL and can trigger AI analysis

## Installation (Firefox)

1. Open `about:debugging#/runtime/this-firefox`
2. Click **Load Temporary Add-on**
3. Select `browser-extension-firefox/manifest.json`
4. The extension is loaded until Firefox restarts

## Configuration

1. Click the extension icon in the toolbar
2. Enter your **ACMind API URL** (e.g. `https://acmind.example.com` or `http://localhost:8080`)
3. Enter your **JWT Token** (get it from ACMind's login response)
4. Click **Save Settings**

## Usage

1. Open [vjudge.net](https://vjudge.net) and log in
2. Navigate to any of these pages:
   - **Status page** (`/status`) — click "Import this page" or "Import all submissions"
   - **Problem page** (`/problem/OJ-ID`) — click "Import to ACMind"
   - **Solution page** (`/solution/12345`) — click "Import this submission" (includes source code)
3. Data is sent to ACMind via the API

## Architecture

```
┌──────────────────────┐     ┌─────────────────────┐     ┌─────────────────┐
│  vjudge.net page     │     │  Browser Extension  │     │  ACMind API     │
│                      │     │                     │     │                 │
│  ┌────────────────┐  │     │  content.js         │     │  /api/v1/import │
│  │ vjudge-scraper │◄─┼─────┤  (injects scraper,  │     │  /vjudge/*      │
│  │ (page-context) │──┼─────┤   relays messages)   │────►│                 │
│  │                │  │     │                     │ JWT │  PostgreSQL     │
│  │ fetch() to     │  │     │  background.js      │     │  + AI analysis  │
│  │ vjudge APIs    │  │     │  (POSTs to API)     │     │                 │
│  └────────────────┘  │     │                     │     │                 │
└──────────────────────┘     └─────────────────────┘     └─────────────────┘
```

## Troubleshooting

- **"Cannot reach ACMind API"**: Check API URL in extension settings, ensure the server is running
- **"Authentication failed"**: Re-login to ACMind and update the JWT token in settings
- **Import button not appearing**: Refresh the VJudge page; check extension is enabled
- **Source code not imported**: Source code requires viewing a solution page directly
