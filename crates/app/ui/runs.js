// Liste der durchgeführten Simulationen. Gespeichert wird lokal im Browserspeicher der App (IndexedDB),
// getrennt in Kopfdaten (für die Liste) und Inhalt (Planung und Ergebnis). Es werden keine Daten gesendet.
"use strict";
const Runs = (() => {
  // legacy: RetirementPlanner – Name der Datenbank bleibt, damit die übernommenen Ergebnisse sichtbar bleiben (#44).
  const DB = "retirementplanner", META = "meta", DATA = "data";
  const mem = { meta: new Map(), data: new Map() };
  let dbp = null, memoryOnly = false;
  const open = () => dbp ||= new Promise((res, rej) => {
    try {
      if (memoryOnly) throw new Error("nur Arbeitsspeicher");
      const q = indexedDB.open(DB, 1);
      q.onupgradeneeded = () => { q.result.createObjectStore(META, { keyPath: "id" }); q.result.createObjectStore(DATA, { keyPath: "id" }); };
      q.onsuccess = () => res(q.result);
      q.onerror = () => rej(q.error);
    } catch (e) { rej(e); }
  });
  /** Ohne IndexedDB (z. B. gesperrter Speicher) bleibt die Liste bis zum Schliessen im Arbeitsspeicher. */
  async function run(stores, mode, fn, fallback) {
    let db;
    try { db = await open(); } catch { return fallback(); }
    return new Promise((res, rej) => {
      const t = db.transaction(stores, mode), out = fn(...stores.map(s => t.objectStore(s)));
      t.oncomplete = () => res(out.result);
      t.onerror = () => rej(t.error);
      // #147: Eine abgebrochene Transaktion (z. B. Speicherkontingent erschöpft) meldet nicht immer «error»; ohne
      // `onabort` bliebe das Versprechen offen und die Oberfläche wartete ohne Ende.
      t.onabort = () => rej(t.error || new Error("Speichern im Browserspeicher abgebrochen"));
    });
  }
  const req = (r, f) => { r.onsuccess = () => { f.result = r.result; }; return f; };
  return {
    /** Für die Oberflächentests (Chrome ohne Fenster schliesst IndexedDB-Abfragen nicht ab). */
    memoryOnly() { memoryOnly = true; dbp = null; },
    async add(meta, data) {
      await run([META, DATA], "readwrite", (m, d) => { m.put(meta); d.put({ id: meta.id, ...data }); return {}; },
        () => { mem.meta.set(meta.id, meta); mem.data.set(meta.id, { id: meta.id, ...data }); });
    },
    /** Kopfdaten, neueste zuerst. */
    async list() {
      const all = await run([META], "readonly", m => req(m.getAll(), {}), () => [...mem.meta.values()]);
      return (all || []).sort((a, b) => b.created.localeCompare(a.created));
    },
    async get(id) {
      return run([DATA], "readonly", d => req(d.get(id), {}), () => mem.data.get(id));
    },
    async update(id, fields) {
      await run([META], "readwrite", m => {
        const r = m.get(id);
        r.onsuccess = () => { if (r.result) m.put({ ...r.result, ...fields }); };
        return {};
      }, () => { const e = mem.meta.get(id); if (e) Object.assign(e, fields); });
    },
    async remove(ids) {
      await run([META, DATA], "readwrite", (m, d) => { for (const id of ids) { m.delete(id); d.delete(id); } return {}; },
        () => ids.forEach(id => { mem.meta.delete(id); mem.data.delete(id); }));
    }
  };
})();
// #147: für die Unit-Tests (node --test mit fake-indexeddb)
if (typeof module !== "undefined") module.exports = { Runs };
