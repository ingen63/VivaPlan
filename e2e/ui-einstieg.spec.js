// User interface tests of VivaPlan in the browser: welcome, first steps, result in words, liability. Every test starts with the example plan.
const { test, expect } = require("@playwright/test");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { MOD, SECTIONS, open, noErrors, field, previewIdle, nav, kpiValue, resultKpi, parseNum, setOpt, msRow, welcome, steps, helpTitle, prefs } = require("./helpers");

// ---------------------------------------------------------------------------------------------
// #70 welcome dialog, #71 first steps, #72 result in words

test.describe("#70 Welcome dialog", () => {
  test("AK1: appears on the first start (test access) with three choices, otherwise not", async ({ page }) => {
    let errors = await open(page);
    await expect(welcome(page)).not.toHaveAttribute("open", "");
    await noErrors(page, errors);
    await page.goto("about:blank");
    errors = await open(page, "welcome");
    await expect(welcome(page)).toHaveAttribute("open", "");
    await expect(welcome(page).locator("h2")).toHaveText("Willkommen bei VivaPlan");
    await expect(welcome(page).locator(".wchoices button b")).toHaveText(["Beispiel erkunden", "Eigene Planung beginnen", "Leere Planung"]);
    // AK6: The first start is remembered immediately.
    expect((await prefs(page)).welcomeShown).toBe(true);
    await noErrors(page, errors);
  });

  test("AK2: explore example loads the example plan and opens the help page on what VivaPlan can do", async ({ page }) => {
    const errors = await open(page, "welcome");
    await welcome(page).getByRole("button", { name: /Beispiel erkunden/ }).click();
    await expect(welcome(page)).not.toHaveAttribute("open", "");
    await expect(helpTitle(page)).toHaveText("Was VivaPlan kann");
    expect(await page.evaluate(() => state.project.name)).toBe("Beispielplanung");
    await noErrors(page, errors);
  });

  test("AK3: start own plan opens the household and project section and shows the checklist", async ({ page }) => {
    // Window height like the desktop app: the checklist is below the preview and key figures and must be scrolled into view.
    await page.setViewportSize({ width: 1440, height: 900 });
    const errors = await open(page, "welcome");
    await welcome(page).getByRole("button", { name: /Eigene Planung beginnen/ }).click();
    await expect(page.locator("#nav a.active")).toContainText("Haushalt & Projekt");
    await expect(steps(page)).toBeVisible();
    await expect(steps(page)).toBeInViewport({ ratio: 0.95 });
    await expect(steps(page).locator("summary")).toBeFocused();
    await expect(page.locator("#stepsProg")).toHaveText("0 von 8");
    expect(await page.evaluate(() => state.project.name)).toBe("Beispielplanung");
    await noErrors(page, errors);
  });

  test("AK4: empty plan acts like File → New", async ({ page }) => {
    const errors = await open(page, "welcome");
    await welcome(page).getByRole("button", { name: /Leere Planung/ }).click();
    // #95: «Neu» takes over the empty plan only after two calls to the engine. «Haushalt & Projekt» is already
    // active before; therefore wait for the plan itself instead of only for the navigation.
    // Features of the empty plan from File → New: no name, no amounts, no real estate
    await expect.poll(() => page.evaluate(() => [state.project.name, state.household.persons.length, state.household.persons[0].name,
      state.flows.spending.value, state.wealth.stocks, state.realestate.properties.length])).toEqual(["", 1, "", 0, 0, 0]); // #170: no project name (no texts in the file)
    await expect(page.locator("#nav a.active")).toContainText("Haushalt & Projekt");
    await noErrors(page, errors);
  });

  test("AK5, AK8: Esc closes without a change and without help; Tab and Enter select", async ({ page }) => {
    let errors = await open(page, "welcome");
    const before = await page.evaluate(() => JSON.stringify(state));
    await page.keyboard.press("Escape");
    await expect(welcome(page)).not.toHaveAttribute("open", "");
    await expect(page.locator("#help")).toBeHidden();
    expect(await page.evaluate(() => JSON.stringify(state))).toBe(before);
    await noErrors(page, errors);
    // Keyboard: the first button has the focus, Tab moves to the second, Enter selects it.
    await page.goto("about:blank");
    errors = await open(page, "welcome");
    await expect(welcome(page).getByRole("button", { name: /Beispiel erkunden/ })).toBeFocused();
    await page.keyboard.press("Tab");
    await expect(welcome(page).getByRole("button", { name: /Eigene Planung beginnen/ })).toBeFocused();
    await page.keyboard.press("Enter");
    await expect(page.locator("#stepsProg")).toHaveText("0 von 8");
    await noErrors(page, errors);
  });

  test("AK7: Help → Welcome opens the dialog; with unsaved changes the app asks back", async ({ page }) => {
    const errors = await open(page);
    await field(page, "project.name").fill("Geändert");
    await field(page, "project.name").press("Tab");
    await page.evaluate(() => { command("welcome"); });
    await expect(welcome(page)).toHaveAttribute("open", "");
    await welcome(page).getByRole("button", { name: /Beispiel erkunden/ }).click();
    await expect(page.locator("#ask")).toHaveAttribute("open", "");
    await expect(page.locator("#askTitle")).toHaveText("Ungespeicherte Änderungen");
    await page.locator('#ask button[value="cancel"]').click();
    expect(await page.evaluate(() => state.project.name)).toBe("Geändert");
    await noErrors(page, errors);
  });
});

