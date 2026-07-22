# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Changed

- Updated direct Rust dependencies and GitHub Actions to their latest stable releases.
- Migrated the GUI to the eframe 0.35 application API and the Glow renderer.
- Added a weekly and dependency-change RustSec audit workflow.
- Added a patched stable `wayland-scanner` dependency constraint for `quick-xml 0.41` to resolve RUSTSEC-2026-0194 and RUSTSEC-2026-0195.
- Documented the workspace-wide stable and security-patched dependency policy.

## [0.0.1] - 2026-07-22

### Added

- Browserless Citrix Gateway and StoreFront authentication.
- Manual OTP and TOTP-secret modes.
- Automatic VDI discovery and native ICA launch.
- GUI and CLI applications sharing one platform-native configuration.
- Windows DPAPI, macOS Keychain, and Linux Secret Service integration.
- Native Windows EXE, macOS app bundle, DEB, and RPM packaging.
- Cross-platform application icon and automated release workflows.
