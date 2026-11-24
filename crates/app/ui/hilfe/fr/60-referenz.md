# Référence

## @ref-shortcuts Raccourcis clavier
Mots-clés: clavier, raccourci, Ctrl, commande, F1

Sur Mac, ⌘ (commande) s’applique, sous Windows et Linux **Ctrl**. ⇧ est la touche majuscule.

| Raccourci | Effet |
|---|---|
| **F1** | Aide sur le thème actuel (champ, section ou graphique des résultats) |
| ⌘⇧? | Manuel (table des matières) |
| Échap | Fermer l’aide; sinon retour à l’éditeur |
| ⌘N | Nouvelle planification |
| ⌘O | Ouvrir |
| ⌘S | Enregistrer |
| ⇧⌘S | Enregistrer sous |
| ⌘I | Importer un ancien `config.json` |
| ⌘↩ | Lancer la simulation |
| ⌘Z | Annuler |
| ⇧⌘Z | Rétablir |
| ⌘1 … ⌘9, ⌘0 | Section 1 à 10 dans l’éditeur, l’onglet dans les résultats |
| ⌘↑ et ⌘↓ | section précédente ou suivante |
| ⌘/ | Vue d’ensemble des raccourcis clavier |

Sur certains claviers Mac, F1 se trouve sur la touche de luminosité. Appuyez alors sur **fn+F1** ou utilisez le bouton «? Aide» ou ⌘⇧?.

## @ref-checks Contrôle: erreurs et avertissements
Mots-clés: contrôle, erreur, avertissement, remarque, validation

À droite dans l’éditeur, l’application contrôle la planification en continu.

- **Erreurs (rouge):** la planification est incomplète ou contradictoire. «Lancer la simulation» n’est alors pas possible. Un clic sur l’erreur saute à la section concernée. **«Lancer la simulation» et «Lancer l’optimisation» sont bloqués** tant qu’il y a des erreurs: une tentative affiche «La planification contient N erreur(s) …» et mène à la première section concernée. Les **champs obligatoires** (par exemple le nom et la date de naissance de chaque personne) portent un astérisque (*); une erreur apparaît directement au champ, de même qu’une valeur illisible (par exemple une date vide). Tant que vous tapez dans un champ, **tout attend**: aucun message n’apparaît, et l’aperçu, les valeurs calculées et la liste de contrôle restent en l’état. Ce n’est que lorsque vous quittez le champ (Tab, clic à côté ou Entrée) que tout est recalculé et qu’un message est affiché ou retiré. Une valeur à moitié saisie comme ‹6› en route vers ‹65› n’est ainsi pas signalée. Les listes de sélection et les cases à cocher agissent immédiatement. La liste complète des saisies obligatoires figure dans la spécification `docs/Pflichteingaben.md`.
- **Avertissements (jaune):** la simulation peut être lancée, mais un point mérite votre attention, par exemple une série temporelle dans une autre monnaie ou l’addition de retraits en capital.

Erreurs typiques: départ après la fin, retraite en dehors de la période, plus de deux personnes, une série temporelle manquante pour une source historique, un paramètre non valable d’une stratégie. Le message en indique la cause.

La ligne **Aperçu** à droite calcule une simulation avec des rendements fixes. Elle montre où mènent les saisies, mais ne remplace pas la simulation.

## @ref-file Le fichier de planification (.vplan)
Mots-clés: format, JSON, fichier, enregistrer, schéma, fichier de projet

Une planification est **un fichier texte** au format JSON avec l’extension `.vplan`. Il peut être copié, archivé et consulté avec un éditeur de texte («Afficher le fichier» dans l’éditeur montre le contenu).

