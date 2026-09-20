# Citrix VDI Launcher

[English](README.md) · [Español](README.es.md) · Français · [Português](README.pt.md) · [Deutsch](README.de.md) · [Italiano](README.it.md) · [Русский](README.ru.md) · [简体中文](README.zh-CN.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [日本語](README.ja.md) · [한국어](README.ko.md)

---

Client desktop et CLI pour se connecter à Citrix Gateway et StoreFront VDI sans navigateur.

## Présentation et fonctionnalités

Il gère authentification, OTP/TOTP, découverte du VDI, téléchargement ICA et lancement de Citrix Workspace. Tout bureau publié par StoreFront peut être sélectionné, et plusieurs bureaux s'ouvrent après une seule authentification. GUI et CLI partagent un cœur Rust ; les secrets utilisent Windows DPAPI, macOS Keychain ou Linux Secret Service. Projet indépendant, non affilié à Citrix Systems, Inc.

## Prérequis et installation

Citrix Workspace, un Gateway/StoreFront compatible et Windows x86-64, macOS Intel/Apple Silicon ou Linux x86-64 avec Wayland/X11 sont requis. Téléchargez depuis [GitHub Releases](https://github.com/psimonov/citrix-launcher/releases/latest) le ZIP Windows, DEB/RPM Linux x86-64 ou ZIP universel macOS. Les paquets ARM64 Linux ne sont pas encore publiés. GitHub affiche le SHA-256 de chaque asset.

## Démarrage rapide

```text
citrix-vdi-cli config set --storefront https://gateway.example/ --vdi MY-DESKTOP --username user
citrix-vdi-cli connect
citrix-vdi-cli connect --otp 123456
citrix-vdi-cli desktops
citrix-vdi-cli launch MY-DESKTOP OTHER-DESKTOP
```

`config show`, `config path`, `detect-citrix` et le stockage de `--password`/`--totp-secret` sont également disponibles.

## Configuration et build

Le fichier est créé selon la plateforme ; `config path` affiche son chemin. Les secrets résident dans le coffre système.

```text
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release --bins
```

Les releases sont créées par GitHub Actions depuis des tags SemVer.

## Sécurité, contribution et licence

Ne publiez jamais credentials, OTP/TOTP, ICA, cookies, CSRF tokens ou logs. Voir [SECURITY.md](SECURITY.md) et [CONTRIBUTING.md](CONTRIBUTING.md). [Licence MIT](LICENSE).
