# Introduction

## @start Ce que VivaPlan sait faire
Mots-clés: vue d’ensemble, but, planification de la retraite, simulation

VivaPlan répond à la question qui se pose avant la retraite: **l’argent suffira-t-il, et avec quelle certitude?**

> Vidéo: [VivaPlan en 2 minutes](video:2min) montre la vue d’ensemble (ouvre le navigateur).

L’application calcule votre avenir financier mois par mois, d’aujourd’hui jusqu’à la fin de la planification. Elle connaît les particularités suisses:

- caisse de pension (2e pilier) avec taux de conversion et retrait en capital, AVS (1er pilier) et pilier 3a
- impôt sur le revenu, impôt sur la fortune, impôt sur les prestations en capital et impôt sur les gains immobiliers
- logement en propriété avec hypothèque SARON ou à taux fixe, calcul de la capacité financière selon les règles des banques, location
- une deuxième personne avec sa propre prévoyance (règles pour couples mariés)

L’avenir est incertain, surtout les rendements boursiers. C’est pourquoi l’application ne calcule pas une seule évolution, mais, selon le réglage, **des milliers** (Monte Carlo ou toutes les dates de départ historiques), et montre dans combien d’entre elles l’argent suffit. La manière dont les rendements sont produits est le thème le plus important de l’application: voir [Rendements et données](#s-returns).

La structure:

1. **Éditeur:** vous saisissez la planification dans onze sections (à gauche). À droite, vous voyez immédiatement un aperçu de l’évolution de la fortune.
2. **Lancer la simulation:** l’application calcule toutes les simulations et affiche les [résultats](#r-overview).
3. **Optimisation:** l’application recherche la [stratégie de prélèvement](#s-optimize) qui correspond à vos objectifs.
4. **Résultats:** chaque simulation est classée comme résultat dans la [liste](#r-list), afin que vous puissiez la consulter et la comparer plus tard.

## @quickstart Première planification en dix minutes
Mots-clés: guide, début, tutoriel, étapes

> Vidéo: [VivaPlan en 10 minutes](video:10min) guide à travers la première planification (ouvre le navigateur).

Commencez avec la planification d’exemple chargée à l’ouverture et adaptez-la pas à pas à votre situation.

La liste de contrôle **«Premiers pas»**, à droite au-dessus du contrôle, vous guide à travers les mêmes huit étapes. Un clic sur une étape ouvre la section, ⓘ l’aide correspondante. Une section est considérée comme effectuée dès que vous y modifiez une valeur ou que vous la cochez à la main; «Annuler» n’enlève pas la coche. «Lancer la simulation» est effectué dès qu’une simulation s’est terminée sans interruption. Avec ✕, vous masquez la liste; vous la réaffichez par *Aide → Premiers pas*. Lorsque toutes les étapes sont effectuées, elle se replie. La progression vaut pour vous sur cet appareil, pas pour une planification en particulier, et ne figure pas dans le fichier de planification.

1. **[Ménage et projet](#s-general):** saisir le nom et le mois de naissance.
2. **[Jalons](#s-milestones):** fixer la prestation de la caisse de pension, le début de la rente AVS ainsi que le début et la fin de la simulation. La fin devrait être assez tardive pour que vous ne comptiez pas vivre au-delà (par exemple 95 ans).
3. **[Prévoyance](#s-pension):** saisir l’avoir actuel de la caisse de pension, le taux de conversion, le retrait en capital et la rente AVS d’après vos certificats de prévoyance.
4. **[Revenus et dépenses](#s-flows):** revenu de l’activité lucrative par an (comme dans la déclaration d’impôt) et coût de la vie par mois. Le coût de la vie doit avoir une valeur réaliste, c’est là que se trouve le plus grand levier.
5. **[Fortune et placement](#s-wealth):** actions, obligations et cash au début de la simulation, ainsi que la part d’actions visée.
6. **[Simulation](#s-returns):** pour une première estimation, choisir «Monte Carlo · historique». Vous définissez les indices (par exemple «Monde (JST)») dans la [stratégie de placement](#f-ret-series) sous «Fortune et placement».
7. **[Impôts](#s-taxes):** remplacer les taux de la planification d’exemple par ceux de votre domicile (calculateur d’impôts de votre canton).
8. **Lancer la simulation** (bouton en haut à droite ou ⌘↩). À gauche du bouton figure une estimation approximative de la **durée**. Pendant le calcul, une **barre de progression** indique l’avancement et le temps restant estimé; avec **Interrompre**, vous arrêtez le calcul et revenez à la planification.

Le contrôle en bas à droite affiche les erreurs et les avertissements avant même que vous ne calculiez. Les erreurs empêchent la simulation, les avertissements non.

> Conseil: appuyez sur **F1** dans un champ de saisie, et l’aide relative à ce champ apparaît.

## @bedienung Utilisation et raccourcis clavier
Mots-clés: menu, clavier, raccourcis, enregistrer, ouvrir, annuler, F1, langue, ouverts récemment, soutenir, don, café, vidéo, vidéo d’introduction

- **Fichier:** Nouveau (une **planification vide avec les valeurs standard**: départ le mois en cours, sans nom de projet, une personne sans nom, aucun montant ni bien immobilier; les rendements, barèmes fiscaux et paramètres des stratégies reprennent les valeurs standard), Ouvrir, Ouverts récemment, Enregistrer, Enregistrer sous. Une planification est un fichier unique avec l’extension `.vplan`. Il peut être copié et archivé.
- **Ouverts récemment:** Le sous-menu sous Fichier → Ouvrir montre jusqu’à 10 planifications que vous avez ouvertes ou enregistrées en dernier, la plus récente en haut (nom du fichier sans `.vplan`; avec le dossier si les noms sont identiques). Un clic ouvre la planification comme «Ouvrir…»; s’il y a des modifications non enregistrées, l’app demande d’abord. Si le fichier n’existe plus, l’app le signale et retire l’entrée. «Effacer la liste» vide la liste; si elle est vide, elle affiche «Aucune entrée».
- **Démarrage et fermeture:** au démarrage, l’application de bureau ouvre la planification que vous avez ouverte ou enregistrée en dernier; le nom du fichier figure dans l’en-tête. S’il n’y en a pas, si le fichier a disparu ou s’il est illisible, elle ouvre la **planification d’exemple** fournie (avec une remarque sur le nom du fichier, sauf lors du tout premier démarrage). La planification d’exemple est considérée comme une nouvelle planification non enregistrée: «Enregistrer» demande un emplacement, elle n’est jamais écrasée. Lors de la fermeture avec des modifications non enregistrées (bouton de fermeture de la fenêtre, ⌘Q ou Ctrl+Q, ou Fichier → Quitter), l’application demande si vous voulez enregistrer, ignorer ou interrompre. «Interrompre» laisse l’application ouverte; si vous interrompez le choix de l’emplacement lors de l’enregistrement, elle reste également ouverte. Seul le chemin du fichier est mémorisé, localement dans le dossier de données de l’application.
- **Enregistrer avant la simulation:** si la planification contient des modifications non enregistrées, l’application de bureau demande avant chaque simulation si vous voulez enregistrer: «Enregistrer» (pour une nouvelle planification, avec choix de l’emplacement), «Simuler sans enregistrer» ou «Interrompre». Cette question peut être désactivée dans les paramètres.
- **Bienvenue:** lors du tout premier démarrage, l’application demande comment vous souhaitez commencer: **Découvrir l’exemple** (charge la planification d’exemple et ouvre l’aide [Ce que VivaPlan sait faire](#start)), **Commencer sa propre planification** (charge la planification d’exemple comme modèle, ouvre «Ménage et projet» et réinitialise la liste de contrôle [Premiers pas](#quickstart)) ou **Planification vide** (comme Fichier → Nouveau). Échap ferme la boîte de dialogue sans modification. Ensuite, elle n’apparaît plus d’elle-même; *Aide → Bienvenue …* l’ouvre en tout temps, avec une question en cas de modifications non enregistrées. Si l’application est lancée avec un fichier (double-clic), elle n’apparaît pas.
- **Annuler / Rétablir:** ⌘Z et ⇧⌘Z (sous Windows et Linux Ctrl+Z et Ctrl+Shift+Z) annulent les saisies.
- **Changer de section:** ⌘1 à ⌘9 et ⌘0 sautent aux sections, ⌘↑ et ⌘↓ avancent ou reculent d’une section. Dans les résultats, les mêmes touches changent d’onglet.
- **Aide:** **F1** ouvre l’aide sur le thème actuel (selon l’endroit où vous vous trouvez: champ, section, graphique des résultats). Le bouton «? Aide» et les signes ⓘ à côté des champs font de même. ⌘⇧? ouvre la table des matières, Échap ferme l’aide.
- **Ligne du temps:** au-dessus de l’éditeur, elle montre tous les jalons. Un clic dessus ouvre les jalons. Sous la ligne figurent les **événements immobiliers** de l’aperçu (triangles verts pour l’achat et la vente, orange pour les ventes forcées) avec une brève légende comme ‹Achat Wohnung 09.2027›. Au-delà de 15 événements, seuls les symboles apparaissent; le texte figure alors dans l’infobulle.
- **Valeurs dépendant du temps:** de nombreux montants peuvent être saisis comme constante ou comme suite de valeurs, voir [Valeurs dépendant du temps](#f-timed).
- **Langue:** menu *Affichage → Langue* ou les [paramètres](#f-settings). Le changement agit immédiatement; la planification et la vue sont conservées. L’aide apparaît dans la langue choisie; si un chapitre n’est pas encore traduit, il apparaît en allemand.
- **Vidéos d’introduction:** *Aide → Vidéos d’introduction …* affiche les vidéos «VivaPlan en 2 minutes» et «VivaPlan en 10 minutes»; la boîte de dialogue de bienvenue contient le lien «Regarder VivaPlan en 2 minutes». Un clic ouvre la vidéo dans la langue choisie dans votre navigateur (Internet Archive).
- **Connecter à une IA:** *Aide → Connecter à une IA …* montre comment configurer VivaPlan dans Claude Desktop, Claude Code, l’application de bureau ChatGPT ou LM Studio, chaque fois avec l’emplacement de la configuration et une ligne à copier ([connecter des programmes d’IA](#ref-mcp)). VivaPlan ne configure rien lui-même.
- **Soutenir VivaPlan:** VivaPlan est gratuit. Si vous souhaitez soutenir librement l’auteur en lui offrant un café: *Aide → Soutenir VivaPlan…*, la ligne «☕ Soutenir VivaPlan» en bas de la navigation ou le lien dans la fenêtre «À propos de VivaPlan» ouvrent la page buymeacoffee.com/vivaplan dans votre navigateur. L’application elle-même ne charge rien depuis Internet.

La liste complète des raccourcis clavier figure dans le menu *Aide → Raccourcis clavier* et dans le [chapitre de référence](#ref-shortcuts).

## @beta Jeton bêta et activation
Mots-clés: bêta, jeton, activation, activer, délai d’utilisation, bêta fermée, testeur, mise à jour

Pendant la bêta fermée, VivaPlan ne peut être utilisé qu’après une **activation unique**. Vous recevez pour cela un jeton bêta personnel, une ligne commençant par `VP1.beta.`. Au premier démarrage, l’application n’affiche que le dialogue « Saisir le jeton bêta » ; collez-y le jeton entier et cliquez sur *Activer*. Les retours à la ligne et les espaces provenant d’un e-mail ne gênent pas.

- **Une seule fois :** l’activation vaut pour cet ordinateur et votre compte d’utilisateur. Ensuite, VivaPlan ne le redemande plus jamais, même après une mise à jour vers une nouvelle version.
- **Délai d’utilisation :** la date du jeton indique seulement jusqu’à quand il peut être saisi. Une fois le délai passé, VivaPlan refuse le jeton ; demandez alors un nouveau jeton. Une installation déjà activée continue de fonctionner après le délai.
- **Sans Internet :** VivaPlan vérifie le jeton sur votre ordinateur. Il n’envoie rien.
- **Paquet pour l’IA :** l’application et la connexion à l’IA (`vivaplan mcp`) partagent l’activation. Si l’application est activée, le paquet pour Claude Desktop n’a pas besoin de jeton. Sinon, saisissez le jeton dans le champ « Beta token » lors de l’installation du paquet ; cela active aussi l’application ([Connecter des programmes d’IA](#ref-mcp)).
- **Affichage :** *Aide → À propos de VivaPlan* affiche vos initiales et le jour de l’activation.

L’activation est conservée dans le fichier `beta-aktivierung.json` du dossier de données de l’application (voir [Protection des données](#privacy)). À partir de la version 1.0.0, VivaPlan n’a plus besoin de jeton.

## @privacy Protection des données et données locales
Mots-clés: Internet, cloud, emplacement, sécurité, réseau, protection des données

Toutes les données restent sur votre ordinateur. L’application ne se connecte jamais à Internet, ni pour les données, ni pour les mises à jour, ni pour des analyses. Seuls deux types de liens ouvrent votre navigateur: les [vidéos d’introduction](#start) et le lien de don «Soutenir VivaPlan» ([utilisation](#bedienung)). Les adresses sont fixées dans l’application; l’application elle-même ne charge rien.

- Les **planifications** se trouvent là où vous enregistrez le fichier `.vplan`. Les chemins des planifications ouvertes ou enregistrées en dernier (liste «Ouverts récemment») figurent dans le fichier `einstellungen.json` du dossier de données de l’application.
- Les **séries temporelles** (la bibliothèque de données) se trouvent dans le dossier de données de l’application. Sous macOS, il s’agit de `~/Library/Application Support/ch.vivaplan-app.desktop/Zeitreihen`.
- **La liste des résultats** se trouve dans la mémoire locale de l’application (pas dans le fichier de planification). Quiconque dispose de votre compte d’utilisateur sur cet ordinateur peut la lire. Supprimez les entrées que vous ne souhaitez pas conserver.
- **Programmes d’IA:** si vous connectez VivaPlan à une IA ([connecter des programmes d’IA](#ref-mcp)), l’IA lit des données de votre planification par le programme `vivaplan mcp`. Avec une IA dans le cloud (par exemple Claude ou ChatGPT), ces données vont à son fournisseur. Avec un modèle local (par exemple dans LM Studio), elles restent sur votre ordinateur. VivaPlan lui-même n’envoie rien.

Vous téléchargez vous-même les nouvelles données de la bibliothèque (Shiller, BNS) sur leurs sites web et les importez au moyen des boutons de la [stratégie de placement](#f-ret-series).

## @limits Limites du modèle
Mots-clés: hypothèses, non pris en compte, conseil, responsabilité, décès

VivaPlan est un **outil de calcul**, pas un conseil en placement, fiscal ou en prévoyance. Un résultat ne vaut que pour les hypothèses saisies. Particulièrement important:

- **L’avenir n’est pas le passé.** Les rendements historiques ne sont pas une prévision. Voir [Pièges des rendements](#ret-pitfalls).
- **Les impôts sont simplifiés:** barèmes de taux, interpolés linéairement, multipliés par le coefficient d’impôt. Les déductions, le détail de la progression et les particularités cantonales manquent. Les résultats montrent donc un **impôt estimé**, voir [Quelle est la précision de l’impôt?](#tax-accuracy).
- **Ne sont pas calculés:** les décès et les rentes de survivants, les successions (sauf comme montant unique), les frais de soins (sauf si vous les saisissez comme dépenses), les modifications législatives, les frais de placement et les commissions, les intérêts sur le cash.
- **Les biens immobiliers** sont simplifiés: valeur vénale avec renchérissement, hypothèques en tranches, vente forcée selon une règle simple.
- **Les stratégies optimisées** ne valent que pour les saisies avec lesquelles elles ont été calculées, voir [Pièges de l’optimisation](#o-pitfalls).

Vérifiez toujours les décisions importantes avec des spécialistes. Vous trouverez la [clause de non-responsabilité](#disclaimer) complète dans la section suivante.

## @disclaimer Clause de non-responsabilité
Mots-clés: responsabilité, garantie, disclaimer, clause de non-responsabilité, conseil, droit, avertissement

Lors du premier démarrage, vous confirmez une version abrégée de cet avertissement. Si le texte change, l’application vous le redemande. Toutes les pages de résultats portent en bas la mention «Calcul modèle sans garantie. Pas de conseil fiscal ni en placement.».

**1. Offre gratuite.** VivaPlan est mis à disposition gratuitement depuis la Suisse. Il n’existe aucun droit à la disponibilité, aux mises à jour, à la correction d’erreurs ou à l’assistance.

**2. Pas de conseil.** L’application est un outil de calcul pour la planification personnelle. Elle n’est **pas un conseil fiscal ni en placement**. Elle n’offre aucun conseil en placement, financier, fiscal, en prévoyance ou juridique et ne donne aucune recommandation d’achat ou de vente d’instruments financiers ou de biens immobiliers.

**3. Calculs modèles sans garantie.** Tous les résultats reposent sur vos saisies ainsi que sur des hypothèses simplifiées concernant le rendement, les coûts, le renchérissement, les impôts et l’espérance de vie. Les évolutions effectives peuvent s’en écarter considérablement. Les calculs fiscaux sont des estimations et ne reproduisent pas toutes les règles, déductions et modifications législatives cantonales et communales. Aucune garantie n’est donnée quant à l’exactitude, l’exhaustivité et l’actualité des calculs.

**4. Responsabilité personnelle.** Les décisions que vous prenez sur la base de l’application relèvent de votre seule responsabilité. Avant des décisions financières importantes, par exemple concernant le retrait en capital, la vente d’un logement ou la stratégie de prélèvement, vous devriez faire appel à une personne qualifiée.

**5. Responsabilité.** Toute responsabilité pour des dommages de quelque nature que ce soit, notamment pour des pertes de fortune, un manque à gagner, une perte de données ou des dommages consécutifs résultant de l’utilisation ou de la non-utilisation de l’application, est exclue dans la mesure permise par la loi. Est réservée la responsabilité pour dol et négligence grave.

**6. Données.** L’application enregistre vos saisies localement sur votre appareil et ne transmet aucune donnée (voir [Protection des données et données locales](#privacy)). La sauvegarde de ces données relève de votre responsabilité.

**7. Droit applicable.** Le droit suisse est applicable.

État: 22 novembre 2026
