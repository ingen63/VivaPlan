# QA-Bericht und Testmatrix

Stand 7. November 2026. Grundlage: Testplan «VivaPlan» (Haushalt bis Performance). Die bestehende Suite
(siehe `TESTS.md`) wurde gegen den Plan abgeglichen; Lücken sind in `crates/engine/tests/qa_matrix.rs` geschlossen.

## Testmatrix

«vorh.» = schon vor diesem Durchgang abgedeckt (Datei laut `TESTS.md`), «neu» = `qa_matrix.rs`.

| Bereich des Plans | Abdeckung |
|---|---|
| Invarianten: Vermögensgleichung (≤ 0.01 CHF) | vorh. (500 Zufallsplanungen), neu `check_books` in allen Fällen |
| Invariante: Cashflow | vorh. `cash_flows_are_conserved_without_returns` (ohne Renditen); neu `monthly_cash_flow_identity_with_returns_and_taxes` (mit Renditen, Steuern, Einmalbeträgen, Renten; alle vier Strategien, fixe, Bootstrap- und parametrische Pfade; Abweichung ≤ 0.01 CHF) |
| Invariante: Alter | neu: Alter im Monatsdatensatz = Alter-Meilenstein = Ereignismonat |
| Invariante: Reproduzierbarkeit | vorh. (fix), neu Bootstrap und parametrisch mit gleichem/anderem Startwert |
| Haushalt: Einzelperson, Ehepaar, 30/60/100 Jahre | neu |
| Haushalt: ungültige Fälle (Ende vor Start, Geburt in Zukunft, Alter > 120, leere Pflichtfelder) | neu, dabei zwei Fehler gefunden (F1, F2) |
| Meilensteine: Datum/Alter, mehrere, Verschieben der Pensionierung | neu |
| PK: 100 % Rente/Kapital/50-50, sehr klein/gross, Rente = Kapital × Umwandlungssatz | neu |
| Säule 3a: Auszahlung früh/bei Pensionierung/danach | neu; laufende Einzahlung und Nicht-Einzahlung vorh. (`reference.rs`) |
| AHV: fix, mit/ohne Teuerung | neu |
| Einnahmen/Ausgaben: Einmalzahlungen, Randwerte, altersabhängiger Wechsel im exakten Monat | neu |
| Aktienanteil 0–100 %, Reserve 0–60 Monate | neu |
| Strategien: Standard, 3-Topf (Januar), Guardrails, feste Rate 2–10 % | neu (Standard, Januar, Raten), Guardrails vorh. (`household.rs`) |
| Entnahmerate: Basis ohne Immobilien (liquid 1'000'000 + Immobilie 1'000'000 ⇒ 40'000, nicht 80'000), feste Rate und Guardrails | neu `withdrawal_rate_base_excludes_property` |
| Immobilien: Bestand, Kauf, Verkauf, Objektwechsel, Notverkauf | neu + vorh. |
| Hypotheken: fest 2/5/10 J., SARON + Marge, Amortisation, Tragbarkeit | neu (Laufzeiten, SARON), Amortisation/Tragbarkeit vorh. |
| Steuern: Einkommen, Vermögen, Kapital (unter/in/über Stufe), Grundstückgewinn, Ehepaar | neu + vorh. |
| Historisch: 1929, 1973, 2000, 2007/08; alle Startmonate | neu (Renditen werden Monat für Monat nachgerechnet; Anzahl Läufe = mögliche Startmonate) |
| CSV-Import: fehlende Spalten, Datum, leer, doppelt, Lücke, Text | neu; Währung vorh. (`api::currency_issue`) |
| Monte Carlo: Reproduzierbarkeit, NaN/∞, Plausibilität, 1'000 vs. 10'000 Pfade | neu; Blockbildung/Korrelation vorh. (`data.rs`) |
| Szenarien 1D/2D, Konsistenz mit Einzellauf | neu (3×3×3×3 = 81 Kombinationen, 4 Punkte gegen separaten Lauf) |
| Ergebnisse: Perzentile = Pfade, Jahreswerte, Vorschau = Simulation | neu |
| Dateien: Roundtrip, Fehler beim Öffnen | neu; Neu/Öffnen/Speichern unter in der Oberfläche **nicht automatisch** |
| CLI: validate, run, export, import | vorh. (`crates/cli/tests/cli.rs`) |
| UI: Ansichten, Konsolenfehler | vorh. (`tools/ui-smoke.sh`); neu Playwright (`e2e/`): Navigation, Eingaben und Prüfung, Live-Vorschau = Simulation, Meilensteine verschieben, Undo/Redo mit 100 Aktionen, Datei Neu/Öffnen/Speichern/Roundtrip, Ergebnisansichten, Tooltips, Responsiveness (4 Breiten), Dunkelmodus, gespeicherte Simulationen |
| Performance und Stresstest | neu, `cargo nextest run --cargo-profile release -p vivaplan-engine --run-ignored only -E 'test(/^qa_matrix::/)' --no-capture` (nextest wie `make test`, siehe `TESTS.md`) |
| Golden Master (10 Fälle) | neu, `crates/engine/tests/golden.rs` mit Ergebnissen in `crates/engine/tests/golden/` |

