# Éditeur: sections et champs

## @s-general Ménage et projet
Mots-clés: nom, monnaie, personnes, naissance, deuxième personne, double revenu, période de simulation

Vous définissez ici les données de base.

- **Nom du projet et monnaie:** tous les montants sont saisis dans la monnaie du projet (CHF, EUR ou USD). Les indices de la [stratégie de placement](#f-ret-series) sont en CHF; pour une planification en EUR ou en USD, une remarque apparaît.
- **Personne 1:** nom et mois de naissance (format MM.AAAA). À partir du mois de naissance, l’application calcule l’âge pour les jalons (‹à 65 ans›) et pour la légende des graphiques.
- **+ Deuxième personne:** ajoute une deuxième personne avec son propre revenu, sa propre caisse de pension et sa propre AVS (double revenu). Les impôts, la fortune et les dépenses s’appliquent au ménage. Voir [Deuxième personne et règles pour couples mariés](#f-couple).
- **Période de simulation:** le début et la fin sont des jalons et sont modifiés sous [Jalons](#s-milestones).

Toutes les indications de temps ont le format **MM.AAAA** (par exemple 05.2027).

**Les nombres** se saisissent avec ou sans séparateur de milliers: apostrophe ou espace (1'234 ou 1 234), point ou virgule comme séparateur décimal (12.5 ou 12,5). Pour les **montants** et les **nombres entiers** (par exemple le nombre de trajectoires), un point suivi d’exactement trois chiffres est ambigu (‹12.500›): le champ signale «Nombre ambigu» et conserve la valeur précédente; écrivez 12500 ou 12'500. Les champs de pour cent et d’années acceptent ‹1.25› comme jusqu’ici. Des notations comme ‹1e5› ou ‹0x10› ne sont admises nulle part.

## @s-milestones Jalons
Mots-clés: prestation de la caisse de pension, retraite, début, fin, début de la rente AVS, AVS, date, âge, moment, limites

Un jalon est une date nommée à laquelle toutes les autres saisies peuvent se référer («dès la prestation de la caisse de pension»). Si vous déplacez la prestation de la caisse de pension, tout ce qui en dépend se déplace aussi. C’est le levier le plus important pour jouer des scénarios ‹retraite plus tôt ou plus tard›.

Le **début de la simulation** et la **fin de la simulation** sont prédéfinis. En outre, **chaque personne** a deux jalons fixes, toujours présents:

| Jalon | Signification | Limites | Standard |
|---|---|---|---|
| **Prestation de la caisse de pension** (jusqu’ici ‹Pensionierung›) | versement de la rente et du capital de la caisse de pension, début de la stratégie de prélèvement | 58 à 65 ans | 65 |
| **Début de la rente AVS** | début de la rente AVS; fin des versements et de l’amortissement indirect au [pilier 3a](#s-pillar3a), versement du solde | 63 à 70 ans | 65 |

Ils ne peuvent pas être supprimés, mais leur date est librement choisie (âge ou date), tant qu’elle se situe dans les limites; les limites elles-mêmes sont admises. Les limites peuvent être modifiées dans les [paramètres](#f-settings). Avec deux personnes, ils s’appellent «Prestation de la caisse de pension personne 2» et «Début de la rente AVS personne 2». Les noms des jalons fixes apparaissent dans la langue choisie et ne sont pas enregistrés dans le fichier de planification. La **fin du revenu de l’activité lucrative** n’est pas un jalon: vous la définissez dans le [revenu](#s-flows). Si le revenu de l’activité lucrative est encore supérieur à 0 après la prestation de la caisse de pension, le contrôle affiche un avertissement avec le montant.

Vous ajoutez des jalons propres avec «+ Jalon», par exemple ‹Verkauf Haus› ou ‹Enfants partis›.

Chaque jalon est défini comme:

- **Date:** un mois fixe (05.2027)
- **Âge de:** l’âge d’une personne, par exemple ‹personne 1 à 64.5 ans›. Les décimales sont admises: 64.5 ans correspondent à 64 ans et 6 mois.

La colonne «Donne» montre le mois qui en résulte. **Lors du passage** entre «Date» et «Âge de», la date reste la même: l’âge ou la date de la colonne «Donne» est reporté dans le nouveau champ. Le jalon peut être supprimé avec ‹✕›. **S’il est encore utilisé**, une boîte de dialogue demande confirmation: elle indique le nombre de saisies et les sections, et les remplace lors de la suppression par la **date fixe** (mois/année) du jalon, pour «Âge de» par la date calculée. S’il en résulte une date en double dans une série temporelle, la ligne placée en dernier s’applique et un message le signale. Avec «Annuler» (⌘Z), tout peut être annulé. Le **début de la simulation, la fin de la simulation, la prestation de la caisse de pension et le début de la rente AVS** ne peuvent pas être supprimés; le bouton est grisé.

**Anciennes planifications:** si l’un des jalons fixes manque, il est ajouté à l’ouverture, fixé à 65 ans. Si un âge se situe en dehors des limites, il est fixé à la limite la plus proche. ‹Pensionierung› s’appelle désormais «Prestation de la caisse de pension». Une boîte de dialogue indique chaque modification.

> Fixez délibérément la **fin de la simulation** tard, par exemple à 95 ou 100 ans. Une fin trop précoce rend le résultat meilleur qu’il ne l’est: l’argent ne doit suffire que jusque-là.

## @s-pension Prévoyance
Mots-clés: caisse de pension, AVS, 2e pilier, 1er pilier, pilier 3a, rente

Caisse de pension (2e pilier) et AVS (1er pilier); avec deux personnes, chacune a ses propres cartes. Vous trouvez les chiffres sur le **certificat de prévoyance** de votre caisse de pension et dans le **compte AVS** (extrait de compte ou calcul anticipé de la rente de la caisse de compensation).

- Caisse de pension: [avoir de vieillesse, prestation, retrait en capital, taux de conversion](#f-pk)
- AVS: [rente et début de la prestation](#f-ahv)

Une ligne sous les cartes montre ce que devient l’avoir lors de la prestation de la caisse de pension (capital et rente).

Le **pilier 3a** a sa propre section, voir [Pilier 3a](#s-pillar3a).

## @s-pillar3a Pilier 3a
Mots-clés: 3a, prévoyance liée, versement, retrait, plan de retrait, âge, date, avoir, montant maximal, amortissement indirect

Le pilier 3a est un compte propre **par personne**, séparé de la caisse de pension et de l’hypothèque.

- **Avoir au début de la simulation** et **intérêt / rendement p.a.**
- **Versement par année:** 0 jusqu’au montant maximal (7 258 francs, modifiable dans les [paramètres](#f-settings)). L’application verse chaque mois un douzième, jusqu’au mois précédant le **début de la rente AVS**. Le versement **compte dans les dépenses**: il apparaît dans «Total des dépenses» des valeurs annuelles, augmente le prélèvement et fait partie du besoin net de la stratégie à 3 compartiments.
- **Plan de retrait:** lignes composées d’un moment et d’un montant, par exemple 2041 50 000 et 2042 50 000. Vous fixez le **moment** comme pour un jalon, sous forme de **date** (MM.AAAA) ou d’**âge** de la personne à qui appartient le compte (par exemple 61.5 ans). À côté figurent le mois qui en résulte et l’âge. Si vous passez de «Date» à «Âge», le moment reste le même. Sans montant, la totalité de l’avoir est retirée. Un retrait est possible **au plus tôt 5 ans avant le début de la rente AVS**, également selon l’âge; une entrée antérieure est signalée comme erreur au champ. Si un montant dépasse l’avoir, seul l’avoir est versé (remarque dans les événements).
- **Avoir tiré de l’aperçu:** chaque ligne montre l’**avoir avant le retrait** et le **solde ensuite**; sous le plan figure l’avoir versé au début de la rente AVS. Les valeurs proviennent de l’aperçu avec rendements fixes; si la planification contient une erreur, il n’y a pas d’aperçu, et «–» est affiché.
- **+ Retrait** crée une ligne un an après la dernière (sans ligne: au mois le plus précoce admis, 5 ans avant le début de la rente AVS, pas avant le début de la simulation), au plus tard au début de la rente AVS. Le montant proposé est l’avoir à ce moment après les retraits antérieurs, arrondi au millier inférieur. Vous pouvez ensuite le modifier librement.
- **Solde au début de la rente AVS:** ce qui reste sur le compte est versé **au plus tard le mois du début de la rente AVS**.
- **Utilisation:** dans la fortune libre ou, s’il existe une tranche avec amortissement indirect sur ce compte, «Rembourser l’hypothèque» (d’abord les tranches avec amortissement indirect).

Chaque retrait est une **prestation en capital** soumise à l’impôt sur les prestations en capital. Les retraits de la même année civile (caisse de pension et pilier 3a) sont additionnés pour le taux, pour les couples non mariés par personne.

L’**amortissement indirect** d’une hypothèque est versé sur ce compte jusqu’au mois précédant le début de la rente AVS (voir [Hypothèques](#f-re-mortgage)). Le versement et l’amortissement indirect ensemble ne devraient pas dépasser le montant maximal par année; sinon, le contrôle affiche un avertissement.

> Exemple: naissance 06.1980, avoir de 120 000 sans intérêt, retrait à 61.5 ans (donne 12.2041) de 50 000: avoir avant le retrait 120 000, solde ensuite 70 000. «+ Retrait» propose une ligne à 62.5 ans (12.2042) de 70 000.

> Exemple: avoir de 120 000, retraits en 2041 et 2042 de 50 000 chacun, début de la rente AVS en 2045: le solde de 20 000 (plus les intérêts) est versé le mois du début de la rente AVS. Si le début de la rente AVS est reporté à 2046, le plan reste valable, et le solde est versé en 2046.

## @f-settings Paramètres
Mots-clés: paramètres, limites, âge AVS, prestation de la caisse de pension, montant maximal, 3a, valeurs standard, langue

Sur Mac, menu «VivaPlan → Paramètres…» (⌘,), sous Windows et Linux menu «Fichier → Paramètres…» (Ctrl+,), ou le bouton «Paramètres» en haut. Les paramètres valent pour **toutes les planifications sur cet appareil** et sont enregistrés localement, **pas dans le fichier de planification**.

| Paramètre | Standard |
|---|---|
| Langue | comme le système |
| Début de la rente AVS: limite inférieure et supérieure | 63 et 70 ans |
| Prestation de la caisse de pension: limite inférieure et supérieure | 58 et 65 ans |
| Pilier 3a: montant maximal du versement par année | 7 258 |

**Langue:** «comme le système» suit la langue du système d’exploitation (au premier démarrage; sans région, fr_FR compte comme français), sinon la langue choisie s’applique. Sont proposés l’allemand, l’anglais, le français et l’italien; si la langue du système manque, l’allemand s’applique. L’aide apparaît dans la langue choisie; un chapitre pas encore traduit apparaît en allemand. Le même choix figure dans le menu «Affichage → Langue». Un changement agit immédiatement, y compris sur les menus; la planification, les modifications non enregistrées et la vue sont conservées. Les nombres, montants et dates apparaissent dans chaque langue au format suisse (1'234'567.50, 2.5 %, 03.2040), et le fichier de planification est le même dans chaque langue. Les noms dans la planification (personnes, biens immobiliers, jalons) ne sont pas traduits; seule la planification d’exemple reçoit ses noms dans la langue choisie à l’ouverture, avec les mêmes chiffres.

La limite inférieure ne doit pas être supérieure à la limite supérieure, les âges sont des années entières, le montant maximal est de 0 ou plus. «Valeurs standard» réinitialise tout après confirmation. Si une planification se situe en dehors des nouvelles limites après une modification, le contrôle le signale; la même planification peut donc donner un message sur un autre appareil avec d’autres paramètres. La ligne de commande utilise toujours les valeurs standard.

## @f-pk Caisse de pension
Mots-clés: LPP, avoir de vieillesse, cotisations, rémunération, certificat de prévoyance

- **Avoir de vieillesse au début de la simulation (MM.AAAA):** l’avoir selon le certificat de prévoyance (avoir de vieillesse, part surobligatoire comprise). On n’inscrit pas ici les projections du certificat (projection avec intérêts); l’application les calcule elle-même.
- **Prestation dès:** toujours le jalon **Prestation de la caisse de pension** de la personne (58 à 65 ans). Vous modifiez la date sous [Jalons](#s-milestones).
- **Cotisations par mois:** somme des cotisations de l’employé et de l’employeur (et des rachats) versées jusqu’à la prestation. Elles prennent fin avec la prestation.
- **Rémunération par année:** taux d’intérêt sur l’avoir de vieillesse. La rémunération est créditée en décembre. 1 % à 2 % sont réalistes.
- La [part du retrait en capital](#f-pk-lumpsum) et le [taux de conversion](#f-pk-conversion) déterminent combien est versé en capital et combien en rente.

Les cotisations et la rémunération peuvent être saisies en fonction du [temps](#f-timed), par exemple des cotisations plus élevées dès 55 ans.

## @f-pk-conversion Taux de conversion
Mots-clés: rente, caisse de pension, pour cent, conversion

Le taux de conversion détermine combien de **rente annuelle** vous recevez par franc d’avoir de vieillesse que vous ne retirez pas en capital.

> Rente annuelle = capital de rente × taux de conversion. Exemple: 500 000 francs × 5.0 % = 25 000 francs par année (environ 2 083 par mois).

- Le **minimum légal** est de 6.8 % dans la partie obligatoire, mais de nombreuses caisses de pension calculent avec des taux **plus bas** (souvent 4.5 % à 5.5 %), car elles convertissent différemment la partie surobligatoire. Utilisez la valeur de votre certificat de prévoyance, pas le minimum légal.
- Le taux dépend de l’âge lors de la prestation. Utilisez le taux correspondant à votre âge de la retraite.
- Dans l’application: rente par mois = capital de rente × taux de conversion / 12, **fixe** (sans renchérissement) jusqu’à la fin de la vie.

Avec la section des scénarios, vous pouvez faire varier le retrait en capital (voir [Scénarios](#f-sweep)).

## @f-pk-lumpsum Part du retrait en capital
Mots-clés: retrait en capital, rente, impôt sur les prestations en capital, prestation unique en capital

La part de l’avoir de vieillesse versée **en capital** à la retraite. 0 % signifie ‹entièrement en rente›, 100 % ‹entièrement en capital›.

- Le capital va net dans votre fortune liquide. L’**impôt sur les prestations en capital** en est déduit (voir [Barèmes fiscaux](#f-tax-tables)). Les retraits de la même année civile (caisse de pension, pilier 3a) sont additionnés pour le taux, pour les couples non mariés par personne.
- Le reste devient une rente au moyen du [taux de conversion](#f-pk-conversion).
- **Rente:** planifiable, à vie, mais non indexée et non transmissible. **Capital:** flexible et transmissible, mais il porte le risque de placement et la responsabilité de ne pas l’épuiser trop tôt.

La meilleure part dépend de l’impôt, du taux de conversion et de votre goût du risque. Faites-la varier avec une [variation](#f-sweep) et voyez comment le taux de réussite change.

## @f-ahv AVS
Mots-clés: rente, prestation, renchérissement, indice, 1er pilier, années de cotisation, 13e rente AVS, décembre, anticipation, ajournement, réduction, supplément

- **Rente par mois au début de la prestation:** votre rente mensuelle probable lors d’une perception à 65 ans (âge de référence) selon le calcul anticipé de la rente de la caisse de compensation, aux prix actuels, **sans la 13e rente** (l’application l’ajoute elle-même, voir ci-dessous).
- **Prestation dès:** toujours le jalon **Début de la rente AVS** de la personne (63 à 70 ans). S’il se situe avant ou après 65 ans, l’application réduit ou augmente elle-même la rente (voir ci-dessous); saisissez néanmoins la rente à 65 ans.

Sous le champ figure toujours la rente avec laquelle l’application calcule, aux prix actuels: sans anticipation ni ajournement égale à la saisie, sinon avec le taux, p. ex. *−13.6 % → effectivement 2'177.00 CHF par mois*. Pour les couples mariés figure en plus à partir de quand les deux ont droit à la rente et quelle rente reste après le plafond et la réduction ou le supplément, p. ex. *dès 03.2032 plafonnée 1'547.00 CHF*.

**Anticipation et ajournement:** Une anticipation réduit la rente de 6,8 % par année, au mois près selon le tableau de l’AVS (p. ex. 15 mois 8,5 %), au plus 13,6 % en cas de perception à 63 ans. Un ajournement d’au moins une année l’augmente par trimestres: 1 an 5,2 %, 2 ans 10,8 %, 3 ans 17,1 %, 4 ans 24,0 %, 5 ans 31,5 %; un ajournement de moins d’une année n’apporte aucun supplément. Réduction et supplément sont des montants fixes en francs entiers; ils valent pour toute la durée de perception, aussi après 65 ans, et par personne selon son propre âge. Exemple: rente 2 520, perception à 63 ans → 2 520 − 343 = 2 177; perception à 68 ans → 2 520 + 431 = 2 951, pas d’AVS entre 65 et 68 ans. Pour les couples mariés, le plafond s’applique comme à l’AVS aux rentes **non réduites** dès que les deux ont droit à la rente (perception, aussi anticipée, ou 65 ans atteints, aussi pendant un ajournement); ce n’est qu’ensuite que l’application déduit la réduction ou ajoute le supplément. La réduction est le taux multiplié par la moyenne des rentes pendant l’anticipation, le supplément le taux multiplié par la moyenne de la rente ajournée, le cas échéant déjà plafonnées. Exemple avec deux rentes de 2 520 et un plafond de 3 780: si la première personne perçoit à 63 ans et la seconde à 65 ans bien plus tard, il y a d’abord 2 177, puis 1 890 − 343 = 1 547 et 1 890. La 13e rente repose sur les rentes versées. Une perception moins d’une année après 65 ans n’est pas un ajournement (pas de supplément); une perception avant 63 ans ou après 70 ans n’est pas possible. Dans les deux cas, la vérification avertit. Source: mémento 3.04 *Retraite flexible* (état au 1.1.2026). Ne sont pas pris en compte l’anticipation et l’ajournement partiels ni les taux particuliers pour les femmes nées de 1961 à 1969. Les planifications enregistrées avant cette modification contenaient la rente déjà adaptée en cas de perception avant ou après 65 ans; à l’ouverture, l’application la reconvertit une fois en rente à 65 ans et le signale. Le résultat reste le même à l’arrondi au franc près; pour les couples mariés, il peut changer à cause du plafond.
- **Adapter chaque année la rente au renchérissement:** l’AVS est augmentée en janvier du renchérissement accumulé depuis le dernier janvier. Laissez la case cochée si la rente doit augmenter en conséquence.

Avec deux personnes mariées, il existe un plafond pour les deux rentes ensemble, voir [Deuxième personne et règles pour couples mariés](#f-couple).

**13e rente AVS:** Dès décembre 2026, l’AVS verse chaque année en décembre une 13e rente; l’application en tient toujours compte, il n’y a pas d’option pour la désactiver. Elle correspond à un douzième des rentes AVS versées de janvier à décembre (chaque mois arrondi au centime, la somme au franc entier). Si la rente commence en cours d’année, elle est versée au prorata: une rente de 2 520 dès juillet donne 6 × 210 = 1 260. Pour les couples mariés, c’est la rente déjà plafonnée qui compte; la 13e rente elle-même n’est pas plafonnée une seconde fois. Elle compte en décembre comme revenu AVS: dans les [valeurs annuelles](#r-table) parmi les recettes, dans le graphique [Revenus et dépenses par année](#r-flows) sous l’AVS. Elle est imposable comme la rente AVS. Si la simulation se termine avant décembre, il n’y a pas de 13e rente cette année-là. À l’ouverture d’une planification enregistrée avant cette modification, une remarque signale une fois que le résultat peut donc différer.

## @f-couple Deuxième personne et règles pour couples mariés
Mots-clés: couple marié, marié, plafonnement, 150 pour cent, partenaire, double revenu

Avec «+ Deuxième personne», vous saisissez un ménage de deux personnes.

- Chaque personne a **son propre revenu, sa propre caisse de pension et sa propre AVS**.
- Les impôts, la fortune, les dépenses et les biens immobiliers s’appliquent au **ménage**.
- **Le ménage est considéré comme retraité** dès que toutes les personnes avec une date de prestation sont à la retraite. Ce n’est qu’alors que les stratégies de prélèvement interviennent.

**Marié·e·s** (règles pour couples mariés): lorsque les deux ont droit à une rente AVS, leurs rentes à 65 ans sont plafonnées ensemble à **3 780 francs par mois** au plus; la réduction en cas d’anticipation et le supplément en cas d’ajournement s’y ajoutent ensuite (voir [AVS](#f-ahv)) (150 % de la rente maximale 2025). La valeur peut être modifiée si la rente maximale augmente. Le plafond augmente une fois chaque janvier avec le renchérissement, dès qu’au moins une rente avec adaptation au renchérissement est versée. Sous [Impôts](#s-taxes), il est possible d’enregistrer des barèmes propres pour les personnes mariées. Les retraits en capital de la même année sont additionnés pour les personnes mariées, pour les couples non mariés par personne.

**Ne sont pas calculés:** le décès d’une personne, les rentes de survivants, la rente de veuve ou de veuf. La planification calcule comme si les deux vivaient jusqu’à la fin.

## @s-flows Revenus et dépenses
Mots-clés: revenu, coût de la vie, dépenses, montants uniques, succession, inflation

Montants mensuels. **Les dépenses sont saisies aux prix actuels** et indexées par l’application à l’inflation. Vous inscrivez donc vos coûts actuels, même pour des années futures.

- **Revenu net de l’activité lucrative par mois:** ce qui arrive sur votre compte après déductions (AVS, caisse de pension; les impôts sont calculés séparément). Il prend fin lorsque vous le fixez à 0 en fonction du [temps](#f-timed), par exemple ‹dès la prestation de la caisse de pension 0›. La prestation de la caisse de pension n’y met pas fin d’elle-même; s’il est encore supérieur à 0 ensuite, le contrôle affiche un avertissement avec le montant.
- **Augmente avec le renchérissement** (par personne): si la case est cochée, les montants, y compris chaque ligne d’un revenu dépendant du temps, s’entendent **aux prix actuels** (prix au début de la simulation) et augmentent avec le renchérissement de la trajectoire concernée, exactement comme le coût de la vie. Exemple: 8 000 par mois avec 2 % de renchérissement donnent 8 160 après un an, environ 8 830 après cinq ans. Sans coche, le montant reste nominalement le même. Les nouvelles planifications et une deuxième personne nouvellement ajoutée ont la case cochée; les anciennes planifications sans indication calculent comme jusqu’ici, sans. Les impôts, les valeurs annuelles et la capacité financière utilisent le montant adapté; l’avertissement après la prestation de la caisse de pension indique alors le montant «aux prix du début de la simulation».
- **Coût de la vie par mois:** la **saisie** comprend tout ce que vous dépensez, **loyer ou frais de logement compris** (intérêts hypothécaires, amortissement, frais accessoires), tel que cela figure sur l’extrait de compte. L’application déduit de chaque ligne, de la même manière, la **part du logement**, c’est-à-dire les frais de logement du mois de départ tirés de la section [Immobilier](#s-realestate) en moyenne mensuelle (les intérêts et l’amortissement sont payés trimestriellement), indexée au renchérissement. Le **calcul** utilise le coût de la vie sans la part du logement plus les frais de logement **simulés**: si l’intérêt change lors d’un renouvellement ou si vous passez à une location après une vente, les dépenses changent en conséquence. Exemple: 8 000 dont 2 000 de frais de logement; après une hausse d’intérêts de 300, les dépenses sont de 8 300, avec un loyer de 2 500, de 8 500. Si une ligne est inférieure à la part du logement, le contrôle signale une erreur. Le versement au [pilier 3a](#s-pillar3a) n’est pas compris. L’application convertit à l’ouverture les anciennes planifications (coût de la vie sans frais de logement); une remarque indique le montant ajouté. Les impôts sont calculés séparément. Une valeur réaliste est le facteur individuel le plus important pour le résultat. Consultez pour cela deux à trois ans d’extraits de compte.
- **Montants uniques:** date, montant et description. Un montant positif est une entrée (succession, donation), un montant négatif une dépense (voiture, voyage, rénovation).

Avec l’âge, les dépenses diminuent souvent, par exemple moins de voyages dès 80 ans, alors que les frais de santé augmentent. Utilisez pour cela des [valeurs dépendant du temps](#f-timed).

## @f-timed Valeurs dépendant du temps
Mots-clés: constant, dépendant du temps, ligne, dès, évolution, mini-graphique

De nombreux montants et taux existent sous deux formes:

- **Constant:** une valeur pour toute la simulation.
- **Dépendant du temps:** une suite de lignes ‹dès *moment* s’applique *valeur*›. Le moment est un jalon («dès la prestation de la caisse de pension») ou une date fixe. Chaque ligne s’applique jusqu’à ce que la suivante commence.

Exemple coût de la vie: ‹dès le début 9 500›, ‹dès la prestation de la caisse de pension 8 500›, ‹dès 80 ans 7 000›.

Le petit graphique à côté du champ montre l’évolution. Avec la molette de la souris, vous zoomez; les boutons «Tout», ‹10 ans›, ‹5 ans› et ‹2 ans› choisissent la plage.

Les lignes qui figurent plus bas mais commencent avant une autre sont classées par l’application selon leur moment. Vous pouvez donc déplacer des jalons sans crainte, l’ordre des valeurs reste cohérent.

## @s-wealth Fortune et placement
Mots-clés: actions, obligations, cash, liquidités, quote-part d’actions, part d’actions, réserve, portefeuille, stratégie de placement, indice

Les **soldes au début de la simulation**, la **répartition** du portefeuille et la **stratégie de placement**.

- **Actions, obligations, cash:** vos soldes au début de la simulation. Les biens immobiliers et les avoirs de prévoyance figurent séparément; ici, seulement la fortune liquide. L’application reprend les soldes tels quels au départ. À côté des actions et des obligations figure leur part du portefeuille (sans cash, en pour cent entiers); par exemple, 80 000 d’actions et 20 000 d’obligations donnent «80 % du portefeuille» et «20 % du portefeuille».
- **Le cash ne fait pas partie du portefeuille.** La [quote-part d’actions](#f-stockshare) ne se rapporte qu’aux actions et aux obligations: 80 000 d’actions, 20 000 d’obligations et 50 000 de cash donnent une quote-part d’actions de 80 %.
- **Jusqu’à la retraite** (prestation de la caisse de pension de la première personne), aucune stratégie de prélèvement ne s’applique. L’application détient autant de cash que le champ **Cash** l’indique, indexé au **renchérissement**: un excédent va dans le portefeuille selon la quote-part d’actions, un manque est comblé depuis le portefeuille.
- **Dès la retraite**, la [stratégie de prélèvement](#s-withdrawal) détermine le cash: «Standard» avec la [réserve de cash](#f-reserve) de sa carte, toutes les autres avec 3 mois ou la stratégie à 3 compartiments avec ses compartiments.

- **Stratégie de placement:** un indice pour les actions (Monde, Monde personnalisé, Suisse, S&P 500) et un pour les obligations (Confédération 10 ans ou court terme, USA). Avec «Monde personnalisé», vous fixez les pondérations de la Suisse, des USA, de l’UE et de l’Asie-Pacifique; elles doivent donner ensemble 100 %. Sous la sélection figurent la période commune et les rendements moyens; si un indice est réduit à la période commune, une remarque apparaît. Détails: [Stratégie de placement](#f-ret-series).

Dans la simulation, la fortune est extrapolée chaque mois avec le rendement des actions et des obligations: à partir des indices de la stratégie de placement, selon la méthode sous [Simulation](#s-returns) (rendements fixes, évolution historique ou Monte Carlo).

## @f-stockshare Part d’actions
Mots-clés: risque, actions, rééquilibrage, portefeuille

La part des **actions dans le portefeuille** (actions + obligations, sans cash). Le reste est constitué d’obligations.

- L’application rétablit cette part chaque mois (**rééquilibrage**), pour autant que la stratégie ne prévoie rien d’autre.
- Une part d’actions élevée apporte en moyenne plus de rendement, mais aussi des creux plus profonds. Le risque est particulièrement élevé au début de la retraite (**risque de séquence**): un krach boursier peu après la retraite nuit beaucoup plus que le même krach 20 ans plus tard.
- La valeur peut dépendre du [temps](#f-timed), par exemple 70 % jusqu’à la retraite et 50 % ensuite.

L’ampleur de l’effet de la part d’actions sur le résultat est montrée par une [variation](#f-sweep) de la part d’actions, ou par l’[optimisation](#s-optimize), qui recherche aussi la part.

## @f-reserve Réserve de cash
Mots-clés: cash, réserve, mois, liquidités, réserve de liquidités

Combien de **mois de dépenses non couvertes** (dépenses moins revenus et rentes) sont détenus en cash **dès la retraite**, avant que le reste ne soit investi selon la [quote-part d’actions](#f-stockshare). Avant la retraite, c’est le champ «Cash» sous [Fortune et placement](#s-wealth) qui s’applique.

- Une réserve élevée protège contre la nécessité de vendre des actions à des cours bas, mais coûte du rendement.
- Réglable uniquement avec la stratégie «Standard» (carte «Standard» sous «Stratégie de prélèvement») et efficace seulement dès la retraite. **Toutes les autres stratégies** (Guardrails, taux fixe, VPW, Endowment) calculent dès la retraite avec une **réserve fixe de 3 mois**; avec la [stratégie à 3 compartiments](#f-wd-three_buckets), les compartiments remplacent la réserve.
- Une ancienne planification avec une autre stratégie et une réserve différente de 3 mois calcule donc avec 3 mois et peut fournir des résultats légèrement différents. L’application le signale une fois, à la première ouverture d’une telle planification; ensuite, la remarque n’apparaît plus. La valeur enregistrée reste dans le fichier et s’applique de nouveau si vous passez à «Standard».
- Utile en fonction du temps: par exemple 6 mois jusqu’à la retraite et 12 mois ensuite.

## @s-taxes Impôts
Mots-clés: impôt sur le revenu, impôt sur la fortune, impôt sur les prestations en capital, coefficient d’impôt, barème

Des **barèmes de taux** simples, que vous pouvez tirer du calculateur d’impôts de votre canton. Entre les paliers, l’interpolation est **linéaire**; le taux est multiplié par le [coefficient d’impôt](#f-tax-multiplier).

- [Coefficient d’impôt et barèmes](#f-tax-tables)
- L’**impôt sur le revenu** est calculé en décembre sur le revenu annuel (rentes comprises), l’**impôt sur la fortune** sur le portefeuille plus l’immobilier net. Les deux sont payés en mai de l’année suivante.
- Un impôt encore dû à la fin de la planification est déduit le dernier mois.
- L’[impôt sur les gains immobiliers](#f-tax-gain) est dû lors de la vente d’un bien immobilier, voir aussi [Immobilier](#s-realestate).

Les valeurs d’exemple ne sont que des valeurs fictives. Pour votre résultat, remplacez-les par les valeurs de votre domicile.

## @f-tax-multiplier Coefficient d’impôt
Mots-clés: canton, commune, multiplicateur

Un facteur appliqué à tous les taux des barèmes, le coefficient d’impôt du **canton et de la commune** ensemble (par exemple 2.0 pour un taux double). Ainsi, vous n’avez pas à réécrire les barèmes si vous changez de domicile, mais seulement à modifier le facteur. Les valeurs de 0 à 10 sont admises.

## @f-tax-gain Impôt sur les gains immobiliers
Mots-clés: gain immobilier, vente, bien immobilier, durée de possession, Zurich, Argovie, canton, seuil d’exonération, tarif, coefficient d’impôt

L’impôt sur le gain réalisé lors de la **vente** d’un bien immobilier. Les règles **diffèrent d’un canton à l’autre** et sont donc entièrement modifiables. **Obligatoire:** si la planification contient un bien immobilier au statut «Propriété» ou «Achat planifié», les règles doivent être définies (modèle Zurich, modèle Argovie ou règles propres, même avec un taux de 0 %). Si elles manquent, c’est une erreur dans la section Impôts, et «Lancer la simulation» ainsi que «Lancer l’optimisation» sont bloqués. Les objets purement loués et les planifications sans bien immobilier n’ont pas besoin de règles.

Deux modèles sont disponibles: **Zurich** (tarif par paliers avec majorations et réductions) et **Argovie** (taux d’imposition direct selon la durée de possession). La **saisie propre** se fonde sur le modèle Argovie.

**Calcul:** gain = valeur de vente − prix d’achat payé. Les investissements générateurs de plus-value, les droits de mutation et les frais de vente ne sont pas déduits. Ensuite:

1. **Seuil d’exonération:** si le gain est inférieur, il est exonéré. À partir de la limite, la **totalité** du gain est imposée.
2. **Modèle Zurich, paliers:** le gain est imposé par paliers. Chaque ligne s’applique à sa part du gain («à» est la largeur du palier); au-delà du dernier palier, son taux s’applique.
3. **Modèle Zurich, durée de possession:** l’impôt est modifié de la majoration ou de la réduction de la ligne correspondante. La dernière ligne dont les années sont atteintes s’applique (durée de possession en années révolues, tirée de la [date d’achat](#f-re-bought)).
4. **Réduction fixe:** une déduction supplémentaire en pour cent, indépendante de la durée de possession. Dans les modèles, elle est fixée à 0.
5. **Coefficient d’impôt:** avec l’option «Multiplier par le coefficient d’impôt», l’impôt est en outre multiplié par le coefficient d’impôt du canton et de la commune (standard: désactivé).

**Modèle du canton de Zurich** (tarif § 225 StG, état 09/2025): paliers 10 % (premiers 4 000), 15 % (6 000), 20 % (8 000), 25 % (12 000), 30 % (20 000), 35 % (50 000), 40 % au-delà. Majoration de 50 % pour une durée de possession inférieure à 1 an, de 25 % inférieure à 2 ans. Réduction de 5 % dès 5 années révolues, puis de 3 points de pourcentage de plus par année jusqu’à 50 % dès 20 ans. Les gains inférieurs à 5 000 sont exonérés. Le coefficient d’impôt n’agit que si l’option ci-dessus est activée.

**Modèle du canton d’Argovie** (§ 109 StG): dans le **modèle Argovie**, au lieu des paliers, un **taux d’imposition direct sur la totalité du gain** s’applique, en fonction de la durée de possession en années révolues: 40 % la première année de possession, puis 2 points de pourcentage de moins par année jusqu’à 20 % la onzième année de possession, ensuite 1 point de pourcentage de moins par année jusqu’à 6 % la 25e année de possession et 5 % dès 25 années révolues. Sans date d’achat, le taux le plus élevé s’applique, et l’application émet un avertissement.

Pour un autre canton, adaptez les barèmes. Vous trouvez les valeurs dans la feuille de tarif de l’administration cantonale des impôts. Vérifiez-les avant de les utiliser; le modèle reflète un état donné et n’est pas un conseil fiscal.

## @f-tax-tables Barèmes fiscaux
Mots-clés: taux, interpolation, revenu, fortune, retrait en capital, personnes mariées

Chaque barème a des paliers composés d’un **montant** et d’un **taux**. Entre eux, l’application interpole linéairement.

| Barème | Montant | Taux |
|---|---|---|
| Impôt sur le revenu | revenu annuel imposable | part du revenu due à titre d’impôt |
| Impôt sur la fortune | fortune imposable | part de la fortune par année |
| Impôt sur les prestations en capital | retrait en capital de l’année | part du retrait |

- **Les taux sont des nombres décimaux:** 0.05 représente 5 %, saisi dans l’interface en pour cent.
- Le montant de l’impôt est taux × montant × coefficient d’impôt, arrondi au centime.
- **Limites:** chaque taux est compris entre 0 % et 100 %, chaque montant est de 0 ou plus. Le contrôle signale au champ les valeurs en dehors.
- **L’impôt ne doit pas baisser:** un montant plus élevé ne donne jamais un impôt plus bas. Si le taux baisse entre deux paliers au point que taux × montant diminue, le contrôle signale un **avertissement** au palier concerné; le calcul est effectué malgré tout. Généralement, c’est alors un taux marginal au lieu du taux moyen qui a été saisi. Avec plusieurs retraits en capital la même année, l’impôt sur un retrait supplémentaire ne devient jamais négatif dans ce cas, mais 0.
- **Les retraits en capital** de la même année civile (caisse de pension et pilier 3a) sont additionnés pour le taux: pour les personnes mariées pour le ménage, pour les couples non mariés par personne (chaque personne paie l’impôt sur ses propres retraits). Si vous répartissez les retraits sur différentes années, le taux baisse.
- **Personnes mariées:** avec deux personnes et la case «Marié·e·s» cochée, vous pouvez enregistrer des barèmes propres pour les couples mariés. Sans barèmes propres, les barèmes normaux s’appliquent.
- Partez de la charge fiscale **effective**, pas du taux marginal. Le calculateur d’impôts de votre canton fournit les deux.

## @s-realestate Immobilier
Mots-clés: maison, appartement, hypothèque, location, achat, vente, capacité financière

Objets en propriété, planifiés et loués. Vous pouvez en saisir autant que vous voulez.

- **Propriété (`owned`):** la valeur vénale au début de la simulation figure dans l’en-tête à côté du nom. Suivent dessous, à des places fixes, la [date d’achat](#f-re-bought), le prix d’achat (payé, pour l’impôt sur les gains immobiliers), «Vente dès» (jamais, jalon ou date) et les frais accessoires par mois. Pour «Achat planifié», les mêmes champs figurent aux mêmes places (au lieu de la date d’achat «Achat dès»), seule la valeur vénale manque; lors d’un changement de statut, les valeurs saisies sont conservées. La vente doit être **postérieure à la date d’achat**, sinon c’est une erreur au champ.
- **Inflation:** la valeur et les frais accessoires peuvent être indexés à l’inflation **séparément** pour chaque objet, de même que le loyer pour une location.
- **Achat planifié (`planned`):** achat dès un jalon, prix d’achat, hypothèque. S’y ajoute **«Vente dès»** (par défaut «jamais», sinon date ou jalon); la simulation vend alors l’objet à ce moment. La vente doit être postérieure à «Achat dès», sinon c’est une erreur au champ. Pour un achat «après la vente de …», la date d’achat n’est pas connue lors de la saisie; ce contrôle n’a alors pas lieu.
- **Location (`rent`):** début et loyer par mois (valeur actuelle, indexé par défaut au renchérissement; l’option «Adapter le loyer à l’inflation» le désactive).
- **Hypothèque:** tranches avec périodes, voir [Hypothèques](#f-re-mortgage).
- [Vente forcée](#f-re-emergency) et [capacité financière](#f-re-afford) pour les achats «après la vente».
- Le commutateur [Ne jamais vendre les biens immobiliers](#f-re-never-sell) en haut de l’onglet exclut toute vente.

Vous saisissez le [coût de la vie](#s-flows) **frais de logement compris**. Le calcul utilise le coût de la vie **sans la part du logement**; loyer, intérêts hypothécaires, amortissement et frais accessoires s’ajoutent comme **frais de logement** simulés. Après la vente du dernier bien immobilier, le suivant avec achat «après la vente» est acheté le mois suivant, pour autant qu’il soit supportable et que les fonds propres suffisent. Sinon, les locations ‹dès après la vente› commencent. Un achat «après la vente» est tenté **une seule fois** le mois suivant. S’il n’est pas finançable, il **est supprimé** avec un message dans les événements, et les règles subséquentes s’appliquent le mois suivant: les locations ‹dès après la vente› de ce bien immobilier commencent, d’autres achats «après la vente» de celui-ci sont tentés. Une **date d’achat ou de vente antérieure au début de la simulation** n’est jamais atteinte; le contrôle la signale comme avertissement.

## @f-re-bought Date d’achat
Mots-clés: durée de possession, impôt sur les gains immobiliers, achat

Le mois au cours duquel vous avez acheté le bien immobilier. Il détermine la **durée de possession** et donc la majoration ou la réduction de l’[impôt sur les gains immobiliers](#f-tax-gain): plus vous possédez le bien longtemps, moins l’impôt est élevé dans de nombreux cantons.

- Le champ n’existe que pour les biens immobiliers en propriété. Pour un bien immobilier **planifié**, c’est le mois d’achat de la simulation qui compte.
- Sans date d’achat, l’application calcule **sans** majoration ni réduction (l’impôt selon le tarif de base). Si des règles de durée de possession sont saisies, le contrôle le signale.
- C’est la durée de possession en **années révolues** jusqu’au mois de la vente qui fait foi.

## @f-re-mortgage Hypothèques
Mots-clés: tranche, SARON, hypothèque à taux fixe, amortissement, marge, intérêt

Une hypothèque se compose de **tranches**. Chaque tranche a une suite de **périodes**:

- **fixe:** taux d’intérêt fixe pour la durée de la période
- **SARON:** SARON plus marge (le SARON a un plancher de 0). L’évolution du SARON est fournie par la série temporelle ou la valeur fixe, voir [SARON](#ret-saron).

La première période commence à la date de début de la tranche (à l’achat), chaque période suivante à la fin de la précédente. Ensuite, la tranche est prolongée aux mêmes conditions ou remboursée.

- **Intérêt** = dette × taux / 12 par mois. Il s’accumule et est payé avec l’amortissement à la **fin du trimestre** (mars, juin, septembre, décembre).
- **Amortissement:** montant fixe par trimestre, direct (réduit la dette) ou **indirect** (versement au [pilier 3a](#s-pillar3a) de la personne; avec deux personnes, vous choisissez le compte à la tranche).
- **Amortissement indirect seulement jusqu’au début de la rente AVS:** dès le mois du début de la rente AVS de la personne, le même montant est amorti **directement** et réduit l’hypothèque. Une période qui ne commence qu’à partir du début de la rente AVS (y compris un renouvellement) amortit toujours directement; la sélection affiche alors «direct (dès l’AVS)». L’aperçu de la période mentionne ‹+ 3a jusqu’à l’AVS› lorsqu’une période dure au-delà du début de la rente AVS. Comme chaque personne a un début de rente AVS, la règle s’applique toujours. Le changement apparaît dans les [événements](#r-events) (‹Amortissement de l’hypothèque … dès la rente AVS direct au lieu du pilier 3a›) lorsque la tranche amortissait auparavant indirectement et continue d’être amortie.
- Si le pilier 3a est retiré avec «Rembourser l’hypothèque», les montants vont d’abord aux tranches avec amortissement indirect sur ce compte.
- En cas de vente, les intérêts accumulés sont payés immédiatement.

## @f-re-emergency Vente forcée
Mots-clés: vente, liquidités, seuil, vente forcée, vente contrainte

Si la fortune liquide devient insuffisante, l’application vend le bien immobilier le mois suivant. Le **seuil** est exprimé en mois de dépenses non couvertes: si la fortune liquide couvre moins de mois (dépenses moins revenus), une vente forcée est nécessaire.

Une vente forcée montre que la planification ne fonctionne pas sans vente immobilière. Si [Ne jamais vendre les biens immobiliers](#f-re-never-sell) est activé, il n’y a pas de vente forcée. Elle apparaît dans les [événements](#r-events) des résultats. Plus le seuil est élevé, plus la vente intervient tôt.

## @f-re-never-sell Ne jamais vendre les biens immobiliers
Mots-clés: commutateur, vente, vente forcée, capacité financière, sans vente, conserver

Avec ce commutateur en haut de l’onglet «Immobilier», vous vérifiez si la planification fonctionne aussi sans vendre de bien immobilier. Il s’applique à **tous** les biens de la planification et est enregistré dans le fichier (par défaut: désactivé).

- **Aucune vente:** pas de vente planifiée («Vente dès»), pas de [vente forcée](#f-re-emergency) et pas de vente parce qu’une hypothèque n’est plus [supportable](#f-re-afford) lors du renouvellement. L’hypothèque est alors **renouvelée en totalité**; un événement signale «non supportable».
- **Les coûts continuent:** intérêts, amortissement et frais accessoires jusqu’à la fin de la planification.
- **Achats:** un achat planifié avec une date ou un jalon a lieu. Les achats et locations «après la vente» n’ont **pas** lieu; la vérification le signale, de même que les ventes planifiées.
- **Fortune épuisée:** si la fortune liquide ne suffit plus, la simulation compte comme épuisée, comme sans bien immobilier. Le taux de réussite est donc en général plus bas qu’avec une vente.
- **Lecture seule:** tant que le commutateur est activé, tous les autres champs de l’onglet sont grisés et en lecture seule. Les valeurs sont conservées et s’appliquent à nouveau sans changement lors de la désactivation.

## @f-re-afford Capacité financière
Mots-clés: banque, capacité financière, taux d’avance, taux d’intérêt théorique, fonds propres, hypothèque

Le calcul de la capacité financière selon les règles des banques, appliqué aux achats «après la vente» et à chaque renouvellement d’une période hypothécaire. Une hypothèque est supportable si les **coûts théoriques** ne dépassent pas une part du revenu.

- **Part maximale du revenu:** généralement un tiers (33 %).
- **Taux d’intérêt théorique:** généralement 5 %, indépendamment de l’intérêt actuel.
- **Frais accessoires:** généralement 1 % du prix par année.
- **Revenu:** seuls le revenu de l’activité lucrative et les rentes sont pris en compte, **pas** le prélèvement sur la fortune. Si le revenu ne suffit pas, l’hypothèque est réduite; l’achat montre alors le **remboursement unique** prélevé sur la fortune comme événement.
- **Taux d’avance maximal:** généralement 80 % du prix. Au moins 20 % sont des fonds propres.

Si l’achat n’est pas supportable, l’hypothèque est réduite. Si même alors les fonds propres ne suffisent pas, l’achat n’a pas lieu et les locations commencent.

**À chaque renouvellement** d’une période hypothécaire (y compris pour les biens immobiliers en propriété), l’application vérifie à nouveau la capacité financière de toute l’hypothèque, avec le revenu du mois du renouvellement (activité lucrative et rentes). Si l’hypothèque n’est plus supportable, la simulation calcule le **paiement unique minimal nécessaire** pour qu’elle le redevienne, et le paie à partir de la fortune (au prorata sur toutes les tranches). Cela ne vaut que si la fortune n’est **pas négative** ensuite **et** que le [critère de vente forcée](#f-re-emergency) ne se déclenche **pas** après le paiement (estimé avec les dépenses planifiées et les frais de logement après le paiement). L’hypothèque est alors renouvelée avec le montant réduit. Sinon, il n’y a **pas de paiement**, l’hypothèque n’est **pas renouvelée**, et une **vente forcée** du bien immobilier est engagée (exécutée le mois suivant). La vue des résultats consigne chaque renouvellement (date et âge, capacité financière, paiement unique, nouvelle hypothèque, vente forcée avec le motif «Fortune < 0» ou «critère de vente forcée») et compte, avec plusieurs simulations, dans combien de simulations des paiements uniques et des ventes forcées surviennent, aussi par percentile. Cas typique: le revenu baisse avec la retraite.

## @s-scenarios Scénarios et simulation
Mots-clés: variation, sweep, paramètres, simulations, ampleur

Vous définissez ici quels paramètres **varient systématiquement**. «Lancer la simulation» calcule la planification telle que saisie et, en plus, chaque combinaison des variations avec toutes les trajectoires de rendement. Le résultat est un tableau et une carte thermique dans les [résultats des scénarios](#r-scenarios).

La durée de la simulation figure en outre à gauche du bouton «Lancer la simulation». Pendant le calcul, vous pouvez l’arrêter avec **Interrompre**.

Le nombre de combinaisons n’est **pas limité**; ce n’est qu’au-delà de **1'000'000** combinaisons que le moteur de calcul refuse (protection de la mémoire). Si le temps de calcul estimé dépasse **une minute**, l’application demande confirmation avant le lancement (durée et nombre de simulations, «Lancer quand même» ou «Interrompre»). L’affichage «Volume» montre combien de simulations sont produites au total (variations × trajectoires); un **symbole d’avertissement jaune (⚠)** apparaît à côté dès que le temps de calcul estimé dépasse **90 secondes**, ainsi qu’une **estimation approximative de la durée**. Elle repose sur une brève mesure de votre planification sur cet ordinateur. La fourchette (‹généralement … à …›) tient compte du fait que les simulations durent plus ou moins longtemps selon la planification. Chaque variation supplémentaire et chaque trajectoire supplémentaire allonge le temps de calcul en conséquence: 35 combinaisons avec 50 000 trajectoires durent environ 10 secondes avec le bootstrap, environ une minute avec le modèle paramétrique.

Les variations elles-mêmes: [Variations](#f-sweep).

## @f-sweep Variations
Mots-clés: sweep, de, à, pas, paramètres, carte thermique

Une variation a un **paramètre**, une plage **De … À** et un **pas**. L’application calcule chaque valeur et montre l’effet sur le taux de réussite et la fortune finale.

| Paramètre | Signification |
|---|---|
| Part d’actions | quote-part d’actions: actions dans le portefeuille composé d’actions et d’obligations (sans cash) |
| Retrait en capital de la caisse de pension | part du retrait en capital |
| Mois de départ historique | seulement avec la source «historique» |
| Date de la prestation de la caisse de pension | mois du jalon «Prestation de la caisse de pension» |
| Facteur de dépenses | facteur appliqué au coût de la vie sans la part du logement (0.9 = 10 % de moins); les frais de logement restent |
| Taux de prélèvement (taux fixe / départ Guardrails) | taux du prélèvement fixe ou taux initial pour Guardrails |

Avec deux variations apparaît une **carte thermique** sur les deux paramètres. Toutes les combinaisons utilisent **les mêmes trajectoires de rendement**. Les différences ne proviennent ainsi que des paramètres, et non du hasard.
