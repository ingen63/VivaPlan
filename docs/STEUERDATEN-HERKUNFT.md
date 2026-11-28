# VivaPlan: Herkunft der Steuerdaten – Evidenz

Stand: 28.11.2026 (ersetzt die Fassung vom 27.11.2026). Erstellt in der Arbeitssitzung vom 26. bis 28.11.2026
(Entwicklung mit KI-Unterstützung). Zweck: Nachweis, dass die mitgelieferte Steuerdatei
`crates/engine/data/steuern-2026.json` ausschliesslich aus offenen, amtlichen Quellen gebaut ist und keine Daten des
ESTV-Steuerrechners enthält. Keine Rechtsberatung; die rechtliche Würdigung bleibt Sache von Fachleuten.

## 1. Ergebnis in Kürze

- Die Steuerdatei besteht aus drei Teilen, alle mit belegter amtlicher Herkunft:
  1. **Tarife, Abzüge und Regeln** (Bund und 26 Kantone): aus Gesetzen, Verordnungen und amtlichen Tarifblättern;
     1033 Zeilen nennen je den Erlass mit Artikel und Stand, die 26 Regeln für Kapitalleistungen und die
     26 Regeln der Grundstückgewinnsteuer den Gesetzesartikel.
  2. **Steuerfüsse** von Kanton, Gemeinden und Kirchen (2110 Gemeinden): aus den Veröffentlichungen der 26 Kantone
     (amtliche Listen, Erlasse, offene Daten), wenige Einzelwerte aus Beschlüssen von Gemeinde oder Kanton.
  3. **Gemeinden** (BFS-Nummer, Name, Kanton): Amtliches Gemeindeverzeichnis des Bundesamts für Statistik, Stand
     1.1.2026.
- **Beweis durch Neubau:** `python3 -I tools/steuerdaten/nachweis.py --ordner local/steuerquellen/2026/steuerfuesse`
  prüft jede Quelldatei gegen ihre SHA-256, prüft, dass keine Quelle von der ESTV stammt und jede Tarif- und
  Abzugszeile ihre Rechtsquelle nennt, baut die Steuerdatei aus diesen Dateien neu und vergleicht sie byte-genau mit der
  Datei im Repository. Ergebnis am 28.11.2026: alle Prüfungen PASS, SHA-256 der Steuerdatei
  `deb7c0727c7c23666c4a125cb8c8c755d747dd8099d9757c4eac7100ff008d69`. Der Export des ESTV-Steuerrechners wird dafür nicht gebraucht.
- Der ESTV-Export dient nur noch **lokal als Gegenprüfung** (`steuerfuesse.py`); er liegt nicht im Repository und
  fliesst in keinen Wert ein.

## 2. Rechtliche Grundlage der Quellen

| Art der Quelle | Beispiele | Grundlage |
|---|---|---|
| Gesetze, Verordnungen, amtliche Erlasse | Steuergesetze, ArCA-2026 GE, LCACant GE, Steuerordnung ERK BS | Art. 5 Abs. 1 lit. a URG: nicht urheberrechtlich geschützt |
| Veröffentlichungen von Behörden | Steuerfusslisten der Steuerverwaltungen, Protokoll einer Gemeindeversammlung, Medienmitteilung des Regierungsrats, Circolare TI | Art. 5 Abs. 1 lit. c URG (Entscheidungen, Protokolle und Berichte von Behörden und Verwaltungen); zudem sind einzelne Zahlen (Steuerfüsse) keine Werke im Sinn von Art. 2 URG |
| Offene Behördendaten | ZH, BL (opendata.swiss «terms_by»: freie Nutzung, Quellenangabe Pflicht), TG (CC0 1.0), FR (CC BY-SA 4.0), BFS-Gemeindeverzeichnis («terms_open»: freie Nutzung) | Lizenz des Datensatzes; Pflichten siehe Abschnitt 6 |

Die Schweiz kennt keinen eigenen Schutz für Datenbanken. Ein Risiko nach Art. 5 lit. c UWG (Übernahme eines
marktreifen Arbeitsergebnisses mit technischen Reproduktionsverfahren) bestand beim ESTV-Export als Ganzem; die
Steuerdatei übernimmt keinen solchen Bestand mehr, jeder Wert stammt aus der Veröffentlichung des zuständigen Gemeinwesens.

## 3. Quellen der Steuerfüsse (je Datei mit Adresse, Abrufdatum, SHA-256)

