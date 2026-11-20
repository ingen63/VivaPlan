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
| Einzahlung, Auszahlung (3a) | versement, versement (retrait) | Auszahlungsplan: plan de retrait |
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
| Vorausschau | anticipation | |
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
