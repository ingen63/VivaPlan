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
- **Impôt sur les gains immobiliers:** `taxes.propertyGain` avec `brackets` (largeur, taux), `duration` (dès années révolues, majoration ou réduction), `exemptBelow` et `reduction`; la date d’achat d’un bien immobilier en propriété figure dans `bought`.
- Le format complet est fourni par `vivaplan schema` dans le terminal.

Le fichier ne contient **aucun résultat**. L’application classe les résultats dans la [liste des résultats](#r-list).

## @ref-cli Ligne de commande
Mots-clés: terminal, vivaplan, commande, script, CLI

Le moteur de calcul est également disponible dans le terminal (programme `vivaplan`):

| Commande | But |
|---|---|
| `run <planification>` | Calculer une planification (`--csv`, `--json`, `--events`) |
| `validate <planification>` | Contrôler une planification |
| `optimize <planification>` | Optimiser la stratégie, voir [Ligne de commande](#o-cli) |
| `bench <planification>` | Mesurer le temps de calcul (`--paths`) |
| `schema` | Afficher le schéma JSON du fichier de planification |
| `library` | Lister les séries temporelles de la bibliothèque de données |
| `import-shiller <fichier>` | Réimporter les données Shiller (.xls, .xlsx, .ods) |
| `import-snb <fichier>…` | Importer des cubes BNS (devkum, plkopr, zimoma, rendoblim) |
| `import-legacy <config.json>` | Reprendre la configuration de l’ancien programme comme point de départ |
| `ui-dev` | Interface dans le navigateur, uniquement en local (développement) |

La ligne de commande affiche ses textes, messages et événements en anglais, indépendamment de la langue de l’interface; les nombres et les dates sont au format suisse.

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
