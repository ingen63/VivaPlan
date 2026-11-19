// Checks on texts and documents, without a browser. Run: `node --test e2e/unit/` (or `make ui-unit`).
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

test("#54/AK6: «Liquidität» kommt in Oberfläche, Hilfe und Dokumentation nur als Suchbegriff vor", () => {
  // Allowed are the keyword lines of the help («Stichworte: …»): whoever still searches for «Liquidität» finds «Cash».
  // Excluded are the lists that quote old titles and criteria (acceptance list, old issue IDs, #117).
  const history = ["ABNAHME.md", "ISSUE-IDS.md"];
  const files = [
    ...walk(path.join(ROOT, "crates/app/ui"), [".js", ".html", ".md", ".css"]),
    ...walk(path.join(ROOT, "docs"), [".md"]).filter(p => !history.includes(path.basename(p))),
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

test("#54/AK6: die Feldnamen der Planungsdatei bleiben (wealth.cash, reserveMonths)", () => {
  const ui = fs.readFileSync(path.join(ROOT, "crates/app/ui/app.js"), "utf8");
  assert.match(ui, /wealth\.cash/);
  assert.match(ui, /reserveMonths/);
});

test("#48/AK7: Verweise in README, CLAUDE.md und docs/ auf Dateien im Repository führen nicht ins Leere", () => {
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

test("#93/AK1, #93/AK3: alle Stellen nennen dieselbe Programmversion, das Changelog beginnt mit ihr", () => {
  const read = f => fs.readFileSync(path.join(ROOT, f), "utf8");
  const toml = f => read(f).match(/^version\s*=\s*"([^"]+)"/m)?.[1];
  const versions = {
    "Cargo.toml": toml("Cargo.toml"),
    "crates/app/Cargo.toml": toml("crates/app/Cargo.toml"),
    "crates/app/tauri.conf.json": JSON.parse(read("crates/app/tauri.conf.json")).version,
    "crates/app/package.json": JSON.parse(read("crates/app/package.json")).version,
  };
  const v = versions["Cargo.toml"];
  assert.match(v, /^\d+\.\d+\.\d+(-beta\.\d+)?$/); // Pre-release versions only as «-beta.N»
  for (const [f, x] of Object.entries(versions)) assert.equal(x, v, f);
  // Cargo.lock: the own packages carry the same version
  for (const lock of ["Cargo.lock", "fuzz/Cargo.lock"]) {
    for (const m of read(lock).matchAll(/name = "(vivaplan-[a-z]+)"\nversion = "([^"]+)"/g)) {
      if (m[1] !== "vivaplan-fuzz") assert.equal(m[2], v, `${lock}: ${m[1]}`);
    }
  }
  // A section «Nächste Version» (changes not yet released) comes before the versions and does not count.
  const heads = [...read("CHANGELOG.md").matchAll(/^## (.+)$/gm)].map(m => m[1]).filter(h => h !== "Nächste Version");
  assert.ok(heads[0].startsWith(`Version ${v}`), `oberster Abschnitt: ${heads[0]}`);
  assert.equal(heads.at(-1), "Version 0.8.0: erster öffentlicher Stand");
  // Since 0.8.0 (restart of the numbering) only 0.8.x and 1.0.0-beta.N; no entries of the earlier project (1.1.x).
  assert.ok(heads.every(h => /^Version (0\.8\.\d+|1\.0\.0-beta\.\d+)\b/.test(h)), "keine Einträge zu früheren Versionen (1.1.x)");
  assert.equal(fs.existsSync(path.join(ROOT, "docs/release-notes-1.1.3.md")), false);
});
