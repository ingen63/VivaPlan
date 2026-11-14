# Auftrag: App von RetirementPlanner in VivaPlan umbenennen

## Kontext und Ziel

Die App (Tauri, Rust-Backend, Web-Frontend) heisst bisher „RetirementPlanner“, das Projekt „RetirementPlanning“. Sie wird vollständig in **VivaPlan** umbenannt und danach in ein **neues GitHub-Repo ohne alte Git-History** übertragen. Die Dateiendung wechselt von `.rplan` auf `.vplan`, alte Dateien müssen weiterhin ladbar sein.

## Verbindliche Namen

| Was | Wert |
|---|---|
| Produktname (überall sichtbar) | `VivaPlan` (grosses V, grosses P) |
| Fenstertitel | `VivaPlan - ‹Dateiname›`, ohne offene Datei nur `VivaPlan` |
| Rust-Paket (`Cargo.toml`) | `vivaplan`, Lib-Name `vivaplan_lib` |
| npm-Paket (`package.json`) | `vivaplan` |
| Bundle-Identifier | `ch.vivaplan-app.desktop` |
| Neue Dateiendung | `.vplan` |
| Dateityp-Beschreibung | `VivaPlan-Plan` |
| Neues GitHub-Repo | `vivaplan` (privat) |
| Domain (für Links, Impressum, About) | `vivaplan-app.ch` |

Regel: Sichtbare Texte schreiben immer `VivaPlan`. Technische Bezeichner (Pakete, Ordner, Crates, Repo) immer klein `vivaplan`.

## Ist-Zustand

- Name „RetirementPlanner“ bzw. „RetirementPlanning“ in Konfiguration, Code, UI-Texten, Icons, README und Build-Skripten.
- Dateien werden als `.rplan` gespeichert, z. B. `demo-data/Demo.rplan`.

## Soll-Verhalten

1. Kein sichtbarer oder technischer Rest des alten Namens, ausser bewusst behaltenen Kompatibilitätsstellen (siehe Grenzfälle).
2. Neue Pläne werden als `.vplan` gespeichert. Der Öffnen-Dialog akzeptiert `.vplan` und `.rplan`.
3. Wird eine `.rplan`-Datei geöffnet und gespeichert, schlägt die App „Speichern unter …“ mit Endung `.vplan` vor. Die `.rplan`-Datei wird nie überschrieben oder gelöscht.
4. Enthält das Dateiformat intern eine Kennung mit dem alten Namen (z. B. `"app": "RetirementPlanner"` oder einen Format-Header): neue Dateien schreiben die neue Kennung, beim Lesen werden alte und neue Kennung akzeptiert.
5. Die Dateizuordnung des Betriebssystems verknüpft `.vplan` mit VivaPlan.

## Arbeitsschritte

### Phase 1: Analyse, danach kurzer Bericht

1. Projekt in einen neuen Ordner `vivaplan` kopieren (ohne `.git`, `node_modules`, `target`, Build-Artefakte). Alle weiteren Arbeiten nur in der Kopie; der alte Ordner bleibt unverändert.
2. Alle Fundstellen erfassen: Suche ohne Beachtung der Gross- und Kleinschreibung nach `retirement`, `rplan` und dem bisherigen Bundle-Identifier. Auch in Dateinamen, Ordnernamen, Icons und Metadaten.
3. Aufbau des Dateiformats prüfen: Gibt es eine interne Kennung oder Versionsnummer mit dem alten Namen?
4. Den bisherigen Bundle-Identifier und den daraus folgenden App-Datenordner ermitteln (unter macOS `~/Library/Application Support/<identifier>`).
5. Mir einen kurzen Bericht geben: Anzahl Fundstellen pro Bereich, Befund zum Dateiformat, alter Identifier. Dann direkt mit Phase 2 weitermachen.

### Phase 2: Umbenennung in der Kopie

6. `tauri.conf.json`: `productName`, `identifier`, Fenstertitel, Dateizuordnung (`fileAssociations` für `.vplan`).
7. `Cargo.toml`, `package.json`, Lockfiles neu erzeugen, alle Imports und Modulpfade.
8. UI-Texte, About-Dialog, Menüs, Disclaimer-Texte, Fenstertitel-Logik.
9. Dateiformat: Speichern als `.vplan`, Laden von `.vplan` und `.rplan`, Kennungen gemäss Soll-Verhalten Punkt 4.
10. Einmalige Migration der App-Daten: Ist der neue App-Datenordner leer und der alte vorhanden, beim ersten Start Einstellungen und Liste der zuletzt geöffneten Dateien kopieren. Den alten Ordner nie löschen.
11. README, Build- und Release-Skripte (z. B. GitHub Actions), Screenshot-Skript und dessen Auftrag `AUFTRAG-screenshots.md` auf neue Namen anpassen.
12. App-Icon: Dateinamen anpassen. Das Motiv bleibt vorerst gleich.

### Phase 3: Neues Repo, erst nach meiner Freigabe

13. Mir zeigen, was committet würde (`git status` nach `git init`), und auf mein OK warten.
14. Danach: `git init`, ein einziger Commit „Initial commit: VivaPlan“, `gh repo create vivaplan --private --source . --push`.
15. Am alten Repo nichts ändern. Archivieren mache ich selbst.

## Grenzfälle

- **Kompatibilitätsstellen** dürfen den alten Namen behalten: Lesen von `.rplan`, alte interne Kennung, alter Identifier für die Migration. Jede solche Stelle mit einem Kommentar `// legacy: RetirementPlanner` markieren.
- **`demo-data/Demo.rplan`** nicht umbenennen, nicht konvertieren, nicht ändern. Die `.vplan`-Version erstelle ich selbst in der App.
- **Code-Signing oder Notarisierung**, falls vorhanden: Zertifikate und Secrets nicht anfassen, nur auflisten, was ich im neuen Repo neu hinterlegen muss.
- **Unklare Fundstellen** (z. B. „retirement“ als Fachbegriff in einem Hilfetext): nicht ändern, im Bericht auflisten.

## Abnahmekriterien

- Die Suche ohne Beachtung der Gross- und Kleinschreibung nach `retirement` und `rplan` findet nur noch die markierten `legacy`-Stellen und fachliche Begriffe aus dem Bericht.
- Die App startet als „VivaPlan“: Dock, Menü, About-Dialog, Fenstertitel `VivaPlan - ‹Dateiname›`.
- Test: `Demo.rplan` öffnen, alle Werte identisch zur alten Version, „Speichern unter“ schlägt `Demo.vplan` vor, die gespeicherte `.vplan` lässt sich wieder öffnen und ergibt dieselben Ergebnisse.
- Test: Beim ersten Start werden Einstellungen aus dem alten App-Datenordner übernommen; der alte Ordner existiert danach noch.
- Ein Doppelklick auf eine `.vplan`-Datei im Finder öffnet VivaPlan.
- Alle bestehenden Tests sind grün, ein vollständiger Release-Build läuft durch.
- `Demo.rplan` ist unverändert (Prüfsumme vorher/nachher).
