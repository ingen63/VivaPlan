// Optimal parameter value per percentile from the results of the variations (scenarios).
// Only the variation steps already calculated are used; between the steps it only
// interpolates where this is unambiguous (highest sustainable value). Pure functions without access to the page.
"use strict";
const Optimum = (() => {
  /**
   * Criterion per parameter:
   *  - "sustainable": highest value at which the wealth lasts until the end (more consumption = higher value);
   *  - "bestEnd": values at which the wealth lasts take precedence; among them the highest final wealth applies,
   *    if it lasts nowhere, the latest age of depletion;
   *  - no entry: criterion not meaningful, no optimum is shown.
   */
  const MODE = {
    "flows.spendingFactor": "sustainable",
    "wealth.stockShare": "bestEnd",
    "pension.pk.lumpsum": "bestEnd"
  };
  const modeOf = param => MODE[param] ?? null;

  /** Final wealth and depletion of the run at percentile `percent` of a scenario; null if not present. */
  function cellOf(result, percent) {
    if (!result || result.error) return null;
    const p = (result.percentiles || []).find(x => x.percent === percent);
    return p ? { end: p.endTotal, ok: !p.depleted, depleted: p.depleted || null } : null;
  }
  const byValue = (a, b) => (a.v < b.v ? -1 : a.v > b.v ? 1 : 0);

  /** Highest sustainable value on a line of points {v, end, ok, depleted}, ascending by v. */
  function highestSustainable(points) {
    const pts = [...points].sort(byValue);
    if (!pts.length) return null;
    let i = -1;
    pts.forEach((p, k) => { if (p.ok) i = k; });
    if (i < 0) return { v: pts[0].v, kind: "below", at: pts[0] };
    if (i === pts.length - 1) return { v: pts[i].v, kind: "atLeast", at: pts[i] };
    const a = pts[i], b = pts[i + 1];
    // Transition between two steps: value at which the (signed) final wealth would be exactly 0.
    if (typeof a.v === "number" && typeof b.v === "number" && a.end >= 0 && b.end < 0) {
      return { v: a.v + (b.v - a.v) * a.end / (a.end - b.end), kind: "approx", at: a };
    }
    return { v: a.v, kind: "exact", at: a };
  }

  /** Best point: sustainable before not sustainable, then highest final wealth; otherwise latest depletion age. */
  function bestPoint(points) {
    const ok = points.filter(p => p.ok);
    if (ok.length) return { p: ok.reduce((a, b) => (b.end > a.end ? b : a)), reaches: true };
    return { p: points.reduce((a, b) => (b.depleted > a.depleted || (b.depleted === a.depleted && b.end > a.end) ? b : a)), reaches: false };
  }

  /** Shares have natural limits (0 % and 100 %); there the edge of the range is the true maximum or minimum. */
  const SHARES = new Set(["wealth.stockShare", "pension.pk.lumpsum"]);
  const naturalBound = (param, v) => SHARES.has(param) && (v === 0 || v === 1);

  /** Edge mark if the best value lies at the edge of the examined range (except at a natural limit). */
  function edgeKind(values, v, param) {
    const vs = [...new Set(values)].sort((a, b) => (a < b ? -1 : a > b ? 1 : 0));
    if (vs.length < 2 || naturalBound(param, v)) return "exact";
    return v === vs[0] ? "edgeLow" : v === vs[vs.length - 1] ? "edgeHigh" : "exact";
  }

  /**
   * Optimum for percentile `percent`. `sc` = { params, results } of the scenarios.
   * Result: { items: [{ param, v, grid, kind, reaches }] } with kind: exact | approx | below | atLeast | edgeLow | edgeHigh | na;
   * `grid` is the calculated variation step that corresponds to the optimum (for «approx» the last sustainable step),
   * so that a run can be selected for it;
   * null if there are no scenarios or no percentile data.
   */
  function find(sc, percent) {
    if (!sc?.params?.length || !sc.results?.length) return null;
    const rows = sc.results.map(r => ({ r, c: cellOf(r, percent) })).filter(x => x.c);
    if (!rows.length) return null;
    const point = x => ({ ...x.c, vals: x.r.values });
    const na = param => ({ param, v: null, grid: null, kind: "na", reaches: true });
    const modes = sc.params.map(modeOf);
    if (modes.some(m => !m)) return { items: sc.params.map(na) };

    if (sc.params.length === 1) {
      const pts = rows.map(x => ({ ...point(x), v: x.r.values[0] }));
      if (modes[0] === "sustainable") {
        const h = highestSustainable(pts);
        return { items: [{ param: sc.params[0], v: h.v, grid: h.at.v, kind: h.kind, reaches: h.kind !== "below" }] };
      }
      const { p, reaches } = bestPoint(pts);
      return { items: [{ param: sc.params[0], v: p.v, grid: p.v, kind: edgeKind(pts.map(x => x.v), p.v, sc.params[0]), reaches }] };
    }

    if (sc.params.length === 2) {
      const fi = modes.indexOf("sustainable"), oi = fi < 0 ? -1 : 1 - fi;
      if (fi >= 0 && modes[oi] !== "sustainable") {
        // For each value of the other parameter determine the highest sustainable factor, then the combination with the highest.
        const others = [...new Set(rows.map(x => JSON.stringify(x.r.values[oi])))].map(JSON.parse);
        const lines = others.map(o => {
          const pts = rows.filter(x => JSON.stringify(x.r.values[oi]) === JSON.stringify(o)).map(x => ({ ...point(x), v: x.r.values[fi] }));
          return { o, h: highestSustainable(pts), pts };
        }).filter(l => l.h);
        const rank = l => (l.h.kind === "below" ? [-Infinity, l.h.at.depleted || "", l.h.at.end] : [l.h.v, "", l.h.at.end]);
        const better = (a, b) => {
          const [ra, rb] = [rank(a), rank(b)];
          for (let k = 0; k < 3; k++) if (ra[k] !== rb[k]) return ra[k] > rb[k];
          return false;
        };
        const best = lines.reduce((a, b) => (better(b, a) ? b : a));
        const items = [];
        items[fi] = { param: sc.params[fi], v: best.h.v, grid: best.h.at.v, kind: best.h.kind, reaches: best.h.kind !== "below" };
        items[oi] = { param: sc.params[oi], v: best.o, grid: best.o, kind: edgeKind(others, best.o, sc.params[oi]), reaches: best.h.kind !== "below" };
        return { items };
      }
      const pts = rows.map(x => ({ ...point(x), v: x.r.values.join("|") }));
      const { p, reaches } = bestPoint(pts);
      const values = [0, 1].map(k => rows.map(x => x.r.values[k]));
      return { items: [0, 1].map(k => ({ param: sc.params[k], v: p.vals[k], grid: p.vals[k], kind: edgeKind(values[k], p.vals[k], sc.params[k]), reaches })) };
    }
    return null; // more than two variations: no optimum
  }

  return { find, modeOf, MODE, highestSustainable, bestPoint };
})();
if (typeof module !== "undefined") module.exports = Optimum;
