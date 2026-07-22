# Citrix VDI Launcher

A cross-platform desktop and CLI application that connects to Citrix VDI through Citrix Gateway and StoreFront without opening a browser.

## Features

- Direct Citrix Gateway and StoreFront network authentication.
- Manual OTP entry or automatic TOTP generation from a stored secret.
- Automatic VDI resource discovery and ICA launch.
- GUI and CLI applications backed by the same configuration.
- Automatic Citrix Workspace discovery on supported operating systems.
- Credentials protected by Windows DPAPI, macOS Keychain, or Linux Secret Service.
- Citrix sessions remain active after the launcher is closed.
- Native Windows EXE, macOS app bundle, Debian package, and RPM package outputs.
- No browser, WebView, .NET, Node.js, Python, or OpenSSL runtime dependency.

Citrix Workspace must be installed on the target computer.

## Configuration

The configuration file is created automatically on first launch. Its location follows each operating system's conventions:

- Windows: `%APPDATA%\CitrixVdiLauncher\config.json`
- macOS: `~/Library/Application Support/CitrixVdiLauncher/config.json`
- Linux: `${XDG_CONFIG_HOME:-~/.config}/citrix-vdi-launcher/config.json`

Print the effective path:

```text
citrix-vdi-cli config path
```

The Citrix Workspace executable is detected in standard installation directories and through `PATH`. Run detection manually with:

```text
citrix-vdi-cli detect-citrix
```

## CLI usage

Show settings without exposing secrets:

```text
citrix-vdi-cli config show
```

Update settings:

```text
citrix-vdi-cli config set --storefront https://gateway.example/ --vdi MY-DESKTOP --username user
citrix-vdi-cli config set --password "password" --totp-secret "BASE32SECRET"
```

Connect using a stored TOTP secret:

```text
citrix-vdi-cli connect
```

If no TOTP secret is stored, the CLI prompts for an OTP. It can also be supplied explicitly:

```text
citrix-vdi-cli connect --otp 123456
```

## Development

```text
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release --bins
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for repository guidelines.

## Packaging

Windows EXE files:

```powershell
.\packaging\build-windows.ps1
```

Native Debian and RPM packages, built on Linux:

```text
./packaging/build-linux.sh
```

macOS app bundle, built on macOS:

```text
./packaging/build-macos.sh
```

GitHub Releases are built and published only by GitHub Actions after a SemVer tag such as `v1.2.3` is pushed. Local tools never publish releases.

Snap, Flatpak, and AppImage packages are intentionally not produced.

## Runtime dependencies

- Windows releases are standalone EXE files and require no Rust or MinGW runtime.
- macOS uses operating-system frameworks and Keychain.
- Linux native packages use X11/XWayland and Secret Service components. Native Wayland is temporarily disabled until the stable upstream dependency chain includes the published `quick-xml` security fixes.
- Citrix Workspace is the only application-level runtime requirement.

## Security

Never commit credentials, OTP/TOTP secrets, ICA files, cookies, CSRF tokens, logs, or StoreFront response dumps. See [SECURITY.md](SECURITY.md).

This repository is publicly readable but proprietary. Public access does not grant permission to use, modify, or redistribute the software; see [LICENSE](LICENSE).
