In der Übersicht eines Monte-Carlo-Ergebnisses entfallen die Kennzahlkarte «Einmalzahlungen (Tragbarkeit)» und die Spalte «Einmalzahlungen» der Tabelle «Endvermögen nach Perzentil». Die Spalte «Notverkäufe» dieser Tabelle heisst «Immobilienverkäufe» und zeigt für den Lauf der jeweiligen Zeile jeden Verkauf einer Immobilie in einer eigenen Zeile im Format «Name: MM.JJJJ» (zum Beispiel «Haus: 05.2034», darunter «Wohnung: 03.2048»). Notverkäufe sind mit dem Warnsymbol (Haus mit Warnzeichen, wie in FEA-12) gekennzeichnet und im Tooltip als «Notverkauf» benannt; geplante Verkäufe tragen kein Symbol. Läufe ohne Verkauf zeigen «–». Die Kennzahlkarte «Notverkäufe» bleibt mit Namen und Inhalt erhalten (Anzahl und Anteil der Läufe mit Notverkauf, davon wegen fehlender Tragbarkeit). Planungen ohne Immobilien zeigen weder Spalte noch Kennzahl. Einzelläufe und Szenarien sind nicht betroffen.

**Abnahme**

- [ ] Die Übersicht hat keine Karte und keine Spalte «Einmalzahlungen».
- [ ] Die Spalte heisst «Immobilienverkäufe» und zeigt pro Verkauf eine Zeile «Haus: 05.2034».
- [ ] Ein Notverkauf trägt das Warnsymbol, ein geplanter Verkauf nicht.
- [ ] Ein Lauf mit zwei Verkäufen zeigt zwei Zeilen.
- [ ] Die Kennzahlkarte «Notverkäufe» zeigt die Zahlen wie bisher.
- [ ] Planungen ohne Immobilien zeigen die Spalte nicht.
- [ ] Zahlen und Daten stimmen mit den Immobilienereignissen des jeweiligen Perzentil-Laufs überein.
- Abhängigkeit: Symbol und Begriffe stimmen mit FEA-12 überein.

**Umsetzungshinweis** (nicht Teil der Anforderung): Daten aus den `propertyEvents` der Perzentil-Läufe (`details`); die Zählfelder bleiben in der Engine.

> «Ergebnisse: Übersicht: Einmalzahlung weglassen. Notverkäufe in Immobilienverkäufe umbennnen: Dort das Datum aller Imobilienverkäufr anzeigen: zB Haus: 05.2034 \n Wohnung 03.2048»

_Ursprüngliche ID: FEA-22_
