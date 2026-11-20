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
          if (cmd === "settings_get") return { disclaimerAccepted: 1, ...window.__settings };
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
    // #172: English has its own table since then, so only languages without a table fall back
    for (const locale of ["fr-FR", "ja-JP", "pt-BR"]) {
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
      // labels, headings, column titles, buttons, hints: each must come from the (pseudo) table
      missing.push(...await page.evaluate(mark => [...document.querySelectorAll(
        "#main h2, #main h3, #main th, #main label.field > span:first-child, #main .hint, #main p.lead, #main button, header button, #nav a, aside h3")]
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
      const p = await page.evaluate(async () => blankPlan(await Engine.call("example_plan"), new Date(2030, 0, 1)));
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
        events: run.single.events.map(e => [msgText(e), engineText(e, k => de[k]), e.text]),
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
    await expect(page.locator("#main")).toContainText("Stock allocation (stocks in the portfolio of stocks and bonds)");
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
    await expect(page.locator("#set-language option")).toHaveText(["as system", "Deutsch", "English"]);
    await expect(page.locator("body")).toContainText("Apply to all plans on this device and are not saved in the plan file.");
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
    await noErrors(page, errors);
  });
});
