# OnePlus Stylo 2 — BLE protocol reference

Confirmed on-device (RE of the owner's own device for interoperability). What an ordinary,
non-root app can read/do over BLE. The pen's **drawing input** (x/y, pressure, tilt, barrel
button) does **not** come over BLE — it's standard Android `MotionEvent`; see `stylus-input.md`.

## GATT profile

| Service | UUID | Use |
|---|---|---|
| **Command / INTERFACE_COM** | `58480000-5a6a-1122-3300-004095112200` | commands + their responses |
| Nordic UART (NUS) | `6e400001-b5a3-f393-e0a9-e50e24dcca9e` | text events (film/charge) |
| Double-click | `180938ac-2a69-11ec-8d3d-0242ac130003`* | double-tap event |
| State / notify | `18093046-2a69-11ec-8d3d-0242ac130003`* | pen enable/state echo |
| Battery, charge, sensitivity, log | other `…-2a69-11ec-…` | notify |
| Device info | `0000180a-…` | SIG |

\* characteristic UUIDs shown; they sit inside the vendor service group.

**Key characteristics**
- **`58480001-5a6a-1122-3300-004095112200`** — WRITE | WRITE_NR | NOTIFY. Write commands here; the
  pen's responses arrive as notifications on the **same** characteristic. Use **Write Request**
  (with response) — that's what ipemanager uses.
- **`180938ac-…`** — NOTIFY. Double-tap events.
- **`6e400003-…`** (NUS TX) — NOTIFY. Text events (`film …`, `chg evt …`).

Subscribe to notify characteristics after discovery. Two HID characteristics (`0x2a4d`-area)
require `BLUETOOTH_PRIVILEGED` and can be skipped — they are not part of the pen features.

## Command frame

`2C <cmdId> <len> <payload…>` written to `58480001`. The response is a notification on `58480001`
echoing `2C <cmdId> <len> <data…>`.

| Purpose | Write | Response (observed) |
|---|---|---|
| Firmware version | `2C 06 01 05` | `2C 06 01 05 4D 45 03 00 12` → **`4D45.03.00.12`** |
| Device name | `2C 26 01 0B` | `2C 26 01 0F "OnePlus Stylo 2"` |
| Battery % | `2C 08 01 01` | `2C 08 01 01 64` → **0x64 = 100** |
| SDK/proto version | `2C 25 01 03` | `2C 25 01 03 "1.0"` |
| Status | `2C 01 01 01` | `2C 01 01 01 00` |
| Pencil enable / disable | `2C 90 01 01 01` / `…00` | state echoed on `18093046` |
| Set vibration type | `2C 94 02 <type> <level+1>` | config only — see note |
| Start vibration | `2C 92 01 02 FF 02` | accepted, **no buzz** — see note |

## Events (notifications — what Cartalith reacts to)

| Event | Characteristic | Payload | Meaning |
|---|---|---|---|
| **Double-tap** | `180938ac-…` | `01` | user double-tapped the pen |
| **Slide / film contact** | `6e400003-…` | ASCII `"film <pos>"` | finger/contact position on the capacitive film: `0x05` one end … `0xFF` other end; **direction = delta** over time |
| Charge event | `6e400003-…` | ASCII `"chg evt …"` | charging status |
| Battery change | battery notify char | byte | live battery % |
| Pen enabled/connected | `18093046-…` | `2C 90 01 01 01` | periodic state echo |

**Barrel button** is **not** a BLE event — it arrives as a standard Android key/`MotionEvent`
button when a digitizer is present.

## The haptic — not reachable over BLE

`setVibrationType` (`2C 94`) only **configures** the pattern/strength. `startVibration` (`2C 92`)
is accepted by the pen but **does not buzz** — verified exhaustively, including a fully-privileged
rooted session. The writing-feedback motor is driven by the touch controller (touch node 38,
`OPLUS_COMPONENT_SAFE`, signature-level), not BLE. An ordinary app **cannot** trigger the buzz; the
only sanctioned route is an Oplus-issued OCS `IPE_PENCIL_CLIENT` authCode. Cartalith can configure
feel but not fire it.
