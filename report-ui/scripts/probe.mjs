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
import assert from "node:assert/strict";

const require = createRequire(import.meta.url);
const axeSource = readFileSync(require.resolve("axe-core/axe.min.js"), "utf8");
const argv = process.argv.slice(2);
const opt = (name) => {
  const i = argv.indexOf("--" + name);
  return i >= 0 ? argv[i + 1] : undefined;
};
const dir = resolve(argv.find((a, i) => !a.startsWith("--") && !(i && argv[i - 1].startsWith("--"))) || "storybook-static");
const filter = opt("filter");
const contractsOnly = argv.includes("--contracts-only");
const contracts = {
  "hosted-portal--meta-scorecard": async (page) => {
    const body = page.locator("#report-body");
    await body.getByText("CEL filter over", { exact: false }).waitFor();
    await body.locator('#kind-list a[href="#scorecard/readiness"]').click();
    await body.getByRole("heading", { name: "Production readiness", exact: true }).waitFor();
    await body.getByRole("button", { name: "Refresh", exact: true }).waitFor();
    assert.equal(await body.locator(".report-catalog-details .asof code").textContent(), "demo-1");
    await page.evaluate(() => { location.hash = "catalog/Campaign"; });
    await body.locator('#kind-list a[href="#campaign/production"]').click();
    await body.getByRole("link", { name: "Browse Campaign in Catalog" }).click();
    await body.locator('#kind-list a[href="#campaign/production"]').waitFor();
    assert.equal(await body.getByText("CEL filter over", { exact: false }).count(), 0);
  },
  "pages-seikika-for-seikika--meta-catalog": async (page) => {
    const body = page.locator("#report-body");
    const navigate = path => page.evaluate(path => { location.hash = path; }, path);
    const cases = [
      ["Scorecard", "scorecard:seikika/maintainer-skill", "scorecard/maintainer-skill", "How this scorecard is evaluated"],
      ["Rule", "rule:seikika/maintainer-skill/has-example", "rule/maintainer-skill/has-example", "How it is decided"],
      ["Campaign", "campaign:seikika/cli-skill-coverage", "campaign/cli-skill-coverage", "CLI skills present"],
      ["Track", "track:seikika/delivery", "track/ci-and-supply-chain", "CI and supply chain"],
      ["Scorecard", "scorecard:seikika/vs-backstage-capabilities", "compare/vs-backstage-capabilities", "Every factor"],
      ["Rubric", "rubric:seikika/testing-contract-quality", "rubric/rubric%3Aseikika%2Ftesting-contract-quality", "Rubric criteria"],
      ["Assessment", "assessment:crate-design/crate:seikika-expr", "assessment/assessment%3Acrate-design%2Fcrate%3Aseikika-expr", "Criterion results"],
      ["Action", "action:seikika/Review skill", "action/action%3Aseikika%2FReview%20skill", "Review skill"],
    ];
    for (const [kind, id, path, heading] of cases) {
      if (kind === "Action") {
        await page.getByRole("tab", { name: /^Actions/ }).click();
        await body.getByRole("combobox", { name: "Action" }).selectOption({ label: "Review skill" });
        assert.equal(new URL(page.url()).hash, "#" + path);
      } else {
        await navigate(path);
      }
      await body.getByRole("heading", { name: heading, exact: true }).waitFor();
      const backToKind = body.getByRole("link", { name: `Browse ${kind} in Catalog`, exact: true });
      await backToKind.waitFor();
      const detail = await body.innerText();
      await backToKind.click();
      const list = body.locator("#kind-list");
      await list.getByRole("searchbox").fill(id);
      await list.locator(`a[href="#${path}"]`).click();
      await body.getByRole("heading", { name: heading, exact: true }).waitFor();
      await backToKind.waitFor();
      assert.equal(await body.innerText(), detail, `${kind} detail from the catalog`);
      await backToKind.click();
      await list.waitFor();
      const query = kind === "Rubric" ? "?q=testing%20contract" : "";
      await navigate("entity/" + encodeURIComponent(id) + query);
      await page.waitForURL(url => url.hash === "#" + path + query);
      await backToKind.waitFor();
      assert.equal(await body.innerText(), detail, `${kind} detail from an existing entity link`);
      if (kind === "Rubric") {
        await page.goBack();
        await list.waitFor();
        await page.goForward();
        await page.waitForURL(url => url.hash === "#" + path + query);
        await backToKind.waitFor();
      }
    }
    await page.getByRole("tab", { name: /^Campaigns/ }).click();
    await body.getByRole("link", { name: "Browse Campaign in Catalog" }).waitFor();
    await navigate("scorecard/maintainer-skill");
    const evaluated = "How this scorecard is evaluated";
    await body.getByRole("button", { name: evaluated, exact: true }).press("Enter");
    await page.waitForFunction(title => document.activeElement?.tagName === "H2" && document.activeElement.textContent === title, evaluated);
    await body.getByText("Show rule results", { exact: true }).click();
    await body.locator(".report-catalog-scores .sc-block").first().waitFor();
    await body.getByText("Catalog fields and relationships", { exact: true }).click();
    await body.getByText("Skill scorecard catalog field", { exact: true }).waitFor();
    await body.getByRole("link", { name: "rule:seikika/maintainer-skill/has-example", exact: true }).click();
    await body.getByRole("heading", { name: "How it is decided" }).waitFor();
    await navigate("entity/" + encodeURIComponent("scorecard:example/maintainer-skill"));
    await body.getByText("Example portal catalog field", { exact: true }).waitFor();
    assert.equal(await body.getByRole("heading", { name: "Catalog details" }).count(), 0);
  },
  "pages-assessments--rubric-detail": async (page) => {
    await page.waitForURL(url => url.hash === "#rubric/rubric%3Aseikika%2Ftesting-contract-quality");
    await page.getByRole("heading", { name: "Testing contract quality", exact: true }).waitFor();
    await page.getByRole("heading", { name: "Rubric criteria", exact: true }).waitFor();
  },
  "components--campaign-bars": async (page) => {
    const segments = await page.locator(".kpi-bar.short span.hollow").evaluateAll(
      (nodes) => nodes.map((node) => getComputedStyle(node).backgroundImage),
    );
    return segments.length === 1 && segments[0].includes("repeating-linear-gradient")
      ? [] : ["the unassessed part of a below-target campaign must render hatched"];
  },
  "states-comparison-not-measured-yet--tracks": async (page) => {
    const tracks = await page.locator(".track").filter({ hasText: "SpiceDB" }).allInnerTexts();
    return tracks.length === 1 && /not measured yet/i.test(tracks[0]) && !/never behind/i.test(tracks[0])
      ? [] : ["an unmeasured comparison track must report that it has no measurements"];
  },
  "pages-assessments--recent-runs": async (page) => {
    await page.getByRole("button", { name: "Show more runs" }).click();
    assert.ok(await page.locator(".assessment-runs li").count() > 12);
    await page.getByRole("searchbox", { name: "Search runs" }).fill("seikika-expr");
    await page.locator(".assessment-runs li").filter({ hasText: "Historical review" }).getByText("Stale · same assessor").waitFor();
    const run = page.locator(".assessment-runs").getByRole("link", { name: "seikika-expr", exact: true });
    await run.click();
    await page.getByRole("heading", { name: "Criterion results" }).waitFor();
    await page.goBack();
    await page.getByRole("searchbox", { name: "Search runs" }).waitFor();
    assert.equal(await page.getByRole("searchbox", { name: "Search runs" }).inputValue(), "seikika-expr");
    await run.click();
    await page.getByRole("heading", { name: "Criterion results" }).waitFor();
    await page.getByRole("heading", { name: "Assessment quality scorecards" }).waitFor();
    await page.getByText("4 of 4 criteria passed", { exact: true }).waitFor();
    await page.getByText("No current independent review", { exact: true }).waitFor();
    await page.getByText("Recorded verdict", { exact: false }).waitFor();
    await page.locator(".assessment-criteria summary").first().focus();
    await page.keyboard.press("Enter");
    assert.equal(await page.locator(".assessment-criteria details").first().getAttribute("open"), "");
    await page.getByRole("link", { name: "Assessments are well-formed" }).waitFor();
    await page.getByRole("link", { name: "Read the recorded review" }).click();
    await page.getByRole("heading", { name: /Historical review of .*seikika-expr/ }).waitFor();
    await page.getByText("Recorded review verdict", { exact: true }).waitFor();
    await page.getByText("Stale · same assessor", { exact: true }).waitFor();
    assert.equal(await page.locator(".assessment-result .assessment-verdict").getAttribute("data-verdict"), "recorded");
    await page.getByRole("link", { name: "Assessment quality rubric", exact: true }).click();
    await page.getByRole("heading", { name: "Rubric criteria" }).waitFor();
    assert.match(await page.getByRole("link", { name: "How assessments are reviewed" }).getAttribute("href"), /^https:/);
    await page.getByRole("link", { name: /^View \d+ run/ }).click();
    await page.getByRole("region", { name: "Recent runs" }).waitFor();
    assert.equal(await page.getByRole("combobox", { name: "Rubric" }).inputValue(), "rubric:seikika/assessment-quality");
    await page.getByRole("searchbox", { name: "Search runs" }).fill("this-does-not-match");
    await page.getByRole("button", { name: "Clear filters" }).click();
    assert.equal(await page.getByRole("combobox", { name: "Rubric" }).inputValue(), "");
  },
  "pages-assessments--timed-run": async (page) => {
    const first = page.locator(".assessment-runs li").first();
    assert.match(await first.innerText(), /seikika-expr/i);
    assert.equal(await first.locator("time").innerText(), "Nov 27, 2026, 20:42 UTC");
    assert.equal(await first.locator("time").getAttribute("datetime"), "2026-11-27T21:42:00+01:00");
    const dateOnly = page.locator('.assessment-runs time[datetime="2026-11-27"]').first();
    assert.equal(await dateOnly.innerText(), "Nov 27, 2026");
    await first.getByRole("link").click();
    await page.getByRole("heading", { name: "Criterion results" }).waitFor();
    assert.equal(await page.locator('.assessment-result time[datetime="2026-11-27T21:42:00+01:00"]').innerText(), "Nov 27, 2026, 20:42 UTC");
  },
  "pages-assessments--no-assessments": async (page) => {
    await page.getByRole("heading", { name: "No assessment runs yet" }).waitFor();
    await page.getByRole("link", { name: "Rubrics 0" }).click();
    await page.getByText("This snapshot has no rubric definitions.").waitFor();
  },
  "pages-assessments--federated-runs": async (page) => {
    await page.getByRole("searchbox", { name: "Search runs" }).fill("seikika-expr");
    const run = page.locator(".assessment-runs").getByRole("link", { name: "seikika-expr", exact: true });
    assert.match(await run.getAttribute("href"), /^#assessment\/%40seikika/);
    await run.click();
    await page.getByRole("link", { name: "Crate design rubric" }).click();
    await page.getByRole("link", { name: "Seikika workspace and crate boundaries" }).waitFor();
    await page.getByRole("heading", { name: "Rubric quality" }).waitFor();
    await page.getByRole("link", { name: "Assessments", exact: true }).click();
    await page.getByRole("searchbox", { name: "Search runs" }).fill("unrelated");
    await page.getByText("No runs match your filters.").waitFor();
  },
  "hosted-portal--public-catalog": async (page) => {
    assert.equal(await page.getByRole("tab").count(), 1);
    await page.getByRole("link", { name: "Public scorecards" }).waitFor();
    assert.equal(await page.getByRole("link", { name: "Meta scoring", exact: true }).getAttribute("href"),
      "https://seikika.com/demo/organization/#scorecard/meta-scorecard");
    assert.equal(await page.getByRole("link", { name: "Operations" }).getAttribute("href"), "/ops");
    await page.locator('a[href="#catalog/Repository"]').first().click();
    assert.equal(await page.getByRole("textbox", { name: /CEL filter/ }).count(), 0);
    await page.locator('a[href="#entity/repository%3Aledger"]').first().click();
    await page.getByRole("heading", { name: "example/ledger", exact: true }).waitFor();
    assert.equal(await page.getByRole("link", { name: "Source" }).getAttribute("href"), "https://github.com/example/ledger");
    assert.equal(await page.getByRole("button", { name: "Refresh" }).count(), 0);
  },
  "hosted-portal--guest-report": async (page) => {
    await page.getByRole("tab", { name: /Catalog/ }).waitFor();
    await page.getByRole("link", { name: "Meta scoring", exact: true }).click();
    await page.waitForURL(url => url.hash === "#scorecard/meta-scorecard");
    await page.locator("#report-body").getByRole("heading", { name: /Scorecards are well-formed/ }).waitFor();
    assert.equal(await page.getByRole("link", { name: "Operations" }).getAttribute("href"), "/ops");
    const search = page.getByRole("searchbox", { name: "Search", exact: true });
    await search.fill("meta-scorecard");
    await search.press("Enter");
    await page.getByRole("row").filter({ hasText: "meta-scorecard" }).first().waitFor();
  },
  "hosted-portal--assessments": async (page) => {
    await page.getByRole("tab", { name: "Assessments", exact: true }).waitFor();
    const run = page.locator(".assessment-runs li").first().getByRole("link");
    assert.match(await run.getAttribute("href"), /^#assessment\//);
  },
  "hosted-portal--assessment-run": async (page) => {
    await page.getByRole("heading", { name: "Criterion results" }).waitFor();
    assert.match(await page.getByRole("link", { name: "Crate design rubric" }).getAttribute("href"), /^#rubric\//);
    assert.equal(await page.getByRole("link", { name: "Assessments are well-formed" }).getAttribute("href"), "#scorecard/meta-assessment");
  },
};
const VIEWS = [
  { name: "desktop-light", viewport: { width: 1280, height: 800 }, colorScheme: "light" },
  { name: "desktop-dark", viewport: { width: 1280, height: 800 }, colorScheme: "dark" },
  { name: "mobile-light", viewport: { width: 390, height: 844 }, colorScheme: "light" },
  { name: "mobile-dark", viewport: { width: 390, height: 844 }, colorScheme: "dark" },
];
const TAGS = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"];
const TYPES = { ".html": "text/html", ".js": "text/javascript", ".json": "application/json", ".css": "text/css", ".svg": "image/svg+xml", ".woff2": "font/woff2" };

const server = createServer((req, res) => {
  const pathname = new URL(req.url, "http://x").pathname;
  if (pathname === "/favicon.ico") {
    res.writeHead(204).end();
    return;
  }
  const path = join(dir, decodeURIComponent(pathname));
  if (!path.startsWith(dir) || !existsSync(path) || statSync(path).isDirectory()) {
    res.writeHead(404).end();
    return;
  }
  res.writeHead(200, { "content-type": TYPES[extname(path)] || "application/octet-stream" }).end(readFileSync(path));
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const base = "http://127.0.0.1:" + server.address().port;

const index = JSON.parse(readFileSync(join(dir, "index.json"), "utf8"));
const stories = Object.values(index.entries).filter((e) => e.type === "story"
  && (!filter || e.id.includes(filter)) && (!contractsOnly || Object.hasOwn(contracts, e.id)));
const missingContracts = Object.keys(contracts).filter(id => (!filter || id.includes(filter))
  && !stories.some(story => story.id === id));
if (!stories.length || missingContracts.length) {
  server.close();
  throw new Error("No matching stories or missing reader-task stories: " + missingContracts.join(", "));
}

async function probe(context, story, view) {
  const page = await context.newPage();
  try {
    return await check(page, story, view);
  } finally {
    await page.close();
  }
}

async function check(page, story, view) {
  const errors = [];
  page.on("pageerror", (e) => errors.push(e.message));
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text() + " (" + message.location().url + ")");
  });
  await page.goto(base + "/iframe.html?viewMode=story&id=" + story.id);
  await page.waitForFunction(() => {
    const root = document.querySelector("#storybook-root");
    if (!root || !root.firstElementChild) return false;
    const report = root.querySelector("#report-body");
    return !report || !!report.querySelector(".view")?.firstElementChild;
  }, null, { timeout: 15000 });
  await page.waitForTimeout(100);
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
  // Text squeezed into a column a few letters wide ("r u n" in a flex callout) overflows nothing,
  // so look for runs of text far taller than they are wide.
  const stacked = await page.evaluate(() => {
    const out = [];
    const walk = document.createTreeWalker(document, NodeFilter.SHOW_TEXT);
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
    ...(await contracts[story.id]?.(page) ?? []),
  ];
  return { story: story.id, view: view.name, overflow, axe, errors, failures };
}

const browser = await chromium.launch({ executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE });
// Keep browser caches for the shared Storybook assets. Each case uses a fresh page.
const contexts = await Promise.all(VIEWS.map(view =>
  browser.newContext({ viewport: view.viewport, colorScheme: view.colorScheme, reducedMotion: "reduce" }),
));
const jobs = stories.flatMap(story => VIEWS.map((view, index) => [story, view, contexts[index]]));
const results = [];
let next = 0;
await Promise.all(
  Array.from({ length: 4 }, async () => {
    while (next < jobs.length) {
      const [s, v, context] = jobs[next++];
      // A render timeout on a busy machine is retried once and reported; any other failure is not.
      for (let attempt = 1; ; attempt++) {
        try {
          results.push({ ...(await probe(context, s, v)), attempts: attempt });
          if (attempt > 1) console.log(`RETRY ${s.id} [${v.name}] rendered on attempt ${attempt}`);
          break;
        } catch (e) {
          if (attempt < 2) continue;
          results.push({ story: s.id, view: v.name, attempts: attempt, failures: ["did not render: " + e.message.split("\n")[0]] });
          break;
        }
      }
      if (results.length % 40 === 0) console.log(`Probed ${results.length}/${jobs.length} story views`);
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
