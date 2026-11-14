Die Tabelle der Meilensteine hat keine Spalte «Verwendet». Ein verwendeter Meilenstein kann gelöscht werden: Der Klick auf «✕» öffnet einen Bestätigungsdialog («‹Name› wird in N Eingaben verwendet (Abschnitte: …). Beim Löschen werden diese Eingaben auf das feste Datum MM.JJJJ umgestellt.», Schaltflächen «Löschen und umstellen» und «Abbrechen»; genannt werden Anzahl und Abschnittsnamen, keine Einzelliste). Nach Bestätigung wird in jeder Verwendung die Meilenstein-ID durch das Datum (Monat/Jahr) des Meilensteins ersetzt, bei Meilensteinen vom Typ «Alter von» durch das berechnete Datum. Der Meilenstein wird gelöscht, eine Meldung nennt die Anzahl («N Eingaben auf MM.JJJJ umgestellt»), und der Vorgang lässt sich mit Rückgängig umkehren. Nicht verwendete Meilensteine werden ohne Dialog gelöscht. Nicht löschbar sind «Simulationsstart», «Simulationsende» und «Pensionierung» (Optimierung und die Variation der Pensionierung benötigen sie); die Schaltfläche fehlt oder ist ausgegraut mit Tooltip. Entsteht durch die Umstellung in einer Zeitreihe ein doppelter Zeitpunkt, gilt der Wert der späteren Zeile, und eine Meldung weist darauf hin.

**Abnahme**

- [ ] Keine Spalte «Verwendet».
- [ ] Meilenstein «AHV» löschen: Dialog nennt Verwendung und Datum; danach zeigt die AHV-Rente das feste Datum, die Rechnung liefert dieselben Zahlen wie vorher.
- [ ] Bei «Alter von» wird das berechnete Datum eingetragen.
- [ ] Rückgängig stellt Meilenstein und Verwendungen wieder her.
- [ ] «Pensionierung», «Simulationsstart» und «Simulationsende» lassen sich nicht löschen.
- [ ] Nicht verwendete Meilensteine lassen sich ohne Dialog löschen.
- [ ] Doppelter Zeitpunkt in einer Reihe: genau eine Zeile bleibt, mit Hinweis.
- [ ] Variation und Optimierung laufen nach dem Löschen eines anderen Meilensteins unverändert.

**Umsetzungshinweis** (nicht Teil der Anforderung): Verwendungen mit demselben Durchlauf wie `countRefs` sammeln und dem Abschnitt zuordnen; die Schaltfläche «✕» für `retire` sperren.

> «Meilensteine: Verwendet weglassen»

_Ursprüngliche ID: FEA-14_
