# Contributing

1. Create a short-lived branch from `main`.
2. Run `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --all-targets`.
3. Never commit configuration files, credentials, OTP secrets, ICA files, logs, or StoreFront response dumps.
4. Use Conventional Commits where practical.
5. Open a pull request and wait for all CI jobs to pass before merging.

## macOS release security

The project does not have an Apple Developer certificate. macOS releases use an
ad-hoc signature and therefore cannot pass Gatekeeper as an identified and
notarized developer. The release ZIP includes `README-RU.txt` and the narrowly
scoped `install-macos.command` helper. It installs into `~/Applications`,
removes `com.apple.quarantine` only from this app, verifies its ad-hoc
signature, and does not disable Gatekeeper or System Integrity Protection
globally.
