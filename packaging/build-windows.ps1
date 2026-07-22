$ErrorActionPreference = 'Stop'
cargo build --release --bins
$destination = Join-Path $PSScriptRoot '..\dist\windows'
New-Item -ItemType Directory -Force $destination | Out-Null
Copy-Item (Join-Path $PSScriptRoot '..\target\release\citrix-vdi-launcher.exe') $destination
Copy-Item (Join-Path $PSScriptRoot '..\target\release\citrix-vdi-cli.exe') $destination
Write-Output $destination
