# VivaPlan – Entwicklung

Technische Beschreibung für Entwicklerinnen und Entwickler: Aufbau des Repos, Bauen, Kommandozeile, Datenbibliothek,
Projektdatei und die Rechenregeln Schritt für Schritt. Was die App kann, steht im [README](../README.md) und in
[BESCHREIBUNG.md](BESCHREIBUNG.md).

## Aufbau

| Ordner | Inhalt |
|---|---|
| `crates/engine/` | Bibliothek: Datenmodell, Simulation, Zeitreihen, Steuern, Prüfung, Import alter Konfigurationen; Tests unter `tests/` |
| `crates/engine/data/` | Datenbibliothek: ausgelieferte Zeitreihen (`Shiller CHF.csv`, `Shiller USD.csv`, `Welt (JST).csv`) sowie in `crates/engine/data/quellen/` SNB-Auszüge und die JST-Tabelle mit Auszug (die Shiller-Originaldatei wird nicht mitgeliefert, siehe `QUELLEN.md`) |
| `crates/engine/assets/` | Beispielplanungen (`beispiel.vplan`, Vorlage `Neu.vplan`) |
| `crates/cli/` | Kommandozeile `vivaplan` (Rechnen, Prüfen, Import, Vergleich, Entwicklungsserver) |
| `crates/app/` | Desktop-App (Tauri 2): Fenster, Dateidialoge, Anbindung an den Rechenkern |
| `crates/app/ui/` | Oberfläche: Editor, Live-Vorschau, Ergebnisansicht (HTML/JS ohne externe Bibliotheken), Hilfe in `hilfe/` |
| `e2e/` | Oberflächentests (Playwright) |
| `fuzz/` | Fuzzing (cargo-fuzz, eigener Workspace) |
| `tools/` | Skripte: Git-Hooks, Build und Installation, Rauchtest `ui-smoke.sh`, Testqualität, Übersichtsseite, Issues |
| `docs/` | Beschreibung, Teststrategie ([`docs/TESTS.md`](TESTS.md)), QA-Bericht, Spezifikation (`docs/spezifikation/`) |
| `local/` | Nur lokal (von Git ausgeschlossen): Mutationsläufe, Berichte der Testqualität, eigene Planungen und alte Konfigurationen (`local/eigene/`), Notizen |

## Desktop-App

```sh
cd crates/app && npm install   # einmalig: Tauri-Kommandozeile
npx tauri build                # erzeugt target/release/bundle/macos/VivaPlan.app und .dmg
npx tauri dev                  # App im Entwicklungsmodus
```

Für Windows bzw. Linux wird `npx tauri build` auf dem jeweiligen System ausgeführt.

Unter macOS baut der Git-Hook `tools/git-hooks/post-commit` nach jedem Commit, der Rechenkern,
Oberfläche oder Cargo-Dateien ändert, die App im Hintergrund und ersetzt `/Applications/VivaPlan.app`
(Skript `tools/build-install.sh`, Protokoll in `target/build-install.log`). Reine Test- und Dokumentationscommits
lösen keinen Build aus. Für einen einzelnen Commit abschalten: `BUILD_INSTALL=0 git commit …`.
Die Hooks gelten nach `git config core.hooksPath tools/git-hooks`.
Zum Testen der Oberfläche im Browser (ohne Dateidialoge): `vivaplan ui-dev` aus dem Projektordner starten
und <http://127.0.0.1:8787/> öffnen; der Server lauscht nur lokal.

**Bedienung:** Menü und Tastenkürzel (macOS ⌘, sonst Ctrl): ⌘N neu, ⌘O öffnen, ⌘S speichern, ⇧⌘S speichern unter,
⌘I alte config.json importieren, ⌘↩ Simulation starten, Esc zurück zum Editor, ⌘Z / ⇧⌘Z rückgängig / wiederholen
(ganze Planung), ⌘1 … ⌘0 Abschnitt bzw. Registerkarte, ⌘↑ / ⌘↓ voriger / nächster Abschnitt, ⌘/ Übersicht.

**Ergebnisse:** Registerkarten Übersicht, Grafiken, Einzelne Läufe (5./50./95. Perzentil), Szenarien, Jahreswerte;
alle Beträge wahlweise nominal oder in heutiger Kaufkraft (Preise des Simulationsstarts). Grafiken eines Laufs:
Vermögensverlauf, liquides Vermögen nach Anlage (Beträge oder Anteile), Einnahmen und Ausgaben pro Jahr,
Entnahmerate, Immobilien (Wert, Hypothek, Eigenkapital), Vorsorge. Bei mehreren Läufen zusätzlich: Bandbreite,
Wahrscheinlichkeit, dass das Geld reicht (über die Zeit), Verteilung von Endvermögen und Vermögen bei Pensionierung,
Alter bei Erschöpfung. Szenarien: Erfolgsquote und Endvermögen je Variante (eine Variation) bzw. Heatmaps (zwei).

