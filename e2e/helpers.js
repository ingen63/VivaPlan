// Gemeinsame Hilfsfunktionen der Oberflächentests (e2e/ui-*.spec.js).
const { expect } = require("@playwright/test");

const MOD = process.platform === "darwin" ? "Meta" : "Control";
const SECTIONS = ["Haushalt & Projekt", "Meilensteine", "Vorsorge", "Säule 3a", "Einnahmen & Ausgaben", "Vermögen & Anlage",
  "Simulation", "Entnahmestrategie", "Optimierung", "Steuern", "Immobilien", "Szenarien & Simulation"];

/**
 * Öffnet die Oberfläche (optional mit Testzugang über die Adresse) und sammelt JavaScript-Fehler.
 * #111: ohne Messung der Rechenzeit (#nobench), denn sie liefe sonst in jedem Test im Hintergrund mit und machte ihn von
 * der Rechnerlast abhängig. Tests, die die echte Messung prüfen, öffnen mit `{ bench: true }`.
 */
async function open(page, hash = "", { bench = false } = {}) {
  const errors = [];
  page.on("pageerror", e => errors.push(e.message));
  const h = bench ? hash : [hash, "nobench"].filter(Boolean).join("/");
  await page.goto("/" + (h ? "#" + h : ""));
  await page.waitForSelector("html[data-ready='1']");
  await expect(page.locator("#chart path").first()).toBeAttached();
  return errors;
}
const noErrors = async (page, errors) => {
  expect(errors).toEqual([]);
  expect(await page.evaluate(() => window.__errors)).toEqual([]);
};
const field = (page, p) => page.locator(`input[data-path="${p}"]`);
/** #111: wartet, bis keine Vorschau mehr aussteht (sonst nimmt eine verzögerte Vorschau die nächste Eingabe halb mit). */
const previewIdle = page => expect(page.locator("html")).toHaveAttribute("data-preview", "idle");
/** #111: Klick ins Menü. Jede Vorschau zeichnet das Menü neu; ein Klick, der in eine Vorschau fällt, ginge verloren. */
const nav = async (page, title) => { await previewIdle(page); await page.locator("#nav a", { hasText: title }).first().click(); };
const kpiValue = (page, label) => page.locator(`#kpis .kpi:has(.k:text-is("${label}")) .v`);
const resultKpi = (page, label) => page.locator(`#resultsBody .kpi:has(.k:text-is("${label}")) .v`);
const parseNum = s => Number(s.replace(/[’'\sk]/g, ""));
/** Stellt die Optimierung ein (z. B. kleine Rechnung) und gibt zurück, was in den Feldern steht. Die Werte gehen über die
 *  Planung (`state.optimization`), denn `saveOptCfg` liest die Felder und überschreibt ein direkt gesetztes `optCfg` (#79). */
const setOpt = (page, cfg) => page.evaluate(cfg => { state.optimization = optToPlan({ ...optCfg, ...cfg }); renderSection(); return readOptCfg(); }, cfg);
/** Tabellenzeile eines Meilensteins (Name steht in einem Eingabefeld, deshalb über den Index der ID). */
async function msRow(page, id) {
  const i = await page.evaluate(id => state.milestones.findIndex(m => m.id === id), id);
  // Feste Meilensteine (#51) haben keinen änderbaren Namen; jede Zeile hat aber die Auswahl «Festgelegt als».
  return page.locator(`#main table.tbl tr:has(select[data-path="milestones.${i}.kind"])`);
}

// ---------------------------------------------------------------------------------------------
const welcome = page => page.locator("#welcome");
const steps = page => page.locator("#steps");
const helpTitle = page => page.locator("#help:not([hidden]) #helpBody h1").first();
const prefs = page => page.evaluate(() => JSON.parse(localStorage.getItem("rp.settings") || "{}"));

module.exports = { MOD, SECTIONS, open, noErrors, field, previewIdle, nav, kpiValue, resultKpi, parseNum, setOpt, msRow, welcome, steps, helpTitle, prefs };
