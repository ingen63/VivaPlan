#!/usr/bin/env python3
"""Übersichtsseite «Testqualität» (FEA-30, FEA-31).

Erzeugt eine statische HTML-Seite (ohne Netzwerk, ohne externe Skripte oder Schriften) aus den vorhandenen
Ergebnissen: nextest-JUnit, lcov, local/mutants.out, Playwright-JSON und (nur beim Erzeugen, über `gh`) den GitHub-Issues.
Es gibt keine Gesamtnote; jede Schicht hat ihren eigenen Status nach der auf der Seite genannten Regel.

Aufruf: report.py [--out <Ordner>] [--summary]
  --out      Ausgabeordner (Vorgabe local/test-quality/report)
  --summary  zusätzlich eine Kurzfassung als Markdown auf die Standardausgabe (GitHub-Lauf-Zusammenfassung)
"""
import glob
import html
import json
import os
import re
import subprocess
import sys
import xml.etree.ElementTree as ET
from datetime import datetime
from pathlib import Path

ROOT = Path(subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, check=True).stdout.strip())
os.chdir(ROOT)
sys.path.insert(0, str(ROOT / "tools"))
MIN_COVERAGE = float((ROOT / "tools" / "coverage-min").read_text().strip())
PACKAGES = {"Engine (vivaplan-engine)": "crates/engine/", "App (vivaplan-app)": "crates/app/"}
OK, HINT, FAIL, NODATA = "erfüllt", "Hinweis", "nicht erfüllt", "keine Daten"
E = html.escape


def sh(*cmd, check=False):
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
        return r.stdout if (r.returncode == 0 or not check) else None
    except (OSError, subprocess.TimeoutExpired):
        return None


def first(*paths):
    for p in paths:
        hits = sorted(glob.glob(p), key=os.path.getmtime, reverse=True)
        if hits:
            return Path(hits[0])
    return None


def when(p):
    return datetime.fromtimestamp(os.path.getmtime(p)).strftime("%d.%m.%Y %H:%M") if p else None


def version():
    m = re.search(r'^version\s*=\s*"([^"]+)"', (ROOT / "Cargo.toml").read_text(), re.M)
    return m.group(1) if m else "?"


# ---------- Hygiene ----------
def read_junit():
    f = first("target/nextest/ci/junit.xml", "junit.xml", "target/nextest/*/junit.xml")
    if not f:
        return None
    r = {"file": f, "when": when(f), "tests": [], "failed": [], "flaky": [], "skipped": 0}
    for tc in ET.parse(f).getroot().iter("testcase"):
        name = f'{tc.get("classname", "")} {tc.get("name", "")}'.strip()
        t = float(tc.get("time", 0) or 0)
        r["tests"].append((name, t))
        if tc.find("failure") is not None or tc.find("error") is not None:
            r["failed"].append(name)
        if tc.find("flakyFailure") is not None or tc.find("flakyError") is not None:
            r["flaky"].append(name)
        if tc.find("skipped") is not None:
            r["skipped"] += 1
    return r


def run_time(iso):
    """Beginn eines Playwright-Laufs (ISO 8601, UTC) als lokale Zeit; ungültig oder fehlend: None."""
    try:
        return datetime.fromisoformat(iso.replace("Z", "+00:00")).astimezone().strftime("%d.%m.%Y %H:%M")
    except (AttributeError, ValueError):
        return None


def read_playwright():
    f = first("e2e/results.json")
    if not f:
        return None
    s = json.loads(f.read_text()).get("stats", {})
    return {"when": run_time(s.get("startTime")) or when(f), "ok": s.get("expected", 0), "failed": s.get("unexpected", 0), "flaky": s.get("flaky", 0), "skipped": s.get("skipped", 0)}


# ---------- Angemessenheit ----------
def read_lcov():
    lcov = ROOT / "local" / "test-quality" / "lcov.info"
    if sh("cargo", "llvm-cov", "report", "--lcov", "--output-path", str(lcov), check=True) is None and not lcov.exists():
        lcov = first("lcov.info", "local/test-quality/lcov.info")
    if not lcov or not Path(lcov).exists():
        return None
    from coverage_diff import read_lcov as parse
    data = parse(str(lcov))
    res = {"file": Path(lcov), "when": when(lcov), "pkgs": {}}
    for label, prefix in PACKAGES.items():
        lines = [h for f, d in data.items() if f.startswith(prefix) for h in d.values()]
        res["pkgs"][label] = (100 * sum(1 for h in lines if h > 0) / len(lines), len(lines)) if lines else None
    return res


