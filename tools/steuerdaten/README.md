# Steuerdatei für den Modus «Wohnort» (#63)

`build.py` erzeugt `crates/engine/data/steuern-<Jahr>.json` aus den Grunddaten des
[Steuerrechners der ESTV](https://swisstaxcalculator.estv.admin.ch/#/taxdata/tax-scales). Die App bettet die Datei ein,
es gibt keinen Netzwerkzugriff. Nur Standardbibliothek von Python, keine weiteren Pakete.

## Inhalt der Steuerdatei

- `income`: je Kanton der Tarif der Einkommenssteuer als Kurve `[steuerbares Einkommen, Satz]` der einfachen Steuer
  (Steuerfuss 100 %), getrennt für Alleinstehende (`single`) und Verheiratete (`married`, mit Splitting des Kantons);
  `municipalIncome`: Schwyz und Wallis zusätzlich der Tarif der Gemeindesteuer;
- `federal`: der Tarif der direkten Bundessteuer, ebenso auf das steuerbare Einkommen;
- `deductions` (je Kanton) und `federalDeductions` (Bund): die Standardabzüge vom Einkommen als Regeln
  `{id, kind, base, rate, min, max}` je Zivilstand und Variante (`retired`: Versicherungsprämien und Sparzinsen ohne
  Beiträge an die 2. und 3. Säule; `working`: mit Beiträgen), dazu `earned`, die Berufskostenpauschale;
- `wealth`: je Kanton die Kurve `[Vermögen, Satz]`, der Satz auf das Vermögen **vor** den Sozialabzügen (eingerechnet);
- `municipalities`: je Gemeinde BFS-Nummer, Name, Kanton und die Steuerfüsse als Anteile (0.95 = 95 %) für Kanton,
  Gemeinde, reformierte, römisch-katholische und christkatholische Kirche, je für Einkommen und Vermögen;
- `capital` (je Kanton, #64): Kurve `[Bezug, Satz]` der einfachen Steuer auf Kapitalbezüge aus Vorsorge (Steuerfuss
  100 %, auf den ganzen Bezug, ohne Abzüge), `single` und `married`; `capitalMunicipal`: eigene Kurve der
  Gemeindesteuer, wo die Regel abweicht; `capitalRule`: die Regel als Text; `capitalApprox`: Hinweis, wenn die Regel nur
  angenähert ist. `federalCapital`: ein Fünftel des Tarifs der direkten Bundessteuer;
- `propertyGain` (je Kanton, #65): der Regelsatz der Grundstückgewinnsteuer im Format von `PropertyGainTax` des
  Rechenkerns; `propertyGainRule`: die Regel als Text; `propertyGainApprox`: Hinweis, wenn die Regel nur angenähert
  ist; `gainMultiplier`: Median von Steuerfuss Kanton + Gemeinde (Einkommenssteuer) über die Gemeinden des Kantons,
  für Immobilien ausserhalb des Wohnkantons. `propertyGainSource`: Quelle der Regeln;
- `year`, `source`, `note`: Steuerjahr, Quelle und Hinweis.

Eine Regel ergibt den Abzug `base + rate × Bezugsgrösse`, mindestens `min`, höchstens `max` (0 = ohne Höchstbetrag).
Bezugsgrösse ist das Einkommen vor Abzügen, bei `earned` das Erwerbseinkommen (höchstens der Erwerb, ohne Erwerb 0).
`kind` ist `insurance` (Versicherungsprämien und Sparzinsen), `savingsInterest` (Sparzinsen, wo getrennt) oder `social`
(Sozialabzug); `id` ist die ID der ESTV.

**Auswahl der Abzüge** (ohne Kinder; je Gruppe gilt der erste vorhandene): `INCOME_DEDUCTIONS` in `build.py`.
Fehlt die gewünschte Variante, gilt die nächste vorhandene (Genf kennt nur «mit Beiträgen»). Berufskosten:
«Pauschalabzug übrige Berufskosten» (`HauptErw_EK`). Basel-Landschaft, Basel-Stadt und Tessin nennen nur einen
Höchstbetrag; er gilt als fester Abzug und trägt `approx: true`. Bei Format `STANDARDIZED` mit Prozent gilt der Betrag
als Mindestbetrag (AI 1000, AR und SG 700). Nicht enthalten: Kinder, einkommensabhängige Abzüge (bescheidene Einkommen,
Rentner, Entlastungsabzüge), Fahrkosten, Verpflegung, Zweitverdienerabzug.

Der Rechenkern (`crates/engine/src/taxdata.rs`) rechnet:
Einkommenssteuer = Kantonskurve(steuerbares Einkommen Kanton) × (Steuerfuss Kanton + Kirche)
+ Gemeindekurve × Steuerfuss Gemeinde + Bundeskurve(steuerbares Einkommen Bund);
Vermögenssteuer = Vermögenskurve × (Steuerfuss Kanton + Gemeinde + Kirche);
Kapitalbezugssteuer = Kapitalkurve × (Steuerfuss Kanton + Kirche) + Gemeindekurve × Steuerfuss Gemeinde + Bundeskurve,
mit den Steuerfüssen der Einkommenssteuer.

## Regeln für Kapitalbezüge (#64)

Die ESTV-Grunddaten enthalten nur für 12 Kantone einen Tarif «Kapitalauszahlung aus Vorsorge» (Steuerart
`VORSORGESTEUER` in `tarife_<Jahr>.csv`). Die Regeln aller 26 Kantone stehen deshalb von Hand erfasst in
`kapitalregeln.json`, abgeleitet aus 364 Rechnungen des ESTV-Steuerrechners 2026 «Kapitalauszahlung aus Vorsorge»
(alle Kantonshauptorte, Bezüge 50 000 bis 5 Millionen, alleinstehend und verheiratet, Alter 65, ohne Kinder, ohne
Konfession): Kantons- und Gemeindesteuer geteilt durch den Steuerfuss ergibt die einfache Steuer. Nicht aus
Steuermäppchen oder Gesetzen geprüft.

- `kind: table`: Tarif «Kapitalauszahlung aus Vorsorge» der ESTV (Stufen vom Typ `ZUERICH` mit Sockelbeträgen gelten
  als Typ `BUND`, Bern); `kind: income`: Anteil `share` der Steuer nach dem Einkommenstarif ohne Abzüge, Satz für
  `Bezug / divisor`, Verheiratete mit dem Splitting des Tarifs;
- `min`, `max`: Grenzen des Satzes; `rebate`: Ermässigung (`rate` des Bezugs, höchstens `max`);
- `single`, `married`: Werte je Zivilstand; `municipal`: eigene Regel der Gemeindesteuer (`target: GEMEINDE` nimmt
  den Gemeindetarif); `cantonMultiplier`: Kantonssteuer mit eigenem Steuerfuss (Freiburg 100 % statt 96 %);
- `text`: die Regel für die Steuerdatei (`capitalRule`); `approx`: Hinweis bei einer Näherung (`capitalApprox`).

Näherungen: Wallis (Höchstsatz der Gemeindesteuer aus Sitten abgeleitet) und Zug (Abweichung bis rund 7 %, genaue
Regel nicht gefunden). Alle anderen Kantone treffen die 364 Rechnungen auf 1 % genau. Die Oberfläche zeigt die Regel
aus den Texten `capitalRule.<Kanton>` in `crates/app/ui/i18n/*.json`; der deutsche Text muss `text` entsprechen
(e2e-Test). Für ein neues Steuerjahr die Rechnungen wiederholen und die Regeln und Referenzfälle
(`crates/engine/tests/tax_capital_location.rs`) nachführen.

## Regeln der Grundstückgewinnsteuer (#65)

Die Regelsätze aller 26 Kantone stehen von Hand erfasst in `grundstueckgewinn.py` (Funktion `rules()`). Quelle:
ESTV-Steuermäppchen «Grundstückgewinnsteuer», Steuerperiode 2025 (Stand 26.11.2025), Tabelle 3, Seite je Kanton;
ZG, FR, VS, BS, BL, SG und VD zusätzlich gegen das kantonale Gesetz geprüft (Stand November 2026). Je Kanton `text`
(Regel für `propertyGainRule`), `source` (Seite oder Gesetz, nur im Bericht) und, wo vereinfacht, `approx`.

- Modelle: `zurich` (Stufen `brackets`, optional `uniformAbove`, `longBrackets`), `aargau` (`durationRates`), `curve`
  (`rateCurve`) und `return` (`returnBounds`, `returnMaxDuration`);
- Ergänzungen: `duration`, `gainDuration`, `durationAbove`, `shortSurcharge`, `exemptBelow`, `allowance`, `maxShare`,
  `useMultiplier` (BE, LU, OW, SO, SH, JU);
- Hilfsfunktionen: `steps` (Zeilen je Jahr mit festem Schritt bis zu einem Endwert), `monthly_surcharge` (Zuschlag je
  fehlenden Monat).

Näherungen: Basel-Stadt (Selbstnutzung angenommen, ohne Ermässigung für wertvermehrende Aufwendungen),
Basel-Landschaft (Ermässigung bei 20 Jahren Selbstnutzung und Anpassung an die Teuerung fehlen), St. Gallen
(Selbstnutzung von 15 Jahren angenommen), Waadt (Selbstnutzung während der ganzen Besitzdauer), Wallis (Stufentarif
nach Gewinnhöhe angenommen). Die Regeln gelten nicht jährlich neu: Bei einer Gesetzesänderung Regel, Texte
`propertyGainRule.<Kanton>` in `crates/app/ui/i18n/*.json` (deutsch = `text`, e2e-Test) und Handwerte in
`crates/engine/tests/tax_gain_cantons.rs` nachführen.

## Ablauf für ein neues Steuerjahr

1. Im ESTV-Steuerrechner unter «Grunddaten abrufen» das Steuerjahr wählen und herunterladen:
   - **Steuerfüsse**, Gesamte Schweiz, Export als Excel, ablegen als `local/estv_income_rates.xlsx`;
   - **Tarife** Einkommen und Vermögen für alle 26 Kantone und den Bund;
   - **Abzüge** Einkommen und Vermögen, Gesamte Schweiz.
   Die Dateien gehören nach `local/estv/` (nicht im Repository).
2. Tarife und Abzüge als CSV (UTF-8) ablegen, Spalten:
   - `local/estv/tarife_<Jahr>.csv`: `Kanton,Hoheit,Steuerart,Tariftyp,Splitting,Gruppe,Zeile,Betrag,Prozent,Steuer,Formel`
     (Kanton `CH` mit Hoheit `BUND` für die Bundessteuer, Hoheit `GEMEINDE` für eigene Gemeindetarife; Steuerart
     `EINKOMMENSSTEUER` oder `VERMOEGENSSTEUER`; Tariftyp `ZUERICH`, `BUND`, `FLATTAX`, `FREIBURG` oder `FORMEL`, wie in
     der Tarifansicht der ESTV; Gruppe wie `ALLE`, `VERHEIRATET` oder `LEDIG_OHNE_KINDER`);
   - `local/estv/abzuege_<Jahr>.csv`: `Kanton,Hoheit,Steuerart,ID,Bezeichnung,Format,Betrag,Prozent,Minimum,Maximum`.
   Für 2026 wurden die Tarife und Abzüge der Ergebnistabelle der ESTV-Seite entnommen, weil der Excel-Export nur je
   Kanton und Steuerart möglich ist. Die undokumentierte Schnittstelle des Rechners nicht verwenden.
3. Erzeugen:
   ```bash
   python3 -I tools/steuerdaten/build.py --steuerfuesse local/estv_income_rates.xlsx \
     --tarife local/estv/tarife_2026.csv --abzuege local/estv/abzuege_2026.csv \
     --out crates/engine/data/steuern-2026.json --bericht local/estv/bericht.txt
   ```
   Die Regeln für Kapitalbezüge liest das Werkzeug aus `kapitalregeln.json` (anderer Ort: `--kapitalregeln`).
   Die Regeln der Grundstückgewinnsteuer kommen aus `grundstueckgewinn.py`.
   Der Bericht nennt je Kanton die gewählten Tarife, Abzüge, die Regel für Kapitalbezüge und die der
   Grundstückgewinnsteuer mit Quelle.
4. Den Dateinamen in `crates/engine/src/taxdata.rs` (`BUILTIN`) anpassen.
5. Referenzfälle prüfen: Im ESTV-Steuerrechner (Einkommens- und Vermögenssteuer, ohne Kinder, ohne Konfession) die Fälle
   aus `crates/engine/tests/tax_location.rs` für das neue Jahr rechnen und die Werte im Test ersetzen. Der Test verlangt
   höchstens ±5 % Abweichung.
6. Tests: `python3 -I -m unittest discover -s tools/steuerdaten` und `make test`.

## Rechtliches

Für die ESTV-Daten gibt es keine offene Lizenz (rechtliche Hinweise admin.ch). Die schriftliche Zustimmung der
Steuerstatistik der ESTV zur Weitergabe der Steuerdatei liegt vor (#63, 22.11.2026). Für ein neues Steuerjahr prüfen, ob
sie weiter gilt.
