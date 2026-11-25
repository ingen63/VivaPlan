// #170: Multilingual user interface – text tables, choice of language, menus. The tests of #170 and #171 replace the
// table «en» by a pseudo language (every German text with the prefix «Ⓔ») by intercepting the requests for the
// tables, so that switching, fallback and formats are checked independently of the translation. #172 checks the real
// English table.
const { test, expect } = require("@playwright/test");
const { open, noErrors } = require("./helpers");

const MARK = "Ⓔ";
/** Pseudo table: the same keys as the German table, every text marked. */
async function pseudoLanguage(page, { languages = { de: "Deutsch", en: "English" } } = {}) {
  await page.route("**/i18n/languages.json", r => r.fulfill({ json: languages }));
  await page.route("**/i18n/en.json", async r => {
    const de = await (await r.fetch({ url: r.request().url().replace("en.json", "de.json") })).json();
    r.fulfill({ json: Object.fromEntries(Object.entries(de).map(([k, v]) => [k, MARK + v])) });
  });
}

/** Desktop app stand-in: settings in memory, every call recorded, several listeners per event. */
async function openApp(page, { settings = {}, firstStart = null, startup = null } = {}) {
  await page.addInitScript(([settings, firstStart, startup]) => {
    window.__calls = []; window.__settings = settings; window.__listeners = {};
    window.__TAURI__ = {
      core: {
        invoke: async (cmd, args = {}) => {
          window.__calls.push([cmd, JSON.parse(JSON.stringify(args))]);
          if (cmd === "settings_get") return { disclaimerAccepted: 2, ...window.__settings };
          if (cmd === "settings_set") { window.__settings[args.key] = args.value; return null; }
          if (cmd === "first_start_language") return firstStart;
          if (cmd === "startup_plan") return startup ?? { path: "/x/Familie.vplan", plan: await (await fetch("/api/example_plan", { method: "POST", body: "{}" })).json(), notice: null, first: false };
          if (cmd === "save_plan") return args.path ?? "/x/neu.vplan";
          if (["set_dirty", "quit_app", "set_window_title", "set_menu_language"].includes(cmd)) return null;
          const r = await fetch(`/api/${cmd}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(args) });
          const j = await r.json(); if (!r.ok) throw new Error(j.error); return j;
        }
      },
      event: { listen: (name, cb) => { (window.__listeners[name] ??= []).push(cb); return Promise.resolve(() => {}); } }
    };
    window.__menu = id => { for (const cb of window.__listeners.menu || []) cb({ payload: id }); };
  }, [settings, firstStart, startup]);
  const errors = [];
  page.on("pageerror", e => errors.push(e.message));
  await page.goto("/#nobench");
  await page.waitForSelector("html[data-ready='1']");
  return errors;
}
const calls = (page, cmd) => page.evaluate(cmd => window.__calls.filter(c => c[0] === cmd).map(c => c[1]), cmd);
const lang = page => page.locator("html").getAttribute("data-language");

test.describe("Multilingual user interface (#170)", () => {
  test("AK7: with another system language the user interface falls back to German", async ({ browser }) => {
    // #172, #174: English and French have their own tables since then, so only languages without a table fall back
    for (const locale of ["es-ES", "ja-JP", "pt-BR"]) {
      const ctx = await browser.newContext({ locale });
      const page = await ctx.newPage();
      const errors = [];
      page.on("pageerror", e => errors.push(e.message));
      await page.goto("/#nobench");
      await page.waitForSelector("html[data-ready='1']");
      expect(await page.evaluate(() => navigator.language)).toBe(locale);
      await expect(page.locator("html")).toHaveAttribute("lang", "de-CH");
      await expect(page.locator("#nav a").first()).toContainText("Haushalt & Projekt");
      await expect(page.locator("header")).toContainText("Simulation starten");
      expect(errors).toEqual([]);
      await ctx.close();
    }
  });

  test("AK2: system language without region; unknown language → German", async ({ browser }) => {
    for (const [locale, expected] of [["en-GB", "en"], ["en_US", "en"], ["fr-CH", "de"], ["it", "de"], ["pt-BR", "de"]]) {
      const ctx = await browser.newContext({ locale: "de-CH" });
      const page = await ctx.newPage();
      await pseudoLanguage(page);
      // very first start of the app: the app (sys-locale) reports the language of the operating system
      const errors = await openApp(page, { firstStart: locale });
      expect(await lang(page)).toBe(expected);
      await expect(page.locator("#nav a").first()).toContainText(expected === "en" ? `${MARK}Haushalt & Projekt` : "Haushalt & Projekt");
      expect(await calls(page, "set_menu_language")).toContainEqual({ language: expected, choice: "system" });
      expect(errors).toEqual([]);
      await ctx.close();
    }
    // later starts with «wie System»: the languages of the web view
    const ctx = await browser.newContext({ locale: "en-AU" });
    const page = await ctx.newPage();
    await pseudoLanguage(page);
    await openApp(page, { settings: { language: "system" } });
    expect(await lang(page)).toBe("en");
    await ctx.close();
  });

  test("AK3, AK9: language from the settings and from the menu View → Language, at once and the same choice", async ({ page }) => {
    await pseudoLanguage(page);
    const errors = await openApp(page);
    expect(await lang(page)).toBe("de");
    // View, plan and unsaved changes before the change
    await page.locator("#nav a", { hasText: "Vermögen & Anlage" }).click();
    await page.locator('input[data-path="wealth.cash"]').fill("123456");
    await page.locator('input[data-path="wealth.cash"]').press("Tab");
    await expect(page.locator("#dirty")).toHaveText("● ungespeichert");
    const before = await page.evaluate(() => JSON.stringify(state));

    // Menu «Ansicht → Sprache → English»
    await page.evaluate(() => window.__menu("language-en"));
    await expect(page.locator("html")).toHaveAttribute("data-language", "en");
    await expect(page.locator("#nav a.active")).toContainText(`${MARK}Vermögen & Anlage`);
    await expect(page.locator("#dirty")).toHaveText(`${MARK}● ungespeichert`);
    await expect(page.locator('input[data-path="wealth.cash"]')).toHaveValue("123'456");
    expect(await page.evaluate(() => JSON.stringify(state))).toBe(before);
    expect(await page.evaluate(() => dirty)).toBe(true);
    expect(await page.evaluate(() => window.__settings.language)).toBe("en");
    expect((await calls(page, "set_menu_language")).at(-1)).toEqual({ language: "en", choice: "en" });
    // The settings show the same choice
    await page.evaluate(() => command("settings"));
    await expect(page.locator("#set-language")).toHaveValue("en");
    await expect(page.locator("#set-language option")).toHaveText([`${MARK}wie System`, "Deutsch", "English"]);
    // Settings → Deutsch: at once, also for the menus
    await page.locator("#set-language").selectOption("de");
    await page.locator("#settingsSave").click();
    await expect(page.locator("html")).toHaveAttribute("data-language", "de");
    await expect(page.locator("#nav a.active")).toHaveText(/Vermögen & Anlage/);
    expect(await page.evaluate(() => window.__settings.language)).toBe("de");
    expect((await calls(page, "set_menu_language")).at(-1)).toEqual({ language: "de", choice: "de" });
    expect(await page.evaluate(() => JSON.stringify(state))).toBe(before);
    // «wie System» from the menu
    await page.evaluate(() => window.__menu("language-system"));
    await expect.poll(() => page.evaluate(() => I18n.choice())).toBe("system");
    expect(await page.evaluate(() => window.__settings.language)).toBe("system");
    await noErrors(page, errors);
  });

  test("AK9: results stay open with tab and time range when switching the language", async ({ page }) => {
    await pseudoLanguage(page);
    const errors = await openApp(page);
    await page.evaluate(() => { state.returns.source = "fixed"; state.scenarios.sweeps = []; });
    await page.evaluate(() => runSimulation("charts", { noConfirm: true }));
    await expect(page.locator("#resultsBody svg").first()).toBeAttached();
    await page.evaluate(() => setCustomZoom(12, 120));
    const tab = await page.evaluate(() => resultTab), zoom = await page.evaluate(() => JSON.stringify(resultZoom));
    await page.evaluate(() => window.__menu("language-en"));
    await expect(page.locator("#resultsTabs")).toContainText(MARK);
    expect(await page.evaluate(() => [resultTab, JSON.stringify(resultZoom)])).toEqual([tab, zoom]);
    await expect(page.locator("#results")).toBeVisible();
    await noErrors(page, errors);
  });

  test("AK1: in another language the texts of the editor come from its table", async ({ page }) => {
    await pseudoLanguage(page);
    const errors = await openApp(page, { settings: { language: "en" } });
    expect(await lang(page)).toBe("en");
    const sections = await page.evaluate(() => SECTIONS.map(s => s.id));
    const missing = [];
    for (const id of sections) {
      await page.evaluate(id => goto(id), id);
      // labels, headings, column titles, buttons, hints: each must come from the (pseudo) table;
      // #259/AK1: labels from the plan's data (name of a property, currency) are not texts of the interface
      missing.push(...await page.evaluate(mark => [...document.querySelectorAll(
        "#main h2, #main h3, #main th:not([data-no-tr]), #main label.field > span:first-child:not([data-no-tr]), #main .hint, #main p.lead, #main button, header button, #nav a, aside h3")]
        .map(el => el.textContent.trim()).filter(t => /[A-Za-zÄÖÜäöü]{3}/.test(t) && !t.includes(mark))
        .map(t => `${document.querySelector("#nav a.active")?.textContent}: ${t}`), MARK));
    }
    expect(missing).toEqual([]);
    await noErrors(page, errors);
  });

  test("AK5: numbers, amounts, percentages and dates in the Swiss format in every language", async ({ page }) => {
    await pseudoLanguage(page);
    const errors = await openApp(page);
    for (const l of ["de", "en"]) {
      await page.evaluate(l => I18n.use(l), l);
      expect(await lang(page)).toBe(l);
      const out = await page.evaluate(() => ({
        money: fmtMoney(1234567.5), int: fmtInt(1234567), pct: `${fmtPct(0.025)} %`, month: ymShow("2040-03"),
        parsed: [parseAmount("1'234'567.50"), parseAmount("1’234.5"), parseNum("2.5")],
      }));
      expect(out.money).toMatch(/^1['’]234['’]56[78]/);
      expect(out.int).toMatch(/^1['’]234['’]567$/);
      expect(out.pct).toBe("2.5 %");
      expect(out.month).toBe("03.2040");
      expect(out.parsed).toEqual([1234567.5, 1234.5, 2.5]);
      // display: an amount in an input field and the time line
      await page.evaluate(() => goto("wealth"));
      await expect(page.locator('input[data-path="wealth.stocks"]')).toHaveValue(/^\d{1,3}(['’]\d{3})*$/);
      await expect(page.locator("#tlRange")).toHaveText(/\d\d\.\d{4} – \d\d\.\d{4} · \d+\.\d /);
    }
    await noErrors(page, errors);
  });

  test("AK6, AK8: a plan is saved unchanged in every language; a new plan gives the same file in every language", async ({ page }) => {
    await pseudoLanguage(page);
    const errors = await openApp(page);
    const saved = [], fresh = [];
    for (const l of ["de", "en", "de"]) {
      await page.evaluate(l => window.__menu(`language-${l}`), l);
      await expect(page.locator("html")).toHaveAttribute("data-language", l);
      await page.evaluate(() => ACTIONS.save());
      await expect.poll(async () => (await calls(page, "save_plan")).length).toBe(saved.length + 1);
      saved.push(JSON.stringify((await calls(page, "save_plan")).at(-1).plan));
    }
    expect(new Set(saved).size).toBe(1);
    // the example plan has no texts of the user interface: no names for the fixed milestones
    const plan = JSON.parse(saved[0]);
    for (const m of plan.milestones.filter(m => ["start", "end", "retire", "ahv"].includes(m.id))) expect(m.name).toBeUndefined();
    // New plan (File → New) in every language: identical file
    for (const l of ["de", "en"]) {
      await page.evaluate(l => I18n.use(l), l);
      const p = await page.evaluate(async () => blankPlan(new Date(2030, 0, 1)));
      fresh.push(JSON.stringify(p));
    }
    expect(fresh[0]).toBe(fresh[1]);
    expect(JSON.parse(fresh[0]).milestones.map(m => m.name)).toEqual([undefined, undefined, undefined, undefined]);
    expect(JSON.parse(fresh[0]).project.name).toBe("");
    await noErrors(page, errors);
  });

  test("AK8: fixed milestones show their name from the table; an older file loses the German default names", async ({ page }) => {
    await pseudoLanguage(page);
    const old = await (await page.request.post("/api/example_plan", { data: {} })).json();
    const names = { start: "Simulationsstart", retire: "Bezug Pensionskasse", ahv: "AHV-Bezug", end: "Simulationsende" };
    for (const m of old.milestones) if (names[m.id]) m.name = names[m.id];
    const errors = await openApp(page, { startup: { path: "/x/alt.vplan", plan: old, notice: null, first: false } });
    expect(await page.evaluate(() => state.milestones.filter(m => m.name !== undefined).map(m => m.id))).toEqual(["sell_house"]);
    await page.evaluate(() => goto("milestones"));
    await expect(page.locator("#main table.tbl")).toContainText("Bezug Pensionskasse");
    await page.evaluate(() => window.__menu("language-en"));
    await expect(page.locator("#main table.tbl")).toContainText(`${MARK}Bezug Pensionskasse`);
    await expect(page.locator("#main table.tbl")).toContainText(`${MARK}Simulationsstart`);
    await noErrors(page, errors);
  });
});

test.describe("Messages of the engine (#171)", () => {
  test("AK2: messages of the engine come from the table of the current language, numbers in the Swiss format", async ({ page }) => {
    await pseudoLanguage(page);
    const errors = await openApp(page, { settings: { language: "en" } });
    expect(await lang(page)).toBe("en");
    const out = await page.evaluate(async () => {
      const plan = JSON.parse(JSON.stringify(state));
      plan.withdrawal.strategy = "fixed_rate"; plan.withdrawal.fixedRate = { rate: 0.09 };
      plan.pillar3a = [{ person: plan.household.persons[0].id, balance: 1000, rate: 0, contribution: 99999, payouts: [] }];
      const issues = await Engine.call("validate", { plan });
      const run = await Engine.call("run_simulation", { plan: state });
      const de = await (await fetch("i18n/de.json")).json();
      return {
        issues: issues.map(i => [i.code, msgText(i), i.message]),
        events: run.single.events.map(e => [msgText(e), engineText(taxRounded(e), k => de[k]), e.text]), // #143/AK2: tax rounded to 100
        label: msgText(run.single.label),
        unknown: msgText({ code: "no_such_code", params: {}, text: "Text des Rechenkerns" }),
      };
    });
    const rate = out.issues.find(i => i[0] === "withdrawal_rate_high");
    expect(rate[1]).toBe("ⒺEine Entnahmerate von 9.0 % ist hoch; das Vermögen schrumpft bei längerer Dauer rasch.");
    // #176/AK1: the engine itself sends the English text; the user interface does not show it
    expect(rate[2]).toBe("A withdrawal rate of 9.0 % is high; wealth shrinks quickly over a longer period.");
    // nested message (account) and amounts with thousands separator '
    const p3a = out.issues.find(i => i[0] === "pillar3a_contribution_range");
    expect(p3a[1]).toMatch(/^ⒺⒺSäule 3a: Die Einzahlung muss zwischen 0 und dem Maximalbetrag von \d{1,3}('\d{3})* pro Jahr/);
    expect(out.events.length).toBeGreaterThan(0);
    for (const [shown, german] of out.events) expect(shown.startsWith(MARK) && shown.replaceAll(MARK, "") === german, `${shown} / ${german}`).toBe(true);
    expect(out.label).toBe(`${MARK}Fixe Renditen`);
    expect(out.unknown).toBe("Text des Rechenkerns");
    // the list of the validation shows the translated text
    await page.evaluate(() => { state.withdrawal.strategy = "fixed_rate"; state.withdrawal.fixedRate = { rate: 0.09 }; refresh(); });
    await expect(page.locator("#val")).toContainText(`${MARK}Eine Entnahmerate von 9.0 % ist hoch`);
    await noErrors(page, errors);
  });

  test("#176/AK4: the German user interface shows no English text of the engine (results, events, checks)", async ({ page }) => {
    const errors = await open(page);
    await page.evaluate(() => { state.withdrawal.strategy = "fixed_rate"; state.withdrawal.fixedRate = { rate: 0.09 }; refresh(); });
    await expect(page.locator("#val")).toContainText("Eine Entnahmerate von 9.0 % ist hoch");
    const out = await page.evaluate(async () => {
      lastRun = await Engine.call("run_simulation", { plan: state, path: null });
      resultTab = "events"; showResults(true); renderResults(lastRun);
      const issues = await Engine.call("validate", { plan: state });
      const de = await (await fetch("i18n/de.json")).json();
      // English texts sent along by the engine that differ from the German rendering
      const sent = [...lastRun.single.events, ...issues, lastRun.single.label]
        .filter(m => m && m.code && m.text !== undefined ? true : m && m.message !== undefined)
        .map(m => [m.text ?? m.message, engineText(m, k => de[k])])
        .filter(([en, ger]) => en !== ger && en.length >= 12).map(([en]) => en);
      return { sent, shown: document.querySelector("#resultsBody").textContent + "\n" + document.querySelector("#val").textContent,
        events: document.querySelectorAll("#resultsBody .evlist li").length };
    });
    expect(out.sent.length).toBeGreaterThan(5);
    expect(out.events).toBeGreaterThan(0);
    expect(out.shown).toContain("Eine Entnahmerate von 9.0 % ist hoch");
    for (const en of out.sent) expect(out.shown.includes(en), en).toBe(false);
    await noErrors(page, errors);
  });

  test("AK2: the cancellation of a computation is recognized by its code, not by the German text", async ({ page }) => {
    const errors = await open(page, "general");
    const out = await page.evaluate(() => [isCancelled({ code: "cancelled", params: {}, text: "x" }), isCancelled(Object.assign(new Error("x"), { code: "cancelled" })), isCancelled("Abgebrochen")]);
    expect(out).toEqual([true, true, false]);
    await noErrors(page, errors);
  });
});

test.describe("English user interface (#172)", () => {
  test("AK1, AK3: system language en: editor, results, settings, menus and messages in English, numbers in the Swiss format", async ({ browser }) => {
    const ctx = await browser.newContext({ locale: "en-US" });
    const page = await ctx.newPage();
    const errors = await openApp(page, { settings: { language: "system" } });
    expect(await lang(page)).toBe("en");
    await expect(page.locator("html")).toHaveAttribute("lang", "en-CH");
    // Menus: the app rebuilds the native menus in English (texts: Rust test issue_172_ak3_menus_and_dialogs_in_english)
    expect(await calls(page, "set_menu_language")).toContainEqual({ language: "en", choice: "system" });

    // Editor
    await expect(page.locator("#nav a").first()).toContainText("Household & project");
    await expect(page.locator("#nav")).toContainText("Wealth & investment");
    await expect(page.locator("header")).toContainText("Start simulation");
    await page.evaluate(() => goto("wealth"));
    await expect(page.locator("#main")).toContainText("Stock allocation (share of stocks in the portfolio of stocks and bonds)");
    await expect(page.locator('input[data-path="wealth.stocks"]')).toHaveValue(/^\d{1,3}(['’]\d{3})+$/);
    await page.evaluate(() => goto("pension"));
    await expect(page.locator("#main")).toContainText("Pension fund (BVG, pillar 2) and old-age pension (AHV, pillar 1)");
    await expect(page.locator("#tlRange")).toHaveText(/^\d\d\.\d{4} – \d\d\.\d{4} · \d+\.\d years$/);
    await expect(page.locator("#kpis")).toContainText("Wealth at retirement");

    // Messages of the engine: validation list and a message with an amount (thousands separator ')
    await page.evaluate(() => { state.withdrawal.strategy = "fixed_rate"; state.withdrawal.fixedRate = { rate: 0.09 }; refresh(); });
    await expect(page.locator("#val")).toContainText("A withdrawal rate of 9.0 % is high; wealth shrinks quickly over a longer period.");
    const p3a = await page.evaluate(async () => {
      const plan = JSON.parse(JSON.stringify(state));
      plan.pillar3a = [{ person: plan.household.persons[0].id, balance: 1000, rate: 0, contribution: 99999, payouts: [] }];
      return (await Engine.call("validate", { plan })).filter(i => i.code === "pillar3a_contribution_range").map(msgText);
    });
    expect(p3a[0]).toMatch(/^Pillar 3a: the contribution must be between 0 and the maximum amount of \d{1,3}('\d{3})+ per year \(settings\)\.$/);

    // Results: title from the engine label, tabs, key figures and yearly values
    await page.evaluate(() => { state.returns.source = "fixed"; state.scenarios.sweeps = []; });
    await page.evaluate(() => runSimulation("overview", { noConfirm: true }));
    await expect(page.locator("#resultsTabs button")).toHaveText(["Overview", "Charts", "Yearly values", "Parameters", "Nominal", "Today's purchasing power"]);
    await expect(page.locator("#results")).toContainText("Single run · Fixed returns");
    await expect(page.locator("#resultsBody")).toContainText("Wealth at the end");
    await page.locator("#resultsTabs button", { hasText: "Yearly values" }).click();
    await expect(page.locator("#resultsBody table.yt tr").first()).toContainText("Total spending");
    await expect(page.locator("#resultsBody table.yt td").filter({ hasText: /^-?\d{1,3}(['’]\d{3})+$/ }).first()).toBeAttached();

    // Settings
    await page.evaluate(() => command("settings"));
    await expect(page.locator("#set-language")).toHaveValue("system");
    await expect(page.locator("#set-language option")).toHaveText(["as system", "Deutsch", "English", "Français", "Italiano"]);
    await expect(page.locator("body")).toContainText("Apply to all plans on this device and are not saved in the plan file.");
    await noErrors(page, errors);
    await ctx.close();
  });
});

test.describe("French user interface (#174)", () => {
  test("AK1, AK3: system language fr: editor, results, settings, menus, messages and help in French, numbers in the Swiss format", async ({ browser }) => {
    const ctx = await browser.newContext({ locale: "fr-CH" });
    const page = await ctx.newPage();
    const errors = await openApp(page, { settings: { language: "system" } });
    expect(await lang(page)).toBe("fr");
    await expect(page.locator("html")).toHaveAttribute("lang", "fr-CH");
    // Menus: the app rebuilds the native menus in French (texts: Rust test issue_174_ak3_menus_and_dialogs_in_french)
    expect(await calls(page, "set_menu_language")).toContainEqual({ language: "fr", choice: "system" });

    // Editor
    await expect(page.locator("#nav a").first()).toContainText("Ménage et projet");
    await expect(page.locator("#nav")).toContainText("Fortune et placement");
    await expect(page.locator("header")).toContainText("Lancer la simulation");
    await page.evaluate(() => goto("wealth"));
    await expect(page.locator("#main")).toContainText("Quote-part d’actions (actions dans le portefeuille composé d’actions et d’obligations)");
    await expect(page.locator("#main")).toContainText("Monde (JST)");
    await expect(page.locator('input[data-path="wealth.stocks"]')).toHaveValue(/^\d{1,3}(['’]\d{3})+$/);
    await page.evaluate(() => goto("pension"));
    await expect(page.locator("#main")).toContainText("Caisse de pension (LPP, 2e pilier) et AVS (1er pilier)");
    await expect(page.locator("#tlRange")).toHaveText(/^\d\d\.\d{4} – \d\d\.\d{4} · \d+\.\d ans$/);
    await expect(page.locator("#kpis")).toContainText("Fortune à la retraite");

    // Help: manual and context help (F1) in French
    await page.evaluate(() => Help.show("start"));
    await expect(page.locator("#help:not([hidden]) #helpBody h1").first()).toHaveText("Ce que VivaPlan sait faire");
    expect(await page.evaluate(() => Help.language())).toBe("fr");
    await page.evaluate(() => { Help.close(); goto("wealth"); });
    await page.locator('#main [data-path^="wealth.stockShare"]').first().focus();
    await page.keyboard.press("F1");
    await expect(page.locator("#help:not([hidden]) #helpBody h1").first()).toHaveText("Part d’actions");
    await expect(page.locator("#help:not([hidden]) #helpBody")).toContainText("La part des actions dans le portefeuille");
    await page.evaluate(() => Help.close());

    // Messages of the engine: validation list and a message with an amount (thousands separator ')
    await page.evaluate(() => { state.withdrawal.strategy = "fixed_rate"; state.withdrawal.fixedRate = { rate: 0.09 }; refresh(); });
    await expect(page.locator("#val")).toContainText("Un taux de prélèvement de 9.0 % est élevé; la fortune diminue rapidement sur une longue durée.");
    const p3a = await page.evaluate(async () => {
      const plan = JSON.parse(JSON.stringify(state));
      plan.pillar3a = [{ person: plan.household.persons[0].id, balance: 1000, rate: 0, contribution: 99999, payouts: [] }];
      return (await Engine.call("validate", { plan })).filter(i => i.code === "pillar3a_contribution_range").map(msgText);
    });
    expect(p3a[0]).toMatch(/^Pilier 3a: le versement doit être compris entre 0 et le montant maximal de \d{1,3}('\d{3})+ par année \(Paramètres\)\.$/);

    // Results: title from the engine label, tabs, key figures and yearly values
    await page.evaluate(() => { state.returns.source = "fixed"; state.scenarios.sweeps = []; });
    await page.evaluate(() => runSimulation("overview", { noConfirm: true }));
    await expect(page.locator("#resultsTabs button")).toHaveText(["Vue d’ensemble", "Graphiques", "Valeurs annuelles", "Paramètres", "Nominal", "Pouvoir d’achat actuel"]);
    await expect(page.locator("#results")).toContainText("Simulation unique · Rendements fixes");
    await expect(page.locator("#resultsBody")).toContainText("Fortune à la fin");
    await page.locator("#resultsTabs button", { hasText: "Valeurs annuelles" }).click();
    await expect(page.locator("#resultsBody table.yt tr").first()).toContainText("Total des dépenses");
    await expect(page.locator("#resultsBody table.yt td").filter({ hasText: /^-?\d{1,3}(['’]\d{3})+$/ }).first()).toBeAttached();

    // Settings
    await page.evaluate(() => command("settings"));
    await expect(page.locator("#set-language")).toHaveValue("system");
    await expect(page.locator("#set-language option")).toHaveText(["comme le système", "Deutsch", "English", "Français", "Italiano"]);
    await expect(page.locator("body")).toContainText("S’appliquent à toutes les planifications sur cet appareil et ne sont pas enregistrés dans le fichier de planification.");

    await noErrors(page, errors);
    await ctx.close();
  });
});

test.describe("Italian user interface (#175)", () => {
  test("AK1, AK3: system language it: editor, results, settings, menus, messages and help in Italian, numbers in the Swiss format", async ({ browser }) => {
    const ctx = await browser.newContext({ locale: "it-CH" });
    const page = await ctx.newPage();
    const errors = await openApp(page, { settings: { language: "system" } });
    expect(await lang(page)).toBe("it");
    await expect(page.locator("html")).toHaveAttribute("lang", "it-CH");
    // Menus: the app rebuilds the native menus in Italian (texts: Rust test issue_175_ak3_menus_and_dialogs_in_italian)
    expect(await calls(page, "set_menu_language")).toContainEqual({ language: "it", choice: "system" });

    // Editor
    await expect(page.locator("#nav a").first()).toContainText("Economia domestica e progetto");
    await expect(page.locator("#nav")).toContainText("Sostanza e investimento");
    await expect(page.locator("header")).toContainText("Avviare la simulazione");
    await page.evaluate(() => goto("wealth"));
    await expect(page.locator("#main")).toContainText("Quota azionaria (azioni nel portafoglio di azioni e obbligazioni)");
    await expect(page.locator("#main")).toContainText("Mondo (JST)");
    await expect(page.locator('input[data-path="wealth.stocks"]')).toHaveValue(/^\d{1,3}(['’]\d{3})+$/);
    await page.evaluate(() => goto("pension"));
    await expect(page.locator("#main")).toContainText("Cassa pensione (LPP, 2° pilastro) e AVS (1° pilastro)");
    await expect(page.locator("#tlRange")).toHaveText(/^\d\d\.\d{4} – \d\d\.\d{4} · \d+\.\d anni$/);
    await expect(page.locator("#kpis")).toContainText("Sostanza al pensionamento");

    // Help: manual and context help (F1) in Italian
    await page.evaluate(() => Help.show("start"));
    await expect(page.locator("#help:not([hidden]) #helpBody h1").first()).toHaveText("Che cosa sa fare VivaPlan");
    expect(await page.evaluate(() => Help.language())).toBe("it");
    await page.evaluate(() => { Help.close(); goto("wealth"); });
    await page.locator('#main [data-path^="wealth.stockShare"]').first().focus();
    await page.keyboard.press("F1");
    await expect(page.locator("#help:not([hidden]) #helpBody h1").first()).toHaveText("Quota azionaria");
    await expect(page.locator("#help:not([hidden]) #helpBody")).toContainText("La quota delle azioni nel portafoglio");
    await page.evaluate(() => Help.close());

    // Messages of the engine: validation list and a message with an amount (thousands separator ')
    await page.evaluate(() => { state.withdrawal.strategy = "fixed_rate"; state.withdrawal.fixedRate = { rate: 0.09 }; refresh(); });
    await expect(page.locator("#val")).toContainText("Un tasso di prelievo del 9.0 % è alto; su una durata più lunga la sostanza si riduce rapidamente.");
    const p3a = await page.evaluate(async () => {
      const plan = JSON.parse(JSON.stringify(state));
      plan.pillar3a = [{ person: plan.household.persons[0].id, balance: 1000, rate: 0, contribution: 99999, payouts: [] }];
      return (await Engine.call("validate", { plan })).filter(i => i.code === "pillar3a_contribution_range").map(msgText);
    });
    expect(p3a[0]).toMatch(/^Pilastro 3a: il versamento deve essere compreso tra 0 e l’importo massimo di \d{1,3}('\d{3})+ all’anno \(Impostazioni\)\.$/);

    // Results: title from the engine label, tabs, key figures and yearly values
    await page.evaluate(() => { state.returns.source = "fixed"; state.scenarios.sweeps = []; });
    await page.evaluate(() => runSimulation("overview", { noConfirm: true }));
    await expect(page.locator("#resultsTabs button")).toHaveText(["Panoramica", "Grafici", "Valori annuali", "Parametri", "Nominale", "Potere d’acquisto attuale"]);
    await expect(page.locator("#results")).toContainText("Simulazione singola · Rendimenti fissi");
    await expect(page.locator("#resultsBody")).toContainText("Sostanza alla fine");
    await page.locator("#resultsTabs button", { hasText: "Valori annuali" }).click();
    await expect(page.locator("#resultsBody table.yt tr").first()).toContainText("Spese totali");
    await expect(page.locator("#resultsBody table.yt td").filter({ hasText: /^-?\d{1,3}(['’]\d{3})+$/ }).first()).toBeAttached();

    // Settings
    await page.evaluate(() => command("settings"));
    await expect(page.locator("#set-language")).toHaveValue("system");
    await expect(page.locator("#set-language option")).toHaveText(["come il sistema", "Deutsch", "English", "Français", "Italiano"]);
    await expect(page.locator("body")).toContainText("Valgono per tutte le pianificazioni su questo dispositivo e non vengono salvate nel file di pianificazione.");

    await noErrors(page, errors);
    await ctx.close();
  });
});

test.describe("Names of the optimization catalog and of the time series (#177)", () => {
  test("AK3: German names in German, English names in English (parameters and ranges, period of the strategy, short info of the series)", async ({ page }) => {
    const errors = await open(page, "wealth");
    // Strategy with a cut part: stocks Switzerland (JST) until 2020, bonds USA (Shiller) from 1922
    await page.locator('select[data-path="returns.strategy.stocks"]').selectOption("chJst");
    await page.locator('select[data-path="returns.strategy.bonds"]').selectOption("usa");
    await page.waitForFunction(() => seriesInfo?.parts?.length === 2 && state.returns.strategy.bonds === "usa" && seriesInfoRef === seriesRef());
    // Short info of a bundled series (S&P 500 with US bonds = «Shiller CHF»)
    const shillerInfo = () => page.evaluate(async () => msgText((await Engine.call("series_info", { path: 'strategy:{"stocks":"sp500","bonds":"usa"}' })).info));
    const check = async (names) => {
      await page.evaluate(() => goto("wealth"));
      await expect(page.locator("#strategyClip")).toContainText(names.stocks);
      await expect(page.locator("#strategyClip")).toContainText(names.bonds);
      await expect(page.locator("#main .infobody.short").first()).toContainText(names.info);
      expect(await shillerInfo()).toBe(names.shiller);
      await page.evaluate(() => goto("optimize"));
      await page.waitForSelector("#optR-guardrails-initialRate-var", { state: "attached" });
      const guard = page.locator("details.optrange:has(#optR-guardrails-initialRate-var)");
      await expect(guard.locator("summary b")).toHaveText("Guardrails");
      await expect(guard.locator("tr", { has: page.locator("#optR-guardrails-initialRate-var") }).locator("td").first()).toHaveText(names.initialRate);
      const cash = page.locator("tr", { has: page.locator("#optR-three_buckets-cashYears-var") });
      await expect(cash.locator("td").first()).toHaveText(names.cash);
      await expect(cash.locator("td").last()).toContainText(names.years);
      await expect(page.locator("details.optrange:has(#optR-three_buckets-cashYears-var) summary b")).toHaveText(names.buckets);
      await expect(page.locator("#val")).toContainText(names.range);
    };
    // Range with «from» greater than «to»: message of the validation with the names of strategy and parameter
    await page.evaluate(() => { state.optimization ??= {}; state.optimization.ranges = { guardrails: { initialRate: { lo: 0.08, hi: 0.02 } } }; refresh(); });
    await check({
      stocks: "Aktien Schweiz (JST)", bonds: "Obligationen USA 10 Jahre (Shiller)", info: "Aktien Schweiz (JST), Obligationen USA 10 Jahre (Shiller): jährlich 1922–2020",
      shiller: "Shiller: S&P Composite und 10-jährige US-Staatsanleihen, in CHF umgerechnet (SNB-Wechselkurs), ohne Währungsabsicherung. Teuerung: Landesindex, SARON ab 06.1999.",
      initialRate: "Startrate", cash: "Topf 1: Cash", years: "Jahre", buckets: "3-Topf", range: "Guardrails, Startrate: Die Bandbreite",
    });
    await page.evaluate(() => I18n.use("en"));
    await check({
      stocks: "Stocks Switzerland (JST)", bonds: "Bonds USA 10 years (Shiller)", info: "Stocks Switzerland (JST), bonds USA 10 years (Shiller): annual 1922–2020",
      shiller: "Shiller: S&P Composite and 10-year US Treasury bonds, converted to CHF (SNB exchange rate), without currency hedging. Inflation: Swiss CPI, SARON from 06.1999.",
      initialRate: "Initial rate", cash: "Bucket 1: cash", years: "years", buckets: "3 buckets", range: "Guardrails, Initial rate: the range from 8 % to 2 % is invalid",
    });
    // #174: French names
    await page.evaluate(() => I18n.use("fr"));
    await check({
      stocks: "Actions Suisse (JST)", bonds: "Obligations USA 10 ans (Shiller)", info: "Actions Suisse (JST), obligations USA 10 ans (Shiller): annuel 1922–2020",
      shiller: "Shiller: S&P Composite et emprunts d’État américains à 10 ans, convertis en CHF (taux de change BNS), sans couverture de change. Renchérissement: indice des prix à la consommation, SARON dès 06.1999.",
      initialRate: "Taux initial", cash: "Compartiment 1: cash", years: "ans", buckets: "3 compartiments", range: "Guardrails, Taux initial: la fourchette de 8 % à 2 % n’est pas valable",
    });
    // #175: Italian names
    await page.evaluate(() => I18n.use("it"));
    await check({
      stocks: "Azioni Svizzera (JST)", bonds: "Obbligazioni USA 10 anni (Shiller)", info: "Azioni Svizzera (JST), obbligazioni USA 10 anni (Shiller): annuali 1922–2020",
      shiller: "Shiller: S&P Composite e titoli di Stato USA a 10 anni, convertiti in CHF (tasso di cambio BNS), senza copertura valutaria. Rincaro: indice nazionale, SARON dal 06.1999.",
      initialRate: "Tasso iniziale", cash: "Comparto 1: cash", years: "anni", buckets: "3 comparti", range: "Guardrails, Tasso iniziale: la fascia da 8 % a 2 % non è valida",
    });
    await noErrors(page, errors);
  });
});

test.describe("Help in the chosen language (#173)", () => {
  const helpTitle = page => page.locator("#help:not([hidden]) #helpBody h1").first();
  const helpBody = page => page.locator("#help:not([hidden]) #helpBody");

  test("AK2: manual and search show the help of the chosen language; an open help follows a change of language", async ({ page }) => {
    const errors = await open(page);
    await page.evaluate(() => Help.show("start"));
    await expect(helpTitle(page)).toHaveText("Was VivaPlan kann");
    await page.evaluate(() => I18n.use("en"));
    await expect(helpTitle(page)).toHaveText("What VivaPlan can do");
    await expect(helpBody(page)).toContainText("Will the money last, and how certain is that?");
    // AK1: same topics in every language; every field, heading and link of the English help leads to a topic
    expect(await page.evaluate(() => Help.check())).toEqual([]);
    // Table of contents and search in English
    await page.evaluate(() => Help.show("toc"));
    await expect(helpBody(page).locator("h3").first()).toHaveText("Introduction");
    await page.locator("#helpSearch").fill("warranty");
    await expect(helpBody(page).locator(".hit b").first()).toHaveText("Disclaimer");
    await expect(helpBody(page)).not.toContainText("Haftungsausschluss");
    // Back to German: the same view in German
    await page.evaluate(() => I18n.use("de"));
    await page.locator("#helpSearch").fill("Gewähr");
    await expect(helpBody(page).locator(".hit b").first()).toHaveText("Haftungsausschluss");
    await noErrors(page, errors);
  });

  test("AK2: context help (F1, ⓘ) shows the topic in the chosen language", async ({ page }) => {
    const errors = await open(page, "wealth");
    await page.evaluate(() => I18n.use("en"));
    await page.locator('#main [data-path^="wealth.stockShare"]').first().focus();
    await page.keyboard.press("F1");
    await expect(helpTitle(page)).toHaveText("Stock allocation");
    await expect(helpBody(page)).toContainText("The share of stocks in the portfolio");
    // Without a field: the topic of the section; ⓘ at the heading
    await page.evaluate(() => { Help.close(); document.activeElement?.blur(); goto("general"); });
    await page.keyboard.press("F1");
    await expect(helpTitle(page)).toHaveText("Household & project");
    await page.evaluate(() => { Help.close(); goto("withdrawal"); });
    await page.locator('#main h2 .hq').click();
    await expect(helpTitle(page)).toHaveText("Withdrawal strategy: overview");
    await noErrors(page, errors);
  });

  test("AK2: a chapter missing in the language and a language without help fall back to German", async ({ page }) => {
    await page.route("**/hilfe/en/00-einstieg.md", r => r.fulfill({ status: 404, body: "" }));
    // #174: French has its own help since then, so the language without help is a pseudo language «xx»
    await page.route("**/i18n/languages.json", r => r.fulfill({ json: { de: "Deutsch", en: "English", xx: "Xx" } }));
    await page.route("**/i18n/xx.json", async r => {
      const de = await (await r.fetch({ url: r.request().url().replace("xx.json", "de.json") })).json();
      r.fulfill({ json: Object.fromEntries(Object.entries(de).map(([k, v]) => [k, MARK + v])) });
    });
    const errors = await open(page);
    await page.evaluate(() => I18n.use("en"));
    await page.evaluate(() => Help.show("start"));
    await expect(helpTitle(page)).toHaveText("Was VivaPlan kann");
    expect(await page.evaluate(() => Help.language())).toBe("en");
    await page.evaluate(() => I18n.use("xx"));
    await page.evaluate(() => Help.show("disclaimer"));
    await expect(helpTitle(page)).toHaveText("Haftungsausschluss");
    expect(await page.evaluate(() => Help.language())).toBe("xx");
    await noErrors(page, errors);
  });
});

test.describe("Example plan in the language of the user interface (#184)", () => {
  const shown = page => page.evaluate(() => ({
    project: state.project.name, person: state.household.persons[0].name, extras: state.flows.extras.map(e => e.label),
    properties: state.realestate.properties.map(p => p.name), age: state.milestones.find(m => m.id === "sell_house").name,
  }));

  test("AK1: first start and the welcome choice «explore example» open the example plan with the names in fr, it and en", async ({ browser }) => {
    for (const [locale, want] of [
      ["fr-CH", { project: "Planification d’exemple", person: "Marie Dupont", extras: ["Nouvelle voiture", "Grand voyage"], properties: ["Maison", "Appartement en PPE", "Appartement en location"], age: "75 ans" }],
      ["it-CH", { project: "Pianificazione d’esempio", person: "Maria Rossi", extras: ["Nuova auto", "Grande viaggio"], properties: ["Casa", "Appartamento in PPP", "Appartamento in affitto"], age: "75 anni" }],
      ["en-GB", { project: "Example plan", person: "Jane Smith", extras: ["New car", "Big trip"], properties: ["House", "Condominium", "Rented apartment"], age: "Age 75" }],
    ]) {
      const ctx = await browser.newContext({ locale });
      const page = await ctx.newPage();
      // first start: the app reports no file, so the user interface opens the example plan
      const start = await (await fetch(`${test.info().project.use.baseURL}/api/example_plan`, { method: "POST", body: "{}" })).json();
      const errors = await openApp(page, { settings: { language: "system" }, startup: { path: null, plan: start, notice: null, first: false } });
      expect(await shown(page)).toEqual(want);
      // the welcome choice «explore example» loads it again in the same language
      await page.evaluate(() => { state.project.name = "x"; History.reset(); return loadExample(); });
      expect(await shown(page)).toEqual(want);
      await noErrors(page, errors);
      await ctx.close();
    }
  });

  test("AK2: the example plans of the four languages give the same results", async ({ page }) => {
    const errors = await openApp(page);
    const results = await page.evaluate(async () => {
      const out = {};
      for (const l of ["de", "en", "fr", "it"]) {
        await I18n.use(l);
        const plan = await examplePlan();
        plan.returns.source = "fixed"; plan.scenarios.sweeps = [];
        const r = await Engine.call("run_simulation", { plan, path: null });
        out[l] = { name: plan.project.name, records: JSON.stringify(r.single.records) };
      }
      return out;
    });
    expect(new Set(Object.values(results).map(r => r.name)).size).toBe(4);
    for (const l of ["en", "fr", "it"]) expect(results[l].records, l).toBe(results.de.records);
    await noErrors(page, errors);
  });

  test("AK3: a saved plan keeps its names in another language", async ({ browser }) => {
    const ctx = await browser.newContext({ locale: "fr-CH" });
    const page = await ctx.newPage();
    const start = await (await fetch(`${test.info().project.use.baseURL}/api/example_plan`, { method: "POST", body: "{}" })).json();
    // the example plan saved in German, opened again with the user interface in French
    const errors = await openApp(page, { settings: { language: "system" }, startup: { path: "/x/Beispiel.vplan", plan: start, notice: null, first: false } });
    expect(await lang(page)).toBe("fr");
    expect(await page.evaluate(() => [state.project.name, state.household.persons[0].name, state.realestate.properties[0].name]))
      .toEqual(["Beispielplanung", "Erika Muster", "Haus"]);
    // switching the language does not translate the open plan either
    await page.evaluate(() => I18n.use("it"));
    expect(await page.evaluate(() => state.project.name)).toBe("Beispielplanung");
    await noErrors(page, errors);
    await ctx.close();
  });
});
