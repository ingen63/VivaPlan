// Oberflächentests von VivaPlan im Browser: Gespeicherte Ergebnisse, Grundstückgewinnsteuer. Jeder Test startet mit der Beispielplanung.
const { test, expect } = require("@playwright/test");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { MOD, SECTIONS, open, noErrors, field, previewIdle, nav, kpiValue, resultKpi, parseNum, setOpt, msRow, welcome, steps, helpTitle, prefs } = require("./helpers");

// ---------------------------------------------------------------------------------------------
test.describe("Gespeicherte Ergebnisse", () => {
  test("#147/AK2: Kann die Liste das Ergebnis nicht speichern, erscheint eine Meldung, und die Oberfläche bleibt bedienbar", async ({ page }) => {
    const errors = await open(page);
    // Nachbildung: Der Browserspeicher ist voll (IndexedDB bricht die Transaktion ab).
    await page.evaluate(() => {
      state.returns.source = "fixed"; state.scenarios.sweeps = [];
      Runs.add = async () => { throw new Error("QuotaExceededError"); };
    });
    await page.locator('button[data-action="run"]').click();
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "overview");
    await expect(page.locator("#toast")).toContainText("Ergebnis nicht in der Liste gespeichert: QuotaExceededError");
    // weiter bedienbar: zurück in den Editor
    await page.keyboard.press("Escape");
    await expect(field(page, "project.name")).toBeVisible();
    expect(errors).toEqual([]);
    expect(await page.evaluate(() => window.__errors)).toEqual(["Ergebnis nicht gespeichert: QuotaExceededError"]);
  });

  test("zwei Läufe ablegen und vergleichen", async ({ page }) => {
    const errors = await open(page, "runsflow/mem");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "compare");
    // #115: Die Karte «Gesamtvermögen im Vergleich» heisst jetzt «Bandbreite des Gesamtvermögens»; der Titel nennt den Vergleich
    await expect(page.locator("#resultsTitle")).toContainText("Vergleich von 2 Ergebnissen");
    await expect(page.locator("#resultsBody")).toContainText("Kennzahlen");
    await noErrors(page, errors);
  });

  test("gespeicherte Simulation ansehen", async ({ page }) => {
    const errors = await open(page, "runsflow/mem/view");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "saved");
    await expect(page.locator("#resultsBody svg path").first()).toBeAttached();
    await noErrors(page, errors);
  });

  test("Simulationen einzeln markieren und löschen, alle markieren, alle löschen", async ({ page }) => {
    const errors = await open(page, "runsflow/mem/del");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "runs");
    // Drei weitere Einträge ablegen, dann Liste neu öffnen: 4 Einträge
    await page.evaluate(async () => { for (let i = 0; i < 3; i++) await rememberRun(await Engine.call("run_simulation", { plan: state, path: null })); await openRuns(); });
    const rows = page.locator("#runsTbl tr:not(:first-child)");
    await expect(rows).toHaveCount(4);
    const ask = async () => { await expect(page.locator("#ask")).toBeVisible(); await page.locator("#ask button[value='ok']").click(); };
    // Einzelne markieren und löschen
    await expect(page.locator('button[data-action="runs-delete"]')).toBeDisabled();
    await rows.nth(0).locator('input[data-action="run-sel"]').check();
    await rows.nth(1).locator('input[data-action="run-sel"]').check();
    await expect(page.locator('button[data-action="runs-delete"]')).toContainText("(2)");
    await page.locator('button[data-action="runs-delete"]').click();
    await ask();
    await expect(rows).toHaveCount(2);
    // Alle markieren (mehr als der Vergleich zulässt ist erlaubt), Vergleichen bleibt gesperrt wenn < 2
    await page.locator('input[data-action="runs-all"]').check();
    await expect(page.locator('#runsTbl input[data-action="run-sel"]:checked')).toHaveCount(2);
    await expect(page.locator('button[data-action="runs-compare"]')).toBeEnabled();
    await page.locator('input[data-action="runs-all"]').uncheck();
    await expect(page.locator('#runsTbl input[data-action="run-sel"]:checked')).toHaveCount(0);
    // Alle löschen
    await page.locator('button[data-action="runs-delete-all"]').click();
    await ask();
    await expect(page.locator("#resultsBody")).toContainText("Noch kein Ergebnis");
    await expect(page.locator('button[data-action="runs-delete-all"]')).toBeDisabled();
    await noErrors(page, errors);
  });

  test("Liste und Vergleich: Spalte Zeitreihe statt Art, Strategie-Tooltip mit den Parametern des Ergebnisses, alte Einträge ohne Angabe", async ({ page }) => {
    const errors = await open(page, "runsflow/mem/del");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "runs");
    await page.evaluate(async () => {
      Runs.memoryOnly();
      state.returns.source = "fixed"; state.scenarios.sweeps = [];
      state.withdrawal.strategy = "three_buckets";
      state.withdrawal.buckets = { cashYears: 3, bondYears: 4, refillThreshold: 0.02, reserveMonths: 3 };
      await rememberRun(await Engine.call("run_simulation", { plan: state, path: null }));
      // Eintrag aus einer früheren Version ohne Zeitreihe und Parameter
      await Runs.add({ id: "alt", created: "2020-01-01T00:00:00.000Z", name: "Alt", note: "", kind: "single", runs: 1, endTotal: 1, source: "historical", strategy: "guardrails" },
        { plan: JSON.parse(JSON.stringify(state)), result: lastRun });
      // Strategie des Planes ändern: das gespeicherte Ergebnis behält seine Parameter
      state.withdrawal.buckets.cashYears = 9;
      await openRuns();
    });
    const headers = await page.locator("#runsTbl tr:first-child th").allInnerTexts();
    expect(headers).toContain("Zeitreihe");
    expect(headers).not.toContain("Art");
    const rows = page.locator("#runsTbl tr:not(:first-child)");
    const fresh = rows.filter({ hasText: "3-Topf" }).first();
    await expect(fresh).toContainText("feste Renditen");
    await expect(fresh).toContainText("Einzellauf");
    const tip = await fresh.locator("td.ptip").getAttribute("data-tip");
    expect(tip).toContain("Topf 1 · Cash: 3 J.");
    expect(tip).toContain("Topf 2 · Obligationen: 4 J.");
    expect(tip).toContain("2 %");
    expect(tip).not.toContain("Alter");
    await fresh.locator("td.ptip").focus();
    expect(await fresh.locator("td.ptip").evaluate(el => getComputedStyle(el, "::after").display)).toBe("block");
    const old = rows.filter({ hasText: "Guardrails" }).first();
    await expect(old.locator("td").nth(4)).toHaveText("–");
    expect(await old.locator("td.ptip").getAttribute("data-tip")).toBe("Parameter nicht verfügbar");
    // Vergleich: gleiche Spalten
    await page.evaluate(() => { runsSel.clear(); runsSel.add(runsIds[0]); runsSel.add(runsIds[1]); return compareRuns(); });
    // #118: der Spaltenkopf trägt zusätzlich den Sortierpfeil
    const cmp = (await page.locator("#resultsBody table").first().locator("tr:first-child th").allInnerTexts()).map(t => t.replace(/[▲▼↕]/g, "").trim());
    expect(cmp).toContain("Zeitreihe");
    expect(cmp).not.toContain("Art");
    await noErrors(page, errors);
  });

  test("#13: Liste zeigt «Erfolgsquote · 5 %» statt Median und das Warnsymbol für ungeplante Immobilienverkäufe ab 5 % der Läufe", async ({ page }) => {
    const errors = await open(page, "runsflow/mem/del");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "runs");
    await page.evaluate(async () => {
      Runs.memoryOnly();
      const base = { created: "2021-01-01T00:00:00.000Z", note: "", source: "bootstrap", strategy: "standard", successRate: 0.9, end50: 2_500_000, end5: 1_200_000 };
      const add = (id, extra) => Runs.add({ id, name: id, kind: "batch", runs: 50, ...base, ...extra }, { plan: JSON.parse(JSON.stringify(state)), result: lastRun });
      await add("mc-viel", { emergencySaleRuns: 6 });
      await add("mc-wenig", { emergencySaleRuns: 2 });
      await add("mc-null", { emergencySaleRuns: 0 });
      await add("mc-alt", {});
      await add("einzel-not", { kind: "single", runs: 1, endTotal: 100, emergencySaleRuns: 1 });
      await add("einzel-ohne", { kind: "single", runs: 1, endTotal: 100, emergencySaleRuns: 0 });
      await openRuns();
    });
    const row = id => page.locator("#runsTbl tr", { has: page.locator(`input.runedit[value="${id}"]`) });
    await expect(row("mc-viel")).toContainText("90 % · 5 %: 1.2 Mio");
    await expect(row("mc-viel").locator("td span[title^='Median']")).toHaveAttribute("title", /^Median 2\.5 Mio/);
    await expect(row("mc-viel")).not.toContainText("Median 2.5");
    expect(await row("mc-viel").locator(".hwarn").getAttribute("title")).toBe("Notverkauf einer Immobilie in 12 % der Läufe (6 von 50 Läufen)");
    await expect(row("mc-wenig").locator(".hwarn")).toHaveCount(0);
    await expect(row("mc-null").locator(".hwarn")).toHaveCount(0);
    await expect(row("mc-alt").locator(".hwarn")).toHaveCount(0);
    await expect(row("mc-alt")).toContainText("5 %:");
    await expect(row("einzel-not").locator(".hwarn")).toHaveCount(1);
    await expect(row("einzel-not")).toContainText("Endvermögen");
    await expect(row("einzel-ohne").locator(".hwarn")).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("gespeicherte Simulation löschen", async ({ page }) => {
    const errors = await open(page, "runsflow/mem/del");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "runs");
    await expect(page.locator("#resultsBody table tr:not(:first-child)")).toHaveCount(1);
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Grundstückgewinnsteuer und Kaufdatum", () => {
  test("#138/AK6: Übernommene Vorlagen entsprechen genau den Vorlagen des Rechenkerns", async ({ page }) => {
    const errors = await open(page, "taxes");
    const t = await (await page.request.post("/api/gain_templates", { data: {} })).json();
    for (const [btn, key] of [["gain-zh", "zurich"], ["gain-ag", "aargau"]]) {
      await page.locator(`button[data-action="${btn}"]`).first().click();
      const g = await page.evaluate(() => state.taxes.propertyGain);
      expect(g).toEqual({ ...t[key], useMultiplier: false });
    }
    await noErrors(page, errors);
  });

  test("Vorlage Zürich übernehmen, Tabellen bearbeiten, Regeln entfernen", async ({ page }) => {
    const errors = await open(page, "taxes");
    await page.evaluate(() => { delete state.taxes.propertyGain; renderSection(); });
    await expect(page.locator("#main")).toContainText("Pflicht");
    await page.locator('button[data-action="gain-zh"]').click();
    await expect(field(page, "taxes.propertyGain.brackets.6.1")).toBeVisible();
    await expect(field(page, "taxes.propertyGain.duration.18.1")).toBeVisible();
    await expect(field(page, "taxes.propertyGain.exemptBelow")).toHaveValue(/5.?000/);
    expect(await page.evaluate(() => state.taxes.propertyGain.duration.length)).toBe(19);
    // Zeile ergänzen und löschen
    await page.locator('button[data-action="gain-add"][data-key="duration"]').click();
    expect(await page.evaluate(() => state.taxes.propertyGain.duration.length)).toBe(20);
    await page.locator('button[data-action="list-del"][data-path="taxes.propertyGain.duration"]').last().click();
    expect(await page.evaluate(() => state.taxes.propertyGain.duration.length)).toBe(19);
    // Wert ändern landet in der Planung
    await field(page, "taxes.propertyGain.exemptBelow").fill("8000");
    await field(page, "taxes.propertyGain.exemptBelow").blur();
    expect(await page.evaluate(() => state.taxes.propertyGain.exemptBelow)).toBe(8000);
    await page.locator('button[data-action="gain-del"]').click();
    expect(await page.evaluate(() => state.taxes.propertyGain)).toBeUndefined();
    await page.locator('button[data-action="gain-own"]').click();
    expect(await page.evaluate(() => state.taxes.propertyGain.duration)).toEqual([]);
    await noErrors(page, errors);
  });

  test("Vorlage Aargau, Modellwechsel und Steuerfuss-Option", async ({ page }) => {
    const errors = await open(page, "taxes");
    await page.locator('button[data-action="gain-ag"]').click();
    const g = () => page.evaluate(() => state.taxes.propertyGain);
    let v = await g();
    expect(v.model).toBe("aargau");
    expect(v.durationRates.length).toBe(26);
    expect(v.durationRates[0]).toEqual([0, 0.4]);
    expect(v.durationRates[10][1]).toBeCloseTo(0.2, 9);
    expect(v.durationRates[11][1]).toBeCloseTo(0.19, 9);
    expect(v.durationRates[24][1]).toBeCloseTo(0.06, 9);
    expect(v.durationRates[25]).toEqual([25, 0.05]);
    await expect(field(page, "taxes.propertyGain.durationRates.25.1")).toBeVisible();
    // Steuerfuss-Option
    await page.locator('input[data-path="taxes.propertyGain.useMultiplier"]').check();
    expect((await g()).useMultiplier).toBe(true);
    // Zeile ergänzen, Modell wechseln und zurück
    await page.locator('button[data-action="gain-add"][data-key="durationRates"]').click();
    expect((await g()).durationRates.length).toBe(27);
    await page.locator('button[data-action="gain-model"][data-v="zurich"]').click();
    v = await g();
    expect(v.model).toBe("zurich");
    expect(v.brackets.length).toBeGreaterThan(0);
    await expect(field(page, "taxes.propertyGain.brackets.0.1")).toBeVisible();
    await page.locator('button[data-action="gain-model"][data-v="aargau"]').click();
    expect((await g()).model).toBe("aargau");
    // Eigene Regeln basieren auf dem Modell Aargau
    await page.locator('button[data-action="gain-del"]').click();
    await page.locator('button[data-action="gain-own"]').click();
    v = await g();
    expect(v.model).toBe("aargau");
    expect(v.durationRates).toEqual([[0, 0.2]]);
    await noErrors(page, errors);
  });

  test("Aargau: Verkauf nach 5 Jahren wird mit dem Satz der Besitzdauer besteuert", async ({ page }) => {
    const errors = await open(page, "taxes");
    const total = rate => page.evaluate(async rate => {
      const plan = JSON.parse(JSON.stringify(state));
      plan.taxes.propertyGain = { model: "aargau", durationRates: [[0, 0.4], [5, rate]], brackets: [], duration: [], exemptBelow: 0, reduction: 0 };
      plan.realestate.properties = [{ name: "Haus", status: "owned", price: 400000, worth: 500000, bought: "2030-01", sell: "2035-06", inflation: false, mortgage: { tranches: [] } }];
      const r = await Engine.call("run_simulation", { plan, path: null });
      return r.single.records.reduce((a, x) => a + x.taxGain, 0);
    }, rate);
    const a = await total(0.3), b = await total(0.2);
    expect(a).toBeGreaterThan(0);
    expect(a / b).toBeCloseTo(1.5, 3);
    await noErrors(page, errors);
  });

  test("Immobilien: Inflation für Wert, Nebenkosten und Miete einzeln wählbar", async ({ page }) => {
    const errors = await open(page, "realestate");
    await page.evaluate(() => {
      state.realestate.properties = [
        { id: "h", name: "Haus", status: "owned", price: 400000, worth: 500000, fixCosts: 500, inflation: true, mortgage: { tranches: [] } },
        { id: "w", name: "Wohnung", status: "rent", from: "retire", rent: 2000 }];
      state = normalize(state); renderSection();
    });
    await expect(page.locator('input[data-path="realestate.properties.0.inflationValue"]')).toBeChecked();
    await expect(page.locator('input[data-path="realestate.properties.0.inflationFixCosts"]')).toBeChecked();
    await expect(page.locator('input[data-path="realestate.properties.1.inflationRent"]')).toBeChecked();
    await page.locator('input[data-path="realestate.properties.0.inflationFixCosts"]').uncheck();
    await page.locator('input[data-path="realestate.properties.1.inflationRent"]').uncheck();
    const p = await page.evaluate(() => state.realestate.properties);
    expect([p[0].inflationValue, p[0].inflationFixCosts, p[1].inflationRent]).toEqual([true, false, false]);
    await noErrors(page, errors);
  });

  test("Kaufdatum nur bei Immobilien im Besitz; Warnung ohne Kaufdatum, Rückgängig stellt die Regeln wieder her", async ({ page }) => {
    const errors = await open(page, "realestate");
    await page.evaluate(() => {
      state.taxes.propertyGain = GAIN_ZH();
      state.realestate.properties = [{ name: "Haus", status: "owned", price: 400000, worth: 500000, sell: "2035-06", mortgage: { tranches: [] } }];
      renderSection();
    });
    await expect(field(page, "realestate.properties.0.bought")).toBeVisible();
    await expect(page.locator("body")).toContainText("Kaufdatum fehlt");
    await field(page, "realestate.properties.0.bought").fill("06.2015");
    await field(page, "realestate.properties.0.bought").blur();
    expect(await page.evaluate(() => state.realestate.properties[0].bought)).toBe("2015-06");
    await expect(page.locator("body")).not.toContainText("Kaufdatum fehlt");
    // geplante Immobilie: kein Feld (Kaufdatum ergibt sich aus dem Kauf)
    await page.evaluate(() => { state.realestate.properties[0].status = "planned"; renderSection(); });
    await expect(field(page, "realestate.properties.0.bought")).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("Besitzdauer verändert den Verkauf: Simulation mit und ohne Zuschlag", async ({ page }) => {
    const errors = await open(page, "taxes");
    const total = () => page.evaluate(async () => {
      const r = await Engine.call("run_simulation", { plan: state, path: null });
      return r.single.summary.taxesTotal;
    });
    await page.evaluate(() => {
      state.realestate.properties = [{ name: "Haus", status: "owned", price: 400000, worth: 500000, sell: "2035-06", bought: "2034-12" }];
      state.taxes.propertyGain = { brackets: [[1e18, 0.2]], duration: [], exemptBelow: 0, reduction: 0 };
    });
    const plain = await total();
    await page.evaluate(() => { state.taxes.propertyGain.duration = [[0, 0.5]]; });
    const half = await total();
    await page.evaluate(() => { state.taxes.propertyGain.duration = [[0, 1.0]]; });
    const full = await total();
    // Zuschlag von 50 % bzw. 100 % in den ersten 12 Monaten: die Steuer wächst mit dem Zuschlag
    expect(half).toBeGreaterThan(plain);
    expect(full).toBeGreaterThan(half);
    await noErrors(page, errors);
  });
});
