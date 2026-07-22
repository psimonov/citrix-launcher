# Architecture

## Repository map

- `src/main.rs`: eframe/egui desktop UI and background launch worker.
- `src/bin/citrix-vdi-cli.rs`: CLI commands and interactive OTP fallback.
- `src/config.rs`: configuration persistence, secret references, and Citrix discovery.
- `src/crypto.rs`: Windows DPAPI or OS keyring integration.
- `src/automation.rs`: validation, OTP selection/generation, and orchestration.
- `src/network.rs`: Citrix Gateway/StoreFront HTTP protocol and detached ICA launch.
- `build.rs`: Windows executable resources and embedded icon.
- `assets/icons/`: cross-platform raster, ICO, and ICNS assets.
- `packaging/`: native packaging helpers and Linux desktop entry.
- `.github/workflows/`: cross-platform CI and tagged releases.

## Connection lifecycle

1. Load configuration and protected credentials.
2. Use manual OTP, or normalize the Base32 seed and generate SHA-1/6-digit/30-second TOTP.
3. GET the gateway entry page and capture the gateway CSRF token and security JavaScript.
4. Parse NetScaler HMAC state arrays from that JavaScript.
5. GET `/nf/auth/getAuthenticationRequirements.do` and capture `StateContext`.
6. POST username, password, OTP, and `StateContext` to `/nf/auth/doAuthentication.do` with the NetScaler-specific AJAX hash and headers.
7. Handle the optional `/p/u/setClient.do` exchange by selecting the ICA client.
8. Enter the StoreFront portal and POST `Home/Configuration` to establish StoreFront state.
9. POST `Resources/List`, preserving cookies and using the StoreFront CSRF token separately from the gateway token.
10. Match the configured VDI display name using normalized text and obtain its launch URL.
11. Request ICA launch data, write a short-lived local `.ica` file, and start Citrix Workspace detached with stdin/stdout/stderr disconnected.

The automation layer emits redaction-safe progress events at the OTP, Gateway, StoreFront, resource discovery, VDI preparation, ICA, and Workspace handoff boundaries. The GUI renders these events in its fixed status region; the CLI prints the same messages.

## Critical protocol knowledge

- Gateway CSRF and StoreFront CSRF are different tokens. Supplying the gateway meta token as StoreFront's `Csrf-Token` caused HTTP 403 in the validated deployment.
- NetScaler's JavaScript-derived `X-Ajax-Token` uses two server-provided SHA-256 states and a non-standard encoded message length. Preserve regression tests around this algorithm.
- Authentication and resource response formats vary by Citrix configuration. Treat current parsing as deployment-tested, not universal.
- Never add TLS certificate bypasses as a convenience fix.

## Process ownership

The launcher waits only until the ICA content is handed to Citrix Workspace. Platform-specific detached process flags/session creation prevent Citrix from being tied to the GUI or CLI lifetime. Any change here must be tested by closing the launcher while the VDI remains connected.