- Le format est identifié par `"format": "vivaplan/1"`. Les anciennes planifications avec l’extension `.rplan` et l’identifiant `pensionsplaner/1` peuvent toujours être ouvertes; l’enregistrement se fait toujours en `.vplan` (l’ancien fichier reste inchangé).
- Les **dates** sont des mois civils (`"2027-05"`) ou des références à des jalons (`"retire"`), `"never"` signifie jamais.
- Les **montants** s’entendent par mois, les **taux** par année sous forme de nombre décimal (0.05 = 5 %).
- Les **valeurs dépendant du temps** sont un nombre ou `{"mode": "timed", "rows": [{"from": "start", "value": 9500}, {"from": "retire", "value": 8500}]}`.
- **Optimisation:** l’objectif, les stratégies, les trajectoires et les **fourchettes des paramètres** par stratégie figurent sous `optimization` (`ranges`: stratégie, paramètre, puis `fixed` ou `lo`/`hi`, parts sous forme de nombre décimal).
- **Impôt sur les gains immobiliers:** `taxes.propertyGain` avec `brackets` (largeur, taux), `duration` (dès années révolues, majoration ou réduction), `exemptBelow` et `reduction`; la date d’achat d’un bien immobilier en propriété figure dans `bought`. Dans le mode «Domicile» s’applique la règle du canton du bien immobilier (`canton` par bien, sans indication le canton de domicile); `taxes.propertyGainOwn: true` passe aux règles propres.
- **Domicile:** `taxes.location` (`canton`, `municipality` comme numéro OFS, `church`) s’applique dès le début; d’autres domiciles figurent dans `taxes.relocations` comme lignes `{"from": "2031-01", "canton": "ZG", "municipality": 1711, "church": "none"}`.
- Le format complet est fourni par `vivaplan schema` dans le terminal.

