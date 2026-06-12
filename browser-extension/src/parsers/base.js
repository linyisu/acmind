// ACMind Importer — Parser API.
//
// New parsers only need to call defineParser({...}) once. The build script
// picks up every file in src/parsers/ and the scraper bridge auto-registers
// them.
//
// Shape:
//   defineParser({
//     name: "vjudge",                  // unique id
//     displayName: "VJudge",           // shown in toast/badge
//     matches: (url) => boolean,
//     detectPageType: (url) => "problem" | "solution" | "status" | "other",
//     // Pick the mode to run when the user clicks the toolbar icon.
//     // Receives the page type, returns a mode id (or null if nothing applies).
//     pickMode: (pageType) => "problem-full" | null,
//     modes: {
//       "problem-full": {
//         label: "题目 + 所有提交",   // short label for the progress toast
//         scrape: async ({ progress, url, document }) => payload,
//       },
//     },
//   })
//
// `progress` accepts either a string (just updates the label, no pct change)
// or { message, pct } where pct is 0-100.

export function defineParser(spec) {
  const required = ["name", "displayName", "matches", "detectPageType", "modes", "pickMode"];
  for (const k of required) {
    if (spec[k] == null) {
      throw new Error(`[ACMind] Parser missing required field: ${k}`);
    }
  }
  if (typeof spec.modes !== "object" || Object.keys(spec.modes).length === 0) {
    throw new Error(`[ACMind] Parser "${spec.name}" has no modes`);
  }
  for (const [id, mode] of Object.entries(spec.modes)) {
    if (typeof mode.scrape !== "function") {
      throw new Error(`[ACMind] Parser "${spec.name}" mode "${id}" missing scrape()`);
    }
  }
  return spec;
}
