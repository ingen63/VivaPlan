# #77 Konzept: VivaPlan in Claude nutzen (MCP)

Issue #77. Dieses Dokument ist das Konzept. Umgesetzt wird erst nach der Abnahme, über die Folge-Issues in
Abschnitt 12. Stand: Entwurf; die offenen Entscheide stehen in Abschnitt 13.

## 1 Ziel und Abgrenzung

VivaPlan soll sich aus Claude heraus benutzen lassen: Claude öffnet eine Planung, prüft sie, ändert Werte, rechnet
Vorschau, Simulation und Optimierung und zeigt die Ergebnisse. Die Verbindung läuft über das Model Context Protocol
(MCP).

Nicht Teil des Konzepts:

- Berechnungen ausserhalb des Rechenkerns. Der Server rechnet nichts selbst (Abschnitt 4).
- Ein Dienst im Internet. Der Server läuft nur lokal und nur, solange Claude ihn gestartet hat.
- Import von Zeitreihen (Shiller, SNB) und der Import alter `config.json`. Das bleibt in der Desktop-App und in der
  Kommandozeile.

## 2 Ausgangslage

Was der Rechenkern schon bietet (`crates/engine/src/api.rs`):

| Funktion | Zweck |
|---|---|
| `read_plan`, `write_plan` | Projektdatei lesen und schreiben |
| `upgrade_plan` | Ältere Planung auf den aktuellen Stand bringen |
| `validate` | Prüfmeldungen mit Abschnitt und Feldpfad |
| `preview` | Ein Lauf mit konstanten Renditen |
| `run`, `rerun_paths` | Simulation, Variationen, einzelne Läufe nachrechnen |
| `opt_params`, `optimize` | Parameter und Optimierung der Entnahmestrategie |
| `bench` | Rechenzeit pro Lauf, Grundlage der Dauerschätzung |
| `library_list`, `series_info` | Datenbibliothek |

Dazu `engine::schema_json()` (Schema der Projektdatei) und `engine::progress` (Fortschritt und Abbruch). Der
Entwicklungsserver (`crates/cli/src/devserver.rs`, Funktion `call`) verteilt bereits Befehlsnamen auf diese Funktionen.

Was fehlt, und was die Folge-Issues schliessen müssen:

| Nr. | Lücke | Beleg |
|---|---|---|
| L1 | Die Antworten sind zu gross für Claude. | Beispielplanung: Vorschau 342 KB (472 Monate × 29 Felder), Simulation mit 2000 Pfaden 2,1 MB. Die Kennzahlen darin sind 0,5 bis 2 KB gross. |
| L2 | Jahreswerte, Ergebnis in Worten, Vergleich und die Liste der Ergebnisse liegen in der Oberfläche. Seit #209 liefert `api` Jahreswerte (`yearly`), Kurzfassung mit Befund (`digest`) und Vergleich (`compare`); offen ist die Liste. | `yearly`, `verdictHtml`, `compareRuns` in `crates/app/ui/app.js`; Liste in `runs.js` (IndexedDB) |
| L3 | Der Rechenkern kann keine einzelnen Werte ändern. | `scenarios::apply` kennt sechs Parameter; der Editor ändert das JSON in JavaScript. |
| L4 | `api` ist zustandslos; es gibt kein Änderungsprotokoll und kein Rückgängig ausserhalb der Oberfläche. | `History` in `app.js` |
| L5 | Grenzen aus den Einstellungen und der Ordner der Datenbibliothek kommen heute von der Oberfläche. | `Engine.limits` in `api.js`; `library_dir` der Kommandozeile zeigt auf einen Pfad des Build-Rechners. |
| L6 | Die Pakete enthalten die Kommandozeile nicht. | `.github/workflows/pakete.yml`, `tauri.conf.json` |

## 3 Varianten

### (a) Lokaler MCP-Server über stdio

Ein Befehl `vivaplan mcp` im Paket `vivaplan-cli`. Claude startet das Programm und spricht über Standardeingabe und
-ausgabe mit ihm. Der Server stellt Werkzeuge bereit (Abschnitt 5), hält die geöffneten Planungen im Speicher und ruft
für jede Rechnung `engine::api` auf.

### (b) Die Oberfläche als interaktive Ansicht in Claude (MCP Apps)

