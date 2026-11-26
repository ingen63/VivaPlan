# Lire les résultats

## @r-overview Résultats: vue d’ensemble
Mots-clés: taux de réussite, indicateurs, percentiles, fortune finale, épuisement, vue des résultats

Après «Lancer la simulation», l’application affiche une **barre de progression** avec le temps écoulé et le temps restant estimé, ainsi qu’un bouton **Interrompre**. Ensuite, la vue des résultats s’ouvre. Son en-tête indique d’abord le **nom du résultat** tel qu’il est classé dans la [liste](#r-list) (par exemple ‹Beispiel · MC hist. · Standard · 16.11.26, 14:05›), puis le type de simulation (‹50'000 simulations› ou ‹Simulation unique · …›). Un nom long est raccourci; le nom complet apparaît lorsque vous pointez dessus; après un renommage dans la liste, le nouveau nom s’applique. La vue comporte plusieurs onglets: **Vue d’ensemble**, **Graphiques**, **Simulations individuelles** (seulement avec plusieurs simulations), **Scénarios** (seulement avec des variations) et **Valeurs annuelles**. Tous les graphiques avec un axe du temps peuvent être zoomés: **tracer une plage de temps en maintenant le bouton de la souris enfoncé** (elle est mise en évidence; les sélections de moins d’un an comptent comme un clic et sont ignorées), ou utiliser les boutons **Tout, 10 ans, 5 ans et 2 ans**. Les boutons fixent la largeur de la plage **centrée sur le milieu de la plage actuelle** (sans zoom: de toute la durée). Si la plage dépasse le début ou la fin, elle est décalée, la largeur reste; si la durée est plus courte, vous voyez tout. Après une sélection à la souris, aucun bouton n’est actif, les graphiques affichent «Personnalisé» avec la plage. Le zoom s’applique **à tous les graphiques des résultats ensemble**, dans tous les onglets et dans la comparaison, et est conservé lors du changement d’onglet; les mini-graphiques de l’éditeur ont leur propre zoom (dès le départ). Les graphiques des scénarios n’ont pas d’axe du temps et donc pas de zoom. En haut à droite, vous passez de **Nominal** à **Pouvoir d’achat actuel**, voir [Rendement, inflation et valeurs réelles](#ret-concepts); à l’ouverture des résultats, le commutateur est sur **Pouvoir d’achat actuel**.

**Résultat en mots:** tout en haut, la vue d’ensemble résume le résultat en deux ou trois phrases. Avec plusieurs simulations: ‹Dans 87 % (moyen) des simulations, la fortune suffit jusqu’en 12.2055 (âge 95.0).› Le classement entre parenthèses suit les couleurs du taux de réussite: **élevé** dès 95 %, **moyen** de 80 % à moins de 95 %, **bas** au-dessous de 80 %; c’est la valeur affichée, arrondie à 0.1 %, qui fait foi. Si l’argent s’épuise dans une simulation, une deuxième phrase indique l’âge dans la simulation la plus défavorable (comme «Épuisement le plus précoce»). Avec une seule simulation, le texte indique si la fortune suffit jusqu’à la fin et à combien elle s’élève alors (nominal ou en pouvoir d’achat actuel, selon le commutateur), ou quand elle est épuisée. Suit une proposition pour la prochaine étape: au-dessous de 95 % ou si la fortune est épuisée, l’[optimisation](#s-optimize), sinon les [scénarios](#s-scenarios). Le texte est une aide à la lecture, pas une évaluation de votre situation.

**Avec une seule simulation** (rendements fixes, évolution historique), la vue d’ensemble affiche cinq indicateurs: la fortune à la retraite, la fortune à la fin, jusqu’à quand l’argent suffit, la rente CP et le total des impôts estimés ([Quelle est la précision de l’impôt?](#tax-accuracy)). Suivent l’évolution de la fortune et la liste des événements.

**Avec plusieurs simulations** (Monte Carlo, toutes les dates de départ), la vue d’ensemble affiche:

- **Taux de réussite:** part des simulations dans lesquelles la fortune suffit jusqu’à la fin. Vert dès 95 %, rouge au-dessous de 80 %.
- **Fortune finale** à la médiane, aux 5e, 1er, 95e et 99e percentiles et dans la **pire simulation**.
- **Épuisement le plus précoce:** l’âge le plus précoce auquel l’argent était épuisé dans une simulation quelconque.
- la [fourchette de la fortune totale](#r-fan) et le [tableau des percentiles](#r-percentiles).

**Quel taux de réussite est suffisant?** Ce n’est pas une question de calcul, mais une question de votre goût du risque. 90 % à 95 % sont considérés comme corrects pour une planification prudente. Notez qu’un taux de réussite de 95 % signifie que l’argent s’épuise dans 1 avenir simulé sur 20. Ce n’est qu’une estimation, voir [Pièges](#ret-pitfalls).

**‹Réussite› signifie:** la fortune (liquide et immobilière) n’est pas épuisée jusqu’à la fin de la simulation. Cela ne signifie **pas** que la vie a été agréable: une simulation peut ‹réussir› et pourtant se terminer de justesse.

## @r-percentiles Fortune finale par percentile
Mots-clés: percentile, médiane, 1er percentile, 99e percentile, pire simulation, quantile

Les simulations sont triées selon leur **fortune finale**. Le **n-ième percentile** est la fortune finale au-dessous de laquelle se situent n % des simulations.

| Ligne | Signification |
|---|---|
| Pire simulation | la simulation avec la fortune finale la plus basse |
| 1er percentile | seulement 1 % des simulations se terminent plus mal, ‹un très mauvais cas› |
| 5e percentile | seulement 5 % se terminent plus mal, ‹un mauvais cas› |
| 50e (médiane) | la moitié se termine mieux, la moitié plus mal |
| 95e et 99e percentile | très bons cas |

Le tableau indique pour chaque ligne la **simulation**, par exemple le mois de départ historique ou le numéro de la trajectoire. Une fortune finale **négative** signifie que l’argent **s’est épuisé avant la fin** et que des dettes (dépenses non couvertes) se sont ensuite accumulées.

**Classement selon l’affichage:** Les simulations sont classées selon la fortune finale de l’affichage choisi: en **pouvoir d’achat actuel** selon la fortune finale réelle, en **nominal** selon la nominale. L’inflation diffère d’une simulation à l’autre; une simulation à forte inflation finit haut en nominal mais bas en termes réels. Un percentile peut donc correspondre à une **autre simulation** selon l’affichage, mais dans les deux affichages les valeurs croissent avec le percentile. Indicateurs, tableau des percentiles, ventes immobilières, [simulations individuelles](#r-runs) et valeurs annuelles montrent toujours les simulations du classement choisi; le changement d’affichage reclasse. Les résultats enregistrés plus anciens sans ce classement montrent, en pouvoir d’achat actuel, la fortune finale réelle des simulations du classement nominal, avec une remarque; recalculés, ils sont classés en termes réels.

> La **médiane** n’est **pas** un cas normal réaliste au sens d’une prévision. Elle dit seulement que la moitié des avenirs simulés se terminent mieux. Pour la planification, ce sont les percentiles **inférieurs** qui comptent, car c’est là que se trouve le risque.

## @r-fan Fourchette de la fortune totale
Mots-clés: éventail, bande, percentiles, fourchette, pire valeur, cône, médiane

Le graphique montre comment la **fortune totale** se **répartit** dans le temps. Chaque mois, sur toutes les simulations, sont représentées:

- la **surface foncée**: 25e à 75e percentile (la moitié centrale des simulations)
- la **surface claire**: 5e à 95e percentile (neuf simulations sur dix)
- la **ligne continue**: médiane
- les **lignes en tirets**: 1er et 99e percentile (dans la légende avec un symbole de ligne en tirets)
- la **ligne rouge**: la **pire valeur** du mois

La ligne rouge est, pour chaque mois, la valeur la plus basse de toutes les simulations. Il peut s’agir d’une autre simulation d’un mois à l’autre; ce n’est donc pas une trajectoire unique, mais la limite inférieure de toutes les trajectoires. Avec la souris, vous voyez les valeurs dans l’infobulle.

**Lecture:** la fourchette s’ouvre dans le temps comme un entonnoir. Plus la bande inférieure est basse, plus le danger est grand. Si le 5e percentile touche la ligne zéro, l’argent s’épuise dans plus de 5 % des simulations.

## @r-survival Probabilité que l’argent suffise
Mots-clés: survie, probabilité de réussite, épuisement, évolution

La part des simulations dans lesquelles la fortune n’est pas encore épuisée **jusqu’à ce moment**. La courbe ne fait que baisser, elle ne monte jamais. Le taux de réussite figure au bord droit.

Une courbe qui baisse **tôt** indique un risque dès les premières années de la retraite (risque de séquence). Une courbe qui ne baisse que tard est un risque de longévité. Par défaut, l’axe affiche **80–100 %**, où se trouvent la plupart des courbes et où les différences deviennent visibles; faites attention à l’échelle. Le commutateur au-dessus du graphique passe à **0–100 %**. Le choix vaut dans le résultat individuel et dans la [comparaison](#r-compare), jusqu’à ce que vous le changiez. Si une courbe descend au-dessous de 80 %, elle est coupée en bas avec l’échelle 80–100 %, et une remarque sous le graphique indique la valeur la plus basse. L’infobulle montre toujours la vraie valeur.

## @r-hist Répartition de la fortune finale
Mots-clés: histogramme, répartition, fréquence, repères, retraite

Un **histogramme**: chaque colonne compte les simulations dont la fortune finale tombe dans cette classe. Les repères en tirets montrent les 1er, 5e, 50e, 95e et 99e percentiles (légende sous le diagramme). La plage va du 1er au 99e percentile; les simulations plus mauvaises ou meilleures se trouvent dans les classes extrêmes. La pire simulation figure dessous dans la remarque.

Une répartition **asymétrique** avec une longue traîne à droite est typique: peu de simulations se terminent très bien, la plupart entre les deux. L’extrémité inférieure montre jusqu’où cela peut aller mal.

Le deuxième graphique montre la répartition de la **fortune à la retraite**. C’est la situation initiale de la phase de prélèvement: une fortune basse à la retraite détermine souvent déjà largement le résultat.

## @r-depletion Âge à l’épuisement
Mots-clés: épuisement, épuisé, âge, barres

Par année d’âge, le nombre de simulations dans lesquelles la fortune est épuisée **cette année-là**. Le graphique n’apparaît que s’il y a des épuisements.

Utile pour situer le risque selon l’âge: si l’argent ne s’épuise que rarement et seulement après 90 ans, c’est un autre risque que s’il s’épuise à 75 ans.

## @r-charts Graphiques
Mots-clés: diagrammes, onglet, graphique

L’onglet «Graphiques» montre les diagrammes en grand format:

- Avec **une simulation:** [Évolution de la fortune](#r-wealth), [Fortune liquide par placement](#r-comp), [Revenus et dépenses](#r-flows), [Prélèvement sur la fortune](#r-draw), [Immobilier](#r-prop), [Prévoyance](#r-pk) et [Actions et obligations](#r-markets).
- Avec **plusieurs simulations:** [Fourchette](#r-fan), [Probabilité que l’argent suffise](#r-survival), [Répartition de la fortune finale](#r-hist) et [Âge à l’épuisement](#r-depletion).

Chaque graphique a un ⓘ au titre. Avec **F1** au-dessus d’un graphique, vous obtenez l’explication correspondante.

**Ligne épaisse à 0:** dans tous les graphiques avec un axe des valeurs qui contient 0, la ligne à 0 est plus épaisse que le reste de la grille, aussi dans la comparaison et dans le mini-graphique de l’éditeur. Au-dessous de cette ligne, l’argent est épuisé. Dans la [répartition de la fortune finale](#r-hist), elle est verticale pour une fortune finale de 0. Si 0 se situe en dehors de la plage affichée, la ligne manque.

## @r-runs Simulations individuelles
Mots-clés: simulation, détail, simulation de percentile, pire simulation, trajectoire, simulation unique

Avec plusieurs simulations, vous pouvez examiner **des simulations individuelles** en détail. Choisissez en haut laquelle:

- **Pire simulation**, **1er**, **5e**, **50e**, **95e** ou **99e percentile**: la simulation dont la **fortune finale** correspond à ce percentile, dans le classement de l’affichage choisi (voir [Fortune finale par percentile](#r-percentiles)).

Dessous, vous voyez les graphiques de cette simulation, ceux que montre une simulation unique, et la liste des événements. Vous reconnaissez ainsi **pourquoi** une simulation se termine mal: krach boursier au début, forte inflation, vente forcée du bien immobilier.

La **comparaison** du 5e avec le 95e percentile est particulièrement instructive: en quoi les rendements des dix premières années de retraite diffèrent-ils? C’est le **risque de séquence** en chiffres. L’application recalcule exactement les simulations de détail.

## @r-wealth Évolution de la fortune
Mots-clés: fortune totale, empilé, liquide, immobilier, prévoyance

La **fortune totale** d’une simulation sous forme de surface empilée en trois parties:

- **Fortune liquide** (actions, obligations, cash)
- **Immobilier (net)**, c’est-à-dire valeur vénale moins hypothèque
- **Prévoyance** (caisse de pension, pilier 3a)

Des lignes verticales marquent les événements (retraite, début de l’AVS, achat, vente, épuisement). Le pointeur de la souris montre les valeurs du mois. Les **sauts** ont généralement une raison: le retrait en capital de la caisse de pension transfère de la prévoyance vers la fortune liquide, la vente d’une maison transfère de l’immobilier vers la fortune liquide.

## @r-comp Fortune liquide par placement
Mots-clés: actions, obligations, cash, parts, composition

Comment la **fortune liquide** se répartit entre actions, obligations et cash. Avec les boutons en haut, vous affichez les **montants** ou les **parts en %**.

La part montre si la stratégie agit comme vous l’attendez. Avec la [stratégie à 3 compartiments](#f-wd-three_buckets), vous voyez les compartiments, avec «Standard» la part d’actions fixe et la réserve.

## @r-flows Revenus et dépenses par année
Mots-clés: revenus, dépenses, impôts, frais de logement, coût de la vie, colonnes

Par **année civile**, deux colonnes:

- **à gauche les revenus:** revenu de l’activité lucrative, rente CP, AVS
- **à droite les dépenses:** coût de la vie, frais de logement, impôts estimés

Là où la colonne de droite est plus haute que celle de gauche, **la fortune est mise à contribution**. La première et la dernière année sont incomplètes, car la simulation commence ou se termine en cours d’année.

Le graphique montre bien comment les rentes comblent l’écart dès la retraite et quelle est encore son ampleur.

## @r-draw Prélèvement sur la fortune
Mots-clés: taux de prélèvement, durable, 4 pour cent, règle empirique, prélèvement

Par année: **dépenses, impôts compris, moins revenus**, par rapport à la fortune liquide. La ligne montre le **taux de prélèvement**.

Une règle empirique veut qu’un taux **inférieur à 4 %** soit durable. Cela ne vaut que grossièrement et pour les conditions américaines sur 30 ans. Si le taux augmente nettement au fil des années, parce que la fortune diminue ou que les dépenses croissent, c’est un signal d’alarme. Pour les stratégies qui suivent la fortune (taux fixe, VPW, …), vous voyez ici leur effet.

## @r-prop Immobilier
Mots-clés: valeur vénale, hypothèque, fonds propres, maison

Les biens immobiliers d’une simulation: **valeur vénale**, **hypothèque** et **fonds propres** (valeur vénale moins hypothèque). Le graphique n’apparaît que s’il y a des biens immobiliers. Les sauts montrent un achat, une vente ou le remboursement d’une hypothèque.

## @r-pk Prévoyance
Mots-clés: caisse de pension, pilier 3a, avoir

Évolution de la **caisse de pension** et du **pilier 3a** d’une simulation. La caisse de pension croît par les cotisations et la rémunération jusqu’à la prestation, puis tombe à la part de rente. Le 3a n’est visible qu’en cas d’amortissement indirect. Le graphique n’apparaît que s’il y a des avoirs de prévoyance.

## @r-markets Actions et obligations
Mots-clés: indice, rendement, évolution, évolution du marché, indice des prix, renchérissement, inflation, logarithmique, pour cent

Montre l’**évolution des rendements de cette simulation** sous forme d’indice, **départ = 100**, avec les distributions réinvesties. L’**inflation** (indice des prix) suit comme les actions et les obligations, en tirets. L’infobulle indique en plus la variation des douze derniers mois, entre parenthèses pour l’inflation.

Deux commutateurs modifient la représentation: **Performance (départ = 100)** ou **Variation en %** (par rapport au même mois de l’année précédente). Pour la performance, vous choisissez **Linéaire** ou **Logarithmique**; en logarithmique, des écarts égaux montrent des variations en pour cent égales. La vue en pour cent est toujours linéaire. La ligne épaisse se situe à **100** pour la performance (état au départ), à **0 %** pour la variation en %.

Vous reconnaissez ainsi **ce qui s’est passé sur le marché dans cette simulation**: une baisse au départ, une longue phase latérale ou une forte inflation. Cela explique souvent pourquoi une simulation se termine bien ou mal.

## @r-params Paramètres
Mots-clés: paramètres, résumé, saisies, état de la simulation, reproduire

L’onglet **«Paramètres»** montre, **en lecture seule**, avec quelles saisies le résultat a été calculé: ménage (mois de naissance, sans noms), jalons, prévoyance, revenus et dépenses, fortune et placement, source de rendements et série temporelle (avec valeur initiale aléatoire), stratégie de prélèvement avec ses paramètres, immobilier, hypothèses fiscales et variations. C’est l’**état de la simulation** qui s’applique, et non l’état actuel dans l’éditeur: si vous modifiez un chiffre après le calcul, l’onglet du résultat calculé reste inchangé. Pour les résultats enregistrés, c’est la planification de l’époque qui figure; si elle manque, «Non disponible» est affiché.

## @r-events Événements
Mots-clés: liste, achat, vente, vente forcée, retraite, épuisement

La liste de tous les **événements** d’une simulation avec la date et l’âge: retraite, début de l’AVS, retrait en capital avec impôt sur les prestations en capital, achats et ventes de biens immobiliers (y compris les **ventes forcées**), périodes hypothécaires, le **passage de l’amortissement indirect à l’amortissement direct** au début de la rente AVS, le **remboursement unique** à l’achat lorsque l’hypothèque doit être réduite pour la capacité financière, retrait du 3a, montants uniques et l’**épuisement** de la fortune. Les **écritures de la stratégie à 3 compartiments** (réalimentations, entrées, réservations de produits de vente) ne figurent pas dans cette liste, mais de manière dépliable dans les [valeurs annuelles](#r-table); les messages de la phase de déficit («tous les compartiments vides») restent ici.

Tous les montants figurent **avec la monnaie** (monnaie du projet).

L’**événement de prélèvement** pour des stratégies comme Guardrails montre les réductions et les augmentations. Les ventes forcées et les épuisements sont des signaux indiquant que la planification ne tient pas dans cette simulation.

## @r-property Immobilier (événements)
Mots-clés: achat, vente, vente forcée, renouvellement de l’hypothèque, paiement unique, capacité financière, taux d’intérêt, fonds propres

Dans la vue d’ensemble d’une simulation unique (et sous «Simulations individuelles»), la section **«Immobilier»** liste tous les événements immobiliers par ordre chronologique, avec le moment (année et âge), l’événement, le bien immobilier et les montants:

- **Achat:** prix d’achat, fonds propres (prix − hypothèque) et hypothèque.
- **Vente / vente forcée:** produit de la vente, hypothèque remboursée, impôt sur les gains immobiliers et **produit net**; pour la vente forcée, le motif («Fortune < 0» ou «critère de vente forcée»), plus une remarque indiquant si le produit a été réservé pour un achat le mois suivant ou si la [stratégie de prélèvement](#s-withdrawal) a été appliquée immédiatement.
- **Renouvellement de l’hypothèque:** capacité financière, hypothèque avant et après, paiement unique pour rétablir la capacité financière et taux d’intérêt de la nouvelle période; si l’hypothèque n’est pas renouvelée, la vente forcée figure avec son motif.

Sans événements, «Aucun événement immobilier» y figure. Avec plusieurs simulations, l’indicateur **«Ventes forcées»** de la vue d’ensemble montre dans combien de simulations (nombre et part, dont faute de capacité financière) des biens immobiliers ont été vendus de manière non planifiée. Si [Ne jamais vendre les biens immobiliers](#f-re-never-sell) est activé, l’indicateur **«Renouvellements non supportables»** figure à côté: dans combien de simulations (nombre et part) une hypothèque n’était pas supportable lors du renouvellement et a été renouvelée en totalité uniquement grâce au commutateur. Un paiement unique effectué n’en fait pas partie. La colonne **«Ventes immobilières»** du tableau des percentiles indique, pour la simulation de la ligne concernée, **chaque vente d’un bien immobilier** avec le mois (‹Haus: 05.2034›, dessous ‹Wohnung: 03.2048›); les ventes forcées portent le symbole d’avertissement (maison avec signe d’avertissement), les ventes planifiées non; sans vente, «–» figure.

## @r-scenarios Scénarios
Mots-clés: variation, carte thermique, tableau, paramètres, taux de réussite par variante

L’onglet des scénarios montre l’effet des [variations](#f-sweep): chaque ligne du tableau est une **combinaison** de valeurs de paramètres avec lesquelles **toutes les trajectoires de rendement** ont été calculées. Les colonnes montrent le taux de réussite, la fortune à la retraite (médiane), la fortune finale (5e, 50e, 95e percentile) et l’épuisement le plus précoce.

- Avec **un** paramètre, deux graphiques montrent le taux de réussite et la fortune finale par valeur.
- Avec **deux** paramètres apparaît une **carte thermique**: la couleur montre le taux de réussite; vous trouvez ainsi la zone sûre d’un coup d’œil.

Au-dessus des graphiques et dans chaque titre de graphique figure **quelle variation** est représentée.

**Simulation unique avec variations:** si vous calculez avec une source sans hasard (par exemple rendements fixes) et des variations, **«Choisir une variation»** apparaît au-dessus des onglets. Vous y choisissez pour chaque variation un niveau (ou la valeur de la planification); les graphiques, indicateurs et valeurs annuelles montrent alors la simulation de ce niveau. «Optimum» indique la meilleure valeur selon les mêmes critères qu’avec plusieurs simulations (ci-dessous), **«Afficher l’optimum»** choisit le pas correspondant. Il n’y a pas de percentiles pour la simulation unique; l’optimum vaut pour cette seule simulation.

**Niveau avec Monte Carlo et simulations historiques:** avec plusieurs simulations et des variations, un champ de sélection par variation apparaît en haut de **«Simulations individuelles»** et de **«Valeurs annuelles»**, avec les niveaux calculés (par exemple ‹Part d’actions 30 %›, ‹Retrait en capital 60 %›) et «Valeur de la planification». Les deux onglets montrent alors les simulations de ce niveau, c’est-à-dire les mêmes simulations (pire, 1er, 5e, 50e, 95e, 99e percentile, en pouvoir d’achat actuel dans le classement réel) sur lesquelles s’appuie le tableau sous «Scénarios». L’application ne recalcule pour cela que ces six simulations, pas toutes les trajectoires; le changement dure quelques secondes. La sélection vaut pour les deux onglets ensemble. Les autres onglets continuent de montrer les valeurs de la planification et portent la mention «Valeurs de la planification, pas la variation choisie». La sélection fonctionne de même pour les résultats enregistrés. Si une fortune finale recalculée diffère du tableau (par exemple parce qu’une série temporelle a changé), une remarque apparaît. Un niveau qui n’a pas été calculé (valeur de la planification hors de la grille) ne peut pas être choisi.

**Valeur optimale par percentile:** dans la vue d’ensemble (fortune finale par percentile) et sous «Simulations individuelles» figure pour chaque percentile la valeur de la variation qui est optimale pour ce percentile. Seuls les pas de variation déjà calculés sont utilisés.

- **Facteur de dépenses:** le facteur le **plus élevé** pour lequel la fortune suffit jusqu’à la fin. Si la transition se situe entre deux pas, on interpole entre eux jusqu’à ce que la fortune finale soit exactement 0 (affichage ‹≈ 73 %›).
- **Part d’actions, retrait en capital de la caisse de pension:** les valeurs pour lesquelles la fortune suffit ont la priorité; parmi elles, la **fortune finale la plus élevée** s’applique. Si elle ne suffit nulle part, c’est la valeur avec l’âge d’épuisement le plus tardif qui s’applique («(ne suffit nulle part)»). Il n’y a pas d’interpolation ici.
- **Deux variations:** si le facteur de dépenses en fait partie, on détermine pour chaque valeur de l’autre paramètre le facteur supportable le plus élevé et on montre la combinaison avec le facteur le plus élevé. Sinon, la meilleure combinaison selon les critères ci-dessus s’applique.
- **Bord de la plage:** ‹< 50 %› signifie que la valeur la plus basse examinée ne suffit déjà pas; ‹≥ 110 %› (ou ‹≥› / ‹≤› pour la meilleure valeur) que la plage ne suffit pas. Élargissez alors la variation. Pour une part (part d’actions, retrait en capital de la caisse de pension), 0 % ou 100 % figure sans ‹≥› ni ‹≤›, car c’est la limite naturelle et non une plage trop étroite; si la meilleure valeur d’une part se trouve à un autre bord (par exemple 50 % pour une plage de 0 à 50 %), ‹≥› ou ‹≤› continue de s’appliquer.
- **Mois de départ historique, date de la retraite, taux de prélèvement:** un optimum n’a pas de sens ici (le mois de départ n’est pas une décision; une date de retraite plus tardive ou un taux plus bas donne toujours la fortune finale la plus élevée). «–» est affiché.

Toutes les variantes utilisent **les mêmes trajectoires aléatoires**. Les différences proviennent donc du paramètre, et non du hasard.

## @r-table Valeurs annuelles
Mots-clés: tableau, année, exportation, chiffres, prélèvement, écart par rapport au plan, impôts, dépenses, loyer, hypothèque, percentile, retrait en capital, gain immobilier

Un tableau avec **une ligne par année** pour la simulation choisie. Montants dans la monnaie du projet, nominaux ou en pouvoir d’achat actuel selon le commutateur. Avec plusieurs simulations, vous choisissez la simulation en haut (pire simulation, 1er, 5e, 50e, 95e, 99e percentile); la médiane est le standard.

| Colonnes | Signification |
|---|---|
| Liquide, Immobilier, Prévoyance, Total | **fortune en fin d’année**: liquidités, immobilier (net), prévoyance (CP, 3a), somme |
| Total des dépenses | **moyenne mensuelle de l’année**: dépenses planifiées (coût de la vie selon votre planification) plus **loyer ou intérêts hypothécaires**, amortissement et autres frais de logement des biens immobiliers plus **versement au pilier 3a** |
| EMS | Seulement avec un [EMS](#f-care-home): moyenne mensuelle des frais d’EMS de l’année, compris dans «Total des dépenses» |
| Prélèvement | moyenne mensuelle de l’année: ce qui est **prélevé** sur la fortune (dépenses effectives moins revenus, avant impôts) |
| Total des revenus | moyenne mensuelle de l’année: revenus (revenu de l’activité lucrative et rentes) plus prélèvement |
| Taux de prélèvement | **prélèvement annuel / fortune totale en début d’année** en pour cent (colonne «Total» de l’année précédente) |
| Écart par rapport au plan | moyenne mensuelle: total des revenus moins total des dépenses. À **0**, tout se déroule comme planifié. Pour les stratégies qui lient les dépenses à la fortune (Guardrails, taux fixe, VPW, Endowment), une valeur **négative** est la réduction par rapport à votre plan, une valeur positive l’augmentation |
| Inflation par année | variation de l’indice des prix pendant l’année civile, la première année depuis le début de la simulation |
| Impôts payés dans l’année | **sommes annuelles des impôts payés** par type: impôt revenu, impôt fortune, impôt prestation en capital, impôt gain immobilier, plus le total. Les colonnes sont **repliées par défaut**; le bouton **«Afficher les impôts»** au-dessus du tableau les affiche et les masque, le choix est conservé. Les en-têtes s’appellent **«Impôt estimé …»**, leur infobulle mentionne l’arrondi à 100 francs |
| Valeur fiscale biens immobiliers | apparaît avec les colonnes d’impôts si un bien immobilier a une [valeur fiscale](#f-re-tax-value) : somme des valeurs fiscales au 31 décembre (biens immobiliers sans valeur fiscale à la valeur vénale) ; l’impôt sur la fortune l’utilise moins l’hypothèque |
| Déductions fortune, Fortune imposable | apparaissent avec les colonnes d’impôts si la planification a des [déductions de la fortune](#f-tax-wealth-deductions) : déductions au 31 décembre avec leur origine dans l’infobulle et fortune moins déductions, jamais inférieure à 0 (dans le mode «domicile», les déductions sociales du canton s’y ajoutent) |

Les colonnes ne sont pas classées en groupes avec un titre. Le fait que les fortunes soient des **soldes en fin d’année** et les flux monétaires des **moyennes mensuelles** figure dans l’indication (infobulle) des en-têtes de colonnes et dans le texte sous le tableau. Le tableau ne montre ni le revenu seul ni les dépenses planifiées sans frais de logement; ils sont compris dans «Total des revenus» et «Total des dépenses».

- **Taux de prélèvement:** le prélèvement annuel est la somme des prélèvements mensuels de l’année civile (pour les années incomplètes, la moyenne mensuelle fois 12). La fortune en début d’année est la fortune totale («Total») de l’année précédente, la première année le solde au début de la simulation. Le calcul utilise toujours des valeurs **nominales**; le commutateur nominal/pouvoir d’achat actuel ne modifie pas les taux. Si le prélèvement est nul ou négatif (revenus supérieurs aux dépenses), la cellule reste vide; si la fortune est nulle ou négative, «–» est affiché.
- **Stratégie à 3 compartiments:** chaque année avec au moins une réalimentation a une flèche (▸) au début. Elle déplie le **calcul de la taille des compartiments et des réalimentations** de cette année (déclencheur, base des dépenses, revenus réguliers, besoin net, objectifs, critère de rendement, transferts, soldes des compartiments avant et après). Avec plusieurs simulations, il vaut pour la simulation unique choisie.
- Les flux monétaires sont des **moyennes mensuelles** de l’année civile, car les mois individuels fluctuent (les intérêts hypothécaires sont par exemple payés trimestriellement).
- L’impôt sur le revenu et l’impôt sur la fortune sont taxés en décembre et payés en **mai de l’année suivante**. Le tableau les montre dans la ligne de l’**année fiscale** à laquelle ils se rapportent: l’impôt de l’année fiscale 2028 figure dans la ligne 2028, même si l’argent ne sort qu’en mai 2029. La fortune et le solde de cash ne diminuent donc que l’année suivante. Si la planification se termine avant décembre, l’impôt de la dernière année, incomplète, manque dans les colonnes. L’impôt sur les prestations en capital (caisse de pension, pilier 3a) et l’impôt sur les gains immobiliers figurent l’année du retrait ou de la vente.
- Les impôts sont **estimés** et **arrondis à 100 francs** (chaque cellule pour elle-même, le total peut donc s’écarter de quelques centaines de francs de la somme des colonnes arrondies). Sous le tableau figure «Impôt estimé avec vos propres barèmes, arrondi à 100 francs. Pas de conseil fiscal.» L’impôt sur les prestations en capital et l’impôt sur les gains immobiliers sont à vérifier avec le calculateur d’impôt du canton ou un spécialiste avant de décider, voir [Quelle est la précision de l’impôt?](#tax-accuracy).
- La première et la dernière année sont généralement incomplètes; elles portent un astérisque (*), le pointeur de la souris indique le nombre de mois.

## @r-list Liste des résultats
Mots-clés: historique, liste, enregistré, renommer, note, supprimer, résultats, simulations

Chaque simulation est **automatiquement classée** comme résultat après son lancement, avec la **planification de l’époque** et le **résultat complet**. Le bouton **«Résultats»** de l’en-tête ouvre la liste.

- **Résultat:** pour les simulations Monte Carlo figure ‹**taux de réussite · 5 %: montant**› (5e percentile de la fortune finale); la médiane apparaît lorsque vous pointez dessus. Les simulations uniques montrent la fortune finale. Le **symbole d’avertissement (maison avec signe d’avertissement)** apparaît lorsqu’un bien immobilier a été vendu de manière non planifiée (vente forcée, y compris faute de capacité financière lors du renouvellement de l’hypothèque; les ventes planifiées ne comptent pas). Avec plusieurs simulations, il s’applique dès **5 % des simulations** avec vente forcée, l’infobulle indique la part et le nombre de simulations; pour une simulation unique, à chaque vente forcée. Les entrées de versions plus anciennes ne montrent pas de symbole.
- **Série temporelle:** nom de la série temporelle avec laquelle le résultat a été calculé (pour les rendements fixes «rendements fixes», pour la source paramétrique «paramétrique»). Les entrées de versions plus anciennes montrent «–». L’indication du type (simulation unique, scénarios, nombre de simulations) figure en petit sous le résultat.
- **Stratégie:** si vous déplacez la souris sur le nom (ou y allez avec la touche Tab), les **paramètres de la stratégie** apparaissent dans l’état du résultat. Les entrées plus anciennes montrent «Paramètres non disponibles».
- **Nom et note:** modifiables directement dans la ligne. Nom standard: projet · source de rendements · stratégie · moment, par exemple ‹Beispiel · MC hist. · 3 comp. + Guard · 16.11.26, 14:05› (pour les scénarios figure ‹Scénarios› au lieu de la source). Abréviations de la source de rendements: **Fixe** (rendements fixes), **Hist.** (historique dès la date de départ), **Tous les départs** (toutes les dates de départ historiques), **MC hist.** (Monte Carlo · historique), **MC param.** (Monte Carlo · paramétrique). Abréviations de la stratégie: **Standard**, **3 compartiments**, **3 comp. + Guard** (3 compartiments avec Guardrails), **Guardrails**, **Taux fixe** (taux de prélèvement fixe), **VPW**, **Endowment**. Le nombre de simulations figure en petit sous le résultat. Les résultats de versions plus anciennes conservent leur nom.
- **Ouvrir:** montre le résultat enregistré avec tous les onglets. C’est la **planification de l’époque** qui s’applique; une remarque en haut indique la date. **«Reprendre la planification dans l’éditeur»** recharge cette planification dans l’éditeur.
- **Supprimer** retire un résultat après confirmation, **«Supprimer les éléments marqués»** plusieurs et **«Tout supprimer»** tous. La case de l’en-tête marque toutes les entrées.
- **Les optimisations** et les comparaisons des stratégies figurent aussi dans la liste, voir [Enregistrer et rouvrir le résultat](#o-saved). «Comparer» ne compte que les simulations et ignore les optimisations marquées.
- **Comparer:** marquez de deux à dix entrées et cliquez sur **«Comparer»**, voir [Comparaison](#r-compare). Si plus de dix sont marquées, «Comparer» est bloqué et une remarque apparaît; la suppression reste possible.

La liste se trouve **localement dans la mémoire de l’application**, pas dans le fichier de planification. Elle grandit jusqu’à ce que vous supprimiez des entrées: une entrée avec de nombreuses simulations occupe quelques mégaoctets. Si la mémoire est pleine, le message «Résultat non enregistré dans la liste» apparaît; le résultat reste affiché, mais n’est pas classé. Supprimez alors des entrées plus anciennes.

## @r-compare Comparer des résultats
Mots-clés: comparaison, côte à côte, différences, planification, indicateurs, valeurs annuelles, pouvoir d’achat

La comparaison montre jusqu’à dix résultats enregistrés côte à côte, chacun dans sa propre couleur. Avec de nombreux résultats, les tableaux larges et les graphiques en barres peuvent défiler horizontalement. Elle offre les mêmes possibilités qu’un résultat individuel, réparties sur quatre onglets:

- **Vue d’ensemble:** indicateurs (série temporelle, stratégie, taux de réussite, fortune finale à la médiane, aux 5e et 1er percentiles et dans la pire simulation, épuisement le plus précoce; un clic sur un en-tête de colonne trie les lignes par ordre croissant, un autre par ordre décroissant, la flèche ▲/▼ indique le sens; les valeurs manquantes figurent toujours à la fin, un épuisement «aucune» compte comme la valeur la plus tardive), la **fourchette de la fortune totale** (médiane en trait plein, 5e percentile en tirets, 95e percentile en pointillé), la **fortune finale par percentile** sous forme de tableau, la **probabilité que l’argent suffise** (axe 80–100 % ou 0–100 %, voir [là](#r-survival)) et l’**âge à l’épuisement** en part des simulations (les résultats avec des nombres de simulations différents restent ainsi comparables).
- **Évolutions:** les graphiques du résultat individuel pour une simulation par résultat: pour une simulation unique, cette simulation; pour plusieurs simulations, la simulation du percentile que vous réglez sous **«Choisir une simulation»** (comme sous «Simulations individuelles»; standard: médiane). La sélection vaut pour tous les résultats et aussi pour les valeurs annuelles; s’il manque le percentile à un résultat, celui-ci montre la simulation médiane avec une mention. Fortune totale, fortune liquide (au choix: total, actions, obligations, cash), revenus, dépenses ou impôts par année, taux de prélèvement, immobilier (fonds propres), prévoyance ainsi qu’actions, obligations et inflation (actions en trait plein, obligations en tirets, inflation en pointillé). Ce graphique a les mêmes commutateurs que dans le résultat individuel: **Performance (départ = 100)**, linéaire ou logarithmique, ou **Variation en %** par rapport au même mois de l’année précédente; le choix vaut dans les deux vues.
- **Valeurs annuelles:** pour la simulation choisie, une ligne par année et une colonne par résultat pour une grandeur au choix (fortune totale, fortune liquide, immobilier, prévoyance, revenus, dépenses, prélèvement, taux de prélèvement, impôts). Les valeurs sont les mêmes que dans les [valeurs annuelles](#r-table) des différents résultats.
- **Planification:** un tableau de toutes les saisies qui diffèrent entre les résultats, avec les désignations et unités de l’éditeur (par exemple ‹Part d’actions: 30 %› au lieu d’un nom de champ interne) et groupées selon les sections de l’éditeur. Les entrées de listes portent leur nom (par exemple ‹Ferienwohnung: valeur vénale›). Les valeurs dépendant du temps apparaissent sous forme de liste (‹dès 01.2030: …›). Vous voyez ainsi exactement **ce qui** a conduit à un autre résultat.

En haut à droite, vous passez de **Nominal** à **Pouvoir d’achat actuel**; chaque résultat est converti avec sa propre inflation. Comme dans le résultat individuel, les simulations des percentiles sont classées selon la fortune finale de l’affichage choisi, en pouvoir d’achat actuel selon la fortune réelle (voir [Fortune finale par percentile](#r-percentiles)); les résultats plus anciens sans ce classement sont nommés dans une remarque. Le zoom vaut, comme dans le résultat individuel, pour tous les graphiques avec un axe du temps.

Les résultats sont alignés selon les **années depuis leur départ**. S’ils commencent en des années civiles différentes, les valeurs annuelles montrent ‹Année 1›, ‹Année 2›, etc.; l’année civile figure dans l’infobulle. S’il manque une grandeur à un résultat, celui-ci manque dans ce graphique ou cette colonne, et une mention l’indique: une simulation unique apparaît dans la fourchette comme une ligne et n’a ni percentiles, ni probabilité, ni âge à l’épuisement; un résultat sans bien immobilier ou sans avoirs de prévoyance manque dans ces graphiques; un résultat avec des scénarios uniquement n’apparaît que comme indicateur; les résultats plus anciens sans valeurs en pouvoir d’achat actuel manquent dans la fourchette; les résultats de la version 1.0.0-beta.1 et plus anciennes n’ont en pouvoir d’achat actuel que la fortune finale de la pire simulation et des 1er, 5e, 50e, 95e et 99e percentiles.

Comparez de préférence des résultats qui diffèrent en **un seul** point (par exemple la part d’actions ou la stratégie).
