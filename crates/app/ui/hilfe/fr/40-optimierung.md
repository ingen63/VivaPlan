# Optimisation

## @s-optimize Optimisation: vue d’ensemble
Mots-clés: meilleure stratégie, paramètres, ruine, consommation, recherche, objectif

L’optimisation répond à la question: **quelle stratégie de prélèvement, avec quels réglages, convient le mieux à mon scénario de base?**

Elle prend votre planification comme donnée et recherche, pour chaque [stratégie de prélèvement](#s-withdrawal) choisie, les paramètres qui atteignent trois choses à la fois:

1. La **probabilité de ruine** reste sous votre limite (par exemple 5 %).
2. La **consommation** est aussi élevée que possible (médiane et mauvaise année, en termes réels).
3. La consommation **fluctue** le moins possible d’une année à l’autre.

Les stratégies sont ensuite comparées. La meilleure figure en haut et porte une étoile ★. D’un clic sur «Reprendre dans la planification», vous reprenez la stratégie et les paramètres dans votre planification.

**Conditions:**

- Une source de rendements avec **dispersion**: Monte Carlo (bootstrap ou paramétrique) ou «Toutes les dates de départ historiques». Avec «Rendements fixes» et «Historique dès la date de départ», il n’y a qu’une seule trajectoire, la ruine serait de 0 % ou de 100 %. L’application le signale et bloque le lancement.
- Une **retraite** dans la planification (caisse de pension «Prestation dès»), car les stratégies s’appliquent à partir de là.

**Combien de temps cela dure-t-il?** Sous le bouton «Lancer l’optimisation» figure une **estimation de la durée**, qui s’adapte à chaque modification des réglages. Elle repose sur une mesure: l’application calcule brièvement quelques milliers de simulations de votre planification et extrapole le nombre de simulations dont l’optimisation a besoin (trajectoires de la recherche, points de départ et stratégies, plus les trajectoires de vérification). L’indication est **approximative**: le nombre d’étapes de recherche dépend du cas, c’est pourquoi une fourchette est indiquée (‹généralement 7 s à 20 s›). Exemples sur un Mac actuel à 10 cœurs: les réglages standard avec les six stratégies durent environ 15 s avec le bootstrap, autant avec le modèle paramétrique, et 2 à 3 s avec «Toutes les dates de départ historiques». Deux fois plus de trajectoires signifient environ deux fois plus de temps. Le calcul s’effectue localement et utilise tous les cœurs du processeur.

**Pendant l’optimisation**, une barre de progression indique l’avancement (le pourcentage est une estimation, car le nombre d’étapes de recherche dépend du cas). Avec **Interrompre**, vous arrêtez le calcul; aucun résultat partiel ne subsiste.

**Quels paramètres varient**, vous le décidez vous-même pour chaque stratégie, voir [Paramètres fixes ou variables](#o-ranges).

**Les réglages sont enregistrés** (objectif, stratégies, trajectoires, fourchettes des paramètres) **avec la planification**: ils figurent dans le fichier `.vplan`, reviennent à l’ouverture et valent aussi pour `optimize` dans le terminal (les options les y remplacent). L’app classe le résultat dans la [liste des résultats](#r-list), voir [Enregistrer et rouvrir le résultat](#o-saved). Un réglage modifié n’invalide pas un résultat antérieur, seule une planification modifiée le fait.

Lisez les résultats avec [Lire le résultat](#o-read) et tenez compte des [pièges](#o-pitfalls). Un optimum n’est **pas une promesse**.

## @o-goal-select Choix de l’objectif
Mots-clés: objectif, pas de vente forcée de biens immobiliers, vente forcée, consommation minimale

L’optimisation a deux objectifs; le choix est enregistré avec la planification (standard: objectif 1).

- **Objectif 1, consommation avec risque de ruine limité:** comme décrit dans ce chapitre: une consommation aussi élevée et régulière que possible, tant que la ruine reste sous la limite.
- **Objectif 2, pas de vente forcée de biens immobiliers:** on recherche la stratégie avec la **plus petite part de simulations dans lesquelles un bien immobilier est vendu de manière non planifiée** (vente forcée). À part égale, la plus faible probabilité de ruine décide (ruine comme pour l’objectif 1). Contraintes: la **médiane de la consommation annuelle dès la retraite** atteint au moins la part saisie des dépenses planifiées (standard 90 %), et la probabilité de ruine est inférieure à la limite (standard 5 %). Les poids de la consommation et la volatilité ne jouent aucun rôle pour l’objectif 2.

Le tableau des résultats indique l’objectif choisi. Pour l’objectif 2, il montre pour chaque stratégie la **part des simulations avec une vente non planifiée**; l’ordre se fonde sur cette part, à égalité sur la ruine. Les stratégies qui violent les contraintes figurent en bas et sont marquées. Si aucune stratégie ne remplit les contraintes, celle qui s’en approche le plus figure en haut, avec une remarque.

L’objectif 2 nécessite au moins un bien immobilier dans la planification (acheté ou planifié); sans bien immobilier, il ne peut pas être choisi.

## @o-goal Objectif: probabilité de ruine
Mots-clés: ruine, limite, âge, épuisement, 5 pour cent, risque

La probabilité de ruine est la **contrainte** centrale. Une stratégie qui ne la respecte pas ne peut pas gagner, quelle que soit sa consommation.

- **Probabilité de ruine maximale** (standard 5 %): part maximale admise des trajectoires dans lesquelles la fortune est épuisée. **Ruine** signifie: plus de fortune liquide ni de biens immobiliers. Les valeurs usuelles vont de 1 % à 10 %. Plus la limite est basse, plus c’est sûr et plus la consommation possible est basse.
- **La ruine compte jusqu’à l’âge de** (vide = jusqu’à la fin de la simulation): si vous n’avez besoin de sécurité que jusqu’à un certain âge, saisissez-le ici. Une ruine à 92 ans ne compte alors pas avec ‹jusqu’à 90›. Il s’agit d’un **assouplissement délibéré**; pensez à la longévité. Avec un champ vide, la limite s’applique jusqu’à la fin de la simulation.

> Quelle limite est raisonnable? Une probabilité de ruine de 5 % signifie que l’argent s’épuise dans 1 avenir simulé sur 20. Comparez cela avec vos réserves: l’AVS et la caisse de pension continuent de verser, la fortune n’est qu’une partie du financement. Avec deux sources ou plus donnant des résultats différents, une limite plus basse (2 % à 3 %) est une prudence raisonnable.

## @o-consumption Objectif: consommation (médiane et P10)
Mots-clés: médiane, percentile, P10, poids, consommation, dépenses, réel

Ce qui est optimisé, c’est la **consommation annuelle dès la retraite**: l’ensemble des dépenses, frais de logement compris, **avant impôts**, aux **prix du mois de départ** (donc en termes réels). L’application rassemble les valeurs annuelles **de toutes les trajectoires et de toutes les années** et en tire deux indicateurs:

- **Médiane:** la consommation annuelle typique. La moitié de toutes les années-trajectoires se situe au-dessus, l’autre moitié au-dessous.
- **P10 (10e percentile):** la consommation annuelle au-dessous de laquelle se situent 10 % des années-trajectoires. C’est le **mauvais cas**, qui mesure à quel point les crises sont dures.

**Poids de la médiane dans l’objectif de consommation** (standard 50 %): comment l’objectif de consommation est composé à partir des deux indicateurs. Le reste va au P10.

> Objectif de consommation = poids × médiane + (1 − poids) × P10

- **100 %:** seule la consommation typique compte. L’optimisation choisit des réglages plus agressifs avec une médiane élevée et de mauvaises années de crise.
- **0 %:** seul le mauvais cas compte. Cela conduit à des réglages prudents.
- **50 %** est un point de départ équilibré. Augmentez le poids du P10 si la **fiabilité** vous importe davantage que la moyenne.

Dès la ruine, seul ce que couvrent les revenus et les rentes compte encore comme consommation, et non plus les dépenses planifiées. Une ruine fait donc baisser la médiane et surtout le P10.

## @o-vol Objectif: volatilité
Mots-clés: volatilité, fluctuation, d’une année à l’autre, déduction, réduction, stable

La **volatilité** mesure à quel point la consommation annuelle change d’une année à la suivante: la **variation absolue moyenne** par rapport à l’année précédente, moyennée sur toutes les trajectoires. 10 % signifie que la consommation augmente ou diminue en moyenne de 10 % par année. Les dépenses fixes (Standard, 3 compartiments) ont une volatilité proche de 0 %, les stratégies qui suivent la fortune (taux fixe, VPW) une volatilité élevée.

Deux réglages déterminent comment la volatilité entre dans l’objectif:

- **Déduction pour la volatilité** (standard 1): l’objectif de consommation est multiplié par le facteur (1 − déduction × volatilité). Avec une déduction de 1 et une volatilité de 10 %, l’objectif baisse de 10 %. **0 signifie que la volatilité est indifférente**; ce sont alors les stratégies avec la consommation la plus élevée qui gagnent, même si elle fluctue fortement.
- **Volatilité maximale** (vide = pas de limite): un **plafond strict** comme contrainte, comme pour la limite de ruine. Exemple 10 %: les stratégies dont la variation annuelle moyenne est plus élevée sont exclues. C’est le réglage pour ‹je veux que mes dépenses ne fluctuent pas en moyenne de plus de 10 % par année›.

## @o-score L’objectif en une formule
Mots-clés: formule, valeur cible, évaluation, fitness, score

> **Objectif** = (poids × médiane + (1 − poids) × P10) × (1 − déduction × volatilité)

L’objectif est affiché en francs par année (prix réels). **Un candidat est admissible** si la probabilité de ruine est inférieure à la limite et (le cas échéant) si la volatilité reste sous le maximum. Parmi les candidats admissibles, celui qui a l’objectif le plus élevé gagne. Si deux ont le même objectif (par exemple avec des dépenses fixes, où toutes les stratégies ont la même consommation), celui qui a la **plus faible probabilité de ruine** gagne. Les candidats non admissibles sont classés selon l’ampleur de leur dépassement de la limite.

Avec des dépenses fixes (Standard, 3 compartiments), la consommation est la même dans toutes les simulations. L’optimisation y décide donc **de la sécurité**, pas de la consommation. Pour les stratégies qui suivent la fortune, la consommation découle de la stratégie.

## @o-strategies Stratégies et leurs paramètres
Mots-clés: paramètres, plage de recherche, Standard, 3 compartiments, Guardrails, VPW, Endowment, taux fixe, part d’actions

Choisissez les stratégies à comparer. Il est recommandé de **toutes** les calculer pour voir en quoi les approches diffèrent. Pour chaque stratégie, l’application recherche ces paramètres dans ces **plages standard**. Vous décidez vous-même quels paramètres varient et dans quelle fourchette, voir [Paramètres fixes ou variables](#o-ranges).

| Stratégie | Paramètres | Plage de recherche |
|---|---|---|
| Standard | part d’actions, réserve (mois) | 10 % à 100 %, 3 à 36 |
| 3 compartiments | compartiment 1: cash (années), compartiment 2: obligations (années), réalimentation dès un rendement de l’année précédente de | 0.5 à 4, 0 à 10, −10 % à +20 % |
| Guardrails | taux initial, garde-fous supérieur et inférieur, ajustement, part d’actions | 2 % à 8 %, chacun 5 % à 50 %, 2 % à 30 %, 10 % à 100 % |
| Taux fixe | taux de prélèvement, part d’actions | 2 % à 8 %, 10 % à 100 % |
| VPW | âge de planification, rendement réel, part d’actions | 85 à 105, −1 % à 5 %, 10 % à 100 % |
| Endowment | taux cible, lissage, part d’actions | 2 % à 8 %, 5 % à 100 %, 10 % à 100 % |

**Important:**

- La **part d’actions** est modifiée **dès la retraite**. Les lignes antérieures d’une valeur dépendant du temps sont conservées.
- Les **limites du coût de la vie** (plancher, plafond) restent par défaut fixées à la valeur de votre planification. Cela vaut pour toutes les stratégies qui fixent elles-mêmes le coût de la vie: taux fixe, Guardrails, VPW, Endowment et 3 compartiments + Guardrails. Vous pouvez également les y faire varier. Un plancher peut fortement restreindre l’optimisation, car il empêche les réductions.
- La **stratégie à 3 compartiments** fixe la part d’actions par les compartiments; c’est pourquoi elle n’y est pas recherchée.
- Un paramètre en bordure de la plage indique que la plage est limitante: l’optimum se trouve au bord, pas au-delà.

Les paramètres sont décrits sous les stratégies: [Standard](#f-wd-standard), [3 compartiments](#f-wd-three_buckets), [Guardrails](#f-wd-guardrails), [Taux fixe](#f-wd-fixed_rate), [VPW](#f-wd-vpw), [Endowment](#f-wd-endowment).

## @o-ranges Paramètres fixes ou variables
Mots-clés: fourchette, fixe, varier, de, à, paramètres, plage de recherche, fourchettes standard, compartiment

Sous «Paramètres des stratégies» figure, pour chaque stratégie, un tableau avec tous ses paramètres:

- **Dans la planification:** la valeur qu’a aujourd’hui votre planification (ou la valeur standard de la stratégie si vous ne l’avez jamais utilisée).
- **varier:** si la case est cochée, l’optimisation recherche ce paramètre dans la fourchette de «de» à «à». **Sans coche, il reste fixe** à la valeur de la planification.
- **de / à:** la fourchette dans l’**unité du paramètre**: les années en années, les mois en mois, les pour cent en pour cent, l’âge en années.

**Exemple 3 compartiments:** compartiment 1 fixe (pas de coche), compartiment 2 de 0 à 2 ans, réalimentation dès un rendement de l’année précédente de −30 % à +10 %. La recherche ne modifie alors que le compartiment 2 et le seuil de réalimentation; le compartiment 1 reste tel qu’il figure dans la planification.

**Bon à savoir:**

- Une **fourchette étroite** rend la recherche plus rapide et le résultat plus proche de votre planification. Une fourchette **large** trouve plus volontiers des surprises, mais demande plus de temps de calcul.
- Si **tous** les paramètres d’une stratégie sont fixes, seule cette planification est calculée. C’est utile pour comparer équitablement des stratégies aux valeurs fixes. C’est plus rapide avec [Comparer les stratégies (sans optimisation)](#o-compare), sans modifier les coches.
- «de» ne doit pas être supérieur à «à». Il existe des limites extrêmes par paramètre (p. ex. part d’actions de 0 % à 100 %); au-delà, l’application signale une erreur.
- **Le plancher et le plafond du coût de la vie** (taux fixe, Guardrails, VPW, Endowment, 3 compartiments + Guardrails) sont d’abord fixes. Qui les fait varier obtient souvent des résultats qui n’ont l’air ‹bons› qu’avec le plancher, car les réductions sont empêchées.
- «Fourchettes standard» réinitialise la stratégie au préréglage.
- Les réglages sont **enregistrés avec la planification**.

## @o-compare Comparer les stratégies (sans optimisation)
Mots-clés: comparer, sans recherche, valeurs de la planification, fortune finale, ne jamais vendre les biens immobiliers, les deux

Le bouton **«Comparer les stratégies (sans optimisation)»** à côté de «Lancer l’optimisation» calcule les stratégies choisies **avec les valeurs des paramètres de votre planification**, sans rechercher de meilleures valeurs. Pour une stratégie que votre planification n’a jamais utilisée, ses valeurs par défaut s’appliquent (colonne «dans la planification» sous «Paramètres des stratégies»).

- Les coches **«varier»** et les fourchettes restent inchangées; la comparaison ne les prend pas en compte. Vous pouvez ensuite optimiser comme d’habitude.
- Le calcul se fait sur **les mêmes trajectoires que la simulation** (nombre et graine de la source de rendements). Les chiffres d’une stratégie correspondent donc à une simulation dans laquelle vous avez choisi cette stratégie. Les trajectoires de recherche et de vérification sous «Avancé» ne valent que pour l’optimisation.
- Le tableau est le même qu’après une optimisation (voir [Lire le résultat](#o-read)); la ligne sous chaque stratégie montre les valeurs de la planification.

**Biens immobiliers:** Si votre planification comporte un bien en propriété ou prévu, le choix «Biens immobiliers» détermine ce qui est calculé, pour la comparaison comme pour l’optimisation:

- **comme planifié:** avec votre planification telle qu’elle est;
- **Ne jamais vendre les biens immobiliers:** avec le commutateur [Ne jamais vendre les biens immobiliers](#f-re-never-sell) activé;
- **les deux:** chaque stratégie deux fois, avec les mentions «comme planifié» et «ne jamais vendre les biens immobiliers».

Sans bien immobilier en propriété, le choix est masqué. Le choix est enregistré avec la planification.

## @o-advanced Réglages avancés
Mots-clés: trajectoires, vérification, points de départ, précision, temps de calcul

- **Trajectoires pour la recherche** (standard 5 000): nombre de trajectoires aléatoires que la recherche évalue pour chaque candidat. **Plus de trajectoires** rendent le résultat plus précis, mais plus lent. Règle empirique: au-dessous de 2 000 trajectoires, l’évaluation devient instable; au-dessus de 20 000, l’effort n’en vaut généralement plus la peine.
- **Trajectoires pour la vérification** (standard 10 000): nombre de trajectoires **différentes** avec lesquelles les paramètres trouvés sont recalculés à la fin. Ce sont ces valeurs que vous voyez dans le résultat.
- **Points de départ aléatoires par stratégie** (standard 30): la recherche commence à la planification, au milieu de la plage de recherche et en autant de points aléatoires. Davantage de points de départ aident à ne pas rester bloqué dans un mauvais optimum local, mais coûtent du temps, surtout pour les stratégies avec de nombreux paramètres (Guardrails).

Les trajectoires proviennent de la source de rendements de la planification (bootstrap ou paramétrique). Les trajectoires de vérification ont une **autre valeur initiale aléatoire**. Avec «Toutes les dates de départ historiques», il n’y a pas de trajectoires aléatoires. La recherche et la vérification calculent alors les mêmes segments historiques; **la vérification n’y est pas indépendante**.

## @o-algorithm Comment fonctionne la recherche
Mots-clés: algorithme, recherche aléatoire, recherche par motifs, marge de sécurité, vérification, indépendante, surajustement

1. **Trajectoires communes:** tous les candidats sont évalués avec les **mêmes** trajectoires de rendement. Les différences proviennent ainsi des paramètres, et non du hasard.
2. **Recherche aléatoire:** l’application évalue des points de départ (la planification actuelle, le milieu de la plage et des points aléatoires) et prend le meilleur.
3. **Recherche par motifs (recherche par coordonnées):** à partir du meilleur point, chaque paramètre est augmenté et diminué individuellement. Si une modification apporte une amélioration, elle est retenue. Si aucune n’est trouvée, le pas est réduit de moitié, jusqu’à devenir très petit.
4. **Marge de sécurité:** sinon, la recherche exploiterait la limite de ruine jusqu’au bout et se retrouverait juste au-dessus sur de nouvelles trajectoires (erreur aléatoire de l’estimation). C’est pourquoi elle recherche avec une limite **plus stricte de 1.5 erreur type**. Avec une limite de 5 % et 5 000 trajectoires, cela représente environ 0.5 point de pourcentage.
5. **Vérification indépendante:** le résultat est recalculé sur des trajectoires **différentes** (autre valeur initiale aléatoire, plus de trajectoires). Seule cette vérification décide si une stratégie est admissible.

La vérification protège contre le **surajustement**: une stratégie qui n’a l’air bonne que sur les trajectoires de recherche échoue à la vérification.

## @o-read Lire le résultat
Mots-clés: tableau, situation initiale, étoile, grisé, ruine, médiane, P10, volatilité, objectif, barre, vérification

Le tableau des résultats montre pour chaque stratégie les indicateurs, **calculés sur les trajectoires de vérification indépendantes**. La consommation est indiquée en francs par année, aux prix du mois de départ, frais de logement compris et avant impôts.

| Colonne | Signification |
|---|---|
| Stratégie | nom; ★ marque la meilleure stratégie admissible |
| Ruine | part des trajectoires dans lesquelles la fortune est épuisée (jusqu’à l’âge choisi) |
| Médiane | consommation annuelle typique |
| P10 | consommation annuelle dans le mauvais cas |
| Volatilité | variation moyenne de la consommation annuelle par rapport à l’année précédente |
| Objectif | la valeur combinée selon la [formule](#o-score), avec une barre pour la comparaison |
| Taux de prélèvement | prélèvement sur la fortune la **première année de retraite** (coût de la vie et frais de logement moins revenus, **avant impôts**), en pour cent de la fortune liquide au début; médiane sur les trajectoires |
| Fortune finale 1 % … 99 % | fortune nominale à la fin de la simulation aux percentiles 1, 5, 50, 95 et 99 % des trajectoires, déterminée comme dans le résultat de la simulation; aussi pour la situation de départ |
| Au plancher | part des mois dès la retraite pendant lesquels le [plancher](#f-wd-limits) fixe le coût de la vie, en moyenne sur les trajectoires de vérification; «–» pour les stratégies sans plancher (Standard, 3 compartiments avec dépenses comme planifié, plancher 0). La colonne figure pour les deux objectifs |

Sous chaque stratégie figure une **ligne propre** sur toute la largeur du tableau avec les paramètres trouvés (pour la comparaison sans optimisation, les valeurs de la planification) et le bouton «Reprendre dans la planification». Pour laisser de la place au tableau, l’aperçu avec la vérification à droite est masqué dans la section «Optimisation».

- La première ligne **«Situation initiale»** montre votre planification inchangée, sur les mêmes trajectoires de vérification. Vous mesurez à cette aune le gain de l’optimisation.
- Les **lignes grisées** («Limite de ruine non respectée») sont des stratégies qui ne respectent pas votre limite de ruine. Elles restent visibles pour que vous perceviez le coût de la sécurité. Si **aucune** stratégie n’est admissible, la limite n’est pas atteignable avec les moyens disponibles; vous devez alors réduire les dépenses, prendre votre retraite plus tard ou assouplir la limite.
- Le **taux de prélèvement** est une valeur de comparaison, pas un objectif de la recherche. Il montre à quel point une stratégie sollicite la fortune au début: un taux élevé pour une consommation aussi élevée signifie que la stratégie puise davantage dans la fortune et compte moins sur les rentes et le revenu. La règle empirique ‹moins de 4 %› ne vaut que grossièrement (voir [Prélèvement sur la fortune](#r-draw)).
- **Au plancher:** si le coût de la vie se trouve souvent au plancher, la stratégie ne décide presque rien: elle permettrait moins, et le plancher relève les dépenses. Plusieurs stratégies calculent alors pratiquement la même chose. À partir de **50 %** des mois, la remarque «Le plancher détermine le plus souvent les dépenses; la stratégie n’a guère d’effet.» figure sous la stratégie. Vérifiez alors si le plancher est trop élevé ou si les dépenses planifiées dépassent la fortune. Les résultats enregistrés avec des versions antérieures affichent ici «–».
- Si la consommation baisse fortement par rapport à la situation initiale, la sécurité coûte d’autant. Ce n’est **pas une erreur**, c’est la réponse.
- Sous le tableau figurent le nombre de trajectoires pour la recherche et la vérification ainsi que le temps de calcul.

Si la planification a été modifiée après l’optimisation, une remarque apparaît: le résultat ne correspond alors plus à la planification et doit être recalculé.

## @o-apply Reprendre dans la planification
Mots-clés: reprendre, paramètres, définir la stratégie, appliquer

«Reprendre dans la planification» définit dans votre planification:

- la **stratégie** et ses paramètres (par exemple le taux initial et les garde-fous pour Guardrails),
- la **part d’actions** et la **réserve de cash** dès la retraite.
- le commutateur **«Ne jamais vendre les biens immobiliers»**, si la ligne a été calculée avec lui.

La planification est ensuite considérée comme modifiée (non enregistrée). Avec **Annuler** (⌘Z), la reprise peut être annulée. Le résultat de l’optimisation reste visible et valable tant que vous ne modifiez pas la planification autrement.

Vérifiez ensuite le résultat avec une **simulation normale** et, si possible, avec une **autre source de rendements**. Si l’optimum y reste bon, il est robuste.

## @o-saved Enregistrer et rouvrir le résultat
Mots-clés: enregistré, liste, résultats, rouvrir, comparaison des stratégies, redémarrage

Chaque optimisation **terminée** et chaque comparaison des stratégies est classée automatiquement dans la [liste des résultats](#r-list), avec le rapport, les réglages de l’optimisation et l’état de la planification au moment du calcul. Une interruption ou une erreur ne classe rien. Comme les simulations, l’entrée est conservée **localement dans la mémoire de l’app**, pas dans le fichier de planification, et reste après un redémarrage. Si la mémoire de l’app est bloquée, elle reste en mémoire vive jusqu’à la fermeture.

- **Dans la liste**, l’entrée s’appelle «… · Optimisation · …» ou «… · Comparaison des stratégies · …». Elle indique la meilleure stratégie (ses paramètres au survol), sa probabilité de ruine et l’objectif; avec l’objectif «Pas de vente forcée», en plus la part des ventes non planifiées. Renommer, note et suppression fonctionnent comme pour les simulations.
- **Ouvrir** affiche le résultat dans la vue large des résultats avec le même tableau que dans la section «Optimisation», avec la date et le nombre de trajectoires vérifiées.
- **«Reprendre dans la planification»** reprend les paramètres dans la planification ouverte. Si le résultat appartient à un autre état de la planification (autre planification ou modifiée depuis), une remarque apparaît en haut et l’app demande confirmation avant la reprise.
- **À l’ouverture d’une planification**, la section «Optimisation» affiche le résultat enregistré le plus récent qui correspond exactement à cet état de la planification.
- Une optimisation ne peut pas être **comparée** avec des simulations: marquée, elle peut être supprimée, «Comparer» l’ignore.

## @o-pitfalls Pièges de l’optimisation
Mots-clés: suroptimisation, robustesse, risque de modèle, contre-épreuve, avertissement

Une optimisation trouve le meilleur résultat **pour les hypothèses**. Elle ne dit rien sur la justesse des hypothèses.

- **L’optimum ne vaut que pour ces rendements.** Si l’optimisation a été faite avec la série Shiller, le résultat ne vaut pas forcément avec une autre série ou avec des rendements plus bas. **Contre-vérifiez le résultat avec une deuxième source**: optimisez avec le bootstrap et vérifiez avec le modèle paramétrique, ou inversement.
- **Des chiffres précis donnent une fausse impression de précision.** Une part d’actions de 63.4 % n’est pas meilleure que 60 %. Arrondissez à des valeurs compréhensibles et vérifiez dans quelle mesure l’objectif change. Souvent, la surface autour de l’optimum est **plate**.
- **Solutions en bordure:** si un paramètre se trouve en bordure de la plage de recherche (par exemple part d’actions de 100 % ou taux initial de 8 %), c’est le signe que le résultat est extrême. Méfiez-vous-en.
- **Une limite de ruine trop optimiste.** Avec 10 % ou plus, des consommations élevées sont facilement possibles, mais avec une probabilité de 1 sur 10, vous perdez votre fortune.
- **Risque de séquence et longues périodes.** L’estimation de la ruine avec le bootstrap et le modèle paramétrique dépend fortement des hypothèses sur les crises.
- **Éléments non modélisés:** frais de placement, modifications législatives, frais de soins, décès. Voir [Limites du modèle](#limits).
- **Ajustement aux trajectoires aléatoires:** sans vérification indépendante, l’optimisation réagirait aux fluctuations aléatoires. L’application vérifie donc sur d’autres trajectoires. Avec «Toutes les dates de départ historiques», cette indépendance fait défaut.

> Un calcul modèle n’est pas un conseil en placement. Un optimum est un **point de départ pour vos réflexions**, pas un mandat.

## @o-example Exemple d’interprétation
Mots-clés: exemple, interprétation, guide, démarche

Les chiffres sont inventés et servent uniquement à s’exercer à la lecture.

| Stratégie | Ruine | Médiane | P10 | Volatilité | Objectif |
|---|---|---|---|---|---|
| Situation initiale (Standard) | 7.8 % | 72 000 | 72 000 | 0 % | 72 000 |
| Guardrails ★ | 3.9 % | 78 500 | 66 000 | 6 % | 68 000 |
| Taux fixe | 1.1 % | 84 000 | 51 000 | 21 % | 53 000 |

- La situation initiale ne respecte **pas** la limite de ruine de 5 % (7.8 %).
- Guardrails atteint **3.9 %** et fournit en même temps **plus** de consommation en médiane. En contrepartie, la consommation baisse à 66 000 dans le mauvais cas et fluctue de 6 % par année. C’est le prix de la sécurité.
- Le taux fixe serait encore plus sûr, mais fluctue si fortement (21 %) que l’objectif est inférieur à celui de Guardrails.

**Ce que vous en faites:** vérifiez si vous pourriez vivre avec 66 000 dans le mauvais cas (logement, santé, coût de la vie minimal). Si oui, Guardrails est un bon candidat. Recalculez-le ensuite avec une deuxième source de rendements.

## @o-cli Optimisation en ligne de commande
Mots-clés: terminal, optimize, commande, options, script

Le même calcul existe dans le terminal:

```
vivaplan optimize <planification.vplan> [options]
```

| Option | Signification |
|---|---|
| `--goal 1\|2` | objectif: 1 consommation avec risque de ruine limité (standard), 2 pas de vente forcée de biens immobiliers |
| `--min-consumption <pour cent>` | objectif 2: part minimale des dépenses planifiées pour la médiane de la consommation (standard 90) |
| `--max-ruin <pour cent>` | probabilité de ruine maximale |
| `--ruin-age <âge>` | la ruine ne compte que jusqu’à cet âge |
| `--weight-median <0 à 1>` | poids de la médiane dans l’objectif de consommation |
| `--vol-weight <nombre>` | déduction pour la volatilité |
| `--max-vol <pour cent>` | volatilité maximale |
| `--paths`, `--check-paths`, `--samples` | trajectoires de la recherche et de la vérification, points de départ |
| `--seed` | valeur initiale aléatoire |
| `--strategy a,b` | stratégies (`standard`, `three_buckets`, `guardrails`, `fixed_rate`, `vpw`, `endowment`) |
| `--json <fichier>` | enregistrer le résultat en JSON |
| `--compare` | comparer les stratégies sans recherche avec les valeurs de la planification, sur les trajectoires de la simulation |
| `--property planned\|never-sell\|both` | biens immobiliers: comme planifié (par défaut), ne jamais vendre ou les deux (chaque stratégie deux fois) |

Si des réglages sont enregistrés dans la planification, ils servent de point de départ. Chaque option indiquée les remplace. La commande lit toujours les **fourchettes des paramètres** (`optimization.ranges`) dans la planification; le terminal n’a pas d’options pour cela. La sortie montre pour chaque stratégie les paramètres et les indicateurs de la recherche et de la vérification; elle est en anglais (voir [Ligne de commande](#ref-cli)). Avec `--compare`, elle montre pour chaque stratégie les valeurs de la planification et une ligne d’indicateurs; la fortune finale à 1, 5, 50, 95 et 99 % figure toujours.
