Stufe 2 der Übersichtsseite «Testqualität» (FEA-30): die Schichten **Anforderungsbezug** und **Ergebnis** aus dem Bericht «Testqualität beurteilen 2026».

**Anforderung**

1. **Anforderungsbezug:** Die Seite zeigt je Issue die Abnahmekriterien und ob sie durch einen Test belegt sind (mit Test, nur manuell geprüft, ohne). Dazu werden Tests mit Issue und Kriterium gekennzeichnet (zum Beispiel Kommentar `FEA-100/AK2` oder Testname `fea_100_ak2_…`). Die Konvention steht in `CLAUDE.md` und `TESTS.md`.
2. **Ergebnis:** Die Seite listet durchgerutschte Fehler: Bug-Issues mit einem Label für den Fundort (zum Beispiel `gefunden:test`, `gefunden:abnahme`, `gefunden:nutzung`), je mit der Antwort auf «welcher Test hätte ihn fangen müssen?» und dem Regressionstest.
3. **Verlauf:** Die Seite vergleicht die Kennzahlen mit der vorherigen Version (Abdeckung je Paket, Zahl missed Mutanten, flaky Tests, Anteil Kriterien mit Test).
4. Das Lesen der Issues geschieht nur beim Erzeugen der Seite über `gh`, nicht in der App (die App bleibt ohne Netzwerk).

**Abnahme**

- [ ] Ein Issue mit drei Kriterien, davon zwei mit gekennzeichnetem Test, erscheint mit 2 von 3 belegt.
- [ ] Ein Bug-Issue mit Label `gefunden:nutzung` erscheint unter «Ergebnis» mit Ursache und Regressionstest.
- [ ] Der Vergleich zur vorherigen Version zeigt Veränderungen je Kennzahl.
- [ ] Ohne `gh`-Anmeldung erscheinen die beiden Schichten als «keine Daten», die übrige Seite bleibt vollständig.
- [ ] Labels und Kennzeichnung sind in `CLAUDE.md`, `TESTS.md` und den Issue-Vorlagen beschrieben.

- Abhängigkeit: FEA-30.

> Stufe 2 der Testqualitäts-Übersicht.
