# Contributing

1. Create a short-lived branch from `main`.
2. Run `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --all-targets`.
3. Never commit configuration files, credentials, OTP secrets, ICA files, logs, or StoreFront response dumps.
4. Use Conventional Commits where practical.
5. Open a pull request and wait for all CI jobs to pass before merging.

