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
    // #65: the multiplier for the property gains tax only with own rules
    await expect(page.locator("#main")).not.toContainText("Steuerfuss für die Grundstückgewinnsteuer");
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

test.describe("Property gains tax by canton (#65)", () => {
  test("AK1, AK3, AK5: canton per property, rule with approximation note, own rules stay possible", async ({ page }) => {
    const errors = await open(page);
    await toLocation(page);
    await choose(page, "ZH", "Zürich");
    // AK1/AK6: no duty for own rules, the rules of the canton apply; no rule editor
    await expect(page.locator("#gain-location")).toContainText("Für jede Immobilie gilt die Regel des Kantons");
    await expect(page.locator("#gain-own")).not.toBeChecked();
    await expect(page.locator('[data-action="gain-zh"]')).toHaveCount(0);
    await expect(page.locator("#main .errcard")).toHaveCount(0);
    await expect(page.locator('input[data-path="taxes.multiplier"]')).toHaveCount(0);
    const idx = await page.evaluate(() => state.realestate.properties.findIndex(p => p.status !== "rent"));
    expect(idx).toBeGreaterThanOrEqual(0);

    // section «Immobilien»: canton of the property, default canton of residence, rule visible
    await nav(page, "Immobilien");
    const sel = page.locator(`select[data-path="realestate.properties.${idx}.canton"]`);
    await expect(sel).toBeVisible();
    await expect(sel.locator("option").first()).toHaveText("Wohnkanton (Zürich)");
    expect(await sel.locator("option").count()).toBe(27);
    const rule = page.locator("[data-gain-rule]").first();
    await expect(rule).toHaveText("Grundstückgewinnsteuer Zürich: Stufentarif 10 % bis 40 %, Zuschlag bis 50 % unter 2 Jahren, Ermässigung bis 50 % ab 20 Jahren.");
    await expect(page.locator("[data-gain-approx]")).toHaveCount(0);
    // AK3: approximated rule (Valais) with ⚠
    await sel.selectOption("VS");
    expect(await page.evaluate(i => state.realestate.properties[i].canton, idx)).toBe("VS");
    await expect(rule).toContainText("Grundstückgewinnsteuer Wallis: Stufentarif 12 % bis 24 %");
    await expect(page.locator("[data-gain-approx]").first()).toHaveText("⚠ Näherung bei der Grundstückgewinnsteuer: Angenommen ist ein Stufentarif nach Gewinnhöhe (nicht ein Satz für den ganzen Gewinn).");
    await sel.selectOption("AG");
    await expect(page.locator("[data-gain-approx]")).toHaveCount(0);
    await sel.selectOption("");
    await expect(rule).toContainText("Grundstückgewinnsteuer Zürich");
    // the parameters tab names the rule of the canton and the canton of the property
    let rows = await page.evaluate(() => Object.fromEntries(paramSections(state, true).flatMap(([, r]) => r)));
    expect(rows["Grundstückgewinnsteuer"]).toBe("Regel des Kantons der Immobilie");

    // AK5: own rules with the check box; then the editor and the multiplier, no canton at the property
    await nav(page, "Steuern");
    await page.locator("#gain-own").check();
    expect(await page.evaluate(() => state.taxes.propertyGainOwn)).toBe(true);
    await expect(page.locator("#gain-location")).toContainText("Es gelten die unten erfassten Regeln für alle Immobilien.");
    await expect(page.locator('input[data-path="taxes.multiplier"]')).toBeVisible();
    await expect(page.locator("#main")).toContainText("Steuerfuss für die Grundstückgewinnsteuer");
    await nav(page, "Immobilien");
    await expect(sel).toHaveCount(0);
    await nav(page, "Steuern");
    await page.locator("#gain-own").uncheck();
    expect(await page.evaluate(() => state.taxes.propertyGainOwn)).toBeUndefined();
    // AK5: in the mode «Eigene Tabellen» always the own rules, no check box
    await page.locator('button[data-action="tax-mode"][data-v="own"]').click();
    await expect(page.locator("#gain-own")).toHaveCount(0);
    await nav(page, "Immobilien");
    await expect(sel).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("AK3: every canton has a property gains rule in all languages; approximations are marked", async ({ page }) => {
    const errors = await open(page);
    await toLocation(page);
    const ids = await page.evaluate(() => taxLoc.cantons.map(c => [c.id, c.propertyGainRule, c.propertyGainApprox ?? null]));
    expect(ids).toHaveLength(26);
    expect(ids.filter(([, , a]) => a).map(([id]) => id).sort()).toEqual(["BL", "BS", "SG", "VD", "VS"]);
    const texts = Object.fromEntries(["de", "en", "fr", "it"].map(l =>
      [l, JSON.parse(fs.readFileSync(path.join(__dirname, "..", "crates", "app", "ui", "i18n", `${l}.json`), "utf8"))]));
    for (const [id, rule, approx] of ids) {
      // the German texts are those of the tax data file
      expect(texts.de[`propertyGainRule.${id}`]).toBe(rule);
      if (approx) expect(texts.de[`propertyGainApprox.${id}`]).toBe(approx);
      for (const l of ["en", "fr", "it"]) {
        expect(texts[l][`propertyGainRule.${id}`], `${l} ${id}`).toBeTruthy();
        if (approx) expect(texts[l][`propertyGainApprox.${id}`], `${l} ${id}`).toBeTruthy();
      }
    }
    await noErrors(page, errors);
  });
});

test.describe("Deductions from income with the proposal of the canton (#131)", () => {
  const card = page => page.locator("#tax-deductions");
  /** Plan with the stored rows `rows` (and optionally in the mode «Wohnort», Zurich). */
  async function withRows(page, rows, loc) {
    await page.evaluate(([rows, loc]) => {
      state.taxes.deductions = rows;
      if (loc) { state.taxes.mode = "location"; state.taxes.location = { canton: loc[0], municipality: loc[1], church: "none" }; }
      markDirty(); renderSection(); scheduleRefresh();
    }, [rows, loc]);
    await nav(page, "Steuern");
    await previewIdle(page);
  }

  test("AK1, AK2: pillar 3a deduction with the maximum per person, can be switched off", async ({ page }) => {
    const errors = await open(page);
    await nav(page, "Steuern");
    const chk = card(page).locator('input[data-path="taxes.deductPillar3a"]');
    await expect(chk).not.toBeChecked();       // existing plan: no new deduction
    await expect(card(page)).toContainText("höchstens 7'258 Franken je Person");
    await chk.check();
    expect(await page.evaluate(() => state.taxes.deductPillar3a)).toBe(true);
    await chk.uncheck();
    expect(await page.evaluate(() => state.taxes.deductPillar3a)).toBe(false);
    await noErrors(page, errors);
  });

  test("AK3: new plans have the rows until and from retirement; more rows possible", async ({ page }) => {
    const errors = await open(page);
    const p = await page.evaluate(async () => blankPlan(await Engine.call("example_plan")));
    expect(p.taxes.deductions).toEqual([{ from: "start", kind: "own", amount: 0, rate: 0 }, { from: "retire", kind: "own", amount: 0, rate: 0 }]);
    expect(p.taxes.deductPillar3a).toBe(true);
    await withRows(page, p.taxes.deductions);
    await expect(card(page).locator("[data-ded-row]")).toHaveCount(2);
    await expect(card(page).locator('select[data-path="taxes.deductions.1.from"]')).toHaveValue("retire");
    await card(page).locator('button[data-action="ded-add"]').click();
    await expect(card(page).locator("[data-ded-row]")).toHaveCount(3);
    await noErrors(page, errors);
  });

  test("AK4, AK5: in the mode «Wohnort» the proposal of the canton is the default; amount of the first year with parts, ⚠ for BL", async ({ page }) => {
    const errors = await open(page);
    await withRows(page, [{ from: "start", kind: "own", amount: 0, rate: 0 }, { from: "retire", kind: "own", amount: 0, rate: 0 }]);
    await toLocation(page);
    expect(await page.evaluate(() => state.taxes.deductions.map(r => r.kind))).toEqual(["canton", "canton"]);
    await choose(page, "ZH", "Zürich");
    await previewIdle(page);
    const kind = card(page).locator('select[data-path="taxes.deductions.0.kind"]');
    expect(await kind.locator("option").allTextContents()).toEqual(["Vorschlag des Kantons", "eigener Wert"]);
    const proposal = card(page).locator('[data-ded-proposal="0"]');
    await expect(proposal).toContainText(/^\d{4}: Kanton [\d']+ \(.*Versicherungsprämien [\d']+.*\), Bund [\d']+ \(/);
    await expect(card(page).locator("[data-ded-approx]")).toHaveCount(0);
    await choose(page, "BL", "Liestal");
    await previewIdle(page);
    await expect(card(page).locator("[data-ded-approx]").first()).toContainText("⚠");
    await noErrors(page, errors);
  });

  test("AK6, AK7: own value per year and share of the income; the input help computes it from the assessment", async ({ page }) => {
    const errors = await open(page);
    await withRows(page, [{ from: "start", kind: "own", amount: 0, rate: 0 }]);
    await expect(card(page).locator('input[data-path="taxes.deductions.0.amount"]')).toBeVisible();
    await expect(card(page).locator('input[data-path="taxes.deductions.0.rate"]')).toBeVisible();
    await page.locator("#ded-calc-income").fill("120000");
    await page.locator("#ded-calc-taxable").fill("80000");
    await expect(page.locator("#ded-calc-result")).toHaveText("40'000");
    await page.locator('button[data-action="ded-calc-apply"]').click();
    expect(await page.evaluate(() => state.taxes.deductions[0])).toEqual({ from: "start", kind: "own", amount: 40000, rate: 0 });
    await expect(card(page).locator('input[data-path="taxes.deductions.0.amount"]')).toHaveValue("40'000");
    await noErrors(page, errors);
  });

  test("AK8, AK9: an own value stays when the canton changes; the explanation of the own value", async ({ page }) => {
    const errors = await open(page);
    await withRows(page, [{ from: "start", kind: "own", amount: 15000, rate: 0 }], ["ZH", 261]);
    await cantonSel(page).selectOption("BE");
    expect(await page.evaluate(() => state.taxes.deductions)).toEqual([{ from: "start", kind: "own", amount: 15000, rate: 0 }]);
    const hint = page.locator("#ded-own-hint");
    await expect(hint).toContainText("netto (nach AHV- und PK-Beiträgen");
    await expect(hint).toContainText("Die Säule 3a gehört nicht dazu");
    await expect(hint).toContainText("ab 2029 ohne Eigenmietwert und ohne Schuldzinsabzug");
    await expect(card(page)).toContainText("Hypothekarzinsen werden nicht automatisch abgezogen");
    await noErrors(page, errors);
  });

  test("AK11, AK13: own tables without a proposal; existing plans keep their deductions until adjusted", async ({ page }) => {
    const errors = await open(page);
    await nav(page, "Steuern");
    // Example plan (own tables, without rows): no deductions, nothing stored
    await expect(card(page)).toContainText("Keine Abzüge.");
    await expect(card(page).locator('select[data-path$=".kind"]')).toHaveCount(0);
    expect(await page.evaluate(() => state.taxes.deductions)).toBeUndefined();
    // Mode «Wohnort» without rows: the proposal of the canton throughout, still not stored
    await toLocation(page);
    await expect(card(page).locator("[data-ded-row]")).toHaveCount(1);
    await expect(card(page)).toContainText("Vorschlag des Kantons");
    expect(await page.evaluate(() => state.taxes.deductions)).toBeUndefined();
    await card(page).locator('button[data-action="ded-edit"]').click();
    expect(await page.evaluate(() => state.taxes.deductions)).toEqual([{ from: "start", kind: "canton" }]);
    // Back to own tables: the proposal becomes an own value of 0
    await page.locator('button[data-action="tax-mode"][data-v="own"]').click();
    expect(await page.evaluate(() => state.taxes.deductions)).toEqual([{ from: "start", kind: "own", amount: 0, rate: 0 }]);
    await noErrors(page, errors);
  });

  test("AK14: a negative amount or a share above 100 % is an error in the taxes section", async ({ page }) => {
    const errors = await open(page);
    await withRows(page, [{ from: "start", kind: "own", amount: -5, rate: 1.5 }]);
    await expect(card(page).locator(".ferr").first()).toContainText("Zeile 1: Der Betrag muss 0 oder grösser sein");
    await expect(card(page).locator(".ferr").nth(1)).toContainText("Anteil am Einkommen muss zwischen 0 % und 100 %");
    expect(await page.evaluate(() => lastIssues.filter(i => i.severity === "error").map(i => [i.section, i.field]))).toEqual([
      ["taxes", "taxes.deductions.0.amount"], ["taxes", "taxes.deductions.0.rate"]]);
    await noErrors(page, errors);
  });

  test("AK15: the tax breakdown shows income, deductions with origin and taxable income", async ({ page }) => {
    const errors = await open(page);
    await withRows(page, [{ from: "start", kind: "own", amount: 10000, rate: 0 }]);
    await page.evaluate(async () => {
      state.returns.source = "fixed"; state.scenarios.sweeps = [];
      lastRun = await Engine.call("run_simulation", { plan: state, path: null });
      resetVariation(); resultTab = "table"; showTaxCols = true; showResults(true); renderResults(lastRun);
    });
    const heads = await page.locator("#resultsBody table.yt th.tx").allTextContents();
    expect(heads.slice(-3)).toEqual(["Einkommen", "Abzüge", "Steuerbar"]);
    const origin = page.locator("#resultsBody table.yt td[data-ded-origin]").first();
    await expect(origin).toHaveAttribute("title", /eigener Wert: 10'000/);
    await noErrors(page, errors);
  });
});

test.describe("Layout of the rows of the deductions from income (#228)", () => {
  const card = page => page.locator("#tax-deductions");
  /** Width of the value of row `i` and whether the button ✕ is on the same height as the value. */
  const measure = (page, i) => card(page).locator(`[data-ded-row="${i}"]`).evaluate(row => {
    const val = row.querySelector(".dedval").getBoundingClientRect(), del = row.querySelector('button[data-action="row-del"]');
    const d = del?.getBoundingClientRect();
    return { width: val.width, sameLine: d ? d.top < val.bottom && d.bottom > val.top : null,
      rightmost: d ? d.left >= val.right : null, clipped: [...row.querySelectorAll(".dedval .computed")].some(e => e.scrollWidth > e.clientWidth) };
  });

  test("AK1–AK3, AK5: proposal of the canton and own value at least 200 px wide, ✕ on the same height", async ({ page }) => {
    const errors = await open(page);
    // Not adjusted yet: proposal of the canton without a button ✕
    await toLocation(page);
    await choose(page, "ZH", "Zürich");
    await previewIdle(page);
    await expect(card(page).locator('[data-ded-proposal="0"]')).toContainText("Kanton");
    let m = await measure(page, 0);
    expect(m.width).toBeGreaterThanOrEqual(200);
    expect(m.clipped).toBe(false);
    // Adjusted: one row each with the proposal and with an own value, both with ✕
    await page.evaluate(() => {
      state.taxes.deductions = [{ from: "start", kind: "canton" }, { from: "retire", kind: "own", amount: 12000, rate: 0.05 }];
      markDirty(); renderSection(); scheduleRefresh();
    });
    await previewIdle(page);
    await expect(card(page).locator('[data-ded-proposal="0"]')).toContainText("Kanton");
    for (const i of [0, 1]) {
      m = await measure(page, i);
      expect(m.width, `row ${i}`).toBeGreaterThanOrEqual(200);
      expect(m.sameLine, `row ${i}`).toBe(true);
      expect(m.rightmost, `row ${i}`).toBe(true);
      expect(m.clipped, `row ${i}`).toBe(false);
    }
    // AK2: amount and share side by side, both readable
    const amount = await card(page).locator('[data-ded-row="1"] input[data-path="taxes.deductions.1.amount"]').boundingBox();
    const rate = await card(page).locator('[data-ded-row="1"] input[data-path="taxes.deductions.1.rate"]').boundingBox();
    expect(Math.abs(amount.y - rate.y)).toBeLessThan(2);
    expect(rate.x).toBeGreaterThan(amount.x + amount.width - 1);
    expect(Math.min(amount.width, rate.width)).toBeGreaterThanOrEqual(60);
    await noErrors(page, errors);
  });

  test("AK4: other time series with .row keep their grid", async ({ page }) => {
    const errors = await open(page);
    await nav(page, "Steuern");
    await toLocation(page);
    const cols = await page.locator("#tax-location .row").first().evaluate(e => getComputedStyle(e).gridTemplateColumns.split(" ").length);
    expect(cols).toBe(4);
    expect(await page.evaluate(() => [...document.querySelectorAll(".row")].every(e => !e.matches("[data-ded-row]")))).toBe(true);
    await noErrors(page, errors);
  });
});

test.describe("Additional and one-off deductions from income (#230)", () => {
  const card = page => page.locator("#tax-deductions");
  async function withRows(page, rows, oneOff, loc = ["ZH", 261]) {
    await page.evaluate(([rows, oneOff, loc]) => {
      state.taxes.deductions = rows;
      if (oneOff) state.taxes.oneOffDeductions = oneOff; else delete state.taxes.oneOffDeductions;
      if (loc) { state.taxes.mode = "location"; state.taxes.location = { canton: loc[0], municipality: loc[1], church: "none" }; }
      markDirty(); renderSection(); scheduleRefresh();
    }, [rows, oneOff, loc]);
    await nav(page, "Steuern");
    await previewIdle(page);
  }
  const startYear = page => page.evaluate(() => Math.floor(startM() / 12));

  test("AK1: a row with the proposal of the canton has the field for the additional amount per year, an own value not", async ({ page }) => {
    const errors = await open(page);
    await withRows(page, [{ from: "start", kind: "canton" }, { from: "retire", kind: "own", amount: 0, rate: 0 }]);
    const extra = card(page).locator('[data-ded-row="0"] input[data-path="taxes.deductions.0.extra"]');
    await expect(extra).toBeVisible();
    await expect(card(page).locator('[data-ded-row="0"]')).toContainText("+ zusätzlich");
    // the label is shown in full (not cut to the width of «ab»)
    expect(await card(page).locator('[data-ded-row="0"] .dedextra').evaluate(e => e.scrollWidth <= e.clientWidth && e.getBoundingClientRect().width > 40)).toBe(true);
    await expect(card(page).locator('input[data-path="taxes.deductions.1.extra"]')).toHaveCount(0);
    await extra.fill("3000");
    await extra.press("Tab");
    expect(await page.evaluate(() => state.taxes.deductions[0].extra)).toBe(3000);
    // #228: the value with the extra field still at least 200 px wide
    expect(await card(page).locator('[data-ded-row="0"] .dedval').evaluate(e => e.getBoundingClientRect().width)).toBeGreaterThanOrEqual(200);
    await noErrors(page, errors);
  });

  test("AK2: the list of one-off deductions with tax year, description and amount; ✕ removes", async ({ page }) => {
    const errors = await open(page);
    await withRows(page, [{ from: "start", kind: "own", amount: 0, rate: 0 }], null);
    const y = await startYear(page);
    await expect(card(page).locator("#ded-oneoff")).toContainText("Einmalige Abzüge");
    await card(page).locator('button[data-action="ded-oneoff-add"]').click();
    expect(await page.evaluate(() => state.taxes.oneOffDeductions)).toEqual([{ year: y + 1, label: "Einmaliger Abzug", amount: 0 }]);
    const row = card(page).locator('[data-oneoff-row="0"]');
    await expect(row.locator('input[data-path="taxes.oneOffDeductions.0.year"]')).toHaveValue(String(y + 1));
    await row.locator('input[data-path="taxes.oneOffDeductions.0.label"]').fill("PK-Einkauf");
    await row.locator('input[data-path="taxes.oneOffDeductions.0.label"]').press("Tab");
    await row.locator('input[data-path="taxes.oneOffDeductions.0.amount"]').fill("50000");
    await row.locator('input[data-path="taxes.oneOffDeductions.0.amount"]').press("Tab");
    expect(await page.evaluate(() => state.taxes.oneOffDeductions[0])).toEqual({ year: y + 1, label: "PK-Einkauf", amount: 50000 });
    await card(page).locator('[data-oneoff-row="0"] button[data-action="list-del"]').click();
    expect(await page.evaluate(() => state.taxes.oneOffDeductions)).toEqual([]);
    await noErrors(page, errors);
  });

  test("AK4: the tax breakdown shows the additional and the one-off deduction as origins of their own", async ({ page }) => {
    const errors = await open(page);
    const y = await startYear(page);
    await withRows(page, [{ from: "start", kind: "canton", extra: 3000 }], [{ year: y, label: "Sanierung", amount: 8000 }]);
    await page.evaluate(async () => {
      state.returns.source = "fixed"; state.scenarios.sweeps = [];
      lastRun = await Engine.call("run_simulation", { plan: state, path: null });
      resetVariation(); resultTab = "table"; showTaxCols = true; showResults(true); renderResults(lastRun);
    });
    const origins = page.locator("#resultsBody table.yt td[data-ded-origin]");
    await expect(origins.first()).toHaveAttribute("title", /zusätzlich: 3'000/);
    await expect(origins.first()).toHaveAttribute("title", /einmalig: 8'000/);
    await expect(origins.nth(1)).not.toHaveAttribute("title", /einmalig/);
    await noErrors(page, errors);
  });

  test("AK6: a negative amount and a tax year outside the simulation are errors at the field", async ({ page }) => {
    const errors = await open(page);
    const y = await startYear(page);
    await withRows(page, [{ from: "start", kind: "canton", extra: -1 }], [{ year: y - 1, label: "", amount: -5 }]);
    expect(await page.evaluate(() => lastIssues.filter(i => i.severity === "error").map(i => [i.section, i.field]))).toEqual([
      ["taxes", "taxes.deductions.0.extra"], ["taxes", "taxes.oneOffDeductions.0.amount"], ["taxes", "taxes.oneOffDeductions.0.year"]]);
    await expect(card(page).locator(".ferr")).toHaveCount(3);
    await expect(card(page)).toContainText(`Das Steuerjahr ${y - 1} liegt ausserhalb der Simulation`);
    await noErrors(page, errors);
  });
});

test.describe("Deductions from wealth (#231)", () => {
  const card = page => page.locator("#tax-wealth-deductions");
  async function withRows(page, rows, oneOff, loc = ["ZH", 261]) {
    await page.evaluate(([rows, oneOff, loc]) => {
      if (rows) state.taxes.wealthDeductions = rows; else delete state.taxes.wealthDeductions;
      if (oneOff) state.taxes.oneOffWealthDeductions = oneOff; else delete state.taxes.oneOffWealthDeductions;
      if (loc) { state.taxes.mode = "location"; state.taxes.location = { canton: loc[0], municipality: loc[1], church: "none" }; }
      else state.taxes.mode = "own";
      markDirty(); renderSection(); scheduleRefresh();
    }, [rows, oneOff, loc]);
    await nav(page, "Steuern");
    await previewIdle(page);
  }
  const startYear = page => page.evaluate(() => Math.floor(startM() / 12));

  test("AK1: card with time-dependent rows "additionally … per year" and the list of one-off deductions", async ({ page }) => {
    const errors = await open(page);
    await withRows(page, null, null);
    const y = await startYear(page);
    await expect(card(page).locator("h3")).toHaveText("Abzüge vom Vermögen");
    await card(page).locator('button[data-action="wded-add"]').click();
    expect(await page.evaluate(() => state.taxes.wealthDeductions)).toEqual([{ from: "start", amount: 0 }]);
    const row = card(page).locator('[data-wded-row="0"]');
    await expect(row).toHaveClass(/dedrow/);
    await expect(row).toContainText("+ zusätzlich");
    await expect(row).toContainText("pro Jahr");
    const amount = row.locator('input[data-path="taxes.wealthDeductions.0.amount"]');
    await amount.fill("50000");
    await amount.press("Tab");
    expect(await page.evaluate(() => state.taxes.wealthDeductions[0].amount)).toBe(50000);
    // #228: the value at least 200 px wide, as with the deductions from income
    expect(await card(page).locator('[data-wded-row="0"] .dedval').evaluate(e => e.getBoundingClientRect().width)).toBeGreaterThanOrEqual(200);
    await card(page).locator('button[data-action="wded-add"]').click();
    expect(await page.evaluate(() => state.taxes.wealthDeductions[1].from === i2ym(startM() + 12))).toBe(true);
    await card(page).locator('button[data-action="ded-oneoff-add"]').click();
    expect(await page.evaluate(() => state.taxes.oneOffWealthDeductions)).toEqual([{ year: y + 1, label: "Einmaliger Abzug", amount: 0 }]);
    expect(await page.evaluate(() => state.taxes.oneOffDeductions)).toBeUndefined();
    await expect(card(page).locator("#wded-oneoff")).toContainText("Einmalige Abzüge");
    await expect(card(page).locator('[data-oneoff-row="0"] input[data-path="taxes.oneOffWealthDeductions.0.year"]')).toHaveValue(String(y + 1));
    await card(page).locator('[data-oneoff-row="0"] button[data-action="list-del"]').click();
    expect(await page.evaluate(() => state.taxes.oneOffWealthDeductions)).toEqual([]);
    await card(page).locator('[data-wded-row="1"] button[data-action="row-del"]').click();
    expect(await page.evaluate(() => state.taxes.wealthDeductions.length)).toBe(1);
    await noErrors(page, errors);
  });

  test("AK2, AK3: the hint says in addition to the canton (Wohnort) or the whole deduction (own tables)", async ({ page }) => {
    const errors = await open(page);
    await withRows(page, [{ from: "start", amount: 1000 }], null);
    await expect(card(page).locator("#wded-hint")).toContainText("kommen zu den Sozialabzügen des Kantons dazu");
    await withRows(page, [{ from: "start", amount: 1000 }], null, null);
    await expect(card(page).locator("#wded-hint")).toContainText("der ganze Abzug vom Vermögen");
    await noErrors(page, errors);
  });

  test("AK5: the tax breakdown shows the deductions from wealth with their origin and the taxable wealth", async ({ page }) => {
    const errors = await open(page);
    const y = await startYear(page);
    await withRows(page, [{ from: "start", amount: 50000 }], [{ year: y, label: "Schuld", amount: 100000 }]);
    await page.evaluate(async () => {
      state.returns.source = "fixed"; state.scenarios.sweeps = [];
      lastRun = await Engine.call("run_simulation", { plan: state, path: null });
      resetVariation(); resultTab = "table"; showTaxCols = true; showResults(true); renderResults(lastRun);
    });
    const origins = page.locator("#resultsBody table.yt td[data-wded-origin]");
    await expect(origins.first()).toHaveAttribute("title", /zusätzlich: 50'000/);
    await expect(origins.first()).toHaveAttribute("title", /einmalig: 100'000/);
    await expect(origins.first()).toHaveText("150'000");
    await expect(origins.nth(1)).not.toHaveAttribute("title", /einmalig/);
    await expect(page.locator("#resultsBody table.yt td[data-taxable-wealth]").first()).toHaveAttribute("title", /Vermögen vor Abzügen/);
    await expect(page.locator("#resultsBody table.yt th", { hasText: "Steuerbares Vermögen" })).toHaveCount(1);
    await noErrors(page, errors);
  });

  test("AK6: a negative amount and a tax year outside the simulation are errors at the field", async ({ page }) => {
    const errors = await open(page);
    const y = await startYear(page);
    await withRows(page, [{ from: "start", amount: -1 }], [{ year: y - 1, label: "", amount: -5 }]);
    expect(await page.evaluate(() => lastIssues.filter(i => i.severity === "error").map(i => [i.section, i.field]))).toEqual([
      ["taxes", "taxes.wealthDeductions.0.amount"], ["taxes", "taxes.oneOffWealthDeductions.0.amount"], ["taxes", "taxes.oneOffWealthDeductions.0.year"]]);
    await expect(card(page).locator(".ferr")).toHaveCount(3);
    await expect(card(page)).toContainText(`Das Steuerjahr ${y - 1} liegt ausserhalb der Simulation`);
    await noErrors(page, errors);
  });
});

test.describe("Time-dependent place of residence (#151)", () => {
  /** Mode «Wohnort» with Zurich from the start. */
  async function zurich(page) {
    await toLocation(page);
    await choose(page, "ZH", "Zürich");
  }

  test("AK1: further rows from a year or an event, each with canton, municipality and church tax", async ({ page }) => {
    const errors = await open(page);
    await zurich(page);
    // First row from the start, without ✕
    await expect(page.locator('[data-loc-row="0"]')).toContainText("Simulationsstart");
    await expect(page.locator('[data-loc-row="0"] [data-action="loc-del"]')).toHaveCount(0);
    expect(await page.evaluate(() => state.taxes.relocations)).toBeUndefined();
    await page.locator('button[data-action="loc-add"]').click();
    const row = page.locator('[data-loc-row="1"]');
    await expect(row).toBeVisible();
    // One year after the start, canton and church of the previous row, municipality to be chosen
    const added = await page.evaluate(() => state.taxes.relocations[0]);
    expect(added.canton).toBe("ZH");
    expect(added.from).toBe(await page.evaluate(() => i2ym(startM() + 12)));
    await row.locator("#tax-canton-1").selectOption("ZG");
    await expect(row.locator("#tax-muni-1")).toBeEnabled();
    await row.locator("#tax-muni-1").fill("Zug");
    await row.locator("#tax-muni-1").press("Tab");
    await row.locator('select[data-path="taxes.relocations.0.church"]').selectOption("reformed");
    // From an event (milestone «AHV-Bezug» of the first person)
    const ahv = await page.evaluate(() => state.milestones.find(m => m.name === "AHV-Bezug" || m.id.endsWith("ahv"))?.id);
    await row.locator('select[data-path="taxes.relocations.0.from"]').selectOption(ahv);
    expect(await page.evaluate(() => state.taxes.relocations)).toEqual([{ from: ahv, canton: "ZG", municipality: 1711, church: "reformed" }]);
    await expect(row.locator("#tax-capital-rule-1")).toBeVisible();
    // The first row keeps its place (form of #63)
    expect(await page.evaluate(() => state.taxes.location)).toEqual({ canton: "ZH", municipality: 261, church: "none" });
    // ✕ removes the row; without further rows the plan has no `relocations`
    await row.locator('[data-action="loc-del"]').click();
    await expect(page.locator('[data-loc-row="1"]')).toHaveCount(0);
    expect(await page.evaluate(() => "relocations" in state.taxes)).toBe(false);
    await noErrors(page, errors);
  });

  test("AK6: errors per row at the field and a row before the previous one", async ({ page }) => {
    const errors = await open(page);
    await zurich(page);
    await page.locator('button[data-action="loc-add"]').click();
    await previewIdle(page);
    await expect(page.locator('[data-path="taxes.relocations.0.municipality"] + .ferr')).toContainText("Zeile 2: Bitte die Gemeinde wählen");
    await page.evaluate(() => { state.taxes.relocations[0].municipality = 351; renderSection(); scheduleRefresh(); });
    await previewIdle(page);
    await expect(page.locator('[data-path="taxes.relocations.0.municipality"] + .ferr')).toContainText("liegt nicht im Kanton ZH");
    await page.evaluate(() => { Object.assign(state.taxes.relocations[0], { canton: "ZG", municipality: 1711 }); state.taxes.relocations.push({ from: i2ym(startM() + 6), canton: "ZH", municipality: 261, church: "none" }); renderSection(); scheduleRefresh(); });
    await previewIdle(page);
    await expect(page.locator("#main")).toContainText("Zeile 3: Die Zeile beginnt");
    await page.evaluate(() => { state.taxes.relocations[1].from = i2ym(startM() + 24); renderSection(); scheduleRefresh(); });
    await previewIdle(page);
    await expect(page.locator("#main .ferr")).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("AK7: parameters name the place of residence per period, the yearly values per year", async ({ page }) => {
    const errors = await open(page);
    await zurich(page);
    await page.evaluate(async () => {
      state.taxes.relocations = [{ from: i2ym(startM() + 18), canton: "ZG", municipality: 1711, church: "none" }];
      state.returns.source = "fixed"; state.scenarios.sweeps = [];
      lastRun = await Engine.call("run_simulation", { plan: state, path: null });
      lastRunPlan = state;
      resetVariation(); resultTab = "table"; showTaxCols = true; showResults(true); renderResults(lastRun);
    });
    const rows = await page.evaluate(() => paramSections(state).flatMap(([, r]) => r).filter(([k]) => String(k).startsWith("Wohnort")));
    const from = await page.evaluate(() => ymShow(i2ym(startM() + 18)));
    expect(rows).toEqual([["Wohnort", "Zürich (ZH), Kirchensteuer keine"], [`Wohnort ab ${from}`, "Zug (ZG), Kirchensteuer keine"]]);
    const heads = await page.locator("#resultsBody table.yt th.tx").allTextContents();
    expect(heads[heads.length - 1]).toBe("Wohnort");
    const cells = await page.locator("#resultsBody table.yt td[data-residence]").allTextContents();
    // Rows from the start year; the year of the move (after 18 months) in Zug for the whole year, the year before in Zurich
    const k = await page.evaluate(() => Math.floor((startM() + 18) / 12) - Math.floor(startM() / 12));
    expect(cells[k - 1]).toBe("Zürich (ZH), Kirchensteuer keine");
    expect(cells[k]).toBe("Zug (ZG), Kirchensteuer keine");
    await noErrors(page, errors);
  });
});

test.describe("Large pillar 3a for persons without a pension fund (#152)", () => {
  test("AK1, AK4: check box per person; warning if the plan has a pension fund for this person", async ({ page }) => {
    const errors = await open(page);
    await nav(page, "Haushalt & Projekt");
    const chk = page.locator('input[data-path="household.persons.0.noPensionFund"]');
    await expect(chk).toBeVisible();
    await expect(chk).not.toBeChecked();
    await expect(page.locator("#main")).toContainText("Ohne Pensionskasse (grosse Säule 3a)");
    await expect(page.locator("#main")).toContainText("höchstens 36'288 pro Jahr");
    await chk.check();
    expect(await page.evaluate(() => state.household.persons[0].noPensionFund)).toBe(true);
    // the example plan has a pension fund: warning in the validation
    expect(await page.evaluate(() => state.pension.pk.capital)).toBeGreaterThan(0);
    await expect(page.locator("#val")).toContainText("«Ohne Pensionskasse (grosse Säule 3a)» ist gesetzt, die Planung enthält aber eine Pensionskasse");
    // a second person has their own check box
    await page.locator('button[data-action="partner-add"]').click();
    await expect(page.locator('input[data-path="household.persons.1.noPensionFund"]')).not.toBeChecked();
    await noErrors(page, errors);
  });

  test("AK2, AK3, AK8: maximum of the large pillar 3a in the settings; contribution limit per person as for the deduction", async ({ page }) => {
    const errors = await open(page);
    await page.locator('header button[data-action="settings"]').click();
    await expect(page.locator("#settings")).toContainText("Grosse Säule 3a (ohne Pensionskasse): Maximalbetrag pro Jahr");
    await expect(page.locator("#set-pillar3aMaxLarge")).toHaveValue("36'288");
    await page.locator("#settings button[value='cancel']").click();
    // AK8: 30'000 is an error with the small pillar 3a, allowed without a pension fund (settings go to the engine)
    const codes = await page.evaluate(async () => {
      const out = [];
      for (const large of [false, true]) {
        const plan = JSON.parse(JSON.stringify(state));
        plan.limits = limits();
        plan.household.persons[0].noPensionFund = large;
        plan.pillar3a = [{ person: plan.household.persons[0].id, balance: 0, rate: 0, contribution: 30000, payouts: [] }];
        out.push((await Engine.call("validate", { plan })).map(i => i.code));
      }
      return out;
    });
    expect(codes[0]).toContain("pillar3a_contribution_range");
    expect(codes[1]).not.toContain("pillar3a_contribution_range");
    // hint at the contribution and at the deduction
    await page.evaluate(() => { state.household.persons[0].noPensionFund = true; state.pillar3a = [{ person: state.household.persons[0].id, balance: 0, rate: 0, contribution: 0, payouts: [] }]; markDirty(); renderSection(); });
    await nav(page, "Säule 3a");
    await expect(page.locator("#main")).toContainText("0 bis 36'288, höchstens 20 % des Erwerbseinkommens");
    await nav(page, "Steuern");
    await expect(page.locator("#tax-deductions")).toContainText("ohne Pensionskasse 20 % des Erwerbseinkommens, höchstens 36'288 Franken");
    await noErrors(page, errors);
  });
});

test.describe("Tax value per property for the wealth tax (#229)", () => {
  const kindSel = (page, i) => page.locator(`select[data-action-change="tax-value-kind"][data-i="${i}"]`);
  const valueField = (page, i) => page.locator(`input[data-path="realestate.properties.${i}.taxValue.value"]`);

  test("AK1: owned and planned properties have the field as amount or share, rentals none; the kind keeps the value", async ({ page }) => {
    const errors = await open(page);
    const kinds = await page.evaluate(() => state.realestate.properties.map(p => p.status));
    await nav(page, "Immobilien");
    for (const [k, i] of kinds.map((k, i) => [k, i])) if (k === "rent") await expect(kindSel(page, i)).toHaveCount(0);
    const i = kinds.indexOf("owned");
    expect(i).toBeGreaterThanOrEqual(0);
    await page.evaluate(i => { state.realestate.properties[i].worth = 1000000; renderSection(); }, i);
    // no default: without a choice no tax value
    await expect(kindSel(page, i)).toHaveValue("");
    await expect(valueField(page, i)).toHaveCount(0);
    expect(await page.evaluate(i => state.realestate.properties[i].taxValue, i)).toBeUndefined();
    // amount: starts with the market value, then the value from the tax return
    await kindSel(page, i).selectOption("amount");
    expect(await page.evaluate(i => state.realestate.properties[i].taxValue, i)).toEqual({ kind: "amount", value: 1000000 });
    await valueField(page, i).fill("650'000");
    await valueField(page, i).press("Tab");
    expect(await page.evaluate(i => state.realestate.properties[i].taxValue, i)).toEqual({ kind: "amount", value: 650000 });
    // share of the market value: the same tax value as 65 %
    await kindSel(page, i).selectOption("rate");
    expect(await page.evaluate(i => state.realestate.properties[i].taxValue, i)).toEqual({ kind: "rate", value: 0.65 });
    await expect(valueField(page, i)).toHaveValue(/^65/);
    await expect(page.locator(`[data-tax-value="${i}"]`)).toContainText("Steuerwert");
    await expect(page.locator("#main")).toContainText("oft 60–80 % des Marktwerts");
    // the parameters name the tax value
    const rows = await page.evaluate(() => Object.fromEntries(paramSections(state, true).flatMap(([, r]) => r)));
    const nm = await page.evaluate(i => state.realestate.properties[i].name, i);
    expect(rows[`${nm}: Steuerwert`]).toMatch(/^65.*des Marktwerts$/);
    // back to «wie Marktwert»: no tax value
    await kindSel(page, i).selectOption("");
    expect(await page.evaluate(i => state.realestate.properties[i].taxValue, i)).toBeUndefined();
    // a planned purchase has the field too
    await page.evaluate(i => { state.realestate.properties[i].status = "planned"; renderSection(); }, i);
    await expect(kindSel(page, i)).toBeVisible();
    await page.evaluate(i => { state.realestate.properties[i].status = "rent"; renderSection(); }, i);
    await expect(kindSel(page, i)).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("AK6: a negative amount and a share above 100 % are errors at the property", async ({ page }) => {
    const errors = await open(page);
    const i = await page.evaluate(() => state.realestate.properties.findIndex(p => p.status === "owned"));
    await nav(page, "Immobilien");
    await kindSel(page, i).selectOption("amount");
    await valueField(page, i).fill("-1");
    await valueField(page, i).press("Tab");
    await expect(page.locator(`[data-tax-value="${i}"] .ferr`)).toContainText("Der Steuerwert darf nicht negativ sein");
    await kindSel(page, i).selectOption("rate");
    await valueField(page, i).fill("120");
    await valueField(page, i).press("Tab");
    await expect(page.locator(`[data-tax-value="${i}"] .ferr`)).toContainText("zwischen 0 % und 100 % des Marktwerts");
    expect(await page.evaluate(() => lastIssues.filter(x => x.severity === "error").map(x => [x.section, x.field])))
      .toEqual([["realestate", `realestate.properties.${i}.taxValue.value`]]);
    await noErrors(page, errors);
  });

  test("AK7: the tax breakdown shows the tax value of the properties as a column of its own", async ({ page }) => {
    const errors = await open(page);
    await page.evaluate(async () => {
      const p = state.realestate.properties.find(x => x.status === "owned");
      p.taxValue = { kind: "rate", value: 0.5 };
      state.returns.source = "fixed"; state.scenarios.sweeps = [];
      lastRun = await Engine.call("run_simulation", { plan: state, path: null });
      resetVariation(); resultTab = "table"; showTaxCols = true; showResults(true); renderResults(lastRun);
    });
    const heads = await page.locator("#resultsBody table.yt th.tx").allTextContents();
    expect(heads).toContain("SteuerwertImmobilien");
    const cell = page.locator("#resultsBody table.yt td[data-property-tax-value]").first();
    await expect(cell).toHaveText(/^\d{1,3}('\d{3})+$/);
    await noErrors(page, errors);
  });
});