MCP Apps ist eine Erweiterung von MCP (SEP-1865, Fassung 2026-01-26): Ein Werkzeug verweist über `_meta.ui.resourceUri`
auf eine Ressource `ui://…`, die eine HTML-Seite ist. Der Host zeigt sie in einem abgeschotteten Rahmen im Gespräch. Die
Seite spricht über `postMessage` mit dem Host und kann darüber Werkzeuge des Servers aufrufen.

Für VivaPlan hiesse das: `crates/app/ui` wird zu einer einzigen HTML-Datei gebündelt, und `Engine.call` in `api.js`
bekommt neben Tauri und dem Entwicklungsserver einen dritten Weg (Werkzeugaufruf über den Host). Variante (b) braucht
den Server aus (a) ohnehin.

### (c) Die laufende Desktop-App steuern

Die Desktop-App nimmt selbst Befehle entgegen; `vivaplan mcp` wäre nur eine Brücke zur offenen App. Änderungen von
Claude erschienen direkt im Editor und liessen sich dort rückgängig machen.

### Vergleich

| Kriterium | (a) stdio-Server | (b) MCP App | (c) App steuern |
|---|---|---|---|
| Läuft in Claude Desktop | ja | ja | ja |
| Läuft in Claude Code | ja | nicht gesichert (Terminal zeigt keine Ansichten) | ja |
| Aufwand | klein bis mittel | gross | mittel |
| Änderungen sichtbar | als Protokoll im Gespräch | in der Ansicht | im Editor der App |
| Rückgängig | Stapel im Server | `History` der Oberfläche | `History` der Oberfläche |
| Daten an Claude | nur Kurzfassungen (Abschnitt 6) | Ansicht holt volle Ergebnisse; ob sie den Rechner verlassen, hängt vom Host ab | wie (a) |
| Netzwerkregel | eingehalten (stdio) | eingehalten (stdio) | verletzt: die App müsste lokal lauschen |
| Grösste Hürde | Lücken L1 bis L3 | 13 von 27 Befehlen gibt es nur in Tauri (Dateidialoge, Menü, Fenstertitel, Beenden); die Vorschau schickt bei jeder Eingabe 342 KB durch den Host; Speicher der Seite (IndexedDB) im Rahmen nicht gesichert | Regel «keine Netzwerkzugriffe» und die Tauri-Berechtigungen müssten gelockert werden |

### Empfehlung

**Variante (a) als Grundlage, danach (b) in kleiner Form.**

- (a) deckt alle verlangten Funktionen ab, läuft in beiden Claude-Programmen und hält die Netzwerkregel ohne Ausnahme
  ein. Die Lücken L1 bis L3 sind Arbeit am Rechenkern, die auch der Oberfläche nützt (weniger Logik in JavaScript).
- (b) folgt als zweite Stufe mit **wenigen eigenen Ansichten** (Vermögensverlauf mit Bandbreite, Vergleich von
  Ergebnissen), gebaut aus `charts.js`. Den ganzen Editor in Claude zu zeigen ist erst sinnvoll, wenn die Ansichten sich
  bewährt haben: Der Aufwand ist gross, und jede Eingabe würde eine volle Vorschau durch den Host schicken.
- (c) wird nicht verfolgt, weil die App dafür eine Schnittstelle nach aussen öffnen müsste.

Das weicht vom Ziel im Issue ab («die ganze App als Teil von Claude»). Entscheid E1 in Abschnitt 13.

## 4 Aufbau

```
Claude (Desktop oder Code)
   │  stdio, JSON-RPC, eine Zeile pro Meldung
   ▼
vivaplan mcp                     crates/cli/src/mcp/
   ├─ Protokoll                  initialize, tools/list, tools/call, Fortschritt, Abbruch
   ├─ Sitzung                    geöffnete Planungen, Änderungsprotokoll, Rückgängig, Ergebnisse
   └─ Werkzeuge                  dünne Hülle: Eingaben prüfen, engine::api aufrufen, Antwort kürzen
   ▼
vivaplan-engine (api)            alle Berechnungen, Kurzfassungen, Vergleich, Ändern per Feldpfad
```

**Trennung von Rechenkern und Oberfläche.** Der Server enthält keine Rechenlogik. Was heute in `app.js` gerechnet wird
und was der Server braucht, wandert in `engine::api`, und die Oberfläche ruft es von dort auf:

