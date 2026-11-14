Die Mindestabdeckung von 80 % Zeilen je Paket (Engine, App) bleibt, wird aber nicht erhöht: Für keine Schwelle gibt es Evidenz, 80 % entspricht OpenSSF Best Practices Silber. Wirksamer ist der Blick auf den geänderten Code, wie Google ihn im Code-Review nutzt. Grundlage: Bericht `reports/Testqualität beurteilen 2026.md`, Empfehlung 2.

**Anforderung**

1. Die 80 % je Paket bleiben unverändert Untergrenze (Hook, `coverage.yml`, `test-quality.yml`).
2. Zusätzlich wird bei jedem Pull Request die **Abdeckung des geänderten Codes** ausgewiesen: Anteil der geänderten bzw. neuen Zeilen, die von Tests ausgeführt werden, mit Liste der nicht abgedeckten Zeilen.
3. Die Angabe erscheint in der Zusammenfassung des Workflow-Laufs auf GitHub. Sie ist ein Hinweis, keine Sperre.
4. Lokal gibt es einen Befehl (zum Beispiel `make coverage-diff`), der dasselbe zeigt.

**Abnahme**

- [ ] Ein Pull Request mit neuem, ungetestetem Code zeigt in der Lauf-Zusammenfassung einen Diff-Abdeckungswert unter 100 % und die nicht abgedeckten Zeilen.
- [ ] Ein Pull Request ohne Rust-Änderungen zeigt «keine Rust-Änderungen».
- [ ] Die Diff-Abdeckung lässt den Lauf nicht fehlschlagen.
- [ ] Der lokale Befehl liefert dasselbe Ergebnis.
- [ ] `TESTS.md` und `CLAUDE.md` nennen die Diff-Abdeckung und dass die 80 % eine Untergrenze sind.

**Umsetzungshinweis** (nicht Teil der Anforderung): `cargo llvm-cov --lcov` plus `git diff` (zum Beispiel diff-cover oder ein kleines Skript).

> Aus dem Bericht «Testqualität beurteilen 2026», Empfehlung 2.
