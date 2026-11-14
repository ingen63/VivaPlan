Die Zahl der Kombinationen aus den Variationen hat keine feste Obergrenze mehr (bisher 1'000). Vor dem Start schätzt die App die Rechenzeit mit der vorhandenen Messung (`scenSeconds`). Liegt die erwartete Rechenzeit über 1 Minute, erscheint vor dem Start eine Warnung mit geschätzter Dauer und Zahl der Läufe, mit den Schaltflächen «Trotzdem starten» und «Abbrechen». Darunter startet die Berechnung wie bisher ohne Rückfrage. Die Schwelle (1 Minute) ist fest; sie kann später in die Einstellungen wandern (FEA-24). Als Speicherschutz lehnt die Engine mehr als 1 Million Kombinationen mit der Meldung «Die Variationen ergeben N Kombinationen; höchstens 1'000'000 sind technisch möglich.» ab. Die Darstellung sehr vieler Kombinationen in der Ergebnistabelle ist nicht Teil dieser Anforderung (Hinweis für später).

**Abnahme**

- [ ] Mit 1'001 Kombinationen (bisher abgelehnt) wird die Berechnung angeboten.
- [ ] Bei geschätzten 90 s erscheint die Warnung mit Dauer und Läufen; «Abbrechen» startet nichts, «Trotzdem starten» startet.
- [ ] Bei geschätzten 30 s keine Warnung.
- [ ] Die Zusammenfassung der Variationen nennt keine Grenze mehr ausser der technischen.
- [ ] 1'000'001 Kombinationen werden mit der technischen Meldung abgelehnt, ohne die App zu belasten.
- [ ] Abbrechen bleibt während der Berechnung möglich.

**Umsetzungshinweis** (nicht Teil der Anforderung): Die Kombinationen nur zählen, nicht vorab anlegen, wenn sie nur geprüft werden.

> «Variationen : Maximum nicht begrenzen, nur eine Warnung sobald die erwartete Rechenzeit über 1min liegt»

_Ursprüngliche ID: FEA-20_
