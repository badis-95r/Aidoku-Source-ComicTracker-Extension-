# Comics Tracker - Extension Aidoku

Cette extension non-officielle pour l'application [Aidoku](https://aidoku.app) permet de lire les bandes dessinées et comics hébergés sur [Comics Tracker](https://comics-tracker.net) directement sur iOS/iPadOS.

##  Fonctionnalités

- **Catalogue & Page d'accueil :** Récupération automatique des dernières sorties et des séries populaires.
- **Recherche :** Moteur de recherche intégré pour trouver vos éditions, runs ou évènements.
- **Support complet :** Gère toutes les structures du site (Tomes simples, Intégrales, Runs, Évènements).
- **Authentification :** Connexion via vos identifiants Comics Tracker pour accéder aux chapitres.
- **Haute Résolution :** Contourne la compression pour afficher les planches avec une netteté maximale, idéal pour lire les doubles-pages sur iPad.

##  Installation

Si vous avez seulement besoin d'utiliser l'extension :

1. Récupérez le fichier `ComicsTracker.aix` (ou générez-le).
2. Transférez-le sur votre appareil iOS/iPadOS (via AirDrop, Fichiers, iCloud...).
3. Ouvrez l'application **Aidoku**.
4. Allez dans l'onglet **Parcourir** > **Sources** > Cliquez sur l'icône **+** en haut à droite > **Sideload source**.
5. Sélectionnez le fichier `ComicsTracker.aix`.
6. Une fois installé, allez dans les **Paramètres de l'extension** (icône d'engrenage) et rentrez vos identifiants (Email et Mot de passe) Comics Tracker pour pouvoir lire les chapitres.

##  Compilation (Build)

Si vous souhaitez modifier le code ou compiler vous-même l'extension à partir des sources.

### Prérequis

- [Rust](https://rustup.rs/) (avec `cargo`)
- La cible WebAssembly pour Rust. Installez-la via :
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- Windows (Powershell) pour le script de packaging.

### Construire l'extension

Ouvrez un terminal (PowerShell) à la racine du projet et exécutez le script :

```powershell
.\build.ps1
```

Ce script va :
1. Vérifier que la cible WASM est installée.
2. Compiler le code Rust en mode `release`.
3. Préparer le dossier `Payload` avec le `.wasm`, `source.json`, `settings.json` et `icon.png`.
4. Compresser le tout et générer le fichier **`ComicsTracker.aix`**.

## Architecture du code (Pour les développeurs)

Le projet est écrit en **Rust** et utilise le SDK Aidoku. Si vous souhaitez y apporter des modifications, voici comment le code est structuré dans le dossier `src/` :

- `lib.rs` : Point d'entrée de l'extension. Définit les méthodes requises par Aidoku (`get_manga_list`, `get_manga_details`, `get_chapter_list`, `get_page_list`).
- `parser.rs` : Cœur de l'extension. Gère le scraping du site, l'extraction du JSON généré par Next.js (`__NEXT_DATA__`) contenu dans les pages HTML, et la génération correcte des URL d'images brutes.
- `auth.rs` : S'occupe de l'authentification Firebase (Identity Toolkit). Gère l'échange de l'email/mot de passe contre un `idToken` sécurisé, et met en cache ce token pour éviter de se reconnecter à chaque page.
- `networking.rs` : Fonctions utilitaires pour créer des requêtes HTTP (GET/POST) compatibles avec le système de requêtes d'Aidoku en injectant automatiquement le token d'autorisation.

### Modifier l'icône ou les métadonnées

- **L'icône** : Remplacez simplement le fichier `res/icon.png` par une image carrée de votre choix, puis recompilez.
- **Nom/Version** : Modifiez le fichier `res/source.json`.
- **Paramètres (Menu de connexion)** : Les champs de connexion dans l'application sont définis dans `res/settings.json`.
