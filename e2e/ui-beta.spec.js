// #270: Beta token. The development server needs no activation; the test access #beta switches the check on, with the
// test key of the engine (engine::beta::TEST_SEED) and without saving anything. Tokens made with
// `beta-token issue --key <file with the test key> --who AB --id E2E1 --until …`.
const { test, expect } = require("@playwright/test");
const { MOD } = require("./helpers");

const VALID = "VP1.beta.E2E1.AB.2099-12-31._VKZK_5JKpCllvQ2KBvS78C-Z-iU4Mo4oh_sfc71dr1MzlzNIfcYbdbYiYymmt3EzzRT5gOwsE66tagyuK3XAQ";
const EXPIRED = "VP1.beta.E2E1.AB.2020-01-01.pVbwtMatiaf2CxbFJq2jD5n4wicA7qi6agFUQ87AwH2s3UT9lNSQJoQDFGzzKIy9xjyIVk0VBxpBdYUjNLyvCw";
// Signed with another key
const FOREIGN = "VP1.beta.E2E1.AB.2099-12-31.PfjBOdvJ0Q56bOJSOtRcLztVxHzVAVpjGtBASAzT8P-ImfGaoV_66QAl_O-quiO_c1QdL4nFwsJleku-cQJrBg";

async function gate(page) {
  const errors = [];
  page.on("pageerror", e => errors.push(e.message));
  await page.goto("/#beta/nobench");
  await page.waitForSelector("html[data-beta='open']");
  await expect(page.locator("#beta")).toHaveAttribute("open", "");
  return errors;
}

async function redeem(page, token) {
  await page.locator("#betaToken").fill(token);
  await page.locator("#betaOk").click();
}

test.describe("Beta token (#270)", () => {
  test("AK4: without an activation only the dialog «Beta-Token eingeben» is reachable", async ({ page }) => {
    const errors = await gate(page);
    await expect(page.locator("#betaTitle")).toHaveText("Beta-Token eingeben");
    // No plan is loaded, nothing is calculated.
    await expect(page.locator("html")).not.toHaveAttribute("data-ready", "1");
    await expect(page.locator("#kpis .kpi")).toHaveCount(0);
    // Escape does not close the dialog; shortcuts for new, open and calculate do nothing.
    await page.keyboard.press("Escape");
    for (const key of ["n", "o", "Enter", "1"]) await page.keyboard.press(`${MOD}+${key}`);
    await expect(page.locator("#beta")).toHaveAttribute("open", "");
    await expect(page.locator("dialog[open]")).toHaveCount(1);
    await expect(page.locator("html")).not.toHaveAttribute("data-job", /.+/);
    // Everything outside the dialog is inert: a click on the navigation does not reach it.
    const blocked = await page.evaluate(() => {
      const a = document.querySelector("#nav a[data-sec]") || document.querySelector("header button");
      if (!a) return true;
      const r = a.getBoundingClientRect();
      const hit = document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2);
      return !a.contains(hit);
    });
    expect(blocked).toBe(true);
    expect(errors).toEqual([]);
  });

  test("AK2: a token whose redemption deadline is over is refused: a new token is needed", async ({ page }) => {
    const errors = await gate(page);
    await redeem(page, EXPIRED);
    await expect(page.locator("#betaErr")).toHaveText("Das Beta-Token liess sich nur bis 01.01.2020 einlösen. Bitte fragen Sie nach einem neuen Token.");
    await expect(page.locator("#beta")).toHaveAttribute("open", "");
    expect(errors).toEqual([]);
  });

  test("AK3: a changed token and a token of another key are refused", async ({ page }) => {
    const errors = await gate(page);
    const changed = VALID.slice(0, 40) + (VALID[40] === "A" ? "B" : "A") + VALID.slice(41);
    for (const token of [changed, FOREIGN]) {
      await redeem(page, token);
      await expect(page.locator("#betaErr")).toContainText("nicht gültig");
      await expect(page.locator("#beta")).toHaveAttribute("open", "");
    }
    await redeem(page, "irgendwas");
    await expect(page.locator("#betaErr")).toContainText("kein vollständiges Beta-Token");
    expect(errors).toEqual([]);
  });

  test("AK1, AK10: a valid token activates; the app starts and “About VivaPlan” shows initials and day", async ({ page }) => {
    const errors = await gate(page);
    // Copied from an e-mail with a line break and spaces.
    await redeem(page, ` ${VALID.slice(0, 50)}\n${VALID.slice(50)} `);
    await page.waitForSelector("html[data-ready='1']");
    await expect(page.locator("html")).toHaveAttribute("data-beta", "ok");
    await expect(page.locator("#beta")).not.toHaveAttribute("open", "");
    await expect(page.locator("#kpis .kpi")).toHaveCount(4);
    await page.evaluate(() => command("about"));
    const today = new Date().toISOString().slice(0, 10).split("-").reverse().join(".");
    await expect(page.locator("#aboutBeta td").first()).toHaveText("Beta aktiviert");
    await expect(page.locator("#aboutBeta td").nth(1)).toHaveText(`AB, am ${today}`);
    expect(errors).toEqual([]);
  });

  test("without the test access the development server starts without the dialog", async ({ page }) => {
    await page.goto("/#nobench");
    await page.waitForSelector("html[data-ready='1']");
    await expect(page.locator("#beta")).not.toHaveAttribute("open", "");
    await expect(page.locator("html")).not.toHaveAttribute("data-beta", /.+/);
  });
});
