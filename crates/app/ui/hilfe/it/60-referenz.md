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
- **Domicilio:** `taxes.location` (`canton`, `municipality` come numero UST, `church`) vale dall’inizio, con `taxes.mode: "location"` (nuove pianificazioni; senza `mode` valgono le tabelle proprie); altri domicili si trovano in `taxes.relocations` come righe `{"from": "2031-01", "canton": "ZG", "municipality": 1711, "church": "none"}`.
- **Reddito imponibile:** `taxes.investmentYield` con `stocks` e `bonds` come tasso annuo (0.02 = 2 %); se manca, vale 0 %.
- **Valore fiscale predefinito:** `realestate.defaultTaxValue: true` (nuove pianificazioni); gli immobili senza `taxValue` contano allora al 50 % del valore venale. Se il campo manca, vale il valore venale.
- **Origine fattura d’imposta:** le righe di `taxes.deductions` riprese dalla fattura d’imposta ([Deduzioni dalla fattura d’imposta](#f-tax-bill)) portano `taxBill` con l’anno fiscale, per esempio `{"from": "start", "kind": "own", "amount": 25000, "taxBill": 2025}`; ciò non cambia il calcolo.
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
| `describe [<sezione>]` | Catalogo per le IA: cosa si può modificare, variare e misurare (`--plan`, `--json`), vedi [Catalogo per le IA](#ref-describe) |
| `mcp` | Server MCP per i programmi di IA, legge, calcola, modifica e salva (`--data-dir`, `--library`, `--names`, `--read-only`, `--allow-dir`), vedi [Collegare programmi di IA](#ref-mcp) |
| `evaluate <pianificazione> <esperimento.json>` | Calcolare un esperimento (`--json`, `--yes`), vedi [Esperimento](#ref-experiment) |
| `bench <pianificazione>` | misurare il tempo di calcolo (`--paths`, `--json`) |
| `schema` | emettere lo schema JSON del file di pianificazione |
| `library` | elencare le serie temporali della biblioteca di dati |
| `import-shiller <file>` | reimportare i dati Shiller (.xls, .xlsx, .ods) |
| `import-snb <file>…` | importare i cubi BNS (devkum, plkopr, zimoma, rendoblim) |
| `import-legacy <config.json>` | riprendere la configurazione del programma precedente come punto di partenza |
| `ui-dev` | interfaccia nel browser, solo in locale (sviluppo) |

La riga di comando emette i suoi testi, messaggi ed eventi in inglese, indipendentemente dalla lingua dell’interfaccia; numeri e date sono nel formato svizzero.

**Programma e cartella dei dati.** Il programma `vivaplan` fa parte del pacchetto d’installazione e si trova accanto all’app. Non viene aggiunto al percorso di ricerca; chiamatelo con il percorso completo:

| Sistema | Percorso del programma |
|---|---|
| macOS | `/Applications/VivaPlan.app/Contents/MacOS/vivaplan` |
| Windows (programma d’installazione `.exe`) | `%LOCALAPPDATA%\VivaPlan\vivaplan.exe` |
| Windows (`.msi`) | `C:\Program Files\VivaPlan\vivaplan.exe` |
| Linux (`.deb`, `.rpm`) | `/usr/bin/vivaplan` |

Con l’AppImage il programma si trova nell’immagine e non si può chiamare direttamente. `vivaplan --help` mostra tutti i comandi. È allegata una [guida](#ref-skill) per gli assistenti IA.

Come l’app, la riga di comando calcola con la cartella dei dati dell’app: con le serie temporali nella sottocartella `Zeitreihen`, anche con le serie importate nell’app (`library:<file>`), e con i limiti e la sostenibilità delle [impostazioni](#f-settings) (`einstellungen.json`). La cartella dei dati è `~/Library/Application Support/ch.vivaplan-app.desktop` con macOS, `%APPDATA%\ch.vivaplan-app.desktop` con Windows e `~/.local/share/ch.vivaplan-app.desktop` con Linux. `--data-dir <cartella>` usa un’altra cartella dei dati, `--library <cartella>` un’altra biblioteca di dati. Se manca la cartella dei dati, la riga di comando calcola con le serie integrate e i limiti standard e lo segnala una volta.

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

**`batch`** (più percorsi): `runs`, `successRate`, `earliestDepletion`, `percentiles` (sostanza finale `endTotal` e `endTotalReal` a 0 = simulazione peggiore, 1, 5, 10, 25, 50, 75, 90, 95 e 99 %, simulazioni ordinate secondo la sostanza finale nominale), `percentilesReal` (lo stesso con le simulazioni ordinate secondo la sostanza finale al potere d’acquisto attuale; la simulazione di un percentile può differire) e `years` con `p5` fino a `p95` della sostanza totale, la mediana reale `p50Real` e la quota delle simulazioni con sostanza residua, `survival`.

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

## @ref-describe Catalogo per le IA (describe)
Parole chiave: describe, catalogo, IA, campi, unità, intervallo di valori, grandezza di stato, indicatore, tempo di calcolo

`vivaplan describe [<sezione>] [--plan <pianificazione>] [--json]` dice a un programma o a un’IA senza codice sorgente cosa può modificare, variare e misurare in una pianificazione. Il catalogo è generato dallo schema e dal codice ed è in inglese come il resto della riga di comando. Resta sotto i 20'000 caratteri; le singole sezioni con `describe <sezione>`:

| Sezione | Contenuto |
|---|---|
| `fields` | Campi della pianificazione raggruppati per oggetto, ciascuno con tipo, unità, intervallo di valori e (solo in questa sezione) significato; `*` sta per il numero in un elenco |
| `series` | Forme di un valore nel tempo (`const`, `timed`, `linear`, `table`, `factor`, `curve`) e i momenti (tappa, `AAAA-MM`, `age:<anni>`, `campo@<momento>`) |
| `state` | Grandezze di stato di una [curva caratteristica](#f-curve) |
| `metrics` | Elementi di un [indicatore](#ref-metric): grandezze, finestra temporale, riduzione per percorso, aggregazione sui percorsi, confronti |
| `experiment` | Forme dell’[esperimento](#ref-experiment): variazioni, vincolo, obiettivo, metodi di ricerca, valori predefiniti |
| `examples` | tre esperimenti di esempio (spese con al massimo il 5 % di rovina, rendita o capitale, quota azionaria e regola di prelievo) che girano sulla pianificazione di esempio |
| `performance` | Microsecondi per calcolo, misurati alla chiamata (circa mezzo secondo) sulla pianificazione di `--plan`, altrimenti sulla pianificazione di esempio con rendimenti parametrici; permettono di stimare la durata di un esperimento |
| `limits` | Limiti del modello: anni indipendenti nel modello parametrico, errore di selezione con molte varianti, incertezza, nominale e reale, tempo di calcolo |
| `choices` | solo richiesta da sola: ogni valore di ogni campo di scelta con la sua denominazione nell’app (nella lingua dell’interfaccia) e il suo significato, p. es. `bootstrap` = «Monte Carlo · storico» |

Con `--json` il catalogo arriva nel formato fisso `vivaplan-describe/1`. Le descrizioni complete e la struttura esatta sono mostrate da `vivaplan schema`.

## @ref-skill Guida per assistenti IA (agent skill)
Parole chiave: skill, agent skill, SKILL.md, Claude, Claude Code, IA, assistente, configurare

Il pacchetto d’installazione contiene una guida per gli assistenti IA in grado di eseguire comandi: l’agent skill `vivaplan-skill` nel formato aperto (cartella con il file `SKILL.md`). Spiega all’IA la [riga di comando](#ref-cli), i campi importanti con le loro unità (quote invece di percentuali), nominale e reale, i limiti dei risultati e l’esclusione di responsabilità. La guida è in inglese; l’IA risponde comunque nella sua lingua.

| Sistema | Cartella dello skill |
|---|---|
| macOS | `/Applications/VivaPlan.app/Contents/Resources/vivaplan-skill` |
| Windows (programma d’installazione `.exe`) | `%LOCALAPPDATA%\VivaPlan\vivaplan-skill` |
| Windows (`.msi`) | `C:\Program Files\VivaPlan\vivaplan-skill` |
| Linux (`.deb`, `.rpm`) | `/usr/lib/VivaPlan/vivaplan-skill` |

**Configurare in Claude Code e in Claude Desktop (sezione *Code*):** entrambi leggono gli skill personali dalla cartella `.claude/skills` nella sua cartella utente. Copi lì la cartella `vivaplan-skill`, su macOS per esempio nel Terminale:

```
mkdir -p ~/.claude/skills && cp -R /Applications/VivaPlan.app/Contents/Resources/vivaplan-skill ~/.claude/skills/
```

Su Windows copi la cartella in `%USERPROFILE%\.claude\skills`. Avvii poi una nuova sessione. Chieda per esempio: *La mia sostanza basta fino a 95 anni? La mia pianificazione è ~/Documenti/pianificazione.vplan*. Claude chiama per questo `vivaplan` e chiede prima se può eseguire comandi. Dopo una nuova versione di VivaPlan copi di nuovo la cartella.

**Protezione dei dati:** ciò che l’IA legge dalla sua pianificazione o dai risultati (date di nascita, redditi, sostanza, rendite) va al fornitore dell’IA. Lo skill indica all’IA di avvisarla prima della prima lettura. VivaPlan stesso non invia nulla.

I risultati sono calcoli di modello senza garanzia, non una consulenza fiscale o d’investimento; le imposte sono stimate.

## @ref-mcp Collegare programmi di IA (server MCP)
Parole chiave: MCP, Model Context Protocol, server, vivaplan mcp, LM Studio, Claude Desktop, IA, strumento, protezione dei dati, salvare, annullare, plan_set, plan_save, cartella delle pianificazioni, sola lettura, avviso, allow-dir, nuova pianificazione, proprie cifre, incognito, memoria

`vivaplan mcp` è un server MCP: i programmi di IA che supportano il Model Context Protocol (per esempio Claude Desktop, Claude Code, ChatGPT Desktop o LM Studio) lo avviano da soli e comunicano con lui tramite l’input e l’output standard. Non apre alcuna connessione di rete. Il server modifica una copia di lavoro in memoria; solo `plan_save` scrive sul disco, secondo regole fisse (vedi sotto).

| Strumento | Scopo |
|---|---|
| `plan_open` | Aprire una pianificazione (`.vplan`, `.rplan`), una pianificazione vuota (`new`) o la pianificazione di esempio; panoramica senza valori e numero dei messaggi di verifica |
| `plan_get` | Leggere una sezione o un singolo campo (percorso di campo) |
| `plan_schema` | Consultare la struttura del file di progetto |
| `plan_validate` | Verificare la pianificazione come l’app |
| `describe` | [Catalogo per le IA](#ref-describe) |
| `result_list`, `result_get` | Risultati della sessione (sintesi, valori annuali, eventi, fascia, variazioni) |
| `library_list` | Serie temporali della [biblioteca di dati](#ref-library) |
| `simulation_run` | Simulare la pianificazione come l’app (con le sue variazioni); indicatori e risultato a parole |
| `evaluate` | [Esperimento](#ref-experiment) sulla pianificazione: modifiche, variazioni, indicatori, condizioni, obiettivo; l’ottimizzazione e il confronto passano da qui |
| `plan_set` | Modificare valori della copia di lavoro tramite percorso del campo (come [`vivaplan set`](#ref-cli)); tutte le modifiche di una chiamata sono un passo, con valore vecchio e nuovo e i messaggi di verifica che ne risultano |
| `plan_changes` | Tutte le differenze tra la copia di lavoro e il file |
| `plan_undo` | Annullare l’ultimo passo, con `all` tutti |
| `plan_apply` | Riprendere nella pianificazione una variante di un risultato di `evaluate` (secondo il rango), in un passo |
| `plan_save` | Salvare la copia di lavoro come `.vplan`; unico strumento che scrive sul disco |

Prima di ogni calcolo il server ne stima la durata. Da 60 secondi calcola solo se l’IA lo conferma espressamente (`"confirm": true`); deve prima chiederlo a voi. I calcoli lunghi segnalano l’avanzamento e si possono interrompere nel programma di IA. Viene eseguito un solo calcolo alla volta.

**Iniziare con le proprie cifre:** Se chiede all’IA di calcolare con le Sue cifre e non ha ancora un file di pianificazione, l’IA apre una pianificazione vuota con `plan_open` e `new`, la stessa di *File → Nuova* nell’app. La risposta indica i dati ancora mancanti (nome o designazione della persona, reddito, spese, sostanza, cassa pensioni, AVS) e i valori presunti (data di nascita, età di pensionamento, tariffe fiscali). L’IA deve chiederglieli e non inventare valori; aggiunge partner, pilastro 3a, immobili e spese uniche solo se ne ha. La nuova pianificazione viene salvata con `plan_save` come nuovo file nella (prima) cartella delle pianificazioni; basta un nome di file come `La mia pianificazione.vplan`. La pianificazione di esempio serve solo per provare e mostrare.

**Salvare:** `plan_save` rispetta da sé queste regole, qualunque cosa faccia l’IA: (1) Senza percorso nasce un **nuovo file** accanto a quello aperto, `<nome> (AI).vplan` (se necessario con un numero); un file esistente resta intatto. (2) Il server sostituisce un file esistente solo con `overwrite` e solo dopo che il programma di IA ve lo ha chiesto mostrando le modifiche; se non può chiedere, il server rifiuta. (3) Prima viene creata una copia di sicurezza `<file>.bak`; la scrittura passa da un file temporaneo. (4) Se il file è stato modificato dopo l’apertura, non viene sostituito. (5) I vecchi file `.rplan` non vengono mai sostituiti; si salva come `.vplan`. (6) Il server scrive solo nella cartella della pianificazione aperta o in una cartella autorizzata con `--allow-dir <cartella>` (sottocartelle comprese). Un percorso può iniziare con `~/` per la Sua cartella utente, come con `--allow-dir`. Se l’IA indica una cartella che non esiste o non è autorizzata, il server risponde che un nome di file da solo salva nella cartella delle pianificazioni; non nomina alcuna cartella. Con `--read-only` il server non offre affatto `plan_save`. Non modificate una pianificazione contemporaneamente nell’app e nell’IA: l’app non si accorge delle modifiche al file.

**Cartella delle pianificazioni e «Sola lettura»:** durante la configurazione sceglie una cartella delle pianificazioni; arriva al server come `--allow-dir <cartella>` (più cartelle: `--allow-dir` più volte). Il server apre e salva allora pianificazioni solo in questa cartella e nelle sue sottocartelle. Rifiuta i percorsi con `..` e i collegamenti simbolici che escono dalla cartella; per una pianificazione esterna segnala che si trova fuori dalla cartella delle pianificazioni. La cartella dei dati dell’app (serie temporali, impostazioni) resta leggibile, affinché il calcolo sia lo stesso dell’app. Aprire, calcolare, consultare i risultati, modificare e salvare nella cartella delle pianificazioni funzionano come senza cartella. Con l’opzione «Sola lettura» (`--read-only`, disattivata di default) l’IA può aprire, modificare e calcolare pianificazioni, ma non salvarle. Senza `--allow-dir` il server legge ogni pianificazione indicata e scrive solo nella sua cartella.

**L’avviso di Claude Desktop:** durante l’installazione del pacchetto Claude Desktop avvisa che l’estensione ottiene accesso a tutto il computer. Claude Desktop lo mostra per ogni estensione, perché il formato del pacchetto non conosce diritti: il server gira con i Suoi diritti di utente, negli altri programmi di IA allo stesso modo, solo senza avviso. È VivaPlan stesso a imporre il limite alla cartella delle pianificazioni, non il sistema operativo. Il server legge pianificazioni solo nella cartella delle pianificazioni, oltre alla cartella dei dati dell’app e ai dati forniti; scrive solo con `plan_save` nella cartella delle pianificazioni e non apre alcuna connessione di rete.

**Configurare con un clic:** il menu *Aiuto → Collegare a un’IA …* mostra per Claude Desktop, Claude Code, l’app desktop di ChatGPT (tramite la configurazione di Codex) e LM Studio dove va la voce, e la riga corrispondente con il percorso del programma sul Suo computer da copiare. Ogni riga contiene la cartella delle pianificazioni: è proposta la cartella dell’ultima pianificazione aperta, altrimenti la cartella Documenti; *Scegliere la cartella …* la cambia. L’opzione «Sola lettura» aggiunge `--read-only`. VivaPlan non configura nulla da sé. Per Claude Desktop su macOS ogni versione comprende il pacchetto `VivaPlan_<versione>_aarch64.mcpb`: un doppio clic installa VivaPlan come estensione in Claude Desktop; il pacchetto contiene esso stesso il programma `vivaplan`. Durante l’installazione Claude Desktop chiede la cartella delle pianificazioni (Plan folder, obbligatoria) e l’opzione «Sola lettura» (Read only). Durante la beta chiede inoltre il campo «Beta token»: se l’app su questo computer è già attivata, lo lasci vuoto; altrimenti il token attiva il server e l’app ([Token beta](#beta)). Senza attivazione il server risponde a ogni chiamata che VivaPlan non è ancora attivato.

**Configurare:** nel programma di IA inserite il programma `vivaplan` con il percorso completo (vedi [riga di comando](#ref-cli)) con gli argomenti `mcp --allow-dir <cartella>` (`~` sta per la Sua cartella utente). In Claude Desktop e LM Studio la voce si presenta così su macOS:

```
{
  "mcpServers": {
    "vivaplan": {
      "command": "/Applications/VivaPlan.app/Contents/MacOS/vivaplan",
      "args": ["mcp", "--allow-dir", "~/Documents"]
    }
  }
}
```

In Claude Code basta `claude mcp add --scope user vivaplan -- /Applications/VivaPlan.app/Contents/MacOS/vivaplan mcp --allow-dir ~/Documents`. Come la riga di comando, il server calcola con la cartella dei dati dell’app (limiti, serie temporali); `--data-dir` e `--library` la sostituiscono.

**Protezione dei dati:** tutto ciò che uno strumento restituisce diventa parte della conversazione con la vostra IA e va al suo fornitore; con un modello locale (per esempio in LM Studio) tutto resta sul computer. Il server legge solo le pianificazioni il cui percorso è indicato esplicitamente, con una cartella delle pianificazioni solo quelle al suo interno, e non esplora cartelle. Sostituisce i nomi delle persone e le denominazioni inserite liberamente con nomi neutri (Person 1, Property 1), salvo con `--names`; le risposte indicano solo il nome del file, non la cartella. Nessuna risposta supera 20'000 caratteri. I testi del server sono in inglese; l’IA risponde comunque nella vostra lingua.

**Chat in incognito:** in una chat in incognito di Claude VivaPlan non è disponibile: l’IA non vi vede alcuno strumento di VivaPlan. Lo decide il programma di IA; VivaPlan non può farci nulla. Se vuole comunque condividere i dati della Sua pianificazione con riserbo: (1) usi una chat normale e la cancelli dopo; ciò che il fornitore conserva in seguito è regolato dalle sue norme sulla protezione dei dati. (2) Disattivi la memoria per queste conversazioni nella parte protezione dei dati delle impostazioni del Suo programma di IA, affinché non ne ricordi nulla. (3) Usi un modello locale in LM Studio; così i dati non lasciano il computer.

I risultati sono calcoli di modello senza garanzia, non una consulenza fiscale o d’investimento; le imposte sono stimate.

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
