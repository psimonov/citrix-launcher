# Troubleshooting playbook

Work from local and least-sensitive checks toward authorized live tests.

## VDI resource not found

1. Confirm the configured value is the StoreFront resource display name, not the backend machine hostname.
2. Check that the value is non-empty and has no accidental whitespace.
3. Inspect sanitized resource metadata: log only resource count, redacted IDs, and normalized display-name comparisons.
4. Verify the correct StoreFront portal/session was established.
5. Check whether the deployment returns resources in a changed JSON/XML structure.
6. Never solve this with a hard-coded list of aliases or internal machine names.

## HTTP 403 after gateway authentication

1. Identify whether failure is at Gateway or StoreFront by endpoint path and status only.
2. Preserve separate gateway `X-csrftoken` and StoreFront `Csrf-Token` values.
3. Ensure `Home/Configuration` establishes StoreFront cookies before `Resources/List`.
4. Recalculate `X-Ajax-Token` from the exact HTTP method, path, and body.
5. Preserve cookie jar, referer, HTTPS, requested-with, and credential/label headers.
6. Do not disable TLS validation.

## Authentication fails

1. Verify system time; TOTP uses a 30-second window.
2. Confirm manual OTP is exactly six ASCII digits or the seed is valid Base32.
3. Ask for a fresh OTP only immediately before a live attempt.
4. Detect additional authentication challenges rather than retrying credentials repeatedly.
5. Avoid account lockout: stop repeated live attempts when the response is ambiguous.

## Citrix not found or does not launch

1. Run `citrix-vdi-cli detect-citrix`.
2. Verify the configured path exists and points to the platform ICA client.
3. Confirm the downloaded content is actually ICA, not an HTML/XML error page.
4. Check filesystem permissions for the application data directory.
5. Test that Citrix remains active after closing both GUI and CLI launcher processes.

## Platform build failure

- Windows: check Rust target/linker pairing (MSVC vs GNU).
- Linux: install DBus, X11/Wayland, keyboard, GL, and `pkg-config` development packages used in CI.
- macOS: build the `.app` on a native macOS runner; do not expect reliable native app signing/bundling from Windows.
- Compare local failures with `.github/workflows/ci.yml`, which is the canonical clean-runner recipe.

## Safe evidence format

Record timestamps, application version, OS, stage name, HTTP status, endpoint path template, content type, and a manually redacted structural error. Do not record request bodies, headers containing values, raw responses, or ICA data.