## Gefundene Fehler

| Nr. | Schwere | Befund | Stand |
|---|---|---|---|
| F1 | mittel | Geburtsdatum nach dem Simulationsstart (z. B. 2999) wurde nicht beanstandet; das Alter wurde negativ gerechnet. | behoben (`validate.rs`) |
| F2 | mittel | Alter über 120 Jahre am Simulationsende (z. B. Geburt 1850) wurde nicht beanstandet; ein Alter-Meilenstein war nur bis 130 begrenzt. | behoben (Grenze 120 am Simulationsende) |
| F3 | niedrig | Kopfzeile mit 11 Schaltflächen brach erst ab 760 px um: zwischen 761 und rund 1'300 px scrollte die ganze Seite seitwärts (Playwright, Responsiveness). | behoben (`index.html`, `flex-wrap`) |
| F4 | mittel | Einkommenssteuer wird im Dezember des Steuerjahrs bezahlt; nach Schweizer Praxis einmal jährlich veranlagt und im Mai des Folgejahres fällig. Test `income_tax_is_assessed_yearly_and_paid_in_may_of_next_year`. | behoben (`sim.rs`): Veranlagung im Dezember, Zahlung im Mai des Folgejahres, offene Steuer im letzten Planmonat |
| F5 | mittel | Teuerung wirkt als Sprung im Januar (Vorjahresrate); erwartet ist monatliche Anwendung (1 + i)^(1/12). Test `inflation_is_applied_every_month`. | behoben (`sim.rs`): monatlich (1 + i)^(1/12) |
| F6 | mittel | Hypothekarzinsen wurden monatlich bezahlt; nach Praxis fallen Zins und Amortisation quartalsweise an (erste Zahlung bei Start im April im Juni). Tests `mortgage_interest_is_paid_quarterly_first_payment_in_june`, `accrued_interest_is_settled_on_sale_and_at_plan_end`, `living_costs_exclude_rent_and_mortgage_interest`. | behoben (`sim.rs`): Zins läuft monatlich auf, Zahlung am Quartalsende; bei Verkauf und Planende Restzahlung |

AHV-Anpassung nur im Januar: geprüft und in Ordnung (`indexed_ahv_changes_only_in_january`). Wegen F4 bis F6 wurden die zehn Golden Masters neu erzeugt und fünf ältere Tests an die neuen Regeln angepasst (Teuerung monatlich, Steuer im Mai).

Kritische oder hohe Fehler (Rechenfehler, verletzte Bilanz, ungültige Zahlen) wurden in keinem Lauf gefunden.
Die übrigen 9 anfänglichen Fehlschläge waren falsche Annahmen in den neuen Tests (Nebenkosten sind monatlich,
Überschüsse werden angelegt, Monatsformat), keine Programmfehler.

**Folge von F2:** Ein 100-Jahre-Plan ist nur noch für eine Person möglich, die höchstens 20 Jahre vor Start
geboren ist (Alter am Ende ≤ 120). Das entspricht dem Plan («Alter > 120 → Validierungsfehler»).

## Referenzrechnung «Test Retirement.ods»

`crates/engine/tests/golden/Test Retirement.ods` ist die Basisrechnung des Altprogramms als Tabelle (Blatt 1: Rendite 0 %,
Blatt 2: 5 %, PK-Zins 10 %, Blatt 3 = Blatt 1). `crates/engine/tests/ods_reference.rs` liest die berechneten Werte der Tabelle
und rechnet dieselbe Planung im Programm (Zeile r = Monat r − 3 ab Januar 2030; Pensionierung, AHV-Beginn, Amortisation
und Verkauf aus den Ereignissen der Tabelle).

