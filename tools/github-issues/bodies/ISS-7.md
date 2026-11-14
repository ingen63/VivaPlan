Ergebnisse, Karte «Variation wählen»: Bei der Variation «Kapitalbezug Pensionskasse» steht «Optimum: Kapitalbezug Pensionskasse ≥ 100 %». Das «≥» sagt sonst, dass der Bereich der Variation nicht reicht. Bei einem Anteil gibt es aber nicht mehr als 100 %; 100 % ist hier das echte Maximum, kein ausgeschöpfter Bereich. Die Anzeige führt in die Irre.

**Erwartet:** Liegt der beste Wert eines Anteils (Kapitalbezug Pensionskasse, Aktienanteil) auf seiner natürlichen Grenze (0 % oder 100 %), steht der Wert ohne «≥» bzw. «≤», zum Beispiel «100 %». Parameter ohne natürliche Grenze (zum Beispiel der Ausgaben-Faktor) behalten «≥» und «≤». Die Erklärung unter der Tabelle nennt den Unterschied.

**Ursache (Befund):** `Optimum` (`app/ui/optimum.js`) kennzeichnet jeden besten Wert am Rand des untersuchten Bereichs als `edgeHigh` bzw. `edgeLow`, unabhängig davon, ob der Rand zugleich die natürliche Grenze des Parameters ist.

**Abnahme**

- [ ] Kapitalbezug Pensionskasse, bester Wert 100 %: Anzeige «100 %», nicht «≥ 100 %».
- [ ] Bester Wert 0 %: Anzeige «0 %», nicht «≤ 0 %».
- [ ] Anteil mit Bereich kleiner als 0 bis 100 % (zum Beispiel 0 bis 50 %), bester Wert 50 %: weiterhin «≥ 50 %».
- [ ] Ausgaben-Faktor am oberen Rand: weiterhin «≥».
- [ ] Die Erklärung unter der Tabelle ist angepasst.
- [ ] Test für `Optimum` und die Anzeige.

> «Warum steht bei Ergebnisse → Tabelle Jahreswerte Kapitalbezug Pensionskasse ≥ 100 %»
