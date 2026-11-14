// Diagramme als SVG ohne externe Bibliotheken: Linien und Bänder, gestapelte Flächen, Balken,
// Histogramm und Heatmap. Farben kommen aus den CSS-Variablen, damit Hell- und Dunkelmodus passen.
"use strict";
const Charts = (() => {
  const cssv = n => getComputedStyle(document.documentElement).getPropertyValue(n).trim();
  const color = c => c.startsWith("--") ? cssv(c) : c;
  const escx = s => String(s ?? "").replace(/[&<>"]/g, c => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
  function step(max, n = 4) {
    const raw = max / n, p = Math.pow(10, Math.floor(Math.log10(raw || 1)));
    return [1, 2, 2.5, 5, 10].map(x => x * p).find(x => raw <= x) || p * 10;
  }
  /** Skala über [lo, hi] mit «schönen» Teilstrichen; 0 wird einbezogen, wenn sinnvoll. */
  function scale(lo, hi, withZero = true) {
    if (withZero) { lo = Math.min(0, lo); hi = Math.max(0, hi); }
    if (!(hi > lo)) { hi = lo + 1; }
    const s = step(hi - lo), a = Math.floor(lo / s) * s, b = Math.ceil(hi / s) * s, ticks = [];
    for (let v = a; v <= b + s * 1e-6; v += s) ticks.push(Math.abs(v) < s * 1e-9 ? 0 : v);
    return { lo: a, hi: b, ticks };
  }
  /** Logarithmische Skala mit Teilstrichen bei 1, 2 und 5 je Zehnerpotenz; lo/hi im Log-Raum. */
  function logScale(values) {
    const v = values.filter(x => x > 0);
    if (!v.length) return { lo: 0, hi: 1, ticks: [1, 10] };
    const mn = Math.min(...v), mx = Math.max(...v), cands = [];
    for (let e = Math.floor(Math.log10(mn)) - 1; e <= Math.ceil(Math.log10(mx)) + 1; e++) for (const k of [1, 2, 5]) cands.push(k * Math.pow(10, e));
    const lo = Math.max(...cands.filter(c => c <= mn * (1 + 1e-9))), hi = Math.min(...cands.filter(c => c >= mx * (1 - 1e-9)));
    const ticks = cands.filter(c => c >= lo * (1 - 1e-9) && c <= hi * (1 + 1e-9));
    return { lo: Math.log10(lo), hi: Math.log10(hi > lo ? hi : lo * 10), ticks };
  }
  function frame(svg, m = {}) {
    const W = svg.clientWidth || 800, H = svg.clientHeight || 300;
    svg.setAttribute("viewBox", `0 0 ${W} ${H}`);
    return { W, H, L: m.L ?? 60, R: m.R ?? 14, T: m.T ?? 12, B: m.B ?? 28 };
  }
  function grid(f, ys, yFmt) {
    return ys.ticks.map(v => {
      const y = f.y(v);
      return `<line x1="${f.L}" x2="${f.W - f.R}" y1="${y}" y2="${y}" stroke="${cssv(v === 0 ? "--border" : "--grid")}"/>` +
        `<text x="${f.L - 6}" y="${y + 4}" text-anchor="end" font-size="10.5" fill="${cssv("--muted")}">${yFmt(v)}</text>`;
    }).join("");
  }
  function tipAt(svg, tip, f, px, html) {
    if (!tip) return;
    const r = svg.getBoundingClientRect();
    tip.style.display = "block"; tip.innerHTML = html;
    const left = px * r.width / f.W;
    tip.style.left = (left > r.width / 2 ? left - tip.offsetWidth - 10 : left + 10) + "px"; tip.style.top = "8px";
  }
  function hideTip(svg, tip) {
    if (tip) tip.style.display = "none";
    svg.querySelector(".hov")?.setAttribute("visibility", "hidden");
  }
  /** Pfad durch die Punkte; Lücken (NaN) unterbrechen die Linie. */
  const pathOf = pts => { let s = "", pen = false; for (const [x, y] of pts) { if (!Number.isFinite(y)) { pen = false; continue; } s += `${pen ? "L" : "M"}${x.toFixed(1)},${y.toFixed(1)}`; pen = true; } return s; };

  /**
   * Zeitachse mit Linien, Bändern und optional gestapelten Flächen.
   * o = { x: [Monatsindex], xFmt, yFmt, lines: [{label, color, values, dash, width}],
   *       bands: [{lo, hi, color, opacity, label}], stack: [{label, color, values}], percent,
   *       marks: [{x, color, label}], tip: (i) => html, yMin }
   */
  /**
   * Bereichsauswahl per Maus: Mit gedrückter Taste einen waagrechten Bereich aufziehen (hervorgehoben),
   * beim Loslassen ruft `done(von, bis)` mit den Indizes auf. `at(px)` liefert den Index zu einer Pixelposition,
   * `span(von, bis)` die Pixelgrenzen des Rechtecks.
   */
  function selectable(svg, tip, f, at, span, done) {
    svg.style.cursor = "crosshair";
    svg.onmousedown = ev => {
      if (ev.button !== 0) return;
      const r = svg.getBoundingClientRect(), px = e => (e.clientX - r.left) * f.W / r.width;
      const i0 = at(px(ev));
      let i1 = i0;
      const rect = document.createElementNS("http://www.w3.org/2000/svg", "rect");
      rect.setAttribute("class", "sel");
      rect.setAttribute("y", f.T);
      rect.setAttribute("height", f.H - f.T - f.B);
      rect.setAttribute("fill", cssv("--accent"));
      rect.setAttribute("fill-opacity", ".2");
      rect.setAttribute("stroke", cssv("--accent"));
      rect.setAttribute("stroke-dasharray", "3 2");
      const draw = () => {
        const [a, b] = span(Math.min(i0, i1), Math.max(i0, i1));
        rect.setAttribute("x", a);
        rect.setAttribute("width", Math.max(1, b - a));
      };
      draw();
      svg.appendChild(rect);
      svg.classList.add("selecting");
      hideTip(svg, tip);
      const move = e => { i1 = at(px(e)); draw(); hideTip(svg, tip); };
      const up = () => {
        window.removeEventListener("mousemove", move);
        window.removeEventListener("mouseup", up);
        document.body.style.userSelect = "";
        svg.classList.remove("selecting");
        rect.remove();
        done(Math.min(i0, i1), Math.max(i0, i1));
      };
      document.body.style.userSelect = "none";
      window.addEventListener("mousemove", move);
      window.addEventListener("mouseup", up);
    };
  }
  /** Ausschnitt `view` = [von, bis] (Monatsindizes): alle Reihen werden auf diesen Bereich gekürzt. */
  function windowed(o) {
    const lo = o.x.findIndex(v => v >= o.view[0]);
    let hi = -1;
    o.x.forEach((v, i) => { if (v <= o.view[1]) hi = i; });
    if (lo < 0 || hi - lo < 1) return o;
    const cut = a => a.slice(lo, hi + 1);
    return { ...o, view: null, x: cut(o.x),
      lines: o.lines && o.lines.map(l => ({ ...l, values: cut(l.values) })),
      bands: o.bands && o.bands.map(b => ({ ...b, lo: cut(b.lo), hi: cut(b.hi) })),
      stack: o.stack && o.stack.map(s => ({ ...s, values: cut(s.values) })),
      tip: i => o.tip(i + lo) };
  }
  function time(svg, tip, o) {
    if (!svg) return;
    if (o.view) o = windowed(o);
    const f = frame(svg, o.margin), n = o.x.length;
    if (n < 2) { svg.innerHTML = ""; return; }
    let stackTop = null;
    if (o.stack) {
      stackTop = o.stack.map(() => new Array(n).fill(0));
      for (let i = 0; i < n; i++) {
        let acc = 0;
        const sum = o.percent ? o.stack.reduce((a, s) => a + Math.max(0, s.values[i]), 0) || 1 : 1;
        o.stack.forEach((s, k) => { acc += Math.max(0, s.values[i]) / sum * (o.percent ? 100 : 1); stackTop[k][i] = acc; });
      }
    }
    const all = [...(o.lines || []).flatMap(l => l.values), ...(o.bands || []).flatMap(b => [...b.lo, ...b.hi]), ...(stackTop ? stackTop[stackTop.length - 1] : [])].filter(Number.isFinite);
    const ys = o.log ? logScale(all) : o.percent ? scale(0, 100) : o.range ? scale(o.range[0], o.range[1], false) : scale(o.yMin ?? Math.min(...all), Math.max(...all));
    f.x = i => f.L + i / (n - 1) * (f.W - f.L - f.R);
    // Logarithmisch: ys.lo/ys.hi liegen im Log-Raum, nicht positive Werte bleiben Lücken.
    const pos = o.log ? v => (v > 0 ? Math.log10(v) : NaN) : v => v;
    f.y = v => f.T + (1 - (pos(v) - ys.lo) / (ys.hi - ys.lo)) * (f.H - f.T - f.B);
    let h = grid(f, ys, o.percent ? v => `${v} %` : o.yFmt);
    let lastLabel = -1e9;
    for (let i = 0; i < n; i++) {
      if (o.x[i] % 12 === 0 && (o.x[i] / 12) % (n > 360 ? 10 : 5) === 0 && f.x(i) - lastLabel > 50) {
        h += `<text x="${f.x(i)}" y="${f.H - 8}" text-anchor="middle" font-size="10.5" fill="${cssv("--muted")}">${o.xFmt(o.x[i])}</text>`;
        lastLabel = f.x(i);
      }
    }
    for (const b of o.bands || []) {
      const up = b.hi.map((v, i) => [f.x(i), f.y(v)]), down = b.lo.map((v, i) => [f.x(i), f.y(v)]).reverse();
      h += `<path d="${pathOf(up)}L${pathOf(down).slice(1)}Z" fill="${color(b.color)}" fill-opacity="${b.opacity ?? 0.2}"/>`;
    }
    if (stackTop) {
      stackTop.forEach((top, k) => {
        const lower = k ? stackTop[k - 1] : new Array(n).fill(0);
        const up = top.map((v, i) => [f.x(i), f.y(v)]), down = lower.map((v, i) => [f.x(i), f.y(v)]).reverse();
        h += `<path d="${pathOf(up)}L${pathOf(down).slice(1)}Z" fill="${color(o.stack[k].color)}" fill-opacity=".85"/>`;
      });
    }
    for (const l of o.lines || []) {
      h += `<path d="${pathOf(l.values.map((v, i) => [f.x(i), f.y(v)]))}" fill="none" stroke="${color(l.color)}" stroke-width="${l.width ?? 2}" ${l.dash ? `stroke-dasharray="${l.dash}"` : ""}/>`;
    }
    for (const mk of o.marks || []) {
      const i = mk.x - o.x[0]; if (i < 0 || i >= n) continue;
      h += `<line x1="${f.x(i)}" x2="${f.x(i)}" y1="${f.T}" y2="${f.H - f.B}" stroke="${color(mk.color || "--muted")}" stroke-dasharray="3 3" stroke-opacity=".7"><title>${escx(mk.label)}</title></line>`;
    }
    h += `<line class="hov" y1="${f.T}" y2="${f.H - f.B}" stroke="${cssv("--text")}" stroke-opacity=".5" visibility="hidden"/>`;
    svg.innerHTML = h;
    const idx = px => Math.max(0, Math.min(n - 1, Math.round((px - f.L) / (f.W - f.L - f.R) * (n - 1))));
    svg.onmousemove = ev => {
      if (svg.classList.contains("selecting")) return;
      const r = svg.getBoundingClientRect(), px = (ev.clientX - r.left) * f.W / r.width;
      const i = idx(px);
      const hov = svg.querySelector(".hov"); hov.setAttribute("x1", f.x(i)); hov.setAttribute("x2", f.x(i)); hov.setAttribute("visibility", "visible");
      tipAt(svg, tip, f, f.x(i), o.tip(i));
    };
    svg.onmouseleave = () => { if (!svg.classList.contains("selecting")) hideTip(svg, tip); };
    // Bereich aufziehen = zoomen; Auswahlen unter einem Jahr (12 Monatsindizes) zählen als Klick und werden ignoriert
    if (o.onZoom) selectable(svg, tip, f, idx, (a, b) => [f.x(a), f.x(b)], (a, b) => { if (o.x[b] - o.x[a] >= 12) o.onZoom(o.x[a], o.x[b]); });
  }

  /**
   * Balken je Kategorie; `series` mit `group` werden innerhalb der Gruppe gestapelt, Gruppen stehen nebeneinander.
   * o = { cats: [Text], series: [{label, color, values, group}], yFmt, tip: (i) => html, line: {label, color, values} }
   */
  function bars(svg, tip, o) {
    if (!svg) return;
    const f = frame(svg, o.margin), n = o.cats.length;
    if (!n) { svg.innerHTML = ""; return; }
    const groups = [...new Set(o.series.map(s => s.group ?? s.label))];
    const tops = [], bottoms = [];
    for (let i = 0; i < n; i++) for (const g of groups) {
      let pos = 0, neg = 0;
      for (const s of o.series.filter(s => (s.group ?? s.label) === g)) { const v = s.values[i] || 0; if (v >= 0) pos += v; else neg += v; }
      tops.push(pos); bottoms.push(neg);
    }
    const extra = o.line ? o.line.values : [];
    const ys = scale(Math.min(...bottoms, ...extra), Math.max(...tops, ...extra));
    const slot = (f.W - f.L - f.R) / n, bw = Math.max(1, slot * 0.8 / groups.length);
    f.y = v => f.T + (1 - (v - ys.lo) / (ys.hi - ys.lo)) * (f.H - f.T - f.B);
    let h = grid(f, ys, o.yFmt);
    const every = Math.ceil(n / Math.max(1, Math.floor((f.W - f.L) / 46)));
    for (let i = 0; i < n; i++) {
      const x0 = f.L + i * slot + slot * 0.1;
      groups.forEach((g, gi) => {
        let pos = 0, neg = 0;
        for (const s of o.series.filter(s => (s.group ?? s.label) === g)) {
          const v = s.values[i] || 0; if (!v) continue;
          const a = v >= 0 ? pos : neg, b = a + v;
          if (v >= 0) pos = b; else neg = b;
          h += `<rect x="${(x0 + gi * bw).toFixed(1)}" y="${f.y(Math.max(a, b)).toFixed(1)}" width="${(bw - 1).toFixed(1)}" height="${Math.max(0.5, Math.abs(f.y(a) - f.y(b))).toFixed(1)}" fill="${color(s.color)}"/>`;
        }
      });
      if (i % every === 0) h += `<text x="${(f.L + (i + 0.5) * slot).toFixed(1)}" y="${f.H - 8}" text-anchor="middle" font-size="10.5" fill="${cssv("--muted")}">${escx(o.cats[i])}</text>`;
    }
    if (o.line) h += `<path d="${pathOf(o.line.values.map((v, i) => [f.L + (i + 0.5) * slot, f.y(v)]))}" fill="none" stroke="${color(o.line.color)}" stroke-width="2"/>`;
    h += `<rect class="hov" y="${f.T}" height="${f.H - f.T - f.B}" width="${slot}" fill="${cssv("--text")}" fill-opacity=".06" visibility="hidden"/>`;
    svg.innerHTML = h;
    const idx = px => Math.max(0, Math.min(n - 1, Math.floor((px - f.L) / slot)));
    svg.onmousemove = ev => {
      if (svg.classList.contains("selecting")) return;
      const r = svg.getBoundingClientRect(), px = (ev.clientX - r.left) * f.W / r.width;
      const i = idx(px);
      const hov = svg.querySelector(".hov"); hov.setAttribute("x", f.L + i * slot); hov.setAttribute("visibility", "visible");
      tipAt(svg, tip, f, f.L + (i + 0.5) * slot, o.tip(i));
    };
    svg.onmouseleave = () => { if (!svg.classList.contains("selecting")) hideTip(svg, tip); };
    // Mehrere Jahre aufziehen = zoomen (ein einzelner Balken zählt als Klick)
    if (o.onSelect) selectable(svg, tip, f, idx, (a, b) => [f.L + a * slot, f.L + (b + 1) * slot], (a, b) => { if (b > a) o.onSelect(a, b); });
  }

  /** Histogramm aus {edges, counts}; `marks` = [{value, label, color}] als senkrechte Linien. */
  function histogram(svg, tip, hist, o) {
    if (!svg) return;
    if (!hist) { svg.innerHTML = ""; return; }
    const f = frame(svg, o.margin), n = hist.counts.length, total = hist.counts.reduce((a, b) => a + b, 0) || 1;
    const lo = hist.edges[0], hi = hist.edges[n], ys = scale(0, Math.max(...hist.counts) / total * 100);
    f.x = v => f.L + (v - lo) / ((hi - lo) || 1) * (f.W - f.L - f.R);
    f.y = v => f.T + (1 - v / ys.hi) * (f.H - f.T - f.B);
    let h = grid(f, ys, v => `${v.toLocaleString("de-CH", { maximumFractionDigits: 1 })} %`);
    for (let i = 0; i < n; i++) {
      const p = hist.counts[i] / total * 100, x0 = f.x(hist.edges[i]), x1 = f.x(hist.edges[i + 1]);
      h += `<rect x="${(x0 + 0.5).toFixed(1)}" y="${f.y(p).toFixed(1)}" width="${Math.max(0.5, x1 - x0 - 1).toFixed(1)}" height="${(f.y(0) - f.y(p)).toFixed(1)}" fill="${color(o.color || "--c1")}" fill-opacity=".75"/>`;
    }
    const sx = scale(lo, hi, false);
    for (const v of sx.ticks) if (v >= lo && v <= hi) h += `<text x="${f.x(v)}" y="${f.H - 8}" text-anchor="middle" font-size="10.5" fill="${cssv("--muted")}">${o.xFmt(v)}</text>`;
    for (const mk of o.marks || []) {
      if (!(mk.value >= lo && mk.value <= hi)) continue;
      h += `<line x1="${f.x(mk.value)}" x2="${f.x(mk.value)}" y1="${f.T}" y2="${f.H - f.B}" stroke="${color(mk.color || "--text")}" stroke-width="1.5" stroke-dasharray="4 3"/>` +
        `<text x="${f.x(mk.value) + 4}" y="${f.T + 10}" font-size="10.5" fill="${color(mk.color || "--text")}">${escx(mk.label)}</text>`;
    }
    h += `<rect class="hov" y="${f.T}" height="${f.H - f.T - f.B}" fill="${cssv("--text")}" fill-opacity=".06" visibility="hidden"/>`;
    svg.innerHTML = h;
    svg.onmousemove = ev => {
      const r = svg.getBoundingClientRect(), px = (ev.clientX - r.left) * f.W / r.width;
      const v = lo + (px - f.L) / (f.W - f.L - f.R) * (hi - lo), i = Math.max(0, Math.min(n - 1, Math.floor((v - lo) / ((hi - lo) / n))));
      const hov = svg.querySelector(".hov"); hov.setAttribute("x", f.x(hist.edges[i])); hov.setAttribute("width", f.x(hist.edges[i + 1]) - f.x(hist.edges[i])); hov.setAttribute("visibility", "visible");
      tipAt(svg, tip, f, f.x(hist.edges[i + 1]), `${o.xFmt(hist.edges[i])} – ${o.xFmt(hist.edges[i + 1])}<br><b>${(hist.counts[i] / total * 100).toLocaleString("de-CH", { maximumFractionDigits: 1 })} %</b> der Läufe (${hist.counts[i].toLocaleString("de-CH")})`);
    };
    svg.onmouseleave = () => hideTip(svg, tip);
  }

  /** Linien über einer Zahlen- bzw. Kategorienachse (Szenarien); o = { xs: [Text], lines, bands, yFmt, tip } */
  function category(svg, tip, o) {
    if (!svg) return;
    const f = frame(svg, o.margin), n = o.xs.length;
    if (!n) { svg.innerHTML = ""; return; }
    const all = [...o.lines.flatMap(l => l.values), ...(o.bands || []).flatMap(b => [...b.lo, ...b.hi])].filter(Number.isFinite);
    const ys = o.range ? scale(o.range[0], o.range[1], false) : scale(Math.min(...all), Math.max(...all));
    f.x = i => n === 1 ? (f.L + f.W - f.R) / 2 : f.L + 20 + i / (n - 1) * (f.W - f.L - f.R - 40);
    f.y = v => f.T + (1 - (v - ys.lo) / (ys.hi - ys.lo)) * (f.H - f.T - f.B);
    let h = grid(f, ys, o.yFmt);
    const every = Math.ceil(n / Math.max(1, Math.floor((f.W - f.L) / 60)));
    o.xs.forEach((t, i) => { if (i % every === 0) h += `<text x="${f.x(i)}" y="${f.H - 8}" text-anchor="middle" font-size="10.5" fill="${cssv("--muted")}">${escx(t)}</text>`; });
    for (const b of o.bands || []) {
      const up = b.hi.map((v, i) => [f.x(i), f.y(v)]), down = b.lo.map((v, i) => [f.x(i), f.y(v)]).reverse();
      h += `<path d="${pathOf(up)}L${pathOf(down).slice(1)}Z" fill="${color(b.color)}" fill-opacity="${b.opacity ?? 0.2}"/>`;
    }
    for (const l of o.lines) {
      const pts = l.values.map((v, i) => [f.x(i), f.y(v)]);
      h += `<path d="${pathOf(pts)}" fill="none" stroke="${color(l.color)}" stroke-width="2" ${l.dash ? `stroke-dasharray="${l.dash}"` : ""}/>` +
        pts.map(([x, y]) => `<circle cx="${x}" cy="${y}" r="3" fill="${color(l.color)}"/>`).join("");
    }
    h += `<line class="hov" y1="${f.T}" y2="${f.H - f.B}" stroke="${cssv("--text")}" stroke-opacity=".4" visibility="hidden"/>`;
    svg.innerHTML = h;
    svg.onmousemove = ev => {
      const r = svg.getBoundingClientRect(), px = (ev.clientX - r.left) * f.W / r.width;
      let i = 0; for (let k = 1; k < n; k++) if (Math.abs(f.x(k) - px) < Math.abs(f.x(i) - px)) i = k;
      const hov = svg.querySelector(".hov"); hov.setAttribute("x1", f.x(i)); hov.setAttribute("x2", f.x(i)); hov.setAttribute("visibility", "visible");
      tipAt(svg, tip, f, f.x(i), o.tip(i));
    };
    svg.onmouseleave = () => hideTip(svg, tip);
  }

  /** Heatmap als Tabelle: o = { xs, ys, value: (xi, yi) => Zahl, fmt, lo, hi, good: "high" | "low" } */
  function heatmap(o) {
    const vals = []; o.ys.forEach((_, yi) => o.xs.forEach((_, xi) => vals.push(o.value(xi, yi))));
    const fin = vals.filter(Number.isFinite), lo = o.lo ?? Math.min(...fin), hi = o.hi ?? Math.max(...fin);
    const cell = v => {
      if (!Number.isFinite(v)) return `<td class="hm na">–</td>`;
      let t = hi > lo ? (v - lo) / (hi - lo) : 1; if (o.good === "low") t = 1 - t;
      // rot (schlecht) → gelb → grün (gut), mit genügend Kontrast für dunkle Schrift
      const hue = Math.round(t * 120);
      return `<td class="hm" style="background:hsl(${hue} 65% 72%)">${o.fmt(v)}</td>`;
    };
    return `<div style="overflow-x:auto"><table class="tbl heat"><tr><th class="l">${escx(o.yLabel)} ↓ · ${escx(o.xLabel)} →</th>${o.xs.map(x => `<th>${escx(x)}</th>`).join("")}</tr>
      ${o.ys.map((y, yi) => `<tr><th class="l">${escx(y)}</th>${o.xs.map((_, xi) => cell(o.value(xi, yi))).join("")}</tr>`).join("")}</table></div>`;
  }

  return { time, bars, histogram, category, heatmap };
})();
