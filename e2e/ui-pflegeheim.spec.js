// User interface tests of the care home per person (#271). Every test starts with the example plan.
const { test, expect } = require("@playwright/test");
const { open, noErrors, nav, parseNum, previewIdle, helpTitle } = require("./helpers");

const toggle = (page, n) => page.locator(`#careCard${n} input[data-action="care-toggle"]`);
const care = (page, n) => page.evaluate(n => JSON.parse(JSON.stringify(state.household.persons[n].careHome ?? null)), n);

test.describe("Care home per person (#271)", () => {
  test("AK1: card per person in the pension section; entry as age, milestone or date, costs and sale", async ({ page }) => {
    const errors = await open(page);
    await page.evaluate(() => { ACTIONS["partner-add"](); renderSection(); });
    await nav(page, "Vorsorge");
    for (const n of [0, 1]) {
      await expect(page.locator(`#careCard${n} h3`)).toContainText("Pflegeheim");
      await expect(toggle(page, n)).not.toBeChecked();
    }
    expect(await care(page, 0)).toBeNull();
    await toggle(page, 0).check();
    expect(await care(page, 0)).toEqual({ age: 85, monthly: 9000, sell: false });
    const card = page.locator("#careCard0");
    await expect(card).toContainText("Heimkosten pro Monat (heutige Preise)");
    await expect(card).toContainText("Die Heimkosten steigen mit der Teuerung.");
    await expect(card).toContainText("bis beide Personen im Pflegeheim sind");
    // age → fixed date keeps the month of the entry
    const birth = await page.evaluate(() => ym2i(state.household.persons[0].birth));
    await card.locator('select[data-type="carewhen"]').selectOption("__date");
    await previewIdle(page);
    const c = await care(page, 0);
    expect(c.age).toBeUndefined();
    expect(await page.evaluate(at => ym2i(at), c.at)).toBe(birth + 85 * 12);
    // milestone
    await page.locator('#careCard0 select[data-type="carewhen"]').selectOption("ahv");
    expect((await care(page, 0)).at).toBe("ahv");
    // back to the age
    await page.locator('#careCard0 select[data-type="carewhen"]').selectOption("age");
    expect((await care(page, 0)).at).toBeUndefined();
    await page.locator('input[data-path="household.persons.0.careHome.age"]').fill("86");
    await page.locator('input[data-path="household.persons.0.careHome.monthly"]').fill("8000");
    await page.locator('input[data-path="household.persons.0.careHome.sell"]').check();
    await previewIdle(page);
    expect(await care(page, 0)).toEqual({ age: 86, monthly: 8000, sell: true });
    // switched off: the entry is removed from the plan
    await page.locator("#careCard0 input[data-action=\"care-toggle\"]").uncheck();
    expect(await care(page, 0)).toBeNull();
    await noErrors(page, errors);
  });

  test("AK2: the yearly values show the care home costs in a column of their own, only with a care home", async ({ page }) => {
    const errors = await open(page);
    const table = async () => {
      await page.evaluate(() => runSimulation("table", { noConfirm: true }));
      await expect(page.locator("#resultsBody table.yt")).toBeVisible();
      const heads = await page.locator("#resultsBody table.yt tr").first().locator("th").allTextContents();
      const rows = await page.locator("#resultsBody table.yt tr").evaluateAll(trs => trs.slice(1).map(tr => [...tr.querySelectorAll("td")].map(td => td.textContent)));
      await page.evaluate(() => showResults(false));
      return { heads, rows };
    };
    await page.evaluate(() => { realView = false; state.returns.source = "fixed"; state.scenarios.sweeps = []; state.returns.fixed.inflation = 0; });
    const without = await table();
    expect(without.heads).not.toContain("Heimkosten");
    const entry = await page.evaluate(() => {
      const at = i2ym(startM() + 24);
      state.household.persons[0].careHome = { at: at.slice(0, 4) + "-01", monthly: 9000 };
      return +at.slice(0, 4);
    });
    const withCare = await table();
    const col = withCare.heads.indexOf("Heimkosten");
    expect(col).toBe(withCare.heads.indexOf("Ausgaben total") + 1);
    const row = withCare.rows.find(r => r[0].startsWith(String(entry + 1)));
    expect(parseNum(row[col])).toBe(9000);
    const before = withCare.rows.find(r => r[0].startsWith(String(entry - 1)));
    expect(before[col]).toBe("");
    await noErrors(page, errors);
  });

  test("AK6: hint when neither a rent nor a care home follows the sale of the last property", async ({ page }) => {
    const errors = await open(page);
    const hint = "folgt weder eine Miete noch ein Pflegeheim";
    await page.evaluate(() => {
      state.realestate.neverSell = false;
      state.realestate.properties = [{ id: "w", name: "Wohnung", status: "owned", price: 500000, worth: 600000, fixCosts: 500, sell: i2ym(startM() + 60), mortgage: { tranches: [] } }];
      markDirty(); renderSection();
    });
    await nav(page, "Immobilien");
    await expect(page.locator("body")).toContainText(hint);
    await page.evaluate(() => {
      state.household.persons[0].careHome = { at: state.realestate.properties[0].sell, monthly: 9000 };
      markDirty(); renderSection();
    });
    await previewIdle(page);
    await expect(page.locator("body")).not.toContainText(hint);
    await noErrors(page, errors);
  });

  test("AK8: F1 on a field of the card opens the help topic «Pflegeheim» with the simplified tax rule", async ({ page }) => {
    const errors = await open(page);
    await nav(page, "Vorsorge");
    await toggle(page, 0).check();
    await page.locator('input[data-path="household.persons.0.careHome.monthly"]').focus();
    await page.keyboard.press("F1");
    await expect(helpTitle(page)).toHaveText("Pflegeheim");
    await expect(page.locator("#helpBody")).toContainText("soweit sie 5 % des Nettoeinkommens übersteigen");
    await noErrors(page, errors);
  });
});
