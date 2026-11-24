<p align="center"><a href="README.md">Deutsch</a> · <a href="README.en.md">English</a> · <a href="README.fr.md">Français</a> · <b>Italiano</b></p>

<h1 align="center">VivaPlan</h1>

<p align="center">
  <b>Il denaro basterà – e con quale sicurezza?</b><br>
  Pianificazione della previdenza per la Svizzera. Mese per mese, migliaia di andamenti di borsa, tutto sul Suo computer.
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/Licenza-GPL%20v3-blue" alt="Licenza GPL v3"></a>
  <img src="https://img.shields.io/badge/macOS%20%C2%B7%20Windows%20%C2%B7%20Linux-lightgrey" alt="macOS, Windows, Linux">
  <a href="https://github.com/ingen63/VivaPlan/actions/workflows/test-quality.yml"><img src="https://github.com/ingen63/VivaPlan/actions/workflows/test-quality.yml/badge.svg" alt="Test quality"></a>
</p>

<p align="center">
  <img src="docs/bilder/uebersicht-it.png" alt="VivaPlan: la pianificazione di Marco e Anna Rossi dei video con linea del tempo, dati e anteprima in tempo reale dell’andamento della sostanza" width="900">
</p>

https://github.com/user-attachments/assets/a36a4b34-f958-4ac2-89be-6eb055769b93

<p align="center">
  ▶ <a href="https://archive.org/details/vivaplan-video-it/vivaplan-2min.mp4">VivaPlan in 2 minuti</a> ·
  ▶ <a href="https://archive.org/details/vivaplan-video-it/vivaplan-10min.mp4">VivaPlan in 10 minuti</a>
</p>

<p align="center"><sub>Lo screenshot e i video mostrano la stessa pianificazione d’esempio con dati inventati.</sub></p>

## Di che cosa si tratta

VivaPlan calcola il Suo futuro finanziario da oggi fino alla fine della pianificazione, con le particolarità svizzere:
AVS, cassa pensione con aliquota di conversione e prelievo del capitale, pilastro 3a, imposta sul reddito, imposta
sulla sostanza, imposta sulle prestazioni in capitale e imposta sugli utili immobiliari, ipoteche SARON e calcolo della
sostenibilità delle banche.

Poiché nessuno conosce la borsa, VivaPlan non calcola un solo andamento, ma migliaia: ogni mese di partenza storico
oppure percorsi Monte Carlo. Alla fine c’è un numero chiaro che indica in quanti casi il denaro basta, con accanto il
caso peggiore.

## Che cosa sa fare VivaPlan

- 🧮 **Mese per mese:** redditi, previdenza, imposte, immobili e ipoteche in un unico calcolo
- 📈 **120 anni di storia della borsa:** mondo (11 paesi, 1900–2020), Svizzera, S&P 500 dal 1872, tutto in CHF
- 🎲 **Monte Carlo:** bootstrap da blocchi storici oppure un modello azionario con salti di corso; 50'000 percorsi in
  pochi secondi, le stesse impostazioni danno sempre gli stessi risultati
- 🪜 **Sei strategie di prelievo:** Standard, 3 comparti (a scelta con guardrail), tasso di prelievo fisso, Guardrails
  (Guyton-Klinger), VPW, Endowment
- 🎯 **Ottimizzazione:** cerca la strategia con il consumo più alto e più regolare con una probabilità di rovina
  limitata e la riprende con un clic
- 🏠 **Immobili:** acquisto, vendita, passaggio da una casa a un appartamento, vendita forzata, imposta sugli utili
  immobiliari
- 👫 **Due persone:** cassa pensione e AVS proprie, regole per i coniugi per l’AVS e le imposte
- 🔀 **Scenari:** «E se andassi in pensione a 63 anni prelevando metà come capitale?» come curve e mappe di calore
- 🕰️ **Che cosa sarebbe successo:** «Come mi sarebbe andata se fossi andato in pensione nel 1973?»
- 🗂️ **Confrontare simulazioni:** ogni simulazione viene salvata con la pianificazione di allora
- ❓ **Aiuto con F1:** per il campo su cui sta lavorando, con un [manuale](crates/app/ui/hilfe/it/00-einstieg.md) e una
  ricerca