## Befehle

```sh
cargo build --release
B=target/release/vivaplan

$B run crates/engine/assets/beispiel.vplan --events          # Jahresübersicht, Kennzahlen, Ereignisse
$B run crates/engine/assets/beispiel.vplan --csv monate.csv  # alle Monatswerte
$B validate crates/engine/assets/beispiel.vplan              # Fehler und Hinweise
$B import-legacy local/eigene/config.json --start-year 2026 --base local/eigene -o meine.vplan
$B bench crates/engine/assets/beispiel.vplan --paths 50000   # Rechenleistung
$B optimize crates/engine/assets/beispiel.vplan --max-ruin 5 --ruin-age 95   # beste Entnahmestrategie suchen
$B schema > planung.schema.json                  # JSON-Schema der Projektdatei
$B library                                       # Zeitreihen der Datenbibliothek (Ordner crates/engine/data/)
$B import-shiller ie_data.xls                    # Shiller-Daten neu einlesen (.xls, .xlsx, .ods)
$B import-snb devkum.csv rendoblim.csv           # SNB-Daten aktualisieren, CHF-Reihen neu berechnen

make test                                        # alle Tests (nextest, Grund in docs/TESTS.md)
tools/ui-smoke.sh                                # Oberflächentest
cargo llvm-cov --workspace --exclude vivaplan-app --summary-only   # Testabdeckung
```

`returns.strategy` (FEA-53) wählt die Indizes: `{"stocks": "worldJst" | "worldCustom" | "chJst" | "sp500",
"bonds": "ch" | "usa" | "chShort", "weights": {"ch", "usa", "eu", "apac"}}` (Gewichte nur für `worldCustom`, zusammen 1).
Daraus entsteht die Zeitreihe (`crates/engine/src/strategy.rs`); ein gesetztes `returns.series` (ältere Planungen,
Kommandozeile) hat Vorrang und wird beim Öffnen in der App zur Strategie.

`returns.source` steuert die Läufe: `fixed` (ein Lauf), `historical` (ab `startDate`), `rolling`
(jeder historische Startzeitpunkt, der den ganzen Zeitraum abdeckt), `bootstrap` und `parametric` (Monte Carlo).
Alle Läufe werden parallel auf allen Kernen gerechnet; Monte-Carlo-Pfade entstehen erst im Lauf (wenig Speicher).

## Monte Carlo

- **`bootstrap`** (`{"paths": 50000, "block": 12, "seed": 42}`): stationärer Bootstrap aus der gewählten Zeitreihe.
  Pro Monat endet ein Block mit Wahrscheinlichkeit 1/`block`, dann folgt ein zufälliger neuer Startmonat; am Ende
  der Reihe geht es am Anfang weiter. Aktien, Obligationen, Inflation und SARON stammen immer aus demselben
  historischen Monat. Bei Jahresreihen gilt die Blocklänge in Jahren (`block` / 12).
- **`parametric`** (`{"paths", "mu", "sigma", "df", "jumpLambda", "jumpMu", "jumpSigma", "seed"}`): Aktienmodell des
  bisherigen Programms in Tagesschritten (21 pro Monat): Log-Rendite (μ − σ²/2)·dt + σ·√dt·Z mit Student-t-Schocks Z
  (auf Varianz 1 skaliert, `df` = 0 → Normalverteilung) plus Poisson-Sprünge (Rate `jumpLambda` pro Jahr, Höhe
  log-normal mit `jumpMu`, `jumpSigma`). Obligationen, Inflation und SARON folgen den fixen Werten.
- Pfad *i* entsteht immer aus (`seed`, *i*): gleiche Einstellungen ⇒ gleiche Ergebnisse; alle Szenarien verwenden
  dieselben Pfade. Die Detailansichten (5./50./95. Perzentil) rechnen den betreffenden Pfad exakt nach.
- «Alle historischen Startzeitpunkte» unterschätzt die Streuung: 864 Startmonate aus 104 Jahren überlappen stark
  (nur gut drei unabhängige 32-Jahres-Abschnitte). Der Bootstrap zeigt deshalb breitere Bandbreiten.

## Szenarien

`scenarios.sweeps` = Liste von `{"param", "from", "to", "step"}`. Parameter: `wealth.stockShare`, `pension.pk.lumpsum`,
`returns.startDate` (Monat, Schritt in Monaten), `milestone.retire` (Monat; setzt die Pensionierung als Datum),
`flows.spendingFactor` (Faktor auf die Lebenshaltungskosten ohne Wohnanteil). Jede Kombination (technisch höchstens 1'000'000) wird mit allen Pfaden
gerechnet: Erfolgsquote, Vermögen bei Pensionierung (Median), Endvermögen 5./50./95. Perzentil, früheste Erschöpfung.

