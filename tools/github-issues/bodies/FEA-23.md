Wurden in einer Simulation Variationen berechnet, kann der Benutzer in den Reitern «Einzelne Läufe» und «Jahreswerte» eine der berechneten Stufen auswählen (ein Auswahlfeld pro Variation, zum Beispiel «Aktienanteil 30 %», «Kapitalbezug 60 %», dazu «Planwert»). Die beiden Reiter zeigen dann die Läufe dieser Stufe, also dieselben Perzentil-Läufe (schlechtester, 1., 5., 50., 95., 99.), auf die sich die Tabelle im Reiter «Szenarien» stützt. Es wird keine neue Simulation über alle Pfade gerechnet: Die Läufe der Stufe werden aus den gespeicherten Pfadnummern (`Percentile.index`) nachgerechnet. Die Auswahl gilt gemeinsam für beide Reiter; ein Hinweis nennt die gewählte Stufe. Die übrigen Reiter bleiben bei den Planwerten und tragen den Vermerk «Planwerte, nicht die gewählte Variation». Das gilt auch für gespeicherte Ergebnisse. Weicht ein nachgerechneter Lauf vom gespeicherten Endvermögen ab (zum Beispiel weil sich eine Zeitreihe geändert hat), erscheint ein Hinweis, die Anzeige bleibt aber verfügbar. Auswählbar sind nur die sechs Perzentil-Läufe. Der Einzellauf mit Variationen verhält sich wie bisher.

**Abnahme**

- [ ] Ein Monte-Carlo-Ergebnis mit zwei Variationen zeigt in «Einzelne Läufe» und «Jahreswerte» zwei Auswahlfelder mit den berechneten Stufen.
- [ ] Die Auswahl «Aktienanteil 30 %, Kapitalbezug 60 %» zeigt den 50.-Perzentil-Lauf dieser Stufe; sein Endvermögen stimmt mit dem Wert in der Szenarientabelle überein.
- [ ] Der Wechsel dauert wenige Sekunden und löst keinen vollständigen Durchlauf aus.
- [ ] Die Auswahl wirkt in beiden Reitern; die anderen Reiter zeigen Planwerte mit Vermerk.
- [ ] Bei gespeicherten Ergebnissen funktioniert die Auswahl; bei abweichendem Endvermögen erscheint der Hinweis.
- [ ] Der Einzellauf mit Variationen funktioniert wie bisher.

**Umsetzungshinweis** (nicht Teil der Anforderung): Neue Engine-Funktion «Läufe zu Pfadnummern nachrechnen» (Plan, gewählte Stufe, Pfadnummern); die Oberfläche nutzt die bestehende Variationsauswahl (`variationPicker`).

> «Einzelne Läufe und Jahresweerte: zusätzlich die Variationen auswählen können. zB Aktienanteil 30%, Kapitalbezug: 60%»

_Ursprüngliche ID: FEA-23_
