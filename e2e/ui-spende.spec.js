// #237: donation link «Buy me a coffee» in the menu Help, at the bottom of the navigation and in the about dialog.
// Desktop app: a stand-in records the commands (the fixed list of addresses and the command `open_link` are checked by
// the Rust tests of crates/app). Development server: the link opens a new browser tab (window.open, replaced here so
// that the test loads nothing from the internet).
const { test, expect } = require("@playwright/test");
const { open, noErrors } = require("./helpers");

const URL = "https://buymeacoffee.com/vivaplan";

/** Desktop app stand-in: every call recorded; the rest goes to the development server. */
async function openApp(page) {
  await page.addInitScript(() => {
    window.__calls = []; window.__listeners = {};
    window.__TAURI__ = {
      core: {
        invoke: async (cmd, args = {}) => {
          window.__calls.push([cmd, JSON.parse(JSON.stringify(args))]);
          const example = async () => (await fetch("/api/example_plan", { method: "POST", body: "{}" })).json();
          if (cmd === "settings_get") return { disclaimerAccepted: 2 };
          if (cmd === "first_start_language") return null;
          if (cmd === "startup_plan") return { path: "/x/Familie.vplan", plan: await example(), notice: null, first: false };
          if (["settings_set", "set_dirty", "quit_app", "set_window_title", "set_menu_language", "open_link"].includes(cmd)) return null;
          const res = await fetch(`/api/${cmd}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(args) });
          const j = await res.json(); if (!res.ok) throw new Error(j.error); return j;
        }
      },
      event: { listen: (name, cb) => { (window.__listeners[name] ??= []).push(cb); return Promise.resolve(() => {}); } }
    };
    window.__menu = id => { for (const cb of window.__listeners.menu || []) cb({ payload: id }); };
  });
  const errors = [];
  page.on("pageerror", e => errors.push(e.message));
  await page.goto("/#nobench");
  await page.waitForSelector("html[data-ready='1']");
  return errors;
}
const linkCalls = page => page.evaluate(() => window.__calls.filter(c => c[0] === "open_link").map(c => c[1]));

/** #237/AK3: records every request that leaves the local development server. */
function external(page) {
  const out = [];
  page.on("request", r => { if (!/^http:\/\/127\.0\.0\.1:\d+\//.test(r.url()) && !r.url().startsWith("data:")) out.push(r.url()); });
  return out;
}

test.describe("Donation link Buy me a coffee (#237)", () => {
  test("AK1, AK2: menu Help entry «support» opens the link via the desktop app", async ({ page }) => {
    const ext = external(page);
    const errors = await openApp(page);
    await page.evaluate(() => window.__menu("support"));
    await expect.poll(() => linkCalls(page)).toEqual([{ id: "donate" }]);
    await expect(page.locator("#ask")).not.toHaveAttribute("open", "");
    expect(ext).toEqual([]);
    await noErrors(page, errors);
  });

  test("AK1, AK2: grey line at the bottom of the navigation opens the link via the desktop app", async ({ page }) => {
    const ext = external(page);
    const errors = await openApp(page);
    const line = page.locator("#nav .navfoot a.navsupport");
    await expect(line).toHaveText("☕ VivaPlan unterstützen");
    await expect(line).toHaveAttribute("title", /Kaffee spendieren/);
    // below the note «Daten bleiben lokal …», grey like it
    await expect(page.locator("#nav .navfoot")).toContainText("Daten bleiben lokal");
    const [lineColor, footColor] = await page.evaluate(() => [getComputedStyle(document.querySelector("#nav a.navsupport")).color,
      getComputedStyle(document.querySelector("#nav .navfoot")).color]);
    expect(lineColor).toBe(footColor);
    await line.click();
    await expect.poll(() => linkCalls(page)).toEqual([{ id: "donate" }]);
    expect(ext).toEqual([]);
    await noErrors(page, errors);
  });

  test("AK1, AK2: about dialog has the row «support» below «website» with a clickable link", async ({ page }) => {
    const ext = external(page);
    const errors = await openApp(page);
    await page.evaluate(() => window.__menu("about"));
    await expect(page.locator("#ask")).toHaveAttribute("open", "");
    const rows = page.locator("#aboutTbl tr");
    await expect(rows.nth(2).locator("td").first()).toHaveText("Website");
    await expect(rows.nth(3).locator("td").first()).toHaveText("Unterstützen");
    const link = page.locator("#aboutSupport a");
    await expect(link).toHaveText("buymeacoffee.com/vivaplan");
    await link.click();
    await expect.poll(() => linkCalls(page)).toEqual([{ id: "donate" }]);
    await expect(page.locator("#ask")).toHaveAttribute("open", "");
    expect(ext).toEqual([]);
    await noErrors(page, errors);
  });

  test("AK4: no popup and no note at the start, the link only on request", async ({ page }) => {
    const ext = external(page);
    const errors = await open(page);
    await expect(page.locator("#ask")).not.toHaveAttribute("open", "");
    await expect(page.locator("#toast")).not.toContainText(/unterstützen|Kaffee/i);
    expect(await page.evaluate(() => [...document.querySelectorAll("dialog[open]")].map(d => d.id))).toEqual([]);
    expect(ext).toEqual([]);
    await noErrors(page, errors);
  });

  test("AK2: in the browser (development server) the link opens a new tab", async ({ page }) => {
    await page.addInitScript(() => { window.__opened = []; window.open = (...a) => { window.__opened.push(a); return null; }; });
    const ext = external(page);
    const errors = await open(page);
    await page.locator("#nav a.navsupport").click();
    await expect.poll(() => page.evaluate(() => window.__opened)).toEqual([[URL, "_blank", "noopener"]]);
    await page.evaluate(() => command("support"));
    await expect.poll(() => page.evaluate(() => window.__opened.length)).toBe(2);
    expect(ext).toEqual([]);
    await noErrors(page, errors);
  });

  test("AK1: texts in all four languages", async ({ page }) => {
    const want = {
      en: ["☕ Support VivaPlan", "Support", "Support VivaPlan…"], fr: ["☕ Soutenir VivaPlan", "Soutenir", "Soutenir VivaPlan…"],
      it: ["☕ Sostenere VivaPlan", "Sostenere", "Sostenere VivaPlan…"], de: ["☕ VivaPlan unterstützen", "Unterstützen", "VivaPlan unterstützen…"]
    };
    for (const [lang, [nav, about, menu]] of Object.entries(want)) {
      const t = await (await page.request.get(`/i18n/${lang}.json`)).json();
      expect([t["nav.support"], t["about.support"], t["menu.help.support"]]).toEqual([nav, about, menu]);
      expect(t["nav.supportTitle"]).toContain("buymeacoffee.com");
    }
  });
});
