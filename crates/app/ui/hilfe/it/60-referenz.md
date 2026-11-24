# Riferimento

## @ref-shortcuts Scelte rapide da tastiera
Parole chiave: tastiera, shortcut, scorciatoia, Ctrl, comando, F1

Sul Mac vale ⌘ (comando), con Windows e Linux **Ctrl**. ⇧ è il tasto maiuscole.

| Scelta rapida | Effetto |
|---|---|
| **F1** | Aiuto sul tema attuale (campo, sezione o grafico dei risultati) |
| ⌘⇧? | Manuale (indice) |
| Esc | Chiudere l’aiuto; altrimenti tornare all’editor |
| ⌘N | Nuova pianificazione |
| ⌘O | Aprire |
| ⌘S | Salvare |
| ⇧⌘S | Salvare con nome |
| ⌘I | Importare il vecchio `config.json` |
| ⌘↩ | Avviare la simulazione |
| ⌘Z | Annullare |
| ⇧⌘Z | Ripristinare |
| ⌘1 … ⌘9, ⌘0 | Sezione da 1 a 10 nell’editor, nei risultati la scheda |
| ⌘↑ e ⌘↓ | sezione precedente o successiva |
| ⌘/ | Panoramica delle scelte rapide da tastiera |

Su alcune tastiere Mac F1 si trova sul tasto della luminosità. Prema allora **fn+F1** oppure usi il pulsante «? Aiuto» o ⌘⇧?.

## @ref-checks Verifica: errori e avvertenze
Parole chiave: verifica, errore, avvertenza, avviso, convalida

A destra nell’editor l’app verifica continuamente la pianificazione.

- **Errori (rosso):** la pianificazione è incompleta o contraddittoria. «Avviare la simulazione» non è allora possibile. Un clic sull’errore porta alla sezione interessata. **«Avviare la simulazione» e «Avviare l’ottimizzazione» sono bloccati** finché vi sono errori: il tentativo mostra «La pianificazione ha N errori …» e porta alla prima sezione interessata. I **campi obbligatori** (ad esempio nome e data di nascita di ogni persona) recano un asterisco (*); un errore appare direttamente sul campo, così come un valore non leggibile (ad esempio una data vuota). Finché digita in un campo, **tutto attende**: non appare alcun messaggio, e anteprima, valori calcolati ed elenco di verifica restano fermi. Solo quando lascia il campo (Tab, clic accanto o Invio) viene ricalcolato e un messaggio viene mostrato o rimosso. Un valore digitato a metà come ‹6› sulla strada verso ‹65› non viene così segnalato. Elenchi di selezione e caselle di controllo hanno effetto subito. L’elenco completo dei dati obbligatori si trova nella specifica `docs/Pflichteingaben.md`.
- **Avvertenze (giallo):** la simulazione funziona, ma un punto merita la Sua attenzione, ad esempio una serie temporale in un’altra valuta o la somma di più prelievi di capitale.

Errori tipici: inizio dopo la fine, pensionamento al di fuori del periodo, più di due persone, una serie temporale mancante con una fonte storica, un parametro non valido di una strategia. Il messaggio indica la causa.

La riga **Anteprima** a destra calcola una simulazione con rendimenti fissi. Mostra dove portano i dati, ma non sostituisce la simulazione.

## @ref-file Il file di pianificazione (.vplan)
Parole chiave: formato, JSON, file, salvare, schema, file di progetto

Una pianificazione è **un file di testo** in formato JSON con l’estensione `.vplan`. Si può copiare, archiviare e visualizzare con un editor di testo («Visualizzare il file» nell’editor mostra il contenuto).

