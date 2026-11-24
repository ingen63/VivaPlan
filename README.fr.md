<p align="center"><a href="README.md">Deutsch</a> · <a href="README.en.md">English</a> · <b>Français</b> · <a href="README.it.md">Italiano</a></p>

<h1 align="center">VivaPlan</h1>

<p align="center">
  <b>L’argent suffira-t-il – et avec quelle sécurité?</b><br>
  Planification de la retraite pour la Suisse. Mois par mois, des milliers d’évolutions boursières, tout sur votre ordinateur.
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/Licence-GPL%20v3-blue" alt="Licence GPL v3"></a>
  <img src="https://img.shields.io/badge/macOS%20%C2%B7%20Windows%20%C2%B7%20Linux-lightgrey" alt="macOS, Windows, Linux">
  <a href="https://github.com/ingen63/VivaPlan/actions/workflows/test-quality.yml"><img src="https://github.com/ingen63/VivaPlan/actions/workflows/test-quality.yml/badge.svg" alt="Test quality"></a>
</p>

<p align="center">
  <img src="docs/bilder/uebersicht-fr.png" alt="VivaPlan: la planification de Jean et Anne Dubois des vidéos avec ligne du temps, saisies et aperçu en direct de l’évolution de la fortune" width="900">
</p>

https://github.com/user-attachments/assets/e9e3b9d8-debe-462f-b199-298ddb8b8a71

<p align="center">
  ▶ <a href="https://archive.org/details/vivaplan-video-fr/vivaplan-2min.mp4">VivaPlan en 2 minutes</a> ·
  ▶ <a href="https://archive.org/details/vivaplan-video-fr/vivaplan-10min.mp4">VivaPlan en 10 minutes</a>
</p>

<p align="center"><sub>La capture d’écran et les vidéos montrent la même planification d’exemple avec des données fictives.</sub></p>

## De quoi il s’agit

VivaPlan calcule votre avenir financier d’aujourd’hui jusqu’à la fin de la planification, avec les particularités
suisses: AVS, caisse de pension avec taux de conversion et retrait en capital, pilier 3a, impôt sur le revenu, impôt
sur la fortune, impôt sur les prestations en capital et impôt sur les gains immobiliers, hypothèques SARON et calcul de
la capacité financière des banques.

Comme personne ne connaît la bourse, VivaPlan ne calcule pas une seule évolution, mais des milliers: chaque mois de
départ historique ou des trajectoires Monte-Carlo. Au final, un chiffre clair indique dans combien de cas l’argent
suffit, avec le pire cas à côté.

## Ce que VivaPlan sait faire

- 🧮 **Mois par mois:** revenus, prévoyance, impôts, biens immobiliers et hypothèques dans un seul calcul
- 📈 **120 ans d’histoire boursière:** monde (11 pays, 1900–2020), Suisse, S&P 500 depuis 1872, tout en CHF
- 🎲 **Monte-Carlo:** bootstrap à partir de blocs historiques ou modèle d’actions avec sauts de cours; 50'000
  trajectoires en quelques secondes, les mêmes réglages donnent toujours les mêmes résultats
- 🪜 **Six stratégies de prélèvement:** Standard, 3 compartiments (au choix avec garde-fous), taux de prélèvement fixe,
  Guardrails (Guyton-Klinger), VPW, Endowment
- 🎯 **Optimisation:** cherche la stratégie avec la consommation la plus élevée et la plus régulière pour une
  probabilité de ruine limitée et la reprend en un clic
- 🏠 **Immobilier:** achat, vente, passage d’une maison à un appartement, vente forcée, impôt sur les gains immobiliers
- 👫 **Deux personnes:** caisse de pension et AVS propres, règles pour les couples mariés pour l’AVS et les impôts
- 🔀 **Scénarios:** «Et si je partais à 63 ans en retirant la moitié en capital?» sous forme de courbes et de cartes de
  chaleur
