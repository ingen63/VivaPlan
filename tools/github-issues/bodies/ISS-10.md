Sammlung von Befunden aus der Abnahme der umgesetzten Funktionen. Jeder Punkt ist ein eigenes Abnahmekriterium. Punkt 8 wird bereits in FEA-18 (#19) geführt und steht hier nur zur Vollständigkeit. Entscheide und die eine offene Frage stehen unten.

**Anzeige der Zeitreihe**

1. Ergebnisse, Abschnitt Parameter, Renditequelle, Zeile «Zeitreihe»: Ist-Zustand `library:Shiller CHF + CH-Obligationen`, Soll `Shiller CHF + CH-Obligationen` (ohne das interne Präfix `library:`).

**Ergebnisse, Tabelle Jahreswerte**

2. Der Titel der Tabelle ist zentriert und fett.
3. Die Spalten «Einkommen» und «Ausgaben geplant» entfallen. «Entnahme» bleibt (Korrektur, siehe Punkt 11).
4. Die Spaltengruppen «Pro Monat (Durchschnitt des Jahres)» und «Vermögen am Jahresende» entfallen (die Überschriften der Gruppen).
5. Von der Entnahmerate bleibt nur «Total», beschriftet «Entnahmerate». «Liquid» entfällt.

**Meilensteine (FEA-14)**

6. Beim Wechsel von «Datum» zu «Alter von» wird das Alter aus der Spalte «Ergibt» in das Feld «Alter» eingetragen.
7. Beim Wechsel von «Alter von» zu «Datum» wird das Datum aus der Spalte «Ergibt» in das Feld «Datum» eingetragen.

**Weitere Funktionen**

8. FEA-18: Die Änderung der Abnahme (Info-Symbol je Zeitreihe, Texte höchstens 2 Zeilen, «Gewählt:» entfällt) wird umgesetzt. Die Anforderung steht in #19.
9. FEA-19, Textänderung: Die Zielbezeichnung «2 · Immobilie behalten» heisst «2 · Kein Notverkauf von Immobilien». Gilt für Auswahl, Ergebniskarte, Hilfe (`40-optimierung.md`) und Kommandozeilenhilfe.
10. FEA-20: Übersteigt die geschätzte Rechenzeit 90 s, erscheint in der Karte «Umfang» neben der Zahl der Simulationsläufe ein gelbes Warnsymbol.

**Entschieden**

- Zu 3 und 5: Die Spalten «Einkommen» und «Ausgaben geplant» entfallen (zu «Entnahme» siehe Punkt 11). Von der Entnahmerate bleibt nur «Total», beschriftet «Entnahmerate».
- Zu 10: Das Warnsymbol erscheint ab 90 s geschätzter Rechenzeit. Die Rückfrage vor dem Start (FEA-20) bleibt bei 1 Minute.

**Befunde nach der Umsetzung (Commit 293deba)**

11. Jahreswerte, Spaltenreihenfolge: Die Spalte «Entnahme» wird wieder eingeblendet, und zwar direkt nach «Ausgaben total». Sie war im Punkt 3 als zu entfernen gelesen worden; das war ein Missverständnis. Die Spalte «Entnahmerate» steht direkt nach «Einkommen total». Reihenfolge der Monatsspalten damit: Ausgaben total, Entnahme, Einkommen total, Entnahmerate, Abweichung vom Plan, Inflation pro Jahr.
12. Jahreswerte: Der Knopf zum Ein- und Ausblenden der Steuerspalten («Steuern anzeigen» bzw. «Steuern ausblenden») ist in der Oberfläche nicht mehr zu sehen. Im Code ist er noch vorhanden (`data-action="tax-cols"` in `yearTable`), die Ursache ist nicht geklärt. Er muss wieder sichtbar und bedienbar sein.

**Offene Frage**

- Zu 4: Fallen die Gruppenüberschriften weg, ist aus der Tabelle nicht mehr ersichtlich, dass die Beträge Monatsdurchschnitte sind und die Vermögen Stände am Jahresende. Vorschlag: den Hinweis in den Spaltenkopf (Tooltip) und in den Text unter der Tabelle übernehmen.

**Abnahme**

- [ ] Spalte «Entnahme» steht nach «Ausgaben total», «Entnahmerate» nach «Einkommen total».
- [ ] Der Knopf «Steuern anzeigen/ausblenden» ist sichtbar und blendet die fünf Steuerspalten ein und aus.
- [ ] Punkte 1 bis 12 sind umgesetzt oder durch Rückfrage geklärt.
- [ ] Hilfe (`app/ui/hilfe/`) und Spezifikation sind nachgeführt, wo sich Texte oder Spalten ändern.
- [ ] Ein Oberflächentest je Änderung der Tabelle, der Meilensteine und der Warnung.

> Befunde des Autors aus der Abnahme (zehn Punkte).
