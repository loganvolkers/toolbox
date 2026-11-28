import assert from "node:assert/strict";
import { createRequire } from "node:module";
const require = createRequire(new URL("../../report-ui/package.json", import.meta.url));
const { chromium } = process.env.NODE_PATH
  ? createRequire(import.meta.url)("playwright")
  : require("playwright");
const [portal, grafana] = process.argv.slice(2);
const browser = await chromium.launch({
  ...(process.env.SEIKIKA_CHROMIUM ? { executablePath: process.env.SEIKIKA_CHROMIUM } : {}),
  args: ["--no-sandbox"],
});
try {
  const page = await browser.newPage({ viewport: { width: 430, height: 900 } });
  await page.goto(portal);
  await page.getByRole("searchbox", { name: "Search" }).fill("Seikika demo");
  await page.getByRole("searchbox", { name: "Search" }).press("Enter");
  await page.getByRole("link", { name: "service:seikika-demo", exact: true }).click();
  await page.getByRole("heading", { name: "Seikika demo", exact: true }).waitFor();
  await page.getByRole("heading", { name: "Recent deployments" }).waitFor();
  await page.getByText("Active at last check", { exact: true }).waitFor();
  await page.getByText("12.5 s", { exact: true }).waitFor();
  await page.getByText("No data", { exact: true }).waitFor();
  await page.getByText("0 records", { exact: true }).waitFor();
  await page.getByRole("link", { name: "seikika/seikika", exact: true }).waitFor();
  assert.equal(await page.getByRole("link", { name: "Open Grafana dashboard" }).getAttribute("href"),
    grafana + "/d/seikika-runtime");
  assert.equal(await page.getByRole("link", { name: "Runbook" }).getAttribute("href"),
    "https://seikika.com/operators/observability/");
  assert.equal(await page.evaluate(() => document.documentElement.scrollWidth), 430);
  await page.goto(portal + "/e/service%3Aseikika-demo");
  await page.getByRole("heading", { name: "Service identity" }).waitFor();
} finally {
  await browser.close();
}
