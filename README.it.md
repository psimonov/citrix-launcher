# Citrix VDI Launcher

[English](README.md) · [Español](README.es.md) · [Français](README.fr.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · Italiano · [Русский](README.ru.md) · [简体中文](README.zh-CN.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [日本語](README.ja.md) · [한국어](README.ko.md)

---

Client desktop e CLI per connettersi a Citrix Gateway e StoreFront VDI senza browser.

## Panoramica e funzionalità

Gestisce autenticazione, OTP/TOTP, ricerca VDI, download ICA e avvio di Citrix Workspace. GUI e CLI condividono un core Rust; i segreti usano Windows DPAPI, macOS Keychain o Linux Secret Service. Progetto indipendente, non affiliato a Citrix Systems, Inc.

## Requisiti e installazione

Servono Citrix Workspace, Gateway/StoreFront compatibile e Windows x86-64, macOS Intel/Apple Silicon o Linux x86-64 con Wayland/X11. Da [GitHub Releases](https://github.com/psimonov/citrix-launcher/releases/latest) scaricare ZIP Windows, DEB/RPM Linux x86-64 o ZIP universale macOS. I pacchetti Linux ARM64 non sono ancora pubblicati. GitHub mostra il digest SHA-256 di ogni asset.

## Avvio rapido

```text
citrix-vdi-cli config set --storefront https://gateway.example/ --vdi MY-DESKTOP --username user
citrix-vdi-cli connect
citrix-vdi-cli connect --otp 123456
```

Sono disponibili anche `config show`, `config path`, `detect-citrix` e memorizzazione di `--password`/`--totp-secret`.

## Configurazione e build

Il file viene creato secondo la piattaforma; `config path` mostra il percorso. I segreti restano nel credential store di sistema.

```text
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release --bins
```

GitHub Actions pubblica release da tag SemVer.

## Sicurezza, contributi e licenza

Non pubblicare credentials, OTP/TOTP, ICA, cookies, CSRF tokens o logs. Vedere [SECURITY.md](SECURITY.md) e [CONTRIBUTING.md](CONTRIBUTING.md). [Licenza MIT](LICENSE).
