# Backlog and known limitations

This is a prioritized technical backlog, not a promise of scope.

## High priority

- Add redaction-safe structured diagnostics with explicit opt-in export.
- Add more parser fixtures synthesized from documented response shapes without production data.
- Improve authentication challenge detection and actionable error messages.
- Test GUI behavior, Citrix discovery, secret storage, and detached launch on each supported desktop environment.
- Add release checksums and define signing strategies for Windows, DEB, and RPM
  artifacts.

## Medium priority

- Add configuration schema/version migration tests.
- Make status text fully dynamic and localizable; consider an English/Russian localization layer.
- Add accessibility and high-DPI tests for the GUI and icon.
- Cache release packaging tools in GitHub Actions to reduce build time.
- Document supported Citrix Workspace and Gateway/StoreFront version ranges after controlled testing.
- Remove the vendored `wayland-scanner` security backport after a stable
  eframe/winit/wayland-scanner release accepts `quick-xml >=0.41.0`; rerun
  `cargo audit` and native Wayland plus X11 tests.

## Known limitations

- Direct protocol parsing is proven against one private deployment and may require adapters for others.
- There is no automated end-to-end test because it would require live credentials and OTP.
- Windows and Linux packages are not yet signed. macOS uses an ad-hoc signature
  and a per-app quarantine workaround because no Developer ID certificate is
  available.
- Linux keyring availability varies by desktop session; headless environments may not provide Secret Service.
- Linux uses native Wayland where available and retains X11/XWayland fallback.
  The vendored `wayland-scanner` constraint remains until patched stable
  upstream support is available.
- The current GUI toolkit is cross-platform and system-themed, but not a separate OS-native widget implementation for each platform.

## Definition of done for future changes

- User-visible requirement is met without weakening authentication or TLS.
- GUI and CLI remain consistent where applicable.
- Citrix session remains independent of launcher lifetime.
- No sensitive/environment-specific data enters the repository or logs.
- Formatting, clippy, tests, and relevant native builds pass.
- Documentation and changelog are updated.
