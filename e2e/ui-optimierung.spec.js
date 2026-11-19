// User interface tests of VivaPlan in the browser: optimization. Every test starts with the example plan.
const { test, expect } = require("@playwright/test");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { MOD, SECTIONS, open, noErrors, field, previewIdle, nav, kpiValue, resultKpi, parseNum, setOpt, msRow, welcome, steps, helpTitle, prefs } = require("./helpers");

// ---------------------------------------------------------------------------------------------
test.describe("Optimization: ranges, withdrawal rate, progress", () => {
  test("parameters fixed or varied; default ranges restore", async ({ page }) => {
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
    // collapsed/expanded show all strategies; default ranges delete the changes
    await page.locator('button[data-action="opt-range-reset"][data-s="three_buckets"]').click();
    expect(await page.evaluate(() => state.optimization?.ranges?.three_buckets)).toBeUndefined();
    await noErrors(page, errors);
  });

  test("optimization runs with progress, withdrawal rate appears in the result", async ({ page }) => {
    const errors = await open(page, "optimize/bootstrap/optset");
    const small = { paths: 300, checkPaths: 500, samples: 6, strategies: ["fixed_rate"] };
    expect(await setOpt(page, small)).toMatchObject(small); // #79: the small computation really arrives in the fields
    await page.locator('button[data-action="opt-run"]').click();
    await expect(page.locator("#main .job progress")).toBeVisible();
    await expect(page.locator("html")).toHaveAttribute("data-job", "off");
    await expect(page.locator("#main th", { hasText: "Entnahmerate" })).toBeVisible();
    await noErrors(page, errors);
  });

  test("#6/AK1, AK9, AK10: 3-bucket with guardrails – selection, cards, name, tooltip and optimization", async ({ page }) => {
    const errors = await open(page);
    await page.evaluate(() => { state.withdrawal.strategy = "three_buckets"; delete state.withdrawal.buckets.spending; goto("withdrawal"); });
    const sel = page.locator('#main select[data-path="withdrawal.buckets.spending"]');
    // AK1: default «wie geplant», without guardrails and limits card
    await expect(sel).toHaveValue("planned");
    await expect(page.locator('#main input[data-path="withdrawal.guardrails.initialRate"]')).toHaveCount(0);
    await expect(page.locator('#main input[data-path="withdrawal.floor"]')).toHaveCount(0);
    await sel.selectOption("guardrails");
    await expect(page.locator('#main input[data-path="withdrawal.guardrails.initialRate"]')).toBeVisible();
    await expect(page.locator('#main input[data-path="withdrawal.floor"]')).toBeVisible();
    await expect(page.locator('#main input[data-path="withdrawal.buckets.cashYears"]')).toBeVisible();
    expect(await page.evaluate(() => state.withdrawal.buckets.spending)).toBe("guardrails");
    // AK9: name and parameters (tooltip #11) with buckets and guardrails
    const info = await page.evaluate(() => ({ name: stratName(stratKey(state.withdrawal)), params: stratParams(state).map(x => x[0]) }));
    expect(info.name).toBe("3-Topf + Guardrails");
    expect(info.params).toEqual(expect.arrayContaining(["Topf 1 · Cash", "Startrate bei Pensionierung", "Untergrenze"]));
    expect(await page.evaluate(() => stratName(stratKey({ strategy: "three_buckets", buckets: { spending: "planned" } })))).toBe("3-Topf-Strategie");
    // AK10: offered in the optimization, not selected by default
    await page.evaluate(() => { delete state.optimization; goto("optimize"); });
    const box = page.locator("#optS-three_buckets_guardrails");
    await expect(box).toBeAttached();
    await expect(box).not.toBeChecked();
    await expect(page.locator("#optS-three_buckets")).toBeChecked();
    // Back to «wie geplant»: cards disappear
    await page.evaluate(() => goto("withdrawal"));
    await sel.selectOption("planned");
    await expect(page.locator('#main input[data-path="withdrawal.guardrails.initialRate"]')).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("#73: ratio of stocks and bonds next to the holdings", async ({ page }) => {
    const errors = await open(page);
    await page.evaluate(() => goto("wealth"));
    const quote = p => page.locator(`label.field:has(input[data-path="${p}"]) [data-computed="wquote"]`);
    const set = async (p, v) => { await field(page, p).fill(v); await field(page, p).blur(); };
    // AK1: 80 000 / 20 000 / 50 000
    await set("wealth.stocks", "80000"); await set("wealth.bonds", "20000"); await set("wealth.cash", "50000");
    await expect(quote("wealth.stocks")).toHaveText("80 % des Portfolios");
    await expect(quote("wealth.bonds")).toHaveText("20 % des Portfolios");
    // AK5: cash does not change the ratios
    for (const c of ["0", "500000"]) {
      await set("wealth.cash", c);
      await expect(quote("wealth.stocks")).toHaveText("80 % des Portfolios");
      await expect(quote("wealth.bonds")).toHaveText("20 % des Portfolios");
    }
    // AK3: unchanged during input, updated after leaving the field
    await previewIdle(page); // #111: no preview of the previous step on its way any more
    await field(page, "wealth.stocks").fill("60000");
    await expect(quote("wealth.stocks")).toHaveText("80 % des Portfolios");
    await field(page, "wealth.stocks").blur();
    await expect(quote("wealth.stocks")).toHaveText("75 % des Portfolios");
    await expect(quote("wealth.bonds")).toHaveText("25 % des Portfolios");
    // AK2: ganzzahlig gerundet
    await set("wealth.stocks", "1"); await set("wealth.bonds", "2");
    await expect(quote("wealth.stocks")).toHaveText("33 % des Portfolios");
    await expect(quote("wealth.bonds")).toHaveText("67 % des Portfolios");
    // AK4: both 0 or empty: no ratio
    await set("wealth.stocks", "0"); await set("wealth.bonds", "0");
    await expect(quote("wealth.stocks")).toHaveText("");
    await expect(quote("wealth.bonds")).toHaveText("");
    await set("wealth.stocks", ""); await set("wealth.bonds", "");
    await expect(quote("wealth.stocks")).toHaveText("");
    await expect(page.locator("#main")).not.toContainText("NaN");
    // AK6: the ratio is not stored in the plan
    const example = await (await page.request.post("/api/example_plan", { data: {} })).json();
    const keys = Object.keys(await page.evaluate(() => state.wealth));
    expect(keys.filter(k => !(k in example.wealth))).toEqual([]);
    expect(JSON.stringify(await page.evaluate(() => state))).not.toContain("des Portfolios");
    await noErrors(page, errors);
  });

  test("#18: cash reserve only in the default card; notice on loading with another strategy and reserve ≠ 3", async ({ page }) => {
    const errors = await open(page);
    // (1) «Vermögen & Anlage» no longer contains the reserve, the card «Standard» does
    await page.evaluate(() => goto("wealth"));
    await expect(page.locator("#main")).toContainText("Aktienquote (Aktien am Portfolio aus Aktien und Obligationen)");
    await expect(page.locator("#main")).not.toContainText("Cash-Reserve (Monate");
    await expect(page.locator('#main [data-path="wealth.reserveMonths"], #main [data-series="wealth.reserveMonths"]')).toHaveCount(0);
    await page.evaluate(() => { state.withdrawal.strategy = "standard"; goto("withdrawal"); });
    await expect(page.locator('#main input[data-path^="wealth.reserveMonths"]').first()).toBeVisible();
    // other strategy: no reserve can be set
    await page.evaluate(() => { state.withdrawal.strategy = "guardrails"; goto("withdrawal"); });
    await expect(page.locator('#main input[data-path^="wealth.reserveMonths"]')).toHaveCount(0);
    // (3) plan with guardrails and reserve 6: notice on loading, once
    const plan = await page.evaluate(() => { const s = JSON.parse(JSON.stringify(state)); s.withdrawal.strategy = "guardrails"; s.wealth.reserveMonths = { mode: "const", value: 6 }; return s; });
    const file = path.join(os.tmpdir(), `fea17-${process.pid}.vplan`);
    fs.writeFileSync(file, JSON.stringify(plan));
    await page.locator("#fileInput").setInputFiles(file);
    await expect(page.locator("#ask")).toBeVisible();
    await expect(page.locator("#askTitle")).toHaveText("Hinweis zur Cash-Reserve");
    await expect(page.locator("#askText")).toContainText("3 Monaten");
    await page.locator('#ask button[value="ok"]').click();
    await expect(page.locator("#ask")).toBeHidden();
    // #74/AK3, AK5: shown once, remembered in the settings; opening again shows no notice
    expect(await page.evaluate(() => Pref.get("reserveNoticeShown", false))).toBe(true);
    await page.locator("#fileInput").setInputFiles(file);
    await expect(page.locator("#fileName")).toHaveText(path.basename(file));
    // #111/AK4: The handler clears the file field only after the notices (showNotes, reserveNotice). If the notice appeared,
    // it would stay at the dialog and the field would stay filled.
    await expect(page.locator("#fileInput")).toHaveValue("");
    await expect(page.locator("#ask")).toBeHidden();
    // Standard with reserve 6: no notice; the saved value remains
    expect(await page.evaluate(() => reserveDiffers({ withdrawal: { strategy: "standard" }, wealth: { reserveMonths: { mode: "const", value: 6 } } }))).toBe(false);
    expect(await page.evaluate(() => reserveDiffers({ withdrawal: { strategy: "vpw" }, wealth: { reserveMonths: { mode: "const", value: 3 } } }))).toBe(false);
    expect(await page.evaluate(() => reserveDiffers({ withdrawal: { strategy: "vpw" }, wealth: { reserveMonths: { mode: "rows", rows: [{ value: 3 }, { value: 12 }] } } }))).toBe(true);
    expect(await page.evaluate(() => state.wealth.reserveMonths.value)).toBe(6);
    fs.rmSync(file);
    await noErrors(page, errors);
  });

  test.describe("#4: start with the plan opened last, question on quitting (desktop app simulated)", () => {
    /** Simulates the desktop app: commands of the Tauri layer are emulated, all others go to the engine. */
    async function openApp(page, startup, settings = {}) {
      settings = { disclaimerAccepted: 1, ...settings }; // #61: The disclaimer counts as confirmed unless a test says otherwise
      await page.addInitScript(([startup, settings]) => {
        window.__calls = [];
        window.__TAURI__ = {
          core: {
            invoke: async (cmd, args = {}) => {
              window.__calls.push([cmd, args]);
              if (cmd === "startup_plan") return startup;
              if (cmd === "save_plan") { if (window.__saveError) throw window.__saveError; return window.__saveResult ?? null; }
              if (["set_dirty", "quit_app", "settings_set", "set_window_title", "set_menu_language", "first_start_language"].includes(cmd)) return null;
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

    test("#61/AK1, AK2: disclaimer on the first start; without the check mark and with Esc it does not continue; confirmation in the settings", async ({ page }) => {
      const p = await plan(page);
      await openApp(page, { path: null, plan: p, notice: null, first: true }, { disclaimerAccepted: null });
      const d = page.locator("#disclaimer"), ok = page.locator("#disclaimerOk");
      await expect(d).toHaveAttribute("open", "");
      await expect(d).toContainText("VivaPlan ist ein kostenloses Hilfsmittel zur persönlichen Planung.");
      await expect(d).toContainText("Prüfen Sie wichtige Entscheidungen mit einer Fachperson.");
      await expect(d).toContainText("Ich habe den Hinweis gelesen und verstanden.");
      // The controls wait until it is confirmed
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
      // not in the plan
      expect(JSON.stringify(await page.evaluate(() => state))).not.toContain("disclaimer");
    });

    test("#61/AK2: a confirmed version does not ask any more, an older version asks again", async ({ page }) => {
      const p = await plan(page);
      await openApp(page, { path: "/x/A.vplan", plan: p, notice: null }, { disclaimerAccepted: 1 });
      await expect(page.locator("#disclaimer")).not.toHaveAttribute("open", "");
      await openApp(page, { path: "/x/A.vplan", plan: p, notice: null }, { disclaimerAccepted: 0 });
      await expect(page.locator("#disclaimer")).toHaveAttribute("open", "");
    });

    test("remembered file: name in the header, no notice; without one the example plan as new plan", async ({ page }) => {
      const p = await plan(page);
      p.project.name = "Gemerkte Planung";
      await openApp(page, { path: "/x/Familie.vplan", plan: p, notice: null });
      await expect(page.locator("#fileName")).toHaveText("Familie.vplan");
      await expect(page.locator("#ask")).toBeHidden();
      await expect(field(page, "project.name")).toHaveValue("Gemerkte Planung");
    });

    /** Plan with guardrails and a reserve of 6 months (affected by the notice on the cash reserve). */
    const affected = async page => { const p = await plan(page); p.withdrawal.strategy = "guardrails"; p.wealth.reserveMonths = { mode: "const", value: 6 }; return p; };

    test("#74/AK1, AK5: first start with an affected plan shows the notice once and remembers it in the settings", async ({ page }) => {
      await openApp(page, { path: "/x/Alt.vplan", plan: await affected(page), notice: null });
      await expect(page.locator("#ask")).toBeVisible();
      await expect(page.locator("#askTitle")).toHaveText("Hinweis zur Cash-Reserve");
      const set = await page.evaluate(() => window.__calls.filter(([c]) => c === "settings_set").map(([, a]) => a));
      expect(set).toContainEqual({ key: "reserveNoticeShown", value: true });
    });

    test("#74/AK2: a further start with an affected plan shows no notice", async ({ page }) => {
      await openApp(page, { path: "/x/Alt.vplan", plan: await affected(page), notice: null }, { reserveNoticeShown: true });
      await expect(page.locator("#fileName")).toHaveText("Alt.vplan");
      // #83: openApp waits for data-ready; the app sets it only after deciding on the notice
      await expect(page.locator("#ask")).toBeHidden();
    });

    test("#74/AK4: an unaffected plan does not count; the first affected one shows the notice, the next one does not", async ({ page }) => {
      await openApp(page, { path: "/x/Standard.vplan", plan: await plan(page), notice: null });
      await expect(page.locator("#fileName")).toHaveText("Standard.vplan");
      // #83: openApp waits for data-ready; the app sets it only after deciding on the notice
      await expect(page.locator("#ask")).toBeHidden();
      expect(await page.evaluate(() => Pref.get("reserveNoticeShown", false))).toBe(false);
      const p = await affected(page);
      await page.evaluate(p => { reserveNotice(p); }, p);
      await expect(page.locator("#askTitle")).toHaveText("Hinweis zur Cash-Reserve");
      await page.locator('#ask button[value="ok"]').click();
      await expect(page.locator("#ask")).toBeHidden();
      // #83: reserveNotice decides synchronously and opens the dialog synchronously; no waiting needed
      await page.evaluate(p => { reserveNotice(p); }, p);
      await expect(page.locator("#ask")).toBeHidden();
    });

    test("file missing: example plan as new plan with notice and file name", async ({ page }) => {
      await openApp(page, { path: null, plan: await plan(page), notice: { key: "startup.notFound", file: "weg.vplan" } });
      await expect(page.locator("#fileName")).toHaveText("Neue Planung");
      await expect(page.locator("#ask")).toBeVisible();
      await expect(page.locator("#askText")).toContainText("Zuletzt geöffnete Datei nicht gefunden");
      await expect(page.locator("#askText")).toContainText("weg.vplan");
    });

    test("#136/AK4: error when saving: message, the plan stays unsaved", async ({ page }) => {
      await openApp(page, { path: "/x/planung.vplan", plan: await plan(page), notice: null });
      const calls = name => page.evaluate(n => window.__calls.filter(c => c[0] === n), name);
      await field(page, "project.name").fill("Geändert");
      await field(page, "project.name").blur();
      await expect(page.locator("#dirty")).toHaveText("● ungespeichert");
      // The desktop app reports the error of write_plan as text (Tauri command with Result<_, String>).
      const saveBtn = page.getByRole("button", { name: "Speichern", exact: true });
      await page.evaluate(() => { window.__saveError = "/x/planung.vplan: Permission denied (os error 13)"; });
      await saveBtn.click();
      await expect.poll(async () => (await calls("save_plan")).length).toBe(1);
      await expect(page.locator("#toast")).toContainText("Permission denied");
      await expect(page.locator("#dirty")).toHaveText("● ungespeichert");
      expect((await calls("set_dirty")).at(-1)[1].dirty).toBe(true);
      // If it succeeds afterwards, the plan counts as saved.
      await page.evaluate(() => { window.__saveError = null; window.__saveResult = "/x/planung.vplan"; });
      await saveBtn.click();
      await expect(page.locator("#dirty")).toHaveText("");
    });

    test("quit: without changes immediately, with changes dialog save/discard/cancel", async ({ page }) => {
      await openApp(page, { path: null, plan: await plan(page), notice: null });
      const calls = name => page.evaluate(n => window.__calls.filter(c => c[0] === n), name);
      // without changes: quit directly
      await page.evaluate(() => { window.__listeners["quit-requested"](); });
      await expect.poll(async () => (await calls("quit_app")).length).toBe(1);
      // with changes the user interface reports the state and asks
      await field(page, "project.name").fill("Geändert");
      await field(page, "project.name").blur();
      await expect.poll(async () => (await calls("set_dirty")).some(c => c[1].dirty === true)).toBe(true);
      await page.evaluate(() => { window.__listeners["quit-requested"](); });
      await expect(page.locator("#ask")).toBeVisible();
      await expect(page.locator('#ask button[value="ok"]')).toHaveText("Speichern");
      await expect(page.locator('#ask button[value="discard"]')).toBeVisible();
      // Cancel: stays open
      await page.locator('#ask button[value="cancel"]').click();
      await expect(page.locator("#ask")).toBeHidden();
      expect((await calls("quit_app")).length).toBe(1);
      // Verwerfen: beendet
      await page.evaluate(() => { window.__listeners["quit-requested"](); });
      await page.locator('#ask button[value="discard"]').click();
      await expect.poll(async () => (await calls("quit_app")).length).toBe(2);
    });
    test("quit: save quits only after saving successfully; if the location dialog is cancelled, the app stays open (#27)", async ({ page }) => {
      await openApp(page, { path: null, plan: await plan(page), notice: null });
      const calls = name => page.evaluate(n => window.__calls.filter(c => c[0] === n), name);
      await field(page, "project.name").fill("Geändert"); await field(page, "project.name").blur();
      await page.evaluate(() => { window.__listeners["quit-requested"](); });
      await page.locator('#ask button[value="ok"]').click();
      await expect.poll(async () => (await calls("save_plan")).length).toBe(1);
      await expect(page.locator("#ask")).toBeHidden();
      expect((await calls("quit_app")).length).toBe(0);   // location dialog cancelled: app stays open
      await page.evaluate(() => { window.__saveResult = "/x/gespeichert.vplan"; window.__listeners["quit-requested"](); });
      await page.locator('#ask button[value="ok"]').click();
      await expect.poll(async () => (await calls("quit_app")).length).toBe(1);
      await expect(page.locator("#fileName")).toHaveText("gespeichert.vplan");
    });
  });

  test("switching a milestone from date to age sets person and years (plan stays valid)", async ({ page }) => {
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

  test("#24: select the level of the variations in single runs and annual values without recomputing all paths", async ({ page }) => {
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
    expect(out.calls).toEqual(["rerun_paths"]);                 // no full recomputation
    expect(out.endShown).toBeCloseTo(out.expected, 4);          // matches the scenario table
    expect(out.tableHasPicker).toBe(2);                          // also applies in «Jahreswerte»
    expect(out.note).toContain("Planwerte, nicht die gewählte Variation");
    expect(out.back).toBeNull();
    await noErrors(page, errors);
  });

  test("#20: goal selection, goal 2 (no emergency sale of real estate) is saved, computed and displayed with the share of unplanned sales", async ({ page }) => {
    const errors = await open(page, "optimize/bootstrap");
    // (1) selection present, default is goal 1, the plan stores nothing additional
    await expect(page.locator("#opt-goal")).toHaveValue("consumption");
    await expect(page.locator("#opt-weightMedian")).toBeVisible();
    expect(await page.evaluate(() => state.optimization?.goal)).toBeUndefined();
    await expect(page.locator("#opt-goal option[value=keepProperty]")).toHaveText("2 · Kein Notverkauf von Immobilien");
    // Select goal 2: minimum consumption appears, consumption weights disappear, value is stored in the plan
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
    // Order: share of unplanned sales ascending (admissible strategies)
    const shares = await page.evaluate(() => lastOpt.report.outcomes.filter(x => x.feasible).map(x => x.validated.emergencyShare));
    expect([...shares].sort((a, b) => a - b)).toEqual(shares);
    expect(await page.evaluate(() => lastOpt.report.goal)).toBe("keepProperty");
    // (5) without real estate goal 2 cannot be selected
    await page.evaluate(() => { state.realestate.properties = []; renderSection(); });
    await expect(page.locator('#opt-goal option[value="keepProperty"]')).toBeDisabled();
    await noErrors(page, errors);
  });

  test("#29: real estate gains tax is required for owned real estate or a planned purchase, not for rent; start blocked", async ({ page }) => {
    const errors = await open(page);
    // Example plan with an owned property but without rules: error in the tax section, start blocked
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
    // With a template the requirement is met; without rules the card shows the notice «Pflicht»
    await page.reload(); await page.waitForSelector("html[data-ready='1']");
    await page.evaluate(() => { delete state.taxes.propertyGain; goto("taxes"); });
    await expect(page.locator("#main .errcard")).toContainText("Pflicht");
    await page.locator('button[data-action="gain-zh"]').click();
    expect(await page.evaluate(async () => (await planErrors()).filter(e => e.message.includes("Grundstückgewinnsteuer fehlt")).length)).toBe(0);
    await noErrors(page, errors);
  });

  // Criteria: #32/AK1 #32/AK2 #32/AK3 #32/AK4 #32/AK5 #32/AK6
  test("#32: real estate with the same field order when owned and purchase planned, market value in the header row", async ({ page }) => {
    const errors = await open(page, "realestate");
    await page.evaluate(() => {
      state.taxes.propertyGain = GAIN_ZH();
      state.realestate.properties = [{ name: "Haus", status: "owned", price: 400000, worth: 500000, bought: "2015-06", sell: "never", fixCosts: 300, mortgage: { tranches: [] } }];
      renderSection();
    });
    const prop = page.locator("#main .prop").first();
    const labels = async () => (await prop.locator(".pgrid > label.field > span:first-child").allInnerTexts()).map(t => t.replace(/\s*ⓘ$/, ""));
    const slots = () => prop.locator(".pgrid > *").evaluateAll(els => { const g = els[0].parentElement.getBoundingClientRect(); return els.map(e => Math.round(e.getBoundingClientRect().left - g.left) + "," + (e.getBoundingClientRect().top - g.top > 40 ? "unten" : "oben")); });
    // Owned: market value in the header row, grid purchase date · purchase price (paid) · sale from · additional costs
    await expect(prop.locator(".phead")).toContainText("Marktwert per Simulationsstart");
    await expect(prop.locator(".pgrid")).not.toContainText("Marktwert");
    expect(await labels()).toEqual(["Kaufdatum", "Kaufpreis (bezahlt)", "Verkauf ab", "Nebenkosten / Monat"]);
    const owned = await slots();
    const before = await page.evaluate(() => JSON.stringify(state.realestate.properties[0]));
    // Status change: same places, market value missing in the header row, entered values remain
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
    // Narrow window: market value below the name
    const rel = () => prop.locator(".phead").evaluate(h => { const w = h.querySelector(".pworth").getBoundingClientRect(), n = h.querySelector(".inp").getBoundingClientRect(); return { below: w.top >= n.bottom - 2, same: Math.abs(w.top - n.top) < 40 }; });
    expect(await rel()).toMatchObject({ same: true });
    await page.setViewportSize({ width: 420, height: 800 });
    expect(await rel()).toMatchObject({ below: true });
    await noErrors(page, errors);
  });

  test("#28: with purchase planned there is a sale-from field (default never); a sale before or in the purchase month is an error at the field", async ({ page }) => {
    const errors = await open(page, "realestate");
    const i = await page.evaluate(() => { state.realestate.properties.push({ id: "zz", name: "Testobjekt", status: "planned", buy: "2030-06", price: 400000, fixCosts: 0, mortgage: newMortgage() }); renderSection(); return state.realestate.properties.length - 1; });
    // Field with default «nie»
    const sel = page.locator(`select[data-path="realestate.properties.${i}.sell"]`);
    await expect(sel).toBeVisible();
    await expect(sel).toHaveValue("never");
    await expect(page.locator(`.prop:has(input[data-path="realestate.properties.${i}.name"]) label.field span`, { hasText: "Verkauf ab" })).toBeVisible();
    // Date selectable; sale before the purchase: error at the field, start blocked
    await page.evaluate(i => { state.realestate.properties[i].sell = "2030-03"; }, i);
    const errs = await page.evaluate(async i => (await planErrors()).filter(e => e.field === `realestate.properties.${i}.sell`), i);
    expect(errs).toHaveLength(1);
    expect(await page.evaluate(e => msgText(e), errs[0])).toContain("nach «Kauf ab»");
    expect(await page.evaluate(() => blockedByErrors())).toBe(true);
    // Sale after the purchase: no error at the field
    await page.evaluate(i => { state.realestate.properties[i].sell = "2035-03"; }, i);
    expect(await page.evaluate(async i => (await planErrors()).filter(e => e.field === `realestate.properties.${i}.sell`).length, i)).toBe(0);
    // Switching to «Besitz» keeps the entered sale
    await page.evaluate(i => ACTIONS.pstat({ i, v: "owned" }), i);
    expect(await page.evaluate(i => state.realestate.properties[i].sell, i)).toBe("2035-03");
    await noErrors(page, errors);
  });
});
