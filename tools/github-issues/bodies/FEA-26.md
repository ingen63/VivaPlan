Die Eingabemaske der Immobilien ist unübersichtlich: Bei «Besitz» stehen Kaufpreis, Marktwert, Kaufdatum, Verkauf und Nebenkosten, bei «Kauf geplant» Kauf ab, Kaufpreis und Nebenkosten. Die Felder haben je Status andere Plätze.

**Anforderung**

1. **Kopfzeile:** Name · Marktwert per Simulationsstart (nur bei «Besitz», bei «Kauf geplant» ausgeblendet) · Status · ✕.
2. **Raster, bei beiden Status gleiche Plätze:** Kaufzeitpunkt («Kaufdatum» bzw. «Kauf ab») · Kaufpreis (bei «Besitz» beschriftet «Kaufpreis (bezahlt)») · «Verkauf ab» (Vorgabe «Nie», siehe ISS-5) · Nebenkosten pro Monat.
3. **Danach:** Inflationsoptionen und Hypothek, unverändert. Die Miete bleibt unverändert.
4. **Schmales Fenster:** Der Marktwert rutscht unter den Namen.

**Abnahme**

- [ ] Reihenfolge und Plätze der Felder sind bei «Besitz» und «Kauf geplant» gleich.
- [ ] Der Marktwert steht bei «Besitz» in der Kopfzeile neben dem Namen und fehlt bei «Kauf geplant».
- [ ] Beim Statuswechsel bleiben Name, Kaufpreis, Verkauf und Nebenkosten an ihrem Platz.
- [ ] Alle Felder bleiben erreichbar, erfasste Werte sind unverändert.
- [ ] Schmales Fenster: Marktwert unter dem Namen.
- [ ] Oberflächentest prüft die Reihenfolge.
- [ ] Hilfe (`app/ui/hilfe/`) ist nachgeführt.
- Abhängigkeit: ISS-5 zuerst (Feld «Verkauf ab» bei «Kauf geplant»).

**Umsetzungshinweis** (nicht Teil der Anforderung): `R.realestate` in `app/ui/app.js`.

> «Das UI für Immobilien ist unübersichtlich. Die Reihenfolge der Felder für Besitz und Kauf geplant ist unterschiedlich. Schiebe das Feld Marktwert neben den Namen und gleiche die Felder dann an.»
