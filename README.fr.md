# Codex Tray

[English](README.md) · [Español](README.es.md) · Français · [Português](README.pt.md) · [Deutsch](README.de.md) · [Italiano](README.it.md) · [Русский](README.ru.md) · [简体中文](README.zh-CN.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [日本語](README.ja.md) · [한국어](README.ko.md)

---

Un indicateur natif dans la zone de notification Windows pour suivre le quota Codex restant.

## Présentation

Codex Tray permet de consulter le quota Codex actuel sans garder l’application ou la CLI au premier plan. Il s’exécute comme une petite application dans la zone de notification Windows, réutilise la session Codex CLI authentifiée de l’utilisateur courant et affiche un panneau compact au survol de l’icône.

L’application communique uniquement avec `codex app-server`, installé localement. Elle ne demande aucune clé API et ne lit ni ne copie directement `~/.codex/auth.json`.

## Fonctionnalités

- Mises à jour du quota en direct via les notifications `account/rateLimits/updated`.
- Panneau compact adapté au DPI avec des lignes stables au format `Libellé: valeur`.
- Prise en charge des thèmes clair et sombre, de la couleur d’accentuation et de la transparence de Windows.
- Icônes alignées sur les pixels pour les niveaux de quota et les états d’erreur.
- Affichage du panneau au survol et masquage lorsque le pointeur s’éloigne.
- Menu contextuel avec gestion du démarrage avec Windows et commande explicite de fermeture.
- Aucune infobulle système au-dessus de l’icône.
- États distincts pour le chargement, la reconnexion, l’authentification, l’abonnement, la CLI absente, le quota épuisé et les erreurs app-server.

## Prérequis

- Windows 11 x86-64.
- [Codex CLI](https://learn.chatgpt.com/docs/codex/cli) disponible dans `PATH`.
- Une session Codex CLI authentifiée créée avec `codex login`.

Codex Tray n’implémente actuellement qu’un backend Windows natif. Aucun artefact Linux, macOS ou Windows ARM64 n’est publié tant que les backends correspondants ne sont pas implémentés et testés.

## Installation

1. Ouvrez la [dernière version GitHub](https://github.com/psimonov/codex-tray/releases/latest).
2. Téléchargez `codex-tray-<version>-windows-x86_64.exe` et son fichier `.sha256`.
3. Vérifiez la somme de contrôle SHA-256.
4. Placez l’exécutable dans un dossier permanent, puis lancez-le.

Exemple de vérification dans PowerShell :

```powershell
Get-FileHash .\codex-tray-0.2.0-windows-x86_64.exe -Algorithm SHA256
```

Aucun programme d’installation n’est nécessaire. La version est distribuée sous la forme d’un exécutable portable unique ; la commande `codex` reste une dépendance d’exécution externe.

## Démarrage rapide

```powershell
codex login
.\codex-tray-0.2.0-windows-x86_64.exe
```

L’application démarre masquée et ajoute son icône à la zone de notification Windows.

## Utilisation

- Survolez l’icône pour afficher le panneau du quota.
- Éloignez le pointeur de l’icône pour masquer le panneau.
- Faites un clic droit pour masquer le panneau et ouvrir le menu contextuel.
- Activez **Démarrer avec Windows** pour enregistrer ou supprimer le chemin de l’exécutable courant dans la clé utilisateur `Run`.
- Sélectionnez **Fermer** pour arrêter Codex Tray et son processus enfant app-server.

Les mises à jour arrivent sur une connexion persistante à `codex app-server`. Codex Tray effectue une lecture initiale du compte et des limites, fusionne les notifications partielles suivantes et se reconnecte si app-server s’arrête de manière inattendue.

## Configuration

Codex Tray n’utilise ni fichier de configuration ni variable d’environnement. Le démarrage facultatif avec Windows se règle depuis le menu contextuel et utilise toujours le chemin détecté dynamiquement de l’exécutable en cours d’utilisation.

## Compilation depuis les sources

Le dépôt fixe la chaîne d’outils Rust requise.

```powershell
git clone https://github.com/psimonov/codex-tray.git
Set-Location codex-tray
cargo build --release --locked
```

L’exécutable produit est `target\release\codex-tray.exe`.

## Tests

```powershell
cargo fmt --all -- --check
cargo test --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
```

## Versions

Les tags utilisent le format `vMAJOR.MINOR.PATCH`. GitHub Actions vérifie la concordance du tag avec `Cargo.toml`, exécute les contrôles, compile l’exécutable Windows x86-64 et publie celui-ci avec sa somme SHA-256 dans une même version GitHub.

Le projet ne prend actuellement en charge que Windows ; seuls les artefacts Windows x86-64 sont donc publiés. Il s’agit d’une décision de plateforme explicite, et non d’une affirmation non vérifiée de compatibilité multiplateforme.

## Sécurité

Consultez [SECURITY.md](SECURITY.md) pour connaître les versions prises en charge et le canal privé de signalement des vulnérabilités. Ne divulguez pas de vulnérabilité dans une issue publique.

## Contribution

Consultez [CONTRIBUTING.md](CONTRIBUTING.md) pour le processus de développement et les règles relatives aux commits.

## Licence

Codex Tray est distribué sous [licence MIT](LICENSE).

## Référence du protocole

- [Codex App Server](https://learn.chatgpt.com/docs/app-server)