Le fichier ne contient **aucun résultat**. L’application classe les résultats dans la [liste des résultats](#r-list).

## @ref-cli Ligne de commande
Mots-clés: terminal, vivaplan, commande, script, CLI

Le moteur de calcul est également disponible dans le terminal (programme `vivaplan`):

| Commande | But |
|---|---|
| `run <planification>` | Calculer une planification (`--csv`, `--json`, `--events`, `--metric`, voir [Indicateurs personnalisés](#ref-metric)) |
| `validate <planification>` | Contrôler une planification (`--json`) |
| `set <planification> <chemin>=<valeur>…` | Modifier des valeurs par chemin de champ, voir [Modifier des valeurs](#ref-set) |
| `diff <a> <b>` | Lister les différences entre deux planifications (`--json`) |
| `optimize <planification>` | Optimiser la stratégie, voir [Ligne de commande](#o-cli) |
| `bench <planification>` | Mesurer le temps de calcul (`--paths`, `--json`) |
| `schema` | Afficher le schéma JSON du fichier de planification |
| `library` | Lister les séries temporelles de la bibliothèque de données |
| `import-shiller <fichier>` | Réimporter les données Shiller (.xls, .xlsx, .ods) |
| `import-snb <fichier>…` | Importer des cubes BNS (devkum, plkopr, zimoma, rendoblim) |
| `import-legacy <config.json>` | Reprendre la configuration de l’ancien programme comme point de départ |
| `ui-dev` | Interface dans le navigateur, uniquement en local (développement) |

La ligne de commande affiche ses textes, messages et événements en anglais, indépendamment de la langue de l’interface; les nombres et les dates sont au format suisse.

## @ref-json Sortie en JSON
Mots-clés: JSON, lisible par machine, IA, script, sortie standard, format

Avec `--json` sans nom de fichier, `run`, `validate`, `optimize`, `bench` et `diff` affichent le résultat dans un format fixe sur la sortie standard, pour les scripts et les assistants IA. Les remarques et les erreurs vont sur la sortie d’erreur; en cas d’erreur, la commande se termine avec un code d’erreur. `--json <fichier.json>` enregistre comme jusqu’ici le résultat détaillé dans un fichier.

Chaque sortie commence par `format` et la version. Les montants sont arrondis à des unités entières de la monnaie, les parts à quatre décimales (0.9512 = 95,12 %), les mois figurent sous la forme `"AAAA-MM"`. Les messages ont `code`, `params` et le `text` anglais; les messages de contrôle ont en plus `severity`, `section` et `field` (chemin de champ).

| `format` | Contenu |
|---|---|
| `vivaplan-run/1` | `error` (ou `null`), `issues`, `elapsedMs` et, selon la méthode, `single`, `batch` et `scenarios` ; avec `--metric` en plus `metrics` |
| `vivaplan-validate/1` | `valid` (aucune erreur), `issues` |
| `vivaplan-optimize/1` | réglages, `reference` (la planification telle quelle) et `outcomes` (meilleure stratégie en premier, avec `params`, `validated`, `feasible` et `patch`) |
| `vivaplan-bench/1` | `runs`, `months`, `elapsedMs`, `usPerRun`, `cores`, `check` |
| `vivaplan-diff/1` | `changes`: par différence `path`, `old`, `new` |

**`single`** (une trajectoire de rendement): indicateurs de la simulation (`endTotal`, `endTotalReal`, `totalAtRetirement`, `depleted`, `minLiquid`, `taxesTotal` …), `years` sous forme de tableau avec `columns` et une ligne par année (`year`, `age`, `liquid`, `propertyEquity`, `pensionFund`, `pillar3a`, `total`: état en fin d’année; `income`, `spending`, `taxes`: somme de l’année) et `events` sous forme de paires mois et texte.

**`batch`** (plusieurs trajectoires): `runs`, `successRate`, `earliestDepletion`, `percentiles` (fortune finale à 0 = pire simulation, 1, 5, 10, 25, 50, 75, 90, 95 et 99 %) et `years` avec `p5` à `p95` de la fortune totale, la médiane réelle `p50Real` et la part des simulations avec de la fortune, `survival`.

**`scenarios`** (variations): `params` et par combinaison `values`, `successRate`, `p5`, `p50`, `p95`, `p50Real`, `atRetirement`, `earliestDepletion`.

Pour la planification d’exemple, la sortie de `run --json` compte moins de 20 000 caractères.

## @ref-set Modifier des valeurs par chemin de champ
Mots-clés: set, diff, chemin de champ, modifier, variante, IA, comparer

`vivaplan set <planification> <chemin>=<valeur> …` modifie des valeurs d’une planification sans éditer le fichier à la main. Un **chemin de champ** nomme les clés du fichier de planification séparées par des points, les entrées de liste par leur numéro à partir de 0: `wealth.stockShare`, `household.persons.0.name`, `milestones.1.years`. Ce sont les mêmes chemins que les messages de contrôle indiquent sous `field`.

- La **valeur** est lue en JSON (`0.6`, `true`, `null`, `"texte"`, `{…}`); tout le reste compte comme texte, par exemple `project.name=Variante B` ou `returns.startDate=1990-01`.
- **Moment** pour les valeurs dépendant du temps: `chemin@<moment>=<nombre>` modifie le palier à partir de ce moment; s’il n’y en a pas, un nouveau commence là. Un moment est un jalon (`@retire`), un âge de la première personne (`@age:62.5`) ou un mois (`@2035-01`). Exemple: `wealth.stockShare@start=0.8 wealth.stockShare@retire=0.5` fixe la part d’actions avant et après la retraite; une valeur constante devient alors dépendante du temps.
- **Contrôle:** un chemin inconnu, un type erroné ou un moment inconnu donne un message, et **aucune** des modifications n’est reprise. Les autres messages de contrôle de la planification modifiée apparaissent comme remarques; le fichier est tout de même écrit.
- **Fichier:** sans `-o`, `set` écrit un nouveau fichier à côté de l’ancien, sous le prochain numéro libre (`plan-1.vplan`, `plan-2.vplan`, …), et affiche son nom. Avec `-o <fichier>`, il écrit à cet endroit. `set` ne remplace un fichier existant qu’avec `--overwrite`, après une copie de sauvegarde `<fichier>.bak`. `--overwrite` sans `-o` remplace la planification elle-même.

`vivaplan diff <a> <b>` liste par différence le chemin de champ, l’ancienne et la nouvelle valeur, par exemple `withdrawal.floor: 0 → 0.8`; avec `--json` au format `vivaplan-diff/1`.

## @ref-metric Indicateurs personnalisés (--metric)
Mots-clés: indicateur, metric, mesure, quantile, médiane, probabilité, fenêtre, consommation, IA

`vivaplan run <planification> --metric '<json>'` calcule en plus des indicateurs que vous composez à partir de briques, sur les mêmes trajectoires de rendement que `run` (sans variations). `--metric` peut être indiqué plusieurs fois et prend un indicateur ou une liste d’indicateurs. Avec `--json`, les valeurs figurent sous `metrics`, sinon à la fin du rapport. Les indicateurs sont collectés pendant la simulation ; aucune valeur mensuelle n’est enregistrée pour cela.

| Brique | Clé | Choix |
|---|---|---|
| Grandeur | `quantity` | un champ des valeurs mensuelles (`total`, `liquid`, `income`, `spending`, `taxes`, `withdrawal`, `housing`, `mortgage`, `pensionFund`, `age` …) ou dérivée : `consumption` (dépenses ; après l’épuisement, seulement ce que le revenu couvre), `plannedConsumption`, `depleted` (1 dès l’épuisement), `atFloor` (1 lorsque le plancher détermine les dépenses), les compteurs `emergencySales`, `lumpSums`, `affordabilitySales`, `unaffordableRenewals` |
| Unité | `real` | `true` : aux prix du mois de départ (montants seulement) ; sinon nominal |
| Fenêtre | `window` | `from`, `to` (inclus) : `start`, `retirement`, `end`, identifiant d’un jalon ou `AAAA-MM` ; `fromAge`, `toAge` : âge de la première personne ; `years` : seulement les n premières années. Sans indication, toute la durée |
| Réduction par trajectoire | `reduce` | `annualSums` (somme par 12 mois, une série annuelle), `sum`, `mean`, `min`, `max`, `atStart`, `atEnd`, `firstWhen` (âge au premier mois remplissant la condition), `share` (part des mois remplissant la condition), `any`, `volatility` (variation moyenne des sommes annuelles), `shareOfStartLiquid` (somme rapportée à la fortune liquide avant la fenêtre) |
| Agrégation sur les trajectoires | `aggregate` | `{"kind": "quantile", "q": 0.5}`, `mean`, `min`, `max`, `probability` et `count` (avec `condition`), `conditionalMean` (avec `lowest` ou `highest` comme part, ou `condition`) |

Une condition s’écrit par exemple `{"op": "lt", "value": 50000}` (`gt`, `ge`, `lt`, `le`) ; sans indication, « supérieur à 0 » s’applique, donc « oui » pour `depleted` et `atFloor`. `condition` à côté de `reduce` vérifie les mois (`firstWhen`, `share`, `any`), `condition` dans `aggregate` les valeurs par trajectoire. Avec `annualSums`, l’agrégation prend ensemble toutes les valeurs annuelles de toutes les trajectoires, comme l’optimisation. `name` nomme l’indicateur dans le résultat.

Chaque entrée de `metrics` contient `name`, `value` (ou `null`), `values` (nombre de valeurs), l’erreur type `stdError` pour les probabilités et les moyennes par trajectoire, la trajectoire `path` pour le quantile, le minimum et le maximum, ainsi que la définition `metric`. Les erreurs dans les indicateurs sont réunies en un message avec la liste ; rien n’est alors calculé. `vivaplan schema` décrit la planification, pas les indicateurs.

Exemples :

```
--metric '{"name": "Consommation années 1–15, médiane", "quantity": "consumption", "real": true, "window": {"from": "retirement", "years": 15}, "reduce": "annualSums", "aggregate": {"kind": "quantile", "q": 0.5}}'
--metric '{"name": "Ruine jusqu’à 85 ans", "quantity": "depleted", "window": {"toAge": 85}, "reduce": "any", "aggregate": {"kind": "probability"}}'
--metric '{"name": "Part des mois au plancher", "quantity": "atFloor", "window": {"from": "retirement"}, "reduce": "share", "aggregate": {"kind": "mean"}}'
```

La médiane et le P10 de `consumption` (réel, toute la durée dès `retirement`) sont les mêmes valeurs que sous `optimize --compare`.

## @ref-library Bibliothèque de données: dossiers et fichiers
Mots-clés: dossier, séries temporelles, emplacement, Application Support, CSV

L’application de bureau gère la bibliothèque dans le dossier de données de l’application et y crée les séries fournies manquantes. La [stratégie de placement](#f-ret-series) y utilise `Shiller CHF.csv`, `Shiller CHF + CH-Obligationen.csv` et les données BNS du sous-dossier `quellen`, de sorte que les données Shiller et BNS mises à jour sont prises en compte; si un fichier manque, la version intégrée s’applique. L’application n’utilise plus vos propres fichiers dans le dossier. Sous macOS: `~/Library/Application Support/ch.vivaplan-app.desktop/Zeitreihen`. Les planifications renvoient à une série de la bibliothèque avec `library:Name.csv`, indépendamment de l’endroit où la planification est enregistrée.

L’application ne complète pas d’elle-même une bibliothèque existante avec les séries nouvellement fournies. Copiez-les au besoin dans le dossier. Les indices figurent sous [Stratégie de placement](#f-ret-series).

## @glossar Glossaire
Mots-clés: termes, lexique, signification, abréviations

**AVS:** assurance-vieillesse et survivants, 1er pilier. L’application ne calcule que la rente de vieillesse. Voir [AVS](#f-ahv).

**Avoir de vieillesse:** le capital épargné auprès de la caisse de pension. Voir [Caisse de pension](#f-pk).

**Annuité:** paiements constants qui épuisent un capital sur une durée déterminée. Base du [VPW](#f-wd-vpw).

**Bootstrap:** méthode qui compose de manière aléatoire de nouvelles évolutions à partir de données réelles. Voir [Bootstrap](#f-ret-bootstrap).

**Taux de réussite:** part des simulations dans lesquelles la fortune suffit jusqu’à la fin. Voir [Vue d’ensemble](#r-overview).

**Queues épaisses (fat tails):** bords épais de la distribution des rendements: les événements extrêmes sont plus fréquents qu’avec une loi normale. Voir [modèle paramétrique](#f-ret-parametric).

**Guardrails:** garde-fous pour le prélèvement. Voir [Guardrails](#f-wd-guardrails).

**Retrait en capital:** versement du capital de prévoyance sous forme de montant unique au lieu d’une rente, avec impôt sur les prestations en capital. Voir [Part du retrait en capital](#f-pk-lumpsum).

**Médiane:** valeur centrale: la moitié des valeurs se situe au-dessus, l’autre moitié au-dessous.

**Monte Carlo:** calcul de nombreuses évolutions aléatoires pour estimer des probabilités. Voir [Simulation](#s-returns).

**Nominal / réel:** les montants nominaux sont exprimés en francs futurs, les montants réels en pouvoir d’achat actuel. Voir [Rendement, inflation et valeurs réelles](#ret-concepts).

**Percentile:** le *n*-ième percentile est la valeur au-dessous de laquelle se situent *n* % des valeurs. Voir [Fortune finale par percentile](#r-percentiles).

**Caisse de pension (CP, LPP):** prévoyance professionnelle, 2e pilier. Voir [Caisse de pension](#f-pk).

**Rééquilibrage:** rétablissement de la répartition souhaitée entre actions et obligations. Voir [Part d’actions](#f-stockshare).

**Ruine:** la fortune (liquide et immobilière) est épuisée. Voir [Optimisation: objectif](#o-goal).

**SARON:** taux d’intérêt de référence pour les hypothèques en Suisse. Voir [SARON](#ret-saron).

**Pilier 3a:** prévoyance privée liée, un compte par personne. Voir [Pilier 3a](#s-pillar3a).

**Seed (valeur initiale aléatoire):** valeur de départ du générateur aléatoire, qui rend les résultats reproductibles. Voir [Valeur initiale aléatoire](#ret-seed).

**Risque de séquence:** le risque que de mauvaises années boursières surviennent justement au début de la phase de prélèvement. Voir [Pièges](#ret-pitfalls).

**Capacité financière:** règle bancaire selon laquelle les coûts hypothécaires ne doivent pas dépasser une part du revenu. Voir [Capacité financière](#f-re-afford).

**Tranche:** partie d’une hypothèque avec sa propre durée et son propre taux d’intérêt. Voir [Hypothèques](#f-re-mortgage).

**Taux de conversion:** part du capital de rente versée chaque année sous forme de rente. Voir [Taux de conversion](#f-pk-conversion).

**VPW:** prélèvement en pourcentage variable. Voir [VPW](#f-wd-vpw).

**Volatilité:** mesure de la fluctuation des rendements, généralement sous forme d’écart type par année. Voir [modèle paramétrique](#f-ret-parametric).
