# Citrix VDI Launcher

English · [Español](README.es.md) · [Français](README.fr.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Italiano](README.it.md) · [Русский](README.ru.md) · [简体中文](README.zh-CN.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [日本語](README.ja.md) · [한국어](README.ko.md)

---

A browserless desktop and CLI client for connecting to Citrix Gateway and StoreFront VDI resources.

## Overview

Citrix VDI Launcher is intended for users who need a direct, repeatable way to authenticate, discover an assigned desktop, download its ICA data, and start it with Citrix Workspace without opening a browser. The GUI and CLI use the same native Rust core and configuration.

This project is independent and is not affiliated with or endorsed by Citrix Systems, Inc.

## Features

- Direct Citrix Gateway and StoreFront authentication.
- Manual OTP entry or automatic TOTP generation from a stored secret.
- Choice of any desktop published by StoreFront, with several desktops launchable
  from a single sign-in.
- Per-desktop session state, including sessions started outside the launcher.
- Automatic VDI resource discovery and ICA launch.
- GUI and scriptable CLI backed by the same configuration.
- Automatic Citrix Workspace discovery on supported operating systems.
- Credential protection through Windows DPAPI, macOS Keychain, or Linux Secret Service.
- No browser, WebView, .NET, Node.js, Python, or OpenSSL runtime dependency.

## Requirements

- Citrix Workspace installed on the target computer.
- Access to a compatible Citrix Gateway or StoreFront deployment.
- Windows x86-64, macOS Intel/Apple Silicon, or x86-64 Linux with Wayland or X11/XWayland.
- Linux desktop integration requires Secret Service and the native libraries installed by the DEB/RPM package manager.

## Installation

Download the package for your platform from [GitHub Releases](https://github.com/psimonov/citrix-launcher/releases/latest):

- Windows x86-64: `citrix-vdi-launcher-windows-x86_64.zip`;
- Debian/Ubuntu x86-64: `citrix-vdi-launcher_<version>_amd64.deb`;
- Fedora/RHEL-compatible x86-64: `citrix-vdi-launcher-<version>.x86_64.rpm`;
- macOS Intel and Apple Silicon: `citrix-vdi-launcher-macos-universal.zip`.

Linux ARM64 DEB and RPM artifacts are not yet published. Do not install x86-64 packages on ARM64 systems.

Release assets are built and published by GitHub Actions from SemVer tags. GitHub displays the SHA-256 digest for each asset on the release page.

## Quick start

1. Install Citrix Workspace.
2. Install or extract Citrix VDI Launcher for your platform.
3. Start the GUI, enter the StoreFront URL, default desktop name, username, password, and optional TOTP secret.
4. Connect and complete OTP authentication when prompted.
5. After the first connection, pick any published desktop from the carousel and connect again without a new one-time code.

CLI example:

```text
citrix-vdi-cli config set --storefront https://gateway.example/ --vdi MY-DESKTOP --username user
citrix-vdi-cli connect
```

## Usage

Show settings without exposing secrets:

```text
citrix-vdi-cli config show
```

Store credentials or a TOTP secret:

```text
citrix-vdi-cli config set --password "password" --totp-secret "BASE32SECRET"
```

Connect with an explicit one-time password:

```text
citrix-vdi-cli connect --otp 123456
```

List the desktops StoreFront offers:

```text
citrix-vdi-cli desktops
```

Launch several desktops from a single sign-in:

```text
citrix-vdi-cli launch MY-DESKTOP OTHER-DESKTOP
```

Locate Citrix Workspace manually:

```text
citrix-vdi-cli detect-citrix
```

## Configuration

The configuration file is created on first launch:

- Windows: `%APPDATA%\CitrixVdiLauncher\config.json`;
- macOS: `~/Library/Application Support/CitrixVdiLauncher/config.json`;
- Linux: `${XDG_CONFIG_HOME:-~/.config}/citrix-vdi-launcher/config.json`.

Print the effective path with `citrix-vdi-cli config path`. Secrets are stored through the operating system credential store rather than written to this JSON file.

## Build and verification

```text
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release --bins
```

Platform packaging scripts are available under `packaging/`. Release publication is reserved for GitHub Actions triggered by a tag such as `v1.2.3`.

## Security

Never commit credentials, OTP/TOTP secrets, ICA files, cookies, CSRF tokens, logs, or StoreFront response dumps. Report vulnerabilities according to [SECURITY.md](SECURITY.md).

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request. Bug reports and reproducible compatibility reports are welcome in [GitHub Issues](https://github.com/psimonov/citrix-launcher/issues).

## License

Distributed under the [MIT License](LICENSE).