def read_mutants():
    d = first("local/mutants.out", "local/test-quality/reports/mutants.out", "mutants-shard-*/")
    if not d or not (Path(d) / "outcomes.json").exists():
        return None
    o = json.loads((Path(d) / "outcomes.json").read_text())
    missed = [l.strip() for l in (Path(d) / "missed.txt").read_text().splitlines() if l.strip()] if (Path(d) / "missed.txt").exists() else []
    return {"when": when(Path(d) / "outcomes.json"), "caught": o.get("caught", 0), "missed": o.get("missed", 0), "unviable": o.get("unviable", 0), "timeout": o.get("timeout", 0), "list": missed}


def read_diff_coverage(lcov):
    try:
        from coverage_diff import changed_lines, read_lcov as parse
        # Basis: im Workflow der Stand vor dem Push bzw. der Basis-Zweig (DIFF_BASE), lokal origin/main (FEA-28)
        base = os.environ.get("DIFF_BASE") or "origin/main"
        changed = changed_lines(base, working_tree=True)
        if not changed:
            return {"none": True, "base": base}
        cov = parse(str(lcov))
        total = covered = 0
        for f, lines in changed.items():
            for n, h in cov.get(f, {}).items():
                if n in lines:
                    total += 1
                    covered += h > 0
        return {"pct": 100 * covered / total, "total": total, "covered": covered, "base": base} if total else {"none": True, "base": base}
    except Exception:
        return None


def proptest_info(junit):
    n = sum(1 for name, _ in junit["tests"] if "proptest" in name) if junit else None
    wf = (ROOT / ".github/workflows/test-quality.yml").read_text()
    m = re.search(r"'(\d+)' \|\| '(\d+)'", wf)
    return {"tests": n, "cases": f"{m.group(2)} je Test (Pull Request), {m.group(1)} (nächtlich)" if m else "?"}


def fuzz_info():
    targets = sorted(p.stem for p in (ROOT / "fuzz" / "fuzz_targets").glob("*.rs"))
    crashes = len(list((ROOT / "fuzz" / "artifacts").glob("*/*"))) if (ROOT / "fuzz" / "artifacts").exists() else 0
    return {"targets": targets, "crashes": crashes}


# ---------- Anforderungsbezug und Ergebnis (nur über gh) ----------
def read_issues():
    out = sh("gh", "issue", "list", "--state", "all", "--limit", "500", "--json", "number,title,state,labels,body", check=True)
    try:
        return json.loads(out) if out else None
    except ValueError:
        return None


MARK = re.compile(r"\b(FEA|ISS|GEN)-(\d+)/AK(\d+)\b", re.I)
NAMEMARK = re.compile(r"\b(fea|iss|gen)_(\d+)_ak(\d+)", re.I)


def test_marks():
    """Kennzeichnung der Tests: `FEA-100/AK2` (Kommentar, Testname) oder `fea_100_ak2_…`; {(ID, Nr): [Fundstellen]}."""
    files = []
    for pat in ("crates/engine/**/*.rs", "crates/cli/**/*.rs", "crates/app/src/**/*.rs", "e2e/*.js"):
        files += glob.glob(pat, recursive=True)
    marks = {}
    for f in files:
        if "/target/" in f or "node_modules" in f:
            continue
        for i, line in enumerate(open(f, encoding="utf-8", errors="ignore"), 1):
            for rx in (MARK, NAMEMARK):
                for m in rx.finditer(line):
                    marks.setdefault((f"{m.group(1).upper()}-{int(m.group(2))}", int(m.group(3))), []).append(f"{f}:{i}")
    return marks


def issue_id(title):
    m = re.match(r"\s*((?:FEA|ISS|GEN)-\d+)\b", title)
    return m.group(1) if m else None


def requirements(issues, marks):
    rows = []
    for it in sorted(issues, key=lambda i: i["number"]):
        iid = issue_id(it["title"])
        crit = re.findall(r"^\s*- \[( |x|X)\] (.+)$", it["body"] or "", re.M)
        if not iid or not crit:
            continue
        items = []
        for n, (done, text) in enumerate(crit, 1):
            ev = marks.get((iid, n), [])
            items.append(("test" if ev else "manuell" if done != " " else "ohne", text.strip(), ev))
        rows.append({"id": iid, "nr": it["number"], "title": it["title"], "state": it["state"], "items": items})
    return rows


