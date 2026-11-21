# Registro delle modifiche

Traduzione italiana di `CHANGELOG.md` (tedesco, versione di riferimento) a partire dalla versione 1.0.0-beta.1.

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
