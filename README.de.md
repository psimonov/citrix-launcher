# Citrix VDI Launcher

[English](README.md) · [Español](README.es.md) · [Français](README.fr.md) · [Português](README.pt.md) · Deutsch · [Italiano](README.it.md) · [Русский](README.ru.md) · [简体中文](README.zh-CN.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [日本語](README.ja.md) · [한국어](README.ko.md)

---

Desktop- und CLI-Client für Citrix Gateway und StoreFront VDI ohne Browser.

## Überblick und Funktionen

Er übernimmt Anmeldung, OTP/TOTP, VDI-Erkennung, ICA-Download und Start von Citrix Workspace. GUI und CLI teilen einen Rust-Kern; Geheimnisse liegen in Windows DPAPI, macOS Keychain oder Linux Secret Service. Unabhängiges, nicht mit Citrix Systems, Inc. verbundenes Projekt.

## Voraussetzungen und Installation

Erforderlich sind Citrix Workspace, kompatibles Gateway/StoreFront sowie Windows x86-64, macOS Intel/Apple Silicon oder Linux x86-64 mit Wayland/X11. Unter [GitHub Releases](https://github.com/psimonov/citrix-launcher/releases/latest) stehen Windows-ZIP, Linux-DEB/RPM x86-64 und universal macOS ZIP bereit. Linux ARM64 wird noch nicht veröffentlicht. GitHub zeigt den SHA-256-Digest jedes Assets.

## Schnellstart

```text
citrix-vdi-cli config set --storefront https://gateway.example/ --vdi MY-DESKTOP --username user
citrix-vdi-cli connect
citrix-vdi-cli connect --otp 123456
```

Verfügbar sind außerdem `config show`, `config path`, `detect-citrix` und das Speichern von `--password`/`--totp-secret`.

## Konfiguration und Build

Die Datei wird plattformgerecht erstellt; `config path` zeigt den Pfad. Geheimnisse verbleiben im System-Credential-Store.

```text
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release --bins
```

GitHub Actions veröffentlicht Releases anhand von SemVer-Tags.

## Sicherheit, Mitwirkung und Lizenz

Veröffentlichen Sie keine credentials, OTP/TOTP, ICA, cookies, CSRF tokens oder logs. Siehe [SECURITY.md](SECURITY.md) und [CONTRIBUTING.md](CONTRIBUTING.md). [MIT-Lizenz](LICENSE).