## Datenbibliothek

Planungen verweisen mit `"series": "library:Shiller CHF.csv"` auf eine Reihe der Bibliothek, sonst mit einem
Pfad (absolut oder relativ zur Projektdatei). Die Desktop-App führt die Bibliothek im App-Datenordner
(macOS: `~/Library/Application Support/ch.vivaplan-app.desktop/Zeitreihen`) und legt dort fehlende
ausgelieferte Reihen an; Kommandozeile und Entwicklungsserver verwenden `crates/engine/data/` (oder `--library <ordner>`).

| Reihe | Inhalt |
|---|---|
| `Shiller USD.csv` | Monatlich ab 01.1872: S&P Composite inkl. Dividenden, 10-jährige US-Staatsanleihen (Gesamtrendite), US-CPI |
| `Shiller CHF.csv` | Monatlich ab 01.1922: dieselben Anlagen in CHF (Monatsmittel USD/CHF der SNB, ohne Absicherung), Schweizer Landesindex der Konsumentenpreise, SARON ab 06.1999 |
| `Shiller CHF + CH-Obligationen.csv` | Monatlich 02.1988 – 07.2025: Aktien und Inflation wie `Shiller CHF`, Obligationen = 10-jährige Obligationen der Eidgenossenschaft |
| `Welt (JST).csv` | Jährlich 1900 – 2020: Welt-Aktienindex aus den JST-Daten (11 Länder, nach realem BIP gewichtet) in CHF, Obligationen der Eidgenossenschaft (JST bis 1988, danach SNB), Schweizer Teuerung; Lizenz CC BY-NC-SA 4.0 |

- **Shiller-Import:** Monatsrendite = Veränderung von realem Gesamtrendite-Index × CPI gegenüber dem Vormonat
  (Aktien entsprechen (P + D/12) / P₋₁ − 1, Obligationen Shillers «Monthly Total Bond Returns», dort um einen Monat
  früher eingetragen). Shillers Kurse sind Monatsdurchschnitte; Einbrüche erscheinen dadurch geglättet.
  Inflation = CPI gegenüber Vorjahresmonat, damit der Dezemberwert der Jahresteuerung entspricht.
- **Schweizer Obligationen:** Gesamtrendite einer 10-jährigen Obligation zu pari aus der Monatsrendite (SNB
  `rendoblim`, 10 Jahre): Coupon = Rendite des Vormonats, Bewertung zur neuen Rendite mit 9 11/12 Jahren Restlaufzeit,
  plus ein Monatscoupon. Die SNB führt diese Reihe seit 07.2025 nicht mehr nach.
- **SNB-Würfel** von data.snb.ch (CSV): `devkum` (Reihe M0/USD1), `plkopr` (LD2010100), `zimoma` (SARON), `rendoblim` (10J).
  Ein Auszug ist eingebaut; neuere Downloads ersetzen ihn (`crates/engine/data/quellen/snb-<würfel>.csv`).
- **CSV-Format:** Kommentarzeilen `# Titel: …`, `# Währung: CHF`, `# Quelle: …`, `# Hinweis: …` vor der Kopfzeile.
  Leere SARON-Werte sind erlaubt; dort gilt der fixe SARON-Wert der Planung.
- Ist die Reihe in einer anderen Währung als die Planung, meldet die Prüfung einen Hinweis.
- **Rechte:** Die Zeitreihen fallen nicht unter die GPL; Quellen, Bedingungen und Haftungsausschluss in
  `crates/engine/data/QUELLEN.md`.

## Projektdatei (`.vplan`, JSON)

Kennung `"format": "vivaplan/1"`. Ältere Dateien (`.rplan`, Kennung `pensionsplaner/1`) bleiben lesbar und werden
beim Speichern als neue `.vplan`-Datei geschrieben.

Alle Zeitpunkte sind Kalendermonate (`"2027-05"`) oder Verweise auf Meilensteine (`"retire"`), `"never"` = nie.
Meilensteine sind ein Datum oder das Alter einer Person (`{"kind": "age", "person": "A", "years": 65}`).
`start` und `end` sind Pflicht. Zeitabhängige Werte sind eine Zahl oder
`{"mode": "timed", "rows": [{"from": "start", "value": 9500}, {"from": "retire", "value": 8500}]}`.
Beträge gelten pro Monat, Sätze pro Jahr als Dezimalzahl (0.05 = 5 %). Das vollständige Format liefert `vivaplan schema`.

## Rechenregeln

Reihenfolge pro Monat:

