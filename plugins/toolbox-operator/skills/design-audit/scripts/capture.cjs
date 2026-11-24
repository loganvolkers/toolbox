#!/usr/bin/env node
// Evidence for a design audit: screenshots of each route at two viewports and two color
// schemes, plus DOM metrics an auditor can compare between runs. Open shadow roots are walked.
//
// Prerequisite: playwright (with chromium) and axe-core resolvable from NODE_PATH, e.g.
//   npm i --prefix /tmp/audit playwright axe-core && npx --prefix /tmp/audit playwright install chromium
//   NODE_PATH=/tmp/audit/node_modules node capture.cjs --url URL --routes '#a,#b' --out DIR
// or, from a served Storybook build, the stories whose ids contain any of --stories:
//   NODE_PATH=... node capture.cjs --storybook http://127.0.0.1:6006 --stories pages-openfga,states --out DIR
//
// Writes DIR/<route>-<viewport>-<scheme>.png, DIR/<route>-desktop-light-full.png and
// DIR/metrics.json. Exit 1 when the page never renders the --ready selector.
const fs = require("fs");
const path = require("path");
const { chromium } = require("playwright");
const axeSource = fs.readFileSync(require.resolve("axe-core/axe.min.js"), "utf8");

function args() {
  const a = { routes: "", ready: "main", out: "audit-evidence" };
  for (let i = 2; i < process.argv.length; i += 2) a[process.argv[i].replace(/^--/, "")] = process.argv[i + 1];
  if (!a.url && !a.storybook) throw new Error("--url or --storybook is required");
  a.routes = a.routes ? a.routes.split(",") : [""];
  return a;
}

