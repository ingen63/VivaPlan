// #213: menu Help «Mit KI verbinden»: instructions for Claude Desktop, Claude Code, ChatGPT Desktop and LM Studio, each
// with the place of the configuration and a setup line to copy. The app only shows and copies (AK4): no command of the
// desktop app besides app_info, no request outside the development server.
// Desktop app stand-in: every call recorded, app_info with a given path of the command line; the clipboard is
// replaced so that the test reads what was copied.
const { test, expect } = require("@playwright/test");
const { open, noErrors } = require("./helpers");

const MAC_CLI = "/Applications/VivaPlan.app/Contents/MacOS/vivaplan";
const WIN_CLI = "C:\\Program Files\\VivaPlan\\vivaplan.exe";

async function openApp(page, cliPath) {
  await page.addInitScript(cli => {
    window.__calls = []; window.__listeners = {}; window.__copied = [];
    Object.defineProperty(navigator, "clipboard", { value: { writeText: async t => { window.__copied.push(t); } } });
    window.__TAURI__ = {
      core: {
        invoke: async (cmd, args = {}) => {
          window.__calls.push(cmd);
          const example = async () => (await fetch("/api/example_plan", { method: "POST", body: "{}" })).json();
          if (cmd === "settings_get") return { disclaimerAccepted: 2 };
          if (cmd === "first_start_language") return null;
          if (cmd === "startup_plan") return { path: "/x/Familie.vplan", plan: await example(), notice: null, first: false };
          if (cmd === "app_info") return { version: "1.2.3", buildEpoch: 1, cliPath: cli };
          if (["settings_set", "set_dirty", "quit_app", "set_window_title", "set_menu_language", "open_link"].includes(cmd)) return null;
          const res = await fetch(`/api/${cmd}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(args) });
          const j = await res.json(); if (!res.ok) throw new Error(j.error); return j;
        }
      },
      event: { listen: (name, cb) => { (window.__listeners[name] ??= []).push(cb); return Promise.resolve(() => {}); } }
    };
    window.__menu = id => { for (const cb of window.__listeners.menu || []) cb({ payload: id }); };
  }, cliPath);
  const errors = [];
  page.on("pageerror", e => errors.push(e.message));
  await page.goto("/#nobench");
  await page.waitForSelector("html[data-ready='1']");
  return errors;
}

/** Records every request that leaves the local development server (AK4: the app itself loads nothing). */
function external(page) {
  const out = [];
  page.on("request", r => { if (!/^http:\/\/127\.0\.0\.1:\d+\//.test(r.url()) && !r.url().startsWith("data:")) out.push(r.url()); });
  return out;
}

/** Opens the dialog through the menu; returns the commands sent to the desktop app meanwhile. */
async function openDialog(page) {
  const before = await page.evaluate(() => window.__calls.length);
  await page.evaluate(() => { lastCommand = {}; window.__menu("connectai"); });
  await expect(page.locator("#ask")).toHaveAttribute("open", "");
  await expect(page.locator("#connectAi")).toBeVisible();
  return page.evaluate(n => window.__calls.slice(n), before);
}
const copyOf = async (page, locator) => {
  await locator.locator("button.aicopy").click();
  return page.evaluate(() => window.__copied.at(-1));
};

test.describe("Connect to AI (#213)", () => {
  test("AK2: menu Help «connectai» shows four programs, each with configuration and setup line to copy", async ({ page }) => {
    const ext = external(page);
    const errors = await openApp(page, MAC_CLI);
    const calls = await openDialog(page);
    await expect(page.locator("#askTitle")).toHaveText("Mit KI verbinden");
    const d = page.locator("#connectAi");
    await expect(d).toContainText("richtet nichts selbst ein");
    // path of the program on top, copyable
    const top = d.locator(":scope > .aicode");
    await expect(top.locator("pre")).toHaveText(MAC_CLI);
    expect(await copyOf(page, top)).toBe(MAC_CLI);
    await expect(top.locator("button.aicopy")).toHaveText("Kopiert ✓");
    await expect(top.locator("button.aicopy")).toHaveText("Kopieren", { timeout: 4000 });

    const sections = d.locator("section.aiprog");
    await expect(sections.locator("h4")).toHaveText(["Claude Desktop", "Claude Code", "ChatGPT Desktop", "LM Studio"]);
    const json = { mcpServers: { vivaplan: { command: MAC_CLI, args: ["mcp"] } } };
    const want = {
      claudeDesktop: { line: JSON.stringify(json), config: "claude_desktop_config.json" },
      claudeCode: { line: `claude mcp add --scope user vivaplan -- ${MAC_CLI} mcp`, config: "Terminal" },
      chatgpt: { line: `[mcp_servers.vivaplan]\ncommand = '${MAC_CLI}'\nargs = ["mcp"]`, config: "/.codex/config.toml" },
      lmstudio: { line: JSON.stringify(json), config: "/.lmstudio/mcp.json" },
    };
    for (const [id, w] of Object.entries(want)) {
      const s = d.locator(`section[data-program="${id}"]`);
      await expect(s).toContainText(w.config);
      await expect(s.locator("pre")).toHaveText(w.line);
      expect(await copyOf(page, s)).toBe(w.line);
      // the setup line names the program with its full path
      expect(w.line).toContain(MAC_CLI);
    }
    // the dialog stays open while copying; Claude Desktop names the one-click package (macOS) of the version
    await expect(page.locator("#ask")).toHaveAttribute("open", "");
    if (await page.evaluate(() => IS_MAC)) await expect(d.locator('section[data-program="claudeDesktop"]')).toContainText("VivaPlan_1.2.3_aarch64.mcpb");
    // AK4: nothing set up, nothing loaded: only app_info was asked, no request left the computer
    expect(calls).toEqual(["app_info"]);
    expect(ext).toEqual([]);
    await noErrors(page, errors);
  });

  test("AK2: a path with spaces stays one argument in every setup line", async ({ page }) => {
    const errors = await openApp(page, WIN_CLI);
    await openDialog(page);
    const line = id => page.locator(`section[data-program="${id}"] pre`).textContent();
    expect(JSON.parse(await line("claudeDesktop")).mcpServers.vivaplan.command).toBe(WIN_CLI);
    expect(JSON.parse(await line("lmstudio")).mcpServers.vivaplan.command).toBe(WIN_CLI);
    expect(await line("claudeCode")).toBe(`claude mcp add --scope user vivaplan -- "${WIN_CLI}" mcp`);
    // TOML literal string: backslashes stay as they are
    expect(await line("chatgpt")).toContain(`command = '${WIN_CLI}'`);
    await noErrors(page, errors);
  });

  test("AK2: without the path of the desktop app the dialog shows the usual path of the system", async ({ page }) => {
    const errors = await openApp(page, null);
    await openDialog(page);
    const usual = await page.evaluate(() => CLI_DEFAULT);
    expect(usual).toMatch(/vivaplan(\.exe)?$/);
    await expect(page.locator("#connectAi > .aicode pre")).toHaveText(usual);
    await expect(page.locator('section[data-program="claudeCode"] pre')).toContainText(usual);
    await noErrors(page, errors);
  });

  test("AK2, AK3: privacy note in the dialog, links to the help topics privacy and MCP server", async ({ page }) => {
    const errors = await openApp(page, MAC_CLI);
    await openDialog(page);
    await expect(page.locator("#connectAi")).toContainText("lokalen Modell in LM Studio bleibt es auf diesem Rechner");
    await page.locator('#connectAi a[data-topic="privacy"]').click();
    await expect(page.locator("#ask")).not.toHaveAttribute("open", "");
    await expect(page.locator("#help:not([hidden]) #helpBody")).toContainText("lokalen Modell");
    await openDialog(page);
    await page.locator('#connectAi a[data-topic="ref-mcp"]').click();
    await expect(page.locator("#help:not([hidden]) #helpBody")).toContainText("Mit KI verbinden");
    await noErrors(page, errors);
  });

  test("AK2: the dialog in all four languages, without unknown texts", async ({ page }) => {
    const errors = await openApp(page, MAC_CLI);
    const want = { de: "Mit KI verbinden", en: "Connect to AI", fr: "Connecter à une IA", it: "Collegare a un’IA" };
    for (const [lang, title] of Object.entries(want)) {
      await page.evaluate(l => I18n.use(l), lang);
      await openDialog(page);
      await expect(page.locator("#askTitle")).toHaveText(title);
      await expect(page.locator("#connectAi")).not.toContainText(/\bai\.[a-zA-Z]/);
      await expect(page.locator("#connectAi button.aicopy")).toHaveCount(5);
      const t = await (await page.request.get(`/i18n/${lang}.json`)).json();
      expect(t["menu.help.connectAi"]).toBe(`${title} …`);
      await page.evaluate(() => $("#ask").close());
    }
    await noErrors(page, errors);
  });

  test("AK3: help topic privacy: with a cloud AI data goes to its provider, with a local model it stays (four languages)", async ({ page }) => {
    const errors = await open(page);
    const want = { de: ["Cloud-KI", "Anbieter", "lokalen Modell", "bleiben sie auf Ihrem Rechner"],
      en: ["cloud AI", "provider", "local model", "stays on your computer"],
      fr: ["IA dans le cloud", "fournisseur", "modèle local", "restent sur votre ordinateur"],
      it: ["IA nel cloud", "fornitore", "modello locale", "restano sul Suo computer"] };
    for (const [lang, words] of Object.entries(want)) {
      await page.evaluate(l => I18n.use(l), lang);
      await page.evaluate(() => Help.show("privacy"));
      for (const w of words) await expect(page.locator("#help:not([hidden]) #helpBody")).toContainText(w);
    }
    expect(await page.evaluate(() => Help.check())).toEqual([]);
    await noErrors(page, errors);
  });

  test("AK2: in the browser (development server) the dialog opens with the usual path", async ({ page }) => {
    const ext = external(page);
    const errors = await open(page);
    await page.evaluate(() => showConnectAi());
    await expect(page.locator("#connectAi section.aiprog")).toHaveCount(4);
    await expect(page.locator("#connectAi > .aicode pre")).toHaveText(await page.evaluate(() => CLI_DEFAULT));
    expect(ext).toEqual([]);
    await noErrors(page, errors);
  });
});