- 🌐 **Tedesco, inglese, francese e italiano:** interfaccia, menu, messaggi e aiuto in tutte e quattro le lingue
  (scelta in *Vista → Lingua*); numeri sempre nel formato svizzero. La riga di comando emette testi in inglese.

La descrizione dettagliata si trova in [docs/BESCHREIBUNG.md](docs/BESCHREIBUNG.md) (in tedesco).

## I Suoi dati restano con Lei

Nessun account, nessun cloud, nessun tracciamento: VivaPlan non accede mai alla rete. Una pianificazione è un unico
file (`.vplan`) che Lei salva, copia e archivia come qualsiasi altro documento.

## Installazione

Il pacchetto per macOS (.dmg) si trova in [Releases](https://github.com/ingen63/VivaPlan/releases). Al momento non ci
sono pacchetti per Windows e Linux.

Il pacchetto non è firmato. Al primo avvio il sistema operativo mostra quindi un avviso:

- **macOS:** trascinare l’app nella cartella «Applicazioni» e aprirla. Se appare il messaggio che non può essere
  aperta, fare clic su «Apri comunque» in *Impostazioni di Sistema → Privacy e sicurezza*.

## Primi passi

1. Avviare VivaPlan: il dialogo di benvenuto propone la pianificazione d’esempio, una pianificazione propria o una
   pianificazione vuota.
2. La lista di controllo «Primi passi» guida attraverso gli otto passi dell’avvio rapido e spunta da sola ciò che è
   fatto.
3. Avviare la simulazione: la panoramica dice a parole quanto è sicuro il piano e propone il passo successivo.

> **Nota:** VivaPlan è uno strumento di calcolo e non una consulenza in materia di investimenti, imposte o previdenza.
> I rendimenti passati non sono una previsione.

## Dati e fonti

I rendimenti provengono dalla Jordà-Schularick-Taylor Macrohistory Database (CC BY-NC-SA 4.0), dai dati di
Robert J. Shiller, della Banca nazionale svizzera e dell’Ufficio federale di statistica. Non rientrano nella licenza
del programma. Condizioni e citazioni si trovano in [QUELLEN.md](crates/engine/data/QUELLEN.md) (in tedesco). A causa
della licenza JST, VivaPlan resta gratuito.

## Sviluppo

Workspace Rust con il nucleo di calcolo (`crates/engine`), la riga di comando (`crates/cli`) e l’applicazione desktop
(Tauri 2, `crates/app`, interfaccia in HTML/JS senza librerie esterne). Oltre 600 test Rust, test dell’interfaccia con
Playwright, copertura di almeno l’80 % per pacchetto, test di mutazione e fuzzing.

- Struttura, compilazione, riga di comando, file di progetto e regole di calcolo:
  [docs/ENTWICKLUNG.md](docs/ENTWICKLUNG.md) (in tedesco)
- Strategia di test: [docs/TESTS.md](docs/TESTS.md) (in tedesco)

```sh
cargo build --release
target/release/vivaplan run crates/engine/assets/example.vplan --events   # output in inglese (#176)
make test        # tutti i test Rust con nextest (non cargo test; motivo in docs/TESTS.md)
```

Errori e desideri sono benvenuti come [issue](https://github.com/ingen63/VivaPlan/issues).

## Sostenere

VivaPlan è gratuito. Se trova utile il progetto, può sostenerlo liberamente con un caffè:
[buymeacoffee.com/vivaplan](https://buymeacoffee.com/vivaplan).

## Licenza

Codice del programma: [GPL v3](LICENSE). Le serie temporali fornite non vi rientrano; i diritti sui dati di base
appartengono alle fonti (Robert J. Shiller, BNS, Ufficio federale di statistica, Jordà-Schularick-Taylor), vedi
[QUELLEN.md](crates/engine/data/QUELLEN.md).
