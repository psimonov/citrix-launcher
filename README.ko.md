# Citrix VDI Launcher

[English](README.md) · [Español](README.es.md) · [Français](README.fr.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Italiano](README.it.md) · [Русский](README.ru.md) · [简体中文](README.zh-CN.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [日本語](README.ja.md) · 한국어

---

브라우저 없이 Citrix Gateway와 StoreFront VDI에 연결하는 desktop 및 CLI client입니다.

## 개요와 기능

인증, OTP/TOTP, VDI 검색, ICA 다운로드, Citrix Workspace 실행을 처리합니다. StoreFront가 게시한 데스크톱을 선택할 수 있으며 한 번의 로그인으로 여러 데스크톱을 실행할 수 있습니다. GUI와 CLI는 Rust core를 공유하며 비밀은 Windows DPAPI, macOS Keychain 또는 Linux Secret Service로 보호합니다. Citrix Systems, Inc.와 관련 없는 독립 프로젝트입니다.

## 요구 사항과 설치

Citrix Workspace, 호환 Gateway/StoreFront, 그리고 Windows x86-64, macOS Intel/Apple Silicon 또는 Wayland/X11 Linux x86-64가 필요합니다. [GitHub Releases](https://github.com/psimonov/citrix-launcher/releases/latest)에서 Windows ZIP, Linux x86-64 DEB/RPM 또는 macOS universal ZIP을 받으세요. Linux ARM64 package는 아직 없습니다. GitHub가 각 asset의 SHA-256 digest를 표시합니다.

## 빠른 시작

```text
citrix-vdi-cli config set --storefront https://gateway.example/ --vdi MY-DESKTOP --username user
citrix-vdi-cli connect
citrix-vdi-cli connect --otp 123456
citrix-vdi-cli desktops
citrix-vdi-cli launch MY-DESKTOP OTHER-DESKTOP
```

`config show`, `config path`, `detect-citrix`, `--password`/`--totp-secret` 저장도 제공합니다. 비밀은 system credential store에 보관됩니다.

## 빌드

```text
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release --bins
```

GitHub Actions가 SemVer tag에서 release를 게시합니다.

## 보안, 기여 및 라이선스

credentials, OTP/TOTP, ICA, cookies, CSRF tokens 또는 logs를 공개하지 마세요. [SECURITY.md](SECURITY.md)와 [CONTRIBUTING.md](CONTRIBUTING.md)를 참조하세요. [MIT License](LICENSE).
