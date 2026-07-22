#!/usr/bin/env sh
set -eu
cargo build --release --bins
cargo install cargo-deb cargo-generate-rpm --locked
cargo deb --no-build
cargo generate-rpm
echo "DEB and RPM packages are available under target/debian and target/generate-rpm"
