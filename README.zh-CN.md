# Citrix VDI Launcher

[English](README.md) · [Español](README.es.md) · [Français](README.fr.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Italiano](README.it.md) · [Русский](README.ru.md) · 简体中文 · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [日本語](README.ja.md) · [한국어](README.ko.md)

---

无需浏览器即可连接 Citrix Gateway 和 StoreFront VDI 的桌面与 CLI 客户端。

## 概述与功能

支持认证、OTP/TOTP、VDI 发现、ICA 下载及启动 Citrix Workspace。可选择 StoreFront 发布的任意桌面，一次登录即可启动多个桌面。GUI 与 CLI 共享 Rust 核心；机密信息使用 Windows DPAPI、macOS Keychain 或 Linux Secret Service。本项目独立开发，与 Citrix Systems, Inc. 无关联。

## 要求与安装

需要 Citrix Workspace、兼容的 Gateway/StoreFront，以及 Windows x86-64、macOS Intel/Apple Silicon 或带 Wayland/X11 的 Linux x86-64。从 [GitHub Releases](https://github.com/psimonov/citrix-launcher/releases/latest) 下载 Windows ZIP、Linux x86-64 DEB/RPM 或 macOS universal ZIP。目前尚无 Linux ARM64 包。GitHub 会显示每个 asset 的 SHA-256 digest。

## 快速开始

```text
citrix-vdi-cli config set --storefront https://gateway.example/ --vdi MY-DESKTOP --username user
citrix-vdi-cli connect
citrix-vdi-cli connect --otp 123456
citrix-vdi-cli desktops
citrix-vdi-cli launch MY-DESKTOP OTHER-DESKTOP
```

还提供 `config show`、`config path`、`detect-citrix`，并可保存 `--password`/`--totp-secret`。配置文件按平台自动创建，机密信息保存在系统 credential store。

## 构建

```text
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release --bins
```

GitHub Actions 根据 SemVer tag 发布 release。

## 安全、贡献与许可证

请勿发布 credentials、OTP/TOTP、ICA、cookies、CSRF tokens 或 logs。参阅 [SECURITY.md](SECURITY.md) 和 [CONTRIBUTING.md](CONTRIBUTING.md)。采用 [MIT 许可证](LICENSE)。
