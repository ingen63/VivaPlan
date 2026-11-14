«Neu» macht im Moment nichts. Es sollten alle Felder auf ihre Standardwerte zurückgesetzt oder leer sein.

**Anforderung:** «Neu» (Knopf und ⌘N) lädt eine leere Planung: Name «Neue Planung», eine Person ohne Namen, Meilensteine Start, Pensionierung, AHV und Ende, alle Beträge 0, keine Immobilien, keine Variationen, keine Optimierung, keine Datei. Bei ungespeicherten Änderungen kommt zuerst die Rückfrage.

**Abnahme**

- [ ] «Neu» setzt alle Felder auf Standardwerte oder leer.
- [ ] Mit ungespeicherten Änderungen erscheint die Rückfrage; «Abbrechen» ändert nichts.
- [ ] Der Dateiname im Kopf zeigt «Neue Planung», das Speichern fragt nach einem Pfad.
- [ ] Oberflächentest vorhanden.

**Stand:** umgesetzt in 2fe32ab (`blankPlan` in `app/ui/app.js`, Tests in `ui-tests/ui.spec.js`). Abnahme durch den Menschen offen.
