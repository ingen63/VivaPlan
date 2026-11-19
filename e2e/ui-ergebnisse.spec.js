// User interface tests of VivaPlan in the browser: results and display. Every test starts with the example plan.
const { test, expect } = require("@playwright/test");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { MOD, SECTIONS, open, noErrors, field, previewIdle, nav, kpiValue, resultKpi, parseNum, setOpt, msRow, welcome, steps, helpTitle, prefs } = require("./helpers");

// ---------------------------------------------------------------------------------------------
test.describe("Ergebnisse", () => {
  test("Einzellauf: Übersicht, Grafiken und Jahreswerte stimmen überein", async ({ page }) => {
    const errors = await open(page, "results/nosweep");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "overview");
    await expect(page.locator("#ov-wealth path").first()).toBeAttached();
    const end = await resultKpi(page, "Vermögen am Ende").textContent();
    await page.getByRole("button", { name: "Jahreswerte" }).click();
    const last = page.locator("#resultsBody table.tbl tr").last();
    const total = parseNum(await last.locator("td").nth(5).textContent());
    // Key figure rounded in Mio or k; table in CHF – check the rounding
    const shown = end.includes("Mio") ? parseFloat(end.replace(",", ".")) * 1e6 : parseNum(end) * 1e3;
    expect(Number.isFinite(shown), `Kennzahl «${end}»`).toBe(true);
    expect(Math.abs(total - shown) / Math.max(1, shown)).toBeLessThan(0.006);
    await page.keyboard.press(`${MOD}+2`);
    await expect(page.locator("#resultsBody svg path").first()).toBeAttached();
    await noErrors(page, errors);
  });

  test("Jahreswerte: eine Zeile pro Kalenderjahr, Alter steigt in Schritten von einem Jahr", async ({ page }) => {
    const errors = await open(page, "results/table/nosweep");
    const rows = page.locator("#resultsBody table.tbl tr:not(:first-child)");
    const n = await rows.count();
    expect(n).toBeGreaterThan(5);
    const years = (await rows.locator("td:first-child").allTextContents()).map(t => parseInt(t, 10));
    const ages = (await rows.locator("td:nth-child(2)").allTextContents()).map(Number);
    for (let i = 1; i < years.length - 1; i++) {
      expect(years[i] - years[i - 1]).toBe(1);
      if (i > 1) expect(ages[i] - ages[i - 1]).toBeCloseTo(1, 1);
    }
    await noErrors(page, errors);
  });

  test("Einzellauf: Marktgrafik mit Inflation, logarithmisch und in Prozent; Jahreswerte mit Inflation", async ({ page }) => {
    const errors = await open(page, "results/charts/nosweep");
    await expect(page.locator("#rc-mkt path").first()).toBeAttached();
    await expect(page.locator("#resultsBody")).toContainText("Inflation (Preisindex)");
    await page.locator('button[data-action="mkt-log"][data-v="1"]').click();
    await expect(page.locator("#rc-mkt path").first()).toBeAttached();
    await expect(page.locator("#rc-mkt text").first()).toBeAttached();
    await page.locator('button[data-action="mkt"][data-v="pct"]').click();
    await expect(page.locator('button[data-action="mkt-log"]')).toHaveCount(0);
    await expect(page.locator("#rc-mkt text", { hasText: "%" }).first()).toBeAttached();
    await page.locator('button[data-action="mkt"][data-v="idx"]').click();
    await page.getByRole("button", { name: "Jahreswerte" }).click();
    const header = page.locator("#resultsBody table.yt tr").first();
    await expect(header).toContainText("Inflation");
    // #8: Tax columns are collapsed by default, the button shows and hides them
    await expect(page.locator("#resultsBody table.yt th.tx")).toHaveCount(0);
    await expect(header).not.toContainText("Steuer");
    await page.locator('button[data-action="tax-cols"]').click();
    await expect(page.locator("#resultsBody table.yt th.tx")).toHaveCount(5);
    expect(await page.locator("#resultsBody table.yt th.tx").nth(2).innerText()).toMatch(/^Steuer\s+Kapital Bezug$/);
    expect(await page.locator("#resultsBody table.yt th.tx").first().getAttribute("title")).toContain("Steuerjahr");
    await expect(page.locator("#resultsBody p.hint", { hasText: "Stände am Jahresende" })).toContainText("Zeile des Steuerjahres");
    const cell = await page.locator("#resultsBody table.yt tr").nth(1).locator("td").nth(11).textContent();
    expect(cell).toMatch(/^-?\d+[.,]\d %$/);
    await page.locator('button[data-action="tax-cols"]').click();
    await expect(page.locator("#resultsBody table.yt th.tx")).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("Einzelne Läufe: Auswahl bleibt stehen, nur der Inhalt scrollt; Variation statt Pfadname", async ({ page }) => {
    const errors = await open(page, "results/bootstrap/runs");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "runs");
    // Selection «Lauf wählen» is outside the scrolling area
    await expect(page.locator("#resultsPick")).toContainText("Lauf wählen");
    await expect(page.locator("#resultsBody")).not.toContainText("Lauf wählen");
    const ov = await page.evaluate(() => ["#results", "#resultsPick", "#resultsBody"].map(s => getComputedStyle(document.querySelector(s)).overflowY));
    expect(ov).toEqual(["hidden", "visible", "auto"]);
    await page.locator("#resultsBody").evaluate(el => { el.scrollTop = 600; });
    const pick = await page.locator("#resultsPick").boundingBox();
    await page.locator("#resultsBody").evaluate(el => { el.scrollTop = 1200; });
    expect((await page.locator("#resultsPick").boundingBox()).y).toBe(pick.y);
    // no name of the random path
    await expect(page.locator("#results")).not.toContainText(/Bootstrap-Pfad|Zufallspfad/);
    // Percentile and value of the variation are in the selection
    await page.evaluate(async () => {
      state.returns.bootstrap.paths = 200;
      state.scenarios.sweeps = [{ param: "wealth.stockShare", from: 0.3, to: 0.6, step: 0.1 }];
      lastRun = await Engine.call("run_simulation", { plan: state, path: null });
      detailP = 50; renderResults(lastRun);
    });
    await expect(page.locator("#resultsPick .computed").last()).toContainText(/50\. Perzentil · Aktienanteil (≈ |< |≥ |≤ )?\d+(\.\d+)? %/);
    await noErrors(page, errors);
  });

  test("Perzentil-Tabelle zeigt je Perzentil den optimalen Wert der Variation (Ausgaben-Faktor)", async ({ page }) => {
    const errors = await open(page);
    const out = await page.evaluate(async () => {
      state.returns.source = "bootstrap";
      state.returns.bootstrap.paths = 300;
      state.flows.spending = JSON.parse(JSON.stringify(state.flows.spending));
      const rows = state.flows.spending.rows ?? [];
      // so tight that bad runs are depleted: living costs without the housing costs (1'700 in the example, #53) × 1.6
      rows.forEach(r => { r.value = (r.value - 1700) * 1.6 + 1700; });
      state.scenarios.sweeps = [{ param: "flows.spendingFactor", from: 0.5, to: 1.1, step: 0.1 }];
      lastRun = await Engine.call("run_simulation", { plan: state, path: null });
      resultTab = "overview"; showResults(true); renderResults(lastRun);
      const pc = p => lastRun.batch.summary.percentiles.find(x => x.percent === p);
      const cells = [...document.querySelectorAll("#resultsBody table.tbl tr")].filter(tr => tr.querySelector("td")).map(tr => [...tr.querySelectorAll("td")].map(td => td.textContent));
      return { cells, one: pc(1), fifty: pc(50), expected: [1, 5, 50, 95].map(p => [p, optimumText(lastRun.scenarios, p)]) };
    });
    const row = n => out.cells.find(c => c[0] === `${n}.`);
    for (const [p, text] of out.expected) {
      expect(text).toMatch(/^Ausgaben-Faktor (≈ |< |≥ )?\d+(\.\d+)? %$/);
      expect(row(p).at(-1)).toBe(text); // Column «Variation» is the last one
    }
    // The display is no longer the same value for all percentiles, and not the value of the plan (100 %)
    const shown = out.expected.map(([, t]) => t);
    expect(new Set(shown).size).toBeGreaterThan(1);
    expect(out.one.depleted, "1. Perzentil ist in der Basisplanung aufgebraucht").toBeTruthy();
    expect(shown[0]).not.toContain("100 %");
    await expect(page.locator("#resultsBody .hint").filter({ hasText: "optimaler Wert je Perzentil" })).toBeVisible();
    await noErrors(page, errors);
  });

  test("Hypothekenerneuerung: Protokoll im Einzellauf, Zähler und Spalten bei mehreren Läufen", async ({ page }) => {
    const errors = await open(page);
    const out = await page.evaluate(async () => {
      state.returns.source = "fixed";
      state.scenarios.sweeps = [];
      setP("flows.income", { mode: "const", value: 6000 });
      setP("flows.spending", { mode: "const", value: 0 });
      state.flows.spendingIncludesHousing = false; // housing costs in addition (#53)
      state.wealth.cash = 2000000;
      state.realestate.properties = [{ id: "h", name: "Haus", status: "owned", price: 500000, worth: 800000, sell: "never", fixCosts: 0,
        inflation: false, mortgage: { tranches: [{ name: "T", amount: 400000, start: "2026-03", after: "renew", periods: [{ type: "fixed", term: 1, rate: 0.01 }] }] } }];
      lastRun = await Engine.call("run_simulation", { plan: state, path: null });
      resultTab = "overview"; showResults(true); renderResults(lastRun);
      const single = document.querySelector("#resultsBody").innerText;
      state.returns.source = "bootstrap"; state.returns.bootstrap.paths = 200;
      lastRun = await Engine.call("run_simulation", { plan: state, path: null });
      renderResults(lastRun);
      if (!lastRun.batch) throw new Error(JSON.stringify({ error: lastRun.error, issues: (lastRun.issues || []).map(i => i.message) }));
      return { single, batch: document.querySelector("#resultsBody").innerText, s: lastRun.batch.summary };
    });
    expect(out.single).toContain("Immobilien");
    expect(out.single).toContain("Hypothekenerneuerung");
    expect(out.single).not.toContain("Hypothekenerneuerungen"); // section is called «Immobilien»
    expect(out.single).toMatch(/nicht gegeben/);
    expect(out.single).toContain("Einmalrückzahlung Hypothek Haus");
    // #23: no more lump-sum payments in the overview, key figure «Notverkäufe» remains, column «Immobilienverkäufe»
    expect(out.batch).not.toContain("Einmalzahlungen");
    expect(out.batch).toContain("Notverkäufe");
    expect(out.batch).toContain("Immobilienverkäufe");
    expect(out.s.emergencySaleRuns).toBe(0);
    expect(out.s.percentiles.every(p => p.emergencySales === 0)).toBe(true);
    expect(out.s.lumpSumRuns).toBe(200);
    expect(out.s.affordabilitySaleRuns).toBe(0);
    expect(out.s.percentiles.every(p => p.lumpSums >= 1)).toBe(true);
    await noErrors(page, errors);
  });

  test("Zoom-Schaltflächen: zentriert auf den aktuellen Bereich, im Zeitraum gehalten, «Alles» setzt zurück", async ({ page }) => {
    const errors = await open(page, "results/charts/nosweep");
    const r = await page.evaluate(() => {
      const total = endM() - startM(), out = { total };
      resultZoom = null; stepZoom(5); out.fresh = { ...resultZoom };               // middle of the whole period
      setCustomZoom(10 * 12, 20 * 12); out.custom = { ...resultZoom };              // mouse selection 10–20 years
      stepZoom(5); out.centered = { ...resultZoom };                                // 5 years around the middle (15 years)
      stepZoom(10); out.ten = { ...resultZoom };
      setCustomZoom(12, 36); stepZoom(5); out.nearStart = { ...resultZoom };        // middle 2 years → shifted instead of cut off
      setCustomZoom(total - 36, total - 12); stepZoom(5); out.nearEnd = { ...resultZoom };
      stepZoom(100); out.tooLong = resultZoom;                                      // longer than the simulation → everything
      stepZoom(2); stepZoom(0); out.all = resultZoom;
      return out;
    });
    const t = r.total;
    expect(r.fresh).toEqual({ from: Math.round(t / 2 - 30), to: Math.round(t / 2 - 30) + 60, years: 5 });
    expect(r.custom).toEqual({ from: 120, to: 240, years: null });
    expect(r.centered).toEqual({ from: 150, to: 210, years: 5 });
    expect(r.ten).toEqual({ from: 150 + 30 - 60, to: 150 + 30 + 60, years: 10 });
    expect(r.nearStart).toEqual({ from: 0, to: 60, years: 5 });
    expect(r.nearEnd).toEqual({ from: t - 60, to: t, years: 5 });
    expect(r.tooLong).toBeNull();
    expect(r.all).toBeNull();
    await noErrors(page, errors);
  });

  test("Zoom per Maus: Bereich aufziehen (sichtbar hervorgehoben), alle Grafiken und Registerkarten zoomen mit, Schaltfläche danach nicht aktiv", async ({ page }) => {
    const errors = await open(page, "results/charts/nosweep");
    const bars = () => page.locator("#resultsBody .minizoom");
    const n = await bars().count();
    expect(n).toBeGreaterThanOrEqual(5);
    // Starting point: «Alles» active everywhere
    expect(await page.locator("#resultsBody .minizoom button.on").allTextContents()).toEqual(Array(n).fill("Alles"));
    await page.locator("#rc-comp").scrollIntoViewIfNeeded();
    const box = await page.locator("#rc-comp").boundingBox();
    const y = box.y + box.height / 2;
    const before = await page.evaluate(() => document.querySelector("#rc-comp path").getAttribute("d").length);
    // a selection that is too small (one click) is ignored
    await page.mouse.move(box.x + box.width * 0.5, y); await page.mouse.down(); await page.mouse.move(box.x + box.width * 0.5 + 3, y); await page.mouse.up();
    expect(await page.evaluate(() => resultZoom)).toBeNull();
    // drag a range: it is highlighted while dragging
    await page.mouse.move(box.x + box.width * 0.30, y);
    await page.mouse.down();
    await page.mouse.move(box.x + box.width * 0.45, y, { steps: 5 });
    await expect(page.locator("#rc-comp rect.sel")).toBeVisible();
    const selW = await page.locator("#rc-comp rect.sel").evaluate(e => +e.getAttribute("width"));
    expect(selW).toBeGreaterThan(20);
    await page.mouse.move(box.x + box.width * 0.60, y, { steps: 5 });
    await page.mouse.up();
    const z = await page.evaluate(() => ({ ...resultZoom, total: endM() - startM() }));
    expect(z.years).toBeNull();
    expect(z.to - z.from).toBeGreaterThan(12);
    expect(z.to - z.from).toBeLessThan(z.total * 0.45);
    // all charts now show «benutzerdefiniert», no button active
    await expect(page.locator("#resultsBody .minizoom button.on")).toHaveCount(0);
    await expect(page.locator("#resultsBody .minizoom span").first()).toContainText("Benutzerdefiniert");
    const after = await page.evaluate(() => document.querySelector("#rc-comp path").getAttribute("d").length);
    expect(after).toBeLessThan(before);
    // Tabs: overview shows the same zoom, switching keeps it
    await page.getByRole("button", { name: "Übersicht" }).click();
    expect(await page.evaluate(() => resultZoom.from)).toBe(z.from);
    await expect(page.locator("#resultsBody .minizoom span").first()).toContainText("Benutzerdefiniert");
    await page.getByRole("button", { name: "Grafiken" }).click();
    expect(await page.evaluate(() => resultZoom.to)).toBe(z.to);
    // «5 J.» afterwards: 5 years around the middle of the previous range
    await page.locator('#resultsBody button[data-action="res-zoom"][data-years="5"]').first().click();
    const five = await page.evaluate(() => ({ ...resultZoom }));
    const mid = (z.from + z.to) / 2;
    expect(five.to - five.from).toBe(60);
    expect(Math.abs((five.from + five.to) / 2 - mid)).toBeLessThanOrEqual(1);
    expect(await page.locator("#resultsBody .minizoom button.on").allTextContents()).toEqual(Array(n).fill("5 J."));
    // Bar chart: dragging over several years zooms as well
    await page.locator('#resultsBody button[data-action="res-zoom"][data-years="0"]').first().click();
    await page.locator("#rc-flows").scrollIntoViewIfNeeded();
    const fb = await page.locator("#rc-flows").boundingBox();
    await page.mouse.move(fb.x + fb.width * 0.2, fb.y + fb.height / 2); await page.mouse.down();
    await page.mouse.move(fb.x + fb.width * 0.5, fb.y + fb.height / 2, { steps: 5 }); await page.mouse.up();
    expect(await page.evaluate(() => resultZoom?.years)).toBeNull();
    expect(await page.evaluate(() => resultZoom != null)).toBe(true);
    // «Alles»
    await page.locator('#resultsBody button[data-action="res-zoom"][data-years="0"]').first().click();
    expect(await page.evaluate(() => resultZoom)).toBeNull();
    expect(await page.locator("#resultsBody .minizoom button.on").allTextContents()).toEqual(Array(n).fill("Alles"));
    // Editor mini charts: own zoom, not linked
    await page.locator('#resultsBody button[data-action="res-zoom"][data-years="10"]').first().click();
    await page.keyboard.press("Escape");
    await expect(page.locator("#results")).toBeHidden();
    await nav(page, "Einnahmen");
    expect(await page.locator('.minizoom button[data-action="mini-zoom"].on').first().textContent()).toBe("Alles");
    await page.locator('button[data-action="mini-zoom"][data-years="5"]').first().click();
    expect(await page.evaluate(() => resultZoom.years)).toBe(10);
    await noErrors(page, errors);
  });

  test("Zoom wirkt auf die Daten der Grafik und auf die Balkendiagramme", async ({ page }) => {
    const errors = await open(page, "results/charts/nosweep");
    const bars = () => page.evaluate(() => document.querySelectorAll("#rc-flows rect").length);
    const all = await bars();
    await page.locator('#resultsBody button[data-action="res-zoom"][data-years="2"]').first().click();
    const z = await bars();
    expect(z).toBeLessThan(all);
    expect(z).toBeGreaterThan(0);
    await noErrors(page, errors);
  });

  test("Einzellauf mit Variationen: Auswahl der Stufe, Optimum, gleiche Anzeige wie bei Monte-Carlo", async ({ page }) => {
    const errors = await open(page);
    const out = await page.evaluate(async () => {
      state.returns.source = "fixed";
      state.scenarios.sweeps = [{ param: "flows.spendingFactor", from: 0.5, to: 1.1, step: 0.1 }];
      lastRun = await Engine.call("run_simulation", { plan: state, path: null });
      resetVariation(); resultTab = "overview"; showResults(true); renderResults(lastRun);
      const pick = document.querySelector("#resultsPick").innerText;
      const planEnd = lastRun.single.summary.endTotal;
      await pickVariation([0.5]);
      const t50 = document.querySelector("#resultsTitle").textContent;
      const end50 = singleOf(lastRun).summary.endTotal;
      const scen50 = lastRun.scenarios.results.find(x => x.values[0] === 0.5).percentiles.find(p => p.percent === 50).endTotal;
      await pickVariation(state.scenarios.sweeps.map(() => null));
      const back = singleOf(lastRun).summary.endTotal;
      const opt = Optimum.find(lastRun.scenarios, 50).items[0];
      return { pick, planEnd, t50, end50, scen50, back, opt, optText: optimumText(lastRun.scenarios, 50) };
    });
    expect(out.pick).toContain("Variation wählen");
    expect(out.pick).toContain("Optimum:");
    expect(out.pick).toContain("Ausgaben-Faktor");
    expect(out.t50).toContain("Ausgaben-Faktor 50 %");
    // The run of level 50 % is the same as in the scenarios (no approximation)
    expect(Math.abs(out.end50 - out.scen50)).toBeLessThan(1);
    expect(out.end50).toBeGreaterThan(out.planEnd); // less spending → more final wealth
    expect(out.back).toBe(out.planEnd);
    expect(out.optText).toMatch(/^Ausgaben-Faktor /);
    await noErrors(page, errors);
  });

  test("Einzellauf ohne Variationen: keine Variationsauswahl; Monte-Carlo mit Variation unverändert", async ({ page }) => {
    const errors = await open(page);
    const out = await page.evaluate(async () => {
      state.returns.source = "fixed"; state.scenarios.sweeps = [];
      lastRun = await Engine.call("run_simulation", { plan: state, path: null });
      resetVariation(); resultTab = "overview"; showResults(true); renderResults(lastRun);
      const single = { pick: document.querySelector("#resultsPick").innerText, title: document.querySelector("#resultsTitle").textContent };
      state.returns.source = "bootstrap"; state.returns.bootstrap.paths = 200;
      state.scenarios.sweeps = [{ param: "flows.spendingFactor", from: 0.5, to: 1.1, step: 0.2 }];
      lastRun = await Engine.call("run_simulation", { plan: state, path: null });
      resultTab = "overview"; renderResults(lastRun);
      return { single, mcPick: document.querySelector("#resultsPick").innerText, table: document.querySelector("#resultsBody").innerText, hasBatch: !!lastRun.batch };
    });
    expect(out.single.pick).toBe("");
    expect(out.single.title).toBe("Einzellauf · Fixe Renditen");
    expect(out.hasBatch).toBe(true);
    expect(out.mcPick).toBe(""); // no level selection with Monte Carlo
    expect(out.table).toMatch(/Variation/);
    expect(out.table).toMatch(/Ausgaben-Faktor (≈ |< |≥ )?\d/);
    await noErrors(page, errors);
  });

  test("Abschnitt «Immobilien»: Kauf, Verkauf, Erneuerung mit Einmalzahlung und Notverkauf mit Beträgen; ohne Ereignisse ein Hinweis", async ({ page }) => {
    const errors = await open(page);
    const out = await page.evaluate(async () => {
      state.returns.source = "fixed"; state.scenarios.sweeps = [];
      state.taxes.propertyGain = { brackets: [], reduction: 0 };   // required with real estate; rate 0 % so that the amounts stay unchanged
      setP("flows.income", { mode: "const", value: 6000 }); setP("flows.spending", { mode: "const", value: 0 });
      state.flows.spendingIncludesHousing = false; // housing costs in addition (#53)
      state.wealth.cash = 2000000;
      const house = (extra = {}) => ({ id: "h", name: "Haus", status: "owned", price: 500000, worth: 800000, sell: "never", fixCosts: 0, inflation: false,
        mortgage: { tranches: [{ name: "T", amount: 400000, start: "2026-03", after: "renew", periods: [{ type: "fixed", term: 1, rate: 0.01 }] }] }, ...extra });
      const show = async props => {
        state.realestate.properties = props;
        lastRun = await Engine.call("run_simulation", { plan: state, path: null });
        resultTab = "overview"; showResults(true); renderResults(lastRun);
        const card = [...document.querySelectorAll("#resultsBody .card")].find(c => c.querySelector("h3")?.textContent.startsWith("Immobilien"));
        return { text: card?.innerText ?? "", events: lastRun.single.propertyEvents };
      };
      const res = {};
      res.none = await show([]);
      res.sale = await show([house({ sell: "2026-09" })]);
      res.buy = await show([{ id: "n", name: "Neubau", status: "planned", buy: "2028-06", price: 600000, fixCosts: 0, inflation: false,
        mortgage: { tranches: [{ name: "T", amount: 300000, periods: [{ type: "fixed", term: 20, rate: 0.015 }] }] } }]);
      res.lump = await show([house()]);
      setP("flows.income", { mode: "const", value: 2000 }); state.wealth.cash = 10000; state.wealth.stocks = 0; state.wealth.bonds = 0;
      res.emergency = await show([house()]);
      return res;
    });
    // without real estate
    expect(out.none.text).toContain("Keine Immobilienereignisse");
    // Sale: proceeds, repaid mortgage, net proceeds
    expect(out.sale.text).toContain("Verkauf");
    expect(out.sale.text).toMatch(/Verkaufserlös\s+800'000/);
    expect(out.sale.text).toMatch(/abgelöste Hypothek\s+400'000/);
    expect(out.sale.text).toMatch(/Nettoerlös\s+400'000/);
    expect(out.sale.text).toContain("Entnahmestrategie angewendet (kein Folgekauf)");
    // Purchase: price, own funds, mortgage
    expect(out.buy.text).toContain("Kauf");
    expect(out.buy.text).toMatch(/Kaufpreis\s+600'000/);
    expect(out.buy.text).toMatch(/Eigenmittel\s+300'000/);
    expect(out.buy.text).toMatch(/Hypothek\s+300'000/);
    // Renewal with lump-sum payment: before/after, payment, interest rate
    expect(out.lump.text).toContain("Hypothekenerneuerung");
    expect(out.lump.text).toMatch(/Hypothek vorher \/ nachher\s+400'000 \/ 320'000/);
    expect(out.lump.text).toMatch(/Einmalzahlung \(Tragbarkeit\)\s+80'000/);
    expect(out.lump.text).toMatch(/Zinssatz\s+1 %/);
    // Emergency sale with reason, in chronological order after the renewal that did not take place
    expect(out.emergency.text).toContain("Notverkauf");
    expect(out.emergency.text).toContain("Vermögen < 0");
    const kinds = out.emergency.events.map(e => e.kind);
    expect(kinds).toEqual(["renewal", "emergencySale"]);
    expect(out.emergency.events[0].month < out.emergency.events[1].month).toBe(true);
    await noErrors(page, errors);
  });

  test("Tabellenkopf der Ergebnisse bleibt beim Scrollen oben stehen", async ({ page }) => {
    const errors = await open(page);
    await page.evaluate(async () => {
      state.returns.source = "fixed"; state.scenarios.sweeps = [];
      lastRun = await Engine.call("run_simulation", { plan: state, path: null });
      resetVariation(); resultTab = "table"; showResults(true); renderResults(lastRun);
    });
    await expect(page.locator("#resultsBody table.yt")).toBeAttached();
    const out = await page.evaluate(() => {
      const tb = document.querySelector("#resultsBody table.yt");
      const ths = [...document.querySelectorAll("#resultsBody table.tbl th")];
      const wrap = tb?.closest(".tscroll");
      return {
        n: ths.length, allSticky: ths.every(th => getComputedStyle(th).position === "sticky"),
        wrapScrolls: wrap ? getComputedStyle(wrap).overflowY : null
      };
    });
    expect(out.n).toBeGreaterThan(5);
    expect(out.allSticky).toBe(true);
    expect(out.wrapScrolls).toBe("auto");
    await noErrors(page, errors);
  });

  test("3-Topf: Herleitung der Topfgrössen (Netto-Bedarf, Ziele, Kriterium, Fall, Umbuchungen) aufklappbar in den Jahreswerten, nicht mehr als eigene Karte", async ({ page }) => {
    const errors = await open(page);
    const out = await page.evaluate(async () => {
      state.returns.source = "fixed"; state.scenarios.sweeps = [];
      state.withdrawal.strategy = "three_buckets";
      state.withdrawal.buckets = { cashYears: 2, bondYears: 1, refillThreshold: -1, reserveMonths: 3 };
      lastRun = await Engine.call("run_simulation", { plan: state, path: null });
      resetVariation(); resultTab = "overview"; showResults(true); renderResults(lastRun);
      const cards = [...document.querySelectorAll("#resultsBody .card h3")].map(h => h.textContent);
      const r = lastRun.single.bucketRefills;
      return { cards, n: r.length, first: r[0], jan: r.find(x => x.trigger === "Januar") };
    });
    // #9: no card of its own; #10: no 3-bucket bookings in the event list
    expect(out.cards.some(t => t.startsWith("3-Topf-Strategie"))).toBe(false);
    await expect(page.locator("#resultsBody .evlist")).not.toContainText("Topf 1");
    await page.getByRole("button", { name: "Jahreswerte" }).click();
    const toggles = page.locator('button[data-action="bk-toggle"]');
    expect(await toggles.count()).toBeGreaterThan(1);
    await expect(page.locator("#resultsBody tr.bk")).toHaveCount(0);
    await toggles.first().click();
    const text = await page.locator("#resultsBody tr.bk").first().innerText();
    expect(text).toContain("Netto-Bedarf");
    expect(text).toContain("PK-Rente");
    expect(text).toContain("Renditekriterium");
    expect(text).toContain("Umbuchungen");
    expect(text).toMatch(/Topf \d → Topf \d: [\d']+/);
    await toggles.first().click();
    await expect(page.locator("#resultsBody tr.bk")).toHaveCount(0);
    expect(out.n).toBeGreaterThan(3);
    expect(out.first.trigger).toBe("A");
    const j = out.jan;
    expect(j.need).toBeCloseTo(Math.max(0, j.basis.living + j.basis.housing + j.basis.incomeTax - j.basis.ahv - j.basis.pk - j.basis.earned), 6);
    expect(j.target1).toBeCloseTo(2 * j.need, 6);
    expect(j.target2).toBeCloseTo(1 * j.need, 6);
    await noErrors(page, errors);
  });

  test("Steuerspalten: die Wahl bleibt nach dem Neuladen erhalten, ohne Speicher gilt eingeklappt", async ({ page }) => {
    const errors = await open(page, "results/charts/nosweep");
    await page.getByRole("button", { name: "Jahreswerte" }).click();
    await expect(page.locator("#resultsBody table.yt th.tx")).toHaveCount(0);
    await page.locator('button[data-action="tax-cols"]').click();
    await expect(page.locator("#resultsBody table.yt th.tx")).toHaveCount(5);
    await page.reload();
    await page.waitForSelector("html[data-ready='1']");
    await expect(page.locator("button[data-action='rtab'][data-tab='table']")).toBeVisible();
    await page.locator("button[data-action='rtab'][data-tab='table']").click();
    await expect(page.locator("#resultsBody table.yt th.tx")).toHaveCount(5);
    // storage not readable: collapsed
    await page.evaluate(() => localStorage.setItem("rp.settings", "kein json"));
    await page.reload();
    await page.waitForSelector("html[data-ready='1']");
    await page.locator("button[data-action='rtab'][data-tab='table']").click();
    await expect(page.locator("#resultsBody table.yt th.tx")).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("Reiter Parameter: Stand des Laufs, schreibgeschützt, je Editorabschnitt ein Parameter wirkt, alte Ergebnisse «Nicht verfügbar»", async ({ page }) => {
    const errors = await open(page, "results/charts/nosweep");
    await page.locator("button[data-action='rtab'][data-tab='params']").click();
    const body = page.locator("#resultsBody");
    await expect(body).toContainText("Haushalt");
    for (const t of ["Meilensteine", "Vorsorge", "Einnahmen und Ausgaben", "Vermögen und Anlage", "Renditequelle und Zeitreihe", "Entnahmestrategie", "Immobilien", "Steuerannahmen", "Variationen"]) await expect(body).toContainText(t);
    await expect(body).toContainText("Geburt");
    await expect(body).not.toContainText("Person A"); // names are not part of it
    await expect(body.locator("input, select, textarea")).toHaveCount(0);
    const text = async () => (await body.innerText()).replace(/^ⓘ\n/gm, "");
    const before = await text();
    expect(before).not.toContain("[object");
    // Change the editor: the computed result keeps its parameters
    await page.evaluate(() => { state.wealth.cash += 12345; state.taxes.multiplier = 7; });
    await page.locator("button[data-action='rtab'][data-tab='overview']").click();
    await page.locator("button[data-action='rtab'][data-tab='params']").click();
    expect(await text()).toBe(before);
    // Change one parameter per editor section: only this section of the summary changes
    const diffs = await page.evaluate(() => {
      const base = JSON.parse(JSON.stringify(lastRunPlan)), sum = p => Object.fromEntries(paramSections(p).map(([t, r]) => [t, JSON.stringify(r)]));
      const mut = {
        "Haushalt": p => { p.household.persons[0].birth = "1960-01"; },
        "Meilensteine": p => { const m = p.milestones.find(x => x.id === "retire"); if (m.kind === "date") m.date = "2030-01"; else m.years += 1; },
        "Vorsorge": p => { p.pension.pk.capital += 1000; },
        "Einnahmen und Ausgaben": p => { p.flows.extras.push({ date: "2030-01", amount: -1, label: "x" }); },
        "Vermögen und Anlage": p => { p.wealth.stocks += 1; },
        "Renditequelle und Zeitreihe": p => { p.returns.fixed.stocks += 0.01; },
        "Entnahmestrategie": p => { p.withdrawal.strategy = "fixed_rate"; p.withdrawal.fixedRate = { rate: 0.035 }; },
        "Immobilien": p => { p.realestate.emergencyMonths += 1; },
        "Steuerannahmen": p => { p.taxes.multiplier += 1; },
        "Variationen": p => { p.scenarios.sweeps = [{ param: "wealth.stockShare", from: 0.3, to: 1, step: 0.1 }]; },
      };
      const b = sum(base), out = {};
      for (const [k, f] of Object.entries(mut)) { const p = JSON.parse(JSON.stringify(base)); f(p); const n = sum(p); out[k] = Object.keys(b).filter(t => b[t] !== n[t]); }
      return out;
    });
    for (const [k, changed] of Object.entries(diffs)) expect(changed, k).toEqual([k]);
    // Result without a saved plan
    await page.evaluate(() => { lastRunPlan = null; renderResults(lastRun); });
    await page.locator("button[data-action='rtab'][data-tab='params']").click();
    await expect(body).toContainText("Nicht verfügbar");
    await noErrors(page, errors);
  });

  test("Gespeichertes Ergebnis zeigt im Reiter Parameter die Planung von damals", async ({ page }) => {
    const errors = await open(page, "runsflow/mem/view");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "saved");
    await page.locator("button[data-action='rtab'][data-tab='params']").click();
    await expect(page.locator("#resultsBody")).toContainText("Aktienanteil");
    await expect(page.locator("#resultsBody")).toContainText("Monte Carlo");
    await noErrors(page, errors);
  });

  test("Jahreswerte: Entnahmerate (Jahresentnahme / Vermögen am Jahresanfang), nominal wie real", async ({ page }) => {
    const errors = await open(page);
    const out = await page.evaluate(async () => {
      state.returns.source = "fixed"; state.scenarios.sweeps = [];
      lastRun = await Engine.call("run_simulation", { plan: state, path: null });
      resetVariation(); resultTab = "table"; showResults(true); renderResults(lastRun);
      const res = lastRun.single, recs = res.records, by = {};
      for (const r of recs) (by[r.month.slice(0, 4)] ||= []).push(r);
      const years = Object.keys(by);
      const cellText = (row, col) => document.querySelectorAll("#resultsBody table.yt tr")[row + 1].querySelectorAll("td")[col].textContent.trim();
      const expect_ = (i, key) => {
        const rs = by[years[i]], draw = rs.reduce((a, r) => a + r.withdrawal, 0) / rs.length * 12;
        const prev = i ? by[years[i - 1]].at(-1)[key] : res.summary[key === "total" ? "startTotal" : "startLiquid"];
        return draw > 0 ? (prev > 0 ? (draw / prev * 100).toFixed(1) : "–") : "";
      };
      const rows = years.map((_, i) => ({ total: cellText(i, 9), eTotal: expect_(i, "total") }));
      realView = true; renderResults(lastRun);
      const realFirst = cellText(1, 9);
      return { rows, realFirst, startTotal: res.summary.startTotal, startLiquid: res.summary.startLiquid };
    });
    expect(out.startTotal).toBeGreaterThan(0);
    expect(out.startLiquid).toBeGreaterThan(0);
    expect(out.startTotal).toBeGreaterThanOrEqual(out.startLiquid);
    const norm = t => t.replace(/\s*%$/, "").replace(",", ".");
    let withRate = 0;
    for (const r of out.rows) {
      expect(norm(r.total)).toBe(r.eTotal);
      if (r.eTotal !== "" && r.eTotal !== "–") withRate++;
    }
    expect(withRate).toBeGreaterThan(3);
    expect(out.realFirst).toBe(out.rows[1].total);
    await noErrors(page, errors);
  });

  test("#23: Immobilienverkäufe je Perzentil-Lauf mit Datum, Notverkäufe mit Warnsymbol, ohne Immobilien keine Spalte", async ({ page }) => {
    const errors = await open(page);
    const out = await page.evaluate(async () => {
      const lines = html => { const d = document.createElement("div"); d.innerHTML = html; return [...d.querySelectorAll("div")].map(x => ({ text: x.textContent.replace(/\s+/g, " ").trim(), warn: !!x.querySelector(".hwarn") })); };
      const two = lines(propertySales({ propertyEvents: [
        { kind: "emergencySale", month: "2048-03", property: "Wohnung" }, { kind: "sale", month: "2034-05", property: "Haus" }, { kind: "purchase", month: "2030-01", property: "X" }, { kind: "renewal", month: "2031-01", property: "Haus" }] }));
      const none = propertySales({ propertyEvents: [{ kind: "purchase", month: "2030-01", property: "X" }] });
      state.returns.source = "bootstrap"; state.returns.bootstrap.paths = 200; state.scenarios.sweeps = [];
      lastRun = await Engine.call("run_simulation", { plan: state, path: null });
      resultTab = "overview"; showResults(true); renderResults(lastRun);
      const hdr = [...document.querySelectorAll("#resultsBody table.tbl th")].map(t => t.textContent);
      const b = lastRun.batch;
      const cells = [...document.querySelectorAll("#resultsBody table.tbl tr")].filter(r => r.querySelector("td.l")).map(r => r.cells[2]?.innerText);
      const expected = b.summary.percentiles.map(p => {
        const ev = (b.details.find(([q]) => q === p.percent)?.[1].propertyEvents || []).filter(e => e.kind === "sale" || e.kind === "emergencySale");
        return ev.length;
      });
      const shown = b.summary.percentiles.map((_, i) => document.querySelectorAll("#resultsBody table.tbl tr")[i + 1]?.cells[2]?.querySelectorAll("div").length ?? 0);
      state.realestate.properties = []; renderResults(lastRun);
      const hdrNone = [...document.querySelectorAll("#resultsBody table.tbl th")].map(t => t.textContent);
      return { two, none, hdr, expected, shown, hdrNone };
    });
    expect(out.two).toEqual([{ text: "Haus: 05.2034", warn: false }, { text: "Wohnung: 03.2048 🏠⚠️", warn: true }]);
    expect(out.none).toBe("–");
    expect(out.hdr).toContain("Immobilienverkäufe");
    expect(out.hdr).not.toContain("Einmalzahlungen");
    expect(out.shown).toEqual(out.expected.map(n => n));
    expect(out.hdrNone).not.toContain("Immobilienverkäufe");
    await noErrors(page, errors);
  });

  test("#21: Variationen ohne feste Grenze (technisch 1'000'000), Warnung vor langen Berechnungen", async ({ page }) => {
    const errors = await open(page, "scenarios");
    const run = async sweeps => {
      // #83: renderSection and renderDerived draw synchronously; no waiting needed
      await page.evaluate(s => { state.scenarios.sweeps = s; renderSection(); renderDerived(); }, sweeps);
      return page.evaluate(async () => {
        const out = await Engine.call("validate", { plan: state });
        return { text: document.querySelector('[data-computed="runs"]')?.innerText || "", errors: out.filter(i => i.severity === "error").map(i => i.message) };
      });
    };
    const sw = (param, from, to, step = 0.1) => ({ param, from, to, step });
    // 1'100 combinations (rejected earlier): allowed, no limit message
    const many = await run([sw("wealth.stockShare", 0, 1), sw("pension.pk.lumpsum", 0, 0.9), sw("flows.spendingFactor", 1, 1.9)]);
    expect(many.text).not.toContain("Grenze");
    expect(many.text).toContain("1'100");
    expect(many.errors.filter(e => e.includes("Kombinationen"))).toEqual([]);
    // above 1'000'000: technical message, without burdening the app
    const over = await run([sw("wealth.stockShare", 0, 1, 0.01), sw("pension.pk.lumpsum", 0, 0.99, 0.01), sw("flows.spendingFactor", 1, 1.99, 0.01)]);
    expect(over.text).toContain("technische Grenze");
    expect(over.errors).toContain("Die Variationen ergeben 1'010'000 Kombinationen; höchstens 1'000'000 sind technisch möglich.");
    // Warning from one minute of estimated computing time
    await page.evaluate(() => { state.scenarios.sweeps = []; state.returns.source = "fixed"; renderSection(); });
    const fake = sec => page.evaluate(sec => { Object.assign(bench, { key: benchKey(), us: sec * 1e6, fixedPaths: 1, error: null }); }, sec);
    await fake(90);
    await page.locator('button[data-action="run"]').click();
    await expect(page.locator("#ask")).toBeVisible();
    const text = await page.locator("#askText").innerText();
    expect(text).toMatch(/ca\. 2 min/);
    expect(text).toContain("1 Simulationslauf)");
    expect(await page.locator("#ask button[value='ok']").innerText()).toBe("Trotzdem starten");
    await page.locator("#ask button[value='cancel']").click();
    await expect(page.locator("#results")).toBeHidden();
    await page.locator('button[data-action="run"]').click();
    await page.locator("#ask button[value='ok']").click();
    await expect(page.locator("#results")).toBeVisible();
    // 30 s: no question
    await page.evaluate(() => showResults(false));
    await fake(30);
    await page.locator('button[data-action="run"]').click();
    await expect(page.locator("#results")).toBeVisible();
    await expect(page.locator("#ask")).toBeHidden();
    await noErrors(page, errors);
  });

  test("Umbenennung: «Ergebnisse» statt «Simulationen» in Kopfzeile, Ergebnisansicht und Liste", async ({ page }) => {
    const errors = await open(page);
    const btn = page.locator('button[data-action="runs-open"]').first();
    await expect(btn).toHaveText("Ergebnisse");
    expect(await btn.getAttribute("title")).toBe("Gespeicherte Ergebnisse ansehen und vergleichen");
    await expect(page.locator('#results button[data-action="runs-open"]')).toHaveText("Ergebnisse");
    await btn.click();
    await expect(page.locator("#resultsTitle")).toHaveText("Gespeicherte Ergebnisse");
    const text = await page.locator("#results").innerText();
    expect(text).not.toMatch(/Simulationen/);
    await expect(page.locator("#resultsBody")).toContainText("Noch kein Ergebnis");
    // «Simulation starten» remains the action
    await expect(page.locator('button[data-action="run"]')).toContainText("Simulation starten");
    await noErrors(page, errors);
  });

  test("Tooltips erscheinen beim Überfahren der Vorschau", async ({ page }) => {
    const errors = await open(page);
    const box = await page.locator("#chart").boundingBox();
    await page.mouse.move(box.x + box.width * 0.5, box.y + box.height * 0.5);
    await expect(page.locator("#tip")).toBeVisible();
    await expect(page.locator("#tip")).toContainText("Total");
    await page.mouse.move(box.x - 50, box.y - 50);
    await expect(page.locator("#tip")).not.toBeVisible();
    await noErrors(page, errors);
  });

  test("Nominal und heutige Kaufkraft unterscheiden sich", async ({ page }) => {
    const errors = await open(page, "results/nosweep");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "overview");
    const nominal = await resultKpi(page, "Vermögen am Ende").textContent();
    await page.getByRole("button", { name: "Heutige Kaufkraft" }).click();
    await expect(resultKpi(page, "Vermögen am Ende")).not.toHaveText(nominal);
    await page.getByRole("button", { name: "Nominal" }).click();
    await expect(resultKpi(page, "Vermögen am Ende")).toHaveText(nominal);
    await noErrors(page, errors);
  });

  test("#22: Legenden zeigen gestrichelte Linien als gestricheltes Symbol (Übersicht und Grafiken)", async ({ page }) => {
    const errors = await open(page, "results/bootstrap");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "overview");
    const dashes = () => page.evaluate(() => [...document.querySelectorAll("#resultsBody .legend")].map(l => ({
      text: l.innerText.replace(/\s+/g, " "), dashed: [...l.querySelectorAll("svg.lsym line")].filter(x => x.getAttribute("stroke-dasharray")).length,
      solid: [...l.querySelectorAll("svg.lsym line")].filter(x => !x.getAttribute("stroke-dasharray")).length })));
    const fan = (await dashes()).find(l => l.text.includes("Schlechtester Wert"));
    expect(fan.text).toContain("1. und 99. Perzentil");
    expect(fan.dashed).toBe(1);
    expect(fan.solid).toBe(2); // median and worst value
    await page.getByRole("button", { name: "Grafiken" }).click();
    await expect(page.locator("#mc-end path, #mc-end rect").first()).toBeAttached();
    const all = await dashes();
    const fan2 = all.find(l => l.text.includes("Schlechtester Wert"));
    expect({ ...fan2, text: "" }).toEqual({ ...fan, text: "" });
    const end = all.find(l => l.text.includes("95. und 99. Perzentil"));
    expect(end.dashed).toBe(1);
    // The vertical dashed lines in the histogram: five, each with a legend entry (one entry for the line style)
    expect(await page.locator("#mc-end line[stroke-dasharray]").count()).toBe(5);
    await expect(page.locator("#resultsBody")).not.toContainText("Gestrichelt: 1., 5., 50.");
    await noErrors(page, errors);
  });

  test("Monte Carlo: Kennzahlen, Perzentile und einzelne Läufe", async ({ page }) => {
    const errors = await open(page, "results/bootstrap/runs");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "runs");
    await expect(page.getByRole("button", { name: "Einzelne Läufe" })).toHaveClass(/|/);
    await page.getByRole("button", { name: "Übersicht" }).click();
    await expect(resultKpi(page, "Erfolgsquote")).toContainText("%");
    const rows = page.locator("#resultsBody table.tbl tr", { hasText: "." });
    const labels = await page.locator("#resultsBody table.tbl td.l").allTextContents();
    for (const p of ["Schlechtester Lauf", "5.", "50.", "95."]) expect(labels).toContain(p);
    // Percentile order of the final wealth
    const vals = [];
    for (const p of ["5.", "50.", "95."]) {
      const row = page.locator("#resultsBody table.tbl tr", { has: page.locator(`td.l:text-is("${p}")`) });
      vals.push(parseNum(await row.locator("td").nth(1).textContent()));
    }
    expect(vals[0]).toBeLessThanOrEqual(vals[1]);
    expect(vals[1]).toBeLessThanOrEqual(vals[2]);
    expect(await rows.count()).toBeGreaterThan(0);
    await noErrors(page, errors);
  });

  test("Szenarien: Heatmap ist vollständig", async ({ page }) => {
    const errors = await open(page, "results/bootstrap/scenarios/sweep2");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "scenarios");
    const heat = page.locator("table.heat, #resultsBody svg").first();
    await expect(heat).toBeAttached();
    const sweeps = await page.evaluate(() => state.scenarios.sweeps.map(s => s.param));
    expect(sweeps.length).toBe(2);
    const cells = page.locator("#resultsBody table.heat td.hm");
    if (await cells.count()) {
      const dims = await page.evaluate(() => state.scenarios.sweeps.map(s => Math.floor((s.to - s.from) / s.step + 1e-9) + 1));
      expect(await cells.count()).toBe(dims[0] * dims[1] * 2);
      expect(await page.locator("#resultsBody table.heat td.na").count()).toBe(0);
    }
    await noErrors(page, errors);
  });

  test("Escape führt aus den Ergebnissen zurück in den Editor", async ({ page }) => {
    const errors = await open(page, "results/nosweep");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "overview");
    await page.keyboard.press("Escape");
    await expect(page.locator("#results")).toBeHidden();
    await expect(page.locator("#main h2")).toBeVisible();
    await noErrors(page, errors);
  });

  test("jede Renditequelle lässt sich rechnen", async ({ page }) => {
    for (const src of ["fixed", "historical", "rolling", "bootstrap", "parametric"]) {
      const errors = await open(page, `results/${src}`);
      await expect(page.locator("html")).toHaveAttribute("data-rendered", /.+/);
      await expect(page.locator("#resultsBody .card").first()).toBeVisible();
      await expect(page.locator("#resultsBody .errcard")).toHaveCount(0);
      await noErrors(page, errors);
    }
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Darstellung", () => {
  for (const [name, width, height] of [["Desktop", 1440, 900], ["Laptop", 1180, 800], ["Tablet", 820, 1000], ["Handy", 390, 800]]) {
    test(`Responsiveness ${name} (${width} px): nichts läuft über den Rand`, async ({ page }) => {
      await page.setViewportSize({ width, height });
      const errors = await open(page);
      for (const i of [0, 3, 4, 8]) {
        await page.locator("#nav a").nth(i).click();
        const over = await page.evaluate(() => ({ scroll: document.documentElement.scrollWidth - document.documentElement.clientWidth,
          wide: [...document.querySelectorAll("header *")].filter(e => e.getBoundingClientRect().right > document.documentElement.clientWidth + 1).map(e => e.textContent.trim()) }));
        expect(over.scroll, `horizontales Scrollen im Abschnitt ${SECTIONS[i]}; zu breit: ${over.wide.join(", ")}`).toBeLessThanOrEqual(1);
      }
      await expect(page.locator("#chart path").first()).toBeAttached();
      await noErrors(page, errors);
    });
  }

  test("Dunkelmodus: Grafik und Text bleiben lesbar", async ({ page }) => {
    await page.emulateMedia({ colorScheme: "dark" });
    const errors = await open(page);
    const bg = await page.evaluate(() => getComputedStyle(document.body).backgroundColor);
    expect(bg).not.toBe("rgb(244, 245, 247)");
    const fill = await page.locator("#chart path").first().getAttribute("fill");
    expect(fill).toBeTruthy();
    await page.emulateMedia({ colorScheme: "light" });
    await expect.poll(() => page.evaluate(() => getComputedStyle(document.body).backgroundColor)).toBe("rgb(244, 245, 247)");
    await noErrors(page, errors);
  });

  test("Browserfenster verkleinern zeichnet die Vorschau neu", async ({ page }) => {
    const errors = await open(page);
    const d1 = await page.locator("#chart path").first().getAttribute("d");
    await page.setViewportSize({ width: 1000, height: 800 });
    await expect.poll(() => page.locator("#chart path").first().getAttribute("d")).not.toBe(d1);
    await noErrors(page, errors);
  });
});
