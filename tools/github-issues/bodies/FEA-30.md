Die Ergebnisse der Tests sind heute verstreut: Terminalausgabe (nextest), HTML der Abdeckung (cargo-llvm-cov), `mutants.out/`, Fuzz-Ausgaben und GitHub-Artefakte. Es fehlt eine Übersichtsseite, die die Testqualität verständlich darstellt. Grundlage: Bericht «Testqualität beurteilen 2026» (Profil aus vier Schichten, keine Gesamtnote, Einzelhinweise statt Score, Fokus auf Änderungen). Dieses Issue ist **Stufe 1** (Hygiene und Angemessenheit); Anforderungsbezug und Ergebnis folgen in FEA-31.

**Anforderung**

1. Ein Befehl (zum Beispiel `make report`) erzeugt lokal eine **statische HTML-Seite** «Testqualität» aus den vorhandenen Ergebnissen. Die Seite funktioniert ohne Netzwerk (keine externen Skripte oder Schriften).
2. **Gliederung nach Schichten, keine Gesamtnote:**
   - **Hygiene:** Tests bestanden, fehlgeschlagen, übersprungen, als flaky markiert (nextest JUnit); Laufzeit gesamt und die zehn langsamsten Tests; Playwright-Ergebnis, wenn vorhanden.
   - **Angemessenheit:** Zeilenabdeckung je Paket (Engine, App) gegen die Untergrenze 80 %; Abdeckung des geänderten Codes, wenn vorhanden (FEA-28); **Liste der nicht erkannten Mutanten** mit Datei, Zeile und Mutation (aus `mutants.out/`), dazu Zahlen caught, missed, unviable, timeout; Stand Property-Tests (Fallzahl) und Fuzzing (Ziele, letzter Lauf, Abstürze).
3. Jede Schicht hat einen **Status (erfüllt, Hinweis, nicht erfüllt)** nach einer festen, auf der Seite genannten Regel, zum Beispiel: Hygiene erfüllt = keine Fehlschläge, keine flaky Tests; Angemessenheit erfüllt = beide Pakete ≥ 80 %, keine neuen missed Mutanten. Ein Status wird nicht über die Schichten gemittelt.
4. Fehlende Daten (zum Beispiel kein Mutationslauf vorhanden) erscheinen als «keine Daten» mit Datum des letzten Laufs, nicht als Fehler.
5. Details (Mutantenliste, langsame Tests) sind aufklappbar; die bestehenden Einzelberichte (Abdeckung als HTML, Playwright-Bericht) sind verlinkt.
6. Die Seite nennt Datum, Commit und Version, auf die sich die Daten beziehen.
7. **GitHub:** `test-quality.yml` erzeugt dieselbe Seite als Artefakt und schreibt eine Kurzfassung (Status je Schicht, Abdeckung, Zahl der missed Mutanten) in die Lauf-Zusammenfassung.
8. Die erzeugte Seite wird nicht eingecheckt.

**Abnahme**

- [ ] `make report` erzeugt die Seite nach einem Lauf von `make test` und `make coverage`; sie öffnet sich ohne Netzwerk.
- [ ] Mit einem absichtlich fehlschlagenden Test zeigt Hygiene «nicht erfüllt» und nennt den Test.
- [ ] Mit einem vorhandenen `mutants.out/` stehen die missed Mutanten einzeln mit Datei und Zeile auf der Seite.
- [ ] Ohne Mutationslauf zeigt der Bereich «keine Daten» statt eines Fehlers.
- [ ] Es gibt keine Gesamtnote; jede Schicht hat ihren eigenen Status mit genannter Regel.
- [ ] Ein Lauf von `test-quality.yml` enthält die Seite als Artefakt und die Kurzfassung in der Zusammenfassung.
- [ ] `TESTS.md` beschreibt die Seite, den Befehl und die Statusregeln.

**Umsetzungshinweis** (nicht Teil der Anforderung): kleines Skript (Rust-Binary unter `tools/` oder Shell mit `jq`), das `junit.xml`, `cargo llvm-cov report --json`, `mutants.out/outcomes.json` und Playwright-JSON liest; Playwright-Reporter um `json` und `html` ergänzen.

> «Was noch fehlt ist ein Report oder eine HTML-Seite, die die Ergebnisse übersichtlich und verständlich darstellt.»
