# FEA-43 Erwerbseinkommen optional mit der Teuerung

Issue: FEA-43 (#58). Diese Seite beschreibt die Umsetzung; Anforderung und Abnahmekriterien stehen im Issue.

## Modell

- `flows.incomeIndexed` (erste Person) und `partner.incomeIndexed` (zweite Person), Wahrheitswert. Fehlt das Feld,
  gilt «nein»; «nein» wird nicht geschrieben (`skip_serializing_if`), ältere Dateien bleiben gleich.
- Neue Planungen (`crates/engine/assets/Neu.vplan`, «Neu» in der Oberfläche) und eine neu hinzugefügte zweite Person setzen «ja»
  (Entscheid im Issue). Der Import alter `config.json` setzt «nein».

## Rechenkern

- `SimSetup::earned(m, infl_idx)`: Einkommen der Person mal Preisindex des Pfades seit dem Start, wenn
  `income_indexed`; sonst nominal. Der Preisindex ist derselbe wie für die Lebenshaltung (`Run::infl_idx`, monatlich
  mit der Teuerung des Vormonats fortgeschrieben), also in allen Renditequellen der Index des jeweiligen Pfades.
- Alle Verwendungen nutzen den angepassten Betrag: Monatsrechnung, Einkommenssteuer, Bedarf der 3-Topf-Strategie
  (`plan_basis`), Ausgabenregeln, Tragbarkeit bei Kauf und Erneuerung, Aufteilung der Bestände ab der Pensionierung.
- Prüfung (FEA-38): Die Warnung «Erwerbseinkommen nach dem Bezug der Pensionskasse» kennt keinen Pfad und nennt bei
  «ja» den Betrag mit dem Zusatz «in Preisen des Simulationsstarts».

## Oberfläche

- Abschnitt «Einnahmen & Ausgaben»: unter dem Erwerbseinkommen jeder Person das Kästchen «Steigt mit der Teuerung
  (Beträge in heutigen Preisen)». Kästchen wirken sofort (auch mit ISS-13).

## Tests

- `crates/engine/tests/income_indexed.rs`: AK1–AK5 (Beispiel 8 000 bei 2 %, Zeilen eines zeitabhängigen Einkommens, beliebiger
  Pfad mit wechselnder Teuerung, je Person, Steuer und Warnung, Schreiben der Datei).
- `e2e/ui.spec.js`, Block «Erwerbseinkommen mit der Teuerung (FEA-43)».
- Golden-Dateien unverändert (Vorgabe «nein»). Tests, die auf `Neu.vplan` aufbauen und nominales Einkommen
  voraussetzen (`crates/engine/tests/sim_rules.rs`), setzen `incomeIndexed` ausdrücklich auf `false`.
