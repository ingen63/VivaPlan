Gefunden beim Test von FEA-02 (#2), Abnahmekriterium «Beenden mit ungespeicherten Änderungen: Dialog mit Speichern, Verwerfen, Abbrechen; Abbrechen lässt die App offen».

**Fehler:** Die App lässt sich mit ungespeicherten Änderungen schliessen, ohne dass ein Dialog erscheint. Das gilt sowohl für den Schliessen-Knopf des Fensters als auch für ⌘Q.

**Erwartet:** Bei ungespeicherten Änderungen erscheint ein Dialog mit «Speichern», «Verwerfen» und «Abbrechen». «Abbrechen» lässt die App offen, «Speichern» speichert und beendet, «Verwerfen» beendet ohne Speichern. Ohne ungespeicherte Änderungen schliesst die App sofort.

**Schritte**

1. Planung öffnen und eine Eingabe ändern (der Kopf zeigt ungespeicherte Änderungen).
2. Fenster über den Schliessen-Knopf schliessen, danach wiederholen mit ⌘Q.

**Abnahme**

- [ ] Schliessen-Knopf mit Änderungen: Dialog erscheint, «Abbrechen» lässt die App offen.
- [ ] ⌘Q mit Änderungen: dasselbe Verhalten.
- [ ] «Speichern» speichert (bei neuer Planung mit Ortsabfrage) und beendet; bricht die Ortsabfrage ab, bleibt die App offen.
- [ ] «Verwerfen» beendet ohne Speichern.
- [ ] Ohne Änderungen schliesst die App ohne Dialog.
- [ ] Test für beide Wege (Fenster schliessen, ⌘Q).