// Each target is a page to capture: a route of --url, or a story of --storybook.
async function targets(a) {
  if (!a.storybook) {
    return a.routes.map((route) => ({ slug: route.replace(/^#/, "").replace(/[^\w-]+/g, "_") || "root", url: a.url.split("#")[0] + route }));
  }
  const base = a.storybook.replace(/\/(index\.html)?$/, "");
  const index = await (await fetch(base + "/index.json")).json();
  const want = (a.stories || "").split(",").filter(Boolean);
  return Object.values(index.entries)
    .filter((e) => e.type === "story" && (!want.length || want.some((w) => e.id.includes(w))))
    .map((e) => ({ slug: e.id, url: base + "/iframe.html?viewMode=story&id=" + e.id }));
}

const VIEWPORTS = { desktop: { width: 1280, height: 800 }, mobile: { width: 390, height: 844 } };

// Runs in the page: facts about what is rendered above the fold and in the whole route.
function measure() {
  const out = { textChars: 0, monoChars: 0, fontSizes: {}, fontWeights: {}, hues: {}, foldWords: 0,
    headings: 0, interactive: 0, svg: 0, smallTargets: 0, pageHeight: document.documentElement.scrollHeight };
  const fold = window.innerHeight;
  const hueOf = (c) => {
    const m = c.match(/rgba?\(([\d.]+),\s*([\d.]+),\s*([\d.]+)(?:,\s*([\d.]+))?/);
    if (!m || (m[4] !== undefined && +m[4] < 0.2)) return null;
    const [r, g, b] = [m[1], m[2], m[3]].map((v) => v / 255);
    const max = Math.max(r, g, b), min = Math.min(r, g, b), l = (max + min) / 2;
    if (max === min) return null;
    const s = l > 0.5 ? (max - min) / (2 - max - min) : (max - min) / (max + min);
    if (s < 0.25) return null;
    let h = max === r ? (g - b) / (max - min) : max === g ? 2 + (b - r) / (max - min) : 4 + (r - g) / (max - min);
    return Math.round(((h * 60 + 360) % 360) / 30) * 30;
  };
  const visit = (root) => {
    for (const node of root.querySelectorAll("*")) {
      if (node.shadowRoot) visit(node.shadowRoot);
      const cs = getComputedStyle(node);
      if (cs.display === "none" || cs.visibility === "hidden") continue;
      const rect = node.getBoundingClientRect();
      if (!rect.width || !rect.height) continue;
      if (/^H[1-3]$/.test(node.tagName)) out.headings++;
      if (node.tagName === "svg") out.svg++;
      if (node.matches("a,button,summary,input,[role=tab],[tabindex='0']")) {
        out.interactive++;
        if (rect.width < 24 || rect.height < 24) out.smallTargets++;
      }
      for (const c of [cs.color, cs.backgroundColor, cs.borderTopColor, cs.fill]) {
        const h = hueOf(c || "");
        if (h !== null) out.hues[h] = (out.hues[h] || 0) + 1;
      }
      for (const t of node.childNodes) {
        if (t.nodeType !== 3) continue;
        const text = t.textContent.trim();
        if (!text) continue;
        out.textChars += text.length;
        if (/mono|Menlo|Consolas/i.test(cs.fontFamily)) out.monoChars += text.length;
        out.fontSizes[cs.fontSize] = (out.fontSizes[cs.fontSize] || 0) + text.length;
        out.fontWeights[cs.fontWeight] = (out.fontWeights[cs.fontWeight] || 0) + text.length;
        if (rect.top < fold && rect.bottom > 0) out.foldWords += text.split(/\s+/).length;
      }
    }
  };
  visit(document);
  out.monoShare = out.textChars ? +(out.monoChars / out.textChars).toFixed(3) : 0;
  out.distinctFontSizes = Object.keys(out.fontSizes).length;
  out.distinctHues = Object.keys(out.hues).length;
  return out;
}

// Entry animations caught mid-way read as faded, low-contrast panels to auditors and axe.
async function settle(page) {
  await page.evaluate(() => {
    const roots = [document];
    for (let i = 0; i < roots.length; i++) for (const n of roots[i].querySelectorAll("*")) if (n.shadowRoot) roots.push(n.shadowRoot);
    const running = roots.flatMap((r) => r.getAnimations()).filter((x) => x.effect?.getComputedTiming().endTime !== Infinity);
    return Promise.race([Promise.all(running.map((x) => x.finished.catch(() => {}))), new Promise((r) => setTimeout(r, 3000))]);
  });
}

async function main() {
  const a = args();
  fs.mkdirSync(a.out, { recursive: true });
  const browser = await chromium.launch();
  const metrics = {};
  for (const { slug, url } of await targets(a)) {
    for (const [vp, size] of Object.entries(VIEWPORTS)) {
      for (const scheme of ["light", "dark"]) {
        const ctx = await browser.newContext({ viewport: size, colorScheme: scheme, deviceScaleFactor: 1 });
        const page = await ctx.newPage();
        await page.goto(url);
        try {
          await page.waitForFunction((sel) => {
            const find = (r) => r.querySelector(sel) || [...r.querySelectorAll("*")].map((n) => n.shadowRoot && find(n.shadowRoot)).find(Boolean);
            return find(document);
          }, a.ready, { timeout: 10000 });
        } catch (e) {
          console.error(`not ready: ${slug} (${a.ready})`);
          process.exitCode = 1;
        }
        await page.waitForTimeout(400);
        await settle(page);
        await page.screenshot({ path: path.join(a.out, `${slug}-${vp}-${scheme}.png`) });
        if (vp === "mobile" && scheme === "light") {
          (metrics[slug] = metrics[slug] || {}).mobileOverflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
        }
        if (vp === "desktop" && scheme === "light") {
          // A full-page shot resizes the viewport, which can replay entry animations mid-capture.
          await page.emulateMedia({ reducedMotion: "reduce" });
          await settle(page);
          await page.screenshot({ path: path.join(a.out, `${slug}-desktop-light-full.png`), fullPage: true });
          await settle(page);
          const m = await page.evaluate(measure);
          await page.addScriptTag({ content: axeSource });
          const axe = await page.evaluate(async () => {
            // axe leaves WCAG 2.2 rules such as target-size out unless their tag is asked for.
            const tags = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa", "best-practice"];
            const r = await window.axe.run(document, { resultTypes: ["violations"], runOnly: { type: "tag", values: tags } });
            return r.violations.map((v) => ({ id: v.id, impact: v.impact, nodes: v.nodes.length }));
          });
          metrics[slug] = { ...metrics[slug], ...m, axe };
        }
        await ctx.close();
      }
    }
  }
  await browser.close();
  const summary = Object.fromEntries(Object.entries(metrics).map(([k, m]) => [k, {
    monoShare: m.monoShare, distinctFontSizes: m.distinctFontSizes, distinctHues: m.distinctHues,
    foldWords: m.foldWords, headings: m.headings, interactive: m.interactive, smallTargets: m.smallTargets,
    svg: m.svg, pageHeight: m.pageHeight, mobileOverflow: m.mobileOverflow,
    axeSeriousNodes: m.axe.filter((v) => v.impact === "serious" || v.impact === "critical").reduce((n, v) => n + v.nodes, 0),
    axe: m.axe.map((v) => `${v.id}:${v.impact}x${v.nodes}`).join(" "),
  }]));
  fs.writeFileSync(path.join(a.out, "metrics.json"), JSON.stringify({ url: a.url || a.storybook, summary, metrics }, null, 2) + "\n");
  console.log(JSON.stringify(summary, null, 2));
}

main().catch((e) => {
  console.error(e.message);
  process.exit(1);
});
