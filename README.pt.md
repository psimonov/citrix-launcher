# Citrix VDI Launcher

[English](README.md) · [Español](README.es.md) · [Français](README.fr.md) · Português · [Deutsch](README.de.md) · [Italiano](README.it.md) · [Русский](README.ru.md) · [简体中文](README.zh-CN.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [日本語](README.ja.md) · [한국어](README.ko.md)

---

Cliente desktop e CLI para conectar ao Citrix Gateway e StoreFront VDI sem navegador.

## Visão geral e recursos

Executa autenticação, OTP/TOTP, descoberta do VDI, download ICA e abertura no Citrix Workspace. GUI e CLI compartilham um núcleo Rust; segredos usam Windows DPAPI, macOS Keychain ou Linux Secret Service. Projeto independente, sem afiliação à Citrix Systems, Inc.

## Requisitos e instalação

Requer Citrix Workspace, Gateway/StoreFront compatível e Windows x86-64, macOS Intel/Apple Silicon ou Linux x86-64 com Wayland/X11. Baixe em [GitHub Releases](https://github.com/psimonov/citrix-launcher/releases/latest) o ZIP Windows, DEB/RPM Linux x86-64 ou ZIP universal macOS. Pacotes Linux ARM64 ainda não são publicados. GitHub mostra o SHA-256 de cada asset.

## Início rápido

```text
citrix-vdi-cli config set --storefront https://gateway.example/ --vdi MY-DESKTOP --username user
citrix-vdi-cli connect
citrix-vdi-cli connect --otp 123456
```

Também há `config show`, `config path`, `detect-citrix` e armazenamento de `--password`/`--totp-secret`.

## Configuração e build

O arquivo é criado conforme a plataforma; `config path` mostra o caminho. Segredos ficam no cofre do sistema.

```text
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release --bins
```

Releases são criados pelo GitHub Actions a partir de tags SemVer.

## Segurança, contribuição e licença

Não publique credentials, OTP/TOTP, ICA, cookies, CSRF tokens ou logs. Veja [SECURITY.md](SECURITY.md) e [CONTRIBUTING.md](CONTRIBUTING.md). [Licença MIT](LICENSE).
