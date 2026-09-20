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

### Desktop chosen from the published list

Offer every desktop StoreFront publishes and let the user pick one; the
configured name is the default selection, not the only launchable desktop. This
replaces the earlier decision to expose exactly one VDI name, which forced a
settings edit to reach a second desktop.

The superseded rule still holds in one respect: there are no hidden aliases or
alternative name lists. A desktop is identified by its StoreFront resource id,
and name matching stays explainable — an exact id, then an exact name, then a
substring, and an ambiguous name is reported with its candidates rather than
resolved by picking the first similar resource.

Cache only the resource id and display name of the desktops seen at the last
sign-in, so the list is available before authenticating. Never cache
`desktophostname`: it is an internal infrastructure name.

### Session kept for the lifetime of the launcher window

Keep the authenticated Gateway/StoreFront session in memory in the worker thread
instead of discarding it after a launch, so several desktops can be opened from
one sign-in. This matters because a one-time code cannot be reused: without it, a
second desktop would require a new code, or a wait of up to 30 seconds for the
next TOTP window. The session never reaches disk and dies with the process.

If the gateway drops the session, the launcher signs in again silently when a
TOTP seed is stored. Without a seed it reports that a new code is needed.

### Session state observed, not remembered

Derive "this desktop is open" from the operating system process table on every
poll rather than from what the launcher itself launched. A Citrix process is
started with the desktop's own ICA file and keeps that path in its command line,
which identifies the desktop without any bookkeeping. This also shows sessions
started outside the launcher, and it cannot go stale.

Only processes that exist for the lifetime of a session may be consulted.
`wfcrun32.exe` is excluded on Windows: it is the connection manager, it survives
the session it started, and it keeps a stale ICA path.

### Native distribution only

Ship Windows EXE, macOS APP, DEB, and RPM. Do not add Snap, Flatpak, or AppImage unless the owner changes this decision.

### Tag-only releases

Build and publish releases only in GitHub Actions after pushing a valid `vMAJOR.MINOR.PATCH` SemVer tag. Local commands and manual workflow dispatch must not publish releases.

### Secure Linux display backend baseline

Support both native Wayland and X11/XWayland. The repository carries stable
`wayland-scanner 0.31.10` with a minimal security backport: its dependency
constraint is raised to patched stable `quick-xml 0.41`, and the renamed XML
1.0 content API is used at the single affected call site. This is required
because accessibility support already brings the Wayland event backend into
the Linux build; omitting the matching renderer backend makes the GUI fail on
Wayland with an unsupported native-window error. Do not suppress the
high-severity RustSec advisories or use an unreleased Git dependency. Remove
the vendored backport when a patched stable upstream release exists.

### Universal, ad-hoc signed macOS releases

Build both `aarch64-apple-darwin` and `x86_64-apple-darwin` slices and combine
the GUI and CLI as universal Mach-O executables. The owner does not have an
Apple Developer certificate, so bundles are ad-hoc signed and cannot be
notarized. Distribute a narrowly scoped installer that copies the app into the
current user's `~/Applications`, removes `com.apple.quarantine` only from that
app, verifies the ad-hoc signature, and launches it. Never disable Gatekeeper
or System Integrity Protection globally.

### OS-standard configuration

Use conventional per-user directories and auto-discover Citrix. The same saved settings serve GUI and CLI.

### English repository, Russian product

Repository-facing README, policies, workflows, commits, and handoff documentation are English. Current end-user labels and messages are Russian.

### Public, proprietary repository

The project is publicly readable and all rights are reserved. Public visibility does not grant a license. Builds are published as GitHub Release assets, not committed to Git.

## Change policy

When reversing a decision, append a dated replacement entry explaining why, update affected documents/code, and retain the old rationale for traceability.
