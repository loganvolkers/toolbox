import assert from "node:assert/strict";
import { execFileSync, spawn } from "node:child_process";
import { createServer } from "node:http";
import { copyFile, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { chromium } from "playwright";

const binary = process.env.SEIKIKA_BIN;
assert(binary, "Set SEIKIKA_BIN to the built CLI");
const directory = await mkdtemp(join(tmpdir(), "seikika-federated-report-"));
const run = (cwd, cmd, ...args) => execFileSync(cmd, args, { cwd, encoding: "utf8" });
const git = (cwd, ...args) => run(cwd, "git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.test", ...args);
const listen = server => new Promise((resolve, reject) =>
  server.once("error", reject).listen(0, "127.0.0.1", resolve));
const pause = () => new Promise(resolve => setTimeout(resolve, 100));

function manifest(portal, scope = "") {
  return `apiVersion: seikika.com/v1alpha1
portal: {name: ${portal}, title: "${portal} portal", owner: "group:maintainers", seikika: "0.2"}
connectors:
  - name: workspace
    package: builtin:workspace
    runtime: in-image
    config: {crates: false, data: [{kind: workflow-fact, globs: [workflow.json]}]}
    records: [workflow-fact]
kinds:
  - name: Workflow
    title: GitHub Actions workflows
    identityFrom: [connector:workspace]
    fields:
      title: {type: string}
      findings: {type: integer}
      notes: {type: string}
    sources: {default: [connector:workspace]}
    map:
      - from: workspace/workflow-fact
        id: record.id
        set: record
tracks: [{name: workflow-security, title: Workflow security, owner: "group:maintainers"}]
campaigns:
  - name: zizmor
    title: Resolve zizmor findings
    owner: "group:maintainers"
    track: workflow-security
    status: active
    includes: [{scorecard: workflow-zizmor, atLeast: bronze}]
scorecards:
  - name: workflow-zizmor
    title: Workflow audit
    kind: Workflow
    owner: "group:maintainers"
    levels: [bronze]
    track: workflow-security
    facets: {summary: public, details: public}
    rules:
      - name: zizmor-clean
        level: bronze
        check: entity.findings == 0
        evidence: "['zizmor findings require review']"
        fix: Review the zizmor findings.
actions:
  - name: review-workflow
    kind: Workflow
    description: Review this workflow.
    permission: can_run_actions
    inputs:
      type: object
      required: [notes]
      properties:
        notes: {type: string, title: Reviewer notes}
        team: {type: string, title: Review team, enumFrom: "['release', 'security']"}
    executor: {type: command, run: "printf 'Done'"}
    unverified: true
${scope}`;
}

async function configure(root, portal, findings, scope) {
  await mkdir(root, { recursive: true });
  await writeFile(join(root, "seikika.yaml"), manifest(portal, scope));
  await writeFile(join(root, "workflow.json"), JSON.stringify([{
    id: "workflow:ci", title: portal + " workflow", findings, notes: portal + " catalog data",
  }]));
}

const parent = join(directory, "parent");
const artifact = join(directory, "offline");
let server, browser, site;
try {
  for (const [key, findings] of [["alpha", 1], ["beta", 0]]) {
    const repo = join(directory, key);
    const child = join(repo, ".seikika");
    await configure(child, key, findings);
    await copyFile(join(child, "workflow.json"), join(repo, "workflow.json"));
    git(repo, "init", "-q", "-b", "main");
    run(repo, binary, "report", child, "-o", "web", "--out", join(directory, key + "-snapshot"));
    await copyFile(join(directory, key + "-snapshot", "report.json"), join(child, "report.json"));
    git(repo, "add", ".");
    git(repo, "commit", "-qm", "Publish child report");
  }
  const bad = join(directory, "invalid");
  await configure(join(bad, ".seikika"), "invalid", 0);
  await writeFile(join(bad, ".seikika", "report.json"), '{"catalog":{},"scorecards":[],"scores":{}}');
  git(bad, "init", "-q", "-b", "main");
  git(bad, "add", ".");
  git(bad, "commit", "-qm", "Publish an invalid report");
  await configure(parent, "parent", 0, `federation:
  children:
    - {git: "../alpha", key: alpha}
    - {git: "../beta", key: beta}
    - {git: "../invalid", key: invalid}
`);
  const env = { ...process.env, SEIKIKA_STATE_DIR: join(directory, "state") };
  delete env.DATABASE_URL;
  execFileSync(binary, ["report", parent, "-o", "web", "--out", artifact], { cwd: parent, env, stdio: "inherit" });
  const snapshot = JSON.parse(await readFile(join(artifact, "report.json"), "utf8"));
  assert.deepEqual(snapshot.children.map(child => child.key), ["alpha", "beta", "invalid"]);
  assert(snapshot.children.slice(0, 2).every(child => child.report.catalog.Workflow.includes("workflow:ci")));
  assert.equal(snapshot.children[2].error, "The child snapshot could not be read.");

  site = createServer(async (_, response) =>
    response.writeHead(200, { "Content-Type": "text/html" }).end(await readFile(join(artifact, "index.html"))));
  await listen(site);
  const offline = "http://127.0.0.1:" + site.address().port;
  const socket = createServer();
  await listen(socket);
  const listenAddress = "127.0.0.1:" + socket.address().port;
  await new Promise(resolve => socket.close(resolve));
  const live = "http://" + listenAddress;
  server = spawn(binary, ["serve", parent, "--listen", listenAddress], { cwd: parent, env, stdio: ["ignore", "ignore", "inherit"] });
  for (let check = 0; check < 150; check++) {
    if (server.exitCode != null) throw new Error("The test portal exited");
    if (await fetch(live + "/readyz").then(r => r.ok).catch(() => false)) break;
    if (check === 149) throw new Error("The test portal did not become ready");
    await pause();
  }
  const forbidden = await fetch(live + "/v1/views/report");
  assert.equal(forbidden.status, 401);
  const response = await fetch(live + "/v1/views/report", { headers: { "x-seikika-user": "fixture" } });
  assert.equal(response.headers.get("cache-control"), "private, no-store");
  const served = await response.json();
  for (const key of ["portalName", "catalog", "entities", "scorecards", "scores", "tracks", "campaigns", "actions", "children"]) {
    assert.deepEqual(served[key], snapshot[key], key);
  }

  browser = await chromium.launch({ executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE });
  for (const [url, isLive] of [[offline + "/", false], [live + "/report/?as=fixture", true]]) {
    const page = await browser.newPage({ viewport: { width: 390, height: 844 } });
    const errors = [];
    const writes = [];
    page.on("pageerror", error => errors.push(error.message));
    page.on("request", request => { if (request.method() !== "GET") writes.push(request.url()); });
    await page.goto(url + "#catalog/Workflow");
    await page.getByRole("link", { name: "alpha workflow", exact: true }).waitFor();
    assert.equal(await page.locator(".federated-catalog-row").count(), 3);
    await page.getByRole("combobox", { name: "Group entities" }).selectOption("repository");
    assert.equal(await page.locator(".federated-catalog-group").count(), 3);
    await page.getByRole("combobox", { name: "Source repository" }).selectOption({ label: "Child-defined · alpha portal" });
    assert.equal(await page.locator(".federated-catalog-row").count(), 1);
    await page.locator(".federated-catalog-group summary").click();
    await page.getByRole("link", { name: "alpha workflow", exact: true }).click();
    await page.getByText("alpha catalog data", { exact: true }).waitFor();
    await page.getByRole("tab", { name: /Campaigns/ }).click();
    await page.getByRole("combobox", { name: "Campaign", exact: true }).waitFor();
    await page.evaluate(() => location.hash = "campaign/" + encodeURIComponent("@alpha/zizmor"));
    await page.getByRole("heading", { name: "Resolve zizmor findings · Child-defined · alpha portal", exact: true }).waitFor();
    await page.getByText("zizmor findings require review", { exact: false }).first().waitFor();
    const body = page.locator("#report-body");
    const alphaCampaign = await body.innerText();
    await body.getByRole("link", { name: "Browse Campaign in Catalog" }).click();
    const campaigns = body.locator(".federated-catalog-row");
    await campaigns.filter({ hasText: "Child-defined · alpha portal" }).getByRole("link", { name: "Resolve zizmor findings" }).click();
    await page.getByRole("heading", { name: "Resolve zizmor findings · Child-defined · alpha portal", exact: true }).waitFor();
    assert.equal(await body.innerText(), alphaCampaign);
    await body.getByRole("link", { name: "Browse Campaign in Catalog" }).click();
    await campaigns.filter({ hasText: "Child-defined · beta portal" }).getByRole("link", { name: "Resolve zizmor findings" }).click();
    await page.getByRole("heading", { name: "Resolve zizmor findings · Child-defined · beta portal", exact: true }).waitFor();
    assert.equal(await body.getByText("zizmor findings require review", { exact: false }).count(), 0);
    await page.getByRole("tab", { name: /Tracks/ }).click();
    await page.getByText("Workflow security · Child-defined · beta portal", { exact: true }).filter({ visible: true }).first().waitFor();
    await page.getByRole("tab", { name: /Actions/ }).click();
    const actionPicker = page.getByRole("combobox", { name: /^Action/ });
    await actionPicker.locator("option").filter({ hasText: "Child-defined · alpha portal" }).waitFor({ state: "attached" });
    if (!isLive) await actionPicker.selectOption({ label: "review-workflow · Child-defined · alpha portal" });
    await page.getByLabel("Reviewer notes", { exact: false }).fill("Do not run this fixture.");
    await page.getByRole("combobox", { name: /Review team/ }).selectOption("security");
    await page.getByRole("button", { name: "Preview action" }).click();
    await page.getByRole("heading", { name: "Dispatch preview" }).waitFor();
    assert.equal(await page.getByRole("button", { name: "Dispatch action" }).isEnabled(), isLive);
    assert.equal(writes.length, isLive ? 1 : 0);
    assert.deepEqual(errors, []);
    assert(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth));
    await page.close();
  }
  console.log("Federation report parity passed for catalog, scores, campaigns, tracks, actions and preview.");
} finally {
  await browser?.close();
  if (server?.pid && server.exitCode == null) {
    await new Promise(resolve => { server.once("exit", resolve); server.kill(); });
  }
  if (site) await new Promise(resolve => site.close(resolve));
  await rm(directory, { recursive: true, force: true });
}
