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

  const offlineUrl = new URL("/offline-artifact", docsOrigin).href;
  const offlineRequests = [];
  function recordRequest(request) {
    if (request.url() !== offlineUrl && new URL(request.url()).pathname !== "/favicon.ico") offlineRequests.push(request.url());
  }
  page.on("request", recordRequest);
  await page.goto(offlineUrl + "#entity/" + encodeURIComponent(id));
  await page.getByRole("heading", { name: "Payments API" }).waitFor();
  await page.getByRole("link", { name: "group:payments", exact: true }).waitFor();
  await page.getByRole("tab", { name: /Scorecards/ }).click();
  await page.getByRole("heading", { name: "Scorecards", exact: true }).waitFor();
  await page.keyboard.press("Home");
  await page.waitForFunction(() =>
    document.activeElement?.id === "tab-overview" &&
    document.activeElement.getAttribute("aria-selected") === "true");
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
  await page.goto(docsOrigin + "report/");
  await page.getByRole("tab", { name: /Catalog/ }).click();
  await page.locator("#report-body").getByRole("heading", { name: "Catalog", exact: true }).waitFor();
  await page.goto(docsOrigin + "storybook/?path=/story/hosted-portal--entity");
  await page.frameLocator("#storybook-preview-iframe").getByRole("heading", { name: "Checkout", exact: true }).waitFor();
  await page.goto(docsOrigin + "start/cli/");
  await page.getByRole("searchbox", { name: "Filter commands" }).fill("report");
  await page.getByText(/\d+ of \d+ commands/).waitFor();
  await page.goto(docsOrigin + "authors/npm/");
  await page.getByRole("link", { name: "@seikika/astro-report", exact: true }).waitFor();
  await page.getByRole("link", { name: "@seikika/ui", exact: true }).waitFor();

  await page.setViewportSize({ width: 390, height: 844 });
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
  console.log("Hosted portal, offline report, Pages report, Storybook, CLI reference, and npm, connector, and package directories passed.");
} finally {
  await browser?.close();
  portal?.kill();
  site.close();
  await rm(temp, { recursive: true, force: true });
}
