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
- **Ottimizzazione:** obiettivo, strategie, percorsi e le **fasce dei parametri** per strategia si trovano sotto `optimization` (`ranges`: strategia, parametro, poi `fixed` o `lo`/`hi`, quote come numero decimale).
- **Imposta sugli utili immobiliari:** `taxes.propertyGain` con `brackets` (ampiezza, aliquota), `duration` (da anni interi, maggiorazione o riduzione), `exemptBelow` e `reduction`; la data d’acquisto di un immobile in proprietà si trova in `bought`.
- Il formato completo lo fornisce `vivaplan schema` nel terminale.

Il file **non contiene risultati**. L’app archivia i risultati nell’[elenco dei risultati](#r-list).

## @ref-cli Riga di comando
Parole chiave: terminale, vivaplan, comando, script, CLI

Il motore di calcolo è disponibile anche nel terminale (programma `vivaplan`):

| Comando | Scopo |
|---|---|
| `run <pianificazione>` | calcolare la pianificazione (`--csv`, `--json`, `--events`) |
| `validate <pianificazione>` | verificare la pianificazione |
| `optimize <pianificazione>` | ottimizzare la strategia, vedi [Riga di comando](#o-cli) |
| `bench <pianificazione>` | misurare il tempo di calcolo (`--paths`) |
| `schema` | emettere lo schema JSON del file di pianificazione |
| `library` | elencare le serie temporali della biblioteca di dati |
| `import-shiller <file>` | reimportare i dati Shiller (.xls, .xlsx, .ods) |
| `import-snb <file>…` | importare i cubi BNS (devkum, plkopr, zimoma, rendoblim) |
| `import-legacy <config.json>` | riprendere la configurazione del programma precedente come punto di partenza |
| `ui-dev` | interfaccia nel browser, solo in locale (sviluppo) |

La riga di comando emette i suoi testi, messaggi ed eventi in inglese, indipendentemente dalla lingua dell’interfaccia; numeri e date sono nel formato svizzero.

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
