Enthält eine Planung mindestens eine Immobilie mit Status «Besitz» oder «Kauf geplant», müssen die Regeln der Grundstückgewinnsteuer gesetzt sein (Vorlage Zürich, Vorlage Aargau oder eigene Regeln, auch mit Satz 0 %). Fehlen sie, ist das ein Fehler im Abschnitt Steuern; «Simulation starten» und «Optimieren» sind blockiert (wie bei FEA-13). Reine Mietobjekte lösen die Pflicht nicht aus. Die leere Planung von «Neu» enthält keine Immobilie, daher keine Pflicht.

Bisher ist die Grundstückgewinnsteuer optional: Ohne Regeln bleibt der Verkauf steuerfrei (`engine/src/validate.rs`).

**Abnahme**

- [ ] Immobilie «Besitz» oder «Kauf geplant» ohne Grundstückgewinnsteuer: Fehler im Abschnitt Steuern mit Meldung, Start blockiert (Engine und Oberfläche).
- [ ] Nur Mietobjekte oder keine Immobilie: kein Fehler.
- [ ] Mit Vorlage oder eigenen Regeln (auch Satz 0 %): kein Fehler.
- [ ] Die Pflichtliste (FEA-13) in der Spezifikation und die Hilfe nennen die neue Pflicht.
- [ ] Test in der Engine und in der Oberfläche.
- Abhängigkeit: ergänzt FEA-13 (Pflichteingaben). Beispiel- und Testplanungen mit Immobilien brauchen dann Regeln.

> «Check falls in einer Planung eine Immobilie existiert, muss zwingend die Grundstückgewinnsteuer gesetzt sein» (zuerst bei ISS-3 erfasst, auf Wunsch als eigene Funktion geführt)
