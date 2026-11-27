# Journal des modifications

Traduction française de `CHANGELOG.md` (allemand, version de référence) à partir de la version 1.0.0-beta.1.

## Version 1.0.0-beta.3

Troisième préversion de 1.0. Elle regroupe les préversions non publiées 1.0.0-beta.4 à 1.0.0-beta.6 et contient toutes les modifications depuis. Le paquet d'installation Windows `.msi` porte le numéro de version 1.0.0.3; les paquets d'installation sont disponibles pour macOS, Windows et Linux. À partir de cette version, l'application peut chercher elle-même les nouvelles versions si vous l'acceptez.

### Peut modifier les résultats

- **Renchérissement (#306, #279):** Les montants qui augmentent avec le renchérissement dans la réalité augmentent toujours avec le renchérissement du parcours; exceptions pour les tests de résistance sous «Expert». Le renchérissement du modèle paramétrique commence en 1950, l'année de départ est réglable. Les planifications existantes calculent comme avant.
- **AVS (#284, #280):** La rente AVS suit l'indice mixte tous les deux ans, aussi avant la perception, avec la nouvelle hypothèse «hausse des salaires réels»; en termes nominaux, elle ne baisse jamais sous la valeur de l'année précédente.
- **Intérêts hypothécaires (#307):** Le SARON et les hypothèques fixes renouvelées suivent l'environnement des taux et du renchérissement du parcours; les marges se trouvent dans les réglages.
- **Données fiscales tirées des lois (#287–#292, #296, #297):** Barèmes, déductions et règles de la Confédération et des cantons proviennent des lois et ordonnances au lieu du calculateur d'impôts de l'AFC; le Valais calcule avec l'indexation et la réduction pour les personnes mariées.
- **Nouvelles planifications (#303):** Elles commencent avec les actions «Monde (JST)» et les obligations à court terme.
- **Part du logement dans le coût de la vie (#264):** La part du logement déduite comprend le loyer, les intérêts et les charges, et non plus l'amortissement. Les planifications plus anciennes sont adaptées à l'ouverture (avec un avis); leur résultat reste le même.
- **Percentiles (#265):** Les résultats s'ouvrent en pouvoir d'achat actuel; les percentiles sont classés selon la présentation choisie (nominale ou réelle).
- **Rendement imposable de la fortune (#149):** Un taux annuel pour les actions et les obligations compte dans le revenu imposable. Les nouvelles planifications commencent avec 2 % et 1 %, les existantes avec 0 %.
- **Sans caisse de pension (#253):** L'interrupteur se trouve désormais dans la carte «Caisse de pension» de la section Prévoyance. S'il est activé, la caisse de pension de cette personne compte pour 0 (ni avoir, ni cotisations, ni rente, ni retrait en capital); les valeurs saisies restent enregistrées, les champs sont verrouillés.
- **Optimisation (#248):** Pour toutes les formes d'évolution, l'évolution avant la retraite est conservée; la valeur optimisée s'applique dès la retraite.
- **Marié·e avec une seule personne (#250) :** «Marié·e·s» s'applique aussi sans deuxième personne ; les impôts sont alors calculés avec les barèmes et déductions pour personnes mariées. Les anciennes planifications avec une personne sont reprises comme non mariées à l'ouverture et calculent comme avant (remarque) ; les nouvelles planifications commencent non mariées.
- **Impôts selon le canton et la commune (#63, #64, #65, #66) :** En mode « Domicile », l'impôt sur le revenu, sur la fortune et sur les prestations en capital suit le canton et la commune, l'impôt sur les gains immobiliers le canton de l'immeuble. Les barèmes suivent le renchérissement, la déduction pour double revenu s'applique aux personnes actives.
- **Déductions et domicile (#131, #151, #152, #228–#232) :** Déductions du revenu selon la proposition du canton ou en lignes propres, déductions supplémentaires et uniques, déductions de la fortune, valeur fiscale par immeuble, revenu d'activité en montant annuel, domicile variable dans le temps et grand pilier 3a pour les personnes sans caisse de pension.
- **AVS (#190, #191, #198) :** La 13e rente AVS est versée en décembre et compte dans le besoin net de la stratégie des 3 pots. Le versement anticipé et l'ajournement réduisent ou augmentent la rente ; le plafond pour les couples mariés est conforme à la loi.
- **Immeubles et stratégies (#199, #204, #239) :** Les rénovations non supportables avec « Ne jamais vendre les immeubles » sont comptées ; les limites des frais de subsistance s'appliquent à VPW et Endowment aussi dans la comparaison et l'optimisation ; les garde-fous sans fortune liquide n'ajustent rien.
- **Revenu d'activité (#236) :** Sur demande, le revenu d'activité prend fin avec le retrait de la caisse de pension.

### Nouveautés, modifications, corrections

- **Pilier 3a (#300, #301, #310):** Retrait en tranches de 60 à 65 ans et versement anticipé pour la propriété du logement à tout âge.
- **Mises à jour automatiques (#309):** Au premier démarrage, l'application demande si elle peut chercher de nouvelles versions; sans accord, elle reste entièrement sans réseau. Les mises à jour sont signées.
- **Réglages en onglets (#308):** Général, Prévoyance, Simulation et Immobilier.
- **Impôts (#143, #281):** L'année fiscale, la source et l'origine des déductions sont visibles partout, la vérification avertit en cas de données fiscales anciennes; le revenu imposable figure dans une colonne à côté des impôts.
- **Connexion IA (#302, #304, #298):** Une nouvelle planification peut être enregistrée avec un nom de fichier ou `~/`; l'IA connaît les libellés de l'application pour tous les champs de choix; remarque sur les chats incognito et la protection des données.
- **Rente AVS dans l'aide (#282):** expliquée comme rente attendue à 65 ans.
- **Jeton bêta (#270):** L'app et le paquet IA s'activent une fois avec un jeton bêta.
- **EMS (#271):** Élément propre avec âge d'entrée, coûts et vente du logement; avis lorsqu'il ne reste plus de frais de logement après la vente du dernier bien immobilier.
- **Impôts (#272–#275):** Les nouvelles planifications calculent les impôts au domicile; la section montre d'abord la commune et la confession, les autres indications comme hypothèses, les détails sous «Expert». Les déductions peuvent être reprises de la dernière taxation, la valeur fiscale des immeubles a une valeur par défaut en pour cent de la valeur vénale, un rachat dans la caisse de pension se saisit directement comme déduction.
- **Déductions de la fortune (#249)** présentées comme les déductions du revenu.
- **Capacité financière (#266):** Au choix avec consommation de la fortune et avance plus basse dès la retraite.
- **Connexion IA (#263, #269):** Le serveur MCP n'ouvre et n'enregistre les planifications que dans le dossier autorisé, au choix en lecture seule; une nouvelle planification commence vide et l'IA demande les indications manquantes.
- **README (#267, #277)** en quatre langues: l'app est 100 % locale, seule la connexion IA envoie des données à l'IA choisie.
- **Affichage (#220):** Choix «Apparence» dans le dialogue des réglages avec le fond du thème.
- **Utiliser VivaPlan avec une IA (#208–#214, #216, #219):** Le serveur MCP `vivaplan mcp` ouvre, modifie, calcule et compare des planifications pour Claude, ChatGPT, LM Studio et d'autres IA, avec progression, interruption et confirmation des longs calculs. Le menu «Aide» › «Connecter à une IA» montre la configuration; pour Claude Desktop, il existe un paquet en un clic `.mcpb` (#213). Les données ne vont qu'à l'IA que vous connectez vous-même.
- **Formes d'évolution et courbes caractéristiques (#217, #218, #243):** Les valeurs peuvent évoluer de façon linéaire, selon un tableau ou avec un facteur et se modifier dans l'éditeur; les courbes caractéristiques font dépendre des paramètres de l'état.
- **Ligne de commande dans le paquet d'installation (#206):** `vivaplan` utilise le même dossier de données que l'application.
- **Affichage (#244, #255, #257, #259):** Les champs numériques et le choix «Valeur fiscale» ne sont plus tronqués, dans aucune langue.
- **Fortune imposable dans la carte «Déductions de la fortune» (#249) :** Remarque avec la fortune imposable estimée de la première année fiscale et la part des immeubles, avertissement si une valeur fiscale manque ; les détails la composent par année fiscale comme la déclaration d'impôt et proposent le champ «Valeur fiscale» par immeuble.
- **Aide à la saisie tirée de la taxation (#251) :** Nouveau champ «dont pilier 3a selon la taxation», afin que le pilier 3a ne soit pas déduit deux fois avec «Déduire les versements au pilier 3a» ; une remarque s'affiche tant que le champ est vide.
- **Comparer les stratégies (#196, #197, #200, #201, #205) :** Les stratégies se comparent sans optimisation, les résultats de l'optimisation s'enregistrent et se rouvrent ; option « Ne jamais vendre les immeubles » ; indication lorsque la limite inférieure détermine les dépenses ; tableau des résultats sans défilement horizontal.
- **Thème sombre (#220, #235) :** Réglage « Apparence » avec thème clair et sombre ; meilleur contraste dans le menu.
- **Utilisation (#143, #227, #233, #234, #241) :** Impôts signalés comme estimation, planifications récentes dans le menu « Fichier », supportabilité dans les réglages, messages du moteur de calcul lisibles, carte « Déductions du revenu » plus claire avec un texte pour la déduction pour double revenu.
- **Vidéos d'introduction (#240, #242) :** « VivaPlan en 2 minutes » et « en 10 minutes » en quatre langues, depuis le menu « Aide », la fenêtre de bienvenue et l'aide ; elles ouvrent le navigateur.
- **Soutenir (#237) :** Lien de don facultatif « Buy me a coffee » dans le menu « Aide », dans la navigation et sous « À propos de VivaPlan » ; il ouvre le navigateur.
- **Ligne de commande pour les IA (#207, #215) :** Sortie JSON et modification de valeurs par chemin de champ ; les trajectoires de rendement sont réutilisées pour plusieurs variantes (plus rapide).

### Tests et développement

- Tests plus rapides et plus stables (#226, #305, #311), notes de version et tests sous Windows (#278), références obsolètes à l'AFC mises à jour (#286).
- Le paquet Linux contient à nouveau `vivaplan` (#261), mémoire des grands calculs réduite (#249), règle Clippy (#260), mesure de l'influence des impôts pour d'autres ménages (#276).
- Tests de l'interface mis à jour (#246, #247), limite de temps du test MCP indépendante de la vitesse de l'ordinateur (#258).
- Chiffres de référence comparés lors du push (#195), évolution de la valeur des immeubles recalculée de manière indépendante (#194), traçabilité des exigences des tests étendue (#192).
- Test de robustesse de toutes les commandes de calcul (#221), trouvailles des tests de propriétés en tests de régression (#223), tests Rust sous Windows (#225), tests ignorés comptés comme erreurs (#224), limite de durée par test (#226), règles pour l'agent de test (#222).
- Notes de version en quatre langues avec un titre neutre (#188) ; mutants-diff hebdomadaire (#202).

## Version 1.0.0-beta.2

Deuxième préversion de 1.0. Le programme d'installation Windows `.msi` porte le numéro de version 1.0.0.2.

### Peut modifier les résultats

- **Impôt sur les prestations en capital des couples non mariés (#150) :** Les retraits en capital de partenaires non mariés sont imposés par personne et ne sont plus additionnés pour le ménage. Cela peut modifier les résultats de ces planifications.

### Nouveautés, modifications, corrections

- **Quatre langues (#170–#177) :** Interface, menus, dialogues de fichiers, messages et aide existent en allemand, anglais, français et italien. Au premier démarrage, la langue du système s'applique, ensuite le choix sous «Affichage → Langue». Les nombres et les dates gardent le format suisse dans chaque langue. Les noms du catalogue de l'optimisation et des séries temporelles sont également traduits.
- **Termes et contrôle des traductions (#179–#182) :** Un glossaire fixe les termes officiels par langue (p. ex. AVS/LPP, caisse de pension, cassa pensione) ; un contrôle automatique compare toutes les traductions avec l'allemand. Tous les textes trouvent leur place dans chaque langue, aussi avec les polices sous Linux et Windows.
- **Ligne de commande en anglais (#176) :** `vivaplan` affiche les messages et les rapports en anglais. Les chiffres sont inchangés.
- **Nouvelle icône (#185) :** «Sentier du sommet – coucher de soleil», avec une version propre pour les petites tailles et un favicon assorti.
- **Planification d'exemple (#184) :** La planification d'exemple s'ouvre dans la langue de l'interface.
- **Aide (#183) :** Contradictions sur le coût de la vie, les combinaisons et le versement 3a supprimées.

### Tests et développement

- Code interne en anglais (#165–#169), absence de réseau contrôlée automatiquement (#157), cas de test communs du calcul des impôts pour l'interface et le moteur de calcul (#155, #156, #158).
- Fichiers d'exemple nommés en anglais et avec des fins de ligne fixes (#186), tests pour des mutants non détectés jusqu'ici (#187).
- Notes de version en quatre langues avec contrôle automatique (#188).

## Version 1.0.0-beta.1

Première préversion de 1.0. Le programme d'installation Windows `.msi` porte le numéro de version 1.0.0.1, car le format MSI ne connaît pas de préversion comme «beta.1» ; les autres paquets s'appellent 1.0.0-beta.1.

### Peut modifier les résultats

- **Critère de rendement de la stratégie à 3 compartiments (#103) :** Chaque trajectoire de rendements reprend les rendements des actions des 12 mois précédant le début de la simulation. Le critère de rendement porte ainsi sur 12 mois complets dès les 12 premiers mois. Cela peut modifier les résultats des planifications avec la stratégie à 3 compartiments la première année.

### Nouveautés, modifications, corrections

- **Comparaison des résultats (#115) :** La comparaison de résultats enregistrés offre les mêmes possibilités qu'un résultat seul : nominal ou en pouvoir d'achat actuel, tous les graphiques et les valeurs annuelles, avec le choix de la simulation (percentile).
- **Ligne de référence dans les graphiques (#114) :** La ligne à 0 (ou à 100 pour l'évolution de la valeur) est nettement mise en évidence dans tous les graphiques de résultats.
- **Menu (#112) :** Un clic dans le menu de gauche ou dans la liste de contrôle n'est plus perdu lorsqu'un aperçu arrive au même moment.

### Tests et développement

- Tests de l'interface indépendants de la charge de l'ordinateur (#111), page de synthèse de la qualité des tests avec les critères vérifiés (#113), limite de mémoire pour les processus de test lors des tests de mutation (#116).
