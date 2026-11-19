# Mitmachen

Danke für dein Interesse an VivaPlan. Das Projekt wird von einer Person betreut. Damit der Aufwand überschaubar bleibt, gilt:

## Zuerst ein Issue oder eine Frage

- **Fehler oder Wunsch:** Lege ein Issue mit der passenden Vorlage an (Fehler melden, Funktion vorschlagen).
- **Frage oder Idee:** Nutze die Discussions.
- **Sicherheitslücke:** nicht öffentlich melden, siehe [SECURITY.md](SECURITY.md).

Bei Funktionen bitte beschreiben, was möglich sein soll, und prüfbare Abnahmekriterien nennen, nicht die Lösung vorschreiben.

## Pull Requests

Pull Requests bitte nur nach Absprache in einem Issue. Ungefragte Änderungen am Rechenkern kann ich nicht sinnvoll prüfen, weil sie Steuern und Vorsorge betreffen.

## Keine privaten Daten

Verwende in Issues, Tests, Beispielen und Screenshots nur erfundene Daten, nie echte Zahlen, Namen oder Dokumente.

## Wenn abgesprochen: Entwicklung

- Aufbau, Rechenregeln und Werkzeuge: [docs/ENTWICKLUNG.md](docs/ENTWICKLUNG.md)
- Tests: [docs/TESTS.md](docs/TESTS.md)
- Vor jeder Änderung `make test-quiet` und `cargo fmt --all --check`
- Berechnung gehört in `crates/engine/`, Darstellung in `crates/app/ui/`
- Änderungen am Rechenkern brauchen einen Regressionstest
- Keine neuen Abhängigkeiten ohne vorherige Absprache
- Die App greift nicht auf das Netzwerk zu und soll es auch künftig nicht tun
- Oberfläche und Hilfe sind deutsch und sprechen mit «Sie»

Mit einem Beitrag stimmst du zu, dass er unter der Lizenz des Projekts (GPL-3.0) steht.
