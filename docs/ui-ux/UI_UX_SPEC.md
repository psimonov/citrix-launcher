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
│ [icon] Citrix VDI Launcher                        3 / 7  │
│                                                        │
│ ‹  [ SELECTED     ] [ DESKTOP      ] [ DESKTOP    ]  ›  │
│    [ MY-DESKTOP   ] [ OTHER        ] [ THIRD      ]     │
│    [ Running      ] [ Not running  ] [ Not running ]    │
│                                                        │
│ [state] Status text, wrapping in a reserved area        │
│         Clear next-step guidance                        │
│                                                        │
│ One-time code                              0 of 6       │
│ [             single six-digit input             ]      │
│                                                        │
│ [ Connect — primary ]  [ Settings — secondary ]         │
└──────────────────────────────────────────────────────────┘
```

### Desktop carousel

- Cards show as many desktops as fit: minimum card width `236 px`, gap `12 px`.
- Arrows appear only when some desktops do not fit, and follow the page grid:
  card, `24 px` gutter, arrow, `24 px` frame margin, window edge. When every card
  fits, the arrows disappear and a single margin separates card from edge.
- Clicking a card only selects it. Launching is always the footer button, which
  stays disabled until a desktop is selected.
- Selection is shown by an accent border *and* the word `SELECTED`; running state
  by a check mark *and* the words `Running` / `Not running`. Color never carries
  state alone.
- The header counter `3 / 7` gives the position of the selected desktop and the
  total, in secondary text colour, right-aligned on the title line. It is hidden
  when fewer than two desktops exist.
- Left/Right move the selection when no text field has focus, and scroll the
  carousel so the selected card stays visible.
- With no desktop configured or cached, one placeholder card explains that a
  desktop must be set in the settings.

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

Each card reports whether that desktop currently has a Citrix session, refreshed
once per second from the process table, including sessions started outside the
launcher. The state is observed rather than remembered: it is never derived from
what this launcher happens to have started. After the last observed session
closes, the connection screen returns to the ready status. Session observation
must remain read-only and must not tie Citrix lifetime to the launcher process.

Between the launch and the appearance of the Citrix window the card still reads
`Not running`; progress belongs to the status region, which names the active
stage.

When "close after launch" is enabled, the window closes shortly after the final
handoff status is shown, leaving the Citrix session running.

## Design tokens

- Spacing unit: `8 px`.
- Outer page padding: `24 px`; minimum viewport may reduce it to `16 px`.
- Control height: `40 px`; OTP height: `48 px`.
- Button text: `15 px`; body: `15 px`; labels: `13 px`; title: `22 px`.
- Input text: `16 px`, vertically centered.
- Control radius: `8 px`; card radius: `12 px`.
- Border: `1 px`; focused input: `2 px` accent.
- Status region: fixed minimum height sufficient for two wrapped message lines plus one guidance line.

## Theme behavior

- Follow the OS dark/light preference in normal operation.
- Keep page, card, and input surfaces visually distinct in both themes.
- Light-theme input borders and placeholder text must remain visible without looking like active content.
- Text inputs keep a `10 px` horizontal content inset in both themes.
- Secondary buttons use a distinct surface and border, especially in the light theme, without competing with the primary action.
- Hover and pressed states change color only; button bounds, corner radius, padding, and stroke width remain constant.
- Disabled controls must remain legible while being clearly less prominent than enabled controls.
- Debug builds may use `CITRIX_UI_PREVIEW_THEME=light` with `CITRIX_UI_PREVIEW=1` for deterministic visual review. `CITRIX_UI_PREVIEW_STATE=error` provides a fixed synthetic long-error state and never loads real configuration. `CITRIX_UI_PREVIEW_STATE=settings` opens the settings screen, `CITRIX_UI_PREVIEW_DESKTOPS=<n>` renders that many synthetic desktops, and `CITRIX_UI_PREVIEW_CLOSE=1` shows the checked state of the close-after-launch option.
- Checkboxes use the input surface, border, and accent fill of the text fields: an `18 px` box with `5 px` radius, a white check mark when set, and an accent border on hover or focus. The whole label row is clickable, and Space or Enter toggles a focused checkbox.

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
- Inspect the carousel with one desktop (no arrows), with more desktops than fit
  (arrows present, one disabled at each end), and on a wide window where four
  cards fit.
- Inspect both checkbox states.
- Compare screenshots at 100% scale; inspect Windows at 125% and 150% scaling.
- No clipped text, background gaps, moving footer, accidental center alignment, or disproportionate typography.
- Do not merge or publish the UI until this checklist passes.
