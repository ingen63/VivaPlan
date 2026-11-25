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

test("#54/AK6: «Liquidität» occurs in the user interface, help and documentation only as a search term", () => { // check-english: allow (searched term)
  // Allowed are the keyword lines of the help («Stichworte: …»): whoever still searches for «Liquidität» finds «Cash».
  // Excluded are the lists that quote old titles and criteria (acceptance list, old issue IDs, #117).
  const history = ["ABNAHME.md", "ISSUE-IDS.md"];
  const files = [
    ...walk(path.join(ROOT, "crates/app/ui"), [".js", ".html", ".md", ".css", ".json"]), // #170: texts in i18n/*.json
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

test("#54/AK6: the field names of the plan file remain (wealth.cash, reserveMonths)", () => {
  const ui = fs.readFileSync(path.join(ROOT, "crates/app/ui/app.js"), "utf8");
  assert.match(ui, /wealth\.cash/);
  assert.match(ui, /reserveMonths/);
});

test("#48/AK7: links in README, CLAUDE.md and docs/ to files in the repository do not lead nowhere", () => {
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

// #267: the four README files (German and the translations).
const READMES = ["README.md", "README.en.md", "README.fr.md", "README.it.md"].map(f => path.join(ROOT, f));
// Anchor of a heading as GitHub builds it: lower case, punctuation removed, spaces become hyphens.
const slug = h => h.trim().toLowerCase().replace(/[^\p{L}\p{N}\s-]/gu, "").replace(/\s/g, "-");

test("#267/AK5: links in all four README lead to existing files and headings", () => {
  const dead = [];
  for (const f of READMES) {
    const raw = fs.readFileSync(f, "utf8");
    const anchors = new Set([...raw.matchAll(/^#{1,6} (.+)$/gm)].map(m => slug(m[1])));
    const text = raw.replace(/```[\s\S]*?```/g, "");
    const hrefs = [...text.matchAll(/\[[^\]]*\]\(([^)\s]+)\)/g), ...text.matchAll(/href="([^"]+)"/g)].map(m => m[1]);
    for (const href of hrefs) {
      if (/^(https?:|mailto:)/.test(href)) continue;
      if (href.startsWith("#")) { if (!anchors.has(decodeURIComponent(href.slice(1)))) dead.push(`${rel(f)}: ${href}`); continue; }
      const file = decodeURIComponent(href.split("#")[0]);
      if (!fs.existsSync(path.resolve(path.dirname(f), file))) dead.push(`${rel(f)}: ${href}`);
    }
    for (const m of raw.matchAll(/<img src="([^"]+)"/g)) {
      if (!/^https?:/.test(m[1]) && !fs.existsSync(path.resolve(path.dirname(f), m[1]))) dead.push(`${rel(f)}: ${m[1]}`);
    }
  }
  assert.deepEqual(dead, []);
});

test("#267/AK1, #267/AK3, #267/AK4: the four README have the same structure and name the AI connection and the packages", () => {
  const shape = f => {
    const text = fs.readFileSync(f, "utf8");
    return {
      sections: (text.match(/^## /gm) || []).length,
      bullets: (text.match(/^- /gm) || []).length,
      tableRows: (text.match(/^\| /gm) || []).length,
      quotes: (text.match(/^(?!> ).*\n> /gm) || []).length, // quote blocks, not lines
    };
  };
  const de = shape(READMES[0]);
  for (const f of READMES.slice(1)) assert.deepEqual(shape(f), de, rel(f));
  for (const f of READMES) {
    const text = fs.readFileSync(f, "utf8");
    for (const word of ["vivaplan mcp", ".mcpb", ".dmg", ".msi", "-setup.exe", ".deb", ".AppImage", "LM Studio", "Claude Code", "ChatGPT", "releases"]) {
      assert.ok(text.includes(word), `${rel(f)}: ${word}`);
    }
    // The AI connection is pointed to near the top: the introduction links to its own section.
    const intro = text.slice(0, text.indexOf("\n## ", text.indexOf("\n## ") + 1));
    assert.match(intro, /\]\(#[^)]+\)/, `${rel(f)}: link to the AI section in the introduction`);
  }
});

test("#93/AK1, #93/AK3: all places name the same program version, the changelog starts with it", () => {
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
  assert.ok(heads[0].startsWith(`Version ${v}`), `top section: ${heads[0]}`);
  assert.equal(heads.at(-1), "Version 0.8.0: erster öffentlicher Stand");
  // Since 0.8.0 (restart of the numbering) only 0.8.x and 1.0.0-beta.N; no entries of the earlier project (1.1.x).
  assert.ok(heads.every(h => /^Version (0\.8\.\d+|1\.0\.0-beta\.\d+)\b/.test(h)), "no entries for earlier versions (1.1.x)");
  assert.equal(fs.existsSync(path.join(ROOT, "docs/release-notes-1.1.3.md")), false);
});

// #183/AK5: The help names no limit of the scenario combinations other than MAX_COMBINATIONS of the engine.
// A sentence counts as naming a limit if it contains a word for «combination» and a limit word; every number in it
// must then equal MAX_COMBINATIONS (examples like «35 combinations with 50 000 paths» name no limit).
const COMBINATION = /kombination|combination|combinaison|combinazion/i;
const LIMIT = /höchstens|maximal|mehr als|nicht begrenzt|erlaubt|at most|maximum|beyond|not limited|allowed|au plus|au-delà|pas limité|admises|al massimo|più di|non è limitato|ammesse/i; // check-english: allow (searched terms)
const limitNumbers = text => text
  .split(/(?<=[.!?])\s+|\n/)
  .filter(s => COMBINATION.test(s) && LIMIT.test(s))
  .flatMap(s => (s.match(/\d(?:[\d'’   ]*\d)?/g) || []).map(n => ({ n: Number(n.replace(/\D/g, "")), s })));

test("#183/AK5: the help names no number of combinations other than MAX_COMBINATIONS", () => {
  const src = fs.readFileSync(path.join(ROOT, "crates/engine/src/scenarios.rs"), "utf8");
  const max = Number(src.match(/MAX_COMBINATIONS:\s*usize\s*=\s*([\d_]+)/)[1].replace(/_/g, ""));
  assert.ok(max > 0);
  // The check finds the former contradiction («at most 400») and accepts the correct limit.
  assert.deepEqual(limitNumbers("Höchstens 400 Kombinationen sind erlaubt.").map(x => x.n), [400]); // check-english: allow (sample text)
  assert.deepEqual(limitNumbers("Erst bei mehr als **1'000'000** Kombinationen.").map(x => x.n), [1000000]); // check-english: allow (sample text)
  const files = walk(path.join(ROOT, "crates/app/ui/hilfe"), [".md"]);
  for (const lang of ["de", "en", "fr", "it"]) assert.ok(files.some(f => rel(f).includes(`/hilfe/${lang}/`)), lang);
  const found = [];
  const wrong = [];
  for (const f of files) {
    for (const { n, s } of limitNumbers(fs.readFileSync(f, "utf8"))) {
      found.push(rel(f));
      if (n !== max) wrong.push(`${rel(f)}: ${n} in «${s.slice(0, 120)}»`);
    }
  }
  assert.deepEqual(wrong, []);
  // Each language names the limit (10-editor.md), so the check does not pass by finding nothing.
  for (const lang of ["de", "en", "fr", "it"]) assert.ok(found.some(f => f.includes(`/hilfe/${lang}/`)), `limit in ${lang}`);
});
