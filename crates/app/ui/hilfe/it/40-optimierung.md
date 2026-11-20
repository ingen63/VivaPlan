# Ottimizzazione

## @s-optimize Ottimizzazione: panoramica
Parole chiave: strategia migliore, parametri, rovina, consumo, ricerca, obiettivo

L’ottimizzazione risponde alla domanda: **quale strategia di prelievo, con quali impostazioni, si adatta meglio al mio scenario di base?**

Prende la Sua pianificazione come data e cerca per ogni [strategia di prelievo](#s-withdrawal) scelta i parametri che raggiungono tre cose contemporaneamente:

1. La **probabilità di rovina** resta sotto il Suo limite (ad esempio 5 %).
2. Il **consumo** è il più alto possibile (mediana e anno sfavorevole, reale).
3. Il consumo **varia** il meno possibile da un anno all’altro.

Le strategie vengono poi confrontate. La migliore figura in alto e reca una stella ★. Con un clic su «Adottare nella pianificazione» riprende strategia e parametri nella Sua pianificazione.

**Presupposti:**

- Una fonte di rendimenti con **dispersione**: Monte Carlo (bootstrap o parametrico) o «Tutti i momenti d’inizio storici». Con «Rendimenti fissi» e «Storico dal momento d’inizio» esiste un solo percorso, la rovina sarebbe 0 % o 100 %. L’app lo segnala e blocca l’avvio.
- Un **pensionamento** nella pianificazione (cassa pensione «Prestazione da»), perché le strategie valgono da allora.

**Quanto dura?** Sotto il pulsante «Avviare l’ottimizzazione» figura una **stima della durata**, che si adegua a ogni modifica delle impostazioni. Si basa su una misurazione: l’app calcola brevemente alcune migliaia di simulazioni della Sua pianificazione e ne estrapola quante simulazioni richiede l’ottimizzazione (percorsi della ricerca, punti d’inizio e strategie, più i percorsi di verifica). L’indicazione è **approssimativa**: il numero di passi di ricerca dipende dal caso, perciò è indicato un intervallo («(per lo più da 7 s a 20 s)»). Esempi su un Mac attuale con 10 core: le impostazioni standard con tutte e sei le strategie durano con il bootstrap circa 15 s, con il modello parametrico altrettanto, con «Tutti i momenti d’inizio storici» da 2 a 3 s. Il doppio dei percorsi significa circa il doppio del tempo. Il calcolo si svolge in locale e usa tutti i core del processore.

**Durante l’ottimizzazione** una barra di avanzamento mostra lo stato (il valore percentuale è una stima, perché il numero di passi di ricerca dipende dal caso). Con **Interrompere** ferma il calcolo; non resta alcun risultato parziale.

**Quali parametri vengono variati** lo stabilisce Lei per ogni strategia, vedi [Parametri fissi o variati](#o-ranges).

**Le impostazioni vengono salvate** (obiettivo, strategie, percorsi, fasce dei parametri) **con la pianificazione**: figurano nel file `.vplan`, ritornano all’apertura e valgono anche per `optimize` nel terminale (le opzioni le sovrascrivono lì). Il risultato di un’ottimizzazione non viene salvato. Un’impostazione modificata non invalida un risultato precedente, solo una pianificazione modificata.

Legga i risultati con [Leggere il risultato](#o-read) e tenga conto delle [insidie](#o-pitfalls). Un ottimo **non è una promessa**.

## @o-goal-select Scelta dell’obiettivo
Parole chiave: obiettivo, nessuna vendita forzata di immobili, vendita forzata, consumo minimo

L’ottimizzazione ha due obiettivi; la scelta viene salvata con la pianificazione (standard: obiettivo 1).

- **Obiettivo 1, consumo con rischio di rovina limitato:** come descritto in questo capitolo: consumo il più alto e regolare possibile, finché la rovina resta sotto il limite.
- **Obiettivo 2, nessuna vendita forzata di immobili:** si cerca la strategia con la **quota minore di simulazioni in cui un immobile viene venduto in modo non pianificato** (vendita forzata). A parità di quota decide la probabilità di rovina minore (rovina come nell’obiettivo 1). Vincoli: la **mediana del consumo annuo dal pensionamento** ammonta almeno alla quota inserita delle spese pianificate (standard 90 %), e la probabilità di rovina è sotto il limite (standard 5 %). Pesi del consumo e volatilità non hanno alcun ruolo nell’obiettivo 2.

La tabella dei risultati indica l’obiettivo scelto. Per l’obiettivo 2 mostra per ogni strategia la **quota delle simulazioni con vendita non pianificata**; l’ordine segue questa quota, a parità la rovina. Le strategie che violano i vincoli figurano in basso e sono contrassegnate. Se nessuna strategia soddisfa i vincoli, in alto figura quella che vi si avvicina di più, con un’avvertenza.

L’obiettivo 2 richiede almeno un immobile nella pianificazione (acquistato o pianificato); senza immobile non è selezionabile.

## @o-goal Obiettivo: probabilità di rovina
Parole chiave: rovina, limite, età, esaurimento, 5 percento, rischio

La probabilità di rovina è il **vincolo** centrale. Una strategia che non la rispetta non può vincere, per quanto alto sia il suo consumo.

- **Probabilità di rovina massima** (standard 5 %): quota massima ammessa dei percorsi in cui la sostanza è esaurita. **Rovina** significa: nessuna sostanza liquida e nessun immobile più. Usuali sono dall’1 % al 10 %. Più basso è il limite, più sicuro e più basso è il consumo possibile.
- **La rovina conta fino all’età** (vuoto = fino alla fine della simulazione): se ha bisogno di sicurezza solo fino a una determinata età, la inserisca qui. Una rovina a 92 anni non conta allora con ‹fino a 90›. È un **allentamento consapevole**, pensi alla longevità. Con il campo vuoto il limite vale fino alla fine della simulazione.

> Quale limite è ragionevole? Una probabilità di rovina del 5 % significa che in 1 futuro simulato su 20 il denaro finisce. Lo confronti con le Sue riserve: AVS e cassa pensione continuano, la sostanza è solo una parte del finanziamento. Con due o più fonti dal risultato diverso, un limite più basso (dal 2 % al 3 %) è una prudenza ragionevole.

## @o-consumption Obiettivo: consumo (mediana e P10)
Parole chiave: mediana, percentile, P10, peso, consumo, spese, reale

Ciò che viene ottimizzato è il **consumo annuo dal pensionamento**: le spese complessive incluse le spese d’alloggio, **prima delle imposte**, ai **prezzi del mese d’inizio** (quindi reale). L’app raccoglie i valori annuali **di tutti i percorsi e di tutti gli anni** e ne ricava due indicatori:

- **Mediana:** il consumo annuo tipico. Metà di tutti gli anni-percorso è al di sopra, metà al di sotto.
- **P10 (10° percentile):** il consumo annuo al di sotto del quale si scende nel 10 % degli anni-percorso. È il **caso sfavorevole** e misura quanto diventa dura nelle crisi.

**Peso della mediana nell’obiettivo di consumo** (standard 50 %): come l’obiettivo di consumo viene composto dai due indicatori. Il resto va al P10.

> Obiettivo di consumo = peso × mediana + (1 − peso) × P10

- **100 %:** conta solo il consumo tipico. L’ottimizzazione sceglie impostazioni più aggressive con mediana alta e anni di crisi difficili.
- **0 %:** conta solo il caso sfavorevole. Ciò porta a impostazioni prudenti.
- **50 %** è un punto di partenza equilibrato. Aumenti il peso del P10 se per Lei l’**affidabilità** è più importante della media.

Dalla rovina conta come consumo solo ciò che coprono redditi e rendite, non più le spese pianificate. Una rovina abbassa quindi la mediana e soprattutto il P10.

## @o-vol Obiettivo: volatilità
Parole chiave: volatilità, variabilità, da un anno all’altro, deduzione, riduzione, stabile

La **volatilità** misura quanto il consumo annuo cambia da un anno al successivo: la **variazione assoluta media** rispetto all’anno precedente, mediata su tutti i percorsi. 10 % significa che il consumo in media aumenta o diminuisce del 10 % all’anno. Le spese fisse (Standard, 3 comparti) hanno una volatilità vicina allo 0 %, le strategie che seguono la sostanza (tasso fisso, VPW) una alta.

Due impostazioni gestiscono come la volatilità entra nell’obiettivo:

- **Deduzione per volatilità** (standard 1): l’obiettivo di consumo viene moltiplicato per il fattore (1 − deduzione × volatilità). Con deduzione 1 e 10 % di volatilità l’obiettivo cala del 10 %. **0 significa che la volatilità è indifferente**, allora vincono le strategie con il consumo più alto, anche se varia molto.
- **Volatilità massima** (vuoto = nessun limite): un **limite superiore rigido** come vincolo, come per il limite di rovina. Esempio 10 %: le strategie la cui variazione annua media è più alta sono escluse. È l’impostazione per ‹Voglio che le mie spese non varino in media più del 10 % all’anno›.

## @o-score L’obiettivo in una formula
Parole chiave: formula, valore obiettivo, valutazione, fitness, score

> **Obiettivo** = (peso × mediana + (1 − peso) × P10) × (1 − deduzione × volatilità)

L’obiettivo viene mostrato in franchi all’anno (prezzi reali). **Un candidato è ammissibile** se la probabilità di rovina è sotto il limite e (se impostata) la volatilità resta sotto il limite massimo. Tra gli ammissibili vince quello con l’obiettivo più alto. Se due hanno lo stesso obiettivo (ad esempio con spese fisse, dove tutte le strategie hanno lo stesso consumo), vince quello con la **probabilità di rovina minore**. I candidati non ammissibili vengono ordinati secondo quanto violano il limite.

Con spese fisse (Standard, 3 comparti) il consumo è uguale in tutte le simulazioni. Lì l’ottimizzazione decide quindi **sulla sicurezza**, non sul consumo. Con le strategie che seguono la sostanza il consumo risulta dalla strategia.

## @o-strategies Strategie e loro parametri
Parole chiave: parametri, campo di ricerca, Standard, 3 comparti, Guardrails, VPW, Endowment, tasso fisso, quota azionaria

Scelga le strategie da confrontare. Si consiglia di calcolarle **tutte**, per vedere come si differenziano gli approcci. Per ogni strategia l’app cerca questi parametri in questi **campi standard**. Quali parametri vengono variati e in quale fascia lo stabilisce Lei, vedi [Parametri fissi o variati](#o-ranges).

| Strategia | Parametri | Campo di ricerca |
|---|---|---|
| Standard | quota azionaria, riserva (mesi) | dal 10 % al 100 %, da 3 a 36 |
| 3 comparti | comparto 1: cash (anni), comparto 2: obbligazioni (anni), rifornimento dalle azioni a partire dal rendimento dell’anno precedente | da 0.5 a 4, da 0 a 10, da −10 % a +20 % |
| Guardrails | tasso iniziale, guardrail superiore e inferiore, adeguamento, quota azionaria | dal 2 % all’8 %, ciascuno dal 5 % al 50 %, dal 2 % al 30 %, dal 10 % al 100 % |
| Tasso fisso | tasso di prelievo, quota azionaria | dal 2 % all’8 %, dal 10 % al 100 % |
| VPW | età di pianificazione, rendimento reale, quota azionaria | da 85 a 105, da −1 % a 5 %, dal 10 % al 100 % |
| Endowment | tasso obiettivo, livellamento, quota azionaria | dal 2 % all’8 %, dal 5 % al 100 %, dal 10 % al 100 % |

**Importante:**

- La **quota azionaria** viene modificata **dal pensionamento**. Le righe precedenti di un valore dipendente dal tempo restano.
- I **limiti del costo della vita** (limite inferiore, limite superiore) restano per impostazione standard fissi sul valore della Sua pianificazione. Con Guardrails e tasso fisso può farli variare. Un limite inferiore può limitare molto l’ottimizzazione, perché impedisce le riduzioni.
- La **strategia a 3 comparti** stabilisce la quota azionaria tramite i comparti, perciò lì non viene cercata.
- Un parametro al bordo del campo indica che il campo è limitante: l’ottimo è al bordo, non oltre.

I parametri sono descritti con le strategie: [Standard](#f-wd-standard), [3 comparti](#f-wd-three_buckets), [Guardrails](#f-wd-guardrails), [Tasso fisso](#f-wd-fixed_rate), [VPW](#f-wd-vpw), [Endowment](#f-wd-endowment).

## @o-ranges Parametri fissi o variati
Parole chiave: fascia, fisso, variare, da, a, parametro, campo di ricerca, fasce standard, comparto

Sotto «Parametri delle strategie» figura per ogni strategia una tabella con tutti i suoi parametri:

- **Nella pianificazione:** il valore che la Sua pianificazione ha oggi (o il valore standard della strategia, se non l’ha mai usata).
- **Variare:** con la spunta l’ottimizzazione cerca questo parametro tra «da» e «a». **Senza spunta resta fisso** sul valore della pianificazione.
- **Da / a:** la fascia nell’**unità del parametro**: anni in anni, mesi in mesi, percentuali in percentuali, età in anni.

**Esempio 3 comparti:** comparto 1 fisso (nessuna spunta), comparto 2 da 0 a 2 anni, rifornimento a partire da un rendimento dell’anno precedente da −30 % a +10 %. La ricerca modifica allora solo il comparto 2 e la soglia di rifornimento, il comparto 1 resta come figura nella pianificazione.

**Buono a sapersi:**

- Una **fascia stretta** rende la ricerca più veloce e il risultato più vicino alla Sua pianificazione. Una **ampia** trova più facilmente sorprese, ma richiede più tempo di calcolo.
- Se **tutti** i parametri di una strategia sono fissi, viene calcolata solo quest’unica pianificazione. È utile per confrontare equamente strategie con valori fissi.
- «da» non può essere maggiore di «a». Esistono limiti estremi per parametro (ad es. quota azionaria da 0 % a 100 %); al di fuori l’app segnala un errore.
- **Limite inferiore e superiore del costo della vita** (Guardrails, tasso fisso) sono inizialmente fissi. Chi li fa variare ottiene spesso risultati che sembrano ‹buoni› solo grazie al limite inferiore, perché le riduzioni vengono impedite.
- «Fasce standard» riporta la strategia all’impostazione predefinita.
- Le impostazioni vengono **salvate con la pianificazione**.

## @o-advanced Impostazioni avanzate
Parole chiave: percorsi, verifica, punti d’inizio, precisione, tempo di calcolo

- **Percorsi per la ricerca** (standard 5'000): numero di percorsi casuali che la ricerca valuta per ogni candidato. **Più percorsi** rendono il risultato più preciso, ma più lento. Regola empirica: sotto 2'000 percorsi la valutazione diventa instabile, oltre 20'000 lo sforzo di solito vale poco.
- **Percorsi per la verifica** (standard 10'000): su tanti percorsi **diversi** vengono ricalcolati alla fine i parametri trovati. Questi valori li vede nel risultato.
- **Punti d’inizio casuali per strategia** (standard 30): la ricerca inizia dalla pianificazione, dal centro del campo di ricerca e da questo numero di punti casuali. Più punti d’inizio aiutano a non restare bloccati in un cattivo ottimo locale, ma costano tempo, soprattutto con strategie con molti parametri (Guardrails).

I percorsi provengono dalla fonte di rendimenti della pianificazione (bootstrap o parametrica). I percorsi di verifica hanno un **altro valore iniziale casuale**. Con «Tutti i momenti d’inizio storici» non vi sono percorsi casuali. Ricerca e verifica calcolano allora gli stessi segmenti storici, la **verifica lì non è indipendente**.

## @o-algorithm Come funziona la ricerca
Parole chiave: algoritmo, ricerca casuale, ricerca a schema, margine di sicurezza, verifica, indipendente, sovradattamento

1. **Percorsi comuni:** tutti i candidati vengono valutati con **gli stessi** percorsi dei rendimenti. Le differenze provengono così dai parametri, non dal caso.
2. **Ricerca casuale:** l’app valuta i punti d’inizio (la pianificazione attuale, il centro del campo e punti casuali) e prende il migliore.
3. **Ricerca a schema (ricerca per coordinate):** dal punto migliore ogni parametro viene singolarmente aumentato e diminuito. Se una modifica porta un miglioramento, viene adottata. Se non se ne trova alcuna, l’ampiezza del passo viene dimezzata, finché è molto piccola.
4. **Margine di sicurezza:** altrimenti la ricerca sfrutterebbe il limite di rovina fino in fondo e su nuovi percorsi finirebbe appena sopra (errore casuale della stima). Perciò cerca con un limite **più severo di 1.5 errori standard**. Con un limite del 5 % e 5'000 percorsi sono circa 0.5 punti percentuali.
5. **Verifica indipendente:** il risultato viene ricalcolato su **altri** percorsi (altro valore iniziale casuale, più percorsi). Solo questa verifica decide se una strategia è ammissibile.

La verifica protegge dal **sovradattamento**: una strategia che sembra buona solo sui percorsi di ricerca non supera la verifica.

## @o-read Leggere il risultato
Parole chiave: tabella, situazione di partenza, stella, in grigio, rovina, mediana, P10, volatilità, obiettivo, barra, verifica

La tabella dei risultati mostra per ogni strategia gli indicatori, **calcolati sui percorsi di verifica indipendenti**. Il consumo è indicato in franchi all’anno, ai prezzi del mese d’inizio, incluse le spese d’alloggio e prima delle imposte.

| Colonna | Significato |
|---|---|
| Strategia | nome; ★ contrassegna la migliore ammissibile |
| Rovina | quota dei percorsi in cui la sostanza è esaurita (fino all’età scelta) |
| Mediana | consumo annuo tipico |
| P10 | consumo annuo nel caso sfavorevole |
| Volatilità | variazione media del consumo annuo rispetto all’anno precedente |
| Obiettivo | il valore combinato secondo la [formula](#o-score), con barra per il confronto |
| Tasso di prelievo | prelievo dalla sostanza nel **primo anno di pensionamento** (costo della vita e spese d’alloggio meno entrate, **prima delle imposte**), in percentuale della sostanza liquida all’inizio; mediana sui percorsi |
| Parametri | le impostazioni trovate; pulsante per adottarle |

- La prima riga **«Situazione di partenza»** mostra la Sua pianificazione invariata, sugli stessi percorsi di verifica. Con essa misura il guadagno dell’ottimizzazione.
- Le **righe in grigio** («Limite di rovina non rispettato») sono strategie che mancano il Suo limite di rovina. Restano visibili, affinché riconosca i costi della sicurezza. Se **nessuna** strategia è ammissibile, il limite non è raggiungibile con i mezzi disponibili; deve allora ridurre le spese, andare in pensione più tardi o allentare il limite.
- Il **tasso di prelievo** è un valore di confronto, non un obiettivo della ricerca. Mostra quanto una strategia attinge alla sostanza all’inizio: un tasso alto con un consumo altrettanto alto significa che la strategia attinge di più alla sostanza e si basa meno su rendite e reddito. La regola empirica «sotto il 4 %» vale solo approssimativamente (vedi [Prelievo dalla sostanza](#r-draw)).
- Se il consumo cala molto rispetto alla situazione di partenza, la sicurezza costa di conseguenza. Non è **un errore**, ma la risposta.
- Sotto la tabella figurano il numero di percorsi per ricerca e verifica e il tempo di calcolo.

Se la pianificazione è stata modificata dopo l’ottimizzazione, appare un’avvertenza: il risultato non corrisponde più alla pianificazione e deve essere ricalcolato.

## @o-apply Adottare nella pianificazione
Parole chiave: adottare, parametri, impostare la strategia, applicare

«Adottare nella pianificazione» imposta nella Sua pianificazione:

- la **strategia** e i suoi parametri (ad esempio tasso iniziale e guardrail con Guardrails),
- la **quota azionaria** e la **riserva di cash** dal pensionamento.

La pianificazione è poi considerata modificata (non salvata). Con **Annullare** (⌘Z) l’adozione può essere revocata. Il risultato dell’ottimizzazione resta visibile e continua a valere, finché non modifica la pianificazione in altro modo.

Verifichi poi il risultato con una **simulazione normale** e, se possibile, con un’**altra fonte di rendimenti**. Se l’ottimo è ancora buono lì, è robusto.

## @o-pitfalls Insidie dell’ottimizzazione
Parole chiave: sovraottimizzazione, robustezza, rischio di modello, controprova, avvertimento

Un’ottimizzazione trova il miglior risultato **per le ipotesi**. Non dice nulla sul fatto che le ipotesi siano giuste.

- **L’ottimo vale solo per questi rendimenti.** Se si è ottimizzato con la serie Shiller, il risultato non vale necessariamente con un’altra serie o con rendimenti più bassi. **Verifichi il risultato con una seconda fonte**: ottimizzi con il bootstrap e verifichi con il modello parametrico, o viceversa.
- **I numeri precisi danno un’illusione di precisione.** Una quota azionaria del 63.4 % non è migliore del 60 %. Arrotondi a valori comprensibili e verifichi quanto cambia così l’obiettivo. Spesso la superficie attorno all’ottimo è **piatta**.
- **Soluzioni al bordo:** se un parametro è al bordo del campo di ricerca (ad esempio quota azionaria 100 % o tasso iniziale 8 %), è un segno che il risultato è estremo. Ne diffidi.
- **Un limite di rovina troppo ottimistico.** Con il 10 % o più sono facilmente possibili valori di consumo alti, ma con una probabilità di 1 su 10 perde la Sua sostanza.
- **Rischio di sequenza e periodi lunghi.** La stima della rovina con il bootstrap e il modello parametrico dipende molto dalle ipotesi sulle crisi.
- **Aspetti non modellati:** costi d’investimento, modifiche di legge, costi di cura, decessi. Vedi [Limiti del modello](#limits).
- **Adattamento ai percorsi casuali:** senza verifica indipendente l’ottimizzazione reagirebbe a variazioni casuali. L’app verifica perciò su altri percorsi. Con «Tutti i momenti d’inizio storici» manca questa indipendenza.

> Un calcolo modello non è una consulenza in investimenti. Un ottimo è un **punto di partenza per le Sue riflessioni**, non un incarico.

## @o-example Esempio di interpretazione
Parole chiave: esempio, interpretazione, istruzioni, procedura

I numeri sono inventati e servono solo a esercitarsi nella lettura.

| Strategia | Rovina | Mediana | P10 | Volatilità | Obiettivo |
|---|---|---|---|---|---|
| Situazione di partenza (Standard) | 7.8 % | 72'000 | 72'000 | 0 % | 72'000 |
| Guardrails ★ | 3.9 % | 78'500 | 66'000 | 6 % | 68'000 |
| Tasso fisso | 1.1 % | 84'000 | 51'000 | 21 % | 53'000 |

- La situazione di partenza **non** rispetta il limite di rovina del 5 % (7.8 %).
- Guardrails raggiunge il **3.9 %** e fornisce in mediana **più** consumo. In compenso il consumo nel caso sfavorevole scende a 66'000 e varia del 6 % all’anno. È il prezzo della sicurezza.
- Il tasso fisso sarebbe ancora più sicuro, ma varia così tanto (21 %) che l’obiettivo è inferiore a Guardrails.

**Che cosa ne ricava:** verifichi se potrebbe vivere con 66'000 nel caso sfavorevole (alloggio, salute, costo della vita minimo). Se sì, Guardrails è un buon candidato. Lo ricalcoli poi con una seconda fonte di rendimenti.

## @o-cli Ottimizzazione nella riga di comando
Parole chiave: terminale, optimize, comando, opzioni, script

Lo stesso calcolo esiste nel terminale:

```
vivaplan optimize <pianificazione.vplan> [opzioni]
```

| Opzione | Significato |
|---|---|
| `--goal 1\|2` | obiettivo: 1 consumo con rischio di rovina limitato (standard), 2 nessuna vendita forzata di immobili |
| `--min-consumption <percento>` | obiettivo 2: quota minima delle spese pianificate per la mediana del consumo (standard 90) |
| `--max-ruin <percento>` | probabilità di rovina massima |
| `--ruin-age <età>` | la rovina conta solo fino a questa età |
| `--weight-median <da 0 a 1>` | peso della mediana nell’obiettivo di consumo |
| `--vol-weight <numero>` | deduzione per la volatilità |
| `--max-vol <percento>` | volatilità massima |
| `--paths`, `--check-paths`, `--samples` | percorsi della ricerca e della verifica, punti d’inizio |
| `--seed` | valore iniziale casuale |
| `--strategy a,b` | strategie (`standard`, `three_buckets`, `guardrails`, `fixed_rate`, `vpw`, `endowment`) |
| `--json <file>` | salvare il risultato come JSON |

Se nella pianificazione sono salvate impostazioni, valgono come punto di partenza. Ogni opzione indicata le sovrascrive. Le **fasce dei parametri** (`optimization.ranges`) il comando le legge sempre dalla pianificazione; nel terminale non vi sono opzioni per esse. L’output mostra per ogni strategia i parametri e gli indicatori di ricerca e verifica; è in inglese (vedi [Riga di comando](#ref-cli)).
