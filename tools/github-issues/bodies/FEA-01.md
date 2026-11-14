Auf der Zeitleiste ist jedes Immobilienereignis (Kauf, Verkauf, Notverkauf) ohne Maus-Hover erkennbar. Es trägt eine Kurzbeschriftung aus Art, Objekt und Datum (zum Beispiel «Kauf Wohnung 09.2027»). Ereignisse sind von Meilensteinen unterscheidbar, auch ohne Farbe (Form oder Position). Überlappende Beschriftungen werden gestaffelt. Bei mehr als 15 Ereignissen erscheinen nur die Symbole, die Beschriftung kommt dann per Hover. Ohne Simulationsergebnis gibt es keine Ereignisse. Einmalbeträge, 3-Topf-Buchungen und «Geld aufgebraucht» gehören nicht dazu und bleiben unverändert.

**Abnahme**

- [ ] Mit `examples/Test 1.rplan` hat jedes Kauf-, Verkaufs- und Notverkaufsereignis eine Beschriftung.
- [ ] Keine zwei Beschriftungen überdecken sich.
- [ ] Ereignisse und Meilensteine sind ohne Farbunterscheidung auseinanderzuhalten.
- [ ] Bei mehr als 15 Ereignissen erscheinen nur Symbole, Text beim Hover.
- [ ] Ohne Ergebnis keine Ereignisse.

**Umsetzungshinweis** (nicht Teil der Anforderung): Grüne Ereignisfarbe, für Notverkäufe die Warnfarbe; Beschriftung unterhalb der Linie; die Staffelung (`lanes` in `renderTimeline`) wiederverwenden.

> «Zeitleiste Ereignisse grün beschriften»

---
Umsetzung laut Git-Log: 6d9a720. Abnahmekriterien vor dem Schliessen prüfen.

_Ursprüngliche ID: FEA-01_
