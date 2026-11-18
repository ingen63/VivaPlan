// Oberflächentests von VivaPlan im Browser: Willkommen, Erste Schritte, Ergebnis in Worten, Haftung. Jeder Test startet mit der Beispielplanung.
const { test, expect } = require("@playwright/test");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { MOD, SECTIONS, open, noErrors, field, previewIdle, nav, kpiValue, resultKpi, parseNum, setOpt, msRow, welcome, steps, helpTitle, prefs } = require("./helpers");

// ---------------------------------------------------------------------------------------------
// #70 Willkommensdialog, #71 Erste Schritte, #72 Ergebnis in Worten

test.describe("#70 Willkommensdialog", () => {
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
    // #95: «Neu» übernimmt die leere Planung erst nach zwei Aufrufen an den Rechenkern. «Haushalt & Projekt» ist schon
    // vorher aktiv; deshalb auf die Planung selbst warten statt nur auf die Navigation.
    // Merkmale der leeren Planung von Datei → Neu: kein Name, keine Beträge, keine Immobilien
    await expect.poll(() => page.evaluate(() => [state.project.name, state.household.persons.length, state.household.persons[0].name,
      state.flows.spending.value, state.wealth.stocks, state.realestate.properties.length])).toEqual(["Neue Planung", 1, "", 0, 0, 0]);
    await expect(page.locator("#nav a.active")).toContainText("Haushalt & Projekt");
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

test.describe("#71 Erste Schritte", () => {
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
    await expect(steps(page)).toBeInViewport({ ratio: 0.95 }); // #70/AK3: in den Blick gescrollt
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

test.describe("#72 Ergebnis in Worten", () => {
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
// #61 Haftungsausschluss: Fusszeile der Ergebnisse und vollständiger Text
test.describe("#61 Haftungsausschluss", () => {
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
    const errors = await open(page, "optimize/bootstrap");
    await expect(page.locator("#optFoot")).toHaveCount(0);
    // kleine Rechnung: nur die Fusszeile wird geprüft, nicht das Ergebnis
    const small = { paths: 50, checkPaths: 50, samples: 1, strategies: ["fixed_rate"] };
    expect(await setOpt(page, small)).toMatchObject(small); // #79
    await page.locator('button[data-action="opt-run"]').click();
    await expect(page.locator("html")).toHaveAttribute("data-job", "off");
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
