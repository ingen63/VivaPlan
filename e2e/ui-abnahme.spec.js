// Oberflächentests von VivaPlan im Browser: Ergänzende Abnahmetests, Monte Carlo, Vorschau. Jeder Test startet mit der Beispielplanung.
const { test, expect } = require("@playwright/test");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { MOD, SECTIONS, open, noErrors, field, previewIdle, nav, kpiValue, resultKpi, parseNum, setOpt, msRow, welcome, steps, helpTitle, prefs } = require("./helpers");

// ---------------------------------------------------------------------------------------------
// Abnahmekriterien, die vorher keinen eigenen Test hatten (Abnahmeliste docs/ABNAHME.md, Spalte «Test»)
test.describe("Ergänzende Abnahmetests", () => {
  const setThreeBuckets = (strategy) => page => page.evaluate(async strategy => {
    state.returns.source = "fixed"; state.scenarios.sweeps = [];
    state.withdrawal.strategy = strategy;
    if (strategy === "three_buckets") state.withdrawal.buckets = { cashYears: 2, bondYears: 1, refillThreshold: -1, reserveMonths: 3 };
    lastRun = await Engine.call("run_simulation", { plan: state, path: null });
    resetVariation(); resultTab = "overview"; showResults(true); renderResults(lastRun);
  }, strategy);

  test("#8/AK4, #8/AK5, #8/AK6: Steuern total bleibt sichtbar, Berechnung unverändert, Umschalten verschiebt keine Spalte", async ({ page }) => {
    const errors = await open(page, "results/nosweep");
    await expect(resultKpi(page, "Steuern total")).toBeVisible();
    const kpi = await resultKpi(page, "Steuern total").textContent();
    await page.getByRole("button", { name: "Jahreswerte" }).click();
    // Inhalt der Tabelle ohne die Steuerspalten (Spaltenköpfe mit Klasse tx) und Grösse des Ergebnisses
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
    expect(open5.head).toEqual(closed.head);          // AK6: dieselben Spalten in derselben Reihenfolge
    expect(open5.rows).toEqual(closed.rows);          // AK6: dieselben Werte darin
    expect(open5.run).toBe(closed.run);               // AK5: das Ergebnis ist unverändert
    await page.getByRole("button", { name: "Übersicht" }).click();
    await expect(resultKpi(page, "Steuern total")).toHaveText(kpi);   // AK4: auch bei ausgeklappten Spalten
    await noErrors(page, errors);
  });

  test("#109/AK1, AK2, AK3: Steuer des Steuerjahres in der Zeile des Jahres, Total ist die Summe, Kapital- und Grundstückgewinnsteuer im Jahr des Bezugs", async ({ page }) => {
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
      expect(Math.abs(ink - r.ai)).toBeLessThanOrEqual(1);       // AK1: veranlagt im Steuerjahr der Zeile
      expect(Math.abs(verm - r.aw)).toBeLessThanOrEqual(1);
      expect(Math.abs(kap - r.kap)).toBeLessThanOrEqual(1);      // AK3: im Jahr des Bezugs
      expect(Math.abs(gain - r.gain)).toBeLessThanOrEqual(1);
      expect(Math.abs(total - (ink + verm + kap + gain))).toBeLessThanOrEqual(2);   // AK2
      if (ink > 0) positive++;
    }
    expect(positive).toBeGreaterThan(0);
    await noErrors(page, errors);
  });

  test("#9/AK3, #9/AK4, #10/AK2, #10/AK3: Aufklapp-Zeilen nur in Jahren mit Auffüllung, Ereignisliste behält alles ausser den 3-Topf-Buchungen", async ({ page }) => {
    const errors = await open(page);
    // 3-Topf: genau ein Schalter je Jahr mit Auffüllung
    await setThreeBuckets("three_buckets")(page);
    const t = await page.evaluate(() => {
      const evs = lastRun.single.events, shown = evs.filter(e => !isBucketBooking(e));
      return { years: new Set(lastRun.single.bucketRefills.map(r => r.month.slice(0, 4))).size, all: evs.length, shown: shown.length, texts: shown.map(e => e.text), booked: evs.filter(e => isBucketBooking(e)).map(e => e.text) };
    });
    const items = page.locator("#resultsBody .evlist li");
    await expect(items).toHaveCount(t.shown);                       // #10/AK2: alles ausser den Buchungen steht in der Liste
    expect(t.shown).toBeLessThan(t.all);
    const listed = await items.locator("span:nth-child(2)").allTextContents();
    expect(listed).toEqual(t.texts);
    expect(listed.some(x => x.startsWith("Bezug Pensionskasse"))).toBe(true);
    expect(listed.some(x => x.includes("Neues Auto"))).toBe(true);
    for (const b of t.booked) expect(listed).not.toContain(b);
    await page.getByRole("button", { name: "Jahreswerte" }).click();
    const rows = await page.locator("#resultsBody table.yt tr").count();
    const toggles = await page.locator('button[data-action="bk-toggle"]').count();
    expect(toggles).toBe(t.years);                                  // #9/AK3: nur Jahre mit Auffüllung
    expect(rows - 1).toBeGreaterThan(toggles);
    // Ohne 3-Topf: keine Aufklapp-Zeilen, die Ereignisliste ist vollständig wie bisher
    await setThreeBuckets("standard")(page);
    const all = await page.evaluate(() => lastRun.single.events.length);
    await expect(page.locator("#resultsBody .evlist li")).toHaveCount(all);   // #10/AK3
    await page.getByRole("button", { name: "Jahreswerte" }).click();
    await expect(page.locator('button[data-action="bk-toggle"]')).toHaveCount(0);   // #9/AK4
    await expect(page.locator("#resultsBody tr.bk")).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("#9/AK6: beim Wechsel des Einzellaufs zeigt die Tabelle die Herleitung des neuen Laufs", async ({ page }) => {
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
      expect(text.replace(/['’]/g, "")).toContain(String(Math.round(first)));      // Netto-Bedarf des angezeigten Laufs
      seen.push(first);
    }
    expect(new Set(seen).size).toBeGreaterThan(1);                  // die drei Läufe zeigen nicht dieselbe Herleitung
    await noErrors(page, errors);
  });

  test("#11/AK3: die Parameter im Tooltip stimmen mit dem Reiter «Entnahmestrategie» überein", async ({ page }) => {
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

  test("#12/AK2, #12/AK3, #13/AK5: Zeitreihe ohne Pfad und .csv, feste und parametrische Renditen in Grau, geplanter Verkauf ohne Warnsymbol", async ({ page }) => {
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
      // geplanter Verkauf (Tragbarkeit, Einmalzahlung) in allen Läufen, aber kein Notverkauf
      await add("geplant", { source: "bootstrap", seriesName: label, emergencySaleRuns: 0, affordabilitySaleRuns: 50, lumpSumRuns: 50 });
      await openRuns();
      return label;
    });
    expect(label).toBeTruthy();
    expect(label).not.toMatch(/[\\/]|\.csv/i);                          // AK2: Name ohne Pfad und Endung
    const row = id => page.locator("#runsTbl tr", { has: page.locator(`input.runedit[value="${id}"]`) });
    await expect(row("hist").locator("td").nth(4)).toHaveText(label);
    for (const [id, text] of [["fix", "feste Renditen"], ["par", "parametrisch"]]) {
      const cell = row(id).locator("span.muted", { hasText: text });
      await expect(cell).toHaveCount(1);                                   // AK3
      const colors = await cell.evaluate(el => [getComputedStyle(el).color, getComputedStyle(el.closest("td")).color]);
      expect(colors[0]).not.toBe(colors[1]);                               // grau, nicht in der Textfarbe
    }
    await expect(row("geplant").locator(".hwarn")).toHaveCount(0);        // #13/AK5
    await noErrors(page, errors);
  });

  test("#23/AK5: die Kennzahlkarte «Notverkäufe» zeigt Läufe und Anteil wie bisher", async ({ page }) => {
    const errors = await open(page, "results/bootstrap");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "overview");
    const s = await page.evaluate(() => ({ n: lastRun.batch.summary.emergencySaleRuns, runs: lastRun.batch.summary.runs, text: fmtInt(lastRun.batch.summary.emergencySaleRuns), pct: pctTxt(lastRun.batch.summary.emergencySaleRuns / lastRun.batch.summary.runs) }));
    const card = page.locator("#resultsBody .kpi:has(.k:text-is('Notverkäufe'))");
    await expect(card.locator(".v")).toHaveText(`${s.text} Läufe`);
    await expect(card).toContainText(`${s.pct} der Läufe`);
    await noErrors(page, errors);
  });

  test("#24/AK5, #24/AK6: Hinweis bei abweichendem Endvermögen; Einzellauf mit Variationen wie bisher", async ({ page }) => {
    const errors = await open(page);
    const out = await page.evaluate(async () => {
      state.returns.source = "fixed"; state.returns.bootstrap.paths = 100;
      state.scenarios.sweeps = [{ param: "wealth.stockShare", from: 0.3, to: 0.6, step: 0.3 }, { param: "pension.pk.lumpsum", from: 0.2, to: 0.6, step: 0.4 }];
      // Einzellauf (AK6): die gewählte Stufe zeigt den Lauf dieser Stufe, so wie ihn die Szenarientabelle nennt
      lastRun = await Engine.call("run_simulation", { plan: state, path: null });
      resetVariation(); resultTab = "runs"; showResults(true); renderResults(lastRun);
      const selects = document.querySelectorAll('select[data-action="var-sel"]').length;
      const lvl = lastRun.scenarios.results.find(x => x.values[0] === 0.3 && x.values[1] === 0.6);
      await pickVariation([0.3, 0.6]);
      const shown = singleOf(lastRun).summary.endTotal;
      await pickVariation([null, null]);
      const back = singleOf(lastRun) === lastRun.single;
      // Abweichendes Endvermögen (AK5): als hätte sich seit dem Speichern etwas geändert
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
    expect(out.clean).toBe(false);                                   // ohne Abweichung kein Hinweis
    expect(out.hint).toContain("weicht von der Szenarientabelle ab");
    await noErrors(page, errors);
  });

  test("#51/AK3: neue Planung hat «AHV-Bezug» und «Bezug Pensionskasse» mit 65 Jahren", async ({ page }) => {
    const errors = await open(page);
    const ms = await page.evaluate(() => {
      const p = blankPlan(JSON.parse(JSON.stringify(state)));
      return p.milestones.filter(m => m.id === "ahv" || m.id === "retire").map(m => ({ id: m.id, name: m.name, kind: m.kind, years: m.years }));
    });
    expect(ms).toEqual(expect.arrayContaining([
      { id: "retire", name: "Bezug Pensionskasse", kind: "age", years: 65 },
      { id: "ahv", name: "AHV-Bezug", kind: "age", years: 65 },
    ]));
    expect(ms).toHaveLength(2);
    // und sie erscheint so in der Oberfläche: «Neue Planung» zeigt beide Meilensteine mit Alter 65
    await page.evaluate(() => { state = blankPlan(JSON.parse(JSON.stringify(state))); goto("milestones"); });
    for (const id of ["retire", "ahv"]) {
      await expect((await msRow(page, id)).locator("input[data-type=years]")).toHaveValue("65");
    }
    await noErrors(page, errors);
  });

  test("#25/AK7: Einstellungen ändern die Planung nicht; liegt sie ausserhalb der neuen Grenzen, meldet die Prüfung es", async ({ page }) => {
    const errors = await open(page);
    await page.evaluate(() => { state.pillar3a[0].contribution = 7500; renderSection(); });
    await expect(page.locator("#val li.err", { hasText: "Die Einzahlung muss" })).toHaveCount(1);   // über 7'258
    const before = await page.evaluate(() => JSON.stringify(state));
    const setMax = async v => {
      await page.locator('header button[data-action="settings"]').click();
      await page.locator("#set-pillar3aMax").fill(v);
      await page.locator("#settingsSave").click();
      await expect(page.locator("#settings")).toBeHidden();
    };
    await setMax("8000");
    expect(await page.evaluate(() => JSON.stringify(state))).toBe(before);                           // Planung unverändert
    await expect(page.locator("#val li.err", { hasText: "Die Einzahlung muss" })).toHaveCount(0);   // innerhalb der neuen Grenze
    await setMax("5000");
    expect(await page.evaluate(() => JSON.stringify(state))).toBe(before);
    await expect(page.locator("#val li.err", { hasText: "Maximalbetrag von 5'000" })).toHaveCount(1);
    await setMax("7258");                                                                            // Standard wieder herstellen
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
// #100: Aktienparameter je Aktienindex; #101: Obligationen und Inflation mit Korrelation je Index
test.describe("#100/#101: Monte Carlo parametrisch", () => {
  const KEYS = ["mu", "sigma", "df", "jumpLambda", "jumpMu", "jumpSigma"];
  const params = page => page.evaluate(k => Object.fromEntries(k.map(x => [x, state.returns.parametric[x]])), KEYS);
  const suggestion = page => page.evaluate(k => Object.fromEntries(k.map(x => [x, seriesInfo.parametric.stocks[x]])), KEYS);
  /** Aktienindex unter «Vermögen & Anlage» wählen und zurück zur Simulation. */
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

  test("#100/AK4: neue Planung, Wechsel des Aktienindex setzt die Vorschlagswerte des neuen Index", async ({ page }) => {
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
    // Neben den Feldern keine Vorschläge, solange die Werte dem Index folgen
    expect((await page.locator('#main [data-computed="psug"]').allInnerTexts()).join("")).toBe("");
    await noErrors(page, errors);
  });

  test("#100/AK5: eigene Werte bleiben beim Wechsel des Index, daneben die Vorschlagswerte; «Vorschlagswerte übernehmen»", async ({ page }) => {
    const errors = await parametric(page);
    await chooseStocks(page, "worldJst");
    const mu = field(page, "returns.parametric.mu");
    await mu.fill("5");
    await mu.press("Tab");
    expect(await page.evaluate(() => [state.returns.parametric.auto, state.returns.parametric.mu])).toEqual([false, 0.05]);
    await chooseStocks(page, "chJst");
    const own = await params(page), sug = await suggestion(page);
    expect(own.mu).toBe(0.05);
    expect(own.sigma).not.toBe(sug.sigma); // Volatilität von «Welt (JST)», nicht von «Schweiz (JST)»
    await expect(page.locator('#main [data-computed="pauto"]')).toContainText("Eigene Werte");
    const hints = await page.locator('#main [data-computed="psug"]').allInnerTexts();
    expect(hints).toHaveLength(6);
    expect(hints[0]).toBe(`Vorschlag: ${(sug.mu * 100).toLocaleString("de-CH", { maximumFractionDigits: 3 })} %`);
    await page.locator('#main [data-action="param-suggest"]').click();
    expect(await page.evaluate(() => state.returns.parametric.auto)).toBe(true);
    expect(await params(page)).toEqual(sug);
    await noErrors(page, errors);
  });

  test("#101/AK4: Matrix der Korrelation im Reiter «Simulation», abhängig von Aktien und Obligationen", async ({ page }) => {
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

  test("#101/AK6: Hinweis beim Öffnen einer älteren Planung mit «Monte Carlo parametrisch», eigene Werte bleiben", async ({ page }) => {
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

// #132: schnellere Zufallsziehung des parametrischen Modells; ältere Planungen erhalten beim Öffnen einen Hinweis
test.describe("Monte Carlo parametrisch rechnet schneller (#132)", () => {
  test("AK7: Hinweis beim Öffnen einer älteren Planung mit «Monte Carlo parametrisch»; neue Planungen tragen das Merkmal", async ({ page }) => {
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

test.describe("Rückfrage zum Speichern vor der Simulation (#110)", () => {
  /** Die Rückfrage ist im Browser nur mit `askSaveDev` aktiv; die Planung wird ungespeichert markiert. */
  const prepare = (page, dev) => page.evaluate(dev => { Pref.set("askSaveDev", dev); Pref.set("askSaveBeforeRun", true); markDirty(); }, dev);

  test("#110/AK1, AK2, AK3, AK5: Rückfrage vor der Simulation bei ungespeicherten Änderungen", async ({ page }) => {
    const errors = await open(page);
    // AK3: ohne Änderungen keine Rückfrage
    await page.evaluate(() => { Pref.set("askSaveDev", true); runSimulation("overview", { noConfirm: true }); });
    await expect(page.locator("#ask")).not.toHaveAttribute("open", "");
    await expect(page.locator("#resultsBody")).toBeVisible();
    await page.evaluate(() => showResults(false));
    // AK1, AK2: Abbrechen startet nichts, die Planung bleibt ungespeichert
    await prepare(page, true);
    await page.evaluate(() => { window.__ran = runSimulation("overview", { noConfirm: true }); });
    await expect(page.locator("#ask")).toHaveAttribute("open", "");
    await expect(page.locator("#askText")).toContainText("ungespeicherte Änderungen");
    await expect(page.locator('#ask button[value="discard"]')).toHaveText("Ohne Speichern simulieren");
    await page.locator('#ask button[value="cancel"]').click();
    await page.evaluate(() => window.__ran);
    await expect(page.locator("#results")).toBeHidden();
    expect(await page.evaluate(() => dirty)).toBe(true);
    // AK2: «Ohne Speichern simulieren» rechnet und lässt die Planung ungespeichert
    await page.evaluate(() => { window.__ran = runSimulation("overview", { noConfirm: true }); });
    await page.locator('#ask button[value="discard"]').click();
    await expect(page.locator("#resultsBody")).toBeVisible();
    expect(await page.evaluate(() => dirty)).toBe(true);
    // AK5: abgeschaltet, keine Rückfrage
    await page.evaluate(() => { showResults(false); Pref.set("askSaveBeforeRun", false); runSimulation("overview", { noConfirm: true }); });
    await expect(page.locator("#ask")).not.toHaveAttribute("open", "");
    await expect(page.locator("#resultsBody")).toBeVisible();
    await noErrors(page, errors);
  });

  test("#110/AK1, AK2, AK4: «Speichern» speichert zuerst und startet danach; neue Planung ohne Dateinamen", async ({ page }) => {
    const errors = await open(page);
    await prepare(page, true);
    await page.evaluate(() => { window.__ran = runSimulation("overview", { noConfirm: true }); });
    // AK1: drei Schaltflächen; AK4: eine Planung ohne Dateinamen sagt das in der Frage (im Browser ohne eigenen Dialog: Download)
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

  test("#110/AK5: die Einstellung steht im Dialog «Einstellungen» und bleibt erhalten", async ({ page }) => {
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
test.describe("Testzugänge für lastunabhängige Tests (#111)", () => {
  test("#111/AK1: #results rechnet Bootstrap mit 500 Pfaden und Startwert 42; zwei Läufe ergeben dieselben Kennzahlen", async ({ page }) => {
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

  test("#111/AK2: #nobench misst die Rechenzeit nicht, zeigt «Dauer wird geschätzt…» und rechnet ohne Rückfrage", async ({ page }) => {
    const api = [];
    page.on("request", r => { const u = new URL(r.url()); if (u.pathname.startsWith("/api/")) api.push(u.pathname); });
    await page.clock.install();
    const errors = await open(page, "scenarios"); // open() setzt #nobench
    await page.evaluate(() => { setP("flows.spendingFactor", 1.01); renderDerived(); });
    await page.clock.runFor(2000); // Verzögerung der Messung (800 ms) sicher vorbei
    await expect(page.locator("#est-scen")).toHaveText("Dauer wird geschätzt…");
    await page.locator('button[data-action="run"]').click();
    await expect(page.locator("#results")).toBeVisible();
    await expect(page.locator("#ask")).toBeHidden();
    expect(api).not.toContain("/api/bench_plan");
    await noErrors(page, errors);
  });

  test("#111/AK2: ohne #nobench misst die App die Rechenzeit (Gegenprobe)", async ({ page }) => {
    const bench = page.waitForRequest(r => new URL(r.url()).pathname === "/api/bench_plan");
    const errors = await open(page, "scenarios", { bench: true });
    await bench;
    await expect(page.locator("#est-scen")).toContainText("Geschätzte Dauer");
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Klick ins Menü während einer Vorschau (#112)", () => {
  /** Drückt die Maustaste auf `loc`, lässt die App wie beim Eintreffen einer Vorschau neu zeichnen und lässt dann los. */
  const clickDuringPreview = async (page, loc) => {
    await loc.scrollIntoViewIfNeeded();
    const b = await loc.boundingBox();
    await page.mouse.move(b.x + b.width / 2, b.y + b.height / 2);
    await page.mouse.down();
    await page.evaluate(() => renderDerived());
    await page.mouse.up();
  };

  test("#112/AK1, AK2: Menüeinträge bleiben dieselben Elemente; der Klick öffnet den Abschnitt", async ({ page }) => {
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

  test("#112/AK3: Klick auf eine Meldung der Prüfliste bei unveränderter Liste", async ({ page }) => {
    const errors = await open(page, "wealth");
    await previewIdle(page); // die Liste nach der ersten Vorschau; danach ändert sie sich nicht mehr
    const item = page.locator("#val li[data-sec]").first();
    await expect(item).toBeVisible();
    const sec = await item.getAttribute("data-sec");
    expect(sec).not.toBe("wealth");
    await clickDuringPreview(page, item);
    expect(await page.evaluate(() => current)).toBe(sec);
    await noErrors(page, errors);
  });

  test("#112/AK4: Zahl der Fehler am Menü erscheint und verschwindet", async ({ page }) => {
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
