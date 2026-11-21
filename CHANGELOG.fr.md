# Journal des modifications

Traduction française de `CHANGELOG.md` (allemand, version de référence) à partir de la version 1.0.0-beta.1.

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
