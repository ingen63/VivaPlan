Beim Start öffnet die Desktop-App automatisch die Planung, die zuletzt geöffnet oder gespeichert wurde. Ist keine bekannt, die Datei nicht mehr vorhanden oder nicht lesbar, öffnet sie stattdessen die im App-Bundle mitgelieferte neutrale Beispielplanung. In den beiden letzten Fällen erscheint ein Hinweis mit Dateinamen. Die Beispielplanung gilt als neue, ungespeicherte Planung (Speichern fragt nach einem Ort, sie wird nie überschrieben). Eine nicht lesbare Datei überschreibt die Merkung nicht. Beim Beenden mit ungespeicherten Änderungen fragt die App, ob gespeichert, verworfen oder abgebrochen werden soll. Im Browser-Entwicklungsmodus (`ui-dev`) gilt die Anforderung nicht.

**Abnahme**

- [ ] Planung öffnen, App beenden, neu starten: dieselbe Planung ist geladen, der Dateiname steht im Kopf.
- [ ] Speichern unter neuem Namen und Neustart: die neue Datei wird geöffnet.
- [ ] Erststart ohne Merkung: Beispielplanung, kein Hinweis.
- [ ] Datei zwischen den Starts gelöscht: Beispielplanung und Hinweis «Zuletzt geöffnete Datei nicht gefunden».
- [ ] «Neue Planung» ohne Speichern, dann Neustart: die zuletzt gespeicherte Planung wird geöffnet.
- [ ] Beenden mit ungespeicherten Änderungen: Dialog mit Speichern, Verwerfen, Abbrechen; Abbrechen lässt die App offen.
- [ ] Die Merkung enthält nur den Dateipfad und bleibt lokal.
- [ ] Die Beispielplanung enthält keine persönlichen Daten und ist ohne Fehler simulierbar.
- Abhängigkeit: Die Beispielplanung muss neutral sein (siehe GEN-01); die heutigen Dateien in `examples/` sind es nicht.

**Umsetzungshinweis** (nicht Teil der Anforderung): Pfad im Datenverzeichnis der App ablegen (dort liegen auch die Zeitreihen); Beispielplanung als Ressource im Bundle.

> «App, immer mit der letzten Planung öffnen»

_Ursprüngliche ID: FEA-02_