test.describe("#71 First steps", () => {
  test("AK1, AK5: eight steps of the quick start with progress", async ({ page }) => {
    const errors = await open(page);
    await expect(steps(page)).toBeVisible();
    await expect(steps(page).locator("li a")).toHaveText(["1. Haushalt & Projekt", "2. Meilensteine", "3. Vorsorge", "4. Einnahmen & Ausgaben",
      "5. Vermögen & Anlage", "6. Simulation", "7. Steuern", "8. Simulation starten"]);
    await expect(page.locator("#stepsProg")).toHaveText("0 von 8");
    await noErrors(page, errors);
  });

  test("AK2: click opens the section, ⓘ the help on the step", async ({ page }) => {
    const errors = await open(page);
    await steps(page).getByText("3. Vorsorge").click();
    await expect(page.locator("#nav a.active")).toContainText("Vorsorge");
    await steps(page).locator('li[data-step="taxes"] .hq').click();
    await expect(helpTitle(page)).toHaveText("Steuern");
    await noErrors(page, errors);
  });

  test("AK3, AK4, AK9: a change or a check mark completes a step, undo does not remove it, saved only in the settings", async ({ page }) => {
    const errors = await open(page);
    await field(page, "project.name").fill("Geändert");
    await field(page, "project.name").press("Tab");
    await expect(steps(page).locator('li[data-step="general"]')).toHaveClass(/done/);
    await expect(page.locator("#stepsProg")).toHaveText("1 von 8");
    await page.evaluate(() => History.undo());
    await expect(steps(page).locator('li[data-step="general"]')).toHaveClass(/done/);
    // Check off by hand and clear again
    await steps(page).locator('li[data-step="taxes"] input').check();
    await expect(page.locator("#stepsProg")).toHaveText("2 von 8");
    await steps(page).locator('li[data-step="taxes"] input').uncheck();
    await expect(page.locator("#stepsProg")).toHaveText("1 von 8");
    expect((await prefs(page)).firstSteps.done).toEqual(["general"]);
    expect(await page.evaluate(() => "firstSteps" in state)).toBe(false);
    await noErrors(page, errors);
  });

  test("AK3: the start simulation step is completed after a simulation, not after a cancel", async ({ page }) => {
    const errors = await open(page, "results/nosweep");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "overview");
    expect((await prefs(page)).firstSteps.done).toEqual(["run"]);
    await noErrors(page, errors);
  });

  test("AK6, AK7: hide, show via the menu; everything completed collapses", async ({ page }) => {
    const errors = await open(page);
    await steps(page).locator("button.x").click();
    await expect(steps(page)).toBeHidden();
    await page.evaluate(() => command("steps"));
    await expect(steps(page)).toBeVisible();
    await expect(steps(page)).toBeInViewport({ ratio: 0.95 }); // #70/AK3: scrolled into view
    for (const id of ["general", "milestones", "pension", "flows", "wealth", "returns", "taxes", "run"])
      await steps(page).locator(`li[data-step="${id}"] input`).check();
    await expect(page.locator("#stepsProg")).toHaveText("8 von 8");
    await expect(page.locator("#toast")).toContainText("Alle ersten Schritte erledigt");
    await expect(steps(page).locator("details")).not.toHaveAttribute("open", "");
    await expect(steps(page).locator(".stepsdone")).toBeAttached();
    await noErrors(page, errors);
  });

  test("AK8: start own plan shows the checklist and resets it", async ({ page }) => {
    const errors = await open(page);
    await steps(page).locator('li[data-step="taxes"] input').check();
    await steps(page).locator("button.x").click();
    await page.evaluate(() => { command("welcome"); });
    await welcome(page).getByRole("button", { name: /Eigene Planung beginnen/ }).click();
    await expect(steps(page)).toBeVisible();
    await expect(page.locator("#stepsProg")).toHaveText("0 von 8");
    await noErrors(page, errors);
  });
});