1. **Teuerung:** jeden Monat Teuerungsindex × (1 + Inflation des Vormonats)^(1/12). Ausgaben, Mieten und (falls
   gewählt) Immobilienwerte und Nebenkosten werden damit fortgeschrieben (je Objekt einzeln wählbar:
   `inflationValue`, `inflationFixCosts`, `inflationRent`; fehlen sie, gilt `inflation` für Wert und Nebenkosten und die
   Miete wird fortgeschrieben); Ausgaben werden in Preisen bei
   Simulationsstart erfasst. Die AHV (falls indexiert) wird nur im Januar um die seit dem letzten Januar
   aufgelaufene Teuerung angepasst.
2. **Ereignisse:** einmalige Beträge; Pensionierung (Kapitalbezug mit Kapitalsteuer, Rente = Restkapital ×
   Umwandlungssatz / 12); AHV-Beginn; Immobilien (Notverkauf aus dem Vormonat, Kauf, Verkauf mit
   Grundstückgewinnsteuer, Hypothekarperioden); Säule 3a (Verzinsung, Auszahlung).
3. **Geldflüsse** in das Cash: Einkommen + Renten + Zuflüsse − Lebenshaltung − Wohnkosten.
4. **Rendite** auf Aktien und Obligationen (Jahresrendite geometrisch auf Monate umgerechnet), sofern das Portfolio positiv ist.
5. **Pensionskasse** bis zum Bezug: Beiträge monatlich, Verzinsung im Dezember.
6. **Steuern:** Im Dezember werden die Einkommenssteuer auf das Jahreseinkommen (inkl. Renten) und die
   Vermögenssteuer auf Portfolio + Immobilien netto veranlagt; bezahlt wird sie im Mai des Folgejahres. Eine bei
   Planende noch offene Steuer wird im letzten Monat abgezogen.
7. **Notverkauf prüfen:** wenn liquides Vermögen < Schwelle × (Ausgaben − Einkommen); ausgeführt im Folgemonat.
   Ist danach (oder nach einem geplanten Verkauf) keine Immobilie mehr im Besitz, wird im Folgemonat die nächste
   Immobilie mit Kauf `"afterSale"` gekauft, sofern tragbar (Tragbarkeit wie bei den Banken, Hypothek allenfalls
   gekürzt) und die Eigenmittel reichen; sonst beginnen die Mieten mit `"from": "afterSale"`.
   Mit `"buy": "afterSale:<id>"` wird im Monat nach dem Verkauf einer bestimmten Immobilie gekauft (Verweis über
   deren `id`); klappt der Kauf nicht und ist keine Immobilie mehr im Besitz, beginnen die Mieten.
   Die Tragbarkeit rechnet nur mit dem Einkommen (Erwerb, Renten): (`sustainability` × Jahreseinkommen − Nebenkosten) /
   `imputedRate`; `wealthContribution` wird nicht mehr verwendet. Sie wird auch bei **jeder Erneuerung** einer Hypothekarperiode
   geprüft (Einkommen im Erneuerungsmonat, Nebenkosten wie erfasst, kalkulatorischer Zins `imputedRate`): Der nicht tragbare
   Teil ist die minimale Einmalzahlung. Sie wird aus dem Vermögen anteilig auf alle Tranchen bezahlt, wenn das Vermögen
   danach >= 0 ist und das Notverkaufskriterium nach der Zahlung nicht auslöst (Schätzung: geplante Ausgaben + Zins,
   Amortisation und Nebenkosten nach der Zahlung − Einkommen); die Hypothek wird dann reduziert erneuert. Sonst keine
   Zahlung, keine Erneuerung, Notverkauf im Folgemonat. Jede Erneuerung steht in `renewals` des Laufs
   (`affordable`, `lumpSum`, `debtBefore`, `debtAfter`, `emergencySale` = «Vermögen < 0» oder «Notverkaufskriterium»); die
   Zusammenfassung zählt `lumpSums` und `affordabilitySales`, die Stapelauswertung `lumpSumRuns` und `affordabilitySaleRuns`.
   Mit `"amountMode": "maxLoan"` beträgt die Hypothek beim Kauf die höchste Belehnung (80 %, also Minimum
   Eigenmittel), gekürzt falls nicht tragbar; die Beträge der Tranchen gelten dann als Anteile.
