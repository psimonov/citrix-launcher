# Configuration model

The application creates a per-user `config.json` automatically.

- Windows: `%APPDATA%\CitrixVdiLauncher\config.json`
- macOS: `~/Library/Application Support/CitrixVdiLauncher/config.json`
- Linux: `${XDG_CONFIG_HOME:-~/.config}/citrix-vdi-launcher/config.json`

## Fields

| Field | Purpose | Sensitive | Default |
|---|---|---:|---|
| `storefront_url` | HTTPS Citrix Gateway/StoreFront entry URL | deployment-sensitive | empty |
| `vdi_name` | Selected desktop; the default shown before the first sign-in | deployment-sensitive | empty |
| `known_desktops` | Desktops offered at the last sign-in: resource id and display name only | deployment-sensitive | empty |
| `close_after_launch` | Quit the GUI after a desktop is handed to Citrix Workspace | no | `false` |
| `username` | Account name | yes | empty |
| `citrix_path` | Citrix Workspace executable | machine-specific | auto-detected or empty |
| `protected_password` | DPAPI blob or keyring marker | yes | empty |
| `protected_secret` | DPAPI blob or keyring marker | highly sensitive | empty |

Do not copy a real configuration into the repository. DPAPI content is bound to the Windows user/machine context and is not a portable secret backup. Keyring entries are likewise external to the JSON file.

`known_desktops` is refreshed after every successful sign-in and exists so the
desktop list is available before authenticating. It deliberately stores no
`desktophostname`: that is an internal infrastructure name and must not be
written to disk. If the selected desktop disappears from StoreFront, the
selection moves to the first remaining entry.

An existing configuration from an earlier version needs no migration: its
`vdi_name` keeps working as the default selection, and the desktop list fills
itself at the next sign-in.

## CLI bootstrap example

```text
citrix-vdi-cli detect-citrix
citrix-vdi-cli config set --storefront https://gateway.example/ --vdi MY-DESKTOP --username user
citrix-vdi-cli config set --password "password" --totp-secret "BASE32SECRET"
citrix-vdi-cli desktops
citrix-vdi-cli launch MY-DESKTOP OTHER-DESKTOP
```

`connect` still launches the configured default. `launch` accepts several
desktops and signs in once for all of them; a continuously open session between
separate CLI invocations is not possible, because session cookies are secrets and
are never written to disk. `desktops` signs in and lists what StoreFront offers.

Omit `--totp-secret` to be prompted for a current OTP on each connection. `config show` reports only whether secrets exist; it must never reveal them.

## Citrix discovery

- Windows: standard Program Files and Local AppData ICA Client paths, then `PATH`.
- macOS: standard Citrix Workspace/Viewer app locations, then `PATH`.
- Linux: common `/opt`, `/usr/lib`, and `/usr/bin` ICA Client paths, then `PATH`.

Explicit user configuration wins when it points to an existing file. Detection should remain cross-platform and free of environment-specific hard-coded values.
