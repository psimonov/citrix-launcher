# Backlog and known limitations

This is a prioritized technical backlog, not a promise of scope.

## High priority

- Add redaction-safe structured diagnostics with explicit opt-in export.
- Add more parser fixtures synthesized from documented response shapes without production data.
- Improve authentication challenge detection and actionable error messages.
- Test GUI behavior, Citrix discovery, secret storage, and detached launch on each supported desktop environment.
- Add release checksums and consider artifact signing/notarization strategy.

## Medium priority

- Add configuration schema/version migration tests.
- Make status text fully dynamic and localizable; consider an English/Russian localization layer.
- Add accessibility and high-DPI tests for the GUI and icon.
- Cache release packaging tools in GitHub Actions to reduce build time.
- Evaluate universal macOS output (Apple Silicon and Intel) instead of the current native runner architecture artifact.
- Document supported Citrix Workspace and Gateway/StoreFront version ranges after controlled testing.
- Remove the vendored `wayland-scanner` security backport and re-enable native Wayland after a stable eframe/winit/wayland-scanner release accepts `quick-xml >=0.41.0`; rerun `cargo audit` and native GNOME/KDE tests. Until then Linux uses X11/XWayland to avoid RUSTSEC-2026-0194, RUSTSEC-2026-0195, and the unmaintained Wayland font-parser chain.

## Known limitations

- Direct protocol parsing is proven against one private deployment and may require adapters for others.
- There is no automated end-to-end test because it would require live credentials and OTP.
- Native packages are not yet signed/notarized.
- Linux keyring availability varies by desktop session; headless environments may not provide Secret Service.
- Native Wayland is temporarily disabled for dependency security; GNOME and KDE use XWayland, while MATE uses X11 directly.
- The current GUI toolkit is cross-platform and system-themed, but not a separate OS-native widget implementation for each platform.

## Definition of done for future changes

- User-visible requirement is met without weakening authentication or TLS.
- GUI and CLI remain consistent where applicable.
- Citrix session remains independent of launcher lifetime.
- No sensitive/environment-specific data enters the repository or logs.
- Formatting, clippy, tests, and relevant native builds pass.
- Documentation and changelog are updated.
