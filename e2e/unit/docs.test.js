// Prüfungen an Texten und Dokumenten, ohne Browser. Ausführen: `node --test e2e/unit/` (oder `make ui-unit`).
const { test } = require("node:test");
const assert = require("node:assert/strict");
const fs = require("fs");
const path = require("path");

const ROOT = path.resolve(__dirname, "../..");
const walk = (dir, ext, out = []) => {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) walk(p, ext, out);
    else if (ext.some(x => p.endsWith(x))) out.push(p);
  }
  return out;
};
const rel = p => path.relative(ROOT, p);

test("FEA-40/AK6: «Liquidität» kommt in Oberfläche, Hilfe und Dokumentation nur als Suchbegriff vor", () => {
  // Erlaubt sind die Stichwortzeilen der Hilfe («Stichworte: …»): Wer noch «Liquidität» sucht, findet «Cash».
  const files = [
    ...walk(path.join(ROOT, "crates/app/ui"), [".js", ".html", ".md", ".css"]),
    ...walk(path.join(ROOT, "docs"), [".md"]).filter(p => path.basename(p) !== "ABNAHME.md"),
    path.join(ROOT, "README.md"), path.join(ROOT, "CLAUDE.md"),
  ];
  const hits = [];
  for (const f of files) {
    fs.readFileSync(f, "utf8").split("\n").forEach((line, i) => {
      if (/Liquidit/.test(line) && !/^Stichworte:/.test(line)) hits.push(`${rel(f)}:${i + 1}: ${line.trim().slice(0, 100)}`);
    });
  }
  assert.deepEqual(hits, []);
});

test("FEA-40/AK6: die Feldnamen der Planungsdatei bleiben (wealth.cash, reserveMonths)", () => {
  const ui = fs.readFileSync(path.join(ROOT, "crates/app/ui/app.js"), "utf8");
  assert.match(ui, /wealth\.cash/);
  assert.match(ui, /reserveMonths/);
});

test("FEA-35/AK7: Verweise in README, CLAUDE.md und docs/ auf Dateien im Repository führen nicht ins Leere", () => {
  const files = [path.join(ROOT, "README.md"), path.join(ROOT, "CLAUDE.md"), ...walk(path.join(ROOT, "docs"), [".md"])];
  const dead = [];
  for (const f of files) {
    const text = fs.readFileSync(f, "utf8").replace(/```[\s\S]*?```/g, "");
    for (const m of text.matchAll(/\[[^\]]*\]\(([^)\s]+)\)/g)) {
      let href = m[1];
      if (/^(https?:|mailto:|#)/.test(href)) continue;
      href = decodeURIComponent(href.split("#")[0]);
      if (href && !fs.existsSync(path.resolve(path.dirname(f), href))) dead.push(`${rel(f)}: ${m[1]}`);
    }
  }
  assert.deepEqual(dead, []);
});
