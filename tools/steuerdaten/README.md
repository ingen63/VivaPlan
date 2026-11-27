# Steuerdatei für den Modus «Wohnort» (#63)

`build.py` erzeugt `crates/engine/data/steuern-<Jahr>.json`. Tarife, Abzüge und Regeln des Bundes und der Kantone
kommen seit #286 aus den Gesetzen und Verordnungen (Quelldateien unter `quellen/<Jahr>/`, siehe unten). Aus den
Grunddaten des [Steuerrechners der ESTV](https://swisstaxcalculator.estv.admin.ch/#/taxdata/tax-scales) kommen nur noch
die Steuerfüsse (bis #285); seit #296 auch das Wallis aus den Gesetzen (`ESTV_ONLY` in `build.py` ist leer). Die App bettet die
Datei ein, es gibt keinen Netzwerkzugriff. Nur Standardbibliothek von Python, keine weiteren Pakete.

## Inhalt der Steuerdatei

- `income`: je Kanton der Tarif der Einkommenssteuer als Kurve `[steuerbares Einkommen, Satz]` der einfachen Steuer
  (Steuerfuss 100 %), getrennt für Alleinstehende (`single`) und Verheiratete (`married`, mit Splitting des Kantons);
  `municipalIncome`: Schwyz und Wallis zusätzlich der Tarif der Gemeindesteuer; `municipalIndex` (Wallis, #296):
  je Gemeinde `[BFS-Nummer, Faktor]`, um den die Engine die Kurven der Gemeindesteuer (Einkommen und Kapitalbezug)
  streckt (Satz bei `x × Faktor` = Satz der Grundkurve bei `x`);
- `federal`: der Tarif der direkten Bundessteuer, ebenso auf das steuerbare Einkommen;
- `deductions` (je Kanton) und `federalDeductions` (Bund): die Standardabzüge vom Einkommen als Regeln
  `{id, kind, base, rate, min, max}` je Zivilstand und Variante (`retired`: Versicherungsprämien und Sparzinsen ohne
  Beiträge an die 2. und 3. Säule; `working`: mit Beiträgen), dazu `earned`, die Berufskostenpauschale, und
  `secondEarner`, der Zweitverdienerabzug für Ehepaare mit zwei Erwerbseinkommen (`reference`: `lower` = tieferes
  Erwerbseinkommen nach der Berufskostenpauschale, `total` = Summe der Erwerbseinkommen, nur Bern; höchstens das tiefere
  Erwerbseinkommen nach der Berufskostenpauschale; fehlt im Thurgau);
- `incomeRebate` (Wallis, #297): Ermässigung der Einkommenssteuer für Verheiratete, je für Kanton (`canton`) und
  Gemeinde (`municipal`) als `{rate, min, max, source}`: die Steuer nach dem Steuerfuss (ohne Kirchensteuer) sinkt um
  `rate` davon, mindestens `min`, höchstens `max` und nie unter null (LF VS Art. 32 Abs. 3 lit. a und Art. 178 Abs. 3
  lit. a: 35 %, 2026 indexiert 680 bis 4 900 Franken). Quelle: Zeilen `Ermaessigung_EK` (Format
  `PERCENT,MINIMUM,MAXIMUM`, Hoheit `KANTON` und `GEMEINDE`) in `VS_abzuege.csv`. Alleinerziehende fehlen, weil das
  Modell keine Kinder kennt;
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
(Sozialabzug); `id` ist der Schlüssel in `build.py` (Benennung wie in den früheren ESTV-Grunddaten, z. B.
`HauptErw_EK`).

**Auswahl der Abzüge** (ohne Kinder; je Gruppe gilt der erste vorhandene): `INCOME_DEDUCTIONS` in `build.py`.
Fehlt die gewünschte Variante, gilt die nächste vorhandene (Genf kennt nur «mit Beiträgen»). Berufskosten:
«Pauschalabzug übrige Berufskosten» (`HauptErw_EK`). Basel-Landschaft, Basel-Stadt und Tessin haben nach dem Gesetz
einen festen Pauschalbetrag (Format `MAXIMUM`); er gilt als fester Abzug und trägt `approx: true`, weil die
tatsächlichen Kosten höher sein können. Bei Format `STANDARDIZED` mit Prozent gilt der Betrag
als Mindestbetrag (AI 1000, AR und SG 700). Nicht enthalten: Kinder, einkommensabhängige Abzüge (bescheidene Einkommen,
Rentner, Entlastungsabzüge), Fahrkosten, Verpflegung.

Der Rechenkern (`crates/engine/src/taxdata.rs`) rechnet:
Einkommenssteuer = Kantonskurve(steuerbares Einkommen Kanton) × (Steuerfuss Kanton + Kirche)
+ Gemeindekurve × Steuerfuss Gemeinde − Ermässigung (Wallis, Verheiratete) + Bundeskurve(steuerbares Einkommen Bund);
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

- `kind: table`: eigener Tarif des Kantons für Kapitalleistungen aus Vorsorge (Stufen vom Typ `ZUERICH` mit Sockelbeträgen gelten
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

Tarife und Abzüge kommen für den Bund und alle Kantone aus den Gesetzen und Verordnungen (seit #296 auch das Wallis). Je Kanton
(und `CH` für den Bund) liegen unter `quellen/<Jahr>/` zwei Dateien:

- `<KT>_tarife.csv`: Spalten wie `tarife_<Jahr>.csv` (siehe unten), dazu `Quelle`;
- `<KT>_abzuege.csv`: Spalten wie `abzuege_<Jahr>.csv`, dazu `Quelle`; nur die Abzüge, die `build.py` verwendet
  (`INCOME_DEDUCTIONS`, `WEALTH_DEDUCTIONS`, `HauptErw_EK`, `ZweitVerdiener_EK`); die IDs sind die Schlüssel von
  `build.py`, die Bezeichnungen eigene Texte.

Jede Zeile nennt in `Quelle` den Erlass mit Artikel und Stand (z. B. `StG ZH § 35 Abs. 1 (LS 631.1; …)`). Ein Kanton
braucht beide Dateien; seine Zeilen aus den ESTV-Tabellen fallen dann ganz weg. Seit #296 haben alle Kantone
Quelldateien (`ESTV_ONLY` leer), es wird keine ESTV-Zeile mehr übernommen. Der Bericht beginnt mit der Herkunft je
Kanton (`laws (source files)`).
Kapitalbezugs- und Grundstückgewinnregeln nennen den Gesetzesartikel in `source` (`kapitalregeln.json`,
`grundstueckgewinn.py`). Stand 2026: Bund und Zürich (#287), Zentralschweiz LU, UR, SZ, OW, NW, ZG (#288), Nordwestschweiz BE, SO, BS, BL, AG (#289), Ostschweiz GL, SH, AR, AI, SG, GR, TG (#290), Westschweiz und Tessin FR, VD, NE, GE, JU,
TI (#291), Wallis (#296). Das Wallis hat dazu `VS_gemeindeindex.csv` (Spalten `Kanton,BFS,Nr,Gemeinde,Index,Quelle`):
den Index der Gemeindesteuer je Gemeinde in Prozent (LF VS Art. 178 Abs. 5, «Coefficients et indexations des
communes», Spalte des Steuerjahres); `build.py` rechnet ihn in den Faktor um (je 10 Punkte über 100 ÷ 1.10, der Rest
linear) und bricht ab, wenn eine Walliser Gemeinde fehlt. Die Kantonsstufen in `VS_tarife.csv` sind bereits mit dem
Index des Kantons umgerechnet. Wo das
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
   Excel) als `local/estv_income_rates.xlsx` ablegen. Die Optionen `--tarife` und `--abzuege` lesen weiterhin die
   ESTV-CSV; deren Zeilen werden für keinen Kanton mehr übernommen (`ESTV_ONLY` leer). Wallis: Index des Kantons und
   der Gemeinden für das neue Jahr nachführen (`VS_tarife.csv`, `VS_gemeindeindex.csv`), ebenso Mindest- und
   Höchstbetrag der Ermässigung für Verheiratete (`Ermaessigung_EK` in `VS_abzuege.csv`, «Adaptation des déductions
   forfaitaires»).
   Die undokumentierte Schnittstelle des Rechners nicht verwenden.
4. **Erzeugen:**
   ```bash
   python3 -I tools/steuerdaten/build.py --steuerfuesse local/estv_income_rates.xlsx \
     --tarife local/estv/tarife_<Jahr>.csv --abzuege local/estv/abzuege_<Jahr>.csv \
     --out crates/engine/data/steuern-<Jahr>.json --bericht local/estv/bericht.txt
   ```
   Das Werkzeug bricht ab, wenn ein Kanton oder der Bund keine Quelldatei hat. Der Bericht nennt je
   Kanton die Herkunft, die gewählten Tarife und Abzüge und die Regeln mit Gesetzesartikel.
5. **Abgleichen:** `vergleich.py` gegen die Datei des Vorjahres; jede Abweichung über 1 % mit der Teuerung oder einer
   Gesetzesänderung erklären.
6. Den Dateinamen in `crates/engine/src/taxdata.rs` (`BUILTIN`) anpassen.
7. **Referenzfälle:** Die Fälle aus den kantonalen Steuerrechnern (ZH, ZG, AG, GR, VS) und aus den amtlichen Tarifblättern
   (Tests `issue_287_…` bis `issue_291_…`, `issue_296_…`) für das neue Jahr neu rechnen. Die älteren Fälle aus dem ESTV-Steuerrechner
   in `tax_location.rs` und `tax_capital_location.rs` bleiben als zweite Prüfung (#292); Toleranz ±5 %.
8. Tests: `python3 -I -m unittest discover -s tools/steuerdaten` und `make test`.

## Steuerfüsse aus den Quellen der Kantone (#285, Weg b3, Versuch)

`steuerfuesse.py` liest die Steuerfüsse aus den Veröffentlichungen der Kantone statt aus dem ESTV-Steuerrechner und
vergleicht sie mit einer Steuerdatei. Als Versuch für alle 26 Kantone (Stand November 2026); die Steuerdatei baut weiter `build.py`.

- **ZH:** offene Daten des Statistischen Amts, `kanton_zuerich_stf_aktuell.csv` («Steuerfüsse der Zürcher Gemeinden»
  auf opendata.swiss, Nutzung mit Quellenangabe). Die Datei enthält den Gemeindesteuerfuss ohne Kirche und mit jeder
  Kirche; der Kirchensteuerfuss ist die Differenz. Der Staatssteuerfuss fehlt darin, er ist ein einziger Wert
  (`--zh-canton`, 2026: 95 %).
- **BE:** PDF der Finanzverwaltung «Steueranlagen der Gemeinden und Steuersätze der Kirchgemeinden»
  (`fv-stat-steueranlage_<Jahr>_1.pdf`), Tabelle 2.1; die Steueranlage des Kantons steht in Abschnitt 1.2. Das PDF wird
  ohne Bibliothek gelesen (nur Standardbibliothek). Es hat keine BFS-Nummern; die Namen werden mit der Steuerdatei
  abgeglichen, auch abgekürzte («Muri b. B.»). Die Geodaten «Steueranlagen pro Gemeinde» (opendata.swiss) enthalten
  keine Kirchensteuer und lagen im November 2026 noch beim Steuerjahr 2025.
- **GE:** Erlass des Staatsrats über die «centimes additionnels» des Jahres (HTML-Seite der Gesetzessammlung,
  `rsg_d3_05p30.htm`), in Hundertsteln der kantonalen Grundsteuer; Kantonssatz ein einziger Wert (`--ge-canton`, 2026:
  147.5). Keine Kirchensteuer (in Genf freiwillig).
- **VD:** Tabelle der Steuerbeschlüsse der Gemeinden (Excel des Kantons, `Arrêtés_d_imposition_<Jahr>.xls`, von Hand
  in CSV umgewandelt, z. B. mit LibreOffice), Spalte «Pour-cent total»; Kantonskoeffizient ein einziger Wert
  (`--vd-canton`, 2026: 155 %). Fraktionen von Gemeinden mit eigenem Satz werden wie im ESTV-Export weggelassen.
- **FR:** offene Daten des Amts für Statistik (`18_03_coefficients`, CSV mit «;», mit BFS-Nummern); genommen wird das
  neuste Jahr bis zum Jahr der Steuerdatei, mit Hinweis, wenn es älter ist. Im November 2026 reichten sie bis 2025;
  die Listen 2026 gibt es nur als PDF auf fr.ch.

- **Übrige 21 Kantone:** beschrieben in `quellen/2026/kantone.json`, je Kanton Quelle (`source`), Datei (`file`),
  Kantonssteuerfuss (`canton`) und ein regulärer Ausdruck für die Zeilen (`row`), dazu Angaben wie Massstab, Gesamt-
  statt Einzelsteuerfüsse, feste Kirchensteuer des ganzen Kantons, addierte Spalten oder Namensaliase (siehe
  `read_table` in `steuerfuesse.py`). Ein einziger Leser liest PDF, HTML, CSV und Text. PDFs, deren Tabellen der
  eigene PDF-Leser nicht zerlegen kann, wandelt `pdf2txt.swift` (PDFKit von macOS) zuerst in Text um:
  `swift tools/steuerdaten/pdf2txt.swift [--rows] <pdf> > <txt>`. Mit `--rows` setzt es die Textstücke nach ihrer Lage
  auf der Seite zu Zeilen zusammen; das braucht es, wo PDFKit spaltenweise liest (AR, GR, SH, SO, VS); ohne `--rows`
  für JU, NW, UR. Excel-Tabellen werden als CSV gespeichert.

Die Dateien lädt der Mensch von Hand herunter; sie liegen nicht im Repository.

```bash
python3 -I tools/steuerdaten/steuerfuesse.py crates/engine/data/steuern-2026.json --zh <csv> --be <pdf> \
  --ge <htm> --vd <csv> --fr <csv> --kantone tools/steuerdaten/quellen/2026/kantone.json --ordner <Ordner>
```

Gemeinden mit mehreren Teilen (ZH) oder Kirchgemeinden mit eigenem Satz (BE, Zeilen unter der Gemeinde) erscheinen als
Hinweis; genommen wird der erste Teil bzw. die Hauptzeile, wie im ESTV-Export.

## Rechtliches

Für die ESTV-Daten gibt es keine offene Lizenz (rechtliche Hinweise admin.ch). Eine Zustimmung der ESTV zur
Weitergabe liegt **nicht** vor (Stand 28.11.2026, #283): Die ESTV hat im November 2026 geantwortet, es gebe keine
Schnittstelle und keine Datenlieferung, und einer Weiterverwendung der Daten in Programmen anderer stimme sie vorerst
nicht zu; die Rückfrage zu den Steuerfüssen läuft. Bis zu einem Entscheid gibt keine verteilte Version die Steuerfüsse
aus dem ESTV-Export weiter (#283/AK5). Tarife, Abzüge und Regeln stammen aus den Erlassen (#286); die Steuerfüsse aus
den Quellen der Kantone sind der Weg ohne ESTV-Daten (#285, Abschnitt oben). Bei diesen gelten die Nutzungsbedingungen
der jeweiligen Quelle (offene Daten meist mit Quellenangabe).
