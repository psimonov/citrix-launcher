#!/bin/sh
set -eu

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
source_app="$script_dir/citrix-vdi-launcher.app"
install_dir="$HOME/Applications"
installed_app="$install_dir/Citrix VDI Launcher.app"

if [ ! -d "$source_app" ]; then
    echo "Не найдено приложение: $source_app" >&2
    exit 1
fi

mkdir -p "$install_dir"
if [ -e "$installed_app" ]; then
    mkdir -p "$HOME/.Trash"
    backup="$HOME/.Trash/Citrix VDI Launcher $(date +%Y%m%d-%H%M%S).app"
    echo "Предыдущая версия перемещается в Корзину: $backup"
    mv "$installed_app" "$backup"
fi

ditto "$source_app" "$installed_app"

# Remove only the download quarantine marker from this application. This does
# not disable Gatekeeper or System Integrity Protection globally.
xattr -dr com.apple.quarantine "$installed_app"
codesign --verify --deep --strict "$installed_app"

echo "Citrix VDI Launcher установлен: $installed_app"
open "$installed_app"
