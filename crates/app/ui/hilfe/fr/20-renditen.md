# Simulation et rendements

## @s-returns Simulation: vue d’ensemble
Mots-clés: rendement, inflation, source, méthode, série temporelle, Monte Carlo, SARON, stratégie de placement

La section «Simulation» détermine **comment les rendements des actions et des obligations, l’inflation et le SARON évoluent dans le temps**: valeurs fixes, évolution historique ou Monte Carlo. C’est elle qui a la plus grande influence sur l’allure des résultats et sur le crédit que vous pouvez leur accorder.

Vous définissez **quels indices** fournissent les rendements (par exemple actions mondiales et obligations de la Confédération) dans la [stratégie de placement](#f-ret-series) de la section «Fortune et placement». Vous ne choisissez plus vous-même de série temporelle; elle découle de la stratégie.

Il existe cinq méthodes. La vue n’affiche que les saisies qui s’appliquent à la méthode choisie.

| Source | Simulations | Pour quoi |
|---|---|---|
| [Rendements fixes](#f-ret-fixed) | 1 | estimation rapide, aperçu, contrôle de plausibilité |
| [Historique dès la date de départ](#f-ret-historical) | 1 | ‹Que se serait-il passé si j’avais pris ma retraite en 1973?› |
| [Toutes les dates de départ historiques](#f-ret-rolling) | env. 400 à 900 | comment chaque départ réel se serait déroulé |
| [Monte Carlo · historique (bootstrap)](#f-ret-bootstrap) | libre, p. ex. 50 000 | large dispersion à partir de vraies données mensuelles |
| [Monte Carlo · paramétrique](#f-ret-parametric) | libre, p. ex. 50 000 | modèle avec valeurs proposées par indice ou hypothèses propres |

Si vous ne savez pas par quoi commencer: lisez [Quelle source convient?](#ret-which).

Les bases qui valent pour toutes les sources figurent sous [Rendement, inflation et valeurs réelles](#ret-concepts). Les erreurs de raisonnement fréquentes sont décrites sous [Pièges](#ret-pitfalls).

## @ret-which Quelle source convient?
Mots-clés: décision, recommandation, choix, choisir la source

Une règle empirique:

1. **Pour saisir et essayer:** «Rendements fixes». L’aperçu à droite calcule toujours avec des valeurs fixes et réagit immédiatement.
2. **Pour la vraie réponse à ‹L’argent suffira-t-il?›:** Monte Carlo, idéalement **«Monte Carlo · historique» (bootstrap)** avec une stratégie de placement qui remonte loin dans le temps (par exemple «Monde (JST)» ou «S&P 500»). Il préserve les vraies propriétés des marchés (crises, reprises, lien entre actions, obligations et inflation) tout en offrant une large dispersion.
3. **Comme contre-épreuve:** «Toutes les dates de départ historiques». Cette source montre concrètement comment le plan se serait déroulé dans des périodes réelles, par exemple avec un départ en 1929, 1966 ou 2000.
4. **Pour des hypothèses propres:** «Monte Carlo · paramétrique», si vous voulez fixer vous-même le rendement attendu et le risque, par exemple parce que vous attendez à l’avenir des rendements plus bas que par le passé.

**Bonne pratique:** calculez les décisions importantes avec **au moins deux sources** (bootstrap et paramétrique) et vérifiez si la conclusion reste la même. Un plan qui n’a l’air bon qu’avec une seule source est fragile.

| Question | Source |
|---|---|
| Quelle est la probabilité que mon argent suffise? | bootstrap ou paramétrique |
| Jusqu’où cela peut-il aller mal? | bootstrap, 1er percentile et pire simulation |
| Le plan aurait-il fonctionné en 1973? | historique dès la date de départ |
| Le plan fonctionne-t-il dans chaque situation historique? | toutes les dates de départ historiques |
| Et si la bourse rapporte moins à l’avenir? | paramétrique avec μ plus bas |

## @ret-concepts Rendement, inflation et valeurs réelles
Mots-clés: nominal, réel, renchérissement, géométrique, rendement annuel, pouvoir d’achat

Les **rendements annuels** sont appliqués mensuellement dans l’application. Un rendement annuel de 5 % donne par mois (1.05)^(1/12) − 1, soit environ 0.41 %. L’évolution annuelle est ainsi exacte.

L’**inflation** agit chaque mois. L’indice du renchérissement croît chaque mois, et toutes les dépenses, loyers et (si choisi) valeurs immobilières sont indexés avec lui. Vous saisissez vos dépenses aux **prix actuels**, l’application les projette dans l’avenir avec l’inflation.

**Nominal et réel:** les résultats existent en deux vues.

- **Nominal:** montants tels qu’ils surviennent alors (en francs futurs).
- **Pouvoir d’achat actuel:** montants aux prix du début de la simulation, corrigés du renchérissement. Cette vue est généralement la plus honnête pour décider, car 2 millions dans 30 ans ont moins de pouvoir d’achat qu’aujourd’hui. Les résultats s’ouvrent dans cette vue.

Avec plusieurs simulations, les percentiles dépendent de la vue choisie: en pouvoir d’achat actuel, les simulations sont classées selon la fortune finale réelle, en nominal selon la nominale (voir [Fortune finale par percentile](#r-percentiles)).

**Arithmétique et géométrique:** un ‹rendement moyen de 7 %› est généralement une moyenne arithmétique. La moyenne **géométrique** (ce dont votre fortune croît réellement) est plus basse en raison des fluctuations. Avec une espérance de 7 % et une volatilité de 15 %, elle est d’environ 5.9 %. L’info-bulle de la stratégie de placement montre le rendement annuel moyen géométrique.

**Rendement global:** toutes les séries contiennent les dividendes et les coupons (total return), sans frais de placement ni impôts sur les revenus.

## @f-ret-source Source des rendements
Mots-clés: choix, fixe, historique, glissant, bootstrap, paramétrique

Choisissez la source avec les cartes en haut de la section. Chaque carte a un ⓘ avec l’explication détaillée:

- [Rendements fixes](#f-ret-fixed)
- [Historique dès la date de départ](#f-ret-historical)
- [Toutes les dates de départ historiques](#f-ret-rolling)
- [Monte Carlo · historique](#f-ret-bootstrap)
- [Monte Carlo · paramétrique](#f-ret-parametric)

Si vous n’êtes pas sûr de celle qui convient, lisez [Quelle source convient?](#ret-which).

## @f-ret-fixed Rendements fixes
Mots-clés: constant, actions, obligations, inflation, SARON, simulation unique

Une seule simulation avec des **valeurs annuelles constantes** pour les actions, les obligations, l’inflation et le SARON (pour les hypothèques SARON).

- Il n’y a **pas de dispersion**: le résultat est ‹l’argent suffit› ou ‹ne suffit pas›. Il n’y a pas de probabilité de réussite.
- Utile pour saisir, pour des comparaisons rapides et pour l’aperçu à droite. **Pas adapté** pour juger si le plan est sûr: il ignore entièrement le risque de séquence.
- **Choix des valeurs:** calculez avec des valeurs prudentes et **réalistes**, pas avec les années record du passé. Une hypothèse sobre serait par exemple 4 % à 5 % pour les actions, 1 % à 2 % pour les obligations, 1 % à 2 % d’inflation.
- Avec les stratégies «Taux fixe», «Guardrails» et autres, une simulation avec rendements fixes ne montre que le cas moyen.

Avec les autres sources, les valeurs fixes s’appliquent malgré tout en partie: pour le **SARON** en dehors de la période de la série et avec le [modèle paramétrique](#f-ret-parametric), ainsi qu’après la fin de la série lors de l’[extrapolation](#f-ret-extrapolation).

## @f-ret-historical Historique dès la date de départ
Mots-clés: rétrospective, mois de départ, 1973, 1929, backtest, série temporelle

Une seule simulation avec l’**évolution réelle des rendements** de la [stratégie de placement](#f-ret-series), à partir du **mois de départ historique**. La simulation démarre alors comme si vous aviez commencé ce mois-là.

- **Exemple:** le mois de départ 01.1973 vérifie ce qui se serait passé si vous aviez pris votre retraite avec votre planification au début de la crise pétrolière (forte inflation, bourse faible).
- La simulation couvre la période de la planification, par exemple 32 ans dès le mois de départ. Si la série se termine plus tôt, l’[extrapolation](#f-ret-extrapolation) prend le relais.
- Pour les stratégies annuelles (toutes sauf S&P 500 avec obligations américaines ou avec obligations de la Confédération), seule l’année compte.
- **Les simulations individuelles sont des anecdotes.** Un plan qui aurait fonctionné en 1973 peut échouer dans une autre situation. Pour une conclusion sur la sécurité, prenez [toutes les dates de départ](#f-ret-rolling) ou [Monte Carlo](#f-ret-bootstrap).
- Avec une [variation du mois de départ](#f-sweep), toute une série de départs historiques peut être calculée. C’est semblable à ‹toutes les dates de départ›, mais avec une grille choisie par vous.

## @f-ret-rolling Toutes les dates de départ historiques
Mots-clés: glissant, rolling, itération, chevauchement, date de départ

L’application calcule **chaque mois de départ possible** pour lequel la série couvre toute la période de planification. Avec une série dès 1922 et une planification sur 32 ans, cela représente quelques centaines de simulations: départ en janvier 1922, février 1922 et ainsi de suite jusqu’au dernier mois qui laisse encore 32 ans jusqu’à la fin de la série.

**Forces:** chaque simulation est une période **réelle** avec les vrais liens entre actions, obligations et inflation. Vous voyez les pires départs historiques (par exemple 1929, 1966, 1973, 2000).

**Faiblesses à connaître:**

- Les simulations **se chevauchent fortement**. 104 ans de données donnent 800 mois de départ, mais à peine **trois périodes de 32 ans vraiment indépendantes**. La dispersion est donc **sous-estimée**, et ‹98 % des simulations réussissent› paraît plus sûr que ce n’est le cas.
- Il n’existe qu’**un seul** passé. Une séquence plus défavorable, qui ne s’est jamais produite, n’apparaît pas.
- Avec de longues périodes de planification et des séries courtes, il y a très peu de points de départ. Avec une série annuelle (par exemple «Suisse (JST)», 121 ans) et une planification de 32 ans, il n’y en a qu’environ 90.

Le [bootstrap](#f-ret-bootstrap) corrige en partie ces faiblesses et montre donc des fourchettes plus larges. Prenez «Toutes les dates de départ historiques» comme contre-épreuve, pas comme source unique.

Le **nombre** de simulations est indiqué dans la section «Passage».

## @f-ret-bootstrap Monte Carlo · historique (bootstrap)
Mots-clés: bootstrap, longueur des blocs, trajectoires, valeur initiale aléatoire, seed, stationnaire, Monte Carlo

Le bootstrap génère **des milliers de nouvelles trajectoires de rendement** en enchaînant des segments aléatoires de la série temporelle de la stratégie de placement. Il en résulte des évolutions qui n’ont jamais existé, mais qui sont composées de **vraies données mensuelles**.

**Fonctionnement:**

1. La trajectoire commence à un mois aléatoire de la série.
2. Chaque mois, le **bloc** en cours se termine avec la probabilité 1 / longueur des blocs. Un nouveau bloc commence alors à un mois de départ aléatoire. À la fin de la série, on reprend au début.
3. Les actions, les obligations, l’inflation et le SARON proviennent toujours **du même mois historique**. Les liens (par exemple krach boursier avec forte inflation) sont conservés.

**Saisies:**

- **Nombre de trajectoires:** plus de trajectoires donnent des chiffres plus lisses, mais un temps de calcul plus long. 10 000 suffisent pour une première réponse, 50 000 pour les percentiles extrêmes (1er percentile, pire simulation). Le taux de réussite est un résultat aléatoire: avec 10 000 trajectoires, il est précis à environ ±0.4 point de pourcentage, avec 50 000 à environ ±0.2 (environ deux erreurs types pour un taux de réussite d’environ 95 %). **Limite supérieure:** trajectoires × mois de la planification au plus 50 mio, afin que la mémoire vive suffise (sur 30 ans environ 138 000 trajectoires, sur 100 ans 41 666). Au-delà, le contrôle signale une erreur au champ avec le nombre maximal possible.
- **Longueur moyenne des blocs (mois):** durée moyenne des segments contigus.
  - **Courte (1 à 6):** les mois sont mélangés de façon presque indépendante. Cela détruit la propriété selon laquelle de mauvaises années sont souvent suivies d’autres (**momentum et reprise**) et rend les trajectoires **trop lisses**.
  - **Longue (24 et plus):** les trajectoires restent proches de l’histoire, la diversité diminue.
  - **12 mois** (standard) est un compromis éprouvé. Pour les séries annuelles, la longueur s’entend en années (longueur des blocs / 12).
- **Valeur initiale aléatoire (seed):** même valeur, mêmes trajectoires. Voir [Valeur initiale aléatoire](#ret-seed).

**Caractéristiques:**

- Les trajectoires sont **plus dispersées** qu’avec «Toutes les dates de départ historiques», car elles ne sont pas liées à l’unique ordre de l’histoire. C’est l’incertitude la plus honnête.
- Le bootstrap ne peut montrer que **ce que contiennent les données**. Des rendements élevés dans le passé conduisent à des rendements élevés à l’avenir. Si vous voulez calculer plus prudemment, prenez la [source paramétrique](#f-ret-parametric) avec un μ plus bas ou une série plus courte qui ne contient pas seulement des années fastes.
- Les vues détaillées (5e, 50e, 95e percentile et les simulations extrêmes) recalculent exactement la trajectoire concernée.

## @f-ret-parametric Monte Carlo · paramétrique
Mots-clés: Student t, queues épaisses, volatilité, dérive, sauts de cours, mu, sigma, degrés de liberté, Poisson, krach, valeurs proposées, corrélation, obligations, inflation

Un **modèle des actions avec des hypothèses propres ou proposées**. Les cours des actions suivent un mouvement aléatoire en **pas journaliers** (21 par mois), complété par de rares **sauts de cours**. **Les obligations et l’inflation** sont également aléatoires et **corrélées** aux actions, comme dans la série de la [stratégie de placement](#f-ret-series). Le SARON s’applique de manière constante comme valeur fixe.

| Saisie | Proposition (exemple Monde (JST)) | Signification |
|---|---|---|
| Nombre de trajectoires | 50 000 | comme pour le bootstrap |
| Rendement attendu (dérive μ) | 12.15 % | dérive du rendement logarithmique plus σ²/2; le rendement annuel moyen (arithmétique) sans les sauts |
| Volatilité σ | 12.25 % | fluctuation des rendements annuels sans les sauts |
| Degrés de liberté | 6 | épaisseur des queues (‹fat tails›), 0 = loi normale |
| Sauts de cours par année | 0.15 | fréquence d’un krach (0.15 = environ tous les 7 ans en moyenne) |
| Amplitude moyenne des sauts (log) | −25 % | taille du saut; −25 % en rendement logarithmique correspondent à environ −22 % de perte de cours |
| Dispersion de l’amplitude des sauts | 15 % | à quel point les sauts diffèrent |
| Valeur initiale aléatoire | | voir [Valeur initiale aléatoire](#ret-seed) |

**Valeurs proposées par indice des actions:** l’application calcule les paramètres à partir de toute la série des actions de l’indice choisi, de sorte que les rendements annuels du modèle (moyenne, dispersion, moyenne géométrique et quantile à 5 %) correspondent à ceux du [bootstrap historique](#f-ret-bootstrap). Les degrés de liberté (6) et la dispersion de l’amplitude des sauts (15 %) restent fixes.

| Indice des actions | μ | σ | Sauts par année | Amplitude des sauts | Années |
|---|---|---|---|---|---|
| Monde (JST) | 12.15 % | 12.25 % | 0.15 | −25 % | 1900–2020 |
| Suisse (JST) | 10.55 % | 14.5 % | 0.26 | −10 % | 1900–2020 |
| S&P 500 (Shiller) | 15.05 % | 16.5 % | 0.26 | −20 % | 1922–2025 |

«Monde personnalisé» reçoit des valeurs tirées de sa propre série pondérée. Après une mise à jour des données Shiller, les valeurs pour le S&P 500 changent aussi. Pour le S&P 500, le bootstrap tire des mois; ses années sont donc un peu plus clémentes que les années civiles, et les valeurs proposées suivent le bootstrap.

- **Nouvelle planification:** les valeurs suivent l’indice des actions. Si vous changez d’indice sous «Fortune et placement», les valeurs proposées du nouvel indice s’appliquent.
- **Valeurs propres:** dès que vous modifiez l’une des six valeurs du modèle, vos valeurs sont conservées, même en cas de changement d’indice. À côté de chaque champ figure alors la proposition pour l’indice choisi. Avec **«Reprendre les valeurs proposées»**, les valeurs suivent de nouveau l’indice.
- **Les anciennes planifications** qui calculaient déjà en mode paramétrique conservent leurs valeurs. À l’ouverture, une remarque apparaît une fois pour indiquer que les obligations et l’inflation sont désormais aléatoires.

**Obligations et inflation:** la carte «Obligations et inflation» montre la moyenne, la dispersion et la corrélation des actions, des obligations et de l’inflation, calculées à partir des valeurs annuelles de la même série que celle utilisée par le bootstrap. Elle dépend de la combinaison de l’indice des actions et de l’indice des obligations. Exemples de corrélation actions/obligations: Monde avec Confédération 10 ans −0.01, Suisse avec Confédération 10 ans 0.19, S&P 500 avec USA 10 ans 0.45 (les deux portent le cours du dollar). Chaque année civile, le modèle tire pour les obligations et l’inflation une valeur corrélée au rendement des actions de la même année. Les années sont indépendantes les unes des autres comme dans le bootstrap; il n’y a pas de mémoire pour l’inflation ou les taux d’intérêt. Sans série temporelle, les valeurs fixes s’appliquent aux obligations et à l’inflation.

**Mathématiques en bref:** le rendement logarithmique par jour est (μ − σ²/2)·dt + σ·√dt·Z. Z est un choc **distribué selon une loi de Student t**, normalisé à la variance 1. S’y ajoutent des **sauts de Poisson** au taux «Sauts de cours par année» et d’amplitude log-normale. Les obligations et l’inflation d’une année suivent une loi normale avec la moyenne et la dispersion de la série; leur dépendance au rendement logarithmique standardisé des actions est choisie de sorte que la corrélation avec le rendement des actions corresponde à la série.

**Comment choisir vos propres valeurs:**

- **μ:** à long terme, des actions largement diversifiées ont historiquement rapporté environ 5 % à 7 % par année en termes réels, et d’autant plus en termes nominaux. Qui planifie prudemment prend des valeurs plus basses que par le passé, car les valorisations sont élevées aujourd’hui. Attention: les sauts de cours tirent le rendement encore plus vers le bas; le rendement géométrique attendu figure dans l’aperçu à droite.
- **σ:** 12 % à 20 % est typique pour les actions; avec des sauts fréquents, la valeur est plus faible, car les sauts portent une partie de la fluctuation.
- **Degrés de liberté:** des valeurs basses (3 à 5) produisent des jours extrêmes, 6 est modéré. Avec 0, les queues épaisses disparaissent et les crises deviennent irréalistement rares.
- **Sauts de cours:** avec les queues épaisses, ils font en sorte que des krachs comme 2008 ou 2020 surviennent de manière plausible. Si vous mettez le taux à 0, il ne reste que le mouvement aléatoire.

**Quand est-ce utile:** comme contre-épreuve au bootstrap, avec des distributions lisses au lieu du nombre limité d’années historiques, et si vous voulez poser vos propres hypothèses (par exemple ‹à l’avenir 4 % de rendement en moins›). **Limites:** les actions suisses reviennent à la moyenne sur plusieurs années; ni le bootstrap ni le modèle ne le reproduisent. La corrélation est moyennée sur toutes les années; lors des années de forte inflation, les actions et les obligations ont historiquement plus souvent baissé ensemble. Avec «S&P 500» et «Confédération 10 ans», la série ne couvre que les années dès 1988, mais les paramètres des actions toute la série dès 1922.

## @f-ret-series Stratégie de placement: indices pour les actions et les obligations
Mots-clés: stratégie de placement, indice, monde, JST, Pictet, Shiller, S&P 500, pondérations, régions, UE, USA, Suisse, Asie-Pacifique, série temporelle, BNS, monnaie, court terme, marché monétaire

Les méthodes «Historique», «Toutes les dates de départ historiques» et «Bootstrap» nécessitent une série temporelle avec les rendements des actions, des obligations et l’inflation. Elle découle de la **stratégie de placement** de la section «Fortune et placement»: un indice pour les actions et un pour les obligations.

**Actions**

| Indice | Contenu |
|---|---|
| **Monde (JST)** | actions de 11 pays (Suisse, USA, Allemagne, France, Italie, Espagne, Pays-Bas, Belgique, Grande-Bretagne, Japon, Australie), pondérées chaque année selon le PIB réel de l’année précédente, en CHF sans couverture de change. **Annuel 1900–2020.** |
| Monde personnalisé (JST) | comme «Monde (JST)», mais vous fixez vous-même les pondérations de la **Suisse, des USA, de l’UE et de l’Asie-Pacifique** (ensemble 100 %). Au sein de l’UE (Allemagne, France, Italie, Espagne, Pays-Bas, Belgique, Grande-Bretagne) et de l’Asie-Pacifique (Japon, Australie), les pays sont pondérés selon le PIB. Si la pondération de l’UE change, la part de chaque pays de l’UE change dans la même proportion: UE 20 % au lieu de 40 % divise par deux chaque pays de l’UE. Les pondérations s’appliquent à nouveau chaque année (rééquilibrage annuel). |
| Suisse (JST) | actions suisses, rendement global. Annuel 1900–2020. |
| **S&P 500 (Shiller)** | actions américaines (S&P Composite) en CHF, sans couverture de change. **Mensuel dès 1922.** |

**Obligations**

| Indice | Contenu |
|---|---|
| **Confédération 10 ans** | obligations de la Confédération à 10 ans: JST jusqu’en 1988, ensuite à partir des rendements mensuels de la BNS. Annuel 1900–2024; avec le S&P 500, mensuel dès 02.1988. |
| USA 10 ans (Shiller) | emprunts d’État américains à 10 ans en CHF, sans couverture de change. Mensuel dès 1922. |
| Confédération court terme | titres à court terme de la Confédération (durée environ 3 mois): JST jusqu’en 2020, ensuite la moyenne annuelle du SARON (BNS). **Annuel 1900–2025.** Le rendement correspond pratiquement au taux du marché monétaire: très peu de fluctuation, mais à long terme nettement moins de rendement que les obligations à 10 ans. |

**Comment la série temporelle est constituée**

- Seules les combinaisons «S&P 500» avec «USA 10 ans» (dès 01.1922) et «S&P 500» avec «Confédération 10 ans» (02.1988 à 07.2025) sont **mensuelles**.
- Toutes les autres combinaisons sont **annuelles** dès qu’une partie est annuelle. Les rendements mensuels sont alors chaînés en années civiles. La série comprend les années pour lesquelles les deux indices ont des valeurs: par exemple «Monde (JST)» avec «USA 10 ans» 1922–2020.
- Le **renchérissement** provient de la source des actions (renchérissement suisse dans tous les cas), le **SARON** des valeurs de la BNS (pour les séries annuelles en moyenne annuelle, dès 2000).
- Tous les indices sont **en CHF**. Pour une planification en EUR ou en USD, une remarque apparaît.

Sous la sélection figurent la période, la fréquence et les rendements annuels moyens (géométriques) de la stratégie choisie; le ⓘ à côté indique la source et la période.

**Période commune:** les rendements moyens valent pour les années que les actions et les obligations ont en commun. Si un indice va plus loin, il est réduit à cette période, et une remarque sous la sélection indique l’indice, ses propres années et la période réduite. La simulation (Monte Carlo et évolutions historiques) n’utilise que la période commune. C’est pourquoi, par exemple, le rendement moyen des obligations «USA 10 ans» change lorsque, pour les actions, vous passez de «S&P 500» (jusqu’en 2025) à «Suisse (JST)» (jusqu’en 2020): les obligations ne comptent alors que jusqu’en 2020.

**Comment choisir?**

- **Monde (JST):** largement diversifié, longue histoire, y compris guerres mondiales et hyperinflation. Attention: l’indice est plutôt **optimiste**. Pour 1900–2020, il dépasse d’environ 1.3 point de pourcentage par année l’indice mondial connu de Dimson, Marsh et Staunton (5.3 % réel en USD), surtout en raison de la pondération annuelle selon le PIB et parce qu’il ne contient que des pays qui existent encore aujourd’hui. Voir [Sources de données](#ret-data).
- **S&P 500:** mensuel et donc le plus précis pour le bootstrap et le mode glissant; ne contient toutefois que des actions américaines et le rendement élevé de la bourse américaine.
- **Suisse (JST):** univers de placement d’un investisseur suisse avec son marché domestique.
- **Monde personnalisé:** si vous voulez reproduire votre propre répartition, par exemple 30 % Suisse, 40 % USA, 20 % UE, 10 % Asie-Pacifique.

**Anciennes planifications:** une planification avec une série temporelle choisie reçoit à l’ouverture la stratégie correspondante et calcule comme avant: ‹Shiller CHF› → S&P 500 avec USA 10 ans, ‹Shiller CHF + CH-Obligationen› → S&P 500 avec Confédération 10 ans. ‹Shiller USD› devient S&P 500 avec USA 10 ans en CHF; les **séries temporelles propres** (fichiers CSV) ne sont plus prises en charge et sont remplacées par «Monde (JST)» avec des obligations de la Confédération. **Pictet:** l’indice «Suisse (Pictet)» n’est plus fourni pour des raisons de licence. Une planification avec ‹Pictet CH› ou avec «Suisse (Pictet)» reçoit pour les actions «Suisse (JST)» et pour les obligations «Confédération 10 ans». Dans tous ces cas, l’application le signale à l’ouverture, et le résultat change.

**Mettre à jour:** «Mettre à jour les données Shiller…» importe un nouveau fichier Shiller (.xls, .xlsx ou .ods; téléchargement du fichier `ie_data.xls` sur shillerdata.com), «Mettre à jour les données BNS…» de nouveaux cubes de data.snb.ch. L’application ne télécharge jamais elle-même de données d’Internet, vous les téléchargez à la main. Les données JST se terminent en 2020 et sont intégrées de manière fixe.

Pour en savoir plus: [Sources de données et leurs particularités](#ret-data).

## @f-ret-extrapolation Extrapolation de la série
Mots-clés: fin de la série, moyenne, fixe, extrapoler

Si la série temporelle se termine avant la fin de la simulation (par exemple «Historique dès 2000» avec une planification jusqu’en 2058), l’application doit combler les années manquantes. Vous définissez ici la **méthode**:

- **Valeurs fixes:** les rendements fixes de la section «Rendements fixes» s’appliquent.
- **Moyenne mobile:** chaque valeur manquante est la moyenne (arithmétique) des *n* dernières années. La fenêtre glisse et contient avec le temps aussi les valeurs déjà extrapolées. L’extrapolation tend donc vers une moyenne stable.
- **Moyenne géométrique mobile:** comme ci-dessus, mais avec une moyenne géométrique.

La période («Moyenne sur» … années) vaut pour les deux méthodes mobiles. Cette saisie n’apparaît que si la série se termine effectivement avant la fin de la planification.

Attention: une extrapolation est une **hypothèse**, pas un résultat historique. Avec des extrapolations très longues, le résultat perd son caractère historique.

## @ret-saron SARON et hypothèques
Mots-clés: SARON, hypothèque, intérêt, marge, série temporelle

Le **SARON** n’est nécessaire que si une hypothèque a une période SARON. Il provient de la série temporelle (dès 06.1999, si disponible), sinon c’est la **valeur fixe du SARON** de la planification qui s’applique.

- En dehors de la période SARON de la série, avec «Rendements fixes», avec le modèle paramétrique et après une extrapolation, la valeur fixe s’applique. La vue n’affiche le champ que s’il est nécessaire.
- Le taux d’intérêt d’une tranche SARON est le SARON plus la marge, avec un plancher de 0.
- Avant 1999, il n’y a pas de SARON. Pour les simulations historiques avant 1999, utilisez la valeur fixe comme approximation.

Pour en savoir plus sur les tranches et les périodes: [Hypothèques](#f-re-mortgage).

## @ret-seed Valeur initiale aléatoire et reproductibilité
Mots-clés: seed, hasard, mêmes résultats, reproductible

Monte Carlo est aléatoire, mais **reproductible**. La trajectoire *i* est toujours générée à partir de la paire (valeur initiale aléatoire, *i*).

- Les mêmes réglages et la même valeur initiale donnent **exactement les mêmes résultats**.
- Tous les **scénarios** utilisent les mêmes trajectoires. Dans l’**optimisation** aussi, tous les candidats voient les mêmes trajectoires (la vérification finale en utilise d’autres). Ainsi, les différences proviennent des paramètres, et non du hasard.
- Les résultats sont les mêmes sous macOS, Windows et Linux.
- Avec une nouvelle version, la méthode de génération des trajectoires aléatoires peut changer. L’application le signale à l’ouverture d’une ancienne planification. Ainsi, «Monte Carlo · paramétrique» génère les trajectoires plus rapidement depuis #132: la distribution des rendements reste la même, mais les trajectoires individuelles sont différentes, et le résultat change légèrement.
- Modifiez la valeur initiale si vous voulez vérifier si votre résultat dépend du hasard. Si le taux de réussite varie de plus de quelques dixièmes de point de pourcentage avec une autre valeur initiale, vous avez besoin de plus de trajectoires.

## @ret-data Sources de données et leurs particularités
Mots-clés: Shiller, BNS, Pictet, moyenne mensuelle, obligations, monnaie

- **Shiller (Robert J. Shiller, «Irrational Exuberance»):** actions sous forme de S&P Composite avec dividendes. Les cours de Shiller sont des **moyennes mensuelles**; les baisses apparaissent donc lissées. Rendement mensuel = variation de l’indice réel de rendement global × IPC (américain) par rapport au mois précédent. Les obligations sont le rendement global des emprunts d’État américains à 10 ans.
- **Conversion en CHF:** avec la moyenne mensuelle du taux de change USD/CHF de la BNS, sans couverture de change, et avec le renchérissement suisse (indice suisse des prix à la consommation par rapport au même mois de l’année précédente). Il en résulte qu’un franc qui s’apprécie réduit le rendement d’un investisseur suisse en actions américaines.
- **Obligations suisses:** le rendement global d’une obligation de la Confédération à 10 ans (au pair) est calculé à partir du rendement mensuel de la BNS: coupon = rendement du mois précédent, évaluation au nouveau rendement avec une durée résiduelle d’un peu moins de 10 ans, plus un coupon mensuel. La BNS ne met plus à jour cette série depuis 07.2025; c’est pourquoi ‹Shiller CHF + CH-Obligationen› se termine à cette date.
- **JST (Jordà-Schularick-Taylor Macrohistory Database, Release 6):** rendements annuels des actions et des emprunts d’État par pays jusqu’en 2020, convertis en CHF aux taux de change de fin d’année. Pondération d’un pays: PIB réel de l’année précédente (PIB par habitant selon Maddison × population). Les années-pays manquantes sont omises (Allemagne 1945–1946, Japon 1946–1947); l’Allemagne 1923 est exclue, car le taux de change pendant l’hyperinflation ne correspond pas au rendement annuel. Les actions suisses s’appuient sur Pictet dès 1926. Renchérissement: prix suisses à la consommation (moyenne annuelle). **Licence CC BY-NC-SA 4.0:** uniquement à des fins non commerciales, avec indication de la source (Jordà, Schularick, Taylor 2017; Jordà, Knoll, Kuvshinov, Schularick, Taylor 2019). Comparaison avec Dimson, Marsh, Staunton (1900–2020): Monde (JST) 6.6 % réel par année en USD, DMS 5.3 %; les séries par pays elles-mêmes concordent (USA 6.7 % contre 6.6 %).
- **Obligations de la Confédération avant 1989:** tirées de JST (rendement global); 1915 manque (bourse fermée), le coupon y fait office de rendement.
- **L’inflation** est, pour toutes les séries, la variation de l’indice des prix par rapport à l’année précédente, de sorte que la valeur de décembre correspond au renchérissement annuel.
- **Pas de frais:** les frais de placement, les impôts sur les revenus et les droits de garde ne sont pas déduits.
- **Droits et garantie:** les séries fournies sont des rendements calculés à partir des sources mentionnées. Elles ne relèvent pas de la licence du programme (GPL v3); les droits sur les données de base appartiennent aux sources (Robert J. Shiller, shillerdata.com; Banque nationale suisse; Office fédéral de la statistique; Jordà, Schularick, Taylor, macrohistory.net, licence CC BY-NC-SA 4.0). Shiller précise que ni lui ni l’Université Yale ne garantissent les données et qu’elles ne constituent ni une prévision ni un conseil en placement. Cela vaut aussi pour cette application.

## @ret-pitfalls Pièges des rendements
Mots-clés: erreur, prudence, survivorship, marché domestique, optimisme excessif, risque de séquence, monnaie

- **Le passé n’est pas une prévision.** La bourse américaine a été au XXe siècle un marché extraordinairement fructueux. Les séries avec des actions américaines contiennent ce succès et peuvent être trop optimistes (‹survivorship›). Cela vaut aussi pour «Monde (JST)», qui ne contient que des pays qui existent encore aujourd’hui. Vérifiez le plan aussi avec des rendements plus bas (paramétrique avec μ = 4 % à 5 %) ou avec des actions suisses.
- **Les périodes qui se chevauchent donnent une fausse impression de sécurité.** Voir [Toutes les dates de départ historiques](#f-ret-rolling): peu de périodes indépendantes, mais des centaines de simulations.
- **Une longueur des blocs trop courte dans le bootstrap** rend les trajectoires trop lisses. Restez à 12 mois ou plus.
- **Risque de séquence:** ce qui est décisif, c’est **quand** surviennent les mauvaises années. Une baisse dans les premières années de la retraite pèse beaucoup plus que plus tard. Les rendements fixes ne le montrent pas.
- **Monnaie:** tous les indices sont en CHF; une planification en EUR ou en USD calcule malgré tout avec des rendements en francs et le renchérissement suisse (remarque dans le contrôle). Les séries en CHF avec des actions étrangères portent aussi un risque de change. L’appréciation du franc au cours des dernières décennies a réduit le rendement d’un investisseur suisse en actions américaines.
- **Confondre nominal et réel:** ‹2 millions à la fin› sont nominaux. En pouvoir d’achat actuel, c’est nettement moins.
- **Une seule source.** Qui ne calcule qu’avec une seule source de rendements obtient une réponse **dépendante du modèle**. Calculez les questions importantes avec le bootstrap et le modèle paramétrique.
- **Les frais ne sont pas pris en compte.** Les frais de placement réels (fonds, dépôt) réduisent le rendement de quelques dixièmes à un pour cent par année. Déduisez-les des valeurs fixes ou paramétriques.