- 🕰️ **Ce qui se serait passé:** «Comment m’en serais-je sorti si j’avais pris ma retraite en 1973?»
- 🗂️ **Comparer des simulations:** chaque simulation est enregistrée avec la planification de l’époque
- ❓ **Aide avec F1:** pour le champ sur lequel vous travaillez, avec un [manuel](crates/app/ui/hilfe/fr/00-einstieg.md)
  et une recherche
- 🌐 **Allemand, anglais, français et italien:** interface, menus, messages et aide dans les quatre langues (choix sous
  *Affichage → Langue*); nombres toujours au format suisse. La ligne de commande affiche des textes en anglais.

La description détaillée se trouve dans [docs/BESCHREIBUNG.md](docs/BESCHREIBUNG.md) (en allemand).

## Vos données restent chez vous

Pas de compte, pas de cloud, pas de suivi: VivaPlan n’accède jamais au réseau. Une planification est un seul fichier
(`.vplan`) que vous enregistrez, copiez et archivez comme tout autre document.

## Installation

Le paquet pour macOS (.dmg) se trouve sous [Releases](https://github.com/ingen63/VivaPlan/releases). Il n’existe
actuellement pas de paquets pour Windows et Linux.

Le paquet n’est pas signé. Au premier démarrage, le système d’exploitation affiche donc un avertissement:

- **macOS:** glisser l’app dans le dossier «Applications» et l’ouvrir. Si un message indique qu’elle ne peut pas être
  ouverte, cliquer sur «Ouvrir quand même» sous *Réglages Système → Confidentialité et sécurité*.

## Premiers pas

1. Démarrer VivaPlan: le dialogue de bienvenue propose la planification d’exemple, une planification propre ou une
   planification vide.
2. La liste de contrôle «Premiers pas» guide à travers les huit étapes du démarrage rapide et coche elle-même ce qui
   est fait.
3. Lancer la simulation: la vue d’ensemble dit en mots à quel point le plan est sûr et propose l’étape suivante.

> **Remarque:** VivaPlan est un outil de calcul et non un conseil en placement, en impôts ou en prévoyance. Les
> rendements passés ne sont pas une prévision.

## Données et sources

Les rendements proviennent de la Jordà-Schularick-Taylor Macrohistory Database (CC BY-NC-SA 4.0), des données de
Robert J. Shiller, de la Banque nationale suisse et de l’Office fédéral de la statistique. Ils ne relèvent pas de la
licence du programme. Les conditions et les citations figurent dans [QUELLEN.md](crates/engine/data/QUELLEN.md) (en
allemand). En raison de la licence JST, VivaPlan reste gratuit.

## Développement

Workspace Rust avec le noyau de calcul (`crates/engine`), la ligne de commande (`crates/cli`) et l’application de
bureau (Tauri 2, `crates/app`, interface en HTML/JS sans bibliothèques externes). Plus de 600 tests Rust, tests de
l’interface avec Playwright, couverture d’au moins 80 % par paquet, tests de mutation et fuzzing.

- Structure, compilation, ligne de commande, fichier de projet et règles de calcul:
  [docs/ENTWICKLUNG.md](docs/ENTWICKLUNG.md) (en allemand)
- Stratégie de test: [docs/TESTS.md](docs/TESTS.md) (en allemand)

```sh
cargo build --release
target/release/vivaplan run crates/engine/assets/example.vplan --events   # sortie en anglais (#176)
make test        # tous les tests Rust avec nextest (pas cargo test; raison dans docs/TESTS.md)
```

Erreurs et souhaits volontiers sous forme d’[issue](https://github.com/ingen63/VivaPlan/issues).

## Soutenir

VivaPlan est gratuit. Si vous trouvez le projet utile, vous pouvez le soutenir librement avec un café:
[buymeacoffee.com/vivaplan](https://buymeacoffee.com/vivaplan).

## Licence

Code du programme: [GPL v3](LICENSE). Les séries temporelles fournies n’en relèvent pas; les droits sur les données de
base appartiennent aux sources (Robert J. Shiller, BNS, Office fédéral de la statistique, Jordà-Schularick-Taylor),
voir [QUELLEN.md](crates/engine/data/QUELLEN.md).
