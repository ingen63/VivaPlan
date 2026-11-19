// User interface tests of VivaPlan in the browser: inputs, milestones, undo, files. Every test starts with the example plan.
const { test, expect } = require("@playwright/test");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { MOD, SECTIONS, open, noErrors, field, previewIdle, nav, kpiValue, resultKpi, parseNum, setOpt, msRow, welcome, steps, helpTitle, prefs } = require("./helpers");

// ---------------------------------------------------------------------------------------------
test.describe("Eingaben, Prüfung und Live-Vorschau", () => {
  test("Vorschau rechnet beim Verlassen des Feldes neu (#57)", async ({ page }) => {
    const errors = await open(page);
    await nav(page, "Vermögen & Anlage");
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
    // Test access computes directly; back to the editor, remember the value, compute again and compare.
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
    await birth.press("Tab"); // #57
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
    await nav(page, "Meilensteine");
    const endRow = await msRow(page, "end");
    await endRow.locator("select").first().selectOption("date");
    await endRow.locator("input[data-type=month]").fill("01.2000");
    await endRow.locator("input[data-type=month]").press("Tab"); // #57: check on leaving (#111: do not wait for a preview)
    await expect(page.locator("#val li.err").first()).toBeVisible();
    await expect(page.locator("#nav a .badge.err").first()).toBeVisible();
    await page.keyboard.press(`${MOD}+Enter`);
    // #14: Start is blocked, the message names the number of errors, the link leads to the affected section
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
    await birth.press("Tab"); // #57: check list only on leaving
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
    await field(page, "project.name").press("Tab");
    await nav(page, "Vermögen & Anlage");
    const stocks = field(page, "wealth.stocks");
    await stocks.fill("");
    await stocks.press("Tab"); // #57
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
    await nav(page, "Meilensteine");
    const row = await msRow(page, "retire");
    await row.locator("select").first().selectOption("date");
    const date = row.locator("input[data-type=month]");
    await date.fill("01.2030");
    await date.press("Tab"); // #57: derived values only on leaving
    const result = row.locator('[data-computed="ms"]');
    await expect(result).toContainText("01.2030");
    const age1 = await result.textContent();
    await date.fill("01.2029");
    await date.press("Tab");
    await expect(result).toContainText("01.2029");
    expect(await result.textContent()).not.toBe(age1);
    // Preview and timeline follow the new date
    await expect(kpiValue(page, "Vermögen bei Pensionierung").locator("xpath=../div[@class='s']")).toHaveText("01.2029");
    await expect(page.locator("#val li.err")).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("#51/AK2: Bezug Pensionskasse ausserhalb der Grenzen: Fehler am Feld statt Zahlen", async ({ page }) => {
    const errors = await open(page);
    await nav(page, "Meilensteine");
    const row = await msRow(page, "retire");
    await row.locator("select").first().selectOption("date");
    await row.locator("input[data-type=month]").fill("01.2060");
    await row.locator("input[data-type=month]").press("Tab"); // #57: check on leaving
    await expect(page.locator("#val li.err").first()).toContainText("«Bezug Pensionskasse»: Alter 89.6 liegt ausserhalb der Grenzen 58 bis 65 Jahre");
    await expect(page.locator("#kpis .kpi")).toHaveCount(1);
    await expect(page.locator("#kpis .kpi .s")).toHaveText("siehe Prüfung");
    await noErrors(page, errors);
  });

  test("Meilenstein nach Alter und nach Datum ergeben denselben Monat", async ({ page }) => {
    const errors = await open(page);
    await nav(page, "Meilensteine");
    const row = await msRow(page, "retire");
    await row.locator("select").first().selectOption("age");
    await row.locator("input[data-type=years]").fill("64");
    await row.locator("input[data-type=years]").press("Tab"); // #57: computed on leaving
    await expect(row.locator('[data-computed="ms"]')).toContainText("Alter 64.0");
    const byAge = await row.locator('[data-computed="ms"]').textContent();
    const m = /(\d{2}\.\d{4})/.exec(byAge)[1];
    await row.locator("select").first().selectOption("date");
    await row.locator("input[data-type=month]").fill(m);
    await row.locator("input[data-type=month]").press("Tab");
    await expect(row.locator('[data-computed="ms"]')).toHaveText(byAge);
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Rückgängig und Wiederholen", () => {
  const reset = page => page.evaluate(() => { lastCommand = { id: "", t: 0 }; });

  test("100 Aktionen vollständig zurück- und wiederherstellen", async ({ page }) => {
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
    for (const text of ["", "kein json", "{}", JSON.stringify({ format: 9 })]) {
      fs.writeFileSync(bad, text);
      await page.locator("#fileInput").setInputFiles(bad);
      await expect(page.locator("#toast")).toContainText("keine Projektdatei");
      await expect(field(page, "project.name")).toBeVisible();
    }
    fs.rmSync(bad);
    await noErrors(page, errors);
  });

  test.describe("Unbekannte Formatversion beim Öffnen (#144)", () => {
    test("AK2, AK4: Eine Datei aus einer neueren Version wird mit Kennung in der Meldung abgelehnt", async ({ page }) => {
      const errors = await open(page);
      await field(page, "project.name").fill("Bleibt");
      await field(page, "project.name").press("Tab");
      const file = path.join(os.tmpdir(), `qa-format-${process.pid}.vplan`);
      const plan = await (await page.request.post("/api/example_plan", { data: {} })).json();
      for (const format of ["vivaplan/2", "andere/9"]) {
        fs.writeFileSync(file, JSON.stringify({ ...plan, format, project: { name: "Neuere Version" } }));
        await page.locator("#fileInput").setInputFiles(file);
        await expect(page.locator("#toast")).toContainText(`Die Datei hat das Format «${format}». Sie braucht eine neuere Version von VivaPlan`);
        await expect(field(page, "project.name")).toHaveValue("Bleibt");
      }
      // AK1: the current and the old identifier open as before
      for (const format of ["vivaplan/1", "pensionsplaner/1"]) {
        fs.writeFileSync(file, JSON.stringify({ ...plan, format, project: { name: `Kennung ${format}` } }));
        await page.locator("#fileInput").setInputFiles(file);
        await expect(page.locator("#toast")).toContainText("geöffnet");
        await expect(field(page, "project.name")).toHaveValue(`Kennung ${format}`);
      }
      fs.rmSync(file);
      await noErrors(page, errors);
    });
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
