# VivaPlan – Glossar

Englische Bezeichner für die Schweizer Fachbegriffe (#165). Der Code ist intern englisch (Regel in
[`CLAUDE.md`](../CLAUDE.md) und [`ENTWICKLUNG.md`](ENTWICKLUNG.md#sprache-im-code)); dieses Glossar legt fest, welches
englische Wort für welchen Begriff steht, damit die Umstellung der Pakete (#169, #166, #167, #168) überall dieselben
Namen verwendet.

**Regeln**

- Die Spalte «Bezeichner» gilt für neue und umgestellte Namen: Rust `snake_case` (Typen `UpperCamelCase`),
  JavaScript `camelCase`, Wortteile wie angegeben (`pension_fund` → `pensionFund`, `PensionFund`).
- Die meisten Bezeichner stammen aus dem bestehenden Code (Datenmodell `crates/engine/src/model.rs`, Ergebnisse
  `sim.rs`, `batch.rs`, `optimize.rs`). Wo der Code heute einen anderen Namen hat, steht er in «Bestehend».
- **Serialisierte Namen bleiben unverändert**, auch wenn sie vom Glossar abweichen oder deutsch sind: Schlüssel der
  Projektdatei (`.vplan`, `.rplan`), Ergebnisfelder, IPC-Befehle, `data-*`-Attribute und CSS-Klassen. Die
  Umstellung ändert nur interne Namen, Kommentare und Testnamen.
- Kommt ein Begriff nicht vor, wird er so übersetzt, wie ihn eine englische Fachperson verstehen würde, und hier
  ergänzt.

## Schweizer Kürzel

Diese Kürzel bleiben als Bezeichner erlaubt: klein in Namen, gross in Kommentaren. Die Prüfung `make english` meldet
Wörter in Grossbuchstaben nicht.

| Kürzel | Bedeutung | Bezeichner | Bemerkung |
|---|---|---|---|
| AHV | Alters- und Hinterlassenenversicherung, 1. Säule | `ahv` | `Ahv`, `ahv_couple_cap` |
| PK | Pensionskasse, 2. Säule | `pk` | als Präfix erlaubt (`pk_pension`, `pk_interest`); ausgeschrieben `pension_fund` |
| BVG | Bundesgesetz über die berufliche Vorsorge | `bvg` | |
| 3a | Säule 3a (gebundene Vorsorge) | `pillar3a`, kurz `p3a` | `Pillar3a`, `p3a_interest` |
| SARON | Swiss Average Rate Overnight | `saron` | |
| SNB | Schweizerische Nationalbank | `snb` | |
| JST | Jordà-Schularick-Taylor Macrohistory Database | `jst` | |
| VPW | Variable Percentage Withdrawal | `vpw` | |
| StG | Steuergesetz | `stg` | nur in Kommentaren |

## Person und Haushalt

| Fachbegriff | Bezeichner | Bestehend / Bemerkung |
|---|---|---|
| Haushalt | `household` | |
| Person, Hauptperson | `person`, `main_person` | erste Person in `household.persons` |
| Partnerin, Partner (zweite Person) | `partner` | |
| verheiratet | `married` | |
| Geburt, Geburtsmonat | `birth` | |
| Alter | `age` | Dezimaljahre (`years`) |
| Meilenstein | `milestone` | `Milestone`, `MilestoneKind` |
| Pensionierung | `retirement` | Meilenstein-Id `retire` (serialisiert) |
| Bezug (Rente, Kapital) | `payout` | Zeitpunkt `at` |
| Zeitpunkt (Datum, Alter, Meilenstein) | `time_ref` | `TimeRef` |
| Monat, Jahr-Monat | `month`, `year_month` | `YearMonth` |
| Simulationsstart, -ende | `start`, `end` | |

## Vorsorge

| Fachbegriff | Bezeichner | Bestehend / Bemerkung |
|---|---|---|
| Vorsorge | `pension` | `Pension` (AHV und PK einer Person) |
| Pensionskasse | `pension_fund` | `PensionFund`; serialisiert `pk` |
| Altersguthaben | `capital` | |
| Kapitalbezug | `lump_sum` | bestehend `lumpsum` (serialisiert) |
| Umwandlungssatz | `conversion_rate` | bestehend `conversion` (serialisiert) |
| PK-Rente | `pk_pension` | |
| Beiträge | `contribution` | |
| Verzinsung | `interest` | |
| AHV-Rente | `ahv_pension` | Feld `monthly` |
| Plafonierung (Ehepaar) | `couple_cap` | `ahv_couple_cap` |
| an die Teuerung angepasst | `indexed` | |
| Säule 3a | `pillar3a` | |
| 3a-Konto | `pillar3a_account` | `Pillar3aAccount` |
| Einzahlung (3a) | `contribution` | |
| Auszahlung (3a) | `payout` | `Pillar3aPayout`, `PayoutAt` |
| Verwendung (Auszahlung) | `usage` | `PayoutUse` |
| Rente (allgemein) | `pension` | Leibrente aus Kapital: `annuity` |

## Einkommen und Ausgaben

| Fachbegriff | Bezeichner | Bestehend / Bemerkung |
|---|---|---|
| Erwerbseinkommen | `earned_income` | bestehend `income` (Planung), `earned` (Ergebnis) |
| Einkommen (inkl. Renten) | `income` | |
| an die Teuerung gebunden | `indexed` | `income_indexed` |
| Lebenshaltungskosten, Ausgaben | `spending` | im Hauptbuch `living` |
| Wohnkosten | `housing` | `spending_includes_housing` |
| Ausgaben-Faktor | `spending_factor` | |
| Sparbetrag, weitere Zuflüsse | `savings` | |
| Einmalbetrag | `extra` | `Extra`; positiv = Zufluss (`inflow`) |
| ungedeckte Ausgaben, Bedarf | `need` | `NeedBasis` |
| Geldfluss | `flow` | `Flows` |
| Zeitreihe (Wert über die Zeit) | `series` | `Series`, `SeriesRow` |

## Vermögen und Anlage

| Fachbegriff | Bezeichner | Bestehend / Bemerkung |
|---|---|---|
| Vermögen | `wealth` | `Wealth` |
| Gesamtvermögen | `total` | `end_total`, `start_total` |
| liquides Vermögen | `liquid` | |
| Aktien | `stocks` | einzelner Wert `stock` |
| Obligationen | `bonds` | einzelner Wert `bond` |
| Cash, Bargeld | `cash` | |
| Aktienanteil | `stock_share` | |
| Cash-Reserve (Monate) | `reserve_months` | |
| Anlagestrategie | `investment` | `Investment` (#69) |
| Rendite | `returns`; einzeln `stock_return`, `bond_return` | `Returns`; `return` allein ist in Rust und JS ein Schlüsselwort |
| Realrendite | `real_return` | |
| Teuerung, Inflation | `inflation` | Teuerungsindex `price_index` |
| heutige Kaufkraft, real | `real` | Gegensatz `nominal` |
| Index (Aktien, Obligationen) | `index` | `StockIndex`, `BondIndex` |
| Region, Gewichte | `region`, `weights` | `RegionWeights` |
| Datenbibliothek | `library` | |
| Fortschreibung | `extrapolation` | `Extrapolation` |
| Rebalancing | `rebalancing` | |

## Entnahmestrategien

| Fachbegriff | Bezeichner | Bestehend / Bemerkung |
|---|---|---|
| Entnahme | `withdrawal` | `Withdrawal` |
| Entnahmestrategie | `strategy` | `Strategy` |
| Entnahmerate | `withdrawal_rate` | Optimierung: `draw_rate` |
| fixe Entnahmerate | `fixed_rate` | `FixedRate` |
| Leitplanken (Guyton-Klinger) | `guardrails` | `upper`, `lower`, `adjust` |
| Startentnahme | `initial_rate` | |
| Untergrenze, Obergrenze | `floor`, `ceiling` | |
| Endowment-Regel, Glättung | `endowment`, `smoothing` | |
| Planungsalter (VPW) | `horizon_age` | |
| 3-Topf-Strategie | `three_buckets` | Typ `Buckets` |
| Topf | `bucket` | Topf 1/2/3: `cash`, `bond`, `stock` bucket |
| Auffüllung | `refill` | `BucketRefill`, `refill_threshold` |
| geplante Ausgaben | `planned` | `BucketSpending::Planned` |

## Immobilien und Hypothek

| Fachbegriff | Bezeichner | Bestehend / Bemerkung |
|---|---|---|
| Immobilie, Liegenschaft | `property` | `Property`, `RealEstate` |
| Besitz, geplant, Miete | `owned`, `planned`, `rent` | `PropertyStatus` |
| Kaufpreis | `price` | |
| Marktwert | `worth` (Planung), `property_value` (Ergebnis) | |
| Kauf, Verkauf | `purchase`, `sale` | Planungsfelder `buy`, `sell`, `bought` |
| Notverkauf | `emergency_sale` | Schwelle `emergency_months` |
| Verkaufserlös, Nettoerlös | `proceeds`, `net_proceeds` | |
| Eigenmittel, Eigenkapital | `equity` | `property_equity` |
| Nebenkosten | `fix_costs` | serialisiert `fixCosts` |
| Miete | `rent` | |
| Hypothek | `mortgage` | `Mortgage` |
| Tranche | `tranche` | `Tranche` |
| Hypothekarperiode, Laufzeit | `period`, `term` | `Period` |
| Festhypothek, SARON-Hypothek | `fixed`, `saron` | `RateKind` |
| Marge | `margin` | |
| Erneuerung | `renewal` | `Renewal`, `AfterLast::Renew` |
| Rückzahlung | `repayment` | `AfterLast::Repay` |
| Einmalzahlung (bei Erneuerung) | `lump_sum` | `lump_sums` |
| Amortisation (direkt, indirekt) | `amortization` (`direct`, `indirect`) | Feld `amort`, `AmortMode` |
| Belehnung (maximal) | `loan_to_value` | `max_loan_to_value`, `AmountMode::MaxLoan` |
| Tragbarkeit | `affordability` | `Affordability`, `affordability_sales` |
| Tragbarkeitsgrenze (Anteil Einkommen) | `sustainability` | |
| kalkulatorischer Zins | `imputed_rate` | |
| Hypothekarzins | `mortgage_interest` | |

## Steuern

| Fachbegriff | Bezeichner | Bestehend / Bemerkung |
|---|---|---|
| Steuern | `taxes` | `Taxes`; einzelne Steuer `tax` |
| Steuertabelle, Tarif | `tax_table` | `TaxTables`, Stufen `brackets` |
| Steuerfuss (Kanton und Gemeinde) | `tax_multiplier` | bestehend `multiplier` (serialisiert) |
| Einkommenssteuer | `income_tax` | Ergebnis `tax_income` |
| Vermögenssteuer | `wealth_tax` | Ergebnis `tax_wealth` |
| Kapitalbezugssteuer | `capital_tax` | Ergebnis `tax_capital`, `lumpsum_tax` |
| Grundstückgewinnsteuer | `property_gain_tax` | `PropertyGainTax`, Ergebnis `tax_gain` |
| Besitzdauer | `holding_period` | Tabellen `duration`, `duration_rates` |
| Freigrenze | `exempt_below` | |
| Ermässigung | `reduction` | |
| veranlagt, Steuerjahr | `assessed`, `tax_year` | `assessed_income`, `assessed_wealth` |
| Verheiratetentarif | `couple` | |
| Kanton, Gemeinde | `canton`, `municipality` | |

## Simulation und Ergebnisse

| Fachbegriff | Bezeichner | Bestehend / Bemerkung |
|---|---|---|
| Planung | `plan` | `Plan`; Projektdatei `project` |
| Rechenkern | `engine` | |
| Simulation | `simulation`, kurz `sim` | |
| Lauf | `run` | Mehrzahl `runs` |
| Pfad (Renditepfad) | `path` | `MarketPath` |
| Renditequelle: fix, historisch, rollierend | `fixed`, `historical`, `rolling` | `ReturnSource` |
| Bootstrap, parametrisches Modell | `bootstrap`, `parametric` | |
| Kurssprung | `jump` | `jump_lambda`, `jump_mu` |
| Startwert (Zufall) | `seed` | |
| Erfolgsquote | `success_rate` | |
| Perzentil | `percentile` | `Percentile` |
| Bandbreite | `bands` | `Bands` |
| Endvermögen | `end_total` | |
| Erschöpfung (Geld aufgebraucht) | `depletion` | `depleted`, `earliest_depletion` |
| Ruin, Ruinwahrscheinlichkeit | `ruin` | `max_ruin`, `ruin_age` |
| Ereignis | `event` | `Event`, `EventKind` |
| Hauptbuch | `ledger` | `Ledger` |
| Monatswerte | `month_record` | `MonthRecord` |
| Jahreswerte | `yearly` | |
| Kennzahlen | `summary`, `metrics` | `Summary`, `Metrics` |
| Vorschau | `preview` | |
| Fortschritt, Abbruch | `progress`, `cancel` | |

## Prüfung, Optimierung, Szenarien

| Fachbegriff | Bezeichner | Bestehend / Bemerkung |
|---|---|---|
| Prüfung | `validation` | Modul `validate` |
| Fehler, Hinweis | `error`, `warning` | `Severity`; Meldung `issue` |
| Pflichteingabe | `required_input` | |
| Optimierung | `optimization` | Modul `optimize` |
| Ziel (Optimierung) | `goal`, `objective` | `Goal`, `Objective` |
| Bandbreite eines Parameters | `range` | `ParamRange` |
| Konsum (Optimierung) | `consumption` | `min_consumption` |
| Szenario, Variation | `scenario`, `sweep` | `Scenarios`, `Sweep` |
| Variante | `variant` | |
| Import alter Konfigurationen | `legacy_import` | Modul `legacy` |
| Aktualisieren älterer Dateien | `upgrade` | Modul `upgrade` |

## Oberfläche

| Fachbegriff | Bezeichner | Bestehend / Bemerkung |
|---|---|---|
| Oberfläche | `ui` | |
| Abschnitt | `section` | |
| Karte | `card` | |
| Registerkarte | `tab` | |
| Feld | `field` | |
| Knopf | `button` | |
| Grafik | `chart` | |
| Hilfe, Thema | `help`, `topic` | Ordner `hilfe/` bleibt |
| Einstellungen | `settings` | |
| Vergleich | `compare` | |
| gespeicherte Ergebnisse | `saved_runs` | |