8. **Entnahmestrategie «Standard»:** Cash = Reserve (Monate × ungedeckte Ausgaben), Rest nach Aktienanteil.
   **«3-Topf»** (`"strategy": "three_buckets"`, `"buckets": {"cashYears", "bondYears", "refillThreshold", "reserveMonths"}`)
   ab dem Start der Entnahmephase (die erste Pensionierung), davor «Standard». Topf 1 = Cash (alle Ausgaben werden
   daraus bezahlt), Topf 2 = Obligationen, Topf 3 = Aktien; ein für einen Kauf im Folgemonat reservierter Verkaufserlös
   gehört zu keinem Topf. **Kein Topf fällt je unter 0** (einzige Ausnahme: alle Töpfe leer und keine Immobilie mehr).
   **Netto-Bedarf** = Lebenshaltung + Wohnen (ohne Amortisation) + Einkommenssteuer − AHV − PK-Rente − Erwerbseinkommen, aus den
   geplanten Werten des laufenden Jahres (Beginn mit einem Auslöser A: Monate ab dem Auslöser, auf 12 Monate hochgerechnet,
   auch die Einkommenssteuer); einmalige Ausgaben, Käufe, Amortisationen und Sondersteuern zählen nicht. Bedarf ≤ 0 → Ziele 0.
   Ziel Topf 1/2 = Jahre × Netto-Bedarf, nie aus dem Topfstand. **Renditekriterium:** Aktienrendite der letzten 12 Monate (in den
   ersten 12 Monaten seit Januar) ≥ `refillThreshold`; sonst sind nur Verkäufe aus Topf 3 gesperrt.
   **Januar-Auffüllung:** Topf 1 aus Topf 2, der Rest aus Topf 3, danach Topf 2 aus Topf 3 (bei nicht erfülltem Kriterium: nur
   Topf 1 aus Topf 2, Topf 2 wird nicht nachgefüllt); Überschuss in Topf 1 oder 2 → Topf 3.
   **A** (Start der Entnahmephase, PK-Kapitalbezug, Neustart nach «alle Töpfe leer»; einmal pro Monat): Kapitalbezug bzw. das
   gesamte liquide Vermögen zuerst in Topf 3, dann wie im Januar ohne Renditekriterium, Ziele um die vergangenen Monate des
   Jahres gekürzt ((Jahre × 12 − Monate) / 12 × Netto-Bedarf). **B** (externe Zuflüsse, 3a, Verkauf ohne Folgekauf, Notverkäufe):
   nur Topf 3, keine Auffüllung. **C** (Vorausschau, jeden Monat; deckt Topf 1 die Ausgaben des Monats (laufende und einmalige) nicht):
   **C1** bei erfülltem Renditekriterium: Ablauf wie die Januar-Auffüllung, Ziele um die seit Januar vergangenen Monate
   gekürzt ((Jahre × 12 − Monate) / 12 × Netto-Bedarf), Ziel Topf 1 plus die einmaligen Ausgaben des Monats, keine
   Überschussbuchung; reicht Topf 1 danach nicht, folgt C2. **C2** bei nicht erfülltem Kriterium: Topf 1 aus Topf 2, dann
   Topf 3 so, dass er die Ausgaben deckt und danach (Min(`reserveMonths`, Monate bis Januar) − 1) Monatsbedarfe enthält;
   Topf 2 wird nicht aufgefüllt. Das Ziel von Topf 1 enthält auch in der Januar-Auffüllung und bei A die einmaligen Ausgaben
   des Monats; Überschussbuchungen in Topf 3 gibt es nur in der Januar-Auffüllung. Reichen die Töpfe nicht: Notverkauf einer Immobilie
   (Erlös in Topf 3, dann C erneut); ohne Immobilie darf Topf 1 negativ werden, Topf 2 und 3 sind stillgelegt, alle Zuflüsse
   fliessen in Topf 1, und sobald das Vermögen wieder positiv ist, läuft A. Reihenfolge im Monat: Einnahmen in Topf 1, Zuflüsse
   direkt an ihr Ziel (Topf 3 oder Reservierung; Protokoll «extern → Topf 3», «extern → Reservierung», «Reservierung → Topf 3»),
   Käufe (Eigenmittel aus der Reservierung, Rest als einmalige Ausgabe des Monats vorgemerkt), Januar-Auffüllung bzw. A, C mit
   allen Ausgaben des Monats, dann die Zahlung aus Topf 1. Die Umbuchungen werden nach dem Renditeschritt gebucht; die
   Entscheidungen gelten für den Stand vor den Ausgaben des Monats. Protokoll `bucketRefills` (`trigger` Januar/A/B/C1/C2,
   Bezug, Ausgaben, AHV, PK-Rente, Erwerb, Bedarf, Ziele, `passedMonths`, Aktienrendite, Kriterium/`conditionIgnored`,
   Umbuchungen `moves` (Topf 0 = extern), Stände vor und nach, `payment`).
   **Sofortige Anwendung bei Zuflüssen** gilt für die dynamischen Strategien (Budget neu berechnen); 3-Topf behandelt Zuflüsse
   nach Fall B. Für die dynamischen Strategien wirken sie zusätzlich zum Januar im Monat eines Zuflusses: einmaliger Betrag
   > 0, PK-Kapitalbezug, Verkaufserlös. Reihenfolge im Monat: Zuflüsse und Verkäufe, Käufe, dann die Strategie (einmal pro
   Monat, auch wenn der Monat ein Januar ist). Ist im Folgemonat der Kauf einer anderen Immobilie geplant (fester Monat M+1
   oder «nach Verkauf»), wird der Verkaufserlös reserviert (bleibt liquide, «Standard» legt ihn nicht an) und die Strategie
   erst im Kaufmonat nach dem Kauf angewendet.
   Die neue Jahresentnahme gilt ab dem Zuflussmonat, nicht rückwirkend. Protokoll: Ereignis «Entnahmestrategie angewendet
   nach …» (nur ausserhalb des Januars und nicht beim Start der Strategie) und `propertyEvents[].note` beim Verkauf.

