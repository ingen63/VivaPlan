# Stratégie de prélèvement

## @s-withdrawal Stratégie de prélèvement: vue d’ensemble
Mots-clés: prélèvement, stratégie, retraite, garde-fous, coût de la vie

La stratégie détermine **comment, après la retraite, l’argent est prélevé sur la fortune et réalloué**. Il existe six stratégies, réparties en deux groupes. La stratégie à 3 compartiments peut en outre être combinée avec Guardrails:

**Les dépenses sont fixes, la fortune fluctue** (décision: comment la fortune est-elle placée?)

- [Standard](#f-wd-standard): réserve de cash plus part d’actions fixe
- [3 compartiments](#f-wd-three_buckets): cash, obligations et actions dans trois compartiments
- [3 compartiments avec Guardrails](#f-wd-three_buckets_guardrails): les compartiments déterminent le placement, les Guardrails les dépenses

**Les dépenses suivent la fortune** (décision: combien est-ce que je dépense?)

- [Taux de prélèvement fixe](#f-wd-fixed_rate): chaque année une part fixe de la fortune
- [Guardrails](#f-wd-guardrails): taux initial avec garde-fous (Guyton-Klinger)
- [VPW](#f-wd-vpw): prélèvement en pourcentage variable, de sorte que la fortune suffise jusqu’à un certain âge
- [Endowment](#f-wd-endowment): prélèvement lissé (Yale/Tobin)

**Sécurité contre confort:** avec les deux premières stratégies, vos dépenses restent comme planifié, mais la fortune peut être épuisée. Avec les quatre autres, la fortune n’est **jamais entièrement épuisée**, mais les dépenses fluctuent avec l’évolution de la bourse. Que vous le souhaitiez ou non est une question personnelle. Avec les [limites du coût de la vie](#f-wd-limits), vous fixez un plancher.

**Quand la stratégie s’applique-t-elle?** Seulement **dès la retraite**, avec deux personnes dès la retraite de la **première** personne. Cela vaut **aussi pour «Standard»**. Auparavant, aucune stratégie ne s’applique: l’application détient autant de cash que saisi sous [Fortune et placement](#s-wealth) (avec le renchérissement) et place le reste selon la quote-part d’actions. Si la deuxième personne travaille encore, son revenu compte dans les «revenus et rentes»: pour les stratégies dont les dépenses suivent la fortune, le coût de la vie est alors plus élevé de ce revenu. S’il disparaît avec la deuxième retraite, le coût de la vie baisse, le prélèvement reste le même.

**Les entrées agissent immédiatement:** pour les quatre stratégies dont les dépenses suivent la fortune (pour 3 compartiments, la règle est plutôt la suivante: les entrées vont uniquement dans le compartiment 3; pour 3 compartiments avec Guardrails, les deux s’appliquent: les Guardrails recalculent le prélèvement, l’argent va dans le compartiment 3), l’application applique la stratégie non seulement en janvier, mais **aussi le mois d’une entrée**: pour un montant unique (par exemple une succession), une prestation de la caisse de pension et le produit d’une vente immobilière. Elle se comporte alors comme en janvier (le prélèvement annuel est recalculé), et cela vaut dès ce mois pour le reste de l’année, sans effet rétroactif. Plusieurs entrées le même mois donnent **une seule** application après toutes les entrées et tous les achats. «Standard» place de toute façon chaque mois.

**Vente avec achat ultérieur:** si l’achat d’un autre bien immobilier est planifié le mois suivant la vente (mois d’achat fixe ou «après la vente»), le produit est **réservé** et la stratégie n’est **pas** appliquée le mois de la vente. Le mois de l’achat, l’achat a lieu d’abord, puis la stratégie s’applique à l’excédent. Pour une vente en décembre et un achat en janvier, l’ordre est: achat, puis l’application habituelle de janvier (une fois). Un produit réservé ne compte **pas** dans la fortune à partir de laquelle les stratégies calculent leur prélèvement. La liste des événements et la section «Immobilier» indiquent ce qui s’est passé («Produit de la vente réservé pour un achat le mois suivant», «Stratégie de prélèvement appliquée après …»).

**Laquelle convient?** Voir [Quelle stratégie convient?](#wd-compare). L’[optimisation](#s-optimize) fournit une réponse chiffrée.

## @wd-compare Quelle stratégie convient?
Mots-clés: comparaison, recommandation, choix, avantages et inconvénients

| Stratégie | Dépenses | Fortune | Adaptée pour | Inconvénient |
|---|---|---|---|---|
| Standard | fixes, comme planifié | peut s’épuiser | planification simple, grand besoin de sécurité pour les dépenses | l’argent peut s’épuiser, aucun mécanisme contre les crises |
| 3 compartiments | fixes, comme planifié | peut s’épuiser | qui veut surmonter les années de crise grâce aux réserves | règles plus compliquées, les paramètres doivent convenir |
| 3 compartiments avec Guardrails | suivent la fortune avec garde-fous | ne s’épuise presque jamais | dépenses planifiables et années de crise surmontées grâce aux réserves | nombreux paramètres, part d’actions plus faible |
| Taux fixe | suivent la fortune | ne s’épuise jamais | règle ‹ne jamais épuiser› la plus simple | fortes fluctuations des dépenses |
| Guardrails | suivent la fortune avec garde-fous | ne s’épuise presque jamais | dépenses planifiables avec correction en cas de crise | réductions de 10 % possibles |
| VPW | suivent la fortune, augmentent avec l’âge | épuisée à l’âge de planification | qui veut consommer sa fortune | les dépenses fluctuent, l’âge de planification est une hypothèse |
| Endowment | suivent la fortune avec retard | ne s’épuise presque jamais | dépenses stables | réagit lentement, peut prélever trop en cas de krach |

**Démarche pratique:** commencez avec «Standard» comme situation initiale, calculez ensuite l’[optimisation](#s-optimize) sur toutes les stratégies et regardez combien de consommation et combien de sécurité apportent les alternatives.

## @f-wd-choice Choisir une stratégie
Mots-clés: choix, carte

Choisissez la stratégie avec les cartes en haut de la section. Chaque carte a un ⓘ avec la description détaillée. Les saisies de la stratégie choisie apparaissent en dessous.

Les stratégies dont les dépenses suivent la fortune, ainsi que 3 compartiments avec Guardrails, fixent **dès la retraite le coût de la vie**. Les dépenses planifiées (section «Revenus et dépenses») ne s’appliquent alors que jusqu’à la retraite.

Voir [Vue d’ensemble](#s-withdrawal) et [Quelle stratégie convient?](#wd-compare).

## @f-wd-standard Standard
Mots-clés: réserve, part d’actions, rééquilibrage

**Les dépenses sont fixes et suivent votre planification.** La fortune est répartie ainsi:

- Le **cash** est maintenu à la [réserve de cash](#f-reserve) (réglable ici; toutes les autres stratégies prennent 3 mois fixes), exprimée en mois de dépenses non couvertes (dépenses moins revenus et rentes).
- Le **reste** est réparti entre actions et obligations selon la [part d’actions](#f-stockshare).
- La proportion entre actions et obligations est rétablie chaque mois (rééquilibrage).

Comme toutes les stratégies, elle ne s’applique qu’à partir de la retraite; auparavant, la règle du cash de [Fortune et placement](#s-wealth) s’applique. Simple, transparente, mais sans protection contre le fait de vendre trop en période de crise.

## @f-wd-three_buckets Stratégie à 3 compartiments
Mots-clés: compartiment, liquidités, obligations, actions, réalimentation, janvier, seuil

**Dès la retraite**, la fortune est répartie en trois compartiments. Les dépenses restent **comme planifié**:

| Compartiment | Contenu | Objectif |
|---|---|---|
| Compartiment 1 | Cash | années × besoin net, couvre les dépenses courantes |
| Compartiment 2 | Obligations | années × besoin net, tampon |
| Compartiment 3 | Actions | le reste |

**Saisies:**

- **Compartiment 1 (années):** combien d’années de besoin net sont détenues en cash. Usuel: 1 à 3.
- **Compartiment 2 (années):** combien d’années en obligations. Usuel: 3 à 8.
- **Ne vendre du compartiment 3 que si le rendement des actions ≥:** critère de rendement. On mesure le **rendement des actions des 12 derniers mois**, mois en cours compris. Pendant les 12 premiers mois de la simulation, les mois antérieurs au départ comptent aussi: chaque trajectoire de rendement les apporte (pour l’historique et le bootstrap, les vrais mois précédents; pour le paramétrique, des mois tirés au hasard; pour les rendements fixes, le taux fixe). S’ils manquent parce que la série temporelle ne commence que plus tard, c’est le rendement depuis le départ qui s’applique. Si le critère n’est pas rempli, seules les **ventes du compartiment 3** lors de la réalimentation de janvier sont bloquées. À 0 %, on ne vend qu’après une année de gains; des valeurs négatives permettent la réalimentation même après de légères pertes.
- **Réserve dans le compartiment 1 en cours d’année (mois):** si le compartiment 1 devient insuffisant en cours d’année (cas C), il y reste, après les dépenses du mois, encore Min(réserve, mois jusqu’à janvier) − 1 besoins mensuels. Standard: 3.

**Règles:**

- Toutes les dépenses sont payées depuis le **compartiment 1**. **Aucun compartiment ne descend au-dessous de 0**; seule exception, voir «tous les compartiments vides».
- **Objectifs** (toujours tirés du besoin net, jamais du solde du compartiment): objectif compartiment 1 = années × besoin net, objectif compartiment 2 = années × besoin net.
- **Réalimentation de janvier (chaque janvier):** si le critère de rendement est rempli, le compartiment 1 est réalimenté depuis le compartiment 2, le reste vient du compartiment 3; ensuite, le compartiment 2 est réalimenté depuis le compartiment 3. S’il n’est pas rempli, rien n’est vendu du compartiment 3, le compartiment 1 n’est réalimenté que depuis le compartiment 2 (si le compartiment 2 ne suffit pas, le compartiment 1 reste sous l’objectif) et le compartiment 2 n’est pas réalimenté. Si le compartiment 1 ou le compartiment 2 dépasse l’objectif, l’excédent va dans le compartiment 3.
- **A · Début de la phase de prélèvement, retrait en capital CP, redémarrage:** au départ (dès que la **première** personne est à la retraite), toute la fortune liquide va d’abord dans le compartiment 3, de même qu’un retrait en capital ultérieur de la CP. Ensuite, la réalimentation se déroule comme en janvier, **sans** critère de rendement et (en cours d’année) sans écriture d’excédent dans le compartiment 3. Les objectifs sont réduits des mois déjà écoulés de l’année ((années × 12 − mois) / 12 × besoin net; départ en mars: 2 mois). A s’exécute à chaque déclenchement, plusieurs le même mois donnent une seule exécution.
- **B · Entrées externes:** les entrées uniques, les versements 3a, une vente immobilière sans achat ultérieur et le produit des ventes forcées vont **uniquement dans le compartiment 3**. Les compartiments 1 et 2 ne sont pas réalimentés.
- **C · Anticipation (chaque mois):** l’application vérifie si le compartiment 1 couvre les dépenses du mois (courantes et uniques, impôts compris). Sinon, le déroulement dépend du critère de rendement:
  - **C1 (critère rempli):** déroulement comme la réalimentation de janvier: compartiment 1 depuis le compartiment 2, le reste depuis le compartiment 3, puis compartiment 2 depuis le compartiment 3. Les objectifs sont réduits des mois écoulés depuis janvier ((années × 12 − mois) / 12 × besoin net; avril = 3 mois, décembre = 11), l’objectif du compartiment 1 étant en outre augmenté des **dépenses uniques de ce mois**. Il n’y a **pas d’écriture d’excédent** dans le compartiment 3. Exemple avril, compartiment 1 = 2 ans, compartiment 2 = 1 an: objectif compartiment 1 = 1.75 × besoin net + dépenses uniques, objectif compartiment 2 = 0.75 × besoin net.
  - **C2 (critère non rempli):** le compartiment 1 est réalimenté depuis le **compartiment 2, puis le compartiment 3**, de sorte qu’il couvre les dépenses et contienne encore ensuite (Min(réserve, mois jusqu’à janvier) − 1) besoins mensuels («mois jusqu’à janvier» compte le mois en cours: décembre = 1). Le compartiment 2 n’est pas réalimenté.
  - Si, après C1, le compartiment 1 ne suffit toujours pas pour les paiements du mois, C2 suit. Si les compartiments 2 et 3 ne suffisent pas, la vente forcée suit (voir «tous les compartiments vides»).
- **Dépenses uniques et objectifs:** lors de la réalimentation de janvier et pour A, l’objectif du compartiment 1 est également augmenté des dépenses uniques du mois, afin qu’une grosse dépense unique en janvier ne déclenche pas en plus C. Des **écritures d’excédent** dans le compartiment 3 n’existent que lors de la réalimentation de janvier.
- **Biens immobiliers:** un produit de vente pour lequel un achat est planifié le mois suivant est **réservé** (il n’appartient à aucun compartiment). Lors de l’achat, la réservation paie d’abord les fonds propres, un solde éventuel vient du compartiment 1 (par C); un excédent de la réservation va dans le compartiment 3. Un achat en janvier après une vente en décembre est traité avant la réalimentation de janvier.
- **Ordre dans le mois:** revenus réguliers dans le compartiment 1, entrées directement à leur destination (compartiment 3 ou réservation, jamais d’abord dans le compartiment 1), achats immobiliers (fonds propres tirés de la réservation; un solde compte comme dépense unique du mois et ne grève pas le compartiment 1 au préalable), réalimentation de janvier ou A, anticipation (C) avec **toutes** les dépenses du mois (courantes, uniques, solde d’achat, amortissements et paiements uniques de l’hypothèque, impôts), puis les dépenses sont payées depuis le compartiment 1. Les transferts sont techniquement comptabilisés après l’étape de rendement du mois; le tableau montre le solde avant les dépenses du mois.
- **Tous les compartiments vides:** si les compartiments 1, 2 et 3 ne suffisent pas, un bien immobilier est vendu de force (produit dans le compartiment 3, puis C de nouveau). Le critère de vente forcée de la logique hypothécaire s’applique parallèlement et peut se déclencher plus tôt. S’il n’y a plus de bien immobilier, le compartiment 1 peut devenir négatif, les compartiments 2 et 3 sont mis en veille, et tous les revenus et entrées vont dans le compartiment 1. Dès que la fortune est de nouveau positive, A redémarre.

**Besoin net (par année, valeurs planifiées de l’année en cours):** coût de la vie + logement + versement au pilier 3a + impôt sur le revenu − revenus réguliers (rente AVS, rente CP, revenu de l’activité lucrative tel qu’indiqué). Ne sont pas comptés les dépenses uniques, acquisitions, voyages, amortissements, achats immobiliers, impôts spéciaux (impôt sur les gains immobiliers, sur les prestations en capital et sur la fortune) et entrées uniques (retrait en capital, succession, produits de vente). L’impôt sur le revenu est l’impôt sur le revenu annuel (rentes et revenu de l’activité lucrative). Si le besoin net est nul ou négatif, tous les objectifs sont nuls. Lors d’un déclenchement A, les mois dès le déclenchement sont extrapolés sur 12 mois, **impôt sur le revenu compris**. Ce besoin net vaut pour toutes les réalimentations jusqu’au prochain janvier ou au prochain déclenchement A, y compris pour l’anticipation C. Si les dépenses augmentent (renchérissement), les compartiments augmentent avec elles. Dans la vue des résultats, le tableau «Stratégie à 3 compartiments» montre le calcul (dépenses, revenus déduits, besoin net, objectifs), le critère de rendement, le cas déclenché (janvier, A, B, C) et tous les transferts.

**Idée:** après un krach boursier, il n’est pas nécessaire de vendre des actions à des cours bas. Les compartiments 1 et 2 permettent de tenir plusieurs années, jusqu’à ce que les actions se soient redressées. Le coût en est la part d’actions plus faible (plus d’obligations et de cash) et donc, en tendance, un rendement moindre.

## @f-wd-three_buckets_guardrails 3 compartiments avec Guardrails
Mots-clés: combinaison, compartiments, garde-fous, dépenses, placement, besoin net

Sur la carte «3 compartiments», vous choisissez sous **Dépenses dès la retraite** entre «comme planifié» et «piloter avec Guardrails». Avec Guardrails:

- **Les Guardrails pilotent les dépenses.** Le taux initial, le renchérissement, les garde-fous et les [limites](#f-wd-limits) agissent exactement comme pour la stratégie [Guardrails](#f-wd-guardrails). Pour une même évolution de la fortune, le prélèvement est identique dans les deux.
- **Les compartiments pilotent le placement.** Toutes les règles de la [stratégie à 3 compartiments](#f-wd-three_buckets) s’appliquent sans changement (réalimentation de janvier, A, B, C, critère de rendement, vente forcée, «tous les compartiments vides»).
- **Le besoin net et les objectifs des compartiments** calculent avec le coût de la vie des Guardrails au lieu des dépenses planifiées, après plancher et plafond. Sans limite et sans frais de logement, le besoin net est le prélèvement des Guardrails plus l’impôt sur le revenu. Le tableau «Stratégie à 3 compartiments» mentionne «Guardrails» pour le coût de la vie.
- **Ordre en janvier:** d’abord les Guardrails (renchérissement, garde-fous), puis le besoin net et les objectifs, puis la réalimentation. Si le garde-fou réduit le prélèvement de 10 %, les objectifs des compartiments 1 et 2 diminuent aussi de 10 %.
- **Départ:** les compartiments et les Guardrails démarrent ensemble avec la retraite de la première personne. Le prélèvement initial est fixé d’abord, puis A répartit la fortune.
- **Entrées:** les Guardrails recalculent le prélèvement, l’argent va dans le compartiment 3. Les objectifs des compartiments restent inchangés jusqu’au prochain janvier; l’anticipation (C) calcule toutefois immédiatement avec le nouveau coût de la vie mensuel.
- **Deux mesures du rendement:** les Guardrails suspendent le renchérissement après une année de pertes pendant l’**année civile**, le critère de rendement des compartiments mesure les **12 derniers mois**. En janvier, c’est la même chose.

**Exemple:** taux initial 4 %, fortune 1 mio, compartiment 1 = 1 an, compartiment 2 = 2 ans, pas de rentes. Le prélèvement est de 40'000 par année, le compartiment 1 contient 40'000, le compartiment 2 80'000, le reste est en actions. Après un krach, le garde-fou réduit à 36'000: au janvier suivant, les objectifs sont de 36'000 et 72'000.

**Idée:** les compartiments évitent de vendre des actions à des cours bas, les garde-fous empêchent de prélever trop. Dans l’[optimisation](#s-optimize), la combinaison s’appelle «3 compartiments + Guardrails».

## @f-wd-fixed_rate Taux de prélèvement fixe
Mots-clés: pour cent, taux, 4 pour cent, Trinity, taux constant

Chaque année, **le même taux** de la fortune liquide est prélevé. Les dépenses **suivent** ainsi la fortune.

- Chaque **janvier** (et à la retraite), le prélèvement annuel est refixé à **taux × fortune liquide**.
- Le prélèvement correspond à **coût de la vie + frais de logement + versement au pilier 3a moins revenus et rentes** (avant impôts). Le coût de la vie en découle. Les dépenses planifiées de la section «Revenus et dépenses» ne s’appliquent que jusqu’à la retraite.
- **La fortune n’est jamais entièrement épuisée**, car on n’en prélève toujours qu’une part. En contrepartie, les dépenses fluctuent fortement: si la fortune baisse de 30 %, le prélèvement baisse de 30 %.

**Quel taux?** La ‹règle des 4 %› souvent citée provient d’une étude américaine sur 30 ans avec un prélèvement fixe adapté au renchérissement (**et non** avec un taux appliqué à la fortune actuelle). Pour un taux appliqué à la fortune actuelle, 3 % à 5 % sont usuels. Attention: le taux s’applique au **prélèvement**, c’est-à-dire à vos dépenses **moins** l’AVS, la rente de la caisse de pension et le revenu. Le coût de la vie est donc taux × fortune **plus** rentes et revenu. Avec des rentes élevées, les dépenses dépendent donc beaucoup moins de la fortune que le taux ne le laisse supposer.

Le **plancher** des [limites du coût de la vie](#f-wd-limits) empêche des dépenses trop basses.

## @f-wd-guardrails Guardrails (Guyton-Klinger)
Mots-clés: garde-fous, taux initial, supérieur, inférieur, ajustement, Guyton, Klinger

Le prélèvement commence avec un **taux initial** et est adapté chaque année au renchérissement. Des **garde-fous** interviennent lorsque le prélèvement devient trop élevé ou trop bas par rapport à la fortune.

**Saisies** (valeur standard entre parenthèses):

- **Taux initial (5 %):** part de la fortune liquide prélevée à la retraite (de la première personne). Un produit de vente réservé pour un achat n’en fait pas partie, ni pour les garde-fous.
- **Garde-fou supérieur (20 %):** si le taux actuel dépasse le taux initial de cette part, le prélèvement est réduit.
- **Garde-fou inférieur (20 %):** si le taux actuel est inférieur au taux initial de cette part, le prélèvement est augmenté.
- **Ajustement (10 %):** de combien le prélèvement est réduit ou augmenté lorsqu’un garde-fou est franchi.

**Déroulement chaque janvier:**

1. Le prélèvement de l’année précédente est adapté au **renchérissement**, **sauf** après une année de pertes sur les actions, tant que le taux actuel est supérieur au taux initial (pas d’adaptation au renchérissement en période de crise).
2. Ensuite, le taux actuel (prélèvement / fortune) est comparé aux garde-fous:
   - **Au-dessus de taux initial × (1 + garde-fou supérieur):** le prélèvement est réduit de l’ajustement.
   - **Au-dessous de taux initial × (1 − garde-fou inférieur):** le prélèvement est augmenté de l’ajustement.

**Exemple:** taux initial 5 %, garde-fou supérieur 20 %, ajustement 10 %. Si la fortune baisse au point que le taux actuel dépasse 6 %, l’application réduit le prélèvement de 10 %.

**Idée:** vous dépensez beaucoup au début (le taux initial est souvent plus élevé qu’avec un taux fixe), mais acceptez d’économiser en période de crise. Les réductions sont rares et faibles, les dépenses restent **plus planifiables** qu’avec un pur taux fixe.

Comme pour le taux fixe, le prélèvement correspond à **coût de la vie + frais de logement moins revenus et rentes** (avant impôts). Des [limites du coût de la vie](#f-wd-limits) sont possibles.

## @f-wd-vpw Prélèvement en pourcentage variable (VPW)
Mots-clés: VPW, âge de planification, rendement réel, annuité, Bogleheads

Chaque janvier, on prélève la part de la fortune liquide qui, à un **rendement réel** supposé, l’**épuise régulièrement jusqu’à l’âge de planification**. Il s’agit d’une **annuité**.

**Saisies:**

- **Âge de planification** (âge de la personne principale, standard 100): la fortune doit suffire jusque-là. Choisissez-le suffisamment élevé pour que la sécurité demeure.
- **Rendement réel** (standard 2 %): le rendement annuel réel de la fortune que la stratégie suppose. Une hypothèse trop élevée conduit à des prélèvements qui épuisent la fortune plus rapidement.

**Caractéristiques:**

- Le **taux prélevé augmente avec l’âge**, car le temps restant diminue (par exemple avec un rendement réel de 0 % et encore 25 ans: 4 %, avec 10 ans: 10 %).
- À l’âge de planification, la fortune serait épuisée. La stratégie est donc conçue pour ‹l’argent doit être épuisé à la fin›, pas pour transmettre un héritage.
- Les dépenses suivent la fortune et fluctuent en conséquence. Un plancher limite cela.

## @f-wd-endowment Règle Endowment (Yale/Tobin)
Mots-clés: lissage, taux cible, Yale, Tobin, lissé

Le prélèvement est une **moyenne pondérée** de l’année précédente (avec renchérissement) et d’un taux appliqué à la fortune actuelle:

> Prélèvement en janvier = (1 − lissage) × prélèvement de l’année précédente avec renchérissement + lissage × taux cible × fortune actuelle

**Saisies:**

- **Taux cible** (standard 4 %): taux dont le prélèvement se rapproche à long terme.
- **Lissage** (standard 30 %): poids de la fortune actuelle. À 100 %, il s’agit du [taux fixe](#f-wd-fixed_rate); à 0 %, seule l’année précédente est reportée avec le renchérissement (comme un prélèvement fixe).

**Effet:** avec un faible lissage, les dépenses changent peu d’une année à l’autre. Elles ne suivent toutefois la fortune que **lentement**. Si la fortune baisse fortement, on prélève trop à court terme, ce qui ne se compense qu’au fil des années. Les fondations et les universités utilisent cette règle pour obtenir des distributions stables.

## @f-wd-limits Limites du coût de la vie
Mots-clés: plancher, plafond, floor, ceiling, dépenses minimales

Pour les stratégies qui fixent elles-mêmes le coût de la vie (taux fixe, Guardrails, VPW, Endowment, 3 compartiments avec Guardrails), vous pouvez définir un **plancher** et un **plafond**. Tous deux sont des **parts des dépenses planifiées**. **0 signifie pas de limite.**

- **Plancher:** le coût de la vie ne descend jamais au-dessous de cette part des dépenses planifiées (aux prix actuels, adaptées au renchérissement). Exemple: 80 % signifie que vous dépensez au moins 80 % de ce que vous avez planifié, même si la stratégie en permettait moins. **La fortune peut alors être épuisée.** Le plancher rend visible le prix de la sécurité des dépenses.
- **Plafond:** le coût de la vie ne dépasse jamais cette part. Exemple: 130 % empêche que vous dépensiez nettement plus que prévu lors des bonnes années.

Les limites sont un moyen de garantir des **besoins fixes** (logement, santé) face au prélèvement flexible. Dans l’[optimisation](#s-optimize), les limites restent comme dans la planification.

Les limites ne s’appliquent pas à «Standard» ni à «3 compartiments» avec dépenses comme planifié, car les dépenses y sont de toute façon fixes.
