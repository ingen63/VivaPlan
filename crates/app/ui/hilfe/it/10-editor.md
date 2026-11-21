# Editor: sezioni e campi

## @s-general Economia domestica e progetto
Parole chiave: nome, valuta, persone, nascita, seconda persona, doppio reddito, periodo di simulazione

Qui stabilisce i dati di base.

- **Nome del progetto e valuta:** tutti gli importi vengono registrati nella valuta del progetto (CHF, EUR o USD). Gli indici della [strategia d’investimento](#f-ret-series) sono in CHF; con una pianificazione in EUR o USD appare un’avvertenza.
- **Persona 1:** nome e mese di nascita (formato MM.AAAA). Dal mese di nascita l’app calcola l’età per le tappe (‹a 65 anni›) e per le didascalie dei grafici.
- **+ Seconda persona:** aggiunge una seconda persona con reddito proprio, cassa pensione propria e AVS propria (doppio reddito). Imposte, sostanza e spese valgono per l’economia domestica. Vedi [Seconda persona e regole per coppie sposate](#f-couple).
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

Non si possono eliminare, ma il loro momento è liberamente selezionabile (età o data), purché sia entro i limiti; i limiti stessi sono ammessi. I limiti sono modificabili nelle [Impostazioni](#f-settings). Con due persone si chiamano «Prestazione della cassa pensione persona 2» e «Inizio della rendita AVS persona 2». I nomi delle tappe fisse appaiono nella lingua scelta e non vengono salvati nel file di pianificazione. La **fine del reddito da attività lucrativa** non è una tappa: la stabilisce nel [reddito](#s-flows). Se dopo la prestazione della cassa pensione il reddito da attività lucrativa è ancora maggiore di 0, la verifica mostra un avviso con l’importo.

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

Una riga sotto le schede mostra che cosa diventa l’avere alla prestazione della cassa pensione (capitale e rendita).

Il **pilastro 3a** ha una sezione propria, vedi [Pilastro 3a](#s-pillar3a).

## @s-pillar3a Pilastro 3a
Parole chiave: 3a, previdenza vincolata, versamento, riscossione, piano di riscossione, età, data, avere, importo massimo, ammortamento indiretto

Il pilastro 3a è un conto proprio **per persona**, separato da cassa pensione e ipoteca.

- **Avere all’inizio della simulazione** e **remunerazione / rendimento annuo**.
- **Versamento all’anno:** da 0 all’importo massimo (7'258 franchi, modificabile nelle [Impostazioni](#f-settings)). L’app versa ogni mese un dodicesimo, fino a prima del mese dell’**inizio della rendita AVS**. Il versamento **conta tra le spese**: appare in «Spese totali» dei valori annuali, aumenta il prelievo e fa parte del fabbisogno netto della strategia a 3 comparti.
- **Piano di riscossione:** righe composte da momento e importo, ad esempio 2041 50'000 e 2042 50'000. Il **momento** lo stabilisce come per una tappa come **data** (MM.AAAA) o come **età** della persona a cui appartiene il conto (ad esempio 61.5 anni). Accanto figurano il mese che ne risulta e l’età. Se passa tra «Data» ed «Età», il momento resta uguale. Senza importo viene riscosso l’intero avere. Una riscossione è possibile **al più presto 5 anni prima dell’inizio della rendita AVS**, anche secondo l’età; una voce anteriore è un errore sul campo. Se un importo è superiore all’avere, viene riscosso solo l’avere (avvertenza negli eventi).
- **Avere dall’anteprima:** ogni riga mostra l’**avere prima della riscossione** e il **resto dopo**, sotto il piano figura l’avere versato all’inizio della rendita AVS. I valori provengono dall’anteprima con rendimenti fissi; se la pianificazione ha un errore, non c’è anteprima, e figura ‹–›.
- **+ Riscossione** crea una riga un anno dopo l’ultima (senza righe: nel primo mese ammesso, 5 anni prima dell’inizio della rendita AVS, non prima dell’inizio della simulazione), al massimo all’inizio della rendita AVS. Come importo figura l’avere in quel momento dopo le riscossioni precedenti, arrotondato per difetto a 1'000. In seguito può modificarlo liberamente.
- **Resto all’inizio della rendita AVS:** ciò che resta sul conto viene versato **al più tardi nel mese d’inizio della rendita AVS**.
- **Utilizzo:** nella sostanza libera o, se esiste una tranche con ammortamento indiretto in questo conto, «Rimborsare l’ipoteca» (prima le tranche con ammortamento indiretto).

Ogni riscossione è un **prelievo di capitale** con imposta sulle prestazioni in capitale. I prelievi nello stesso anno civile (cassa pensione e pilastro 3a) vengono sommati per l’aliquota, per le coppie non sposate per persona.

L’**ammortamento indiretto** di un’ipoteca confluisce in questo conto fino a prima dell’inizio della rendita AVS (vedi [Ipoteche](#f-re-mortgage)). Versamento e ammortamento indiretto insieme non dovrebbero superare l’importo massimo annuo; altrimenti la verifica mostra un avviso.

> Esempio: nascita 06.1980, avere 120'000 senza interesse, riscossione a 61.5 anni (risulta 12.2041) di 50'000: avere prima della riscossione 120'000, resto dopo 70'000. «+ Riscossione» propone una riga a 62.5 anni (12.2042) di 70'000.

> Esempio: avere 120'000, riscossioni 2041 e 2042 di 50'000 ciascuno, inizio della rendita AVS 2045: il resto di 20'000 (più interesse) viene versato nel mese d’inizio della rendita AVS. Se l’inizio della rendita AVS viene spostato al 2046, il piano resta valido, e il resto arriva nel 2046.

## @f-settings Impostazioni
Parole chiave: impostazioni, limiti, età AVS, prestazione della cassa pensione, importo massimo, 3a, valori standard, lingua

Sul Mac menu ‹VivaPlan → Impostazioni…› (⌘,), con Windows e Linux menu «File → Impostazioni…» (Ctrl+,), oppure il pulsante «Impostazioni» in alto. Le impostazioni valgono per **tutte le pianificazioni su questo dispositivo** e vengono salvate in locale, **non nel file di pianificazione**.

| Impostazione | Standard |
|---|---|
| Lingua | come il sistema |
| Inizio della rendita AVS: limite inferiore e superiore | 63 e 70 anni |
| Prestazione della cassa pensione: limite inferiore e superiore | 58 e 65 anni |
| Pilastro 3a: importo massimo del versamento all’anno | 7'258 |

**Lingua:** «come il sistema» segue la lingua del sistema operativo (al primo avvio; senza regione, it_CH vale come italiano), altrimenti vale la lingua scelta. Sono offerti tedesco, inglese, francese e italiano; se manca la lingua del sistema, vale il tedesco. L’aiuto appare nella lingua scelta; un capitolo non ancora tradotto appare in tedesco. La stessa scelta si trova nel menu «Vista → Lingua». Un cambio ha effetto subito, anche sui menu; la pianificazione, le modifiche non salvate e la vista restano. Numeri, importi e date appaiono in ogni lingua nel formato svizzero (1'234'567.50, 2.5 %, 03.2040), e il file di pianificazione è lo stesso in ogni lingua. I nomi nella pianificazione (persone, immobili, tappe) non vengono tradotti; solo la pianificazione d’esempio riceve i suoi nomi nella lingua scelta all’apertura, con le stesse cifre.

Il limite inferiore non può essere superiore a quello superiore, le età sono anni interi, l’importo massimo è 0 o superiore. «Valori standard» ripristina tutto dopo una domanda di conferma. Se dopo una modifica una pianificazione è al di fuori dei nuovi limiti, la verifica lo segnala; la stessa pianificazione può perciò dare un messaggio su un altro dispositivo con altre impostazioni. La riga di comando usa sempre i valori standard.

## @f-pk Cassa pensione
Parole chiave: LPP, avere di vecchiaia, contributi, remunerazione, certificato di previdenza

- **Avere di vecchiaia all’inizio della simulazione (MM.AAAA):** l’avere secondo il certificato di previdenza (avere di vecchiaia inclusa la parte sovraobbligatoria). Le proiezioni del certificato (proiezione con interesse) non vanno inserite qui, le calcola l’app stessa.
- **Prestazione da:** sempre la tappa **Prestazione della cassa pensione** della persona (da 58 a 65 anni). Il momento lo modifica sotto [Tappe](#s-milestones).
- **Contributi al mese:** somma dei contributi del lavoratore e del datore di lavoro (e dei riscatti) versati fino alla prestazione. Con la prestazione terminano.
- **Remunerazione all’anno:** tasso d’interesse sull’avere di vecchiaia. La remunerazione viene accreditata a dicembre. Realistici sono dall’1 % al 2 %.
- [Quota del prelievo del capitale](#f-pk-lumpsum) e [aliquota di conversione](#f-pk-conversion) determinano quanto viene versato come capitale e quanto come rendita.

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
Parole chiave: rendita, riscossione, rincaro, indice, 1° pilastro, anni di contribuzione

- **Rendita al mese all’inizio della prestazione:** la Sua rendita mensile presumibile secondo il calcolo anticipato della rendita della cassa di compensazione, ai prezzi attuali.
- **Prestazione da:** sempre la tappa **Inizio della rendita AVS** della persona (da 63 a 70 anni). Un’anticipazione o un rinvio modifica la rendita. Adegui allora la rendita di conseguenza.
- **Adeguare annualmente la rendita al rincaro:** l’AVS viene aumentata a gennaio del rincaro accumulato dall’ultimo gennaio. Lasci il campo spuntato se la rendita deve crescere di pari passo.

Con due persone coniugate esiste un limite superiore per entrambe le rendite insieme, vedi [Seconda persona e regole per coppie sposate](#f-couple).

## @f-couple Seconda persona e regole per coppie sposate
Parole chiave: coppia sposata, coniugati, limitazione, 150 percento, partner, doppio reddito

Con «+ Seconda persona» registra un’economia domestica con due persone.

- Ogni persona ha **reddito proprio, cassa pensione propria e AVS propria**.
- Imposte, sostanza, spese e immobili valgono per l’**economia domestica**.
- **L’economia domestica è considerata pensionata** non appena tutte le persone con data di riscossione sono pensionate. Solo allora intervengono le strategie di prelievo.

**Coniugati** (regole per coppie sposate): se entrambe le rendite AVS sono esigibili, insieme vengono limitate al massimo a **3'780 franchi al mese** (150 % della rendita massima 2025). Il valore è modificabile, se la rendita massima aumenta. Il limite aumenta ogni gennaio una volta con il rincaro, non appena almeno una rendita con adeguamento al rincaro è in corso. Sotto [Imposte](#s-taxes) si possono registrare tabelle proprie per coniugati. I prelievi di capitale nello stesso anno vengono sommati per i coniugati, per le coppie non sposate per persona.

**Non calcolati:** decesso di una persona, rendite per superstiti, rendita vedovile. La pianificazione calcola come se entrambi vivessero fino alla fine.

## @s-flows Entrate e spese
Parole chiave: reddito, costo della vita, spese, importi unici, eredità, inflazione

Importi mensili. **Le spese vengono registrate ai prezzi attuali** e adeguate dall’app all’inflazione. Inserisce quindi i Suoi costi attuali, anche per anni futuri.

- **Reddito da attività lucrativa netto al mese:** ciò che arriva sul Suo conto dopo le deduzioni (AVS, cassa pensione; le imposte sono calcolate separatamente). Termina quando lo imposta a 0 in modo [dipendente dal tempo](#f-timed), ad esempio ‹da prestazione della cassa pensione 0›. La prestazione della cassa pensione non lo termina da sé; se in seguito è ancora maggiore di 0, la verifica mostra un avviso con l’importo.
- **Aumenta con il rincaro** (per persona): con la spunta gli importi, anche ogni riga di un reddito dipendente dal tempo, valgono ai **prezzi attuali** (prezzi all’inizio della simulazione) e crescono con il rincaro del rispettivo percorso, esattamente come il costo della vita. Esempio: 8'000 al mese con un rincaro del 2 % sono dopo un anno 8'160, dopo cinque anni circa 8'830. Senza spunta l’importo resta nominalmente uguale. Le nuove pianificazioni e una seconda persona appena aggiunta hanno la spunta; le pianificazioni meno recenti senza indicazione calcolano come finora senza. Imposte, valori annuali e sostenibilità usano l’importo adeguato; l’avviso dopo la prestazione della cassa pensione indica allora l’importo «ai prezzi dell’inizio della simulazione».
- **Costo della vita al mese:** l’**importo inserito** comprende tutto ciò che spende, **incluso l’affitto o le spese d’alloggio** (interesse ipotecario, ammortamento, spese accessorie), come figura sull’estratto conto. L’app deduce la **quota d’alloggio**, cioè le spese d’alloggio del mese d’inizio, tratte dalla sezione [Immobili](#s-realestate), come media mensile (interesse e ammortamento vengono pagati trimestralmente), adeguata al rincaro, e cioè in ugual misura da ogni riga. Il **calcolo** usa il costo della vita senza la quota d’alloggio più le spese d’alloggio **simulate**: se l’interesse cambia a un rinnovo o se dopo una vendita passa in affitto, le spese cambiano di conseguenza. Esempio: 8'000 incluse 2'000 di spese d’alloggio; dopo un aumento dell’interesse di 300 le spese sono 8'300, con un affitto di 2'500 sono 8'500. Se una riga è inferiore alla quota d’alloggio, la verifica segnala un errore. Il versamento nel [pilastro 3a](#s-pillar3a) non è incluso. Le pianificazioni meno recenti (costo della vita senza spese d’alloggio) l’app le converte all’apertura, un’avvertenza indica l’importo aggiunto. Le imposte vengono calcolate separatamente. Un valore realistico è il singolo fattore più importante per il risultato. A tale scopo osservi due o tre anni di estratti conto.
- **Importi unici:** data, importo e descrizione. Positivo è un’entrata (eredità, donazione), negativo una spesa (auto, viaggio, ristrutturazione).

Con l’età le spese spesso diminuiscono, ad esempio meno viaggi dagli 80 anni, mentre aumentano i costi per la salute. Usi a tale scopo i [valori dipendenti dal tempo](#f-timed).

## @f-timed Valori dipendenti dal tempo
Parole chiave: costante, dipendente dal tempo, riga, da, andamento, minigrafico

Molti importi e aliquote esistono in due forme:

- **Costante:** un valore per l’intera simulazione.
- **Dipendente dal tempo:** una sequenza di righe ‹da *momento* vale *valore*›. Il momento è una tappa (‹da prestazione della cassa pensione›) o una data fissa. Ogni riga vale finché inizia la successiva.

Esempio costo della vita: ‹da inizio 9'500›, ‹da prestazione della cassa pensione 8'500›, ‹da 80 7'000›.

Il piccolo grafico accanto al campo mostra l’andamento. Con la rotella del mouse ingrandisce, i pulsanti «Tutto», ‹10 a.›, ‹5 a.› e ‹2 a.› scelgono l’intervallo.

Le righe inserite più in basso nell’elenco, ma che iniziano prima di un’altra, l’app le ordina secondo il momento. Può quindi spostare tranquillamente le tappe, l’ordine dei valori resta coerente.

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
- Il valore può essere [dipendente dal tempo](#f-timed), ad esempio 70 % fino al pensionamento e 50 % dopo.

Quanto la quota azionaria cambi il risultato lo mostra una [variazione](#f-sweep) della quota azionaria, o l’[ottimizzazione](#s-optimize), che cerca contemporaneamente anche la quota.

## @f-reserve Riserva di cash
Parole chiave: cash, riserva, mesi, liquidità, riserva di liquidità

Quanti **mesi di spese non coperte** (spese meno redditi e rendite) vengono tenuti come cash **dal pensionamento**, prima che il resto venga investito secondo la [quota azionaria](#f-stockshare). Prima del pensionamento vale il campo «Cash» sotto [Sostanza e investimento](#s-wealth).

- Una riserva alta protegge dal dover vendere azioni a corsi bassi, ma costa rendimento.
- Impostabile solo con la strategia «Standard» (scheda «Standard» sotto «Strategia di prelievo») ed efficace solo dal pensionamento. **Tutte le altre strategie** (Guardrails, tasso fisso, VPW, Endowment) calcolano dal pensionamento con una **riserva fissa di 3 mesi**; con la [strategia a 3 comparti](#f-wd-three_buckets) i comparti sostituiscono la riserva.
- Una pianificazione meno recente con un’altra strategia e una riserva diversa da 3 mesi calcola perciò con 3 mesi e può dare risultati leggermente diversi. L’app lo segnala una volta, alla prima apertura di una tale pianificazione; in seguito l’avvertenza non appare più. Il valore salvato resta nel file e vale di nuovo se passa a «Standard».
- Sensato in modo dipendente dal tempo: ad esempio 6 mesi fino al pensionamento e 12 mesi dopo.

## @s-taxes Imposte
Parole chiave: imposta sul reddito, imposta sulla sostanza, imposta sulle prestazioni in capitale, moltiplicatore d’imposta, tabella

Semplici **tabelle delle aliquote**, che può ricavare dal calcolatore fiscale del Suo cantone. Tra gli scaglioni si **interpola linearmente**, l’aliquota viene moltiplicata per il [moltiplicatore d’imposta](#f-tax-multiplier).

- [Moltiplicatore d’imposta e tabelle](#f-tax-tables)
- L’**imposta sul reddito** viene calcolata a dicembre sul reddito annuo (incluse le rendite), l’**imposta sulla sostanza** sul portafoglio più gli immobili netti. Entrambe vengono pagate a maggio dell’anno successivo.
- Un’imposta ancora aperta alla fine della pianificazione viene dedotta nell’ultimo mese.
- L’[imposta sugli utili immobiliari](#f-tax-gain) è dovuta alla vendita di un immobile, vedi anche [Immobili](#s-realestate).

I valori d’esempio sono solo segnaposto. Per il Suo risultato li sostituisca con i valori del Suo luogo di domicilio.

## @f-tax-multiplier Moltiplicatore d’imposta
Parole chiave: cantone, comune, moltiplicatore

Un fattore su tutte le aliquote delle tabelle, il moltiplicatore d’imposta di **cantone e comune** insieme (ad esempio 2.0 per un’aliquota doppia). Così non deve riscrivere le tabelle se cambia luogo di domicilio, ma modifica solo il fattore. Sono ammessi valori da 0 a 10.

## @f-tax-gain Imposta sugli utili immobiliari
Parole chiave: utile immobiliare, vendita, immobile, durata di possesso, Zurigo, Argovia, cantone, soglia di esenzione, tariffa, moltiplicatore d’imposta

L’imposta sull’utile alla **vendita** di un immobile. Le regole sono **diverse da cantone a cantone** e perciò interamente modificabili. **Obbligatorio:** se la pianificazione contiene un immobile con stato «Proprietà» o «Acquisto pianificato», le regole devono essere impostate (modello Zurigo, modello Argovia o regole proprie, anche con aliquota 0 %). Se mancano, è un errore nella sezione Imposte, e «Avviare la simulazione» e «Avviare l’ottimizzazione» sono bloccati. I soli oggetti in affitto e le pianificazioni senza immobile non hanno bisogno di regole.

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
- **Coniugati:** con due persone e «Coniugati» spuntato può registrare tabelle proprie per coppie sposate. Senza tabelle proprie valgono quelle normali.
- Parta dall’onere fiscale **effettivo**, non dall’aliquota marginale. Il calcolatore fiscale del Suo cantone fornisce entrambi.

## @s-realestate Immobili
Parole chiave: casa, appartamento, ipoteca, affitto, acquisto, vendita, sostenibilità

Oggetti propri, pianificati e in affitto. Può registrarne quanti vuole.

- **Proprietà (`owned`):** il valore venale all’inizio della simulazione figura nell’intestazione accanto al nome. Sotto seguono in posizioni fisse la [data d’acquisto](#f-re-bought), il prezzo d’acquisto (pagato, per l’imposta sugli utili immobiliari), «Vendita da» (mai, tappa o data) e spese accessorie al mese. Con «Acquisto pianificato» gli stessi campi figurano nelle stesse posizioni (al posto della data d’acquisto «Acquisto da»), manca solo il valore venale; al cambio di stato i valori registrati si mantengono. La vendita deve essere **successiva alla data d’acquisto**, altrimenti è un errore sul campo.
- **Inflazione:** valore e spese accessorie si possono adeguare all’inflazione **singolarmente** per ogni oggetto; per gli oggetti in affitto, anche l’affitto.
- **Acquisto pianificato (`planned`):** acquisto da tappa, prezzo d’acquisto, ipoteca. A ciò si aggiunge **«Vendita da»** (predefinito ‹mai›, altrimenti data o tappa); la simulazione vende allora l’oggetto in quel momento. La vendita deve essere successiva ad «Acquisto da», altrimenti è un errore sul campo. Con acquisto «dopo la vendita di …» il momento dell’acquisto non è noto al momento della registrazione, allora questa verifica viene meno.
- **Affitto (`rent`):** inizio e affitto al mese (valore attuale, viene adeguato per impostazione standard al rincaro; l’opzione «Adeguare l’affitto all’inflazione» lo disattiva).
- **Ipoteca:** tranche con periodi, vedi [Ipoteche](#f-re-mortgage).
- [Vendita forzata](#f-re-emergency) e [sostenibilità](#f-re-afford) per acquisti «dopo la vendita».

Il [costo della vita](#s-flows) lo inserisce **incluse** le spese d’alloggio. Il calcolo usa il costo della vita **senza la quota d’alloggio**; affitto, interessi ipotecari, ammortamento e spese accessorie si aggiungono come **spese d’alloggio** simulate. Dopo la vendita dell’ultimo immobile, nel mese successivo viene acquistato il prossimo con acquisto «dopo la vendita», se è sostenibile e i fondi propri bastano. Altrimenti iniziano gli affitti ‹da dopo la vendita›. Un acquisto «dopo la vendita» viene tentato **una volta** nel mese successivo. Se non è finanziabile, **viene soppresso** con un messaggio negli eventi, e le regole successive intervengono il mese seguente: iniziano gli affitti ‹da dopo la vendita› di questo immobile, vengono tentati ulteriori acquisti «dopo la vendita» di esso. Una **data d’acquisto o di vendita anteriore all’inizio della simulazione** non viene mai raggiunta; la verifica la segnala come avvertenza.

## @f-re-bought Data d’acquisto
Parole chiave: durata di possesso, imposta sugli utili immobiliari, acquisto

Il mese in cui ha acquistato l’immobile. Determina la **durata di possesso** e quindi la maggiorazione o riduzione dell’[imposta sugli utili immobiliari](#f-tax-gain): più a lungo possiede l’immobile, meno imposta è dovuta in molti cantoni.

- Il campo esiste solo per gli immobili in proprietà. Per un immobile **pianificato** conta il mese d’acquisto della simulazione.
- Senza data d’acquisto l’app calcola **senza** maggiorazione o riduzione (l’imposta secondo la tariffa di base). Se sono registrate regole sulla durata di possesso, la verifica lo segnala.
- Determinante è la durata di possesso in **anni interi** fino al mese della vendita.

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

Una vendita forzata mostra che la pianificazione non funziona senza vendita immobiliare. Appare negli [eventi](#r-events) dei risultati. Più alta è la soglia, prima si vende.

## @f-re-afford Sostenibilità
Parole chiave: banca, sostenibilità, grado d’anticipo, tasso d’interesse teorico, fondi propri, ipoteca

Il calcolo della sostenibilità delle banche, applicato agli acquisti «dopo la vendita» e a ogni rinnovo di un periodo ipotecario. Un’ipoteca è sostenibile se i **costi teorici** non superano una quota del reddito.

- **Quota massima del reddito:** di solito un terzo (33 %).
- **Tasso d’interesse teorico:** di solito il 5 %, indipendentemente dall’interesse attuale.
- **Spese accessorie:** di solito l’1 % del prezzo all’anno.
- **Reddito:** sono computati solo il reddito da attività lucrativa e le rendite, **non** il prelievo dalla sostanza. Se il reddito non basta, l’ipoteca viene ridotta; l’acquisto mostra allora il **rimborso unico** dalla sostanza come evento.
- **Grado d’anticipo massimo:** di solito l’80 % del prezzo. Almeno il 20 % sono fondi propri.

Se l’acquisto non è sostenibile, l’ipoteca viene ridotta. Se anche allora i fondi propri non bastano, non si acquista e iniziano gli affitti.

**A ogni rinnovo** di un periodo ipotecario (anche per immobili in proprietà) l’app verifica di nuovo la sostenibilità dell’intera ipoteca, con il reddito del mese del rinnovo (attività lucrativa e rendite). Se l’ipoteca non è più sostenibile, la simulazione calcola il **pagamento unico minimo necessario** perché lo sia di nuovo, e lo paga dalla sostanza (in proporzione su tutte le tranche). Ciò vale solo se poi la sostanza **non è negativa** **e** il [criterio di vendita forzata](#f-re-emergency) dopo il pagamento **non** scatta (stimato con le spese pianificate e le spese d’alloggio dopo il pagamento). Allora l’ipoteca viene rinnovata con l’importo ridotto. Altrimenti **non vi è alcun pagamento**, l’ipoteca **non viene rinnovata**, e viene avviata una **vendita forzata** dell’immobile (eseguita nel mese successivo). La vista dei risultati registra ogni rinnovo (data ed età, sostenibilità, pagamento unico, nuova ipoteca, vendita forzata con motivo «sostanza < 0» o «criterio di vendita forzata») e conta con più simulazioni in quante simulazioni avvengono pagamenti unici e vendite forzate, anche per percentile. Caso tipico: il reddito diminuisce con il pensionamento.

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
