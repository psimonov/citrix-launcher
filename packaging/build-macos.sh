#!/usr/bin/env sh
set -eu

if [ "$(uname -s)" != "Darwin" ]; then
    echo "macOS bundles must be built on macOS" >&2
    exit 1
fi

TARGETS="${MACOS_TARGETS:-aarch64-apple-darwin x86_64-apple-darwin}"
export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-11.0}"

for target in $TARGETS; do
    rustup target add "$target"
    cargo build --release --bins --target "$target"
done

APP="target/release/bundle/osx/citrix-vdi-launcher.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

set -- $TARGETS
first_target="$1"
if [ "$#" -eq 1 ]; then
    cp "target/$first_target/release/citrix-vdi-launcher" "$APP/Contents/MacOS/citrix-vdi-launcher"
    cp "target/$first_target/release/citrix-vdi-cli" "$APP/Contents/MacOS/citrix-vdi-cli"
else
    gui_inputs=""
    cli_inputs=""
    for target in $TARGETS; do
        gui_inputs="$gui_inputs target/$target/release/citrix-vdi-launcher"
        cli_inputs="$cli_inputs target/$target/release/citrix-vdi-cli"
    done
    # The values above are repository-controlled paths without whitespace.
    # shellcheck disable=SC2086
    lipo -create $gui_inputs -output "$APP/Contents/MacOS/citrix-vdi-launcher"
    # shellcheck disable=SC2086
    lipo -create $cli_inputs -output "$APP/Contents/MacOS/citrix-vdi-cli"
fi

cp assets/icons/citrix-vdi-launcher.icns "$APP/Contents/Resources/citrix-vdi-launcher.icns"
version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
sed "s/@VERSION@/$version/g" packaging/macos/Info.plist.in > "$APP/Contents/Info.plist"

signing_identity="${MACOS_SIGNING_IDENTITY:--}"
timestamp_option="--timestamp"
if [ "$signing_identity" = "-" ]; then
    timestamp_option="--timestamp=none"
fi
codesign --force --options runtime "$timestamp_option" \
    --sign "$signing_identity" "$APP/Contents/MacOS/citrix-vdi-cli"
codesign --force --options runtime "$timestamp_option" \
    --sign "$signing_identity" "$APP"
codesign --verify --deep --strict --verbose=2 "$APP"
lipo -archs "$APP/Contents/MacOS/citrix-vdi-launcher"
lipo -archs "$APP/Contents/MacOS/citrix-vdi-cli"
echo "$APP"
