// #240: introductory videos «VivaPlan in 2 Minuten» and «VivaPlan in 10 Minuten», one per language, opened in the browser.
// Desktop app: a stand-in records the commands (the fixed list of addresses and the command `open_link` are checked by
// the Rust tests of crates/app). Development server: the link opens a new browser tab (window.open, replaced here so
// that the test loads nothing from the internet).
const { test, expect } = require("@playwright/test");
const { open, noErrors, welcome } = require("./helpers");

const video = (lang, len) => `https://archive.org/details/vivaplan-video-${lang}/vivaplan-${len}.mp4`;

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
const linkCalls = page => page.evaluate(() => window.__calls.filter(c => c[0] === "open_link").map(c => c[1].id));

/** Records every request that leaves the local development server (AK5: the app itself loads nothing). */
function external(page) {
  const out = [];
  page.on("request", r => { if (!/^http:\/\/127\.0\.0\.1:\d+\//.test(r.url()) && !r.url().startsWith("data:")) out.push(r.url()); });
  return out;
}

test.describe("Introductory videos (#240)", () => {
  test("AK1: menu Help «videos» opens a dialog with both videos (title, length, note browser)", async ({ page }) => {
    const ext = external(page);
    const errors = await openApp(page);
    await page.evaluate(() => window.__menu("videos"));
    const d = page.locator("#ask");
    await expect(d).toHaveAttribute("open", "");
    await expect(page.locator("#askTitle")).toHaveText("Einführungsvideos");
    const items = page.locator("#videoList li");
    await expect(items).toHaveCount(2);
    await expect(items.nth(0)).toContainText("VivaPlan in 2 Minuten");
    await expect(items.nth(0)).toContainText("2 Minuten");
    await expect(items.nth(1)).toContainText("VivaPlan in 10 Minuten");
    await expect(items.nth(1)).toContainText("10 Minuten");
    await expect(d).toContainText("öffnet das Video im Browser");
    // the note «Video auf Deutsch» is dropped (comment of #240: every language has its own video)
    await expect(d).not.toContainText("auf Deutsch");
    await items.nth(0).locator("a").click();
    await items.nth(1).locator("a").click();
    await expect.poll(() => linkCalls(page)).toEqual(["video-2min-de", "video-10min-de"]);
    await expect(d).toHaveAttribute("open", "");
    expect(ext).toEqual([]);
    await noErrors(page, errors);
  });

  test("AK2: the welcome dialog has the link to the 2-minute video below the three choices", async ({ page }) => {
    const ext = external(page);
    const errors = await openApp(page);
    await page.evaluate(() => window.__menu("welcome"));
    await expect(welcome(page)).toHaveAttribute("open", "");
    const link = welcome(page).locator("p.wvideo a");
    await expect(link).toHaveText("▶ VivaPlan in 2 Minuten ansehen");
    // below the three choices
    const [choicesBottom, linkTop] = await page.evaluate(() =>
      [document.querySelector("#welcome .wchoices").getBoundingClientRect().bottom, document.querySelector("#welcome .wvideo a").getBoundingClientRect().top]);
    expect(linkTop).toBeGreaterThan(choicesBottom);
    await link.click();
    await expect.poll(() => linkCalls(page)).toEqual(["video-2min-de"]);
    // the dialog stays open, the choice is still to be made
    await expect(welcome(page)).toHaveAttribute("open", "");
    expect(ext).toEqual([]);
    await noErrors(page, errors);
  });

  test("AK3: help topics start and quickstart link the matching video in all four languages", async ({ page }) => {
    const ext = external(page);
    const errors = await openApp(page);
    const want = [];
    for (const lang of ["de", "en", "fr", "it"]) {
      await page.evaluate(l => I18n.use(l), lang);
      for (const [topic, len] of [["start", "2min"], ["quickstart", "10min"]]) {
        await page.evaluate(t => Help.show(t), topic);
        const a = page.locator(`#help:not([hidden]) #helpBody a[data-action="video"]`);
        await expect(a).toHaveCount(1);
        await expect(a).toHaveAttribute("data-video", len);
        await expect(a).toContainText(len === "2min" ? "2" : "10");
        await expect(page.locator("#helpBody")).not.toContainText(/Untertitel|subtitles|sous-titres|sottotitoli/);
        await a.click();
        want.push(`video-${len}-${lang}`);
        await expect.poll(() => linkCalls(page)).toEqual(want);
      }
    }
    expect(await page.evaluate(() => Help.check())).toEqual([]);
    expect(ext).toEqual([]);
    await noErrors(page, errors);
  });

  test("AK1, AK3: the app opens the video in the language set", async ({ page }) => {
    const errors = await openApp(page);
    for (const lang of ["en", "fr", "it", "de"]) {
      await page.evaluate(l => I18n.use(l), lang);
      // the same menu command within 250 ms counts once (command, app.js): reset that for the test
      await page.evaluate(() => { lastCommand = {}; window.__menu("videos"); });
      await expect(page.locator("#askTitle")).toHaveText(await page.evaluate(() => tr("videos.title")));
      await page.locator("#videoList li a").nth(1).click();
      await page.evaluate(() => $("#ask").close());
      await expect(page.locator("#ask")).not.toHaveAttribute("open", "");
    }
    await expect.poll(() => linkCalls(page)).toEqual(["video-10min-en", "video-10min-fr", "video-10min-it", "video-10min-de"]);
    await noErrors(page, errors);
  });

  test("AK5: in the browser (development server) the links open a new tab with the fixed address", async ({ page }) => {
    await page.addInitScript(() => { window.__opened = []; window.open = (...a) => { window.__opened.push(a); return null; }; });
    const ext = external(page);
    const errors = await open(page);
    await page.evaluate(() => command("videos"));
    await page.locator("#videoList li a").nth(0).click();
    await expect.poll(() => page.evaluate(() => window.__opened)).toEqual([[video("de", "2min"), "_blank", "noopener"]]);
    await page.evaluate(() => $("#ask").close());
    await page.evaluate(() => I18n.use("fr"));
    await page.evaluate(() => { showWelcome(); });
    await welcome(page).locator("p.wvideo a").click();
    await expect.poll(() => page.evaluate(() => window.__opened.length)).toBe(2);
    expect((await page.evaluate(() => window.__opened))[1]).toEqual([video("fr", "2min"), "_blank", "noopener"]);
    expect(ext).toEqual([]);
    await noErrors(page, errors);
  });

  test("AK1, AK2: texts in all four languages", async ({ page }) => {
    const want = {
      de: ["Einführungsvideos …", "▶ VivaPlan in 2 Minuten ansehen", "VivaPlan in 2 Minuten", "VivaPlan in 10 Minuten"],
      en: ["Introductory videos …", "▶ Watch VivaPlan in 2 minutes", "VivaPlan in 2 minutes", "VivaPlan in 10 minutes"],
      fr: ["Vidéos d’introduction …", "▶ Regarder VivaPlan en 2 minutes", "VivaPlan en 2 minutes", "VivaPlan en 10 minutes"],
      it: ["Video introduttivi …", "▶ Guardare VivaPlan in 2 minuti", "VivaPlan in 2 minuti", "VivaPlan in 10 minuti"]
    };
    for (const [lang, texts] of Object.entries(want)) {
      const t = await (await page.request.get(`/i18n/${lang}.json`)).json();
      expect([t["menu.help.videos"], t["welcome.video"], t["videos.short"], t["videos.long"]]).toEqual(texts);
      expect(t["videos.browser"]).toBeTruthy();
    }
  });

  test("AK6: help topic privacy says that only the videos and the donation link open the browser", async ({ page }) => {
    const errors = await open(page);
    const want = { de: ["Einführungsvideos", "VivaPlan unterstützen", "Browser"], en: ["introductory videos", "Support VivaPlan", "browser"],
      fr: ["vidéos d’introduction", "Soutenir VivaPlan", "navigateur"], it: ["video introduttivi", "Sostenere VivaPlan", "browser"] };
    for (const [lang, words] of Object.entries(want)) {
      await page.evaluate(l => I18n.use(l), lang);
      await page.evaluate(() => Help.show("privacy"));
      for (const w of words) await expect(page.locator("#help:not([hidden]) #helpBody")).toContainText(w);
    }
    await noErrors(page, errors);
  });
});
