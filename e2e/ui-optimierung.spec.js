// Oberflächentests von VivaPlan im Browser: Optimierung. Jeder Test startet mit der Beispielplanung.
const { test, expect } = require("@playwright/test");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { MOD, SECTIONS, open, noErrors, field, previewIdle, nav, kpiValue, resultKpi, parseNum, setOpt, msRow, welcome, steps, helpTitle, prefs } = require("./helpers");

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
    const errors = await open(page, "optimize/bootstrap/optset");
    const small = { paths: 300, checkPaths: 500, samples: 6, strategies: ["fixed_rate"] };
    expect(await setOpt(page, small)).toMatchObject(small); // #79: die kleine Rechnung kommt wirklich in den Feldern an
    await page.locator('button[data-action="opt-run"]').click();
    await expect(page.locator("#main .job progress")).toBeVisible();
    await expect(page.locator("html")).toHaveAttribute("data-job", "off");
    await expect(page.locator("#main th", { hasText: "Entnahmerate" })).toBeVisible();
    await noErrors(page, errors);
  });

  test("#6/AK1, AK9, AK10: 3-Topf mit Guardrails – Auswahl, Karten, Name, Tooltip und Optimierung", async ({ page }) => {
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
    // AK9: Name und Parameter (Tooltip #11) mit Töpfen und Guardrails
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

  test("#73: Quote von Aktien und Obligationen neben den Beständen", async ({ page }) => {
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
    await previewIdle(page); // #111: keine Vorschau des vorigen Schritts mehr unterwegs
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

  test("#18: Cash-Reserve nur in der Karte «Standard»; Hinweis beim Laden bei anderer Strategie und Reserve ≠ 3", async ({ page }) => {
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
    // #74/AK3, AK5: einmal gezeigt, gemerkt in den Einstellungen; erneutes Öffnen zeigt keinen Hinweis
    expect(await page.evaluate(() => Pref.get("reserveNoticeShown", false))).toBe(true);
    await page.locator("#fileInput").setInputFiles(file);
    await expect(page.locator("#fileName")).toHaveText(path.basename(file));
    // #111/AK4: Der Handler leert das Dateifeld erst nach den Hinweisen (showNotes, reserveNotice). Erschiene der Hinweis,
    // bliebe er beim Dialog stehen und das Feld gefüllt.
    await expect(page.locator("#fileInput")).toHaveValue("");
    await expect(page.locator("#ask")).toBeHidden();
    // Standard mit Reserve 6: kein Hinweis; der gespeicherte Wert bleibt
    expect(await page.evaluate(() => reserveDiffers({ withdrawal: { strategy: "standard" }, wealth: { reserveMonths: { mode: "const", value: 6 } } }))).toBe(false);
    expect(await page.evaluate(() => reserveDiffers({ withdrawal: { strategy: "vpw" }, wealth: { reserveMonths: { mode: "const", value: 3 } } }))).toBe(false);
    expect(await page.evaluate(() => reserveDiffers({ withdrawal: { strategy: "vpw" }, wealth: { reserveMonths: { mode: "rows", rows: [{ value: 3 }, { value: 12 }] } } }))).toBe(true);
    expect(await page.evaluate(() => state.wealth.reserveMonths.value)).toBe(6);
    fs.rmSync(file);
    await noErrors(page, errors);
  });

  test.describe("#4: Start mit zuletzt geöffneter Planung, Rückfrage beim Beenden (Desktop-App simuliert)", () => {
    /** Simuliert die Desktop-App: Befehle der Tauri-Schicht werden nachgebildet, alle anderen gehen an den Rechenkern. */
    async function openApp(page, startup, settings = {}) {
      settings = { disclaimerAccepted: 1, ...settings }; // #61: Haftungsausschluss gilt als bestätigt, ausser ein Test sagt anders
      await page.addInitScript(([startup, settings]) => {
        window.__calls = [];
        window.__TAURI__ = {
          core: {
            invoke: async (cmd, args = {}) => {
              window.__calls.push([cmd, args]);
              if (cmd === "startup_plan") return startup;
              if (cmd === "save_plan") { if (window.__saveError) throw window.__saveError; return window.__saveResult ?? null; }
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

    test("#61/AK1, AK2: Haftungsausschluss beim ersten Start; ohne Häkchen und mit Esc geht es nicht weiter; Bestätigung in den Einstellungen", async ({ page }) => {
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

    test("#61/AK2: bestätigte Version fragt nicht mehr, eine ältere Version fragt erneut", async ({ page }) => {
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

    test("#74/AK1, AK5: erster Start mit betroffener Planung zeigt den Hinweis einmal und merkt ihn in den Einstellungen", async ({ page }) => {
      await openApp(page, { path: "/x/Alt.vplan", plan: await affected(page), notice: null });
      await expect(page.locator("#ask")).toBeVisible();
      await expect(page.locator("#askTitle")).toHaveText("Hinweis zur Cash-Reserve");
      const set = await page.evaluate(() => window.__calls.filter(([c]) => c === "settings_set").map(([, a]) => a));
      expect(set).toContainEqual({ key: "reserveNoticeShown", value: true });
    });

    test("#74/AK2: weiterer Start mit betroffener Planung zeigt keinen Hinweis", async ({ page }) => {
      await openApp(page, { path: "/x/Alt.vplan", plan: await affected(page), notice: null }, { reserveNoticeShown: true });
      await expect(page.locator("#fileName")).toHaveText("Alt.vplan");
      // #83: openApp wartet auf data-ready; das setzt die App erst nach dem Entscheid über den Hinweis
      await expect(page.locator("#ask")).toBeHidden();
    });

    test("#74/AK4: nicht betroffene Planung zählt nicht; die erste betroffene zeigt den Hinweis, die nächste nicht", async ({ page }) => {
      await openApp(page, { path: "/x/Standard.vplan", plan: await plan(page), notice: null });
      await expect(page.locator("#fileName")).toHaveText("Standard.vplan");
      // #83: openApp wartet auf data-ready; das setzt die App erst nach dem Entscheid über den Hinweis
      await expect(page.locator("#ask")).toBeHidden();
      expect(await page.evaluate(() => Pref.get("reserveNoticeShown", false))).toBe(false);
      const p = await affected(page);
      await page.evaluate(p => { reserveNotice(p); }, p);
      await expect(page.locator("#askTitle")).toHaveText("Hinweis zur Cash-Reserve");
      await page.locator('#ask button[value="ok"]').click();
      await expect(page.locator("#ask")).toBeHidden();
      // #83: reserveNotice entscheidet synchron und öffnet den Dialog synchron; kein Warten nötig
      await page.evaluate(p => { reserveNotice(p); }, p);
      await expect(page.locator("#ask")).toBeHidden();
    });

    test("Datei fehlt: Beispielplanung als «Neue Planung» mit Hinweis und Dateiname", async ({ page }) => {
      await openApp(page, { path: null, plan: await plan(page), notice: "Zuletzt geöffnete Datei nicht gefunden: weg.vplan. Es wird die Beispielplanung geöffnet." });
      await expect(page.locator("#fileName")).toHaveText("Neue Planung");
      await expect(page.locator("#ask")).toBeVisible();
      await expect(page.locator("#askText")).toContainText("Zuletzt geöffnete Datei nicht gefunden");
      await expect(page.locator("#askText")).toContainText("weg.vplan");
    });

    test("#136/AK4: Fehler beim Speichern: Meldung, die Planung bleibt ungespeichert", async ({ page }) => {
      await openApp(page, { path: "/x/planung.vplan", plan: await plan(page), notice: null });
      const calls = name => page.evaluate(n => window.__calls.filter(c => c[0] === n), name);
      await field(page, "project.name").fill("Geändert");
      await field(page, "project.name").blur();
      await expect(page.locator("#dirty")).toHaveText("● ungespeichert");
      // Die Desktop-App meldet den Fehler von write_plan als Text (Tauri-Befehl mit Result<_, String>).
      const saveBtn = page.getByRole("button", { name: "Speichern", exact: true });
      await page.evaluate(() => { window.__saveError = "/x/planung.vplan: Permission denied (os error 13)"; });
      await saveBtn.click();
      await expect.poll(async () => (await calls("save_plan")).length).toBe(1);
      await expect(page.locator("#toast")).toContainText("Permission denied");
      await expect(page.locator("#dirty")).toHaveText("● ungespeichert");
      expect((await calls("set_dirty")).at(-1)[1].dirty).toBe(true);
      // Gelingt es danach, gilt die Planung als gespeichert.
      await page.evaluate(() => { window.__saveError = null; window.__saveResult = "/x/planung.vplan"; });
      await saveBtn.click();
      await expect(page.locator("#dirty")).toHaveText("");
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
    test("Beenden: Speichern beendet erst nach erfolgreichem Speichern; bricht die Ortsabfrage ab, bleibt die App offen (#27)", async ({ page }) => {
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

  test("#24: Stufe der Variationen in «Einzelne Läufe» und «Jahreswerte» wählen, ohne Neuberechnung aller Pfade", async ({ page }) => {
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

  test("#20: Zielauswahl, Ziel 2 «Kein Notverkauf von Immobilien» wird gespeichert, gerechnet und mit Anteil ungeplanter Verkäufe angezeigt", async ({ page }) => {
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
    expect(await setOpt(page, small)).toMatchObject({ ...small, goal: "keepProperty", minConsumption: 80 }); // #79
    await page.locator('button[data-action="opt-run"]').click();
    await expect(page.locator("html")).toHaveAttribute("data-job", "off");
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

  test("#29: Grundstückgewinnsteuer ist Pflicht bei Immobilie im Besitz oder mit geplantem Kauf, nicht bei Miete; Start blockiert", async ({ page }) => {
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

  // Kriterien: #32/AK1 #32/AK2 #32/AK3 #32/AK4 #32/AK5 #32/AK6
  test("#32: Immobilien mit gleicher Feldreihenfolge bei Besitz und Kauf geplant, Marktwert in der Kopfzeile", async ({ page }) => {
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

  test("#28: Bei «Kauf geplant» gibt es «Verkauf ab» (Vorgabe nie); Verkauf vor oder im Kaufmonat ist ein Fehler am Feld", async ({ page }) => {
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
