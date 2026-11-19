// #147: Unit tests of the results list (crates/app/ui/runs.js) against real IndexedDB behavior (fake-indexeddb, only a
// test dependency), without a browser. The user interface tests use the list only in memory (test access «mem»).
require("./tr.js"); // #170: texts from the text table
const { test } = require("node:test");
const assert = require("node:assert/strict");
const { IDBFactory, IDBTransaction: FDBTransaction } = require("fake-indexeddb");

const RUNS = require.resolve("../../crates/app/ui/runs.js");
/** Fresh database and fresh module (runs.js remembers the opened database). */
function fresh(idb = new IDBFactory()) {
  globalThis.indexedDB = idb;
  delete require.cache[RUNS];
  return require(RUNS).Runs;
}
const meta = (id, created, extra = {}) => ({ id, created, name: `Lauf ${id}`, note: "", ...extra });

test("#147/AK1: add, list (newest first), get, update and remove; header data and content separate", async () => {
  const Runs = fresh();
  await Runs.add(meta("a", "2026-11-01T10:00:00Z"), { plan: { p: 1 }, result: { r: 1 } });
  await Runs.add(meta("b", "2026-11-02T10:00:00Z"), { plan: { p: 2 }, result: { r: 2 } });
  const list = await Runs.list();
  assert.deepEqual(list.map(m => m.id), ["b", "a"]);
  assert.equal(list[0].plan, undefined, "die Liste enthält nur Kopfdaten");
  assert.deepEqual(await Runs.get("a"), { id: "a", plan: { p: 1 }, result: { r: 1 } });
  await Runs.update("a", { name: "Neu", note: "Notiz" });
  const a = (await Runs.list()).find(m => m.id === "a");
  assert.equal(a.name, "Neu"); assert.equal(a.note, "Notiz"); assert.equal(a.created, "2026-11-01T10:00:00Z");
  await Runs.update("gibt-es-nicht", { name: "x" });
  assert.equal((await Runs.list()).length, 2);
  await Runs.remove(["a"]);
  assert.deepEqual((await Runs.list()).map(m => m.id), ["b"]);
  assert.equal(await Runs.get("a"), undefined);
  // the same database is kept after reloading the module (as after a restart of the app)
  const idb = globalThis.indexedDB;
  const again = fresh(idb);
  assert.deepEqual((await again.list()).map(m => m.id), ["b"]);
});

test("#147/AK2: an aborted transaction (e.g. quota exhausted) rejects the promise", async () => {
  const Runs = fresh();
  await Runs.list(); // create the database
  const orig = FDBTransaction.prototype.objectStore;
  // Simulation: the browser aborts the transaction (event «abort») without a request failing.
  FDBTransaction.prototype.objectStore = function (...a) {
    const t = this;
    queueMicrotask(() => t.onabort?.({ type: "abort" }));
    return orig.apply(this, a);
  };
  try {
    const outcome = await Promise.race([
      Runs.add(meta("q", "2026-11-03T10:00:00Z"), { plan: {}, result: {} }).then(() => "erfüllt", e => `abgelehnt: ${e.message}`),
      new Promise(r => setTimeout(() => r("hängt"), 1000)),
    ]);
    assert.match(outcome, /^abgelehnt: /);
  } finally {
    FDBTransaction.prototype.objectStore = orig;
  }
});

test("#147/AK3: without IndexedDB the list stays in memory", async () => {
  const Runs = fresh(undefined);
  await Runs.add(meta("m", "2026-11-04T10:00:00Z"), { plan: { p: 3 }, result: {} });
  assert.deepEqual((await Runs.list()).map(m => m.id), ["m"]);
  assert.deepEqual((await Runs.get("m")).plan, { p: 3 });
  await Runs.update("m", { note: "x" });
  assert.equal((await Runs.list())[0].note, "x");
  await Runs.remove(["m"]);
  assert.equal((await Runs.list()).length, 0);
});

test("#147/AK4: a result of an older version (without later fields) can be read", async () => {
  const Runs = fresh();
  // before #126: percentiles without endTotalReal, range without real
  const old = { plan: { format: "pensionsplaner/1" }, result: { batch: { summary: { percentiles: [{ percent: 50, endTotal: 1 }] }, bands: { p50: [1] } } } };
  await Runs.add(meta("alt", "2025-01-01T10:00:00Z", { version: "1.0.0-beta.1" }), old);
  assert.deepEqual((await Runs.get("alt")).result, old.result);
  assert.equal((await Runs.list())[0].version, "1.0.0-beta.1");
});
