// User interface tests of VivaPlan in the browser: settings and plan fields. Every test starts with the example plan.
const { test, expect } = require("@playwright/test");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { MOD, SECTIONS, open, noErrors, field, previewIdle, nav, kpiValue, resultKpi, parseNum, setOpt, msRow, welcome, steps, helpTitle, prefs } = require("./helpers");

// ---------------------------------------------------------------------------------------------
test.describe("Settings, fixed milestones and pillar 3a (#25, #50, #52, #51)", () => {
  test("#25/AK1–AK4: show, check and save settings, keep them after reloading, default values", async ({ page }) => {
    const errors = await open(page);
    await page.locator('header button[data-action="settings"]').click();
    const dlg = page.locator("#settings");
    await expect(dlg).toBeVisible();
    for (const t of ["AHV-Bezug: untere Grenze", "AHV-Bezug: obere Grenze", "Bezug Pensionskasse: untere Grenze", "Bezug Pensionskasse: obere Grenze", "Säule 3a: Maximalbetrag der Einzahlung pro Jahr"])
      await expect(dlg).toContainText(t);
    await expect(page.locator("#set-pkMin")).toHaveValue("58");
    await expect(page.locator("#set-pillar3aMax")).toHaveValue("7'258");
    // AK4: a lower limit above the upper one is rejected and not saved
    await page.locator("#set-ahvMin").fill("71");
    await page.locator("#settingsSave").click();
    await expect(page.locator("#settingsErr")).toContainText("AHV-Bezug: Die untere Grenze (71) liegt über der oberen Grenze (70).");
    await expect(dlg).toBeVisible();
    expect(await page.evaluate(() => limits().ahvMin)).toBe(63);
    // AK2: a valid change still applies after reloading
    await page.locator("#set-ahvMin").fill("63");
    await page.locator("#set-pkMax").fill("67");
    await page.locator("#settingsSave").click();
    await expect(dlg).toBeHidden();
    await page.reload();
    await page.waitForSelector("html[data-ready='1']");
    expect(await page.evaluate(() => limits().pkMax)).toBe(67);
    // The limits are not part of the plan
    expect(await page.evaluate(() => "limits" in state)).toBe(false);
    // AK3: default values after a question
    await page.keyboard.press(`${MOD}+,`);
    await expect(dlg).toBeVisible();
    await page.locator("#settingsDefaults").click();
    await page.locator("#ask button[value='ok']").click();
    await expect(page.locator("#set-pkMax")).toHaveValue("65");
    expect(await page.evaluate(() => limits().pkMax)).toBe(65);
    await noErrors(page, errors);
  });

  test("#25/AK6, #51/AK2: upper limit pension fund withdrawal 67: 67 valid, 68 rejected", async ({ page }) => {
    const errors = await open(page, "milestones");
    await page.evaluate(() => Pref.set("limits", { ...LIMIT_DEFAULTS, pkMax: 67 }));
    const row = await msRow(page, "retire");
    const years = row.locator("input[data-type=years]");
    await years.fill("67");
    await expect(page.locator("#val li.err")).toHaveCount(0);
    await years.fill("68");
    await years.press("Tab"); // #57: message at the field only on leaving
    await expect(page.locator("#val li.err").first()).toContainText("«Bezug Pensionskasse»: Alter 68 liegt ausserhalb der Grenzen 58 bis 67 Jahre (Einstellungen).");
    await expect(row).toContainText("Alter 68 liegt ausserhalb");
    await noErrors(page, errors);
  });

  test("#51/AK1, AK5: fixed milestones with a fixed name and own person; pension fund and AHV start there", async ({ page }) => {
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
    // Second person: own fixed milestones
    await page.evaluate(() => ACTIONS["partner-add"]());
    // #170/AK8: no stored name; the name comes from the text table by the id
    expect(await page.evaluate(() => state.milestones.filter(m => m.id.endsWith("2")).map(m => [m.name, msLabel(m)])))
      .toEqual([[undefined, "Bezug Pensionskasse Person 2"], [undefined, "AHV-Bezug Person 2"]]);
    await noErrors(page, errors);
  });

  test("#51/AK8: warning on earned income after the pension fund withdrawal", async ({ page }) => {
    const errors = await open(page, "flows");
    await page.evaluate(() => { state.flows.income = { mode: "const", value: 12500 }; renderSection(); });
    await expect(page.locator("#val li.warn", { hasText: "Das Erwerbseinkommen beträgt nach dem Bezug der Pensionskasse" })).toContainText("noch 12'500 pro Monat");
    await noErrors(page, errors);
  });

  test("#52/AK1, AK2: pillar 3a section with account, contribution and payout plan", async ({ page }) => {
    const errors = await open(page, "pillar3a");
    await expect(page.locator("#main h2")).toContainText("Säule 3a");
    await expect(field(page, "pillar3a.0.balance")).toBeVisible();
    await expect(field(page, "pillar3a.0.contribution")).toBeVisible();
    const before = await page.evaluate(() => state.pillar3a[0].payouts.length);
    await page.locator('button[data-action="p3a-payout-add"]').click();
    expect(await page.evaluate(() => state.pillar3a[0].payouts.length)).toBe(before + 1);
    await field(page, `pillar3a.0.payouts.${before}.amount`).fill("50000");
    expect(await page.evaluate(n => state.pillar3a[0].payouts[n].amount, before)).toBe(50000);
    // Empty = whole balance
    await field(page, `pillar3a.0.payouts.${before}.amount`).fill("");
    expect(await page.evaluate(n => "amount" in state.pillar3a[0].payouts[n] && state.pillar3a[0].payouts[n].amount !== undefined, before)).toBe(false);
    await page.locator(`button[data-action="list-del"][data-path="pillar3a.0.payouts"]`).last().click();
    expect(await page.evaluate(() => state.pillar3a[0].payouts.length)).toBe(before);
    // AK4: a payout more than 5 years before the AHV start is an error
    await page.evaluate(() => { state.pillar3a[0].payouts[0].date = "2028-01"; renderSection(); });
    await expect(page.locator("#val li.err").first()).toContainText("liegt mehr als 5 Jahre vor dem AHV-Bezug");
    // AK7: contribution above the maximum amount
    await page.evaluate(() => { state.pillar3a[0].payouts[0].date = "2031-01"; state.pillar3a[0].contribution = 8000; renderSection(); });
    await expect(page.locator("#val li.err").first()).toContainText("Die Einzahlung muss zwischen 0 und dem Maximalbetrag von 7'258 pro Jahr liegen");
    await noErrors(page, errors);
  });

  test("#52/AK8: the contribution appears in total spending of the annual values", async ({ page }) => {
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

  test("#50/AK4: period from the AHV start amortizes directly", async ({ page }) => {
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

  test("#51/AK6, #52/AK13: an older file is taken over when opened, a dialog names the changes", async ({ page }) => {
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
test.describe("Living costs including housing and cash (#53, #54)", () => {
  test("#53/AK1, AK2: label including housing costs, notice with the housing share at the simulation start", async ({ page }) => {
    const errors = await open(page, "flows");
    await expect(page.locator("#main")).toContainText("Lebenshaltungskosten / Monat inklusive Miete bzw. Wohnkosten (Zins, Amortisation, Nebenkosten)");
    // Example plan: housing costs at the simulation start 1'700 per month
    await expect(page.locator("#main .hint").filter({ hasText: "Wohnanteil zum Simulationsstart" })).toContainText("1'700");
    await noErrors(page, errors);
  });

  test("#53/AK4: living costs smaller than the housing share are an error with both amounts", async ({ page }) => {
    const errors = await open(page);
    const issues = await page.evaluate(async () => {
      const s = JSON.parse(JSON.stringify(state));
      s.flows.spending = { mode: "const", value: 1000 };
      return (await Engine.call("validate", { plan: s })).map(i => msgText(i));
    });
    expect(issues).toContain("Die Lebenshaltungskosten (1'000) sind kleiner als die Wohnkosten zum Simulationsstart (1'700), die sie enthalten.");
    await noErrors(page, errors);
  });

  test("#54/AK1, AK4: stock ratio without cash; notice on the cash rule before and from retirement", async ({ page }) => {
    const errors = await open(page, "wealth");
    await expect(page.locator("#main")).toContainText("Aktienquote (Aktien am Portfolio aus Aktien und Obligationen)");
    await expect(page.locator("#main")).toContainText("Cash zählt nicht zum Portfolio");
    await expect(page.locator("#main")).not.toContainText("Liquidität");
    await page.evaluate(() => { state.withdrawal.strategy = "standard"; goto("withdrawal"); });
    await expect(page.locator("#main")).toContainText("ab der Pensionierung");
    await noErrors(page, errors);
  });

  test("#53/AK7, #54/AK8: older file: housing share added, notices on housing costs and cash", async ({ page }) => {
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
test.describe("Pillar 3a: payout plan with date or age (#55)", () => {
  // Invented example: birth 1980-06 (AHV start 06.2045, earliest 06.2040), balance 120'000, interest 0, no contribution.
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

  test("#55/AK1, AK2, AK8: date or age; month and age next to it; switching keeps the point in time; old rows are dates", async ({ page }) => {
    const errors = await open(page);
    await setup(page, [{ kind: "age", years: 61.5, amount: 50000, use: "wealth" }, { date: "2043-01", amount: 1000, use: "wealth" }]);
    await expect(at(page, 0)).toHaveText("12.2041 · Alter 61.5");
    // AK8: a row with only a date appears as «Datum»
    await expect(page.locator('select[data-path="pillar3a.0.payouts.1.kind"]')).toHaveValue("date");
    await expect(at(page, 1)).toHaveText("01.2043 · Alter 62.6");
    // AK2: age 61.5 → date 12.2041
    await page.locator('select[data-path="pillar3a.0.payouts.0.kind"]').selectOption("date");
    await expect(field(page, "pillar3a.0.payouts.0.date")).toHaveValue("12.2041");
    expect(await page.evaluate(() => state.pillar3a[0].payouts[0])).toEqual({ amount: 50000, use: "wealth", date: "2041-12" });
    // and back
    await page.locator('select[data-path="pillar3a.0.payouts.0.kind"]').selectOption("age");
    await expect(field(page, "pillar3a.0.payouts.0.years")).toHaveValue("61.5");
    await expect(at(page, 0)).toHaveText("12.2041 · Alter 61.5");
    await noErrors(page, errors);
  });

  test("#55/AK3, AK4: at the earliest 5 years before the AHV start also by age; without a preview a dash is shown", async ({ page }) => {
    const errors = await open(page);
    await setup(page, [{ kind: "age", years: 59.9, amount: 50000, use: "wealth" }]);
    await expect(page.locator("#val li.err").first()).toContainText("Die Auszahlung im 05.2040 liegt mehr als 5 Jahre vor dem AHV-Bezug (06.2045)");
    await expect(val(page, 0, "before")).toHaveText("–");
    await expect(val(page, 0, "after")).toHaveText("–");
    await expect(page.locator('#main [data-p3a="rest"]')).toHaveText("–");
    await noErrors(page, errors);
  });

  test("#55/AK4–AK7: balance before and after the payout, remainder at the AHV start; adding a payout prefills point in time and amount", async ({ page }) => {
    const errors = await open(page);
    await setup(page, [{ kind: "age", years: 61.5, amount: 50000, use: "wealth" }]);
    await expect(val(page, 0, "before")).toHaveText("120'000");
    await expect(val(page, 0, "after")).toHaveText("70'000");
    await expect(page.locator('#main [data-p3a="rest"]')).toHaveText(/^70'000/);
    // AK7: one year later, remainder 70'000 (same kind as the last row)
    await page.locator('button[data-action="p3a-payout-add"]').click();
    expect(await page.evaluate(() => state.pillar3a[0].payouts[1])).toEqual({ use: "wealth", kind: "age", years: 62.5, amount: 70000 });
    await expect(at(page, 1)).toHaveText("12.2042 · Alter 62.5");
    // The amount stays editable
    await field(page, "pillar3a.0.payouts.1.amount").fill("20000");
    expect(await page.evaluate(() => state.pillar3a[0].payouts[1].amount)).toBe(20000);
    await field(page, "pillar3a.0.payouts.1.amount").press("Tab"); // #57: preview on leaving (#111)
    await expect(page.locator('#main [data-p3a="rest"]')).toHaveText(/^50'000/);
    // Without a row: earliest allowed month (06.2040) with the whole balance, rounded down to 1'000
    await setup(page, []);
    await page.evaluate(() => { state.pillar3a[0].balance = 120400; refresh(); });
    await expect(page.locator('#main [data-p3a="rest"]')).toHaveText(/^120'400/);
    await page.locator('button[data-action="p3a-payout-add"]').click();
    expect(await page.evaluate(() => state.pillar3a[0].payouts[0])).toEqual({ use: "wealth", date: "2040-06", amount: 120000 });
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Error message at the field only on leaving (#57)", () => {
  const ferr = el => el.locator("xpath=ancestor-or-self::div[contains(@class,'inp')][1]/following-sibling::span[contains(@class,'ferr')]");
  /**
   * #111: Checks that nothing happens during a pause in the input. Instead of waiting a fixed time, the page runs with
   * the clock of Playwright (page.clock): `pause()` advances it by 800 ms and runs every due timer (preview
   * after 120 ms, collection time of the undo history 400 ms), independent of the machine load. In addition the page counts the
   * requests to the engine: none may start during the input.
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

  test("#57/AK1–AK4: during input everything waits; on leaving message, preview and check list anew", async ({ page }) => {
    const errors = await openWithClock(page, "milestones");
    const i = await page.evaluate(() => state.milestones.findIndex(m => m.id === "ahv"));
    const years = field(page, `milestones.${i}.years`);
    const end = kpiValue(page, "Vermögen am Ende");
    await expect(end).not.toHaveText("–");
    const before = await end.textContent();
    // AK1, AK4: digit by digit, with a pause: no message at the field, none in the check list, preview unchanged,
    // no request to the engine (pause).
    await previewIdle(page); await apiCalls(page);
    await years.click(); await years.press("End");
    await years.press("Backspace"); await years.press("Backspace"); await years.type("6");
    await pause(page);
    await expect(ferr(years)).toHaveCount(0);
    await expect(page.locator("#val li.err")).toHaveCount(0);
    await expect(end).toHaveText(before);
    await years.type("3"); // 63: valid, the value is taken over, computed only on leaving
    await pause(page);
    expect(await page.evaluate(n => state.milestones[n].years, i)).toBe(63);
    await expect(end).toHaveText(before);
    await years.press("Tab");
    await expect(end).not.toHaveText(before);
    await expect(ferr(years)).toHaveCount(0);
    // AK2: enter age 6: message at the field and in the check list only on leaving
    await previewIdle(page); await apiCalls(page); // Computations after leaving the field are allowed, counting starts with the next input
    await years.fill("6");
    await pause(page);
    await expect(ferr(years)).toHaveCount(0);
    await expect(page.locator("#val li.err")).toHaveCount(0);
    await years.press("Tab");
    await expect(ferr(years)).toContainText("ausserhalb der Grenzen");
    await expect(page.locator("#val li.err").first()).toContainText("AHV-Bezug");
    // AK3: back into the field, make it valid: the message stays during input and disappears on leaving
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

  test("#57/AK1, AK2: an invalid number is reported only on leaving or with Enter", async ({ page }) => {
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
test.describe("Number input with a dot as thousands separator (#140)", () => {
  const ferr = el => el.locator("xpath=ancestor-or-self::div[contains(@class,'inp')][1]/following-sibling::span[contains(@class,'ferr')]");

  test("AK1, AK4: amount «12.500» is rejected with a message, the previous value remains", async ({ page }) => {
    const errors = await open(page, "wealth");
    const cash = field(page, "wealth.cash");
    const before = await page.evaluate(() => state.wealth.cash);
    await cash.fill("12.500");
    await cash.press("Tab");
    await expect(ferr(cash)).toContainText("Mehrdeutige Zahl");
    await expect(cash).toHaveClass(/invalid/);
    expect(await page.evaluate(() => state.wealth.cash)).toBe(before);
    // With an apostrophe the same amount is taken over
    await cash.fill("12'500");
    await cash.press("Tab");
    await expect(cash).not.toHaveClass(/invalid/);
    expect(await page.evaluate(() => state.wealth.cash)).toBe(12500);
    await noErrors(page, errors);
  });

  test("AK1, AK2: number of paths «50.000» and «1e5» are rejected, percent «1.25» stays valid", async ({ page }) => {
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

test.describe("Earned income with inflation (#58)", () => {
  test("#58/AK1: selection per person, preview recomputes immediately", async ({ page }) => {
    const errors = await open(page, "flows");
    const own = page.locator('input[data-path="flows.incomeIndexed"]');
    await expect(own).toBeVisible();
    await expect(own).not.toBeChecked(); // Example plan without a value: «nein»
    await expect(page.locator('input[data-path="partner.incomeIndexed"]')).toHaveCount(0); // without a second person
    // Exact final value of the preview (the key figure is rounded)
    const endValue = () => page.evaluate(() => lastSim?.total.at(-1));
    await expect(kpiValue(page, "Vermögen am Ende")).not.toHaveText("–");
    const before = await endValue();
    await own.check();
    expect(await page.evaluate(() => state.flows.incomeIndexed)).toBe(true);
    await expect.poll(endValue).not.toBe(before);
    await own.uncheck();
    await expect.poll(endValue).toBe(before);
    // Second person: own selection, new with «ja»
    await nav(page, "Haushalt & Projekt");
    await page.getByRole("button", { name: "+ Zweite Person" }).click();
    await nav(page, "Einnahmen & Ausgaben");
    await expect(page.locator('input[data-path="partner.incomeIndexed"]')).toBeChecked();
    await expect(own).not.toBeChecked();
    await noErrors(page, errors);
  });

  test("#58: new plans and a new second person with rises with inflation", async ({ page }) => {
    const errors = await open(page);
    await page.locator('button[data-action="new"]').click();
    await page.waitForFunction(() => state.project.name === "" && state.wealth.stocks === 0); // #170: no project name in a new plan
    expect(await page.evaluate(() => state.flows.incomeIndexed)).toBe(true);
    await nav(page, "Einnahmen & Ausgaben");
    await expect(page.locator('input[data-path="flows.incomeIndexed"]')).toBeChecked();
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("13th AHV pension (#190)", () => {
  test("AK8: the field for the monthly AHV pension explains which amount is meant", async ({ page }) => {
    const errors = await open(page, "pension");
    const label = page.locator("label.field", { hasText: "Rente pro Monat bei Bezugsbeginn" });
    await expect(label).toHaveCount(1);
    await expect(label.locator(".computed")).toHaveText(
      "Erwartete monatliche AHV-Rente beim Bezug mit 65 (Referenzalter) laut Rentenvorausberechnung, in heutigen Preisen, ohne 13. Rente (die App rechnet sie ab Dezember 2026 selbst dazu).");
    await noErrors(page, errors);
  });

  test("AK6: the annual values contain the 13th AHV pension in December in the AHV income", async ({ page }) => {
    const errors = await open(page);
    const out = await page.evaluate(async () => {
      state.returns.source = "fixed"; state.scenarios.sweeps = [];
      lastRun = await Engine.call("run_simulation", { plan: state, path: null });
      realView = false;
      const recs = lastRun.single.records, at = m => recs.find(r => r.month === m);
      const y = yearly(recs).find(a => String(a.y) === "2036");
      return { jan: at("2036-01").ahv, nov: at("2036-11").ahv, dec: at("2036-12").ahv, year: y.ahv };
    });
    // example plan: AHV 2'000 from 06.2035, indexed in January; 2036 a full year with the same monthly pension
    expect(out.jan).toBeGreaterThan(2000);
    expect(out.nov).toBeCloseTo(out.jan, 6);
    const thirteenth = Math.round(12 * Math.round(out.jan / 12 * 100) / 100);
    expect(out.dec).toBeCloseTo(out.jan + thirteenth, 6);
    expect(out.year).toBeCloseTo(12 * out.jan + thirteenth, 6);
    await noErrors(page, errors);
  });

  test("AK12: an older plan shows the note on the 13th AHV pension once", async ({ page }) => {
    const errors = await open(page);
    const plan = await page.evaluate(() => {
      const s = JSON.parse(JSON.stringify(state));
      delete s.pension.ahv.thirteenthNoted;
      return s;
    });
    const file = path.join(os.tmpdir(), `ahv13-${process.pid}.vplan`);
    fs.writeFileSync(file, JSON.stringify(plan));
    await page.locator("#fileInput").setInputFiles(file);
    await expect(page.locator("#ask")).toBeVisible();
    await expect(page.locator("#askText")).toContainText("Ab Dezember 2026 rechnet die Simulation mit der 13. AHV-Rente");
    await page.locator("#ask button[value='ok']").click();
    expect(await page.evaluate(() => state.pension.ahv.thirteenthNoted)).toBe(true);
    // the plan as saved now: no note on the next opening
    fs.writeFileSync(file, JSON.stringify(await page.evaluate(() => state)));
    await page.locator("#fileInput").setInputFiles(file);
    await expect(page.locator("#main")).toBeVisible();
    await page.waitForTimeout(300);
    const again = await page.locator("#ask").isVisible() ? await page.locator("#askText").innerText() : "";
    expect(again).not.toContain("13. AHV-Rente");
    fs.rmSync(file);
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Early and deferred AHV payout (#191)", () => {
  test("AK8: the effective AHV pension next to the field follows pension, payout and cap", async ({ page }) => {
    const errors = await open(page, "pension");
    const eff = i => page.locator(`[data-computed="ahvEff"][data-arg="${i}"]`);
    // Swiss format: the apostrophe of the browser (’) as in the issue (')
    const text = async i => (await eff(i).innerText()).replace(/’/g, "'");
    const expectText = (i, t) => expect.poll(() => text(i)).toBe(t);
    // example plan: AHV 2'000 at 65
    await expectText(0, "Ohne Kürzung oder Zuschlag → effektiv 2'000.00 CHF pro Monat");
    await field(page, "pension.ahv.monthly").fill("2520");
    await field(page, "pension.ahv.monthly").press("Tab");
    await expectText(0, "Ohne Kürzung oder Zuschlag → effektiv 2'520.00 CHF pro Monat");
    // payout at 63: −13.6 %
    const payout = async years => {
      await nav(page, "Meilensteine");
      const input = (await msRow(page, "ahv")).locator("input[data-type=years]");
      await input.fill(String(years));
      await input.press("Tab");
      await nav(page, "Vorsorge");
    };
    await payout(63);
    await expectText(0, "−13.6 % → effektiv 2'177.00 CHF pro Monat");
    await payout(68);
    await expectText(0, "+17.1 % → effektiv 2'951.00 CHF pro Monat");
    await payout(65);
    await expectText(0, "Ohne Kürzung oder Zuschlag → effektiv 2'520.00 CHF pro Monat");

    // married couple: both 2'520 at 65, cap 3'780 → from the month both pensions run 1'890.00 each
    await page.evaluate(() => { ACTIONS["partner-add"](); state.household.persons[1].birth = "1972-03"; renderSection(); requestPreview(); });
    await field(page, "partner.ahv.monthly").fill("2520");
    await field(page, "partner.ahv.monthly").press("Tab");
    // person 1: 65 in 06.2035, person 2: 65 in 03.2037
    await expectText(0, "Ohne Kürzung oder Zuschlag → effektiv 2'520.00 CHF pro Monat, ab 03.2037 plafoniert 1'890.00 CHF");
    await expectText(1, "Ohne Kürzung oder Zuschlag → effektiv 2'520.00 CHF pro Monat, ab 03.2037 plafoniert 1'890.00 CHF");
    // person 1 early at 63 (06.2033): from 03.2037 capped 1'890, minus the fixed reduction 343 (13.6 % of 2'520)
    await payout(63);
    await expectText(0, "−13.6 % → effektiv 2'177.00 CHF pro Monat, ab 03.2037 plafoniert 1'547.00 CHF");
    await expectText(1, "Ohne Kürzung oder Zuschlag → effektiv 2'520.00 CHF pro Monat, ab 03.2037 plafoniert 1'890.00 CHF");
    await payout(65);
    // a higher cap: no longer capped
    await nav(page, "Haushalt & Projekt");
    await field(page, "household.ahvCoupleCap").fill("6000");
    await field(page, "household.ahvCoupleCap").press("Tab");
    await nav(page, "Vorsorge");
    await expectText(1, "Ohne Kürzung oder Zuschlag → effektiv 2'520.00 CHF pro Monat");
    await noErrors(page, errors);
  });

  test("AK9: an older plan with an AHV payout at 63 is converted back to the pension at 65 with a note", async ({ page }) => {
    const errors = await open(page);
    const plan = await page.evaluate(() => {
      const s = JSON.parse(JSON.stringify(state));
      delete s.pension.ahv.referenceAge;
      s.milestones.find(m => m.id === "ahv").years = 63;
      s.pension.ahv.monthly = 2177.28;
      return s;
    });
    const file = path.join(os.tmpdir(), `ahv191-${process.pid}.vplan`);
    fs.writeFileSync(file, JSON.stringify(plan));
    await page.locator("#fileInput").setInputFiles(file);
    await expect(page.locator("#ask")).toBeVisible();
    await expect(page.locator("#askText")).toContainText("Die AHV-Rente wird neu als Rente mit 65 (Referenzalter) erfasst; die App kürzt sie beim Vorbezug selbst um 13.6 %. Der bisherige Betrag 2177.28 ist deshalb auf 2520.00 pro Monat umgerechnet.");
    await page.locator("#ask button[value='ok']").click();
    expect(await page.evaluate(() => [state.pension.ahv.monthly, state.pension.ahv.referenceAge])).toEqual([2520, true]);
    await nav(page, "Vorsorge");
    await expect.poll(async () => (await page.locator('[data-computed="ahvEff"][data-arg="0"]').innerText()).replace(/’/g, "'"))
      .toBe("−13.6 % → effektiv 2'177.00 CHF pro Monat");
    fs.rmSync(file);
    await noErrors(page, errors);
  });
});
