# Einstellungen, feste Meilensteine, Säule 3a und indirekte Amortisation

Issues: FEA-24 (#25), FEA-38 (#51), FEA-37 (#52), FEA-36 (#50). Diese Seite beschreibt die Umsetzung; die
Anforderungen und Abnahmekriterien stehen in den Issues.

## Einstellungen (FEA-24)

- Anwendungsweit, lokal gespeichert (`Pref`, Schlüssel `limits`; Desktop-App: Datei im Datenverzeichnis), nie in der
  Planungsdatei. Zugang: Menüeintrag «Einstellungen…» (CmdOrCtrl+,) und Schaltfläche «Einstellungen». FEA-44 (#59):
  Auf macOS steht der Eintrag im Programmmenü «VivaPlan» nach «Über VivaPlan» (mit Trennlinien),
  auf Windows und Linux am Ende von «Datei» vor «Beenden» (`SETTINGS`, `APP_MENU`, `file_entries` in `main.rs`).
- Werte und Standard: AHV-Bezug 63–70 Jahre, Bezug Pensionskasse 58–65 Jahre, Maximalbetrag Säule 3a 7 258 pro Jahr.
- Prüfung: ganze Jahre 0–130, untere ≤ obere Grenze, Maximalbetrag ≥ 0 (`Limits::problems` im Rechenkern, gleiche
  Regeln in der Oberfläche).
- Die Oberfläche gibt die Grenzen bei jedem Aufruf des Rechenkerns als `limits` in der Planung mit (`api.js`, ausser
  beim Speichern). Der Rechenkern liest sie aus `Plan::limits` (`crates/engine/src/settings.rs`); fehlen sie (Kommandozeile,
  Tests), gelten die Standardwerte. `limits` wird nie geschrieben (`skip_serializing`).
- Die Schwelle des Warnsymbols aus FEA-12 ist keine Einstellung (fest 5 %).

## Feste Meilensteine (FEA-38)

- Ids: erste Person `retire` («Bezug Pensionskasse») und `ahv` («AHV-Bezug»), zweite Person `retire2` und `ahv2`
  (Namen mit «Person 2»). Fehlen sie im Rechenkern, gelten sie mit 65 Jahren (`Resolver::new`).
- Prüfung: Alter des Meilensteins innerhalb der Grenzen, Fehler am Feld `milestones.N.years`.
- Warnung, wenn das Erwerbseinkommen im Monat des Bezugs der Pensionskasse oder danach grösser als 0 ist (erster
  Monat mit Einkommen, Betrag dieses Monats).
- Pensionskasse und AHV-Rente beginnen an den Meilensteinen der Person (in der Oberfläche nicht mehr wählbar).
- Laden älterer Dateien (`crates/engine/src/upgrade.rs`, Befehl `upgrade_plan`): fehlende Meilensteine mit 65 Jahren,
  Namen nachgeführt, Alter ausserhalb der Grenzen auf die nächste Grenze, Bezüge von PK und AHV auf die Meilensteine;
  jede Änderung als Hinweis.

## Säule 3a (FEA-37)

- Modell `pillar3a: [{ person, balance, rate, contribution, payouts: [{ kind?: date|age, date?, years?, amount?, use }], use }]`, höchstens ein
  Konto je Person. `amount` fehlt = ganzes Guthaben. `use` = `wealth` (Standard) oder `repay`.
- Monat: Zins, Einzahlung (Jahresbetrag / 12, bis vor dem AHV-Bezug), geplante Auszahlungen (Betrag höchstens
  Guthaben, sonst Hinweis), im Monat des AHV-Bezugs der Rest. Auszahlung = Kapitalbezug (Steuer mit PK-Bezügen des
  Jahres zusammengerechnet); `repay` zahlt zuerst die Tranchen mit indirekter Amortisation in dieses Konto zurück.
- Die Einzahlung zählt zu den Ausgaben: `MonthRecord::pillar3a_in`, in `spending` und `withdrawal`, im Netto-Bedarf
  (`NeedBasis::pillar3a`), in der Lebenshaltung der Ausgabenregeln wie die Wohnkosten.
- Prüfung: Einzahlung 0 bis Maximalbetrag (Fehler); Einzahlung + indirekte Amortisation über dem Maximalbetrag
  (Warnung); Auszahlung mehr als 5 Jahre vor dem AHV-Bezug (Fehler), nach dem AHV-Bezug (Warnung); `repay` ohne Tranche
  mit indirekter Amortisation (Warnung, die Auszahlung geht ins Vermögen).
- Alte Säule 3a an der Hypothek: wird ein Konto der ersten Person; die frühere Auszahlung wird ein Plan-Eintrag über
  das ganze Guthaben (frühestens 5 Jahre vor dem AHV-Bezug, nach dem AHV-Bezug: kein Eintrag); mehrere werden
  zusammengefasst; leere Konten ohne indirekte Amortisation entfallen; `repay` ohne indirekte Tranche wird `wealth`.

## Indirekte Amortisation (FEA-36)

- Am Quartalsende vor dem AHV-Bezug der Person der Tranche (`Tranche::pillar3a`, Standard erste Person): Einzahlung in
  deren 3a; ab dem AHV-Bezug: direkte Amortisation (Schuld sinkt um denselben Betrag).
- Ereignis `amortizationSwitch` im Monat des AHV-Bezugs, wenn die Tranche vorher indirekt amortisiert hat (Periode mit
  Amortisation vor dem AHV-Bezug) und im AHV-Monat weiter amortisiert wird; Text «Amortisation Hypothek {Immobilie}
  ab AHV-Bezug direkt statt über die Säule 3a: {Betrag} pro Jahr». Kein Ereignis, wenn die Simulation erst im
  AHV-Monat beginnt.
- Perioden, die ab dem AHV-Bezug beginnen, sind in der Oberfläche fest «direkt (ab AHV)».
- Da jede Person einen AHV-Bezug hat (FEA-38), gibt es den Fall «keine Person mit AHV-Bezug» nicht mehr.

## Golden-Dateien

Geändert (Begründung in den Issues): Ereignistext «Bezug Pensionskasse» statt «Pensionierung»; neue Spalte
`pillar3aIn`; Warnung zum Erwerbseinkommen nach dem PK-Bezug (02-ehepaar, 05-immobilie-hypothek); 3a-Auszahlung im
AHV-Bezug statt danach und ins Vermögen statt an die Hypothek, weil keine Tranche indirekt amortisiert
(05-immobilie-hypothek, 10-notverkauf).

## FEA-41 Auszahlungsplan mit Datum oder Alter

- Jede Zeile von `pillar3a[].payouts[]` hat `kind` («date», Vorgabe für ältere Planungen, oder «age») mit `date` bzw. `years` (Alter der Kontoinhaberin bzw. des Kontoinhabers). Monat bei Alter: Geburt + round(years × 12).
- Die Regel «frühestens 5 Jahre vor dem AHV-Bezug» gilt für beide Arten; die Meldung steht am Feld `date` bzw. `years`.
- Die Vorschau liefert je Konto das Guthaben jedes Monats, vor und nach jeder Auszahlung und den Rest im AHV-Bezug (`PreviewOut.pillar3a`); gespeicherte Ergebnisse enthalten das nicht.
- «+ Auszahlung»: ein Jahr nach der letzten Zeile, ohne Zeile 5 Jahre vor dem AHV-Bezug (nicht vor dem Start), höchstens im AHV-Bezug; Betrag = Guthaben laut Vorschau, abgerundet auf 1 000.
- Tests: `crates/engine/tests/pillar3a_plan.rs`, `e2e/ui.spec.js` (FEA-41).
