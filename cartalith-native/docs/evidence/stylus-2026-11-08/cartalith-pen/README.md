# Cartalith ↔ OnePlus Stylo 2 — integration pack

A self-contained reference for talking to the **OnePlus Stylo 2** from an ordinary, non-root
Android app (for the Cartalith project). Built from interoperability research on the owner's own
devices. Copy this folder into Cartalith; nothing here depends on the rest of the research repo.

## TL;DR — what you actually need

| You want… | How | Needs our RE? |
|---|---|---|
| **Pen drawing** (x/y, **pressure**, **tilt**, hover, barrel button) | **Standard Android input** — `MotionEvent` axes. See `stylus-input.md`. | ❌ No — free for any app |
| **Battery %, firmware version, pen name** | BLE query (`Stylo2Pen.kt`) | ✅ (BLE protocol in `protocol.md`) |
| **Pen shortcut / double-click button events** | BLE notifications (`Stylo2Pen.kt`) | ✅ |
| **Configure** the writing-feedback vibration pattern/strength | BLE `setVibrationType` | ✅ (configures only) |
| **Trigger** the haptic buzz from your app | **Not possible for an ordinary app** | ⛔ signature-walled (see below) |

## The one wall you can't cross

The Stylo 2's **writing-feedback vibration** is driven by a privileged touch-controller path
(`OPLUS_COMPONENT_SAFE`, a signature-level permission), **not** by BLE. Over BLE you can *configure*
the pattern/strength (`setVibrationType`), but there is **no command an unprivileged app can send to
make the motor buzz** — we verified this exhaustively on-device. The only sanctioned way to drive
the haptic is an **Oplus-issued OCS `IPE_PENCIL_CLIENT` authCode** (the vendor authorises your app).
So: Cartalith can read the pen, show battery, react to its button, and configure feel — it just
can't fire the buzz itself.

## Files

- **`README.md`** — this overview. ✅
- **`stylus-input.md`** — *(coming next)* the standard Android way to read pen pressure/tilt/buttons.
- **`protocol.md`** — *(coming next, after the on-device button/tap/slide capture)* the pen's BLE
  GATT profile and `0x2C` command set.
- **`Stylo2Pen.kt`** — *(coming next)* a drop-in, non-root BLE client: connect to the bonded pen,
  query battery/firmware/name, receive button/tap notifications, configure vibration. Modern BLE
  (minSdk 33). Needs the runtime permission `BLUETOOTH_CONNECT`.

> **Status:** WIP. The protocol reference and client code are finalized once we capture the exact
> bytes for the pen's **button / double-click / slide-tap** events (the notable functions for
> Cartalith) — either from the inspector's live feed on the Pad or a clean HCI snoop from the
> rooted 6T.

## Legal note

This describes the owner's own devices, analysed for interoperability (EU Software Directive
2009/24/EC Art. 6 / NL Auteurswet Art. 45m). It documents the protocol for interop; it does not
include or enable any signature forgery, key extraction, or circumvention of a protection measure.