**Hypotheken:** Tranchen mit einer Abfolge von Perioden (fest oder SARON + Marge, SARON mit Untergrenze 0).
Die erste Periode beginnt am Startdatum der Tranche (bzw. beim Kauf), jede weitere am Ende der vorherigen;
danach Verlängerung zu gleichen Konditionen oder Rückzahlung. Zins = Schuld × Satz / 12 pro Monat, aufgelaufen und zusammen mit der Amortisation am Quartalsende (März, Juni, September, Dezember) bezahlt; beginnt die Hypothek im April, ist die erste Zahlung im Juni fällig (April bis Juni). Bei einem Verkauf werden die aufgelaufenen Zinsen sofort, am Planende im letzten Monat bezahlt. Die Lebenshaltung enthält keine Miete und keine Hypothekarzinsen; diese kommen als Wohnkosten hinzu.
Amortisation als fixer Betrag pro Quartal, fällig Ende März/Juni/September/Dezember; direkt (reduziert die Schuld)
oder indirekt (Einzahlung in die Säule 3a). Die 3a-Auszahlung zahlt zuerst Tranchen mit indirekter Amortisation zurück.

**Steuern:** Satz aus der Tabelle linear interpoliert × Steuerfuss, auf Rappen gerundet. Kapitalbezüge im selben
Kalenderjahr (PK, 3a) werden für den Satz zusammengerechnet.

## Import alter Konfigurationen

`import-legacy` übernimmt eine `config.json` des früheren Programms als Ausgangspunkt. Das frühere Programm gilt
nicht als Referenz; die Rechenregeln sind in diesem Dokument beschrieben und mit eigenen Referenzfällen getestet.
Immobilien werden nur näherungsweise übernommen (Hypotheken zu gleichen Konditionen verlängert, Amortisation pro
Quartal); der Import meldet das. Mehrere Kapitalbezüge (`LumpsumRatio`) werden zu einem Bezug zusammengefasst.

## Zweite Person, weitere Entnahmestrategien, Pakete

- **Double Income:** `household.persons` mit zwei Personen und `partner` (`income`, `pk`, `ahv`, wie bei der
  Hauptperson). Pensionskasse und AHV laufen pro Person; Steuern, Vermögen und Ausgaben gelten für den Haushalt.
  Als pensioniert gilt der Haushalt (Entnahmestrategien), sobald alle Personen mit Bezugsdatum pensioniert sind.
  Ehepaar-Regeln (`household.married`): beide AHV-Renten zusammen höchstens `ahvCoupleCap` (Standard 3 780 pro
  Monat = 150 % der Maximalrente 2025), optional eigene Steuertabellen `taxes.couple`. Hinterlassenenrenten
  und Todesfälle werden nicht berechnet.
- **Fixe Entnahmerate** (`"strategy": "fixed_rate"`, `"fixedRate": {"rate"}`) und **Guardrails** (`"guardrails":
  {"initialRate", "upper", "lower", "adjust"}`, nach Guyton-Klinger): ab Pensionierung bestimmt die Strategie die
  Lebenshaltung. Entnahme = Lebenshaltung + Wohnkosten − Einkommen und Renten (vor Steuern). Fixe Rate: im Januar
  `rate` × liquides Vermögen. Guardrails: Startentnahme bei Pensionierung, danach im Januar mit der Teuerung
  (nicht nach einem Aktien-Verlustjahr, solange die Rate über der Startrate liegt); Rate über Startrate × (1 +
  `upper`) ⇒ Kürzung um `adjust`, unter Startrate × (1 − `lower`) ⇒ Erhöhung. `withdrawal.floor` / `ceiling`
  begrenzen die Lebenshaltung (Anteil der geplanten Ausgaben, 0 = keine Grenze). Variation `withdrawal.rate`.
- **VPW** (`"strategy": "vpw"`, `"vpw": {"horizonAge", "realReturn"}`) und **Endowment** (`"endowment": {"rate",
  "smoothing"}`): wie fixe Rate bestimmt die Strategie die Lebenshaltung. VPW entnimmt im Januar den Anteil des
  liquiden Vermögens, der es bis `horizonAge` (Alter der Hauptperson) mit der Realrendite gleichmässig aufbraucht
  (Annuitätenfaktor). Endowment (Yale/Tobin): Entnahme = (1 − `smoothing`) × Vorjahr mit Teuerung + `smoothing` ×
  `rate` × Vermögen. Beide kennen `floor` / `ceiling`.
