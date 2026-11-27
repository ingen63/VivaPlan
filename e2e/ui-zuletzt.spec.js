// #227: «Datei → Zuletzt geöffnet» in the user interface. The desktop app is replaced by a stand-in that answers
// `open_recent` (the list itself, the menu and the settings are checked by the Rust tests of crates/app).
const { test, expect } = require("@playwright/test");
const { noErrors } = require("./helpers");

/** Desktop app stand-in: every call recorded, `open_recent` answers with `recent` (index → answer). */
async function openApp(page, recent) {
  await page.addInitScript(recent => {
    window.__calls = []; window.__listeners = {};
    window.__TAURI__ = {
      core: {
        invoke: async (cmd, args = {}) => {
          window.__calls.push([cmd, JSON.parse(JSON.stringify(args))]);
          const example = async () => (await fetch("/api/example_plan", { method: "POST", body: "{}" })).json();
          if (cmd === "settings_get") return { disclaimerAccepted: 2, updateCheck: false };
          if (cmd === "first_start_language") return null;
          if (cmd === "startup_plan") return { path: "/x/Familie.vplan", plan: await example(), notice: null, first: false };
          if (cmd === "open_recent") {
            const r = recent[args.index];
            if (r === "error") throw "Datei nicht lesbar";
            return r.opened ? { opened: { path: r.opened, plan: { ...await example(), project: { name: "Zweite" } } }, missing: null } : r;
          }
          if (["settings_set", "set_dirty", "quit_app", "set_window_title", "set_menu_language"].includes(cmd)) return null;
          const res = await fetch(`/api/${cmd}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(args) });
          const j = await res.json(); if (!res.ok) throw new Error(j.error); return j;
        }
      },
      event: { listen: (name, cb) => { (window.__listeners[name] ??= []).push(cb); return Promise.resolve(() => {}); } }
    };
    window.__menu = id => { for (const cb of window.__listeners.menu || []) cb({ payload: id }); };
  }, recent);
  const errors = [];
  page.on("pageerror", e => errors.push(e.message));
  await page.goto("/#nobench");
  await page.waitForSelector("html[data-ready='1']");
  return errors;
}
const recentCalls = page => page.evaluate(() => window.__calls.filter(c => c[0] === "open_recent").map(c => c[1]));

test.describe("Recently opened plans in the menu File (#227)", () => {
  test("AK3: a click on an entry opens the plan like Open", async ({ page }) => {
    const errors = await openApp(page, [{ opened: "/y/Zweite.vplan" }]);
    await page.evaluate(() => window.__menu("recent-0"));
    await expect(page.locator("#fileName")).toHaveText("Zweite.vplan");
    await expect(page.locator("#toast")).toContainText("«Zweite.vplan» geöffnet.");
    expect(await recentCalls(page)).toEqual([{ index: 0 }]);
    await noErrors(page, errors);
  });

  test("AK3: with unsaved changes the same question comes first; «Abbrechen» keeps the plan", async ({ page }) => {
    const errors = await openApp(page, [{ opened: "/y/Zweite.vplan" }]);
    await page.evaluate(() => markDirty());
    await page.evaluate(() => window.__menu("recent-0"));
    await expect(page.locator("#ask")).toBeVisible();
    await expect(page.locator("#askText")).toHaveText("Die aktuelle Planung hat ungespeicherte Änderungen. Trotzdem fortfahren?");
    await page.locator("#ask button[value='cancel']").click();
    await expect(page.locator("#fileName")).toHaveText("Familie.vplan");
    expect(await recentCalls(page)).toEqual([]);
    // confirming opens the plan
    await page.waitForTimeout(300); // the same menu id within 250 ms counts as a double click
    await page.evaluate(() => window.__menu("recent-0"));
    await page.locator("#ask button[value='ok']").click();
    await expect(page.locator("#fileName")).toHaveText("Zweite.vplan");
    expect(await recentCalls(page)).toEqual([{ index: 0 }]);
    await noErrors(page, errors);
  });

  test("AK4: a file that no longer exists is reported; the open plan stays", async ({ page }) => {
    const errors = await openApp(page, [{ opened: null, missing: "Weg.vplan" }, "error"]);
    await page.evaluate(() => window.__menu("recent-0"));
    await expect(page.locator("#toast")).toHaveText("Die Datei «Weg.vplan» gibt es nicht mehr. Sie wurde aus der Liste «Zuletzt geöffnet» entfernt.");
    await expect(page.locator("#fileName")).toHaveText("Familie.vplan");
    // an unreadable file is reported like with «Öffnen …»
    await page.evaluate(() => window.__menu("recent-1"));
    await expect(page.locator("#toast")).toHaveText("Datei nicht lesbar");
    await noErrors(page, errors);
  });
});
