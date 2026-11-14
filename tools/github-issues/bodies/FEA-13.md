

1. **Liste:** Die Pflichteingaben stehen in der Spezifikation (Datei unter `specification/`, Abschnitt «Pflichteingaben»), pro Eingabe mit Abschnitt, Bedingung und Fehlermeldung. Grundlage ist der Prüfbericht (Abschnitt «FEA-13»).
2. **Markierung:** Pflichtfelder sind im Editor mit einem Sternchen gekennzeichnet. Fehlt ein Wert oder ist er ungültig, steht die Meldung direkt am Feld; die Abschnittsliste zeigt die Zahl der Fehler.
3. **Blockade:** «Simulation starten» und «Optimieren» sind blockiert, solange Fehler vorliegen. Der Versuch zeigt «Die Planung hat N Fehler» mit Verweis auf den ersten betroffenen Abschnitt. Warnungen blockieren nicht.
4. **Neue Prüfungen:** Der Name der Person ist Pflicht (nicht leer, höchstens 60 Zeichen). Ein Kauf- oder Verkaufsdatum vor dem Simulationsstart ist eine Warnung im Abschnitt Immobilien und wird nicht stillschweigend ignoriert (löst den zweiten Teil von ISS-2).

**Abnahme**

- [ ] Leerer Name: Fehler am Feld, Start blockiert.
- [ ] Leeres oder ungültiges Geburtsdatum: Fehler am Feld, Start blockiert.
- [ ] Jede Zeile der Pflichtliste hat einen Test, der den Fehler auslöst und die Meldung prüft.
- [ ] Sind alle Pflichtfelder gültig, startet die Simulation wie bisher.
- [ ] Beim Blockieren führt der Verweis in den ersten Abschnitt mit Fehler.
- [ ] Verkaufsdatum vor dem Simulationsstart: Warnung im Abschnitt Immobilien.
- [ ] Die Liste in der Spezifikation stimmt mit den Prüfungen in der Engine überein.

**Umsetzungshinweis** (nicht Teil der Anforderung): `engine/src/validate.rs` um Name und Datum-vor-Start ergänzen; Oberfläche: Sternchen und Feldmeldung, Prüfung vor dem Start.

> «Prüfe auf mandatory Eingaben im UI. zB Geburt, Name, etc. Mach ein Liste, was zwingend für eine Simulation nötig ist.»

_Ursprüngliche ID: FEA-13_
