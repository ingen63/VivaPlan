// #220: Theme of the user interface (setting appearance: system / light / dark). The system theme is emulated with
// `colorScheme`; the contrast is measured from the computed colors (WCAG ratio, at least 4.5:1).
const { test, expect } = require("@playwright/test");
const { open, noErrors, prefs } = require("./helpers");

const theme = page => page.locator("html").getAttribute("data-theme");

/** Opens the settings dialog, chooses a theme and saves. */
async function chooseTheme(page, value) {
  await page.locator('header [data-action="settings"]').click();
  await page.locator("#set-theme").selectOption(value);
  await page.locator("#settingsSave").click();
  await expect(page.locator("#settings")).not.toBeVisible();
}

/**
 * Contrast of the text of the elements against their effective background (first non-transparent background of the
 * element or its ancestors). Returns [{ what, ratio }] for every selector that exists.
 */
const contrasts = (page, selectors) => page.evaluate(selectors => {
  const rgb = s => { const m = s.match(/[\d.]+/g).map(Number); return { r: m[0], g: m[1], b: m[2], a: m.length > 3 ? m[3] : 1 }; };
  const lum = ({ r, g, b }) => {
    const f = c => { c /= 255; return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4); };
    return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b);
  };
  const background = el => {
    for (let e = el; e; e = e.parentElement) {
      const c = rgb(getComputedStyle(e).backgroundColor);
      if (c.a > 0.5) return c;
    }
    return rgb(getComputedStyle(document.body).backgroundColor);
  };
  const out = [];
  for (const [what, sel, prop = "color"] of selectors) {
    const el = document.querySelector(sel);
    if (!el) { out.push({ what, ratio: null }); continue; }
    const fg = lum(rgb(getComputedStyle(el)[prop])), bg = lum(background(el));
    out.push({ what, ratio: Math.round((Math.max(fg, bg) + 0.05) / (Math.min(fg, bg) + 0.05) * 100) / 100 });
  }
  return out;
}, selectors);

/** The most important elements: page, menu, fields, select and option, buttons. */
const MAIN = [
  ["body", "body"], ["header", "header .brand"], ["menu", "#nav a:not(.active)"], ["menu active", "#nav a.active"],
  ["field label", "#main .field>span"], ["input", "#main .inp input"], ["select", "#main select"],
  ["option", "#main select option"], ["button", "header .btn:not(.primary)"], ["primary button", "header .btn.primary"],
  ["hint", "#main .lead"], ["preview text", "aside .sub"], ["help button", "#main .hq"],
];
const DIALOG = [["dialog title", "#settings[open] b, #settings[open] h2, #settings[open] .field>span"], ["dialog select", "#settings[open] #set-theme"],
  ["dialog option", "#settings[open] #set-theme option"], ["dialog button", "#settings[open] .btn:not(.primary)"], ["dialog primary", "#settings[open] .btn.primary"]];
const HELP = [["help text", "#help:not([hidden]) #helpBody p"], ["help title", "#help:not([hidden]) #helpBody h1"], ["help search", "#help:not([hidden]) .hh input"]];

async function expectReadable(page, selectors, choice) {
  const all = await contrasts(page, selectors);
  for (const { what, ratio } of all) expect(ratio, what).not.toBeNull();
  for (const { what, ratio } of all) {
    // #235/AK2: no exception any more, 4.5:1 for every element in both themes (choice only names the theme)
    expect.soft(ratio, `contrast of «${what}» (${choice})`).toBeGreaterThanOrEqual(4.5);
  }
}

