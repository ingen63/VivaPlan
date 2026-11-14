In der Liste «Gespeicherte Ergebnisse» und in der Vergleichsansicht ersetzt die neue Spalte **Zeitreihe** die Spalte **Art**. Sie zeigt den Namen der Zeitreihe, mit der das Ergebnis berechnet wurde (zum Beispiel «Shiller CHF + CH-Obligationen», ohne Ordner und Dateiendung). Hat die Renditequelle keine Zeitreihe, steht die Quelle in Grau: «feste Renditen» bzw. «parametrisch». Der Name wird beim Speichern des Ergebnisses in den Kopfdaten abgelegt; bei bereits gespeicherten Ergebnissen ohne diese Angabe steht «–» (kein Nachladen). Die Renditequelle (historisch, Bootstrap, parametrisch) wird nicht zusätzlich angezeigt. Die Angaben der bisherigen Spalte «Art» (Einzellauf, Szenarien mit Variantenzahl, Anzahl Läufe) bleiben als zweite, kleine Zeile in der Spalte «Ergebnis» erhalten.

**Abnahme**

- [ ] Liste und Vergleich haben keine Spalte «Art», aber eine Spalte «Zeitreihe».
- [ ] Ein historisches oder Bootstrap-Ergebnis zeigt den Reihennamen ohne Pfad und «.csv».
- [ ] Feste und parametrische Ergebnisse zeigen «feste Renditen» bzw. «parametrisch» in Grau.
- [ ] Ein Ergebnis zeigt auch nach Änderung der Zeitreihe in der aktuellen Planung weiterhin die Zeitreihe von damals.
- [ ] Einzellauf, Szenarien und Anzahl Läufe sind weiterhin erkennbar (zweite Zeile in «Ergebnis»).
- [ ] Alte Einträge zeigen «–» ohne Fehler.

**Umsetzungshinweis** (nicht Teil der Anforderung): `runMeta` um den Reihennamen erweitern; Anzeige in `openRuns` und `compareRuns` (`app/ui/app.js`).

> «Gespeicherte Ergebnisse: neue Spalte : Zeitreihe, Zeigt die ausgewählte Zeitreihe, dafür die Spalte Art wegnehmen.»

---
Umsetzung laut Git-Log: dd4b187. Abnahmekriterien vor dem Schliessen prüfen.

_Ursprüngliche ID: FEA-11_
