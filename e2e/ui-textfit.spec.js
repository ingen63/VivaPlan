// #182/AK4: In DE, EN, FR and IT no view, dialog or help page shows cut-off or overlapping text (check: e2e/textfit.js).
// Window size of the desktop app (tauri.conf.json); the language comes from the system language of the browser (#170/AK7).
const { test, expect } = require("@playwright/test");
const { textProblems } = require("./textfit");

const LANGUAGES = ["de", "en", "fr", "it"];
const WINDOW = { width: 1440, height: 920 };

/** Opens the user interface in the language with test access via the address; returns the page and the collected findings. */
async function start(browser, language) {
  const context = await browser.newContext({ locale: `${language}-CH`, viewport: WINDOW });
  const page = await context.newPage();
  const errors = [];
  page.on("pageerror", e => errors.push(e.message));
  const found = {};
  return {
    page, errors, found,
    async open(hash = "") {
      await page.goto("about:blank"); // a new address part after «#» alone would not load the page again
      await page.goto("/#" + [hash, "nobench"].filter(Boolean).join("/"));
      await page.waitForSelector("html[data-ready='1']");
      await expect(page.locator("html")).toHaveAttribute("data-language", language);
    },
    /** Checks the visible part (or the root) and keeps the findings under the name of the view. */
    async check(name, root = "body") {
      await page.waitForTimeout(150); // drawing of the preview and the charts
      const problems = await textProblems(page, root);
      if (problems.length) found[name] = problems;
    },
    close: () => context.close(),
  };
}

test.describe("Text fits in every language (#182)", () => {
  for (const language of LANGUAGES) {
    test(`AK4 ${language}: editor sections, also with partner, every withdrawal strategy and return source`, async ({ browser }) => {
      test.setTimeout(180_000);
      const t = await start(browser, language);
      const variants = { "": null, partner: null, buckets: ["withdrawal"], guardrails: ["withdrawal"], vpw: ["withdrawal"], endowment: ["withdrawal"],
        fixedrate: ["withdrawal"], historical: ["returns"], rolling: ["returns"], bootstrap: ["returns"], parametric: ["returns"] };
      for (const [hash, only] of Object.entries(variants)) {
        await t.open(hash);
        for (const id of only ?? await t.page.evaluate(() => SECTIONS.map(s => s.id))) {
          await t.page.evaluate(id => goto(id), id);
          await t.check(`${hash || "plan"} › ${id}`);
        }
      }
      expect(t.found).toEqual({});
      expect(t.errors).toEqual([]);
      await t.close();
    });

    test(`AK4 ${language}: dialogs`, async ({ browser }) => {
      const t = await start(browser, language);
      await t.open();
      for (const [name, call, root] of [["settings", "command('settings')", "#settings"], ["about", "command('about')", "#ask"],
        ["shortcuts", "command('help')", "#ask"], ["welcome", "showWelcome()", "#welcome"], ["steps", "command('steps')", "body"], ["json", "ACTIONS.json({})", "#dlg"],
        ["connectai", "await showConnectAi()", "#ask"]]) { // #213: waits for app_info, then the dialog is open
        await t.page.evaluate(`(async () => { ${call}; })()`); // the dialog promise is fulfilled only when it closes
        await t.check(name, root);
        await t.page.evaluate(() => document.querySelectorAll("dialog[open]").forEach(d => d.close()));
      }
      expect(t.found).toEqual({});
      expect(t.errors).toEqual([]);
      await t.close();
    });

    test(`AK4 ${language}: results, list and comparison of simulations`, async ({ browser }) => {
      test.setTimeout(240_000);
      const t = await start(browser, language);
      for (const hash of ["results/overview", "results/charts", "results/table", "results/bootstrap/overview", "results/bootstrap/charts",
        "results/bootstrap/runs", "results/bootstrap/table", "results/bootstrap/scenarios/sweep2", "results/bootstrap/overview/real"]) {
        await t.open(hash);
        await t.page.waitForFunction(() => !document.querySelector("#results").hidden && document.documentElement.dataset.rendered, null, { timeout: 120_000 });
        await t.check(hash);
        const tabs = t.page.locator('#resultsTabs button[data-action="rtab"]');
        if (hash.endsWith("overview")) for (let i = 0; i < await tabs.count(); i++) { await tabs.nth(i).click(); await t.check(`${hash} › tab ${i + 1}`); }
      }
      await t.open("runsflow/mem");
      await expect(t.page.locator("html")).toHaveAttribute("data-rendered", "compare", { timeout: 120_000 });
      await t.page.evaluate(() => showResults(true)); // the test access compares without opening the results
      await t.check("comparison");
      const tabs = t.page.locator('button[data-action="cmp-tab"]:visible');
      for (let i = 0; i < await tabs.count(); i++) { await tabs.nth(i).click(); await t.check(`comparison › tab ${i + 1}`); }
      await t.page.evaluate(() => { openRuns(); });
      await t.check("list of simulations");
      expect(t.found).toEqual({});
      expect(t.errors).toEqual([]);
      await t.close();
    });

    test(`AK4 ${language}: every help topic`, async ({ browser }) => {
      const t = await start(browser, language);
      await t.open();
      await t.page.evaluate(() => Help.show("start"));
      await expect(t.page.locator("#help")).toBeVisible();
      for (const id of await t.page.evaluate(() => ["toc", ...Help.topics.keys()])) {
        await t.page.evaluate(id => Help.show(id), id);
        await t.check(id, "#help");
      }
      expect(t.found).toEqual({});
      expect(t.errors).toEqual([]);
      await t.close();
    });
  }
});

