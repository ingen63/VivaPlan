// #213: menu Help «Mit KI verbinden»: instructions for Claude Desktop, Claude Code, ChatGPT Desktop and LM Studio, each
// with the place of the configuration and a setup line to copy. The app only shows and copies (AK4): no command of the
// desktop app besides app_info, no request outside the development server.
// Desktop app stand-in: every call recorded, app_info with a given path of the command line; the clipboard is
// replaced so that the test reads what was copied.
const { test, expect } = require("@playwright/test");
const { open, noErrors, previewIdle } = require("./helpers");

const MAC_CLI = "/Applications/VivaPlan.app/Contents/MacOS/vivaplan";
const WIN_CLI = "C:\\Program Files\\VivaPlan\\vivaplan.exe";

const PLAN_DIR = "/Users/test/Planungen";

async function openApp(page, cliPath, planDir = PLAN_DIR, picked = null) {
  await page.addInitScript(([cli, dir, chosen]) => {
    window.__calls = []; window.__listeners = {}; window.__copied = [];
    Object.defineProperty(navigator, "clipboard", { value: { writeText: async t => { window.__copied.push(t); } } });
    window.__TAURI__ = {
      core: {
        invoke: async (cmd, args = {}) => {
          window.__calls.push(cmd);
          const example = async () => (await fetch("/api/example_plan", { method: "POST", body: "{}" })).json();
          if (cmd === "settings_get") return { disclaimerAccepted: 2, updateCheck: false };
          if (cmd === "first_start_language") return null;
          if (cmd === "startup_plan") return { path: "/x/Familie.vplan", plan: await example(), notice: null, first: false };
          if (cmd === "app_info") return { version: "1.2.3", buildEpoch: 1, cliPath: cli, planDir: dir };
          if (cmd === "pick_plan_folder") { window.__picked = args; return chosen; }
          if (["settings_set", "set_dirty", "quit_app", "set_window_title", "set_menu_language", "open_link"].includes(cmd)) return null;
          const res = await fetch(`/api/${cmd}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(args) });
          const j = await res.json(); if (!res.ok) throw new Error(j.error); return j;
        }
      },
      event: { listen: (name, cb) => { (window.__listeners[name] ??= []).push(cb); return Promise.resolve(() => {}); } }
    };
    window.__menu = id => { for (const cb of window.__listeners.menu || []) cb({ payload: id }); };
  }, [cliPath, planDir, picked]);
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
    await previewIdle(page); // the preview of the start computes on its own; only the dialog's commands count below
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
    const json = { mcpServers: { vivaplan: { command: MAC_CLI, args: ["mcp", "--allow-dir", PLAN_DIR] } } };
    const want = {
      claudeDesktop: { line: JSON.stringify(json), config: "claude_desktop_config.json" },
      claudeCode: { line: `claude mcp add --scope user vivaplan -- ${MAC_CLI} mcp --allow-dir ${PLAN_DIR}`, config: "Terminal" },
      chatgpt: { line: `[mcp_servers.vivaplan]\ncommand = '${MAC_CLI}'\nargs = ["mcp", "--allow-dir", "${PLAN_DIR}"]`, config: "/.codex/config.toml" },
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
    const errors = await openApp(page, WIN_CLI, "C:\\Users\\test\\Meine Planungen");
    await openDialog(page);
    const line = id => page.locator(`section[data-program="${id}"] pre`).textContent();
    expect(JSON.parse(await line("claudeDesktop")).mcpServers.vivaplan.command).toBe(WIN_CLI);
    expect(JSON.parse(await line("lmstudio")).mcpServers.vivaplan.command).toBe(WIN_CLI);
    expect(await line("claudeCode")).toBe(`claude mcp add --scope user vivaplan -- "${WIN_CLI}" mcp --allow-dir "C:\\Users\\test\\Meine Planungen"`);
    // TOML literal string: backslashes stay as they are; the arguments are basic strings with escaped backslashes
    expect(await line("chatgpt")).toContain(`command = '${WIN_CLI}'`);
    expect(await line("chatgpt")).toContain(`args = ["mcp", "--allow-dir", "C:\\\\Users\\\\test\\\\Meine Planungen"]`);
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
    await expect(page.locator("#connectAi")).toContainText("lokalen Modell in LM Studio bleibt alles auf diesem Rechner");
    await page.locator('#connectAi a[data-topic="privacy"]').click();
    await expect(page.locator("#ask")).not.toHaveAttribute("open", "");
    await expect(page.locator("#help:not([hidden]) #helpBody")).toContainText("lokalen Modell");
    await openDialog(page);
    await page.locator("#connectAi").getByRole("link", { name: "Mehr zum MCP-Server" }).click();
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

test.describe("Plan folder for every AI (#263)", () => {
  const lines = page => page.locator("#connectAi section.aiprog pre").allTextContents();
  const argsOf = async page => {
    const [desktop, code, chatgpt, lm] = await lines(page);
    return {
      desktop: JSON.parse(desktop).mcpServers.vivaplan.args, lm: JSON.parse(lm).mcpServers.vivaplan.args,
      code: code.split(" -- ")[1], chatgpt: JSON.parse(chatgpt.split("args = ")[1]),
    };
  };

  test("AK4: the folder of the last plan is suggested and set as --allow-dir into all four setup lines", async ({ page }) => {
    const errors = await openApp(page, MAC_CLI);
    await openDialog(page);
    await expect(page.locator("#aiDir")).toHaveText(PLAN_DIR);
    await expect(page.locator("#connectAi")).toContainText("nur in diesem Ordner und seinen Unterordnern");
    const want = ["mcp", "--allow-dir", PLAN_DIR];
    const a = await argsOf(page);
    expect(a.desktop).toEqual(want);
    expect(a.lm).toEqual(want);
    expect(a.chatgpt).toEqual(want);
    expect(a.code).toBe(`${MAC_CLI} ${want.join(" ")}`);
    // the copied line is the shown one
    const s = page.locator('section[data-program="claudeCode"]');
    expect(await copyOf(page, s)).toBe(await s.locator("pre").textContent());
    await noErrors(page, errors);
  });

  test("AK4: «Choose folder …» opens the folder dialog at the shown folder and puts the chosen one into every line", async ({ page }) => {
    const chosen = "/Users/test/Andere Pläne";
    const errors = await openApp(page, MAC_CLI, PLAN_DIR, chosen);
    await openDialog(page);
    const button = page.locator('#connectAi [data-action="ai-folder"]');
    await expect(button).toHaveText("Ordner wählen …");
    await button.click();
    await expect(page.locator("#aiDir")).toHaveText(chosen);
    expect(await page.evaluate(() => window.__picked)).toEqual({ start: PLAN_DIR });
    const a = await argsOf(page);
    for (const args of [a.desktop, a.lm, a.chatgpt]) expect(args).toEqual(["mcp", "--allow-dir", chosen]);
    expect(a.code).toBe(`${MAC_CLI} mcp --allow-dir "${chosen}"`);
    expect(await copyOf(page, page.locator('section[data-program="lmstudio"]'))).toContain(chosen);
    await noErrors(page, errors);
  });

  test("AK4: cancelling the folder dialog keeps the folder; without a last plan the documents folder", async ({ page }) => {
    const errors = await openApp(page, MAC_CLI, "/Users/test/Documents", null);
    await openDialog(page);
    await expect(page.locator("#aiDir")).toHaveText("/Users/test/Documents");
    await page.locator('#connectAi [data-action="ai-folder"]').click();
    await expect.poll(() => page.evaluate(() => window.__picked)).toEqual({ start: "/Users/test/Documents" });
    await expect(page.locator("#aiDir")).toHaveText("/Users/test/Documents");
    expect((await argsOf(page)).desktop).toEqual(["mcp", "--allow-dir", "/Users/test/Documents"]);
    await noErrors(page, errors);
  });

  test("AK5: switch «read only», off by default, adds --read-only to every line and removes it again", async ({ page }) => {
    const errors = await openApp(page, MAC_CLI);
    await openDialog(page);
    const box = page.locator("#aiReadOnly");
    await expect(box).not.toBeChecked();
    await expect(page.locator("#connectAi .aiopt")).toContainText("nicht speichern");
    await box.check();
    let a = await argsOf(page);
    for (const args of [a.desktop, a.lm, a.chatgpt]) expect(args).toEqual(["mcp", "--allow-dir", PLAN_DIR, "--read-only"]);
    expect(a.code).toMatch(/ --read-only$/);
    expect(await copyOf(page, page.locator('section[data-program="chatgpt"]'))).toContain('"--read-only"]');
    await box.uncheck();
    a = await argsOf(page);
    expect(a.desktop).toEqual(["mcp", "--allow-dir", PLAN_DIR]);
    await noErrors(page, errors);
  });

  test("AK4, AK5: the folder row and the switch in all four languages, without unknown texts", async ({ page }) => {
    const errors = await openApp(page, MAC_CLI);
    const want = { de: "Ordner wählen …", en: "Choose folder …", fr: "Choisir le dossier …", it: "Scegliere la cartella …" };
    for (const [lang, label] of Object.entries(want)) {
      await page.evaluate(l => I18n.use(l), lang);
      await openDialog(page);
      await expect(page.locator('#connectAi [data-action="ai-folder"]')).toHaveText(label);
      await expect(page.locator("#connectAi")).not.toContainText(/\b(ai|dialog)\.[a-zA-Z]/);
      if (await page.evaluate(() => IS_MAC)) await expect(page.locator('section[data-program="claudeDesktop"]')).toContainText("Plan folder");
      await page.evaluate(() => $("#ask").close());
    }
    await noErrors(page, errors);
  });
});

test.describe("Incognito chats and privacy (#298)", () => {
  test("AK6, AK3: the dialog names incognito chats and links to the paragraph in «Connecting AI programs»", async ({ page }) => {
    const errors = await openApp(page, MAC_CLI);
    await openDialog(page);
    await expect(page.locator("#aiIncognito")).toContainText("In Inkognito-Chats von Claude steht VivaPlan nicht zur Verfügung");
    const link = page.locator('#aiIncognito a[data-topic="ref-mcp"]');
    await expect(link).toHaveText("Inkognito-Chats");
    await link.click();
    await expect(page.locator("#ask")).not.toHaveAttribute("open", "");
    const body = page.locator("#help:not([hidden]) #helpBody");
    await expect(body).toContainText("Inkognito-Chats:");
    await expect(body).toContainText("VivaPlan kann nichts dagegen tun");
    await noErrors(page, errors);
  });

  test("AK4: the privacy note says that names are replaced, amounts and dates go to the provider", async ({ page }) => {
    const errors = await openApp(page, MAC_CLI);
    await openDialog(page);
    const note = page.locator("#connectAi").getByText(/^Datenschutz:/);
    await expect(note).toContainText("ersetzt Namen von Personen");
    await expect(note).toContainText("Beträge und Daten wie Geburtsdaten gehen unverändert an den Anbieter");
    await noErrors(page, errors);
  });

  test("AK2, AK5: help paragraph «Incognito chats» with its alternatives and keyword, four languages", async ({ page }) => {
    const errors = await openApp(page, MAC_CLI);
    const want = {
      de: ["Inkognito-Chats:", "VivaPlan kann nichts dagegen tun", "normalen Chat", "Gedächtnis", "LM Studio", "Inkognito-Chats"],
      en: ["Incognito chats:", "VivaPlan cannot do anything about it", "normal chat", "memory", "LM Studio", "Incognito chats"],
      fr: ["Chats incognito :", "VivaPlan n’y peut rien", "chat normal", "mémoire", "LM Studio", "Chats incognito"],
      it: ["Chat in incognito:", "VivaPlan non può farci nulla", "chat normale", "memoria", "LM Studio", "Chat in incognito"] };
    for (const [lang, [head, cannot, normal, memory, local, linkText]] of Object.entries(want)) {
      await page.evaluate(l => I18n.use(l), lang);
      await page.evaluate(() => Help.show("ref-mcp"));
      const body = page.locator("#help:not([hidden]) #helpBody");
      for (const w of [head, cannot, normal, memory, local]) await expect(body).toContainText(w);
      await page.evaluate(q => Help.show("search:" + q), lang === "de" ? "inkognito" : "incognito");
      await expect(page.locator('#help:not([hidden]) #helpBody a[data-topic="ref-mcp"]').first()).toBeVisible();
      await openDialog(page);
      await expect(page.locator('#aiIncognito a[data-topic="ref-mcp"]')).toHaveText(linkText);
      await expect(page.locator("#connectAi")).not.toContainText(/\bai\.[a-zA-Z]/);
      await page.evaluate(() => $("#ask").close());
    }
    expect(await page.evaluate(() => Help.check())).toEqual([]);
    await noErrors(page, errors);
  });
});
