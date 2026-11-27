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
      settings = { disclaimerAccepted: 2, updateCheck: false, ...settings }; // #61, #309: disclaimer and update question count as answered unless a test says otherwise
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
      expect(set).toContainEqual({ key: "disclaimerAccepted", value: 2 });
      // not in the plan
      expect(JSON.stringify(await page.evaluate(() => state))).not.toContain("disclaimer");
    });

    test("#61/AK2: a confirmed version does not ask any more, an older version asks again", async ({ page }) => {
      const p = await plan(page);
      await openApp(page, { path: "/x/A.vplan", plan: p, notice: null }, { disclaimerAccepted: 2 });
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
      // #265: in today's purchasing power (start) the run of the real order
      const expected = (realView ? lvl.percentilesReal : lvl.percentiles).find(p => p.percent === 50).endTotal;
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
    // no full recomputation; #209: yearly values and digest of the recomputed runs come from the engine
    expect(out.calls.filter(c => c !== "yearly" && c !== "digest")).toEqual(["rerun_paths"]);
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

// ---------------------------------------------------------------------------------------------
test.describe("Compare strategies without optimization (#197)", () => {
  /** Optimization view with 300 bootstrap paths and two strategies. */
  async function openSmall(page) {
    const errors = await open(page, "optimize/bootstrap");
    await page.evaluate(() => { state.returns.bootstrap.paths = 300; });
    await setOpt(page, { strategies: ["standard", "guardrails"], paths: 200, checkPaths: 300, samples: 2 });
    return errors;
  }
  /** Clicks the button and waits for the new result. */
  async function start(page, action = "opt-compare") {
    await page.evaluate(() => { lastOpt = null; });
    await page.locator(`button[data-action="${action}"]`).click();
    await page.waitForFunction(() => lastOpt && !optBusy, null, { timeout: 120_000 });
    await expect(page.locator("html")).toHaveAttribute("data-job", "off");
  }

  test("AK1: the button computes the chosen strategies with the plan's values without search; the vary check marks stay", async ({ page }) => {
    const errors = await openSmall(page);
    const button = page.locator('button[data-action="opt-compare"]');
    await expect(button).toHaveText("Strategien vergleichen (ohne Optimierung)");
    // next to «Optimierung starten»
    await expect(page.locator('button[data-action="opt-run"] + button[data-action="opt-compare"]')).toBeVisible();
    await page.evaluate(() => { state.withdrawal.guardrails.initialRate = 0.043; });
    const ranges = await page.evaluate(() => JSON.stringify(state.optimization.ranges ?? {}));
    await start(page);
    await expect(page.locator("#main h3", { hasText: "Vergleich ohne Optimierung" })).toBeVisible();
    const r = await page.evaluate(() => lastOpt.report);
    expect(r.compare).toBe(true);
    expect(r.checkPaths).toBe(300);
    expect(r.outcomes).toHaveLength(2);
    for (const o of r.outcomes) { expect(o.params).toEqual([]); expect(o.evaluations).toBe(1); }
    const g = r.outcomes.find(o => o.patch.withdrawal.strategy === "guardrails");
    expect(g.fixed.find(f => f[0] === "initialRate")[2]).toBe(0.043);
    expect(await page.evaluate(() => JSON.stringify(state.optimization.ranges ?? {}))).toBe(ranges);
    await expect(page.locator("#optR-guardrails-initialRate-var")).toBeChecked();
    // the plan's values are listed as parameters
    await expect(page.locator("#main table.tbl small", { hasText: "4.3 %" }).first()).toBeVisible();
    await noErrors(page, errors);
  });

  test("AK2: end wealth at 1, 5, 50, 95 and 99 % in the result, also after an optimization", async ({ page }) => {
    const errors = await openSmall(page);
    const check = async () => {
      await expect(page.locator("#main th", { hasText: "Endvermögen (nominal)" })).toBeVisible();
      for (const p of ["1 %", "5 %", "50 %", "95 %", "99 %"]) await expect(page.locator("#main th", { hasText: new RegExp(`^${p}$`) })).toBeVisible();
      const r = await page.evaluate(() => lastOpt.report);
      // reference row and every strategy: 5 values, rising
      for (const m of [r.reference.metrics, ...r.outcomes.map(o => o.validated)]) {
        expect(m.endTotal).toHaveLength(5);
        expect([...m.endTotal].sort((a, b) => a - b)).toEqual(m.endTotal);
      }
      // one row per strategy plus the plan: 5 cells each with an amount after the metrics
      const row = page.locator("#main table.tbl").last().locator("tr").nth(2);
      await expect(row.locator("td")).toHaveCount(13); // #200: parameters in a sub-row, no column of their own; #205: column «Auf Untergrenze»
    };
    await start(page);
    await check();
    await setOpt(page, { strategies: ["fixed_rate"] });
    await start(page, "opt-run");
    await check();
    expect(await page.evaluate(() => lastOpt.report.compare)).toBe(false);
    await noErrors(page, errors);
  });

  test("AK3: selection as planned / never sell / both; both shows every strategy twice; hidden without property", async ({ page }) => {
    const errors = await openSmall(page);
    const sel = page.locator("#opt-property");
    await expect(sel).toHaveValue("planned");
    await expect(sel.locator("option")).toHaveText(["wie geplant", "Immobilien nie verkaufen", "beide (jede Strategie zweimal)"]);
    await sel.selectOption("both");
    expect(await page.evaluate(() => state.optimization.property)).toBe("both");
    await start(page);
    const names = await page.locator("#main table.tbl").last().locator("tr td:first-child b").allTextContents();
    for (const s of ["Standard", "Guardrails"]) {
      expect(names.filter(n => n.startsWith(s + " · wie geplant"))).toHaveLength(1);
      expect(names.filter(n => n.startsWith(s + " · Immobilien nie verkaufen"))).toHaveLength(1);
    }
    // taking over a «never sell» result switches it on in the plan
    const i = await page.evaluate(() => lastOpt.report.outcomes.findIndex(o => o.neverSell));
    expect(await page.evaluate(() => !!state.realestate.neverSell)).toBe(false);
    await page.locator(`button[data-action="opt-apply"][data-i="${i}"]`).click();
    expect(await page.evaluate(() => state.realestate.neverSell)).toBe(true);
    // «nie verkaufen»: every strategy once, with the switch
    await page.evaluate(() => { state.realestate.neverSell = false; renderSection(); });
    await sel.selectOption("neverSell");
    await start(page);
    expect(await page.evaluate(() => lastOpt.report.outcomes.map(o => o.neverSell))).toEqual([true, true]);
    // without a property of its own the selection is hidden (the engine then computes «as planned», see the Rust test)
    await page.evaluate(() => { state.realestate.properties = []; renderSection(); });
    await expect(sel).toHaveCount(0);
    await expect(page.locator('button[data-action="opt-compare"]')).toBeVisible();
    await noErrors(page, errors);
  });
});

test.describe("Result table of the optimization without horizontal scrolling (#200)", () => {
  const LANGUAGES = ["de", "en", "fr", "it"];
  /** Compares four strategies (reference plus four result rows) with a small computation; `goal` as in the plan. */
  async function compareFour(page, goal = "consumption") {
    await page.evaluate(() => { state.returns.bootstrap.paths = 300; });
    await setOpt(page, { goal, strategies: ["standard", "three_buckets", "guardrails", "fixed_rate"], paths: 200, checkPaths: 300, samples: 2 });
    await page.evaluate(() => { lastOpt = null; });
    await page.locator('button[data-action="opt-compare"]').click();
    await page.waitForFunction(() => lastOpt && !optBusy, null, { timeout: 120_000 });
    await expect(page.locator("html")).toHaveAttribute("data-job", "off");
    await expect(page.locator("#main table.opttbl tbody.optgrp")).toHaveCount(5);
  }
  /** Widths of the result: table area (client/scroll), card, page and hints. */
  const widths = page => page.evaluate(() => {
    const s = document.querySelector("#main .optres .tscroll"), card = document.querySelector("#main .optres");
    return { client: s.clientWidth, scroll: s.scrollWidth, table: s.querySelector("table").getBoundingClientRect().width,
      card: card.getBoundingClientRect().width, main: document.querySelector("#main").getBoundingClientRect().width,
      pageScroll: document.documentElement.scrollWidth, window: window.innerWidth, mainScroll: document.querySelector("#main").scrollWidth - document.querySelector("#main").clientWidth,
      hints: [...card.querySelectorAll(".hint")].map(h => h.getBoundingClientRect().width) };
  });

  test("AK1: parameters and button in a sub-row over the full width per strategy, for both goals", async ({ page }) => {
    const errors = await open(page, "optimize/bootstrap");
    for (const goal of ["consumption", "keepProperty"]) {
      await compareFour(page, goal);
      const t = page.locator("#main table.opttbl");
      const cols = await t.locator("tbody.optgrp").first().locator("tr.optmain td").count();
      // no column of its own for the parameters: the header has exactly as many columns as the metrics row
      expect(await t.evaluate(tbl => [...tbl.tHead.rows[0].cells].reduce((n, c) => n + c.colSpan, 0))).toBe(cols);
      const groups = t.locator("tbody.optgrp");
      for (let i = 0; i < 5; i++) {
        const g = groups.nth(i);
        await expect(g.locator("tr")).toHaveCount(2);
        const sub = g.locator("tr.optsub td");
        await expect(sub).toHaveAttribute("colspan", String(cols));
        if (i > 0) await expect(sub.locator('button[data-action="opt-apply"]')).toHaveAttribute("data-i", String(i - 1));
        // the sub-row belongs visibly to its strategy: no line between them, a line below the group
        const border = await g.evaluate(el => [...el.rows].map(r => getComputedStyle(r.cells[0]).borderBottomWidth));
        expect(border).toEqual(["0px", "1px"]);
      }
      await expect(t.locator('tr.optmain button[data-action="opt-apply"]')).toHaveCount(0);
      await expect(t.locator("tr.optsub", { hasText: /\d %/ })).toHaveCount(4); // the plan's values as parameters
    }
    await noErrors(page, errors);
  });

  test("AK2, AK3: result card up to 1400 px, other cards 900 px; right column hidden only in the optimization section", async ({ page }) => {
    const errors = await open(page, "optimize/bootstrap");
    await compareFour(page);
    const max = sel => page.locator(sel).evaluateAll(els => els.map(e => getComputedStyle(e).maxWidth));
    expect(await max("#main .card.optres")).toEqual(["1400px"]);
    expect(new Set(await max("#main .card:not(.optres)"))).toEqual(new Set(["900px"]));
    await expect(page.locator(".layout > aside")).toBeHidden();
    const geo = () => page.evaluate(() => ({ layout: document.querySelector(".layout").getBoundingClientRect().width,
      nav: document.querySelector("nav").getBoundingClientRect().width, main: document.querySelector("#main").getBoundingClientRect().width }));
    let g = await geo();
    expect(g.main).toBeCloseTo(g.layout - g.nav, 0);
    for (const sec of ["general", "withdrawal"]) {
      await page.evaluate(s => goto(s), sec);
      await expect(page.locator(".layout > aside")).toBeVisible();
      g = await geo();
      expect(g.main).toBeCloseTo(g.layout - g.nav - 400, 0);
      expect(new Set(await max("#main .card"))).toEqual(new Set(["900px"]));
    }
    await page.evaluate(() => goto("optimize"));
    await expect(page.locator(".layout > aside")).toBeHidden();
    await noErrors(page, errors);
  });

  test("AK4, AK6: 1440 × 900 px with five result rows: no horizontal scroll bar in any language; hints at most 900 px", async ({ browser }) => {
    test.setTimeout(180_000);
    const context = await browser.newContext({ locale: "de-CH", viewport: { width: 1440, height: 900 } });
    const page = await context.newPage();
    const errors = await open(page, "optimize/bootstrap");
    await compareFour(page);
    for (const goal of ["consumption", "keepProperty"]) {
      if (goal === "keepProperty") await compareFour(page, goal);
      for (const l of LANGUAGES) {
        await page.evaluate(l => I18n.use(l).then(() => renderSection()), l);
        await expect(page.locator("html")).toHaveAttribute("data-language", l);
        const w = await widths(page);
        expect(w.scroll, `${goal} ${l}`).toBeLessThanOrEqual(w.client);
        expect(w.mainScroll, `${goal} ${l}`).toBeLessThanOrEqual(0);
        for (const h of w.hints) expect(h).toBeLessThanOrEqual(900);
      }
      await page.evaluate(() => I18n.use("de"));
    }
    await noErrors(page, errors);
    await context.close();
  });

  test("AK5: at 2560 px the table is at most 1400 px wide", async ({ browser }) => {
    const context = await browser.newContext({ locale: "de-CH", viewport: { width: 2560, height: 1200 } });
    const page = await context.newPage();
    const errors = await open(page, "optimize/bootstrap");
    await compareFour(page);
    const w = await widths(page);
    expect(w.card).toBeLessThanOrEqual(1400);
    expect(w.table).toBeLessThanOrEqual(1400);
    expect(w.scroll).toBeLessThanOrEqual(w.client);
    await noErrors(page, errors);
    await context.close();
  });

  test("AK7: below 760 px the table scrolls inside the card, the page itself does not", async ({ browser }) => {
    const context = await browser.newContext({ locale: "de-CH", viewport: { width: 700, height: 900 } });
    const page = await context.newPage();
    const errors = await open(page, "optimize/bootstrap");
    await compareFour(page);
    const w = await widths(page);
    expect(w.scroll).toBeGreaterThan(w.client);
    expect(w.card).toBeLessThanOrEqual(w.window);
    expect(w.pageScroll).toBeLessThanOrEqual(w.window);
    expect(w.mainScroll).toBeLessThanOrEqual(0);
    await noErrors(page, errors);
    await context.close();
  });
});

test.describe("Save and reopen the result of the optimization (#201)", () => {
  /** Small optimization (`compare` false) or comparison; waits until it is stored in the list (in memory). */
  async function runOpt(page, { compare = false, goal = "consumption", strategies = ["fixed_rate", "standard"] } = {}) {
    await page.evaluate(() => { state.returns.bootstrap.paths = 300; });
    await setOpt(page, { goal, strategies, paths: 200, checkPaths: 300, samples: 2 });
    const before = await page.evaluate(() => +document.documentElement.dataset.saved || 0);
    await page.locator(`button[data-action="${compare ? "opt-compare" : "opt-run"}"]`).click();
    await page.waitForFunction(n => (+document.documentElement.dataset.saved || 0) > n && !optBusy, before, { timeout: 120_000 });
  }
  const list = page => page.evaluate(() => Runs.list());
  const openList = async page => { await page.evaluate(() => openRuns()); await expect(page.locator("html")).toHaveAttribute("data-rendered", "runs"); };
  const row = (page, kind) => page.locator("#runsTbl tr", { hasText: kind });

  test("AK1: a finished optimization creates an entry with report, settings and plan; cancel or error none", async ({ page }) => {
    const errors = await open(page, "optimize/bootstrap/mem");
    await runOpt(page);
    const [m] = await list(page);
    expect(m.kind).toBe("optimization");
    const d = await page.evaluate(id => Runs.get(id), m.id);
    expect(d.opt.report.outcomes.length).toBe(2);
    expect(d.opt.req).toMatchObject({ goal: "consumption", checkPaths: 300 });
    expect([...d.opt.req.strategies].sort()).toEqual(["fixed_rate", "standard"]);
    expect(d.plan.returns.bootstrap.paths).toBe(300);
    expect(d.opt.planJson).toBe(await page.evaluate(() => optKey()));
    // cancel and error: no entry
    for (const err of [{ code: "cancelled" }, { message: "kaputt" }]) {
      await page.evaluate(err => { const call = Engine.call.bind(Engine); Engine.call = (n, a) => n === "optimize" ? Promise.reject(Object.assign(new Error(err.message || "x"), err)) : call(n, a); }, err);
      await page.locator('button[data-action="opt-run"]').click();
      await page.waitForFunction(() => !optBusy);
    }
    expect(await list(page)).toHaveLength(1);
    expect(await page.evaluate(() => window.__errors)).toEqual([]);
    expect(errors).toEqual([]);
  });

  test("AK2: recognizable as optimization or strategy comparison with best strategy, ruin and goal; rename, note and delete", async ({ page }) => {
    const errors = await open(page, "optimize/bootstrap/mem");
    await runOpt(page);
    await runOpt(page, { compare: true, goal: "keepProperty" });
    await openList(page);
    const opt = row(page, "Ziel Konsum"), cmp = row(page, "Ziel kein Notverkauf");
    const [mc, mo] = await list(page);
    await expect(opt.locator("input.runedit[data-f=name]")).toHaveValue(/ · Optimierung · /);
    await expect(opt).toContainText("Optimierung · Ziel Konsum · 300 Pfade");
    await expect(opt).toContainText(`Ruin ${await page.evaluate(v => pctTxt(v), mo.ruin)}`);
    await expect(opt.locator("td.ptip")).toHaveText(await page.evaluate(k => stratName(k), mo.strategy));
    expect(await opt.locator("td.ptip").getAttribute("data-tip")).toMatch(/: /);
    await expect(cmp.locator("input.runedit[data-f=name]")).toHaveValue(/ · Strategievergleich · /);
    await expect(cmp).toContainText("Strategievergleich · Ziel kein Notverkauf");
    await expect(cmp).toContainText(`Ungeplanter Verkauf ${await page.evaluate(v => pctTxt(v), mc.emergencyShare)} · Ruin`);
    // rename and note as for simulations
    const name = opt.locator("input.runedit[data-f=name]");
    await name.fill("Meine Optimierung"); await name.blur();
    const note = page.locator(`input.runedit[data-f=note][data-id="${mo.id}"]`);
    await note.fill("Notiz A"); await note.blur();
    await expect.poll(async () => (await list(page)).find(m => m.id === mo.id)).toMatchObject({ name: "Meine Optimierung", note: "Notiz A" });
    // delete
    await page.locator(`button[data-action="run-del"][data-id="${mo.id}"]`).click();
    await page.locator('#ask button[value="ok"]').click();
    await expect.poll(async () => (await list(page)).map(m => m.id)).toEqual([mc.id]);
    await noErrors(page, errors);
  });

  test("AK3: open shows the table of the optimization section in the wide results view with date and number of paths", async ({ page }) => {
    const errors = await open(page, "optimize/bootstrap/mem");
    await runOpt(page, { strategies: ["fixed_rate", "standard", "vpw"] });
    const [m] = await list(page);
    await openList(page);
    await page.locator(`button[data-action="run-view"][data-id="${m.id}"]`).click();
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "savedOpt");
    await expect(page.locator("#results")).toBeVisible();
    await expect(page.locator("#resultsBody .card.optres table.opttbl tbody.optgrp")).toHaveCount(4);
    await expect(page.locator('#resultsBody tr.optsub button[data-action="opt-apply"]')).toHaveCount(3);
    const date = await page.evaluate(iso => runDate(iso), m.created);
    await expect(page.locator("#resultsPick .savedbar")).toContainText(`Gespeichertes Ergebnis vom ${date}`);
    await expect(page.locator("#resultsPick .savedbar")).toContainText("geprüft mit 300 Pfaden");
    await expect(page.locator("#resultsPick .savedother")).toHaveCount(0);
    await expect(page.locator("#resultsBody .optres h3 .sub")).toContainText("300");
    // same card as in the section (#200): identical table
    const saved = await page.locator("#resultsBody table.opttbl").innerHTML();
    await page.locator('#resultsPick button[data-action="runs-open"]').click();
    await page.keyboard.press("Escape");
    await page.evaluate(() => goto("optimize"));
    expect(await page.locator("#main table.opttbl").innerHTML()).toBe(saved);
    await noErrors(page, errors);
  });

  test("AK5: apply to plan takes the parameters over; with another state of the plan only after a question", async ({ page }) => {
    const errors = await open(page, "optimize/bootstrap/mem");
    await runOpt(page, { strategies: ["fixed_rate"] });
    const [m] = await list(page);
    const rate = await page.evaluate(id => Runs.get(id).then(d => d.opt.report.outcomes[0].patch.withdrawal.fixedRate.rate), m.id);
    const view = async () => { await openList(page); await page.locator(`button[data-action="run-view"][data-id="${m.id}"]`).click(); await expect(page.locator("html")).toHaveAttribute("data-rendered", "savedOpt"); };
    // matching plan: no question
    await page.evaluate(() => { state.withdrawal.strategy = "standard"; state.withdrawal.fixedRate.rate = 0.0123; lastOpt = null; });
    await page.evaluate(id => Runs.get(id).then(d => { state = normalize(JSON.parse(JSON.stringify(d.plan))); }), m.id);
    await view();
    await page.locator('#resultsBody button[data-action="opt-apply"][data-i="0"]').click();
    await expect(page.locator("#results")).toBeHidden();
    await expect(page.locator("#ask")).not.toHaveAttribute("open", "");
    expect(await page.evaluate(() => [state.withdrawal.strategy, state.withdrawal.fixedRate.rate])).toEqual(["fixed_rate", rate]);
    // the section shows the result again (matching the plan now)
    await expect(page.locator("#main table.opttbl")).toBeVisible();
    // other state: question; cancel leaves the plan, OK takes over
    await page.evaluate(() => { state.withdrawal.strategy = "standard"; state.household.persons[0].name = "Andere"; });
    await view();
    await expect(page.locator("#resultsPick .savedother")).toContainText("anderen Planungsstand");
    await page.locator('#resultsBody button[data-action="opt-apply"][data-i="0"]').click();
    await expect(page.locator("#askText")).toContainText("anderen Planungsstand");
    await page.locator('#ask button[value="cancel"]').click();
    expect(await page.evaluate(() => state.withdrawal.strategy)).toBe("standard");
    await page.locator('#resultsBody button[data-action="opt-apply"][data-i="0"]').click();
    await page.locator('#ask button[value="ok"]').click();
    await expect(page.locator("#results")).toBeHidden();
    expect(await page.evaluate(() => [state.withdrawal.strategy, state.withdrawal.fixedRate.rate, state.household.persons[0].name])).toEqual(["fixed_rate", rate, "Andere"]);
    await noErrors(page, errors);
  });

  test("AK6: optimizations can be marked for deleting but are left out of the comparison", async ({ page }) => {
    const errors = await open(page, "optimize/bootstrap/mem");
    await runOpt(page, { strategies: ["fixed_rate"] });
    await page.evaluate(async () => {
      const plan = JSON.parse(JSON.stringify(state)), sim = await Engine.call("run_simulation", { plan: { ...plan, returns: { ...plan.returns, source: "fixed" }, scenarios: { ...plan.scenarios, sweeps: [] } }, path: null });
      await rememberRun(sim); await rememberRun(sim);
    });
    await openList(page);
    await page.locator('input[data-action="runs-all"]').check();
    await expect(page.locator('#runsTbl input[data-action="run-sel"]:checked')).toHaveCount(3);
    await expect(page.locator('button[data-action="runs-compare"]')).toHaveText("Vergleichen (2)");
    await expect(page.locator('button[data-action="runs-compare"]')).toBeEnabled();
    await expect(page.locator('button[data-action="runs-delete"]')).toHaveText("Markierte löschen (3)");
    expect(await row(page, "Optimierung").locator('input[data-action="run-sel"]').getAttribute("title")).toContain("nicht mit Simulationen vergleichen");
    // only the optimization and one simulation marked: comparison not possible
    await page.locator('input[data-action="runs-all"]').uncheck();
    await row(page, "Optimierung").locator('input[data-action="run-sel"]').check();
    await page.locator('#runsTbl input[data-action="run-sel"]:not(:checked)').first().check();
    await expect(page.locator('button[data-action="runs-compare"]')).toHaveText("Vergleichen (1)");
    await expect(page.locator('button[data-action="runs-compare"]')).toBeDisabled();
    // all marked: the comparison shows the two simulations
    await page.locator('input[data-action="runs-all"]').check();
    await page.locator('button[data-action="runs-compare"]').click();
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "compare");
    await expect(page.locator("#resultsTitle")).toContainText("2");
    await noErrors(page, errors);
  });

  test("AK7: opening a plan matching a saved optimization shows the newest of them in the section", async ({ page }) => {
    const errors = await open(page, "optimize/bootstrap/mem");
    await runOpt(page, { strategies: ["fixed_rate"] });
    await runOpt(page, { compare: true, strategies: ["fixed_rate", "standard"] });
    const plan = await page.evaluate(() => JSON.parse(JSON.stringify(state)));
    const restored = () => page.evaluate(() => +document.documentElement.dataset.optRestored || 0);
    // another plan: the section shows no saved result
    let n = await restored();
    await page.evaluate(async () => { lastOpt = null; await adopt(await examplePlan()); });
    await page.waitForFunction(n => +document.documentElement.dataset.optRestored > n, n);
    await page.evaluate(() => goto("optimize"));
    await expect(page.locator("#main table.opttbl")).toHaveCount(0);
    // the plan of that time (through the engine as when opening a file, keys in another order): newest = comparison
    n = await restored();
    await page.evaluate(async p => { const r = {}; for (const k of Object.keys(p).reverse()) r[k] = p[k]; await adopt(r); goto("optimize"); }, plan);
    await page.waitForFunction(n => +document.documentElement.dataset.optRestored > n, n);
    await expect(page.locator("#main .optres h3")).toContainText("Vergleich ohne Optimierung");
    await expect(page.locator("#main table.opttbl tbody.optgrp")).toHaveCount(3);
    await expect(page.locator("#main .optres")).not.toContainText("seit dieser Optimierung geändert");
    await noErrors(page, errors);
  });

  test("AK8: without IndexedDB the result stays in memory, without a message", async ({ page }) => {
    // «mem»: opening the database fails as with blocked storage (runs.js throws before indexedDB.open), the list falls back to memory
    const errors = await open(page, "optimize/bootstrap/mem");
    await runOpt(page, { strategies: ["fixed_rate"] });
    expect((await list(page)).map(m => m.kind)).toEqual(["optimization"]);
    await expect(page.locator("#toast")).not.toContainText("nicht");
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Limits of living expenses with VPW and endowment in comparison and optimization (#204)", () => {
  const FIVE = ["fixed_rate", "guardrails", "vpw", "endowment", "three_buckets_guardrails"];
  /** Runs the comparison (or optimization) of `strategies` with the given limits and returns the report. */
  async function run(page, strategies, floor, ceiling, action = "opt-compare") {
    await page.evaluate(([f, c]) => { state.returns.bootstrap.paths = 300; state.withdrawal.floor = f; state.withdrawal.ceiling = c; }, [floor, ceiling]);
    await setOpt(page, { strategies, paths: 100, checkPaths: 300, samples: 2 });
    await page.evaluate(() => { lastOpt = null; });
    await page.locator(`button[data-action="${action}"]`).click();
    await page.waitForFunction(() => lastOpt && !optBusy, null, { timeout: 120_000 });
    return page.evaluate(() => lastOpt.report);
  }
  /** Parameter text of the result row of `strategy`. */
  const row = (page, r, strategy) => {
    const i = r.outcomes.findIndex(o => o.patch.withdrawal.strategy === strategy.replace("_guardrails", "") && (strategy === "three_buckets_guardrails") === (o.patch.withdrawal.strategy === "three_buckets"));
    expect(i, strategy).toBeGreaterThanOrEqual(0);
    return page.locator(`#main .optsubin:has(button[data-i="${i}"])`);
  };

  test("AK1: comparison and optimization list floor and ceiling with the plan value for VPW and endowment", async ({ page }) => {
    const errors = await open(page, "optimize/bootstrap");
    // the limits appear among the parameters of both strategies, fixed by default
    for (const s of ["vpw", "endowment"]) for (const k of ["floor", "ceiling"]) await expect(page.locator(`#optR-${s}-${k}-var`)).not.toBeChecked();
    let r = await run(page, ["vpw", "endowment"], 0.8, 1.25);
    for (const s of ["vpw", "endowment"]) {
      await expect(row(page, r, s)).toContainText("Untergrenze der Lebenshaltung 80 %");
      await expect(row(page, r, s)).toContainText("Obergrenze der Lebenshaltung 125 %");
    }
    r = await run(page, ["vpw", "endowment"], 0.8, 1.25, "opt-run");
    for (const s of ["vpw", "endowment"]) await expect(row(page, r, s).locator("small.computed")).toContainText("fix: Untergrenze der Lebenshaltung 80 %, Obergrenze der Lebenshaltung 125 %");
    await noErrors(page, errors);
  });

  test("AK2: a limit of 0 (no limit) is shown alike for all five strategies", async ({ page }) => {
    const errors = await open(page, "optimize/bootstrap");
    const r = await run(page, FIVE, 0, 0);
    expect(r.outcomes).toHaveLength(5);
    for (const s of FIVE) {
      await expect(row(page, r, s)).toContainText("Untergrenze der Lebenshaltung 0 %");
      await expect(row(page, r, s)).toContainText("Obergrenze der Lebenshaltung 0 %");
    }
    await noErrors(page, errors);
  });

  test("AK3: adopting a VPW or endowment row leaves the limits of the plan unchanged", async ({ page }) => {
    const errors = await open(page, "optimize/bootstrap");
    for (const s of ["vpw", "endowment"]) {
      const r = await run(page, [s], 0.8, 1.25);
      await page.evaluate(() => { state.withdrawal.strategy = "standard"; });
      await row(page, r, s).locator('button[data-action="opt-apply"]').click();
      expect(await page.evaluate(() => [state.withdrawal.strategy, state.withdrawal.floor, state.withdrawal.ceiling])).toEqual([s, 0.8, 1.25]);
    }
    await noErrors(page, errors);
  });
});

test.describe("Hint in the result table when the floor sets the spending (#205)", () => {
  const HINT = "Die Untergrenze bestimmt meist die Ausgaben; die Strategie wirkt kaum.";
  /** Runs the comparison (or optimization) of `strategies` with floor `floor`; returns the report. */
  async function run(page, strategies, floor, action = "opt-compare", goal = "consumption") {
    await page.evaluate(f => { state.returns.bootstrap.paths = 300; state.withdrawal.floor = f; state.withdrawal.ceiling = 0; }, floor);
    await setOpt(page, { goal, strategies, paths: 100, checkPaths: 300, samples: 2 });
    await page.evaluate(() => { lastOpt = null; });
    await page.locator(`button[data-action="${action}"]`).click();
    await page.waitForFunction(() => lastOpt && !optBusy, null, { timeout: 120_000 });
    return page.evaluate(() => lastOpt.report);
  }
  /** Texts of the column «Auf Untergrenze» and whether the hint stands in the sub-row, per group (reference first). */
  const shown = (page, root = "#main") => page.locator(`${root} table.opttbl tbody.optgrp`).evaluateAll(gs => gs.map(g => ({
    cell: g.querySelector("tr.optmain td.optfloor")?.textContent, hint: g.querySelector("tr.optsub .optfloorhint")?.textContent ?? null })));
  const expected = (page, r) => page.evaluate(r => [r.reference.metrics, ...r.outcomes.map(o => o.validated)].map(m =>
    ({ cell: m.floorShare == null ? "–" : pctTxt(m.floorShare), hint: m.floorShare >= 0.5 ? "Die Untergrenze bestimmt meist die Ausgaben; die Strategie wirkt kaum." : null })), r);

  test("AK1: column with the share of months at the floor per strategy, in comparison and optimization, for both goals", async ({ page }) => {
    const errors = await open(page, "optimize/bootstrap");
    for (const [action, goal] of [["opt-compare", "consumption"], ["opt-run", "consumption"], ["opt-compare", "keepProperty"]]) {
      const r = await run(page, ["fixed_rate", "guardrails"], 0.8, action, goal);
      await expect(page.locator("#main table.opttbl th", { hasText: /^Auf Untergrenze$/ })).toBeVisible();
      expect(await page.locator("#main table.opttbl th", { hasText: /^Auf Untergrenze$/ }).getAttribute("title")).toContain("ab Pensionierung");
      for (const o of r.outcomes) expect(o.validated.floorShare).toBeGreaterThanOrEqual(0);
      expect(await shown(page)).toEqual(await expected(page, r));
    }
    await noErrors(page, errors);
  });

  test("AK2: hint from 50 % of the months in the sub-row of the strategy", async ({ page }) => {
    const errors = await open(page, "optimize/bootstrap");
    await run(page, ["fixed_rate", "guardrails"], 0.8);
    for (const [v, hint] of [[0.5, true], [0.499, false], [1, true], [0, false]]) {
      await page.evaluate(v => { lastOpt.report.outcomes[0].validated.floorShare = v; renderSection(); }, v);
      const g = page.locator("#main table.opttbl tbody.optgrp").nth(1);
      await expect(g.locator("td.optfloor")).toHaveText(await page.evaluate(v => pctTxt(v), v));
      await expect(g.locator("tr.optsub .optfloorhint")).toHaveCount(hint ? 1 : 0);
      if (hint) await expect(g.locator("tr.optsub")).toContainText(HINT);
    }
    await noErrors(page, errors);
  });

  test("AK3: Standard, 3 buckets without guardrails and floor 0 show «–» and no hint; 3 buckets with guardrails has a share", async ({ page }) => {
    const errors = await open(page, "optimize/bootstrap");
    let r = await run(page, ["standard", "three_buckets", "three_buckets_guardrails", "fixed_rate"], 0.8);
    const byStrategy = r.outcomes.map(o => [o.patch.withdrawal.strategy, o.patch.withdrawal.buckets?.spending ?? null, o.validated.floorShare]);
    for (const [s, sp, f] of byStrategy) {
      if (s === "standard" || (s === "three_buckets" && sp !== "guardrails")) expect(f, s).toBeUndefined();
      else expect(f, `${s} ${sp}`).toBeGreaterThanOrEqual(0);
    }
    expect(await shown(page)).toEqual(await expected(page, r));
    r = await run(page, ["fixed_rate", "guardrails", "vpw"], 0);
    const rows = await shown(page);
    expect(rows.slice(1)).toEqual(r.outcomes.map(() => ({ cell: "–", hint: null })));
    await noErrors(page, errors);
  });

  test("AK6: a saved optimization shows the share again after opening; one saved before #205 shows «–» without hint", async ({ page }) => {
    const errors = await open(page, "optimize/bootstrap/mem");
    await page.evaluate(() => { state.returns.bootstrap.paths = 300; state.withdrawal.floor = 0.8; });
    await setOpt(page, { strategies: ["fixed_rate", "standard"], paths: 100, checkPaths: 300, samples: 2 });
    const before = await page.evaluate(() => +document.documentElement.dataset.saved || 0);
    await page.locator('button[data-action="opt-run"]').click();
    await page.waitForFunction(n => (+document.documentElement.dataset.saved || 0) > n && !optBusy, before, { timeout: 120_000 });
    const inSection = await shown(page);
    expect(inSection.some(x => x.cell !== "–")).toBe(true);
    const [m] = await page.evaluate(() => Runs.list());
    await page.evaluate(() => openRuns());
    await page.locator(`button[data-action="run-view"][data-id="${m.id}"]`).click();
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "savedOpt");
    expect(await shown(page, "#resultsBody")).toEqual(inSection);
    // saved before #205: no share in the report
    await page.evaluate(async id => {
      const [meta] = (await Runs.list()).filter(x => x.id === id), d = await Runs.get(id);
      const old = JSON.parse(JSON.stringify(d));
      for (const x of [old.opt.report.reference.metrics, ...old.opt.report.outcomes.flatMap(o => [o.fitted, o.validated])]) delete x.floorShare;
      viewOpt(meta, old);
    }, m.id);
    expect(await shown(page, "#resultsBody")).toEqual(inSection.map(() => ({ cell: "–", hint: null })));
    await noErrors(page, errors);
  });
});
