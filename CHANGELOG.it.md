# Registro delle modifiche

Traduzione italiana di `CHANGELOG.md` (tedesco, versione di riferimento) a partire dalla versione 1.0.0-beta.1.

## Versione 1.0.0-beta.6

Sesta versione preliminare di 1.0. Il pacchetto d'installazione Windows `.msi` porta il numero di versione 1.0.0.6; i pacchetti d'installazione sono disponibili per macOS, Windows e Linux, costruiti a mano per tutte le piattaforme prima del tag (#262).

### Può modificare i risultati

- **Quota abitativa del costo della vita (#264):** La quota abitativa dedotta comprende affitto, interessi e spese accessorie, non più l'ammortamento. Le pianificazioni più vecchie vengono adattate all'apertura (con un avviso); il loro risultato resta uguale.
- **Percentili (#265):** I risultati si aprono nel potere d'acquisto odierno; i percentili sono ordinati secondo la rappresentazione scelta (nominale o reale).
- **Reddito imponibile del patrimonio (#149):** Un tasso annuo per azioni e obbligazioni conta nel reddito imponibile. Le nuove pianificazioni iniziano con 2 % e 1 %, quelle esistenti con 0 %.

### Nuovo, modificato, corretto

- **Token beta (#270):** L'app e il pacchetto IA si attivano una volta con un token beta.
- **Casa di cura (#271):** Elemento proprio con età d'entrata, costi e vendita dell'abitazione; avviso se dopo la vendita dell'ultimo immobile non restano costi d'abitazione.
- **Imposte (#272–#275):** Le nuove pianificazioni calcolano le imposte al domicilio; la sezione mostra prima comune e confessione, le altre indicazioni come ipotesi, i dettagli sotto «Esperto». Le deduzioni si possono riprendere dall'ultima tassazione, il valore fiscale degli immobili ha un valore predefinito in percento del valore venale, un riscatto nella cassa pensione si registra direttamente come deduzione.
- **Deduzioni dal patrimonio (#249)** presentate come le deduzioni dal reddito.
- **Sostenibilità (#266):** A scelta con consumo del patrimonio e anticipo più basso dal pensionamento.
- **Collegamento IA (#263, #269):** Il server MCP apre e salva le pianificazioni solo nella cartella autorizzata, a scelta in sola lettura; una nuova pianificazione inizia vuota e l'IA chiede le indicazioni mancanti.
- **README (#267, #277)** in quattro lingue: l'app è al 100 % locale, solo il collegamento IA invia dati all'IA scelta.
- **Visualizzazione (#220):** Scelta «Aspetto» nel dialogo delle impostazioni con lo sfondo del tema.

### Test e sviluppo

- Il pacchetto Linux contiene di nuovo `vivaplan` (#261), memoria dei grandi calcoli ridotta (#249), regola Clippy (#260), misurazione dell'influsso delle imposte per altre economie domestiche (#276).

## Versione 1.0.0-beta.5

Quinta versione preliminare di 1.0. Il pacchetto d'installazione Windows `.msi` porta il numero di versione 1.0.0.5; i pacchetti d'installazione sono di nuovo disponibili per macOS, Windows e Linux (#203).

### Può modificare i risultati

- **Senza cassa pensione (#253):** L'interruttore si trova ora nella scheda «Cassa pensione» della sezione Previdenza. Se attivo, la cassa pensione di questa persona vale 0 (nessun avere, nessun contributo, nessuna rendita, nessun prelievo di capitale); i valori inseriti restano salvati, i campi sono bloccati.
- **Ottimizzazione (#248):** Per tutte le forme di andamento, l'andamento prima del pensionamento resta invariato; il valore ottimizzato vale dal pensionamento.

### Novità, modifiche, correzioni

- **Usare VivaPlan con un'IA (#208–#214, #216, #219):** Il server MCP `vivaplan mcp` apre, modifica, calcola e confronta pianificazioni per Claude, ChatGPT, LM Studio e altre IA, con avanzamento, interruzione e conferma dei calcoli lunghi. Il menu «Aiuto» › «Collegare a un’IA» mostra la configurazione; per Claude Desktop esiste un pacchetto con un clic `.mcpb` (#213). I dati vanno solo all'IA che collega Lei stesso.
- **Forme di andamento e curve caratteristiche (#217, #218, #243):** I valori possono seguire una retta, una tabella o un fattore e si modificano nell'editor; le curve caratteristiche rendono i parametri dipendenti dallo stato.
- **Riga di comando nel pacchetto d'installazione (#206):** `vivaplan` usa la stessa cartella dei dati dell'app.
- **Visualizzazione (#244, #255, #257, #259):** I campi numerici e la scelta «Valore fiscale» non vengono più troncati in nessuna lingua.

### Test e sviluppo

- Test dell'interfaccia aggiornati (#246, #247), limite di tempo nel test MCP indipendente dalla velocità del computer (#258).

## Versione 1.0.0-beta.4

Quarta versione preliminare di 1.0; contiene anche tutte le modifiche della 1.0.0-beta.3 non pubblicata. Il pacchetto di installazione Windows `.msi` porta il numero di versione 1.0.0.4; per ora i pacchetti di installazione vengono creati solo per macOS (#202).

### Può modificare i risultati

- **Coniugati con una sola persona (#250):** «Coniugati» vale anche senza seconda persona; le imposte sono allora calcolate con le tariffe e le deduzioni per coniugati. Le pianificazioni meno recenti con una persona vengono riprese come non coniugate all'apertura e calcolano come prima (avvertenza); le nuove pianificazioni iniziano non coniugate.
- **Imposte secondo cantone e comune (#63, #64, #65, #66):** Nella modalità «Domicilio», l'imposta sul reddito, sulla sostanza e sulle prestazioni in capitale segue cantone e comune, l'imposta sugli utili immobiliari il cantone dell'immobile. Le tariffe crescono con il rincaro, la deduzione per doppio reddito vale per le persone che esercitano un'attività lucrativa.
- **Deduzioni e domicilio (#131, #151, #152, #228–#232):** Deduzioni dal reddito secondo la proposta del cantone o con righe proprie, deduzioni supplementari e uniche, deduzioni dalla sostanza, valore fiscale per immobile, reddito da attività lucrativa come importo annuo, domicilio variabile nel tempo e grande pilastro 3a per le persone senza cassa pensione.
- **AVS (#190, #191, #198):** La 13a rendita AVS è versata in dicembre e conta nel fabbisogno netto della strategia dei 3 vasi. Anticipo e rinvio riducono o aumentano la rendita; il plafond per i coniugi è conforme alla legge.
- **Immobili e strategie (#199, #204, #239):** I rinnovi non sostenibili con «Non vendere mai gli immobili» vengono contati; i limiti delle spese di sostentamento valgono per VPW ed Endowment anche nel confronto e nell'ottimizzazione; i guardrail senza sostanza liquida non adeguano nulla.
- **Reddito da attività lucrativa (#236):** Su richiesta, il reddito da attività lucrativa termina con il prelievo della cassa pensione.

### Novità, modifiche, correzioni

- **Sostanza imponibile nella scheda «Deduzioni dalla sostanza» (#249):** Avvertenza con la sostanza imponibile stimata del primo anno fiscale e la quota degli immobili, avviso se manca un valore fiscale; i dettagli la compongono per anno fiscale come la dichiarazione d'imposta e offrono il campo «Valore fiscale» per immobile.
- **Aiuto all'inserimento dalla tassazione (#251):** Nuovo campo «di cui pilastro 3a secondo la tassazione», affinché il pilastro 3a non venga dedotto due volte con «Dedurre i versamenti nel pilastro 3a»; un'avvertenza appare finché il campo è vuoto.
- **Confrontare strategie (#196, #197, #200, #201, #205):** Le strategie si confrontano senza ottimizzazione, i risultati dell'ottimizzazione si salvano e si riaprono; opzione «Non vendere mai gli immobili»; indicazione quando il limite inferiore determina le spese; tabella dei risultati senza scorrimento orizzontale.
- **Tema scuro (#220, #235):** Impostazione «Aspetto» con tema chiaro e scuro; contrasto migliore nel menu.
- **Utilizzo (#143, #227, #233, #234, #241):** Imposte indicate come stima, pianificazioni aperte di recente nel menu «File», sostenibilità nelle impostazioni, messaggi del motore di calcolo leggibili, scheda «Deduzioni dal reddito» più chiara con un testo per la deduzione per doppio reddito.
- **Video introduttivi (#240, #242):** «VivaPlan in 2 minuti» e «in 10 minuti» in quattro lingue, dal menu «Aiuto», dalla finestra di benvenuto e dall'aiuto; aprono il browser.
- **Sostenere (#237):** Link facoltativo per una donazione «Buy me a coffee» nel menu «Aiuto», nella navigazione e sotto «Informazioni su VivaPlan»; apre il browser.
- **Riga di comando per le IA (#207, #215):** Output JSON e modifica dei valori tramite percorso del campo; i percorsi di rendimento sono riutilizzati per più varianti (più veloce).

### Test e sviluppo

- Cifre di riferimento confrontate nel push (#195), variazione del valore degli immobili ricalcolata in modo indipendente (#194), tracciabilità dei requisiti dei test estesa (#192).
- Test di robustezza di tutti i comandi di calcolo (#221), risultati dei test di proprietà come test di regressione (#223), test Rust sotto Windows (#225), test saltati contati come errori (#224), limite di durata per test (#226), regole per l'agente di test (#222).
- Note di rilascio in quattro lingue con titolo neutro (#188); mutants-diff settimanale (#202).

## Versione 1.0.0-beta.2

Seconda versione preliminare di 1.0. Il pacchetto d'installazione Windows `.msi` porta il numero di versione 1.0.0.2.

### Può modificare i risultati

- **Imposta sulle prestazioni in capitale delle coppie non sposate (#150):** I prelievi di capitale di partner non sposati sono tassati per persona e non più sommati per l'economia domestica. Ciò può modificare i risultati di tali pianificazioni.

### Novità, modifiche, correzioni

- **Quattro lingue (#170–#177):** Interfaccia, menu, finestre dei file, messaggi e aiuto sono disponibili in tedesco, inglese, francese e italiano. Al primo avvio vale la lingua del sistema, poi la scelta in «Vista → Lingua». Numeri e date mantengono il formato svizzero in ogni lingua. Anche i nomi del catalogo dell'ottimizzazione e delle serie temporali sono tradotti.
- **Termini e verifica delle traduzioni (#179–#182):** Un glossario fissa i termini ufficiali per lingua (p. es. AVS/LPP, caisse de pension, cassa pensione); una verifica automatica confronta tutte le traduzioni con il tedesco. Tutti i testi trovano posto in ogni lingua, anche con i caratteri di Linux e Windows.
- **Riga di comando in inglese (#176):** `vivaplan` emette messaggi e rapporti in inglese. I numeri sono invariati.
- **Nuova icona (#185):** «Sentiero della vetta – tramonto», con una versione propria per le dimensioni piccole e una favicon coordinata.
- **Pianificazione di esempio (#184):** La pianificazione di esempio si apre nella lingua dell'interfaccia.
- **Aiuto (#183):** Eliminate le contraddizioni su costo della vita, combinazioni e versamento 3a.

### Test e sviluppo

- Codice interno in inglese (#165–#169), assenza di rete verificata automaticamente (#157), casi di test comuni del calcolo delle imposte per interfaccia e motore di calcolo (#155, #156, #158).
- File di esempio con nomi inglesi e terminazioni di riga fisse (#186), test per mutanti finora non rilevati (#187).
- Note di rilascio in quattro lingue con verifica automatica (#188).

## Versione 1.0.0-beta.1

Prima versione preliminare di 1.0. Il pacchetto d'installazione Windows `.msi` porta il numero di versione 1.0.0.1, perché il formato MSI non conosce versioni preliminari come «beta.1»; gli altri pacchetti si chiamano 1.0.0-beta.1.

### Può modificare i risultati

- **Criterio di rendimento della strategia a 3 comparti (#103):** Ogni percorso di rendimento riporta i rendimenti azionari dei 12 mesi precedenti l'inizio della simulazione. Il criterio di rendimento copre così 12 mesi interi anche nei primi 12 mesi. Ciò può modificare i risultati delle pianificazioni con la strategia a 3 comparti nel primo anno.

### Novità, modifiche, correzioni

- **Confronto dei risultati (#115):** Il confronto dei risultati salvati offre le stesse possibilità di un singolo risultato: nominale o nel potere d'acquisto odierno, tutti i grafici e i valori annuali, con la scelta della simulazione (percentile).
- **Linea di riferimento nei grafici (#114):** La linea a 0 (o a 100 nell'andamento del valore) è chiaramente evidenziata in tutti i grafici dei risultati.
- **Menu (#112):** Un clic nel menu a sinistra o nella lista di controllo non va più perso quando nello stesso momento arriva un'anteprima.

### Test e sviluppo

- Test dell'interfaccia indipendenti dal carico del computer (#111), pagina di sintesi della qualità dei test con i criteri verificati (#113), limite di memoria per i processi di test nei test di mutazione (#116).
