#!/usr/bin/env sh
set -eu
cargo build --release --bins
cargo install cargo-bundle --locked
cargo bundle --release --bin citrix-vdi-launcher
APP="target/release/bundle/osx/citrix-vdi-launcher.app"
cp target/release/citrix-vdi-cli "$APP/Contents/MacOS/citrix-vdi-cli"
echo "$APP"