Vollständige Liste im Repository: `tools/steuerdaten/quellen/2026/steuerfuesse_quellen.json`. Die Dateien selbst liegen
lokal in `local/steuerquellen/2026/steuerfuesse/` (nicht im Repository).

| Datei | Herausgeber und Inhalt | Art | Nutzung | Abruf | SHA-256 (Anfang) |
|---|---|---|---|---|---|
| `AG.pdf` | [Kantonales Steueramt Aargau, Verzeichnis der Gemeinde- und Kirchensteuerfüsse 2026 (v5)](https://www.ag.ch/media/kanton-aargau/dfr/dokumente/steuern/natuerliche-personen/steuerberechnung-tarife-natuerliche-personen/gemeinde-und-kirchensteuerf-sse-2026-v5.pdf) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `530aad3d424c` |
| `AI.pdf` | [Kantonale Steuerverwaltung AI, Steuerfüsse der verschiedenen Körperschaften, Stand 2026](https://ai-api.webcloud7.ch/themen/steuern/publikationen/504937da83424b3887dc4a0ccad377b7/steuerfuesse-der-verschiedenen-koerperschaften-stand-2021.pdf/@@download/file/Steuerf%C3%BCsse%20der%20verschiedenen%20K%C3%B6rperschaften%20-%20Stand%202026.pdf) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `b3962f179cc9` |
| `AR.pdf` | [Kantonale Steuerverwaltung Appenzell Ausserrhoden, Steuerfüsse 2026](https://ar.ch/fileadmin/user_upload/Departement_Finanzen/Steuerverwaltung/Bibliothek/2026/Steuerfuesse_2026.pdf) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `34b9d573d5d5` |
| `BE.pdf` | [Finanzverwaltung Kanton Bern, Steueranlagen der Gemeinden und Steuersätze der Kirchgemeinden 2026](https://www.fin.be.ch/content/dam/fin/dokumente/de/finanzverwaltung/finanzausgleich/publikationen/fv-stat-steueranlage_2026_1.pdf) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `ada779d1970b` |
| `BL.csv` | [Amt für Daten und Statistik BL, Steuerfüsse und Steuersätze nach Gemeinde und Jahr (10580)](https://data.bl.ch/api/explore/v2.1/catalog/datasets/10580/exports/csv?delimiter=%3B) | offene Daten | Freie Nutzung, Quellenangabe Pflicht (opendata.swiss terms_by) | 2026-11-28 | `81e46aeb184e` |
| `BS.html` | [Steuerverwaltung Basel-Stadt, Steuerfüsse (Stand 2026)](https://www.bs.ch/themen/arbeit-und-steuern/steuererklaerung/steuerrechner) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `05bdabcb75dd` |
| `BS_erk_steuerordnung.pdf` | [Evangelisch-reformierte Kirche Basel-Stadt, Steuerordnung (§ 34)](https://www.erk-bs.ch/dok/11788) | Erlass einer öffentlich-rechtlichen Körperschaft | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `871c65662a91` |
| `FR.csv` | [Service des communes FR, Coefficients et taux d'impôts par commune (18_03_coefficients)](https://opendata.fr.ch/api/explore/v2.1/catalog/datasets/18_03_coefficients/exports/csv?delimiter=%3B) | offene Daten | CC BY-SA 4.0 | 2026-11-28 | `20815a16666d` |
| `FR_Heitenried.html` | [Gemeinde Heitenried, Steuerfuss 2026](https://www.heitenried.ch/steuerfuss) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `9e79383faeeb` |
| `FR_Mezieres_GV.pdf` | [Commune de Mézières (FR), procès-verbal de l'assemblée communale du 10.12.2025](https://www.mezieres-fr.ch/assets/idm_medias/d02826624746b4b21a15b0c0bfd503a2.pdf) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `7f3a70570738` |
| `FR_kath.pdf` | [Service cantonal des contributions FR, Coefficients d'impôt des paroisses catholiques 2026](https://www.fr.ch/document/602756) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `1fa6d14a7f75` |
| `FR_ref.pdf` | [Service cantonal des contributions FR, Coefficients d'impôt des paroisses réformées 2026](https://www.fr.ch/de/document/602761) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `f14ddee84f41` |
| `GE.htm` | [Conseil d'État GE, ArCA-2026 (RSG D 3 05.30)](https://silgeneve.ch/legis/data/rsg_d3_05p30.htm) | amtlicher Erlass | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `cabbbdcd4590` |
| `GE_LCACant_D3_07.htm` | [Loi sur les centimes additionnels cantonaux (RSG D 3 07)](https://silgeneve.ch/legis/data/rsg_d3_07.htm) | amtlicher Erlass | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `dbe6eac7fcfd` |
| `GE_calculette_ipc_2026.xlsx` | [AFC GE, Calcul de l'impôt sur les prestations en capital 2026 (calculette IPC); nur zur Bestätigung der Regel D 3 06 Art. 1 / LCACant Art. 2, keine Zahl übernommen](https://www.ge.ch/document/43113/telecharger) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `31afff65304a` |
| `GL.pdf` | [Kantonale Steuerverwaltung Glarus, Gemeinde- und Kirchensteuer 2026](https://www.gl.ch/public/upload/assets/65564/091%20Gemeindezuschl%C3%A4ge%202026.pdf?fp=2) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `36b024789cf2` |
| `GR.pdf` | [Steuerverwaltung Graubünden, Gemeinde- und Kirchensteuerfüsse 2026](https://www.gr.ch/DE/institutionen/verwaltung/dfg/stv/berechnen/Documents/gemeindesteuerfuesse2026.pdf) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `61f42a905d3f` |
| `JU.pdf` | [Service des contributions JU, Quotités d'impôt des communes et taux d'impôts 2026](https://www.jura.ch/Htdocs/Files/v/df8873b29ac94b19ca37ba3cd2342e441d1d36d69901ca072f0552c0edab3822.pdf/quotites_2026.pdf?download=1) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `852214a934d4` |
| `LU.html` | [Dienststelle Steuern Luzern, Gemeindeübersicht Steuereinheiten 2026](https://steuern.lu.ch/publikationen/nav_einheiten_tarife/stnap_steuereinheiten_2026) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `a9188f2093a2` |
| `NE.html` | [Service des contributions NE, Coefficients et barèmes](https://www.ne.ch/themes/impots/coefficients-et-baremes) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `63e2c66d453b` |
| `NW.pdf` | [Kantonales Steueramt Nidwalden, Steuerfüsse 2026](https://www.steuern-nw.ch/app/uploads/2025/12/NW-1173433-v1-Steuerfuesse_2026.pdf) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `b6cc7eb14b2a` |
| `OW.html` | [Kanton Obwalden, Steuerfüsse 2001–2026](https://www.ow.ch/publikationen/8258) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `420b545b28d0` |
| `SG.pdf` | [Kantonales Steueramt St. Gallen, Steuerfüsse der Gemeinden 2026](https://www.sg.ch/content/dam/sgch/steuern-finanzen/steuern/formulare-und-wegleitungen/einkommens-und-vermoegenssteuer/tarife-und-steuerfuesse/steuerfuesse-st-gallische-gemeinden/Steuerf%C3%BCsse%202026%20f%C3%BCr%20Versand.pdf) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `abde299107d3` |
| `SH.pdf` | [Kantonale Steuerverwaltung Schaffhausen, Steuerfüsse 2026](https://sh.ch/CMS/get/file/7a9bb85b-3d02-4076-bec3-9f9936e71009) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `21a3835c1447` |
| `SO.pdf` | [Amt für Gemeinden SO, Statistische Mitteilung Steuern und Gebühren 2026 (Tabelle 4.4 nennt den ESTV-Steuerrechner als Quelle, Entscheid #285)](https://so.ch/fileadmin/internet/vwd/vwd-agem/pdf/Gemeindefinanzen/Statistik/2026/Statistische_Mitteilung_Steuern_und_Gebuehren_2026_V3.0.pdf) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `9bbfb8f43b28` |
| `SO_Dornach_RRB.html` | [Staatskanzlei SO, Medienmitteilung 30.6.2026 (Dornach 92 %)](https://so.ch/verwaltung/staatskanzlei/medien/medienarchiv-2026/juni/juni-2026/news/einwohnergemeinde-dornach-regierungsrat-beschliesst-budget/) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `6e2410c29a5e` |
| `SZ.pdf` | [Steuerverwaltung Schwyz, Steuerfusstabelle 2026](https://www.sz.ch/public/upload/assets/90947/Steuerfusstabelle_2026.pdf?fp=4) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `707684dd2cd9` |
| `TG_sk-stat-70.csv` | [Amt für Daten und Statistik Thurgau, Gesamtsteuerfüsse der Thurgauer Gemeinden (sk-stat-70)](https://data.tg.ch/api/explore/v2.1/catalog/datasets/sk-stat-70/exports/csv?delimiter=%3B) | offene Daten | CC0 1.0 | 2026-11-28 | `c72ff60d37a2` |
| `TI.html` | [Divisione delle contribuzioni TI, Moltiplicatori d'imposta comunali](https://www4.ti.ch/dfe/dc/sportello/moltiplicatori-comunali/?tendina=523) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `a769d0b172aa` |
| `TI_circolare_2006_03.pdf` | [Divisione delle contribuzioni TI, Circolare n. 3/2006 mit Umwandlungstabelle (Kapitalleistungen, kapitalregeln.json)](https://m4.ti.ch/fileadmin/DFE/DC/DOC-CIRC/circ_2006_03.pdf) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `3f853281e36a` |
| `UR_gem.pdf` | [Kanton Uri, Steuerfüsse der Einwohnergemeinden ab 2009](https://www.ur.ch/_doc/439292) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `6c288ede9718` |
| `UR_kirche.pdf` | [Kanton Uri, Steuerfüsse der Kirchgemeinden ab 2009](https://www.ur.ch/_doc/439280) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `b901feb74ca9` |
| `VD.xls` | [Canton de Vaud, Tableau des arrêtés d'imposition 2026](https://www.vd.ch/fileadmin/user_upload/themes/territoire/communes/finances_communales/fichiers_xls/Arr%C3%AAt%C3%A9s_d_imposition_2026.xls) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `e7e04fc78b69` |
| `VS.pdf` | [Service cantonal des contributions VS, Coefficients et indexations des communes 2022–2027](https://www.vs.ch/documents/508074/628286/Coefficients_Indexations_Communes_2022-2027.pdf/53d0770a-3e87-6d30-03d7-011ca4c7a3e0) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `a8e2c5aaabeb` |
| `VS_Saxon_reglement_culte.pdf` | [Commune de Saxon, règlement sur l'impôt du culte](https://www.saxon.ch/wp-content/uploads/2018/10/24-regle_culte_23.pdf) | amtlicher Erlass | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `80d19b8b2567` |
| `VS_Saxon_tarifs.pdf` | [Commune de Saxon, tarifs communaux (impôt du culte 2,5 %)](https://www.saxon.ch/wp-content/uploads/2021/06/tarifs-communaux-1.pdf) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `d8de553f217b` |
| `ZG.pdf` | [Steuerverwaltung Zug, Steuerfüsse der Gemeinden 2023–2026](https://cdn.zg.ch/dam/jcr:24fb8832-3383-4718-9a48-9461bc7501fc/Steuerf%C3%BCsse%202026_effektiv%2021.1.26.pdf) | amtliche Veröffentlichung | amtliche Veröffentlichung einer Behörde (Art. 5 Abs. 1 lit. a/c URG) | 2026-11-28 | `8ca1615cef34` |
| `ZH.csv` | [Statistisches Amt Kanton Zürich, Steuerfüsse der Zürcher Gemeinden, aktuell](https://www.web.statistik.zh.ch/ogd/data/steuerfuesse/kanton_zuerich_stf_aktuell.csv) | offene Daten | Freie Nutzung, Quellenangabe Pflicht (opendata.swiss terms_by) | 2026-11-28 | `741392e13fb7` |
| `gemeinden.csv` | [Bundesamt für Statistik (BFS), Amtliches Gemeindeverzeichnis, Stand 1.1.2026](https://www.agvchapp.bfs.admin.ch/api/communes/snapshot?date=01-01-2026) | offene Daten | Freie Nutzung (opendata.swiss terms_open) | 2026-11-28 | `e0748c06c754` |
Abgeleitete Dateien (nur Formatwandel, Befehl in der Liste): Text aus PDF mit `tools/steuerdaten/pdf2txt.swift`
(PDFKit von macOS) für GR, JU, NW, SH, SO, UR, VS; CSV aus der Excel-Tabelle VD mit LibreOffice.

Wie jede Quelle gelesen wird (Spalten, Muster, Massstab, Kantonssteuerfuss), steht je Kanton in
`tools/steuerdaten/quellen/2026/kantone.json`, das Werkzeug in `tools/steuerdaten/steuerfuesse.py` (Tests
`test_steuerfuesse.py`).

## 4. Einzelwerte (Ausnahmen) und Annahmen

`tools/steuerdaten/quellen/2026/ausnahmen.json` hält Werte fest, die nach der Kantonsliste beschlossen wurden; jeder
Eintrag nennt seine Quelle und die Belegdatei (Feld `beleg`):

| Gemeinde | Wert | Beleg |
|---|---|---|
| SO Dornach | Steuerfuss 92 % | Regierungsrat SO, Medienmitteilung 30.6.2026 |
| FR Mézières | 93 % | Protokoll der Gemeindeversammlung 10.12.2025 (angenommen 68:15) |
| FR Heitenried | 81 % | Seite der Gemeinde «Steuerfuss» 2026 |
| FR Fétigny-Ménières | 86 % | Fusion 1.1.2026; Vorjahr der grösseren Gemeinde Fétigny (offene Daten FR) – Annahme |
| VS Saxon | Kultussteuer 2,5 % | Tarifblatt und Reglement der Gemeinde |
| SO Mümliswil-Ramiswil, AR Stein, Lutzenberg, Wolfhalden, OW Engelberg | Kirchensteuer der ersten Kirchgemeinde bzw. keine | Kantonsliste (Lesart bei mehreren Kirchgemeinden, #313) |

Festgelegte Annahmen (ohne Wert aus fremder Quelle):
- GE und NE: keine Kirchensteuer, weil die Kirchenabgabe freiwillig ist (GE «contribution religieuse volontaire», NE
  Konkordat); BL vorerst ohne Kirchensteuer (#314); BS römisch- und christkatholisch sowie VS Sion, Savièse, Törbel,
  Vouvry ohne Kirchensteuer, weil kein amtlicher Satz 2026 gefunden wurde. Hinweis in der Hilfe.
- FR: 9 Gemeinden mit geändertem Koeffizienten 2026 (Lully, Meyriez, Ursy, Vuisternens-devant-Romont, Villaz,
  La Roche, Muntelier, Brünisried, La Verrerie) mit dem Wert 2025 der offenen Daten, weil die Liste 2026 nicht
  veröffentlicht ist und kein amtlicher Beschluss online gefunden wurde (für Lully nur ein Zeitungsartikel, für Meyriez
  nur der Antrag an die Gemeindeversammlung).
- Kantonssteuer GE: Einkommen 130,8 % der Grundsteuer (47,5 Zuschlagsrappen, Ermässigung 12 % nach D 3 06 Art. 1,
  1 Rappen für die Hilfe zu Hause nach LCACant Art. 2), Vermögen 148,5 %; bestätigt durch die Kalkulation der
  Steuerverwaltung GE (calculette IPC 2026).

## 5. Tarife, Abzüge und Regeln

Unverändert gegenüber der Fassung vom 27.11.2026 (Issues #286–#292, #296, #297): Quelldateien
`tools/steuerdaten/quellen/2026/<KT>_tarife.csv` und `<KT>_abzuege.csv` mit Spalte `Quelle` je Zeile; heruntergeladene
Erlasse mit Adresse und Datum in `local/steuerquellen/2026/quellen.txt` (87 Einträge). Neu (#285):
- Kapitalleistungen GE nach LIPP Art. 45 Abs. 2 (ein Fünftel des Tarifs) statt eines aus ESTV-Rechnungen abgeleiteten
  Anteils; TI nach LT Art. 38 Abs. 2 mit der Umwandlungstabelle im Anhang der Circolare DC TI n. 3/2006 (Alter 65,
  Männer). Beide stimmen in den Referenzfällen auf 0,2 % mit dem ESTV-Rechner überein (nur Gegenprüfung).
- Zweitverdienerabzug BE mit StG BE Art. 38 Abs. 2 belegt (vorher als «aus dem ESTV-Rechner abgeleitet» kommentiert).
- Grundstückgewinnsteuer: alle 26 Regeln nennen den Gesetzesartikel; das ESTV-Steuermäppchen 2025 diente nur als
  Ausgangspunkt (Tabelle mit Gesetzeszitaten), kein Wert wird von dort übernommen.

## 6. Verbleibende Punkte (offen gelegt)

| Punkt | Art | Stand |
|---|---|---|
| SO Kirchensteuer: Tabelle 4.4 der Statistischen Mitteilung SO 2026 nennt als Quelle den ESTV-Steuerrechner | Wert aus kantonaler Veröffentlichung, die ihrerseits die ESTV zitiert | übernommen auf Entscheid (#285); Ersatz durch Listen der Landeskirchen SO möglich |
| VS Kapitalleistungen: Divisor 19,77 und Höchstsatz der Gemeinde aus dem Walliser Steuerrechner (Sitten) kalibriert | kantonaler, nicht ESTV-Rechner; Rechner können urheberrechtlich oder vertraglich geschützt sein | offen; Gesetzesregel LF VS Art. 33b (Satz der Jahresleistung, höchstens 4 %) mit der amtlichen Umwandlungstabelle wäre der Ersatz |
| Rundungsregeln GR und AG mit dem kantonalen Steuerrechner geprüft | nur Prüfung, kein Wert übernommen | unkritisch |
| Testfälle im Repository mit Ergebnissen des ESTV-Steuerrechners (`tax_location.rs`, `tax_capital_location.rs`, `tax_gain_cantons.rs`) | einzelne Rechenergebnisse für erfundene Fälle, im öffentlichen Repository | Entscheid offen: behalten oder in lokale Tests verschieben |
| Git-Historie: frühere Fassungen von `steuern-2026.json` (vor #285) und `kapitalregeln.json` enthalten Steuerfüsse aus dem ESTV-Export bzw. Faktoren aus ESTV-Rechnungen | im öffentlichen Repository, die Historie wird nicht umgeschrieben (Projektregel) | offen gelegt; ab dem Commit dieser Fassung ist die Datei ESTV-frei |
| FR offene Daten CC BY-SA 4.0 | Lizenzpflicht: Quellenangabe und Weitergabe abgeleiteter Daten unter gleicher Lizenz | Quelle steht in der Steuerdatei und in der Hilfe; Lizenzfrage der Steuerdatei klären |
| Quellenangabe Pflicht (ZH, BL) | Lizenzpflicht | erfüllt über `kantone.json`, Bericht und Hilfe; ein Hinweis in der App («Quellen der Steuerdaten») wäre die sichtbare Form |
| VD: Senkung der Kantonssteuer 2026 um 5 % (laut ESTV-Kantonsblatt) | im Gesetz und im Tarif 2026 (Koeffizient 100 %, Kanton 155 %) nicht gefunden | offen |
| AR: Lesart StG Art. 52 Abs. 2 | Auslegung | offen (wie bisher) |
| #283: Antwort der ESTV zur früheren Nutzung | rechtlich | offen; für die neue Datei nicht mehr nötig |

## 7. Prüfung der Inhalte (Gegenprüfung, nicht Herkunft)

- Rust-Tests: 1437 grün (28.11.2026), Golden-Dateien unverändert.
- Gegenprüfung der Steuerfüsse gegen den ESTV-Export (lokal): 2062 von 2110 Gemeinden gleich; jede Abweichung ist erklärt
  (neuere Kantonsliste AG/TI, mehrere Kirchgemeinden #313, FR Vorjahr, Kirchensteuer ohne Quelle, gewollt GE/NE/BL).
  Ergebnis: `local/steuerquellen/2026/gegenpruefung.txt`.
- Bericht des Baus: `local/steuerquellen/2026/bericht.txt` (Herkunft je Kanton, Kirchensteuern ohne Quelle).

## 8. Wo die Belege liegen

| Beleg | Ort |
|---|---|
| Liste der Quellen der Steuerfüsse mit Adresse, Datum, SHA-256 | Repository `tools/steuerdaten/quellen/2026/steuerfuesse_quellen.json` |
| Beschreibung je Kanton, Ausnahmen mit Beleg | Repository `tools/steuerdaten/quellen/2026/kantone.json`, `ausnahmen.json` |
| Prüfskript (Neubau, Hashes, keine ESTV-Quelle) | Repository `tools/steuerdaten/nachweis.py` |
| Quelldateien der Steuerfüsse | lokal `local/steuerquellen/2026/steuerfuesse/` |
| Erlasse für Tarife und Abzüge | Repository `tools/steuerdaten/quellen/2026/*.csv`; lokal `local/steuerquellen/2026/` mit `quellen.txt` |
| ESTV-Export (nur Gegenprüfung) | lokal `local/estv_income_rates.xlsx`, `local/estv/` |
| Begründungen | GitHub-Issues #283, #285, #286–#292, #296, #297, #313, #314 |