| Neu in `api` | Ersetzt in der Oberfläche |
|---|---|
| `yearly(result)`: Summen pro Kalenderjahr | `yearly` in `app.js` |
| `digest(run)`: Kennzahlen, Ergebnis in Worten (ohne HTML), Hinweise | Zahlen und Sätze von `verdictHtml`; die Oberfläche setzt nur noch die Verweise in die Hilfe |
| `compare(a, b, …)`: Kennzahlen nebeneinander, Unterschiede der Planungen | Rechenteil von `compareRuns` und `flat` |
| `set_value(plan, pfad, wert)`, `diff(plan_a, plan_b)` | keine Entsprechung; der Editor ändert weiter direkt |

**Protokoll von Hand.** Der Server braucht nur Werkzeuge, Fortschritt, Abbruch und (wo vorhanden) Rückfragen. Das
Protokoll wird wie der Entwicklungsserver ohne zusätzliche Abhängigkeit geschrieben; die Kommandozeile bleibt bei `serde`
und `serde_json`. Damit ist auch prüfbar, dass das Programm keine Netzwerkbibliothek enthält. Entscheid E4.

**Eine Berechnung aufs Mal.** `engine::progress` gilt für den ganzen Prozess. Der Server liest die Eingabe in einem
eigenen Thread und rechnet in einem zweiten; ein zweiter Rechenaufruf während einer laufenden Berechnung wird mit einer
klaren Meldung abgelehnt.

**Gleiche Ergebnisse wie in der App.** Der Server liest die Grenzen aus `einstellungen.json` im Datenordner der App und
gibt sie wie die Oberfläche als `limits` mit. Die Datenbibliothek ist der Ordner «Zeitreihen» desselben Datenordners;
fehlt er, legt `library::seed` ihn an. Beides lässt sich mit `--data-dir <ordner>` übersteuern (für Tests). Die
Auflösung des Datenordners pro Betriebssystem kommt in den Rechenkern oder die Kommandozeile, damit sie testbar ist.

## 5 Werkzeuge

