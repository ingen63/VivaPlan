// List of the simulations run. Stored locally in the app's browser storage (IndexedDB),
// separated into header data (for the list) and content (plan and result). No data is sent.
"use strict";
const Runs = (() => {
  // legacy: RetirementPlanner – the name of the database stays so that the results taken over stay visible (#44).
  const DB = "retirementplanner", META = "meta", DATA = "data";
  const mem = { meta: new Map(), data: new Map() };
  let dbp = null, memoryOnly = false;
  /** #209: Counts the changes of the content (add, remove); the comparison keeps what it read until the next change. */
  let version = 0;
  const open = () => dbp ||= new Promise((res, rej) => {
    try {
      if (memoryOnly) throw new Error("nur Arbeitsspeicher"); // i18n: allow (internal, never shown)
      const q = indexedDB.open(DB, 1);
      q.onupgradeneeded = () => { q.result.createObjectStore(META, { keyPath: "id" }); q.result.createObjectStore(DATA, { keyPath: "id" }); };
      q.onsuccess = () => res(q.result);
      q.onerror = () => rej(q.error);
    } catch (e) { rej(e); }
  });
  /** Without IndexedDB (e.g. blocked storage) the list stays in memory until closing. */
  async function run(stores, mode, fn, fallback) {
    let db;
    try { db = await open(); } catch { return fallback(); }
    return new Promise((res, rej) => {
      const t = db.transaction(stores, mode), out = fn(...stores.map(s => t.objectStore(s)));
      t.oncomplete = () => res(out.result);
      t.onerror = () => rej(t.error);
      // #147: An aborted transaction (e.g. storage quota exhausted) does not always report «error»; without
      // `onabort` the promise would stay pending and the user interface would wait forever.
      t.onabort = () => rej(t.error || new Error(tr("runs.storageAborted")));
    });
  }
  const req = (r, f) => { r.onsuccess = () => { f.result = r.result; }; return f; };
  return {
    /** For the user interface tests (headless Chrome does not complete IndexedDB queries). */
    memoryOnly() { memoryOnly = true; dbp = null; },
    get version() { return version; },
    async add(meta, data) {
      version++;
      await run([META, DATA], "readwrite", (m, d) => { m.put(meta); d.put({ id: meta.id, ...data }); return {}; },
        () => { mem.meta.set(meta.id, meta); mem.data.set(meta.id, { id: meta.id, ...data }); });
    },
    /** Header data, newest first. */
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
      version++;
      await run([META, DATA], "readwrite", (m, d) => { for (const id of ids) { m.delete(id); d.delete(id); } return {}; },
        () => ids.forEach(id => { mem.meta.delete(id); mem.data.delete(id); }));
    }
  };
})();
// #147: for the unit tests (node --test with fake-indexeddb)
if (typeof module !== "undefined") module.exports = { Runs };
