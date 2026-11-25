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
    await expect(page.locator("#val li.warn", { hasText: "Das Erwerbseinkommen beträgt nach dem Bezug der Pensionskasse" })).toContainText("noch 150'000 pro Jahr"); // #232: per year (12 × 12'500)
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
      await page.evaluate(c => { realView = false; state.returns.source = "fixed"; state.scenarios.sweeps = []; state.pillar3a[0].contribution = c; }, contribution);
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
test.describe("Housing share without amortization (#264)", () => {
  test("AK6: label and notice name the housing share without amortization", async ({ page }) => {
    const errors = await open(page, "flows");
    await expect(page.locator("#main")).toContainText("Lebenshaltungskosten / Monat inklusive Miete bzw. Wohnkosten (Zins, Nebenkosten; ohne Amortisation)");
    await expect(page.locator("#main .hint").filter({ hasText: "Wohnanteil zum Simulationsstart" })).toContainText("die Amortisation ist Vermögensbildung und zählt nicht dazu");
    await noErrors(page, errors);
  });

  test("AK2: changing the amortization leaves the housing share and the living costs without housing unchanged", async ({ page }) => {
    const errors = await open(page);
    const living = await page.evaluate(async () => {
      const share = async amort => {
        const s = JSON.parse(JSON.stringify(state));
        for (const p of s.realestate.properties) for (const t of p.mortgage?.tranches ?? []) for (const d of t.periods) d.amort = amort;
        const r = await Engine.call("preview", { plan: s });
        return r.result.records.slice(0, 12).map(x => x.plannedSpending);
      };
      return [await share(0), await share(3000)];
    });
    expect(living[0][0]).toBeCloseTo(9700 - 1200, 6);
    expect(living[0]).toEqual(living[1]);
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Living costs including housing and cash (#53, #54)", () => {
  test("#53/AK1, AK2: label including housing costs, notice with the housing share at the simulation start", async ({ page }) => {
    const errors = await open(page, "flows");
    await expect(page.locator("#main")).toContainText("Lebenshaltungskosten / Monat inklusive Miete bzw. Wohnkosten");
    // Example plan: housing costs at the simulation start 1'200 per month (#264: without the amortization of 500)
    await expect(page.locator("#main .hint").filter({ hasText: "Wohnanteil zum Simulationsstart" })).toContainText("1'200");
    await noErrors(page, errors);
  });

  test("#53/AK4: living costs smaller than the housing share are an error with both amounts", async ({ page }) => {
    const errors = await open(page);
    const issues = await page.evaluate(async () => {
      const s = JSON.parse(JSON.stringify(state));
      s.flows.spending = { mode: "const", value: 1000 };
      return (await Engine.call("validate", { plan: s })).map(i => msgText(i));
    });
    expect(issues).toContain("Die Lebenshaltungskosten (1'000) sind kleiner als die Wohnkosten zum Simulationsstart (1'200), die sie enthalten.");
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
    expect(text).toContain("Die Wohnkosten zum Simulationsstart (1200 pro Monat) sind zu jedem Wert addiert");
    expect(text).toContain("Cash wird neu bis zur Pensionierung (Bezug Pensionskasse) auf dem erfassten Betrag gehalten");
    await page.locator("#ask button[value='ok']").click();
    expect(await page.evaluate(() => [state.flows.spendingIncludesHousing, state.flows.spending.value])).toEqual([true, 6200]); // #264: share without amortization
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

test.describe("Earned income ends with the pension fund payout (#236)", () => {
  test("AK1: new plans and a new second person with the switch on", async ({ page }) => {
    const errors = await open(page);
    await page.locator('button[data-action="new"]').click();
    await page.waitForFunction(() => state.project.name === "" && state.wealth.stocks === 0);
    expect(await page.evaluate(() => state.flows.incomeEndsAtRetirement)).toBe(true);
    await nav(page, "Einnahmen & Ausgaben");
    const own = page.locator('input[data-path="flows.incomeEndsAtRetirement"]');
    await expect(own).toBeChecked();
    await expect(page.locator("label.check", { has: own })).toContainText("Erwerbseinkommen endet mit dem Bezug der Pensionskasse");
    await nav(page, "Haushalt & Projekt");
    await page.getByRole("button", { name: "+ Zweite Person" }).click();
    await nav(page, "Einnahmen & Ausgaben");
    await expect(page.locator('input[data-path="partner.incomeEndsAtRetirement"]')).toBeChecked();
    await noErrors(page, errors);
  });

  test("AK2, AK4, AK5: older plan opens with the switch off; switching changes the preview and is kept in the plan", async ({ page }) => {
    const errors = await open(page, "flows");
    const own = page.locator('input[data-path="flows.incomeEndsAtRetirement"]');
    await expect(own).not.toBeChecked(); // example plan without a value: off
    // income 10'000 per month also after the payout
    await page.evaluate(() => { state.flows.income = { mode: "const", value: 10000 }; renderSection(); });
    const warning = page.locator("#val li.warn", { hasText: "Das Erwerbseinkommen beträgt nach dem Bezug der Pensionskasse" });
    await expect(warning).toHaveCount(1);
    const endValue = () => page.evaluate(() => lastSim?.total.at(-1));
    await expect(kpiValue(page, "Vermögen am Ende")).not.toHaveText("–");
    await expect.poll(endValue).toBeGreaterThan(0);
    const before = await endValue();
    await own.check();
    expect(await page.evaluate(() => state.flows.incomeEndsAtRetirement)).toBe(true);
    await expect.poll(endValue).toBeLessThan(before);
    await expect(warning).toHaveCount(0); // AK5: the validation knows the switch
    await own.uncheck();
    expect(await page.evaluate(() => state.flows.incomeEndsAtRetirement)).toBe(false);
    await expect.poll(endValue).toBe(before);
    await noErrors(page, errors);
  });
});

test.describe("Tax tariffs with inflation (#66)", () => {
  test("AK4: option in the taxes section, off for existing plans, on for new plans, named in the overview", async ({ page }) => {
    const errors = await open(page, "taxes");
    const box = page.locator('input[data-path="taxes.indexed"]');
    await expect(box).toBeVisible();
    await expect(page.locator("label.check", { has: box })).toContainText("Tarife mit der Teuerung (Beträge der Tabellen in heutigen Preisen)");
    await expect(box).not.toBeChecked(); // example plan without a value: «nein»
    const row = () => page.evaluate(() => paramSections(state).flatMap(([, r]) => r).find(([k]) => k === "Tarife mit der Teuerung")?.[1]);
    expect(await row()).toBe("nein");
    // AK1: the preview recomputes with the shifted brackets
    const endValue = () => page.evaluate(() => lastSim?.total.at(-1));
    await expect(kpiValue(page, "Vermögen am Ende")).not.toHaveText("–");
    const before = await endValue();
    await box.check();
    expect(await page.evaluate(() => state.taxes.indexed)).toBe(true);
    expect(await row()).toBe("ja");
    await expect.poll(endValue).not.toBe(before);
    await box.uncheck();
    await expect.poll(endValue).toBe(before);
    // New plan: on (the unchecked change is discarded)
    await page.locator('button[data-action="new"]').click();
    await page.locator('#ask button[value="ok"]').click();
    await page.waitForFunction(() => state.project.name === "" && state.wealth.stocks === 0);
    expect(await page.evaluate(() => state.taxes.indexed)).toBe(true);
    await nav(page, "Steuern");
    await expect(box).toBeChecked();
    expect(await row()).toBe("ja");
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
      await derive(lastRun);
      const y = yearFlows(lastRun.single).find(a => a.y === "2036");
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

// ---------------------------------------------------------------------------------------------
test.describe("Never sell properties (#196)", () => {
  const box = page => page.locator('input[type=checkbox][data-path="realestate.neverSell"]');

  test("AK1: switch at the top of the tab, off by default, saved in the plan, older files open with off", async ({ page }) => {
    const errors = await open(page, "realestate");
    await expect(box(page)).toBeVisible();
    await expect(box(page)).not.toBeChecked();
    await expect(page.locator("#neverSellCard")).toContainText("Immobilien nie verkaufen");
    // At the top: before the card «Notverkauf» and before the properties
    const order = await page.evaluate(() => {
      const main = document.querySelector("#main");
      const cards = [...main.querySelectorAll(".card, .prop")];
      return cards.findIndex(c => c.id === "neverSellCard");
    });
    expect(order).toBe(0);
    expect(await page.evaluate(() => state.realestate.neverSell)).toBeFalsy();
    await box(page).check();
    expect(await page.evaluate(() => JSON.parse(JSON.stringify(state)).realestate.neverSell)).toBe(true);
    // An older file without the field opens with «aus».
    const plan = await page.evaluate(() => { const s = JSON.parse(JSON.stringify(state)); delete s.realestate.neverSell; return s; });
    const file = path.join(os.tmpdir(), `qa-never-sell-${process.pid}.vplan`);
    fs.writeFileSync(file, JSON.stringify(plan));
    await page.locator("#fileInput").setInputFiles(file);
    if (await page.locator("#ask").isVisible()) await page.locator("#ask button[value='ok']").click();
    await nav(page, "Immobilien");
    await expect(box(page)).not.toBeChecked();
    expect(await page.evaluate(() => state.realestate.neverSell)).toBeFalsy();
    fs.rmSync(file);
    await noErrors(page, errors);
  });

  test("AK2: switched on, the rest of the tab is grey and read-only; values stay and apply again", async ({ page }) => {
    const errors = await open(page, "realestate");
    const body = page.locator("#realestateBody");
    const before = await page.evaluate(() => JSON.stringify(state.realestate.properties));
    await expect(body).not.toHaveAttribute("disabled");
    expect(await body.locator("input:enabled, select:enabled, button:enabled").count()).toBeGreaterThan(10);
    await box(page).check();
    await expect(body).toHaveAttribute("disabled");
    // Mortgages and costs included: no field or button of the tab can be used any more
    expect(await body.locator("input:enabled, select:enabled, button:enabled").count()).toBe(0);
    expect(await body.locator("input, select, button").count()).toBeGreaterThan(10);
    await expect(body.locator('input[data-path$=".fixCosts"]').first()).toBeDisabled();
    await expect(body.locator('input[data-path*=".mortgage.tranches.0"]').first()).toBeDisabled();
    expect(Number(await body.evaluate(e => getComputedStyle(e).opacity))).toBeLessThan(1);
    // The switch itself stays usable
    await expect(box(page)).toBeEnabled();
    await box(page).uncheck();
    await expect(body).not.toHaveAttribute("disabled");
    expect(await body.locator("input:enabled, select:enabled, button:enabled").count()).toBeGreaterThan(10);
    expect(await page.evaluate(() => JSON.stringify(state.realestate.properties))).toBe(before);
    await noErrors(page, errors);
  });

  test("AK6: the validation names the sales and purchases after sale that are not carried out", async ({ page }) => {
    const errors = await open(page, "realestate");
    await box(page).check();
    await expect(page.locator("body")).toContainText("«Immobilien nie verkaufen» ist eingeschaltet – der geplante Verkauf wird nicht ausgeführt.");
    await noErrors(page, errors);
  });
});

test.describe("Earned income per year as in the tax return (#232)", () => {
  test("#232/AK1–AK5: AK6 input 120'000 per year shows 10'000 per month and stores 10'000 per month", async ({ page }) => {
    const errors = await open(page, "flows");
    await page.evaluate(() => { state.flows.income = { mode: "const", value: 9000 }; renderSection(); });
    const own = field(page, "flows.income.value");
    await expect(own).toHaveValue("108'000");
    // AK1: name and hint as in the tax return
    const head = page.locator(".series", { has: own }).locator(".shead");
    await expect(head.locator(".t")).toHaveText(/^Einkünfte aus Erwerb netto \/ Jahrⓘ?$/); // ⓘ: help button
    await expect(page.locator(".series", { has: own }).locator(".shint")).toHaveText("wie Nettolohn im Lohnausweis (Ziffer 11), inkl. 13. Monatslohn und Boni; Selbständige: Einkommen aus selbständiger Erwerbstätigkeit");
    // AK6/AK2/AK4: input per year, display per month, stored per month
    await own.fill("120000");
    await own.blur();
    await expect(own).toHaveValue("120'000");
    await expect(page.locator('[data-permonth="flows.income.value"]')).toHaveText("= 10'000 / Monat");
    expect(await page.evaluate(() => state.flows.income)).toEqual({ mode: "const", value: 10000 });
    // AK4: an existing plan shows the monthly amount × 12
    await page.evaluate(() => { state.flows.income = { mode: "const", value: 8333 }; renderSection(); });
    await expect(own).toHaveValue("99'996");
    await expect(page.locator('[data-permonth="flows.income.value"]')).toHaveText("= 8'333 / Monat");
    // AK3: every row of the time series and the second person
    await page.evaluate(() => { ACTIONS["partner-add"](); state.flows.income = { mode: "timed", rows: [{ from: "start", value: 10000 }, { from: "retire", value: 2000 }] };
      state.partner.income = { mode: "const", value: 5000 }; goto("flows"); });
    await expect(field(page, "flows.income.rows.0.value")).toHaveValue("120'000");
    await expect(field(page, "flows.income.rows.1.value")).toHaveValue("24'000");
    await expect(page.locator('[data-permonth="flows.income.rows.1.value"]')).toHaveText("= 2'000 / Monat");
    await expect(field(page, "partner.income.value")).toHaveValue("60'000");
    await expect(page.locator('[data-permonth="partner.income.value"]')).toHaveText("= 5'000 / Monat");
    await field(page, "partner.income.value").fill("90'000");
    await field(page, "partner.income.value").blur();
    expect(await page.evaluate(() => state.partner.income.value)).toBe(7500);
    // AK5: parameters of a run per year
    const params = await page.evaluate(() => Object.fromEntries(paramSections(state).find(([t]) => t === "Einnahmen und Ausgaben")[1]));
    expect(params["Einkommen pro Jahr"]).toContain("120'000");
    expect(params["Einkommen pro Jahr"]).toContain("24'000");
    expect(params["Einkommen Person 2 pro Jahr"]).toBe("90'000 CHF");
    // AK5: the message names the amount per year (for two persons with the name in front)
    await page.evaluate(() => { state.flows.income = { mode: "const", value: 2500 }; goto("flows"); });
    const warn = page.locator("#val li.warn", { hasText: "Das Erwerbseinkommen beträgt nach dem Bezug der Pensionskasse" });
    await expect(warn.filter({ hasText: "noch 30'000 pro Jahr" })).toHaveCount(1);
    // #247: a new second person's income ends with the pension fund payout (#236), so there is no message
    await expect(warn.filter({ hasText: "Person 2: " })).toHaveCount(0);
    await page.evaluate(() => { state.partner.incomeEndsAtRetirement = false; goto("flows"); });
    await expect(warn.filter({ hasText: "Person 2: " })).toContainText("noch 90'000 pro Jahr");
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Affordability in the settings (#233)", () => {
  const dlg = page => page.locator("#settings");
  const openSettings = async page => { await page.keyboard.press(`${MOD}+,`); await expect(dlg(page)).toBeVisible(); };
  /** Mortgage of the purchase of the flat «nach Verkauf» in the events of a result (German text of the event). */
  const purchaseMortgage = events => {
    const e = events.find(x => /^Kauf Eigentumswohnung/.test(x.label));
    return e ? parseNum(/Hypothek ([\d'’]+)/.exec(e.label)[1]) : null;
  };
  /** Example plan: the flat wants a mortgage of 90 % and the income easily carries it (only the loan-to-value limits it). */
  const highMortgage = page => page.evaluate(() => {
    const flat = state.realestate.properties.find(p => p.status === "planned");
    flat.mortgage.tranches[0].amount = 900000; flat.inflationValue = false; // price 1'000'000 at the purchase
    state.flows.income = { mode: "const", value: 50000 };
    renderSection();
  });
  const previewMortgage = async page => { await previewIdle(page); return purchaseMortgage(await page.evaluate(() => lastSim.events)); };
  const simulationMortgage = page => page.evaluate(async () => {
    const plan = JSON.parse(JSON.stringify(state)); plan.returns.source = "fixed"; plan.scenarios.sweeps = [];
    return (await Engine.call("run_simulation", { plan, path: null })).single.events.map(e => ({ label: msgText(e) }));
  }).then(purchaseMortgage);

  test("AK1: no affordability card in the real estate section, a note with the values and a link to the settings", async ({ page }) => {
    const errors = await open(page);
    await nav(page, "Immobilien");
    await expect(page.locator('#main input[data-path^="realestate.affordability"]')).toHaveCount(0);
    await expect(page.locator("#main h3", { hasText: "Tragbarkeit" })).toHaveCount(0);
    await expect(page.locator("#affordNote")).toHaveText("Tragbarkeit: 33.3 % des Einkommens, Zins 5 %, Nebenkosten 1 %, Belehnung 80 % (ab der Pensionierung 65 %) – änderbar in den Einstellungen.");
    await page.locator("#affordNote a").click();
    await expect(dlg(page)).toBeVisible();
    await expect(page.locator("#set-afford-maxLoanToValue")).toBeVisible();
    await noErrors(page, errors);
  });

  test("AK2: four fields with the defaults, invalid input rejected, the default values button resets them", async ({ page }) => {
    const errors = await open(page);
    await openSettings(page);
    for (const t of ["Höchstens vom Einkommen", "Kalkulatorischer Zins", "Nebenkosten p.a. (vom Preis)", "Höchste Belehnung"]) await expect(dlg(page)).toContainText(t);
    for (const [k, v] of [["sustainability", "33.333"], ["imputedRate", "5"], ["fixCosts", "1"], ["maxLoanToValue", "80"]])
      await expect(page.locator(`#set-afford-${k}`)).toHaveValue(v);
    await expect(page.locator("#settings label.field", { has: page.locator("#set-afford-sustainability") })).toContainText("Standard 33.3 %");
    // as before in the editor: a number is required
    await page.locator("#set-afford-imputedRate").fill("abc");
    await page.locator("#settingsSave").click();
    await expect(page.locator("#settingsErr")).toContainText("Tragbarkeit, Kalkulatorischer Zins: Bitte eine Zahl eingeben.");
    await expect(dlg(page)).toBeVisible();
    expect(await page.evaluate(() => affordability().imputedRate)).toBe(0.05);
    await page.locator("#set-afford-imputedRate").fill("4.5");
    await page.locator("#set-afford-maxLoanToValue").fill("70");
    await page.locator("#settingsSave").click();
    await expect(dlg(page)).toBeHidden();
    await page.reload();
    await page.waitForSelector("html[data-ready='1']");
    expect(await page.evaluate(() => affordability())).toMatchObject({ imputedRate: 0.045, maxLoanToValue: 0.7 });
    expect(await prefs(page)).toMatchObject({ theme: "system", limits: { ahvMin: 63 } }); // the other settings stay
    await openSettings(page);
    await page.locator("#settingsDefaults").click();
    await page.locator("#ask button[value='ok']").click();
    await expect(page.locator("#set-afford-maxLoanToValue")).toHaveValue("80");
    expect(await page.evaluate(() => affordability())).toMatchObject({ sustainability: 1 / 3, imputedRate: 0.05, fixCosts: 0.01, maxLoanToValue: 0.8 });
    await noErrors(page, errors);
  });

  test("AK3: loan-to-value 70 % instead of 80 % gives a smaller mortgage for the purchase after sale in preview and simulation", async ({ page }) => {
    const errors = await open(page);
    // #266: the purchase is after retirement; the lower limit after retirement is not the subject here.
    await page.evaluate(() => Pref.set("affordability", { ...affordability(), maxLoanToValueRetired: 1 }));
    await highMortgage(page);
    expect(await previewMortgage(page)).toBe(800000);
    expect(await simulationMortgage(page)).toBe(800000);
    await openSettings(page);
    await page.locator("#set-afford-maxLoanToValue").fill("70");
    await page.locator("#settingsSave").click();
    await expect(dlg(page)).toBeHidden();
    expect(await previewMortgage(page)).toBe(700000);
    expect(await simulationMortgage(page)).toBe(700000);
    await nav(page, "Immobilien");
    await expect(page.locator("#affordNote")).toContainText("Belehnung 70 %");
    await noErrors(page, errors);
  });

  test("AK4: the plan file holds no affordability; an older file with other values sets the settings and says so", async ({ page }) => {
    const errors = await open(page);
    expect(await page.evaluate(() => "affordability" in state.realestate)).toBe(false);
    const [dl] = await Promise.all([page.waitForEvent("download"), page.evaluate(() => download())]);
    const saved = JSON.parse(fs.readFileSync(await dl.path(), "utf8"));
    expect("affordability" in saved.realestate).toBe(false);
    // older file with its own values
    const older = JSON.parse(JSON.stringify(saved));
    older.realestate.affordability = { sustainability: 0.3, imputedRate: 0.045, fixCosts: 0.01, wealthContribution: 0.03, maxLoanToValue: 0.75 };
    const file = path.join(os.tmpdir(), `qa-233-${process.pid}.vplan`);
    fs.writeFileSync(file, JSON.stringify(older));
    await page.locator("#fileInput").setInputFiles(file);
    const ask = page.locator("#ask");
    await expect(ask).toBeVisible();
    await expect(ask).toContainText("Die Tragbarkeit aus der Datei gilt jetzt für alle Planungen (Einstellungen): 30 % des Einkommens, Zins 4.5 %, Nebenkosten 1 %, Belehnung 75 % (ab der Pensionierung 65 %); bisher 33.3 % des Einkommens, Zins 5 %, Nebenkosten 1 %, Belehnung 80 % (ab der Pensionierung 65 %).");
    await ask.locator("button[value='ok']").click();
    expect(await page.evaluate(() => affordability())).toMatchObject({ sustainability: 0.3, imputedRate: 0.045, fixCosts: 0.01, maxLoanToValue: 0.75 });
    expect(await page.evaluate(() => "affordability" in state.realestate)).toBe(false);
    // the same values again: no note
    await page.locator("#fileInput").setInputFiles(file);
    await expect(page.locator("#toast")).toContainText("geöffnet");
    await expect(ask).not.toBeVisible();
    fs.rmSync(file);
    await noErrors(page, errors);
  });

  test("AK5: a saved result and the comparison show the values the plan was calculated with", async ({ page }) => {
    const errors = await open(page);
    await openSettings(page);
    await page.locator("#set-afford-maxLoanToValue").fill("70");
    await page.locator("#settingsSave").click();
    await page.evaluate(() => { state.returns.source = "fixed"; state.scenarios.sweeps = []; });
    await page.evaluate(() => runSimulation("overview", { noConfirm: true }));
    await expect(page.locator("html")).toHaveAttribute("data-saved", "1");
    const [m] = await page.evaluate(() => Runs.list());
    const plan = await page.evaluate(id => Runs.get(id).then(d => d.plan), m.id);
    expect(plan.realestate.affordability.maxLoanToValue).toBe(0.7);
    // later change of the settings: the saved result keeps 70 %
    await page.evaluate(() => { Pref.set("affordability", { ...AFFORD_DEFAULTS }); });
    const rows = await page.evaluate(p => paramSections(p, true).flatMap(s => s[1]), plan);
    expect(rows).toContainEqual(["Tragbarkeit: Höchste Belehnung", "70 %"]);
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Course forms linear, table and factor (#217)", () => {
  test("AK7: the editor shows the new forms read-only with a note and the mini chart, computes them and replaces them on request", async ({ page }) => {
    const errors = await open(page, "wealth");
    await page.evaluate(() => {
      state.wealth.stockShare = { mode: "linear", start: { age: 55, value: 0.9 }, end: { age: 65, value: 0.5 } };
      state.wealth.reserveMonths = { mode: "table", interpolation: "step", points: [{ at: "start", value: 6 }, { age: 80, value: 12 }] };
      state.flows.spending = { mode: "factor", base: 9000, factor: { mode: "table", interpolation: "step", points: [{ at: "start", value: 1 }, { age: 80, value: 0.8 }] } };
      markDirty(); renderSection();
    });
    const share = page.locator('.series:has(svg[data-mini="wealth.stockShare"])');
    // #243: the editor edits the forms instead of a read-only note
    await expect(share.locator('.rows[data-form="linear"]')).toBeVisible();
    await expect(share.locator('svg[data-mini="wealth.stockShare"] path').first()).toBeAttached();
    await expect(share.locator('.shead > .seg button.on')).toHaveText("Gerade");
    // The value in the course: 70 % at 60 like the engine (#217/AK5), only for display.
    expect(await page.evaluate(() => {
      const b = ym2i(state.household.persons[0].birth);
      return Math.round(seriesAt(state.wealth.stockShare, b + 60 * 12) * 1000) / 1000;
    })).toBe(0.7);
    await previewIdle(page);
    await expect(page.locator("#chart path").first()).toBeAttached();
    await nav(page, "Einnahmen & Ausgaben");
    const spending = page.locator('.series:has(svg[data-mini="flows.spending"])');
    await expect(spending.locator('.rows[data-form="factor"]')).toBeVisible();
    // «Constant» replaces the course with its value at the simulation start.
    await spending.locator('button[data-action="series-mode"][data-path="flows.spending"][data-mode="const"]').click();
    expect(await page.evaluate(() => state.flows.spending)).toEqual({ mode: "const", value: 9000 });
    await nav(page, "Entnahmestrategie");
    await page.evaluate(() => { state.withdrawal.strategy = "standard"; renderSection(); });
    await expect(page.locator('.series:has(svg[data-mini="wealth.reserveMonths"]) .rows[data-form="table"]')).toBeVisible();
    await previewIdle(page);
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Editor for the course forms linear, table and factor (#243)", () => {
  const box = (page, p) => page.locator(`.series:has(svg[data-mini="${p}"])`);
  const form = (page, p, mode) => box(page, p).locator(`button[data-action="series-mode"][data-path="${p}"][data-mode="${mode}"]`);
  /** Types a value into a field and leaves it (the plan takes it over like after Tab). */
  const type = async (page, p, v) => { const f = field(page, p); await f.fill(v); await f.press("Tab"); };
  const when = (page, p) => page.locator(`select[data-path="${p}"][data-type="point"]`);
  const at = (page, p, age) => page.evaluate(([p, age]) => seriesAt(getP(p), ym2i(state.household.persons[0].birth) + Math.round(age * 12)), [p, age]);

  test("AK1: every course offers the five forms; switching keeps the value at the simulation start", async ({ page }) => {
    const errors = await open(page);
    // In every section, every course has the selector with the five forms
    let seen = 0;
    for (const id of await page.evaluate(() => SECTIONS.map(s => s.id))) {
      await page.evaluate(id => goto(id), id);
      for (const seg of await page.locator("#main .series > .shead > .seg").all()) {
        await expect(seg.locator("button")).toHaveText(["Konstant", "Treppe", "Gerade", "Tabelle", "Faktor"]);
        seen++;
      }
    }
    expect(seen).toBeGreaterThanOrEqual(5);
    await page.evaluate(() => goto("flows"));
    const p = "flows.spending", first = await page.evaluate(p => seriesAt(getP(p), startM()), p);
    for (const mode of ["timed", "linear", "table", "factor", "const"]) {
      await form(page, p, mode).click();
      await expect(form(page, p, mode)).toHaveClass(/\bon\b/);
      expect(await page.evaluate(p => getP(p).mode, p)).toBe(mode);
      expect(await page.evaluate(p => seriesAt(getP(p), startM()), p)).toBeCloseTo(first, 6);
    }
    await previewIdle(page);
    await noErrors(page, errors);
  });

  test("AK2: straight line with value a, point t1, value b, point t2 as age, date or milestone", async ({ page }) => {
    const errors = await open(page, "wealth");
    const p = "wealth.stockShare", pid = await page.evaluate(() => state.household.persons[0].id);
    await form(page, p, "linear").click();
    await expect(box(page, p).locator(".row")).toHaveCount(2);
    await when(page, `${p}.start`).selectOption(`age:${pid}`);
    await type(page, `${p}.start.age`, "55");
    await type(page, `${p}.start.value`, "90");
    await when(page, `${p}.end`).selectOption(`age:${pid}`);
    await type(page, `${p}.end.age`, "65");
    await type(page, `${p}.end.value`, "50");
    expect(await page.evaluate(p => getP(p), p)).toEqual({ mode: "linear", start: { age: 55, value: 0.9 }, end: { age: 65, value: 0.5 } });
    expect(await at(page, p, 60)).toBeCloseTo(0.7, 6);
    expect(await at(page, p, 50)).toBeCloseTo(0.9, 6);
    // Milestone: the end at the pension fund payout; date: the start as a fixed month, same point in time
    await when(page, `${p}.end`).selectOption("retire");
    expect(await page.evaluate(p => getP(p).end, p)).toEqual({ at: "retire", value: 0.5 });
    const m55 = await page.evaluate(() => i2ym(ym2i(state.household.persons[0].birth) + 55 * 12));
    await when(page, `${p}.start`).selectOption("__date");
    expect(await page.evaluate(p => getP(p).start, p)).toEqual({ at: m55, value: 0.9 });
    await expect(field(page, `${p}.start.at`)).toBeVisible();
    await previewIdle(page);
    await expect(page.locator("#main .ferr")).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("AK3: table with support points by age or date, add, change and delete, linear or steps", async ({ page }) => {
    const errors = await open(page, "flows");
    const p = "flows.spending", pid = await page.evaluate(() => state.household.persons[0].id);
    await form(page, p, "const").click(); // the example plan has steps; a table would take over their rows
    await form(page, p, "table").click();
    const rows = box(page, p).locator(".row");
    await expect(rows).toHaveCount(1);
    await expect(rows.first().locator("button")).toHaveCount(0); // the last point cannot be deleted
    await type(page, `${p}.points.0.value`, "8000");
    await box(page, p).locator('button[data-action="point-add"]').click();
    await box(page, p).locator('button[data-action="point-add"]').click();
    await expect(rows).toHaveCount(3);
    await when(page, `${p}.points.1`).selectOption(`age:${pid}`);
    await type(page, `${p}.points.1.age`, "75");
    await type(page, `${p}.points.1.value`, "6000");
    await when(page, `${p}.points.2`).selectOption("__date");
    const d = await page.evaluate(() => i2ym(ym2i(state.household.persons[0].birth) + 85 * 12));
    await type(page, `${p}.points.2.at`, `${d.slice(5)}.${d.slice(0, 4)}`);
    await type(page, `${p}.points.2.value`, "5000");
    expect(await page.evaluate(p => getP(p).points.slice(1), p)).toEqual([{ age: 75, value: 6000 }, { at: d, value: 5000 }]);
    // Linear in between by default, steps on request
    expect(await at(page, p, 80)).toBeCloseTo(5500, 6);
    await box(page, p).locator('button[data-value="step"]').click();
    expect(await page.evaluate(p => getP(p).interpolation, p)).toBe("step");
    expect(await at(page, p, 80)).toBeCloseTo(6000, 6);
    // Delete the middle point
    await rows.nth(1).locator('button[data-action="row-del"]').click();
    expect(await page.evaluate(p => getP(p).points.map(x => x.value), p)).toEqual([8000, 5000]);
    await previewIdle(page);
    await noErrors(page, errors);
  });

  test("AK4: factor with base value and course of factors", async ({ page }) => {
    const errors = await open(page, "flows");
    const p = "flows.spending", pid = await page.evaluate(() => state.household.persons[0].id);
    await form(page, p, "const").click();
    await form(page, p, "factor").click();
    await type(page, `${p}.base`, "6000");
    // The course of factors has its own selector (without a factor in the factor) and fields in percent
    const sub = box(page, p).locator(".shead.sub .seg button");
    await expect(sub).toHaveText(["Konstant", "Treppe", "Gerade", "Tabelle"]);
    await box(page, p).locator(`button[data-path="${p}.factor"][data-mode="table"]`).click();
    await box(page, p).locator(`button[data-action="point-add"][data-path="${p}.factor.points"]`).click();
    await when(page, `${p}.factor.points.1`).selectOption(`age:${pid}`);
    await type(page, `${p}.factor.points.1.age`, "80");
    await type(page, `${p}.factor.points.1.value`, "80");
    await box(page, p).locator(`button[data-path="${p}.factor.interpolation"][data-value="step"]`).click();
    expect(await page.evaluate(p => getP(p), p)).toEqual({ mode: "factor", base: 6000,
      factor: { mode: "table", interpolation: "step", points: [{ at: "start", value: 1 }, { age: 80, value: 0.8 }] } });
    expect(await at(page, p, 79)).toBeCloseTo(6000, 6);
    expect(await at(page, p, 85)).toBeCloseTo(4800, 6);
    await previewIdle(page);
    await expect(page.locator("#main .ferr")).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("AK5: the mini chart shows the course and follows every change", async ({ page }) => {
    const errors = await open(page, "wealth");
    const p = "wealth.stockShare", line = box(page, p).locator(`svg[data-mini="${p}"] path`).nth(1);
    await form(page, p, "linear").click();
    const flat = await line.getAttribute("d");
    await type(page, `${p}.end.value`, "20");
    await expect(line).not.toHaveAttribute("d", flat);
    // A straight line is drawn month by month, not as a single step
    expect((await line.getAttribute("d")).split("V").length).toBeGreaterThan(24);
    await noErrors(page, errors);
  });

  test("AK6: a plan with the new forms opens, can be edited and saved without losing values", async ({ page }) => {
    const errors = await open(page);
    const plan = await page.evaluate(() => {
      const s = JSON.parse(JSON.stringify(state)), pid = s.household.persons[1]?.id;
      s.wealth.stockShare = { mode: "linear", start: { age: 55, value: 0.9 }, end: { at: "2040-01", value: 0.5 } };
      s.wealth.reserveMonths = { mode: "table", interpolation: "step", points: [{ at: "start", value: 6 }, { age: 80, value: 12 }, { at: "retire", value: 9 }] };
      s.flows.spending = { mode: "factor", base: 9000, factor: { mode: "table", points: [{ at: "start", value: 1 }, { age: 80, ...(pid ? { person: pid } : {}), value: 0.8 }] } };
      s.pension.pk.interest = { mode: "factor", base: 0.02, factor: 1 };
      return s;
    });
    const file = path.join(os.tmpdir(), `issue243-${process.pid}.vplan`);
    fs.writeFileSync(file, JSON.stringify(plan));
    await page.locator("#fileInput").setInputFiles(file);
    await expect.poll(() => page.evaluate(() => state.flows.spending.mode)).toBe("factor");
    // Open every section once (the editor must not change the forms) and edit one value
    for (const id of await page.evaluate(() => SECTIONS.map(s => s.id))) await page.evaluate(id => goto(id), id);
    await page.evaluate(() => goto("flows"));
    await type(page, "flows.spending.base", "9500");
    await previewIdle(page);
    const download = page.waitForEvent("download");
    await page.evaluate(() => command("save"));
    const saved = JSON.parse(fs.readFileSync(await (await download).path(), "utf8"));
    const expected = JSON.parse(JSON.stringify(plan));
    expected.flows.spending.base = 9500;
    expected.pension.pk.interest.factor = { mode: "const", value: 1 }; // a number as factor becomes a constant (same value)
    for (const k of ["stockShare", "reserveMonths"]) expect(saved.wealth[k]).toEqual(expected.wealth[k]);
    expect(saved.flows.spending).toEqual(expected.flows.spending);
    expect(saved.pension.pk.interest).toEqual(expected.pension.pk.interest);
    // Reopen the saved file: the same values in the editor
    const again = path.join(os.tmpdir(), `issue243-${process.pid}-2.vplan`);
    fs.writeFileSync(again, JSON.stringify(saved));
    await page.evaluate(() => { dirty = false; });
    await page.locator("#fileInput").setInputFiles(again);
    await expect.poll(() => page.evaluate(() => state.flows.spending.base)).toBe(9500);
    expect(await page.evaluate(() => state.wealth.reserveMonths)).toEqual(expected.wealth.reserveMonths);
    await page.evaluate(() => goto("withdrawal"));
    fs.rmSync(file); fs.rmSync(again);
    await previewIdle(page);
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Characteristic curves (#218)", () => {
  test("AK2: the editor shows a curve of the stock share read-only with a note, computes it and replaces it on request", async ({ page }) => {
    const errors = await open(page, "wealth");
    await page.evaluate(() => {
      state.wealth.stockShare = { mode: "curve", input: "withdrawalRate", points: [[0.02, 0.8], [0.06, 0.4]], eval: "yearly" };
      markDirty(); renderSection();
    });
    const share = page.locator('.series:has(.rows[data-form="curve"])');
    await expect(share.locator(".shint")).toContainText("Kennlinie über «Entnahmerate», ausgewertet einmal pro Jahr");
    await expect(share.locator(".row")).toHaveCount(2);
    await expect(share.locator(".row").first()).toContainText("2 %");
    await expect(share.locator(".row").first()).toContainText("80 %");
    // No course over time: no mini chart, no form marked.
    await expect(share.locator("svg[data-mini]")).toHaveCount(0);
    await expect(share.locator(".shead > .seg button.on")).toHaveCount(0);
    // The plan computes with the curve (no check error).
    await previewIdle(page);
    await expect(page.locator("#chart path").first()).toBeAttached();
    // «Constant» replaces the curve with the value of its first support point.
    await share.locator('button[data-action="series-mode"][data-path="wealth.stockShare"][data-mode="const"]').click();
    expect(await page.evaluate(() => state.wealth.stockShare)).toEqual({ mode: "const", value: 0.8 });
    await previewIdle(page);
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Married with one person (#250)", () => {
  test("AK1: «Verheiratet» also for one person, with a hint; older plans are taken over as unmarried", async ({ page }) => {
    const errors = await open(page);
    await nav(page, "Haushalt & Projekt");
    expect(await page.evaluate(() => state.household.persons.length)).toBe(1);
    const box = page.locator('#main input[data-path="household.married"]');
    await expect(box).toBeVisible();
    await expect(box).not.toBeChecked();
    await expect(page.locator("#main")).toContainText("Verheiratet mit einer Person: Steuern mit den Tarifen und Abzügen für Verheiratete");
    // no AHV cap with one person
    await expect(page.locator('#main input[data-path="household.ahvCoupleCap"]')).toHaveCount(0);
    await box.check();
    expect(await page.evaluate(() => state.household.married)).toBe(true);
    // AK4: an older plan with one person and `married` becomes unmarried with a note
    const r = await page.evaluate(async () => {
      const p = structuredClone(state);
      p.household.married = true; delete p.household.marriedNoted;
      const u = await Engine.call("upgrade_plan", { plan: p });
      return { married: u.plan.household.married, noted: u.plan.household.marriedNoted, notes: u.notes.map(msgText) };
    });
    expect(r.married).toBe(false);
    expect(r.noted).toBe(true);
    expect(r.notes.some(n => n.startsWith("«Verheiratet» gilt neu auch für eine Person allein"))).toBe(true);
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Wealth drawdown and loan-to-value after retirement in the affordability (#266)", () => {
  const dlg = page => page.locator("#settings");
  const openSettings = async page => { await page.keyboard.press(`${MOD}+,`); await expect(dlg(page)).toBeVisible(); };
  test("AK1: switch, rate and loan-to-value after retirement with defaults; saved values and older settings", async ({ page }) => {
    const errors = await open(page);
    // Older settings without the new values get the defaults.
    await page.evaluate(() => Pref.set("affordability", { sustainability: 0.3, imputedRate: 0.05, fixCosts: 0.01, maxLoanToValue: 0.8 }));
    expect(await page.evaluate(() => affordability())).toMatchObject({ wealthDrawdown: false, wealthDrawdownRate: 0.03, maxLoanToValueRetired: 0.65 });
    await openSettings(page);
    for (const t of ["Vermögensverzehr in der Tragbarkeit anrechnen", "Vermögensverzehr pro Jahr", "Höchste Belehnung ab der Pensionierung"])
      await expect(dlg(page)).toContainText(t);
    await expect(page.locator("#set-afford-wealthDrawdown")).not.toBeChecked();
    await expect(page.locator("#set-afford-wealthDrawdownRate")).toHaveValue("3");
    await expect(page.locator("#set-afford-maxLoanToValueRetired")).toHaveValue("65");
    await expect(page.locator("#settings label.field", { has: page.locator("#set-afford-maxLoanToValueRetired") })).toContainText("Standard 65 %");
    await page.locator("#set-afford-wealthDrawdown").check();
    await page.locator("#set-afford-wealthDrawdownRate").fill("4");
    await page.locator("#set-afford-maxLoanToValueRetired").fill("60");
    await page.locator("#settingsSave").click();
    await expect(dlg(page)).toBeHidden();
    await page.reload();
    await page.waitForSelector("html[data-ready='1']");
    expect(await page.evaluate(() => affordability())).toMatchObject({ wealthDrawdown: true, wealthDrawdownRate: 0.04, maxLoanToValueRetired: 0.6 });
    // The note in the real estate section names the values (#266/AK7).
    await nav(page, "Immobilien");
    await expect(page.locator("#affordNote")).toContainText("Belehnung 80 % (ab der Pensionierung 60 %), Vermögensverzehr 4 % pro Jahr");
    // Default values: switch off again.
    await openSettings(page);
    await page.locator("#settingsDefaults").click();
    await page.locator("#ask button[value='ok']").click();
    await expect(page.locator("#set-afford-wealthDrawdown")).not.toBeChecked();
    expect(await page.evaluate(() => affordability())).toMatchObject({ wealthDrawdown: false, wealthDrawdownRate: 0.03, maxLoanToValueRetired: 0.65 });
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("New plan from the engine (#269)", () => {
  // The construction of the user interface before #269 (`blankPlan` in app.js), kept here as the reference.
  const oldBlankPlan = (base, now) => {
    const i2ym = i => { const y = Math.floor(i / 12), mo = i - y * 12 + 1; return `${y}-${String(mo).padStart(2, "0")}`; };
    const p = JSON.parse(JSON.stringify(base)), cur = now.getFullYear() * 12 + now.getMonth(), start = i2ym(cur);
    const zero = { mode: "const", value: 0 };
    p.project = { ...p.project, name: "" };
    p.household = { persons: [{ id: "A", name: "", birth: i2ym(cur - 60 * 12) }] };
    p.milestones = [
      { id: "start", kind: "date", date: start, fixed: true },
      { id: "retire", kind: "age", person: "A", years: 65 },
      { id: "ahv", kind: "age", person: "A", years: 65 },
      { id: "end", kind: "age", person: "A", years: 95, fixed: true },
    ];
    delete p.partner;
    p.pillar3a = [];
    p.pension = { ...p.pension, pk: { ...p.pension.pk, capital: 0, at: "retire", lumpsum: 0, contribution: { ...zero }, interest: { mode: "const", value: 0.02 } },
      ahv: { ...p.pension.ahv, monthly: 0, at: "ahv" } };
    p.flows = { income: { ...zero }, incomeIndexed: true, incomeEndsAtRetirement: true, spending: { ...zero }, extras: [], spendingIncludesHousing: true, amortExcluded: true };
    p.taxes = { ...p.taxes, indexed: true };
    p.taxes.deductions = [{ from: "start", kind: "own", amount: 0, rate: 0 }, { from: "retire", kind: "own", amount: 0, rate: 0 }];
    p.taxes.deductPillar3a = true;
    p.wealth = { ...p.wealth, stocks: 0, bonds: 0, cash: 0 };
    p.realestate = { ...p.realestate, properties: [] };
    p.scenarios = { ...p.scenarios, sweeps: [] };
    delete p.optimization;
    return p;
  };
  const sorted = v => Array.isArray(v) ? v.map(sorted) : v && typeof v === "object"
    ? Object.fromEntries(Object.keys(v).sort().map(k => [k, sorted(v[k])])) : v;

  test("AK1: «Neu» gives the same plan as the old construction of the user interface", async ({ page }) => {
    const errors = await open(page);
    const example = await (await page.request.post("/api/example_plan", { data: {} })).json();
    // engine construction for several months (January, December, end of a year) equals the old one
    for (const [y, m] of [[2030, 0], [2026, 10], [1999, 11]]) {
      const now = new Date(y, m, 15);
      const got = await page.evaluate(async t => blankPlan(new Date(t)), now.getTime());
      expect(sorted(got)).toEqual(sorted(oldBlankPlan(example, now)));
    }
    // the action «Neu» takes over the plan of the engine with the current month
    await page.evaluate(() => { const orig = adopt; adopt = (p, ...r) => { window.adopted = JSON.parse(JSON.stringify(p)); return orig(p, ...r); }; });
    await page.locator('button[data-action="new"]').click();
    await page.waitForFunction(() => window.adopted && state.wealth.stocks === 0);
    const adopted = await page.evaluate(() => window.adopted);
    expect(sorted(adopted)).toEqual(sorted(oldBlankPlan(example, new Date())));
    await noErrors(page, errors);
  });
});