- **Optimieren:** In der App unter «Optimierung» (Ergebnis lässt sich mit einem Klick in die Planung übernehmen; die Einstellungen werden in der Planung unter `optimization` gespeichert und gelten auch für `optimize`, Optionen überschreiben sie), im Terminal mit `optimize <planung>`: sucht für jede Strategie die Parameter, die den Konsum (Median und P10,
  real) maximieren, solange die Ruinwahrscheinlichkeit unter der Grenze bleibt, und prüft sie auf unabhängigen Pfaden.
- **Ergebnisse:** Bandbreite mit 1. und 99. Perzentil und dem monatlich schlechtesten Wert; unter «Einzelne Läufe»
  zusätzlich 1., 99. Perzentil und der Lauf mit dem tiefsten Endvermögen. Einzelläufe zeigen den Verlauf von
  Aktien und Obligationen als Index.
- **Installationspakete:** `.github/workflows/pakete.yml` baut auf GitHub Windows (.msi, .exe), Linux (.deb, .rpm,
  .AppImage) und macOS (.dmg) – von Hand gestartet oder durch ein Tag `v…` (dann als Entwurf unter «Releases»).
  Lokal geht jeweils nur das eigene System (`npx tauri build`). Die Pakete sind nicht signiert.

### Grundstückgewinnsteuer

Unter «Steuern» sind die Regeln der Grundstückgewinnsteuer vollständig editierbar (`taxes.propertyGain`): Stufen (Breite und
Satz), Zu- und Abschläge nach Besitzdauer (`duration`: ab vollen Jahren, Änderung der Steuer), Freigrenze (`exemptBelow`) und
eine feste Ermässigung (`reduction`). Die Besitzdauer folgt aus dem Kaufdatum der Immobilie (`bought`, bei geplanten Käufen
der Kaufmonat). Die Schaltfläche «Vorlage Kanton Zürich» setzt den Tarif § 225 StG (StA eForm. 225.1, Stand 09/2025): Stufen
10 % bis 40 %, Zuschlag 50 % unter 1 Jahr und 25 % unter 2 Jahren, Abzug 5 % ab 5 Jahren plus 3 Prozentpunkte pro Jahr bis
50 % ab 20 Jahren, Gewinne unter 5 000 steuerfrei. Tests (`crates/engine/tests/reference.rs`) vergleichen mit der amtlichen Tabelle.
Für andere Kantone die Tabellen anpassen; ohne Kaufdatum gilt kein Zu- oder Abschlag.

`taxes.propertyGain.model` wählt das Modell: `zurich` (Standard, wie oben) oder `aargau`. Das Modell Aargau kennt keine Stufen:
`durationRates` (ab vollen Jahren, Steuersatz) gibt den **direkten Satz auf den ganzen Gewinn** nach Besitzdauer an (Vorlage
§ 109 StG: 40 % im 1. Besitzjahr, 2 Prozentpunkte weniger je Jahr bis 20 % im 11. Jahr, danach 1 Punkt weniger bis 6 % im
25. Jahr, 5 % ab 25 vollen Jahren); ohne Kaufdatum gilt der höchste Satz. Freigrenze und feste Ermässigung gelten in
beiden Modellen. `useMultiplier` multipliziert die Steuer zusätzlich mit dem Steuerfuss (`taxes.multiplier`, Standard
aus). «Eigene Regeln erfassen» beginnt mit einer Zeile im Modell Aargau.

### Bandbreiten der Optimierung

Für jede Strategie lässt sich pro Parameter festlegen, ob er fix auf dem Wert der Planung bleibt oder in einer Bandbreite
variiert wird (Einheit des Parameters: Jahre, Monate, Prozent, Alter). Beispiel 3-Topf: Topf 1 fix, Topf 2 von 0 bis 2 Jahre,
Nachfüllen ab Vorjahresrendite −30 % bis +10 %. Gespeichert wird das in der Planung:

```json
"optimization": { "ranges": { "three_buckets": {
  "cashYears": { "fixed": true },
  "bondYears": { "lo": 0, "hi": 2 },
  "refillThreshold": { "lo": -0.3, "hi": 0.1 } } } }
```

Ohne Eintrag gelten die Standard-Bandbreiten (`engine::optimize::param_defs`); Untergrenze und Obergrenze der Lebenshaltung
(Guardrails, Fixe Rate) sind ohne Eintrag fix. Der Befehl `opt_params` liefert der Oberfläche die Parameter mit Einheit,
Standard-Bandbreite und aktuellem Wert; `optimize` auf der Kommandozeile liest die Bandbreiten aus der Planung.

### Dauerschätzung

