// #309: Updates with consent. The development server has no updater; the test access #updateask shows the question as
// at the first start of the app, #updateoffer a version found (9.9.9). The answer lands in the settings (browser storage).
const { test, expect } = require("@playwright/test");
const { open, noErrors, prefs, settingsTab } = require("./helpers");

async function ask(page) {
  const errors = [];
  page.on("pageerror", e => errors.push(e.message));
  await page.goto("/#updateask/nobench");
  await expect(page.locator("#updateAsk")).toHaveAttribute("open", "");
  return errors;
}

test.describe("Updates with consent (#309)", () => {
  test("AK1: the question names what is sent and comes only once", async ({ page }) => {
    const errors = await ask(page);
    await expect(page.locator("#updateAskTitle")).toHaveText("Nach Updates suchen?");
    await expect(page.locator("#updateAsk")).toContainText("Soll VivaPlan beim Start nach neuen Versionen suchen?");
    await expect(page.locator("#updateAsk")).toContainText("nur die Version von VivaPlan und das Betriebssystem");
    await expect(page.locator("#updateAsk")).toContainText("keine Planungsdaten");
    await expect(page.locator("#updateAsk .askbtns button")).toHaveText(["Nein", "Ja"]);
    await page.locator("#updateYes").click();
    await page.waitForSelector("html[data-ready='1']");
    expect((await prefs(page)).updateCheck).toBe(true);
    // the next start asks no more
    await page.reload();
    await page.waitForSelector("html[data-ready='1']");
    await expect(page.locator("#updateAsk")).not.toHaveAttribute("open", "");
    await noErrors(page, errors);
  });

  test("AK4: «Nein» and Escape count as no consent", async ({ page }) => {
    const errors = await ask(page);
    await page.locator("#updateNo").click();
    await page.waitForSelector("html[data-ready='1']");
    expect((await prefs(page)).updateCheck).toBe(false);
    // the same start without an answer yet, closed with Escape
    await page.evaluate(() => localStorage.removeItem("rp.settings"));
    await page.reload();
    await expect(page.locator("#updateAsk")).toHaveAttribute("open", "");
    await page.keyboard.press("Escape");
    await page.waitForSelector("html[data-ready='1']");
    expect((await prefs(page)).updateCheck).toBe(false);
    await noErrors(page, errors);
  });

  test("AK2: the choice is in the settings, tab «Allgemein», and can be changed", async ({ page }) => {
    const errors = await open(page);
    await page.locator('[data-action="settings"]').first().click();
    await settingsTab(page, "general");
    const box = page.locator("#set-updateCheck");
    await expect(page.locator("#setpanel-general")).toContainText("Beim Start nach neuen Versionen suchen");
    await expect(box).not.toBeChecked();
    await box.check();
    await page.locator("#settingsSave").click();
    expect((await prefs(page)).updateCheck).toBe(true);
    await page.locator('[data-action="settings"]').first().click();
    await expect(page.locator("#set-updateCheck")).toBeChecked();
    // «Standardwerte» leaves the consent as it is
    await page.locator("#settingsDefaults").click();
    await page.locator('#ask button[value="ok"]').click();
    await expect(page.locator("#set-updateCheck")).toBeChecked();
    await page.locator("#set-updateCheck").uncheck();
    await page.locator("#settingsSave").click();
    expect((await prefs(page)).updateCheck).toBe(false);
    await noErrors(page, errors);
  });

  test("AK3: a version found shows version, changes and the buttons install now and later", async ({ page }) => {
    const errors = await open(page, "updateoffer");
    const d = page.locator("#updateOffer");
    await expect(d).toHaveAttribute("open", "");
    await expect(page.locator("#updateOfferTitle")).toHaveText("Neue Version verfügbar");
    await expect(page.locator("#updateOfferText")).toHaveText("VivaPlan 9.9.9 ist verfügbar.");
    await expect(page.locator("#updateNotes")).toHaveText("Neu: Test");
    await expect(d.locator(".askbtns button")).toHaveText(["Später", "Jetzt installieren"]);
    await page.locator("#updateLater").click();
    await expect(d).not.toHaveAttribute("open", "");
    await noErrors(page, errors);
  });
});