def escapes(issues):
    rows = []
    for it in issues:
        labels = [l["name"] for l in it["labels"]]
        found = [l for l in labels if l.startswith("gefunden:")]
        if "bug" not in labels or not found:
            continue
        body = it["body"] or ""
        def para(key):
            m = re.search(r"\*\*" + key + r"[^*]*\*\*:?\s*(.+?)(?:\n\n|\Z)", body, re.S | re.I)
            return m.group(1).strip() if m else None
        rows.append({"nr": it["number"], "title": it["title"], "found": found[0].split(":", 1)[1], "which": para("Welcher Test"), "reg": para("Regressionstest")})
    return rows


# ---------- Verlauf ----------
def history(version_, metrics):
    d = ROOT / "local" / "test-quality" / "history"
    d.mkdir(parents=True, exist_ok=True)
    others = sorted((p for p in d.glob("*.json") if p.stem != version_), key=os.path.getmtime)
    prev = (others[-1].stem, json.loads(others[-1].read_text())) if others else None
    (d / f"{version_}.json").write_text(json.dumps(metrics, indent=1))
    return prev


# ---------- Statusregeln ----------
def status_hygiene(junit):
    if not junit:
        return NODATA
    return FAIL if junit["failed"] else HINT if junit["flaky"] else OK


def status_fit(cov, mut):
    if not cov:
        return NODATA
    pcts = [v[0] for v in cov["pkgs"].values() if v]
    if len(pcts) < len(PACKAGES) or min(pcts) < MIN_COVERAGE:
        return FAIL
    return HINT if (mut is None or mut["missed"] > 0) else OK


def fmt(v, suffix=" %"):
    return "–" if v is None else f"{v:.1f}".replace(".", ",") + suffix


def delta(cur, prev, better_high=True):
    if cur is None or prev is None:
        return "–"
    d = cur - prev
    return "unverändert" if abs(d) < 0.05 else f"{'+' if d > 0 else '−'}{abs(d):.1f}".replace(".", ",") + (" ▲" if (d > 0) == better_high else " ▼")


CSS = """
:root{--bg:#fff;--fg:#1d2330;--muted:#5b6475;--line:#d9dde5;--ok:#1a7f4b;--hint:#a66a00;--fail:#c0262d;--none:#6b7280;--card:#f6f7fa}
@media (prefers-color-scheme:dark){:root{--bg:#14171d;--fg:#e6e9ef;--muted:#9aa3b2;--line:#2c313c;--ok:#4cc38a;--hint:#e0a030;--fail:#ff6b6b;--none:#9ca3af;--card:#1c2028}}
body{background:var(--bg);color:var(--fg);font:15px/1.5 -apple-system,BlinkMacSystemFont,"Segoe UI",Helvetica,Arial,sans-serif;max-width:980px;margin:0 auto;padding:24px 16px}
h1{margin:0 0 4px}h2{margin:28px 0 6px}small,.muted{color:var(--muted)}section{border:1px solid var(--line);border-radius:10px;padding:14px 18px;margin:16px 0;background:var(--card)}
.badge{display:inline-block;padding:2px 10px;border-radius:12px;font-weight:600;font-size:13px;border:1px solid currentColor}
.erfüllt{color:var(--ok)}.Hinweis{color:var(--hint)}.nicht{color:var(--fail)}.keine{color:var(--none)}
table{border-collapse:collapse;width:100%;margin:8px 0}th,td{text-align:left;padding:4px 8px;border-bottom:1px solid var(--line);vertical-align:top;font-size:14px}td.n,th.n{text-align:right;white-space:nowrap}
details{margin:8px 0}summary{cursor:pointer;font-weight:600}code{font-size:13px}.rule{font-size:13px;color:var(--muted);margin:2px 0 10px}
"""


def badge(status):
    cls = {OK: "erfüllt", HINT: "Hinweis", FAIL: "nicht", NODATA: "keine"}[status]
    return f'<span class="badge {cls}">{status}</span>'


def table(head, rows, num=()):
    th = "".join(f'<th class="{"n" if i in num else ""}">{E(h)}</th>' for i, h in enumerate(head))
    body = "".join("<tr>" + "".join(f'<td class="{"n" if i in num else ""}">{c}</td>' for i, c in enumerate(r)) + "</tr>" for r in rows)
    return f"<table><tr>{th}</tr>{body}</table>"


def link(path, text):
    p = ROOT / path
    return f'<a href="file://{E(str(p))}">{E(text)}</a>' if p.exists() else ""


