# Decision record

## Accepted decisions

### Browserless protocol implementation

Use direct HTTPS requests instead of browser automation or an embedded WebView. This makes normal operation invisible, deterministic, and usable from both GUI and CLI. The cost is sensitivity to Citrix Gateway/StoreFront protocol variations.

### Rust shared core

Use one Rust library for authentication, configuration, crypto, and launch orchestration, with separate GUI and CLI entry points. This minimizes runtime dependencies and behavior drift.

### Same-device TOTP is optional

Support a stored Base32 TOTP seed because the owner explicitly accepts the reduction in second-device separation. Preserve manual OTP as the fallback. Do not attempt to extract enrollment secrets from an authenticator.

### Native secret protection

Use DPAPI on Windows and the platform keyring on macOS/Linux. Never store plaintext secrets in JSON.

### Detached Citrix lifecycle

Citrix Workspace owns the VDI session after ICA handoff; closing the launcher must not terminate Citrix.

### One configured VDI display name

Expose exactly one VDI-name setting. Do not maintain hidden aliases or alternative name lists. Resource matching may normalize superficial formatting, but it must remain explainable and driven by the configured name.

### Native distribution only

Ship Windows EXE, macOS APP, DEB, and RPM. Do not add Snap, Flatpak, or AppImage unless the owner changes this decision.

### Tag-only releases

Build and publish releases only in GitHub Actions after pushing a valid `vMAJOR.MINOR.PATCH` SemVer tag. Local commands and manual workflow dispatch must not publish releases.

### OS-standard configuration

Use conventional per-user directories and auto-discover Citrix. The same saved settings serve GUI and CLI.

### English repository, Russian product

Repository-facing README, policies, workflows, commits, and handoff documentation are English. Current end-user labels and messages are Russian.

### Private, proprietary repository

The project is private and all rights are reserved. Builds are published as private GitHub Release assets, not committed to Git.

## Change policy

When reversing a decision, append a dated replacement entry explaining why, update affected documents/code, and retain the old rationale for traceability.
