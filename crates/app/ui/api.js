// Verbindung zum Rechenkern: in der Desktop-App über Tauri, im Browser über den lokalen
// Entwicklungsserver (`vivaplan ui-dev`). Es werden keine Daten ins Internet gesendet.
"use strict";
// Fehler der Oberfläche zählen (für die automatischen Oberflächentests sichtbar als data-errors).
window.__errors = [];
const noteError = msg => { window.__errors.push(String(msg)); document.documentElement.dataset.errors = window.__errors.length; document.documentElement.dataset.lastError = String(msg).slice(0, 300); };
window.addEventListener("error", e => noteError(e.message));
window.addEventListener("unhandledrejection", e => noteError(e.reason?.message || e.reason));
const Engine = {
  inApp: !!(window.__TAURI__ && window.__TAURI__.core),
  /** #25: Grenzen aus den Einstellungen (setzt app.js); sie gehen bei jedem Aufruf mit der Planung mit. */
  limits: () => null,
  async call(cmd, args = {}) {
    // Die Grenzen gehören nicht zur Planung: nie mitspeichern.
    const limits = this.limits();
    if (limits && args.plan && typeof args.plan === "object" && cmd !== "save_plan") args = { ...args, plan: { ...args.plan, limits } };
    if (this.inApp) return window.__TAURI__.core.invoke(cmd, args);
    const r = await fetch(`/api/${cmd}`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(args)
    });
    const j = await r.json();
    if (!r.ok) throw new Error(j.error || r.statusText);
    return j;
  }
};
const errText = e => typeof e === "string" ? e : (e && e.message) || String(e);
