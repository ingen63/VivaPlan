// Connection to the engine: in the desktop app via Tauri, in the browser via the local
// development server (`vivaplan ui-dev`). No data is sent to the internet.
"use strict";
// Counts errors of the user interface (visible to the automated user interface tests as data-errors).
window.__errors = [];
const noteError = msg => { window.__errors.push(String(msg)); document.documentElement.dataset.errors = window.__errors.length; document.documentElement.dataset.lastError = String(msg).slice(0, 300); };
window.addEventListener("error", e => noteError(e.message));
window.addEventListener("unhandledrejection", e => noteError(e.reason?.message || e.reason));
const Engine = {
  inApp: !!(window.__TAURI__ && window.__TAURI__.core),
  /** #25: Limits from the settings (set by app.js); they are sent with the plan on every call. */
  limits: () => null,
  async call(cmd, args = {}) {
    // The limits do not belong to the plan: never save them with it.
    const limits = this.limits();
    if (limits && args.plan && typeof args.plan === "object" && cmd !== "save_plan") args = { ...args, plan: { ...args.plan, limits } };
    if (this.inApp) return window.__TAURI__.core.invoke(cmd, args).catch(e => { throw engineError(e); });
    const r = await fetch(`/api/${cmd}`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(args)
    });
    const j = await r.json();
    if (!r.ok) throw j.code ? engineError({ code: j.code, params: j.params, text: j.error }) : new Error(j.error || r.statusText);
    return j;
  }
};
// #171: An error of the engine arrives as a message `{code, params, text}` (desktop app) or as `{error, code, params}`
// (development server); it becomes an Error with `code` and `params`, so that `errText` can translate it.
function engineError(e) {
  if (!e || typeof e !== "object" || !e.code) return e;
  return Object.assign(new Error(e.text ?? e.code), { code: e.code, params: e.params || {}, text: e.text });
}
const errText = e => typeof e === "string" ? e : e?.code ? msgText(e) : (e && e.message) || String(e);