def main():
    out = Path(sys.argv[sys.argv.index("--out") + 1]) if "--out" in sys.argv else ROOT / "local" / "test-quality" / "report"
    out.mkdir(parents=True, exist_ok=True)
    ver = version()
    commit = (sh("git", "rev-parse", "--short", "HEAD") or "?").strip()
    now = datetime.now().strftime("%d.%m.%Y %H:%M")

    junit, pw, cov, mut = read_junit(), read_playwright(), read_lcov(), read_mutants()
    diffcov = read_diff_coverage(cov["file"]) if cov else None
    issues = read_issues()
    reqs = requirements(issues, test_marks()) if issues is not None else None
    esc = escapes(issues) if issues is not None else None
    st_h, st_f = status_hygiene(junit), status_fit(cov, mut)
    st_r = NODATA if reqs is None else (OK if reqs and all(i[0] != "ohne" for r in reqs for i in r["items"]) else HINT)
    st_e = NODATA if not esc else (OK if all(e["which"] and e["reg"] for e in esc) else HINT)

    n_crit = sum(len(r["items"]) for r in reqs) if reqs else None
    n_test = sum(1 for r in reqs for i in r["items"] if i[0] == "test") if reqs else None
    metrics = {"coverage_engine": (cov["pkgs"].get(list(PACKAGES)[0]) or [None])[0] if cov else None,
               "coverage_app": (cov["pkgs"].get(list(PACKAGES)[1]) or [None])[0] if cov else None,
               "missed": mut["missed"] if mut else None, "flaky": len(junit["flaky"]) if junit else None,
               "criteria_share": round(100 * n_test / n_crit, 1) if n_crit else None}
    prev = history(ver, metrics)

    h = [f"<!doctype html><html lang='de'><head><meta charset='utf-8'><meta name='viewport' content='width=device-width,initial-scale=1'><title>Testqualität</title><style>{CSS}</style></head><body>",
         f"<h1>Testqualität</h1><p class='muted'>Version {E(ver)} · Commit <code>{E(commit)}</code> · erzeugt {now}. Es gibt keine Gesamtnote: Jede Schicht hat ihren eigenen Status nach der genannten Regel, die Hinweise stehen einzeln darunter.</p>"]

    # Hygiene
    h.append(f"<section><h2>1 Hygiene {badge(st_h)}</h2><p class='rule'>Regel: erfüllt = keine fehlgeschlagenen und keine als flaky markierten Tests; Hinweis = flaky Tests; nicht erfüllt = mindestens ein fehlgeschlagener Test; keine Daten = kein JUnit-Ergebnis (nextest, Profil ci) vorhanden.</p>")
    if junit:
        total = sum(t for _, t in junit["tests"])
        h.append(f"<p>{len(junit['tests'])} Tests, {len(junit['failed'])} fehlgeschlagen, {junit['skipped']} übersprungen, {len(junit['flaky'])} flaky · Laufzeit der Tests zusammen {total:.0f} s <small>(Stand {junit['when']})</small></p>")
        if junit["failed"]:
            h.append("<p><b>Fehlgeschlagen:</b></p><ul>" + "".join(f"<li><code>{E(n)}</code></li>" for n in junit["failed"]) + "</ul>")
        if junit["flaky"]:
            h.append("<p><b>Flaky:</b></p><ul>" + "".join(f"<li><code>{E(n)}</code></li>" for n in junit["flaky"]) + "</ul>")
        slow = sorted(junit["tests"], key=lambda x: -x[1])[:10]
        h.append("<details><summary>Die zehn langsamsten Tests</summary>" + table(["Test", "Sekunden"], [(f"<code>{E(n)}</code>", f"{t:.1f}") for n, t in slow], num=(1,)) + "</details>")
    else:
        h.append("<p class='muted'>Keine Daten: <code>make test</code> mit dem Profil ci (<code>cargo nextest run --profile ci</code>) erzeugt <code>target/nextest/ci/junit.xml</code>.</p>")
    if pw:
        h.append(f"<p>Oberfläche (Playwright): {pw['ok']} bestanden, {pw['failed']} fehlgeschlagen, {pw['flaky']} flaky, {pw['skipped']} übersprungen <small>(Stand {pw['when']})</small> {link('e2e/playwright-report/index.html', 'Bericht')}</p>")
    else:
        h.append("<p class='muted'>Oberfläche (Playwright): keine Daten (<code>e2e/results.json</code> fehlt; <code>npx playwright test</code> erzeugt sie).</p>")
    h.append("</section>")

    # Angemessenheit
    h.append(f"<section><h2>2 Angemessenheit {badge(st_f)}</h2><p class='rule'>Regel: erfüllt = beide Pakete mindestens {MIN_COVERAGE:.0f} % Zeilenabdeckung und keine nicht erkannten Mutanten; Hinweis = Abdeckung erreicht, aber nicht erkannte Mutanten oder kein Mutationslauf vorhanden; nicht erfüllt = ein Paket unter {MIN_COVERAGE:.0f} %; keine Daten = keine Abdeckung vorhanden.</p>")
    if cov:
        h.append("<h3>Zeilenabdeckung je Paket</h3>" + table(["Paket", "Abdeckung", "Untergrenze", "Zeilen"], [
            (E(label), (f"<b class='{'erfüllt' if v[0] >= MIN_COVERAGE else 'nicht'}'>{fmt(v[0])}</b>" if v else "keine Daten"), f"{MIN_COVERAGE:.0f} %", str(v[1]) if v else "–") for label, v in cov["pkgs"].items()], num=(1, 2, 3)) + f"<small>Stand {cov['when']} {link('target/llvm-cov/html/index.html', '· Abdeckung als HTML')}</small>")
        if diffcov and diffcov.get("none"):
            h.append(f"<p>Abdeckung des geänderten Codes: keine Rust-Änderungen gegenüber <code>{E(diffcov['base'])}</code>.</p>")
        elif diffcov:
            h.append(f"<p>Abdeckung des geänderten Codes (gegenüber <code>{E(diffcov['base'])}</code>): <b>{fmt(diffcov['pct'])}</b> ({diffcov['covered']} von {diffcov['total']} ausführbaren Zeilen). Ein Hinweis, keine Sperre; Zeilen: <code>make coverage-diff</code>.</p>")
    else:
        h.append("<p class='muted'>Keine Abdeckung: <code>make coverage</code> erzeugt sie (benötigt cargo-llvm-cov).</p>")
    h.append("<h3>Mutationstest</h3>")
    if mut:
        h.append(f"<p>caught {mut['caught']} · <b>missed {mut['missed']}</b> · unviable {mut['unviable']} · timeout {mut['timeout']} · geprüfte Mutanten insgesamt {mut['caught'] + mut['missed'] + mut['unviable'] + mut['timeout']} <small>(Stand {mut['when']}; ein Lauf auf einzelne Dateien oder Änderungen ist kein voller Lauf)</small></p><p class='rule'>Kein Mutation Score als Ziel: Jeder nicht erkannte Mutant wird einzeln mit einem Test getötet oder mit Begründung ausgeschlossen.</p>")
        h.append(f"<details {'open' if mut['list'] else ''}><summary>Nicht erkannte Mutanten ({len(mut['list'])})</summary>" + ("<ul>" + "".join(f"<li><code>{E(l)}</code></li>" for l in mut["list"]) + "</ul>" if mut["list"] else "<p>Keine.</p>") + "</details>")
    else:
        h.append("<p class='muted'>Keine Daten: kein Mutationslauf vorhanden (<code>make mutants</code>; er dauert Stunden, <code>make mutants-diff</code> prüft nur die Änderung).</p>")
    pt, fz = proptest_info(junit), fuzz_info()
    h.append(f"<h3>Property-Tests und Fuzzing</h3><p>Property-Tests: {pt['tests'] if pt['tests'] is not None else 'keine Daten'} Tests mit «proptest» im Namen, Fallzahl {E(pt['cases'])}.</p><p>Fuzzing: {len(fz['targets'])} Ziele ({E(', '.join(fz['targets']))}), {fz['crashes']} gespeicherte Abstürze in <code>fuzz/artifacts</code>; letzter Lauf: keine Daten (läuft nächtlich auf GitHub).</p></section>")

    # Anforderungsbezug
    h.append(f"<section><h2>3 Anforderungsbezug {badge(st_r)}</h2><p class='rule'>Regel: erfüllt = jedes Abnahmekriterium ist durch einen gekennzeichneten Test belegt oder abgehakt; Hinweis = es gibt Kriterien ohne Test und ohne Haken; keine Daten = keine Verbindung zu GitHub (<code>gh</code>). Ein Test wird mit <code>FEA-100/AK2</code> im Kommentar oder Namen (<code>fea_100_ak2_…</code>) gekennzeichnet; AK2 ist das zweite Kriterium des Issues.</p>")
    if reqs is None:
        h.append("<p class='muted'>Keine Daten: <code>gh</code> ist nicht angemeldet oder nicht erreichbar.</p>")
    else:
        rows = []
        for r in reqs:
            t = sum(1 for i in r["items"] if i[0] == "test"); m = sum(1 for i in r["items"] if i[0] == "manuell"); n = len(r["items"])
            det = "<details><summary>Kriterien</summary><ul>" + "".join(f"<li>{ {'test': '✔ Test', 'manuell': '☑ nur manuell geprüft', 'ohne': '○ ohne Test'}[k] }: {E(tx)}" + (f" <small><code>{E(', '.join(ev[:2]))}</code></small>" if ev else "") + "</li>" for k, tx, ev in r["items"]) + "</ul></details>"
            rows.append((f"#{r['nr']} {E(r['title'])}{det}", f"{t} von {n}", str(m), str(n - t - m)))
        h.append(table(["Issue", "mit Test", "nur manuell", "ohne"], rows, num=(1, 2, 3)))
    h.append("</section>")

    # Ergebnis
    h.append(f"<section><h2>4 Ergebnis (durchgerutschte Fehler) {badge(st_e)}</h2><p class='rule'>Regel: Bug-Issues mit dem Label <code>gefunden:test</code>, <code>gefunden:abnahme</code> oder <code>gefunden:nutzung</code> (Fundort); erfüllt = jedes beantwortet «Welcher Test hätte ihn fangen müssen?» und nennt den Regressionstest; Hinweis = eine Antwort fehlt; keine Daten = keine Verbindung zu GitHub oder noch kein Fehler mit Fundort-Label erfasst.</p>")
    if esc is None:
        h.append("<p class='muted'>Keine Daten: <code>gh</code> ist nicht angemeldet oder nicht erreichbar.</p>")
    elif not esc:
        h.append("<p class='muted'>Keine Daten: noch kein Bug-Issue mit Fundort-Label erfasst.</p>")
    else:
        h.append(table(["Issue", "Gefunden bei", "Welcher Test hätte ihn fangen müssen?", "Regressionstest"], [(f"#{e['nr']} {E(e['title'])}", E(e["found"]), E(e["which"] or "–"), E(e["reg"] or "–")) for e in esc]))
    h.append("</section>")

    # Verlauf
    rows = []
    cur = metrics
    for key, label, better in (("coverage_engine", "Abdeckung Engine", True), ("coverage_app", "Abdeckung App", True), ("missed", "Nicht erkannte Mutanten", False), ("flaky", "Flaky Tests", False), ("criteria_share", "Kriterien mit Test (Anteil)", True)):
        p = prev[1].get(key) if prev else None
        rows.append((label, fmt(cur[key], "") if cur[key] is not None else "–", fmt(p, "") if p is not None else "–", delta(cur[key], p, better)))
    h.append(f"<section><h2>Verlauf</h2><p class='rule'>Vergleich mit {'Version ' + E(prev[0]) if prev else 'keiner vorherigen Version (erster Lauf; die Werte je Version stehen in <code>local/test-quality/history/</code>)'}.</p>" + table(["Kennzahl", f"Version {ver}", "vorher", "Veränderung"], rows, num=(1, 2)) + "</section>")
    h.append("</body></html>")
    (out / "index.html").write_text("\n".join(h), encoding="utf-8")
    print(f"Seite: {out / 'index.html'}", file=sys.stderr)

    if "--summary" in sys.argv:
        lines = [f"### Testqualität (Version {ver}, Commit {commit})", "", f"- **Hygiene:** {st_h}" + (f" ({len(junit['tests'])} Tests, {len(junit['failed'])} fehlgeschlagen, {len(junit['flaky'])} flaky)" if junit else ""),
                 f"- **Angemessenheit:** {st_f}" + (" · Abdeckung " + ", ".join(f"{k.split(' ')[0]} {fmt(v[0]) if v else '–'}" for k, v in cov['pkgs'].items()) if cov else "") + (f" · nicht erkannte Mutanten: {mut['missed']}" if mut else " · kein Mutationslauf"),
                 f"- **Anforderungsbezug:** {st_r}" + (f" ({n_test} von {n_crit} Kriterien mit Test)" if n_crit else ""), f"- **Ergebnis:** {st_e}"]
        print("\n".join(lines))


if __name__ == "__main__":
    main()
