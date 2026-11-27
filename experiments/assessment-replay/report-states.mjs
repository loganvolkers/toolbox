// Exercise two historical report defects using the exact merged JavaScript and shared fixtures.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createServer } from "node:http";
import { createRequire } from "node:module";

const require = createRequire(new URL("../../report-ui/package.json", import.meta.url));
const { chromium } = require("playwright");
const before = "b15820e";
const after = "08f071a";
const show = (ref, path) => execFileSync("git", ["show", `${ref}:${path}`], {
  encoding: "utf8", maxBuffer: 32 * 1024 * 1024,
});
const render = (ref, fixture) => {
  const script = show(ref, "crates/toolbox-cli/src/toolbox-report.js");
  const data = show(after, `report-ui/fixtures/${fixture}.json`);
  const template = show(ref, "crates/toolbox-cli/src/report.html");
  return template.replace("/*ELEMENT*/", () => script)
    .replace("/*DATA*/", () => data.replaceAll("</", "<\\/"));
};

const pages = new Map([before, after].flatMap(ref => ["t4t", "openfga-unmeasured"].map(
  fixture => [`/${ref}/${fixture}`, render(ref, fixture)],
)));
const server = createServer((req, res) => {
  const body = pages.get(req.url);
  res.writeHead(body ? 200 : 404, { "Content-Type": "text/html; charset=utf-8" }).end(body);
});
let browser;
try {
  await new Promise((resolve, reject) => server.once("error", reject).listen(0, "127.0.0.1", resolve));
  const base = `http://127.0.0.1:${server.address().port}`;
  browser = await chromium.launch({ executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE });
  const result = {};
  for (const ref of [before, after]) {
    const page = await browser.newPage({ viewport: { width: 1280, height: 800 }, reducedMotion: "reduce" });
    await page.goto(`${base}/${ref}/t4t#campaign/close-review-findings`);
    const hollow = page.locator("toolbox-report .kpi-bar.short span.hollow");
    await hollow.first().waitFor();
    const backgrounds = await hollow.evaluateAll(nodes => nodes.map(node => getComputedStyle(node).backgroundImage));
    await page.goto(`${base}/${ref}/openfga-unmeasured#tracks`);
    const track = page.locator("toolbox-report .track").filter({ hasText: /SpiceDB/i });
    await track.first().waitFor();
    const text = (await track.allTextContents()).join(" ");
    result[ref] = {
      short_bar_segments: backgrounds.length,
      hatched_segments: backgrounds.filter(value => value.includes("repeating-linear-gradient")).length,
      empty_comparison_says_never_behind: /never behind/i.test(text),
      empty_comparison_says_not_measured: /not measured yet/i.test(text),
    };
    await page.close();
  }
  assert.equal(result[before].hatched_segments, 0);
  assert.equal(result[after].hatched_segments, result[after].short_bar_segments);
  assert.equal(result[before].empty_comparison_says_never_behind, true);
  assert.equal(result[after].empty_comparison_says_not_measured, true);
  console.log(JSON.stringify(result));
} finally {
  await browser?.close();
  server.close();
}
