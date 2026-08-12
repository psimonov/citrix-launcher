# Citrix VDI Launcher

[English](README.md) · [Español](README.es.md) · [Français](README.fr.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Italiano](README.it.md) · [Русский](README.ru.md) · [简体中文](README.zh-CN.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · 日本語 · [한국어](README.ko.md)

---

ブラウザーを使わずCitrix GatewayとStoreFront VDIへ接続するDesktopおよびCLIクライアントです。

## 概要と機能

認証、OTP/TOTP、VDI検出、ICAダウンロード、Citrix Workspace起動を行います。GUIとCLIはRustコアを共有し、秘密情報はWindows DPAPI、macOS Keychain、Linux Secret Serviceで保護します。Citrix Systems, Inc.とは無関係の独立プロジェクトです。

## 要件とインストール

Citrix Workspace、互換Gateway/StoreFront、およびWindows x86-64、macOS Intel/Apple Silicon、またはWayland/X11対応Linux x86-64が必要です。[GitHub Releases](https://github.com/psimonov/citrix-launcher/releases/latest)からWindows ZIP、Linux x86-64 DEB/RPM、macOS universal ZIPを取得してください。Linux ARM64 packageは未提供です。GitHubに各assetのSHA-256 digestが表示されます。

## クイックスタート

```text
citrix-vdi-cli config set --storefront https://gateway.example/ --vdi MY-DESKTOP --username user
citrix-vdi-cli connect
citrix-vdi-cli connect --otp 123456
```

`config show`、`config path`、`detect-citrix`、`--password`/`--totp-secret`保存も利用できます。秘密情報はsystem credential storeに保存されます。

## ビルド

```text
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release --bins
```

GitHub ActionsがSemVer tagからreleaseを公開します。

## セキュリティ、貢献、ライセンス

credentials、OTP/TOTP、ICA、cookies、CSRF tokens、logsを公開しないでください。[SECURITY.md](SECURITY.md)と[CONTRIBUTING.md](CONTRIBUTING.md)を参照してください。[MIT License](LICENSE)。
