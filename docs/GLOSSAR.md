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
| Immobilien nie verkaufen (Schalter) | `never_sell` | serialisiert `neverSell` (#196) |
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
| Freibetrag (Grundstückgewinn) | `allowance` | #65 |
| Höchstbelastung | `max_share` | #65, Anteil des Gewinns |
| Anlagekosten | `cost` | #65, Modell «return»: bezahlter Kaufpreis |
| Regelsatz des Kantons | `rule`, `GainRule` | #65 |
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

## Englisch (Oberflächenbegriffe)

Begriffe der englischen Oberfläche (#172, Tabelle `crates/app/ui/i18n/en.json`). Schweizer Fachbegriffe werden
englisch umschrieben, das Schweizer Kürzel steht in Klammern; Schreibweise amerikanisch (*optimization*,
*amortization*, *gray*). Die Wörter folgen den Code-Bezeichnern der Tabellen oben.

**Regeln**

- Volle Form mit Kürzel in Überschriften, Einleitungen und Erklärungen, z. B. «old-age pension (AHV)», «pension fund
  (BVG)»; in kurzen Beschriftungen, Tabellenköpfen und Ereignissen genügt die Kurzform («AHV pension», «pension fund»,
  «PK pension»).
- Kürzel bleiben unübersetzt: AHV, BVG, PK, 3a, SARON, SNB, JST, VPW, StG, CHF.
- Zahlen, Beträge und Daten bleiben im Schweizer Format (1'234'567.50 CHF, 2.5 %, MM.YYYY); das Datumsmuster heisst
  englisch «MM.YYYY».
- Inhalte der Planung (Namen von Personen, Meilensteinen, Objekten, Projekt) werden nicht übersetzt.
- Namen der Indizes werden übersetzt («World (JST)», «Switzerland (JST)», «Swiss Confederation 10 years»), die
  Bezeichner der Strategien im Code nicht (`standard`, `three_buckets` …).
- Anrede: direkt mit «you», Befehle im Imperativ («Start simulation», «Save as…»).

| Deutsch | Englisch (Oberfläche) | Kurzform / Bemerkung |
|---|---|---|
| AHV, AHV-Rente | old-age pension (AHV) | AHV pension |
| AHV-Bezug (Meilenstein) | AHV pension start | |
| Plafonierung (Ehepaar) | cap on AHV pensions for couples | |
| 1. / 2. Säule | pillar 1 / pillar 2 | |
| Pensionskasse | pension fund (BVG) | pension fund; in Zusammensetzungen PK |
| Bezug Pensionskasse (Meilenstein) | pension fund payout | |
| PK-Rente | PK pension | Rente aus der Pensionskasse |
| Kapitalbezug | lump sum, lump-sum withdrawal | |
| Altersguthaben | retirement savings | |
| Umwandlungssatz | conversion rate | |
| Säule 3a, 3a-Konto | pillar 3a, 3a account | |
| gebundene Vorsorge | tied pension provision | |
| Vorsorge(vermögen) | pension assets | Abschnitt «Pensions» |
| Einzahlung, Auszahlung (3a) | contribution, payout | |
| Pensionierung | retirement | |
| Simulationsstart, -ende | simulation start, simulation end | |
| Meilenstein | milestone | |
| Haushalt | household | |
| Erwerbseinkommen | earned income | |
| Lebenshaltungskosten, Lebenshaltung | living expenses | |
| Ausgaben | spending | |
| Wohnkosten | housing costs | |
| Einmaliger Betrag | one-off amount | |
| Zufluss | inflow | |
| ungedeckte Ausgaben, Netto-Bedarf | uncovered spending, net need | |
| Teuerung, Inflation | inflation | «adjusted for inflation» für «mit der Teuerung angepasst» |
| in heutigen Preisen, heutige Kaufkraft | in today's prices, today's purchasing power | |
| Vermögen | wealth | |
| liquides Vermögen | liquid wealth | |
| Gesamtvermögen, Endvermögen | total wealth, final wealth | |
| Aktien, Obligationen | stocks, bonds | |
| Aktienanteil, Aktienquote | stock allocation | |
| Anlagestrategie | investment strategy | |
| Zeitreihe | time series | |
| Renditequelle | return source | |
| Fortschreibung | extrapolation | |
| Zufallsstartwert | random seed | |
| Cash-Reserve | cash reserve | |
| Entnahme, Entnahmerate | withdrawal, withdrawal rate | |
| Entnahmestrategie | withdrawal strategy | |
| fixe Entnahmerate | fixed withdrawal rate | |
| Leitplanke, Guardrails | guardrail, guardrails | |
| Startrate | initial rate | |
| Untergrenze, Obergrenze | floor, ceiling | |
| 3-Topf-Strategie, Topf | 3-bucket strategy, bucket | Topf 1/2/3: bucket 1/2/3 |
| Auffüllung, Januar-Auffüllung | refill, January refill | |
| Vorausschau | look-ahead | |
| Renditekriterium | return criterion | |
| Reservierung | reservation | |
| Variable Prozententnahme | variable percentage withdrawal (VPW) | |
| Planungsalter | planning age | |
| Realrendite | real return | |
| Immobilie, Objekt | property; Abschnitt «Real estate» | |
| Besitz, Kauf geplant, Miete geplant | owned, purchase planned, rent planned | |
| Kaufpreis, Marktwert | purchase price, market value | |
| Eigenmittel, Eigenkapital | equity | |
| Nebenkosten | fixed costs | |
| Verkaufserlös, Nettoerlös | sale proceeds, net proceeds | |
| Notverkauf, Notverkaufskriterium | emergency sale, emergency sale criterion | |
| Immobilien nie verkaufen | never sell properties | |
| Hypothek, Anschlusshypothek | mortgage, follow-up mortgage | |
| Tranche, Periode, Laufzeit | tranche, period, term | |
| Festhypothek, SARON-Hypothek | fixed, SARON mortgage | |
| Marge | margin | |
| Erneuerung, Rückzahlung | renewal, repayment | |
| Einmalzahlung, Einmalrückzahlung | one-off payment, one-off repayment | |
| Amortisation (direkt, indirekt) | amortization (direct, indirect) | |
| Belehnung | loan-to-value | |
| Tragbarkeit | affordability | |
| kalkulatorischer Zins | imputed interest rate | |
| Steuerfuss | tax multiplier | |
| Stufe (Steuertabelle) | bracket | |
| Einkommens-, Vermögenssteuer | income tax, wealth tax | |
| Kapitalbezugssteuer | lump-sum tax | |
| Grundstückgewinnsteuer | property gains tax | |
| Besitzdauer | holding period | |
| Zuschlag, Abzug, Ermässigung | surcharge, reduction | |
| Verheiratete | married (couples) | |
| Kanton, Gemeinde | canton, municipality | Modell Zürich/Aargau: Zurich/Aargau model |
| Planung | plan | |
| Projektdatei | project file | |
| Ergebnis, gespeicherte Ergebnisse | result, saved results | |
| Lauf, Einzellauf, Einzelne Läufe | run, single run, individual runs | |
| Pfad | path | |
| Perzentil | percentile | 5. Perzentil: 5th percentile |
| Median | median | |
| Erfolgsquote | success rate | |
| Erschöpfung, aufgebraucht | depletion, depleted | |
| Ruin, Ruinwahrscheinlichkeit | ruin, probability of ruin | |
| Schwankung | volatility | |
| Konsum | consumption | |
| Szenario, Variation, Variante | scenario, variation, variant | |
| Stufe (Variation) | step | |
| Ereignis | event | |
| Jahreswerte, Kennzahlen | yearly values, key figures | |
| Vorschau | preview | |
| Prüfung, Fehler, Hinweis | validation, error, warning / note | |
| Optimierung, Ziel, Nebenbedingung | optimization, goal, constraint | |
| Einstellungen | settings | |
| Hilfe, Handbuch, Erste Schritte | help, manual, getting started | |
| Tastenkürzel | keyboard shortcuts | |
| wie System | as system | Sprachwahl |

## Französisch (Oberflächenbegriffe)

Begriffe der französischen Oberfläche (#174, Tabelle `crates/app/ui/i18n/fr.json`, Hilfe `crates/app/ui/hilfe/fr/`).
Massgebend ist die Schweizer Verwaltungssprache (Begriffe der Bundesverwaltung, der AHV-Ausgleichskassen und des
BVG); französische Begriffe aus Frankreich (*retraite complémentaire*, *PER*, *livret*) werden nicht verwendet.

**Regeln**

- Anrede mit «vous», Befehle im Infinitiv («Lancer la simulation», «Enregistrer sous…»), Meldungen in ganzen Sätzen.
- Schweizer Kürzel in der französischen Form: AVS (statt AHV), LPP (statt BVG), CP (caisse de pension, statt PK),
  BNS (statt SNB), UE (statt EU); unverändert bleiben 3a, SARON, JST, VPW, CHF.
- Schweizer Wortschatz: *fortune* (nicht *patrimoine*), *renchérissement* neben *inflation*, *impôt sur les
  prestations en capital*, *coefficient d’impôt*, *commune*, *année civile*.
- Anführungszeichen «…» ohne Leerraum innen (Schweizer Satz); Beschriftungen der Oberfläche stehen in der Hilfe in
  «…», andere Zitate (Beispiele, Eingaben) in ‹…›. Kein Leerraum vor «:», «;», «?», «!».
- Ordnungszahlen ohne Hochstellung: 1er, 5e, 95e percentile; Datumsmuster «MM.AAAA», Monatsformat in Meldungen
  «AAAA-MM».
- Zahlen, Beträge und Daten im Schweizer Format wie in allen Sprachen (1'234'567.50 CHF, 2.5 %); Inhalte der
  Planung (Namen) werden nicht übersetzt; Strategienamen «Standard», «Guardrails», «Endowment», «VPW» bleiben.
- Inklusive Form nur, wo die Person angesprochen ist und die Kürze es erlaubt («Marié·e·s»).

| Deutsch | Französisch (Oberfläche) | Kurzform / Bemerkung |
|---|---|---|
| AHV, AHV-Rente | assurance-vieillesse et survivants (AVS), rente AVS | AVS |
| AHV-Bezug (Meilenstein) | début de la rente AVS | |
| Plafonierung (Ehepaar) | plafonnement des rentes AVS (couples) | |
| 1. / 2. / 3. Säule | 1er / 2e / 3e pilier | |
| BVG | prévoyance professionnelle (LPP) | LPP |
| Pensionskasse | caisse de pension | CP; in Zusammensetzungen «rente CP», «retrait en capital CP» |
| Bezug Pensionskasse (Meilenstein) | prestation de la caisse de pension | |
| PK-Rente | rente de la caisse de pension | rente CP |
| Kapitalbezug | retrait en capital, prestation en capital | |
| Altersguthaben | avoir de vieillesse | |
| Umwandlungssatz | taux de conversion | |
| Verzinsung (PK) | rémunération | |
| Säule 3a, 3a-Konto | pilier 3a, compte 3a | |
| gebundene Vorsorge | prévoyance liée | |
| Vorsorge(vermögen) | prévoyance, avoirs de prévoyance | Abschnitt «Prévoyance» |
| Einzahlung, Auszahlung (3a) | versement, retrait | Auszahlungsplan: plan de retrait; «versé» bleibt, wo die Bank auszahlt (#182) |
| Pensionierung | retraite | |
| Simulationsstart, -ende | début de la simulation, fin de la simulation | |
| Meilenstein | jalon | |
| Haushalt | ménage | |
| Verheiratet, Verheiratete | marié·e·s, personnes mariées | |
| Erwerbseinkommen | revenu de l’activité lucrative | Kurzform: revenu, activité |
| Lebenshaltungskosten, Lebenshaltung | coût de la vie | |
| Ausgaben | dépenses | |
| Wohnkosten | frais de logement | |
| Einmaliger Betrag | montant unique | |
| Zufluss | entrée | |
| ungedeckte Ausgaben, Netto-Bedarf | dépenses non couvertes, besoin net | |
| Teuerung, Inflation | renchérissement, inflation | «adapté au renchérissement» |
| in heutigen Preisen, heutige Kaufkraft | aux prix actuels, pouvoir d’achat actuel | |
| Vermögen | fortune | |
| liquides Vermögen, freies Vermögen | fortune liquide, fortune libre | |
| Gesamtvermögen, Endvermögen | fortune totale, fortune finale | |
| Aktien, Obligationen | actions, obligations | |
| Aktienanteil, Aktienquote | part d’actions, quote-part d’actions | |
| Anlagestrategie | stratégie de placement | |
| Zeitreihe | série temporelle | |
| Renditequelle | source de rendements | |
| Fortschreibung | extrapolation | |
| Zufallsstartwert | valeur initiale aléatoire | |
| Cash-Reserve | réserve de cash | |
| Entnahme, Entnahmerate | prélèvement, taux de prélèvement | |
| Entnahmestrategie | stratégie de prélèvement | |
| fixe Entnahmerate | taux de prélèvement fixe | |
| Leitplanke, Guardrails | garde-fou, Guardrails | Strategiename bleibt «Guardrails» |
| Startrate | taux initial | |
| Untergrenze, Obergrenze | plancher, plafond | |
| 3-Topf-Strategie, Topf | stratégie à 3 compartiments, compartiment | Topf 1/2/3: compartiment 1/2/3; T1: C1 |
| Auffüllung, Januar-Auffüllung | réalimentation, réalimentation de janvier | |
| Vorausschau | prévision | «anticipation» steht für den Vorbezug (#182) |
| Renditekriterium | critère de rendement | |
| Reservierung | réservation | |
| Variable Prozententnahme | prélèvement en pourcentage variable (VPW) | |
| Planungsalter | âge de planification | |
| Realrendite | rendement réel | |
| Immobilie, Objekt | bien immobilier, objet | Abschnitt «Immobilier» |
| Besitz, Kauf geplant, Miete geplant | propriété, achat planifié, location planifiée | |
| Kaufpreis, Marktwert | prix d’achat, valeur vénale | |
| Eigenmittel, Eigenkapital | fonds propres | |
| Nebenkosten | frais accessoires | |
| Verkaufserlös, Nettoerlös | produit de la vente, produit net | |
| Notverkauf, Notverkaufskriterium | vente forcée, critère de vente forcée | |
| Immobilien nie verkaufen | ne jamais vendre les biens immobiliers | |
| Hypothek, Anschlusshypothek | hypothèque, hypothèque subséquente | |
| Tranche, Periode, Laufzeit | tranche, période, durée | |
| Festhypothek, SARON-Hypothek | hypothèque à taux fixe, hypothèque SARON | |
| Marge | marge | |
| Erneuerung, Rückzahlung | renouvellement, remboursement | |
| Einmalzahlung, Einmalrückzahlung | paiement unique, remboursement unique | |
| Amortisation (direkt, indirekt) | amortissement (direct, indirect) | |
| Belehnung | taux d’avance | |
| Tragbarkeit | capacité financière | |
| kalkulatorischer Zins | taux d’intérêt théorique | |
| Steuerfuss | coefficient d’impôt | |
| Stufe (Steuertabelle) | palier | |
| Steuertabelle | barème | |
| Einkommens-, Vermögenssteuer | impôt sur le revenu, impôt sur la fortune | |
| Kapitalbezugssteuer | impôt sur les prestations en capital | |
| Grundstückgewinnsteuer | impôt sur les gains immobiliers | |
| Besitzdauer | durée de possession | |
| Zuschlag, Abzug, Ermässigung | majoration, réduction | |
| Kanton, Gemeinde | canton, commune | Modell Zürich/Aargau: modèle Zurich/Argovie |
| Steuerjahr, veranlagt | année fiscale, taxé | |
| Planung | planification | |
| Projektdatei | fichier de projet | |
| Ergebnis, gespeicherte Ergebnisse | résultat, résultats enregistrés | |
| Lauf, Einzellauf, Einzelne Läufe | simulation, simulation unique, simulations individuelles | |
| Pfad | trajectoire | Zufallspfad: trajectoire aléatoire |
| Perzentil | percentile | 5. Perzentil: 5e percentile |
| Median | médiane | |
| Erfolgsquote | taux de réussite | |
| Erschöpfung, aufgebraucht | épuisement, épuisé | |
| Ruin, Ruinwahrscheinlichkeit | ruine, probabilité de ruine | |
| Schwankung | volatilité | |
| Konsum | consommation | |
| Szenario, Variation, Variante | scénario, variation, variante | |
| Stufe, Schritt (Variation) | niveau, pas | |
| Ereignis | événement | |
| Jahreswerte, Kennzahlen | valeurs annuelles, indicateurs | |
| Vorschau | aperçu | |
| Prüfung, Fehler, Hinweis | contrôle, erreur, avertissement / remarque | |
| Optimierung, Ziel, Nebenbedingung | optimisation, objectif, contrainte | |
| Einstellungen | paramètres | Menü «Paramètres…» |
| Hilfe, Handbuch, Erste Schritte | aide, manuel, premiers pas | |
| Tastenkürzel | raccourcis clavier | |
| Ansicht → Sprache | Affichage → Langue | |
| wie System | comme le système | Sprachwahl |
| Welt (JST), Schweiz (JST) | Monde (JST), Suisse (JST) | Indizes |
| Eidgenossenschaft 10 Jahre / kurzfristig | Confédération 10 ans / court terme | Indizes |

## Italienisch (Oberflächenbegriffe)

Begriffe der italienischen Oberfläche (#175, Tabelle `crates/app/ui/i18n/it.json`, Hilfe `crates/app/ui/hilfe/it/`).
Massgebend ist die Schweizer Verwaltungssprache (Begriffe der Bundesverwaltung, der Tessiner Behörden, der
AHV-Ausgleichskassen und des BVG); Begriffe aus Italien (*TFR*, *previdenza complementare* im italienischen Sinn,
*patrimonio* für das steuerbare Vermögen, *IRPEF*) werden nicht verwendet.

**Regeln**

- Anrede mit «Lei» (gross geschrieben: «Suo», «Sua»), Befehle und Schaltflächen im Infinitiv («Avviare la
  simulazione», «Salvare con nome…»), Meldungen in ganzen Sätzen.
- Schweizer Kürzel in der italienischen Form: AVS (statt AHV), LPP (statt BVG), CP (cassa pensione, statt PK),
  BNS (statt SNB), UE (statt EU); unverändert bleiben 3a, SARON, JST, VPW, CHF.
- Schweizer Wortschatz: *sostanza* (nicht *patrimonio*), *rincaro* neben *inflazione*, *imposta sulle prestazioni in
  capitale*, *moltiplicatore d’imposta*, *comune*, *anno civile*, *avere di vecchiaia*, *aliquota di conversione*.
- Anführungszeichen «…» ohne Leerraum innen; Beschriftungen der Oberfläche stehen in der Hilfe in «…», andere Zitate
  (Beispiele, Eingaben) in ‹…›. Apostroph typografisch (’): «dell’anno», «l’ipoteca».
- Ordnungszahlen mit «°» ohne Hochstellung (1°, 5°, 95° percentile); wo eine Zahl eingesetzt wird, ohne Ordnungszahl
  («Anno {v}», «Percentile {p}», «versamento n. {n}»). Datumsmuster «MM.AAAA», Monatsformat in Meldungen «AAAA-MM».
- Zahlen, Beträge und Daten im Schweizer Format wie in allen Sprachen (1'234'567.50 CHF, 2.5 %); Inhalte der
  Planung (Namen) werden nicht übersetzt; Strategienamen «Standard», «Guardrails», «Endowment», «VPW» bleiben.
- «Lauf» heisst *simulazione* (Einzellauf: *simulazione singola*), damit es nicht mit *percorso* (Pfad) verwechselt
  wird.

| Deutsch | Italienisch (Oberfläche) | Kurzform / Bemerkung |
|---|---|---|
| AHV, AHV-Rente | assicurazione vecchiaia e superstiti (AVS), rendita AVS | AVS |
| AHV-Bezug (Meilenstein) | inizio della rendita AVS | |
| Plafonierung (Ehepaar) | plafonamento delle rendite AVS (coniugi) | |
| 1. / 2. / 3. Säule | 1° / 2° / 3° pilastro | |
| BVG | previdenza professionale (LPP) | LPP |
| Pensionskasse | cassa pensione | CP; in Zusammensetzungen «rendita CP», «prelievo del capitale CP» |
| Bezug Pensionskasse (Meilenstein) | prestazione della cassa pensione | |
| PK-Rente | rendita della cassa pensione | rendita CP |
| Kapitalbezug | prelievo del capitale, prestazione in capitale | |
| Altersguthaben | avere di vecchiaia | |
| Umwandlungssatz | aliquota di conversione | |
| Verzinsung (PK) | remunerazione | |
| Säule 3a, 3a-Konto | pilastro 3a, conto 3a | |
| gebundene Vorsorge | previdenza vincolata | |
| Vorsorge(vermögen) | previdenza, averi di previdenza | Abschnitt «Previdenza» |
| Einzahlung, Auszahlung (3a) | versamento, riscossione | Auszahlungsplan: piano di riscossione; «versato» bleibt, wo die Bank auszahlt (#182) |
| Pensionierung | pensionamento | |
| Simulationsstart, -ende | inizio della simulazione, fine della simulazione | |
| Meilenstein | tappa | |
| Haushalt | economia domestica | |
| Verheiratet, Verheiratete | coniugati | |
| Erwerbseinkommen | reddito da attività lucrativa | Kurzform: reddito, lavoro |
| Lebenshaltungskosten, Lebenshaltung | costo della vita | |
| Ausgaben | spese | |
| Wohnkosten | spese d’alloggio | |
| Einmaliger Betrag | importo unico | |
| Zufluss | entrata | |
| ungedeckte Ausgaben, Netto-Bedarf | spese non coperte, fabbisogno netto | |
| Teuerung, Inflation | rincaro, inflazione | «adeguato al rincaro» |
| in heutigen Preisen, heutige Kaufkraft | ai prezzi attuali, potere d’acquisto attuale | |
| Vermögen | sostanza | |
| liquides Vermögen, freies Vermögen | sostanza liquida, sostanza libera | |
| Gesamtvermögen, Endvermögen | sostanza totale, sostanza finale | |
| Aktien, Obligationen | azioni, obbligazioni | |
| Aktienanteil, Aktienquote | quota azionaria | |
| Anlagestrategie | strategia d’investimento | |
| Zeitreihe | serie temporale | |
| Renditequelle | fonte di rendimenti | |
| Fortschreibung | estrapolazione | |
| Zufallsstartwert | valore iniziale casuale | |
| Cash-Reserve | riserva di cash | |
| Entnahme, Entnahmerate | prelievo, tasso di prelievo | |
| Entnahmestrategie | strategia di prelievo | |
| fixe Entnahmerate | tasso di prelievo fisso | |
| Leitplanke, Guardrails | guardrail, Guardrails | Strategiename bleibt «Guardrails»; männlich: «i Guardrails», «dei Guardrails» (#182) |
| Startrate | tasso iniziale | |
| Untergrenze, Obergrenze | limite inferiore, limite superiore | |
| 3-Topf-Strategie, Topf | strategia a 3 comparti, comparto | Topf 1/2/3: comparto 1/2/3; T1: C1 |
| Auffüllung, Januar-Auffüllung | rifornimento, rifornimento di gennaio | |
| Vorausschau | previsione | «anticipazione» steht für den Vorbezug (#182) |
| Renditekriterium | criterio di rendimento | |
| Reservierung | riservazione | |
| Variable Prozententnahme | prelievo percentuale variabile (VPW) | |
| Planungsalter | età di pianificazione | |
| Realrendite | rendimento reale | |
| Immobilie, Objekt | immobile, oggetto | Abschnitt «Immobili» |
| Besitz, Kauf geplant, Miete geplant | proprietà, acquisto pianificato, affitto pianificato | |
| Kaufpreis, Marktwert | prezzo d’acquisto, valore venale | |
| Eigenmittel, Eigenkapital | fondi propri | |
| Nebenkosten | spese accessorie | |
| Verkaufserlös, Nettoerlös | ricavo della vendita, ricavo netto | |
| Notverkauf, Notverkaufskriterium | vendita forzata, criterio di vendita forzata | |
| Immobilien nie verkaufen | non vendere mai gli immobili | |
| Hypothek, Anschlusshypothek | ipoteca, ipoteca successiva | |
| Tranche, Periode, Laufzeit | tranche, periodo, durata | |
| Festhypothek, SARON-Hypothek | ipoteca fissa, ipoteca SARON | |
| Marge | margine | |
| Erneuerung, Rückzahlung | rinnovo, rimborso | |
| Einmalzahlung, Einmalrückzahlung | pagamento unico, rimborso unico | |
| Amortisation (direkt, indirekt) | ammortamento (diretto, indiretto) | |
| Belehnung | grado d’anticipo | |
| Tragbarkeit | sostenibilità | |
| kalkulatorischer Zins | tasso d’interesse teorico | |
| Steuerfuss | moltiplicatore d’imposta | |
| Stufe (Steuertabelle) | scaglione | |
| Steuertabelle | tabella delle aliquote | |
| Einkommens-, Vermögenssteuer | imposta sul reddito, imposta sulla sostanza | |
| Kapitalbezugssteuer | imposta sulle prestazioni in capitale | |
| Grundstückgewinnsteuer | imposta sugli utili immobiliari | |
| Besitzdauer | durata di possesso | |
| Zuschlag, Abzug, Ermässigung | maggiorazione, riduzione | |
| Kanton, Gemeinde | cantone, comune | Modell Zürich/Aargau: modello Zurigo/Argovia |
| Steuerjahr, veranlagt | anno fiscale, tassato | |
| Planung | pianificazione | |
| Projektdatei | file di progetto | |
| Ergebnis, gespeicherte Ergebnisse | risultato, risultati salvati | |
| Lauf, Einzellauf, Einzelne Läufe | simulazione, simulazione singola, simulazioni singole | |
| Pfad | percorso | Zufallspfad: percorso casuale |
| Perzentil | percentile | 5. Perzentil: 5° percentile |
| Median | mediana | |
| Erfolgsquote | tasso di successo | |
| Erschöpfung, aufgebraucht | esaurimento, esaurita | |
| Ruin, Ruinwahrscheinlichkeit | rovina, probabilità di rovina | |
| Schwankung | volatilità | |
| Konsum | consumo | |
| Szenario, Variation, Variante | scenario, variazione, variante | |
| Stufe, Schritt (Variation) | livello, passo | |
| Ereignis | evento | |
| Jahreswerte, Kennzahlen | valori annuali, indicatori | |
| Vorschau | anteprima | |
| Prüfung, Fehler, Hinweis | verifica, errore, avvertenza | |
| Optimierung, Ziel, Nebenbedingung | ottimizzazione, obiettivo, vincolo | |
| Einstellungen | impostazioni | Menü «Impostazioni…» |
| Hilfe, Handbuch, Erste Schritte | aiuto, manuale, primi passi | |
| Tastenkürzel | scelte rapide da tastiera | |
| Datei, Bearbeiten, Ansicht → Sprache | File, Modifica, Vista → Lingua | |
| wie System | come il sistema | Sprachwahl |
| Welt (JST), Schweiz (JST) | Mondo (JST), Svizzera (JST) | Indizes |
| Eidgenossenschaft 10 Jahre / kurzfristig | Confederazione 10 anni / a breve termine | Indizes |

## Übersetzungen

Verbindliche Übersetzung der Fachbegriffe aus Vorsorge, Steuern, Immobilien und Planung in Deutsch, Englisch,
Französisch und Italienisch (#179, Teil von #62). Die Tabellen gelten für die Oberfläche (`crates/app/ui/i18n/*.json`)
und die Hilfe (`crates/app/ui/hilfe/*/`). Sie sind die Grundlage der automatischen Prüfung `make i18n-check` (#180)
und der fachlichen Prüfung (#181, #182). Die Tabellen pro Sprache weiter oben (Oberflächenbegriffe) bleiben die
Arbeitslisten der Übersetzung; bei Widerspruch gilt dieser Abschnitt.

**Format (maschinell lesbar)**

- Zwei Markdown-Tabellen mit denselben sechs Spalten `DE | EN | FR | IT | Quelle | verbotene Varianten`:
  «Kürzel» und «Fachbegriffe». Eine Zeile je Begriff, die Kopfzeile ist genau wie hier geschrieben, die zweite Zeile
  ist der Trenner `|---|…`. Keine Zelle enthält `|`, Markdown-Auszeichnung oder Zeilenumbrüche.
- In den Sprachspalten stehen die erlaubten Formen, getrennt mit `; `. Die erste Form ist die bevorzugte, weitere
  Formen sind gleichwertig erlaubt (Kurzform, amtliche Nebenform, Wortverbindung). In der Spalte DE stehen der Begriff
  und seine deutschen Nebenformen, an denen die Prüfung den Begriff im deutschen Text erkennt.
- Vergleich: Fachbegriffe ohne Rücksicht auf Gross- und Kleinschreibung als Teilzeichenkette (Mehrzahl und
  Zusammensetzungen sind damit eingeschlossen); Kürzel mit Gross- und Kleinschreibung als ganzes Wort. Der
  Apostroph `'` und der typografische Apostroph `’` gelten als gleich. Passen mehrere Begriffe, gilt der längste.
  In den Zielsprachen darf jedes Wort ab vier Buchstaben am Ende um bis zu drei Buchstaben abweichen (Mehrzahl,
  Angleichung: «séries temporelles» erfüllt «série temporelle»). Verbotene Varianten folgen der Regel ihrer Tabelle.
  Text in Code-Schreibweise (`…`), Platzhalter und Link-Ziele zählen nicht.
- Spalte «Quelle»: Angaben getrennt mit `; `, je Angabe ein Kürzel aus der Liste unten mit Artikel und, wo die
  Quelle nicht alle Sprachen belegt, den belegten Sprachen in Klammern, z. B. `BVG Art. 14 (FR, IT)`. Begriffe ohne
  amtliche Quelle beginnen mit `keine amtliche Quelle`; fehlt die amtliche Quelle nur für eine Sprache, steht z. B.
  `EN: keine amtliche Quelle`. Eine Erläuterung folgt nach einem Doppelpunkt.
- Spalte «verbotene Varianten»: Einträge getrennt mit `; `, je Eintrag `<sprache>: <Form>` mit der Sprache als
  Kürzel `en`, `fr` oder `it`. Die Form darf im Text dieser Sprache nicht vorkommen (Teilzeichenkette, ohne Rücksicht
  auf Gross- und Kleinschreibung). Ein einzelnes `-` heisst «keine»; leere Zellen gibt es nicht.
- Gelesen werden die Tabellen mit `tools/glossary.py` (Funktion `parse`, Aufruf `python3 tools/glossary.py`); das
  Format prüft `python3 -m unittest discover -s tools -p 'test_glossary.py'` (Spaltenzahl, leere Zellen,
  Sprachkürzel, Quellen, Mindestbegriffe).

**Quellen** (geprüft am 20.11.2026)

- `AHVG`: Bundesgesetz über die Alters- und Hinterlassenenversicherung, SR 831.10 (fedlex, DE, FR, IT; keine
  englische Fassung).
- `BVG`: Bundesgesetz über die berufliche Alters-, Hinterlassenen- und Invalidenvorsorge, SR 831.40 (fedlex, DE, FR,
  IT; keine englische Fassung).
- `DBG`: Bundesgesetz über die direkte Bundessteuer, SR 642.11 (fedlex, DE, FR, IT; keine englische Fassung).
- `StHG`: Bundesgesetz über die Harmonisierung der direkten Steuern der Kantone und Gemeinden, SR 642.14 (fedlex, DE,
  FR, IT).
- `ZGB`: Schweizerisches Zivilgesetzbuch, SR 210 (fedlex, DE, FR, IT, EN).
- `MB 3.01`: Merkblatt 3.01 «Altersrenten und Hilflosenentschädigungen der AHV» der Informationsstelle AHV/IV, Stand
  1.1.2026 (DE, FR, IT, EN).
- `MB 3.04`: Merkblatt 3.04 «Flexibler Rentenbezug» der Informationsstelle AHV/IV, Stand 1.1.2026 (DE, FR, IT).
- `ISO 4217`: Währungscodes.

TERMDAT war für die Recherche nicht erreichbar; die Begriffe sind an den Gesetzestexten und Merkblättern belegt.
Englisch ist in der Schweiz keine Amtssprache: Wo eine Quelle keine englische Fassung hat, gilt die englische Form
als Festlegung des Projekts (Regeln im Abschnitt «Englisch (Oberflächenbegriffe)»).

### Kürzel

Kürzel, die unübersetzt bleiben oder nur in die Form der Sprache wechseln. Die englische Oberfläche behält die
Schweizer Kürzel (AHV statt der amtlichen englischen Form OASI im MB 3.01).

| DE | EN | FR | IT | Quelle | verbotene Varianten |
|---|---|---|---|---|---|
| AHV | AHV | AVS | AVS | AHVG Titel (DE, FR, IT); MB 3.01: EN amtlich OASI, in VivaPlan nicht verwendet | fr: AHV; it: AHV |
| BVG | BVG | LPP | LPP | BVG Titel (DE, FR, IT); EN: keine amtliche Quelle | fr: BVG; it: BVG |
| PK | PK | CP | CP | keine amtliche Quelle: Kurzform von Pensionskasse | fr: PK; it: PK |
| 3a | 3a | 3a | 3a | keine amtliche Quelle: Kurzform von Säule 3a | - |
| CHF | CHF | CHF | CHF | ISO 4217 | - |
| SNB | SNB | BNS | BNS | keine amtliche Quelle: Eigenname der Schweizerischen Nationalbank | fr: SNB; it: SNB |
| EU | EU | UE | UE | keine amtliche Quelle | fr: EU; it: EU |
| SARON | SARON | SARON | SARON | keine amtliche Quelle: Eigenname | - |
| JST | JST | JST | JST | keine amtliche Quelle: Eigenname | - |
| VPW | VPW | VPW | VPW | keine amtliche Quelle: Eigenname | - |
| StG | StG | StG | StG | keine amtliche Quelle: Steuergesetz, nur in Verweisen | - |

### Fachbegriffe

| DE | EN | FR | IT | Quelle | verbotene Varianten |
|---|---|---|---|---|---|
| Alters- und Hinterlassenenversicherung | old-age and survivors' insurance | assurance-vieillesse et survivants | assicurazione per la vecchiaia e per i superstiti; assicurazione vecchiaia e superstiti | AHVG Titel (DE, FR, IT); MB 3.01 (EN) | - |
| AHV-Rente; Altersrente | AHV pension; old-age pension | rente AVS; rentes AVS; rente de vieillesse; rentes de vieillesse | rendita AVS; rendite AVS; rendita di vecchiaia; rendite di vecchiaia | MB 3.01 | - |
| Vorbezug | early withdrawal; drawing early | anticipation; perception anticipée | anticipazione; riscossione anticipata | MB 3.04 (DE, FR, IT); MB 3.01 (EN); AHVG Art. 40 | - |
| Aufschub | deferral; deferring | ajournement | rinvio | MB 3.04 (DE, FR, IT); MB 3.01 (EN) | - |
| Plafonierung | cap; capped | plafonnement; plafonné | limitazione; limite massimo | MB 3.01; AHVG Art. 35 | - |
| Maximalrente | maximum pension | rente maximale | rendita massima | MB 3.01 | - |
| Ausgleichskasse | compensation office | caisse de compensation | cassa di compensazione | MB 3.01; AHVG | - |
| Beitragsjahre | contribution years; years of contributions | années de cotisation | anni di contribuzione | MB 3.01 | - |
| Hinterlassenenrente | survivors' pension | rente de survivants; rentes de survivants | rendita per superstiti; rendite per superstiti | MB 3.01 | - |
| Witwenrente | widow's pension | rente de veuve | rendita vedovile | keine amtliche Quelle | - |
| Rentenvorausberechnung | pension forecast | calcul anticipé de la rente | calcolo anticipato della rendita | keine amtliche Quelle: im MB 3.01 nicht belegt | - |
| Vorsorgeausweis | pension statement; statement | certificat de prévoyance; certificats de prévoyance | certificato di previdenza; certificati di previdenza | keine amtliche Quelle | - |
| 1. Säule; 2. Säule; 3. Säule | pillar | pilier | pilastro | keine amtliche Quelle | - |
| berufliche Vorsorge | occupational pension | prévoyance professionnelle | previdenza professionale | BVG Titel (FR, IT); EN: keine amtliche Quelle | - |
| Vorsorgevermögen | pension assets | avoirs de prévoyance; avoir de prévoyance | averi di previdenza; avere di previdenza | keine amtliche Quelle | - |
| Pensionskasse | pension fund | caisse de pensions; caisse de pension | cassa pensioni; cassa pensione | BVG (FR, IT); EN: keine amtliche Quelle | - |
| PK-Rente | PK pension | rente CP; rente de la caisse de pension | rendita CP; rendita della cassa pension | keine amtliche Quelle | - |
| Altersguthaben | retirement savings | avoir de vieillesse | avere di vecchiaia | BVG Art. 14 (FR, IT); EN: keine amtliche Quelle | fr: capital de vieillesse; it: capitale di vecchiaia |
| Umwandlungssatz | conversion rate | taux de conversion | aliquota di conversione | BVG Art. 14 (FR, IT); EN: keine amtliche Quelle | en: annuity rate; it: tasso di conversione |
| Kapitalbezug; Kapitalbezüge | lump-sum withdrawal; lump-sum; lump sum | retrait en capital; retraits en capital; prestation en capital | prelievo del capitale; prelievi di capitale; prestazione in capitale; liquidazione in capitale | BVG Art. 37 (FR, IT); DBG Art. 38 (FR, IT); EN: keine amtliche Quelle | - |
| Kapitalbezugssteuer; Kapitalsteuer | lump-sum tax | impôt sur les prestations en capital | imposta sulle prestazioni in capitale | DBG Art. 38 (FR, IT); EN: keine amtliche Quelle | en: capital tax; fr: impôt sur le capital; it: imposta sul capitale |
| Säule 3a | pillar 3a | pilier 3a | pilastro 3a | keine amtliche Quelle | - |
| Einzahlung | contribution | versement | versamento | keine amtliche Quelle: Einzahlung in die Säule 3a; getrennt von der Auszahlung (#182) | - |
| Auszahlung | payout; paid out | retrait; prestation | riscossione; prestazione | keine amtliche Quelle: Auszahlung aus der Säule 3a bzw. Leistung der Pensionskasse; nicht «versement/versamento», das für die Einzahlung steht (#182) | - |
| gebundene Vorsorge | tied pension provision | prévoyance liée; prévoyance individuelle liée | previdenza vincolata; previdenza individuale vincolata | DBG (FR, IT); EN: keine amtliche Quelle | - |
| Pensionierung | retirement | retraite | pensionamento | keine amtliche Quelle | - |
| Haushalt | household | ménage | economia domestica | keine amtliche Quelle | it: nucleo familiare |
| Ehepaar; verheiratet; Verheiratete | married; couple | marié; couple | coniugi; coniugati; coppia sposata; coppie sposate | MB 3.01 | - |
| Erwerbseinkommen | earned income | revenu de l'activité lucrative; activité lucrative | reddito da attività lucrativa; attività lucrativa | keine amtliche Quelle | - |
| Lebenshaltungskosten; Lebenshaltung | living expenses | coût de la vie | costo della vita | keine amtliche Quelle | - |
| Erbschaft | inheritance | succession; héritage | eredità; successione | DBG (FR, IT); EN: keine amtliche Quelle | - |
| Teuerung | inflation | renchérissement; inflation | rincaro; inflazione | DBG Art. 39 (FR, IT); EN: keine amtliche Quelle | - |
| Landesindex der Konsumentenpreise; Landesindex | consumer price index; CPI | indice suisse des prix à la consommation; indice des prix à la consommation | indice nazionale dei prezzi al consumo; indice nazionale | keine amtliche Quelle | - |
| Vermögen | wealth | fortune | sostanza | StHG Art. 2 (FR, IT); EN: keine amtliche Quelle | fr: patrimoine; it: patrimonio |
| Einkommenssteuer | income tax | impôt sur le revenu | imposta sul reddito | DBG Art. 1 (FR, IT); StHG Art. 2 (FR, IT); EN: keine amtliche Quelle | - |
| Vermögenssteuer | wealth tax | impôt sur la fortune; sur la fortune | imposta sulla sostanza; sulla sostanza | StHG Art. 2 (FR, IT); EN: keine amtliche Quelle | en: property tax; it: imposta patrimoniale |
| Grundstückgewinnsteuer | property gains tax; property gains | impôt sur les gains immobiliers; gains immobiliers | imposta sugli utili immobiliari; utili immobiliari; utili da sostanza immobiliare | StHG Art. 2 (FR, IT); EN: keine amtliche Quelle | en: capital gains tax; fr: impôt sur la plus-value |
| Eigenmietwert | imputed rental value | valeur locative | valore locativo | DBG Art. 21 (FR, IT); EN: keine amtliche Quelle | en: own rental value |
| Steuerfuss | tax multiplier | coefficient d'impôt | moltiplicatore d'imposta | keine amtliche Quelle | - |
| Steuertabelle | tax table | barème | tabella delle aliquote; tabelle delle aliquote; tabella fiscale; tabelle fiscali | keine amtliche Quelle | - |
| Steuersatz | tax rate; rate | taux | aliquota | keine amtliche Quelle | - |
| Freigrenze | exemption limit | seuil d'exonération | soglia di esenzione | keine amtliche Quelle: bei Erreichen ist der ganze Gewinn steuerbar; nicht «franchise/franchigia» (Freibetrag) (#182) | fr: franchise; it: franchigia |
| Grenzsteuersatz | marginal tax rate | taux marginal | aliquota marginale | keine amtliche Quelle | - |
| Kanton | canton | canton | cantone; cantoni; cantonale; cantonali | StHG Art. 2 (FR, IT); EN: keine amtliche Quelle | - |
| Gemeinde | municipality | commune | comune | StHG Art. 2 (FR, IT); EN: keine amtliche Quelle | - |
| Besitzdauer | holding period | durée de possession | durata di possesso | keine amtliche Quelle | - |
| Steuerjahr | tax year | année fiscale | anno fiscale | keine amtliche Quelle: DBG spricht von Steuerperiode | - |
| Immobilie | property; properties; real estate | bien immobilier; immobilier; immobilière | immobile; immobili | keine amtliche Quelle | - |
| Hypothek | mortgage | hypothèque; hypothécaire | ipoteca; ipoteche; ipotecari; ipotecaria | ZGB Art. 824 | en: hypothec |
| Anschlusshypothek | follow-up mortgage | hypothèque subséquente | ipoteca successiva | keine amtliche Quelle | - |
| Festhypothek | fixed | taux fixe | fissa; fisso | keine amtliche Quelle | - |
| Hypothekarzins | mortgage interest | intérêts hypothécaires; intérêt hypothécaire; taux hypothécaire | interessi ipotecari; interesse ipotecario; tasso ipotecario | keine amtliche Quelle | - |
| Amortisation | amortization | amortissement | ammortamento | keine amtliche Quelle: Begriff der Banken | en: amortisation |
| Belehnung | loan-to-value | taux d'avance | grado d'anticipo | keine amtliche Quelle: Begriff der Banken | - |
| Tragbarkeit | affordability | capacité financière | sostenibilità | keine amtliche Quelle: Begriff der Banken | - |
| kalkulatorischer Zins | imputed interest rate | taux d'intérêt théorique | tasso d'interesse teorico | keine amtliche Quelle: Begriff der Banken | - |
| Eigenmittel | equity | fonds propres | fondi propri | keine amtliche Quelle | - |
| Nebenkosten | fixed costs | frais accessoires | spese accessorie | keine amtliche Quelle | - |
| Marktwert | market value | valeur vénale | valore venale | keine amtliche Quelle | - |
| Verkaufserlös; Nettoerlös | proceeds | produit | ricavo; ricavi | keine amtliche Quelle | - |
| Notverkauf | emergency sale | vente forcée | vendita forzata | keine amtliche Quelle | - |
| Aktien | stocks; stock | actions | azioni; azionario; azionaria; azionari | keine amtliche Quelle | - |
| Obligationen | bonds; bond | obligations; obligataire | obbligazioni; obbligazionario | keine amtliche Quelle | - |
| Aktienanteil; Aktienquote | stock allocation | part d'actions; quote-part d'actions | quota azionaria | keine amtliche Quelle | - |
| Anlagestrategie | investment strategy | stratégie de placement | strategia d'investimento | keine amtliche Quelle | - |
| Realrendite | real return | rendement réel | rendimento reale | keine amtliche Quelle | - |
| Rendite | return; yield | rendement | rendimento; rendimenti | keine amtliche Quelle | - |
| Volatilität; Schwankung | volatility; fluctuation | volatilité; fluctuation | volatilità | keine amtliche Quelle | - |
| Korrelation | correlation | corrélation | correlazione | keine amtliche Quelle | - |
| Rebalancing | rebalancing | rééquilibrage | ribilanciamento | keine amtliche Quelle | - |
| Währungsabsicherung | currency hedging | couverture de change | copertura valutaria | keine amtliche Quelle | - |
| Sequenzrisiko | sequence risk | risque de séquence | rischio di sequenza | keine amtliche Quelle | - |
| Bootstrap | bootstrap | bootstrap | bootstrap | keine amtliche Quelle: Fachbegriff der Statistik | - |
| Zeitreihe | time series | série temporelle | serie temporale | keine amtliche Quelle | - |
| Cash-Reserve | cash reserve | réserve de cash | riserva di cash | keine amtliche Quelle | - |
| Entnahmestrategie | withdrawal strategy | stratégie de prélèvement | strategia di prelievo | keine amtliche Quelle | - |
| Entnahmerate | withdrawal rate | taux de prélèvement | tasso di prelievo | keine amtliche Quelle | - |
| Entnahme | withdrawal | prélèvement | prelievo | keine amtliche Quelle | - |
| Leitplanke | guardrail | garde-fou | guardrail | keine amtliche Quelle | - |
| Variable Prozententnahme | variable percentage withdrawal | prélèvement en pourcentage variable | prelievo percentuale variabile | keine amtliche Quelle | - |
| 3-Topf-Strategie | 3-bucket strategy | stratégie à 3 compartiments | strategia a 3 comparti | keine amtliche Quelle | - |
| Topf | bucket | compartiment | comparto; comparti | keine amtliche Quelle | - |
| Auffüllung | refill | réalimentation | rifornimento; rifornimenti | keine amtliche Quelle | - |
| Vorausschau | look-ahead | prévision | previsione | keine amtliche Quelle: Prüfung der 3-Topf-Strategie im laufenden Monat; nicht «anticipation», das für den Vorbezug steht (#182) | - |
| Konsum | consumption | consommation | consumo | keine amtliche Quelle | - |
| Monte-Carlo | Monte Carlo | Monte-Carlo; Monte Carlo | Monte Carlo | keine amtliche Quelle: Fachbegriff der Statistik | - |
| Monte-Carlo-Erfolgsquote; Erfolgsquote | success rate | taux de réussite | tasso di successo | keine amtliche Quelle | - |
| Ruinwahrscheinlichkeit | probability of ruin | probabilité de ruine | probabilità di rovina | keine amtliche Quelle | - |
| Erschöpfung | depletion | épuisement | esaurimento | keine amtliche Quelle | - |
| Perzentil | percentile | percentile | percentile; percentili | keine amtliche Quelle | - |
| Median | median | médiane | mediana; mediano | keine amtliche Quelle | - |
| Einzellauf | single run | simulation unique; simulations uniques | simulazione singola; simulazioni singole | keine amtliche Quelle | - |
| Pfad | path | trajectoire | percorso; percorsi | keine amtliche Quelle | - |
| Szenario | scenario | scénario | scenario | keine amtliche Quelle | - |
| in heutigen Preisen | in today's prices | aux prix actuels | ai prezzi attuali | keine amtliche Quelle | - |
| heutige Kaufkraft | today's purchasing power | pouvoir d'achat actuel | potere d'acquisto attuale | keine amtliche Quelle | - |
| Optimierung | optimization | optimisation | ottimizzazione | keine amtliche Quelle | en: optimisation |
| Nebenbedingung | constraint | contrainte | vincolo; vincoli | keine amtliche Quelle | - |
| Meilenstein | milestone | jalon | tappa; tappe | keine amtliche Quelle | - |
| Planungsalter | planning age | âge de planification | età di pianificazione | keine amtliche Quelle | - |
| Simulation | simulation | simulation | simulazione; simulazioni | keine amtliche Quelle | - |
