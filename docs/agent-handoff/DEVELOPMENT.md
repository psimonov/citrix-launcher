# Development and delivery workflow

## Prerequisites

- Stable Rust toolchain with `rustfmt` and `clippy`.
- Git.
- Native platform development libraries required by eframe/keyring.
- Citrix Workspace only for end-to-end launch testing.
- GitHub CLI is optional; standard Git and the GitHub web UI are sufficient.

## Dependency baseline

- Use the latest stable, supported, security-patched releases verified from official sources at the time of the change.
- Review Dependabot and ecosystem security advisories; run `cargo audit` when dependency state changes.
- Do not use prerelease, nightly, deprecated, end-of-life, or known-vulnerable components without an explicit documented exception.
- Commit `Cargo.lock` and validate every upgrade across all supported operating systems through CI.
- If the newest stable release is incompatible, document the exact blocker and security implications instead of silently retaining an old version.

Windows development may use rustup and Scoop. Match the installed Rust target to its linker: MSVC Rust needs Visual C++ Build Tools; GNU Rust needs MinGW. Do not assume MinGW is required when using the MSVC target.

## Change loop

1. Start from current `main`; inspect `git status` and preserve unrelated changes.
2. Read this handoff and the affected source/tests.
3. Reproduce with sanitized evidence where possible.
4. Implement the smallest coherent change.
5. Run formatting, tests, and a build locally for the developer's current OS:

```text
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release --bins
```

6. For process, keyring, UI, or Citrix protocol changes, perform the relevant native/manual validation on the current OS.
7. Update tests, CHANGELOG when user-visible, and this handoff when assumptions or workflows change.
8. Use a concise English Conventional Commit where practical.
9. Push source code to a short-lived branch. A normal code push runs cross-platform CI but must not create a GitHub Release. Merge through a PR after all platform CI jobs pass. Direct `main` pushes were used for initial bootstrap but are not the preferred steady-state workflow.

## Test boundaries

- Unit tests must use synthetic protocol states and credentials only.
- Cross-platform compile/test coverage comes from GitHub Actions native runners.
- Successful compilation is not proof of successful connection on every Citrix deployment.
- End-to-end tests may require a fresh manual OTP and access to the authorized private environment.

## Release procedure

1. Develop, test, and build locally for the current OS, then push the source code normally.
2. Wait for the pushed code to pass Windows, macOS, and Linux CI.
3. Update version and `CHANGELOG.md`, merge the release-ready code, and ensure `main` CI is green.
4. Create and push an annotated SemVer tag such as `v0.2.0` from the release commit.
5. Tag push is the only release trigger. Do not use local tooling, an AI agent, or manual workflow dispatch to build or publish a release.
6. The Release workflow builds Windows ZIP, macOS app ZIP, DEB, and RPM on native runners.
7. Verify all assets and generated release notes in GitHub Releases.
8. Never commit generated binaries or `target/`; builds belong in Releases.

## Documentation maintenance

Treat `docs/agent-handoff/` as code. A change is incomplete if a future contributor would infer obsolete behavior from these files.
