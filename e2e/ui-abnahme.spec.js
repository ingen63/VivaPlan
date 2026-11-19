// User interface tests of VivaPlan in the browser: additional acceptance tests, Monte Carlo, preview. Every test starts with the example plan.
const { test, expect } = require("@playwright/test");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { MOD, SECTIONS, open, noErrors, field, previewIdle, nav, kpiValue, resultKpi, parseNum, setOpt, msRow, welcome, steps, helpTitle, prefs } = require("./helpers");

// ---------------------------------------------------------------------------------------------
// Acceptance criteria that had no test of their own before (acceptance list docs/ABNAHME.md, column «Test»)
test.describe("Additional acceptance tests", () => {
  const setThreeBuckets = (strategy) => page => page.evaluate(async strategy => {
    state.returns.source = "fixed"; state.scenarios.sweeps = [];
    state.withdrawal.strategy = strategy;
    if (strategy === "three_buckets") state.withdrawal.buckets = { cashYears: 2, bondYears: 1, refillThreshold: -1, reserveMonths: 3 };
    lastRun = await Engine.call("run_simulation", { plan: state, path: null });
    resetVariation(); resultTab = "overview"; showResults(true); renderResults(lastRun);
  }, strategy);

  test("#8/AK4, #8/AK5, #8/AK6: total taxes stays visible, computation unchanged, toggling moves no column", async ({ page }) => {
    const errors = await open(page, "results/nosweep");
    await expect(resultKpi(page, "Steuern total")).toBeVisible();
    const kpi = await resultKpi(page, "Steuern total").textContent();
    await page.getByRole("button", { name: "Jahreswerte" }).click();
    // Content of the table without the tax columns (column headers with class tx) and size of the result
    const snap = () => page.evaluate(() => {
      const t = document.querySelector("#resultsBody table.yt");
      const heads = [...t.rows[0].cells];
      const keep = heads.map((c, i) => (c.classList.contains("tx") ? -1 : i)).filter(i => i >= 0);
      return { head: keep.map(i => heads[i].innerText.trim()), rows: [...t.rows].slice(1).map(r => keep.map(i => r.cells[i]?.innerText.trim())), run: JSON.stringify(lastRun).length };
    });
    const closed = await snap();
    await page.locator('button[data-action="tax-cols"]').click();
    await expect(page.locator("#resultsBody table.yt th.tx")).toHaveCount(5);
    const open5 = await snap();
    expect(open5.head).toEqual(closed.head);          // AK6: the same columns in the same order
    expect(open5.rows).toEqual(closed.rows);          // AK6: the same values in them
    expect(open5.run).toBe(closed.run);               // AK5: the result is unchanged
    await page.getByRole("button", { name: "Übersicht" }).click();
    await expect(resultKpi(page, "Steuern total")).toHaveText(kpi);   // AK4: also with expanded columns
    await noErrors(page, errors);
  });

  test("#109/AK1, AK2, AK3: tax of the tax year in the row of the year, total is the sum, capital and real estate gains tax in the year of the withdrawal", async ({ page }) => {
    const errors = await open(page, "results/nosweep");
    await page.getByRole("button", { name: "Jahreswerte" }).click();
    await page.locator('button[data-action="tax-cols"]').click();
    await expect(page.locator("#resultsBody table.yt th.tx")).toHaveCount(5);
    const t = await page.evaluate(() => {
      const num = s => Number(s.replace(/[’'\s\u00a0\u202f]/g, "").replace("−", "-")) || 0;
      const head = [...document.querySelector("#resultsBody table.yt").rows[0].cells];
      const first = head.findIndex(c => c.classList.contains("tx"));
      const recs = lastRun.single.months ?? lastRun.single.records;
      const rows = [...document.querySelectorAll("#resultsBody table.yt tr")].slice(1).filter(r => /^\d{4}/.test(r.cells[0].innerText.trim()));
      return rows.map(r => {
        const y = r.cells[0].innerText.trim().slice(0, 4), cells = [...r.cells].slice(first, first + 5).map(c => num(c.innerText));
        const rs = recs.filter(m => m.month.startsWith(y)), sum = f => rs.reduce((a, m) => a + f(m), 0);
        return { y, cells, ai: sum(m => m.assessedIncome), aw: sum(m => m.assessedWealth), kap: sum(m => m.taxCapital), gain: sum(m => m.taxGain) };
      });
    });
    expect(t.length).toBeGreaterThan(5);
    let positive = 0;
    for (const r of t) {
      const [ink, verm, kap, gain, total] = r.cells;
      expect(Math.abs(ink - r.ai)).toBeLessThanOrEqual(1);       // AK1: assessed in the tax year of the row
      expect(Math.abs(verm - r.aw)).toBeLessThanOrEqual(1);
      expect(Math.abs(kap - r.kap)).toBeLessThanOrEqual(1);      // AK3: in the year of the withdrawal
      expect(Math.abs(gain - r.gain)).toBeLessThanOrEqual(1);
      expect(Math.abs(total - (ink + verm + kap + gain))).toBeLessThanOrEqual(2);   // AK2
      if (ink > 0) positive++;
    }
    expect(positive).toBeGreaterThan(0);
    await noErrors(page, errors);
  });

  test("#9/AK3, #9/AK4, #10/AK2, #10/AK3: expandable rows only in years with refill, event list keeps everything except the 3-bucket bookings", async ({ page }) => {
    const errors = await open(page);
    // 3-bucket: exactly one toggle per year with refill
    await setThreeBuckets("three_buckets")(page);
    const t = await page.evaluate(() => {
      const evs = lastRun.single.events, shown = evs.filter(e => !isBucketBooking(e));
      return { years: new Set(lastRun.single.bucketRefills.map(r => r.month.slice(0, 4))).size, all: evs.length, shown: shown.length, texts: shown.map(e => e.text), booked: evs.filter(e => isBucketBooking(e)).map(e => e.text) };
    });
    const items = page.locator("#resultsBody .evlist li");
    await expect(items).toHaveCount(t.shown);                       // #10/AK2: everything except the bookings is in the list
    expect(t.shown).toBeLessThan(t.all);
    const listed = await items.locator("span:nth-child(2)").allTextContents();
    expect(listed).toEqual(t.texts);
    expect(listed.some(x => x.startsWith("Bezug Pensionskasse"))).toBe(true);
    expect(listed.some(x => x.includes("Neues Auto"))).toBe(true);
    for (const b of t.booked) expect(listed).not.toContain(b);
    await page.getByRole("button", { name: "Jahreswerte" }).click();
    const rows = await page.locator("#resultsBody table.yt tr").count();
    const toggles = await page.locator('button[data-action="bk-toggle"]').count();
    expect(toggles).toBe(t.years);                                  // #9/AK3: only years with refill
    expect(rows - 1).toBeGreaterThan(toggles);
    // Without 3-bucket: no expandable rows, the event list is complete as before
    await setThreeBuckets("standard")(page);
    const all = await page.evaluate(() => lastRun.single.events.length);
    await expect(page.locator("#resultsBody .evlist li")).toHaveCount(all);   // #10/AK3
    await page.getByRole("button", { name: "Jahreswerte" }).click();
    await expect(page.locator('button[data-action="bk-toggle"]')).toHaveCount(0);   // #9/AK4
    await expect(page.locator("#resultsBody tr.bk")).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("#9/AK6: when switching the single run the table shows the derivation of the new run", async ({ page }) => {
    const errors = await open(page);
    await page.evaluate(async () => {
      state.returns.source = "bootstrap"; state.returns.bootstrap.paths = 60; state.scenarios.sweeps = [];
      state.withdrawal.strategy = "three_buckets";
      state.withdrawal.buckets = { cashYears: 2, bondYears: 1, refillThreshold: 0.02, reserveMonths: 3 };
      lastRun = await Engine.call("run_simulation", { plan: state, path: null });
      resetVariation(); resultTab = "table"; showResults(true); renderResults(lastRun);
    });
    const seen = [];
    for (const p of [5, 50, 95]) {
      await page.locator(`button[data-action="detail"][data-p="${p}"]`).click();
      const expected = await page.evaluate(p => new Set(detailsOf(lastRun).find(([q]) => q === p)[1].bucketRefills.map(r => r.month.slice(0, 4))).size, p);
      await expect(page.locator('button[data-action="bk-toggle"]')).toHaveCount(expected);
      await page.locator('button[data-action="bk-toggle"]').first().click();
      const text = await page.locator("#resultsBody tr.bk").first().innerText();
      const first = await page.evaluate(p => detailsOf(lastRun).find(([q]) => q === p)[1].bucketRefills[0].need, p);
      expect(text.replace(/['’]/g, "")).toContain(String(Math.round(first)));      // Net need of the displayed run
      seen.push(first);
    }
    expect(new Set(seen).size).toBeGreaterThan(1);                  // the three runs do not show the same derivation
    await noErrors(page, errors);
  });

  test("#11/AK3: the parameters in the tooltip match the withdrawal strategy tab", async ({ page }) => {
    const errors = await open(page, "runsflow/mem/del");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "runs");
    await page.evaluate(async () => {
      Runs.memoryOnly();
      state.returns.source = "fixed"; state.scenarios.sweeps = [];
      state.withdrawal.strategy = "three_buckets";
      state.withdrawal.buckets = { cashYears: 3, bondYears: 4, refillThreshold: 0.02, reserveMonths: 3 };
      await rememberRun(await Engine.call("run_simulation", { plan: state, path: null }));
      await openRuns();
    });
    const tip = await page.locator("#runsTbl td.ptip", { hasText: "3-Topf" }).first().getAttribute("data-tip");
    await page.evaluate(() => goto("withdrawal"));
    const val = p => page.locator(`#main input[data-path="withdrawal.buckets.${p}"]`).inputValue();
    expect(tip).toContain(`Topf 1 · Cash: ${await val("cashYears")} J.`);
    expect(tip).toContain(`Topf 2 · Obligationen: ${await val("bondYears")} J.`);
    expect(tip).toContain(`${(await val("refillThreshold")).replace(/\s*%$/, "").replace(",", ".")} %`);
    await noErrors(page, errors);
  });

  test("#12/AK2, #12/AK3, #13/AK5: time series without path and .csv, fixed and parametric returns in gray, planned sale without warning symbol", async ({ page }) => {
    const errors = await open(page, "runsflow/mem/del");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "runs");
    const label = await page.evaluate(async () => {
      Runs.memoryOnly();
      state.returns.source = "historical";
      const label = seriesLabel();
      const base = { created: "2021-01-01T00:00:00.000Z", note: "", strategy: "standard", kind: "batch", runs: 50, successRate: 0.9, end50: 2_500_000, end5: 1_200_000 };
      const add = (id, extra) => Runs.add({ id, name: id, ...base, ...extra }, { plan: JSON.parse(JSON.stringify(state)), result: lastRun });
      await add("hist", { source: "historical", seriesName: label, emergencySaleRuns: 0 });
      await add("fix", { source: "fixed", seriesName: null, emergencySaleRuns: 0 });
      await add("par", { source: "parametric", seriesName: null, emergencySaleRuns: 0 });
      // planned sale (affordability, lump-sum payment) in all runs, but no emergency sale
      await add("geplant", { source: "bootstrap", seriesName: label, emergencySaleRuns: 0, affordabilitySaleRuns: 50, lumpSumRuns: 50 });
      await openRuns();
      return label;
    });
    expect(label).toBeTruthy();
    expect(label).not.toMatch(/[\\/]|\.csv/i);                          // AK2: name without path and extension
    const row = id => page.locator("#runsTbl tr", { has: page.locator(`input.runedit[value="${id}"]`) });
    await expect(row("hist").locator("td").nth(4)).toHaveText(label);
    for (const [id, text] of [["fix", "feste Renditen"], ["par", "parametrisch"]]) {
      const cell = row(id).locator("span.muted", { hasText: text });
      await expect(cell).toHaveCount(1);                                   // AK3
      const colors = await cell.evaluate(el => [getComputedStyle(el).color, getComputedStyle(el.closest("td")).color]);
      expect(colors[0]).not.toBe(colors[1]);                               // gray, not in the text color
    }
    await expect(row("geplant").locator(".hwarn")).toHaveCount(0);        // #13/AK5
    await noErrors(page, errors);
  });

  test("#23/AK5: the key figure card for emergency sales shows runs and share as before", async ({ page }) => {
    const errors = await open(page, "results/bootstrap");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "overview");
    const s = await page.evaluate(() => ({ n: lastRun.batch.summary.emergencySaleRuns, runs: lastRun.batch.summary.runs, text: fmtInt(lastRun.batch.summary.emergencySaleRuns), pct: pctTxt(lastRun.batch.summary.emergencySaleRuns / lastRun.batch.summary.runs) }));
    const card = page.locator("#resultsBody .kpi:has(.k:text-is('Notverkäufe'))");
    await expect(card.locator(".v")).toHaveText(`${s.text} Läufe`);
    await expect(card).toContainText(`${s.pct} der Läufe`);
    await noErrors(page, errors);
  });

  test("#24/AK5, #24/AK6: notice on different final wealth; single run with variations as before", async ({ page }) => {
    const errors = await open(page);
    const out = await page.evaluate(async () => {
      state.returns.source = "fixed"; state.returns.bootstrap.paths = 100;
      state.scenarios.sweeps = [{ param: "wealth.stockShare", from: 0.3, to: 0.6, step: 0.3 }, { param: "pension.pk.lumpsum", from: 0.2, to: 0.6, step: 0.4 }];
      // Single run (AK6): the selected level shows the run of this level, as the scenario table names it
      lastRun = await Engine.call("run_simulation", { plan: state, path: null });
      resetVariation(); resultTab = "runs"; showResults(true); renderResults(lastRun);
      const selects = document.querySelectorAll('select[data-action="var-sel"]').length;
      const lvl = lastRun.scenarios.results.find(x => x.values[0] === 0.3 && x.values[1] === 0.6);
      await pickVariation([0.3, 0.6]);
      const shown = singleOf(lastRun).summary.endTotal;
      await pickVariation([null, null]);
      const back = singleOf(lastRun) === lastRun.single;
      // Different final wealth (AK5): as if something had changed since saving
      state.returns.source = "bootstrap";
      lastRun = await Engine.call("run_simulation", { plan: state, path: null });
      resetVariation(); resultTab = "runs"; showResults(true); renderResults(lastRun);
      await pickVariation([0.3, 0.6]);
      const clean = document.querySelector("#resultsPick").textContent.includes("weicht von der Szenarientabelle ab");
      const level = lastRun.scenarios.results.find(x => x.values[0] === 0.3 && x.values[1] === 0.6);
      level.percentiles.find(p => p.percent === 50).endTotal += 50_000;
      resetVariation(); renderResults(lastRun);
      await pickVariation([0.3, 0.6]);
      const hint = document.querySelector("#resultsPick").textContent;
      return { selects, shown, p50: lvl.p50, back, clean, hint };
    });
    expect(out.selects).toBe(2);
    expect(out.shown).toBeCloseTo(out.p50, 4);
    expect(out.back).toBe(true);
    expect(out.clean).toBe(false);                                   // no difference, no notice
    expect(out.hint).toContain("weicht von der Szenarientabelle ab");
    await noErrors(page, errors);
  });

  test("#51/AK3: a new plan has the AHV start and the pension fund withdrawal at age 65", async ({ page }) => {
    const errors = await open(page);
    const ms = await page.evaluate(() => {
      const p = blankPlan(JSON.parse(JSON.stringify(state)));
      return p.milestones.filter(m => m.id === "ahv" || m.id === "retire").map(m => ({ id: m.id, name: m.name, label: msLabel(m), kind: m.kind, years: m.years }));
    });
    expect(ms).toEqual(expect.arrayContaining([
      // #170/AK8: no stored name; the name comes from the text table by the id
      { id: "retire", name: undefined, label: "Bezug Pensionskasse", kind: "age", years: 65 },
      { id: "ahv", name: undefined, label: "AHV-Bezug", kind: "age", years: 65 },
    ]));
    expect(ms).toHaveLength(2);
    // and it appears like this in the user interface: «Neue Planung» shows both milestones with age 65
    await page.evaluate(() => { state = blankPlan(JSON.parse(JSON.stringify(state))); goto("milestones"); });
    for (const id of ["retire", "ahv"]) {
      await expect((await msRow(page, id)).locator("input[data-type=years]")).toHaveValue("65");
    }
    await noErrors(page, errors);
  });

  test("#25/AK7: settings do not change the plan; if it lies outside the new limits, the check reports it", async ({ page }) => {
    const errors = await open(page);
    await page.evaluate(() => { state.pillar3a[0].contribution = 7500; renderSection(); });
    await expect(page.locator("#val li.err", { hasText: "Die Einzahlung muss" })).toHaveCount(1);   // above 7'258
    const before = await page.evaluate(() => JSON.stringify(state));
    const setMax = async v => {
      await page.locator('header button[data-action="settings"]').click();
      await page.locator("#set-pillar3aMax").fill(v);
      await page.locator("#settingsSave").click();
      await expect(page.locator("#settings")).toBeHidden();
    };
    await setMax("8000");
    expect(await page.evaluate(() => JSON.stringify(state))).toBe(before);                           // plan unchanged
    await expect(page.locator("#val li.err", { hasText: "Die Einzahlung muss" })).toHaveCount(0);   // within the new limit
    await setMax("5000");
    expect(await page.evaluate(() => JSON.stringify(state))).toBe(before);
    await expect(page.locator("#val li.err", { hasText: "Maximalbetrag von 5'000" })).toHaveCount(1);
    await setMax("7258");                                                                            // restore the default
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
// #100: stock parameters per stock index; #101: bonds and inflation with correlation per index
test.describe("#100/#101: parametric Monte Carlo", () => {
  const KEYS = ["mu", "sigma", "df", "jumpLambda", "jumpMu", "jumpSigma"];
  const params = page => page.evaluate(k => Object.fromEntries(k.map(x => [x, state.returns.parametric[x]])), KEYS);
  const suggestion = page => page.evaluate(k => Object.fromEntries(k.map(x => [x, seriesInfo.parametric.stocks[x]])), KEYS);
  /** Select the stock index under «Vermögen & Anlage» and go back to the simulation. */
  async function chooseStocks(page, value, bonds) {
    await nav(page, "Vermögen & Anlage");
    await page.locator('select[data-path="returns.strategy.stocks"]').selectOption(value);
    if (bonds) await page.locator('select[data-path="returns.strategy.bonds"]').selectOption(bonds);
    await nav(page, "Simulation");
    await expect(page.locator("#main h2")).toHaveText(/^Simulation/);
    await page.waitForFunction(v => seriesInfo?.parametric && state.returns.strategy.stocks === v && seriesInfoRef === seriesRef(), value);
  }
  async function parametric(page) {
    const errors = await open(page, "returns");
    await page.locator('#main .choice[data-value="parametric"]').click();
    await page.waitForFunction(() => seriesInfo?.parametric?.stocks);
    return errors;
  }

  test("#100/AK4: new plan, switching the stock index sets the suggested values of the new index", async ({ page }) => {
    const errors = await parametric(page);
    expect(await page.evaluate(() => state.returns.parametric.auto)).toBe(true);
    await chooseStocks(page, "chJst");
    const ch = await params(page);
    expect(ch).toEqual(await suggestion(page));
    await expect(page.locator('#main [data-computed="pauto"]')).toContainText("Die Werte folgen dem Aktienindex Schweiz (JST)");
    await expect(field(page, "returns.parametric.mu")).toHaveValue(String(+(ch.mu * 100).toFixed(4)));
    await chooseStocks(page, "sp500");
    const sp = await params(page);
    expect(sp).toEqual(await suggestion(page));
    expect(sp.mu).not.toBe(ch.mu);
    // No suggestions next to the fields as long as the values follow the index
    expect((await page.locator('#main [data-computed="psug"]').allInnerTexts()).join("")).toBe("");
    await noErrors(page, errors);
  });

  test("#100/AK5: own values remain when switching the index, the suggested values next to them; button to take over the suggested values", async ({ page }) => {
    const errors = await parametric(page);
    await chooseStocks(page, "worldJst");
    const mu = field(page, "returns.parametric.mu");
    await mu.fill("5");
    await mu.press("Tab");
    expect(await page.evaluate(() => [state.returns.parametric.auto, state.returns.parametric.mu])).toEqual([false, 0.05]);
    await chooseStocks(page, "chJst");
    const own = await params(page), sug = await suggestion(page);
    expect(own.mu).toBe(0.05);
    expect(own.sigma).not.toBe(sug.sigma); // Volatility of «Welt (JST)», not of «Schweiz (JST)»
    await expect(page.locator('#main [data-computed="pauto"]')).toContainText("Eigene Werte");
    const hints = await page.locator('#main [data-computed="psug"]').allInnerTexts();
    expect(hints).toHaveLength(6);
    expect(hints[0]).toBe(`Vorschlag: ${(sug.mu * 100).toLocaleString("de-CH", { maximumFractionDigits: 3 })} %`);
    await page.locator('#main [data-action="param-suggest"]').click();
    expect(await page.evaluate(() => state.returns.parametric.auto)).toBe(true);
    expect(await params(page)).toEqual(sug);
    await noErrors(page, errors);
  });

  test("#101/AK4: correlation matrix in the simulation tab, depending on stocks and bonds", async ({ page }) => {
    const errors = await parametric(page);
    const table = page.locator("#paramJoint table");
    await chooseStocks(page, "worldJst", "ch");
    await expect(table).toBeVisible();
    const a = await table.innerText();
    expect(await page.locator("#main").innerText()).not.toContain("Obligationen, Inflation und SARON pro Jahr");
    await chooseStocks(page, "worldJst", "usa");
    await expect(table).not.toHaveText(a);
    const b = await table.innerText();
    const j = await page.evaluate(() => seriesInfo.parametric.joint);
    expect(b).toContain(j.corr[0][1].toLocaleString("de-CH", { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
    await chooseStocks(page, "chJst", "usa");
    await expect(table).not.toHaveText(b);
    await noErrors(page, errors);
  });

  test("#101/AK6: notice when opening an older plan with parametric Monte Carlo, own values remain", async ({ page }) => {
    const errors = await open(page);
    const plan = await page.evaluate(() => {
      const s = JSON.parse(JSON.stringify(state));
      s.returns.source = "parametric";
      s.returns.parametric = { paths: 100, mu: 0.05, sigma: 0.12, df: 6, jumpLambda: 0.05, jumpMu: -0.3, jumpSigma: 0.15, seed: 42 };
      return s;
    });
    const file = path.join(os.tmpdir(), `fea79-${process.pid}.vplan`);
    fs.writeFileSync(file, JSON.stringify(plan));
    await page.locator("#fileInput").setInputFiles(file);
    await expect(page.locator("#ask")).toBeVisible();
    await expect(page.locator("#askText")).toContainText("Monte Carlo parametrisch rechnet neu auch Obligationen und Inflation zufällig");
    await page.locator("#ask button[value='ok']").click();
    expect(await page.evaluate(() => [state.returns.parametric.auto, state.returns.parametric.mu])).toEqual([false, 0.05]);
    fs.rmSync(file);
    await noErrors(page, errors);
  });
});

// #132: faster random draws of the parametric model; older plans get a notice when opened
test.describe("Parametric Monte Carlo computes faster (#132)", () => {
  test("AK7: notice when opening an older plan with parametric Monte Carlo; new plans carry the flag", async ({ page }) => {
    const errors = await open(page);
    expect(await page.evaluate(() => state.returns.parametric.sampler)).toBe(1);
    const plan = await page.evaluate(() => {
      const s = JSON.parse(JSON.stringify(state));
      s.returns.source = "parametric";
      s.returns.parametric = { paths: 100, mu: 0.05, sigma: 0.12, df: 6, jumpLambda: 0.05, jumpMu: -0.3, jumpSigma: 0.15, seed: 42, auto: false };
      return s;
    });
    const file = path.join(os.tmpdir(), `issue132-${process.pid}.vplan`);
    fs.writeFileSync(file, JSON.stringify(plan));
    await page.locator("#fileInput").setInputFiles(file);
    await expect(page.locator("#ask")).toBeVisible();
    await expect(page.locator("#askText")).toContainText("Monte Carlo parametrisch erzeugt die Zufallspfade mit einem schnelleren Verfahren");
    await page.locator("#ask button[value='ok']").click();
    expect(await page.evaluate(() => [state.returns.parametric.sampler, state.returns.parametric.mu])).toEqual([1, 0.05]);
    fs.rmSync(file);
    await noErrors(page, errors);
  });
});

test.describe("Question about saving before the simulation (#110)", () => {
  /** In the browser the question is only active with `askSaveDev`; the plan is marked as unsaved. */
  const prepare = (page, dev) => page.evaluate(dev => { Pref.set("askSaveDev", dev); Pref.set("askSaveBeforeRun", true); markDirty(); }, dev);

  test("#110/AK1, AK2, AK3, AK5: question before the simulation with unsaved changes", async ({ page }) => {
    const errors = await open(page);
    // AK3: no changes, no question
    await page.evaluate(() => { Pref.set("askSaveDev", true); runSimulation("overview", { noConfirm: true }); });
    await expect(page.locator("#ask")).not.toHaveAttribute("open", "");
    await expect(page.locator("#resultsBody")).toBeVisible();
    await page.evaluate(() => showResults(false));
    // AK1, AK2: cancel starts nothing, the plan stays unsaved
    await prepare(page, true);
    await page.evaluate(() => { window.__ran = runSimulation("overview", { noConfirm: true }); });
    await expect(page.locator("#ask")).toHaveAttribute("open", "");
    await expect(page.locator("#askText")).toContainText("ungespeicherte Änderungen");
    await expect(page.locator('#ask button[value="discard"]')).toHaveText("Ohne Speichern simulieren");
    await page.locator('#ask button[value="cancel"]').click();
    await page.evaluate(() => window.__ran);
    await expect(page.locator("#results")).toBeHidden();
    expect(await page.evaluate(() => dirty)).toBe(true);
    // AK2: «Ohne Speichern simulieren» computes and leaves the plan unsaved
    await page.evaluate(() => { window.__ran = runSimulation("overview", { noConfirm: true }); });
    await page.locator('#ask button[value="discard"]').click();
    await expect(page.locator("#resultsBody")).toBeVisible();
    expect(await page.evaluate(() => dirty)).toBe(true);
    // AK5: switched off, no question
    await page.evaluate(() => { showResults(false); Pref.set("askSaveBeforeRun", false); runSimulation("overview", { noConfirm: true }); });
    await expect(page.locator("#ask")).not.toHaveAttribute("open", "");
    await expect(page.locator("#resultsBody")).toBeVisible();
    await noErrors(page, errors);
  });

  test("#110/AK1, AK2, AK4: save saves first and starts afterwards; new plan without a file name", async ({ page }) => {
    const errors = await open(page);
    await prepare(page, true);
    await page.evaluate(() => { window.__ran = runSimulation("overview", { noConfirm: true }); });
    // AK1: three buttons; AK4: a plan without a file name says so in the question (in the browser without its own dialog: download)
    await expect(page.locator('#ask button[value="ok"]')).toHaveText("Speichern");
    await expect(page.locator('#ask button[value="discard"]')).toHaveText("Ohne Speichern simulieren");
    await expect(page.locator('#ask button[value="cancel"]')).toBeVisible();
    await expect(page.locator("#askText")).toContainText("noch keinen Dateinamen");
    const download = page.waitForEvent("download");
    await page.locator('#ask button[value="ok"]').click();
    expect((await download).suggestedFilename()).toMatch(/\.vplan$/);
    await page.evaluate(() => window.__ran);
    await expect(page.locator("#resultsBody")).toBeVisible();
    expect(await page.evaluate(() => dirty)).toBe(false);
    await noErrors(page, errors);
  });

  test("#110/AK5: the setting is in the settings dialog and is kept", async ({ page }) => {
    const errors = await open(page);
    await page.evaluate(() => showSettings());
    const box = page.locator("#set-askSave");
    await expect(box).toBeChecked();
    await box.uncheck();
    await page.locator("#settingsSave").click();
    expect(await page.evaluate(() => Pref.get("askSaveBeforeRun", true))).toBe(false);
    await page.evaluate(() => showSettings());
    await expect(page.locator("#set-askSave")).not.toBeChecked();
    await page.locator("#settingsDefaults").click();
    await page.locator('#ask button[value="ok"]').click();
    await expect(page.locator("#set-askSave")).toBeChecked();
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Test access for load-independent tests (#111)", () => {
  test("#111/AK1: #results computes bootstrap with 500 paths and seed 42; two runs yield the same key figures", async ({ page }) => {
    const kpis = async () => {
      await open(page, "results/bootstrap/nosweep");
      await expect(page.locator("html")).toHaveAttribute("data-rendered", "overview");
      return page.evaluate(() => ({
        paths: state.returns.bootstrap.paths, seed: state.returns.bootstrap.seed,
        pPaths: state.returns.parametric.paths, pSeed: state.returns.parametric.seed,
        kpis: [...document.querySelectorAll("#resultsBody .kpi")].map(k => k.innerText),
      }));
    };
    const a = await kpis();
    expect(a).toMatchObject({ paths: 500, seed: 42, pPaths: 500, pSeed: 42 });
    expect(a.kpis.length).toBeGreaterThan(3);
    expect(a.kpis.find(t => t.includes("Erfolgsquote"))).toMatch(/\d+ %/);
    expect(await kpis()).toEqual(a);
  });

  test("#111/AK2: #nobench does not measure the computing time, shows the estimating message and computes without a question", async ({ page }) => {
    const api = [];
    page.on("request", r => { const u = new URL(r.url()); if (u.pathname.startsWith("/api/")) api.push(u.pathname); });
    await page.clock.install();
    const errors = await open(page, "scenarios"); // open() sets #nobench
    await page.evaluate(() => { setP("flows.spendingFactor", 1.01); renderDerived(); });
    await page.clock.runFor(2000); // delay of the measurement (800 ms) safely over
    await expect(page.locator("#est-scen")).toHaveText("Dauer wird geschätzt…");
    await page.locator('button[data-action="run"]').click();
    await expect(page.locator("#results")).toBeVisible();
    await expect(page.locator("#ask")).toBeHidden();
    expect(api).not.toContain("/api/bench_plan");
    await noErrors(page, errors);
  });

  test("#111/AK2: without #nobench the app measures the computing time (cross-check)", async ({ page }) => {
    const bench = page.waitForRequest(r => new URL(r.url()).pathname === "/api/bench_plan");
    const errors = await open(page, "scenarios", { bench: true });
    await bench;
    await expect(page.locator("#est-scen")).toContainText("Geschätzte Dauer");
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Click into the menu during a preview (#112)", () => {
  /** Presses the mouse button on `loc`, lets the app redraw as when a preview arrives and then releases. */
  const clickDuringPreview = async (page, loc) => {
    await loc.scrollIntoViewIfNeeded();
    const b = await loc.boundingBox();
    await page.mouse.move(b.x + b.width / 2, b.y + b.height / 2);
    await page.mouse.down();
    await page.evaluate(() => renderDerived());
    await page.mouse.up();
  };

  test("#112/AK1, AK2: menu entries stay the same elements; the click opens the section", async ({ page }) => {
    const errors = await open(page);
    await previewIdle(page);
    const same = await page.evaluate(() => {
      const before = [...document.querySelectorAll("#nav a")];
      renderDerived();
      return before.every((a, i) => a.isConnected && a === document.querySelectorAll("#nav a")[i]);
    });
    expect(same).toBe(true);
    await clickDuringPreview(page, page.locator("#nav a", { hasText: "Vermögen & Anlage" }));
    await expect(page.locator("#nav a.active")).toContainText("Vermögen & Anlage");
    expect(await page.evaluate(() => current)).toBe("wealth");
    await expect(page.locator("#nav a.active")).toHaveCount(1);
    await noErrors(page, errors);
  });

  test("#112/AK3: click on a message of the check list while the list is unchanged", async ({ page }) => {
    const errors = await open(page, "wealth");
    await previewIdle(page); // the list after the first preview; it does not change after that
    const item = page.locator("#val li[data-sec]").first();
    await expect(item).toBeVisible();
    const sec = await item.getAttribute("data-sec");
    expect(sec).not.toBe("wealth");
    await clickDuringPreview(page, item);
    expect(await page.evaluate(() => current)).toBe(sec);
    await noErrors(page, errors);
  });

  test("#112/AK4: number of errors at the menu appears and disappears", async ({ page }) => {
    const errors = await open(page);
    const badge = page.locator("#nav a", { hasText: "Haushalt" }).locator(".badge.err");
    await expect(badge).toHaveCount(0);
    await field(page, "project.name").fill("");
    await field(page, "household.persons.0.birth").fill("01.2999");
    await field(page, "household.persons.0.birth").press("Tab");
    await expect(badge).toHaveText(/^\d+$/);
    await field(page, "household.persons.0.birth").fill("01.1960");
    await field(page, "household.persons.0.birth").press("Tab");
    await expect(badge).toHaveCount(0);
    await noErrors(page, errors);
  });
});