Unter «Optimierung» und «Szenarien & Simulation» steht eine grobe Schätzung der Rechendauer. Der Befehl `bench_plan` misst
die Zeit eines Laufs (einige tausend Läufe, meist unter einer Sekunde) und trennt das Erzeugen der Renditepfade vom Rechnen;
die Oberfläche rechnet damit die Läufe hoch (bei der Optimierung mit den gemessenen Suchschritten je Strategie). Gegenprobe mit
echten Läufen: Abweichungen von 0.8 bis 1.4, deshalb zeigt die Anzeige eine Spanne. Die Dauer der Simulation steht
ausserdem links neben «Simulation starten».

### Fortschritt und Abbruch

Simulation, Szenarien und Optimierung zeigen einen Fortschrittsbalken mit vergangener und geschätzter Restzeit und einen
Knopf «Abbrechen». Die Befehle `progress_get` und `cancel_job` fragen den Stand ab bzw. verlangen den Abbruch (`engine::progress`);
die Läufe werden in Blöcken gerechnet, nach jedem Block wird der Abbruch geprüft. Es läuft immer nur eine Berechnung.

### Liste der Simulationen

Jede Simulation wird nach dem Start automatisch abgelegt, mit der Planung von damals und dem vollständigen Ergebnis.
Die Schaltfläche «Simulationen» öffnet die Liste: Name und Notiz sind direkt in der Zeile editierbar, «Öffnen» zeigt das
gespeicherte Ergebnis (mit der Planung von damals; «Planung in den Editor übernehmen» lädt sie zurück), «Löschen» und
«Markierte löschen» entfernen Einträge. Zwei bis sechs markierte Einträge lassen sich vergleichen: Kennzahlen nebeneinander,
Verlauf des Gesamtvermögens (Median und 5. Perzentil) und eine Tabelle der geänderten Eingaben. Die Liste liegt lokal im
Speicher der App (IndexedDB) und nicht in der Planungsdatei.

### Hilfe

Die Hilfe öffnet als Panel rechts und ist kontextabhängig: **F1** zeigt das Thema des Felds mit dem Fokus, sonst des
aktuellen Abschnitts bzw. der Ergebnisansicht; der Knopf «? Hilfe», die Zeichen ⓘ neben Feldern, Kartenüberschriften und
Grafiken sowie das Menü *Hilfe* (Handbuch, ⌘⇧?) führen ebenfalls hin. Die Texte liegen als Markdown in `crates/app/ui/hilfe/`
(ein Thema beginnt mit `## @kennung Titel`, Verweise als `[Text](#kennung)`); neue Felder ordnet man in `crates/app/ui/help.js` zu
(`FIELD_TOPICS`, `HEADING_TOPICS`). `tools/ui-smoke.sh` prüft, dass alle Verweise und Zuordnungen auf ein Thema zeigen.

## Test quality

Die Testqualität wird auf mehreren Ebenen gemessen (Details und Schichten: `docs/TESTS.md`):

| Ebene | Werkzeug | Lokal | Report |
|---|---|---|---|
| Ausführung | cargo-nextest (Profil `ci`, Wiederholungen, flaky = Fehler) | `make test` | `junit.xml` |
| Abdeckung | cargo-llvm-cov (Gate: Zeilen ≥ 80 % je Paket, `tools/coverage-min`) | `make coverage` | HTML, LCOV, Cobertura |
| Wirksamkeit | cargo-mutants (PR und nächtlich: nur geänderte Zeilen seit dem letzten erfolgreichen nächtlichen Lauf; voll: lokal oder manuell in Actions) | `make mutants`, `make mutants-diff` | `local/mutants.out/` (`missed.txt`) |
| Eingabebreite | proptest (`crates/engine/tests/proptest_parsers.rs`), cargo-fuzz (`fuzz/`) | `make proptest`, `make fuzz` | Fehlerfälle, Korpora |

Miri und loom sind nicht eingerichtet: der Code enthält kein `unsafe` und nur einfache Atomic-Flags (`progress.rs`).

Workflows: `.github/workflows/test-quality.yml` (Push auf `main`, Pull Requests, nächtlich und von Hand) und `mutation-full.yml` (nur manuell gestartet; ein voller Lauf dauert viele Stunden und passt nicht ins Free-Kontingent privater Repos).
Reports liegen als Actions-Artefakte und in der Job-Summary. Geplante Läufe werden **übersprungen**, wenn es seit dem
letzten erfolgreichen Lauf keinen neuen Commit gibt. Lokal ohne CI: `make quality` (`FORCE=1 make quality` erzwingt
einen Lauf; Reports unter `local/test-quality/reports/`). Nötig: `cargo install cargo-nextest cargo-llvm-cov cargo-mutants`.

## Offen für spätere Etappen

weitere Datenquellen (z. B. Schweizer Aktien und Obligationen), Export der Ergebnisse.
