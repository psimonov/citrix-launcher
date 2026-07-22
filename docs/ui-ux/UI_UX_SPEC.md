# UI/UX specification

This document is the acceptance contract for the launcher UI. Business and network behavior are outside its scope.

## Product principles

1. One primary task per screen.
2. One primary action per screen.
3. Every state answers: what just happened, what is happening now, and what can the user do next?
4. Controls use native keyboard behavior; custom visuals must never break caret, paste, focus, Tab, Shift+Tab, or Backspace.
5. Content must fit the minimum supported client area without resizing the window.
6. Long and unexpected text wraps inside reserved regions and never changes the overall page geometry unexpectedly.
7. Color supplements text and is never the only state indicator.

## Supported viewport contract

- Default client size: `650 × 590`.
- Minimum client size: `570 × 500`.
- No horizontal scrolling.
- Primary action is always visible.
- Settings fields scroll; settings header and footer remain fixed.
- Switching between connection and settings views does not change the painted root height or background.

## Screens

### Connect

```text
┌──────────────────────────────────────────────────────────┐
│ [icon] Citrix VDI Launcher                              │
│                                                        │
│ Desktop                                                 │
│ MY-DESKTOP                                              │
│ [state] Status text, wrapping in a reserved area        │
│         Clear next-step guidance                        │
│                                                        │
│ One-time code                              0 of 6       │
│ [             single six-digit input             ]      │
│                                                        │
│ [ Connect — primary ]  [ Settings — secondary ]         │
└──────────────────────────────────────────────────────────┘
```

The OTP is one real text-edit control, not six independent controls. It accepts digits only, limits input to six characters, keeps native caret/selection/paste/Backspace behavior, displays entered digits, and reports completion separately. Decorative slot rendering may be added only if it is painted behind the same single editor and passes keyboard acceptance tests.

### Settings

```text
┌──────────────────────────────────────────────────────────┐
│ [icon] Citrix VDI Launcher                              │
│ Settings                                      [ Back ]   │
│ ┌──────────────── scrollable form ────────────────────┐ │
│ │ Label                                                │ │
│ │ [ value                                            ] │ │
│ │ ...                                                  │ │
│ └──────────────────────────────────────────────────────┘ │
│ [ Save — primary ]  [ Detect Citrix — secondary ]        │
└──────────────────────────────────────────────────────────┘
```

## State model

| State | Message purpose | Next action | Primary action |
|---|---|---|---|
| Not configured | Explain missing setup | Open settings | Settings |
| Ready, manual OTP | Request six digits | Enter code | Connect disabled until complete |
| Ready, automatic TOTP | Confirm automatic code | Connect | Connect enabled |
| Connecting | Name current stage | Wait | Disabled with progress |
| Success | Confirm ICA handoff | Work in Citrix | Connect available for retry |
| Recoverable error | Explain failure in plain language | Correct input/settings or retry | Contextual |
| Settings saved | Confirm save | Return or connect | Save remains available |

During a connection attempt, the status region names the active protocol stage. The connect button and status marker show progress, while OTP editing and navigation to settings are disabled. During native Citrix executable selection, the settings form, navigation, and footer actions are disabled and the browse button shows progress.

After an observed Citrix desktop session closes, the connection screen returns to the ready status. Session observation must remain read-only and must not tie Citrix lifetime to the launcher process.

## Design tokens

- Spacing unit: `8 px`.
- Outer page padding: `24 px`; minimum viewport may reduce it to `16 px`.
- Control height: `40 px`; OTP height: `48 px`.
- Button text: `15 px`; body: `15 px`; labels: `13 px`; title: `22 px`.
- Input text: `16 px`, vertically centered.
- Control radius: `8 px`; card radius: `12 px`.
- Border: `1 px`; focused input: `2 px` accent.
- Status region: fixed minimum height sufficient for two wrapped message lines plus one guidance line.

## Keyboard acceptance

- Tab order follows visual order.
- Enter submits only when the primary action is enabled.
- OTP supports click placement, Left/Right, Home/End, selection, Backspace/Delete, and six-digit paste.
- Opening settings places focus predictably without stealing subsequent input.
- Escape or the Back button returns to Connect without saving.

## Visual acceptance checklist

- Inspect light and dark themes.
- Inspect `650 × 590` and `570 × 500`.
- Inspect empty, ready, connecting, success, short error, and 300-character error states.
- Compare screenshots at 100% scale; inspect Windows at 125% and 150% scaling.
- No clipped text, background gaps, moving footer, accidental center alignment, or disproportionate typography.
- Do not merge or publish the UI until this checklist passes.
