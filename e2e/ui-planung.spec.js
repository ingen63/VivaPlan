// Oberflächentests von VivaPlan im Browser: Einstellungen und Planungsfelder. Jeder Test startet mit der Beispielplanung.
const { test, expect } = require("@playwright/test");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { MOD, SECTIONS, open, noErrors, field, previewIdle, nav, kpiValue, resultKpi, parseNum, setOpt, msRow, welcome, steps, helpTitle, prefs } = require("./helpers");

// ---------------------------------------------------------------------------------------------
test.describe("Einstellungen, feste Meilensteine und Säule 3a (#25, #50, #52, #51)", () => {
  test("#25/AK1–AK4: Einstellungen anzeigen, prüfen, speichern, nach Neuladen behalten, Standardwerte", async ({ page }) => {
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

  test("#25/AK6, #51/AK2: obere Grenze Bezug Pensionskasse 67: 67 gültig, 68 abgelehnt", async ({ page }) => {
    const errors = await open(page, "milestones");
    await page.evaluate(() => Pref.set("limits", { ...LIMIT_DEFAULTS, pkMax: 67 }));
    const row = await msRow(page, "retire");
    const years = row.locator("input[data-type=years]");
    await years.fill("67");
    await expect(page.locator("#val li.err")).toHaveCount(0);
    await years.fill("68");
    await years.press("Tab"); // #57: Meldung am Feld erst beim Verlassen
    await expect(page.locator("#val li.err").first()).toContainText("«Bezug Pensionskasse»: Alter 68 liegt ausserhalb der Grenzen 58 bis 67 Jahre (Einstellungen).");
    await expect(row).toContainText("Alter 68 liegt ausserhalb");
    await noErrors(page, errors);
  });

  test("#51/AK1, AK5: feste Meilensteine mit festem Namen und eigener Person; Pensionskasse und AHV beginnen dort", async ({ page }) => {
    const errors = await open(page, "milestones");
    for (const id of ["retire", "ahv"]) {
      const row = await msRow(page, id);
      await expect(row.locator("input[data-path$='.name']")).toHaveCount(0);
      await expect(row.locator("select[data-path$='.person']")).toHaveCount(0);
    }
    await expect(await msRow(page, "retire")).toContainText("Bezug Pensionskasse");
    await expect(await msRow(page, "retire")).toContainText("58 bis 65 Jahre");
    await nav(page, "Vorsorge");
    await expect(page.locator("#main")).toContainText("Meilenstein «Bezug Pensionskasse»");
    await expect(page.locator("#main")).toContainText("Meilenstein «AHV-Bezug»");
    await expect(page.locator('#main select[data-path="pension.pk.at"]')).toHaveCount(0);
    // Zweite Person: eigene feste Meilensteine
    await page.evaluate(() => ACTIONS["partner-add"]());
    expect(await page.evaluate(() => state.milestones.filter(m => m.id.endsWith("2")).map(m => m.name))).toEqual(["Bezug Pensionskasse Person 2", "AHV-Bezug Person 2"]);
    await noErrors(page, errors);
  });

  test("#51/AK8: Warnung bei Erwerbseinkommen nach dem Bezug der Pensionskasse", async ({ page }) => {
    const errors = await open(page, "flows");
    await page.evaluate(() => { state.flows.income = { mode: "const", value: 12500 }; renderSection(); });
    await expect(page.locator("#val li.warn", { hasText: "Das Erwerbseinkommen beträgt nach dem Bezug der Pensionskasse" })).toContainText("noch 12'500 pro Monat");
    await noErrors(page, errors);
  });

  test("#52/AK1, AK2: Abschnitt Säule 3a mit Konto, Einzahlung und Auszahlungsplan", async ({ page }) => {
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

  test("#52/AK8: Einzahlung erscheint in «Ausgaben total» der Jahreswerte", async ({ page }) => {
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

  test("#50/AK4: Periode ab dem AHV-Bezug amortisiert direkt", async ({ page }) => {
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

  test("#51/AK6, #52/AK13: ältere Datei wird beim Öffnen übernommen, ein Dialog nennt die Änderungen", async ({ page }) => {
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
test.describe("Lebenshaltung inklusive Wohnen und Cash (#53, #54)", () => {
  test("#53/AK1, AK2: Beschriftung inklusive Wohnkosten, Hinweis mit dem Wohnanteil zum Simulationsstart", async ({ page }) => {
    const errors = await open(page, "flows");
    await expect(page.locator("#main")).toContainText("Lebenshaltungskosten / Monat inklusive Miete bzw. Wohnkosten (Zins, Amortisation, Nebenkosten)");
    // Beispielplanung: Wohnkosten zum Simulationsstart 1'700 pro Monat
    await expect(page.locator("#main .hint").filter({ hasText: "Wohnanteil zum Simulationsstart" })).toContainText("1'700");
    await noErrors(page, errors);
  });

  test("#53/AK4: Lebenshaltung kleiner als der Wohnanteil ist ein Fehler mit beiden Beträgen", async ({ page }) => {
    const errors = await open(page);
    const issues = await page.evaluate(async () => {
      const s = JSON.parse(JSON.stringify(state));
      s.flows.spending = { mode: "const", value: 1000 };
      return (await Engine.call("validate", { plan: s })).map(i => i.message);
    });
    expect(issues).toContain("Die Lebenshaltungskosten (1'000) sind kleiner als die Wohnkosten zum Simulationsstart (1'700), die sie enthalten.");
    await noErrors(page, errors);
  });

  test("#54/AK1, AK4: Aktienquote ohne Cash; Hinweis zur Cash-Regel vor und ab der Pensionierung", async ({ page }) => {
    const errors = await open(page, "wealth");
    await expect(page.locator("#main")).toContainText("Aktienquote (Aktien am Portfolio aus Aktien und Obligationen)");
    await expect(page.locator("#main")).toContainText("Cash zählt nicht zum Portfolio");
    await expect(page.locator("#main")).not.toContainText("Liquidität");
    await page.evaluate(() => { state.withdrawal.strategy = "standard"; goto("withdrawal"); });
    await expect(page.locator("#main")).toContainText("ab der Pensionierung");
    await noErrors(page, errors);
  });

  test("#53/AK7, #54/AK8: ältere Datei: Wohnanteil addiert, Hinweise zu Wohnkosten und Cash", async ({ page }) => {
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
test.describe("Säule 3a: Auszahlungsplan mit Datum oder Alter (#55)", () => {
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

  test("#55/AK1, AK2, AK8: Datum oder Alter; Monat und Alter daneben; beim Wechsel bleibt der Zeitpunkt; alte Zeilen sind «Datum»", async ({ page }) => {
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

  test("#55/AK3, AK4: frühestens 5 Jahre vor dem AHV-Bezug auch nach Alter; ohne Vorschau steht «–»", async ({ page }) => {
    const errors = await open(page);
    await setup(page, [{ kind: "age", years: 59.9, amount: 50000, use: "wealth" }]);
    await expect(page.locator("#val li.err").first()).toContainText("Die Auszahlung im 05.2040 liegt mehr als 5 Jahre vor dem AHV-Bezug (06.2045)");
    await expect(val(page, 0, "before")).toHaveText("–");
    await expect(val(page, 0, "after")).toHaveText("–");
    await expect(page.locator('#main [data-p3a="rest"]')).toHaveText("–");
    await noErrors(page, errors);
  });

  test("#55/AK4–AK7: Guthaben vor und nach der Auszahlung, Rest im AHV-Bezug; «+ Auszahlung» füllt Zeitpunkt und Betrag vor", async ({ page }) => {
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
    await field(page, "pillar3a.0.payouts.1.amount").press("Tab"); // #57: Vorschau beim Verlassen (#111)
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
test.describe("Fehlermeldung am Feld erst beim Verlassen (#57)", () => {
  const ferr = el => el.locator("xpath=ancestor-or-self::div[contains(@class,'inp')][1]/following-sibling::span[contains(@class,'ferr')]");
  /**
   * #111: Geprüft wird, dass während einer Pause in der Eingabe nichts geschieht. Statt fest zu warten, läuft die Seite mit
   * der Uhr von Playwright (page.clock): `pause()` stellt sie um 800 ms vor und führt dabei jeden fälligen Timer aus (Vorschau
   * nach 120 ms, Sammelzeit des Rückgängig-Verlaufs 400 ms), unabhängig von der Rechnerlast. Zusätzlich zählt die Seite die
   * Anfragen an den Rechenkern: Während der Eingabe darf keine starten.
   */
  async function openWithClock(page, hash) {
    await page.clock.install();
    const errors = await open(page, hash);
    await page.evaluate(() => {
      window.__api = [];
      const f = window.fetch;
      window.fetch = (u, ...a) => { if (String(u).startsWith("/api/")) window.__api.push(String(u)); return f(u, ...a); };
    });
    return errors;
  }
  const apiCalls = page => page.evaluate(() => window.__api.splice(0));
  const pause = async page => { await page.clock.runFor(800); expect(await apiCalls(page)).toEqual([]); };

  test("#57/AK1–AK4: Während der Eingabe wartet alles; beim Verlassen Meldung, Vorschau und Prüfliste neu", async ({ page }) => {
    const errors = await openWithClock(page, "milestones");
    const i = await page.evaluate(() => state.milestones.findIndex(m => m.id === "ahv"));
    const years = field(page, `milestones.${i}.years`);
    const end = kpiValue(page, "Vermögen am Ende");
    await expect(end).not.toHaveText("–");
    const before = await end.textContent();
    // AK1, AK4: Ziffer für Ziffer, mit Pause: keine Meldung am Feld, keine in der Prüfliste, Vorschau unverändert,
    // keine Anfrage an den Rechenkern (pause).
    await previewIdle(page); await apiCalls(page);
    await years.click(); await years.press("End");
    await years.press("Backspace"); await years.press("Backspace"); await years.type("6");
    await pause(page);
    await expect(ferr(years)).toHaveCount(0);
    await expect(page.locator("#val li.err")).toHaveCount(0);
    await expect(end).toHaveText(before);
    await years.type("3"); // 63: gültig, der Wert ist übernommen, gerechnet wird erst beim Verlassen
    await pause(page);
    expect(await page.evaluate(n => state.milestones[n].years, i)).toBe(63);
    await expect(end).toHaveText(before);
    await years.press("Tab");
    await expect(end).not.toHaveText(before);
    await expect(ferr(years)).toHaveCount(0);
    // AK2: Alter 6 eingeben: erst beim Verlassen Meldung am Feld und in der Prüfliste
    await previewIdle(page); await apiCalls(page); // Rechnungen nach dem Verlassen des Felds sind erlaubt, gezählt wird ab der nächsten Eingabe
    await years.fill("6");
    await pause(page);
    await expect(ferr(years)).toHaveCount(0);
    await expect(page.locator("#val li.err")).toHaveCount(0);
    await years.press("Tab");
    await expect(ferr(years)).toContainText("ausserhalb der Grenzen");
    await expect(page.locator("#val li.err").first()).toContainText("AHV-Bezug");
    // AK3: zurück ins Feld, gültig machen: Die Meldung bleibt während der Eingabe und verschwindet beim Verlassen
    await previewIdle(page); await apiCalls(page);
    await years.click(); await years.press("End"); await years.type("5");
    await pause(page);
    await expect(ferr(years)).toContainText("ausserhalb der Grenzen");
    await expect(years).toBeFocused();
    await years.press("Tab");
    await expect(ferr(years)).toHaveCount(0);
    await expect(page.locator("#val li.err")).toHaveCount(0);
    await expect(end).toHaveText(before);
    await noErrors(page, errors);
  });

  test("#57/AK1, AK2: ungültige Zahl erst beim Verlassen bzw. mit Enter gemeldet", async ({ page }) => {
    const errors = await openWithClock(page, "wealth");
    await previewIdle(page); await apiCalls(page);
    const cash = field(page, "wealth.cash");
    await cash.click(); await cash.press("End"); await cash.type("x");
    await pause(page);
    await expect(ferr(cash)).toHaveCount(0);
    await expect(cash).not.toHaveClass(/invalid/);
    await cash.press("Enter");
    await expect(ferr(cash)).toContainText("Ungültige Zahl.");
    await expect(cash).toHaveClass(/invalid/);
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Zahleneingabe mit Punkt als Tausendertrennzeichen (#140)", () => {
  const ferr = el => el.locator("xpath=ancestor-or-self::div[contains(@class,'inp')][1]/following-sibling::span[contains(@class,'ferr')]");

  test("AK1, AK4: Betrag «12.500» wird mit Meldung abgelehnt, der bisherige Wert bleibt", async ({ page }) => {
    const errors = await open(page, "wealth");
    const cash = field(page, "wealth.cash");
    const before = await page.evaluate(() => state.wealth.cash);
    await cash.fill("12.500");
    await cash.press("Tab");
    await expect(ferr(cash)).toContainText("Mehrdeutige Zahl");
    await expect(cash).toHaveClass(/invalid/);
    expect(await page.evaluate(() => state.wealth.cash)).toBe(before);
    // Mit Apostroph wird derselbe Betrag übernommen
    await cash.fill("12'500");
    await cash.press("Tab");
    await expect(cash).not.toHaveClass(/invalid/);
    expect(await page.evaluate(() => state.wealth.cash)).toBe(12500);
    await noErrors(page, errors);
  });

  test("AK1, AK2: Anzahl Pfade «50.000» und «1e5» werden abgelehnt, Prozent «1.25» bleibt gültig", async ({ page }) => {
    const errors = await open(page);
    await page.evaluate(() => { state.returns.source = "bootstrap"; });
    await nav(page, "Simulation");
    const paths = field(page, "returns.bootstrap.paths");
    const before = await page.evaluate(() => state.returns.bootstrap.paths);
    for (const t of ["50.000", "1e5"]) {
      await paths.fill(t);
      await paths.press("Tab");
      await expect(paths).toHaveClass(/invalid/);
      expect(await page.evaluate(() => state.returns.bootstrap.paths)).toBe(before);
    }
    await expect(ferr(paths)).toContainText("Ungültige Zahl.");
    await paths.fill("2000");
    await paths.press("Tab");
    await expect(paths).not.toHaveClass(/invalid/);
    expect(await page.evaluate(() => state.returns.bootstrap.paths)).toBe(2000);
    await noErrors(page, errors);
  });
});

test.describe("Erwerbseinkommen mit der Teuerung (#58)", () => {
  test("#58/AK1: Auswahl je Person, Vorschau rechnet sofort neu", async ({ page }) => {
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
    await nav(page, "Haushalt & Projekt");
    await page.getByRole("button", { name: "+ Zweite Person" }).click();
    await nav(page, "Einnahmen & Ausgaben");
    await expect(page.locator('input[data-path="partner.incomeIndexed"]')).toBeChecked();
    await expect(own).not.toBeChecked();
    await noErrors(page, errors);
  });

  test("#58: neue Planungen und neue zweite Person mit «steigt mit der Teuerung»", async ({ page }) => {
    const errors = await open(page);
    await page.locator('button[data-action="new"]').click();
    await page.waitForFunction(() => state.project.name === "Neue Planung");
    expect(await page.evaluate(() => state.flows.incomeIndexed)).toBe(true);
    await nav(page, "Einnahmen & Ausgaben");
    await expect(page.locator('input[data-path="flows.incomeIndexed"]')).toBeChecked();
    await noErrors(page, errors);
  });
});
