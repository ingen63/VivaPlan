// User interface tests of the taxable yield of the wealth (#149). Every test starts with the example plan.
const { test, expect } = require("@playwright/test");
const { open, noErrors, nav, parseNum, previewIdle, helpTitle } = require("./helpers");

const rates = page => page.evaluate(() => JSON.parse(JSON.stringify(state.taxes.investmentYield ?? null)));

test.describe("Taxable yield of the wealth (#149)", () => {
  test("AK1: card in the section taxes with a rate for stocks and bonds, no field for cash", async ({ page }) => {
    const errors = await open(page);
    await nav(page, "Steuern");
    const card = page.locator("#tax-investment-yield");
    await expect(card.locator("h3")).toContainText("Steuerbarer Ertrag des Vermögens");
    await expect(card).toContainText("Aktien (Ausschüttungsrendite)");
    await expect(card).toContainText("Obligationen (Coupon)");
    await expect(card.locator("input")).toHaveCount(2);
    await expect(card).toContainText("Cash hat keinen steuerbaren Ertrag");
    await card.locator('input[data-path="taxes.investmentYield.stocks"]').fill("2,5");
    await card.locator('input[data-path="taxes.investmentYield.bonds"]').fill("1");
    await card.locator('input[data-path="taxes.investmentYield.bonds"]').blur();
    await previewIdle(page);
    expect(await rates(page)).toEqual({ stocks: 0.025, bonds: 0.01 });
    await noErrors(page, errors);
  });

  test("AK5: existing plans open with 0 %, a new plan has stocks 2 % and bonds 1 %", async ({ page }) => {
    const errors = await open(page);
    expect(await rates(page)).toEqual({ stocks: 0, bonds: 0 });
    await page.evaluate(async () => { await adopt(await blankPlan(), false); });
    expect(await rates(page)).toEqual({ stocks: 0.02, bonds: 0.01 });
    await nav(page, "Steuern");
    await expect(page.locator('input[data-path="taxes.investmentYield.stocks"]')).toHaveValue("2");
    await expect(page.locator('input[data-path="taxes.investmentYield.bonds"]')).toHaveValue("1");
    await noErrors(page, errors);
  });

  test("AK6: the yearly values show the column «Steuerbarer Ertrag» before the income tax, only with a yield", async ({ page }) => {
    const errors = await open(page);
    const table = async () => {
      await page.evaluate(async () => {
        lastRun = await Engine.call("run_simulation", { plan: state, path: null });
        realView = false; resetVariation(); resultTab = "table"; showTaxCols = true; showResults(true); renderResults(lastRun);
      });
      await expect(page.locator("#resultsBody table.yt")).toBeVisible();
      const heads = await page.locator("#resultsBody table.yt tr").first().locator("th").allTextContents();
      const cells = await page.locator("#resultsBody table.yt td[data-tax-yield]").allTextContents();
      await page.evaluate(() => showResults(false));
      return { heads, cells };
    };
    await page.evaluate(() => { state.returns.source = "fixed"; state.scenarios.sweeps = []; });
    const without = await table();
    expect(without.heads).not.toContain("SteuerbarerErtrag");
    expect(without.cells).toEqual([]);
    await page.evaluate(() => { state.taxes.investmentYield = { stocks: 0.02, bonds: 0.01 }; });
    const withYield = await table();
    const col = withYield.heads.indexOf("SteuerbarerErtrag");
    expect(col).toBeGreaterThan(0);
    expect(withYield.heads[col + 1]).toContain("Einkommen");
    expect(withYield.cells.some(c => parseNum(c) > 0)).toBe(true);
    await noErrors(page, errors);
  });

  test("AK8: a rate below 0 % or above 100 % is an error at the field in the section taxes", async ({ page }) => {
    const errors = await open(page);
    await nav(page, "Steuern");
    const field = page.locator('input[data-path="taxes.investmentYield.stocks"]');
    await field.fill("120");
    await field.blur();
    await previewIdle(page);
    await expect.poll(() => page.evaluate(() => lastIssues.filter(i => i.severity === "error").map(i => [i.section, i.field])))
      .toEqual([["taxes", "taxes.investmentYield.stocks"]]);
    await page.locator('input[data-path="taxes.investmentYield.bonds"]').fill("-1");
    await page.locator('input[data-path="taxes.investmentYield.bonds"]').blur();
    await previewIdle(page);
    await expect.poll(() => page.evaluate(() => lastIssues.filter(i => i.severity === "error").map(i => i.field)))
      .toEqual(["taxes.investmentYield.stocks", "taxes.investmentYield.bonds"]);
    await noErrors(page, errors);
  });

  test("AK10: F1 on a rate opens the help topic with the guide values; the overview of the assumptions names the rates", async ({ page }) => {
    const errors = await open(page);
    await nav(page, "Steuern");
    await page.locator('input[data-path="taxes.investmentYield.stocks"]').focus();
    await page.keyboard.press("F1");
    await expect(helpTitle(page)).toHaveText("Steuerbarer Ertrag des Vermögens");
    await expect(page.locator("#helpBody")).toContainText("Swiss Bond Index bei rund 1 %");
    await page.keyboard.press("Escape");
    await page.evaluate(() => { state.taxes.investmentYield = { stocks: 0.02, bonds: 0.01 }; });
    const params = await page.evaluate(() => JSON.stringify(paramSections(state, true)));
    expect(params).toContain("Steuerbarer Ertrag des Vermögens");
    expect(params).toContain("Aktien 2 %, Obligationen 1 %");
    await noErrors(page, errors);
  });
});