| Prüfung | Ergebnis |
|---|---|
| Blatt 1, Monate 0 bis zur Amortisation (270 Monate) | Vermögen auf den Rappen gleich |
| Blatt 2, Monate 0 bis zur Pensionierung (161 Monate, mit 5 % und halbierter Rendite ab Zeile 153) | auf den Rappen gleich |
| Blatt 2, PK-Kapitalbezug (PK-Zins 10 %) | Tabelle 845'142, Programm 832'147 (−1.5 %): andere Zinskonvention |
| Blatt 1, alle 604 Monate (inkl. Amortisation und Hausverkauf, nach Korrektur T1 bis T4) | Vermögen auf den Rappen gleich |

**Beurteilung der Tabelle.** Die Grundrechnung (Rendite, Sparen, PK-Beiträge, Kapitalbezug 50 %, Rente = Restkapital × 5 % / 12,
AHV, Ausgabenwechsel, Einmalbetrag) war plausibel und stimmt mit dem Programm überein. Vier Fehler der Tabelle sind korrigiert
(Zellformeln und berechnete Werte in allen drei Blättern):

| Nr. | Befund | Korrektur |
|---|---|---|
| T1 | Das Vermögen wurde bei 0 abgeschnitten (`E = IF(F<=0;0;F)`): bei der Amortisation von 500'000 mit 50'623 Vermögen verschwanden 454'029 Schuld; der Hausverkauf brachte nur das Eigenkapital. | `E = F` (Schuld bleibt sichtbar); der Verkauf nach der Amortisation bringt den vollen Marktwert (`T2 + S2`), vor der Amortisation das Eigenkapital. |
| T2 | Dasselbe Haus wurde ein zweites Mal verkauft (Zeile 454, +1'000'000, Miete +650), weil das Flag «Sold» nur bei Gewinn + Amortisation gesetzt wurde; Sonderformel `T+S` nur in Zeile 348. | `Sold` wird bei jedem Verkauf gesetzt; nach dem Verkauf entfällt die Amortisation (`S`); einheitliche Formel in allen Zeilen. |
| T3 | Rendite ab Zeile 153 hart halbiert, Ausgaben nach AHV-Beginn (8350) hart in Zeile 186, Verkaufsschwelle 50'000 in der Formel. | Neue Eingaben `V2` (Faktor Rendite, 0.5), `W2` (Ausgaben ab AHV, 8350), `X2` (Verkaufsschwelle, 50'000); Ereignis «Derisk» in Spalte A (Zeile 153). |
| T4 | Rente = Kapital × `M2` × `N2` / 12 mit dem Kapitalbezugsanteil `M2` als Rentenanteil (nur bei 50 % richtig). | Rente = Kapital × (1 − `M2`) × `N2` / 12. |

