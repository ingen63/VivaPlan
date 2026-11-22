// User interface tests of the tax by place of residence (#63). Every test starts with the example plan.
const { test, expect } = require("@playwright/test");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { open, noErrors, nav, previewIdle } = require("./helpers");

const cantonSel = page => page.locator("#tax-canton");
const muni = page => page.locator("#tax-muni");

/** Switches the section «Steuern» to the mode «Wohnort». */
async function toLocation(page) {
  await nav(page, "Steuern");
  await page.locator('button[data-action="tax-mode"][data-v="location"]').click();
  await expect(cantonSel(page)).toBeVisible();
}

/** Chooses canton and municipality like a user: canton from the list, municipality by typing the name. */
async function choose(page, canton, name) {
  await cantonSel(page).selectOption(canton);
  await expect(muni(page)).toBeEnabled();
  await muni(page).fill(name);
  await muni(page).press("Tab");
  await expect(muni(page)).toHaveValue(name);
}

test.describe("Tax by place of residence (#63)", () => {
  test("AK1: mode place of residence or own tables; only canton, municipality (search filtered by canton) and church tax", async ({ page }) => {
    const errors = await open(page);
    await nav(page, "Steuern");
    // Example plan: own tables (AK7)
    await expect(page.locator('button[data-action="tax-mode"][data-v="own"]')).toHaveClass(/on/);
    await expect(page.locator('input[data-path="taxes.income.0.0"]')).toBeVisible();
    await toLocation(page);
    expect(await page.evaluate(() => state.taxes.mode)).toBe("location");
    // Tables for income and wealth hidden (they stay in the plan); since #64 also the lump-sum table
    await expect(page.locator('input[data-path="taxes.income.0.0"]')).toHaveCount(0);
    await expect(page.locator('input[data-path="taxes.wealth.0.0"]')).toHaveCount(0);
    await expect(page.locator('input[data-path="taxes.capital.0.0"]')).toHaveCount(0);
    expect(await page.evaluate(() => state.taxes.income.length)).toBeGreaterThan(0);
    // Municipality only after the canton; the list holds the municipalities of the canton
    await expect(muni(page)).toBeDisabled();
    await cantonSel(page).selectOption("BE");
    const names = await page.locator("#tax-munis option").evaluateAll(o => o.map(x => x.value));
    expect(names).toContain("Bern");
    expect(names).not.toContain("Zürich");
    await choose(page, "ZH", "Winterthur");
    expect(await page.evaluate(() => state.taxes.location)).toEqual({ canton: "ZH", municipality: 230, church: "none" });
    // Search ignores upper and lower case; another canton resets the municipality
    await muni(page).fill("zürich");
    await muni(page).press("Tab");
    expect(await page.evaluate(() => state.taxes.location.municipality)).toBe(261);
    await cantonSel(page).selectOption("AG");
    expect(await page.evaluate(() => state.taxes.location.municipality)).toBeUndefined();
    await expect(muni(page)).toHaveValue("");
    // Church tax: four denominations
    const church = page.locator('select[data-path="taxes.location.church"]');
    expect(await church.locator("option").allTextContents()).toEqual(["keine", "reformiert", "römisch-katholisch", "christkatholisch"]);
    await church.selectOption("catholic");
    expect(await page.evaluate(() => state.taxes.location.church)).toBe("catholic");
    // Back to own tables: the tables are there again, the place of residence stays in the plan
    await page.locator('button[data-action="tax-mode"][data-v="own"]').click();
    await expect(page.locator('input[data-path="taxes.income.0.0"]')).toBeVisible();
    expect(await page.evaluate(() => [state.taxes.mode, state.taxes.location.canton])).toEqual([undefined, "AG"]);
    await noErrors(page, errors);
  });

  test("AK2: all 26 cantons and all municipalities of the tax data are selectable", async ({ page }) => {
    const errors = await open(page);
    await toLocation(page);
    await expect(cantonSel(page).locator("option")).toHaveCount(27); // with «– bitte wählen –»
    const counts = await page.evaluate(() => taxLoc.cantons.map(c => [c.id, c.municipalities.length]));
    expect(counts.reduce((a, [, n]) => a + n, 0)).toBeGreaterThan(2000);
    for (const [id, n] of [counts[0], counts[counts.length - 1]]) {
      await cantonSel(page).selectOption(id);
      await expect(page.locator("#tax-munis option")).toHaveCount(n);
    }
    await noErrors(page, errors);
  });

  test("AK8: missing canton or municipality is an error in the taxes section", async ({ page }) => {
    const errors = await open(page);
    await toLocation(page);
    await previewIdle(page);
    await expect(page.locator('[data-path="taxes.location.canton"] + .ferr')).toContainText("Bitte den Kanton wählen");
    await cantonSel(page).selectOption("ZH");
    await previewIdle(page);
    await expect(page.locator('[data-path="taxes.location.municipality"] + .ferr')).toContainText("Bitte die Gemeinde wählen");
    // A municipality of another canton (Bern in the canton of Zurich), e.g. from an edited file
    await page.evaluate(() => { state.taxes.location.municipality = 351; renderSection(); scheduleRefresh(); });
    await previewIdle(page);
    await expect(page.locator('[data-path="taxes.location.municipality"] + .ferr')).toContainText("liegt nicht im Kanton ZH");
    await choose(page, "ZH", "Zürich");
    await previewIdle(page);
    await expect(page.locator("#main .ferr")).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("AK9: multipliers of the municipality, tax year, source and the note on the estimate", async ({ page }) => {
    const errors = await open(page);
    await toLocation(page);
    await expect(page.locator("#tax-source")).toContainText("Steuerdaten 2026, Quelle: ESTV");
    await expect(page.locator("#taxEstimateNote")).toContainText("Die Steuer ist eine Schätzung mit den Standardabzügen des Kantons.");
    await expect(page.locator("#tax-multipliers")).toHaveCount(0);
    await choose(page, "ZH", "Zürich");
    const m = await page.evaluate(() => taxLoc.cantons.find(c => c.id === "ZH").municipalities.find(x => x[0] === 261)[2]);
    const pct = v => (v * 100).toLocaleString("de-CH", { maximumFractionDigits: 3 });
    await expect(page.locator("#tax-multipliers")).toContainText(`Steuerfüsse Einkommen: Kanton ${pct(m[0])} %, Gemeinde ${pct(m[1])} %`);
    await page.locator('select[data-path="taxes.location.church"]').selectOption("reformed");
    await expect(page.locator("#tax-multipliers")).toContainText(`Kirche ${pct(m[2])} %`);
    // Professional expenses with only a maximum in the ESTV data: marked
    await expect(page.locator("#tax-earned-approx")).toHaveCount(0);
    await cantonSel(page).selectOption("BL");
    await expect(page.locator("#tax-earned-approx")).toContainText("⚠");
    await noErrors(page, errors);
  });

  test("AK10: open a plan, simulate, the parameters tab names the municipality without opening the taxes section", async ({ page }) => {
    const errors = await open(page);
    const plan = await page.evaluate(() => {
      const s = JSON.parse(JSON.stringify(state));
      s.taxes.mode = "location"; s.taxes.location = { canton: "ZH", municipality: 261, church: "none" };
      return s;
    });
    const file = path.join(os.tmpdir(), `steuern-wohnort-${process.pid}.vplan`);
    fs.writeFileSync(file, JSON.stringify(plan));
    await page.locator("#fileInput").setInputFiles(file);
    await expect.poll(() => page.evaluate(() => state.taxes.mode)).toBe("location");
    await previewIdle(page);
    await page.locator('button[data-action="run"]').click();
    await expect(page.locator("#results")).toBeVisible();
    await page.locator("button[data-action='rtab'][data-tab='params']").click();
    const body = page.locator("#resultsBody");
    await expect(body).toContainText("Zürich (ZH), Kirchensteuer keine");
    await expect(body).toContainText("Wohnort");
    // Unknown municipality number: with the canton, no error
    expect(await page.evaluate(() => locationText({ canton: "ZH", municipality: 99999, church: "none" }))).toBe("99999 (ZH), Kirchensteuer keine");
    // The section «Steuern» was never opened
    expect(await page.evaluate(() => current)).not.toBe("taxes");
    fs.rmSync(file);
    await noErrors(page, errors);
  });

  test("AK7: the own tables keep their summary in the parameters tab", async ({ page }) => {
    const errors = await open(page);
    const rows = await page.evaluate(() => Object.fromEntries(paramSections(state).flatMap(([, r]) => r)));
    expect(rows["Einkommens- und Vermögenssteuer"]).toBe("Eigene Tabellen");
    expect(rows["Einkommenssteuer"]).toBeTruthy();
    await noErrors(page, errors);
  });
});

test.describe("Lump-sum tax by place of residence (#64)", () => {
  test("AK1: no input for the lump-sum tax in the mode place of residence; the rule of the canton is shown below the municipality", async ({ page }) => {
    const errors = await open(page);
    await nav(page, "Steuern");
    await expect(page.locator('input[data-path="taxes.capital.0.0"]')).toBeVisible();
    // married couple: in the mode own tables the own tables for married couples are available
    expect(await page.evaluate(() => { ACTIONS["partner-add"](); state.household.married = true; renderSection(); return !!state.partner; })).toBe(true);
    await expect(page.locator('input[data-action="couple-tax"]')).toBeVisible();
    await toLocation(page);
    await expect(page.locator('input[data-path="taxes.capital.0.0"]')).toHaveCount(0);
    await expect(page.locator('[data-path^="taxes.couple"]')).toHaveCount(0);
    await expect(page.locator('input[data-action="couple-tax"]')).toHaveCount(0);
    expect(await page.evaluate(() => state.taxes.capital.length)).toBeGreaterThan(0);
    await expect(page.locator("#tax-capital-rule")).toHaveCount(0);
    await choose(page, "ZH", "Zürich");
    const rule = page.locator("#tax-capital-rule");
    await expect(rule).toContainText("Kapitalbezugssteuer im Kanton: Satz des Einkommenstarifs für einen Zwanzigstel des Bezugs, mindestens 2 %.");
    await expect(rule).toContainText("ein Fünftel des Tarifs, ohne Abzüge");
    // the rule stands below the municipality field
    const muniBox = await muni(page).boundingBox(), ruleBox = await rule.boundingBox();
    expect(ruleBox.y).toBeGreaterThan(muniBox.y);
    await expect(page.locator("#tax-location p.lumpcheck")).toHaveText(/^Kapitalbezugssteuer: Vor dem Entscheid/);
    await expect(page.locator("#main")).toContainText("Steuerfuss für die Grundstückgewinnsteuer");
    // the parameters tab names the rule instead of the table
    const rows = await page.evaluate(() => Object.fromEntries(paramSections(state).flatMap(([, r]) => r)));
    expect(rows["Kapitalbezugssteuer"]).toBe("nach Wohnort (Satz des Einkommenstarifs für einen Zwanzigstel des Bezugs, mindestens 2 %)");
    // back to own tables: the table is there again
    await page.locator('button[data-action="tax-mode"][data-v="own"]').click();
    await expect(page.locator('input[data-path="taxes.capital.0.0"]')).toBeVisible();
    await noErrors(page, errors);
  });

  test("AK3: every canton has a rule in all languages; approximated rules are marked with ⚠", async ({ page }) => {
    const errors = await open(page);
    await toLocation(page);
    const ids = await page.evaluate(() => taxLoc.cantons.map(c => [c.id, c.capitalRule, !!c.capitalApprox]));
    expect(ids).toHaveLength(26);
    expect(ids.filter(([, , a]) => a).map(([id]) => id).sort()).toEqual(["VS", "ZG"]);
    const texts = Object.fromEntries(["de", "en", "fr", "it"].map(l =>
      [l, JSON.parse(fs.readFileSync(path.join(__dirname, "..", "crates", "app", "ui", "i18n", `${l}.json`), "utf8"))]));
    for (const [id, rule, approx] of ids) {
      // the German text is the rule of the tax data file
      expect(texts.de[`capitalRule.${id}`]).toBe(rule);
      for (const l of ["en", "fr", "it"]) {
        expect(texts[l][`capitalRule.${id}`], `${l} ${id}`).toBeTruthy();
        if (approx) expect(texts[l][`capitalApprox.${id}`], `${l} ${id}`).toBeTruthy();
      }
    }
    await cantonSel(page).selectOption("ZG");
    await expect(page.locator("#tax-capital-approx")).toContainText("⚠ Kapitalbezugssteuer angenähert: Die Regel ist angenähert");
    await cantonSel(page).selectOption("ZH");
    await expect(page.locator("#tax-capital-approx")).toHaveCount(0);
    await expect(page.locator("#tax-capital-rule")).toBeVisible();
    await noErrors(page, errors);
  });
});
