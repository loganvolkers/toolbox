import assert from "node:assert/strict";
import { execFileSync, spawn } from "node:child_process";
import { globSync } from "node:fs";
import { createServer } from "node:http";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { basename, dirname, extname, join, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const root = fileURLToPath(new URL("../../", import.meta.url));
const seikika = process.env.SEIKIKA_BIN ?? join(root, "target/debug/seikika");
const docs = resolve(root, "site/dist");
const fixture = resolve(root, "examples/reference-portal");
const id = "repository:payments-api";
const temp = await mkdtemp(join(tmpdir(), "seikika-ui-smoke-"));
const types = { ".html": "text/html", ".js": "text/javascript", ".json": "application/json", ".css": "text/css", ".svg": "image/svg+xml", ".woff2": "font/woff2" };
const site = createServer(async (request, response) => {
  const pathname = new URL(request.url, "http://localhost").pathname;
  if (pathname === "/favicon.ico") return response.writeHead(204).end();
  const offline = pathname === "/offline-artifact";
  const file = offline ? join(temp, "index.html") :
    resolve(docs, "." + decodeURIComponent(pathname), ...(pathname.endsWith("/") ? ["index.html"] : []));
  if (!offline && !file.startsWith(docs + sep)) return response.writeHead(404).end();
  try {
    const contents = await readFile(file);
    response.writeHead(200, { "Content-Type": types[extname(file)] ?? "application/octet-stream" }).end(contents);
  } catch {
    response.writeHead(404).end();
  }
});
const listen = server => new Promise((resolve, reject) => server.once("error", reject).listen(0, "127.0.0.1", resolve));

let portal, browser;
try {
  await listen(site);
  const docsOrigin = "http://127.0.0.1:" + site.address().port + "/";
  const keeper = createServer();
  await listen(keeper);
  const port = keeper.address().port;
  await new Promise(resolve => keeper.close(resolve));
  const portalOrigin = "http://127.0.0.1:" + port;
  portal = spawn(seikika, ["serve", fixture, "--fixtures", "--listen", "127.0.0.1:" + port], {
    env: { ...process.env, SEIKIKA_AUTH_MODE: "dev" },
    stdio: ["ignore", "ignore", "inherit"],
  });
  execFileSync(seikika, ["report", fixture, "--fixtures", "-o", "html", "--out", join(temp, "index.html")]);
  let ready = false;
  for (let check = 0; check < 60 && !ready; check++) {
    try { ready = (await fetch(portalOrigin + "/readyz")).ok; } catch {}
    if (!ready) await new Promise(resolve => setTimeout(resolve, 100));
  }
  assert(ready, "The fixture portal started");
  const entityUrl = portalOrigin + "/e/" + encodeURIComponent(id) + "?as=alice";
  const response = await fetch(entityUrl);
  assert.equal(response.status, 200);
  assert.equal(response.headers.get("cache-control"), "private, no-store");
  assert.match(await response.text(), /Owner: group:payments/);

  browser = await chromium.launch({ executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE });
  const page = await browser.newPage();
  page.setDefaultTimeout(10_000);
  const errors = [];
  page.on("pageerror", error => errors.push(error.message));
  await page.goto(entityUrl);
  await page.locator("#root").getByRole("heading", { name: "Payments API" }).waitFor();
  await page.locator("#root .asof code").waitFor();
  await page.getByRole("link", { name: "group:payments", exact: true }).first().waitFor();
  await page.getByRole("textbox", { name: "Search" }).fill("payments");
  await page.getByRole("textbox", { name: "Search" }).press("Enter");
  await page.getByRole("heading", { name: /Search.*payments/ }).waitFor();
  await page.getByRole("link", { name: "Assessments", exact: true }).click();
  await page.getByRole("heading", { name: "No assessment runs yet" }).waitFor();
  await page.getByRole("link", { name: "Rubrics 0" }).click();
  await page.getByText("This snapshot has no rubric definitions.").waitFor();

  const offlineUrl = new URL("/offline-artifact", docsOrigin).href;
  const offlineRequests = [];
  function recordRequest(request) {
    if (request.url() !== offlineUrl && new URL(request.url()).pathname !== "/favicon.ico") offlineRequests.push(request.url());
  }
  page.on("request", recordRequest);
  await page.goto(offlineUrl + "#entity/" + encodeURIComponent(id));
  await page.getByRole("heading", { name: "Payments API" }).waitFor();
  await page.locator("dl").getByRole("link", { name: "group:payments", exact: true }).waitFor();
  await page.getByRole("tab", { name: /Scorecards/ }).click();
  await page.getByRole("heading", { name: "Scorecards", exact: true }).waitFor();
  await page.keyboard.press("Home");
  await page.waitForFunction(() =>
    document.activeElement?.id === "tab-overview" &&
    document.activeElement.getAttribute("aria-selected") === "true");
  await page.getByRole("tab", { name: "Assessments" }).click();
  await page.getByRole("heading", { name: "No assessment runs yet" }).waitFor();
  page.off("request", recordRequest);
  assert.deepEqual(offlineRequests, [], "The offline artifact must make no requests for code, styles, or data.");

  await page.goto(docsOrigin);
  assert.equal(await page.locator('link[rel="canonical"]').getAttribute("href"), "https://seikika.com/");
  await page.getByRole("main").getByRole("link", { name: "Browse connectors", exact: true }).click();
  await page.getByRole("heading", { name: "Connector directory", exact: true }).waitFor();
  const guideLinks = 'a[href^="/connectors/"]:not([href="/connectors/"])';
  const directoryGuides = await page.locator(".sl-markdown-content").locator(guideLinks)
    .evaluateAll(links => links.map(link => link.pathname).sort());
  const sidebarGuides = await page.locator("#starlight__sidebar").locator(guideLinks)
    .evaluateAll(links => links.map(link => link.pathname).filter(path => /^\/connectors\/[^/]+\/$/.test(path)).sort());
  assert(directoryGuides.length > 0, "The connector directory lists published guides.");
  assert.deepEqual(directoryGuides, sidebarGuides, "The directory lists the same connector landing pages as the sidebar.");
  await page.goto(docsOrigin);
  await page.getByRole("main").getByRole("link", { name: "Browse standards packages", exact: true }).click();
  await page.getByRole("heading", { name: "Standards package directory", exact: true }).waitFor();
  const packageGuides = await page.locator(".sl-markdown-content .sl-link-card a")
    .evaluateAll(links => links.map(link => link.pathname).sort());
  const packageRoutes = globSync("packages/*/seikika-package.yaml", { cwd: root })
    .map(path => `/packages/${basename(dirname(path))}/`).sort();
  assert(packageRoutes.length > 0, "There are first-party standards packages.");
  assert.deepEqual(packageGuides, packageRoutes, "Every package manifest has a public directory entry.");
  const sidebarPackages = await page.locator('#starlight__sidebar a[href^="/packages/"]')
    .evaluateAll(links => links.map(link => link.pathname).filter(path => /^\/packages\/[^/]+\/$/.test(path)).sort());
  assert.deepEqual(sidebarPackages, packageRoutes, "Every package guide is reachable from the sidebar.");

  await page.goto(docsOrigin);
  await page.getByRole("main").getByRole("link", { name: "Browse Reference", exact: true }).click();
  await page.getByRole("heading", { name: "Reference", exact: true }).waitFor();
  const sidebar = page.locator("#starlight__sidebar .top-level");
  const referenceGroup = sidebar.locator(":scope > li").filter({
    has: page.locator("summary > .group-label").getByText("Reference", { exact: true }),
  });
  const guideGroup = sidebar.locator(":scope > li").filter({
    has: page.locator("summary > .group-label").getByText("Guides", { exact: true }),
  });
  assert.equal(await referenceGroup.count(), 1, "Reference is a top-level navigation group.");
  assert.equal(await guideGroup.count(), 1, "Guides is a sibling of Reference.");
  const lookups = await page.getByRole("main").locator(".sl-link-card a").evaluateAll(
    links => links.map(link => link.pathname));
  assert.deepEqual(lookups, [
    "/users/api/", "/reference/mcp/", "/start/cli/", "/reference/sdks/",
    "/operators/manifest/", "/reference/contracts/",
  ]);
  for (const path of lookups) {
    assert.equal(await referenceGroup.locator(`a[href="${path}"]`).count(), 1, path);
    assert.equal((await page.request.get(new URL(path, docsOrigin).href)).status(), 200, path);
  }
  await page.getByRole("main").getByRole("link", { name: "MCP tools", exact: true }).click();
  const toolIndex = page.getByRole("navigation", { name: "Built-in tools" });
  const advertisedTools = execFileSync(seikika, ["describe"], { encoding: "utf8" }).trim().split("\n")
    .map(line => JSON.parse(line)).filter(record => record.kind === "mcp-tool").map(record => record.data.name).sort();
  assert.deepEqual((await toolIndex.getByRole("link").allTextContents()).sort(), advertisedTools);
  await toolIndex.getByRole("link", { name: "graph", exact: true }).click();
  const graph = page.getByRole("region", { name: "graph", exact: true });
  const direction = graph.getByRole("row").filter({ hasText: "direction" });
  assert.deepEqual(await direction.getByRole("cell").allTextContents().then(cells => cells.slice(0, 2)), ["No", "in, out, both"]);
  await graph.getByRole("link", { name: "seikika graph", exact: true }).click();
  await page.locator("#graph").getByRole("heading", { name: "seikika graph" }).waitFor();
  assert.equal(new URL(page.url()).hash, "#graph");
  await page.goto(docsOrigin + "users/api/#action-runs");
  await page.locator("#action-runs").waitFor();

  await page.goto(docsOrigin + "report/");
  await page.getByRole("tab", { name: /Catalog/ }).click();
  await page.locator("#report-body").getByRole("heading", { name: "Catalog", exact: true }).waitFor();
  await page.goto(docsOrigin + "demo/organization/");
  await page.getByRole("heading", { name: "Seikika organization scorecards", exact: true }).waitFor();
  if (process.env.SEIKIKA_ORG_REPORT_DATA) {
    const snapshot = JSON.parse(await readFile(process.env.SEIKIKA_ORG_REPORT_DATA, "utf8"));
    const names = snapshot.catalog.Repository.map(id => snapshot.entityOverview[id].name.toLowerCase());
    const workflows = snapshot.catalog.Workflow;
    assert(workflows.length > 0, "The organization workflow inventory is populated.");
    await page.evaluate(() => location.hash = "catalog/Workflow");
    await page.getByRole("combobox", { name: "Group entities" }).waitFor();
    const workflowSource = snapshot.entities[workflows[0]].fields.repository;
    const rows = page.locator(".federated-catalog-row");
    const more = page.getByRole("button", { name: /Show more/ });
    while (await more.isVisible()) await more.click();
    assert.equal(await rows.count(), workflows.length);
    await page.getByRole("combobox", { name: "Group entities" }).selectOption("repository");
    assert.equal(await page.locator(".federated-catalog-group").count(),
      new Set(workflows.map(id => snapshot.entities[id].fields.repository)).size);
    await page.getByRole("combobox", { name: "Source repository" }).selectOption({ label: "Source repository · " + workflowSource });
    assert.equal(await rows.count(), workflows.filter(id => snapshot.entities[id].fields.repository === workflowSource).length);
    assert.equal(await page.locator(".federated-catalog-group").count(), 1);
    await page.locator(".federated-catalog-group summary").click();
    await rows.first().getByRole("link").first().click();
    await page.getByRole("link", { name: /Resolve zizmor findings · Federation standard/ }).waitFor();
    await page.locator('a[href^="#track/"]').filter({ hasText: /Workflow security · Federation standard/ }).waitFor();
    await page.getByText("Repository hygiene inventory", { exact: true }).click();
    const table = page.getByRole("region", { name: "Organization scores" }).getByRole("table");
    assert.deepEqual(await table.getByRole("rowheader").locator(":scope > a").allTextContents(), names);
    console.log(`The organization report renders ${names.length} public repositories.`);
    const example = table.getByRole("row").filter({ hasText: "seikika/example-malicious-repo" });
    const exampleScore = snapshot.scores["repo-hygiene"]["repository:seikika/example-malicious-repo"];
    const marker = exampleScore?.evidence?.["has-readme"]?.find(text => text.includes("<federation-marker>"));
    if (marker) {
      await example.locator("summary").click();
      await example.getByText(marker, { exact: true }).waitFor();
      assert.equal(await example.locator("federation-marker").count(), 0);
    }
  }
  await page.goto(docsOrigin + "storybook/?path=/story/hosted-portal--entity");
  await page.frameLocator("#storybook-preview-iframe").getByRole("heading", { name: "Checkout", exact: true }).waitFor();
  await page.goto(docsOrigin + "start/cli/");
  await page.getByRole("searchbox", { name: "Filter commands" }).fill("report");
  await page.getByText(/\d+ of \d+ commands/).waitFor();
  await page.goto(docsOrigin + "authors/npm/");
  await page.getByRole("link", { name: "@seikika/astro-report", exact: true }).waitFor();
  await page.getByRole("link", { name: "@seikika/ui", exact: true }).waitFor();

  for (const width of [320, 390]) {
    await page.setViewportSize({ width, height: 844 });
    await page.goto(docsOrigin + "demo/organization/#catalog/Workflow?group=repository");
    if (process.env.SEIKIKA_ORG_REPORT_DATA) {
      await page.getByRole("combobox", { name: "Group entities" }).waitFor();
      await page.waitForFunction(() => document.querySelector('[role="tablist"]').getBoundingClientRect().top >=
        document.querySelector('header.header').getBoundingClientRect().bottom);
    }
    assert(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth),
      `The grouped organization report fits a ${width}px viewport.`);
  }
  await page.goto(docsOrigin);
  await page.locator('button[popovertarget="starlight__sidebar"]').click();
  const mobileReference = page.locator("#starlight__sidebar").getByRole("link", { name: "Reference overview" });
  await mobileReference.focus();
  await page.keyboard.press("Enter");
  await page.getByRole("heading", { name: "Reference", exact: true }).waitFor();
  assert(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth),
    "The Reference home fits the mobile viewport.");
  await page.getByRole("main").getByRole("link", { name: "MCP tools", exact: true }).click();
  await page.getByRole("table", { name: "Arguments for graph" }).waitFor();
  assert(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth),
    "The MCP argument tables fit the mobile viewport.");
  const story = id => docsOrigin + "storybook/iframe.html?viewMode=story&id=" + id;
  await page.goto(story("pages-seikika-for-seikika--scorecards"));
  await page.getByRole("button", { name: "Standards that check themselves", exact: true }).click();
  await page.waitForFunction(() => document.activeElement?.textContent?.includes("Standards that check themselves") &&
    document.activeElement.tagName === "H3");
  await page.goto(story("pages-seikika-for-seikika--scorecard"));
  const filterJump = page.getByRole("button", { name: /Filter entities/ });
  await filterJump.waitFor();
  assert(await filterJump.evaluate(button => {
    const { top, bottom } = button.getBoundingClientRect();
    return top >= 0 && bottom <= window.innerHeight;
  }), "The filters shortcut must be visible on entry at mobile width.");
  await filterJump.click();
  await page.waitForFunction(() => document.activeElement?.textContent === "Entities");
  assert.deepEqual(errors, []);
  console.log("Hosted portal, offline report, Pages report, Storybook, reference navigation, CLI and MCP references, and npm, connector, and package directories passed.");
} finally {
  await browser?.close();
  portal?.kill();
  site.close();
  await rm(temp, { recursive: true, force: true });
}
