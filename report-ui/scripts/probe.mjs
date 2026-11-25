// Probes every story of a built Storybook at desktop (1280×800) and mobile (390×844), light and
// dark: no script errors, no serious or critical axe-core violation (WCAG 2.0–2.2 A and AA, which
// includes target-size), and no sideways scroll at 390 px. Exit 1 on any failure.
//
//   node scripts/probe.mjs storybook-static [--filter openfga] [--out probe.json]
import { createServer } from "node:http";
import { readFileSync, writeFileSync, existsSync, statSync } from "node:fs";
import { createRequire } from "node:module";
import { extname, join, resolve } from "node:path";
import { chromium } from "playwright";

const require = createRequire(import.meta.url);
const axeSource = readFileSync(require.resolve("axe-core/axe.min.js"), "utf8");
const argv = process.argv.slice(2);
const opt = (name) => {
  const i = argv.indexOf("--" + name);
  return i >= 0 ? argv[i + 1] : undefined;
};
const dir = resolve(argv.find((a, i) => !a.startsWith("--") && !(i && argv[i - 1].startsWith("--"))) || "storybook-static");
const filter = opt("filter");
const VIEWS = [
  { name: "desktop-light", viewport: { width: 1280, height: 800 }, colorScheme: "light" },
  { name: "desktop-dark", viewport: { width: 1280, height: 800 }, colorScheme: "dark" },
  { name: "mobile-light", viewport: { width: 390, height: 844 }, colorScheme: "light" },
  { name: "mobile-dark", viewport: { width: 390, height: 844 }, colorScheme: "dark" },
];
const TAGS = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"];
const TYPES = { ".html": "text/html", ".js": "text/javascript", ".json": "application/json", ".css": "text/css", ".svg": "image/svg+xml", ".woff2": "font/woff2" };

const server = createServer((req, res) => {
  const path = join(dir, decodeURIComponent(new URL(req.url, "http://x").pathname));
  if (!path.startsWith(dir) || !existsSync(path) || statSync(path).isDirectory()) {
    res.writeHead(404).end();
    return;
  }
  res.writeHead(200, { "content-type": TYPES[extname(path)] || "application/octet-stream" }).end(readFileSync(path));
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const base = "http://127.0.0.1:" + server.address().port;

const index = JSON.parse(readFileSync(join(dir, "index.json"), "utf8"));
const stories = Object.values(index.entries).filter((e) => e.type === "story" && (!filter || e.id.includes(filter)));

async function probe(browser, story, view) {
  const ctx = await browser.newContext({ viewport: view.viewport, colorScheme: view.colorScheme, reducedMotion: "reduce" });
  try {
    return await check(ctx, story, view);
  } finally {
    await ctx.close();
  }
}

async function check(ctx, story, view) {
  const page = await ctx.newPage();
  const errors = [];
  page.on("pageerror", (e) => errors.push(e.message));
  page.on("console", (m) => m.type() === "error" && errors.push(m.text()));
  await page.goto(base + "/iframe.html?viewMode=story&id=" + story.id);
  await page.waitForFunction(() => {
    const root = document.querySelector("#storybook-root");
    if (!root || !root.firstElementChild) return false;
    const report = root.querySelector("toolbox-report");
    return !report || (report.shadowRoot && report.shadowRoot.getElementById("body").childElementCount > 0);
  }, null, { timeout: 15000 });
  await page.waitForTimeout(100);
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
  // Text squeezed into a column a few letters wide ("r u n" in a flex callout) overflows nothing,
  // so look for runs of text far taller than they are wide.
  const stacked = await page.evaluate(() => {
    const out = [];
    const roots = [document];
    document.querySelectorAll("*").forEach((n) => n.shadowRoot && roots.push(n.shadowRoot));
    roots.forEach((root) => {
      const walk = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
      for (let t = walk.nextNode(); t; t = walk.nextNode()) {
        const s = t.textContent.trim();
        const p = t.parentElement;
        if (s.length < 3 || !p || p.closest("svg,style,script,select,option")) continue;
        const r = document.createRange();
        r.selectNodeContents(t);
        const b = r.getBoundingClientRect();
        const fs = parseFloat(getComputedStyle(p).fontSize) || 14;
        if (b.width > 0 && b.width < fs * 2.5 && b.height > fs * 4) out.push(s.slice(0, 30));
      }
    });
    return out;
  });
  await page.addScriptTag({ content: axeSource });
  const axe = await page.evaluate(async (tags) => {
    const r = await window.axe.run(document, { resultTypes: ["violations"], runOnly: { type: "tag", values: tags } });
    return r.violations
      .filter((v) => v.impact === "serious" || v.impact === "critical")
      .map((v) => ({ id: v.id, impact: v.impact, nodes: v.nodes.length, first: v.nodes[0] && v.nodes[0].target.join(" > ") }));
  }, TAGS);
  const failures = [
    ...errors.map((e) => "script error: " + e),
    ...axe.map((v) => `axe ${v.id} (${v.impact}) on ${v.nodes} node(s), first ${v.first}`),
    ...(view.viewport.width < 600 && overflow > 0 ? ["scrolls sideways by " + overflow + " px"] : []),
    ...(stacked.length ? ["text squeezed into a narrow column: " + stacked.slice(0, 3).map((s) => JSON.stringify(s)).join(", ")] : []),
  ];
  return { story: story.id, view: view.name, overflow, axe, errors, failures };
}

const browser = await chromium.launch();
const jobs = stories.flatMap((s) => VIEWS.map((v) => [s, v]));
const results = [];
let next = 0;
await Promise.all(
  Array.from({ length: 4 }, async () => {
    while (next < jobs.length) {
      const [s, v] = jobs[next++];
      // A render timeout on a busy machine is retried once and reported; any other failure is not.
      for (let attempt = 1; ; attempt++) {
        try {
          results.push({ ...(await probe(browser, s, v)), attempts: attempt });
          if (attempt > 1) console.log(`RETRY ${s.id} [${v.name}] rendered on attempt ${attempt}`);
          break;
        } catch (e) {
          if (attempt < 2) continue;
          results.push({ story: s.id, view: v.name, attempts: attempt, failures: ["did not render: " + e.message.split("\n")[0]] });
          break;
        }
      }
    }
  }),
);
await browser.close();
server.close();

results.sort((a, b) => a.story.localeCompare(b.story) || a.view.localeCompare(b.view));
const failed = results.filter((r) => r.failures.length);
for (const r of failed) for (const f of r.failures) console.log(`FAIL ${r.story} [${r.view}] ${f}`);
if (opt("out")) writeFileSync(opt("out"), JSON.stringify(results, null, 2) + "\n");
console.log(`${stories.length} stories × ${VIEWS.length} views: ${results.length - failed.length} passed, ${failed.length} failed`);
process.exit(failed.length ? 1 : 0);