test.describe("Number field in the section flows not cut off (#244)", () => {
  for (const language of LANGUAGES) {
    // #244/AK2: an amount per year with 7 digits and the per-month amount fit beside the longest milestone (also fr and it).
    test(`AK2 ${language}: income per year with 7 digits in a step course`, async ({ browser }) => {
      const t = await start(browser, language);
      await t.open();
      await t.page.evaluate(() => goto("flows"));
      const field = t.page.locator('input[data-path="flows.income.rows.0.value"]');
      await field.fill("1'200'000");
      await field.press("Tab");
      await expect(field).toHaveValue("1'200'000");
      await t.check("flows › 1'200'000");
      expect(t.found).toEqual({});
      expect(t.errors).toEqual([]);
      await t.close();
    });
  }
});

test.describe("Choice «Steuerwert» of a property not cut off (#257)", () => {
  for (const language of LANGUAGES) {
    test(`AK1 ${language}: every choice fully visible, also with the amount and at the smallest window width`, async ({ browser }) => {
      const t = await start(browser, language);
      for (const width of [WINDOW.width, 1000]) {
        await t.page.setViewportSize({ width, height: WINDOW.height });
        await t.open();
        const i = await t.page.evaluate(() => state.realestate.properties.findIndex(p => p.status === "owned"));
        expect(i).toBeGreaterThanOrEqual(0);
        for (const tv of [undefined, { kind: "amount", value: 99999999 }, { kind: "rate", value: 0.6543 }]) {
          await t.page.evaluate(([i, tv]) => { const p = state.realestate.properties[i]; if (tv) p.taxValue = tv; else delete p.taxValue; markDirty(); goto("realestate"); }, [i, tv]);
          await t.check(`${width} › ${tv?.kind ?? "none"}`, `[data-tax-value="${i}"]`);
          // every option, not only the selected one, fits into the choice (the widest one sets its width)
          const cut = await t.page.locator(`[data-tax-value="${i}"] select`).evaluate(sel => {
            const s = getComputedStyle(sel), ctx = document.createElement("canvas").getContext("2d");
            ctx.font = s.font;
            const free = sel.clientWidth - parseFloat(s.paddingLeft) - parseFloat(s.paddingRight) - 20; // arrow of a select (as textfit.js)
            return [...sel.options].map(o => o.textContent.trim()).filter(x => ctx.measureText(x).width > free + 0.5);
          });
          expect(cut, `${width} ${tv?.kind ?? "none"}`).toEqual([]);
        }
      }
      expect(t.found).toEqual({});
      expect(t.errors).toEqual([]);
      await t.close();
    });
  }
});
