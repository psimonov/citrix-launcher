# Development handoff

This directory is the canonical, vendor-neutral context package for continuing development from a clean clone. It is intended for humans and stateless AI coding agents. Repository documentation is written in English; the current product UI and CLI messages are Russian.

## Required reading order

1. [PRODUCT.md](PRODUCT.md) — goals, behavior, and boundaries.
2. [ARCHITECTURE.md](ARCHITECTURE.md) — code map and connection lifecycle.
3. [SECURITY.md](SECURITY.md) — non-negotiable data-handling rules.
4. [DEVELOPMENT.md](DEVELOPMENT.md) — setup, validation, Git, and release workflow.
5. [CONFIGURATION.md](CONFIGURATION.md) — portable configuration model.
6. [DECISIONS.md](DECISIONS.md) — decisions and their rationale.
7. [TROUBLESHOOTING.md](TROUBLESHOOTING.md) — diagnostic sequence.
8. [BACKLOG.md](BACKLOG.md) — known gaps and suggested next work.

## Fast session bootstrap

1. Read the documents above and inspect `git status` and recent commits.
2. Never invent or request production secrets before exhausting code, tests, sanitized diagnostics, and public Citrix documentation.
3. Ask the operator for a current OTP only at the exact point an authorized live authentication test needs it. Never persist or repeat that OTP.
4. Make the smallest safe change, preserve unrelated work, and validate proportionally.
5. Update this handoff whenever behavior, assumptions, packaging, or operating procedures change.

## Current baseline

- Version: `0.1.0`.
- Default branch: `main`.
- Repository visibility: private.
- GUI and CLI share one Rust library and one OS-standard configuration.
- Direct network authentication has been proven against one Citrix Gateway/StoreFront deployment without opening a browser.
- CI builds and tests on Windows, macOS, and Linux.
- Release artifacts are Windows ZIP, macOS app ZIP, Debian package, and RPM package.

No environment-specific values are intentionally recorded here. Obtain them from the operator and store them only through the application configuration mechanisms.
