✔ geklärt (Alternative A). Die Karte «3-Topf-Strategie, Herleitung der Topfgrössen und Auffüllung» erscheint in den Ergebnissen nicht mehr als eigene, immer sichtbare Tabelle. Die Angaben nach Abschnitt 10 der 3-Topf-Spezifikation bleiben abrufbar: In den Jahreswerten eines Einzellaufs hat jedes Jahr mit mindestens einer Auffüllung eine aufklappbare Zeile mit den Auffüllungen dieses Jahres und allen Angaben (Zeitpunkt, Auslöser, Ausgabenbasis, regelmässige Einnahmen, Netto-Bedarf, Ziele Topf 1 und 2, Renditekriterium, Umbuchungen, Topfstände vorher und nachher). Standardmässig sind alle Zeilen zugeklappt. Bei Läufen ohne 3-Topf-Strategie und in Jahren ohne Auffüllung gibt es nichts aufzuklappen. Bei mehreren Läufen (Monte Carlo) gilt die Herleitung für den gewählten Einzellauf, wie die Jahreswerte. Abschnitt 10 der 3-Topf-Spezifikation wird dahingehend geändert, dass der Anzeigeort die Jahreswerte sind (der Inhalt bleibt).

**Abnahme**

- [ ] Ein 3-Topf-Lauf hat keine Karte «3-Topf-Strategie, Herleitung …».
- [ ] In den Jahreswerten lässt sich zu einem Jahr mit Auffüllung die Herleitung aufklappen, mit allen Angaben nach Abschnitt 10.
- [ ] Jahre ohne Auffüllung haben keine Aufklapp-Zeile.
- [ ] Läufe ohne 3-Topf-Strategie zeigen keine Aufklapp-Zeilen.
- [ ] Der UI-Test zur Herleitung (`ui-tests/ui.spec.js:728`), die 3-Topf-Spezifikation (Abschnitt 10), die Hilfe und die Beschreibung sind nachgeführt.
- [ ] Beim Wechsel des gewählten Einzellaufs zeigt die Tabelle die Herleitung des neuen Laufs.
- Abhängigkeit: FEA-09 (3-Topf-Ereignisse aus der Ereignisliste) setzt diese Ersatzanzeige voraus.

**Umsetzungshinweis** (nicht Teil der Anforderung): `bucketTable` in `app/ui/app.js` liefert die Zeilen, sie lassen sich pro Jahr in die Jahrestabelle einhängen.

> «Ergebnisse:  3 Topf Strategie - 3-Topf-Strategie Herleitung der Topfgrössen und Auffüllung  braucht es nicht mehr als Eigenes Tabelle»

---
Umsetzung laut Git-Log: dd4b187. Abnahmekriterien vor dem Schliessen prüfen.

_Ursprüngliche ID: FEA-08_