test.describe("#72 Result in words", () => {
  test("AK1, AK3, AK6: several runs with classification, worst run and next step", async ({ page }) => {
    const errors = await open(page, "results/bootstrap/nosweep");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "overview");
    const v = page.locator("#verdict");
    const rate = (await resultKpi(page, "Erfolgsquote").textContent()).trim();
    const cls = await page.evaluate(() => rateClass(lastRun.batch.summary.successRate));
    await expect(v).toContainText(`In ${rate} (${cls}) der Läufe reicht das Vermögen bis`);
    const ed = (await resultKpi(page, "Früheste Erschöpfung").textContent()).trim();
    if (ed === "keine") await expect(v).not.toContainText("ungünstigsten Lauf");
    else await expect(v).toContainText(`Im ungünstigsten Lauf ist das Geld mit ${ed} aufgebraucht.`);
    await expect(v.locator(".next a")).toHaveText(cls === "hoch" ? "Szenarien" : "Optimierung");
    await v.getByRole("link", { name: cls }).or(v.locator(`a:text-is("${cls}")`)).first().click();
    await expect(helpTitle(page)).toHaveText("Ergebnisse: Übersicht");
    await noErrors(page, errors);
  });

  test("AK2: limits of the classification at the displayed, rounded value", async ({ page }) => {
    const errors = await open(page);
    expect(await page.evaluate(() => [0.95, 0.9496, 0.9494, 0.8, 0.7996, 0.7994, 0].map(rateClass)))
      .toEqual(["hoch", "hoch", "mittel", "mittel", "mittel", "tief", "tief"]);
    await noErrors(page, errors);
  });

  test("AK4, AK5, AK7: single run, amounts follow nominal/purchasing power, depleted wealth refers to the optimization", async ({ page }) => {
    const errors = await open(page, "results/nosweep");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "overview");
    const v = page.locator("#verdict");
    const nominal = await page.evaluate(() => fmtMoney(lastRun.single.summary.endTotal));
    await expect(v).toContainText(`Das Vermögen reicht bis zum Ende der Planung und beträgt dann ${nominal}.`);
    await expect(v.locator(".next a")).toHaveText("Szenarien");
    await page.getByRole("button", { name: "Heutige Kaufkraft" }).click();
    const real = await page.evaluate(() => fmtMoney(lastRun.single.summary.endTotalReal));
    await expect(v).toContainText(`beträgt dann ${real} in heutiger Kaufkraft.`);
    // Spending so high that the money does not last
    await page.evaluate(async () => { showResults(false); state.flows.spending = { mode: "const", value: 60000 }; await runSimulation("overview", { noConfirm: true }); });
    const dep = await page.evaluate(() => lastRun.single.summary.depleted);
    expect(dep).toBeTruthy();
    await expect(v).toContainText(/Das Vermögen ist im \d\d\.\d{4} \(Alter [\d.,]+\) aufgebraucht\./);
    await expect(v.locator(".next a")).toHaveText("Optimierung");
    await noErrors(page, errors);
  });

  test("AK6: saved results from the list show the text", async ({ page }) => {
    const errors = await open(page, "runsflow/view/mem");
    await expect(page.locator(".savedbar")).toBeVisible();
    await expect(page.locator("#verdict")).toContainText("der Läufe reicht das Vermögen bis");
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
// #61 Disclaimer: footer of the results and full text
test.describe("#61 Disclaimer", () => {
  const FOOT = "Modellrechnung ohne Gewähr. Keine Finanz- oder Steuerberatung.";
  const prefs = page => page.evaluate(() => JSON.parse(localStorage.getItem("rp.settings") || "{}"));
  const helpTitle = page => page.locator("#help:not([hidden]) #helpBody h1").first();

  test("AK2: in the browser with test access confirm once, saved afterwards (browser storage)", async ({ page }) => {
    await page.goto("/#disclaimer");
    await page.waitForSelector("html[data-disclaimer='open']");
    await page.locator("#disclaimerCheck").check();
    await page.locator("#disclaimerOk").click();
    await page.waitForSelector("html[data-ready='1']");
    expect((await prefs(page)).disclaimerAccepted).toBe(1);
    await page.reload();
    await page.waitForSelector("html[data-ready='1']");
    await expect(page.locator("#disclaimer")).not.toHaveAttribute("open", "");
  });

  test("AK3: footer on all result pages", async ({ page }) => {
    const errors = await open(page, "results/nosweep");
    await expect(page.locator("#results")).toBeVisible();
    for (const tab of await page.locator('[data-action="rtab"]').all()) {
      await tab.click();
      await expect(page.locator("#resultsFoot")).toBeVisible();
      await expect(page.locator("#resultsFoot")).toHaveText(FOOT);
    }
    // saved results (list) are in the same view
    await expect(page.locator("#results .rfoot")).toHaveCount(1);
    await noErrors(page, errors);
  });

  test("AK3: footer below the result of the optimization", async ({ page }) => {
    const errors = await open(page, "optimize/bootstrap");
    await expect(page.locator("#optFoot")).toHaveCount(0);
    // small computation: only the footer is checked, not the result
    const small = { paths: 50, checkPaths: 50, samples: 1, strategies: ["fixed_rate"] };
    expect(await setOpt(page, small)).toMatchObject(small); // #79
    await page.locator('button[data-action="opt-run"]').click();
    await expect(page.locator("html")).toHaveAttribute("data-job", "off");
    await expect(page.locator("#optFoot")).toHaveText(FOOT);
    await noErrors(page, errors);
  });

  test("AK4: full text reachable in the about dialog and as a help page, with date", async ({ page }) => {
    const errors = await open(page);
    await page.evaluate(() => command("about"));
    await expect(page.locator("#ask")).toHaveAttribute("open", "");
    await expect(page.locator("#aboutDisclaimer")).toContainText(FOOT);
    await page.locator('#aboutDisclaimer a[data-action="about-disclaimer"]').click();
    await expect(page.locator("#ask")).not.toHaveAttribute("open", "");
    await expect(helpTitle(page)).toHaveText("Haftungsausschluss");
    const body = page.locator("#help:not([hidden]) #helpBody");
    for (const h of ["1. Kostenloses Angebot.", "2. Keine Beratung.", "3. Modellrechnungen ohne Gewähr.", "4. Eigene Verantwortung.",
      "5. Haftung.", "6. Daten.", "7. Anwendbares Recht."]) await expect(body).toContainText(h);
    await expect(body).toContainText("VivaPlan wird aus der Schweiz unentgeltlich zur Verfügung gestellt.");
    await expect(body).toContainText("Stand: 13. November 2026");
    // also via the search of the help
    await page.evaluate(() => Help.show("disclaimer"));
    await expect(helpTitle(page)).toHaveText("Haftungsausschluss");
    await noErrors(page, errors);
  });
});
