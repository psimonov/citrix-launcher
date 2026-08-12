# Citrix VDI Launcher

[English](README.md) · [Español](README.es.md) · [Français](README.fr.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Italiano](README.it.md) · [Русский](README.ru.md) · [简体中文](README.zh-CN.md) · हिन्दी · [العربية](README.ar.md) · [日本語](README.ja.md) · [한국어](README.ko.md)

---

ब्राउज़र के बिना Citrix Gateway और StoreFront VDI से जुड़ने वाला desktop और CLI client।

## परिचय और विशेषताएँ

यह authentication, OTP/TOTP, VDI discovery, ICA download और Citrix Workspace launch करता है। GUI और CLI एक Rust core साझा करते हैं; secrets Windows DPAPI, macOS Keychain या Linux Secret Service में सुरक्षित रहते हैं। यह स्वतंत्र project है और Citrix Systems, Inc. से संबद्ध नहीं।

## आवश्यकताएँ और स्थापना

Citrix Workspace, compatible Gateway/StoreFront और Windows x86-64, macOS Intel/Apple Silicon या Wayland/X11 वाला Linux x86-64 आवश्यक है। [GitHub Releases](https://github.com/psimonov/citrix-launcher/releases/latest) से Windows ZIP, Linux x86-64 DEB/RPM या universal macOS ZIP लें। Linux ARM64 packages अभी उपलब्ध नहीं हैं। GitHub हर asset का SHA-256 digest दिखाता है।

## त्वरित शुरुआत

```text
citrix-vdi-cli config set --storefront https://gateway.example/ --vdi MY-DESKTOP --username user
citrix-vdi-cli connect
citrix-vdi-cli connect --otp 123456
```

`config show`, `config path`, `detect-citrix` और `--password`/`--totp-secret` storage भी उपलब्ध हैं। Secrets system credential store में रहते हैं।

## Build

```text
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release --bins
```

GitHub Actions SemVer tags से releases बनाता है।

## सुरक्षा, योगदान और लाइसेंस

credentials, OTP/TOTP, ICA, cookies, CSRF tokens या logs प्रकाशित न करें। [SECURITY.md](SECURITY.md) और [CONTRIBUTING.md](CONTRIBUTING.md) देखें। [MIT License](LICENSE)।
