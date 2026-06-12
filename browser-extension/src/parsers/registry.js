// ACMind Importer — Parser registry.

import { parsers as registered } from "./_generated.js";

export const PARSERS = registered;

export function findParser(url) {
  return PARSERS.find((p) => p.matches(url)) ?? null;
}

// Describe what the current tab can do. Returns null when no parser matches.
export function describePage(url) {
  const parser = findParser(url);
  if (!parser) return null;
  const pageType = parser.detectPageType(url);
  const mode = parser.pickMode(pageType);
  const modeSpec = mode ? parser.modes[mode] : null;
  return {
    parser: parser.name,
    displayName: parser.displayName,
    pageType,
    mode,
    modeLabel: modeSpec?.label ?? null,
  };
}
