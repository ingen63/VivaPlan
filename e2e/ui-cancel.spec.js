// In the engine, cancel applies to all running computations. These tests therefore run separately
// (project «cancel» in playwright.config.js, after all others, one after another).
// User interface tests of VivaPlan in the browser. Every test starts with the example plan.
const { test, expect } = require("@playwright/test");
const fs = require("fs");
const os = require("os");
const path = require("path");

/**
 * Opens the user interface (optionally with test access via the address) and collects JavaScript errors.
 * #111: without measuring the computing time (#nobench), since it would otherwise run in the background of every test and
 * make it depend on the machine load. Tests that check the real measurement open with `{ bench: true }`.
 */
async function open(page, hash = "", { bench = false } = {}) {
  const errors = [];
  page.on("pageerror", e => errors.push(e.message));
  const h = bench ? hash : [hash, "nobench"].filter(Boolean).join("/");
  await page.goto("/" + (h ? "#" + h : ""));
  await page.waitForSelector("html[data-ready='1']");
  await expect(page.locator("#chart path").first()).toBeAttached();
  return errors;
}
const noErrors = async (page, errors) => {
  expect(errors).toEqual([]);
  expect(await page.evaluate(() => window.__errors)).toEqual([]);
};

test.describe("Cancel running computations", () => {
  test("cancel ends a running optimization", async ({ page }) => {
    test.setTimeout(120_000);
    const errors = await open(page, "optimize/bootstrap/optset");
    // Set via the plan: saveOptCfg reads the fields and would overwrite a directly set optCfg (#79)
    const big = await page.evaluate(() => { state.optimization = optToPlan({ ...optCfg, paths: 20000, checkPaths: 20000, samples: 60 }); renderSection(); return readOptCfg(); });
    expect(big).toMatchObject({ paths: 20000, checkPaths: 20000, samples: 60 });
    await page.locator('button[data-action="opt-run"]').click();
    await expect(page.locator("#jobCancel")).toBeVisible();
    // #83: cancel as soon as the optimization is demonstrably computing (progress bar above 0) instead of waiting a fixed time
    await expect.poll(() => page.evaluate(() => document.getElementById("jobBar")?.value ?? 0), { timeout: 60_000 }).toBeGreaterThan(0);
    await page.locator("#jobCancel").click();
    await expect(page.locator("html")).toHaveAttribute("data-job", "off", { timeout: 60_000 });
    await expect(page.locator("#toast")).toContainText("abgebrochen");
    await expect(page.locator("#main .job")).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("simulation shows the duration on the button and can be cancelled", async ({ page }) => {
    test.setTimeout(120_000);
    const errors = await open(page, "scenarios/bootstrap", { bench: true });
    await expect(page.locator("#est-run")).toContainText("Dauer:", { timeout: 30_000 });
    // #139: 100'000 paths over the example plan (472 months) stay below the memory limit of 50 million path months
    await page.evaluate(() => { state.returns.bootstrap.paths = 100000; });
    await page.evaluate(() => { runSimulation("overview", { noConfirm: true }); });
    await expect(page.locator("#jobCancel")).toBeVisible();
    await page.locator("#jobCancel").click();
    await expect(page.locator("html")).toHaveAttribute("data-job", "off", { timeout: 60_000 });
    await expect(page.locator("#resultsBody .kpi")).toHaveCount(0);
    await noErrors(page, errors);
  });
});
