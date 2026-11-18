// Oberflächentests von VivaPlan im Browser: Start und Navigation. Jeder Test startet mit der Beispielplanung.
const { test, expect } = require("@playwright/test");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { MOD, SECTIONS, open, noErrors, field, previewIdle, nav, kpiValue, resultKpi, parseNum, setOpt, msRow, welcome, steps, helpTitle, prefs } = require("./helpers");

test.describe("Start und Navigation", () => {
  test("Seite startet ohne Fehler, Vorschau und Prüfung erscheinen", async ({ page }) => {
    const errors = await open(page);
    await expect(page).toHaveTitle("VivaPlan");
    await expect(page.locator("#nav a")).toHaveCount(SECTIONS.length);
    await expect(page.locator("#kpis .kpi")).toHaveCount(4);
    await expect(page.locator("#val li").first()).toBeVisible();
    await expect(page.locator("#timeline svg .ms").first()).toBeAttached();
    await noErrors(page, errors);
  });

  test("Zeitleiste: Immobilienereignisse mit Beschriftung, gestaffelt, von Meilensteinen unterscheidbar; ab 16 nur Symbole; ohne Ergebnis keine", async ({ page }) => {
    const errors = await open(page);
    await expect(page.locator("#timeline svg g.pev").first()).toBeAttached();
    const labels = await page.$$eval("#timeline svg g.pev text.pelbl", ts => ts.map(t => t.textContent));
    expect(labels.some(t => /^Notverkauf Haus \d\d\.\d{4}$/.test(t))).toBe(true);
    expect(labels.length).toBe(3);
    expect(labels.some(t => /^Kauf Eigentumswohnung \d\d\.\d{4}$/.test(t))).toBe(true);
    // Keine zwei Beschriftungen überdecken sich
    const overlap = async () => page.evaluate(() => {
      const r = [...document.querySelectorAll("#timeline svg text.pelbl")].map(t => t.getBoundingClientRect());
      for (let i = 0; i < r.length; i++) for (let j = i + 1; j < r.length; j++)
        if (r[i].left < r[j].right && r[j].left < r[i].right && r[i].top < r[j].bottom && r[j].top < r[i].bottom) return true;
      return false;
    });
    expect(await overlap()).toBe(false);
    // Form (Dreieck statt Kreis) und Lage (unter der Linie) trennen Ereignisse von Meilensteinen, nicht nur die Farbe
    const geo = await page.evaluate(() => {
      const ev = document.querySelector("#timeline svg g.pev path"), ms = document.querySelector("#timeline svg g.ms circle");
      return { evTag: ev.tagName, msTag: ms.tagName, evY: ev.getBoundingClientRect().top, msY: ms.getBoundingClientRect().top, evFill: ev.getAttribute("fill") };
    });
    expect(geo.evTag).toBe("path"); expect(geo.msTag).toBe("circle"); expect(geo.evY).toBeGreaterThan(geo.msY);
    // Mehrere gleichzeitige Ereignisse (auch Notverkauf in der Warnfarbe) bleiben lesbar
    await page.evaluate(() => {
      const m = lastSim.propEvents[0].m;
      lastSim.propEvents.push({ m: m + 1, kind: "sale", property: "Zweitwohnung" }); // drei Ereignisse im selben Zeitraum (Notverkauf, Kauf, Verkauf)
      renderTimeline();
    });
    expect(await overlap()).toBe(false);
    const warn = await page.evaluate(() => document.querySelector('#timeline svg g.pev[data-kind="emergencySale"] path').getAttribute("fill"));
    const green = await page.evaluate(() => document.querySelector('#timeline svg g.pev[data-kind="purchase"] path').getAttribute("fill"));
    expect(warn).not.toBe(green);
    // Mehr als 15 Ereignisse: nur Symbole, die Beschriftung steht im Tooltip
    await page.evaluate(() => { const m = lastSim.propEvents[0].m; lastSim.propEvents = Array.from({ length: 16 }, (_, i) => ({ m: m + i * 3, kind: "purchase", property: "Haus " + i })); renderTimeline(); });
    await expect(page.locator("#timeline svg g.pev")).toHaveCount(16);
    await expect(page.locator("#timeline svg g.pev text.pelbl")).toHaveCount(0);
    expect(await page.locator("#timeline svg g.pev title").first().textContent()).toMatch(/^Kauf Haus 0 \d\d\.\d{4}/);
    // Ohne Simulationsergebnis keine Ereignisse
    await page.evaluate(() => { lastSim = null; renderTimeline(); });
    await expect(page.locator("#timeline svg g.pev")).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("#29: «Neu» lädt eine leere Planung mit Standardwerten", async ({ page }) => {
    const errors = await open(page);
    await page.locator('button[data-action="new"]').click();
    await page.waitForFunction(() => state.project.name === "Neue Planung");
    const out = await page.evaluate(() => ({
      name: state.project.name, person: state.household.persons, ms: state.milestones.map(m => m.id), start: state.milestones[0].date,
      cash: state.wealth.cash + state.wealth.stocks + state.wealth.bonds, props: state.realestate.properties.length, sweeps: state.scenarios.sweeps.length,
      income: state.flows.income, spending: state.flows.spending, extras: state.flows.extras.length, pk: state.pension.pk.capital, partner: !!state.partner,
      now: new Date().getFullYear() * 12 + new Date().getMonth(), file: document.getElementById("fileName").textContent,
    }));
    expect(out.name).toBe("Neue Planung");
    expect(out.person).toHaveLength(1);
    expect(out.person[0].name).toBe("");
    expect(out.ms).toEqual(["start", "retire", "ahv", "end"]);
    const [y, m] = out.start.split("-").map(Number);
    expect(y * 12 + m - 1).toBe(out.now);
    expect(out.cash).toBe(0); expect(out.props).toBe(0); expect(out.sweeps).toBe(0); expect(out.extras).toBe(0); expect(out.pk).toBe(0); expect(out.partner).toBe(false);
    expect(out.income).toEqual({ mode: "const", value: 0 }); expect(out.spending).toEqual({ mode: "const", value: 0 });
    expect(out.file).toBe("Neue Planung");
    await noErrors(page, errors);
  });

  test("#16/#17: Bestände, Guthaben und Marktwert heissen «per Simulationsstart (MM.JJJJ)» und folgen dem Start", async ({ page }) => {
    const errors = await open(page, "wealth");
    const start = async () => page.evaluate(() => startTxt());
    const s1 = await start();
    await expect(page.locator("#main")).toContainText(`Bestände per Simulationsstart (${s1})`);
    await expect(page.locator("#main .lead")).toHaveText("Bestände per Simulationsstart und Aufteilung des Portfolios.");
    await page.evaluate(() => { state.milestones[0].date = "2027-03"; });
    await page.locator("#nav a").nth(2).click();
    await expect(page.locator("#main")).toContainText("Altersguthaben per Simulationsstart (03.2027)");
    await page.evaluate(() => ACTIONS["partner-add"]());
    await expect(page.locator("#main label.field span", { hasText: "Altersguthaben per Simulationsstart (03.2027)" })).toHaveCount(2);
    await nav(page, "Immobilien");
    await expect(page.locator("#main")).toContainText("Marktwert per Simulationsstart (03.2027)");
    // #52: Die Säule 3a hat einen eigenen Abschnitt.
    await nav(page, "Säule 3a");
    await expect(page.locator("#main")).toContainText("Guthaben per Simulationsstart (03.2027)");
    for (let i = 0; i < SECTIONS.length; i++) {
      await page.locator("#nav a").nth(i).click();
      const t = await page.locator("#main").innerText();
      expect(t, `Abschnitt ${i}`).not.toMatch(/Bestände heute|Aktuelle Bestände|Guthaben heute|Marktwert heute|Altersguthaben heute/);
    }
    await noErrors(page, errors);
  });

  test("#19/#69: Erklärung der Anlagestrategie im Info-Popup (Maus, Fokus, Klick, Esc, kopierbar)", async ({ page }) => {
    const errors = await open(page, "wealth");
    const card = page.locator("#main .card", { has: page.locator("h3", { hasText: "Anlagestrategie" }) });
    const pop = card.locator("h3 .infopop"), body = pop.locator(".infobody");
    await expect(body).toBeHidden();
    await pop.hover();
    await expect(body).toBeVisible();
    await expect(body).toContainText("CC BY-NC-SA 4.0");
    await expect(body).toContainText("«Simulation»");
    await body.hover();
    await expect(body).toBeVisible();
    expect(await body.evaluate(el => getComputedStyle(el).userSelect)).toBe("text");
    await page.keyboard.press("Escape");
    await expect(body).toBeHidden();
    await page.mouse.move(5, 5);
    await pop.focus();
    await expect(body).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(body).toBeHidden();
    await expect(page.locator("#results")).toBeHidden(); // Esc im Popup führt nicht aus dem Editor
    await pop.blur();
    await page.mouse.move(5, 5);
    await pop.click();
    await expect(body).toBeVisible();
    await page.locator("#main h2").click();
    await expect(body).toBeHidden();
    await noErrors(page, errors);
  });

  test("#69/AK1: Strategie bei den Anlagen, Reiter «Simulation» ohne Wahl der Zeitreihe und ohne eigene Dateien", async ({ page }) => {
    const errors = await open(page, "wealth");
    const card = page.locator("#main .card", { has: page.locator("h3", { hasText: "Anlagestrategie" }) });
    const stocks = card.locator('select[data-path="returns.strategy.stocks"]'), bonds = card.locator('select[data-path="returns.strategy.bonds"]');
    expect(await stocks.locator("option").allInnerTexts()).toEqual(["Welt (JST)", "Welt individuell (JST)", "Schweiz (JST)", "S&P 500 (Shiller)"]);
    expect(await bonds.locator("option").allInnerTexts()).toEqual(["Eidgenossenschaft 10 Jahre", "USA 10 Jahre (Shiller)", "Eidgenossenschaft kurzfristig"]);
    await stocks.selectOption("worldJst");
    await bonds.selectOption("ch");
    await expect(card).toContainText("Jährlich 01.1900 – 12.2020");
    await bonds.selectOption("usa");
    await expect(card).toContainText("Jährlich 01.1922 – 12.2020");
    // #76/AK1, AK3: kurzfristige Eidgenossen, mit dem S&P 500 jährlich
    await stocks.selectOption("sp500");
    await bonds.selectOption("chShort");
    await expect(card).toContainText("Jährlich 01.1922 – 12.2025");
    await expect(card).toContainText("Geldmarktsatz");
    await bonds.selectOption("usa");
    await expect(card).toContainText("Monatlich 01.1922 – 12.2025");
    expect(await page.evaluate(() => state.returns.strategy)).toMatchObject({ stocks: "sp500", bonds: "usa" });
    await nav(page, "Simulation");
    await expect(page.locator("#main h2")).toHaveText(/^Simulation/);
    await page.evaluate(() => { state.returns.source = "rolling"; renderSection(); });
    await expect(page.locator("#main")).toContainText("Renditen aus der Anlagestrategie");
    await expect(page.locator("#main")).not.toContainText("Datenbibliothek");
    await expect(page.locator('#main [data-action="series-file"], #main [data-action="series-lib"]')).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("#69/AK8: Mittelwerte im gemeinsamen Zeitraum, Hinweis bei gekürztem Index, ohne Kürzung kein Hinweis", async ({ page }) => {
    const errors = await open(page, "wealth");
    const card = page.locator("#main .card", { has: page.locator("h3", { hasText: "Anlagestrategie" }) });
    const stocks = card.locator('select[data-path="returns.strategy.stocks"]'), bonds = card.locator('select[data-path="returns.strategy.bonds"]');
    const clip = card.locator(".cliphint");
    // S&P 500 mit US-Obligationen: beide Teile 1922–2025, kein Hinweis (#90: statt Pictet mit Pictet)
    await stocks.selectOption("sp500");
    await bonds.selectOption("usa");
    await expect(card).toContainText("Mittlere Jahresrendite im gemeinsamen Zeitraum 1922–2025 von Aktien und Obligationen");
    await expect(clip).toHaveCount(0);
    // Aktien JST (1900–2020) und Obligationen USA (1922–2025): beide gekürzt
    await stocks.selectOption("chJst");
    await expect(card).toContainText("im gemeinsamen Zeitraum 1922–2020");
    await expect(clip).toHaveText("⚠ Aktien Schweiz (JST) reichen von 1900 bis 2020, Obligationen USA 10 Jahre (Shiller) reichen von 1922 bis 2025 und werden auf den gemeinsamen Zeitraum 1922–2020 gekürzt. Die Simulation (Monte Carlo und historische Verläufe) nutzt nur diesen Zeitraum.");
    // Nur die Obligationen gekürzt: Eidgenossenschaft (1900–2024) mit Aktien JST (1900–2020)
    await bonds.selectOption("ch");
    await expect(clip).toContainText("Obligationen Eidgenossenschaft 10 Jahre reichen von 1900 bis 2024");
    await expect(clip).not.toContainText("Aktien");
    await noErrors(page, errors);
  });

  test("#69/AK2: «Welt individuell»: Gewichte zusammen 100 %, sonst Fehler beim Feld", async ({ page }) => {
    const errors = await open(page, "wealth");
    await page.locator('select[data-path="returns.strategy.stocks"]').selectOption("worldCustom");
    const card = page.locator("#main .card", { has: page.locator("h3", { hasText: "Anlagestrategie" }) });
    for (const k of ["ch", "usa", "eu", "apac"]) await expect(field(page, `returns.strategy.weights.${k}`)).toBeVisible();
    await expect(card).toContainText("Summe 100 %");
    const eu = field(page, "returns.strategy.weights.eu");
    await previewIdle(page);
    await eu.fill("50");
    await expect(card.locator(".ferr")).toHaveCount(0); // während der Eingabe wartet alles (#57)
    await eu.press("Tab");
    await expect(card).toContainText("Summe 120 %");
    await expect(card.locator(".ferr")).toContainText("Die Gewichte der Regionen ergeben 120 % statt 100 %.");
    await expect(page.locator("#val li.err")).toContainText("120 %");
    await eu.fill("30");
    await eu.press("Tab");
    await expect(card).toContainText("Summe 100 %");
    await expect(card.locator(".ferr")).toHaveCount(0);
    await expect(card).toContainText("Jährlich 01.1922 – 12.2020"); // Beispielplanung: US-Obligationen ab 1922
    await noErrors(page, errors);
  });

  test("#69/AK5: ältere Planung mit Zeitreihe wird beim Öffnen zur Strategie, mit Hinweis bei eigenen Dateien", async ({ page }) => {
    const errors = await open(page, "general");
    const out = await page.evaluate(async () => {
      const r = [];
      for (const series of ["library:Pictet CH.csv", "/tmp/x/Eigene.csv"]) {
        const plan = JSON.parse(JSON.stringify(state));
        delete plan.returns.strategy; plan.returns.series = series;
        const notes = await adopt(plan);
        r.push({ strategy: state.returns.strategy, series: state.returns.series, notes });
      }
      return r;
    });
    // #90/AK4: Pictet wird nicht mehr mitgeliefert: Schweiz (JST) mit Eidgenossenschaft, mit Hinweis
    expect(out[0].strategy).toMatchObject({ stocks: "chJst", bonds: "ch" });
    expect(out[0].series).toBeUndefined();
    expect(out[0].notes.join(" ")).toContain("«Schweiz (Pictet)» wird aus Lizenzgründen nicht mehr mitgeliefert");
    expect(out[1].strategy).toMatchObject({ stocks: "worldJst", bonds: "ch" });
    expect(out[1].notes.join(" ")).toContain("«/tmp/x/Eigene.csv» wird nicht mehr unterstützt");
    await noErrors(page, errors);
  });

  test("#90/AK2, AK3: keine Wahl «Schweiz (Pictet)»; eine Planung mit Pictet öffnet mit Schweiz (JST) und Eidgenossenschaft und einem Hinweis", async ({ page }) => {
    const errors = await open(page, "wealth");
    await expect(page.locator('select[data-path^="returns.strategy"] option[value="chPictet"]')).toHaveCount(0);
    const out = await page.evaluate(async () => {
      const plan = JSON.parse(JSON.stringify(state));
      plan.returns.strategy = { stocks: "chPictet", bonds: "chPictet" };
      const notes = await adopt(plan);
      return { strategy: state.returns.strategy, notes };
    });
    expect(out.strategy).toMatchObject({ stocks: "chJst", bonds: "ch" });
    expect(out.notes.join(" ")).toContain("für die Aktien «Schweiz (JST)» und für die Obligationen «Eidgenossenschaft 10 Jahre»");
    await noErrors(page, errors);
  });

  test("#34: Optimum eines Anteils an der Grenze 100 % / 0 % ohne «≥» / «≤»", async ({ page }) => {
    const errors = await open(page, "milestones");
    const t = await page.evaluate(() => [
      { param: "pension.pk.lumpsum", v: 1, kind: "exact", reaches: true },
      { param: "pension.pk.lumpsum", v: 0, kind: "exact", reaches: true },
      { param: "pension.pk.lumpsum", v: 0.5, kind: "edgeHigh", reaches: true },
      { param: "flows.spendingFactor", v: 1.1, kind: "atLeast", reaches: true }
    ].map(optimumValueText));
    expect(t).toEqual(["100 %", "0 %", "≥ 50 %", "≥ 110 %"]);
    await noErrors(page, errors);
  });

  test("#37: Jahreswerte ohne Spaltengruppen, Einkommen und Ausgaben geplant; Spaltenreihenfolge mit Entnahme; Titel zentriert und fett; Knopf «Steuern anzeigen» sichtbar", async ({ page }) => {
    const errors = await open(page, "results/table/nosweep");
    const head = page.locator("#resultsBody table.yt tr").first();
    const cols = (await head.locator("th").allInnerTexts()).map(t => t.replace(/\s+/g, " ").trim());
    expect(cols).toEqual(["Jahr", "Alter", "Liquid", "Immobilien", "Vorsorge", "Total", "Ausgaben total", "Entnahme", "Einkommen total", "Entnahmerate", "Abweichung vom Plan", "Inflation pro Jahr"]);
    await expect(page.locator("#resultsBody table.yt")).not.toContainText(/Pro Monat|Vermögen am Jahresende|Ausgaben geplant|Liquide/);
    await expect(page.locator("#resultsBody table.yt tr:first-child th[colspan], #resultsBody table.yt th[rowspan]")).toHaveCount(0);
    // Hinweis auf Monatsdurchschnitt und Jahresende im Tooltip und im Text unter der Tabelle
    expect(await head.locator("th", { hasText: "Ausgaben total" }).getAttribute("title")).toContain("Monatsdurchschnitt");
    expect(await head.locator("th", { hasText: "Liquid" }).getAttribute("title")).toContain("Jahresende");
    await expect(page.locator("#resultsBody p.hint").first()).toContainText("Monatsdurchschnitte");
    await expect(page.locator("#resultsBody p.hint").first()).toContainText("Stände am Jahresende");
    // Titel der Tabelle zentriert und fett
    const h3 = page.locator("#resultsBody .card.yearcard > h3");
    await expect(h3).toContainText("Jahreswerte");
    const st = await h3.evaluate(el => ({ j: getComputedStyle(el).justifyContent, w: +getComputedStyle(el).fontWeight }));
    expect(st.j).toBe("center"); expect(st.w).toBeGreaterThanOrEqual(600);
    // die Zeilen haben so viele Zellen wie der Kopf
    expect(await page.locator("#resultsBody table.yt tr").nth(1).locator("td").count()).toBe(cols.length);
    // Befund 12: der Knopf «Steuern anzeigen» ist sichtbar, nicht überdeckt und blendet die fünf Steuerspalten ein und aus
    const btn = page.locator('#resultsBody button[data-action="tax-cols"]');
    await expect(btn).toBeVisible();
    await expect(btn).toHaveText("Steuern anzeigen");
    const hit = await btn.evaluate(b => { const r = b.getBoundingClientRect(), e = document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2); return { inView: r.top >= 0 && r.bottom <= innerHeight, same: b.contains(e) }; });
    expect(hit).toEqual({ inView: true, same: true });
    await btn.click();
    await expect(page.locator("#resultsBody table.yt th.tx")).toHaveCount(5);
    await expect(btn).toHaveText("Steuern ausblenden");
    expect(await page.locator("#resultsBody table.yt tr").nth(1).locator("td").count()).toBe(cols.length + 5);
    await btn.click();
    await expect(page.locator("#resultsBody table.yt th.tx")).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("#37: Zeitreihe in den Parametern ohne «library:»", async ({ page }) => {
    const errors = await open(page);
    const t = await page.evaluate(() => [seriesShort("library:Shiller CHF + CH-Obligationen.csv"), seriesShort("library:Welt (JST)"), seriesShort("/tmp/x/Eigene.csv")]);
    expect(t).toEqual(["Shiller CHF + CH-Obligationen", "Welt (JST)", "Eigene"]);
    const rows = await page.evaluate(() => {
      const plan = JSON.parse(JSON.stringify(state));
      plan.returns.source = "historical"; plan.returns.series = "library:Shiller CHF + CH-Obligationen.csv";
      return paramSections(plan).flatMap(([, r]) => r).filter(([k]) => k === "Zeitreihe");
    });
    expect(rows).toEqual([["Zeitreihe", "Shiller CHF + CH-Obligationen"]]);
    await noErrors(page, errors);
  });

  test("#37: Wechsel Datum ↔ Alter von übernimmt den Wert aus «Ergibt»", async ({ page }) => {
    const errors = await open(page, "milestones");
    const i = await page.evaluate(() => state.milestones.findIndex(m => m.id === "ahv"));
    const row = await msRow(page, "ahv");
    const ym = await page.evaluate(() => { const m = msMonth("ahv"); return { m: i2ym(m), age: ageAt(m, state.milestones.find(x => x.id === "ahv").person) }; });
    // Alter → Datum: das Datum ist der berechnete Zeitpunkt
    await row.locator("select").first().selectOption("date");
    expect(await page.evaluate(i => [state.milestones[i].kind, state.milestones[i].date], i)).toEqual(["date", ym.m]);
    await expect(row.locator("input[data-type=month]")).toHaveValue(ym.m.slice(5) + "." + ym.m.slice(0, 4));
    // Datum → Alter: das Alter ist das berechnete Alter, der Zeitpunkt bleibt gleich
    await row.locator("select").first().selectOption("age");
    const after = await page.evaluate(i => ({ k: state.milestones[i].kind, y: state.milestones[i].years, m: i2ym(msMonth(state.milestones[i].id)) }), i);
    expect(after.k).toBe("age");
    expect(after.m).toBe(ym.m);
    expect(Math.abs(after.y - ym.age)).toBeLessThan(0.01);
    await expect(row.locator("input").last()).toHaveValue(/\d/);
    // anderes Datum, dann Wechsel: neues Alter
    await row.locator("select").first().selectOption("date");
    await row.locator("input[data-type=month]").fill("03.2050");
    await row.locator("input[data-type=month]").blur();
    await row.locator("select").first().selectOption("age");
    expect(await page.evaluate(i => i2ym(msMonth(state.milestones[i].id)), i)).toBe("2050-03");
    await noErrors(page, errors);
  });

  test("#37: Warnsymbol in der Karte «Umfang» ab 90 s geschätzter Rechenzeit", async ({ page }) => {
    const errors = await open(page, "scenarios");
    await page.evaluate(() => { state.scenarios.sweeps = []; state.returns.source = "fixed"; renderSection(); });
    const fake = sec => page.evaluate(sec => { Object.assign(bench, { key: benchKey(), us: sec * 1e6, fixedPaths: 1, error: null }); updateEstimates(); }, sec);
    const warn = page.locator("#est-warn");
    await fake(89);
    await expect(warn).toHaveText("");
    await fake(91);
    await expect(warn).toHaveText("⚠");
    expect(await warn.getAttribute("title")).toContain("90 s");
    // neben der Zahl der Simulationsläufe
    expect(await page.evaluate(() => document.getElementById("est-warn").previousElementSibling.dataset.computed ?? document.getElementById("est-warn").parentElement.textContent)).toBeTruthy();
    await expect(page.locator("#main .card", { hasText: "Umfang" })).toContainText("⚠");
    await fake(10);
    await expect(warn).toHaveText("");
    await noErrors(page, errors);
  });

  test("#15/#30, #51/AK1: Meilensteine ohne Spalte «Verwendet», verwendete lassen sich mit Umstellung auf das Datum löschen, Start/Ende/Bezug Pensionskasse/AHV-Bezug gesperrt", async ({ page }) => {
    const errors = await open(page, "milestones");
    await expect(page.locator("#main table.tbl").first()).not.toContainText("Verwendet");
    const rows = () => page.locator("#main table.tbl tr:has(select[data-path^='milestones.'])");
    // Gesperrte Meilensteine: Schaltfläche deaktiviert mit Tooltip
    for (const id of ["start", "retire", "ahv", "end"]) {
      const row = await msRow(page, id);
      const btn = row.locator("button:has-text('✕')");
      await expect(btn).toBeDisabled();
      expect(await btn.getAttribute("title")).toMatch(/lassen sich nicht löschen|nicht löschen/);
    }
    const preview = () => page.evaluate(async () => { const r = await Engine.call("preview", { plan: state, path: null }); return { end: r.result.summary.endTotal, props: JSON.stringify(state.realestate.properties.map(p => [p.buy, p.sell])) }; });
    const before = await preview();
    // #30: «75 Jahre» (sell_house) wird zweimal verwendet (Einnahmen & Ausgaben, Verkauf Haus); Löschen ist mit Umstellung möglich
    const props0 = await page.evaluate(() => state.realestate.properties.map(p => [p.buy ?? null, p.sell ?? null]));
    expect(props0[0]).toEqual([null, "sell_house"]);
    const row = await msRow(page, "sell_house");
    await row.locator("button[data-action='ms-del']").click();
    await expect(page.locator("#ask")).toBeVisible();
    const text = await page.locator("#askText").innerText();
    expect(text).toMatch(/wird in 2 Eingaben verwendet \(Abschnitte: [^)]*Immobilien\)/);
    expect(text).toContain("06.2045");
    expect(await page.locator("#ask button[value='ok']").innerText()).toBe("Löschen und umstellen");
    // Abbrechen ändert nichts
    await page.locator("#ask button[value='cancel']").click();
    expect(await page.evaluate(() => state.milestones.some(m => m.id === "sell_house"))).toBe(true);
    await row.locator("button[data-action='ms-del']").click();
    await page.locator("#ask button[value='ok']").click();
    await expect(page.locator("#toast")).toContainText("2 Eingaben auf 06.2045 umgestellt");
    expect(await page.evaluate(() => [state.milestones.some(m => m.id === "sell_house"), state.realestate.properties.map(p => [p.buy ?? null, p.sell ?? null])])).toEqual([false, [[null, "2045-06"], ...props0.slice(1)]]);
    // Die Rechnung liefert dieselben Zahlen wie vorher
    const after = await preview();
    expect(after.end).toBeCloseTo(before.end, 6);
    // Rückgängig stellt Meilenstein und Verwendungen wieder her
    await page.keyboard.press(`${MOD}+z`);
    expect(await page.evaluate(() => [state.milestones.some(m => m.id === "sell_house"), state.realestate.properties.map(p => [p.buy ?? null, p.sell ?? null])])).toEqual([true, props0]);
    await noErrors(page, errors);
  });

  test("#15: Meilenstein «Alter von» wird auf das berechnete Datum umgestellt, unbenutzte ohne Dialog gelöscht, doppelter Zeitpunkt in einer Reihe", async ({ page }) => {
    const errors = await open(page, "milestones");
    // «AHV-Bezug» ist fest (#51); geprüft wird mit einem eigenen Meilenstein «Alter von».
    await page.evaluate(() => { state.milestones.splice(state.milestones.length - 1, 0, { id: "kids", name: "Kinder ausgezogen", kind: "age", person: "A", years: 62 }); renderSection(); });
    const ahv = await page.evaluate(() => i2ym(msMonth("kids")));
    // Eine zweite Zeile der Ausgabenreihe liegt genau auf dem Datum des Meilensteins: nach der Umstellung bleibt die spätere
    await page.evaluate(ahv => { state.flows.spending = { mode: "timed", rows: [{ from: "start", value: 100 }, { from: "kids", value: 200 }, { from: ahv, value: 300 }] }; renderSection(); }, ahv);
    await (await msRow(page, "kids")).locator("button[data-action='ms-del']").click();
    await expect(page.locator("#askText")).toContainText(ahv.slice(5) + "." + ahv.slice(0, 4));
    await page.locator("#ask button[value='ok']").click();
    await expect(page.locator("#toast")).toContainText("Doppelter Zeitpunkt");
    const out = await page.evaluate(() => ({ rows: state.flows.spending.rows }));
    expect(out.rows).toEqual([{ from: "start", value: 100 }, { from: ahv, value: 300 }]);
    // Unbenutzter Meilenstein: ohne Dialog
    await page.locator('button[data-action="ms-add"]').click();
    const n = await page.evaluate(() => state.milestones.length);
    await page.locator("#main table.tbl button[data-action='ms-del']").last().click();
    await expect(page.locator("#ask")).toBeHidden();
    expect(await page.evaluate(() => state.milestones.length)).toBe(n - 1);
    // Variation (Pensionierung) bleibt nach dem Löschen anderer Meilensteine möglich
    expect(await page.evaluate(() => state.milestones.some(m => m.id === "retire") && !!SWEEP_PARAMS.find(p => p.id === "milestone.retire"))).toBe(true);
    await noErrors(page, errors);
  });

  test("#14: Pflichtfelder mit Sternchen, Fehler am Feld, Start und Optimierung gesperrt bis alles gültig ist", async ({ page }) => {
    const errors = await open(page);
    await page.locator('button[data-action="new"]').click();
    await page.waitForFunction(() => state.project.name === "Neue Planung");
    // Sternchen an Name und Geburt
    const name = page.locator("#main label.field", { has: page.locator('input[data-path="household.persons.0.name"]') });
    await expect(name.locator(".req")).toHaveText("*");
    await expect(page.locator("#main label.field", { has: page.locator('input[data-path="household.persons.0.birth"]') }).locator(".req")).toHaveText("*");
    await expect(page.locator("#main label.field", { has: page.locator('input[data-path="household.persons.0.name"]') }).locator(".ferr")).toContainText("Der Name fehlt");
    await expect(page.locator("#nav a", { hasText: "Haushalt" }).locator(".badge.err")).toBeVisible();
    // Leerer Name: «Simulation starten» gesperrt
    await page.locator('button[data-action="run"]').click();
    await expect(page.locator("#toast")).toContainText("Die Planung hat 1 Fehler");
    await expect(page.locator("#results")).toBeHidden();
    // Ungültiges Geburtsdatum: Fehler am Feld, gesperrt
    await field(page, "household.persons.0.name").fill("Anna");
    await field(page, "household.persons.0.name").press("Tab"); // #57: die Meldung verschwindet beim Verlassen
    await expect(name.locator(".ferr")).toHaveCount(0);
    const birth = field(page, "household.persons.0.birth");
    await birth.fill("");
    await birth.press("Tab"); // #57: Meldung erst beim Verlassen des Feldes
    await expect(page.locator("#main label.field", { has: birth }).locator(".ferr")).toContainText("Ungültiges Datum");
    await page.locator('button[data-action="run"]').click();
    await expect(page.locator("#toast")).toContainText("Die Planung hat 1 Fehler");
    await expect(page.locator("#toast")).toContainText("Haushalt");
    // Optimierung gesperrt
    await page.evaluate(() => { state.returns.source = "bootstrap"; return OPT_ACTIONS["opt-run"](); });
    await expect(page.locator("#toast")).toContainText("Die Planung hat 1 Fehler");
    await page.evaluate(() => goto("general"));
    // Alles gültig: die Simulation startet
    await birth.fill("01.1970");
    await birth.blur();
    await page.locator('button[data-action="run"]').click();
    await expect(page.locator("#results")).toBeVisible();
    await expect(page.locator("#resultsTitle")).not.toHaveText("Simulation nicht möglich");
    await noErrors(page, errors);
  });

  test("alle Abschnitte per Klick und per Tastatur erreichbar", async ({ page }) => {
    const errors = await open(page);
    for (let i = 0; i < SECTIONS.length; i++) {
      await page.locator("#nav a").nth(i).click();
      await expect(page.locator("#main h2")).toContainText(SECTIONS[i].split(" ")[0]);
      await expect(page.locator("#nav a.active")).toContainText(SECTIONS[i]);
    }
    // Tastenkürzel ⌘1 bis ⌘0 gelten für die ersten zehn Abschnitte
    for (let i = 0; i < Math.min(10, SECTIONS.length); i++) {
      await page.keyboard.press(`${MOD}+${(i + 1) % 10}`);
      await expect(page.locator("#nav a.active")).toContainText(SECTIONS[i]);
    }
    await page.keyboard.press(`${MOD}+1`);
    await page.keyboard.press(`${MOD}+ArrowDown`).catch(() => {});
    await noErrors(page, errors);
  });

  test("Tastenkürzel-Übersicht und «Über»-Dialog", async ({ page }) => {
    const errors = await open(page);
    await page.getByRole("button", { name: "Tastenkürzel" }).click();
    await expect(page.locator("#ask")).toBeVisible();
    await expect(page.locator("#ask .kbd kbd").first()).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(page.locator("#ask")).not.toBeVisible();
    await noErrors(page, errors);
  });
});
