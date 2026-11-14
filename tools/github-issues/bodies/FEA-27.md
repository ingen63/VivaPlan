Der `rust-test-agent` schreibt heute Tests, bis 80 % Zeilenabdeckung je Paket erreicht sind. Studien von 2025 und 2026 zeigen, dass automatisch geschriebene Tests hohe Abdeckung bei schwachen Prüfungen erreichen (MUTGEN: 100 % Abdeckung, 4 % Mutation Score; SAP HANA: 25 %). Abdeckung zeigt, was ausgeführt wird, nicht was geprüft wird. Grundlage: Bericht `reports/Testqualität beurteilen 2026.md`, Empfehlung 1.

**Anforderung**

1. Nach dem Schreiben neuer Tests lässt der Agent `cargo mutants --in-diff` auf die geänderten Dateien der betrachteten Commits laufen (Vollmodus vor dem Push; im Schnellmodus nach dem Commit nur, wenn es unter etwa 2 Minuten bleibt).
2. Ziel des Agenten ist **«keine neuen nicht erkannten (missed) Mutanten im Rechenkern»**, nicht mehr nur «80 % erreicht». Die 80 % bleiben Untergrenze.
3. Der Agent schreibt keine Tests ohne Prüfung (`assert!`, `assert_eq!` o. Ä.) und keine reinen «läuft ohne Panik»-Tests, ausser der Test prüft ausdrücklich, dass kein Absturz auftritt.
4. Der Bericht des Agenten nennt die verbleibenden missed Mutanten mit Datei und Zeile.

**Abnahme**

- [ ] Die Anweisung `.claude/agents/rust-test-agent.md` enthält den Mutationslauf, das neue Ziel und das Verbot von Tests ohne Prüfung.
- [ ] Ein Beispiellauf mit einer absichtlich schwach getesteten Änderung im Rechenkern meldet einen missed Mutanten, und der Agent ergänzt einen Test, der ihn tötet.
- [ ] Der Bericht des Agenten listet verbleibende missed Mutanten.
- [ ] Die Laufzeit des pre-push-Hooks ist gemessen und im Issue vermerkt.
- [ ] `TESTS.md` beschreibt das neue Vorgehen.

**Umsetzungshinweis** (nicht Teil der Anforderung): `--in-diff` mit `git diff` über die zu pushenden Commits; nextest führt keine Doctests aus (falsche missed, siehe Empfehlung 4 im Bericht).

> Aus dem Bericht «Testqualität beurteilen 2026», Empfehlung 1.
