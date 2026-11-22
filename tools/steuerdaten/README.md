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
Vermögenssteuer = Vermögenskurve × (Steuerfuss Kanton + Gemeinde + Kirche).

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
   Der Bericht nennt je Kanton die gewählten Tarife und Abzüge.
4. Den Dateinamen in `crates/engine/src/taxdata.rs` (`BUILTIN`) anpassen.
5. Referenzfälle prüfen: Im ESTV-Steuerrechner (Einkommens- und Vermögenssteuer, ohne Kinder, ohne Konfession) die Fälle
   aus `crates/engine/tests/tax_location.rs` für das neue Jahr rechnen und die Werte im Test ersetzen. Der Test verlangt
   höchstens ±5 % Abweichung.
6. Tests: `python3 -I -m unittest discover -s tools/steuerdaten` und `make test`.

## Rechtliches

Für die ESTV-Daten gibt es keine offene Lizenz (rechtliche Hinweise admin.ch). Die schriftliche Zustimmung der
Steuerstatistik der ESTV zur Weitergabe der Steuerdatei liegt vor (#63, 22.11.2026). Für ein neues Steuerjahr prüfen, ob
sie weiter gilt.
