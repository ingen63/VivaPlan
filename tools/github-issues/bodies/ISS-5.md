Bei einer Immobilie mit Status «Kauf geplant» lässt sich kein Verkaufszeitpunkt eingeben. Das Feld «Verkauf» erscheint nur bei «Besitz» (`app/ui/app.js`, Abschnitt Immobilien). Die Engine kennt `sell` für geplante Objekte bereits und verkauft zum Zeitpunkt (`engine/src/sim.rs`), nur die Oberfläche fehlt.

**Anforderung**

1. **Eingabe:** Bei «Kauf geplant» gibt es das Feld «Verkauf ab» wie bei «Besitz». Vorgabe ist «Nie». Wählbar sind wie bei «Besitz» «Nie», ein Datum oder ein Meilenstein.
2. **Prüfung:** Der Verkaufszeitpunkt muss nach dem Kaufzeitpunkt liegen (bei «Besitz» nach dem Kaufdatum, bei «Kauf geplant» nach «Kauf ab»). Liegt er auf oder vor dem Kauf, ist das ein Fehler am Feld «Verkauf» mit Meldung. Ist der Kauf von einem Verkauf abhängig («nach Verkauf von …») und damit beim Erfassen nicht bestimmbar, entfällt die Prüfung.
3. **Wechsel des Status:** Wird von «Kauf geplant» auf «Besitz» oder «Miete» gewechselt, bleibt der erfasste Verkauf bei «Besitz» erhalten, bei «Miete» wird er nicht verwendet.

**Abnahme**

- [ ] Bei «Kauf geplant» erscheint das Feld «Verkauf ab» mit Vorgabe «Nie».
- [ ] Datum und Meilenstein sind wählbar; die Simulation verkauft das Objekt zum gewählten Zeitpunkt (Ereignis «Verkauf»).
- [ ] Verkauf vor oder im Monat des Kaufs: Fehler am Feld, Start blockiert (Engine und Oberfläche).
- [ ] Gleiche Prüfung bei «Besitz» gegenüber dem Kaufdatum.
- [ ] Bei Kauf «nach Verkauf von …» keine Fehlermeldung zum Datum.
- [ ] Testfall in der Engine für die Prüfung, Oberflächentest für das Feld.
- [ ] Hilfe (`app/ui/hilfe/`) und Spezifikation sind nachgeführt.

**Umsetzungshinweis** (nicht Teil der Anforderung): Feld in `R.realestate` auch bei `planned` anzeigen; Prüfung in `engine/src/validate.rs` neben der Warnung «Verkaufszeitpunkt vor dem Simulationsstart».

> «Immobilien: Falls "Kauf geplant" ausgewählt lässt sich kein Verkauf ab eingeben. Default: Nie, Check Datum immer nach Kauf ab.»
