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
- **Autres formes** ([Valeurs dépendant du temps](#f-timed)): droite `{"mode": "linear", "start": {"age": 55, "value": 0.9}, "end": {"age": 65, "value": 0.5}}`, tableau `{"mode": "table", "interpolation": "step", "points": [{"at": "start", "value": 1}, {"age": 80, "value": 0.8}]}` (`interpolation` `linear` ou `step`, `linear` sans indication), facteur `{"mode": "factor", "base": 6000, "factor": <évolution>}`. Un point d’appui a soit `at` (date ou jalon) soit `age` (âge, avec `person`, sinon de la première personne).
- **Courbe caractéristique** (seulement `wealth.stockShare`, [Courbe caractéristique](#f-curve)): `{"mode": "curve", "input": "withdrawalRate", "points": [[0.02, 0.8], [0.06, 0.4]], "eval": "yearly"}`; `input` vaut `age`, `liquidReal`, `withdrawalRate`, `fundedRatio`, `stockReturn12m` ou `drawdown`, `eval` `yearly` (par défaut) ou `monthly`.
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
| `describe [<section>]` | Catalogue pour les IA : ce qui peut être modifié, varié et mesuré (`--plan`, `--json`), voir [Catalogue pour les IA](#ref-describe) |
| `mcp` | Serveur MCP pour les programmes d’IA, lit, calcule, modifie et enregistre (`--data-dir`, `--library`, `--names`, `--read-only`, `--allow-dir`), voir [Connecter des programmes d’IA](#ref-mcp) |
| `evaluate <planification> <experience.json>` | Calculer une expérience (`--json`, `--yes`), voir [Expérience](#ref-experiment) |
| `bench <planification>` | Mesurer le temps de calcul (`--paths`, `--json`) |
| `schema` | Afficher le schéma JSON du fichier de planification |
| `library` | Lister les séries temporelles de la bibliothèque de données |
| `import-shiller <fichier>` | Réimporter les données Shiller (.xls, .xlsx, .ods) |
| `import-snb <fichier>…` | Importer des cubes BNS (devkum, plkopr, zimoma, rendoblim) |
| `import-legacy <config.json>` | Reprendre la configuration de l’ancien programme comme point de départ |
| `ui-dev` | Interface dans le navigateur, uniquement en local (développement) |

La ligne de commande affiche ses textes, messages et événements en anglais, indépendamment de la langue de l’interface; les nombres et les dates sont au format suisse.

**Programme et dossier de données.** Le programme `vivaplan` fait partie du paquet d’installation et se trouve à côté de l’application. Il n’est pas ajouté au chemin de recherche; appelez-le avec le chemin complet:

| Système | Chemin du programme |
|---|---|
| macOS | `/Applications/VivaPlan.app/Contents/MacOS/vivaplan` |
| Windows (programme d’installation `.exe`) | `%LOCALAPPDATA%\VivaPlan\vivaplan.exe` |
| Windows (`.msi`) | `C:\Program Files\VivaPlan\vivaplan.exe` |
| Linux (`.deb`, `.rpm`) | `/usr/bin/vivaplan` |

Avec l’AppImage, le programme se trouve dans l’image et ne peut pas être appelé directement. `vivaplan --help` affiche toutes les commandes. Un [guide](#ref-skill) pour les assistants IA est fourni.

Comme l’application, la ligne de commande calcule avec le dossier de données de l’application: avec les séries temporelles du sous-dossier `Zeitreihen`, y compris les séries importées dans l’application (`library:<fichier>`), et avec les limites et la capacité financière des [paramètres](#f-settings) (`einstellungen.json`). Le dossier de données est `~/Library/Application Support/ch.vivaplan-app.desktop` sous macOS, `%APPDATA%\ch.vivaplan-app.desktop` sous Windows et `~/.local/share/ch.vivaplan-app.desktop` sous Linux. `--data-dir <dossier>` prend un autre dossier de données, `--library <dossier>` une autre bibliothèque de données. Si le dossier de données manque, la ligne de commande calcule avec les séries intégrées et les limites par défaut et le signale une fois.

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
- **Moment** pour les valeurs dépendant du temps: `chemin@<moment>=<nombre>` modifie le palier à partir de ce moment; s’il n’y en a pas, un nouveau commence là. Un moment est un jalon (`@retire`), un âge de la première personne (`@age:62.5`) ou un mois (`@2035-01`). Exemple: `wealth.stockShare@start=0.8 wealth.stockShare@retire=0.5` fixe la part d’actions avant et après la retraite; une valeur constante devient alors dépendante du temps. Une droite, un tableau, un facteur ou une courbe caractéristique n’admet pas de moment; `set` signale alors une erreur.
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

## @ref-experiment Expérience (evaluate)
Mots-clés: expérience, evaluate, variation, grille, recherche par motifs, contrainte, objectif, budget, IA

`vivaplan evaluate <planification> <experience.json> [--json] [--yes]` pose une question à la planification sous forme d’une structure JSON et la calcule en un appel : modifications, variations, [indicateurs](#ref-metric), contraintes et objectif. Tout est du JSON structuré sans formules dans des chaînes; le schéma se trouve sous `definitions.Experiment` dans `vivaplan schema`.

| Clé | Contenu |
|---|---|
| `set` | Modifications pour toutes les variantes : chemin de champ → valeur, comme avec [`set`](#ref-set), aussi avec un moment (`"wealth.stockShare@retire": 0.4`) |
| `vary` | Variations, chacune combinée avec chacune : `{"path", "values": […]}` (liste), `{"path", "range": [de, à, pas]}` (plage), `{"path", "follows", "offset"}` (couplée : valeur de l’autre chemin plus `offset`) ou `{"group": [{"set", "vary"}, …]}` (groupe : les membres ne sont pas combinés entre eux) |
| `metrics` | Indicateurs comme avec `--metric`, chacun avec un `name` |
| `constraints` | Contraintes `{"metric", "op", "value"}` avec `op` = `le`, `lt`, `ge`, `gt` |
| `objective` | exactement un : `{"maximize": "<indicateur>"}`, `{"minimize": "<indicateur>"}`, `{"weighted": [{"metric", "weight"}, …]}` (maximiser la somme pondérée) ou `{"score": {…}}` (objectif de l’[optimisation](#o-cli) avec `maxRuin`, `weightMedian`, `volWeight` …). Sans objectif, toutes les variantes sont listées (au plus 50) |
| `search` | `method` : `grid` (grille, toutes les variantes; par défaut) ou `pattern` (recherche par motifs dans les plages comme l’optimisation, par combinaison des autres valeurs; `samples` points de départ aléatoires, 30 par défaut; `seed`) |
| `budget` | `paths` trajectoires de recherche (1'000), `margin` marge sur les contraintes en erreurs types (2), `top` nombre de variantes confirmées (5), `confirmPaths` trajectoires de confirmation (10'000) |
| `confirm` | `true` : calculer même si le calcul est long |

**Déroulement.** D’abord toute l’expérience est vérifiée : les erreurs forment une liste avec le chemin dans l’expérience (par exemple `vary[0].values[1]`), la raison et les valeurs valides, et rien n’est calculé. Puis la durée est estimée; à partir de 60 secondes, `evaluate` ne calcule qu’avec `--yes` (ou `"confirm": true`) et indique sinon l’estimation. La recherche utilise peu de trajectoires; une contrainte doit y être respectée avec la marge. Les `top` meilleures variantes sont confirmées sur des trajectoires **indépendantes** (autre graine); le classement suit les valeurs confirmées. Toutes les variantes calculent sur les mêmes trajectoires, de sorte que les différences entre elles sont plus précises que les valeurs isolées. Avec des trajectoires fixes (historique, toutes les dates de départ), il n’y a pas de trajectoires indépendantes; une remarque le signale.

**Résultat** (avec `--json` sous `result`) : `ranking` avec `rank`, les valeurs de la variante (`settings`), `search` et `confirm` (chacun `objective`, `feasible`, `figures` avec `value` et `stdError`; avec `score` aussi `optimization`), `hints` (par exemple au bord de la grille : la meilleure variante se trouve au bord d’une plage), `paths` (nombre et graines des deux étapes, `independent`), l’`experiment` complète avec toutes les valeurs par défaut et la `version` du moteur de calcul. `status` vaut `done`, `invalid`, `needsConfirmation`, `cancelled` ou `failed`; `estimate` indique variantes, simulations et secondes. La sortie reste courte (moins de 20'000 caractères).

Exemple : 10 ans jusqu’à la retraite, part d'actions de 40, 60 ou 80 % avant et de 30 ou 50 % après, prélèvement à taux fixe de 3, 4 ou 5 % ou endowment 3 ou 4 %, la consommation réelle la plus élevée durant les 15 premières années de retraite avec au plus 2 % de ruine – 30 variantes :

```
{
  "vary": [
    { "path": "wealth.stockShare@start", "values": [0.4, 0.6, 0.8] },
    { "path": "wealth.stockShare@retire", "values": [0.3, 0.5] },
    { "group": [
      { "set": { "withdrawal.strategy": "fixed_rate" },
        "vary": [ { "path": "withdrawal.fixedRate.rate", "values": [0.03, 0.04, 0.05] } ] },
      { "set": { "withdrawal.strategy": "endowment" },
        "vary": [ { "path": "withdrawal.endowment.rate", "values": [0.03, 0.04] } ] }
    ] }
  ],
  "metrics": [
    { "name": "ruin", "quantity": "depleted", "reduce": "any", "aggregate": { "kind": "probability" } },
    { "name": "cons15", "quantity": "consumption", "real": true, "window": { "from": "retire", "years": 15 },
      "reduce": "annualSums", "aggregate": { "kind": "mean" } }
  ],
  "constraints": [ { "metric": "ruin", "op": "le", "value": 0.02 } ],
  "objective": { "maximize": "cons15" }
}
```

Variations, comparaison des stratégies et optimisation s’expriment ainsi : les variations sont une grille sans objectif; la comparaison des stratégies est un groupe des stratégies avec `"objective": {"score": {}}` et `budget.paths` = trajectoires de la planification; l’optimisation est une recherche par motifs avec `score` et `"margin": 1.5`.

## @ref-describe Catalogue pour les IA (describe)
Mots-clés: describe, catalogue, IA, champs, unité, plage de valeurs, grandeur d’état, indicateur, temps de calcul

`vivaplan describe [<section>] [--plan <planification>] [--json]` indique à un programme ou à une IA sans code source ce qu’elle peut modifier, varier et mesurer dans une planification. Le catalogue est généré à partir du schéma et du code et est en anglais comme le reste de la ligne de commande. Il reste sous 20'000 caractères; les sections séparément avec `describe <section>` :

| Section | Contenu |
|---|---|
| `fields` | Champs de la planification groupés par objet, chacun avec type, unité, plage de valeurs et (uniquement dans cette section) signification; `*` représente le numéro dans une liste |
| `series` | Formes d’une valeur dans le temps (`const`, `timed`, `linear`, `table`, `factor`, `curve`) et les moments (jalon, `AAAA-MM`, `age:<années>`, `chemin@<moment>`) |
| `state` | Grandeurs d’état d’une [courbe caractéristique](#f-curve) |
| `metrics` | Éléments d’un [indicateur](#ref-metric) : grandeurs, fenêtre de temps, réduction par trajectoire, agrégation sur les trajectoires, comparaisons |
| `experiment` | Formes de l’[expérience](#ref-experiment) : variations, contrainte, objectif, méthodes de recherche, valeurs par défaut |
| `examples` | trois expériences d’exemple (dépenses avec au plus 5 % de ruine, rente ou capital, part d’actions et règle de prélèvement) qui tournent sur la planification d’exemple |
| `performance` | Microsecondes par calcul, mesurées lors de l’appel (environ une demi-seconde) sur la planification de `--plan`, sinon sur la planification d’exemple avec rendements paramétriques; elles permettent d’estimer la durée d’une expérience |
| `limits` | Limites du modèle : années indépendantes dans le modèle paramétrique, erreur de sélection avec de nombreuses variantes, incertitude, nominal et réel, temps de calcul |

Avec `--json`, le catalogue est fourni au format fixe `vivaplan-describe/1`. Les descriptions complètes et la structure exacte sont affichées par `vivaplan schema`.

## @ref-skill Guide pour les assistants IA (agent skill)
Mots-clés: skill, agent skill, SKILL.md, Claude, Claude Code, IA, assistant, configurer

Le paquet d’installation contient un guide pour les assistants IA capables d’exécuter des commandes: l’agent skill `vivaplan-skill` au format ouvert (dossier avec le fichier `SKILL.md`). Il explique à l’IA la [ligne de commande](#ref-cli), les champs importants avec leurs unités (parts au lieu de pourcentages), nominal et réel, les limites des résultats et la clause de non-responsabilité. Le guide est en anglais; l’IA répond néanmoins dans votre langue.

| Système | Dossier du skill |
|---|---|
| macOS | `/Applications/VivaPlan.app/Contents/Resources/vivaplan-skill` |
| Windows (programme d’installation `.exe`) | `%LOCALAPPDATA%\VivaPlan\vivaplan-skill` |
| Windows (`.msi`) | `C:\Program Files\VivaPlan\vivaplan-skill` |
| Linux (`.deb`, `.rpm`) | `/usr/lib/VivaPlan/vivaplan-skill` |

**Configurer dans Claude Code et dans Claude Desktop (section *Code*):** les deux lisent les skills personnels dans le dossier `.claude/skills` de votre dossier utilisateur. Copiez-y le dossier `vivaplan-skill`, sous macOS par exemple dans le Terminal:

```
mkdir -p ~/.claude/skills && cp -R /Applications/VivaPlan.app/Contents/Resources/vivaplan-skill ~/.claude/skills/
```

Sous Windows, copiez le dossier dans `%USERPROFILE%\.claude\skills`. Démarrez ensuite une nouvelle session. Demandez par exemple: *Ma fortune suffit-elle jusqu’à 95 ans? Ma planification est ~/Documents/planification.vplan*. Claude appelle pour cela `vivaplan` et demande auparavant s’il peut exécuter des commandes. Après une nouvelle version de VivaPlan, copiez à nouveau le dossier.

**Protection des données:** ce que l’IA lit dans votre planification ou dans les résultats (dates de naissance, revenus, fortune, rentes) est transmis au fournisseur de l’IA. Le skill demande à l’IA de vous en avertir avant la première lecture. VivaPlan lui-même n’envoie rien.

Les résultats sont des calculs de modèle sans garantie, pas un conseil fiscal ou en placement; les impôts sont estimés.

## @ref-mcp Connecter des programmes d’IA (serveur MCP)
Mots-clés: MCP, Model Context Protocol, serveur, vivaplan mcp, LM Studio, Claude Desktop, IA, outil, protection des données, enregistrer, annuler, plan_set, plan_save

`vivaplan mcp` est un serveur MCP : les programmes d’IA qui prennent en charge le Model Context Protocol (par exemple Claude Desktop, Claude Code, ChatGPT Desktop ou LM Studio) le démarrent eux-mêmes et communiquent avec lui par l’entrée et la sortie standard. Il n’ouvre aucune connexion réseau. Le serveur modifie une copie de travail en mémoire ; seul `plan_save` écrit sur le disque, selon des règles fixes (voir ci-dessous).

| Outil | But |
|---|---|
| `plan_open` | Ouvrir une planification (`.vplan`, `.rplan`) ou la planification d’exemple ; aperçu sans valeurs et nombre de messages de contrôle |
| `plan_get` | Lire une section ou un seul champ (chemin de champ) |
| `plan_schema` | Consulter la structure du fichier de projet |
| `plan_validate` | Contrôler la planification comme l’application |
| `describe` | [Catalogue pour les IA](#ref-describe) |
| `result_list`, `result_get` | Résultats de la session (résumé, valeurs annuelles, événements, fourchette, variations) |
| `library_list` | Séries temporelles de la [bibliothèque de données](#ref-library) |
| `simulation_run` | Simuler la planification comme l’application (avec ses variations) ; indicateurs et résultat en mots |
| `evaluate` | [Expérience](#ref-experiment) sur la planification : modifications, variations, indicateurs, conditions, objectif ; l’optimisation et la comparaison passent par là |
| `plan_set` | Modifier des valeurs de la copie de travail par chemin de champ (comme [`vivaplan set`](#ref-cli)) ; toutes les modifications d’un appel forment une étape, avec l’ancienne et la nouvelle valeur et les messages de vérification qui en résultent |
| `plan_changes` | Toutes les différences entre la copie de travail et le fichier |
| `plan_undo` | Annuler la dernière étape, avec `all` toutes |
| `plan_apply` | Reprendre dans la planification une variante d’un résultat d’`evaluate` (selon son rang), en une étape |
| `plan_save` | Enregistrer la copie de travail en `.vplan` ; seul outil qui écrit sur le disque |

Avant chaque calcul, le serveur estime sa durée. À partir de 60 secondes, il ne calcule que si l’IA le confirme expressément (`"confirm": true`) ; elle doit vous le demander auparavant. Les calculs longs indiquent leur progression et peuvent être interrompus dans le programme d’IA. Un seul calcul s’exécute à la fois.

**Enregistrer :** `plan_save` respecte lui-même ces règles, quoi que fasse l’IA : (1) Sans chemin, un **nouveau fichier** est créé à côté de celui ouvert, `<nom> (AI).vplan` (avec un numéro si nécessaire) ; un fichier existant reste intact. (2) Le serveur ne remplace un fichier existant qu’avec `overwrite` et seulement après que le programme d’IA vous a posé la question en montrant les modifications ; s’il ne peut pas poser de question, le serveur refuse. (3) Une copie de sauvegarde `<fichier>.bak` est créée auparavant ; l’écriture passe par un fichier temporaire. (4) Si le fichier a été modifié depuis son ouverture, il n’est pas remplacé. (5) Les anciens fichiers `.rplan` ne sont jamais remplacés ; l’enregistrement se fait en `.vplan`. (6) Le serveur n’écrit que dans le dossier de la planification ouverte ou dans un dossier autorisé avec `--allow-dir <dossier>`. Avec `--read-only`, le serveur ne propose pas du tout `plan_save`. Ne modifiez pas une planification en même temps dans l’application et dans l’IA : l’application ne remarque pas les modifications du fichier.

**Configurer :** dans le programme d’IA, indiquez le programme `vivaplan` avec son chemin complet (voir [ligne de commande](#ref-cli)) et l’argument `mcp`. Dans Claude Desktop et LM Studio, l’entrée se présente ainsi sous macOS :

```
{
  "mcpServers": {
    "vivaplan": { "command": "/Applications/VivaPlan.app/Contents/MacOS/vivaplan", "args": ["mcp"] }
  }
}
```

Dans Claude Code, `claude mcp add --scope user vivaplan -- /Applications/VivaPlan.app/Contents/MacOS/vivaplan mcp` suffit. Comme la ligne de commande, le serveur calcule avec le dossier de données de l’application (limites, séries temporelles) ; `--data-dir` et `--library` le remplacent.

**Protection des données :** tout ce qu’un outil renvoie fait partie de la conversation avec votre IA et est transmis à son fournisseur ; avec un modèle local (par exemple dans LM Studio), tout reste sur l’ordinateur. Le serveur ne lit que les planifications dont le chemin est indiqué explicitement et ne parcourt aucun dossier. Il remplace les noms de personnes et les désignations saisies librement par des noms neutres (Person 1, Property 1), sauf avec `--names` ; les réponses n’indiquent que le nom du fichier, pas le dossier. Aucune réponse ne dépasse 20'000 caractères. Les textes du serveur sont en anglais ; l’IA répond néanmoins dans votre langue.

Les résultats sont des calculs de modèle sans garantie, pas un conseil fiscal ou en placement ; les impôts sont estimés.

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
