# FEA-45 Haftungsausschluss

Issue #61. Vorlage: Entwurf «Disclaimer» aus dem Marketing, auf «Sie» angepasst.

## Entscheide (Kommentar im Issue)

| Frage | Entscheid |
|---|---|
| Anbieter in Punkt 1 | «VivaPlan wird aus der Schweiz unentgeltlich zur Verfügung gestellt.» Kein Name. |
| App-Name | VivaPlan (FEA-34 ist abgeschlossen). |
| Anrede | «Sie». |
| Kommandozeile | Kurzfassung als eine Zeile in der Hilfe-Ausgabe. |
| Rechtliche Prüfung | Nicht Teil dieses Issues. |

## Bestätigung beim ersten Start (AK1, AK2)

- Dialog `#disclaimer` mit der Kurzfassung, dem Kästchen «Ich habe den Hinweis gelesen und verstanden.» und «Weiter».
  «Weiter» ist erst mit Häkchen aktiv. Esc schliesst den Dialog nicht; schliesst ihn der Browser trotzdem, öffnet er
  sich sofort wieder. Solange er offen ist, sind Menübefehle und Tastenkürzel gesperrt; Willkommensdialog und Hinweise
  zur Planung folgen erst danach.
- Gespeichert wird `disclaimerAccepted` = Version des Textes (`DISCLAIMER_VERSION` in `app.js`, heute 1) in den
  Einstellungen (Desktop-App: `einstellungen.json` im Datenordner; nie in der Planung). Weicht der gespeicherte Wert ab
  (oder fehlt er), fragt die App erneut. Bei einer Änderung des Textes die Version erhöhen.
- Desktop-App: immer geprüft. Browser-Entwicklungsmodus: nur mit dem Testzugang `#disclaimer` (Browserspeicher).

## Fusszeile (AK3)

«Modellrechnung ohne Gewähr. Keine Finanz- oder Steuerberatung.»

- Ergebnisansicht: Fusszeile `#resultsFoot` unter allen Registerkarten (Übersicht, Grafiken, Einzelne Läufe mit den
  Ereignissen, Szenarien, Jahreswerte, Parameter) und bei gespeicherten Ergebnissen.
- Optimierung: Fusszeile `#optFoot` unter dem Ergebnis, sobald eines vorliegt.
- Einen Ausdruck oder PDF-Export gibt es nicht.

## Vollständiger Text (AK4)

- Hilfe, Abschnitt «Haftungsausschluss» (`hilfe/00-einstieg.md`, Anker `disclaimer`), sieben Punkte mit Stand-Datum.
- «Über VivaPlan» (App-Menü und Menü Hilfe): Fusszeile und Verweis «Vollständiger Haftungsausschluss», der die Hilfe
  an dieser Stelle öffnet.

## Kommandozeile

Die Hilfe-Ausgabe (ohne oder mit unbekanntem Befehl) endet mit der Zeile der Fusszeile.

## Tests (AK5)

`e2e/ui.spec.js`:
- `FEA-45/AK1, AK2: Haftungsausschluss beim ersten Start …` (Desktop-App simuliert: ohne Häkchen und mit Esc kein
  Weiterkommen, Tastenkürzel gesperrt, Bestätigung in den Einstellungen, nicht in der Planung, danach Willkommen).
- `FEA-45/AK2: bestätigte Version fragt nicht mehr, eine ältere Version fragt erneut`.
- `FEA-45 Haftungsausschluss`: AK2 im Browserspeicher, AK3 Fusszeile auf allen Registerkarten und unter der
  Optimierung, AK4 «Über» und Hilfe mit allen sieben Punkten und Stand.
- `crates/cli/tests/cli.rs`, `usage_and_schema`: Zeile in der Hilfe-Ausgabe.
