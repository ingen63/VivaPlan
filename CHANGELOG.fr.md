# Journal des modifications

Traduction française de `CHANGELOG.md` (allemand, version de référence) à partir de la version 1.0.0-beta.1.

## Version 1.0.0-beta.4

Quatrième préversion de 1.0 ; elle contient aussi toutes les modifications de la 1.0.0-beta.3 non publiée. Le paquet d'installation Windows `.msi` porte le numéro de version 1.0.0.4 ; pour l'instant, les paquets d'installation ne sont construits que pour macOS (#202).

### Peut modifier les résultats

- **Marié·e avec une seule personne (#250) :** «Marié·e·s» s'applique aussi sans deuxième personne ; les impôts sont alors calculés avec les barèmes et déductions pour personnes mariées. Les anciennes planifications avec une personne sont reprises comme non mariées à l'ouverture et calculent comme avant (remarque) ; les nouvelles planifications commencent non mariées.
- **Impôts selon le canton et la commune (#63, #64, #65, #66) :** En mode « Domicile », l'impôt sur le revenu, sur la fortune et sur les prestations en capital suit le canton et la commune, l'impôt sur les gains immobiliers le canton de l'immeuble. Les barèmes suivent le renchérissement, la déduction pour double revenu s'applique aux personnes actives.
- **Déductions et domicile (#131, #151, #152, #228–#232) :** Déductions du revenu selon la proposition du canton ou en lignes propres, déductions supplémentaires et uniques, déductions de la fortune, valeur fiscale par immeuble, revenu d'activité en montant annuel, domicile variable dans le temps et grand pilier 3a pour les personnes sans caisse de pension.
- **AVS (#190, #191, #198) :** La 13e rente AVS est versée en décembre et compte dans le besoin net de la stratégie des 3 pots. Le versement anticipé et l'ajournement réduisent ou augmentent la rente ; le plafond pour les couples mariés est conforme à la loi.
- **Immeubles et stratégies (#199, #204, #239) :** Les rénovations non supportables avec « Ne jamais vendre les immeubles » sont comptées ; les limites des frais de subsistance s'appliquent à VPW et Endowment aussi dans la comparaison et l'optimisation ; les garde-fous sans fortune liquide n'ajustent rien.
- **Revenu d'activité (#236) :** Sur demande, le revenu d'activité prend fin avec le retrait de la caisse de pension.

### Nouveautés, modifications, corrections

- **Aide à la saisie tirée de la taxation (#251) :** Nouveau champ «dont pilier 3a selon la taxation», afin que le pilier 3a ne soit pas déduit deux fois avec «Déduire les versements au pilier 3a» ; une remarque s'affiche tant que le champ est vide.
- **Comparer les stratégies (#196, #197, #200, #201, #205) :** Les stratégies se comparent sans optimisation, les résultats de l'optimisation s'enregistrent et se rouvrent ; option « Ne jamais vendre les immeubles » ; indication lorsque la limite inférieure détermine les dépenses ; tableau des résultats sans défilement horizontal.
- **Thème sombre (#220, #235) :** Réglage « Apparence » avec thème clair et sombre ; meilleur contraste dans le menu.
- **Utilisation (#143, #227, #233, #234, #241) :** Impôts signalés comme estimation, planifications récentes dans le menu « Fichier », supportabilité dans les réglages, messages du moteur de calcul lisibles, carte « Déductions du revenu » plus claire avec un texte pour la déduction pour double revenu.
- **Vidéos d'introduction (#240, #242) :** « VivaPlan en 2 minutes » et « en 10 minutes » en quatre langues, depuis le menu « Aide », la fenêtre de bienvenue et l'aide ; elles ouvrent le navigateur.
- **Soutenir (#237) :** Lien de don facultatif « Buy me a coffee » dans le menu « Aide », dans la navigation et sous « À propos de VivaPlan » ; il ouvre le navigateur.
- **Ligne de commande pour les IA (#207, #215) :** Sortie JSON et modification de valeurs par chemin de champ ; les trajectoires de rendement sont réutilisées pour plusieurs variantes (plus rapide).

### Tests et développement

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
