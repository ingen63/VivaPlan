# Quellen und Rechte der Zeitreihen

Die GPL v3 gilt für den Programmcode von VivaPlan. Die Zeitreihen in diesem Ordner und die daraus in das
Programm eingebauten Reihen fallen **nicht** unter die GPL. Die Rechte an den Grunddaten liegen bei den jeweiligen
Quellen; deren Nutzungsbedingungen gelten. Die Reihen werden mit Quellenangabe und nur für die Pensionsplanung
weitergegeben.

Ausgeliefert werden keine Originaldateien, sondern daraus berechnete Renditen (Monats- bzw. Jahreswerte). Wie sie
entstehen, steht in `docs/ENTWICKLUNG.md` (Abschnitt «Datenbibliothek»).

## Quellen

| Reihe | Quelle |
|---|---|
| `Shiller USD.csv` | Robert J. Shiller, U.S. Stock Markets 1871–Present (`ie_data.xls`), shillerdata.com: S&P Composite mit Dividenden, 10-jährige US-Staatsanleihen, US-CPI. Monatsrenditen berechnet für den Pensionsplaner. |
| `Shiller CHF.csv` | wie `Shiller USD.csv`, umgerechnet mit dem Monatsmittel USD/CHF der Schweizerischen Nationalbank (SNB, data.snb.ch); Teuerung: Landesindex der Konsumentenpreise (Bundesamt für Statistik, über data.snb.ch); SARON: SNB |
| `Shiller CHF + CH-Obligationen.csv` | wie `Shiller CHF.csv`; Obligationen aus der Rendite 10-jähriger Obligationen der Eidgenossenschaft (SNB) |
| `Welt (JST).csv` | Jordà-Schularick-Taylor Macrohistory Database, Release 6 (macrohistory.net/database, abgerufen 13.11.2026): Aktienrenditen von 11 Ländern, nach realem BIP gewichtet und in CHF umgerechnet; Obligationen der Eidgenossenschaft (JST bis 1988, danach SNB `rendoblim`); Teuerung Schweiz (JST). Zitate: Jordà, Schularick, Taylor (2017), «Macrofinancial History and the New Business Cycle Facts», NBER Macroeconomics Annual 2016; Jordà, Knoll, Kuvshinov, Schularick, Taylor (2019), «The Rate of Return on Everything, 1870–2015», Quarterly Journal of Economics. **Lizenz CC BY-NC-SA 4.0** (siehe unten). |

## Lizenz der JST-Daten

Die JST-Daten stehen unter Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International (CC BY-NC-SA 4.0).
Daraus folgt:

- **Nicht kommerziell:** VivaPlan muss kostenlos bleiben, solange es Reihen aus den JST-Daten enthält (auch die
  Strategien «Schweiz (JST)», «Welt (JST)», «Welt individuell» und die Obligationen der Eidgenossenschaft).
- **Quellenangabe:** mit den Zitaten oben und dem Abrufdatum, im Kopf der Datei und in der Hilfe.
- **Gleiche Lizenz:** `Welt (JST).csv`, der Auszug `quellen/jst-r6.csv` und die Rohdatei `quellen/JSTdatasetR6.xlsx`
  werden unter CC BY-NC-SA 4.0 weitergegeben, nicht unter der GPL.

Die Obligationen «Eidgenossenschaft kurzfristig» (FEA-59) entstehen in der Strategie aus derselben Datenbank (Spalte
`bill_rate`, Schweiz, 1900–2020, abgerufen 13.11.2026, Lizenz CC BY-NC-SA 4.0) und ab 2021 aus dem Jahresmittel des
SARON (SNB, data.snb.ch, Würfel `zimoma`, Datei `quellen/snb-zimoma.csv`); der Auszug `quellen/jst-r6.csv` enthält dazu
die Spalte `bill_rate`.

Die Schweizer Aktienreihe der JST-Daten beruht ab 1926 auf Pictet und Cie (1998).

Die frühere Reihe `Pictet CH.csv` wird seit FEA-72 nicht mehr mitgeliefert: Für die Tabelle von Pictet Wealth
Management liegt keine offene Lizenz vor.

## Haftungsausschluss der Shiller-Daten

Sinngemäss nach shillerdata.com: Die Datenreihen geben die Einschätzung von Robert J. Shiller, RSBB-I, LLC und ihren
Beratern wieder. Weder Professor Shiller noch die Yale University noch andere an der Zusammenstellung Beteiligte
geben eine ausdrückliche oder stillschweigende Gewähr für Inhalt, Form oder Verwendung der Daten. Die Daten sind keine
Prognose, keine Garantie und keine Anlageberatung; wer sie verwendet, trägt das ganze Risiko.

Dasselbe gilt für VivaPlan: Vergangene Renditen sind keine Prognose.

## Ordner `quellen/`

`quellen/` enthält die Grunddateien (SNB-Auszüge, JST-Tabelle mit Auszug), aus denen die Tests die ausgelieferten Reihen
nachrechnen. Sie gehören nicht zur App. Neuere Fassungen laden Sie selbst herunter (shillerdata.com, data.snb.ch) und
importieren sie in der App.

**Shiller-Tabelle (FEA-76):** Die Originaltabelle von Robert Shiller wird nicht mitgeliefert, weil shillerdata.com
keine Lizenz für die Weitergabe nennt. Ausgeliefert werden nur die daraus berechneten Renditen. Wer die Tests mit der
Originaltabelle laufen lassen will, lädt `ie_data.xls` von shillerdata.com und legt sie lokal als
`quellen/shiller-index.ods` ab (der Pfad ist von Git ausgeschlossen). Ohne die Datei überspringen die Tests diesen
Vergleich. Die ausgelieferten Reihen beruhen auf der Fassung vom Januar 2026; mit einer neueren Fassung weicht der
Vergleich ab, bis die Reihen neu erzeugt sind (`vivaplan import-shiller`).

## Nutzungsbedingungen der SNB und des BFS

- **SNB (data.snb.ch):** Die Daten dürfen mit Quellenangabe für nicht kommerzielle Zwecke verwendet und weitergegeben
  werden, die ihrem Zweck entsprechen; kommerzielle Nutzung nur mit Erlaubnis der SNB. Die Daten sind nicht als
  Grundlage für Finanztransaktionen (zum Beispiel als Benchmark) bestimmt. Quelle: Schweizerische Nationalbank,
  data.snb.ch, Würfel `devkum`, `plkopr`, `rendoblim`, `zimoma`.
- **BFS (Landesindex der Konsumentenpreise):** Nutzung erlaubt, auch kommerziell; die Quellenangabe mit Autor, Titel
  und Link ist Pflicht. Quelle: Bundesamt für Statistik (BFS), Landesindex der Konsumentenpreise, bezogen über das
  Datenportal der SNB, https://data.snb.ch, Würfel `plkopr`.
