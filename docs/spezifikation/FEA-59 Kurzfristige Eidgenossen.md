# FEA-59 Obligationen: kurzfristige Eidgenossen in der Anlagestrategie

Issue #76. Ergänzt die Anlagestrategie (FEA-53) um einen vierten Index für die Obligationen.

## Entscheide

| Frage | Entscheid |
|---|---|
| Anschluss nach 2020 | SARON (SNB `zimoma`, schon im Projekt), nicht die Geldmarktbuchforderungen der Eidgenossenschaft. |
| 1–3-jährige Eidgenossen | Nicht Teil von FEA-59; frei verfügbar nur ab 1988 (SNB `rendoblim`). |

## Reihe

- Wert in der Planung: `returns.strategy.bonds = "chShort"`, Name «Eidgenossenschaft kurzfristig».
- **1900–2020:** JST-Datenbank, Release 6, Schweiz, Spalte `bill_rate` (Rendite kurzfristiger Staatspapiere, etwa
  3 Monate, als Anteil). Die Spalte ist lückenlos ab 1870; verwendet ab 1900 wie die übrigen JST-Reihen.
- **Ab 2021:** Jahresmittel der zwölf SARON-Monatswerte der SNB (Prozent / 100). Nur ganze Kalenderjahre; ein
  angefangenes Jahr fällt weg. Mit den ausgelieferten SNB-Daten reicht die Reihe bis 2025.
- Der Übergang liegt direkt nach dem letzten JST-Jahr; es gibt keine Lücke.
- Bei etwa 3 Monaten Laufzeit entspricht die Jahresrendite dem Satz; Kursgewinne und -verluste werden vernachlässigt.
- Funktion: `jst::ch_short_yearly`.

## Strategie

Es gibt keine ausgelieferte Reihe mit kurzfristigen Eidgenossen. Jede Kombination ist deshalb jährlich über die
gemeinsamen Jahre (Regel von FEA-53):

| Aktien | Zeitraum mit `chShort` |
|---|---|
| `worldJst`, `worldCustom`, `chJst` | 1900–2020 |
| `chPictet` (entfernt mit FEA-72) | 1926–2024 |
| `sp500` | 1922–2025 |

Teuerung und SARON wie bei FEA-53. Quelle: JST und SNB `zimoma`; die Hinweise nennen die JST-Lizenz und dass die
Rendite praktisch dem Geldmarktsatz entspricht.

## Oberfläche und Hilfe

- Karte «Anlagestrategie»: vierte Wahl bei den Obligationen, mit Beschreibung (Quelle, Zeitraum, Geldmarktsatz).
- Hilfe «Anlagestrategie» (`20-renditen.md`): neue Zeile in der Tabelle der Obligationen.

## Tests

- `strategy::fea59_ak2_short_bonds_switch_from_jst_to_saron_without_a_gap`: 1900, 2020 (JST) und 2021 (SARON) von Hand.
- `strategy::fea59_ak1_ak3_short_bonds_in_strategies`: Name, Speicherform, Zeiträume, Quelle und Hinweise.
- Oberfläche: `FEA-53/AK1` in `e2e/ui.spec.js` prüft die vierte Wahl und den Zeitraum mit dem S&P 500.
- AK4: Golden-Dateien und bestehende Tests unverändert.
