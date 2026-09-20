# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [1.1.0] - 2026-09-21

### Added

- Desktop selection: the GUI presents every desktop published by StoreFront in a
  carousel, and connecting applies to the selected card. Desktops seen during the
  previous sign-in are remembered, so the choice is available before connecting.
- Several desktops can be launched from a single sign-in. The authenticated
  session is kept in memory for as long as the launcher window is open, so a
  second desktop needs no new one-time code.
- Per-desktop session state: each card reports whether that desktop currently has
  a Citrix session open, including sessions started outside the launcher.
- A "close after launch" setting that quits the launcher once a desktop has been
  handed over to Citrix Workspace.
- CLI commands `desktops`, to list the desktops StoreFront offers, and `launch`,
  to start one or more of them from a single sign-in.

### Changed

- Desktop matching is now ordered and refuses ambiguity: an exact resource id
  wins, then an exact name, and a name matching several resources reports the
  candidates instead of launching the first similar one.
- Each desktop gets its own ICA file, so simultaneous launches cannot overwrite
  one another.
- The settings field for the desktop name is now the default selection rather
  than the only launchable desktop.

### Fixed

- The GUI no longer reports a desktop as running after it was disconnected. The
  session monitor previously watched `wfcrun32`, which outlives the session it
  starts, and it never read process command lines because `sysinfo` does not
  collect them by default.

## [1.0.1] - 2026-07-24

### Fixed

- Enabled matching Wayland window and OpenGL backends so the RPM GUI starts in
  modern Wayland desktop sessions instead of failing while the CLI continues to
  work.
- Replaced the incomplete native-only macOS bundle with a correctly identified,
  icon-bearing universal app containing both Apple Silicon and Intel binaries.
- Added ad-hoc signing and a narrowly scoped per-application quarantine removal
  helper for macOS installations without an Apple Developer certificate.

## [1.0.0] - 2026-07-23

### Changed

- Redesigned the GUI with a responsive connection screen, a bounded settings form, a six-digit OTP editor, native file selection, and cross-platform icon assets.
- Added stage-specific connection feedback, progress indicators, and interaction locking while authentication, VDI preparation, or file selection is active.
- Added cross-platform active ICA session monitoring so the GUI returns to its ready state after the desktop session closes.
- Improved theme contrast for inputs, placeholders, secondary and disabled actions, cards, and long status messages, with consistent input padding.
- Stabilized button geometry across idle, hover, and pressed states to prevent visual layout jumps.
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
