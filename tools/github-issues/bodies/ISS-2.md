Beispielplannung: Haus wird verkauft 08.2026 , Wohnung aber erst Jahre später gekauft, obwohl definiert ist Kauf ab: Verakuf Haus. Problem liegt wahrscheinlich an Tranchen

- Entscheid: (1) Ein Kauf- oder Verkaufsdatum vor dem Simulationsstart ist eine **Warnung** (deckt sich mit FEA-13). (2) Ist der Kauf nach einem Verkauf nicht finanzierbar, **entfällt er sofort, und die Folgeregeln greifen** (hier Mietbeginn); es gibt keinen wiederholten Kaufversuch. Das Ereignis «Kauf … nicht möglich» nennt Eigenmittel nötig und vorhanden.
- Befund: Ursache war die Tragbarkeit (Einkommen null, daher keine Hypothek, der ganze Kaufpreis wäre bar zu bezahlen), nicht die Tranchen. Der Kauf wurde einmal versucht und danach nie wiederholt, die Folgeregel (Miete) griff nicht sauber.
- Test: jeweils auf `issues/Steuer Bug.rplan`.
- ISS-3 Neu maccht im MOment nichts, es sollten alle Felder auf ihre Default Werte zurückgesetzt werden oder leer sein
- ISS-4 Neu.rplan Haus Verkauf lääst sich nicht löscehn obwohl es in der Config keine Referenz gint. Esmelder aber 2

---
Umgesetzt in e0939c1.

_Ursprüngliche ID: ISS-2_
