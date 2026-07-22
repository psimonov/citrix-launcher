# Configuration model

The application creates a per-user `config.json` automatically.

- Windows: `%APPDATA%\CitrixVdiLauncher\config.json`
- macOS: `~/Library/Application Support/CitrixVdiLauncher/config.json`
- Linux: `${XDG_CONFIG_HOME:-~/.config}/citrix-vdi-launcher/config.json`

## Fields

| Field | Purpose | Sensitive | Default |
|---|---|---:|---|
| `storefront_url` | HTTPS Citrix Gateway/StoreFront entry URL | deployment-sensitive | empty |
| `vdi_name` | StoreFront resource display name | deployment-sensitive | empty |
| `username` | Account name | yes | empty |
| `citrix_path` | Citrix Workspace executable | machine-specific | auto-detected or empty |
| `protected_password` | DPAPI blob or keyring marker | yes | empty |
| `protected_secret` | DPAPI blob or keyring marker | highly sensitive | empty |

Do not copy a real configuration into the repository. DPAPI content is bound to the Windows user/machine context and is not a portable secret backup. Keyring entries are likewise external to the JSON file.

## CLI bootstrap example

```text
citrix-vdi-cli detect-citrix
citrix-vdi-cli config set --storefront https://gateway.example/ --vdi MY-DESKTOP --username user
citrix-vdi-cli config set --password "password" --totp-secret "BASE32SECRET"
citrix-vdi-cli connect
```

Omit `--totp-secret` to be prompted for a current OTP on each connection. `config show` reports only whether secrets exist; it must never reveal them.

## Citrix discovery

- Windows: standard Program Files and Local AppData ICA Client paths, then `PATH`.
- macOS: standard Citrix Workspace/Viewer app locations, then `PATH`.
- Linux: common `/opt`, `/usr/lib`, and `/usr/bin` ICA Client paths, then `PATH`.

Explicit user configuration wins when it points to an existing file. Detection should remain cross-platform and free of environment-specific hard-coded values.
