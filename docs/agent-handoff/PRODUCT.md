# Product contract

## Problem

Launching a Citrix VDI through a browser-based StoreFront portal requires repetitive username/password/OTP entry and browser interaction. The launcher automates the authorized login and ICA handoff while keeping Citrix Workspace independent of the launcher process.

## Required behavior

- Provide native desktop GUI and CLI on Windows, macOS, and Linux.
- Never show or embed a browser during normal operation.
- Authenticate through network requests to Citrix Gateway and StoreFront.
- Accept a manually entered six-digit OTP when no TOTP seed is configured.
- Generate RFC 6238 TOTP locally from a configured Base32 seed when present.
- Discover the requested desktop by its configured display name and request an ICA launch.
- Start the locally installed Citrix Workspace client with the ICA file.
- Ensure the Citrix/VDI process survives launcher exit.
- Auto-detect Citrix Workspace where possible while allowing an explicit path override.
- Create configuration automatically in the conventional per-user OS location.
- Keep GUI and CLI behavior/configuration consistent.

## UX agreement

- A normal user interacts only with this application and Citrix Workspace.
- If configuration is complete and a TOTP seed exists, connecting should require one action.
- If no seed exists, the application displays its own OTP input; CLI prompts on stdin.
- Settings expose gateway/StoreFront URL, VDI display name, username, password, TOTP seed, and Citrix executable path.
- Do not reintroduce browser settings, WebView settings, or alternative VDI-name lists.
- The product currently uses Russian user-facing text and follows the system theme as far as the GUI toolkit supports it.

## Packaging contract

- Windows: ordinary standalone `.exe` binaries distributed in ZIP.
- macOS: native `.app` bundle distributed in ZIP.
- Linux: native `.deb` and `.rpm`; no Snap, Flatpak, or AppImage.
- Rust/build tools are not runtime requirements.
- Citrix Workspace remains a required external application.

## Non-goals

- Bypassing MFA, access controls, device posture, or organizational policy.
- Automating enrollment or extracting a TOTP seed from a mobile authenticator.
- Bundling or redistributing Citrix Workspace.
- Supporting arbitrary Citrix deployments without deployment-specific validation.
- Keeping the launcher alive for the lifetime of a Citrix session.
