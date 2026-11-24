# Introduzione

## @start Che cosa sa fare VivaPlan
Parole chiave: panoramica, scopo, pianificazione della previdenza, simulazione

VivaPlan risponde alla domanda che si pone prima del pensionamento: **il denaro basta, e con quale sicurezza?**

> Video: [VivaPlan in 2 minuti](video:2min) mostra la panoramica (apre il browser).

L’app calcola il Suo futuro finanziario mese per mese, da oggi fino alla fine della pianificazione. Conosce le particolarità svizzere:

- cassa pensione (2° pilastro) con aliquota di conversione e prelievo del capitale, AVS (1° pilastro) e pilastro 3a
- imposta sul reddito, sulla sostanza, sulle prestazioni in capitale e sugli utili immobiliari
- abitazione propria con ipoteca SARON o fissa, calcolo della sostenibilità delle banche, affitto
- una seconda persona con previdenza propria (regole per coppie sposate)

Il futuro è incerto, soprattutto i rendimenti di borsa. Perciò l’app non calcola un solo andamento, ma a seconda dell’impostazione **migliaia** (Monte Carlo o tutti i momenti d’inizio storici) e mostra in quanti di essi il denaro basta. Come nascono i rendimenti è il tema più importante dell’app: vedi [Rendimenti e dati](#s-returns).

La struttura:

1. **Editor:** registra la pianificazione in undici sezioni (a sinistra). A destra vede subito un’anteprima dell’andamento della sostanza.
2. **Avviare la simulazione:** l’app calcola tutte le simulazioni e mostra i [risultati](#r-overview).
3. **Ottimizzazione:** l’app cerca la [strategia di prelievo](#s-optimize) adatta ai Suoi obiettivi.
4. **Risultati:** ogni simulazione viene archiviata come risultato nell’[elenco](#r-list), affinché possa visualizzarla e confrontarla in seguito.

## @quickstart La prima pianificazione in dieci minuti
Parole chiave: istruzioni, inizio, tutorial, passi

> Video: [VivaPlan in 10 minuti](video:10min) guida attraverso la prima pianificazione (apre il browser).

Inizi con la pianificazione d’esempio caricata all’apertura e la modifichi passo dopo passo secondo la Sua situazione.

La lista di controllo **«Primi passi»** a destra sopra la verifica guida attraverso gli stessi otto passi. Un clic su un passo apre la sezione, ⓘ il relativo aiuto. Una sezione è considerata compiuta non appena vi modifica un valore o la spunta a mano; «Annullare» non toglie la spunta. «Avviare la simulazione» è compiuto non appena una simulazione è giunta al termine senza interruzione. Con ✕ nasconde l’elenco, con *Aiuto → Primi passi* lo mostra di nuovo. Quando tutti i passi sono compiuti, si chiude. L’avanzamento vale per Lei su questo dispositivo, non per una singola pianificazione, e non figura nel file di pianificazione.

1. **[Economia domestica e progetto](#s-general):** inserire nome e mese di nascita.
2. **[Tappe](#s-milestones):** stabilire prestazione della cassa pensione, inizio della rendita AVS, inizio e fine della simulazione. La fine dovrebbe essere fissata così tardi da non doverla verosimilmente superare in vita (ad esempio a 95 anni).
3. **[Previdenza](#s-pension):** inserire l’avere attuale della cassa pensione, l’aliquota di conversione, il prelievo del capitale e la rendita AVS dai Suoi certificati di previdenza.
4. **[Entrate e spese](#s-flows):** reddito da attività lucrativa all’anno (come nella dichiarazione d’imposta) e costo della vita al mese. Il costo della vita richiede un valore realistico: qui sta la leva più grande.
5. **[Sostanza e investimento](#s-wealth):** azioni, obbligazioni e cash all’inizio della simulazione, più la quota azionaria perseguita.
6. **[Simulazione](#s-returns):** per una prima valutazione scegliere «Monte Carlo · storico». Gli indici (ad esempio «Mondo (JST)») si stabiliscono nella [strategia d’investimento](#f-ret-series) sotto «Sostanza e investimento».
7. **[Imposte](#s-taxes):** sostituire le aliquote della pianificazione d’esempio con quelle del Suo luogo di domicilio (calcolatore fiscale del Suo cantone).
8. **Avviare la simulazione** (pulsante in alto a destra o ⌘↩). A sinistra del pulsante figura una stima approssimativa della **durata**. Durante il calcolo una **barra di avanzamento** mostra lo stato e il tempo residuo stimato; con **Interrompere** ferma il calcolo e torna alla pianificazione.

La verifica in basso a destra mostra errori e avvertenze ancora prima del calcolo. Gli errori impediscono la simulazione, le avvertenze no.

> Consiglio: prema **F1** in un campo d’inserimento e appare esattamente l’aiuto relativo a quel campo.

## @bedienung Utilizzo e scelte rapide da tastiera
Parole chiave: menu, tastiera, scorciatoie, salvare, aprire, annullare, F1, lingua, aperti di recente, sostenere, donazione, caffè, video, video introduttivo

- **File:** Nuova (una **pianificazione vuota con valori standard**: inizio nel mese attuale, senza nome del progetto, una persona senza nome, nessun importo e nessun immobile; rendimenti, tabelle delle aliquote e parametri delle strategie adottano i valori standard), Aprire, Aperti di recente, Salvare, Salvare con nome. Una pianificazione è un singolo file con l’estensione `.vplan`. Si può copiare e archiviare.
- **Aperti di recente:** Il sottomenu sotto File → Aprire mostra fino a 10 pianificazioni aperte o salvate per ultime, la più recente in alto (nome del file senza `.vplan`; con la cartella se i nomi sono uguali). Un clic apre la pianificazione come «Aprire…»; se ci sono modifiche non salvate, l’app chiede prima. Se il file non esiste più, l’app lo segnala e rimuove la voce. «Cancella elenco» svuota l’elenco; se è vuoto, mostra «Nessuna voce».
- **Avvio e chiusura:** all’avvio l’app desktop apre la pianificazione che ha aperto o salvato per ultima; il nome del file figura nell’intestazione. Se non ve n’è una, se il file è scomparso o non è leggibile, apre la **pianificazione d’esempio** fornita (con avvertenza sul nome del file, tranne al primissimo avvio). La pianificazione d’esempio è considerata una pianificazione nuova, non salvata: «Salvare» chiede un percorso, non viene mai sovrascritta. Uscendo con modifiche non salvate (pulsante di chiusura della finestra, ⌘Q o Ctrl+Q oppure File → Uscire) l’app chiede se desidera salvare, scartare o interrompere. «Interrompere» lascia l’app aperta; se la richiesta del percorso al salvataggio viene interrotta, resta anch’essa aperta. Viene memorizzato solo il percorso del file, in locale nella cartella dei dati dell’app.
- **Salvare prima della simulazione:** se la pianificazione ha modifiche non salvate, l’app desktop chiede prima di ogni simulazione se desidera salvare: «Salvare» (per una nuova pianificazione con richiesta del percorso), «Simulare senza salvare» o «Interrompere». La domanda si può disattivare nelle impostazioni.
- **Benvenuto:** al primissimo avvio l’app chiede come desidera iniziare: **Esplorare l’esempio** (carica la pianificazione d’esempio e apre l’aiuto [Che cosa sa fare VivaPlan](#start)), **Iniziare una pianificazione propria** (carica la pianificazione d’esempio come modello, apre «Economia domestica e progetto» e azzera la lista di controllo [Primi passi](#quickstart)) o **Pianificazione vuota** (come File → Nuova). Esc chiude il dialogo senza modifiche. In seguito non appare più da sé; *Aiuto → Benvenuto …* lo apre in qualsiasi momento, con domanda di conferma se vi sono modifiche non salvate. Se l’app viene avviata con un file (doppio clic), non appare.
- **Annullare / Ripristinare:** ⌘Z e ⇧⌘Z (con Windows e Linux Ctrl+Z e Ctrl+Shift+Z) annullano gli inserimenti.
- **Cambiare sezione:** da ⌘1 a ⌘9 e ⌘0 saltano alle sezioni, ⌘↑ e ⌘↓ vanno avanti o indietro di una. Nei risultati gli stessi tasti cambiano la scheda.
- **Aiuto:** **F1** apre l’aiuto sul tema attuale (a seconda di dove si trova: campo, sezione, grafico dei risultati). Il pulsante «? Aiuto» e i segni ⓘ accanto ai campi fanno lo stesso. ⌘⇧? apre l’indice, Esc chiude l’aiuto.
- **Linea del tempo:** sopra l’editor mostra tutte le tappe. Un clic su di essa apre le tappe. Sotto la linea figurano gli **eventi immobiliari** dell’anteprima (triangoli verdi per acquisto e vendita, arancioni per le vendite forzate) con una breve didascalia come ‹Acquisto Wohnung 09.2027›. Con più di 15 eventi appaiono solo i simboli, il testo figura allora nel tooltip.
- **Valori dipendenti dal tempo:** molti importi si possono registrare come costante o come sequenza di valori, vedi [Valori dipendenti dal tempo](#f-timed).
- **Lingua:** menu *Vista → Lingua* o le [Impostazioni](#f-settings). Il cambio ha effetto subito; pianificazione e vista restano. L’aiuto appare nella lingua scelta; se un capitolo non è ancora tradotto, appare in tedesco.
- **Video introduttivi:** *Aiuto → Video introduttivi …* mostra i video «VivaPlan in 2 minuti» e «VivaPlan in 10 minuti»; la finestra di benvenuto contiene il link «Guardare VivaPlan in 2 minuti». Un clic apre il video nella lingua impostata nel Suo browser (Internet Archive).
- **Collegare a un’IA:** *Aiuto → Collegare a un’IA …* mostra come configurare VivaPlan in Claude Desktop, Claude Code, l’app desktop di ChatGPT o LM Studio, ogni volta con il luogo della configurazione e una riga da copiare ([collegare programmi di IA](#ref-mcp)). VivaPlan non configura nulla da sé.
- **Sostenere VivaPlan:** VivaPlan è gratuito. Se desidera sostenere liberamente l’autore offrendogli un caffè: *Aiuto → Sostenere VivaPlan…*, la riga «☕ Sostenere VivaPlan» in fondo alla navigazione o il link nella finestra «Informazioni su VivaPlan» aprono la pagina buymeacoffee.com/vivaplan nel Suo browser. L’app stessa non carica nulla da Internet.

L’elenco completo delle scelte rapide da tastiera si trova nel menu *Aiuto → Scelte rapide da tastiera* e nel [capitolo di riferimento](#ref-shortcuts).

## @privacy Protezione dei dati e dati locali
Parole chiave: Internet, cloud, luogo di salvataggio, sicurezza, rete, protezione dei dati

Tutti i dati restano sul Suo computer. L’app non si collega mai a Internet, nemmeno per i dati, gli aggiornamenti o le statistiche. Solo due tipi di link aprono il Suo browser: i [video introduttivi](#start) e il link per le donazioni «Sostenere VivaPlan» ([uso](#bedienung)). Gli indirizzi sono fissati nell’app; l’app stessa non carica nulla.

- Le **pianificazioni** si trovano dove salva il file `.vplan`. I percorsi delle ultime pianificazioni aperte o salvate (elenco «Aperti di recente») figurano nel file `einstellungen.json` nella cartella dei dati dell’app.
- Le **serie temporali** (la biblioteca di dati) si trovano nella cartella dei dati dell’app. Con macOS è `~/Library/Application Support/ch.vivaplan-app.desktop/Zeitreihen`.
- **L’elenco dei risultati** si trova nella memoria locale dell’app (non nel file di pianificazione). Chi dispone del Suo account utente su questo computer può leggerlo. Elimini le voci che non desidera conservare.
- **Programmi di IA:** se collega VivaPlan a un’IA ([collegare programmi di IA](#ref-mcp)), l’IA legge dati della Sua pianificazione tramite il programma `vivaplan mcp`. Con un’IA nel cloud (per esempio Claude o ChatGPT) questi dati vanno al suo fornitore. Con un modello locale (per esempio in LM Studio) restano sul Suo computer. VivaPlan stesso non invia nulla.

I nuovi dati della biblioteca (Shiller, BNS) li scarica Lei stesso dai rispettivi siti web e li importa con i pulsanti nella [strategia d’investimento](#f-ret-series).

## @limits Limiti del modello
Parole chiave: ipotesi, non considerato, consulenza, responsabilità, decesso

VivaPlan è uno **strumento di calcolo**, non una consulenza in investimenti, fiscale o previdenziale. Un risultato vale solo per le ipotesi inserite. Particolarmente importante:

- **Il futuro non è il passato.** I rendimenti storici non sono una previsione. Vedi [Insidie dei rendimenti](#ret-pitfalls).
- **Le imposte sono semplificate:** tabelle delle aliquote, interpolate linearmente, per il moltiplicatore d’imposta. Mancano deduzioni, progressione in dettaglio e particolarità cantonali. I risultati mostrano quindi un’**imposta stimata**, vedi [Quanto è precisa l’imposta?](#tax-accuracy).
- **Non vengono calcolati:** decessi e rendite per superstiti, eredità se non come importo unico, costi di cura (a meno che li registri come spese), modifiche di legge, costi d’investimento e commissioni, interessi sul cash.
- Gli **immobili** sono semplificati: valore venale con il rincaro, ipoteche in tranche, vendita forzata secondo una regola semplice.
- Le **strategie ottimizzate** valgono solo per i dati con cui sono state calcolate, vedi [Insidie dell’ottimizzazione](#o-pitfalls).

Verifichi sempre le decisioni importanti con specialisti. L’[esclusione della responsabilità](#disclaimer) completa si trova nella sezione successiva.

## @disclaimer Esclusione della responsabilità
Parole chiave: responsabilità, garanzia, disclaimer, esclusione di responsabilità, consulenza, diritto, avvertenza

Al primo avvio conferma una versione breve di questa avvertenza. Se il testo cambia, l’app chiede di nuovo. Tutte le pagine dei risultati recano in basso la nota «Calcolo basato su un modello, senza garanzia. Nessuna consulenza fiscale o in investimenti.».

**1. Offerta gratuita.** VivaPlan è messo a disposizione gratuitamente dalla Svizzera. Non sussiste alcun diritto a disponibilità, aggiornamenti, correzione di errori o assistenza.

**2. Nessuna consulenza.** L’app è uno strumento di calcolo per la pianificazione personale. **Non è una consulenza fiscale o in investimenti.** Non offre consulenza in investimenti, finanziaria, fiscale, previdenziale o giuridica e non dà raccomandazioni sull’acquisto o la vendita di strumenti finanziari o immobili.

**3. Calcoli modello senza garanzia.** Tutti i risultati si basano sui Suoi dati e su ipotesi semplificate su rendimento, costi, rincaro, imposte e speranza di vita. Gli sviluppi effettivi possono differire notevolmente. I calcoli fiscali sono stime e non riproducono tutte le regole, deduzioni e modifiche di legge cantonali e comunali. Non si assume alcuna garanzia per l’esattezza, la completezza e l’attualità dei calcoli.

**4. Responsabilità propria.** Le decisioni che prende sulla base dell’app sono di Sua esclusiva responsabilità. Prima di decisioni finanziarie importanti, ad esempio sul prelievo del capitale, sulla vendita della casa o sulla strategia di prelievo, dovrebbe consultare uno specialista qualificato.

**5. Responsabilità.** È esclusa, nella misura consentita dalla legge, la responsabilità per danni di qualsiasi tipo, in particolare per perdite patrimoniali, mancato guadagno, perdita di dati o danni conseguenti derivanti dall’utilizzo o dal mancato utilizzo dell’app. È fatta salva la responsabilità per dolo e colpa grave.

**6. Dati.** L’app salva i Suoi dati in locale sul Suo dispositivo e non trasmette alcun dato (vedi [Protezione dei dati e dati locali](#privacy)). Il backup di questi dati è di Sua responsabilità.

**7. Diritto applicabile.** Si applica il diritto svizzero.

Stato: 22 novembre 2026
