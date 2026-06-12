#!/usr/bin/env node
// ACMind Importer — Build script.
// Bundles src/ with esbuild into dist/chrome/ and dist/firefox/.

import { build, context } from "esbuild";
import { readFile, writeFile, mkdir, rm, cp, readdir } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const ROOT = dirname(fileURLToPath(import.meta.url));
const SRC = join(ROOT, "src");
const watch = process.argv.includes("--watch");

const ENTRIES = [
  { in: "background.js", out: "background" },
  { in: "content.js", out: "content" },
  { in: "scraper-bridge.js", out: "scraper-bridge" },
  { in: "token-reader.js", out: "token-reader" },
  { in: "options/options.js", out: "options/options" },
];

const STATIC_COPIES = [
  { from: "options/options.html", to: "options/options.html" },
  { from: "options/options.css", to: "options/options.css" },
];

const BROWSERS = ["chrome", "firefox"];

const baseManifest = JSON.parse(await readFile(join(ROOT, "manifest.base.json"), "utf-8"));

function manifestFor(browser) {
  const m = structuredClone(baseManifest);
  if (browser === "chrome") {
    m.background = { service_worker: "background.js" };
  } else {
    m.background = { scripts: ["background.js"] };
    m.browser_specific_settings = {
      gecko: { id: "acmind-importer@mengh04.dev", strict_min_version: "115.0" },
    };
  }
  return m;
}

async function listParsers() {
  const dir = join(SRC, "parsers");
  const entries = await readdir(dir, { withFileTypes: true });
  return entries
    .filter((e) =>
      e.isFile() &&
      e.name.endsWith(".js") &&
      !e.name.startsWith("_") &&
      e.name !== "registry.js" &&
      e.name !== "base.js")
    .map((e) => e.name);
}

async function generateParserIndex() {
  const parsers = await listParsers();
  const imports = parsers.map((p, i) => `import p${i} from "./${p}";`).join("\n");
  const list = parsers.map((_, i) => `p${i}`).join(", ");
  const body = `// AUTO-GENERATED at build time.\n${imports}\nexport const parsers = [${list}];\n`;
  await writeFile(join(SRC, "parsers", "_generated.js"), body);
  return parsers.length;
}

async function buildBrowser(browser) {
  const dist = join(ROOT, "dist", browser);
  await rm(dist, { recursive: true, force: true });
  await mkdir(dist, { recursive: true });

  await writeFile(
    join(dist, "manifest.json"),
    JSON.stringify(manifestFor(browser), null, 2) + "\n",
  );

  const options = {
    entryPoints: ENTRIES.map((e) => ({ in: join(SRC, e.in), out: e.out })),
    bundle: true,
    format: "iife",
    target: ["chrome88", "firefox115"],
    outdir: dist,
    platform: "browser",
    legalComments: "none",
    logLevel: "warning",
  };

  if (watch) {
    const ctx = await context(options);
    await ctx.watch();
    console.log(`  ✓ ${browser}: watching`);
  } else {
    await build(options);
  }

  for (const { from, to } of STATIC_COPIES) {
    await mkdir(dirname(join(dist, to)), { recursive: true });
    await cp(join(SRC, from), join(dist, to));
  }
  await cp(join(ROOT, "icons"), join(dist, "icons"), { recursive: true });

  console.log(`  ✓ ${browser}`);
}

console.log("ACMind Importer — Build\n");
const count = await generateParserIndex();
console.log(`  ✓ ${count} parser(s) registered`);
for (const browser of BROWSERS) {
  await buildBrowser(browser);
}
console.log(watch ? "\nWatching for changes..." : "\nDone! dist/chrome/ and dist/firefox/ are ready.");
