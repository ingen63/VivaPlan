// Help: panel on the right, context-sensitive via F1, «?» buttons at fields and headings, search.
// The texts are in `hilfe/*.md` (offline, shipped with the app). A topic starts with «## @id Title».
"use strict";
const Help = (() => {
  const CHAPTERS = ["00-einstieg", "10-editor", "20-renditen", "30-entnahme", "40-optimierung", "50-ergebnisse", "60-referenz"];
  /** Input field (data-path) → topic; the first matching row applies. */
  const FIELD_TOPICS = [
    [/^returns\.source$/, "f-ret-source"], [/^returns\.fixed\./, "f-ret-fixed"], [/^returns\.startDate$/, "f-ret-historical"],
    [/^returns\.bootstrap\./, "f-ret-bootstrap"], [/^returns\.parametric\./, "f-ret-parametric"], [/^returns\.extrapolation\./, "f-ret-extrapolation"],
    [/^returns\.(series|strategy)\b/, "f-ret-series"], [/^taxes\.propertyGain\./, "f-tax-gain"], [/^realestate\.properties\.\d+\.bought$/, "f-re-bought"],
    [/^(pension|partner)\.pk\.conversion$/, "f-pk-conversion"], [/^(pension|partner)\.pk\.lumpsum$/, "f-pk-lumpsum"], [/^(pension|partner)\.pk\./, "f-pk"],
    [/^(pension|partner)\.ahv\./, "f-ahv"], [/^household\.(married|ahvCoupleCap)$/, "f-couple"],
    [/^wealth\.stockShare/, "f-stockshare"], [/^wealth\.reserveMonths/, "f-reserve"],
    [/^withdrawal\.strategy$/, "f-wd-choice"], [/^withdrawal\.buckets\.spending$/, "f-wd-three_buckets_guardrails"], [/^withdrawal\.buckets\./, "f-wd-three_buckets"], [/^withdrawal\.fixedRate\./, "f-wd-fixed_rate"],
    [/^withdrawal\.guardrails\./, "f-wd-guardrails"], [/^withdrawal\.vpw\./, "f-wd-vpw"], [/^withdrawal\.endowment\./, "f-wd-endowment"],
    [/^withdrawal\.(floor|ceiling)$/, "f-wd-limits"],
    [/^taxes\.multiplier$/, "f-tax-multiplier"], [/^taxes\./, "f-tax-tables"],
    [/^realestate\.emergencyMonths/, "f-re-emergency"], [/^realestate\.affordability\./, "f-re-afford"], [/^realestate\.properties\..*(mortgage|tranche)/, "f-re-mortgage"],
    [/^scenarios\.sweeps/, "f-sweep"], [/^milestones\./, "s-milestones"], [/^pillar3a\./, "s-pillar3a"],
    [/^flows\.(income|spending)\b/, "f-timed"], [/^partner\.income\b/, "f-timed"]
  ];
  /** Input fields of the optimization (without data-path) → topic. */
  const OPT_TOPICS = [[/^opt-(maxRuin|ruinAge)$/, "o-goal"], [/^opt-weightMedian$/, "o-consumption"], [/^opt-(volWeight|maxVol)$/, "o-vol"],
    [/^opt-(paths|checkPaths|samples)$/, "o-advanced"], [/^optS-/, "o-strategies"], [/^optR-/, "o-ranges"]];
  /** Card headings (start of the text) → topic. */
  const HEADING_TOPICS = [
    ["Pensionskasse", "f-pk"], ["AHV", "f-ahv"], ["Säule 3a", "s-pillar3a"], ["Töpfe", "f-wd-three_buckets"], ["Notverkauf", "f-re-emergency"], ["Tragbarkeit", "f-re-afford"],
    ["Variationen", "f-sweep"], ["Zeitreihe", "f-ret-series"], ["Anlagestrategie", "f-ret-series"], ["Renditen aus der Anlagestrategie", "f-ret-series"], ["Fortschreibung", "f-ret-extrapolation"], ["Bootstrap", "f-ret-bootstrap"],
    ["Aktienmodell", "f-ret-parametric"], ["Fixe Renditen", "f-ret-fixed"], ["Durchlauf", "f-ret-rolling"], ["Ziel", "o-goal"], ["Strategien", "o-strategies"], ["Parameter der Strategien", "o-ranges"],
    ["Standard", "f-wd-standard"], ["Grenzen der Lebenshaltung", "f-wd-limits"], ["Fixe Entnahmerate", "f-wd-fixed_rate"], ["Guardrails", "f-wd-guardrails"], ["Variable Prozententnahme", "f-wd-vpw"], ["Endowment", "f-wd-endowment"],
    ["Grundstückgewinnsteuer", "f-tax-gain"], ["Verheiratete", "f-tax-tables"], ["Einkommenssteuer", "f-tax-tables"], ["Steuerfuss", "f-tax-multiplier"],
    ["Bandbreite des Gesamtvermögens", "r-fan"], ["Wahrscheinlichkeit, dass das Geld reicht", "r-survival"], ["Verteilung des Endvermögens", "r-hist"],
    ["Verteilung des Vermögens bei Pensionierung", "r-hist"], ["Alter bei Erschöpfung", "r-depletion"], ["Endvermögen nach Perzentil", "r-percentiles"],
    ["Aktien und Obligationen", "r-markets"], ["Vermögensverlauf", "r-wealth"], ["Entnahme aus dem Vermögen", "r-draw"], ["Ereignisse", "r-events"],
    ["Lauf wählen", "r-runs"], ["Erfolgsquote je Variante", "r-scenarios"], ["Endvermögen je Variante", "r-scenarios"], ["Jahreswerte", "r-table"],
    ["Kennzahlen", "r-compare"], ["Gesamtvermögen im Vergleich", "r-compare"], ["Unterschiede in der Planung", "r-compare"], ["Ergebnis", "o-read"], ["Liquides Vermögen nach Anlage", "r-comp"], ["Einnahmen und Ausgaben pro Jahr", "r-flows"], ["Immobilien", "r-prop"], ["Vorsorge", "r-pk"]
  ];
  const topics = new Map(), chapters = [];
  let loading = null, hist = [], pos = -1, panel = null;
  const $h = s => panel.querySelector(s);
  const esc = s => String(s ?? "").replace(/[&<>"]/g, c => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));

  // ----- Markdown (small subset) -----
  const inline = s => esc(s)
    .replace(/`([^`]+)`/g, "<code>$1</code>").replace(/\*\*([^*]+)\*\*/g, "<b>$1</b>").replace(/(^|[\s(])\*([^*\s][^*]*)\*/g, "$1<i>$2</i>")
    .replace(/\[([^\]]+)\]\(#([\w-]+)\)/g, '<a data-topic="$2">$1</a>');
  function render(md) {
    const out = [], lines = md.split("\n");
    for (let i = 0; i < lines.length; i++) {
      const l = lines[i];
      if (!l.trim()) continue;
      let m;
      if ((m = /^(#{3,4}) (.*)/.exec(l))) { out.push(`<h${m[1].length}>${inline(m[2])}</h${m[1].length}>`); continue; }
      if (l.startsWith("```")) {
        const code = [];
        for (i++; i < lines.length && !lines[i].startsWith("```"); i++) code.push(lines[i]);
        out.push(`<pre>${esc(code.join("\n"))}</pre>`); continue;
      }
      if (/^\s*[-*] /.test(l) || /^\s*\d+\. /.test(l)) {
        const ordered = /^\s*\d+\. /.test(l), items = [];
        while (i < lines.length && (/^\s*[-*] /.test(lines[i]) || /^\s*\d+\. /.test(lines[i]) || /^\s{2,}\S/.test(lines[i]))) {
          if (/^\s{2,}\S/.test(lines[i]) && !/^\s*([-*]|\d+\.) /.test(lines[i])) items[items.length - 1] += " " + lines[i].trim();
          else items.push(lines[i].replace(/^\s*([-*]|\d+\.) /, ""));
          i++;
        }
        i--; out.push(`<${ordered ? "ol" : "ul"}>${items.map(x => `<li>${inline(x)}</li>`).join("")}</${ordered ? "ol" : "ul"}>`); continue;
      }
      if (l.startsWith("|")) {
        const rows = [];
        while (i < lines.length && lines[i].startsWith("|")) { rows.push(lines[i].split("|").slice(1, -1).map(c => c.trim())); i++; }
        i--;
        const head = rows[0], body = rows.slice(/^[-:\s|]+$/.test(lines[i - rows.length + 2] || "") ? 2 : 1);
        out.push(`<table><tr>${head.map(c => `<th>${inline(c)}</th>`).join("")}</tr>${body.map(r => `<tr>${r.map(c => `<td>${inline(c)}</td>`).join("")}</tr>`).join("")}</table>`); continue;
      }
      if (l.startsWith(">")) {
        const q = [];
        while (i < lines.length && lines[i].startsWith(">")) { q.push(lines[i].replace(/^>\s?/, "")); i++; }
        i--; out.push(`<blockquote><p>${inline(q.join(" "))}</p></blockquote>`); continue;
      }
      const p = [];
      while (i < lines.length && lines[i].trim() && !/^(#{3,4} |\s*[-*] |\s*\d+\. |\||>|```)/.test(lines[i])) { p.push(lines[i]); i++; }
      i--; out.push(`<p>${inline(p.join(" "))}</p>`);
    }
    return out.join("");
  }

  // ----- Loading -----
  function parse(chapterId, text) {
    const lines = text.split("\n"), ch = { id: chapterId, title: chapterId, topics: [] };
    let cur = null;
    for (const l of lines) {
      let m;
      if ((m = /^# (.*)/.exec(l))) { ch.title = m[1].trim(); continue; }
      if ((m = /^## @([\w-]+) (.*)/.exec(l))) { cur = { id: m[1], title: m[2].trim(), chapter: ch, keywords: "", body: [] }; ch.topics.push(cur); topics.set(cur.id, cur); continue; }
      if (!cur) continue;
      if (/^Stichworte:/.test(l) && !cur.body.length) { cur.keywords = l.slice(11).trim(); continue; }
      cur.body.push(l);
    }
    for (const t of ch.topics) { t.md = t.body.join("\n"); t.text = (t.title + " " + t.keywords + " " + t.md).toLowerCase(); }
    return ch;
  }
  function load() {
    return loading ||= Promise.all(CHAPTERS.map(async f => {
      const r = await fetch(`hilfe/${f}.md`); if (!r.ok) throw new Error(`hilfe/${f}.md: ${r.status}`);
      return parse(f, await r.text());
    })).then(list => { chapters.push(...list); }, e => { loading = null; throw e; });
  }

  // ----- Display -----
  function build() {
    if (panel) return;
    const css = document.createElement("link"); css.rel = "stylesheet"; css.href = "help.css"; document.head.append(css);
    panel = document.createElement("aside"); panel.id = "help"; panel.hidden = true; panel.setAttribute("aria-label", "Hilfe");
    panel.innerHTML = `<div class="hh"><button type="button" data-h="back" title="Zurück">←</button><button type="button" data-h="fwd" title="Vorwärts">→</button>
      <button type="button" data-h="toc" title="Inhaltsverzeichnis">☰</button><input type="search" id="helpSearch" placeholder="Hilfe durchsuchen…" aria-label="Hilfe durchsuchen">
      <button type="button" data-h="close" title="Schliessen (Esc)">✕</button></div><div class="hb" id="helpBody"></div>`;
    document.body.append(panel);
    panel.addEventListener("click", e => {
      const a = e.target.closest("[data-topic]"); if (a) { e.preventDefault(); return show(a.dataset.topic); }
      const b = e.target.closest("[data-h]"); if (!b) return;
      ({ back: () => go(-1), fwd: () => go(1), toc: () => show("toc"), close: close })[b.dataset.h]();
    });
    $h("#helpSearch").addEventListener("input", e => { const q = e.target.value.trim(); q ? show("search:" + q, true) : show("toc"); });
  }
  const snippet = (t, q) => {
    const i = t.md.toLowerCase().indexOf(q), plain = t.md.replace(/[#*`|>\[\]()]/g, "").replace(/\s+/g, " ");
    if (i < 0) return esc(plain.slice(0, 120)) + "…";
    const j = plain.toLowerCase().indexOf(q), s = Math.max(0, j - 40);
    return (s ? "…" : "") + esc(plain.slice(s, j)) + `<mark>${esc(plain.slice(j, j + q.length))}</mark>` + esc(plain.slice(j + q.length, j + q.length + 90)) + "…";
  };
  function view(id) {
    if (id === "toc") return `<div class="toc"><h1>Handbuch</h1><p>Hilfe zum aktuellen Thema: <b>F1</b>. Hier das ganze Handbuch.</p>${chapters.map(c => `<h3>${esc(c.title)}</h3>${c.topics.map(t => `<a data-topic="${t.id}">${esc(t.title)}</a>`).join("")}`).join("")}</div>`;
    if (id.startsWith("search:")) {
      const q = id.slice(7).toLowerCase(), words = q.split(/\s+/).filter(Boolean);
      const hits = [...topics.values()].map(t => {
        if (!words.every(w => t.text.includes(w))) return null;
        const score = words.reduce((s, w) => s + (t.title.toLowerCase().includes(w) ? 5 : 0) + (t.keywords.toLowerCase().includes(w) ? 3 : 0) + Math.min(5, t.text.split(w).length - 1), 0);
        return { t, score };
      }).filter(Boolean).sort((a, b) => b.score - a.score).slice(0, 25);
      return `<h1>Suche: «${esc(id.slice(7))}»</h1>` + (hits.length ? hits.map(({ t }) => `<div class="hit"><a data-topic="${t.id}"><b>${esc(t.title)}</b></a><small>${esc(t.chapter.title)}</small><small>${snippet(t, words[0])}</small></div>`).join("") : "<p>Nichts gefunden. Versuchen Sie einen anderen Begriff.</p>");
    }
    const t = topics.get(id);
    if (!t) return `<h1>Thema nicht gefunden</h1><p>Zu «${esc(id)}» gibt es noch keinen Hilfetext. <a data-topic="toc">Zum Handbuch</a></p>`;
    return `<div class="crumb">${esc(t.chapter.title)}</div><h1>${esc(t.title)}</h1>${render(t.md)}`;
  }
  async function draw(id, keepSearch = false) {
    build(); panel.hidden = false; document.body.classList.add("help-open");
    try { await load(); } catch (e) { $h("#helpBody").innerHTML = `<h1>Hilfe nicht verfügbar</h1><p>${esc(e.message)}</p>`; return; }
    $h("#helpBody").innerHTML = view(id); $h("#helpBody").scrollTop = 0;
    if (!keepSearch) $h("#helpSearch").value = id.startsWith("search:") ? id.slice(7) : "";
    $h('[data-h="back"]').disabled = pos <= 0; $h('[data-h="fwd"]').disabled = pos >= hist.length - 1;
    document.documentElement.dataset.help = id;
  }
  function show(id, keepSearch = false) {
    if (hist[pos] !== id) {
      // Search queries while typing replace the last entry so that «Zurück» does not step through every letter.
      if (id.startsWith("search:") && hist[pos]?.startsWith("search:")) hist[pos] = id;
      else { hist = hist.slice(0, pos + 1); hist.push(id); pos = hist.length - 1; }
    }
    return draw(id, keepSearch);
  }
  function go(d) { const p = pos + d; if (p >= 0 && p < hist.length) { pos = p; draw(hist[pos]); } }
  function close() { if (!panel) return; panel.hidden = true; document.body.classList.remove("help-open"); delete document.documentElement.dataset.help; }
  const isOpen = () => !!panel && !panel.hidden;

  // ----- Context -----
  const topicFor = (list, key) => list.find(([re]) => re.test(key))?.[1];
  const fieldTopic = path => topicFor(FIELD_TOPICS, path);
  function contextTopic() {
    const el = document.activeElement;
    if (el && el !== document.body && !el.closest("#help")) {
      const own = el.dataset?.path ? el : el.closest?.("[data-path]");
      const t = (own && fieldTopic(own.dataset.path)) || (el.id && topicFor(OPT_TOPICS, el.id));
      if (t) return t;
    }
    if (!$("#results").hidden) {
      const r = document.documentElement.dataset.rendered;
      if (r === "runs") return "r-list";
      if (r === "compare") return "r-compare";
      if (typeof resultTab === "string") return "r-" + resultTab;
      return "r-overview";
    }
    return "s-" + current;
  }
  const openContext = () => show(contextTopic());

  // ----- Buttons at fields and headings -----
  const hq = topic => `<button type="button" class="hq" data-help="${topic}" title="Hilfe zu diesem Thema" aria-label="Hilfe">ⓘ</button>`;
  function put(container, where, topic) {
    if (!container || container.querySelector(":scope > .hq, :scope > * > .hq")) return;
    where.insertAdjacentHTML("beforeend", hq(topic));
  }
  function decorate() {
    const main = $("#main"), body = $("#resultsBody");
    if (main) {
      const h2 = main.querySelector("h2"); if (h2 && !h2.querySelector(".hq")) h2.insertAdjacentHTML("beforeend", hq("s-" + current));
      for (const el of main.querySelectorAll("[data-path]")) {
        const path = el.dataset.path, topic = fieldTopic(path); if (!topic) continue;
        const choice = el.classList.contains("choice") ? el : null;
        if (choice) { const b = choice.querySelector("b"); if (b && !b.querySelector(".hq")) b.insertAdjacentHTML("beforeend", hq(path === "returns.source" ? "f-ret-" + choice.dataset.value : "f-wd-" + choice.dataset.value)); continue; }
        const label = el.closest("label.field"), series = el.closest(".series");
        if (series) { const t = series.querySelector(".shead .t"); if (t && !t.querySelector(".hq")) t.insertAdjacentHTML("beforeend", hq(topic)); }
        else if (label) { const s = label.querySelector(":scope > span"); if (s && !s.querySelector(".hq")) s.insertAdjacentHTML("beforeend", hq(topic)); }
      }
      for (const el of main.querySelectorAll("input[id^=opt]")) {
        const topic = topicFor(OPT_TOPICS, el.id), s = el.closest("label")?.querySelector(":scope > span");
        if (topic && s && !s.querySelector(".hq")) s.insertAdjacentHTML("beforeend", hq(topic));
      }
    }
    for (const h of document.querySelectorAll("aside h3")) {
      if (!h.querySelector(".hq") && /^(Prüfung|Vorschau)/.test(h.textContent.trim())) h.insertAdjacentHTML("beforeend", hq("ref-checks"));
    }
    for (const root of [main, body]) {
      if (!root) continue;
      for (const h of root.querySelectorAll(".card h3")) {
        if (h.querySelector(".hq")) continue;
        const text = (h.firstChild?.textContent || "").trim(), hit = HEADING_TOPICS.find(([k]) => text.startsWith(k));
        if (hit) h.insertAdjacentHTML("beforeend", hq(hit[1]));
      }
    }
  }
  let scheduled = 0;
  const schedule = () => { cancelAnimationFrame(scheduled); scheduled = requestAnimationFrame(() => { obs.disconnect(); try { decorate(); } finally { watch(); } }); };
  const obs = new MutationObserver(schedule);
  const watch = () => { for (const id of ["main", "resultsBody"]) { const n = document.getElementById(id); if (n) obs.observe(n, { childList: true, subtree: true }); } };

  // ----- Check (for the user interface test) -----
  async function check() {
    await load();
    const problems = [];
    for (const [, id] of [...FIELD_TOPICS, ...OPT_TOPICS, ...HEADING_TOPICS]) if (!topics.has(id)) problems.push(`Thema fehlt: ${id}`);
    for (const id of ["general", "milestones", "pension", "pillar3a", "flows", "wealth", "returns", "withdrawal", "optimize", "taxes", "realestate", "scenarios"]) if (!topics.has("s-" + id)) problems.push(`Abschnitt ohne Thema: s-${id}`);
    for (const id of ["overview", "charts", "runs", "scenarios", "table", "list", "compare"]) if (!topics.has("r-" + id)) problems.push(`Ergebnisansicht ohne Thema: r-${id}`);
    for (const k of ["fixed", "historical", "rolling", "bootstrap", "parametric"]) if (!topics.has("f-ret-" + k)) problems.push(`Quelle ohne Thema: f-ret-${k}`);
    for (const k of ["standard", "three_buckets", "guardrails", "fixed_rate", "vpw", "endowment"]) if (!topics.has("f-wd-" + k)) problems.push(`Strategie ohne Thema: f-wd-${k}`);
    for (const t of topics.values()) for (const m of t.md.matchAll(/\]\(#([\w-]+)\)/g)) if (!topics.has(m[1])) problems.push(`Verweis ins Leere: ${t.id} → ${m[1]}`);
    return problems;
  }

  document.addEventListener("click", e => {
    const b = e.target.closest(".hq"); if (!b) return;
    e.preventDefault(); e.stopPropagation(); show(b.dataset.help);
  }, true);
  document.addEventListener("keydown", e => {
    if (e.key === "F1") { e.preventDefault(); openContext(); }
    else if ((e.metaKey || e.ctrlKey) && e.shiftKey && (e.key === "?" || e.key === "/")) { e.preventDefault(); show("toc"); }
    else if (e.key === "Escape" && isOpen() && e.target.closest?.("#help")) { e.preventDefault(); e.stopPropagation(); close(); }
  }, true);
  window.addEventListener("DOMContentLoaded", () => {
    document.querySelector('header [data-action="runs-open"]')?.insertAdjacentHTML("beforebegin",
      `<button class="btn" id="helpBtn" type="button" title="Hilfe zum aktuellen Thema (F1)">? Hilfe</button>`);
    document.getElementById("helpBtn")?.addEventListener("click", () => isOpen() ? close() : openContext());
    watch(); schedule();
    window.__TAURI__?.event?.listen("menu", ev => { if (ev.payload === "helpctx") openContext(); else if (ev.payload === "manual") show("toc"); });
    const open = /helpopen\/([\w-]+)/.exec(location.hash); // test access: …/helpopen/<topic>
    if (open) show(open[1]);
    if (location.hash.startsWith("#helpcheck")) check().then(p => { document.documentElement.dataset.helpProblems = p.length; if (p.length) document.documentElement.dataset.helpFirst = p[0]; document.documentElement.dataset.rendered = "help"; })
      .catch(e => { document.documentElement.dataset.helpProblems = "x"; document.documentElement.dataset.helpFirst = String(e.message); document.documentElement.dataset.rendered = "help"; });
  });
  return { show, close, openContext, check, topics };
})();
