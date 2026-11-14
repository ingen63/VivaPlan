// Oberflächentests von VivaPlan im Browser. Jeder Test startet mit der Beispielplanung.
const { test, expect } = require("@playwright/test");
const fs = require("fs");
const os = require("os");
const path = require("path");

const MOD = process.platform === "darwin" ? "Meta" : "Control";
const SECTIONS = ["Haushalt & Projekt", "Meilensteine", "Vorsorge", "Säule 3a", "Einnahmen & Ausgaben", "Vermögen & Anlage",
  "Simulation", "Entnahmestrategie", "Optimierung", "Steuern", "Immobilien", "Szenarien & Simulation"];

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
const field = (page, p) => page.locator(`input[data-path="${p}"]`);
const kpiValue = (page, label) => page.locator(`#kpis .kpi:has(.k:text-is("${label}")) .v`);
const resultKpi = (page, label) => page.locator(`#resultsBody .kpi:has(.k:text-is("${label}")) .v`);
const parseNum = s => Number(s.replace(/[’'\sk]/g, ""));
/** Stellt die Optimierung ein (z. B. kleine Rechnung) und gibt zurück, was in den Feldern steht. Die Werte gehen über die
 *  Planung (`state.optimization`), denn `saveOptCfg` liest die Felder und überschreibt ein direkt gesetztes `optCfg` (ISS-16). */
const setOpt = (page, cfg) => page.evaluate(cfg => { state.optimization = optToPlan({ ...optCfg, ...cfg }); renderSection(); return readOptCfg(); }, cfg);
/** Tabellenzeile eines Meilensteins (Name steht in einem Eingabefeld, deshalb über den Index der ID). */
async function msRow(page, id) {
  const i = await page.evaluate(id => state.milestones.findIndex(m => m.id === id), id);
  // Feste Meilensteine (FEA-38) haben keinen änderbaren Namen; jede Zeile hat aber die Auswahl «Festgelegt als».
  return page.locator(`#main table.tbl tr:has(select[data-path="milestones.${i}.kind"])`);
}

// ---------------------------------------------------------------------------------------------
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

  test("ISS-3: «Neu» lädt eine leere Planung mit Standardwerten", async ({ page }) => {
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

  test("FEA-15/16: Bestände, Guthaben und Marktwert heissen «per Simulationsstart (MM.JJJJ)» und folgen dem Start", async ({ page }) => {
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
    await page.locator("#nav a", { hasText: "Immobilien" }).click();
    await expect(page.locator("#main")).toContainText("Marktwert per Simulationsstart (03.2027)");
    // FEA-37: Die Säule 3a hat einen eigenen Abschnitt.
    await page.locator("#nav a", { hasText: "Säule 3a" }).click();
    await expect(page.locator("#main")).toContainText("Guthaben per Simulationsstart (03.2027)");
    for (let i = 0; i < SECTIONS.length; i++) {
      await page.locator("#nav a").nth(i).click();
      const t = await page.locator("#main").innerText();
      expect(t, `Abschnitt ${i}`).not.toMatch(/Bestände heute|Aktuelle Bestände|Guthaben heute|Marktwert heute|Altersguthaben heute/);
    }
    await noErrors(page, errors);
  });

  test("FEA-18/FEA-53: Erklärung der Anlagestrategie im Info-Popup (Maus, Fokus, Klick, Esc, kopierbar)", async ({ page }) => {
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

  test("FEA-53/AK1: Strategie bei den Anlagen, Reiter «Simulation» ohne Wahl der Zeitreihe und ohne eigene Dateien", async ({ page }) => {
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
    // FEA-59/AK1, AK3: kurzfristige Eidgenossen, mit dem S&P 500 jährlich
    await stocks.selectOption("sp500");
    await bonds.selectOption("chShort");
    await expect(card).toContainText("Jährlich 01.1922 – 12.2025");
    await expect(card).toContainText("Geldmarktsatz");
    await bonds.selectOption("usa");
    await expect(card).toContainText("Monatlich 01.1922 – 12.2025");
    expect(await page.evaluate(() => state.returns.strategy)).toMatchObject({ stocks: "sp500", bonds: "usa" });
    await page.locator("#nav a", { hasText: "Simulation" }).first().click();
    await expect(page.locator("#main h2")).toHaveText(/^Simulation/);
    await page.evaluate(() => { state.returns.source = "rolling"; renderSection(); });
    await expect(page.locator("#main")).toContainText("Renditen aus der Anlagestrategie");
    await expect(page.locator("#main")).not.toContainText("Datenbibliothek");
    await expect(page.locator('#main [data-action="series-file"], #main [data-action="series-lib"]')).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("FEA-53/AK8: Mittelwerte im gemeinsamen Zeitraum, Hinweis bei gekürztem Index, ohne Kürzung kein Hinweis", async ({ page }) => {
    const errors = await open(page, "wealth");
    const card = page.locator("#main .card", { has: page.locator("h3", { hasText: "Anlagestrategie" }) });
    const stocks = card.locator('select[data-path="returns.strategy.stocks"]'), bonds = card.locator('select[data-path="returns.strategy.bonds"]');
    const clip = card.locator(".cliphint");
    // S&P 500 mit US-Obligationen: beide Teile 1922–2025, kein Hinweis (FEA-72: statt Pictet mit Pictet)
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

  test("FEA-53/AK2: «Welt individuell»: Gewichte zusammen 100 %, sonst Fehler beim Feld", async ({ page }) => {
    const errors = await open(page, "wealth");
    await page.locator('select[data-path="returns.strategy.stocks"]').selectOption("worldCustom");
    const card = page.locator("#main .card", { has: page.locator("h3", { hasText: "Anlagestrategie" }) });
    for (const k of ["ch", "usa", "eu", "apac"]) await expect(field(page, `returns.strategy.weights.${k}`)).toBeVisible();
    await expect(card).toContainText("Summe 100 %");
    const eu = field(page, "returns.strategy.weights.eu");
    await eu.fill("50");
    await expect(card.locator(".ferr")).toHaveCount(0); // während der Eingabe wartet alles (ISS-13)
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

  test("FEA-53/AK5: ältere Planung mit Zeitreihe wird beim Öffnen zur Strategie, mit Hinweis bei eigenen Dateien", async ({ page }) => {
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
    // FEA-72/AK4: Pictet wird nicht mehr mitgeliefert: Schweiz (JST) mit Eidgenossenschaft, mit Hinweis
    expect(out[0].strategy).toMatchObject({ stocks: "chJst", bonds: "ch" });
    expect(out[0].series).toBeUndefined();
    expect(out[0].notes.join(" ")).toContain("«Schweiz (Pictet)» wird aus Lizenzgründen nicht mehr mitgeliefert");
    expect(out[1].strategy).toMatchObject({ stocks: "worldJst", bonds: "ch" });
    expect(out[1].notes.join(" ")).toContain("«/tmp/x/Eigene.csv» wird nicht mehr unterstützt");
    await noErrors(page, errors);
  });

  test("FEA-72/AK2, AK3: keine Wahl «Schweiz (Pictet)»; eine Planung mit Pictet öffnet mit Schweiz (JST) und Eidgenossenschaft und einem Hinweis", async ({ page }) => {
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

  test("ISS-7: Optimum eines Anteils an der Grenze 100 % / 0 % ohne «≥» / «≤»", async ({ page }) => {
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

  test("ISS-10: Jahreswerte ohne Spaltengruppen, Einkommen und Ausgaben geplant; Spaltenreihenfolge mit Entnahme; Titel zentriert und fett; Knopf «Steuern anzeigen» sichtbar", async ({ page }) => {
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

  test("ISS-10: Zeitreihe in den Parametern ohne «library:»", async ({ page }) => {
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

  test("ISS-10: Wechsel Datum ↔ Alter von übernimmt den Wert aus «Ergibt»", async ({ page }) => {
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

  test("ISS-10: Warnsymbol in der Karte «Umfang» ab 90 s geschätzter Rechenzeit", async ({ page }) => {
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

  test("FEA-14/ISS-4, FEA-38/AK1: Meilensteine ohne Spalte «Verwendet», verwendete lassen sich mit Umstellung auf das Datum löschen, Start/Ende/Bezug Pensionskasse/AHV-Bezug gesperrt", async ({ page }) => {
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
    // ISS-4: «75 Jahre» (sell_house) wird zweimal verwendet (Einnahmen & Ausgaben, Verkauf Haus); Löschen ist mit Umstellung möglich
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

  test("FEA-14: Meilenstein «Alter von» wird auf das berechnete Datum umgestellt, unbenutzte ohne Dialog gelöscht, doppelter Zeitpunkt in einer Reihe", async ({ page }) => {
    const errors = await open(page, "milestones");
    // «AHV-Bezug» ist fest (FEA-38); geprüft wird mit einem eigenen Meilenstein «Alter von».
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

  test("FEA-13: Pflichtfelder mit Sternchen, Fehler am Feld, Start und Optimierung gesperrt bis alles gültig ist", async ({ page }) => {
    test.setTimeout(120_000);
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
    await expect(name.locator(".ferr")).toHaveCount(0);
    const birth = field(page, "household.persons.0.birth");
    await birth.fill("");
    await birth.press("Tab"); // ISS-13: Meldung erst beim Verlassen des Feldes
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

// ---------------------------------------------------------------------------------------------
test.describe("Eingaben, Prüfung und Live-Vorschau", () => {
  test("Vorschau rechnet beim Verlassen des Feldes neu (ISS-13)", async ({ page }) => {
    const errors = await open(page);
    await page.locator("#nav a", { hasText: "Vermögen & Anlage" }).click();
    const before = await kpiValue(page, "Vermögen am Ende").textContent();
    const stocks = page.locator('input[data-path="wealth.stocks"]');
    const original = await stocks.inputValue();
    await stocks.fill("5000000");
    await stocks.press("Tab");
    await expect(kpiValue(page, "Vermögen am Ende")).not.toHaveText(before);
    await stocks.fill(original);
    await stocks.press("Tab");
    await expect(kpiValue(page, "Vermögen am Ende")).toHaveText(before);
    await noErrors(page, errors);
  });

  test("Live-Vorschau stimmt mit der endgültigen Simulation überein", async ({ page }) => {
    const errors = await open(page, "results/nosweep");
    // Testzugang rechnet direkt; zurück in den Editor, Wert merken, erneut rechnen und vergleichen.
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "overview");
    const finalEnd = await resultKpi(page, "Vermögen am Ende").textContent();
    const finalRet = await resultKpi(page, "Vermögen bei Pensionierung").textContent();
    await page.getByRole("button", { name: "← Zurück zum Editor" }).first().click();
    await expect(kpiValue(page, "Vermögen am Ende")).not.toHaveText("–");
    expect(await kpiValue(page, "Vermögen am Ende").textContent()).toBe(finalEnd);
    expect(await kpiValue(page, "Vermögen bei Pensionierung").textContent()).toBe(finalRet);
    await noErrors(page, errors);
  });

  test("ungültige Eingaben werden markiert und gemeldet", async ({ page }) => {
    const errors = await open(page);
    const birth = field(page, "household.persons.0.birth");
    await birth.fill("13.1970");
    await birth.press("Tab"); // ISS-13
    await expect(birth).toHaveClass(/invalid/);
    await birth.fill("xyz");
    await birth.press("Tab");
    await expect(birth).toHaveClass(/invalid/);
    await birth.fill("03.1970");
    await birth.press("Tab");
    await expect(birth).not.toHaveClass(/invalid/);
    await noErrors(page, errors);
  });

  test("Simulationsende vor Start: Fehler im Menü, keine Berechnung", async ({ page }) => {
    const errors = await open(page);
    await page.locator("#nav a", { hasText: "Meilensteine" }).click();
    const endRow = await msRow(page, "end");
    await endRow.locator("select").first().selectOption("date");
    await endRow.locator("input[data-type=month]").fill("01.2000");
    await expect(page.locator("#val li.err").first()).toBeVisible();
    await expect(page.locator("#nav a .badge.err").first()).toBeVisible();
    await page.keyboard.press(`${MOD}+Enter`);
    // FEA-13: Der Start ist gesperrt, die Meldung nennt die Fehlerzahl, der Verweis führt in den betroffenen Abschnitt
    await expect(page.locator("#toast")).toContainText("Die Planung hat 1 Fehler");
    await expect(page.locator("#toast")).toContainText("Meilensteine");
    await expect(page.locator("#results")).toBeHidden();
    await expect(page.locator("#nav a.active")).toContainText("Meilensteine");
    await noErrors(page, errors);
  });

  test("Geburtsdatum in der Zukunft und Alter über 120 werden gemeldet", async ({ page }) => {
    const errors = await open(page);
    const birth = field(page, "household.persons.0.birth");
    await birth.fill("01.2999");
    await birth.press("Tab"); // ISS-13: Prüfliste erst beim Verlassen
    await expect(page.locator("#val li.err").first()).toBeVisible();
    await birth.fill("01.1850");
    await birth.press("Tab");
    await expect(page.locator("#val li.err").first()).toBeVisible();
    await birth.fill("01.1965");
    await birth.press("Tab");
    await expect(page.locator("#val li.err")).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("leerer Projektname und leere Zahlen führen nicht zum Absturz", async ({ page }) => {
    const errors = await open(page);
    await field(page, "project.name").fill("");
    await page.locator("#nav a", { hasText: "Vermögen & Anlage" }).click();
    const stocks = field(page, "wealth.stocks");
    await stocks.fill("");
    await stocks.press("Tab"); // ISS-13
    await expect(stocks).toHaveClass(/invalid/);
    await stocks.fill("1'000'000");
    await stocks.press("Tab");
    await expect(stocks).not.toHaveClass(/invalid/);
    await noErrors(page, errors);
  });

  test("Zweite Person hinzufügen und entfernen", async ({ page }) => {
    const errors = await open(page);
    await page.getByRole("button", { name: "+ Zweite Person" }).click();
    await expect(page.locator(".person", { hasText: "Person 2" })).toBeVisible();
    await expect(page.locator("#kpis .kpi").first()).toBeVisible();
    await page.keyboard.press(`${MOD}+3`);
    await expect(page.locator("#main")).toContainText("Pensionskasse");
    await page.keyboard.press(`${MOD}+1`);
    await page.locator('[data-action="partner-del"]').click();
    await expect(page.locator(".person", { hasText: "Person 2" })).toHaveCount(0);
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Meilensteine", () => {
  test("Pensionierung verschieben: abhängige Werte folgen", async ({ page }) => {
    const errors = await open(page);
    await page.locator("#nav a", { hasText: "Meilensteine" }).click();
    const row = await msRow(page, "retire");
    await row.locator("select").first().selectOption("date");
    const date = row.locator("input[data-type=month]");
    await date.fill("01.2030");
    await date.press("Tab"); // ISS-13: berechnete Werte erst beim Verlassen
    const result = row.locator('[data-computed="ms"]');
    await expect(result).toContainText("01.2030");
    const age1 = await result.textContent();
    await date.fill("01.2029");
    await date.press("Tab");
    await expect(result).toContainText("01.2029");
    expect(await result.textContent()).not.toBe(age1);
    // Vorschau und Zeitleiste folgen dem neuen Datum
    await expect(kpiValue(page, "Vermögen bei Pensionierung").locator("xpath=../div[@class='s']")).toHaveText("01.2029");
    await expect(page.locator("#val li.err")).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("FEA-38/AK2: Bezug Pensionskasse ausserhalb der Grenzen: Fehler am Feld statt Zahlen", async ({ page }) => {
    const errors = await open(page);
    await page.locator("#nav a", { hasText: "Meilensteine" }).click();
    const row = await msRow(page, "retire");
    await row.locator("select").first().selectOption("date");
    await row.locator("input[data-type=month]").fill("01.2060");
    await expect(page.locator("#val li.err").first()).toContainText("«Bezug Pensionskasse»: Alter 89.6 liegt ausserhalb der Grenzen 58 bis 65 Jahre");
    await expect(page.locator("#kpis .kpi")).toHaveCount(1);
    await expect(page.locator("#kpis .kpi .s")).toHaveText("siehe Prüfung");
    await noErrors(page, errors);
  });

  test("Meilenstein nach Alter und nach Datum ergeben denselben Monat", async ({ page }) => {
    const errors = await open(page);
    await page.locator("#nav a", { hasText: "Meilensteine" }).click();
    const row = await msRow(page, "retire");
    await row.locator("select").first().selectOption("age");
    await row.locator("input[data-type=years]").fill("64");
    await expect(row.locator('[data-computed="ms"]')).toContainText("Alter 64.0");
    const byAge = await row.locator('[data-computed="ms"]').textContent();
    const m = /(\d{2}\.\d{4})/.exec(byAge)[1];
    await row.locator("select").first().selectOption("date");
    await row.locator("input[data-type=month]").fill(m);
    await expect(row.locator('[data-computed="ms"]')).toHaveText(byAge);
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Rückgängig und Wiederholen", () => {
  const reset = page => page.evaluate(() => { lastCommand = { id: "", t: 0 }; });

  test("100 Aktionen vollständig zurück- und wiederherstellen", async ({ page }) => {
    test.setTimeout(180_000);
    const errors = await open(page);
    const name = field(page, "project.name");
    const original = await name.inputValue();
    for (let i = 1; i <= 100; i++) {
      await name.fill(`Plan ${i}`);
      await page.evaluate(() => History.commit());
    }
    for (let i = 0; i < 100; i++) { await reset(page); await page.keyboard.press(`${MOD}+z`); }
    await expect(field(page, "project.name")).toHaveValue(original);
    await reset(page); await page.keyboard.press(`${MOD}+z`);
    await expect(page.locator("#toast")).toContainText("Nichts mehr rückgängig");
    for (let i = 0; i < 100; i++) { await reset(page); await page.keyboard.press(`${MOD}+Shift+z`); }
    await expect(field(page, "project.name")).toHaveValue("Plan 100");
    await reset(page); await page.keyboard.press(`${MOD}+Shift+z`);
    await expect(page.locator("#toast")).toContainText("Nichts zu wiederholen");
    await noErrors(page, errors);
  });

  test("über Abschnittswechsel hinweg", async ({ page }) => {
    const errors = await open(page);
    const name = await field(page, "project.name").inputValue();
    await field(page, "project.name").fill("Eins");
    await page.evaluate(() => History.commit());
    await page.keyboard.press(`${MOD}+6`);
    await field(page, "wealth.cash").fill("123456");
    await page.evaluate(() => History.commit());
    await page.keyboard.press(`${MOD}+3`);
    await reset(page); await page.keyboard.press(`${MOD}+z`);
    await page.keyboard.press(`${MOD}+6`);
    await expect(field(page, "wealth.cash")).not.toHaveValue("123'456");
    await reset(page); await page.keyboard.press(`${MOD}+z`);
    await page.keyboard.press(`${MOD}+1`);
    await expect(field(page, "project.name")).toHaveValue(name);
    await reset(page); await page.keyboard.press(`${MOD}+y`);
    await expect(field(page, "project.name")).toHaveValue("Eins");
    await noErrors(page, errors);
  });

  test("nach Laden einer Datei beginnt der Verlauf neu", async ({ page }) => {
    const errors = await open(page);
    await field(page, "project.name").fill("Vorher");
    await page.evaluate(() => History.commit());
    const plan = await page.evaluate(() => { const s = JSON.parse(JSON.stringify(state)); s.project.name = "Geladen"; return s; });
    const file = path.join(os.tmpdir(), `qa-undo-${process.pid}.vplan`);
    fs.writeFileSync(file, JSON.stringify(plan));
    await page.locator("#fileInput").setInputFiles(file);
    await expect(field(page, "project.name")).toHaveValue("Geladen");
    await reset(page); await page.keyboard.press(`${MOD}+z`);
    await expect(page.locator("#toast")).toContainText("Nichts mehr rückgängig");
    await expect(field(page, "project.name")).toHaveValue("Geladen");
    fs.rmSync(file);
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Dateien", () => {
  async function saveToFile(page, name) {
    const [dl] = await Promise.all([page.waitForEvent("download"), page.getByRole("button", { name: "Speichern", exact: true }).click()]);
    const file = path.join(os.tmpdir(), name);
    await dl.saveAs(file);
    return { file, suggested: dl.suggestedFilename() };
  }

  test("Speichern, Öffnen und Roundtrip ergeben identische Daten", async ({ page }) => {
    const errors = await open(page);
    await field(page, "project.name").fill("Roundtrip Test");
    const a = await saveToFile(page, `qa-rt-a-${process.pid}.vplan`);
    expect(a.suggested).toBe("Roundtrip Test.vplan");
    const first = JSON.parse(fs.readFileSync(a.file, "utf8"));
    expect(first.format).toBe("vivaplan/1");
    await page.locator("#fileInput").setInputFiles(a.file);
    await expect(page.locator("#toast")).toContainText("geöffnet");
    await expect(field(page, "project.name")).toHaveValue("Roundtrip Test");
    const b = await saveToFile(page, `qa-rt-b-${process.pid}.vplan`);
    const second = JSON.parse(fs.readFileSync(b.file, "utf8"));
    expect(second).toEqual(first);
    await page.locator("#fileInput").setInputFiles(b.file);
    const c = await saveToFile(page, `qa-rt-c-${process.pid}.vplan`);
    expect(fs.readFileSync(c.file, "utf8")).toBe(fs.readFileSync(b.file, "utf8"));
    for (const f of [a.file, b.file, c.file]) fs.rmSync(f);
    await noErrors(page, errors);
  });

  test("Öffnen einer fremden oder defekten Datei wird abgelehnt", async ({ page }) => {
    const errors = await open(page);
    const bad = path.join(os.tmpdir(), `qa-bad-${process.pid}.vplan`);
    for (const text of ["", "kein json", "{}", JSON.stringify({ format: "andere/9" })]) {
      fs.writeFileSync(bad, text);
      await page.locator("#fileInput").setInputFiles(bad);
      await expect(page.locator("#toast")).toContainText("keine Projektdatei");
      await expect(field(page, "project.name")).toBeVisible();
    }
    fs.rmSync(bad);
    await noErrors(page, errors);
  });

  test("Neu fragt bei ungespeicherten Änderungen nach", async ({ page }) => {
    const errors = await open(page);
    await field(page, "project.name").fill("Geändert");
    await expect(page.locator("#dirty")).toContainText("ungespeichert");
    await page.getByRole("button", { name: "Neu", exact: true }).click();
    await expect(page.locator("#ask")).toBeVisible();
    await page.locator("#ask button[value=cancel]").click();
    await expect(field(page, "project.name")).toHaveValue("Geändert");
    await page.getByRole("button", { name: "Neu", exact: true }).click();
    await page.locator("#ask button[value=ok]").click();
    await expect(field(page, "project.name")).not.toHaveValue("Geändert");
    await expect(page.locator("#dirty")).toHaveText("");
    await noErrors(page, errors);
  });

  test("Projektdatei ansehen zeigt das aktuelle JSON", async ({ page }) => {
    const errors = await open(page);
    await field(page, "project.name").fill("Sichtbar");
    await page.getByRole("button", { name: "Datei ansehen" }).click();
    await expect(page.locator("#dlg")).toBeVisible();
    const json = JSON.parse(await page.locator("#dlgPre").textContent());
    expect(json.project.name).toBe("Sichtbar");
    await page.locator("#dlg button", { hasText: "Schliessen" }).click();
    await expect(page.locator("#dlg")).not.toBeVisible();
    await noErrors(page, errors);
  });
});

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
    // Kennzahl in Mio oder k gerundet; Tabelle in CHF – Rundung prüfen
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
    // FEA-07: Steuerspalten sind standardmässig eingeklappt, die Schaltfläche blendet sie ein und aus
    await expect(page.locator("#resultsBody table.yt th.tx")).toHaveCount(0);
    await expect(header).not.toContainText("Steuer");
    await page.locator('button[data-action="tax-cols"]').click();
    await expect(page.locator("#resultsBody table.yt th.tx")).toHaveCount(5);
    expect(await page.locator("#resultsBody table.yt th.tx").nth(2).innerText()).toMatch(/^Steuer\s+Kapital Bezug$/);
    expect(await page.locator("#resultsBody table.yt th.tx").first().getAttribute("title")).toContain("Im Jahr bezahlt");
    await expect(page.locator("#resultsBody p.hint", { hasText: "Stände am Jahresende" })).toContainText("Steuerjahres 2029");
    const cell = await page.locator("#resultsBody table.yt tr").nth(1).locator("td").nth(11).textContent();
    expect(cell).toMatch(/^-?\d+[.,]\d %$/);
    await page.locator('button[data-action="tax-cols"]').click();
    await expect(page.locator("#resultsBody table.yt th.tx")).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("Einzelne Läufe: Auswahl bleibt stehen, nur der Inhalt scrollt; Variation statt Pfadname", async ({ page }) => {
    const errors = await open(page, "results/bootstrap/runs");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "runs");
    // Auswahl «Lauf wählen» liegt ausserhalb des scrollenden Bereichs
    await expect(page.locator("#resultsPick")).toContainText("Lauf wählen");
    await expect(page.locator("#resultsBody")).not.toContainText("Lauf wählen");
    const ov = await page.evaluate(() => ["#results", "#resultsPick", "#resultsBody"].map(s => getComputedStyle(document.querySelector(s)).overflowY));
    expect(ov).toEqual(["hidden", "visible", "auto"]);
    await page.locator("#resultsBody").evaluate(el => { el.scrollTop = 600; });
    const pick = await page.locator("#resultsPick").boundingBox();
    await page.locator("#resultsBody").evaluate(el => { el.scrollTop = 1200; });
    expect((await page.locator("#resultsPick").boundingBox()).y).toBe(pick.y);
    // kein Name des Zufallspfads
    await expect(page.locator("#results")).not.toContainText(/Bootstrap-Pfad|Zufallspfad/);
    // Perzentil und Wert der Variation stehen in der Auswahl
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
    test.setTimeout(180_000);
    const errors = await open(page);
    const out = await page.evaluate(async () => {
      state.returns.source = "bootstrap";
      state.returns.bootstrap.paths = 300;
      state.flows.spending = JSON.parse(JSON.stringify(state.flows.spending));
      const rows = state.flows.spending.rows ?? [];
      // so knapp, dass schlechte Läufe aufgebraucht werden: Lebenshaltung ohne die Wohnkosten (1'700 im Beispiel, FEA-39) × 1.6
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
      expect(row(p).at(-1)).toBe(text); // Spalte «Variation» ist die letzte
    }
    // Die Anzeige ist nicht mehr für alle Perzentile derselbe Wert, und nicht der Wert der Planung (100 %)
    const shown = out.expected.map(([, t]) => t);
    expect(new Set(shown).size).toBeGreaterThan(1);
    expect(out.one.depleted, "1. Perzentil ist in der Basisplanung aufgebraucht").toBeTruthy();
    expect(shown[0]).not.toContain("100 %");
    await expect(page.locator("#resultsBody .hint").filter({ hasText: "optimaler Wert je Perzentil" })).toBeVisible();
    await noErrors(page, errors);
  });

  test("Hypothekenerneuerung: Protokoll im Einzellauf, Zähler und Spalten bei mehreren Läufen", async ({ page }) => {
    test.setTimeout(180_000);
    const errors = await open(page);
    const out = await page.evaluate(async () => {
      state.returns.source = "fixed";
      state.scenarios.sweeps = [];
      setP("flows.income", { mode: "const", value: 6000 });
      setP("flows.spending", { mode: "const", value: 0 });
      state.flows.spendingIncludesHousing = false; // Wohnkosten zusätzlich (FEA-39)
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
    expect(out.single).not.toContain("Hypothekenerneuerungen"); // Abschnitt heisst «Immobilien»
    expect(out.single).toMatch(/nicht gegeben/);
    expect(out.single).toContain("Einmalrückzahlung Hypothek Haus");
    // FEA-22: keine Einmalzahlungen mehr in der Übersicht, Kennzahl «Notverkäufe» bleibt, Spalte «Immobilienverkäufe»
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
      resultZoom = null; stepZoom(5); out.fresh = { ...resultZoom };               // Mitte der ganzen Laufzeit
      setCustomZoom(10 * 12, 20 * 12); out.custom = { ...resultZoom };              // Mausauswahl 10–20 J.
      stepZoom(5); out.centered = { ...resultZoom };                                // 5 J. um die Mitte (15 J.)
      stepZoom(10); out.ten = { ...resultZoom };
      setCustomZoom(12, 36); stepZoom(5); out.nearStart = { ...resultZoom };        // Mitte 2 J. → verschoben statt abgeschnitten
      setCustomZoom(total - 36, total - 12); stepZoom(5); out.nearEnd = { ...resultZoom };
      stepZoom(100); out.tooLong = resultZoom;                                      // länger als die Simulation → alles
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
    test.setTimeout(180_000);
    const errors = await open(page, "results/charts/nosweep");
    const bars = () => page.locator("#resultsBody .minizoom");
    const n = await bars().count();
    expect(n).toBeGreaterThanOrEqual(5);
    // Ausgangslage: «Alles» überall aktiv
    expect(await page.locator("#resultsBody .minizoom button.on").allTextContents()).toEqual(Array(n).fill("Alles"));
    await page.locator("#rc-comp").scrollIntoViewIfNeeded();
    const box = await page.locator("#rc-comp").boundingBox();
    const y = box.y + box.height / 2;
    const before = await page.evaluate(() => document.querySelector("#rc-comp path").getAttribute("d").length);
    // zu kleine Auswahl (ein Klick) wird ignoriert
    await page.mouse.move(box.x + box.width * 0.5, y); await page.mouse.down(); await page.mouse.move(box.x + box.width * 0.5 + 3, y); await page.mouse.up();
    expect(await page.evaluate(() => resultZoom)).toBeNull();
    // Bereich aufziehen: während des Ziehens ist er hervorgehoben
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
    // alle Grafiken zeigen jetzt «benutzerdefiniert», keine Schaltfläche aktiv
    await expect(page.locator("#resultsBody .minizoom button.on")).toHaveCount(0);
    await expect(page.locator("#resultsBody .minizoom span").first()).toContainText("Benutzerdefiniert");
    const after = await page.evaluate(() => document.querySelector("#rc-comp path").getAttribute("d").length);
    expect(after).toBeLessThan(before);
    // Registerkarten: Übersicht zeigt denselben Zoom, Wechsel behält ihn
    await page.getByRole("button", { name: "Übersicht" }).click();
    expect(await page.evaluate(() => resultZoom.from)).toBe(z.from);
    await expect(page.locator("#resultsBody .minizoom span").first()).toContainText("Benutzerdefiniert");
    await page.getByRole("button", { name: "Grafiken" }).click();
    expect(await page.evaluate(() => resultZoom.to)).toBe(z.to);
    // «5 J.» danach: 5 Jahre um die Mitte des bisherigen Bereichs
    await page.locator('#resultsBody button[data-action="res-zoom"][data-years="5"]').first().click();
    const five = await page.evaluate(() => ({ ...resultZoom }));
    const mid = (z.from + z.to) / 2;
    expect(five.to - five.from).toBe(60);
    expect(Math.abs((five.from + five.to) / 2 - mid)).toBeLessThanOrEqual(1);
    expect(await page.locator("#resultsBody .minizoom button.on").allTextContents()).toEqual(Array(n).fill("5 J."));
    // Balkendiagramm: mehrere Jahre aufziehen zoomt ebenfalls
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
    // Editor-Minigrafiken: eigener Zoom, nicht gekoppelt
    await page.locator('#resultsBody button[data-action="res-zoom"][data-years="10"]').first().click();
    await page.keyboard.press("Escape");
    await expect(page.locator("#results")).toBeHidden();
    await page.locator("#nav a", { hasText: "Einnahmen" }).click();
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
    test.setTimeout(180_000);
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
    // Der Lauf der Stufe 50 % ist derselbe wie in den Szenarien (kein Näherungswert)
    expect(Math.abs(out.end50 - out.scen50)).toBeLessThan(1);
    expect(out.end50).toBeGreaterThan(out.planEnd); // weniger Ausgaben → mehr Endvermögen
    expect(out.back).toBe(out.planEnd);
    expect(out.optText).toMatch(/^Ausgaben-Faktor /);
    await noErrors(page, errors);
  });

  test("Einzellauf ohne Variationen: keine Variationsauswahl; Monte-Carlo mit Variation unverändert", async ({ page }) => {
    test.setTimeout(180_000);
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
    expect(out.mcPick).toBe(""); // bei Monte-Carlo keine Stufenauswahl
    expect(out.table).toMatch(/Variation/);
    expect(out.table).toMatch(/Ausgaben-Faktor (≈ |< |≥ )?\d/);
    await noErrors(page, errors);
  });

  test("Abschnitt «Immobilien»: Kauf, Verkauf, Erneuerung mit Einmalzahlung und Notverkauf mit Beträgen; ohne Ereignisse ein Hinweis", async ({ page }) => {
    test.setTimeout(180_000);
    const errors = await open(page);
    const out = await page.evaluate(async () => {
      state.returns.source = "fixed"; state.scenarios.sweeps = [];
      state.taxes.propertyGain = { brackets: [], reduction: 0 };   // Pflicht bei Immobilien; Satz 0 %, damit die Beträge unverändert bleiben
      setP("flows.income", { mode: "const", value: 6000 }); setP("flows.spending", { mode: "const", value: 0 });
      state.flows.spendingIncludesHousing = false; // Wohnkosten zusätzlich (FEA-39)
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
    // ohne Immobilien
    expect(out.none.text).toContain("Keine Immobilienereignisse");
    // Verkauf: Erlös, abgelöste Hypothek, Nettoerlös
    expect(out.sale.text).toContain("Verkauf");
    expect(out.sale.text).toMatch(/Verkaufserlös\s+800'000/);
    expect(out.sale.text).toMatch(/abgelöste Hypothek\s+400'000/);
    expect(out.sale.text).toMatch(/Nettoerlös\s+400'000/);
    expect(out.sale.text).toContain("Entnahmestrategie angewendet (kein Folgekauf)");
    // Kauf: Preis, Eigenmittel, Hypothek
    expect(out.buy.text).toContain("Kauf");
    expect(out.buy.text).toMatch(/Kaufpreis\s+600'000/);
    expect(out.buy.text).toMatch(/Eigenmittel\s+300'000/);
    expect(out.buy.text).toMatch(/Hypothek\s+300'000/);
    // Erneuerung mit Einmalzahlung: vorher/nachher, Zahlung, Zinssatz
    expect(out.lump.text).toContain("Hypothekenerneuerung");
    expect(out.lump.text).toMatch(/Hypothek vorher \/ nachher\s+400'000 \/ 320'000/);
    expect(out.lump.text).toMatch(/Einmalzahlung \(Tragbarkeit\)\s+80'000/);
    expect(out.lump.text).toMatch(/Zinssatz\s+1 %/);
    // Notverkauf mit Grund, in zeitlicher Reihenfolge nach der nicht erfolgten Erneuerung
    expect(out.emergency.text).toContain("Notverkauf");
    expect(out.emergency.text).toContain("Vermögen < 0");
    const kinds = out.emergency.events.map(e => e.kind);
    expect(kinds).toEqual(["renewal", "emergencySale"]);
    expect(out.emergency.events[0].month < out.emergency.events[1].month).toBe(true);
    await noErrors(page, errors);
  });

  test("Tabellenkopf der Ergebnisse bleibt beim Scrollen oben stehen", async ({ page }) => {
    test.setTimeout(180_000);
    const errors = await open(page);
    const out = await page.evaluate(async () => {
      state.returns.source = "fixed"; state.scenarios.sweeps = [];
      lastRun = await Engine.call("run_simulation", { plan: state, path: null });
      resetVariation(); resultTab = "table"; showResults(true); renderResults(lastRun);
      await new Promise(r => setTimeout(r, 100));
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
    test.setTimeout(180_000);
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
    // FEA-08: keine eigene Karte; FEA-09: keine 3-Topf-Buchungen in der Ereignisliste
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
    // Speicher nicht lesbar: eingeklappt
    await page.evaluate(() => localStorage.setItem("rp.settings", "kein json"));
    await page.reload();
    await page.waitForSelector("html[data-ready='1']");
    await page.locator("button[data-action='rtab'][data-tab='table']").click();
    await expect(page.locator("#resultsBody table.yt th.tx")).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("Reiter Parameter: Stand des Laufs, schreibgeschützt, je Editorabschnitt ein Parameter wirkt, alte Ergebnisse «Nicht verfügbar»", async ({ page }) => {
    test.setTimeout(120_000);
    const errors = await open(page, "results/charts/nosweep");
    await page.locator("button[data-action='rtab'][data-tab='params']").click();
    const body = page.locator("#resultsBody");
    await expect(body).toContainText("Haushalt");
    for (const t of ["Meilensteine", "Vorsorge", "Einnahmen und Ausgaben", "Vermögen und Anlage", "Renditequelle und Zeitreihe", "Entnahmestrategie", "Immobilien", "Steuerannahmen", "Variationen"]) await expect(body).toContainText(t);
    await expect(body).toContainText("Geburt");
    await expect(body).not.toContainText("Person A"); // Namen gehören nicht dazu
    await expect(body.locator("input, select, textarea")).toHaveCount(0);
    const text = async () => (await body.innerText()).replace(/^ⓘ\n/gm, "");
    const before = await text();
    expect(before).not.toContain("[object");
    // Editor ändern: das berechnete Ergebnis behält seine Parameter
    await page.evaluate(() => { state.wealth.cash += 12345; state.taxes.multiplier = 7; });
    await page.locator("button[data-action='rtab'][data-tab='overview']").click();
    await page.locator("button[data-action='rtab'][data-tab='params']").click();
    expect(await text()).toBe(before);
    // Je Editorabschnitt ein Parameter ändern: nur dieser Abschnitt der Zusammenfassung ändert sich
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
    // Ergebnis ohne gespeicherte Planung
    await page.evaluate(() => { lastRunPlan = null; renderResults(lastRun); });
    await page.locator("button[data-action='rtab'][data-tab='params']").click();
    await expect(body).toContainText("Nicht verfügbar");
    await noErrors(page, errors);
  });

  test("Gespeichertes Ergebnis zeigt im Reiter Parameter die Planung von damals", async ({ page }) => {
    test.setTimeout(240_000);
    const errors = await open(page, "runsflow/mem/view");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "saved", { timeout: 180_000 });
    await page.locator("button[data-action='rtab'][data-tab='params']").click();
    await expect(page.locator("#resultsBody")).toContainText("Aktienanteil");
    await expect(page.locator("#resultsBody")).toContainText("Monte Carlo");
    await noErrors(page, errors);
  });

  test("Jahreswerte: Entnahmerate (Jahresentnahme / Vermögen am Jahresanfang), nominal wie real", async ({ page }) => {
    test.setTimeout(120_000);
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

  test("FEA-22: Immobilienverkäufe je Perzentil-Lauf mit Datum, Notverkäufe mit Warnsymbol, ohne Immobilien keine Spalte", async ({ page }) => {
    test.setTimeout(180_000);
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

  test("FEA-20: Variationen ohne feste Grenze (technisch 1'000'000), Warnung vor langen Berechnungen", async ({ page }) => {
    test.setTimeout(120_000);
    const errors = await open(page, "scenarios");
    const run = async sweeps => {
      await page.evaluate(s => { state.scenarios.sweeps = s; renderSection(); renderDerived(); }, sweeps);
      await page.waitForTimeout(300);
      return page.evaluate(async () => {
        const out = await Engine.call("validate", { plan: state });
        return { text: document.querySelector('[data-computed="runs"]')?.innerText || "", errors: out.filter(i => i.severity === "error").map(i => i.message) };
      });
    };
    const sw = (param, from, to, step = 0.1) => ({ param, from, to, step });
    // 1'100 Kombinationen (früher abgelehnt): erlaubt, keine Grenzmeldung
    const many = await run([sw("wealth.stockShare", 0, 1), sw("pension.pk.lumpsum", 0, 0.9), sw("flows.spendingFactor", 1, 1.9)]);
    expect(many.text).not.toContain("Grenze");
    expect(many.text).toContain("1'100");
    expect(many.errors.filter(e => e.includes("Kombinationen"))).toEqual([]);
    // über 1'000'000: technische Meldung, ohne die App zu belasten
    const over = await run([sw("wealth.stockShare", 0, 1, 0.01), sw("pension.pk.lumpsum", 0, 0.99, 0.01), sw("flows.spendingFactor", 1, 1.99, 0.01)]);
    expect(over.text).toContain("technische Grenze");
    expect(over.errors).toContain("Die Variationen ergeben 1'010'000 Kombinationen; höchstens 1'000'000 sind technisch möglich.");
    // Warnung ab einer Minute geschätzter Rechenzeit
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
    // 30 s: keine Rückfrage
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
    // «Simulation starten» bleibt die Aktion
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

  test("FEA-21: Legenden zeigen gestrichelte Linien als gestricheltes Symbol (Übersicht und Grafiken)", async ({ page }) => {
    test.setTimeout(120_000);
    const errors = await open(page, "results/bootstrap");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "overview", { timeout: 90_000 });
    const dashes = () => page.evaluate(() => [...document.querySelectorAll("#resultsBody .legend")].map(l => ({
      text: l.innerText.replace(/\s+/g, " "), dashed: [...l.querySelectorAll("svg.lsym line")].filter(x => x.getAttribute("stroke-dasharray")).length,
      solid: [...l.querySelectorAll("svg.lsym line")].filter(x => !x.getAttribute("stroke-dasharray")).length })));
    const fan = (await dashes()).find(l => l.text.includes("Schlechtester Wert"));
    expect(fan.text).toContain("1. und 99. Perzentil");
    expect(fan.dashed).toBe(1);
    expect(fan.solid).toBe(2); // Median und schlechtester Wert
    await page.getByRole("button", { name: "Grafiken" }).click();
    await expect(page.locator("#mc-end path, #mc-end rect").first()).toBeAttached();
    const all = await dashes();
    const fan2 = all.find(l => l.text.includes("Schlechtester Wert"));
    expect({ ...fan2, text: "" }).toEqual({ ...fan, text: "" });
    const end = all.find(l => l.text.includes("95. und 99. Perzentil"));
    expect(end.dashed).toBe(1);
    // Die senkrechten gestrichelten Linien im Histogramm: fünf, jede mit Legendeneintrag (ein Eintrag für die Linienart)
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
    // Perzentil-Reihenfolge der Endvermögen
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
    test.setTimeout(180_000); // 9 Kombinationen mit je 50 000 Pfaden, bei parallelen Tests entsprechend langsam
    const errors = await open(page, "results/bootstrap/scenarios/sweep2");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "scenarios", { timeout: 150_000 });
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
    test.setTimeout(180_000);
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

// ---------------------------------------------------------------------------------------------
test.describe("Gespeicherte Ergebnisse", () => {
  test("zwei Läufe ablegen und vergleichen", async ({ page }) => {
    test.setTimeout(240_000); // zwei Simulationen (Testzugang runsflow, 1000 Pfade)
    const errors = await open(page, "runsflow/mem");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "compare", { timeout: 180_000 });
    await expect(page.locator("#resultsBody")).toContainText("Vergleich");
    await noErrors(page, errors);
  });

  test("gespeicherte Simulation ansehen", async ({ page }) => {
    test.setTimeout(240_000);
    const errors = await open(page, "runsflow/mem/view");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "saved", { timeout: 180_000 });
    await expect(page.locator("#resultsBody svg path").first()).toBeAttached();
    await noErrors(page, errors);
  });

  test("Simulationen einzeln markieren und löschen, alle markieren, alle löschen", async ({ page }) => {
    test.setTimeout(240_000);
    const errors = await open(page, "runsflow/mem/del");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "runs", { timeout: 180_000 });
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
    test.setTimeout(240_000);
    const errors = await open(page, "runsflow/mem/del");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "runs", { timeout: 180_000 });
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
    const cmp = await page.locator("#resultsBody table").first().locator("tr:first-child th").allInnerTexts();
    expect(cmp).toContain("Zeitreihe");
    expect(cmp).not.toContain("Art");
    await noErrors(page, errors);
  });

  test("FEA-12: Liste zeigt «Erfolgsquote · 5 %» statt Median und das Warnsymbol für ungeplante Immobilienverkäufe ab 5 % der Läufe", async ({ page }) => {
    const errors = await open(page, "runsflow/mem/del");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "runs", { timeout: 180_000 });
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

// ---------------------------------------------------------------------------------------------
test.describe("Optimierung: Bandbreiten, Entnahmerate, Fortschritt", () => {
  test("Parameter fix oder variiert; Standard-Bandbreiten stellen wieder her", async ({ page }) => {
    const errors = await open(page, "optimize");
    await page.waitForSelector("#optR-three_buckets-cashYears-var", { state: "attached" });
    await page.locator("details.optrange:has(#optR-three_buckets-cashYears-var) summary").click();
    const lo = page.locator("#optR-three_buckets-bondYears-lo");
    await page.locator("#optR-three_buckets-cashYears-var").uncheck();
    await expect(page.locator("#optR-three_buckets-cashYears-lo")).toBeDisabled();
    await lo.fill("0"); await lo.blur();
    await page.locator("#optR-three_buckets-bondYears-hi").fill("2");
    await page.locator("#optR-three_buckets-bondYears-hi").blur();
    const r = await page.evaluate(() => state.optimization.ranges.three_buckets);
    expect(r.cashYears.fixed).toBe(true);
    expect(r.bondYears.hi).toBe(2);
    await expect(page.locator("details.optrange summary .computed").nth(1)).toContainText("Parametern variiert");
    // eingeklappt/ausgeklappt zeigen alle Strategien; Standard-Bandbreiten löschen die Änderungen
    await page.locator('button[data-action="opt-range-reset"][data-s="three_buckets"]').click();
    expect(await page.evaluate(() => state.optimization?.ranges?.three_buckets)).toBeUndefined();
    await noErrors(page, errors);
  });

  test("Optimierung läuft mit Fortschritt, Entnahmerate erscheint im Ergebnis", async ({ page }) => {
    test.setTimeout(180_000);
    const errors = await open(page, "optimize/bootstrap/optset");
    const small = { paths: 300, checkPaths: 500, samples: 6, strategies: ["fixed_rate"] };
    expect(await setOpt(page, small)).toMatchObject(small); // ISS-16: die kleine Rechnung kommt wirklich in den Feldern an
    await page.locator('button[data-action="opt-run"]').click();
    await expect(page.locator("#main .job progress")).toBeVisible();
    await expect(page.locator("html")).toHaveAttribute("data-job", "off", { timeout: 150_000 });
    await expect(page.locator("#main th", { hasText: "Entnahmerate" })).toBeVisible();
    await noErrors(page, errors);
  });

  test("FEA-05/AK1, AK9, AK10: 3-Topf mit Guardrails – Auswahl, Karten, Name, Tooltip und Optimierung", async ({ page }) => {
    const errors = await open(page);
    await page.evaluate(() => { state.withdrawal.strategy = "three_buckets"; delete state.withdrawal.buckets.spending; goto("withdrawal"); });
    const sel = page.locator('#main select[data-path="withdrawal.buckets.spending"]');
    // AK1: Standard «wie geplant», ohne Guardrails- und Grenzen-Karte
    await expect(sel).toHaveValue("planned");
    await expect(page.locator('#main input[data-path="withdrawal.guardrails.initialRate"]')).toHaveCount(0);
    await expect(page.locator('#main input[data-path="withdrawal.floor"]')).toHaveCount(0);
    await sel.selectOption("guardrails");
    await expect(page.locator('#main input[data-path="withdrawal.guardrails.initialRate"]')).toBeVisible();
    await expect(page.locator('#main input[data-path="withdrawal.floor"]')).toBeVisible();
    await expect(page.locator('#main input[data-path="withdrawal.buckets.cashYears"]')).toBeVisible();
    expect(await page.evaluate(() => state.withdrawal.buckets.spending)).toBe("guardrails");
    // AK9: Name und Parameter (Tooltip FEA-10) mit Töpfen und Guardrails
    const info = await page.evaluate(() => ({ name: stratName(stratKey(state.withdrawal)), params: stratParams(state).map(x => x[0]) }));
    expect(info.name).toBe("3-Topf + Guardrails");
    expect(info.params).toEqual(expect.arrayContaining(["Topf 1 · Cash", "Startrate bei Pensionierung", "Untergrenze"]));
    expect(await page.evaluate(() => stratName(stratKey({ strategy: "three_buckets", buckets: { spending: "planned" } })))).toBe("3-Topf-Strategie");
    // AK10: In der Optimierung angeboten, standardmässig nicht gewählt
    await page.evaluate(() => { delete state.optimization; goto("optimize"); });
    const box = page.locator("#optS-three_buckets_guardrails");
    await expect(box).toBeAttached();
    await expect(box).not.toBeChecked();
    await expect(page.locator("#optS-three_buckets")).toBeChecked();
    // Zurück auf «wie geplant»: Karten verschwinden
    await page.evaluate(() => goto("withdrawal"));
    await sel.selectOption("planned");
    await expect(page.locator('#main input[data-path="withdrawal.guardrails.initialRate"]')).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("FEA-57: Quote von Aktien und Obligationen neben den Beständen", async ({ page }) => {
    const errors = await open(page);
    await page.evaluate(() => goto("wealth"));
    const quote = p => page.locator(`label.field:has(input[data-path="${p}"]) [data-computed="wquote"]`);
    const set = async (p, v) => { await field(page, p).fill(v); await field(page, p).blur(); };
    // AK1: 80 000 / 20 000 / 50 000
    await set("wealth.stocks", "80000"); await set("wealth.bonds", "20000"); await set("wealth.cash", "50000");
    await expect(quote("wealth.stocks")).toHaveText("80 % des Portfolios");
    await expect(quote("wealth.bonds")).toHaveText("20 % des Portfolios");
    // AK5: Cash ändert die Quoten nicht
    for (const c of ["0", "500000"]) {
      await set("wealth.cash", c);
      await expect(quote("wealth.stocks")).toHaveText("80 % des Portfolios");
      await expect(quote("wealth.bonds")).toHaveText("20 % des Portfolios");
    }
    // AK3: während der Eingabe unverändert, nach dem Verlassen des Feldes nachgeführt
    await field(page, "wealth.stocks").fill("60000");
    await expect(quote("wealth.stocks")).toHaveText("80 % des Portfolios");
    await field(page, "wealth.stocks").blur();
    await expect(quote("wealth.stocks")).toHaveText("75 % des Portfolios");
    await expect(quote("wealth.bonds")).toHaveText("25 % des Portfolios");
    // AK2: ganzzahlig gerundet
    await set("wealth.stocks", "1"); await set("wealth.bonds", "2");
    await expect(quote("wealth.stocks")).toHaveText("33 % des Portfolios");
    await expect(quote("wealth.bonds")).toHaveText("67 % des Portfolios");
    // AK4: beide 0 bzw. leer: keine Quote
    await set("wealth.stocks", "0"); await set("wealth.bonds", "0");
    await expect(quote("wealth.stocks")).toHaveText("");
    await expect(quote("wealth.bonds")).toHaveText("");
    await set("wealth.stocks", ""); await set("wealth.bonds", "");
    await expect(quote("wealth.stocks")).toHaveText("");
    await expect(page.locator("#main")).not.toContainText("NaN");
    // AK6: die Quote steht nicht in der Planung
    const example = await (await page.request.post("/api/example_plan", { data: {} })).json();
    const keys = Object.keys(await page.evaluate(() => state.wealth));
    expect(keys.filter(k => !(k in example.wealth))).toEqual([]);
    expect(JSON.stringify(await page.evaluate(() => state))).not.toContain("des Portfolios");
    await noErrors(page, errors);
  });

  test("FEA-17: Cash-Reserve nur in der Karte «Standard»; Hinweis beim Laden bei anderer Strategie und Reserve ≠ 3", async ({ page }) => {
    const errors = await open(page);
    // (1) «Vermögen & Anlage» enthält die Reserve nicht mehr, die Karte «Standard» schon
    await page.evaluate(() => goto("wealth"));
    await expect(page.locator("#main")).toContainText("Aktienquote (Aktien am Portfolio aus Aktien und Obligationen)");
    await expect(page.locator("#main")).not.toContainText("Cash-Reserve (Monate");
    await expect(page.locator('#main [data-path="wealth.reserveMonths"], #main [data-series="wealth.reserveMonths"]')).toHaveCount(0);
    await page.evaluate(() => { state.withdrawal.strategy = "standard"; goto("withdrawal"); });
    await expect(page.locator('#main input[data-path^="wealth.reserveMonths"]').first()).toBeVisible();
    // andere Strategie: keine Reserve einstellbar
    await page.evaluate(() => { state.withdrawal.strategy = "guardrails"; goto("withdrawal"); });
    await expect(page.locator('#main input[data-path^="wealth.reserveMonths"]')).toHaveCount(0);
    // (3) Planung mit Guardrails und Reserve 6: Hinweis beim Laden, einmal
    const plan = await page.evaluate(() => { const s = JSON.parse(JSON.stringify(state)); s.withdrawal.strategy = "guardrails"; s.wealth.reserveMonths = { mode: "const", value: 6 }; return s; });
    const file = path.join(os.tmpdir(), `fea17-${process.pid}.vplan`);
    fs.writeFileSync(file, JSON.stringify(plan));
    await page.locator("#fileInput").setInputFiles(file);
    await expect(page.locator("#ask")).toBeVisible();
    await expect(page.locator("#askTitle")).toHaveText("Hinweis zur Cash-Reserve");
    await expect(page.locator("#askText")).toContainText("3 Monaten");
    await page.locator('#ask button[value="ok"]').click();
    await expect(page.locator("#ask")).toBeHidden();
    // FEA-58/AK3, AK5: einmal gezeigt, gemerkt in den Einstellungen; erneutes Öffnen zeigt keinen Hinweis
    expect(await page.evaluate(() => Pref.get("reserveNoticeShown", false))).toBe(true);
    await page.locator("#fileInput").setInputFiles(file);
    await expect(page.locator("#fileName")).toHaveText(path.basename(file));
    await page.waitForTimeout(300);
    await expect(page.locator("#ask")).toBeHidden();
    // Standard mit Reserve 6: kein Hinweis; der gespeicherte Wert bleibt
    expect(await page.evaluate(() => reserveDiffers({ withdrawal: { strategy: "standard" }, wealth: { reserveMonths: { mode: "const", value: 6 } } }))).toBe(false);
    expect(await page.evaluate(() => reserveDiffers({ withdrawal: { strategy: "vpw" }, wealth: { reserveMonths: { mode: "const", value: 3 } } }))).toBe(false);
    expect(await page.evaluate(() => reserveDiffers({ withdrawal: { strategy: "vpw" }, wealth: { reserveMonths: { mode: "rows", rows: [{ value: 3 }, { value: 12 }] } } }))).toBe(true);
    expect(await page.evaluate(() => state.wealth.reserveMonths.value)).toBe(6);
    fs.rmSync(file);
    await noErrors(page, errors);
  });

  test.describe("FEA-02: Start mit zuletzt geöffneter Planung, Rückfrage beim Beenden (Desktop-App simuliert)", () => {
    /** Simuliert die Desktop-App: Befehle der Tauri-Schicht werden nachgebildet, alle anderen gehen an den Rechenkern. */
    async function openApp(page, startup, settings = {}) {
      settings = { disclaimerAccepted: 1, ...settings }; // FEA-45: Haftungsausschluss gilt als bestätigt, ausser ein Test sagt anders
      await page.addInitScript(([startup, settings]) => {
        window.__calls = [];
        window.__TAURI__ = {
          core: {
            invoke: async (cmd, args = {}) => {
              window.__calls.push([cmd, args]);
              if (cmd === "startup_plan") return startup;
              if (cmd === "save_plan") return window.__saveResult ?? null;
              if (["set_dirty", "quit_app", "settings_set", "set_window_title"].includes(cmd)) return null;
              if (cmd === "settings_get") return settings;
              const r = await fetch(`/api/${cmd}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(args) });
              const j = await r.json(); if (!r.ok) throw new Error(j.error); return j;
            }
          },
          event: { listen: (name, cb) => { (window.__listeners ??= {})[name] = cb; return Promise.resolve(() => {}); } }
        };
      }, [startup, settings]);
      await page.goto("/");
      await page.waitForSelector("html[data-ready='1'], html[data-disclaimer='open']");
    }
    const plan = async page => (await (await page.request.post("/api/example_plan", { data: {} })).json());

    test("FEA-45/AK1, AK2: Haftungsausschluss beim ersten Start; ohne Häkchen und mit Esc geht es nicht weiter; Bestätigung in den Einstellungen", async ({ page }) => {
      const p = await plan(page);
      await openApp(page, { path: null, plan: p, notice: null, first: true }, { disclaimerAccepted: null });
      const d = page.locator("#disclaimer"), ok = page.locator("#disclaimerOk");
      await expect(d).toHaveAttribute("open", "");
      await expect(d).toContainText("VivaPlan ist ein kostenloses Hilfsmittel zur persönlichen Planung.");
      await expect(d).toContainText("Prüfen Sie wichtige Entscheidungen mit einer Fachperson.");
      await expect(d).toContainText("Ich habe den Hinweis gelesen und verstanden.");
      // Die Bedienung wartet, bis bestätigt ist
      await expect(page.locator("html")).not.toHaveAttribute("data-ready", "1");
      await expect(ok).toBeDisabled();
      await page.keyboard.press("Escape");
      await expect(d).toHaveAttribute("open", "");
      await page.keyboard.press(`${MOD}+,`);
      await expect(page.locator("#settings")).not.toHaveAttribute("open", "");
      await page.locator("#disclaimerCheck").check();
      await expect(ok).toBeEnabled();
      await page.locator("#disclaimerCheck").uncheck();
      await expect(ok).toBeDisabled();
      await page.locator("#disclaimerCheck").check();
      await ok.click();
      await expect(d).not.toHaveAttribute("open", "");
      await expect(page.locator("html")).toHaveAttribute("data-ready", "1");
      const set = await page.evaluate(() => window.__calls.filter(([c]) => c === "settings_set").map(([, a]) => a));
      expect(set).toContainEqual({ key: "disclaimerAccepted", value: 1 });
      // nicht in der Planung
      expect(JSON.stringify(await page.evaluate(() => state))).not.toContain("disclaimer");
    });

    test("FEA-45/AK2: bestätigte Version fragt nicht mehr, eine ältere Version fragt erneut", async ({ page }) => {
      const p = await plan(page);
      await openApp(page, { path: "/x/A.vplan", plan: p, notice: null }, { disclaimerAccepted: 1 });
      await expect(page.locator("#disclaimer")).not.toHaveAttribute("open", "");
      await openApp(page, { path: "/x/A.vplan", plan: p, notice: null }, { disclaimerAccepted: 0 });
      await expect(page.locator("#disclaimer")).toHaveAttribute("open", "");
    });

    test("gemerkte Datei: Name im Kopf, kein Hinweis; ohne Merkung Beispielplanung als «Neue Planung»", async ({ page }) => {
      const p = await plan(page);
      p.project.name = "Gemerkte Planung";
      await openApp(page, { path: "/x/Familie.vplan", plan: p, notice: null });
      await expect(page.locator("#fileName")).toHaveText("Familie.vplan");
      await expect(page.locator("#ask")).toBeHidden();
      await expect(field(page, "project.name")).toHaveValue("Gemerkte Planung");
    });

    /** Planung mit Guardrails und Reserve 6 Monate (betroffen vom Hinweis zur Cash-Reserve). */
    const affected = async page => { const p = await plan(page); p.withdrawal.strategy = "guardrails"; p.wealth.reserveMonths = { mode: "const", value: 6 }; return p; };

    test("FEA-58/AK1, AK5: erster Start mit betroffener Planung zeigt den Hinweis einmal und merkt ihn in den Einstellungen", async ({ page }) => {
      await openApp(page, { path: "/x/Alt.vplan", plan: await affected(page), notice: null });
      await expect(page.locator("#ask")).toBeVisible();
      await expect(page.locator("#askTitle")).toHaveText("Hinweis zur Cash-Reserve");
      const set = await page.evaluate(() => window.__calls.filter(([c]) => c === "settings_set").map(([, a]) => a));
      expect(set).toContainEqual({ key: "reserveNoticeShown", value: true });
    });

    test("FEA-58/AK2: weiterer Start mit betroffener Planung zeigt keinen Hinweis", async ({ page }) => {
      await openApp(page, { path: "/x/Alt.vplan", plan: await affected(page), notice: null }, { reserveNoticeShown: true });
      await expect(page.locator("#fileName")).toHaveText("Alt.vplan");
      await page.waitForTimeout(300);
      await expect(page.locator("#ask")).toBeHidden();
    });

    test("FEA-58/AK4: nicht betroffene Planung zählt nicht; die erste betroffene zeigt den Hinweis, die nächste nicht", async ({ page }) => {
      await openApp(page, { path: "/x/Standard.vplan", plan: await plan(page), notice: null });
      await expect(page.locator("#fileName")).toHaveText("Standard.vplan");
      await page.waitForTimeout(300);
      await expect(page.locator("#ask")).toBeHidden();
      expect(await page.evaluate(() => Pref.get("reserveNoticeShown", false))).toBe(false);
      const p = await affected(page);
      await page.evaluate(p => { reserveNotice(p); }, p);
      await expect(page.locator("#askTitle")).toHaveText("Hinweis zur Cash-Reserve");
      await page.locator('#ask button[value="ok"]').click();
      await expect(page.locator("#ask")).toBeHidden();
      await page.evaluate(p => { reserveNotice(p); }, p);
      await page.waitForTimeout(300);
      await expect(page.locator("#ask")).toBeHidden();
    });

    test("Datei fehlt: Beispielplanung als «Neue Planung» mit Hinweis und Dateiname", async ({ page }) => {
      await openApp(page, { path: null, plan: await plan(page), notice: "Zuletzt geöffnete Datei nicht gefunden: weg.vplan. Es wird die Beispielplanung geöffnet." });
      await expect(page.locator("#fileName")).toHaveText("Neue Planung");
      await expect(page.locator("#ask")).toBeVisible();
      await expect(page.locator("#askText")).toContainText("Zuletzt geöffnete Datei nicht gefunden");
      await expect(page.locator("#askText")).toContainText("weg.vplan");
    });

    test("Beenden: ohne Änderungen sofort, mit Änderungen Dialog Speichern/Verwerfen/Abbrechen", async ({ page }) => {
      await openApp(page, { path: null, plan: await plan(page), notice: null });
      const calls = name => page.evaluate(n => window.__calls.filter(c => c[0] === n), name);
      // ohne Änderungen: direkt beenden
      await page.evaluate(() => { window.__listeners["quit-requested"](); });
      await expect.poll(async () => (await calls("quit_app")).length).toBe(1);
      // mit Änderungen meldet die Oberfläche den Zustand und fragt
      await field(page, "project.name").fill("Geändert");
      await field(page, "project.name").blur();
      await expect.poll(async () => (await calls("set_dirty")).some(c => c[1].dirty === true)).toBe(true);
      await page.evaluate(() => { window.__listeners["quit-requested"](); });
      await expect(page.locator("#ask")).toBeVisible();
      await expect(page.locator('#ask button[value="ok"]')).toHaveText("Speichern");
      await expect(page.locator('#ask button[value="discard"]')).toBeVisible();
      // Abbrechen: bleibt offen
      await page.locator('#ask button[value="cancel"]').click();
      await expect(page.locator("#ask")).toBeHidden();
      expect((await calls("quit_app")).length).toBe(1);
      // Verwerfen: beendet
      await page.evaluate(() => { window.__listeners["quit-requested"](); });
      await page.locator('#ask button[value="discard"]').click();
      await expect.poll(async () => (await calls("quit_app")).length).toBe(2);
    });
    test("Beenden: Speichern beendet erst nach erfolgreichem Speichern; bricht die Ortsabfrage ab, bleibt die App offen (ISS-6)", async ({ page }) => {
      await openApp(page, { path: null, plan: await plan(page), notice: null });
      const calls = name => page.evaluate(n => window.__calls.filter(c => c[0] === n), name);
      await field(page, "project.name").fill("Geändert"); await field(page, "project.name").blur();
      await page.evaluate(() => { window.__listeners["quit-requested"](); });
      await page.locator('#ask button[value="ok"]').click();
      await expect.poll(async () => (await calls("save_plan")).length).toBe(1);
      await expect(page.locator("#ask")).toBeHidden();
      expect((await calls("quit_app")).length).toBe(0);   // Ortsabfrage abgebrochen: App bleibt offen
      await page.evaluate(() => { window.__saveResult = "/x/gespeichert.vplan"; window.__listeners["quit-requested"](); });
      await page.locator('#ask button[value="ok"]').click();
      await expect.poll(async () => (await calls("quit_app")).length).toBe(1);
      await expect(page.locator("#fileName")).toHaveText("gespeichert.vplan");
    });
  });

  test("Meilenstein von «Datum» auf «Alter von» umstellen setzt Person und Jahre (Planung bleibt gültig)", async ({ page }) => {
    const errors = await open(page);
    await page.evaluate(() => goto("milestones"));
    const i = await page.evaluate(() => state.milestones.findIndex(m => m.id === "retire"));
    await page.locator(`select[data-path="milestones.${i}.kind"]`).selectOption("age");
    const ms = await page.evaluate(() => state.milestones.find(m => m.id === "retire"));
    expect(ms.kind).toBe("age");
    expect(ms.person).toBe("A");
    expect(ms.years).toBeGreaterThan(0);
    expect(await page.evaluate(() => Engine.call("validate", { plan: state }).then(r => r.filter(i => i.severity === "error")))).toEqual([]);
    await noErrors(page, errors);
  });

  test("FEA-23: Stufe der Variationen in «Einzelne Läufe» und «Jahreswerte» wählen, ohne Neuberechnung aller Pfade", async ({ page }) => {
    test.setTimeout(180_000);
    const errors = await open(page);
    const out = await page.evaluate(async () => {
      state.returns.source = "bootstrap"; state.returns.bootstrap.paths = 200;
      state.scenarios.sweeps = [{ param: "wealth.stockShare", from: 0.3, to: 0.6, step: 0.3 }, { param: "pension.pk.lumpsum", from: 0.2, to: 0.6, step: 0.4 }];
      lastRun = await Engine.call("run_simulation", { plan: state, path: null });
      resetVariation(); resultTab = "runs"; showResults(true); renderResults(lastRun);
      const calls = []; const orig = Engine.call.bind(Engine);
      Engine.call = (c, a) => { calls.push(c); return orig(c, a); };
      const selects = document.querySelectorAll('select[data-action="var-sel"]').length;
      const sc = lastRun.scenarios;
      const lvl = sc.results.find(x => x.values[0] === 0.3 && x.values[1] === 0.6);
      await pickVariation([0.3, 0.6]);
      const shown = document.querySelector("#resultsPick").textContent;
      const endShown = detailRun50().summary.endTotal;
      function detailRun50() { return detailsOf(lastRun).find(([q]) => q === 50)[1]; }
      const expected = lvl.percentiles.find(p => p.percent === 50).endTotal;
      resultTab = "table"; renderResults(lastRun);
      const tableHasPicker = document.querySelectorAll('select[data-action="var-sel"]').length;
      resultTab = "overview"; renderResults(lastRun);
      const note = document.querySelector("#resultsPick").textContent;
      resultTab = "runs"; renderResults(lastRun);
      await pickVariation([null, null]);
      Engine.call = orig;
      return { selects, calls, endShown, expected, tableHasPicker, note, back: variationSel, shown };
    });
    expect(out.selects).toBe(2);
    expect(out.calls).toEqual(["rerun_paths"]);                 // keine vollständige Neuberechnung
    expect(out.endShown).toBeCloseTo(out.expected, 4);          // stimmt mit der Szenarientabelle überein
    expect(out.tableHasPicker).toBe(2);                          // gilt auch in «Jahreswerte»
    expect(out.note).toContain("Planwerte, nicht die gewählte Variation");
    expect(out.back).toBeNull();
    await noErrors(page, errors);
  });

  test("FEA-19: Zielauswahl, Ziel 2 «Kein Notverkauf von Immobilien» wird gespeichert, gerechnet und mit Anteil ungeplanter Verkäufe angezeigt", async ({ page }) => {
    test.setTimeout(240_000);
    const errors = await open(page, "optimize/bootstrap");
    // (1) Auswahl vorhanden, Standard ist Ziel 1, der Plan speichert nichts Zusätzliches
    await expect(page.locator("#opt-goal")).toHaveValue("consumption");
    await expect(page.locator("#opt-weightMedian")).toBeVisible();
    expect(await page.evaluate(() => state.optimization?.goal)).toBeUndefined();
    await expect(page.locator("#opt-goal option[value=keepProperty]")).toHaveText("2 · Kein Notverkauf von Immobilien");
    // Ziel 2 wählen: Mindestkonsum erscheint, Konsumgewichte entfallen, Wert steht in der Planung
    await page.locator("#opt-goal").selectOption("keepProperty");
    await expect(page.locator("#opt-minConsumption")).toBeVisible();
    await expect(page.locator("#opt-weightMedian")).toHaveCount(0);
    expect(await page.evaluate(() => state.optimization.goal)).toBe("keepProperty");
    await page.locator("#opt-minConsumption").fill("80"); await page.locator("#opt-minConsumption").blur();
    expect(await page.evaluate(() => state.optimization.minConsumption)).toBe(0.8);
    const small = { paths: 200, checkPaths: 300, samples: 3, strategies: ["standard", "fixed_rate"] };
    expect(await setOpt(page, small)).toMatchObject({ ...small, goal: "keepProperty", minConsumption: 80 }); // ISS-16
    await page.locator('button[data-action="opt-run"]').click();
    await expect(page.locator("html")).toHaveAttribute("data-job", "off", { timeout: 200_000 });
    await expect(page.locator("#main th", { hasText: "Ungeplanter Verkauf" })).toBeVisible();
    await expect(page.locator("#main h3", { hasText: "Ziel 2" })).toBeVisible();
    // Reihenfolge: Anteil ungeplanter Verkäufe aufsteigend (zulässige Strategien)
    const shares = await page.evaluate(() => lastOpt.report.outcomes.filter(x => x.feasible).map(x => x.validated.emergencyShare));
    expect([...shares].sort((a, b) => a - b)).toEqual(shares);
    expect(await page.evaluate(() => lastOpt.report.goal)).toBe("keepProperty");
    // (5) ohne Immobilien ist Ziel 2 nicht wählbar
    await page.evaluate(() => { state.realestate.properties = []; renderSection(); });
    await expect(page.locator('#opt-goal option[value="keepProperty"]')).toBeDisabled();
    await noErrors(page, errors);
  });

  test("ISS-3: Grundstückgewinnsteuer ist Pflicht bei Immobilie im Besitz oder mit geplantem Kauf, nicht bei Miete; Start blockiert", async ({ page }) => {
    const errors = await open(page);
    // Beispielplanung mit Besitz-Immobilie, aber ohne Regeln: Fehler im Abschnitt Steuern, Start blockiert
    const res = await page.evaluate(async () => {
      delete state.taxes.propertyGain;
      const blocked = await blockedByErrors();
      const msg = document.querySelector("#toast").textContent;
      const sec = current;
      state.realestate.properties.forEach(p => { p.status = "rent"; });
      const rentOnly = (await planErrors()).filter(e => e.message.includes("Grundstückgewinnsteuer fehlt")).length;
      state.realestate.properties = [];
      const none = (await planErrors()).filter(e => e.message.includes("Grundstückgewinnsteuer fehlt")).length;
      return { blocked, msg, sec, rentOnly, none };
    });
    expect(res.blocked).toBe(true);
    expect(res.sec).toBe("taxes");
    expect(res.msg).toContain("Grundstückgewinnsteuer fehlt");
    expect(res.rentOnly).toBe(0);
    expect(res.none).toBe(0);
    // Mit Vorlage ist die Pflicht erfüllt; die Karte zeigt ohne Regeln den Hinweis «Pflicht»
    await page.reload(); await page.waitForSelector("html[data-ready='1']");
    await page.evaluate(() => { delete state.taxes.propertyGain; goto("taxes"); });
    await expect(page.locator("#main .errcard")).toContainText("Pflicht");
    await page.locator('button[data-action="gain-zh"]').click();
    expect(await page.evaluate(async () => (await planErrors()).filter(e => e.message.includes("Grundstückgewinnsteuer fehlt")).length)).toBe(0);
    await noErrors(page, errors);
  });

  // Kriterien: FEA-26/AK1 FEA-26/AK2 FEA-26/AK3 FEA-26/AK4 FEA-26/AK5 FEA-26/AK6
  test("FEA-26: Immobilien mit gleicher Feldreihenfolge bei Besitz und Kauf geplant, Marktwert in der Kopfzeile", async ({ page }) => {
    const errors = await open(page, "realestate");
    await page.evaluate(() => {
      state.taxes.propertyGain = GAIN_ZH();
      state.realestate.properties = [{ name: "Haus", status: "owned", price: 400000, worth: 500000, bought: "2015-06", sell: "never", fixCosts: 300, mortgage: { tranches: [] } }];
      renderSection();
    });
    const prop = page.locator("#main .prop").first();
    const labels = async () => (await prop.locator(".pgrid > label.field > span:first-child").allInnerTexts()).map(t => t.replace(/\s*ⓘ$/, ""));
    const slots = () => prop.locator(".pgrid > *").evaluateAll(els => { const g = els[0].parentElement.getBoundingClientRect(); return els.map(e => Math.round(e.getBoundingClientRect().left - g.left) + "," + (e.getBoundingClientRect().top - g.top > 40 ? "unten" : "oben")); });
    // Besitz: Marktwert in der Kopfzeile, Raster Kaufdatum · Kaufpreis (bezahlt) · Verkauf ab · Nebenkosten
    await expect(prop.locator(".phead")).toContainText("Marktwert per Simulationsstart");
    await expect(prop.locator(".pgrid")).not.toContainText("Marktwert");
    expect(await labels()).toEqual(["Kaufdatum", "Kaufpreis (bezahlt)", "Verkauf ab", "Nebenkosten / Monat"]);
    const owned = await slots();
    const before = await page.evaluate(() => JSON.stringify(state.realestate.properties[0]));
    // Statuswechsel: gleiche Plätze, Marktwert fehlt in der Kopfzeile, erfasste Werte bleiben
    await prop.locator('[data-action="pstat"][data-v="planned"]').click();
    await expect(prop.locator(".phead")).not.toContainText("Marktwert");
    expect(await labels()).toEqual(["Kauf ab", "Kaufpreis", "Verkauf ab", "Nebenkosten / Monat"]);
    expect(await slots()).toEqual(owned);
    const after = await page.evaluate(() => state.realestate.properties[0]);
    expect(after).toMatchObject({ name: "Haus", price: 400000, sell: "never", fixCosts: 300, worth: 500000 });
    await prop.locator('[data-action="pstat"][data-v="owned"]').click();
    expect(await labels()).toEqual(["Kaufdatum", "Kaufpreis (bezahlt)", "Verkauf ab", "Nebenkosten / Monat"]);
    expect(await page.evaluate(() => JSON.stringify(state.realestate.properties[0]))).toContain('"worth":500000');
    expect(before).toContain('"worth":500000');
    // Schmales Fenster: Marktwert unter dem Namen
    const rel = () => prop.locator(".phead").evaluate(h => { const w = h.querySelector(".pworth").getBoundingClientRect(), n = h.querySelector(".inp").getBoundingClientRect(); return { below: w.top >= n.bottom - 2, same: Math.abs(w.top - n.top) < 40 }; });
    expect(await rel()).toMatchObject({ same: true });
    await page.setViewportSize({ width: 420, height: 800 });
    expect(await rel()).toMatchObject({ below: true });
    await noErrors(page, errors);
  });

  test("ISS-5: Bei «Kauf geplant» gibt es «Verkauf ab» (Vorgabe nie); Verkauf vor oder im Kaufmonat ist ein Fehler am Feld", async ({ page }) => {
    const errors = await open(page, "realestate");
    const i = await page.evaluate(() => { state.realestate.properties.push({ id: "zz", name: "Testobjekt", status: "planned", buy: "2030-06", price: 400000, fixCosts: 0, mortgage: newMortgage() }); renderSection(); return state.realestate.properties.length - 1; });
    // Feld mit Vorgabe «nie»
    const sel = page.locator(`select[data-path="realestate.properties.${i}.sell"]`);
    await expect(sel).toBeVisible();
    await expect(sel).toHaveValue("never");
    await expect(page.locator(`.prop:has(input[data-path="realestate.properties.${i}.name"]) label.field span`, { hasText: "Verkauf ab" })).toBeVisible();
    // Datum wählbar; Verkauf vor dem Kauf: Fehler am Feld, Start blockiert
    await page.evaluate(i => { state.realestate.properties[i].sell = "2030-03"; }, i);
    const errs = await page.evaluate(async i => (await planErrors()).filter(e => e.field === `realestate.properties.${i}.sell`), i);
    expect(errs).toHaveLength(1);
    expect(errs[0].message).toContain("nach «Kauf ab»");
    expect(await page.evaluate(() => blockedByErrors())).toBe(true);
    // Verkauf nach dem Kauf: kein Fehler am Feld
    await page.evaluate(i => { state.realestate.properties[i].sell = "2035-03"; }, i);
    expect(await page.evaluate(async i => (await planErrors()).filter(e => e.field === `realestate.properties.${i}.sell`).length, i)).toBe(0);
    // Wechsel auf «Besitz» behält den erfassten Verkauf
    await page.evaluate(i => ACTIONS.pstat({ i, v: "owned" }), i);
    expect(await page.evaluate(i => state.realestate.properties[i].sell, i)).toBe("2035-03");
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Einstellungen, feste Meilensteine und Säule 3a (FEA-24, FEA-36, FEA-37, FEA-38)", () => {
  test("FEA-24/AK1–AK4: Einstellungen anzeigen, prüfen, speichern, nach Neuladen behalten, Standardwerte", async ({ page }) => {
    const errors = await open(page);
    await page.locator('header button[data-action="settings"]').click();
    const dlg = page.locator("#settings");
    await expect(dlg).toBeVisible();
    for (const t of ["AHV-Bezug: untere Grenze", "AHV-Bezug: obere Grenze", "Bezug Pensionskasse: untere Grenze", "Bezug Pensionskasse: obere Grenze", "Säule 3a: Maximalbetrag der Einzahlung pro Jahr"])
      await expect(dlg).toContainText(t);
    await expect(page.locator("#set-pkMin")).toHaveValue("58");
    await expect(page.locator("#set-pillar3aMax")).toHaveValue("7'258");
    // AK4: untere Grenze über der oberen wird abgelehnt und nicht gespeichert
    await page.locator("#set-ahvMin").fill("71");
    await page.locator("#settingsSave").click();
    await expect(page.locator("#settingsErr")).toContainText("AHV-Bezug: Die untere Grenze (71) liegt über der oberen Grenze (70).");
    await expect(dlg).toBeVisible();
    expect(await page.evaluate(() => limits().ahvMin)).toBe(63);
    // AK2: gültige Änderung gilt nach dem Neuladen weiter
    await page.locator("#set-ahvMin").fill("63");
    await page.locator("#set-pkMax").fill("67");
    await page.locator("#settingsSave").click();
    await expect(dlg).toBeHidden();
    await page.reload();
    await page.waitForSelector("html[data-ready='1']");
    expect(await page.evaluate(() => limits().pkMax)).toBe(67);
    // Die Grenzen gehören nicht zur Planung
    expect(await page.evaluate(() => "limits" in state)).toBe(false);
    // AK3: Standardwerte nach Rückfrage
    await page.keyboard.press(`${MOD}+,`);
    await expect(dlg).toBeVisible();
    await page.locator("#settingsDefaults").click();
    await page.locator("#ask button[value='ok']").click();
    await expect(page.locator("#set-pkMax")).toHaveValue("65");
    expect(await page.evaluate(() => limits().pkMax)).toBe(65);
    await noErrors(page, errors);
  });

  test("FEA-24/AK6, FEA-38/AK2: obere Grenze Bezug Pensionskasse 67: 67 gültig, 68 abgelehnt", async ({ page }) => {
    const errors = await open(page, "milestones");
    await page.evaluate(() => Pref.set("limits", { ...LIMIT_DEFAULTS, pkMax: 67 }));
    const row = await msRow(page, "retire");
    const years = row.locator("input[data-type=years]");
    await years.fill("67");
    await expect(page.locator("#val li.err")).toHaveCount(0);
    await years.fill("68");
    await years.press("Tab"); // ISS-13: Meldung am Feld erst beim Verlassen
    await expect(page.locator("#val li.err").first()).toContainText("«Bezug Pensionskasse»: Alter 68 liegt ausserhalb der Grenzen 58 bis 67 Jahre (Einstellungen).");
    await expect(row).toContainText("Alter 68 liegt ausserhalb");
    await noErrors(page, errors);
  });

  test("FEA-38/AK1, AK5: feste Meilensteine mit festem Namen und eigener Person; Pensionskasse und AHV beginnen dort", async ({ page }) => {
    const errors = await open(page, "milestones");
    for (const id of ["retire", "ahv"]) {
      const row = await msRow(page, id);
      await expect(row.locator("input[data-path$='.name']")).toHaveCount(0);
      await expect(row.locator("select[data-path$='.person']")).toHaveCount(0);
    }
    await expect(await msRow(page, "retire")).toContainText("Bezug Pensionskasse");
    await expect(await msRow(page, "retire")).toContainText("58 bis 65 Jahre");
    await page.locator("#nav a", { hasText: "Vorsorge" }).click();
    await expect(page.locator("#main")).toContainText("Meilenstein «Bezug Pensionskasse»");
    await expect(page.locator("#main")).toContainText("Meilenstein «AHV-Bezug»");
    await expect(page.locator('#main select[data-path="pension.pk.at"]')).toHaveCount(0);
    // Zweite Person: eigene feste Meilensteine
    await page.evaluate(() => ACTIONS["partner-add"]());
    expect(await page.evaluate(() => state.milestones.filter(m => m.id.endsWith("2")).map(m => m.name))).toEqual(["Bezug Pensionskasse Person 2", "AHV-Bezug Person 2"]);
    await noErrors(page, errors);
  });

  test("FEA-38/AK8: Warnung bei Erwerbseinkommen nach dem Bezug der Pensionskasse", async ({ page }) => {
    const errors = await open(page, "flows");
    await page.evaluate(() => { state.flows.income = { mode: "const", value: 12500 }; renderSection(); });
    await expect(page.locator("#val li.warn", { hasText: "Das Erwerbseinkommen beträgt nach dem Bezug der Pensionskasse" })).toContainText("noch 12'500 pro Monat");
    await noErrors(page, errors);
  });

  test("FEA-37/AK1, AK2: Abschnitt Säule 3a mit Konto, Einzahlung und Auszahlungsplan", async ({ page }) => {
    const errors = await open(page, "pillar3a");
    await expect(page.locator("#main h2")).toContainText("Säule 3a");
    await expect(field(page, "pillar3a.0.balance")).toBeVisible();
    await expect(field(page, "pillar3a.0.contribution")).toBeVisible();
    const before = await page.evaluate(() => state.pillar3a[0].payouts.length);
    await page.locator('button[data-action="p3a-payout-add"]').click();
    expect(await page.evaluate(() => state.pillar3a[0].payouts.length)).toBe(before + 1);
    await field(page, `pillar3a.0.payouts.${before}.amount`).fill("50000");
    expect(await page.evaluate(n => state.pillar3a[0].payouts[n].amount, before)).toBe(50000);
    // Leer = ganzes Guthaben
    await field(page, `pillar3a.0.payouts.${before}.amount`).fill("");
    expect(await page.evaluate(n => "amount" in state.pillar3a[0].payouts[n] && state.pillar3a[0].payouts[n].amount !== undefined, before)).toBe(false);
    await page.locator(`button[data-action="list-del"][data-path="pillar3a.0.payouts"]`).last().click();
    expect(await page.evaluate(() => state.pillar3a[0].payouts.length)).toBe(before);
    // AK4: Auszahlung mehr als 5 Jahre vor dem AHV-Bezug ist ein Fehler
    await page.evaluate(() => { state.pillar3a[0].payouts[0].date = "2028-01"; renderSection(); });
    await expect(page.locator("#val li.err").first()).toContainText("liegt mehr als 5 Jahre vor dem AHV-Bezug");
    // AK7: Einzahlung über dem Maximalbetrag
    await page.evaluate(() => { state.pillar3a[0].payouts[0].date = "2031-01"; state.pillar3a[0].contribution = 8000; renderSection(); });
    await expect(page.locator("#val li.err").first()).toContainText("Die Einzahlung muss zwischen 0 und dem Maximalbetrag von 7'258 pro Jahr liegen");
    await noErrors(page, errors);
  });

  test("FEA-37/AK8: Einzahlung erscheint in «Ausgaben total» der Jahreswerte", async ({ page }) => {
    const errors = await open(page);
    const total = async contribution => {
      await page.evaluate(c => { state.returns.source = "fixed"; state.scenarios.sweeps = []; state.pillar3a[0].contribution = c; }, contribution);
      await page.evaluate(() => runSimulation("table", { noConfirm: true }));
      await expect(page.locator("#resultsBody table.yt")).toBeVisible();
      const v = await page.locator("#resultsBody table.yt tr").nth(2).locator("td").nth(6).textContent();
      await page.evaluate(() => showResults(false));
      return parseNum(v);
    };
    const a = await total(0), b = await total(6000);
    expect(b - a).toBe(500);
    await noErrors(page, errors);
  });

  test("FEA-36/AK4: Periode ab dem AHV-Bezug amortisiert direkt", async ({ page }) => {
    const errors = await open(page, "realestate");
    await page.evaluate(() => {
      const t = state.realestate.properties[0].mortgage.tranches[1];
      t.periods.push(newPeriod({ term: 5, amort: 1500, amortMode: "indirect" }), newPeriod({ term: 5, amort: 1500, amortMode: "indirect" }));
      renderSection();
    });
    await expect(page.locator("#main")).toContainText("direkt (ab AHV)");
    await expect(page.locator("#main")).toContainText("+ 3a bis AHV");
    await noErrors(page, errors);
  });

  test("FEA-38/AK6, FEA-37/AK13: ältere Datei wird beim Öffnen übernommen, ein Dialog nennt die Änderungen", async ({ page }) => {
    const errors = await open(page);
    const plan = await page.evaluate(() => {
      const s = JSON.parse(JSON.stringify(state));
      delete s.pillar3a;
      s.milestones = s.milestones.filter(m => m.id !== "ahv");
      Object.assign(s.milestones.find(m => m.id === "retire"), { name: "Pensionierung", years: 56 });
      s.realestate.properties[0].mortgage.pillar3a = { balance: 20000, rate: 0.01, payout: "retire", use: "repay" };
      return s;
    });
    const file = path.join(os.tmpdir(), `qa-upgrade-${process.pid}.vplan`);
    fs.writeFileSync(file, JSON.stringify(plan));
    await page.locator("#fileInput").setInputFiles(file);
    await expect(page.locator("#ask")).toBeVisible();
    const text = await page.locator("#askText").innerText();
    expect(text).toContain("Der Meilenstein «Pensionierung» heisst neu «Bezug Pensionskasse».");
    expect(text).toContain("«Bezug Pensionskasse» lag bei 56 Jahren, ausserhalb der Grenzen 58 bis 65 Jahre; neu 58 Jahre.");
    expect(text).toContain("Der Meilenstein «AHV-Bezug» fehlte und steht neu auf 65 Jahren.");
    expect(text).toContain("Die Säule 3a von «Haus» ist neu ein eigenes 3a-Konto");
    await page.locator("#ask button[value='ok']").click();
    expect(await page.evaluate(() => [state.pillar3a.length, state.pillar3a[0].balance, state.realestate.properties[0].mortgage.pillar3a])).toEqual([1, 20000, undefined]);
    fs.rmSync(file);
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Lebenshaltung inklusive Wohnen und Cash (FEA-39, FEA-40)", () => {
  test("FEA-39/AK1, AK2: Beschriftung inklusive Wohnkosten, Hinweis mit dem Wohnanteil zum Simulationsstart", async ({ page }) => {
    const errors = await open(page, "flows");
    await expect(page.locator("#main")).toContainText("Lebenshaltungskosten / Monat inklusive Miete bzw. Wohnkosten (Zins, Amortisation, Nebenkosten)");
    // Beispielplanung: Wohnkosten zum Simulationsstart 1'700 pro Monat
    await expect(page.locator("#main .hint").filter({ hasText: "Wohnanteil zum Simulationsstart" })).toContainText("1'700");
    await noErrors(page, errors);
  });

  test("FEA-39/AK4: Lebenshaltung kleiner als der Wohnanteil ist ein Fehler mit beiden Beträgen", async ({ page }) => {
    const errors = await open(page);
    const issues = await page.evaluate(async () => {
      const s = JSON.parse(JSON.stringify(state));
      s.flows.spending = { mode: "const", value: 1000 };
      return (await Engine.call("validate", { plan: s })).map(i => i.message);
    });
    expect(issues).toContain("Die Lebenshaltungskosten (1'000) sind kleiner als die Wohnkosten zum Simulationsstart (1'700), die sie enthalten.");
    await noErrors(page, errors);
  });

  test("FEA-40/AK1, AK4: Aktienquote ohne Cash; Hinweis zur Cash-Regel vor und ab der Pensionierung", async ({ page }) => {
    const errors = await open(page, "wealth");
    await expect(page.locator("#main")).toContainText("Aktienquote (Aktien am Portfolio aus Aktien und Obligationen)");
    await expect(page.locator("#main")).toContainText("Cash zählt nicht zum Portfolio");
    await expect(page.locator("#main")).not.toContainText("Liquidität");
    await page.evaluate(() => { state.withdrawal.strategy = "standard"; goto("withdrawal"); });
    await expect(page.locator("#main")).toContainText("ab der Pensionierung");
    await noErrors(page, errors);
  });

  test("FEA-39/AK7, FEA-40/AK8: ältere Datei: Wohnanteil addiert, Hinweise zu Wohnkosten und Cash", async ({ page }) => {
    const errors = await open(page);
    const plan = await page.evaluate(() => {
      const s = JSON.parse(JSON.stringify(state));
      delete s.flows.spendingIncludesHousing;
      s.flows.spending = { mode: "const", value: 5000 };
      return s;
    });
    const file = path.join(os.tmpdir(), `fea39-${process.pid}.vplan`);
    fs.writeFileSync(file, JSON.stringify(plan));
    await page.locator("#fileInput").setInputFiles(file);
    await expect(page.locator("#ask")).toBeVisible();
    const text = await page.locator("#askText").innerText();
    expect(text).toContain("Die Wohnkosten zum Simulationsstart (1700 pro Monat) sind zu jedem Wert addiert");
    expect(text).toContain("Cash wird neu bis zur Pensionierung (Bezug Pensionskasse) auf dem erfassten Betrag gehalten");
    await page.locator("#ask button[value='ok']").click();
    expect(await page.evaluate(() => [state.flows.spendingIncludesHousing, state.flows.spending.value])).toEqual([true, 6700]);
    fs.rmSync(file);
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Säule 3a: Auszahlungsplan mit Datum oder Alter (FEA-41)", () => {
  // Erfundenes Beispiel: Geburt 1980-06 (AHV-Bezug 06.2045, frühestens 06.2040), Guthaben 120'000, Zins 0, keine Einzahlung.
  const setup = async (page, payouts) => {
    await page.evaluate(p => {
      state.household.persons[0].birth = "1980-06";
      state.realestate.properties.forEach(x => x.mortgage?.tranches?.forEach(t => t.periods.forEach(d => { d.amortMode = "direct"; })));
      state.pillar3a = [{ person: state.household.persons[0].id, balance: 120000, rate: 0, contribution: 0, use: "wealth", payouts: p }];
      goto("pillar3a");
    }, payouts);
  };
  const at = (page, j) => page.locator(`tr[data-p3a-row="${j}"] [data-p3a="at"]`);
  const val = (page, j, which) => page.locator(`tr[data-p3a-row="${j}"] [data-p3a="${which}"]`);

  test("FEA-41/AK1, AK2, AK8: Datum oder Alter; Monat und Alter daneben; beim Wechsel bleibt der Zeitpunkt; alte Zeilen sind «Datum»", async ({ page }) => {
    const errors = await open(page);
    await setup(page, [{ kind: "age", years: 61.5, amount: 50000, use: "wealth" }, { date: "2043-01", amount: 1000, use: "wealth" }]);
    await expect(at(page, 0)).toHaveText("12.2041 · Alter 61.5");
    // AK8: Zeile nur mit Datum erscheint als «Datum»
    await expect(page.locator('select[data-path="pillar3a.0.payouts.1.kind"]')).toHaveValue("date");
    await expect(at(page, 1)).toHaveText("01.2043 · Alter 62.6");
    // AK2: Alter 61.5 → Datum 12.2041
    await page.locator('select[data-path="pillar3a.0.payouts.0.kind"]').selectOption("date");
    await expect(field(page, "pillar3a.0.payouts.0.date")).toHaveValue("12.2041");
    expect(await page.evaluate(() => state.pillar3a[0].payouts[0])).toEqual({ amount: 50000, use: "wealth", date: "2041-12" });
    // und zurück
    await page.locator('select[data-path="pillar3a.0.payouts.0.kind"]').selectOption("age");
    await expect(field(page, "pillar3a.0.payouts.0.years")).toHaveValue("61.5");
    await expect(at(page, 0)).toHaveText("12.2041 · Alter 61.5");
    await noErrors(page, errors);
  });

  test("FEA-41/AK3, AK4: frühestens 5 Jahre vor dem AHV-Bezug auch nach Alter; ohne Vorschau steht «–»", async ({ page }) => {
    const errors = await open(page);
    await setup(page, [{ kind: "age", years: 59.9, amount: 50000, use: "wealth" }]);
    await expect(page.locator("#val li.err").first()).toContainText("Die Auszahlung im 05.2040 liegt mehr als 5 Jahre vor dem AHV-Bezug (06.2045)");
    await expect(val(page, 0, "before")).toHaveText("–");
    await expect(val(page, 0, "after")).toHaveText("–");
    await expect(page.locator('#main [data-p3a="rest"]')).toHaveText("–");
    await noErrors(page, errors);
  });

  test("FEA-41/AK4–AK7: Guthaben vor und nach der Auszahlung, Rest im AHV-Bezug; «+ Auszahlung» füllt Zeitpunkt und Betrag vor", async ({ page }) => {
    const errors = await open(page);
    await setup(page, [{ kind: "age", years: 61.5, amount: 50000, use: "wealth" }]);
    await expect(val(page, 0, "before")).toHaveText("120'000");
    await expect(val(page, 0, "after")).toHaveText("70'000");
    await expect(page.locator('#main [data-p3a="rest"]')).toHaveText(/^70'000/);
    // AK7: ein Jahr später, Rest 70'000 (gleiche Art wie die letzte Zeile)
    await page.locator('button[data-action="p3a-payout-add"]').click();
    expect(await page.evaluate(() => state.pillar3a[0].payouts[1])).toEqual({ use: "wealth", kind: "age", years: 62.5, amount: 70000 });
    await expect(at(page, 1)).toHaveText("12.2042 · Alter 62.5");
    // Der Betrag bleibt änderbar
    await field(page, "pillar3a.0.payouts.1.amount").fill("20000");
    expect(await page.evaluate(() => state.pillar3a[0].payouts[1].amount)).toBe(20000);
    await expect(page.locator('#main [data-p3a="rest"]')).toHaveText(/^50'000/);
    // Ohne Zeile: frühester erlaubter Monat (06.2040) mit dem ganzen Guthaben, abgerundet auf 1'000
    await setup(page, []);
    await page.evaluate(() => { state.pillar3a[0].balance = 120400; refresh(); });
    await expect(page.locator('#main [data-p3a="rest"]')).toHaveText(/^120'400/);
    await page.locator('button[data-action="p3a-payout-add"]').click();
    expect(await page.evaluate(() => state.pillar3a[0].payouts[0])).toEqual({ use: "wealth", date: "2040-06", amount: 120000 });
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Fehlermeldung am Feld erst beim Verlassen (ISS-13)", () => {
  const ferr = el => el.locator("xpath=ancestor-or-self::div[contains(@class,'inp')][1]/following-sibling::span[contains(@class,'ferr')]");

  test("ISS-13/AK1–AK4: Während der Eingabe wartet alles; beim Verlassen Meldung, Vorschau und Prüfliste neu", async ({ page }) => {
    const errors = await open(page, "milestones");
    const i = await page.evaluate(() => state.milestones.findIndex(m => m.id === "ahv"));
    const years = field(page, `milestones.${i}.years`);
    const end = kpiValue(page, "Vermögen am Ende");
    await expect(end).not.toHaveText("–");
    const before = await end.textContent();
    // AK1, AK4: Ziffer für Ziffer, mit Pause: keine Meldung am Feld, keine in der Prüfliste, Vorschau unverändert
    await years.click(); await years.press("End");
    await years.press("Backspace"); await years.press("Backspace"); await years.type("6");
    await page.waitForTimeout(800);
    await expect(ferr(years)).toHaveCount(0);
    await expect(page.locator("#val li.err")).toHaveCount(0);
    await expect(end).toHaveText(before);
    await years.type("3"); // 63: gültig, der Wert ist übernommen, gerechnet wird erst beim Verlassen
    await page.waitForTimeout(800);
    expect(await page.evaluate(n => state.milestones[n].years, i)).toBe(63);
    await expect(end).toHaveText(before);
    await years.press("Tab");
    await expect(end).not.toHaveText(before);
    await expect(ferr(years)).toHaveCount(0);
    // AK2: Alter 6 eingeben: erst beim Verlassen Meldung am Feld und in der Prüfliste
    await years.fill("6");
    await page.waitForTimeout(500);
    await expect(ferr(years)).toHaveCount(0);
    await expect(page.locator("#val li.err")).toHaveCount(0);
    await years.press("Tab");
    await expect(ferr(years)).toContainText("ausserhalb der Grenzen");
    await expect(page.locator("#val li.err").first()).toContainText("AHV-Bezug");
    // AK3: zurück ins Feld, gültig machen: Die Meldung bleibt während der Eingabe und verschwindet beim Verlassen
    await years.click(); await years.press("End"); await years.type("5");
    await page.waitForTimeout(500);
    await expect(ferr(years)).toContainText("ausserhalb der Grenzen");
    await expect(years).toBeFocused();
    await years.press("Tab");
    await expect(ferr(years)).toHaveCount(0);
    await expect(page.locator("#val li.err")).toHaveCount(0);
    await expect(end).toHaveText(before);
    await noErrors(page, errors);
  });

  test("ISS-13/AK1, AK2: ungültige Zahl erst beim Verlassen bzw. mit Enter gemeldet", async ({ page }) => {
    const errors = await open(page, "wealth");
    const cash = field(page, "wealth.cash");
    await cash.click(); await cash.press("End"); await cash.type("x");
    await page.waitForTimeout(500);
    await expect(ferr(cash)).toHaveCount(0);
    await expect(cash).not.toHaveClass(/invalid/);
    await cash.press("Enter");
    await expect(ferr(cash)).toContainText("Ungültige Zahl.");
    await expect(cash).toHaveClass(/invalid/);
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Erwerbseinkommen mit der Teuerung (FEA-43)", () => {
  test("FEA-43/AK1: Auswahl je Person, Vorschau rechnet sofort neu", async ({ page }) => {
    const errors = await open(page, "flows");
    const own = page.locator('input[data-path="flows.incomeIndexed"]');
    await expect(own).toBeVisible();
    await expect(own).not.toBeChecked(); // Beispielplanung ohne Angabe: «nein»
    await expect(page.locator('input[data-path="partner.incomeIndexed"]')).toHaveCount(0); // ohne zweite Person
    // Genauer Endwert der Vorschau (die Kennzahl ist gerundet)
    const endValue = () => page.evaluate(() => lastSim?.total.at(-1));
    await expect(kpiValue(page, "Vermögen am Ende")).not.toHaveText("–");
    const before = await endValue();
    await own.check();
    expect(await page.evaluate(() => state.flows.incomeIndexed)).toBe(true);
    await expect.poll(endValue).not.toBe(before);
    await own.uncheck();
    await expect.poll(endValue).toBe(before);
    // Zweite Person: eigene Auswahl, neu mit «ja»
    await page.locator("#nav a", { hasText: "Haushalt & Projekt" }).click();
    await page.getByRole("button", { name: "+ Zweite Person" }).click();
    await page.locator("#nav a", { hasText: "Einnahmen & Ausgaben" }).click();
    await expect(page.locator('input[data-path="partner.incomeIndexed"]')).toBeChecked();
    await expect(own).not.toBeChecked();
    await noErrors(page, errors);
  });

  test("FEA-43: neue Planungen und neue zweite Person mit «steigt mit der Teuerung»", async ({ page }) => {
    const errors = await open(page);
    await page.locator('button[data-action="new"]').click();
    await page.waitForFunction(() => state.project.name === "Neue Planung");
    expect(await page.evaluate(() => state.flows.incomeIndexed)).toBe(true);
    await page.locator("#nav a", { hasText: "Einnahmen & Ausgaben" }).click();
    await expect(page.locator('input[data-path="flows.incomeIndexed"]')).toBeChecked();
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
// FEA-54 Willkommensdialog, FEA-55 Erste Schritte, FEA-56 Ergebnis in Worten
const welcome = page => page.locator("#welcome");
const steps = page => page.locator("#steps");
const helpTitle = page => page.locator("#help:not([hidden]) #helpBody h1").first();
const prefs = page => page.evaluate(() => JSON.parse(localStorage.getItem("rp.settings") || "{}"));

test.describe("FEA-54 Willkommensdialog", () => {
  test("AK1: erscheint beim ersten Start (Testzugang) mit drei Wahlmöglichkeiten, sonst nicht", async ({ page }) => {
    let errors = await open(page);
    await expect(welcome(page)).not.toHaveAttribute("open", "");
    await noErrors(page, errors);
    await page.goto("about:blank");
    errors = await open(page, "welcome");
    await expect(welcome(page)).toHaveAttribute("open", "");
    await expect(welcome(page).locator("h2")).toHaveText("Willkommen bei VivaPlan");
    await expect(welcome(page).locator(".wchoices button b")).toHaveText(["Beispiel erkunden", "Eigene Planung beginnen", "Leere Planung"]);
    // AK6: Der erste Start ist sofort gemerkt.
    expect((await prefs(page)).welcomeShown).toBe(true);
    await noErrors(page, errors);
  });

  test("AK2: «Beispiel erkunden» lädt die Beispielplanung und öffnet die Hilfe «Was VivaPlan kann»", async ({ page }) => {
    const errors = await open(page, "welcome");
    await welcome(page).getByRole("button", { name: /Beispiel erkunden/ }).click();
    await expect(welcome(page)).not.toHaveAttribute("open", "");
    await expect(helpTitle(page)).toHaveText("Was VivaPlan kann");
    expect(await page.evaluate(() => state.project.name)).toBe("Beispielplanung");
    await noErrors(page, errors);
  });

  test("AK3: «Eigene Planung beginnen» öffnet «Haushalt & Projekt» und blendet die Checkliste ein", async ({ page }) => {
    // Fensterhöhe wie die Desktop-App: Die Checkliste steht unter Vorschau und Kennzahlen und muss in den Blick gescrollt werden.
    await page.setViewportSize({ width: 1440, height: 900 });
    const errors = await open(page, "welcome");
    await welcome(page).getByRole("button", { name: /Eigene Planung beginnen/ }).click();
    await expect(page.locator("#nav a.active")).toContainText("Haushalt & Projekt");
    await expect(steps(page)).toBeVisible();
    await expect(steps(page)).toBeInViewport({ ratio: 0.95 });
    await expect(steps(page).locator("summary")).toBeFocused();
    await expect(page.locator("#stepsProg")).toHaveText("0 von 8");
    expect(await page.evaluate(() => state.project.name)).toBe("Beispielplanung");
    await noErrors(page, errors);
  });

  test("AK4: «Leere Planung» wirkt wie Datei → Neu", async ({ page }) => {
    const errors = await open(page, "welcome");
    await welcome(page).getByRole("button", { name: /Leere Planung/ }).click();
    await expect(page.locator("#nav a.active")).toContainText("Haushalt & Projekt");
    // Merkmale der leeren Planung von Datei → Neu: kein Name, keine Beträge, keine Immobilien
    expect(await page.evaluate(() => [state.project.name, state.household.persons.length, state.household.persons[0].name,
      state.flows.spending.value, state.wealth.stocks, state.realestate.properties.length])).toEqual(["Neue Planung", 1, "", 0, 0, 0]);
    await noErrors(page, errors);
  });

  test("AK5, AK8: Esc schliesst ohne Änderung und ohne Hilfe; Tab und Enter wählen", async ({ page }) => {
    let errors = await open(page, "welcome");
    const before = await page.evaluate(() => JSON.stringify(state));
    await page.keyboard.press("Escape");
    await expect(welcome(page)).not.toHaveAttribute("open", "");
    await expect(page.locator("#help")).toBeHidden();
    expect(await page.evaluate(() => JSON.stringify(state))).toBe(before);
    await noErrors(page, errors);
    // Tastatur: der erste Knopf hat den Fokus, Tab geht zum zweiten, Enter wählt ihn.
    await page.goto("about:blank");
    errors = await open(page, "welcome");
    await expect(welcome(page).getByRole("button", { name: /Beispiel erkunden/ })).toBeFocused();
    await page.keyboard.press("Tab");
    await expect(welcome(page).getByRole("button", { name: /Eigene Planung beginnen/ })).toBeFocused();
    await page.keyboard.press("Enter");
    await expect(page.locator("#stepsProg")).toHaveText("0 von 8");
    await noErrors(page, errors);
  });

  test("AK7: Hilfe → «Willkommen …» öffnet den Dialog; bei ungespeicherten Änderungen fragt die App nach", async ({ page }) => {
    const errors = await open(page);
    await field(page, "project.name").fill("Geändert");
    await field(page, "project.name").press("Tab");
    await page.evaluate(() => { command("welcome"); });
    await expect(welcome(page)).toHaveAttribute("open", "");
    await welcome(page).getByRole("button", { name: /Beispiel erkunden/ }).click();
    await expect(page.locator("#ask")).toHaveAttribute("open", "");
    await expect(page.locator("#askTitle")).toHaveText("Ungespeicherte Änderungen");
    await page.locator('#ask button[value="cancel"]').click();
    expect(await page.evaluate(() => state.project.name)).toBe("Geändert");
    await noErrors(page, errors);
  });
});

test.describe("FEA-55 Erste Schritte", () => {
  test("AK1, AK5: acht Schritte des Schnellstarts mit Fortschritt", async ({ page }) => {
    const errors = await open(page);
    await expect(steps(page)).toBeVisible();
    await expect(steps(page).locator("li a")).toHaveText(["1. Haushalt & Projekt", "2. Meilensteine", "3. Vorsorge", "4. Einnahmen & Ausgaben",
      "5. Vermögen & Anlage", "6. Simulation", "7. Steuern", "8. Simulation starten"]);
    await expect(page.locator("#stepsProg")).toHaveText("0 von 8");
    await noErrors(page, errors);
  });

  test("AK2: Klick öffnet den Abschnitt, ⓘ die Hilfe zum Schritt", async ({ page }) => {
    const errors = await open(page);
    await steps(page).getByText("3. Vorsorge").click();
    await expect(page.locator("#nav a.active")).toContainText("Vorsorge");
    await steps(page).locator('li[data-step="taxes"] .hq').click();
    await expect(helpTitle(page)).toHaveText("Steuern");
    await noErrors(page, errors);
  });

  test("AK3, AK4, AK9: Änderung oder Haken erledigt einen Schritt, Rückgängig nimmt ihn nicht weg, gespeichert nur in den Einstellungen", async ({ page }) => {
    const errors = await open(page);
    await field(page, "project.name").fill("Geändert");
    await field(page, "project.name").press("Tab");
    await expect(steps(page).locator('li[data-step="general"]')).toHaveClass(/done/);
    await expect(page.locator("#stepsProg")).toHaveText("1 von 8");
    await page.evaluate(() => History.undo());
    await expect(steps(page).locator('li[data-step="general"]')).toHaveClass(/done/);
    // Von Hand abhaken und wieder lösen
    await steps(page).locator('li[data-step="taxes"] input').check();
    await expect(page.locator("#stepsProg")).toHaveText("2 von 8");
    await steps(page).locator('li[data-step="taxes"] input').uncheck();
    await expect(page.locator("#stepsProg")).toHaveText("1 von 8");
    expect((await prefs(page)).firstSteps.done).toEqual(["general"]);
    expect(await page.evaluate(() => "firstSteps" in state)).toBe(false);
    await noErrors(page, errors);
  });

  test("AK3: «Simulation starten» ist nach einer Simulation erledigt, nicht nach einem Abbruch", async ({ page }) => {
    const errors = await open(page, "results/nosweep");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "overview");
    expect((await prefs(page)).firstSteps.done).toEqual(["run"]);
    await noErrors(page, errors);
  });

  test("AK6, AK7: ausblenden, über das Menü einblenden; alles erledigt klappt zu", async ({ page }) => {
    const errors = await open(page);
    await steps(page).locator("button.x").click();
    await expect(steps(page)).toBeHidden();
    await page.evaluate(() => command("steps"));
    await expect(steps(page)).toBeVisible();
    await expect(steps(page)).toBeInViewport({ ratio: 0.95 }); // FEA-54/AK3: in den Blick gescrollt
    for (const id of ["general", "milestones", "pension", "flows", "wealth", "returns", "taxes", "run"])
      await steps(page).locator(`li[data-step="${id}"] input`).check();
    await expect(page.locator("#stepsProg")).toHaveText("8 von 8");
    await expect(page.locator("#toast")).toContainText("Alle ersten Schritte erledigt");
    await expect(steps(page).locator("details")).not.toHaveAttribute("open", "");
    await expect(steps(page).locator(".stepsdone")).toBeAttached();
    await noErrors(page, errors);
  });

  test("AK8: «Eigene Planung beginnen» blendet die Checkliste ein und setzt sie zurück", async ({ page }) => {
    const errors = await open(page);
    await steps(page).locator('li[data-step="taxes"] input').check();
    await steps(page).locator("button.x").click();
    await page.evaluate(() => { command("welcome"); });
    await welcome(page).getByRole("button", { name: /Eigene Planung beginnen/ }).click();
    await expect(steps(page)).toBeVisible();
    await expect(page.locator("#stepsProg")).toHaveText("0 von 8");
    await noErrors(page, errors);
  });
});

test.describe("FEA-56 Ergebnis in Worten", () => {
  test("AK1, AK3, AK6: mehrere Läufe mit Einordnung, ungünstigster Lauf und nächster Schritt", async ({ page }) => {
    const errors = await open(page, "results/bootstrap/nosweep");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "overview");
    const v = page.locator("#verdict");
    const rate = (await resultKpi(page, "Erfolgsquote").textContent()).trim();
    const cls = await page.evaluate(() => rateClass(lastRun.batch.summary.successRate));
    await expect(v).toContainText(`In ${rate} (${cls}) der Läufe reicht das Vermögen bis`);
    const ed = (await resultKpi(page, "Früheste Erschöpfung").textContent()).trim();
    if (ed === "keine") await expect(v).not.toContainText("ungünstigsten Lauf");
    else await expect(v).toContainText(`Im ungünstigsten Lauf ist das Geld mit ${ed} aufgebraucht.`);
    await expect(v.locator(".next a")).toHaveText(cls === "hoch" ? "Szenarien" : "Optimierung");
    await v.getByRole("link", { name: cls }).or(v.locator(`a:text-is("${cls}")`)).first().click();
    await expect(helpTitle(page)).toHaveText("Ergebnisse: Übersicht");
    await noErrors(page, errors);
  });

  test("AK2: Grenzen der Einordnung am angezeigten, gerundeten Wert", async ({ page }) => {
    const errors = await open(page);
    expect(await page.evaluate(() => [0.95, 0.9496, 0.9494, 0.8, 0.7996, 0.7994, 0].map(rateClass)))
      .toEqual(["hoch", "hoch", "mittel", "mittel", "mittel", "tief", "tief"]);
    await noErrors(page, errors);
  });

  test("AK4, AK5, AK7: Einzellauf, Beträge folgen Nominal/Kaufkraft, erschöpftes Vermögen verweist auf die Optimierung", async ({ page }) => {
    const errors = await open(page, "results/nosweep");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "overview");
    const v = page.locator("#verdict");
    const nominal = await page.evaluate(() => fmtMoney(lastRun.single.summary.endTotal));
    await expect(v).toContainText(`Das Vermögen reicht bis zum Ende der Planung und beträgt dann ${nominal}.`);
    await expect(v.locator(".next a")).toHaveText("Szenarien");
    await page.getByRole("button", { name: "Heutige Kaufkraft" }).click();
    const real = await page.evaluate(() => fmtMoney(lastRun.single.summary.endTotalReal));
    await expect(v).toContainText(`beträgt dann ${real} in heutiger Kaufkraft.`);
    // Ausgaben so hoch, dass das Geld nicht reicht
    await page.evaluate(async () => { showResults(false); state.flows.spending = { mode: "const", value: 60000 }; await runSimulation("overview", { noConfirm: true }); });
    const dep = await page.evaluate(() => lastRun.single.summary.depleted);
    expect(dep).toBeTruthy();
    await expect(v).toContainText(/Das Vermögen ist im \d\d\.\d{4} \(Alter [\d.,]+\) aufgebraucht\./);
    await expect(v.locator(".next a")).toHaveText("Optimierung");
    await noErrors(page, errors);
  });

  test("AK6: gespeicherte Ergebnisse aus der Liste zeigen den Text", async ({ page }) => {
    const errors = await open(page, "runsflow/view/mem");
    await expect(page.locator(".savedbar")).toBeVisible();
    await expect(page.locator("#verdict")).toContainText("der Läufe reicht das Vermögen bis");
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
// FEA-45 Haftungsausschluss: Fusszeile der Ergebnisse und vollständiger Text
test.describe("FEA-45 Haftungsausschluss", () => {
  const FOOT = "Modellrechnung ohne Gewähr. Keine Finanz- oder Steuerberatung.";
  const prefs = page => page.evaluate(() => JSON.parse(localStorage.getItem("rp.settings") || "{}"));
  const helpTitle = page => page.locator("#help:not([hidden]) #helpBody h1").first();

  test("AK2: im Browser mit Testzugang einmal bestätigen, danach gespeichert (Browserspeicher)", async ({ page }) => {
    await page.goto("/#disclaimer");
    await page.waitForSelector("html[data-disclaimer='open']");
    await page.locator("#disclaimerCheck").check();
    await page.locator("#disclaimerOk").click();
    await page.waitForSelector("html[data-ready='1']");
    expect((await prefs(page)).disclaimerAccepted).toBe(1);
    await page.reload();
    await page.waitForSelector("html[data-ready='1']");
    await expect(page.locator("#disclaimer")).not.toHaveAttribute("open", "");
  });

  test("AK3: Fusszeile auf allen Ergebnisseiten", async ({ page }) => {
    const errors = await open(page, "results/nosweep");
    await expect(page.locator("#results")).toBeVisible();
    for (const tab of await page.locator('[data-action="rtab"]').all()) {
      await tab.click();
      await expect(page.locator("#resultsFoot")).toBeVisible();
      await expect(page.locator("#resultsFoot")).toHaveText(FOOT);
    }
    // gespeicherte Ergebnisse (Liste) liegen in derselben Ansicht
    await expect(page.locator("#results .rfoot")).toHaveCount(1);
    await noErrors(page, errors);
  });

  test("AK3: Fusszeile unter dem Ergebnis der Optimierung", async ({ page }) => {
    test.setTimeout(240_000);
    const errors = await open(page, "optimize/bootstrap");
    await expect(page.locator("#optFoot")).toHaveCount(0);
    // kleine Rechnung: nur die Fusszeile wird geprüft, nicht das Ergebnis
    const small = { paths: 50, checkPaths: 50, samples: 1, strategies: ["fixed_rate"] };
    expect(await setOpt(page, small)).toMatchObject(small); // ISS-16
    await page.locator('button[data-action="opt-run"]').click();
    await expect(page.locator("html")).toHaveAttribute("data-job", "off", { timeout: 200_000 });
    await expect(page.locator("#optFoot")).toHaveText(FOOT);
    await noErrors(page, errors);
  });

  test("AK4: vollständiger Text im Dialog «Über VivaPlan» erreichbar und als Seite der Hilfe, mit Stand", async ({ page }) => {
    const errors = await open(page);
    await page.evaluate(() => command("about"));
    await expect(page.locator("#ask")).toHaveAttribute("open", "");
    await expect(page.locator("#aboutDisclaimer")).toContainText(FOOT);
    await page.locator('#aboutDisclaimer a[data-action="about-disclaimer"]').click();
    await expect(page.locator("#ask")).not.toHaveAttribute("open", "");
    await expect(helpTitle(page)).toHaveText("Haftungsausschluss");
    const body = page.locator("#help:not([hidden]) #helpBody");
    for (const h of ["1. Kostenloses Angebot.", "2. Keine Beratung.", "3. Modellrechnungen ohne Gewähr.", "4. Eigene Verantwortung.",
      "5. Haftung.", "6. Daten.", "7. Anwendbares Recht."]) await expect(body).toContainText(h);
    await expect(body).toContainText("VivaPlan wird aus der Schweiz unentgeltlich zur Verfügung gestellt.");
    await expect(body).toContainText("Stand: 13. November 2026");
    // auch über die Suche der Hilfe
    await page.evaluate(() => Help.show("disclaimer"));
    await expect(helpTitle(page)).toHaveText("Haftungsausschluss");
    await noErrors(page, errors);
  });
});
