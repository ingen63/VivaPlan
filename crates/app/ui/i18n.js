// #170: Multilingual user interface. All texts come from the text tables `i18n/<language>.json` (flat keys,
// placeholders `{name}`); `i18n/languages.json` lists the languages available, each named in its own language.
// The choice («system» or a language) is a setting of the app, not part of the plan. Numbers and dates stay in the
// Swiss format in every language (util.js formats with "de-CH").
//
// Loaded first (after api.js); it loads the tables and then the other scripts of the page, so that every script can
// use `tr()` already while it is evaluated. Constants holding texts are defined with `onLanguage(() => X = …)`; they
// are defined again after a change of language, and the views are drawn again (`I18n.onChange`).
"use strict";
const I18n = (() => {
  const FALLBACK = "de";
  const SYSTEM = "system";
  const SETTING = "language";
  const inApp = !!(window.__TAURI__ && window.__TAURI__.core);
  const invoke = (cmd, args) => window.__TAURI__.core.invoke(cmd, args);
  const isMac = /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);
  let languages = { [FALLBACK]: "Deutsch" };
  let lang = FALLBACK, choice = SYSTEM, base = {}, table = {};
  const tables = new Map();
  const definitions = [], listeners = [];
  const reported = new Set();

  const report = msg => { if (typeof noteError === "function") noteError(msg); else console.error(msg); };

  /** Replaces the placeholders `{name}` with the values of `params`; unknown placeholders stay. */
  const format = (text, params) => text.replace(/\{(\w+)\}/g, (m, k) => (k in params ? String(params[k]) : m));

  /** Text of the key in the current language (fallback: German); an unknown key is reported and shown as such. */
  function tr(key, params) {
    const text = table[key] ?? base[key];
    if (text === undefined) {
      if (!reported.has(key)) { reported.add(key); report(`i18n: unknown key ${key}`); }
      return key;
    }
    return params ? format(text, params) : text;
  }

  /** Shortcuts in texts are written for the Mac (⌘, ⇧); other systems show Ctrl and Shift. */
  const shortcuts = s => (isMac ? s : s.replace(/⇧⌘/g, "Ctrl+Shift+").replace(/⌘↩/g, "Ctrl+Enter").replace(/⌘/g, "Ctrl+"));

  /** Language part of a locale: "fr_FR.UTF-8" → "fr", "de-CH" → "de". */
  const languageOf = s => String(s || "").toLowerCase().split(/[-_.@]/)[0];
  /** First available language of the candidates, otherwise German. */
  function pick(candidates) {
    for (const c of candidates) { const l = languageOf(c); if (Object.hasOwn(languages, l)) return l; }
    return FALLBACK;
  }

  async function fetchJson(path) {
    const r = await fetch(path);
    if (!r.ok) throw new Error(`${path}: ${r.status}`);
    return r.json();
  }
  async function tableOf(l) {
    if (!tables.has(l)) tables.set(l, await fetchJson(`i18n/${l}.json`));
    return tables.get(l);
  }

  /** Language for a choice. «system»: at the very first start the language of the operating system determined by
   *  the app (sys-locale), otherwise the languages of the web view. */
  let firstStartLanguage = null;
  async function resolve(c) {
    if (c !== SYSTEM) return pick([c]);
    if (firstStartLanguage) return pick([firstStartLanguage]);
    return pick([...(navigator.languages || []), navigator.language]);
  }

  async function readChoice() {
    try {
      const s = inApp ? await invoke("settings_get") : JSON.parse(localStorage.getItem("rp.settings") || "{}");
      return typeof s?.[SETTING] === "string" ? s[SETTING] : SYSTEM;
    } catch { return SYSTEM; }
  }
  function saveChoice(c) {
    // The settings of app.js (Pref) keep all settings in one object; saved through it, nothing is overwritten.
    if (typeof Pref === "object" && Pref.cache) { Pref.set(SETTING, c); return; }
    if (inApp) { invoke("settings_set", { key: SETTING, value: c }).catch(e => report(e)); return; }
    try {
      const s = JSON.parse(localStorage.getItem("rp.settings") || "{}");
      s[SETTING] = c;
      localStorage.setItem("rp.settings", JSON.stringify(s));
    } catch { /* then it applies only to this session */ }
  }

  /** Static texts of the page: `data-i18n` (content, may contain markup), `data-i18n-title`, `-aria-label`, `-placeholder`. */
  function applyStatic(root = document) {
    for (const el of root.querySelectorAll("[data-i18n]")) el.innerHTML = tr(el.dataset.i18n);
    for (const attr of ["title", "aria-label", "placeholder"]) {
      const data = "data-i18n-" + attr;
      for (const el of root.querySelectorAll(`[${data}]`)) el.setAttribute(attr, shortcuts(tr(el.getAttribute(data))));
    }
  }

  /** Switches the language. Plan, unsaved changes and view stay; only the texts are drawn again. */
  async function use(c, { save = true } = {}) {
    const next = c === SYSTEM || Object.hasOwn(languages, c) ? c : SYSTEM;
    const l = await resolve(next);
    base = await tableOf(FALLBACK);
    table = l === FALLBACK ? base : await tableOf(l);
    choice = next; lang = l;
    document.documentElement.lang = `${l}-CH`;
    document.documentElement.dataset.language = l;
    if (save) saveChoice(choice);
    applyStatic();
    for (const d of definitions) d();
    for (const f of listeners) { try { f(); } catch (e) { report(e?.message || e); } }
    if (inApp) invoke("set_menu_language", { language: lang, choice }).catch(e => report(e));
  }

  /** Loads the tables in the language chosen and then the scripts of the page (in this order). */
  async function boot(scripts) {
    try { languages = await fetchJson("i18n/languages.json"); } catch (e) { report(e.message); }
    const saved = await readChoice();
    if (inApp && saved === SYSTEM) { try { firstStartLanguage = await invoke("first_start_language"); } catch { /* none */ } }
    try { await use(saved, { save: false }); } catch (e) { report(e.message); }
    firstStartLanguage = null;
    for (const src of scripts) {
      await new Promise((ok, fail) => {
        const s = document.createElement("script");
        s.src = src; s.onload = ok; s.onerror = () => fail(new Error(src));
        document.body.appendChild(s);
      });
    }
  }

  // The native menu «Ansicht → Sprache» reports its entries as «language-<choice>».
  window.__TAURI__?.event?.listen("menu", ev => {
    const m = /^language-(\w+)$/.exec(ev.payload);
    if (m) use(m[1]).catch(e => report(e.message));
  });

  return {
    SYSTEM, tr, format, shortcuts, use, boot, applyStatic, languageOf, pick,
    language: () => lang,
    choice: () => choice,
    languages: () => ({ ...languages }),
    /** Runs `define` now and again after every change of language (constants holding texts). */
    onLanguage(define) { definitions.push(define); define(); },
    /** Runs `f` after every change of language (drawing the views again). */
    onChange(f) { listeners.push(f); },
  };
})();
const tr = I18n.tr;
const onLanguage = define => I18n.onLanguage(define);
// The page names the scripts to load after the tables (attribute `data-load`; no inline script because of the CSP).
I18n.boot((document.currentScript?.dataset.load || "").split(/\s+/).filter(Boolean));
