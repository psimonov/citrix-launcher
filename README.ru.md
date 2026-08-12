# Citrix VDI Launcher

[English](README.md) · [Español](README.es.md) · [Français](README.fr.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Italiano](README.it.md) · Русский · [简体中文](README.zh-CN.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [日本語](README.ja.md) · [한국어](README.ko.md)

---

Desktop- и CLI-клиент для подключения к Citrix Gateway и StoreFront VDI без браузера.

## Обзор и возможности

Приложение выполняет аутентификацию, OTP/TOTP, поиск назначенного VDI, загрузку ICA и запуск Citrix Workspace. GUI и CLI используют общее Rust-ядро; секреты защищаются через Windows DPAPI, macOS Keychain или Linux Secret Service. Проект независим и не аффилирован с Citrix Systems, Inc.

## Требования

Citrix Workspace, доступ к совместимому Gateway/StoreFront и одна из платформ: Windows x86-64, macOS Intel/Apple Silicon либо Linux x86-64 с Wayland или X11/XWayland.

## Установка

Скачайте пакет из [GitHub Releases](https://github.com/psimonov/citrix-launcher/releases/latest): Windows x86-64 ZIP, DEB/RPM для Linux x86-64 или universal ZIP для macOS. Linux ARM64 DEB/RPM пока не публикуются. SHA-256 digest каждого файла отображается GitHub на странице релиза.

## Быстрый старт и CLI

```text
citrix-vdi-cli config set --storefront https://gateway.example/ --vdi MY-DESKTOP --username user
citrix-vdi-cli connect
citrix-vdi-cli connect --otp 123456
```

Дополнительно доступны `config show`, `config path`, `detect-citrix` и сохранение `--password`/`--totp-secret`.

## Конфигурация

Файл создаётся автоматически: `%APPDATA%\CitrixVdiLauncher\config.json` в Windows, `~/Library/Application Support/CitrixVdiLauncher/config.json` в macOS и `${XDG_CONFIG_HOME:-~/.config}/citrix-vdi-launcher/config.json` в Linux. Секреты хранятся в credential store ОС.

## Сборка

```text
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release --bins
```

Релизы собираются GitHub Actions по SemVer-тегам.

## Безопасность, участие и лицензия

Не публикуйте credentials, OTP/TOTP, ICA, cookies, CSRF tokens или логи. См. [SECURITY.md](SECURITY.md) и [CONTRIBUTING.md](CONTRIBUTING.md). Проект распространяется по [лицензии MIT](LICENSE).