test.describe("Theme light and dark (#220)", () => {
  test("AK1: setting appearance with system / light / dark, default system, kept after a restart", async ({ page }) => {
    const errors = await open(page);
    await page.locator('header [data-action="settings"]').click();
    const sel = page.locator("#set-theme");
    await expect(page.locator("#settings label.field", { has: sel }).locator("span")).toHaveText("Darstellung");
    await expect(sel.locator("option")).toHaveText(["System", "Hell", "Dunkel"]);
    await expect(sel).toHaveValue("system");
    await page.locator("#settings .btn[value=cancel]").click();
    await chooseTheme(page, "dark");
    expect((await prefs(page)).theme).toBe("dark");
    expect(await theme(page)).toBe("dark");
    // restart: the choice comes back from the settings
    await page.reload();
    await page.waitForSelector("html[data-ready='1']");
    expect(await theme(page)).toBe("dark");
    await page.locator('header [data-action="settings"]').click();
    await expect(page.locator("#set-theme")).toHaveValue("dark");
    // «Standardwerte» sets it back to System
    await page.locator("#settingsDefaults").click();
    await page.locator("#ask .btn.primary").click();
    await expect(page.locator("#set-theme")).toHaveValue("system");
    expect((await prefs(page)).theme).toBe("system");
    await noErrors(page, errors);
  });

  test("AK1: in the desktop app the choice is stored in the app settings", async ({ page }) => {
    await page.addInitScript(() => {
      window.__settings = { disclaimerAccepted: 2, theme: "dark" };
      window.__TAURI__ = {
        core: {
          invoke: async (cmd, args = {}) => {
            if (cmd === "settings_get") return { ...window.__settings };
            if (cmd === "settings_set") { window.__settings[args.key] = args.value; return null; }
            if (cmd === "first_start_language") return null;
            if (cmd === "startup_plan") return { path: "/x/Familie.vplan", plan: await (await fetch("/api/example_plan", { method: "POST", body: "{}" })).json(), notice: null, first: false };
            if (["set_dirty", "quit_app", "set_window_title", "set_menu_language"].includes(cmd)) return null;
            const r = await fetch(`/api/${cmd}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(args) });
            const j = await r.json(); if (!r.ok) throw new Error(j.error); return j;
          },
        },
        event: { listen: () => Promise.resolve(() => {}) },
      };
    });
    await page.goto("/#nobench");
    await page.waitForSelector("html[data-ready='1']");
    expect(await theme(page)).toBe("dark");
    await chooseTheme(page, "light");
    expect(await page.evaluate(() => window.__settings.theme)).toBe("light");
    expect(await theme(page)).toBe("light");
  });

  test("AK2: «System» follows prefers-color-scheme, also when it changes at runtime", async ({ page }) => {
    await page.emulateMedia({ colorScheme: "dark" });
    const errors = await open(page);
    expect(await theme(page)).toBe("dark");
    await page.emulateMedia({ colorScheme: "light" });
    await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
    await page.emulateMedia({ colorScheme: "dark" });
    await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
    // a fixed choice does not follow the system
    await chooseTheme(page, "light");
    await page.emulateMedia({ colorScheme: "light" });
    await page.emulateMedia({ colorScheme: "dark" });
    expect(await theme(page)).toBe("light");
    await noErrors(page, errors);
  });

  for (const system of ["light", "dark"]) {
    for (const choice of ["light", "dark"]) {
      test(`AK3 AK6: theme ${choice} under system ${system} – page, select, dialog and help readable`, async ({ page }) => {
        await page.emulateMedia({ colorScheme: system });
        const errors = await open(page);
        await chooseTheme(page, choice);
        expect(await theme(page)).toBe(choice);
        expect(await page.evaluate(() => getComputedStyle(document.documentElement).colorScheme)).toBe(choice);
        await expectReadable(page, MAIN, choice);
        await page.locator('header [data-action="settings"]').click();
        await expectReadable(page, DIALOG, choice);
        await page.locator("#settings .btn[value=cancel]").click();
        await page.evaluate(() => Help.show("f-settings"));
        await expect(page.locator("#help:not([hidden]) #helpBody h1")).toBeVisible();
        await expectReadable(page, HELP, choice);
        await noErrors(page, errors);
      });
    }
  }

  test("AK4: charts in the dark theme – axis labels readable, grid and lines distinct from the background", async ({ page }) => {
    await page.emulateMedia({ colorScheme: "dark" });
    const errors = await open(page);
    expect(await theme(page)).toBe("dark");
    const res = await page.evaluate(() => {
      const rgb = s => {
        const c = document.createElement("canvas").getContext("2d"); c.fillStyle = s; const h = c.fillStyle;
        if (h.startsWith("#")) return [1, 3, 5].map(i => parseInt(h.slice(i, i + 2), 16));
        return h.match(/[\d.]+/g).slice(0, 3).map(Number);
      };
      const lum = ([r, g, b]) => { const f = c => { c /= 255; return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4); }; return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b); };
      const ratio = (a, b) => { const x = lum(rgb(a)), y = lum(rgb(b)); return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05); };
      const bg = getComputedStyle(document.querySelector("aside")).backgroundColor;
      const svg = document.querySelector("#chart");
      const texts = [...svg.querySelectorAll("text")].map(t => ratio(t.getAttribute("fill"), bg));
      const strokes = [...svg.querySelectorAll("path[stroke], line[stroke]")].map(p => p.getAttribute("stroke")).filter(s => s && s !== "none");
      const fills = [...svg.querySelectorAll("path[fill]")].map(p => p.getAttribute("fill")).filter(s => s && s !== "none");
      return { texts, strokes: strokes.map(s => ratio(s, bg)), fills: fills.map(s => ratio(s, bg)), panelDark: lum(rgb(bg)) < 0.05 };
    });
    expect(res.panelDark).toBe(true);
    expect(res.texts.length).toBeGreaterThan(0);
    for (const r of res.texts) expect(r).toBeGreaterThanOrEqual(4.5);
    expect(res.strokes.length).toBeGreaterThan(0);
    // grid lines are faint on purpose, but visible; lines and areas stand out clearly
    for (const r of res.strokes) expect(r).toBeGreaterThan(1.1);
    expect(Math.max(...res.strokes)).toBeGreaterThanOrEqual(3);
    for (const r of res.fills) expect(r).toBeGreaterThan(1.1);
    await noErrors(page, errors);
  });

  test("AK5: the light theme looks the same under a light and a dark system theme", async ({ page }) => {
    const look = () => page.evaluate(() => {
      const props = ["color", "backgroundColor", "borderTopColor", "colorScheme", "fill", "stroke"];
      const els = [document.documentElement, document.body, ...document.querySelectorAll("header *, #nav *, #main *, aside *")];
      return els.map(e => { const cs = getComputedStyle(e); return props.map(p => cs[p]).join("|"); }).join("\n")
        + [...document.querySelectorAll("svg [fill], svg [stroke]")].map(e => `${e.getAttribute("fill")}|${e.getAttribute("stroke")}`).join("\n");
    });
    await page.emulateMedia({ colorScheme: "light" });
    const errors = await open(page);
    await chooseTheme(page, "light");
    const underLight = await look();
    await page.emulateMedia({ colorScheme: "dark" });
    await page.reload();
    await page.waitForSelector("html[data-ready='1']");
    await expect(page.locator("#chart path").first()).toBeAttached();
    expect(await theme(page)).toBe("light");
    expect(await look()).toBe(underLight);
    await noErrors(page, errors);
  });
});

// #235: contrast of the active menu entry in the light theme (own variable --accent-text, nothing else changes).
test.describe("Active menu entry in the light theme (#235)", () => {
  test("AK1: active entry at least 4.5:1 in the light theme and clearly marked as active", async ({ page }) => {
    await page.emulateMedia({ colorScheme: "light" });
    const errors = await open(page);
    await chooseTheme(page, "light");
    const [active] = await contrasts(page, [["menu active", "#nav a.active"]]);
    expect(active.ratio).toBeGreaterThanOrEqual(4.5);
    const look = await page.evaluate(() => {
      const cs = el => getComputedStyle(document.querySelector(el));
      const a = cs("#nav a.active"), n = cs("#nav a:not(.active)");
      return { aBg: a.backgroundColor, nBg: n.backgroundColor, aW: a.fontWeight, nW: n.fontWeight, aC: a.color, nC: n.color };
    });
    expect(look.aBg).not.toBe(look.nBg);
    expect(look.aC).not.toBe(look.nC);
    expect(Number(look.aW)).toBeGreaterThan(Number(look.nW));
    await noErrors(page, errors);
  });

  test("AK2: the test of #220 has no exception left (4.5:1 for all elements in both themes)", async () => {
    const src = require("fs").readFileSync(__filename, "utf8");
    expect(src).not.toContain("LIGHT_" + "EXCEPTIONS");
  });

  test("AK3: only the active entry changes – colors of the light theme and the dark theme as before", async ({ page }) => {
    const errors = await open(page);
    const vars = () => page.evaluate(() => {
      const cs = getComputedStyle(document.documentElement);
      return ["--accent", "--accent-soft", "--accent-text", "--text", "--bg", "--panel", "--muted"].map(v => cs.getPropertyValue(v).trim());
    });
    const active = () => page.evaluate(() => getComputedStyle(document.querySelector("#nav a.active")).color);
    await chooseTheme(page, "light");
    expect(await vars()).toEqual(["#2f6fdf", "#e8f0fd", "#2864d0", "#1c2230", "#f4f5f7", "#ffffff", "#667085"]);
    expect(await active()).toBe("rgb(40, 100, 208)");
    await chooseTheme(page, "dark");
    // dark theme unchanged: the active entry keeps the accent color #6c9cf5
    expect(await vars()).toEqual(["#6c9cf5", "#1f2b44", "#6c9cf5", "#e6e9ef", "#12151b", "#1a1e26", "#9aa3b2"]);
    expect(await active()).toBe("rgb(108, 156, 245)");
    await noErrors(page, errors);
  });
});