**Stand der Umsetzung (#210, Entscheid vom 23.11.2026 in #210):** Hauptweg sind `evaluate` (#216) und `describe`
(#219). `plan_preview`, `optimize_run` und `result_compare` entfallen; `simulation_run` bleibt als einfacher Einstieg
(intern ein Experiment ohne Variation). #210 setzt die lesenden Werkzeuge um (`plan_open`, `plan_get`, `plan_schema`,
`plan_validate`, `result_get`, `result_list`, `library_list`, `describe`), #211 die rechnenden, #212 Ändern und
Speichern. Die Daten stehen als kompaktes JSON im Text (ohne `structuredContent`); Namen werden ersetzt wie in
Abschnitt 6. Die Tabelle unten ist der ursprüngliche Entwurf.

Jede geöffnete Planung bekommt eine Kennung (`plan`, zum Beispiel `p1`), jedes Ergebnis eine Kennung (`result`, zum
Beispiel `r3`). Claude schickt nie die ganze Planung; es arbeitet mit Kennungen und Feldpfaden. Alle Werkzeuge liefern
Text und dieselben Daten strukturiert. Feldpfade haben die Form der Prüfmeldungen (`household.persons.0.name`).

| Name | Zweck | Eingaben | Ausgaben |
|---|---|---|---|
| `plan_open` | Planung öffnen und auf den aktuellen Stand bringen | `path` (Datei `.vplan` oder `.rplan`), oder `example: true` | `plan`, Name, Zeitraum, Zusammenfassung der Abschnitte, Hinweise von `upgrade_plan`, Zahl der Prüfmeldungen |
| `plan_get` | Teile der Planung lesen | `plan`, `section` oder `path` | Werte des Abschnitts oder Feldes |
| `plan_schema` | Aufbau der Projektdatei nachschlagen | `section` (optional) | Ausschnitt aus `schema_json()` mit Beschreibungen |
| `plan_validate` | Planung prüfen | `plan` | Prüfmeldungen mit Schwere, Abschnitt, Feldpfad |
| `plan_set` | Werte ändern | `plan`, `changes`: Liste aus `{path, value}` | je Änderung alter und neuer Wert, Prüfmeldungen danach; bei Fehler in der Form wird keine der Änderungen übernommen |
| `plan_changes` | Zeigen, was gegenüber der Datei geändert ist | `plan` | Liste aus Feldpfad, Wert in der Datei, aktueller Wert |
| `plan_undo` | Letzte Änderung zurücknehmen oder alle verwerfen | `plan`, `all` (optional) | zurückgenommene Änderungen, verbleibende Zahl |
| `plan_preview` | Ein Lauf mit konstanten Renditen | `plan` | `result`, Kennzahlen, Ergebnis in Worten, Grundlage der Vorschau, Prüfmeldungen |
| `simulation_estimate` | Dauer vor dem Start schätzen | `plan` | Zahl der Läufe, geschätzte Dauer |
| `simulation_run` | Simulation gemäss Renditequelle, mit Variationen | `plan`, `paths` (optional, überschreibt die Pfadzahl nur für diesen Lauf) | `result`, Erfolgsquote, Perzentile des Endvermögens, früheste Erschöpfung, Ergebnis in Worten, Variationen als Tabelle, Rechenzeit |
| `optimize_run` | Entnahmestrategie optimieren | `plan`, `request` (Felder wie `optimize::Request`; leer = gespeicherte Einstellungen) | `result`, Referenz, Rangliste der Strategien mit Kennzahlen und Parametern |
| `optimize_apply` | Ein Optimum in die Planung übernehmen | `plan`, `result`, `strategy` | übernommene Änderungen (wie `plan_set`) |
| `result_get` | Details eines Ergebnisses abrufen | `result`, `view`: `summary`, `yearly`, `events`, `bands`, `variations`; `percentile`, `from`, `to` (optional) | gewählte Sicht, auf den Zeitraum begrenzt |
| `result_list` | Ergebnisse der Sitzung auflisten | keine | Kennung, Planung, Art, Zeitpunkt, wichtigste Kennzahl |
| `result_compare` | Ergebnisse vergleichen | `results` (2 bis 6) | Kennzahlen nebeneinander, Unterschiede der Planungen |
| `plan_save` | Planung speichern | `plan`, `path` (optional), `overwrite` (optional) | geschriebener Pfad, Pfad der Sicherungskopie; Regeln in Abschnitt 7 |
| `library_list` | Zeitreihen der Datenbibliothek | keine | Datei, Währung, Zeitraum, Durchschnitte |

Zuordnung zu den verlangten Funktionen: Planung öffnen (`plan_open`), prüfen (`plan_validate`), Werte ändern
(`plan_set`, `plan_undo`, `optimize_apply`), Vorschau (`plan_preview`), Simulation (`simulation_run`), Optimierung
(`optimize_run`), Ergebnisse abrufen und vergleichen (`result_get`, `result_list`, `result_compare`), Planung speichern
(`plan_save`).

Regeln für alle Werkzeuge:

- **Grösse:** Keine Antwort über 20'000 Zeichen. `result_get` mit `yearly` liefert eine Zeile pro Jahr; Monatswerte gibt
  es nur für einen Zeitraum von höchstens 24 Monaten. Was nicht passt, wird gekürzt, mit dem Hinweis, wie der Rest
  abzurufen ist.
- **Lesend oder ändernd:** Die Werkzeuge tragen die Kennzeichen des Protokolls (`readOnlyHint`, `destructiveHint`).
  Nur `plan_save` schreibt auf die Platte.
- **Fehler:** Fehler der Planung kommen als Prüfmeldungen zurück, nicht als Protokollfehler, damit Claude sie beheben
  kann.
- **Haftungsausschluss:** Jede Antwort mit Ergebnissen endet mit «Modellrechnung ohne Gewähr. Keine Finanz- oder
  Steuerberatung.» (Abschnitt 8).

Die Ergebnisse der Sitzung liegen nur im Speicher des Servers und verschwinden mit ihm. Die Liste der Ergebnisse der
Desktop-App ist davon getrennt.

## 6 Daten und Datenschutz

**Was sich nicht ändert.** Die Desktop-App und der Rechenkern greifen weiterhin nie aufs Netzwerk zu. Auch
`vivaplan mcp` öffnet keine Verbindung: Es liest und schreibt nur Standardeingabe, Standardausgabe und Dateien.

**Was neu ist.** Wer VivaPlan in Claude einrichtet, entscheidet bewusst, Daten an Claude zu geben. Alles, was ein
Werkzeug zurückgibt, und alles, was Claude einem Werkzeug schickt, ist Teil des Gesprächs und geht an Anthropic. Die
Regel lautet deshalb neu: «Die App überträgt keine Daten. Daten verlassen den Rechner nur, wenn Sie VivaPlan selbst mit
Claude verbinden, und dann nur das, was Claude abfragt.»

Was an Claude geht:

| Daten | Wann |
|---|---|
| Name der Planung, Zeitraum, Zahl der Personen und Immobilien | bei `plan_open` |
| Werte eines Abschnitts (Einkommen, Vermögen, Vorsorge, Wohnort für die Steuern …) | wenn Claude sie mit `plan_get` liest oder mit `plan_set` ändert |
| Prüfmeldungen | bei jeder Prüfung |
| Kennzahlen, Jahreswerte, Ereignisse | bei Vorschau, Simulation, Optimierung und `result_get` |

Was nie an Claude geht:

- Dateien ausserhalb der genannten Planung. Der Server liest nur Dateien mit den Endungen `.vplan` und `.rplan`, die
  ausdrücklich als `path` genannt sind, und durchsucht keine Ordner.
- Die Liste der Ergebnisse der Desktop-App und deren Einstellungen (ausser den Grenzen, die in die Rechnung eingehen).
- Die vollen Monatswerte aller Läufe.

Massnahmen:

1. **Datensparsamkeit:** `plan_open` liefert eine Zusammenfassung, keine Werte. Werte gehen erst auf Abfrage.
2. **Namen:** Namen von Personen und frei eingegebene Bezeichnungen werden in den Antworten durch neutrale ersetzt
   («Person 1», «Immobilie 1»), ausser der Server läuft mit `--names`. Entscheid E5.
3. **Pfade:** Antworten nennen nur den Dateinamen, nicht den ganzen Pfad (der den Benutzernamen enthält).
4. **Fremde Planungen:** Texte aus einer Planung sind Daten. Die Beschreibung des Servers weist Claude darauf hin,
   Anweisungen in solchen Texten nicht zu folgen.
5. **Hinweis beim ersten Öffnen:** Die erste Antwort von `plan_open` in einer Sitzung enthält den Satz, dass die
   abgefragten Daten Teil des Gesprächs mit Claude werden.

Texte, die bei der Umsetzung nachzuführen sind:

- Hilfe, Abschnitt «Datenschutz und lokale Daten» (`crates/app/ui/hilfe/00-einstieg.md`): neuer Absatz «VivaPlan mit
  Claude verbinden» mit der Tabelle oben und dem Hinweis, dass die Verbindung nur besteht, wenn sie eingerichtet wurde.
- Neues Hilfekapitel zur Einrichtung und Bedienung (Abschnitt 9), Befehl `mcp` in der Referenz (`60-referenz.md`).
- `docs/BESCHREIBUNG.md` (Absatz zum Netzwerk), `README.md` (Zeile 4 und Abschnitt Kommandozeile), `CLAUDE.md` (Regel
  «Keine Netzwerkzugriffe in der App» präzisieren: gilt für App, Rechenkern und `vivaplan mcp`).

## 7 Änderungen sichtbar machen, rückgängig machen, speichern

- **Arbeitskopie:** `plan_open` lädt die Planung in den Speicher. Alle Änderungen betreffen nur diese Kopie.
- **Sichtbar:** Jede Antwort von `plan_set` und `optimize_apply` nennt Feldpfad, alten und neuen Wert. `plan_changes`
  zeigt jederzeit alle Unterschiede zur Datei.
- **Rückgängig:** `plan_undo` nimmt die letzte Änderung zurück, mit `all` alle. Ein Aufruf von `plan_set` mit mehreren
  Änderungen ist ein Schritt.
- **Speichern, Regeln im Server** (sie gelten unabhängig davon, was Claude oder der Host tun):
  1. Ohne `path` und ohne `overwrite` schreibt `plan_save` eine **neue Datei** neben der geöffneten
     (`<name> (Claude).vplan`, bei Bedarf mit Zähler). Eine vorhandene Datei wird so nie berührt.
  2. Eine vorhandene Datei wird nur mit `overwrite: true` ersetzt. Davor fragt der Server über das Protokoll nach
     (Elicitation) und zeigt die Liste der Änderungen. Kann der Host keine Rückfrage zeigen, lehnt der Server ab und
     nennt Regel 1 als Ausweg.
  3. Vor dem Ersetzen entsteht eine Sicherungskopie `<name>.vplan.bak`. Geschrieben wird in eine Zwischendatei, die
     dann umbenannt wird.
  4. Wurde die Datei seit dem Öffnen von jemand anderem geändert (Änderungszeit oder Inhalt), wird nicht ersetzt.
  5. Alte `.rplan`-Dateien werden nie ersetzt (wie #44); gespeichert wird als `.vplan`.
  6. Geschrieben wird nur in den Ordner der geöffneten Planung oder in einen mit `--allow-dir` freigegebenen Ordner.
- **Desktop-App gleichzeitig offen:** Die App bemerkt Änderungen an der Datei nicht. Die Hilfe rät, eine Planung nicht
  gleichzeitig in der App und in Claude zu bearbeiten; Regel 1 und 4 verhindern den stillen Verlust auf der Seite des
  Servers. Eine Prüfung in der App vor dem Speichern wäre ein eigenes Issue.

## 8 Grenzen und Risiken

| Thema | Risiko | Massnahme |
|---|---|---|
| Rechenzeit | Meist kurz (10 µs pro Lauf, 2000 Pfade 0,1 s, Optimierung etwa 15 s), mit vielen Variationen oder Pfaden aber Minuten. | `simulation_estimate` vor dem Start; ab 60 s geschätzter Dauer (wie #21) startet die Rechnung nur mit `confirm_long: true`. |
| Fortschritt | Der Host bricht einen Aufruf ohne Lebenszeichen ab (Claude Code: 30 Minuten bei stdio). | Fortschrittsmeldungen aus `progress::snapshot()` etwa jede Sekunde. |
| Abbruch | Rechnung läuft weiter, obwohl der Benutzer abgebrochen hat. | Die Abbruchmeldung des Protokolls ruft `progress::cancel()`; endet die Eingabe, bricht der Server ab und beendet sich. |
| Grösse der Ergebnisse | Antwort sprengt die Grenze des Hosts (Claude Code: Warnung ab 10'000 Token, Grenze 25'000). | Obergrenze 20'000 Zeichen, Sichten in `result_get`, Test je Werkzeug. |
| Fehlbedienung durch Claude | Falsches Feld, falsche Einheit (Anteil 0.05 statt 5), erfundene Feldpfade. | Unbekannte Pfade werden abgelehnt; `plan_set` prüft sofort und zeigt alt und neu; `plan_schema` nennt Einheiten; Speichern nur nach den Regeln in Abschnitt 7. |
| Falsche Wiedergabe | Claude rundet, verwechselt nominal und real oder deutet Ergebnisse als Empfehlung. | Antworten nennen Einheit und «nominal» oder «real»; das Ergebnis in Worten kommt aus dem Rechenkern; Haftungsausschluss in jeder Antwort. |
| Haftung (#61, Issue #61, Abnahme ausstehend) | Der Benutzer hat den Haftungsausschluss der App nie gesehen, wenn er nur über Claude arbeitet. | Kurzfassung in der Beschreibung des Servers und in jeder Ergebnisantwort; voller Text über `plan_open` beim ersten Aufruf. Der Wortlaut folgt #61; die Umsetzung von Folge-Issue 3 setzt den abgenommenen Text voraus. |
| Abweichung von der App | Andere Grenzen oder Zeitreihen als in der App. | Gleiche Einstellungen und Datenbibliothek (Abschnitt 4); Test, der Server und `api` vergleicht. |
| Protokoll ändert sich | MCP und MCP Apps entwickeln sich schnell. | Kleiner Umfang (nur Werkzeuge), Protokollfassung wird ausgehandelt, Test mit festgehaltener Werkzeugliste. |
| Unsignierte Pakete | Das Betriebssystem blockiert ein Programm, das Claude startet. | Abschnitt 9; Signierung ist ein eigenes Thema. |

## 9 Installation und Einrichtung

**Auslieferung (Entscheid E3).** Empfohlen: Das Programm `vivaplan` wird dem App-Paket beigelegt (Tauri `externalBin`)
und zusätzlich einzeln je Betriebssystem in den Versionen angeboten. Die Desktop-App zeigt unter «Hilfe → Mit Claude
verbinden» den Pfad des Programms und die fertige Einrichtungszeile zum Kopieren. Die App richtet nichts selbst ein.

**Claude Code:**

```bash
claude mcp add --scope user vivaplan -- "<pfad>/vivaplan" mcp
```

**Claude Desktop:** Eintrag in `claude_desktop_config.json` (Einstellungen → Entwickler → Konfiguration bearbeiten):

```json
{
  "mcpServers": {
    "vivaplan": { "command": "<pfad>/vivaplan", "args": ["mcp"] }
  }
}
```

Als spätere Vereinfachung kommt ein Erweiterungspaket für Claude Desktop in Frage, das mit einem Klick installiert wird.

**Optionen von `vivaplan mcp`:** `--data-dir <ordner>` (Einstellungen und Datenbibliothek), `--allow-dir <ordner>`
(zusätzlicher Ordner zum Speichern), `--names` (Namen nicht ersetzen), `--read-only` (ohne `plan_save`).

**Entfernen:** `claude mcp remove vivaplan` beziehungsweise den Eintrag löschen. Danach besteht keine Verbindung mehr.

## 10 Tests

Der MCP-Server wird automatisch geprüft, ohne Claude und ohne Netzwerk:

| Ebene | Was | Wo |
|---|---|---|
| Rechenkern | Neue Funktionen in `api` (Jahreswerte, Kurzfassung, Vergleich, Ändern per Feldpfad): Einheitentests, dazu ein Vergleich mit den bisherigen Werten der Oberfläche an der Beispielplanung | `crates/engine/tests/` |
| Sitzung | Öffnen, Ändern, Rückgängig, Änderungsprotokoll, Speicherregeln 1 bis 6 als Einheitentests | `crates/cli/src/mcp/` |
| Protokoll | Das Programm `vivaplan mcp` starten, Meldungen zeilenweise schicken, Antworten prüfen (Muster wie der Test des Entwicklungsservers in `crates/cli/tests/cli.rs`): Anmeldung, Werkzeugliste, jeder Werkzeugaufruf, Fehlerfälle, defekte Eingaben | `crates/cli/tests/mcp.rs` |
| Werkzeugliste | Namen, Beschreibungen und Schemas als festgehaltene Datei; jede Änderung der Schnittstelle fällt auf | `crates/cli/tests/` |
| Grösse | Jedes Werkzeug bleibt an der Beispielplanung und an einer grossen Planung (viele Variationen, 50'000 Pfade) unter der Obergrenze | `crates/cli/tests/mcp.rs` |
| Gleichheit | Kennzahlen aus `simulation_run` stimmen mit `api::run` überein (keine doppelte Rechnung) | `crates/cli/tests/mcp.rs` |
| Abbruch | Lange Rechnung starten, Abbruch schicken, Antwort «Abgebrochen»; Eingabe schliessen beendet das Programm | `crates/cli/tests/mcp.rs` |
| Datenschutz | Antworten enthalten weder Namen noch ganze Pfade; ein Werkzeug liest keine Datei mit anderer Endung | `crates/cli/tests/mcp.rs` |
| Netzwerk | Die Abhängigkeiten von `vivaplan-cli` enthalten keine Netzwerkbibliothek (Prüfung von `cargo tree` im Workflow) | `.github/workflows/` |
| Robustheit | Fuzzing des Zeilenlesers und der Feldpfade | `fuzz/` |

Tests, die ein Abnahmekriterium belegen, nennen Issue und Kriterium wie gewohnt. Von Hand vor einer Version: einmal in
Claude Desktop und einmal in Claude Code einrichten und die Beispielplanung öffnen, ändern, rechnen, speichern
(Prüfliste in `docs/QA.md`). Die Abdeckung von `vivaplan-cli` wird heute nicht mit einer Schwelle geprüft; die Logik der
Sitzung soll deshalb so liegen, dass sie ohne Prozessstart testbar ist.

## 11 Ansichten in Claude (zweite Stufe)

Sobald der Server steht, bekommen ausgewählte Werkzeuge eine Ansicht nach MCP Apps:

- `result_get` mit `view: "bands"`: Vermögensverlauf mit Bandbreite, nominal und real umschaltbar.
- `result_compare`: Tabelle der Kennzahlen und Verlauf im Vergleich.

Die Ansicht ist eine einzelne HTML-Datei aus `charts.js` und einem kleinen Teil von `app.js`, im Programm eingebettet,
ohne Zugriff auf fremde Adressen. Hosts ohne MCP Apps zeigen weiterhin den Text. Ob die Daten der Ansicht den Rechner
verlassen, hängt vom Host ab und wird vor der Umsetzung geklärt; bis dahin gilt für sie dieselbe Datensparsamkeit wie
für Text.

## 12 Folge-Issues

Reihenfolge wie nummeriert; 1 und 2 sind voneinander unabhängig.

**1. Rechenkern: Jahreswerte, Kurzfassung und Vergleich in `api`** (engine, ui)
- [ ] `api` liefert Jahreswerte, Kennzahlen mit Ergebnis in Worten und den Vergleich mehrerer Ergebnisse.
- [ ] Die Oberfläche nutzt diese Funktionen; `yearly` und der Rechenteil von `compareRuns` in `app.js` entfallen.
- [ ] Die Anzeige der Beispielplanung ist unverändert (Oberflächentest).
- [ ] Die Golden-Dateien ändern nicht.

**2. Rechenkern: Werte per Feldpfad ändern und Unterschiede zweier Planungen** (engine)
- [ ] `api` setzt einen Wert über einen Feldpfad und lehnt unbekannte Pfade und falsche Typen mit einer Meldung ab.
- [ ] `api` liefert die Unterschiede zweier Planungen als Liste aus Feldpfad, altem und neuem Wert.
- [ ] Die Feldpfade der Prüfmeldungen lassen sich unverändert verwenden.

**3. `vivaplan mcp`: Grundgerüst, nur lesend** (engine, repo)
- [ ] `vivaplan mcp` meldet sich über stdio an und listet die Werkzeuge.
- [ ] `plan_open`, `plan_get`, `plan_schema`, `plan_validate`, `plan_preview`, `result_get`, `result_list`,
      `library_list` arbeiten wie in Abschnitt 5.
- [ ] Grenzen und Datenbibliothek kommen aus dem Datenordner der App; `--data-dir` übersteuert.
- [ ] Keine Antwort überschreitet die Obergrenze; Namen und Pfade sind ersetzt.
- [ ] Jede Ergebnisantwort trägt den Haftungsausschluss.
- [ ] Die Kommandozeile hat keine Netzwerkabhängigkeit (Prüfung im Workflow).

**4. Simulation und Optimierung mit Fortschritt und Abbruch** (engine)
- [ ] `simulation_estimate`, `simulation_run`, `optimize_run` arbeiten wie in Abschnitt 5.
- [ ] Lange Rechnungen melden Fortschritt und lassen sich abbrechen.
- [ ] Ab 60 s geschätzter Dauer startet die Rechnung nur mit Bestätigung.
- [ ] Die Kennzahlen stimmen mit `api::run` und `api::optimize` überein.

**5. Ändern, Rückgängig, Vergleichen, Speichern** (engine)
- [ ] `plan_set`, `plan_changes`, `plan_undo`, `optimize_apply`, `result_compare` arbeiten wie in Abschnitt 5.
- [ ] `plan_save` hält die Regeln 1 bis 6 aus Abschnitt 7 ein; jede Regel hat einen Test.
- [ ] `--read-only` entfernt `plan_save` aus der Werkzeugliste.

**6. Auslieferung, Einrichtung und Hilfe** (repo, ui)
- [ ] Die Pakete enthalten das Programm `vivaplan`.
- [ ] «Hilfe → Mit Claude verbinden» zeigt Pfad und Einrichtungszeile für Claude Desktop und Claude Code.
- [ ] Hilfe (Datenschutz, neues Kapitel, Referenz), `BESCHREIBUNG.md`, `README.md` und `CLAUDE.md` sind nachgeführt.
- [ ] `docs/QA.md` enthält die Prüfliste für beide Claude-Programme.

**7. Ansichten in Claude (MCP Apps)** (ui), Meilenstein «Später»
- [ ] Vermögensverlauf und Vergleich erscheinen in Claude Desktop als Ansicht.
- [ ] Hosts ohne MCP Apps erhalten weiterhin Text.
- [ ] Die Ansicht lädt nichts von fremden Adressen.

## 13 Offene Entscheide

| Nr. | Frage | Empfehlung |
|---|---|---|
| E1 | Ganze Oberfläche in Claude oder Werkzeuge mit kleinen Ansichten? | Werkzeuge, danach kleine Ansichten (Abschnitt 3) |
| E2 | Darf die Auswertung aus `app.js` in den Rechenkern wandern (Folge-Issue 1)? | Ja; sonst wäre sie doppelt |
| E3 | Auslieferung des Programms | Im App-Paket und einzeln (Abschnitt 9) |
| E4 | Protokoll von Hand oder mit dem Rust-SDK `rmcp`? | Von Hand (Abschnitt 4) |
| E5 | Namen standardmässig ersetzen? | Ja, mit `--names` abschaltbar |

## 14 Quellen

- MCP Apps, Übersicht und unterstützte Programme: <https://modelcontextprotocol.io/extensions/apps/overview>
- SEP-1865 MCP Apps: <https://modelcontextprotocol.io/seps/1865-mcp-apps-interactive-user-interfaces-for-mcp>
- Claude Code und MCP (Einrichtung, Ausgabegrenzen, Zeitlimiten): <https://code.claude.com/docs/en/mcp>