- Il formato è contrassegnato con `"format": "vivaplan/1"`. Le pianificazioni meno recenti con l’estensione `.rplan` e l’identificativo `pensionsplaner/1` si possono ancora aprire; il salvataggio avviene sempre come `.vplan` (il vecchio file resta invariato).
- I **momenti** sono mesi civili (`"2027-05"`) o riferimenti a tappe (`"retire"`), `"never"` significa mai.
- Gli **importi** valgono al mese, le **aliquote** all’anno come numero decimale (0.05 = 5 %).
- I **valori dipendenti dal tempo** sono un numero oppure `{"mode": "timed", "rows": [{"from": "start", "value": 9500}, {"from": "retire", "value": 8500}]}`.
- **Altre forme** ([Valori dipendenti dal tempo](#f-timed)): retta `{"mode": "linear", "start": {"age": 55, "value": 0.9}, "end": {"age": 65, "value": 0.5}}`, tabella `{"mode": "table", "interpolation": "step", "points": [{"at": "start", "value": 1}, {"age": 80, "value": 0.8}]}` (`interpolation` `linear` o `step`, senza indicazione `linear`), fattore `{"mode": "factor", "base": 6000, "factor": <andamento>}`. Un punto d’appoggio ha `at` (data o tappa) oppure `age` (età, con `person`, altrimenti della prima persona).
- **Curva caratteristica** (solo `wealth.stockShare`, [Curva caratteristica](#f-curve)): `{"mode": "curve", "input": "withdrawalRate", "points": [[0.02, 0.8], [0.06, 0.4]], "eval": "yearly"}`; `input` è `age`, `liquidReal`, `withdrawalRate`, `fundedRatio`, `stockReturn12m` o `drawdown`, `eval` `yearly` (predefinito) o `monthly`.
- **Ottimizzazione:** obiettivo, strategie, percorsi e le **fasce dei parametri** per strategia si trovano sotto `optimization` (`ranges`: strategia, parametro, poi `fixed` o `lo`/`hi`, quote come numero decimale).
- **Imposta sugli utili immobiliari:** `taxes.propertyGain` con `brackets` (ampiezza, aliquota), `duration` (da anni interi, maggiorazione o riduzione), `exemptBelow` e `reduction`; la data d’acquisto di un immobile in proprietà si trova in `bought`. Nel modo «Domicilio» vale la regola del cantone dell’immobile (`canton` per immobile, senza indicazione il cantone di domicilio); `taxes.propertyGainOwn: true` passa alle regole proprie.
- **Domicilio:** `taxes.location` (`canton`, `municipality` come numero UST, `church`) vale dall’inizio; altri domicili si trovano in `taxes.relocations` come righe `{"from": "2031-01", "canton": "ZG", "municipality": 1711, "church": "none"}`.
- Il formato completo lo fornisce `vivaplan schema` nel terminale.

Il file **non contiene risultati**. L’app archivia i risultati nell’[elenco dei risultati](#r-list).

## @ref-cli Riga di comando
Parole chiave: terminale, vivaplan, comando, script, CLI

Il motore di calcolo è disponibile anche nel terminale (programma `vivaplan`):

| Comando | Scopo |
|---|---|
| `run <pianificazione>` | calcolare la pianificazione (`--csv`, `--json`, `--events`, `--metric`, vedi [Indicatori propri](#ref-metric)) |
| `validate <pianificazione>` | verificare la pianificazione (`--json`) |
| `set <pianificazione> <percorso>=<valore>…` | modificare valori per percorso di campo, vedi [Modificare valori](#ref-set) |
| `diff <a> <b>` | elencare le differenze tra due pianificazioni (`--json`) |
| `optimize <pianificazione>` | ottimizzare la strategia, vedi [Riga di comando](#o-cli) |
| `evaluate <pianificazione> <esperimento.json>` | Calcolare un esperimento (`--json`, `--yes`), vedi [Esperimento](#ref-experiment) |
| `bench <pianificazione>` | misurare il tempo di calcolo (`--paths`, `--json`) |
| `schema` | emettere lo schema JSON del file di pianificazione |
| `library` | elencare le serie temporali della biblioteca di dati |
| `import-shiller <file>` | reimportare i dati Shiller (.xls, .xlsx, .ods) |
| `import-snb <file>…` | importare i cubi BNS (devkum, plkopr, zimoma, rendoblim) |
| `import-legacy <config.json>` | riprendere la configurazione del programma precedente come punto di partenza |
| `ui-dev` | interfaccia nel browser, solo in locale (sviluppo) |

La riga di comando emette i suoi testi, messaggi ed eventi in inglese, indipendentemente dalla lingua dell’interfaccia; numeri e date sono nel formato svizzero.

## @ref-json Output in JSON
Parole chiave: JSON, leggibile da macchina, IA, script, output standard, format

Con `--json` senza nome di file, `run`, `validate`, `optimize`, `bench` e `diff` emettono il risultato in un formato fisso sull’output standard, per script e assistenti IA. Avvisi ed errori vanno sull’output degli errori; in caso di errore il comando termina con un codice d’errore. `--json <file.json>` salva come finora il risultato dettagliato in un file.

Ogni output inizia con `format` e la versione. Gli importi sono arrotondati a unità intere della valuta, le quote a quattro decimali (0.9512 = 95,12 %), i mesi sono indicati come `"AAAA-MM"`. I messaggi hanno `code`, `params` e il `text` inglese; i messaggi di verifica hanno inoltre `severity`, `section` e `field` (percorso di campo).

| `format` | Contenuto |
|---|---|
| `vivaplan-run/1` | `error` (o `null`), `issues`, `elapsedMs` e, secondo il metodo, `single`, `batch` e `scenarios`; con `--metric` anche `metrics` |
| `vivaplan-validate/1` | `valid` (nessun errore), `issues` |
| `vivaplan-optimize/1` | impostazioni, `reference` (la pianificazione così com’è) e `outcomes` (la strategia migliore per prima, con `params`, `validated`, `feasible` e `patch`) |
| `vivaplan-bench/1` | `runs`, `months`, `elapsedMs`, `usPerRun`, `cores`, `check` |
| `vivaplan-diff/1` | `changes`: per ogni differenza `path`, `old`, `new` |

**`single`** (un percorso di rendimento): indicatori della simulazione (`endTotal`, `endTotalReal`, `totalAtRetirement`, `depleted`, `minLiquid`, `taxesTotal` …), `years` come tabella con `columns` e una riga per anno (`year`, `age`, `liquid`, `propertyEquity`, `pensionFund`, `pillar3a`, `total`: stato a fine anno; `income`, `spending`, `taxes`: somma dell’anno) ed `events` come coppie di mese e testo.

**`batch`** (più percorsi): `runs`, `successRate`, `earliestDepletion`, `percentiles` (sostanza finale a 0 = simulazione peggiore, 1, 5, 10, 25, 50, 75, 90, 95 e 99 %) e `years` con `p5` fino a `p95` della sostanza totale, la mediana reale `p50Real` e la quota delle simulazioni con sostanza residua, `survival`.

**`scenarios`** (variazioni): `params` e per ogni combinazione `values`, `successRate`, `p5`, `p50`, `p95`, `p50Real`, `atRetirement`, `earliestDepletion`.

Per la pianificazione d’esempio l’output di `run --json` conta meno di 20 000 caratteri.

## @ref-set Modificare valori per percorso di campo
Parole chiave: set, diff, percorso di campo, modificare, variante, IA, confrontare

`vivaplan set <pianificazione> <percorso>=<valore> …` modifica valori di una pianificazione senza modificare il file a mano. Un **percorso di campo** indica le chiavi del file di pianificazione separate da punti, le voci di un elenco con il loro numero a partire da 0: `wealth.stockShare`, `household.persons.0.name`, `milestones.1.years`. Sono gli stessi percorsi che i messaggi di verifica indicano sotto `field`.

- Il **valore** è letto come JSON (`0.6`, `true`, `null`, `"testo"`, `{…}`); tutto il resto vale come testo, per esempio `project.name=Variante B` o `returns.startDate=1990-01`.
- **Momento** per i valori dipendenti dal tempo: `percorso@<momento>=<numero>` modifica il gradino a partire da questo momento; se non ce n’è, lì ne inizia uno nuovo. Un momento è una tappa (`@retire`), un’età della prima persona (`@age:62.5`) o un mese (`@2035-01`). Esempio: `wealth.stockShare@start=0.8 wealth.stockShare@retire=0.5` imposta la quota azionaria prima e dopo il pensionamento; un valore costante diventa così dipendente dal tempo. Una retta, una tabella, un fattore o una curva caratteristica non ammette un momento; `set` segnala allora un errore.
- **Verifica:** un percorso sconosciuto, un tipo errato o un momento sconosciuto danno un messaggio, e **nessuna** delle modifiche viene ripresa. Gli altri messaggi di verifica della pianificazione modificata appaiono come avvisi; il file viene comunque scritto.
- **File:** senza `-o`, `set` scrive un nuovo file accanto a quello vecchio, con il prossimo numero libero (`plan-1.vplan`, `plan-2.vplan`, …), e ne emette il nome. Con `-o <file>` scrive lì. `set` sostituisce un file esistente solo con `--overwrite`, dopo una copia di sicurezza `<file>.bak`. `--overwrite` senza `-o` sostituisce la pianificazione stessa.

`vivaplan diff <a> <b>` elenca per ogni differenza il percorso di campo, il valore vecchio e quello nuovo, per esempio `withdrawal.floor: 0 → 0.8`; con `--json` nel formato `vivaplan-diff/1`.

## @ref-metric Indicatori propri (--metric)
Parole chiave: indicatore, metric, misura, quantile, mediana, probabilità, finestra, consumo, IA

`vivaplan run <pianificazione> --metric '<json>'` calcola inoltre indicatori che Lei compone da elementi, sugli stessi percorsi di rendimento di `run` (senza variazioni). `--metric` si può indicare più volte e accetta un indicatore o un elenco di indicatori. Con `--json` i valori si trovano sotto `metrics`, altrimenti alla fine del rapporto. Gli indicatori vengono raccolti durante la simulazione; a tale scopo non vengono salvati valori mensili.

| Elemento | Chiave | Scelta |
|---|---|---|
| Grandezza | `quantity` | un campo dei valori mensili (`total`, `liquid`, `income`, `spending`, `taxes`, `withdrawal`, `housing`, `mortgage`, `pensionFund`, `age` …) o derivata: `consumption` (spese; dopo l’esaurimento solo quanto coperto dal reddito), `plannedConsumption`, `depleted` (1 dall’esaurimento), `atFloor` (1 quando il minimo determina le spese), i contatori `emergencySales`, `lumpSums`, `affordabilitySales`, `unaffordableRenewals` |
| Unità | `real` | `true`: ai prezzi del mese iniziale (solo importi); altrimenti nominale |
| Finestra | `window` | `from`, `to` (incluso): `start`, `retirement`, `end`, identificativo di una tappa o `AAAA-MM`; `fromAge`, `toAge`: età della prima persona; `years`: solo i primi n anni. Senza indicazione l’intera durata |
| Riduzione per percorso | `reduce` | `annualSums` (somma per 12 mesi, una serie annuale), `sum`, `mean`, `min`, `max`, `atStart`, `atEnd`, `firstWhen` (età nel primo mese che soddisfa la condizione), `share` (quota di mesi che soddisfano la condizione), `any`, `volatility` (variazione media delle somme annuali), `shareOfStartLiquid` (somma rispetto alla sostanza liquida prima della finestra) |
| Aggregazione sui percorsi | `aggregate` | `{"kind": "quantile", "q": 0.5}`, `mean`, `min`, `max`, `probability` e `count` (con `condition`), `conditionalMean` (con `lowest` o `highest` come quota, oppure `condition`) |

Una condizione si scrive per esempio `{"op": "lt", "value": 50000}` (`gt`, `ge`, `lt`, `le`); senza indicazione vale «maggiore di 0», cioè «sì» per `depleted` e `atFloor`. `condition` accanto a `reduce` verifica i mesi (`firstWhen`, `share`, `any`), `condition` in `aggregate` i valori per percorso. Con `annualSums` l’aggregazione prende insieme tutti i valori annuali di tutti i percorsi, come l’ottimizzazione. `name` dà il nome all’indicatore nel risultato.

Ogni voce di `metrics` contiene `name`, `value` (o `null`), `values` (numero di valori), l’errore standard `stdError` per le probabilità e le medie per percorso, il percorso `path` per quantile, minimo e massimo, nonché la definizione `metric`. Gli errori negli indicatori vengono riuniti in un messaggio con l’elenco; in tal caso non si calcola nulla. `vivaplan schema` descrive la pianificazione, non gli indicatori.

Esempi:

```
--metric '{"name": "Consumo anni 1–15, mediana", "quantity": "consumption", "real": true, "window": {"from": "retirement", "years": 15}, "reduce": "annualSums", "aggregate": {"kind": "quantile", "q": 0.5}}'
--metric '{"name": "Rovina fino a 85 anni", "quantity": "depleted", "window": {"toAge": 85}, "reduce": "any", "aggregate": {"kind": "probability"}}'
--metric '{"name": "Quota di mesi al minimo", "quantity": "atFloor", "window": {"from": "retirement"}, "reduce": "share", "aggregate": {"kind": "mean"}}'
```

Mediana e P10 di `consumption` (reale, intera durata da `retirement`) sono gli stessi valori che in `optimize --compare`.

## @ref-experiment Esperimento (evaluate)
Parole chiave: esperimento, evaluate, variazione, griglia, ricerca per pattern, vincolo, obiettivo, budget, IA

`vivaplan evaluate <pianificazione> <esperimento.json> [--json] [--yes]` pone una domanda alla pianificazione come una struttura JSON e la calcola in una chiamata: modifiche, variazioni, [indicatori](#ref-metric), vincoli e obiettivo. Tutto è JSON strutturato senza formule nelle stringhe; lo schema si trova sotto `definitions.Experiment` in `vivaplan schema`.

| Chiave | Contenuto |
|---|---|
| `set` | Modifiche per tutte le varianti: percorso di campo → valore, come con [`set`](#ref-set), anche con un momento (`"wealth.stockShare@retire": 0.4`) |
| `vary` | Variazioni, ognuna combinata con ognuna: `{"path", "values": […]}` (elenco), `{"path", "range": [da, a, passo]}` (intervallo), `{"path", "follows", "offset"}` (accoppiata: valore dell’altro percorso più `offset`) oppure `{"group": [{"set", "vary"}, …]}` (gruppo: i membri non vengono combinati tra loro) |
| `metrics` | Indicatori come con `--metric`, ognuno con un `name` |
| `constraints` | Vincoli `{"metric", "op", "value"}` con `op` = `le`, `lt`, `ge`, `gt` |
| `objective` | esattamente uno: `{"maximize": "<indicatore>"}`, `{"minimize": "<indicatore>"}`, `{"weighted": [{"metric", "weight"}, …]}` (massimizzare la somma ponderata) oppure `{"score": {…}}` (obiettivo dell’[ottimizzazione](#o-cli) con `maxRuin`, `weightMedian`, `volWeight` …). Senza obiettivo vengono elencate tutte le varianti (al massimo 50) |
| `search` | `method`: `grid` (griglia, tutte le varianti; predefinito) oppure `pattern` (ricerca per pattern negli intervalli come l’ottimizzazione, per ogni combinazione degli altri valori; `samples` punti di partenza casuali, predefinito 30; `seed`) |
| `budget` | `paths` percorsi di ricerca (1'000), `margin` margine sui vincoli in errori standard (2), `top` numero di varianti confermate (5), `confirmPaths` percorsi di conferma (10'000) |
| `confirm` | `true`: calcolare anche se il calcolo è lungo |

**Svolgimento.** Prima viene verificato l’intero esperimento: gli errori arrivano come elenco con il percorso nell’esperimento (per esempio `vary[0].values[1]`), il motivo e i valori validi, e non viene calcolato nulla. Poi viene stimata la durata; da 60 secondi `evaluate` calcola solo con `--yes` (o `"confirm": true`) e altrimenti indica la stima. La ricerca usa pochi percorsi; un vincolo deve esservi rispettato con il margine. Le `top` varianti migliori vengono confermate su percorsi **indipendenti** (altro seme); la classifica segue i valori confermati. Tutte le varianti calcolano sugli stessi percorsi, così le differenze tra loro sono più precise dei singoli valori. Con percorsi fissi (storico, tutte le date di inizio) non esistono percorsi indipendenti; un avviso lo segnala.

**Risultato** (con `--json` sotto `result`): `ranking` con `rank`, i valori della variante (`settings`), `search` e `confirm` (ognuno `objective`, `feasible`, `figures` con `value` e `stdError`; con `score` anche `optimization`), `hints` (per esempio al bordo della griglia: la variante migliore si trova al bordo di un intervallo), `paths` (numero e semi delle due fasi, `independent`), l’`experiment` completo con tutti i valori predefiniti e la `version` del motore di calcolo. `status` è `done`, `invalid`, `needsConfirmation`, `cancelled` o `failed`; `estimate` indica varianti, simulazioni e secondi. L’output resta breve (meno di 20'000 caratteri).

Esempio: 10 anni fino al pensionamento, quota azionaria prima 40, 60 o 80 % e dopo 30 o 50 %, prelievo a tasso fisso del 3, 4 o 5 % oppure endowment 3 o 4 %, il consumo reale più alto nei primi 15 anni di pensione con al massimo il 2 % di rovina – 30 varianti:

```
{
  "vary": [
    { "path": "wealth.stockShare@start", "values": [0.4, 0.6, 0.8] },
    { "path": "wealth.stockShare@retire", "values": [0.3, 0.5] },
    { "group": [
      { "set": { "withdrawal.strategy": "fixed_rate" },
        "vary": [ { "path": "withdrawal.fixedRate.rate", "values": [0.03, 0.04, 0.05] } ] },
      { "set": { "withdrawal.strategy": "endowment" },
        "vary": [ { "path": "withdrawal.endowment.rate", "values": [0.03, 0.04] } ] }
    ] }
  ],
  "metrics": [
    { "name": "ruin", "quantity": "depleted", "reduce": "any", "aggregate": { "kind": "probability" } },
    { "name": "cons15", "quantity": "consumption", "real": true, "window": { "from": "retire", "years": 15 },
      "reduce": "annualSums", "aggregate": { "kind": "mean" } }
  ],
  "constraints": [ { "metric": "ruin", "op": "le", "value": 0.02 } ],
  "objective": { "maximize": "cons15" }
}
```

Variazioni, confronto delle strategie e ottimizzazione si formulano così: le variazioni sono una griglia senza obiettivo; il confronto delle strategie è un gruppo delle strategie con `"objective": {"score": {}}` e `budget.paths` = percorsi della pianificazione; l’ottimizzazione è una ricerca per pattern con `score` e `"margin": 1.5`.

## @ref-library Biblioteca di dati: cartelle e file
Parole chiave: cartella, serie temporali, luogo di salvataggio, Application Support, CSV

L’app desktop gestisce la biblioteca nella cartella dei dati dell’app e vi crea le serie fornite mancanti. La [strategia d’investimento](#f-ret-series) ne usa ‹Shiller CHF›, ‹Shiller CHF + CH-Obligationen› e i dati BNS nella sottocartella `quellen`, così che i dati Shiller e BNS aggiornati abbiano effetto; se manca un file, vale quello integrato. L’app non usa più file propri nella cartella. Con macOS: `~/Library/Application Support/ch.vivaplan-app.desktop/Zeitreihen`. Le pianificazioni rimandano con `library:Name.csv` a una serie della biblioteca, indipendentemente da dove la pianificazione è salvata.

L’app non completa da sé una biblioteca già esistente con le serie fornite nuove. Se necessario, le copi nella cartella. Gli indici si trovano sotto [Strategia d’investimento](#f-ret-series).

## @glossar Glossario
Parole chiave: termini, lessico, significato, abbreviazioni

**AVS:** assicurazione vecchiaia e superstiti, 1° pilastro. L’app calcola solo la rendita di vecchiaia. Vedi [AVS](#f-ahv).

**Avere di vecchiaia:** il capitale risparmiato presso la cassa pensione. Vedi [Cassa pensione](#f-pk).

**Annualità:** pagamenti costanti che consumano un capitale in un determinato periodo. Base del [VPW](#f-wd-vpw).

**Bootstrap:** procedimento per comporre in modo casuale nuovi andamenti da dati reali. Vedi [Bootstrap](#f-ret-bootstrap).

**Tasso di successo:** quota delle simulazioni in cui la sostanza basta fino alla fine. Vedi [Panoramica](#r-overview).

**Fat tails:** ‹code spesse› della distribuzione dei rendimenti: gli eventi estremi sono più frequenti che con una distribuzione normale. Vedi [modello parametrico](#f-ret-parametric).

**Guardrails:** guardrail per il prelievo. Vedi [Guardrails](#f-wd-guardrails).

**Prelievo del capitale:** versamento del capitale di previdenza come importo unico invece che come rendita, con imposta sulle prestazioni in capitale. Vedi [Quota del prelievo del capitale](#f-pk-lumpsum).

**Mediana:** valore centrale: metà dei valori è superiore, metà inferiore.

**Monte Carlo:** calcolo di molti andamenti casuali per stimare probabilità. Vedi [Simulazione](#s-returns).

**Nominale / reale:** nominali sono gli importi in franchi futuri, reali quelli al potere d’acquisto di oggi. Vedi [Rendimento, inflazione e valori reali](#ret-concepts).

**Percentile:** il percentile *n* è il valore al di sotto del quale si trova l’*n* % dei valori. Vedi [Sostanza finale per percentile](#r-percentiles).

**Cassa pensione (CP, LPP):** previdenza professionale, 2° pilastro. Vedi [Cassa pensione](#f-pk).

**Ribilanciamento:** ripristino della ripartizione desiderata tra azioni e obbligazioni. Vedi [Quota azionaria](#f-stockshare).

**Rovina:** la sostanza (liquida e immobili) è esaurita. Vedi [Ottimizzazione: obiettivo](#o-goal).

**SARON:** tasso d’interesse di riferimento per le ipoteche in Svizzera. Vedi [SARON](#ret-saron).

**Pilastro 3a:** previdenza privata vincolata, un conto per persona. Vedi [Pilastro 3a](#s-pillar3a).

**Seed (valore iniziale casuale):** valore iniziale del generatore casuale che rende i risultati riproducibili. Vedi [Valore iniziale casuale](#ret-seed).

**Rischio di sequenza:** il rischio che gli anni di borsa negativi arrivino proprio all’inizio della fase di prelievo. Vedi [Insidie](#ret-pitfalls).

**Sostenibilità:** regola bancaria secondo cui i costi ipotecari non possono superare una quota del reddito. Vedi [Sostenibilità](#f-re-afford).

**Tranche:** parte di un’ipoteca con durata e interesse propri. Vedi [Ipoteche](#f-re-mortgage).

**Aliquota di conversione:** quota del capitale di rendita versata annualmente come rendita. Vedi [Aliquota di conversione](#f-pk-conversion).

**VPW:** prelievo percentuale variabile. Vedi [VPW](#f-wd-vpw).

**Volatilità:** misura della variabilità dei rendimenti, di solito come deviazione standard all’anno. Vedi [modello parametrico](#f-ret-parametric).
