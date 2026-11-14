// Optimaler Parameterwert je Perzentil aus den Ergebnissen der Variationen (Szenarien).
// Es werden nur die bereits gerechneten Variationsschritte verwendet; zwischen den Schritten wird nur
// dort interpoliert, wo das eindeutig ist (höchster tragbarer Wert). Reine Funktionen ohne Zugriff auf die Seite.
"use strict";
const Optimum = (() => {
  /**
   * Kriterium je Parameter:
   *  - "sustainable": höchster Wert, bei dem das Vermögen bis zum Ende reicht (mehr Konsum = höherer Wert);
   *  - "bestEnd": Vorrang haben Werte, bei denen das Vermögen reicht; darunter gilt das höchste Endvermögen,
   *    reicht es nirgends, das späteste Alter der Erschöpfung;
   *  - kein Eintrag: Kriterium nicht sinnvoll, es wird kein Optimum angezeigt.
   */
  const MODE = {
    "flows.spendingFactor": "sustainable",
    "wealth.stockShare": "bestEnd",
    "pension.pk.lumpsum": "bestEnd"
  };
  const modeOf = param => MODE[param] ?? null;

  /** Endvermögen und Erschöpfung des Laufs am Perzentil `percent` eines Szenarios; null, wenn nicht vorhanden. */
  function cellOf(result, percent) {
    if (!result || result.error) return null;
    const p = (result.percentiles || []).find(x => x.percent === percent);
    return p ? { end: p.endTotal, ok: !p.depleted, depleted: p.depleted || null } : null;
  }
  const byValue = (a, b) => (a.v < b.v ? -1 : a.v > b.v ? 1 : 0);

  /** Höchster tragbarer Wert auf einer Linie von Punkten {v, end, ok, depleted}, aufsteigend nach v. */
  function highestSustainable(points) {
    const pts = [...points].sort(byValue);
    if (!pts.length) return null;
    let i = -1;
    pts.forEach((p, k) => { if (p.ok) i = k; });
    if (i < 0) return { v: pts[0].v, kind: "below", at: pts[0] };
    if (i === pts.length - 1) return { v: pts[i].v, kind: "atLeast", at: pts[i] };
    const a = pts[i], b = pts[i + 1];
    // Übergang zwischen zwei Schritten: Wert, bei dem das (vorzeichenbehaftete) Endvermögen genau 0 wäre.
    if (typeof a.v === "number" && typeof b.v === "number" && a.end >= 0 && b.end < 0) {
      return { v: a.v + (b.v - a.v) * a.end / (a.end - b.end), kind: "approx", at: a };
    }
    return { v: a.v, kind: "exact", at: a };
  }

  /** Bester Punkt: tragbar vor nicht tragbar, dann höchstes Endvermögen; sonst spätestes Erschöpfungsalter. */
  function bestPoint(points) {
    const ok = points.filter(p => p.ok);
    if (ok.length) return { p: ok.reduce((a, b) => (b.end > a.end ? b : a)), reaches: true };
    return { p: points.reduce((a, b) => (b.depleted > a.depleted || (b.depleted === a.depleted && b.end > a.end) ? b : a)), reaches: false };
  }

  /** Anteile haben natürliche Grenzen (0 % und 100 %); dort ist der Rand des Bereichs das echte Maximum bzw. Minimum. */
  const SHARES = new Set(["wealth.stockShare", "pension.pk.lumpsum"]);
  const naturalBound = (param, v) => SHARES.has(param) && (v === 0 || v === 1);

  /** Randkennzeichnung, wenn der beste Wert am Rand des untersuchten Bereichs liegt (ausser an einer natürlichen Grenze). */
  function edgeKind(values, v, param) {
    const vs = [...new Set(values)].sort((a, b) => (a < b ? -1 : a > b ? 1 : 0));
    if (vs.length < 2 || naturalBound(param, v)) return "exact";
    return v === vs[0] ? "edgeLow" : v === vs[vs.length - 1] ? "edgeHigh" : "exact";
  }

  /**
   * Optimum für das Perzentil `percent`. `sc` = { params, results } der Szenarien.
   * Ergebnis: { items: [{ param, v, grid, kind, reaches }] } mit kind: exact | approx | below | atLeast | edgeLow | edgeHigh | na;
   * `grid` ist der berechnete Variationsschritt, der dem Optimum entspricht (bei «approx» der letzte tragbare Schritt),
   * damit ein Lauf dafür ausgewählt werden kann;
   * null, wenn keine Szenarien oder keine Perzentil-Daten vorliegen.
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
        // Für jeden Wert des anderen Parameters den höchsten tragbaren Faktor bestimmen, dann die Kombination mit dem höchsten.
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
    return null; // mehr als zwei Variationen: kein Optimum
  }

  return { find, modeOf, MODE, highestSustainable, bestPoint };
})();
if (typeof module !== "undefined") module.exports = Optimum;