Nicht geändert (bewusste Vereinfachungen der Tabelle): PK-Zins und Teuerung nur im Januar (Blatt 2 hat 10 % PK-Zins als Testwert,
BVG-Minimum 1.25 %), keine Steuern, keine Hypothekarzinsen (in den 10'000 enthalten), Alter in Spalte C um einen Monat zu tief.
Die korrigierte Datei wurde mit LibreOffice (headless) neu berechnet: alle rund 6'600 Zahlenzellen je Blatt stimmen mit den in
der Datei gespeicherten Werten überein, keine Fehlerwerte. (Ein erster Wurf ohne die ursprünglichen Namensraum-Deklarationen
zeigte in LibreOffice überall «Err:5xx»; behoben.)

## Golden Masters

Zehn feste Planungen (Einzelperson, Ehepaar, PK-Rente, PK-Kapitalbezug, Immobilie mit Hypothek, historisch ab 1973,
Bootstrap, parametrisch, Szenarien, Notverkauf). Gespeichert sind Kennzahlen, Ereignisse, Perzentile bzw. Szenario-
Ergebnisse und die Monatswerte jedes Januars und Julis sowie des letzten Monats. Zahlen gelten mit 1e-9 relativer
Toleranz. Der Test schlägt bei jeder Abweichung fehl und nennt die Stelle. Eine gewollte Änderung der Rechenregeln
wird bewusst übernommen mit `GOLDEN_UPDATE=1 cargo nextest run -p vivaplan-engine -E 'test(/^golden::/)'`; danach die Änderung der JSON-Dateien in
Git prüfen. Die Ergebnisse sind über mehrere Läufe stabil (parallele Berechnung ändert nichts).

## Messungen (Release-Build, Apple-Rechner des Entwicklers)

| Fall | Zeit |
|---|---|
| parametrisch, 30 Jahre, 1'000 / 10'000 / 50'000 Pfade | 0.07 s / 0.34 s / 1.8 s |
| Bootstrap, 30 Jahre, 1'000 / 10'000 / 50'000 Pfade | 0.01 s / 0.12 s / 0.7 s |
| Stresstest: Ehepaar, 4 Objekte mit je 2 Tranchen, 100 Jahre, 40'000 Pfade (#139: unter der Grenze von 50 Mio. Pfad-Monaten; mit 50'000 Pfaden brach er an der Speichergrenze der Tests ab) | 8.1 s, Bilanzen der Perzentil-Läufe stimmen; an der Grenze (41'666 Pfade) höchstens 1'337 MB (18.11.2026). Läuft nächtlich im Job `release-golden` |

CPU-Auslastung wurde nicht separat erfasst (rayon nutzt alle Kerne; `time`: 101 s Benutzerzeit auf 37 s Wanduhr
inklusive Kompilieren).

## Prüfung von Hand: Agent Skill mit Claude (#208)

Vor einer Version, nach Änderungen an `vivaplan-skill/SKILL.md` oder an der Kommandozeile. Paket mit
`npx tauri build --config tauri.cli.conf.json` gebaut und installiert.

1. Skill einrichten wie in der Hilfe (Referenz, «Anleitung für KI-Assistenten»): Ordner `vivaplan-skill` aus dem Paket
   nach `~/.claude/skills/` kopieren, neue Sitzung in Claude Code oder Claude Desktop (Bereich «Code») starten.
2. Beispielplanung in der App öffnen und als Datei speichern (zum Beispiel `~/Documents/beispiel.vplan`).
3. In einem leeren Ordner auf Deutsch fragen: «Reicht mein Vermögen bis 95? Meine Planung ist
   ~/Documents/beispiel.vplan».
4. Erwartet:
   - Claude nutzt die Skill (sichtbar im Verlauf) und weist vor dem ersten Lesen der Planung darauf hin, dass die Daten
     an den Anbieter der KI gehen.
   - Claude ruft `vivaplan run … --json` mit dem vollen Pfad des Programms auf, mit oder ohne `--metric`, und
     rechnet nichts selbst.
   - Die Antwort ist deutsch, nennt nominal oder real und die Methode (ein fester Pfad oder viele Pfade); bei einem
     festen Pfad sagt sie, dass das keine Wahrscheinlichkeit ist, und bietet eine Monte-Carlo-Rechnung an.
   - Die Antwort endet mit dem Haftungsausschluss; die Planungsdatei ist unverändert (`vivaplan set` schreibt eine
     neue Datei).
5. Ergebnis mit Datum, Version und Modell unter «Messungen» oder im Issue festhalten.

## Prüfung von Hand: MCP-Server mit LM Studio (#210)

Vor einer Version und nach Änderungen an `crates/cli/src/mcp/`. Prüft, dass der Server auch mit einem MCP-Programm
läuft, das nicht von Anthropic ist (AK8). Paket gebaut und installiert, LM Studio mit einem lokalen Modell, das
Werkzeuge aufrufen kann (zum Beispiel Qwen 3 oder Llama 3.1 Instruct, in LM Studio als «Tool Use» gekennzeichnet).

1. In LM Studio unter «Program → Install → Edit mcp.json» den Eintrag aus der Hilfe (Referenz, «KI-Programme
   verbinden») eintragen, mit dem vollen Pfad des Programms und `"args": ["mcp"]`; speichern.
2. Erwartet: LM Studio zeigt den Server `vivaplan` mit fünfzehn Werkzeugen (`describe`, `plan_open`, `plan_get`,
   `plan_schema`, `plan_validate`, `result_list`, `result_get`, `library_list`, `simulation_run`, `evaluate`,
   `plan_set`, `plan_changes`, `plan_undo`, `plan_apply`, `plan_save`), ohne Fehlermeldung.
3. Beispielplanung in der App als Datei speichern (zum Beispiel `~/Documents/beispiel.vplan`), in LM Studio ein neues
   Gespräch beginnen und fragen: «Öffne ~/Documents/beispiel.vplan mit VivaPlan und sag mir, ob die Planung Fehler
   hat.» (bei Bedarf den vollen Pfad nennen).
4. Erwartet:
   - LM Studio fragt vor jedem Werkzeugaufruf nach der Erlaubnis; nach dem Erlauben ruft das Modell `plan_open` und
     `plan_validate` auf.
   - Die Antwort von `plan_open` nennt nur den Dateinamen, die Person als «Person 1» und enthält den Hinweis «your AI
     and is processed by its provider» und den vollen Haftungsausschluss.
   - Das Modell meldet keine Fehler (Beispielplanung) und gibt die Zahl der Hinweise wieder.
5. Fragen: «Welche Zeitreihen gibt es in der Datenbibliothek?» – Erwartet: Aufruf von `library_list`, Liste der Dateien.
   Dann (#212): «Setz den Aktienanteil auf 40 % und speichere die Planung.» – Erwartet: `plan_set` mit
   `wealth.stockShare` 0.4, alter und neuer Wert; `plan_save` schreibt `beispiel (AI).vplan` neben die Datei,
   `beispiel.vplan` bleibt unverändert. «Ersetze die Originaldatei.» – Erwartet: Unterstützt LM Studio Rückfragen
   (Elicitation), erscheint eine Frage mit der Liste der Änderungen, und erst nach «Ja» entsteht `beispiel.vplan.bak`;
   sonst lehnt der Server ab und nennt den Ausweg über eine neue Datei.
6. In LM Studio den Server ausschalten: Der Prozess `vivaplan mcp` endet (Aktivitätsanzeige).
7. Ergebnis mit Datum, Version, LM-Studio-Version und Modell unter «Messungen» oder im Issue festhalten.

## Prüfung von Hand: Mit KI verbinden (#213)

Vor einer Version und nach Änderungen am Dialog *Hilfe → Mit KI verbinden …*, an `tools/mcpb.py` oder an der
Kommandozeile. Paket gebaut (`pakete.yml` oder `npx tauri build --config tauri.cli.conf.json`, dann
`python3 tools/mcpb.py build --binary target/release/bundle/macos/VivaPlan.app/Contents/MacOS/vivaplan --out
target/release/bundle/mcpb`), `VivaPlan.app` nach `/Applications` kopiert. Beispielplanung als
`~/Documents/beispiel.vplan` gespeichert, eine Kopie als `~/Desktop/aussen.vplan` (#263).

**Für alle Programme gleich**

1. In der App *Hilfe → Mit KI verbinden …* öffnen. Erwartet: oben der Pfad
   `/Applications/VivaPlan.app/Contents/MacOS/vivaplan` (der Ort der installierten App, nicht der Standardpfad), darunter
   Claude Desktop, Claude Code, ChatGPT Desktop und LM Studio, je mit Ort der Einrichtung und Zeile zum Kopieren.
2. «Kopieren» drücken und in einem Texteditor einfügen: Erwartet ist genau die angezeigte Zeile; der Knopf zeigt kurz
   «Kopiert ✓».
3. AK4: Vor und nach dem Öffnen des Dialogs `ls -l ~/Library/Application\ Support/Claude ~/.codex ~/.lmstudio`
   vergleichen: keine Datei geändert. Die App öffnet keine Netzwerkverbindung (Little Snitch oder `nettop -p vivaplan-app`).
4. Nach dem Einrichten in jedem Programm fragen: «Öffne ~/Documents/beispiel.vplan mit VivaPlan und sag mir, ob die
   Planung Fehler hat.» Erwartet: Aufruf von `plan_open` und `plan_validate`, Person als «Person 1», keine Fehler.
   #263: Planordner im Dialog ist `~/Documents` (oder dorthin geändert). Dann fragen: «Öffne ~/Desktop/aussen.vplan.»
   Erwartet: Fehler «outside the plan folder», die KI bittet, die Planung in den Planordner zu legen. «Speichere die
   Planung als Kopie.» Erwartet: neue Datei `beispiel (AI).vplan` in `~/Documents`.
5. #263/AK4, AK5: Im Dialog «Ordner wählen …» drücken: Erwartet der Ordnerdialog des Systems am angezeigten Ordner; der
   gewählte Ordner steht danach in allen vier Zeilen hinter `--allow-dir`. «Nur lesen» einschalten: alle Zeilen enden
   auf `--read-only`; nach dem Einrichten damit bietet die KI kein Speichern an (`plan_save` fehlt in der Werkzeugliste).
6. Ergebnis mit Datum, Version, Programmversion und Modell im Issue festhalten.

**Claude Desktop, Ein-Klick-Paket (AK1)**

1. Ist VivaPlan schon als Server in `claude_desktop_config.json` eingetragen, den Eintrag vorher entfernen.
2. `VivaPlan_<Version>_aarch64.mcpb` im Finder doppelklicken. Erwartet: Claude Desktop öffnet die Installationsansicht
   mit Name «VivaPlan», Version, Symbol, Beschreibung und den fünfzehn Werkzeugen; «Installieren» gelingt ohne
   Fehlermeldung (auch keine von Gatekeeper zum Programm `server/vivaplan`). #263/AK1: Vor dem Einschalten fragt
   Claude Desktop nach «Plan folder» (Pflicht, Vorschlag Dokumente-Ordner) und «Read only» (aus). #270/AK8: dazu das
   Feld «Beta token» (freiwillig, verdeckt); `ps` zeigt am Ende `--beta-token=<eingegebenes Token>`, bei leerem Feld
   `--beta-token=`. Prüfen, ob Claude
   Desktop beide Werte in die Argumente einsetzt: `ps -ax -o args | grep '[v]ivaplan mcp'` im Terminal zeigt
   `mcp --allow-dir <ordner> --read-only=false`. Mit
   «Read only» eingeschaltet `--read-only=true` und kein `plan_save` in der Werkzeugliste.
3. Unter *Settings → Extensions* ist VivaPlan eingeschaltet; Prüfung 4 oben gelingt. In der Aktivitätsanzeige läuft
   `vivaplan mcp` aus dem Erweiterungsordner von Claude Desktop.
4. Erweiterung deaktivieren: Der Prozess `vivaplan mcp` endet.

**Claude Desktop, von Hand**

1. Settings → Developer → Edit Config, die Zeile aus dem Dialog einfügen (oder bei vorhandenen Einträgen nur
   `"vivaplan": {…}` ergänzen), Claude Desktop neu starten.
2. Erwartet: VivaPlan erscheint unter den Werkzeugen (Schieberegler-Symbol) mit fünfzehn Werkzeugen; Prüfung 4 gelingt.

**Claude Code**

1. Die Zeile `claude mcp add --scope user vivaplan -- … mcp` im Terminal ausführen.
2. Erwartet: `claude mcp list` zeigt `vivaplan` als verbunden; in einer neuen Sitzung gelingt Prüfung 4.

**ChatGPT Desktop (über die Konfiguration von Codex)**

1. Die drei Zeilen aus dem Dialog in `~/.codex/config.toml` einfügen (Datei anlegen, falls sie fehlt), die
   ChatGPT-Desktop-App neu starten.
2. Erwartet: VivaPlan steht in der App als verbundener MCP-Server; Prüfung 4 gelingt. Falls die App die Konfiguration
   von Codex nicht liest (Annahme aus `local/analysen/ki-anbindung.md`, Stand 11/2026), dies im Issue festhalten:
   Dann ist die Anleitung im Dialog zu ändern.

**LM Studio**

1. Program → Install → Edit mcp.json, die Zeile aus dem Dialog einfügen, speichern.
2. Erwartet: wie in der Prüfung «MCP-Server mit LM Studio (#210)», Schritte 2 bis 6.

## Prüfung von Hand: Neue Planung über die KI (#269)

Voraussetzung: VivaPlan ist in Claude Desktop eingerichtet (Prüfung «Mit KI verbinden (#213)»), mit einem Planordner.

1. In Claude Desktop ein neues Gespräch beginnen und fragen: «Rechne meine Pension mit VivaPlan.» (ohne Datei zu
   nennen).
2. Erwartet (AK6): Claude ruft `plan_open` mit `new: true` auf, nicht mit `example: true`; die Antwort zeigt
   `"file": "(new plan)"` und eine Liste `missing`. Claude fragt nach den fehlenden Angaben (Geburtsdatum, Einkommen,
   Ausgaben, Vermögen, Pensionskasse, AHV) und rechnet nicht mit Werten des Beispielplans (keine Partnerin, keine
   Liegenschaften, keine Säule 3a, die nicht genannt wurden).
3. Erfundene Antworten geben (z. B. Geburt 06.1970, Einkommen 90'000, Ausgaben 70'000, Cash 300'000, PK 400'000,
   AHV 2'200 pro Monat). Erwartet: `plan_set` mit diesen Werten, danach `simulation_run`.
4. Bitten: «Speichere die Planung als Meine Pension.vplan.» Erwartet: `plan_save` schreibt eine neue Datei in den
   Planordner; in der App geöffnet zeigt sie die eingegebenen Werte und sonst die Werte von *Datei → Neu*.
5. Gegenprobe: «Zeig mir VivaPlan am Beispiel.» Erwartet: `plan_open` mit `example: true`.
6. Ergebnis mit Datum, Version, Claude-Desktop-Version und Modell im Issue festhalten.

## Prüfung von Hand: Beta-Token (#270)

Voraussetzung: echter öffentlicher Schlüssel in `crates/engine/src/beta.rs` (`docs/ENTWICKLUNG.md`, «Beta-Token»), ein
installiertes Paket dieser Version und zwei Tokens von `beta-token issue`: eines mit Frist in der Zukunft, eines mit
abgelaufener Frist. Vorher `~/Library/Application Support/ch.vivaplan-app.desktop/beta-aktivierung.json` wegsichern.

1. App starten (AK4): Nur der Dialog «Beta-Token eingeben» erscheint; Menü *Datei → Neu/Öffnen*, Doppelklick auf eine
   `.vplan` und die Tastenkürzel bewirken nichts.
2. Abgelaufenes Token eingeben (AK2): Meldung «… nur bis TT.MM.JJJJ einlösen. Bitte fragen Sie nach einem neuen
   Token.». Ein Zeichen im gültigen Token ändern (AK3): «nicht gültig».
3. Gültiges Token eingeben (AK1): Die App startet; nach Beenden und Neustart kein Dialog. *Hilfe → Über VivaPlan*
   zeigt Kürzel und Tag (AK10).
4. Systemuhr nach die Frist des Tokens stellen, App neu starten (AK5): kein Dialog, kein Hinweis. Uhr zurückstellen.
5. Update (AK6): Ein Paket mit höherer Version über die installierte App installieren und starten: kein Dialog.
6. Claude Desktop (AK7, AK8): Mit aktivierter App das `.mcpb` mit leerem Feld «Beta token» installieren; Prüfung 4 von
   «Mit KI verbinden» gelingt. Dann `beta-aktivierung.json` entfernen, Erweiterung neu installieren mit gültigem Token:
   gelingt; danach startet die App ohne Dialog. Mit leerem Feld und ohne Aktivierung (AK9): Jede Frage an VivaPlan
   endet mit «VivaPlan is not activated yet …».
7. Ergebnis mit Datum und Version im Issue festhalten; die gesicherte Aktivierungsdatei zurücklegen.

## Offene Punkte

1. ~~**Cashflow-Invariante mit Immobilien, Pensionskasse und 3a**~~ – erledigt mit #133: Der Monatsdatensatz führt intern
   ein Hauptbuch (`sim::Ledger`: Einkommen, Einmal- und Sparbeträge, Lebenshaltung, Wohnkosten, Amortisation, Steuern,
   Rendite, PK-Beiträge und -Zins, 3a-Zins, Wertänderung der Liegenschaften, Umwandlung in die PK-Rente). Die Veränderung
   des Gesamtvermögens muss jeden Monat genau dieser Summe entsprechen (±0.01 CHF), geprüft in `proptest_plans.rs`,
   `qa_matrix.rs` (`check_books`) und für die Golden-Planungen. Die Gleichung «Total = Summe der Teile» allein war keine
   Prüfung: `sim.rs` rechnet das Total genau so.
2. **Oberfläche:** Die Dialoge der Desktop-App (Datei öffnen/speichern über das Betriebssystem, Menüleiste) lassen sich im
   Browser nicht prüfen; dort gelten Dateiauswahl und Download. Die Darstellung der Grafiken wird nur auf Vorhandensein
   und Tooltips geprüft, nicht pixelgenau.
3. **Statistik bei 50'000 Pfaden** wird nur über die Laufzeit-/Absturzprüfung abgedeckt, die Stabilitätsprüfung
   vergleicht 1'000 mit 10'000 Pfaden.
4. Das Geburtsdatum «in der Zukunft» wird relativ zum Simulationsstart geprüft, nicht zum heutigen Datum, damit
   der Rechenkern ohne Uhr deterministisch bleibt.
