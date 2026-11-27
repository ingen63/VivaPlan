# Steuerdatei für den Modus «Wohnort» (#63)

`build.py` erzeugt `crates/engine/data/steuern-<Jahr>.json`. Tarife, Abzüge und Regeln des Bundes und der Kantone
kommen seit #286 aus den Gesetzen und Verordnungen (Quelldateien unter `quellen/<Jahr>/`, siehe unten). Aus den
Grunddaten des [Steuerrechners der ESTV](https://swisstaxcalculator.estv.admin.ch/#/taxdata/tax-scales) kommen nur noch
die Steuerfüsse (bis #285) und Tarife und Abzüge des Wallis (bis #296; `ESTV_ONLY` in `build.py`). Die App bettet die
Datei ein, es gibt keinen Netzwerkzugriff. Nur Standardbibliothek von Python, keine weiteren Pakete.

## Inhalt der Steuerdatei

- `income`: je Kanton der Tarif der Einkommenssteuer als Kurve `[steuerbares Einkommen, Satz]` der einfachen Steuer
  (Steuerfuss 100 %), getrennt für Alleinstehende (`single`) und Verheiratete (`married`, mit Splitting des Kantons);
  `municipalIncome`: Schwyz und Wallis zusätzlich der Tarif der Gemeindesteuer;
- `federal`: der Tarif der direkten Bundessteuer, ebenso auf das steuerbare Einkommen;
- `deductions` (je Kanton) und `federalDeductions` (Bund): die Standardabzüge vom Einkommen als Regeln
  `{id, kind, base, rate, min, max}` je Zivilstand und Variante (`retired`: Versicherungsprämien und Sparzinsen ohne
  Beiträge an die 2. und 3. Säule; `working`: mit Beiträgen), dazu `earned`, die Berufskostenpauschale, und
  `secondEarner`, der Zweitverdienerabzug für Ehepaare mit zwei Erwerbseinkommen (`reference`: `lower` = tieferes
  Erwerbseinkommen nach der Berufskostenpauschale, `total` = Summe der Erwerbseinkommen, nur Bern; höchstens das tiefere
  Erwerbseinkommen nach der Berufskostenpauschale; fehlt im Thurgau);
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
Rentner, Entlastungsabzüge), Fahrkosten, Verpflegung.

Der Rechenkern (`crates/engine/src/taxdata.rs`) rechnet:
Einkommenssteuer = Kantonskurve(steuerbares Einkommen Kanton) × (Steuerfuss Kanton + Kirche)
+ Gemeindekurve × Steuerfuss Gemeinde + Bundeskurve(steuerbares Einkommen Bund);
Vermögenssteuer = Vermögenskurve × (Steuerfuss Kanton + Gemeinde + Kirche);
Kapitalbezugssteuer = Kapitalkurve × (Steuerfuss Kanton + Kirche) + Gemeindekurve × Steuerfuss Gemeinde + Bundeskurve,
mit den Steuerfüssen der Einkommenssteuer.

## Regeln für Kapitalbezüge (#64)

Die Regeln aller 26 Kantone stehen von Hand erfasst in `kapitalregeln.json`. Sie wurden zuerst aus 364 Rechnungen des
ESTV-Steuerrechners 2026 «Kapitalauszahlung aus Vorsorge» abgeleitet (#64; alle Kantonshauptorte, Bezüge 50 000 bis
5 Millionen, alleinstehend und verheiratet, Alter 65, ohne Kinder, ohne Konfession: Kantons- und Gemeindesteuer
geteilt durch den Steuerfuss ergibt die einfache Steuer) und danach Kanton für Kanton gegen das Gesetz geprüft
(#287–#291); jede Regel nennt den Artikel in `source`. Graubünden (Art. 40a: Satz für einen Fünfzehntel des Bezugs)
und Zug (§ 37 Abs. 2) sind mit dem kantonalen Steuerrechner geprüft. Bei Genf und Tessin enthalten die Regeln
zusätzlich Faktoren aus den ESTV-Rechnungen; das steht in `source`.

- `kind: table`: Tarif «Kapitalauszahlung aus Vorsorge» der ESTV (Stufen vom Typ `ZUERICH` mit Sockelbeträgen gelten
  als Typ `BUND`, Bern); `kind: income`: Anteil `share` der Steuer nach dem Einkommenstarif ohne Abzüge, Satz für
  `Bezug / divisor`, Verheiratete mit dem Splitting des Tarifs; mit `parts` (Zug, #288) statt dessen je Teil des
  Bezugs bis `upTo` die Tarifsteuer auf diesem Teil mal `share`;
- `min`, `max`: Grenzen des Satzes; `rebate`: Ermässigung (`rate` des Bezugs, höchstens `max`);
- `single`, `married`: Werte je Zivilstand; `municipal`: eigene Regel der Gemeindesteuer (`target: GEMEINDE` nimmt
  den Gemeindetarif); `cantonMultiplier`: Kantonssteuer mit eigenem Steuerfuss (Freiburg 100 % statt 96 %);
- `text`: die Regel für die Steuerdatei (`capitalRule`); `approx`: Hinweis bei einer Näherung (`capitalApprox`).

Näherung: Wallis (Höchstsatz der Gemeindesteuer aus Sitten abgeleitet). Zug folgt seit #288 dem Gesetz (§ 37 Abs. 2
StG: 30 % der Tarifsteuer auf den ersten 230 600 Franken, 40 % auf dem Rest, mindestens 1 %; Grenzbetrag 2026 der
Steuerverwaltung) und trifft den Zuger Steuerrechner auf den Rappen. Alle anderen Kantone treffen die 364 Rechnungen auf
1 % genau. Die Oberfläche zeigt die Regel
aus den Texten `capitalRule.<Kanton>` in `crates/app/ui/i18n/*.json`; der deutsche Text muss `text` entsprechen
(e2e-Test). Für ein neues Steuerjahr die Rechnungen wiederholen und die Regeln und Referenzfälle
(`crates/engine/tests/tax_capital_location.rs`) nachführen.

## Regeln der Grundstückgewinnsteuer (#65)

Die Regelsätze aller 26 Kantone stehen von Hand erfasst in `grundstueckgewinn.py` (Funktion `rules()`). Sie wurden
zuerst nach dem ESTV-Steuermäppchen «Grundstückgewinnsteuer» 2025 erfasst (#65) und danach Kanton für Kanton gegen das
Gesetz geprüft (#287–#291). Je Kanton `text` (Regel für `propertyGainRule`), `source` (Gesetz mit Artikel, nur im
Bericht) und, wo vereinfacht, `approx`. Funde dabei: Solothurn § 57 kürzt den Gewinn, nicht die Steuer
(`gainDuration`); Luzern hat seit 2025 einen festen Steuerfuss von 4,2 Einheiten (§ 23 GGStG), in die Sätze
eingerechnet (ohne `useMultiplier`).

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

## Quelldateien aus den Erlassen (#286, #287)

Tarife und Abzüge kommen Kanton für Kanton aus den Gesetzen und Verordnungen statt aus den ESTV-Tabellen. Je Kanton
(und `CH` für den Bund) liegen unter `quellen/<Jahr>/` zwei Dateien:

- `<KT>_tarife.csv`: Spalten wie `tarife_<Jahr>.csv` (siehe unten), dazu `Quelle`;
- `<KT>_abzuege.csv`: Spalten wie `abzuege_<Jahr>.csv`, dazu `Quelle`; nur die Abzüge, die `build.py` verwendet
  (`INCOME_DEDUCTIONS`, `WEALTH_DEDUCTIONS`, `HauptErw_EK`, `ZweitVerdiener_EK`); die IDs sind die Schlüssel von
  `build.py`, die Bezeichnungen eigene Texte.

Jede Zeile nennt in `Quelle` den Erlass mit Artikel und Stand (z. B. `StG ZH § 35 Abs. 1 (LS 631.1; …)`). Ein Kanton
braucht beide Dateien; seine Zeilen aus den ESTV-Tabellen fallen dann ganz weg, die übrigen Kantone kommen unverändert
aus den ESTV-Tabellen. Der Bericht beginnt mit der Herkunft je Kanton (`laws (source files)` oder `ESTV tables`).
Kapitalbezugs- und Grundstückgewinnregeln nennen den Gesetzesartikel in `source` (`kapitalregeln.json`,
`grundstueckgewinn.py`). Stand 2026: Bund und Zürich (#287), Zentralschweiz LU, UR, SZ, OW, NW, ZG (#288), Nordwestschweiz BE, SO, BS, BL, AG (#289), Ostschweiz GL, SH, AR, AI, SG, GR, TG (#290), Westschweiz und Tessin FR, VD, NE, GE, JU,
TI (#291; Wallis mit eigenem Issue, weil die Gemeinden eigene Indexstufen haben). Wo das
Gesetz Grundbeträge nennt und die Verwaltung die Teuerung ausgleicht (Zug), nennt `Quelle` auch die Veröffentlichung
der Steuerverwaltung für das Jahr (z. B. Aargau, Basel-Landschaft) oder die Teuerungsverordnung (Solothurn).

Die Erlasse selbst (PDF und Textauszug) liegen lokal unter `local/steuerquellen/<Jahr>/` mit `quellen.txt` (Datei,
Adresse, Abrufdatum), nicht im Repository.

**Abgleich:** `vergleich.py` vergleicht zwei Steuerdateien je Kanton und Bund (Kurven, Abzüge, Regeln; ohne
Steuerfüsse) und meldet Sätze, die mehr als 1 % abweichen:

```bash
python3 -I tools/steuerdaten/vergleich.py crates/engine/data/steuern-2026.json neu.json --kanton CH --kanton ZH
```

Jede gemeldete Abweichung wird im Teil-Issue erklärt (Erfassungsfehler oder echte Abweichung).

## Ablauf für ein neues Steuerjahr

1. **Erlasse beschaffen:** Für den Bund und jeden Kanton die Fassung des Steuergesetzes, der Verordnungen und der
   Veröffentlichungen zum Teuerungsausgleich für das neue Jahr herunterladen (Fedlex, kantonale Gesetzessammlung,
   Steuerverwaltung). Ablage unter `local/steuerquellen/<Jahr>/`, jede Datei mit Datum und Adresse in `quellen.txt`
   (nicht im Repository). Zugangssperren nicht umgehen.
2. **Quelldateien nachführen:** `quellen/<Vorjahr>/` nach `quellen/<Jahr>/` kopieren und je Zeile Betrag, Satz und
   `Quelle` (Erlass, Artikel, Stand) nach den neuen Erlassen anpassen. Viele Kantone passen Tarifstufen und Abzüge
   jährlich an die Teuerung an; massgebend ist dann die Veröffentlichung der Steuerverwaltung oder die
   Teuerungsverordnung (Beispiele: SO Steuerverordnung Nr. 20, GE RCEPF, JU Arrêté, GR Indexstufe nach Art. 4,
   AR Anhang 1, ZG und VD Tarifblätter). `kapitalregeln.json` und `grundstueckgewinn.py` ebenso prüfen.
3. **Steuerfüsse** (bis #285): Im ESTV-Steuerrechner unter «Grunddaten abrufen» die Steuerfüsse (Gesamte Schweiz,
   Excel) als `local/estv_income_rates.xlsx` ablegen. Für das Wallis (bis #296) auch Tarife und Abzüge als CSV nach
   `local/estv/` (Spalten wie die Quelldateien ohne `Quelle`); die Zeilen der übrigen Kantone werden ignoriert.
   Die undokumentierte Schnittstelle des Rechners nicht verwenden.
4. **Erzeugen:**
   ```bash
   python3 -I tools/steuerdaten/build.py --steuerfuesse local/estv_income_rates.xlsx \
     --tarife local/estv/tarife_<Jahr>.csv --abzuege local/estv/abzuege_<Jahr>.csv \
     --out crates/engine/data/steuern-<Jahr>.json --bericht local/estv/bericht.txt
   ```
   Das Werkzeug bricht ab, wenn ausser dem Wallis ein Kanton oder der Bund keine Quelldatei hat. Der Bericht nennt je
   Kanton die Herkunft, die gewählten Tarife und Abzüge und die Regeln mit Gesetzesartikel.
5. **Abgleichen:** `vergleich.py` gegen die Datei des Vorjahres; jede Abweichung über 1 % mit der Teuerung oder einer
   Gesetzesänderung erklären.
6. Den Dateinamen in `crates/engine/src/taxdata.rs` (`BUILTIN`) anpassen.
7. **Referenzfälle:** Die Fälle aus den kantonalen Steuerrechnern (ZH, ZG, AG, GR) und aus den amtlichen Tarifblättern
   (Tests `issue_287_…` bis `issue_291_…`) für das neue Jahr neu rechnen. Die älteren Fälle aus dem ESTV-Steuerrechner
   in `tax_location.rs` und `tax_capital_location.rs` bleiben als zweite Prüfung (#292); Toleranz ±5 %.
8. Tests: `python3 -I -m unittest discover -s tools/steuerdaten` und `make test`.

## Rechtliches

Für die ESTV-Daten gibt es keine offene Lizenz (rechtliche Hinweise admin.ch). Die schriftliche Zustimmung der
Steuerstatistik der ESTV zur Weitergabe der Steuerdatei liegt vor (#63, 22.11.2026). Für ein neues Steuerjahr prüfen, ob
sie weiter gilt.
