# Security and privacy rules

These rules apply to code, tests, commits, issues, pull requests, Actions output, screenshots, AI prompts, and handoff documents.

## Never commit or disclose

- Real gateway or StoreFront URLs and internal host names.
- Usernames, passwords, OTP values, or TOTP seeds/QR payloads.
- Cookies, CSRF values, `StateContext`, authentication headers, or session identifiers.
- Downloaded ICA files or their contents.
- Raw portal HTML/XML/JSON, JavaScript captures, packet captures, or diagnostic logs from a real environment.
- Machine-specific configuration files and keyring/DPAPI material.

Use unmistakably fictional values such as `https://gateway.example/`, `user`, `MY-DESKTOP`, and `BASE32SECRET` in documentation and tests.

## Credential storage

- Windows: DPAPI-protected blobs are stored in the per-user JSON configuration.
- macOS: secrets are stored in Keychain; JSON stores only a marker.
- Linux: secrets are stored through Secret Service; JSON stores only a marker.
- OTP values are ephemeral and must not be saved.
- TOTP seeds intentionally allow same-device MFA automation. This is an explicit usability/security tradeoff accepted by the owner; do not silently broaden access or exportability.

## Diagnostics

- Prefer status codes, endpoint path templates, content types, field names, and redacted structural summaries.
- Redact values before saving or displaying diagnostics.
- Never print passwords, OTP/TOTP seeds, cookies, request bodies, ICA content, or complete server responses.
- Live tests require explicit operator authorization and should request an OTP only immediately before use.

## Repository history status

The repository history was rebuilt after a full-tree audit to remove early environment-specific defaults and personal commit metadata. Continue to audit both the current tree and all reachable Git objects before changing visibility or widening access.

## Dependency and release security

- Keep `Cargo.lock` committed.
- Review Dependabot updates and CI results before merging.
- Release only from an immutable `v*` tag whose commit passed CI.
- Do not add telemetry or external reporting without explicit owner approval.
