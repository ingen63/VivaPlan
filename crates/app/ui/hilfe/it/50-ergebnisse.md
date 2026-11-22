# Leggere i risultati

## @r-overview Risultati: panoramica
Parole chiave: tasso di successo, indicatori, percentili, sostanza finale, esaurimento, vista dei risultati

Dopo «Avviare la simulazione» l’app mostra una **barra di avanzamento** con il tempo trascorso e il tempo residuo stimato e un pulsante **Interrompere**. Poi si apre la vista dei risultati. La sua intestazione indica prima il **nome del risultato** come è archiviato nell’[elenco](#r-list) (ad esempio ‹Beispiel · MC hist. · Standard · 16.11.26, 14:05›), poi il tipo di simulazione (‹50'000 simulazioni› o ‹Simulazione singola · …›). Un nome lungo viene accorciato, il nome completo appare passandoci sopra con il mouse; dopo averlo rinominato nell’elenco vale il nuovo nome. La vista ha più schede: **Panoramica**, **Grafici**, **Simulazioni singole** (solo con più simulazioni), **Scenari** (solo con variazioni) e **Valori annuali**. Tutti i grafici con asse temporale si possono ingrandire: **tenendo premuto il tasto del mouse selezionare un intervallo di tempo** (viene evidenziato; selezioni inferiori a un anno contano come clic e vengono ignorate), oppure con i pulsanti **Tutto, 10 a., 5 a. e 2 a.** I pulsanti impostano l’ampiezza dell’intervallo **centrata sul centro dell’intervallo attuale** (senza zoom: dell’intera durata). Se l’intervallo va oltre l’inizio o la fine, viene spostato, l’ampiezza resta; se la durata è più breve, vede tutto. Dopo una selezione con il mouse nessun pulsante è attivo, i grafici mostrano «Personalizzato» con l’intervallo. Lo zoom vale **per tutti i grafici dei risultati insieme**, in tutte le schede e nel confronto, e si mantiene cambiando scheda; i minigrafici nell’editor hanno il loro zoom (dall’inizio). I grafici degli scenari non hanno un asse temporale e quindi nessuno zoom. In alto a destra passa tra **Nominale** e **Potere d’acquisto attuale**, vedi [Rendimento, inflazione e valori reali](#ret-concepts).

**Il risultato in parole:** in cima la panoramica riassume il risultato in due o tre frasi. Con più simulazioni: ‹Nel 87 % (medio) delle simulazioni la sostanza basta fino a 12.2055 (età 95.0).› La classificazione tra parentesi segue i colori del tasso di successo: **alto** dal 95 %, **medio** dall’80 % a meno del 95 %, **basso** sotto l’80 %; determinante è il valore mostrato, arrotondato allo 0.1 %. Se in una simulazione il denaro finisce, una seconda frase indica l’età nella simulazione più sfavorevole (come «Esaurimento più precoce»). Con una sola simulazione figura se la sostanza basta fino alla fine e quanto è grande allora (nominale o al potere d’acquisto attuale, a seconda del commutatore), oppure quando è esaurita. Sotto segue una proposta per il passo successivo: sotto il 95 % o con sostanza esaurita l’[ottimizzazione](#s-optimize), altrimenti gli [scenari](#s-scenarios). Il testo è un aiuto alla lettura, non una valutazione della Sua situazione.

**Con una sola simulazione** (rendimenti fissi, andamento storico) la panoramica mostra cinque indicatori: sostanza al pensionamento, sostanza alla fine, fino a quando basta il denaro, la rendita CP e le imposte stimate totali ([Quanto è precisa l’imposta?](#tax-accuracy)). Sotto seguono l’andamento della sostanza e l’elenco degli eventi.

**Con più simulazioni** (Monte Carlo, tutti i momenti d’inizio) la panoramica mostra:

- **Tasso di successo:** quota delle simulazioni in cui la sostanza basta fino alla fine. Verde dal 95 %, rosso sotto l’80 %.
- **Sostanza finale** alla mediana, al 5°, 1°, 95° e 99° percentile e nella **simulazione peggiore**.
- **Esaurimento più precoce:** l’età più precoce in cui in una qualsiasi simulazione il denaro era esaurito.
- la [fascia della sostanza totale](#r-fan) e la [tabella dei percentili](#r-percentiles).

**Quale tasso di successo è sufficiente?** Non è una questione di calcolo, ma della Sua propensione al rischio. Dal 90 % al 95 % è considerato adeguato in una pianificazione prudente. Tenga presente che un tasso di successo del 95 % significa che in 1 futuro simulato su 20 il denaro finisce. È solo una stima, vedi [Insidie](#ret-pitfalls).

**‹Successo› significa:** la sostanza (liquida e immobili) non si esaurisce fino alla fine della simulazione. **Non** significa che la vita sia stata piacevole: una simulazione può ‹avere successo› e terminare comunque molto di misura.

## @r-percentiles Sostanza finale per percentile
Parole chiave: percentile, mediana, 1° percentile, 99° percentile, simulazione peggiore, quantile

Le simulazioni vengono ordinate secondo la loro **sostanza finale**. Il **percentile n** è la sostanza finale al di sotto della quale si trova l’n % delle simulazioni.

| Riga | Significato |
|---|---|
| Simulazione peggiore | la simulazione con la sostanza finale più bassa |
| 1° percentile | solo l’1 % delle simulazioni termina peggio, ‹un caso molto sfavorevole› |
| 5° percentile | solo il 5 % termina peggio, ‹un caso sfavorevole› |
| 50° (mediana) | metà termina meglio, metà peggio |
| 95° e 99° percentile | casi molto buoni |

La tabella indica per ogni riga la **simulazione**, ad esempio il mese d’inizio storico o il numero del percorso. Una sostanza finale **negativa** significa che il denaro **è finito prima della fine** e che in seguito si sono accumulati debiti (spese scoperte).

La tabella mostra la sostanza finale nominale. Nel [confronto](#r-compare) si può mostrare anche al potere d’acquisto attuale.

> La **mediana** **non** è un caso normale realistico nel senso di una previsione. Dice solo che metà dei futuri simulati va meglio. Per la pianificazione contano i percentili **inferiori**, perché lì sta il rischio.

## @r-fan Fascia della sostanza totale
Parole chiave: ventaglio, banda, percentili, fascia, valore peggiore, cono, mediana

Il grafico mostra come la **sostanza totale** si **distribuisce** nel tempo. In ogni mese sono calcolati su tutte le simulazioni:

- la **superficie scura**: dal 25° al 75° percentile (la metà centrale delle simulazioni)
- la **superficie chiara**: dal 5° al 95° percentile (nove simulazioni su dieci)
- la **linea continua**: mediana
- le **linee tratteggiate**: 1° e 99° percentile (nella legenda con il simbolo di linea tratteggiata)
- la **linea rossa**: il **valore peggiore** del mese

La linea rossa è per ogni mese il valore più basso di tutte le simulazioni. Può essere ogni mese una simulazione diversa, quindi non è un singolo percorso, ma il limite inferiore di tutti i percorsi. Con il mouse vede i valori nel tooltip.

**Lettura:** la fascia si apre nel tempo come un imbuto. Più in basso si trova la banda inferiore, maggiore è il pericolo. Se il 5° percentile tocca la linea dello zero, in più del 5 % delle simulazioni il denaro finisce.

## @r-survival Probabilità che il denaro basti
Parole chiave: sopravvivenza, probabilità di successo, esaurimento, andamento

La quota delle simulazioni in cui la sostanza **fino a quel momento** non è ancora esaurita. La curva scende soltanto, non sale mai. Al margine destro figura il tasso di successo.

Una curva che scende **presto** indica un rischio già nei primi anni del pensionamento (rischio di sequenza). Una curva che scende solo tardi è un rischio di longevità. L’asse mostra per impostazione standard **80–100 %**, dove si trova la maggior parte delle curve e le differenze diventano visibili; tenga conto della scala. Il commutatore sopra il grafico passa a **0–100 %**. La scelta vale nel singolo risultato e nel [confronto](#r-compare), finché non la modifica. Se una curva scende sotto l’80 %, con 80–100 % è tagliata in basso, e un’avvertenza sotto il grafico indica il valore più basso. Il tooltip mostra sempre il valore vero.

## @r-hist Distribuzione della sostanza finale
Parole chiave: istogramma, distribuzione, frequenza, marcatori, pensionamento

Un **istogramma**: ogni colonna conta le simulazioni la cui sostanza finale cade in quella classe. I marcatori tratteggiati mostrano il 1°, 5°, 50°, 95° e 99° percentile (legenda sotto il diagramma). Il campo va dal 1° al 99° percentile, le simulazioni peggiori o migliori sono nelle classi marginali. La simulazione peggiore figura sotto nella nota.

Tipica è una distribuzione **asimmetrica** con una lunga coda a destra: poche simulazioni terminano molto bene, la maggior parte nel mezzo. L’estremità inferiore mostra quanto male può andare.

Il secondo grafico mostra la distribuzione della **sostanza al pensionamento**. È la situazione di partenza per la fase di prelievo: una sostanza bassa al pensionamento spesso determina già in larga misura il risultato.

## @r-depletion Età all’esaurimento
Parole chiave: esaurimento, esaurita, età, barre

Per ogni anno d’età il numero di simulazioni in cui la sostanza è esaurita **in quell’anno**. Il grafico appare solo se vi sono esaurimenti.

Utile per classificare il rischio in età avanzata: se il denaro finisce solo raramente e solo oltre i 90 anni, è un rischio diverso rispetto a quando succede a 75.

## @r-charts Grafici
Parole chiave: diagrammi, scheda, grafico

La scheda «Grafici» mostra i diagrammi a grandezza piena:

- Con **una simulazione:** [Andamento della sostanza](#r-wealth), [Sostanza liquida per investimento](#r-comp), [Entrate e spese](#r-flows), [Prelievo dalla sostanza](#r-draw), [Immobili](#r-prop), [Previdenza](#r-pk) e [Azioni e obbligazioni](#r-markets).
- Con **più simulazioni:** [Fascia](#r-fan), [Probabilità che il denaro basti](#r-survival), [Distribuzione della sostanza finale](#r-hist) ed [Età all’esaurimento](#r-depletion).

Ogni grafico ha una ⓘ sul titolo. Con **F1** sopra un grafico ottiene la spiegazione corrispondente.

**Linea spessa a 0:** in tutti i grafici con un asse dei valori che contiene 0, la linea a 0 è più spessa del resto della griglia, anche nel confronto e nel minigrafico nell’editor. Sotto questa linea il denaro è esaurito. Nella [distribuzione della sostanza finale](#r-hist) è verticale in corrispondenza di una sostanza finale di 0. Se 0 è al di fuori del campo mostrato, la linea manca.

## @r-runs Simulazioni singole
Parole chiave: simulazione, dettaglio, simulazione del percentile, simulazione peggiore, percorso, simulazione singola

Con più simulazioni può osservare in dettaglio **singole simulazioni**. Scelga in alto quale:

- **Simulazione peggiore**, **1°**, **5°**, **50°**, **95°** o **99° percentile**: la simulazione la cui **sostanza finale** corrisponde a questo percentile.

Sotto vede, per questa simulazione, gli stessi grafici di una simulazione singola e l’elenco degli eventi. Così riconosce **perché** una simulazione termina male: crollo di borsa all’inizio, inflazione alta, una vendita forzata dell’immobile.

Particolarmente istruttivo è il **confronto** tra il 5° e il 95° percentile: come si differenziano i rendimenti dei primi dieci anni di pensionamento? È il **rischio di sequenza** in cifre. Le simulazioni di dettaglio l’app le ricalcola esattamente.

## @r-wealth Andamento della sostanza
Parole chiave: sostanza totale, impilato, liquida, immobili, previdenza

La **sostanza totale** di una simulazione come superficie impilata composta da tre parti:

- **Sostanza liquida** (azioni, obbligazioni, cash)
- **Immobili (netto)**, cioè valore venale meno ipoteca
- **Previdenza** (cassa pensione, pilastro 3a)

Linee verticali contrassegnano gli eventi (pensionamento, inizio AVS, acquisto, vendita, esaurimento). Il puntatore del mouse mostra i valori del mese. I **salti** hanno di solito una ragione: il prelievo del capitale della cassa pensione sposta previdenza in sostanza liquida, una vendita della casa sposta immobili in sostanza liquida.

## @r-comp Sostanza liquida per investimento
Parole chiave: azioni, obbligazioni, cash, quote, composizione

Come la **sostanza liquida** si distribuisce su azioni, obbligazioni e cash. Con i pulsanti in alto mostra **Importi** o **Quote in %**.

La quota mostra se la strategia agisce come si aspetta. Con la [strategia a 3 comparti](#f-wd-three_buckets) vede i comparti, con «Standard» la quota azionaria fissa e la riserva.

## @r-flows Entrate e spese all’anno
Parole chiave: entrate, spese, imposte, spese d’alloggio, costo della vita, colonne

Per ogni **anno civile** due colonne:

- **a sinistra le entrate:** reddito da attività lucrativa, rendita CP, AVS
- **a destra le spese:** costo della vita, spese d’alloggio, imposte stimate

Dove la colonna destra è più alta di quella sinistra, si **attinge alla sostanza**. Il primo e l’ultimo anno sono incompleti, perché la simulazione inizia o termina a metà anno.

Il grafico mostra bene come le rendite dal pensionamento chiudono la lacuna e quanto è ancora grande.

## @r-draw Prelievo dalla sostanza
Parole chiave: tasso di prelievo, sostenibile, 4 percento, regola empirica, prelievo

All’anno: **spese incluse le imposte meno entrate**, in rapporto alla sostanza liquida. La linea mostra il **tasso di prelievo**.

Una regola empirica afferma che un tasso **sotto il 4 %** sia sostenibile. Vale solo approssimativamente e per le condizioni statunitensi su 30 anni. Se il tasso aumenta nettamente nel corso degli anni, perché la sostanza si riduce o le spese crescono, è un segnale d’allarme. Con le strategie che seguono la sostanza (tasso fisso, VPW, …) vede qui il loro effetto.

## @r-prop Immobili
Parole chiave: valore venale, ipoteca, fondi propri, casa

Gli immobili di una simulazione: **valore venale**, **ipoteca** e **fondi propri** (valore venale meno ipoteca). Il grafico appare solo se vi sono immobili. I salti mostrano acquisto, vendita o rimborso di un’ipoteca.

## @r-pk Previdenza
Parole chiave: cassa pensione, pilastro 3a, avere

Andamento di **cassa pensione** e **pilastro 3a** di una simulazione. La cassa pensione cresce con contributi e remunerazione fino alla prestazione e scende poi alla parte di rendita. Il 3a è visibile solo con ammortamento indiretto. Il grafico appare solo se vi sono averi di previdenza.

## @r-markets Azioni e obbligazioni
Parole chiave: indice, rendimento, andamento, andamento del mercato, indice dei prezzi, rincaro, inflazione, logaritmico, percentuale

Mostra l’**andamento dei rendimenti di questa simulazione** come indice, **inizio = 100**, con distribuzioni reinvestite. L’**inflazione** (indice dei prezzi) è mostrata come azioni e obbligazioni, tratteggiata. Nel tooltip figura inoltre la variazione degli ultimi dodici mesi, per l’inflazione tra parentesi.

Due commutatori cambiano la rappresentazione: **Andamento del valore (inizio = 100)** o **Variazione in %** (rispetto allo stesso mese dell’anno precedente). Per l’andamento del valore sceglie **Lineare** o **Logaritmica**; in scala logaritmica distanze uguali mostrano variazioni percentuali uguali. La vista in percentuale è sempre lineare. La linea spessa si trova per l’andamento del valore a **100** (stato all’inizio), per la variazione in % a **0 %**.

Così riconosce **che cosa è successo sul mercato in questa simulazione**: un crollo all’inizio, una lunga fase laterale o un’inflazione alta. Ciò spiega spesso perché una simulazione termina male o bene.

## @r-params Parametri
Parole chiave: parametri, riepilogo, dati, stato della simulazione, riprodurre

La scheda **«Parametri»** mostra **in sola lettura** con quali dati è stato calcolato il risultato: economia domestica (mese di nascita, nessun nome), tappe, previdenza, entrate e spese, sostanza e investimento, fonte di rendimenti e serie temporale (con valore iniziale casuale), strategia di prelievo con i parametri, immobili, ipotesi fiscali e variazioni. Vale lo **stato della simulazione**, non lo stato attuale nell’editor: se dopo il calcolo modifica un numero, la scheda del risultato calcolato resta invariata. Per i risultati salvati figura la pianificazione di allora; se manca, figura «Non disponibile».

## @r-events Eventi
Parole chiave: elenco, acquisto, vendita, vendita forzata, pensionamento, esaurimento

L’elenco di tutti gli **eventi** di una simulazione con data ed età: pensionamento, inizio AVS, prelievo del capitale con imposta sulle prestazioni in capitale, acquisti e vendite di immobili (anche **vendite forzate**), periodi ipotecari, il **passaggio dall’ammortamento indiretto a quello diretto** all’inizio della rendita AVS, il **rimborso unico** all’acquisto se l’ipoteca deve essere ridotta per la sostenibilità, riscossione del 3a, importi unici e l’**esaurimento** della sostanza. Le **registrazioni della strategia a 3 comparti** (rifornimenti, entrate, riservazioni di ricavi di vendita) non figurano in questo elenco, ma si possono aprire nei [valori annuali](#r-table); i messaggi della fase di deficit («tutti i comparti vuoti») restano qui.

Tutti gli importi figurano **con la valuta** (valuta del progetto).

L’**evento di prelievo** con strategie come Guardrails mostra le riduzioni e gli aumenti. Vendite forzate ed esaurimenti sono segnali che la pianificazione in questa simulazione non regge.

## @r-property Immobili (eventi)
Parole chiave: acquisto, vendita, vendita forzata, rinnovo dell’ipoteca, pagamento unico, sostenibilità, tasso d’interesse, fondi propri

Nella panoramica di una simulazione singola (e in «Simulazioni singole») la sezione **«Immobili»** elenca tutti gli eventi immobiliari in ordine cronologico, con momento (anno ed età), evento, immobile e importi:

- **Acquisto:** prezzo d’acquisto, fondi propri (prezzo − ipoteca) e ipoteca.
- **Vendita / vendita forzata:** ricavo della vendita, ipoteca estinta, imposta sugli utili immobiliari e **ricavo netto**; per la vendita forzata il motivo («sostanza < 0» o «criterio di vendita forzata»), più un’avvertenza se il ricavo è stato riservato per un acquisto nel mese successivo o se la [strategia di prelievo](#s-withdrawal) è stata applicata subito.
- **Rinnovo dell’ipoteca:** sostenibilità, ipoteca prima e dopo, pagamento unico per ristabilire la sostenibilità e tasso d’interesse del nuovo periodo; se l’ipoteca non viene rinnovata, figura la vendita forzata con il motivo.

Senza eventi vi figura «Nessun evento immobiliare». Con più simulazioni l’indicatore **«Vendite forzate»** della panoramica mostra in quante simulazioni (numero e quota, di cui per mancanza di sostenibilità) degli immobili sono stati venduti in modo non pianificato. Se [Non vendere mai gli immobili](#f-re-never-sell) è attivato, accanto figura l’indicatore **«Rinnovi non sostenibili»**: in quante simulazioni (numero e quota) un’ipoteca non era sostenibile al rinnovo ed è stata rinnovata per intero solo grazie al commutatore. Un pagamento unico effettuato non vi rientra. La colonna **«Vendite di immobili»** della tabella dei percentili indica per la simulazione della rispettiva riga **ogni vendita di un immobile** con il mese (‹Haus: 05.2034›, sotto ‹Wohnung: 03.2048›); le vendite forzate recano il simbolo di avvertimento (casa con segnale di avvertimento), le vendite pianificate no, senza vendita figura ‹–›.

## @r-scenarios Scenari
Parole chiave: variazione, heatmap, tabella, parametro, tasso di successo per variante

La scheda degli scenari mostra l’effetto delle [variazioni](#f-sweep): ogni riga della tabella è una **combinazione** di valori dei parametri, con cui sono stati calcolati **tutti i percorsi dei rendimenti**. Le colonne mostrano tasso di successo, sostanza al pensionamento (mediana), sostanza finale (5°, 50°, 95° percentile) e l’esaurimento più precoce.

- Con **un** parametro due grafici mostrano tasso di successo e sostanza finale per valore.
- Con **due** parametri nasce una **heatmap**: il colore mostra il tasso di successo, così trova il campo sicuro a colpo d’occhio.

Sopra i grafici e in ogni titolo di grafico figura **quale variazione** è rappresentata.

**Simulazione singola con variazioni:** se calcola con una fonte senza caso (ad esempio rendimenti fissi) e variazioni, sopra le schede appare **«Scegliere la variazione»**. Lì sceglie per ogni variazione un livello (o il valore della pianificazione); grafici, indicatori e valori annuali mostrano allora la simulazione di quel livello. «Ottimo» indica il miglior valore secondo gli stessi criteri usati con più simulazioni (sotto), **«Mostrare l’ottimo»** sceglie il passo corrispondente. Nella simulazione singola non esistono percentili, l’ottimo vale per l’unica simulazione.

**Livello con Monte Carlo e simulazioni storiche:** con più simulazioni e variazioni appare in **«Simulazioni singole»** e **«Valori annuali»** in alto un campo di selezione per variazione con i livelli calcolati (ad esempio ‹Quota azionaria 30 %›, ‹Prelievo del capitale 60 %›) e «Valore della pianificazione». Le due schede mostrano allora le simulazioni di quel livello, cioè le stesse simulazioni (peggiore, 1°, 5°, 50°, 95°, 99° percentile) su cui si basa la tabella sotto «Scenari». L’app ricalcola a tale scopo solo queste sei simulazioni, non tutti i percorsi; il cambio dura pochi secondi. La selezione vale per entrambe le schede insieme. Le altre schede continuano a mostrare i valori della pianificazione e recano la nota «Valori della pianificazione, non la variazione scelta». Con i risultati salvati la selezione funziona allo stesso modo. Se una sostanza finale ricalcolata differisce dalla tabella (ad esempio perché una serie temporale è cambiata), appare un’avvertenza. Un livello che non è stato calcolato (valore della pianificazione non nella griglia) non si può scegliere.

**Valore ottimale per percentile:** nella panoramica (sostanza finale per percentile) e in «Simulazioni singole» figura per ogni percentile il valore della variazione ottimale per quel percentile. A tale scopo vengono usati solo i passi di variazione già calcolati.

- **Fattore spese:** il fattore **più alto** con cui la sostanza basta fino alla fine. Se il passaggio cade tra due passi, si interpola tra essi finché la sostanza finale è esattamente 0 (visualizzazione ‹≈ 73 %›).
- **Quota azionaria, prelievo del capitale cassa pensione:** hanno la precedenza i valori con cui la sostanza basta; tra questi vale la **sostanza finale più alta**. Se non basta da nessuna parte, vale il valore con l’età di esaurimento più tardiva («(non basta in nessun caso)»). Qui non si interpola.
- **Due variazioni:** se c’è il fattore spese, per ogni valore dell’altro parametro viene determinato il fattore sostenibile più alto e viene mostrata la combinazione con il fattore più alto. Altrimenti vale la combinazione migliore secondo i criteri sopra.
- **Bordo del campo:** ‹< 50 %› significa che già il valore più basso esaminato non basta; ‹≥ 110 %› (o ‹≥› / ‹≤› per il valore migliore) che il campo non basta. Allarghi allora la variazione. Per una quota (quota azionaria, prelievo del capitale cassa pensione) figura 0 % o 100 % senza ‹≥› o ‹≤›, perché è il limite naturale, non un campo troppo stretto; se il valore migliore di una quota è a un altro bordo (ad esempio 50 % con un campo da 0 a 50 %), ‹≥› o ‹≤› continua a valere.
- **Mese d’inizio storico, data di pensionamento, tasso di prelievo:** qui non ha senso un ottimo (il mese d’inizio non è una decisione; una data di pensionamento più tardiva o un tasso più basso dà sempre la sostanza finale più alta). Viene mostrato ‹–›.

Tutte le varianti usano **gli stessi percorsi casuali**. Le differenze provengono quindi dal parametro, non dal caso.

## @r-table Valori annuali
Parole chiave: tabella, anno, esportazione, numeri, prelievo, scostamento dal piano, imposte, spese, affitto, ipoteca, percentile, prelievo del capitale, utile immobiliare

Una tabella con **una riga per anno** per la simulazione scelta. Importi nella valuta del progetto, nominali o al potere d’acquisto attuale secondo il commutatore. Con più simulazioni sceglie in alto la simulazione (simulazione peggiore, 1°, 5°, 50°, 95°, 99° percentile); lo standard è la mediana.

| Colonne | Significato |
|---|---|
| Liquida, immobili, previdenza, totale | **sostanza a fine anno**: mezzi liquidi, immobili (netto), previdenza (CP, 3a), somma |
| Spese totali | **media mensile dell’anno**: spese pianificate (costo della vita dalla Sua pianificazione) più **affitto o interesse ipotecario**, ammortamento e altre spese d’alloggio degli immobili più **versamento nel pilastro 3a** |
| Prelievo | media mensile dell’anno: ciò che viene **prelevato** dalla sostanza (spese effettive meno reddito, prima delle imposte) |
| Reddito totale | media mensile dell’anno: reddito (reddito da attività lucrativa e rendite) più prelievo |
| Tasso di prelievo | **prelievo annuo / sostanza totale a inizio anno** in percentuale (colonna «Totale» dell’anno precedente) |
| Scostamento dal piano | media mensile: reddito totale meno spese totali. Con **0** tutto procede come pianificato. Con le strategie che legano le spese alla sostanza (Guardrails, tasso fisso, VPW, Endowment), un valore **negativo** è la riduzione rispetto al Suo piano, uno positivo l’aumento |
| Inflazione all’anno | variazione dell’indice dei prezzi nell’anno civile, nel primo anno dall’inizio della simulazione |
| Imposte pagate nell’anno | **somme annue delle imposte pagate** per tipo: imposta reddito, imposta sostanza, imposta prestazione in capitale, imposta utili immobiliari, più il totale. Le colonne sono **chiuse per impostazione standard**; il pulsante **«Mostrare le imposte»** sopra la tabella le mostra e le nasconde, la scelta resta memorizzata. Le intestazioni si chiamano **«Imposta stimata …»**, il loro tooltip indica l’arrotondamento a 100 franchi |

Le colonne non sono ordinate in gruppi con un titolo. Che la sostanza sia composta da **saldi a fine anno** e i flussi di denaro da **medie mensili** è indicato nell’avvertenza (tooltip) delle intestazioni delle colonne e nel testo sotto la tabella. Il reddito da solo e le spese pianificate senza spese d’alloggio la tabella non li mostra; sono contenuti in «Reddito totale» e «Spese totali».

- **Tasso di prelievo:** il prelievo annuo è la somma dei prelievi mensili dell’anno civile (per anni incompleti la media mensile per 12). La sostanza a inizio anno è la sostanza totale («Totale») dell’anno precedente, nel primo anno il saldo all’inizio della simulazione. Il calcolo usa sempre valori **nominali**, il commutatore nominale/potere d’acquisto attuale non cambia i tassi. Se il prelievo è nullo o negativo (reddito superiore alle spese), la cella resta vuota; se la sostanza è nulla o negativa, figura ‹–›.
- **Strategia a 3 comparti:** ogni anno con almeno un rifornimento ha davanti una freccia (▸). Apre la **derivazione delle dimensioni dei comparti e dei rifornimenti** di quell’anno (evento scatenante, base delle spese, entrate regolari, fabbisogno netto, obiettivi, criterio di rendimento, trasferimenti, saldi dei comparti prima e dopo). Con più simulazioni vale per la simulazione singola scelta.
- I flussi di denaro sono **medie mensili** dell’anno civile, perché i singoli mesi variano (gli interessi ipotecari vengono ad esempio pagati trimestralmente).
- L’imposta sul reddito e quella sulla sostanza vengono calcolate a dicembre e pagate a **maggio dell’anno successivo**. La tabella le mostra nella riga dell’**anno fiscale** a cui si riferiscono: l’imposta dell’anno fiscale 2028 figura nella riga 2028, anche se il denaro esce solo a maggio 2029. La sostanza e il saldo di cash diminuiscono perciò solo l’anno successivo. Se la pianificazione termina prima di dicembre, l’imposta dell’ultimo anno incompleto manca nelle colonne. L’imposta sulle prestazioni in capitale (cassa pensione, pilastro 3a) e l’imposta sugli utili immobiliari figurano nell’anno del prelievo o della vendita.
- Le imposte sono **stimate** e **arrotondate a 100 franchi** (ogni cella per sé, il totale può quindi differire di qualche centinaio di franchi dalla somma delle colonne arrotondate). Sotto la tabella figura «Imposta stimata con le Sue tabelle, arrotondata a 100 franchi. Nessuna consulenza fiscale.» Prima di decidere, verificare l’imposta sulle prestazioni in capitale e l’imposta sugli utili immobiliari con il calcolatore fiscale del Cantone o con uno specialista, vedi [Quanto è precisa l’imposta?](#tax-accuracy).
- Il primo e l’ultimo anno sono per lo più incompleti; recano un asterisco (*), il puntatore del mouse indica il numero di mesi.

## @r-list Elenco dei risultati
Parole chiave: cronologia, elenco, salvato, rinominare, nota, eliminare, risultati, simulazioni

Ogni simulazione viene **archiviata automaticamente** come risultato dopo l’avvio, con la **pianificazione di allora** e il **risultato completo**. Con il pulsante **«Risultati»** nell’intestazione apre l’elenco.

- **Risultato:** per le simulazioni Monte Carlo figura ‹**Tasso di successo · 5 %: importo**› (5° percentile della sostanza finale); la mediana appare passandoci sopra con il mouse. Le simulazioni singole mostrano la sostanza finale. Il **simbolo di avvertimento (casa con segnale di avvertimento)** appare se un immobile è stato venduto in modo non pianificato (vendita forzata, anche per mancanza di sostenibilità al rinnovo dell’ipoteca; le vendite pianificate non contano). Con più simulazioni vale dal **5 % delle simulazioni** con vendita forzata, il tooltip indica quota e numero di simulazioni; con una simulazione singola a ogni vendita forzata. Le voci di versioni precedenti non mostrano alcun simbolo.
- **Serie temporale:** nome della serie temporale con cui è stato calcolato il risultato (con rendimenti fissi «rendimenti fissi», con la fonte parametrica «parametrico»). Le voci di versioni precedenti mostrano ‹–›. L’indicazione del tipo (simulazione singola, scenari, numero di simulazioni) figura in piccolo sotto il risultato.
- **Strategia:** se passa con il mouse sul nome (o vi arriva con il tasto Tab), appaiono i **parametri della strategia** allo stato del risultato. Le voci meno recenti mostrano «Parametri non disponibili».
- **Nome e nota:** modificabili direttamente nella riga. Nome standard: progetto · fonte di rendimenti · strategia · momento, ad esempio ‹Beispiel · MC hist. · 3 comparti + Guard · 16.11.26, 14:05› (per gli scenari figura ‹Scenari› al posto della fonte). Abbreviazioni della fonte di rendimenti: **Fisso** (rendimenti fissi), **Stor.** (storico dal momento d’inizio), **Tutti gli inizi** (tutti i momenti d’inizio storici), **MC hist.** (Monte Carlo · storico), **MC param.** (Monte Carlo · parametrico). Abbreviazioni della strategia: **Standard**, **3 comparti**, **3 comparti + Guard** (3 comparti con Guardrails), **Guardrails**, **Tasso fisso** (tasso di prelievo fisso), **VPW**, **Endowment**. Il numero di simulazioni figura in piccolo sotto il risultato. I risultati di versioni precedenti mantengono il loro nome.
- **Aprire:** mostra il risultato salvato con tutte le schede. Vale la **pianificazione di allora**, un’avvertenza in alto indica la data. **«Adottare la pianificazione nell’editor»** ricarica questa pianificazione nell’editor.
- **Eliminare** rimuove un risultato dopo una domanda di conferma, **«Eliminare i contrassegnati»** più risultati ed **«Eliminare tutti»** tutti. La casella nell’intestazione contrassegna tutte le voci.
- **Le ottimizzazioni** e i confronti delle strategie compaiono anch’essi nell’elenco, vedi [Salvare e riaprire il risultato](#o-saved). «Confrontare» conta solo le simulazioni e tralascia le ottimizzazioni contrassegnate.
- **Confrontare:** contrassegni da due a dieci voci e clicchi su **«Confrontare»**, vedi [Confronto](#r-compare). Se ne sono contrassegnate più di dieci, «Confrontare» è bloccato e appare un’avvertenza; l’eliminazione resta possibile.

L’elenco si trova **in locale nella memoria dell’app**, non nel file di pianificazione. Cresce finché non elimina voci: una voce con molte simulazioni occupa alcuni megabyte. Se la memoria è piena, appare il messaggio «Risultato non salvato nell’elenco»; il risultato resta visualizzato, ma non viene archiviato. Elimini allora le voci meno recenti.

## @r-compare Confrontare i risultati
Parole chiave: confronto, affiancati, differenze, pianificazione, indicatori, valori annuali, potere d’acquisto

Il confronto mostra fino a dieci risultati salvati affiancati, ognuno con un colore proprio. Le tabelle larghe e i grafici a barre si possono scorrere orizzontalmente con molti risultati. Ha le stesse possibilità di un singolo risultato, distribuite su quattro schede:

- **Panoramica:** indicatori (serie temporale, strategia, tasso di successo, sostanza finale alla mediana, al 5° e 1° percentile e nella simulazione peggiore, esaurimento più precoce; un clic su un’intestazione di colonna ordina le righe in ordine crescente, un altro decrescente, la freccia ▲/▼ mostra la direzione; i valori mancanti figurano sempre alla fine, «nessuna» come esaurimento vale come valore più tardivo), la **fascia della sostanza totale** (mediana continua, 5° percentile tratteggiato, 95° percentile punteggiato), la **sostanza finale per percentile** come tabella, la **probabilità che il denaro basti** (asse 80–100 % o 0–100 %, vedi [lì](#r-survival)), e l’**età all’esaurimento** come quota delle simulazioni (così i risultati con un numero diverso di simulazioni restano confrontabili).
- **Andamenti:** i grafici del singolo risultato per una simulazione per risultato: con una simulazione singola quella simulazione, con più simulazioni la simulazione del percentile che imposta sotto **«Scegliere la simulazione»** (come in «Simulazioni singole»; standard: mediana). La scelta vale per tutti i risultati e anche per i valori annuali; se a un risultato manca il percentile, mostra la simulazione mediana con una nota. Sostanza totale, sostanza liquida (selezionabile: totale, azioni, obbligazioni, cash), entrate, spese o imposte all’anno, tasso di prelievo, immobili (fondi propri), previdenza nonché azioni, obbligazioni e inflazione (azioni continue, obbligazioni tratteggiate, inflazione punteggiata). Questo grafico ha gli stessi commutatori del singolo risultato: **Andamento del valore (inizio = 100)**, lineare o logaritmico, oppure **Variazione in %** rispetto allo stesso mese dell’anno precedente; la scelta vale in entrambe le viste.
- **Valori annuali:** per la simulazione scelta una riga per anno e una colonna per risultato per una grandezza selezionabile (sostanza totale, sostanza liquida, immobili, previdenza, reddito, spese, prelievo, tasso di prelievo, imposte). I valori sono gli stessi dei [valori annuali](#r-table) dei singoli risultati.
- **Pianificazione:** una tabella di tutti i dati che differiscono tra i risultati, con le denominazioni e le unità dell’editor (ad esempio ‹Quota azionaria: 30 %› invece di un nome di campo interno) e raggruppati secondo le sezioni dell’editor. Le voci negli elenchi recano il loro nome (ad esempio ‹Ferienwohnung: valore venale›). I valori dipendenti dal tempo appaiono come elenco (‹da 01.2030: …›). Così vede esattamente **che cosa** ha portato a un risultato diverso.

In alto a destra passa tra **Nominale** e **Potere d’acquisto attuale**; ogni risultato viene convertito con la propria inflazione. Al potere d’acquisto attuale vale per ogni percentile la sostanza finale **della simulazione che in termini nominali si trova su quel percentile**, convertita con il rincaro di quella simulazione. Se le simulazioni hanno un rincaro casuale, l’ordine può così differire da quello nominale: il 99° percentile può essere in termini reali inferiore al 95°. Lo zoom vale come nel singolo risultato per tutti i grafici con asse temporale.

I risultati vengono allineati secondo gli **anni dal loro inizio**. Se iniziano in anni civili diversi, i valori annuali mostrano ‹Anno 1›, ‹Anno 2› ecc., l’anno civile figura nel tooltip. Se a un risultato manca una grandezza, manca in quel grafico o in quella colonna, e una nota lo indica: una simulazione singola appare nella fascia come una linea e non ha percentili, probabilità né età all’esaurimento; un risultato senza immobile o senza averi di previdenza manca in questi grafici; un risultato con soli scenari appare solo come indicatore; i risultati meno recenti senza valori al potere d’acquisto attuale mancano nella fascia; i risultati della versione 1.0.0-beta.1 e precedenti hanno al potere d’acquisto attuale solo la sostanza finale della simulazione peggiore, del 1°, 5°, 50°, 95° e 99° percentile.

Confronti preferibilmente risultati che differiscono in **un solo** punto (ad esempio quota azionaria o strategia).
