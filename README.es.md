# Citrix VDI Launcher

[English](README.md) · Español · [Français](README.fr.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Italiano](README.it.md) · [Русский](README.ru.md) · [简体中文](README.zh-CN.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [日本語](README.ja.md) · [한국어](README.ko.md)

---

Cliente desktop y CLI para conectarse a Citrix Gateway y StoreFront VDI sin navegador.

## Descripción y funciones

Gestiona autenticación, OTP/TOTP, descubrimiento del VDI, descarga ICA y ejecución de Citrix Workspace. Se puede elegir cualquier escritorio publicado en StoreFront y abrir varios con un solo inicio de sesión. GUI y CLI comparten un núcleo Rust; los secretos usan Windows DPAPI, macOS Keychain o Linux Secret Service. Proyecto independiente, no afiliado a Citrix Systems, Inc.

## Requisitos e instalación

Requiere Citrix Workspace, acceso a Gateway/StoreFront compatible y Windows x86-64, macOS Intel/Apple Silicon o Linux x86-64 con Wayland/X11. Descargue desde [GitHub Releases](https://github.com/psimonov/citrix-launcher/releases/latest) el ZIP Windows, DEB/RPM Linux x86-64 o ZIP universal macOS. Aún no hay DEB/RPM ARM64. GitHub muestra el digest SHA-256 de cada asset.

## Inicio rápido

```text
citrix-vdi-cli config set --storefront https://gateway.example/ --vdi MY-DESKTOP --username user
citrix-vdi-cli connect
citrix-vdi-cli connect --otp 123456
citrix-vdi-cli desktops
citrix-vdi-cli launch MY-DESKTOP OTHER-DESKTOP
```

También están disponibles `config show`, `config path`, `detect-citrix` y almacenamiento de `--password`/`--totp-secret`.

## Configuración

El archivo se crea automáticamente según la plataforma; `config path` muestra su ruta. Los secretos se guardan en el almacén de credenciales del sistema.

## Compilación

```text
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release --bins
```

Los releases se crean mediante GitHub Actions desde tags SemVer.

## Seguridad, contribución y licencia

No publique credenciales, OTP/TOTP, ICA, cookies, CSRF tokens o logs. Consulte [SECURITY.md](SECURITY.md) y [CONTRIBUTING.md](CONTRIBUTING.md). [Licencia MIT](LICENSE).
