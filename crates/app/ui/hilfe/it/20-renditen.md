# Simulazione e rendimenti

## @s-returns Simulazione: panoramica
Parole chiave: rendimento, inflazione, fonte, metodo, serie temporale, Monte Carlo, SARON, strategia d’investimento

La sezione «Simulazione» determina **come evolvono nel tempo i rendimenti di azioni e obbligazioni, l’inflazione e il SARON**: valori fissi, un andamento storico o Monte Carlo. Ha la maggiore influenza sull’aspetto dei risultati e su quanto può fidarsene.

**Quali indici** forniscono i rendimenti (ad esempio azioni mondiali e obbligazioni della Confederazione) lo stabilisce nella [strategia d’investimento](#f-ret-series) nella sezione «Sostanza e investimento». Una serie temporale non la sceglie più Lei; risulta dalla strategia.

Esistono cinque metodi. La vista mostra solo i dati che valgono per il metodo scelto.

| Fonte | Simulazioni | Per che cosa |
|---|---|---|
| [Rendimenti fissi](#f-ret-fixed) | 1 | stima rapida, anteprima, controllo di plausibilità |
| [Storico dal momento d’inizio](#f-ret-historical) | 1 | ‹Che cosa sarebbe successo se fossi andato in pensione nel 1973?› |
| [Tutti i momenti d’inizio storici](#f-ret-rolling) | circa da 400 a 900 | come avrebbe agito ogni inizio reale |
| [Monte Carlo · storico (bootstrap)](#f-ret-bootstrap) | libero, ad es. 50'000 | ampia dispersione da veri dati mensili |
| [Monte Carlo · parametrico](#f-ret-parametric) | libero, ad es. 50'000 | modello con valori proposti per indice o ipotesi proprie |

Se non sa da dove iniziare: legga [Quale fonte è adatta?](#ret-which).

Le basi valide per tutte le fonti si trovano sotto [Rendimento, inflazione e valori reali](#ret-concepts). Gli errori di ragionamento frequenti sono descritti sotto [Insidie](#ret-pitfalls).

## @ret-which Quale fonte è adatta?
Parole chiave: decisione, raccomandazione, scelta, scegliere la fonte

Una regola empirica:

1. **Per inserire e provare:** «Rendimenti fissi». L’anteprima a destra calcola sempre con valori fissi e reagisce subito.
2. **Per la vera risposta ‹Il denaro basta?›:** Monte Carlo, preferibilmente **«Monte Carlo · storico» (bootstrap)** con una strategia d’investimento che risale molto indietro (ad esempio «Mondo (JST)» o «S&P 500»). Conserva le vere caratteristiche dei mercati (crisi, riprese, nesso tra azioni, obbligazioni e inflazione) e ha comunque un’ampia dispersione.
3. **Come controprova:** «Tutti i momenti d’inizio storici». Mostra concretamente come il piano si sarebbe svolto in periodi reali, ad esempio con inizio nel 1929, 1966 o 2000.
4. **Per ipotesi proprie:** «Monte Carlo · parametrico», se vuole indicare Lei stesso il rendimento atteso e il rischio, ad esempio perché in futuro si aspetta rendimenti più bassi che in passato.

**Buona prassi:** calcoli le decisioni importanti con **almeno due fonti** (bootstrap e parametrica) e verifichi se l’affermazione resta la stessa. Un piano che sembra buono solo con un’unica fonte è fragile.

| Domanda | Fonte |
|---|---|
| Con quale probabilità basta il mio denaro? | bootstrap o parametrica |
| Quanto male può andare? | bootstrap, 1° percentile e simulazione peggiore |
| Il piano avrebbe funzionato nel 1973? | Storico dal momento d’inizio |
| Il piano funziona in ogni situazione storica? | Tutti i momenti d’inizio storici |
| E se in futuro la borsa rende meno? | parametrica con μ più basso |

## @ret-concepts Rendimento, inflazione e valori reali
Parole chiave: nominale, reale, rincaro, geometrico, rendimento annuo, potere d’acquisto

I **rendimenti annui** vengono applicati nell’app mensilmente. Un rendimento annuo del 5 % dà al mese (1.05)^(1/12) − 1, cioè circa lo 0.41 %. L’andamento annuo è così esatto.

L’**inflazione** agisce ogni mese. L’indice del rincaro cresce mensilmente, e tutte le spese, gli affitti e (se scelto) i valori degli immobili vengono adeguati con esso. Le Sue spese le registra ai **prezzi attuali**, l’app le proietta nel futuro con l’inflazione.

**Nominale e reale:** i risultati esistono in due viste.

- **Nominale:** importi come si presentano allora (in franchi futuri).
- **Potere d’acquisto attuale:** importi ai prezzi dell’inizio della simulazione, depurati dal rincaro. Questa vista è per le decisioni di solito la più onesta, perché 2 milioni tra 30 anni hanno meno potere d’acquisto di oggi.

**Aritmetico e geometrico:** un ‹rendimento medio del 7 %› è di solito una media aritmetica. La media **geometrica** (di quanto cresce davvero la Sua sostanza) è più bassa a causa delle variazioni. Con un’attesa del 7 % e una volatilità del 15 % è di circa il 5.9 %. L’informazione breve della strategia d’investimento mostra il rendimento annuo medio geometrico.

**Rendimento complessivo:** tutte le serie contengono dividendi e cedole (total return), nessun costo d’investimento, nessuna imposta sui redditi.

## @f-ret-source Fonte dei rendimenti
Parole chiave: scelta, fisso, storico, rolling, bootstrap, parametrico

Scelga la fonte con le schede in alto nella sezione. Ogni scheda ha una ⓘ con la spiegazione dettagliata:

- [Rendimenti fissi](#f-ret-fixed)
- [Storico dal momento d’inizio](#f-ret-historical)
- [Tutti i momenti d’inizio storici](#f-ret-rolling)
- [Monte Carlo · storico](#f-ret-bootstrap)
- [Monte Carlo · parametrico](#f-ret-parametric)

Se non è sicuro di quale sia adatta, legga [Quale fonte è adatta?](#ret-which).

## @f-ret-fixed Rendimenti fissi
Parole chiave: costante, azioni, obbligazioni, inflazione, SARON, simulazione singola

Un’unica simulazione con **valori annuali costanti** per azioni, obbligazioni, inflazione e SARON (per ipoteche SARON).

- Non vi è **alcuna dispersione**: il risultato è ‹il denaro basta› o ‹non basta›. Una probabilità di successo non esiste.
- Adatta per inserire dati, per confronti rapidi e per l’anteprima a destra. **Non adatta** per giudicare se il piano è sicuro: ignora completamente il rischio di sequenza.
- **Scelta dei valori:** calcoli con valori prudenti e **realistici**, non con gli anni migliori del passato. Un’ipotesi sobria sarebbe ad esempio dal 4 % al 5 % per le azioni, dall’1 % al 2 % per le obbligazioni, dall’1 % al 2 % di inflazione.
- Con le strategie «Tasso fisso», «Guardrails» e altre una simulazione con rendimenti fissi mostra solo il caso medio.

Con le altre fonti i valori fissi valgono comunque in parte: per il **SARON** al di fuori del periodo della serie e con il [modello parametrico](#f-ret-parametric), e dopo la fine della serie con l’[estrapolazione](#f-ret-extrapolation).

## @f-ret-historical Storico dal momento d’inizio
Parole chiave: retrospettiva, mese d’inizio, 1973, 1929, backtest, serie temporale

Un’unica simulazione con il **vero andamento dei rendimenti** della [strategia d’investimento](#f-ret-series), a partire dal **mese d’inizio storico**. La simulazione inizia allora come se fosse partito in quel mese.

- **Esempio:** il mese d’inizio 01.1973 verifica che cosa sarebbe successo se, con la Sua pianificazione, fosse andato in pensione all’inizio della crisi petrolifera (inflazione alta, borsa debole).
- La simulazione si svolge sul periodo della pianificazione, ad esempio 32 anni dal mese d’inizio. Se la serie termina prima, interviene l’[estrapolazione](#f-ret-extrapolation).
- Con le strategie annuali (tutto tranne S&P 500 con obbligazioni USA o con obbligazioni della Confederazione) conta solo l’anno.
- **Le singole simulazioni sono aneddoti.** Un piano che avrebbe funzionato nel 1973 può fallire in un’altra situazione. Per un’affermazione sulla sicurezza usi [tutti i momenti d’inizio](#f-ret-rolling) o [Monte Carlo](#f-ret-bootstrap).
- Con una [variazione del mese d’inizio](#f-sweep) si può calcolare tutta una serie di inizi storici. È simile a «Tutti i momenti d’inizio storici», ma con una griglia scelta da Lei.

## @f-ret-rolling Tutti i momenti d’inizio storici
Parole chiave: mobile, rolling, iterazione, sovrapposizione, momento d’inizio

L’app calcola **ogni mese d’inizio possibile** per il quale la serie copre l’intero periodo di pianificazione. Con una serie dal 1922 e una pianificazione su 32 anni sono alcune centinaia di simulazioni: inizio gennaio 1922, febbraio 1922 e così via fino all’ultimo mese che lascia ancora 32 anni fino alla fine della serie.

**Punti di forza:** ogni simulazione è un periodo **reale** con veri nessi tra azioni, obbligazioni e inflazione. Vede gli inizi storicamente peggiori (ad esempio 1929, 1966, 1973, 2000).

**Punti deboli che dovrebbe conoscere:**

- Le simulazioni **si sovrappongono molto**. Da 104 anni di dati risultano 800 mesi di punti d’inizio, ma solo poco più di **tre periodi di 32 anni davvero indipendenti**. La dispersione viene perciò **sottostimata**, e ‹il 98 % delle simulazioni ha successo› suona più sicuro di quanto sia.
- Esiste solo **l’unico** passato. Una sequenza peggiore che non si è mai verificata non compare.
- Con periodi di pianificazione lunghi e serie brevi vi sono pochissimi punti d’inizio. Con una serie annuale (ad esempio «Svizzera (JST)», 121 anni) e una pianificazione di 32 anni sono solo circa 90.

Il [bootstrap](#f-ret-bootstrap) elimina in parte questi punti deboli e mostra perciò fasce più ampie. Usi «Tutti i momenti d’inizio storici» come controprova, non come unica fonte.

Il **numero** di simulazioni è indicato nella sezione «Svolgimento».

## @f-ret-bootstrap Monte Carlo · storico (bootstrap)
Parole chiave: bootstrap, lunghezza dei blocchi, percorsi, valore iniziale casuale, seed, stazionario, Monte Carlo

Il bootstrap genera **migliaia di nuovi percorsi dei rendimenti** concatenando segmenti casuali della serie temporale della strategia d’investimento. Nascono così andamenti che non sono mai esistiti, ma che sono composti da **veri dati mensili**.

**Come funziona:**

1. Il percorso inizia in un mese casuale della serie.
2. In ogni mese il **blocco** attuale termina con la probabilità 1 / lunghezza dei blocchi. Poi inizia un nuovo blocco in un mese d’inizio casuale. Alla fine della serie si riprende dall’inizio.
3. Azioni, obbligazioni, inflazione e SARON provengono sempre **dallo stesso mese storico**. I nessi (ad esempio crisi di borsa con inflazione alta) si conservano.

**Dati:**

- **Numero di percorsi:** più percorsi danno numeri più regolari, ma un tempo di calcolo più lungo. 10'000 bastano per una prima risposta, 50'000 per i percentili al margine (1° percentile, simulazione peggiore). Il tasso di successo è un risultato casuale: con 10'000 percorsi è preciso a circa ±0.4 punti percentuali, con 50'000 a circa ±0.2 (circa due errori standard con un tasso di successo intorno al 95 %). **Limite superiore:** percorsi × mesi della pianificazione al massimo 50 mio, affinché la memoria basti (con 30 anni circa 138'000 percorsi, con 100 anni 41'666). Oltre, la verifica segnala un errore sul campo con il numero massimo possibile.
- **Lunghezza media dei blocchi (mesi):** quanto sono lunghi in media i segmenti consecutivi.
  - **Corta (da 1 a 6):** i mesi vengono mescolati in modo quasi indipendente. Ciò distrugge la caratteristica che ad anni negativi ne seguono spesso altri (**momentum e ripresa**) e rende i percorsi **troppo regolari**.
  - **Lunga (24 e più):** i percorsi restano vicini alla storia, la varietà diminuisce.
  - **12 mesi** (standard) è un compromesso collaudato. Con serie annuali la lunghezza vale in anni (lunghezza dei blocchi / 12).
- **Valore iniziale casuale (seed):** stesso valore, stessi percorsi. Vedi [Valore iniziale casuale](#ret-seed).

**Caratteristiche:**

- I percorsi hanno una dispersione **più ampia** che con «Tutti i momenti d’inizio storici», perché non sono legati all’unico ordine della storia. È l’incertezza più onesta.
- Il bootstrap può mostrare solo ciò che **è contenuto nei dati**. Rendimenti alti del passato portano a rendimenti alti nel futuro. Se vuole calcolare con più prudenza, usi la [fonte parametrica](#f-ret-parametric) con μ più basso o una serie più corta che non contenga solo anni di boom.
- Le viste di dettaglio (5°, 50°, 95° percentile e le simulazioni al margine) ricalcolano esattamente il rispettivo percorso.

## @f-ret-parametric Monte Carlo · parametrico
Parole chiave: Student-t, fat tails, volatilità, drift, salti di corso, mu, sigma, gradi di libertà, Poisson, crash, valori proposti, correlazione, obbligazioni, inflazione

Un **modello azionario con ipotesi proprie o proposte**. I corsi azionari seguono un movimento casuale in **passi giornalieri** (21 al mese), integrato da rari **salti di corso**. Anche **obbligazioni e inflazione** sono casuali e **correlate** con le azioni, come nella serie della [strategia d’investimento](#f-ret-series). Il SARON vale costante come valore fisso.

| Dato | Proposta (esempio Mondo (JST)) | Significato |
|---|---|---|
| Numero di percorsi | 50'000 | come per il bootstrap |
| Rendimento atteso (drift μ) | 12.15 % | drift del rendimento logaritmico più σ²/2; il rendimento annuo medio (aritmetico) senza salti |
| Volatilità σ | 12.25 % | variabilità dei rendimenti annui senza i salti |
| Gradi di libertà | 6 | spessore delle code («fat tails»), 0 = distribuzione normale |
| Salti di corso all’anno | 0.15 | con quale frequenza avviene un crash (0.15 = in media circa ogni 7 anni) |
| Ampiezza media del salto (log) | −25 % | entità del salto; −25 % in rendimento logaritmico sono circa −22 % di perdita di corso |
| Dispersione dell’ampiezza del salto | 15 % | quanto sono diversi i salti |
| Valore iniziale casuale | | vedi [Valore iniziale casuale](#ret-seed) |

**Valori proposti per indice azionario:** l’app calcola i parametri dall’intera serie azionaria dell’indice scelto, in modo che i rendimenti annui del modello (media, dispersione, media geometrica e quantile del 5 %) corrispondano a quelli del [bootstrap storico](#f-ret-bootstrap). Gradi di libertà (6) e dispersione dell’ampiezza del salto (15 %) restano fissi.

| Indice azionario | μ | σ | Salti all’anno | Ampiezza del salto | Anni |
|---|---|---|---|---|---|
| Mondo (JST) | 12.15 % | 12.25 % | 0.15 | −25 % | 1900–2020 |
| Svizzera (JST) | 10.55 % | 14.5 % | 0.26 | −10 % | 1900–2020 |
| S&P 500 (Shiller) | 15.05 % | 16.5 % | 0.26 | −20 % | 1922–2025 |

«Mondo personalizzato» riceve valori dalla propria serie ponderata. Dopo un aggiornamento dei dati Shiller cambiano di conseguenza i valori per lo S&P 500. Per lo S&P 500 il bootstrap estrae mesi; i suoi anni sono così un po’ più miti degli anni civili, e i valori proposti seguono il bootstrap.

- **Nuova pianificazione:** i valori seguono l’indice azionario. Se cambia l’indice sotto «Sostanza e investimento», valgono i valori proposti del nuovo indice.
- **Valori propri:** non appena modifica uno dei sei valori del modello, i Suoi valori restano, anche cambiando l’indice. Accanto a ogni campo figura allora la proposta per l’indice scelto. Con **«Adottare i valori proposti»** i valori seguono di nuovo l’indice.
- Le **pianificazioni meno recenti** che calcolavano già in modo parametrico mantengono i loro valori. All’apertura appare una volta un’avvertenza che obbligazioni e inflazione sono ora casuali.

**Obbligazioni e inflazione:** la scheda «Obbligazioni e inflazione» mostra media, dispersione e la correlazione di azioni, obbligazioni e inflazione, calcolate dai valori annuali della stessa serie usata anche dal bootstrap. Dipende dalla combinazione di indice azionario e obbligazionario. Esempi di correlazione azioni/obbligazioni: Mondo con Confederazione 10 anni −0.01, Svizzera con Confederazione 10 anni 0.19, S&P 500 con USA 10 anni 0.45 (entrambi portano il corso del dollaro). Ogni anno civile il modello estrae un valore per obbligazioni e inflazione correlato con il rendimento azionario dello stesso anno. Gli anni sono indipendenti tra loro come nel bootstrap, non esiste una memoria per inflazione o tassi. Senza serie temporale per obbligazioni e inflazione valgono i valori fissi.

**Matematica in breve:** il rendimento logaritmico per giorno è (μ − σ²/2)·dt + σ·√dt·Z. Z è uno shock con **distribuzione t di Student**, scalato a varianza 1. A ciò si aggiungono **salti di Poisson** con il tasso «Salti di corso all’anno» e ampiezza log-normale. Obbligazioni e inflazione di un anno hanno distribuzione normale con media e dispersione della serie; la loro dipendenza dal rendimento logaritmico standardizzato delle azioni è scelta in modo che la correlazione con il rendimento azionario corrisponda alla serie.

**Come scegliere valori propri:**

- **μ:** a lungo termine le azioni ampiamente diversificate hanno reso storicamente in termini reali dal 5 % al 7 % circa all’anno, in termini nominali di più. Chi pianifica con prudenza usa valori più bassi del passato, perché oggi le valutazioni sono alte. Tenga presente: i salti di corso abbassano ulteriormente il rendimento; il rendimento geometrico atteso figura nell’anteprima a destra.
- **σ:** dal 12 % al 20 % è tipico per le azioni; con salti frequenti più piccolo, perché i salti portano una parte della variabilità.
- **Gradi di libertà:** valori bassi (da 3 a 5) generano giorni estremi, 6 è moderato. Con 0 le fat tails scompaiono, e le crisi diventano irrealisticamente rare.
- **Salti di corso:** insieme alle fat tails fanno sì che crash come nel 2008 o nel 2020 avvengano in modo plausibile. Se imposta il tasso a 0, resta solo il movimento casuale.

**Quando è sensato:** come controprova al bootstrap con distribuzioni regolari invece del numero limitato di anni storici, e se vuole fissare ipotesi proprie (ad esempio ‹in futuro il 4 % di rendimento in meno›). **Limitazioni:** le azioni svizzere ritornano verso la media su più anni; né il bootstrap né il modello lo riproducono. La correlazione è mediata su tutti gli anni; negli anni di inflazione alta azioni e obbligazioni scendono storicamente più spesso insieme. Con «S&P 500» e «Confederazione 10 anni» la serie copre solo gli anni dal 1988, i parametri azionari invece l’intera serie dal 1922.

## @f-ret-series Strategia d’investimento: indici per azioni e obbligazioni
Parole chiave: strategia d’investimento, indice, mondo, JST, Pictet, Shiller, S&P 500, ponderazioni, regioni, UE, USA, Svizzera, Asia-Pacifico, serie temporale, BNS, valuta, breve termine, mercato monetario

I metodi «Storico», «Tutti i momenti d’inizio storici» e «Bootstrap» richiedono una serie temporale con rendimenti di azioni, obbligazioni e inflazione. Essa risulta dalla **strategia d’investimento** nella sezione «Sostanza e investimento»: un indice per le azioni e uno per le obbligazioni.

**Azioni**

| Indice | Contenuto |
|---|---|
| **Mondo (JST)** | azioni di 11 Paesi (Svizzera, USA, Germania, Francia, Italia, Spagna, Paesi Bassi, Belgio, Gran Bretagna, Giappone, Australia), ponderate ogni anno secondo il PIL reale dell’anno precedente, in CHF senza copertura valutaria. **Annuali 1900–2020.** |
| Mondo personalizzato (JST) | come «Mondo (JST)», ma le ponderazioni di **Svizzera, USA, UE e Asia-Pacifico** le stabilisce Lei (insieme 100 %). All’interno dell’UE (Germania, Francia, Italia, Spagna, Paesi Bassi, Belgio, Gran Bretagna) e dell’Asia-Pacifico (Giappone, Australia) i Paesi sono ponderati secondo il PIL. Se cambia la ponderazione dell’UE, la quota di ogni Paese UE cambia nella stessa proporzione: UE 20 % invece di 40 % dimezza ogni Paese UE. Le ponderazioni valgono di nuovo ogni anno (ribilanciamento annuale). |
| Svizzera (JST) | azioni svizzere, rendimento complessivo. Annuali 1900–2020. |
| **S&P 500 (Shiller)** | azioni USA (S&P Composite) in CHF, senza copertura valutaria. **Mensili dal 1922.** |

**Obbligazioni**

| Indice | Contenuto |
|---|---|
| **Confederazione 10 anni** | obbligazioni della Confederazione a 10 anni: JST fino al 1988, poi dai rendimenti mensili della BNS. Annuali 1900–2024; insieme allo S&P 500 mensili dal 02.1988. |
| USA 10 anni (Shiller) | titoli di Stato USA a 10 anni in CHF, senza copertura valutaria. Mensili dal 1922. |
| Confederazione a breve termine | titoli a breve termine della Confederazione (durata circa 3 mesi): JST fino al 2020, poi la media annua del SARON (BNS). **Annuali 1900–2025.** Il rendimento corrisponde praticamente al tasso del mercato monetario: quasi nessuna variabilità, ma a lungo termine un reddito nettamente inferiore alle obbligazioni a 10 anni. |

**Come nasce la serie temporale**

- **Mensili** sono solo «S&P 500» con «USA 10 anni» (dal 01.1922) e «S&P 500» con «Confederazione 10 anni» (dal 02.1988 al 07.2025).
- Tutte le altre combinazioni sono **annuali**, non appena una parte è annuale. I rendimenti mensili vengono allora concatenati in anni civili. La serie comprende gli anni per i quali entrambi gli indici hanno valori: ad esempio «Mondo (JST)» con «USA 10 anni» 1922–2020.
- Il **rincaro** proviene dalla fonte delle azioni (rincaro svizzero in tutti i casi), il **SARON** dai valori della BNS (per le serie annuali come media annua, dal 2000).
- Tutti gli indici sono **in CHF**. Con una pianificazione in EUR o USD appare un’avvertenza.

Sotto la selezione figurano periodo, frequenza e i rendimenti annui medi (geometrici) della strategia scelta; la ⓘ accanto indica fonte e periodo.

**Periodo comune:** i rendimenti medi valgono per gli anni che azioni e obbligazioni hanno in comune. Se un indice va oltre, viene ridotto a questo periodo, e un’avvertenza sotto la selezione indica l’indice, i suoi anni propri e il periodo ridotto. La simulazione (Monte Carlo e andamenti storici) usa solo il periodo comune. Perciò cambia ad esempio il rendimento medio delle obbligazioni «USA 10 anni» se per le azioni passa da «S&P 500» (fino al 2025) a «Svizzera (JST)» (fino al 2020): le obbligazioni contano allora solo fino al 2020.

**Come scegliere?**

- **Mondo (JST):** ampiamente diversificato, lunga storia, anche guerre mondiali e iperinflazione. Attenzione: l’indice è piuttosto **ottimistico**. Per il 1900–2020 supera di circa 1.3 punti percentuali all’anno il noto indice mondiale di Dimson, Marsh e Staunton (5.3 % reale in USD), soprattutto per la ponderazione annuale secondo il PIL e perché contiene solo Paesi che esistono ancora oggi. Vedi [Fonti dei dati](#ret-data).
- **S&P 500:** mensile e quindi il più preciso per bootstrap e rolling; contiene però solo azioni USA e l’alto rendimento della borsa USA.
- **Svizzera (JST):** universo d’investimento di un investitore svizzero con mercato domestico.
- **Mondo personalizzato:** se vuole riprodurre la Sua ripartizione, ad esempio 30 % Svizzera, 40 % USA, 20 % UE, 10 % Asia-Pacifico.

**Pianificazioni meno recenti:** una pianificazione con una serie temporale scelta riceve all’apertura la strategia corrispondente e calcola come prima: ‹Shiller CHF› → S&P 500 con USA 10 anni, ‹Shiller CHF + CH-Obligationen› → S&P 500 con Confederazione 10 anni. ‹Shiller USD› diventa S&P 500 con USA 10 anni in CHF; le **serie temporali proprie** (file CSV) non sono più supportate e vengono sostituite da «Mondo (JST)» con obbligazioni della Confederazione. **Pictet:** l’indice «Svizzera (Pictet)» non è più fornito per motivi di licenza. Una pianificazione con ‹Pictet CH› o con «Svizzera (Pictet)» riceve per le azioni «Svizzera (JST)» e per le obbligazioni «Confederazione 10 anni». In tutti questi casi l’app lo segnala all’apertura, e il risultato cambia.

**Aggiornare:** «Aggiornare i dati Shiller…» importa un nuovo file Shiller (.xls, .xlsx o .ods; download del file `ie_data.xls` su shillerdata.com), «Aggiornare i dati BNS…» nuovi cubi da data.snb.ch. L’app non scarica mai da sé dati da Internet, li scarica Lei a mano. I dati JST terminano nel 2020 e sono integrati in modo fisso.

Ulteriori informazioni: [Fonti dei dati e loro particolarità](#ret-data).

## @f-ret-extrapolation Estrapolazione della serie
Parole chiave: fine della serie, media, fisso, estrapolare

Se la serie temporale termina prima della fine della simulazione (ad esempio «Storico dal 2000» con pianificazione fino al 2058), l’app deve riempire gli anni mancanti. Il **metodo** lo stabilisce qui:

- **Valori fissi:** valgono i rendimenti fissi della sezione «Rendimenti fissi».
- **Media mobile:** ogni valore mancante è la media (aritmetica) degli ultimi *n* anni. La finestra scorre e con il tempo contiene anche i valori già estrapolati. L’estrapolazione converge perciò verso una media tranquilla.
- **Media geometrica mobile:** come sopra, ma con media geometrica.

Il periodo («Media su» … anni) vale per i due metodi mobili. Questo dato appare solo se la serie termina davvero prima che la pianificazione sia finita.

Tenga presente: un’estrapolazione è un’**ipotesi**, non un risultato storico. Con estrapolazioni molto lunghe il risultato perde il suo carattere storico.

## @ret-saron SARON e ipoteche
Parole chiave: SARON, ipoteca, interesse, margine, serie temporale

Il **SARON** serve solo se un’ipoteca ha un periodo SARON. Proviene dalla serie temporale (dal 06.1999, se disponibile), altrimenti vale il **valore SARON fisso** della pianificazione.

- Al di fuori del periodo SARON della serie, con «Rendimenti fissi», con il modello parametrico e dopo un’estrapolazione vale il valore fisso. La vista mostra il campo solo se serve.
- Il tasso d’interesse di una tranche SARON è SARON più margine, con limite inferiore 0.
- Prima del 1999 il SARON non esiste. Per simulazioni storiche prima del 1999 usi il valore fisso come approssimazione.

Altro su tranche e periodi: [Ipoteche](#f-re-mortgage).

## @ret-seed Valore iniziale casuale e riproducibilità
Parole chiave: seed, caso, stessi risultati, riproducibile

Monte Carlo è casuale, ma **riproducibile**. Il percorso *i* nasce sempre dalla coppia (valore iniziale casuale, *i*).

- Stesse impostazioni e stesso valore iniziale danno **esattamente gli stessi risultati**.
- Tutti gli **scenari** usano gli stessi percorsi. Anche nell’**ottimizzazione** tutti i candidati vedono gli stessi percorsi (la verifica finale ne usa altri). Così le differenze provengono dai parametri, non dal caso.
- I risultati sono gli stessi su macOS, Windows e Linux.
- Con una nuova versione può cambiare il procedimento con cui nascono i percorsi casuali. L’app lo indica all’apertura di una pianificazione meno recente. Così «Monte Carlo parametrico» genera i percorsi più velocemente dal #132: la distribuzione dei rendimenti resta la stessa, ma i singoli percorsi sono diversi, e il risultato cambia leggermente.
- Modifichi il valore iniziale se vuole verificare se il Suo risultato dipende dal caso. Se il tasso di successo con un altro valore iniziale varia di più di qualche decimo di punto percentuale, servono più percorsi.

## @ret-data Fonti dei dati e loro particolarità
Parole chiave: Shiller, BNS, Pictet, media mensile, obbligazioni, valuta

- **Shiller (Robert J. Shiller, «Irrational Exuberance»):** azioni come S&P Composite con dividendi. I corsi di Shiller sono **medie mensili**, i crolli appaiono perciò livellati. Rendimento mensile = variazione dell’indice reale di rendimento complessivo × CPI rispetto al mese precedente. Le obbligazioni sono il rendimento complessivo dei titoli di Stato USA a 10 anni.
- **Conversione in CHF:** con la media mensile del tasso di cambio USD/CHF della BNS, senza copertura valutaria, e con il rincaro svizzero (indice nazionale dei prezzi al consumo rispetto allo stesso mese dell’anno precedente). Nasce così il nesso per cui un franco che si apprezza riduce il rendimento di un investitore svizzero in azioni USA.
- **Obbligazioni svizzere:** il rendimento complessivo di un’obbligazione federale a 10 anni (alla pari) viene calcolato dal rendimento mensile della BNS: cedola = rendimento del mese precedente, valutazione al nuovo rendimento con una durata residua di poco meno di 10 anni, più una cedola mensile. La BNS non aggiorna più questa serie dal 07.2025, perciò ‹Shiller CHF + CH-Obligationen› termina lì.
- **JST (Jordà-Schularick-Taylor Macrohistory Database, Release 6):** rendimenti annui di azioni e titoli di Stato per Paese fino al 2020, convertiti in CHF con i tassi di cambio di fine anno. Ponderazione di un Paese: PIL reale dell’anno precedente (PIL pro capite secondo Maddison × popolazione). Gli anni-Paese mancanti vengono tralasciati (Germania 1945–1946, Giappone 1946–1947); la Germania 1923 è esclusa perché nell’iperinflazione il tasso di cambio non si accorda con il rendimento annuo. Le azioni svizzere si basano dal 1926 su Pictet. Rincaro: prezzi al consumo svizzeri (media annua). **Licenza CC BY-NC-SA 4.0:** solo non commerciale, con indicazione della fonte (Jordà, Schularick, Taylor 2017; Jordà, Knoll, Kuvshinov, Schularick, Taylor 2019). Confronto con Dimson, Marsh, Staunton (1900–2020): Mondo (JST) 6.6 % reale all’anno in USD, DMS 5.3 %; le serie dei Paesi stesse coincidono (USA 6.7 % contro 6.6 %).
- **Obbligazioni della Confederazione prima del 1989:** da JST (rendimento complessivo); il 1915 manca (borsa chiusa), lì vale la cedola come rendimento.
- L’**inflazione** è per tutte le serie la variazione dell’indice nazionale rispetto all’anno precedente, così che il valore di dicembre corrisponde al rincaro annuo.
- **Nessun costo:** costi d’investimento, imposte sui redditi e spese di deposito non sono dedotti.
- **Diritti e garanzia:** le serie fornite sono rendimenti calcolati dalle fonti indicate. Non rientrano nella licenza del programma (GPL v3); i diritti sui dati di base spettano alle fonti (Robert J. Shiller, shillerdata.com; Banca nazionale svizzera; Ufficio federale di statistica; Jordà, Schularick, Taylor, macrohistory.net, licenza CC BY-NC-SA 4.0). Shiller avverte che né lui né la Yale University danno una garanzia per i dati e che non sono una previsione né una consulenza in investimenti. Ciò vale anche per questa app.

## @ret-pitfalls Insidie dei rendimenti
Parole chiave: errori, prudenza, survivorship, mercato domestico, eccessivo ottimismo, rischio di sequenza, valuta

- **Il passato non è una previsione.** Nel XX secolo la borsa USA è stata un mercato straordinariamente di successo. Le serie con azioni USA contengono questo successo e possono essere troppo ottimistiche (‹survivorship›). Ciò vale anche per «Mondo (JST)», che contiene solo Paesi che esistono ancora oggi. Verifichi il piano anche con rendimenti più bassi (parametrico con μ = dal 4 % al 5 %) o con azioni svizzere.
- **I periodi sovrapposti simulano sicurezza.** Vedi [Tutti i momenti d’inizio storici](#f-ret-rolling): pochi periodi indipendenti, ma centinaia di simulazioni.
- **Una lunghezza dei blocchi troppo corta nel bootstrap** rende i percorsi troppo regolari. Resti a 12 mesi o più.
- **Rischio di sequenza:** decisivo è **quando** arrivano gli anni negativi. Un crollo nei primi anni del pensionamento agisce molto più fortemente che più tardi. I rendimenti fissi non lo mostrano.
- **Valuta:** tutti gli indici sono in CHF; una pianificazione in EUR o USD calcola comunque con rendimenti in franchi e rincaro svizzero (avvertenza nella verifica). Anche le serie in CHF con azioni estere portano un rischio valutario. L’apprezzamento del franco negli ultimi decenni ha ridotto il rendimento di un investitore svizzero in azioni USA.
- **Confondere nominale e reale:** ‹2 milioni alla fine› sono nominali. Al potere d’acquisto attuale è nettamente meno.
- **Un’unica fonte.** Chi calcola una sola fonte di rendimenti ottiene una risposta **dipendente dal modello**. Calcoli le questioni importanti con bootstrap e modello parametrico.
- **Mancano i costi.** I veri costi d’investimento (fondi, deposito) riducono il rendimento da qualche decimo fino a un punto percentuale all’anno. Li deduca dai valori fissi o parametrici.
