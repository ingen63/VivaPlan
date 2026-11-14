// Abbrechen gilt im Rechenkern für alle laufenden Berechnungen. Diese Tests laufen deshalb getrennt
// (Projekt «cancel» in playwright.config.js, erst nach allen anderen, nacheinander).
// Oberflächentests von VivaPlan im Browser. Jeder Test startet mit der Beispielplanung.
const { test, expect } = require("@playwright/test");
const fs = require("fs");
const os = require("os");
const path = require("path");

/** Öffnet die Oberfläche (optional mit Testzugang über die Adresse) und sammelt JavaScript-Fehler. */
async function open(page, hash = "") {
  const errors = [];
  page.on("pageerror", e => errors.push(e.message));
  await page.goto("/" + (hash ? "#" + hash : ""));
  await page.waitForSelector("html[data-ready='1']");
  await expect(page.locator("#chart path").first()).toBeAttached();
  return errors;
}
const noErrors = async (page, errors) => {
  expect(errors).toEqual([]);
  expect(await page.evaluate(() => window.__errors)).toEqual([]);
};

test.describe("Abbrechen laufender Berechnungen", () => {
  test("Abbrechen beendet eine laufende Optimierung", async ({ page }) => {
    test.setTimeout(120_000);
    const errors = await open(page, "optimize/bootstrap/optset");
    // Über die Planung setzen: saveOptCfg liest die Felder und würde ein direkt gesetztes optCfg überschreiben (ISS-16)
    const big = await page.evaluate(() => { state.optimization = optToPlan({ ...optCfg, paths: 20000, checkPaths: 20000, samples: 60 }); renderSection(); return readOptCfg(); });
    expect(big).toMatchObject({ paths: 20000, checkPaths: 20000, samples: 60 });
    await page.locator('button[data-action="opt-run"]').click();
    await expect(page.locator("#jobCancel")).toBeVisible();
    // FEA-65: abbrechen, sobald die Optimierung nachweislich rechnet (Fortschrittsbalken über 0), statt fest zu warten
    await expect.poll(() => page.evaluate(() => document.getElementById("jobBar")?.value ?? 0), { timeout: 60_000 }).toBeGreaterThan(0);
    await page.locator("#jobCancel").click();
    await expect(page.locator("html")).toHaveAttribute("data-job", "off", { timeout: 60_000 });
    await expect(page.locator("#toast")).toContainText("abgebrochen");
    await expect(page.locator("#main .job")).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("Simulation zeigt die Dauer am Knopf und lässt sich abbrechen", async ({ page }) => {
    test.setTimeout(120_000);
    const errors = await open(page, "scenarios/bootstrap");
    await expect(page.locator("#est-run")).toContainText("Dauer:", { timeout: 30_000 });
    await page.evaluate(() => { state.returns.bootstrap.paths = 1000000; });
    await page.evaluate(() => { runSimulation("overview"); });
    await expect(page.locator("#jobCancel")).toBeVisible();
    await page.locator("#jobCancel").click();
    await expect(page.locator("html")).toHaveAttribute("data-job", "off", { timeout: 60_000 });
    await expect(page.locator("#resultsBody .kpi")).toHaveCount(0);
    await noErrors(page, errors);
  });
});
