// A11 in a real browser: open an entity page, change the source (mock GitHub), click Refresh,
// and expect the page to re-render at a newer snapshot with the change.
// Usage: node scripts/ui-a11.cjs <toolbox url> <mock github url>
const { chromium } = require("playwright");
(async () => {
  const [url, gh] = process.argv.slice(2);
  const browser = await chromium.launch();
  const page = await browser.newPage();
  page.on("console", (m) => m.type() === "error" && console.error("  browser:", m.text()));
  await page.goto(`${url}/e/repository%3Apayments-api?as=alice`);
  await page.getByRole("heading", { name: "Payments API" }).waitFor();
  const before = await page.locator(".asof code").first().textContent();
  const ruleBefore = await page.locator("li", { hasText: "branch-protected" }).first().getAttribute("class");
  await fetch(`${gh}/_mock/repos/payments-api`, { method: "POST", body: JSON.stringify({ default_branch_protected: false }) });
  const started = Date.now();
  await page.getByRole("button", { name: "Refresh" }).click();
  try {
    await page.getByRole("status").filter({ hasText: "Refreshed at snapshot" }).waitFor({ timeout: 30000 });
  } catch (e) {
    console.error("  status text:", await page.getByRole("status").first().textContent());
    throw e;
  }
  await page.waitForFunction((b) => document.querySelector(".asof code")?.textContent !== b, before, { timeout: 10000 });
  const after = await page.locator(".asof code").first().textContent();
  const ruleAfter = await page.locator("li", { hasText: "branch-protected" }).first().getAttribute("class");
  await page.screenshot({ path: process.env.SCREENSHOT || "/tmp/a11.png", fullPage: true });
  console.log(`  snapshot ${before} → ${after} in ${Date.now() - started} ms; branch-protected ${ruleBefore} → ${ruleAfter}`);
  await browser.close();
  if (before === after || ruleAfter !== "rule-fail") process.exit(1);
})().catch((e) => { console.error(e); process.exit(1); });
