# Editor: sezioni e campi

## @s-general Economia domestica e progetto
Parole chiave: nome, valuta, persone, nascita, seconda persona, doppio reddito, periodo di simulazione

Qui stabilisce i dati di base.

- **Nome del progetto e valuta:** tutti gli importi vengono registrati nella valuta del progetto (CHF, EUR o USD). Gli indici della [strategia d’investimento](#f-ret-series) sono in CHF; con una pianificazione in EUR o USD appare un’avvertenza.
- **Persona 1:** nome e mese di nascita (formato MM.AAAA). Dal mese di nascita l’app calcola l’età per le tappe (‹a 65 anni›) e per le didascalie dei grafici.
- **+ Seconda persona:** aggiunge una seconda persona con reddito proprio, cassa pensione propria e AVS propria (doppio reddito). Imposte, sostanza e spese valgono per l’economia domestica. Vedi [Seconda persona e regole per coppie sposate](#f-couple).
- **Senza cassa pensione (grande pilastro 3a):** l’opzione si trova per ogni persona nella sezione [Previdenza](#s-pension), in cima alla scheda [Cassa pensione](#f-pk).
- **Periodo di simulazione:** inizio e fine sono tappe e si modificano sotto [Tappe](#s-milestones).

Tutte le indicazioni temporali hanno il formato **MM.AAAA** (ad esempio 05.2027).

I **numeri** si inseriscono con o senza separatore delle migliaia: apostrofo o spazio (1'234 o 1 234), come separatore decimale punto o virgola (12.5 o 12,5). Negli **importi** e nei **numeri interi** (ad esempio numero di percorsi) un punto seguito da esattamente tre cifre è ambiguo (‹12.500›): il campo segnala «Numero ambiguo» e mantiene il valore precedente; scriva 12500 o 12'500. I campi percentuali e degli anni accettano ‹1.25› come finora. Scritture come ‹1e5› o ‹0x10› non sono ammesse da nessuna parte.

## @s-milestones Tappe
Parole chiave: prestazione della cassa pensione, pensionamento, inizio, fine, inizio della rendita AVS, AVS, data, età, momento, limiti

Una tappa è un momento con un nome a cui possono riferirsi tutti gli altri dati (‹da prestazione della cassa pensione›). Se sposta la prestazione della cassa pensione, si sposta tutto ciò che ne dipende. È la leva più importante per provare scenari ‹andare in pensione prima o dopo›.

Fissi sono l’**inizio della simulazione** e la **fine della simulazione**. Inoltre **ogni persona** ha due tappe fisse sempre presenti:

| Tappa | Significato | Limiti | Standard |
|---|---|---|---|
| **Prestazione della cassa pensione** (finora ‹pensionamento›) | riscossione di rendita e capitale della cassa pensione, inizio della strategia di prelievo | da 58 a 65 anni | 65 |
| **Inizio della rendita AVS** | inizio della rendita AVS; fine dei versamenti e dell’ammortamento indiretto nel [pilastro 3a](#s-pillar3a), versamento del resto | da 63 a 70 anni | 65 |

Non si possono eliminare, ma il loro momento è liberamente selezionabile (età o data), purché sia entro i limiti; i limiti stessi sono ammessi. I limiti sono modificabili nelle [Impostazioni](#f-settings). Con due persone si chiamano «Prestazione della cassa pensione persona 2» e «Inizio della rendita AVS persona 2». I nomi delle tappe fisse appaiono nella lingua scelta e non vengono salvati nel file di pianificazione. La **fine del reddito da attività lucrativa** non è una tappa a sé: con l’interruttore «Il reddito da attività lucrativa termina con la prestazione della cassa pensione» nel [reddito](#s-flows) termina con la prestazione della cassa pensione della persona e si sposta con essa. Se l’interruttore è disattivato e dopo la prestazione della cassa pensione il reddito da attività lucrativa è ancora maggiore di 0, la verifica mostra un avviso con l’importo.

Tappe proprie si aggiungono con «+ Tappa», ad esempio ‹Verkauf Haus› o ‹Kinder ausgezogen›.

Ogni tappa è definita come:

- **Data:** un mese fisso (05.2027)
- **Età di:** l’età di una persona, ad esempio ‹persona 1 a 64.5 anni›. I decimali sono ammessi: 64.5 anni sono 64 anni e 6 mesi.

La colonna «Risulta» mostra il mese calcolato da questi dati. **Al passaggio** tra «Data» ed «Età di» il momento resta lo stesso: l’età o la data della colonna «Risulta» viene inserita nel nuovo campo. La tappa si può eliminare con ‹✕›. **Se è ancora utilizzata**, un dialogo chiede conferma: indica il numero di dati e le sezioni e, con l’eliminazione, li converte nella **data fissa** (mese/anno) della tappa, con «Età di» nella data calcolata. Se così nasce in una serie temporale un momento doppio, vale la riga successiva e un messaggio lo segnala. Con «Annullare» (⌘Z) si può revocare tutto. Non eliminabili sono **inizio della simulazione, fine della simulazione, prestazione della cassa pensione e inizio della rendita AVS**; il pulsante è in grigio.

**Pianificazioni meno recenti:** se manca una delle tappe fisse, all’apertura viene aggiunta a 65 anni. Se un’età è al di fuori dei limiti, viene posta sul limite più vicino. ‹Pensionamento› si chiama ora «Prestazione della cassa pensione». Un dialogo indica ogni modifica.

> La **fine della simulazione** dovrebbe essere fissata consapevolmente tardi, ad esempio all’età di 95 o 100 anni. Una fine troppo precoce rende il risultato migliore di quanto sia: il denaro deve bastare solo fino ad allora.

## @s-pension Previdenza
Parole chiave: cassa pensione, AVS, 2° pilastro, 1° pilastro, pilastro 3a, rendita

Cassa pensione (2° pilastro) e AVS (1° pilastro), con due persone schede proprie per ogni persona. I numeri si trovano sul **certificato di previdenza** della Sua cassa pensione e nel **conto AVS** (estratto conto individuale o calcolo anticipato della rendita della cassa di compensazione).

- Cassa pensione: [avere di vecchiaia, prestazione, prelievo del capitale, aliquota di conversione](#f-pk)
- AVS: [rendita e inizio della riscossione](#f-ahv)
- Casa di cura: [entrata, spese della casa di cura, vendita dell’abitazione di proprietà](#f-care-home)

Una riga sotto le schede mostra che cosa diventa l’avere alla prestazione della cassa pensione (capitale e rendita).

Il **pilastro 3a** ha una sezione propria, vedi [Pilastro 3a](#s-pillar3a).

## @s-pillar3a Pilastro 3a
Parole chiave: 3a, previdenza vincolata, versamento, riscossione, piano di riscossione, età, data, avere, importo massimo, ammortamento indiretto

Il pilastro 3a è un conto proprio **per persona**, separato da cassa pensione e ipoteca.

- **Avere all’inizio della simulazione** e **remunerazione / rendimento annuo**.
- **Versamento all’anno:** da 0 all’importo massimo (7'258 franchi; con «Senza cassa pensione (grande pilastro 3a)» per la persona 36'288 franchi, al massimo il 20 % del reddito da attività lucrativa; modificabile nelle [Impostazioni](#f-settings)). L’app versa ogni mese un dodicesimo, fino a prima del mese dell’**inizio della rendita AVS**. Il versamento **conta tra le spese**: appare in «Spese totali» dei valori annuali, aumenta il prelievo e fa parte del fabbisogno netto della strategia a 3 comparti.
- **Piano di riscossione:** righe composte da momento e importo, ad esempio 2041 50'000 e 2042 50'000. Il **momento** lo stabilisce come per una tappa come **data** (MM.AAAA) o come **età** della persona a cui appartiene il conto (ad esempio 61.5 anni). Accanto figurano il mese che ne risulta e l’età. Se passa tra «Data» ed «Età», il momento resta uguale. Senza importo viene riscosso l’intero avere. Una riscossione è possibile **al più presto 5 anni prima dell’inizio della rendita AVS**, anche secondo l’età; una voce anteriore è un errore sul campo. Se un importo è superiore all’avere, viene riscosso solo l’avere (avvertenza negli eventi).
- **Avere dall’anteprima:** ogni riga mostra l’**avere prima della riscossione** e il **resto dopo**, sotto il piano figura l’avere versato all’inizio della rendita AVS. I valori provengono dall’anteprima con rendimenti fissi; se la pianificazione ha un errore, non c’è anteprima, e figura ‹–›.
- **+ Riscossione** crea una riga un anno dopo l’ultima (senza righe: nel primo mese ammesso, 5 anni prima dell’inizio della rendita AVS, non prima dell’inizio della simulazione), al massimo all’inizio della rendita AVS. Come importo figura l’avere in quel momento dopo le riscossioni precedenti, arrotondato per difetto a 1'000. In seguito può modificarlo liberamente.
- **Resto all’inizio della rendita AVS:** ciò che resta sul conto viene versato **al più tardi nel mese d’inizio della rendita AVS**.
- **Utilizzo:** nella sostanza libera o, se esiste una tranche con ammortamento indiretto in questo conto, «Rimborsare l’ipoteca» (prima le tranche con ammortamento indiretto).

Ogni riscossione è un **prelievo di capitale** con imposta sulle prestazioni in capitale. I prelievi nello stesso anno civile (cassa pensione e pilastro 3a) vengono sommati per l’aliquota, per le coppie non sposate per persona.

L’**ammortamento indiretto** di un’ipoteca confluisce in questo conto fino a prima dell’inizio della rendita AVS (vedi [Ipoteche](#f-re-mortgage)). Versamento e ammortamento indiretto insieme non dovrebbero superare l’importo massimo annuo; altrimenti la verifica mostra un avviso. Senza cassa pensione la verifica avvisa inoltre non appena la somma supera il 20 % del reddito da attività lucrativa all’anno (con il primo mese in cui il reddito è troppo basso); viene dedotta solo questa quota.

> Esempio: nascita 06.1980, avere 120'000 senza interesse, riscossione a 61.5 anni (risulta 12.2041) di 50'000: avere prima della riscossione 120'000, resto dopo 70'000. «+ Riscossione» propone una riga a 62.5 anni (12.2042) di 70'000.

> Esempio: avere 120'000, riscossioni 2041 e 2042 di 50'000 ciascuno, inizio della rendita AVS 2045: il resto di 20'000 (più interesse) viene versato nel mese d’inizio della rendita AVS. Se l’inizio della rendita AVS viene spostato al 2046, il piano resta valido, e il resto arriva nel 2046.

## @f-settings Impostazioni
Parole chiave: impostazioni, limiti, età AVS, prestazione della cassa pensione, importo massimo, 3a, valori standard, lingua, aspetto, scuro, chiaro, sostenibilità, grado d’anticipo, rincaro, anno iniziale

Sul Mac menu ‹VivaPlan → Impostazioni…› (⌘,), con Windows e Linux menu «File → Impostazioni…» (Ctrl+,), oppure il pulsante «Impostazioni» in alto. Le impostazioni valgono per **tutte le pianificazioni su questo dispositivo** e vengono salvate in locale, **non nel file di pianificazione**.

| Impostazione | Standard |
|---|---|
| Lingua | come il sistema |
| Aspetto | Sistema |
| Inizio della rendita AVS: limite inferiore e superiore | 63 e 70 anni |
| Prestazione della cassa pensione: limite inferiore e superiore | 58 e 65 anni |
| Pilastro 3a: importo massimo del versamento all’anno | 7'258 |
| Grande pilastro 3a (senza cassa pensione): importo massimo all’anno | 36'288 |
| Rincaro nel modello parametrico dall’anno | 1950 |
| Sostenibilità: quota massima del reddito | un terzo (33 %) |
| Sostenibilità: tasso d’interesse teorico | 5 % |
| Sostenibilità: spese accessorie annue (del prezzo) | 1 % |
| Sostenibilità: grado d’anticipo massimo | 80 % |

**Lingua:** «come il sistema» segue la lingua del sistema operativo (al primo avvio; senza regione, it_CH vale come italiano), altrimenti vale la lingua scelta. Sono offerti tedesco, inglese, francese e italiano; se manca la lingua del sistema, vale il tedesco. L’aiuto appare nella lingua scelta; un capitolo non ancora tradotto appare in tedesco. La stessa scelta si trova nel menu «Vista → Lingua». Un cambio ha effetto subito, anche sui menu; la pianificazione, le modifiche non salvate e la vista restano. Numeri, importi e date appaiono in ogni lingua nel formato svizzero (1'234'567.50, 2.5 %, 03.2040), e il file di pianificazione è lo stesso in ogni lingua. I nomi nella pianificazione (persone, immobili, tappe) non vengono tradotti; solo la pianificazione d’esempio riceve i suoi nomi nella lingua scelta all’apertura, con le stesse cifre.

**Aspetto:** «Sistema» segue l’aspetto chiaro o scuro del sistema operativo, anche se cambia durante il lavoro; «Chiaro» e «Scuro» valgono indipendentemente. Il tema vale per tutta l’app: campi di immissione, menu di selezione, dialoghi, guida e grafici. La scelta ha effetto al salvataggio.

**Sostenibilità:** I quattro valori della [sostenibilità](#f-re-afford) sono regole della banca, non dati della propria pianificazione; ogni valore deve essere un numero. Una modifica agisce subito sull’anteprima e sulla simulazione. La sezione «Immobili» mostra i valori applicabili in un’avvertenza. Se si apre un vecchio file di pianificazione che contiene ancora valori propri diversi dalle impostazioni, l’app riprende questi valori nelle impostazioni; il dialogo «Avvertenze sulla pianificazione» indica i valori ripresi e quelli precedenti. I risultati salvati e il confronto delle pianificazioni mostrano i valori con cui è stato calcolato. La riga di comando usa i valori di un vecchio file, altrimenti, come l’app, i valori delle impostazioni (vedi [riga di comando](#ref-cli)).

**Rincaro nel modello parametrico dall’anno:** «Monte Carlo · parametrico» stima media, dispersione e correlazioni del rincaro solo dagli anni della serie a partire da questo anno (predefinito 1950). Altrimenti gli anni di guerra e del dopoguerra fanno salire molto la dispersione: per «Mondo (JST)» 5.0 % all’anno con tutti gli anni dal 1900, 2.3 % dal 1950, con una media quasi uguale (2.3 % e 2.2 %). Azioni e obbligazioni usano sempre l’intera serie; le fonti storiche (fisso, storico, rolling, bootstrap) non sono interessate. Un anno precedente l’inizio della serie usa l’intera serie. Se dall’anno iniziale restano meno di 10 anni, la simulazione segnala un errore e non calcola. Vedi [Monte Carlo · parametrico](#f-ret-parametric).

Il limite inferiore non può essere superiore a quello superiore, le età sono anni interi, l’importo massimo è 0 o superiore. «Valori standard» ripristina tutto dopo una domanda di conferma. Se dopo una modifica una pianificazione è al di fuori dei nuovi limiti, la verifica lo segnala; la stessa pianificazione può perciò dare un messaggio su un altro dispositivo con altre impostazioni. La riga di comando usa gli stessi limiti dell’app (vedi [riga di comando](#ref-cli)).

## @f-pk Cassa pensione
Parole chiave: LPP, avere di vecchiaia, contributi, remunerazione, certificato di previdenza, senza cassa pensione, grande pilastro 3a, indipendenti

- **Senza cassa pensione (grande pilastro 3a):** opzione in cima alla scheda, per persona, per chi esercita un’attività lucrativa senza cassa pensione, soprattutto gli indipendenti. Per questa persona versamento e deduzione del [pilastro 3a](#s-pillar3a) arrivano fino al **20 % del reddito da attività lucrativa**, al massimo fino all’importo massimo del grande pilastro 3a (36'288 franchi, modificabile nelle [Impostazioni](#f-settings)); per tutte le altre vale il piccolo pilastro 3a (7'258 franchi). Se l’opzione è attiva, tutti i campi della cassa pensione di questa persona sono grigi e in sola lettura, e il calcolo avviene **senza cassa pensione** per lei: nessun avere, nessun contributo, nessuna rendita e nessun prelievo di capitale (quindi nessuna imposta sulle prestazioni in capitale). I valori inseriti restano salvati nel file di pianificazione e valgono di nuovo appena disattiva l’opzione. L’AVS resta invariata.
- **Avere di vecchiaia all’inizio della simulazione (MM.AAAA):** l’avere secondo il certificato di previdenza (avere di vecchiaia inclusa la parte sovraobbligatoria). Le proiezioni del certificato (proiezione con interesse) non vanno inserite qui, le calcola l’app stessa.
- **Prestazione da:** sempre la tappa **Prestazione della cassa pensione** della persona (da 58 a 65 anni). Il momento lo modifica sotto [Tappe](#s-milestones).
- **Contributi al mese:** somma dei contributi del lavoratore e del datore di lavoro (e dei riscatti) versati fino alla prestazione. Con la prestazione terminano.
- **Remunerazione all’anno:** tasso d’interesse sull’avere di vecchiaia. La remunerazione viene accreditata a dicembre. Realistici sono dall’1 % al 2 %.
- [Quota del prelievo del capitale](#f-pk-lumpsum) e [aliquota di conversione](#f-pk-conversion) determinano quanto viene versato come capitale e quanto come rendita.
- **Registrare il riscatto come deduzione:** Un riscatto nella cassa pensioni è deducibile dal reddito imponibile nell’anno del riscatto. Il link aggiunge una [deduzione unica](#f-tax-deductions) «Riscatto CP» nella sezione Imposte e vi passa; inserite lì l’anno fiscale e l’importo. Il riscatto stesso (pagamento, avere più alto) si registra come prima tramite i contributi o l’avere.

Contributi e remunerazione si possono inserire in modo [dipendente dal tempo](#f-timed), ad esempio contributi più alti dai 55 anni.

## @f-pk-conversion Aliquota di conversione
Parole chiave: rendita, cassa pensione, percentuale, conversione

L’aliquota di conversione stabilisce quanta **rendita annua** riceve per ogni franco di avere di vecchiaia che non riscuote come capitale.

> Rendita annua = capitale di rendita × aliquota di conversione. Esempio: 500'000 franchi × 5.0 % = 25'000 franchi all’anno (circa 2'083 al mese).

- Il **minimo legale** nella parte obbligatoria è del 6.8 %, ma molte casse pensioni calcolano con aliquote **più basse** (spesso dal 4.5 % al 5.5 %), perché convertono diversamente la parte sovraobbligatoria. Usi il valore del Suo certificato di previdenza, non il minimo legale.
- L’aliquota dipende dall’età alla prestazione. Usi l’aliquota per la Sua età di pensionamento.
- Nell’app vale: rendita al mese = capitale di rendita × aliquota di conversione / 12, **fissa** (nessun rincaro) per tutta la vita.

Con la parte scenari può far variare il prelievo del capitale (vedi [Scenari](#f-sweep)).

## @f-pk-lumpsum Quota del prelievo del capitale
Parole chiave: prelievo del capitale, rendita, imposta sulle prestazioni in capitale, liquidazione unica

La quota dell’avere di vecchiaia versata al pensionamento **come capitale**. 0 % significa ‹interamente come rendita›, 100 % ‹interamente come capitale›.

- Il capitale confluisce al netto nella Sua sostanza liquida. Ne viene dedotta l’**imposta sulle prestazioni in capitale** (vedi [Tabelle fiscali](#f-tax-tables)). I prelievi nello stesso anno civile (cassa pensione, pilastro 3a) vengono sommati per l’aliquota, per le coppie non sposate per persona.
- Il resto diventa rendita con l’[aliquota di conversione](#f-pk-conversion).
- **Rendita:** pianificabile, vitalizia, ma non indicizzata e non ereditabile. **Capitale:** flessibile ed ereditabile, ma porta il rischio d’investimento e la responsabilità di non consumarlo troppo presto.

La quota migliore dipende dall’imposta, dall’aliquota di conversione e dalla Sua propensione al rischio. La faccia variare con una [variazione](#f-sweep) e osservi come cambia il tasso di successo.

## @f-ahv AVS
Parole chiave: rendita, riscossione, rincaro, indice, 1° pilastro, anni di contribuzione, 13a rendita AVS, dicembre, anticipazione, rinvio, riduzione, supplemento

- **Rendita al mese all’inizio della prestazione:** la Sua rendita mensile presumibile con riscossione a 65 anni (età di riferimento) secondo il calcolo anticipato della rendita della cassa di compensazione, come importo nominale (fino al diritto alla rendita l’applicazione calcola senza rincaro), **senza la 13a rendita** (l’applicazione la aggiunge da sé, vedi sotto).
- **Prestazione da:** sempre la tappa **Inizio della rendita AVS** della persona (da 63 a 70 anni). Se si trova prima o dopo i 65 anni, l’applicazione riduce o aumenta da sé la rendita (vedi sotto); inserisca comunque la rendita a 65 anni.

Sotto il campo figura sempre la rendita con cui l’applicazione calcola all’insorgere del diritto, nominale prima del primo adeguamento al rincaro: senza anticipazione né rinvio uguale all’inserimento, altrimenti con l’aliquota, p. es. *−13.6 % → effettivamente 2'177.00 CHF al mese*. Per le coppie sposate figura inoltre da quando entrambi hanno diritto alla rendita e quale rendita resta dopo il limite superiore e la riduzione o il supplemento, p. es. *dal 03.2032 plafonata 1'547.00 CHF*.

**Anticipazione e rinvio:** Un’anticipazione riduce la rendita del 6,8 % all’anno, al mese esatto secondo la tabella dell’AVS (p. es. 15 mesi 8,5 %), al massimo 13,6 % con riscossione a 63 anni. Un rinvio di almeno un anno la aumenta per trimestri: 1 anno 5,2 %, 2 anni 10,8 %, 3 anni 17,1 %, 4 anni 24,0 %, 5 anni 31,5 %; un rinvio inferiore a un anno non dà alcun supplemento. Riduzione e supplemento sono importi fissi in franchi interi; valgono per l’intera durata della riscossione, anche dopo i 65 anni, e per persona secondo la propria età. Esempio: rendita 2 520, riscossione a 63 anni → 2 520 − 343 = 2 177; riscossione a 68 anni → 2 520 + 431 = 2 951, nessuna AVS tra 65 e 68 anni. Per le coppie sposate il limite superiore vale, come per l’AVS, per le rendite **non ridotte** non appena entrambi hanno diritto alla rendita (riscossione, anche anticipata, o 65 anni raggiunti, anche durante un rinvio); solo dopo l’applicazione sottrae la riduzione o aggiunge il supplemento. La riduzione è l’aliquota per la media delle rendite durante l’anticipazione, il supplemento l’aliquota per la media della rendita rinviata, se del caso già limitate. Esempio con due rendite di 2 520 e un limite di 3 780: se la prima persona riscuote a 63 anni e la seconda a 65 anni molto più tardi, ci sono prima 2 177, poi 1 890 − 343 = 1 547 e 1 890. La 13a rendita si basa sulle rendite versate. Una riscossione meno di un anno dopo i 65 anni non è un rinvio (nessun supplemento); una riscossione prima dei 63 o dopo i 70 anni non è possibile. In entrambi i casi la verifica avverte. Fonte: promemoria 3.04 *Pensionamento flessibile* (stato al 1.1.2026). Non sono considerati l’anticipazione e il rinvio parziali né le aliquote particolari per le donne nate dal 1961 al 1969. Le pianificazioni salvate prima di questa modifica contenevano la rendita già adeguata in caso di riscossione prima o dopo i 65 anni; all’apertura l’applicazione la riconverte una volta nella rendita a 65 anni e lo segnala. Il risultato resta invariato salvo l’arrotondamento al franco; per le coppie sposate può cambiare a causa del limite superiore.
- **Adeguamento al rincaro:** fino al diritto alla rendita (riscossione, o 65 anni in caso di rinvio) la rendita inserita resta invariata; il primo adeguamento avviene nel gennaio successivo. L’AVS viene aumentata a gennaio del rincaro accumulato dall’ultimo gennaio. In termini nominali non diminuisce mai: con un rincaro negativo resta invariata, e in seguito aumenta di nuovo solo quando i prezzi superano il livello più alto raggiunto finora, e solo della parte che lo supera. Esempio: rendita di 30 000 all’anno, rincaro −8,4 % → 30 000 l’anno successivo; poi +15,7 % → 30 000 × 0,916 × 1,157 ≈ 31 794 (+6,0 %), non 34 710. La riduzione, il supplemento e il limite per i coniugati seguono la stessa regola. Una rendita senza adeguamento si può calcolare come eccezione sotto «Simulazione» › «Esperto», vedi [Che cosa cresce con il rincaro](#inflation).

Con due persone coniugate esiste un limite superiore per entrambe le rendite insieme, vedi [Seconda persona e regole per coppie sposate](#f-couple).

**13a rendita AVS:** Da dicembre 2026 l’AVS versa ogni anno a dicembre una 13a rendita; l’applicazione ne tiene sempre conto, non c’è un’opzione per disattivarla. Corrisponde a un dodicesimo delle rendite AVS versate da gennaio a dicembre (ogni mese arrotondato al centesimo, la somma al franco intero). Se la rendita inizia nel corso dell’anno, è versata pro rata: una rendita di 2 520 da luglio dà 6 × 210 = 1 260. Per le coppie sposate conta la rendita già plafonata; la 13a rendita stessa non viene plafonata una seconda volta. A dicembre conta come reddito AVS: nei [valori annuali](#r-table) tra le entrate, nel grafico [Entrate e spese all’anno](#r-flows) alla voce AVS. È imponibile come la rendita AVS. Se la simulazione termina prima di dicembre, in quell’anno non c’è la 13a rendita. All’apertura di una pianificazione salvata prima di questa modifica, un avviso segnala una volta che il risultato può quindi differire.

## @f-couple Seconda persona e regole per coppie sposate
Parole chiave: coppia sposata, coniugati, limitazione, 150 percento, partner, doppio reddito

Con «+ Seconda persona» registra un’economia domestica con due persone.

- Ogni persona ha **reddito proprio, cassa pensione propria e AVS propria**.
- Imposte, sostanza, spese e immobili valgono per l’**economia domestica**.
- **L’economia domestica è considerata pensionata** non appena tutte le persone con data di riscossione sono pensionate. Solo allora intervengono le strategie di prelievo.

**Coniugati** (regole per coppie sposate): se entrambi hanno diritto a una rendita AVS, le loro rendite a 65 anni vengono limitate insieme al massimo a **3'780 franchi al mese**; la riduzione in caso di anticipazione e il supplemento in caso di rinvio si aggiungono dopo (vedi [AVS](#f-ahv)) (150 % della rendita massima 2025). Il valore è modificabile, se la rendita massima aumenta. Il limite aumenta ogni gennaio una volta con il rincaro, non appena almeno una rendita con adeguamento al rincaro è in corso. Sotto [Imposte](#s-taxes) si possono registrare tabelle proprie per coniugati. I prelievi di capitale nello stesso anno vengono sommati per i coniugati, per le coppie non sposate per persona.

**Coniugati con una sola persona:** La casella «Coniugati» esiste anche senza seconda persona, ad esempio se il coniuge non ha né reddito né rendita. Valgono allora le tariffe e le deduzioni per coniugati (nella modalità «Domicilio» cantone, comune e Confederazione, con tabelle proprie le tabelle per coniugati); l’imposta è la stessa di una coppia sposata con una seconda persona senza reddito. La limitazione delle rendite AVS vale solo con due persone. Le pianificazioni meno recenti con una persona avevano «Coniugati» spuntato senza effetto; all’apertura vengono riprese come non coniugate e calcolano come prima, un’avvertenza lo segnala. Le nuove pianificazioni iniziano non coniugate.

**Non calcolati:** decesso di una persona, rendite per superstiti, rendita vedovile. La pianificazione calcola come se entrambi vivessero fino alla fine.

## @f-care-home Casa di cura
Parole chiave: casa di cura, casa per anziani, spese della casa di cura, cure, età, entrata, vendere l’abitazione di proprietà, spese di malattia, deduzione

Una scheda propria per persona nella scheda «Previdenza». Con **«Prevedere una casa di cura»** inserite:

- **Entrata:** età della persona (p. es. 85), una tappa o una data fissa.
- **Spese della casa di cura al mese (prezzi attuali):** pensione, cure e assistenza nella misura in cui le pagate voi stessi. Le spese aumentano con il rincaro. Dal mese dell’entrata contano tra le spese; i [valori annuali](#r-table) le mostrano nella colonna «Casa di cura», il grafico [Entrate e spese all’anno](#r-flows) come barra propria e gli [eventi](#r-events) indicano l’entrata.
- **Vendere l’abitazione di proprietà all’entrata:** gli immobili abitati dal proprietario vengono venduti nel mese dell’entrata, come una vendita pianificata con l’imposta sugli utili immobiliari; ipoteca, interessi e spese accessorie terminano.

**Spese d’alloggio:** Quando una persona sola vive in una casa di cura, l’affitto e le altre spese d’alloggio terminano dal mese dell’entrata. Gli affitti «dopo la vendita» non iniziano, nemmeno dopo una vendita o una [vendita forzata](#f-re-emergency); non ha luogo nemmeno un acquisto «dopo la vendita». Senza «vendere» l’immobile resta con le sue spese e le spese della casa di cura si aggiungono; una vendita forzata in caso di sostanza scarsa continua a valere.

**Coppie:** Le spese d’alloggio dell’economia domestica continuano finché una persona vive a casa; le spese della casa di cura della persona in casa di cura si aggiungono. Solo quando **entrambe** vivono in una casa di cura terminano l’affitto e le spese d’alloggio, e l’immobile viene venduto se «vendere» è scelto per una delle due persone. La pianificazione non rappresenta un decesso.

**[Non vendere mai gli immobili](#f-re-never-sell)** ha la precedenza: nessuna vendita all’entrata, le spese dell’immobile continuano; il controllo lo segnala.

**Imposte (semplificato):** Le spese della casa di cura contano come spese di malattia e spese dovute a disabilità. Sono deducibili per l’imposta federale e l’imposta del cantone nella misura in cui superano il 5 % del reddito netto; il reddito netto è il reddito dopo le altre [deduzioni](#f-tax-deductions). Esempio: reddito netto 60 000, spese della casa di cura 108 000 all’anno → deduzione 108 000 − 3 000 = 105 000; il reddito imponibile non scende mai sotto 0. L’app applica questa regola per tutti i cantoni; alcuni cantoni calcolano diversamente, per esempio solo con la quota per le cure. L’app non calcola le prestazioni complementari.

**Controllo:** Un’entrata mancante, un’età al di fuori da 0 a 150 anni o spese negative sono errori. Dopo la vendita dell’ultimo immobile (pianificata o a causa della casa di cura) il controllo segnala quando non segue né un affitto né una casa di cura, perché allora la pianificazione non avrebbe più spese d’alloggio.

## @s-flows Entrate e spese
Parole chiave: reddito, costo della vita, spese, importi unici, eredità, inflazione

Importi mensili, il reddito da attività lucrativa all’anno. **Tutti gli importi vengono registrati ai prezzi attuali** e adeguati dall’app all’inflazione, anche il reddito da attività lucrativa e gli importi unici (vedi [Che cosa cresce con il rincaro](#inflation)). Inserisce quindi i Suoi costi attuali, anche per anni futuri.

- **Reddito da attività lucrativa netto all’anno:** come nella dichiarazione d’imposta, il salario netto del certificato di salario (cifra 11), incluse 13a mensilità e bonus, cioè ciò che resta dopo le deduzioni per AVS e cassa pensione (le imposte sono calcolate separatamente); per gli indipendenti il reddito da attività lucrativa indipendente. Accanto al campo c’è l’importo al mese, ad esempio 120 000 all’anno ‹= 10 000 / mese›; l’app calcola e salva al mese (un dodicesimo), anche in ogni riga di un reddito dipendente dal tempo e per entrambe le persone. Le pianificazioni esistenti mostrano l’importo mensile precedente per 12. Anche i parametri di un calcolo e l’avviso sotto indicano l’importo all’anno. Quando termina lo stabilisce l’interruttore sotto; senza di esso termina quando lo imposta a 0 in modo [dipendente dal tempo](#f-timed).
- **Rincaro del reddito da attività lucrativa** (per persona): gli importi, anche ogni riga di un reddito dipendente dal tempo, valgono ai **prezzi attuali** (prezzi all’inizio della simulazione) e crescono con il rincaro del rispettivo percorso, esattamente come il costo della vita. Esempio: 8'000 al mese con un rincaro del 2 % sono dopo un anno 8'160, dopo cinque anni circa 8'830. L’eccezione «nominale, senza rincaro» si trova sotto «Simulazione» › «Esperto» ([Che cosa cresce con il rincaro](#inflation)); le pianificazioni meno recenti vi mantengono la loro scelta precedente. Imposte, valori annuali e sostenibilità usano l’importo adeguato; l’avviso dopo la prestazione della cassa pensione indica allora l’importo «ai prezzi dell’inizio della simulazione».
- **Il reddito da attività lucrativa termina con la prestazione della cassa pensione** (per persona): con la spunta il reddito da attività lucrativa è 0 dal mese della tappa [Prestazione della cassa pensione](#s-milestones) di questa persona, anche se un reddito dipendente dal tempo ha ancora un importo in seguito. Se sposta la tappa, la fine si sposta con essa. Esempio: prestazione della cassa pensione a luglio 2030, reddito 120 000 all’anno; nel 2030 contano 60 000, dal 2031 più nulla. Tutto ciò che dipende dal reddito da attività lucrativa si adegua: il grande pilastro 3a (20 % del reddito da attività lucrativa), le spese professionali e le deduzioni per persone attive nella proposta del cantone nonché la deduzione per coppie con doppio reddito. Senza spunta il reddito vale come inserito, ad esempio in caso di pensionamento parziale o di lavoro dopo il pensionamento; se allora dopo la prestazione della cassa pensione è ancora maggiore di 0, la verifica mostra un avviso con l’importo. Le nuove pianificazioni e una seconda persona appena aggiunta hanno la spunta; le pianificazioni meno recenti senza indicazione calcolano come finora senza.
- **Costo della vita al mese:** l’**importo inserito** comprende tutto ciò che spende, **incluso l’affitto o le spese d’alloggio** (interesse ipotecario, spese accessorie), come figura sull’estratto conto, ma **senza l’ammortamento**: costituisce sostanza (riduzione del debito o versamento nel pilastro 3a) e si aggiunge come spesa nella simulazione; il suo importo non modifica il costo della vita senza alloggio. L’app deduce la **quota d’alloggio**, cioè le spese d’alloggio del mese d’inizio, tratte dalla sezione [Immobili](#s-realestate), come media mensile (l’interesse viene pagato trimestralmente), adeguata al rincaro, e cioè in ugual misura da ogni riga. Il **calcolo** usa il costo della vita senza la quota d’alloggio più le spese d’alloggio **simulate**: se l’interesse cambia a un rinnovo o se dopo una vendita passa in affitto, le spese cambiano di conseguenza. Esempio: 8'000 incluse 2'000 di spese d’alloggio; dopo un aumento dell’interesse di 300 le spese sono 8'300, con un affitto di 2'500 sono 8'500. Se una riga è inferiore alla quota d’alloggio, la verifica segnala un errore. Il versamento nel [pilastro 3a](#s-pillar3a) non è incluso. Le pianificazioni meno recenti (costo della vita senza spese d’alloggio) l’app le converte all’apertura, un’avvertenza indica l’importo aggiunto. Per le pianificazioni la cui quota d’alloggio conteneva ancora l’ammortamento, all’apertura l’app riduce il costo della vita dell’ammortamento del mese d’inizio, affinché il risultato resti lo stesso; un’avvertenza indica l’importo. Le imposte vengono calcolate separatamente. Un valore realistico è il singolo fattore più importante per il risultato. A tale scopo osservi due o tre anni di estratti conto.
- **Importi unici:** data, importo e descrizione. Positivo è un’entrata (eredità, donazione), negativo una spesa (auto, viaggio, ristrutturazione).

Con l’età le spese spesso diminuiscono, ad esempio meno viaggi dagli 80 anni, mentre aumentano i costi per la salute. Usi a tale scopo i [valori dipendenti dal tempo](#f-timed).

## @f-timed Valori dipendenti dal tempo
Parole chiave: costante, dipendente dal tempo, gradini, retta, tabella, fattore, punto d’appoggio, riga, da, andamento, minigrafico, percorso

Molti importi e aliquote dipendono dal tempo. In alto a destra del campo sceglie la forma dell’andamento:

- **Costante:** un valore per l’intera simulazione.
- **Gradini:** una sequenza di righe ‹da *momento* vale *valore*›. Il momento è una tappa (‹da prestazione della cassa pensione›) o una data fissa. Ogni riga vale finché inizia la successiva. Esempio costo della vita: ‹da inizio 9'500›, ‹da prestazione della cassa pensione 8'500›, ‹da 80 7'000›.
- **Retta:** ‹da› un valore a un momento in modo uniforme, mese per mese, ‹a› un secondo valore a un momento successivo. Prima vale il primo valore, dopo il secondo. Esempio quota azionaria: 90 % a 55, 50 % a 65 dà 70 % a 60.
- **Tabella:** punti d’appoggio ‹a *momento* vale *valore*›. «+ Punto» ne aggiunge un altro, ✕ lo elimina. «Tra i punti» sceglie se il valore tra due punti d’appoggio procede in modo **graduale** come una retta oppure **a gradini**. Prima del primo punto d’appoggio vale il suo valore, dopo l’ultimo l’ultimo valore.
- **Fattore:** un **valore di base** moltiplicato per un andamento di fattori in percento. Il fattore stesso lo inserisce di nuovo come costante, gradini, retta o tabella. Esempio costo della vita: valore di base 6'000, fattore come tabella a gradini: all’inizio della simulazione 100 %, a 80 80 %.

Per retta e tabella il momento di un punto d’appoggio è un’**età** di una persona (con decimali, 64.5 = 64 anni e 6 mesi), una **tappa** o una **data fissa**. Se cambia il tipo, il momento resta lo stesso; una tappa diventa per esempio l’età in cui cade.

Se cambia la forma, l’app riprende ciò che si può riprendere: gradini, retta e tabella mantengono i loro momenti e valori, «Costante» prende il valore all’inizio della simulazione, «Fattore» lo prende come valore di base e l’andamento precedente come fattori.

Il piccolo grafico accanto al campo mostra l’andamento (per un fattore il risultato valore di base per fattore). Con la rotella del mouse ingrandisce, i pulsanti «Tutto», ‹10 a.›, ‹5 a.› e ‹2 a.› scelgono l’intervallo.

Le righe e i punti d’appoggio inseriti più in basso nell’elenco, ma che iniziano prima di un altro, l’app li ordina secondo il momento. Può quindi spostare tranquillamente le tappe, l’ordine dei valori resta coerente. Tutte le forme si trovano così anche nel [file di pianificazione](#ref-file).

## @s-wealth Sostanza e investimento
Parole chiave: azioni, obbligazioni, cash, liquidità, quota azionaria, riserva, portafoglio, strategia d’investimento, indice

I **saldi all’inizio della simulazione**, la **ripartizione** del portafoglio e la **strategia d’investimento**.

- **Azioni, obbligazioni, cash:** i Suoi saldi all’inizio della simulazione. Immobili e averi di previdenza figurano separatamente, qui solo la sostanza liquida. L’app riprende i saldi all’inizio senza modifiche. Accanto ad azioni e obbligazioni figura la loro quota del portafoglio (senza cash, percentuali intere), ad esempio 80'000 di azioni e 20'000 di obbligazioni danno ‹80 % del portafoglio› e ‹20 % del portafoglio›.
- **Il cash non fa parte del portafoglio.** La [quota azionaria](#f-stockshare) si riferisce solo ad azioni e obbligazioni: 80'000 di azioni, 20'000 di obbligazioni e 50'000 di cash danno una quota azionaria dell’80 %.
- **Fino al pensionamento** (prestazione della cassa pensione della prima persona) non vale alcuna strategia di prelievo. L’app mantiene tanto cash quanto figura nel campo **Cash**, adeguato al **rincaro**: un’eccedenza va nel portafoglio secondo la quota azionaria, un ammanco viene colmato dal portafoglio.
- **Dal pensionamento** il cash è determinato dalla [strategia di prelievo](#s-withdrawal): «Standard» con la [riserva di cash](#f-reserve) della sua scheda, tutte le altre con 3 mesi o la strategia a 3 comparti con i suoi comparti.

- **Strategia d’investimento:** un indice per le azioni (Mondo, Mondo personalizzato, Svizzera, S&P 500) e uno per le obbligazioni (Confederazione 10 anni o a breve termine, USA). Con «Mondo personalizzato» stabilisce le ponderazioni di Svizzera, USA, UE e Asia-Pacifico; insieme devono dare 100 %. Sotto la selezione figurano il periodo comune e i rendimenti medi; se un indice viene ridotto al periodo comune, appare un’avvertenza. Dettagli: [Strategia d’investimento](#f-ret-series).

Nella simulazione la sostanza viene aggiornata ogni mese con il rendimento di azioni e obbligazioni: dagli indici della strategia d’investimento, secondo il metodo sotto [Simulazione](#s-returns) (rendimenti fissi, andamento storico o Monte Carlo).

## @f-stockshare Quota azionaria
Parole chiave: rischio, azioni, ribilanciamento, portafoglio

La quota delle **azioni nel portafoglio** (azioni + obbligazioni, senza cash). Il resto sono obbligazioni.

- L’app ripristina questa quota ogni mese (**ribilanciamento**), salvo che la strategia preveda altrimenti.
- Una quota azionaria alta porta in media più rendimento, ma anche minimi più bassi. Proprio all’inizio del pensionamento il rischio è alto (**rischio di sequenza**): un crollo di borsa poco dopo il pensionamento nuoce molto di più dello stesso crollo 20 anni dopo.
- Il valore può essere [dipendente dal tempo](#f-timed), ad esempio 70 % fino al pensionamento e 50 % dopo, oppure una [curva caratteristica](#f-curve) in funzione dello stato della simulazione, ad esempio del tasso di prelievo.

Quanto la quota azionaria cambi il risultato lo mostra una [variazione](#f-sweep) della quota azionaria, o l’[ottimizzazione](#s-optimize), che cerca contemporaneamente anche la quota.

## @f-curve Curva caratteristica
Stichworte: curva caratteristica, curva, stato, variabile di stato, tasso di prelievo, grado di copertura, drawdown, massimo, quota azionaria dinamica, curve

Invece di un valore o di un andamento nel tempo, la [quota azionaria](#f-stockshare) può essere una **curva caratteristica**: dipende allora dallo **stato** della simulazione, ad esempio dal tasso di prelievo. La curva è composta da punti d’appoggio, ognuno un *valore della variabile di stato* con la *quota azionaria* che vi si applica; tra di essi è una retta, sotto il primo punto d’appoggio vale il suo valore, sopra l’ultimo l’ultimo valore. Le quote azionarie sono comprese tra 0 e 1.

**Variabili di stato:**

- **Età** (`age`): età della prima persona in anni.
- **Sostanza liquida reale** (`liquidReal`): cash, obbligazioni e azioni ai prezzi dell’inizio della simulazione (divisi per il rincaro dall’inizio).
- **Tasso di prelievo** (`withdrawalRate`): prelievo degli ultimi 12 mesi diviso per la sostanza liquida. Il prelievo di un mese sono le uscite (costo della vita, costi dell’abitazione, imposte) meno le entrate (reddito da attività lucrativa, rendite, versamenti di risparmio); gli importi unici, l’acquisto e la vendita di immobili e il pilastro 3a non contano. Nel primo anno contano i mesi trascorsi, estrapolati a 12; all’inizio della simulazione le uscite non coperte del mese iniziale per 12.
- **Grado di copertura** (`fundedRatio`): sostanza liquida divisa per la lacuna restante fino alla fine della simulazione, entrambe ai prezzi dell’inizio della simulazione. La lacuna è il costo della vita meno il reddito da attività lucrativa e le rendite (AVS e cassa pensioni, per quanto noti nel mese della valutazione), senza attualizzazione, senza costi dell’abitazione e imposte. Da 1 la sostanza copre la lacuna senza rendimento. Se reddito e rendite coprono tutto, vale l’ultimo valore della curva.
- **Rendimento azionario 12 mesi** (`stockReturn12m`): rendimento dell’indice azionario degli ultimi 12 mesi, 0.1 = 10 %.
- **Distanza dal massimo** (`drawdown`): di quanto l’indice azionario si trova sotto il suo massimo finora (compreso il periodo preliminare del percorso dei rendimenti), 0 al massimo, 0.3 = 30 % sotto.

**Valutazione:** al **ribilanciamento**. Con `"eval": "yearly"` (predefinito) al primo ribilanciamento di ogni anno civile; il valore vale poi fino all’anno seguente. Con `"eval": "monthly"` ogni mese. Come un andamento, con la [strategia a 3 comparti](#f-wd-three_buckets) la quota azionaria vale solo fino al pensionamento.

**Esempio:** punti d’appoggio 2 % → 80 % e 6 % → 40 % in funzione del tasso di prelievo. Con un tasso di prelievo del 4 % vale una quota azionaria del 60 %, con l’1 % dell’80 %, con l’8 % del 40 %.

**Inserimento:** nell’editor la curva è solo visualizzata (variabile di stato, valutazione, punti d’appoggio). La inserisca e la modifichi nel [file di progetto](#ref-file) o tramite l’interfaccia IA, ad esempio `"stockShare": {"mode": "curve", "input": "withdrawalRate", "points": [[0.02, 0.8], [0.06, 0.4]], "eval": "yearly"}`. Un’altra forma in alto nel campo sostituisce la curva con il valore del suo primo punto d’appoggio.

**Limite del modello parametrico:** il [modello parametrico](#f-ret-parametric) estrae il rendimento di ogni anno indipendentemente dagli anni precedenti; lì un crollo non è seguito più spesso da una ripresa. Le regole che puntano su una ripresa dopo i crolli (ad esempio più azioni con una grande distanza dal massimo o dopo 12 mesi negativi) non possono esservi verificate in modo sensato: lì non possono mostrare alcun vantaggio. Verifichi tali regole con percorsi storici ([storico](#f-ret-historical), [tutti i punti di partenza](#f-ret-rolling) o [bootstrap](#f-ret-bootstrap) a blocchi) e tenga presente il rischio di adattare la regola al passato.

## @f-reserve Riserva di cash
Parole chiave: cash, riserva, mesi, liquidità, riserva di liquidità

Quanti **mesi di spese non coperte** (spese meno redditi e rendite) vengono tenuti come cash **dal pensionamento**, prima che il resto venga investito secondo la [quota azionaria](#f-stockshare). Prima del pensionamento vale il campo «Cash» sotto [Sostanza e investimento](#s-wealth).

- Una riserva alta protegge dal dover vendere azioni a corsi bassi, ma costa rendimento.
- Impostabile solo con la strategia «Standard» (scheda «Standard» sotto «Strategia di prelievo») ed efficace solo dal pensionamento. **Tutte le altre strategie** (Guardrails, tasso fisso, VPW, Endowment) calcolano dal pensionamento con una **riserva fissa di 3 mesi**; con la [strategia a 3 comparti](#f-wd-three_buckets) i comparti sostituiscono la riserva.
- Una pianificazione meno recente con un’altra strategia e una riserva diversa da 3 mesi calcola perciò con 3 mesi e può dare risultati leggermente diversi. L’app lo segnala una volta, alla prima apertura di una tale pianificazione; in seguito l’avvertenza non appare più. Il valore salvato resta nel file e vale di nuovo se passa a «Standard».
- Sensato in modo dipendente dal tempo: ad esempio 6 mesi fino al pensionamento e 12 mesi dopo.

## @s-taxes Imposte
Parole chiave: imposta sul reddito, imposta sulla sostanza, imposta sulle prestazioni in capitale, moltiplicatore d’imposta, tabella

Le nuove pianificazioni calcolano con il **domicilio**: imposta sul reddito, sulla sostanza e sulle prestazioni in capitale secondo cantone, comune e imposta di culto secondo le leggi tributarie della Confederazione e dei cantoni, l’imposta sugli utili immobiliari secondo il cantone dell’immobile ([Imposta secondo il domicilio](#f-tax-location)). Servono solo il comune e la confessione; tutto il resto vale con valori predefiniti visibili come **ipotesi**. Le **tabelle proprie** (tabelle delle aliquote e moltiplicatore d’imposta, come descritto sotto) restano solo per le pianificazioni esistenti che già le usano: lì si trovano sotto **«Esperto»**, e la scheda offre il pulsante **«Passare al domicilio»**. Le pianificazioni esistenti si aprono invariate e calcolano allo stesso modo.

Semplici **tabelle delle aliquote**, che può ricavare dal calcolatore fiscale del Suo cantone. Tra gli scaglioni si **interpola linearmente**, l’aliquota viene moltiplicata per il [moltiplicatore d’imposta](#f-tax-multiplier).

- [Imposta secondo il domicilio](#f-tax-location)
- [Deduzioni dal reddito](#f-tax-deductions)
- [Deduzioni dalla fattura d’imposta](#f-tax-bill)
- [Reddito imponibile della sostanza](#f-tax-yield)
- [Moltiplicatore d’imposta e tabelle](#f-tax-tables)
- [Tariffe con il rincaro](#f-tax-indexed)
- [Quanto è precisa l’imposta?](#tax-accuracy)
- L’**imposta sul reddito** viene calcolata a dicembre sul reddito annuo (incluse le rendite e il [reddito imponibile della sostanza](#f-tax-yield)), l’**imposta sulla sostanza** sul portafoglio più gli immobili netti (con il [valore fiscale](#f-re-tax-value) se registrato, altrimenti con il valore predefinito o, nelle pianificazioni più vecchie, con il valore venale). Entrambe vengono pagate a maggio dell’anno successivo.
- Un’imposta ancora aperta alla fine della pianificazione viene dedotta nell’ultimo mese.
- L’[imposta sugli utili immobiliari](#f-tax-gain) è dovuta alla vendita di un immobile, vedi anche [Immobili](#s-realestate).

I valori d’esempio sono solo segnaposto. Per il Suo risultato li sostituisca con i valori del Suo luogo di domicilio.

## @f-tax-location Imposta secondo il domicilio
Parole chiave: domicilio, cantone, comune, imposta di culto, confessione, AFC, dati fiscali, deduzioni standard, spese professionali, imposta federale, reddito imponibile, prelievo del capitale, imposta sulle prestazioni in capitale, imposta sugli utili immobiliari, cantone dell’immobile, trasloco, cambio di domicilio, data determinante

Nel modo **Domicilio** indica solo il **comune** e l’**imposta di culto** (nessuna, riformata, cattolica romana, cattolica cristiana). La ricerca comprende tutti i comuni dell’anno fiscale: digitare il nome e scegliere dall’elenco (nome con cantone); il **cantone** risulta dal comune ed è indicato accanto. Tariffe e deduzioni provengono dal file dei dati fiscali fornito con l’app, secondo le leggi tributarie della Confederazione e dei cantoni; i moltiplicatori d’imposta provengono dai dati di base del calcolatore d’imposta dell’AFC; la scheda indica l’anno fiscale, la fonte e i moltiplicatori d’imposta del comune. Non c’è accesso alla rete.

**Imposta stimata:** In alto nella scheda figurano due cifre ai prezzi attuali, arrotondate a 100 franchi: l’imposta sul reddito e sulla sostanza nel **primo anno civile completo** e nel **primo anno civile completo dopo il pensionamento** (dopo l’ultimo prelievo della cassa pensioni dell’economia domestica), senza prelievi di capitale né utili immobiliari. Le confronti con la sua ultima fattura d’imposta. Se il pensionamento è prima dell’inizio, figura una sola cifra. Le cifre provengono dall’anteprima; finché mancano dati (per esempio il comune), una nota sostituisce le cifre. Le deduzioni diventano più precise con **«Riprendere dalla fattura d'imposta»** sotto le cifre, vedi [Deduzioni dalla fattura d’imposta](#f-tax-bill).

**Ipotesi:** Sotto il domicilio figura una riga con i valori predefiniti in vigore, a parole: deduzioni standard del cantone fino al e dal pensionamento ([Deduzioni dal reddito](#f-tax-deductions)), [tariffe con o senza rincaro](#f-tax-indexed), deduzione del pilastro 3a (sì o no), [valore fiscale](#f-re-tax-value) degli immobili (valore predefinito 50 % del valore venale) e [reddito imponibile](#f-tax-yield) della sostanza. Un clic sulla riga apre le impostazioni; la riga mostra subito le modifiche. Le nuove pianificazioni hanno questi valori predefiniti; la misurazione su diverse economie domestiche fittizie mostra che senza le deduzioni l’imposta sarebbe nettamente troppo alta negli anni di attività lucrativa.

**Esperto:** Sotto figura l’area chiusa **«Esperto»** (deduzioni in dettaglio, deduzioni dalla sostanza, tariffe) con le righe delle [Deduzioni dal reddito](#f-tax-deductions) (periodo, proposta o valore proprio, deduzione aggiuntiva), le [Deduzioni dalla sostanza](#f-tax-wealth-deductions), la nota sulle [tariffe con il rincaro](#f-tax-indexed) e, nelle pianificazioni esistenti, le tabelle proprie. La maggior parte delle pianificazioni non ne ha bisogno. Se una pianificazione vi ha valori propri (una riga con un valore proprio o una deduzione aggiuntiva, valori ripresi dalla notifica di tassazione, deduzioni dalla sostanza o tabelle proprie), l’area si apre da sé. La casella del pilastro 3a si trova tra le ipotesi, le [deduzioni uniche](#f-tax-deductions) in una scheda propria direttamente nella sezione. Aprire e chiudere non cambia né la pianificazione né il calcolo.

**Reddito imponibile** = reddito dell’anno (reddito da attività lucrativa netto, rendita della cassa pensioni, AVS) meno le **deduzioni standard** (proposta del cantone), mai negativo. Invece della proposta può valere per periodo un valore proprio, più il pilastro 3a, vedi [Deduzioni dal reddito](#f-tax-deductions). Cantone e Confederazione hanno ciascuno le proprie deduzioni:

- **Deduzioni sociali** per persone sole o coniugate, se il cantone le prevede.
- **Premi assicurativi e interessi su capitali a risparmio:** negli anni con reddito da attività lucrativa l’importo «con contributi al 2° e 3° pilastro», altrimenti l’importo «senza».
- **Deduzione forfettaria per spese professionali** sul reddito da attività lucrativa di ogni persona attiva: aliquota, importo minimo e massimo del cantone, al massimo il reddito da attività lucrativa, senza reddito da attività lucrativa nessuna deduzione. Esempio Zurigo: 3 %, almeno 2 000, al massimo 4 000 franchi, quindi 3 000 con 100 000 di reddito da attività lucrativa. Per Basilea Campagna, Basilea Città e Ticino è registrato un importo forfettario fisso secondo la legge (500, 4 200 e 3 500 franchi); le spese effettive possono essere più elevate (⚠).
- **Deduzione per coppie con doppio reddito** per i coniugi se entrambi hanno un reddito da attività lucrativa: secondo la regola del cantone e della Confederazione, calcolata sul reddito da attività lucrativa più basso dopo la deduzione forfettaria per spese professionali e al massimo pari a questo; Berna applica il 2 % del reddito complessivo da attività lucrativa. Esempio Confederazione: 50 %, almeno 8 600, al massimo 14 100 franchi; Zurigo un importo fisso di 6 200 franchi. La Turgovia non prevede una deduzione per coppie con doppio reddito.

**Imposta:**

- Imposta sul reddito = tariffa del cantone sul reddito imponibile × (moltiplicatore d’imposta cantone + Chiesa) + tariffa del comune × moltiplicatore d’imposta comune + imposta federale diretta sul reddito imponibile della Confederazione. La tariffa del comune è quella del cantone, salvo a Svitto e nel Vallese. Nel Vallese la tariffa del cantone e dei comuni è indicizzata: si applica a un reddito ridotto secondo l’indicizzazione (cantone 173 %, comune da 110 a 176 % secondo il comune). Nel Vallese i coniugati beneficiano di una riduzione del 35 % sull’imposta del cantone e del comune (dopo il moltiplicatore, senza imposta di culto), di almeno 680 e al massimo 4 900 franchi ciascuna (2026, indicizzata; LF VS art. 32 e 178). La riduzione per le famiglie monoparentali non è inclusa, perché la pianificazione non prevede figli.
- Imposta sulla sostanza = tariffa del cantone (deduzioni sociali sulla sostanza comprese) × (moltiplicatore d’imposta cantone + comune + Chiesa), senza imposta federale.
- **Imposta sulle prestazioni in capitale** (cassa pensioni, pilastro 3a) = regola del cantone per il capitale × (moltiplicatore d’imposta cantone + Chiesa) + regola del comune × moltiplicatore d’imposta comune + imposta federale (un quinto della tariffa dell’imposta federale diretta, senza deduzioni). Valgono i moltiplicatori d’imposta dell’imposta sul reddito; la regola vale per l’intero prelievo. I prelievi dello stesso anno civile sono sommati: per i coniugati per l’economia domestica con la tariffa per coniugati, per le coppie non sposate per persona con la tariffa per persone sole. Sotto il comune figura la regola del cantone, per esempio Zurigo «aliquota della tariffa del reddito per un ventesimo del prelievo, almeno 2 %». Le regole provengono dalle leggi tributarie cantonali; nel Vallese sono solo approssimate (⚠).
- **Coniugati** («Coniugati» spuntato, anche per una sola persona): tariffe e deduzioni per coniugati. Le coppie non sposate sono calcolate con quelle per persone sole sul reddito comune.
- Con le [tariffe con il rincaro](#f-tax-indexed) gli importi di tariffe e deduzioni crescono con il rincaro simulato.
- **Imposta sugli utili immobiliari:** per ogni immobile vale la regola del cantone in cui si trova; il cantone si sceglie presso l’immobile sotto [Immobili](#s-realestate), senza indicazione vale il cantone di domicilio. Dettagli sotto [Imposta sugli utili immobiliari](#f-tax-gain). Con la casella **«Regole proprie invece delle regole del cantone»** valgono invece le regole proprie con il [moltiplicatore d’imposta](#f-tax-multiplier) per tutti gli immobili.

**Cambio di domicilio:** Con «Aggiungi domicilio» aggiunge un altro domicilio, da un anno o da un evento (per esempio il pensionamento), ciascuno con cantone, comune e imposta di culto; la prima riga vale dall’inizio della simulazione. Valgono le regole della legge sull’armonizzazione delle imposte dirette (LAID art. 4b): l’**imposta sul reddito e sulla sostanza** di un anno si calcola con il domicilio al **31 dicembre**, per tutto l’anno – se trasloca a giugno, paga tutto l’anno nel nuovo luogo. L’**imposta sulle prestazioni in capitale** si calcola con il domicilio nel **mese del prelievo**; un trasloco in un cantone più conveniente ha quindi effetto solo se avviene prima del prelievo. L’**imposta sugli utili immobiliari** di un immobile senza cantone proprio si calcola con il cantone di domicilio nel mese della vendita. La [proposta del cantone](#f-tax-deductions) segue il domicilio dell’anno fiscale, un valore proprio resta. Non sono calcolati il trasferimento all’estero, le spese di trasloco e l’assoggettamento solo per una parte dell’anno. I parametri di un calcolo indicano ogni domicilio con il suo inizio, i valori annuali mostrano il domicilio di ogni anno con le colonne delle imposte.

**Nascosti:** le tabelle proprie per reddito, sostanza e prelievo di capitale (anche quelle per coniugati), le regole proprie dell’imposta sugli utili immobiliari e il moltiplicatore d’imposta (senza la casella) restano nel file. **«Passare al domicilio»** in una pianificazione con tabelle proprie passa al modo «Domicilio» (le deduzioni senza valore proprio diventano la proposta del cantone); dopo la scelta del comune una nota indica l’imposta stimata del primo anno prima (tabelle proprie) e dopo (domicilio). Si può annullare come ogni modifica.

**Verifica:** se in una riga manca il comune, se il comune non si trova nel cantone o se una riga inizia prima della precedente, è un errore nella sezione Imposte, e «Avviare la simulazione» è bloccato. La panoramica delle ipotesi e la scheda «Parametri» indicano il domicilio, per esempio «Zürich (ZH), imposta di culto nessuna».

## @f-tax-deductions Deduzioni dal reddito
Parole chiave: deduzioni, proposta del cantone, valore proprio, tassazione, reddito imponibile, pilastro 3a, valore locativo, interessi ipotecari, interessi passivi, spese professionali, deduzione per coppie con doppio reddito, dettagli

La scheda **Deduzioni dal reddito** nella sezione Imposte (sotto «Esperto») stabilisce che cosa viene dedotto dal reddito dell’anno (reddito da lavoro netto, rendita della cassa pensioni, AVS). Il **reddito imponibile** = reddito − deduzioni, mai negativo.

**Righe:** Le deduzioni dipendono dal tempo come il reddito da attività lucrativa; una riga vale dalla sua data fino alla successiva. Le nuove pianificazioni hanno due righe con la proposta del cantone, una dall’inizio della simulazione (fino al pensionamento), l’altra dal prelievo della cassa pensioni (dal pensionamento); altre righe sono possibili, per esempio «dal 2029» per l’abolizione del valore locativo. Se la riga cambia nel corso dell’anno, ogni riga conta per i suoi mesi.

Ogni riga è (le spiegazioni si trovano anche nell’ⓘ accanto al titolo della scheda):

- **Proposta del cantone** (solo nella modalità [domicilio](#f-tax-location), dove è il valore predefinito): le deduzioni standard secondo le leggi tributarie, per cantone e Confederazione – deduzioni sociali, premi assicurativi e interessi di risparmio, spese professionali, per i coniugi la deduzione per coppie con doppio reddito. L’intestazione della riga mostra la somma per cantone e Confederazione nel primo anno della riga; **Dettagli** apre una tabella con le componenti (righe) per cantone e Confederazione (colonne) e il totale, le componenti pari a 0 sono omesse. Le spese professionali approssimate (Basilea Campagna, Basilea Città, Ticino) sono segnate con ⚠. La proposta viene ricalcolata se cambiate cantone, comune o stato civile. **Deduzione supplementare:** «+ Deduzione supplementare» mostra il campo (con un importo salvato è sempre visibile). Un importo proprio ai prezzi dell’inizio della simulazione (cresce con «Tariffe con il rincaro») si aggiunge alla proposta, per cantone e Confederazione. Esempio: proposta 20 000, in aggiunta 3 000, deduzione 23 000.
- **valore proprio:** un importo all’anno ai prezzi dell’inizio della simulazione (cresce con le [tariffe con il rincaro](#f-tax-indexed)), facoltativamente più una percentuale del reddito. Vale per cantone e Confederazione e rimane se cambiate cantone, comune o stato civile. Nella modalità «Tabelle proprie» c’è solo il valore proprio; senza riga vale 0. Esempio: valore proprio 40 000, reddito 120 000, imposta come su 80 000.

**Determinare il valore proprio:** L’aiuto all’inserimento lo calcola dal **reddito** e dal **reddito imponibile secondo la tassazione**, per esempio 120 000 e 80 000 danno 40 000; «Usare come valore proprio» lo inserisce nella riga. Da notare:

- Il reddito è **netto**, dopo i contributi AVS e CP, come il salario netto del certificato di salario, rendite comprese.
- **Senza pilastro 3a:** i versamenti provengono dalla pianificazione (vedi sotto). La deduzione secondo la tassazione li comprende già: se «Dedurre i versamenti nel pilastro 3a» è attivato, inserisca il pilastro 3a secondo la tassazione in **«di cui pilastro 3a»**; l’aiuto all’inserimento lo sottrae, altrimenti conta due volte. Esempio: 120 000, imponibile 80 000, di cui pilastro 3a 7 000 dà 33 000 (senza la casella 40 000). Se il campo è vuoto, l’aiuto all’inserimento lo segnala.
- La **proprietà dell’abitazione** (valore locativo, interessi ipotecari, manutenzione) rientra nel valore proprio. Dal 2029 non ci sono più valore locativo né deduzione degli interessi per l’abitazione occupata dal proprietario; create per questo una riga «dal 2029». **Gli interessi ipotecari non vengono dedotti automaticamente.**

**Deduzioni uniche:** Si trovano in una scheda propria, **«Deduzioni uniche»**, direttamente nella sezione Imposte, non sotto «Esperto», per averle a portata di mano nelle decisioni. «+ Deduzione unica» aggiunge una deduzione per un singolo anno fiscale, con anno fiscale, descrizione e importo (ai prezzi attuali, cresce con il rincaro fino all’anno fiscale), per esempio un riscatto nella cassa pensioni, un risanamento o spese di malattia. L’importo conta in questo anno fiscale in aggiunta alla riga applicabile, con la proposta del cantone come con il valore proprio e in entrambe le modalità. Esempio: reddito 120 000, valore proprio 30 000, unica 50 000 nell’anno 2027 dà nel 2027 un reddito imponibile di 40 000, negli altri anni 90 000. La stima del fabbisogno netto della [strategia a 3 comparti](#f-wd-three_buckets) usa solo le deduzioni correnti. **Riscatto nella cassa pensioni:** Nella scheda della cassa pensioni (sezione Previdenza) il link **«Registrare il riscatto come deduzione»** aggiunge una deduzione unica «Riscatto CP» (con il nome se ci sono due persone) e passa a questa scheda. L’anno fiscale (predefinito: l’anno dopo l’inizio) è pronto per l’inserimento; inserite l’anno e l’importo del riscatto.

**Pilastro 3a:** Con la casella **«Dedurre i versamenti nel pilastro 3a»** i versamenti di un anno della sezione [Pilastro 3a](#s-pillar3a) (con l’ammortamento indiretto) vengono dedotti dal reddito imponibile, per cantone e Confederazione, al massimo l’importo massimo per persona delle impostazioni (gli importi massimi si trovano nell’ⓘ accanto alla casella). Esempio: reddito 100 000, nessun’altra deduzione, versamento 7 000, imposta come su 93 000. Per una persona «Senza cassa pensione (grande pilastro 3a)» vale il 20 % del suo reddito da attività lucrativa dell’anno, al massimo l’importo massimo del grande pilastro 3a; come gli altri importi cresce con «Tariffe con il rincaro». Esempio: reddito 100 000, versamento 25 000, deduzione 20 000; reddito 300 000, versamento 40 000, deduzione 36 288. Le nuove pianificazioni hanno la casella selezionata.

**Le pianificazioni esistenti** calcolano come prima: nella modalità «domicilio» con la proposta del cantone per tutta la durata, con tabelle proprie senza deduzioni, e senza deduzione del pilastro 3a. Solo «Adattare le deduzioni» crea righe.

**Verifica:** Un importo negativo (anche «in aggiunta» e unico), una percentuale inferiore a 0 % o superiore a 100 % e un anno fiscale al di fuori della simulazione sono errori nella sezione Imposte.

**Risultati:** Con «Mostrare le imposte» i valori annuali mostrano il reddito e le deduzioni (cantone; l’origine – proposta del cantone, in aggiunta, valore proprio, unica, pilastro 3a – nel tooltip della cella); il reddito imponibile è nella colonna **Reddito imponibile** subito prima delle imposte (quello della Confederazione nel tooltip se diverso).

**Casa di cura:** L’app deduce in aggiunta le spese della casa di cura dalla scheda [Casa di cura](#f-care-home) come spese di malattia nella misura in cui superano il 5 % del reddito netto (Confederazione e cantone ciascuno con il proprio reddito netto). I valori annuali indicano la deduzione nell’origine delle deduzioni.

## @f-tax-bill Deduzioni dalla fattura d’imposta
Parole chiave: fattura d’imposta, tassazione, deduzioni, reddito imponibile, sostanza imponibile, resto personale, controllo

La via più precisa verso le [deduzioni dal reddito](#f-tax-deductions) sono le cifre della sua ultima **fattura d’imposta definitiva**. Il pulsante **«Riprendere dalla fattura d'imposta»** nella scheda [Domicilio](#f-tax-location) (solo con l’imposta secondo il domicilio) apre un dialogo con campi della prima pagina della fattura d’imposta: **anno fiscale**, **totale dei redditi**, **reddito imponibile** del cantone, **sostanza imponibile** e facoltativamente **totale imposte** (cantone, comune, Confederazione). L’app ricalcola a ogni inserimento.

- **Fino al pensionamento:** deduzione = totale dei redditi − reddito imponibile, mai negativa. Diventa la prima riga (dall’inizio della simulazione) come valore proprio ai prezzi dell’inizio. L’app deduce separatamente i versamenti nel pilastro 3a della pianificazione; vengono quindi tolti dal valore proprio, altrimenti conterebbero due volte.
- **Dal pensionamento** le spese professionali, la deduzione per coppie con doppio reddito e il pilastro 3a decadono. L’app prende quindi la proposta del cantone senza attività lucrativa e vi aggiunge il **resto personale** come deduzione supplementare: resto = deduzione senza pilastro 3a − proposta del cantone con attività lucrativa, mai negativo. L’app prende la proposta con attività lucrativa dal primo anno completo della pianificazione con reddito da attività lucrativa, per le coppie per l’intera economia domestica. La riga inizia con il prelievo della cassa pensioni; per le coppie con quello della persona che va in pensione per ultima. Esempio: redditi 120 000, imponibile 95 000 → deduzione 25 000; proposta con attività lucrativa 21 000 → resto 4 000; proposta senza attività lucrativa 9 000 → dal pensionamento 13 000.
- **Già in pensione:** se il pensionamento precede l’inizio, la deduzione vale dall’inizio; non c’è resto.
- **Sostanza imponibile:** il dialogo mostra la sostanza imponibile della pianificazione al 31 dicembre del primo anno dopo le deduzioni sociali del cantone (stimate dalla tariffa della sostanza) e la differenza rispetto alla fattura d’imposta. Se differisce, propone con immobili una quota del [valore fiscale](#f-re-tax-value) sul valore venale, senza immobili una [deduzione dalla sostanza](#f-tax-wealth-deductions) dall’inizio. La proposta viene ripresa solo con una spunta. Se la fattura d’imposta indica più sostanza senza immobili, verifichi la sostanza nella pianificazione.
- **Controllo:** con il totale imposte il dialogo mostra la stima dell’app per gli stessi valori imponibili (l’imposta federale sul reddito imponibile del cantone) e lo scostamento in percento. Dal 15 % un avviso indica cause possibili: comune, stato civile, confessione o reddito. L’app continua senza fattore di correzione.

**«Riprendere»** sostituisce le righe delle deduzioni dal reddito; le deduzioni uniche restano. Le righe riprese indicano la loro origine (dalla fattura d’imposta e l’anno fiscale) nella scheda Deduzioni dal reddito, nella riga Ipotesi, nel tooltip delle deduzioni nei valori annuali e nella scheda «Parametri» di un calcolo. Se modifica una riga a mano, la sua origine decade.

**Verifica:** un anno fiscale successivo all’inizio, un importo negativo e un campo obbligatorio vuoto sono errori al campo; «Riprendere» è allora bloccato.

## @f-tax-wealth-deductions Deduzioni dalla sostanza
Parole chiave: deduzioni, sostanza imponibile, imposta sulla sostanza, debiti, deduzioni sociali, unica

La scheda **Deduzioni dalla sostanza** nella sezione Imposte (sotto «Esperto») stabilisce che cosa viene dedotto dalla sostanza al 31 dicembre, per esempio debiti diversi dall’ipoteca o deduzioni cantonali che mancano nelle tariffe. La **sostanza imponibile** = sostanza − deduzioni, mai negativa. Gli immobili contano al loro [valore fiscale](#f-re-tax-value) meno l’ipoteca.

**Sostanza imponibile stimata:** In alto un’avvertenza indica la sostanza imponibile al 31 dicembre del primo anno fiscale, arrotondata a 1 000 franchi, con la quota degli immobili (valore fiscale meno ipoteca), per esempio 500 000 al 31.12.2026, di cui immobili 250 000. La confronti con l’ultima dichiarazione d’imposta. Se un immobile non ha un valore fiscale, l’avvertenza lo segnala: si calcola allora con il valore venale e la sostanza imponibile è spesso nettamente troppo alta. Il link porta al campo «Valore fiscale». Nella modalità «Domicilio» le franchigie cantonali contenute nelle tariffe non sono dedotte; la dichiarazione d’imposta può quindi essere più bassa.

**Dettagli:** La composizione mostra per un anno fiscale a scelta come si forma la sostanza imponibile, strutturata come nella dichiarazione d’imposta: sostanza senza immobili (liquidità, obbligazioni, azioni; senza cassa pensione e pilastro 3a), per ogni immobile il valore fiscale (senza valore fiscale il valore venale, con ⚠) e l’ipoteca, le deduzioni dalla sostanza e la sostanza imponibile. Sotto, ogni immobile ha il campo [Valore fiscale](#f-re-tax-value), lo stesso della sezione Immobili.

**Righe:** «+ Riga» aggiunge una riga dipendente dal tempo con il campo «+ in aggiunta» (all’anno); vale dalla sua data fino alla successiva. Per un anno fiscale conta la riga che vale al 31 dicembre. L’importo è ai prezzi dell’inizio della simulazione e cresce con le [tariffe con il rincaro](#f-tax-indexed).

**Deduzioni uniche:** «+ Deduzione unica» aggiunge una deduzione per un singolo anno fiscale, con anno fiscale, descrizione e importo (ai prezzi attuali, cresce con il rincaro fino all’anno fiscale). Vale alla data di riferimento del 31 dicembre del suo anno fiscale, in aggiunta alla riga applicabile. Esempio: sostanza 800 000, corrente 50 000, unica 100 000 nell’anno 2027 dà nel 2027 una sostanza imponibile di 650 000, negli altri anni 750 000.

**Modalità «domicilio»:** Le deduzioni sociali del cantone sono già comprese nelle tariffe sulla sostanza dei dati fiscali. Le vostre deduzioni si aggiungono; le deduzioni del cantone non possono essere sostituite. La scheda lo indica con una nota.

**Tabelle proprie:** Gli importi sono l’intera deduzione dalla sostanza.

**Le pianificazioni esistenti** non hanno deduzioni dalla sostanza e calcolano come prima.

**Verifica:** Un importo negativo e un anno fiscale al di fuori della simulazione sono errori nella sezione Imposte.

**Risultati:** Con «Mostrare le imposte» i valori annuali mostrano le deduzioni dalla sostanza (l’origine – in aggiunta, unica – nel tooltip della cella) e la sostanza imponibile (la sostanza prima delle deduzioni nel tooltip).

## @f-tax-yield Reddito imponibile della sostanza
Parole chiave: reddito, dividendi, interessi, cedola, distribuzione, rendimento distribuito, reddito imponibile, azioni, obbligazioni, utile di capitale, ad accumulazione

Dividendi e interessi della sostanza sono reddito imponibile. La scheda **Reddito imponibile della sostanza** nella sezione Imposte assume per questo un tasso annuo per ogni classe d’investimento:

- **Azioni (rendimento distribuito):** dividendi in percentuale del saldo.
- **Obbligazioni (cedola):** interessi in percentuale del saldo.
- **La liquidità** e il ricavo di vendita riservato non hanno reddito imponibile (la simulazione non li remunera).

**Calcolo:** Il reddito imponibile di un anno è il saldo medio di ogni classe d’investimento nell’anno × tasso. Esempio: azioni in media 500 000 al 2 %, obbligazioni 300 000 all’1 %: reddito imponibile 13 000. Nel primo e nell’ultimo anno contano solo i mesi della pianificazione. Il reddito conta nel reddito dell’imposta sul reddito, allo stesso modo in entrambe le modalità fiscali; nella modalità [Domicilio](#f-tax-location) prima delle deduzioni del Cantone e della Confederazione. Quali deduzioni valgono (attivo o pensionato) dipende sempre solo dal reddito da attività lucrativa. Il reddito fa parte anche del reddito netto su cui si calcola la quota a carico dei costi di una [casa di cura](#f-care-home). Il tasso non cresce con le [tariffe con il rincaro](#f-tax-indexed), perché è una quota del saldo.

**Parte del rendimento:** Il reddito non è un afflusso supplementare. Il rendimento della simulazione resta uguale (le [serie di rendimento](#s-returns) contengono già dividendi e cedole); nuova è solo l’imposta su di esso. **Gli utili di capitale** non sono imponibili. Il tasso copre i fondi a distribuzione e ad accumulazione, perché anche il reddito trattenuto di un fondo ad accumulazione è imponibile.

**Valori indicativi:** Il rendimento da dividendi di un indice azionario globale è di circa il 2 %, il rendimento dello Swiss Bond Index di circa l’1 %. Le nuove pianificazioni iniziano con azioni 2 % e obbligazioni 1 %. **Le pianificazioni esistenti** si aprono con 0 % e calcolano come prima.

**Strategia a 3 comparti:** Il fabbisogno netto della [strategia a 3 comparti](#f-wd-three_buckets) contiene l’imposta sul reddito del reddito della sostanza (saldi attuali × tassi). Il reddito stesso non conta lì come entrata, perché fa parte del rendimento.

**Non compresi:** imposta preventiva e il suo rimborso, imposta alla fonte estera, deduzione dei costi di gestione patrimoniale.

**Verifica:** Un tasso inferiore a 0 % o superiore a 100 % è un errore nella sezione Imposte.

**Risultati:** Con «Mostrare le imposte» i valori annuali mostrano la colonna **Reddito imponibile** prima dell’imposta sul reddito, se è inserito un tasso.

## @f-tax-multiplier Moltiplicatore d’imposta
Parole chiave: cantone, comune, moltiplicatore

Un fattore su tutte le aliquote delle tabelle, il moltiplicatore d’imposta di **cantone e comune** insieme (ad esempio 2.0 per un’aliquota doppia). Così non deve riscrivere le tabelle se cambia luogo di domicilio, ma modifica solo il fattore. Sono ammessi valori da 0 a 10. Nel modo [Domicilio](#f-tax-location) vale solo per le regole proprie dell’imposta sugli utili immobiliari (casella «Regole proprie invece delle regole del cantone») ed è altrimenti nascosto.

## @f-tax-indexed Tariffe con il rincaro
Parole chiave: progressione a freddo, rincaro, inflazione, scaglioni, tariffa, prezzi attuali

Gli **importi** delle tabelle per reddito, sostanza e prelievo di capitale valgono ai **prezzi attuali** (prezzi all’inizio della simulazione) e crescono con il rincaro simulato del rispettivo percorso; le **aliquote** restano invariate. Così l’onere fiscale non aumenta solo a causa del rincaro (compensazione della progressione a freddo), come fanno la Confederazione e i cantoni.

- **Esempio:** tabella 0 → 0 %, 200 000 → 10 %, rincaro 2 % all’anno. Nel decimo anno lo scaglione superiore è a 243 799. Un reddito di 121 899 (100 000 con il rincaro) ha l’aliquota del 5 % come 100 000 nell’anno iniziale; con tariffe nominali sarebbe il 6.09 %.
- Determinante è l’indice dei prezzi nel mese della tassazione: dicembre per reddito e sostanza, il mese del prelievo per i prelievi di capitale. Anche l’imposta sul reddito stimata nel fabbisogno della [strategia a 3 comparti](#f-wd-three_buckets) è calcolata così.
- Ciò vale anche per le tabelle dei coniugati.
- L’[imposta sugli utili immobiliari](#f-tax-gain) resta invariata, la sua tariffa si riferisce al singolo utile.
- Le deduzioni correnti dal reddito e dalla sostanza crescono allo stesso modo.
- L’eccezione «nominale, senza rincaro» si trova sotto «Simulazione» › «Esperto» ([Che cosa cresce con il rincaro](#inflation)): gli importi valgono allora in termini nominali per tutta la pianificazione. Le pianificazioni più vecchie senza indicazione e le vecchie configurazioni importate mantengono così il loro calcolo precedente.

## @f-tax-gain Imposta sugli utili immobiliari
Parole chiave: utile immobiliare, vendita, immobile, durata di possesso, Zurigo, Argovia, cantone, soglia di esenzione, tariffa, moltiplicatore d’imposta, importo esente, domicilio, approssimazione

L’imposta sull’utile alla **vendita** di un immobile. Le regole sono **diverse da cantone a cantone**. Nel modo [Domicilio](#f-tax-location) valgono le [regole del cantone](#f-tax-gain-canton) dell’immobile; nel modo «Tabelle proprie» (o con la casella «Regole proprie invece delle regole del cantone») le regole proprie, interamente modificabili. **Obbligatorio** (solo per le regole proprie): se la pianificazione contiene un immobile con stato «Proprietà» o «Acquisto pianificato», le regole devono essere impostate (modello Zurigo, modello Argovia o regole proprie, anche con aliquota 0 %). Se mancano, è un errore nella sezione Imposte, e «Avviare la simulazione» e «Avviare l’ottimizzazione» sono bloccati. I soli oggetti in affitto e le pianificazioni senza immobile non hanno bisogno di regole.

Sono disponibili due modelli: **Zurigo** (tariffa a scaglioni con maggiorazioni e riduzioni) e **Argovia** (aliquota diretta secondo la durata di possesso). L’**inserimento proprio** si basa sul modello Argovia.

**Calcolo:** utile = valore di vendita − prezzo d’acquisto pagato. Investimenti che aumentano il valore, costi di trapasso e di vendita non vengono dedotti. Poi:

1. **Soglia di esenzione:** se l’utile è inferiore, è esente da imposta. Dal limite viene tassato l’**intero** utile.
2. **Modello Zurigo, scaglioni:** l’utile viene tassato a scaglioni. Ogni riga vale per la sua parte di utile («a» è l’ampiezza dello scaglione), oltre l’ultimo scaglione vale la sua aliquota.
3. **Modello Zurigo, durata di possesso:** l’imposta viene modificata della maggiorazione o riduzione della riga corrispondente. Vale l’ultima riga i cui anni sono raggiunti (durata di possesso in anni interi, dalla [data d’acquisto](#f-re-bought)).
4. **Riduzione fissa:** una deduzione supplementare in percentuale, indipendente dalla durata di possesso. Nei modelli è impostata a 0.
5. **Moltiplicatore d’imposta:** con l’opzione «Moltiplicare per il moltiplicatore d’imposta» l’imposta viene inoltre moltiplicata per il moltiplicatore d’imposta di cantone e comune (standard: disattivato).

**Modello Cantone Zurigo** (tariffa § 225 StG, stato 09/2025): scaglioni 10 % (primi 4'000), 15 % (6'000), 20 % (8'000), 25 % (12'000), 30 % (20'000), 35 % (50'000), 40 % oltre. Maggiorazione del 50 % con durata di possesso inferiore a 1 anno, del 25 % inferiore a 2 anni. Riduzione da 5 anni interi del 5 %, poi 3 punti percentuali in più all’anno fino al 50 % da 20 anni. Gli utili inferiori a 5'000 sono esenti da imposta. Il moltiplicatore d’imposta agisce solo se l’opzione sopra è attivata.

**Modello Cantone Argovia** (§ 109 StG): nel **modello Argovia** vale, invece degli scaglioni, un’**aliquota diretta sull’intero utile**, dipendente dalla durata di possesso in anni interi: 40 % nel primo anno di possesso, poi ogni anno 2 punti percentuali in meno fino al 20 % nell’undicesimo anno di possesso, poi 1 punto percentuale in meno all’anno fino al 6 % nel 25° anno di possesso e 5 % da 25 anni interi. Senza data d’acquisto vale l’aliquota più alta, e l’app avvisa.

Per un altro cantone adegui le tabelle. I valori si trovano nel foglio delle tariffe dell’amministrazione cantonale delle contribuzioni. Li verifichi prima dell’uso: il modello riflette la situazione a una determinata data e non è una consulenza fiscale.

## @f-tax-gain-canton Imposta sugli utili immobiliari secondo il cantone
Parole chiave: imposta sugli utili immobiliari, cantone, immobile, domicilio, regole, opuscolo fiscale, approssimazione, durata di possesso, mesi, moltiplicatore d’imposta, mediana

Nel modo [Domicilio](#f-tax-location) per ogni immobile vale automaticamente la **regola del cantone in cui si trova**. Sono registrati tutti i 26 cantoni; la fonte è la legge tributaria cantonale (stato 2025/2026).

- **Cantone dell’immobile:** campo per ogni immobile in proprietà o con acquisto pianificato sotto [Immobili](#s-realestate). Senza indicazione vale il cantone di domicilio. Sotto figura la regola del cantone, ad esempio Zurigo «tariffa a scaglioni dal 10 % al 40 %, maggiorazione fino al 50 % sotto 2 anni, riduzione fino al 50 % da 20 anni».
- **Durata di possesso in mesi:** le regole dei cantoni contano la durata di possesso in mesi dalla [data d’acquisto](#f-re-bought) alla vendita; diversi cantoni calcolano maggiorazioni al mese.
- **Moltiplicatore d’imposta:** Berna, Lucerna, Obvaldo, Soletta, Sciaffusa e Giura calcolano un’imposta semplice per il moltiplicatore d’imposta. Se l’immobile si trova nel cantone di domicilio vale il moltiplicatore del comune di domicilio (cantone e comune), altrimenti il moltiplicatore mediano dei comuni del cantone.
- **Procedura:** soglia di esenzione, importo esente, variazione dell’utile secondo la durata di possesso, imposta secondo il modello del cantone (tariffa a scaglioni, aliquota secondo la durata di possesso, aliquota secondo l’utile o aliquota secondo il rendimento annuo), maggiorazione oltre una soglia con breve durata di possesso, maggiorazione o riduzione secondo la durata di possesso, moltiplicatore d’imposta, riduzione, onere massimo.
- **Senza data d’acquisto** non vale alcuna maggiorazione o riduzione secondo la durata di possesso, per i cantoni con aliquota secondo la durata di possesso l’aliquota più alta; la verifica avverte.
- **Approssimazioni (⚠):** per Basilea Città, Basilea Campagna, San Gallo, Vaud e Vallese la regola è solo approssimata; ad esempio a Basilea Città, San Gallo e Vaud si presuppone l’occupazione da parte del proprietario. L’interfaccia mostra la nota con ⚠ presso l’immobile.
- **Non compreso:** l’occupazione da parte del proprietario per immobile, la riduzione per occupazione da parte del proprietario a Basilea Campagna, la riduzione per le spese che aumentano il valore e l’adeguamento dei costi d’investimento al rincaro.
- **Verifica:** le regole proprie non sono obbligatorie. Un cantone sconosciuto di un immobile è un errore sul campo.

Con la casella **«Regole proprie invece delle regole del cantone»** nella sezione Imposte valgono le regole proprie e il moltiplicatore d’imposta per tutti gli immobili, come nel modo «Tabelle proprie».

## @f-tax-tables Tabelle fiscali
Parole chiave: aliquota, interpolazione, reddito, sostanza, prelievo del capitale, coniugati

Ogni tabella ha scaglioni composti da **importo** e **aliquota**. Tra questi l’app interpola linearmente.

| Tabella | Importo | Aliquota |
|---|---|---|
| Imposta sul reddito | reddito annuo imponibile | quota del reddito dovuta come imposta |
| Imposta sulla sostanza | sostanza imponibile | quota della sostanza all’anno |
| Imposta sulle prestazioni in capitale | prelievo di capitale nell’anno | quota del prelievo |

- **Le aliquote sono numeri decimali:** 0.05 sta per 5 %, nell’interfaccia digitato come percentuale.
- L’importo dell’imposta è aliquota × importo × moltiplicatore d’imposta, arrotondato ai centesimi.
- **Limiti:** ogni aliquota è compresa tra 0 % e 100 %, ogni importo è 0 o superiore. I valori al di fuori la verifica li segnala sul campo.
- **L’imposta non può diminuire:** un importo più alto non dà mai un’imposta più bassa. Se l’aliquota tra due scaglioni diminuisce così tanto che aliquota × importo diventa più piccolo, la verifica segnala un **avviso** sullo scaglione interessato; il calcolo avviene comunque. Di solito è stata registrata un’aliquota marginale invece dell’aliquota media. Con più prelievi di capitale nello stesso anno l’imposta su un ulteriore prelievo in questo caso non è mai negativa, ma 0.
- I **prelievi di capitale** nello stesso anno civile (cassa pensione e pilastro 3a) vengono sommati per l’aliquota: per i coniugati per l’economia domestica, per le coppie non sposate per persona (ogni persona paga l’imposta sui propri prelievi). Se ripartisce i prelievi su anni diversi, l’aliquota diminuisce.
- **Coniugati:** con «Coniugati» spuntato (anche per una sola persona) può registrare tabelle proprie per coppie sposate. Senza tabelle proprie valgono quelle normali.
- Parta dall’onere fiscale **effettivo**, non dall’aliquota marginale. Il calcolatore fiscale del Suo cantone fornisce entrambi.

## @tax-accuracy Quanto è precisa l’imposta?
Parole chiave: stima, imposta stimata, precisione, arrotondamento, arrotondato, casi di riferimento, approssimazione, deduzioni, consulenza fiscale, intervallo

L’imposta in VivaPlan è una **stima per la pianificazione**, non una tassazione. I risultati la chiamano quindi «imposta stimata» e mostrano gli importi d’imposta **arrotondati a 100 franchi**. Il calcolo è esatto, l’arrotondamento vale solo per la visualizzazione; la riga di comando e la sua esportazione CSV forniscono i valori esatti.

**Casi di riferimento:** le regole di calcolo sono testate con casi ricalcolati a mano: aliquota della tabella interpolata linearmente per il moltiplicatore d’imposta, prelievi di capitale di un anno sommati per l’aliquota, gradini e durata di possesso dell’imposta sugli utili immobiliari. Il modello del Cantone di Zurigo per l’imposta sugli utili immobiliari corrisponde alla tabella ufficiale. VivaPlan calcola quindi correttamente con le Sue tabelle; quanto ciò sia vicino alla Sua imposta effettiva dipende dalle tabelle.

**Modo «Domicilio»:** 25 casi inventati a Zurigo, Berna, Lucerna, Aarau e San Gallo (reddito da 80 000 a 300 000, sostanza da 300 000 a 4 milioni, pensionati, senza figli né confessione) si scostano meno dell’1 % dal calcolatore d’imposta 2026 dell’AFC; è richiesto al massimo ±5 %. Per le persone attive (sole, coniugate con uno o due redditi da attività lucrativa; salario lordo da 70 000 a 210 000, sostanza da 50 000 a 800 000) anche 25 casi inventati negli stessi comuni si scostano meno dell’1 % dal calcolatore d’imposta dell’AFC. Inoltre concordano casi tratti dai calcolatori d’imposta di Zurigo, Argovia, Grigioni e Vallese (Sion e Agarn; coniugati a Sion con la riduzione) e da tariffe ufficiali (Lucerna, Soletta, Basilea Campagna, Vaud, Giura, Ginevra). Non sono comprese deduzioni individuali, figli, deduzioni dipendenti dal reddito, spese di trasporto e di vitto.

**Prelievi di capitale nel modo «Domicilio»:** 48 casi inventati a Zurigo, Berna, Lucerna, Aarau, San Gallo, Ginevra, Friburgo e Bellinzona (prelievi di 100 000, 500 000 e 1 milione, persone sole e coniugate) si scostano al massimo ±5 % dal calcolatore d’imposta 2026 dell’AFC. Nel Vallese la regola è approssimata (rendita annua per una persona di 65 anni); tre prelievi a Sion si scostano meno dell’1 % dal calcolatore d’imposta del Vallese. A Zugo la regola corrisponde al centesimo al calcolatore d’imposta del cantone.

**Imposta sugli utili immobiliari nel modo «Domicilio»:** per ogni cantone un test verifica casi inventati (utile e durata di possesso) con valori ricalcolati a mano secondo la legge. A Basilea Città, Basilea Campagna, San Gallo, Vaud e nel Vallese la regola è solo approssimata (⚠).

**Approssimazioni:**

- Le Sue tabelle danno un’aliquota media per importo; nel mezzo VivaPlan interpola linearmente. La tariffa reale ha un altro andamento.
- Un unico moltiplicatore d’imposta per Cantone e Comune vale per tutte le tabelle.
- Con le [tariffe con il rincaro](#f-tax-indexed) i gradini crescono con il rincaro simulato; la compensazione dei Cantoni può differire.
- Utile immobiliare = valore di vendita − prezzo d’acquisto pagato.

**Cosa manca:**

- Le singole deduzioni. Sono comprese solo nella misura in cui Lei le include nelle aliquote.
- Il valore locativo, le deduzioni per interessi passivi e manutenzione.
- L’imposta di culto e l’imposta personale nonché l’imposta federale diretta, se non figurano nelle tabelle o nel moltiplicatore.
- Gli investimenti che aumentano il valore, i costi di trapasso e di vendita per l’imposta sugli utili immobiliari.
- Le future modifiche di legge.

**Anno fiscale e fonte:** Nella modalità «Domicilio» la scheda «Domicilio» della sezione Imposte indica sempre l’anno fiscale e la fonte dei dati fiscali, e sotto le approssimazioni del cantone con ⚠: spese professionali, imposta sulle prestazioni in capitale e imposta sugli utili immobiliari nei cantoni dei Suoi immobili. Sotto i valori annuali con la scomposizione dell’imposta e nel confronto (valori annuali «Imposte stimate totali», grafico dei flussi con le imposte) una riga indica l’anno fiscale, la fonte, l’origine delle deduzioni (deduzioni standard del cantone, deduzioni proprie o entrambe) e «Nessuna consulenza fiscale». Se l’anno civile in corso supera di più di un anno l’anno fiscale dei dati, la verifica segnala un avvertimento: tariffe, deduzioni e moltiplicatori possono essere cambiati da allora.

VivaPlan non mostra un **intervallo** (± %): manca una base, e un numero in più farebbe credere a una precisione che non esiste.

**L’imposta sulle prestazioni in capitale e l’imposta sugli utili immobiliari** riguardano spesso molto denaro: prima di decidere, verificare con il calcolatore fiscale del Cantone o con uno specialista. VivaPlan non è una consulenza fiscale.

## @s-realestate Immobili
Parole chiave: casa, appartamento, ipoteca, affitto, acquisto, vendita, sostenibilità

Oggetti propri, pianificati e in affitto. Può registrarne quanti vuole.

- **Proprietà (`owned`):** il valore venale all’inizio della simulazione figura nell’intestazione accanto al nome. Sotto seguono in posizioni fisse la [data d’acquisto](#f-re-bought), il prezzo d’acquisto (pagato, per l’imposta sugli utili immobiliari), «Vendita da» (mai, tappa o data) e spese accessorie al mese. Nel modo fiscale [Domicilio](#f-tax-location) segue il **cantone dell’immobile**, con la regola dell’[imposta sugli utili immobiliari](#f-tax-gain-canton) di questo cantone. Con «Acquisto pianificato» gli stessi campi figurano nelle stesse posizioni (al posto della data d’acquisto «Acquisto da»), manca solo il valore venale; al cambio di stato i valori registrati si mantengono. La vendita deve essere **successiva alla data d’acquisto**, altrimenti è un errore sul campo.
- **Inflazione:** valore e spese accessorie crescono con il rincaro; per ogni oggetto si possono calcolare in termini nominali sotto «Simulazione» › «Esperto» ([Che cosa cresce con il rincaro](#inflation)).
- **Acquisto pianificato (`planned`):** acquisto da tappa, prezzo d’acquisto, ipoteca. A ciò si aggiunge **«Vendita da»** (predefinito ‹mai›, altrimenti data o tappa); la simulazione vende allora l’oggetto in quel momento. La vendita deve essere successiva ad «Acquisto da», altrimenti è un errore sul campo. Con acquisto «dopo la vendita di …» il momento dell’acquisto non è noto al momento della registrazione, allora questa verifica viene meno.
- **Affitto (`rent`):** inizio e affitto al mese (valore attuale; cresce con il rincaro fino al trasloco, poi resta nominale; un affitto che decorre all’inizio è nominale fin dall’inizio; eccezioni sotto «Simulazione» › «Esperto»).
- **Ipoteca:** tranche con periodi, vedi [Ipoteche](#f-re-mortgage).
- **Valore fiscale:** facoltativo per ogni immobile di proprietà o con acquisto pianificato, per l’imposta sulla sostanza, vedi [Valore fiscale](#f-re-tax-value).
- [Vendita forzata](#f-re-emergency) per acquisti «dopo la vendita» e un’avvertenza con i valori della [sostenibilità](#f-re-afford); questi si trovano nelle [impostazioni](#f-settings).
- Il commutatore [Non vendere mai gli immobili](#f-re-never-sell) in alto nella scheda esclude qualsiasi vendita.

Il [costo della vita](#s-flows) lo inserisce **incluse** le spese d’alloggio. Il calcolo usa il costo della vita **senza la quota d’alloggio**; affitto, interessi ipotecari, ammortamento e spese accessorie si aggiungono come **spese d’alloggio** simulate. Dopo la vendita dell’ultimo immobile, nel mese successivo viene acquistato il prossimo con acquisto «dopo la vendita», se è sostenibile e i fondi propri bastano. Altrimenti iniziano gli affitti ‹da dopo la vendita›. Un acquisto «dopo la vendita» viene tentato **una volta** nel mese successivo. Se non è finanziabile, **viene soppresso** con un messaggio negli eventi, e le regole successive intervengono il mese seguente: iniziano gli affitti ‹da dopo la vendita› di questo immobile, vengono tentati ulteriori acquisti «dopo la vendita» di esso. Una **data d’acquisto o di vendita anteriore all’inizio della simulazione** non viene mai raggiunta; la verifica la segnala come avvertenza.

## @f-re-bought Data d’acquisto
Parole chiave: durata di possesso, imposta sugli utili immobiliari, acquisto

Il mese in cui ha acquistato l’immobile. Determina la **durata di possesso** e quindi la maggiorazione o riduzione dell’[imposta sugli utili immobiliari](#f-tax-gain): più a lungo possiede l’immobile, meno imposta è dovuta in molti cantoni.

- Il campo esiste solo per gli immobili in proprietà. Per un immobile **pianificato** conta il mese d’acquisto della simulazione.
- Senza data d’acquisto l’app calcola **senza** maggiorazione o riduzione (l’imposta secondo la tariffa di base). Se sono registrate regole sulla durata di possesso, la verifica lo segnala.
- Determinante è la durata di possesso fino al mese della vendita: con regole proprie in **anni interi**, con le [regole del cantone](#f-tax-gain-canton) (modo Domicilio) in **mesi**.

## @f-re-tax-value Valore fiscale
Stichworte: valore fiscale, imposta sulla sostanza, valore ufficiale, stima

Il **valore fiscale** è il valore ufficiale con cui l’immobile figura nella dichiarazione d’imposta. Spesso è pari al **60–80 % del valore venale**, con stime più vecchie anche nettamente inferiore. Riprenda il valore dalla sua **dichiarazione d’imposta** o dall’ultima tassazione.

- Il campo esiste per gli immobili di proprietà e con acquisto pianificato, non per gli affitti. Si può scegliere il **valore predefinito** (nessuna indicazione; nelle pianificazioni più vecchie «come il valore venale»), un **importo** o **% del valore venale**.
- Solo l’**imposta sulla sostanza** lo utilizza: per ogni immobile è imponibile il valore fiscale meno l’ipoteca invece del valore venale meno l’ipoteca. Esempio: valore venale 1'000'000, valore fiscale 650'000, ipoteca 400'000 danno una sostanza imponibile di 250'000 invece di 600'000.
- Sostanza, risultati, imposta sugli utili immobiliari, sostenibilità e vendita forzata continuano a usare il **valore venale**.
- Un importo mantiene il suo **rapporto con il valore venale** (per un acquisto pianificato con il prezzo d’acquisto): se il valore venale aumenta con l’inflazione, il valore fiscale aumenta nella stessa proporzione. In realtà il valore fiscale cambia solo con una nuova stima; negli anni è un’approssimazione.
- **Valore predefinito:** Le nuove pianificazioni usano il **50 % del valore venale** per gli immobili senza un valore fiscale proprio (campo: «valore predefinito (50 % del valore venale)»). Il valore predefinito è un’ipotesi uniforme di VivaPlan per tutti i cantoni, non un dato ufficiale: secondo il cantone e l’età della stima, i valori fiscali sono per lo più nettamente inferiori al valore venale. Un importo o una quota propri lo sostituiscono. La pianificazione salva solo che vale il valore predefinito; la quota la determina il programma. Esempio: valore venale 1'000'000, ipoteca 400'000 danno con il valore predefinito una sostanza imponibile di 100'000.
- **Le pianificazioni più vecchie** continuano a usare il valore venale senza valore fiscale. Un’avvertenza nella scheda dell’immobile propone il valore predefinito; il link lo applica a tutti gli immobili senza un valore fiscale proprio. La [composizione della sostanza imponibile](#f-tax-wealth-deductions) indica la provenienza «valore predefinito».
- Un importo negativo o una percentuale fuori da 0–100 % è un errore nel campo.
- Nei [valori annuali](#r-table) le colonne delle imposte comprendono la colonna **Valore fiscale immobili** (somma al 31 dicembre, immobili senza valore fiscale al valore predefinito o al valore venale).
- Non sono calcolati: il valore locativo, il riparto fiscale per un immobile in un altro cantone e le deduzioni per sottoutilizzo.

## @f-re-mortgage Ipoteche
Parole chiave: tranche, SARON, ipoteca fissa, ammortamento, margine, interesse

Un’ipoteca è composta da **tranche**. Ogni tranche ha una sequenza di **periodi**:

- **fissa:** tasso d’interesse fisso per la durata del periodo
- **SARON:** SARON più margine (il SARON ha un limite inferiore di 0). L’andamento del SARON lo fornisce la serie temporale o il valore fisso, vedi [SARON](#ret-saron).

Il primo periodo inizia alla data d’inizio della tranche (all’acquisto), ogni periodo successivo alla fine del precedente. Poi la tranche viene prorogata alle stesse condizioni o rimborsata.

- **Interesse** = debito × tasso / 12 al mese. Matura e viene pagato insieme all’ammortamento alla **fine del trimestre** (marzo, giugno, settembre, dicembre).
- **Ammortamento:** importo fisso per trimestre, diretto (riduce il debito) o **indiretto** (versamento nel [pilastro 3a](#s-pillar3a) della persona; con due persone sceglie il conto sulla tranche).
- **Ammortamento indiretto solo fino all’inizio della rendita AVS:** dal mese d’inizio della rendita AVS della persona lo stesso importo viene ammortizzato **direttamente** e riduce l’ipoteca. Un periodo che inizia solo dall’inizio della rendita AVS (anche un rinnovo) ammortizza sempre direttamente; la selezione mostra allora «diretto (dall’AVS)». L’anteprima del periodo annota ‹+ 3a fino all’AVS› se un periodo va oltre l’inizio della rendita AVS. Poiché ogni persona ha un inizio della rendita AVS, la regola vale sempre. Il passaggio appare negli [eventi](#r-events) (‹Ammortamento ipoteca … dall’inizio della rendita AVS diretto invece che tramite il pilastro 3a›), se la tranche ha ammortizzato prima indirettamente e continua ad ammortizzare.
- Se il pilastro 3a viene riscosso con «Rimborsare l’ipoteca», gli importi vanno prima alle tranche con ammortamento indiretto in questo conto.
- Alla vendita gli interessi maturati vengono pagati subito.

## @f-re-emergency Vendita forzata
Parole chiave: vendita, liquidità, soglia, vendita forzata, vendita obbligata

Se la sostanza liquida diventa scarsa, l’app vende l’immobile nel mese successivo. La **soglia** sono mesi di spese non coperte: se la sostanza liquida copre meno mesi (spese meno reddito), è necessaria una vendita forzata.

Una vendita forzata mostra che la pianificazione non funziona senza vendita immobiliare. Se [Non vendere mai gli immobili](#f-re-never-sell) è attivato, non c’è vendita forzata. Appare negli [eventi](#r-events) dei risultati. Più alta è la soglia, prima si vende.

## @f-re-never-sell Non vendere mai gli immobili
Parole chiave: commutatore, vendita, vendita forzata, sostenibilità, senza vendita, mantenere

Con questo commutatore in alto nella scheda «Immobili» verificate se la pianificazione funziona anche senza vendere un immobile. Vale per **tutti** gli immobili della pianificazione ed è salvato nel file (predefinito: disattivato).

- **Nessuna vendita:** nessuna vendita pianificata («Vendita da»), nessuna [vendita forzata](#f-re-emergency) e nessuna vendita perché un’ipoteca non è più [sostenibile](#f-re-afford) al rinnovo. L’ipoteca viene allora **rinnovata per intero**; un evento segnala «non sostenibile». Con più simulazioni l’indicatore **«Rinnovi non sostenibili»** della [panoramica](#r-property) mostra in quante simulazioni ciò è accaduto (numero e quota); un pagamento unico effettuato non vi rientra.
- **I costi continuano:** interessi, ammortamento e spese accessorie fino alla fine della pianificazione.
- **Acquisti:** un acquisto pianificato con una data o una tappa ha luogo. Acquisti e affitti «dopo la vendita» **non** hanno luogo; la verifica lo segnala, come pure le vendite pianificate.
- **Sostanza esaurita:** se la sostanza liquida non basta più, l’esecuzione conta come esaurita, come senza immobile. Il tasso di successo è quindi di solito più basso che con una vendita.
- **Sola lettura:** finché il commutatore è attivo, tutti gli altri campi della scheda sono grigi e di sola lettura. I valori restano invariati e tornano a valere quando il commutatore viene disattivato.
- **Casa di cura:** Anche con «Vendere l’abitazione di proprietà all’entrata» non viene venduto nulla all’entrata nella [casa di cura](#f-care-home); le spese dell’immobile continuano.

## @f-re-afford Sostenibilità
Parole chiave: banca, sostenibilità, grado d’anticipo, tasso d’interesse teorico, fondi propri, ipoteca, consumo della sostanza, pensionamento

Il calcolo della sostenibilità delle banche, applicato agli acquisti «dopo la vendita» e a ogni rinnovo di un periodo ipotecario. Un’ipoteca è sostenibile se i **costi teorici** non superano una quota del reddito. I valori si trovano nelle [impostazioni](#f-settings) e valgono per tutte le pianificazioni su questo dispositivo.

- **Quota massima del reddito:** di solito un terzo (33 %).
- **Tasso d’interesse teorico:** di solito il 5 %, indipendentemente dall’interesse attuale.
- **Spese accessorie:** di solito l’1 % del prezzo all’anno.
- **Reddito:** sono computati il reddito da attività lucrativa e le rendite. Se il reddito non basta, l’ipoteca viene ridotta; l’acquisto mostra allora il **rimborso unico** dalla sostanza come evento.
- **Consumo della sostanza nella sostenibilità** (interruttore, standard disattivato) con un **tasso all’anno** (standard 3 %): attivato, conta come reddito anche questa quota della **sostanza libera** (liquidità, azioni, obbligazioni nel mese della verifica); all’acquisto solo la sostanza dopo la deduzione dei fondi propri per questo oggetto. Il pilastro 3a e il libero passaggio non contano. Esempio: rendita 60'000, sostanza libera 2'000'000, tasso 3 % → reddito computato 120'000.
- **Grado d’anticipo massimo:** di solito l’80 % del prezzo. Almeno il 20 % sono fondi propri.
- **Grado d’anticipo massimo dal pensionamento** (standard 65 %): appena tutte le persone dell’economia domestica sono in pensione (prelievo della cassa pensione), l’ipoteca all’acquisto e a ogni rinnovo è al massimo questa quota del valore venale nel mese della verifica. L’eccedenza è trattata come sostenibilità insufficiente.

La prassi delle banche è diversa: alcune computano dal 3 al 5 % della sostanza libera, altre nessun tasso fisso; dal pensionamento finanziano di solito dal 60 al 67 %. Adatti i valori alla Sua banca.

Se l’acquisto non è sostenibile, l’ipoteca viene ridotta. Se anche allora i fondi propri non bastano, non si acquista e iniziano gli affitti.

**A ogni rinnovo** di un periodo ipotecario (anche per immobili in proprietà) l’app verifica di nuovo la sostenibilità dell’intera ipoteca, con il reddito del mese del rinnovo (attività lucrativa e rendite, con il consumo della sostanza se attivato) e, appena tutti sono in pensione, con il grado d’anticipo dal pensionamento. Se l’ipoteca non è più sostenibile, la simulazione calcola il **pagamento unico minimo necessario** perché lo sia di nuovo, e lo paga dalla sostanza (in proporzione su tutte le tranche). Ciò vale solo se poi la sostanza **non è negativa** **e** il [criterio di vendita forzata](#f-re-emergency) dopo il pagamento **non** scatta (stimato con le spese pianificate e le spese d’alloggio dopo il pagamento). Allora l’ipoteca viene rinnovata con l’importo ridotto. Altrimenti **non vi è alcun pagamento**, l’ipoteca **non viene rinnovata**, e viene avviata una **vendita forzata** dell’immobile (eseguita nel mese successivo). La vista dei risultati registra ogni rinnovo (data ed età, sostenibilità, pagamento unico, nuova ipoteca, vendita forzata con motivo «sostanza < 0» o «criterio di vendita forzata») e conta con più simulazioni in quante simulazioni avvengono pagamenti unici e vendite forzate, anche per percentile. Caso tipico: il reddito diminuisce con il pensionamento.

## @s-scenarios Scenari e simulazione
Parole chiave: variazione, sweep, parametro, simulazioni, entità

Qui stabilisce quali parametri vengono **variati sistematicamente**. «Avviare la simulazione» calcola la pianificazione come registrata e in più ogni combinazione delle variazioni con tutti i percorsi dei rendimenti. Il risultato è una tabella e una heatmap nei [risultati degli scenari](#r-scenarios).

La durata della simulazione figura inoltre a sinistra del pulsante «Avviare la simulazione». Durante il calcolo può fermarla con **Interrompere**.

Il numero di combinazioni **non è limitato**; solo con più di **1'000'000** di combinazioni il motore di calcolo rifiuta (protezione della memoria). Se il tempo di calcolo stimato supera **un minuto**, l’app chiede conferma prima dell’avvio (durata e numero di simulazioni, «Avviare comunque» o «Interrompere»). L’indicazione «Dimensione» mostra quante simulazioni nascono in tutto (variazioni × percorsi); un **simbolo di avvertimento giallo (⚠)** accanto appare non appena il tempo di calcolo stimato supera **90 secondi**, e una **stima approssimativa della durata**. Si basa su una breve misurazione della Sua pianificazione su questo computer. L’intervallo «per lo più da … a …» tiene conto del fatto che le simulazioni durano più o meno a lungo a seconda della pianificazione. Ogni variazione e ogni percorso supplementare allunga di conseguenza il tempo di calcolo: 35 combinazioni con 50'000 percorsi durano con il bootstrap circa 10 secondi, con il modello parametrico circa un minuto.

Le variazioni stesse: [Variazioni](#f-sweep).

## @f-sweep Variazioni
Parole chiave: sweep, da, a, passo, parametro, heatmap

Una variazione ha un **parametro**, un intervallo **da … a** e un **passo**. L’app calcola ogni valore e mostra l’effetto su tasso di successo e sostanza finale.

| Parametro | Significato |
|---|---|
| Quota azionaria | quota azionaria: azioni nel portafoglio di azioni e obbligazioni (senza cash) |
| Prelievo del capitale cassa pensione | quota del prelievo del capitale |
| Mese d’inizio storico | solo con la fonte ‹storico› |
| Data della prestazione della cassa pensione | mese della tappa «Prestazione della cassa pensione» |
| Fattore spese | fattore sul costo della vita senza la quota d’alloggio (0.9 = 10 % in meno); le spese d’alloggio restano |
| Tasso di prelievo (tasso fisso / inizio Guardrails) | tasso del prelievo fisso o tasso iniziale con Guardrails |

Con due variazioni nasce una **heatmap** su entrambi i parametri. Tutte le combinazioni usano **gli stessi percorsi dei rendimenti**. Le differenze provengono così solo dai parametri, non dal caso.
