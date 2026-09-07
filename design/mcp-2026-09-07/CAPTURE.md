# Rendering the Claude Design canvases, and capturing them

**Imported from the live project 2026-09-07** via the `claude_design` MCP
(`DesignSync`), project `067f80e7-dbb7-4492-8e69-96aaa8050a4d`
("UI mockups planning", owner Vincent). These are the owner's three named
form-factor canvases plus the libraries they import.

| File | Form factor | Owner's word |
|---|---|---|
| `Cartalith DCC Environment.dc.html` | **PC** (and tablet frames) | *"That was pc"* |
| `Cartalith Tablet.dc.html` | **Tablet** | *"Implement: Tablet"* |
| `Cartalith Android.dc.html` | **Smartphone** | *"Implement: Smartphone"* |
| `support.js` | the **dc-runtime** — parses and mounts a `.dc.html` | |
| `cartalith-dcc-parts.js` | heavy method bodies for DCC Environment + Tablet | |
| `landmark-glyphs.js` | `window.LM_GLYPHS`, needed by all three — **fetched 2026-09-07**, 20 232 B, 49 glyph keys | |

## The thing that unblocked this

A `.dc.html` from this project is **a complete HTML document**, not a fragment:
it carries `<script src="./support.js">` in its own `<head>` and the design as
`<x-dc>` in its body. **So it renders standalone in a browser with its sibling
libraries beside it** — no canvas editor, no seeding, no artboard iframes.

That matters because **the canvas EDITOR does not render in headless Chrome.**
Five configurations were tried against
`round3-corrected/cartalith-round3-redrawn.html` — `--headless=new`,
`--headless=old`, `--virtual-time-budget` up to 120 s,
`--run-all-compositor-stages-before-draw`, and `--disable-web-security` with
site isolation off. Every one produced the editor chrome with its artboards
stuck on *"Loading artboard…"*; `--headless=old --timeout` produced a blank
page. The artboards are nested sandboxed preview iframes and their ready
handshake never completes under headless. **Do not spend time re-trying that.**

## The recipe that works

1. **Keep the `.dc.html` and its libraries in one directory** (this one). The
   `src="./…"` references are relative and resolve as siblings.
2. **Launch HEADED Chrome in app mode** — there is a real display on this
   machine:
   ```
   chrome --new-window --window-size=1600,1100 --window-position=0,0 \
          --user-data-dir=<scratch>/chprof --no-first-run \
          --no-default-browser-check --allow-file-access-from-files \
          --app="file:///C:/…/design/mcp-2026-09-07/Cartalith%20DCC%20Environment.dc.html"
   ```
   `--app` drops the browser UI, so the capture is the page and nothing else.
3. **Capture the WINDOW, not the screen**, with
   `<scratch>/capture_window.ps1 -TitleLike "DCC Environment" -OutPath …`.
   It resolves the window by title, uses `DwmGetWindowAttribute`
   (`DWMWA_EXTENDED_FRAME_BOUNDS`) so the drop shadow is excluded, and copies
   only that rectangle.

   **Capture the window and never the primary display.** A full-screen grab
   picks up whatever else the owner has open, and these captures get attached
   to reports and committed. The first attempt in this session grabbed the
   whole desktop and caught the owner's other windows — that is the reason this
   script exists.

## Two things that will bite

- **A Windows Firewall prompt can steal focus and overlay the capture.** It is
  raised by a local `python -m http.server` binding a socket. **The HTTP server
  is not needed** — `file://` works for these standalone documents. Don't start
  one. If a prompt is already up, close it before capturing.
- **`landmark-glyphs.js` is here now** (fetched 2026-09-07 with `DesignSync`,
  20 232 B, 49 keys; all three canvases report `LM_GLYPHS = 49` at runtime and
  the glyphs draw in every capture). Keep it: `cartalith-dcc-parts.js`
  *references* that global but does not define it, so deleting the file blanks
  every landmark icon.

## RESOLVED: there is only ONE Android canvas

This section previously said two different Android canvases existed and told
the next reader to diff them. **They are the same document.** Two lanes and the
re-check each established it independently:

| Path | Bytes | CR bytes | sha256 after stripping CRs |
|---|---|---|---|
| `design/Cartalith-Android-2026-09-07.dc.html` | 170 327 | **1 491** | `c1e4d5ec…` |
| `design/mcp-2026-09-07/Cartalith Android.dc.html` | 168 836 | **0** | `c1e4d5ec…` |

**170 327 − 168 836 = 1 491 = the line count.** The delta is exactly the CRLF
line endings; normalised, the two hash identically and `diff` is empty. Both
were rendered and pixel-diffed across six matched states: **0 differing pixels
on all six.**

**The framing was mine and it was wrong** — I compared sizes and hashes and
concluded "larger and different content" without normalising line endings, when
a byte delta that exactly equals the line count is the signature of precisely
that. **There is no side to pick.**

## Capture with CDP, not with the window-capture script

`capture_window.ps1` is kept because its *reason* is right — a capture must
contain only the thing under test, never the owner's other windows. But
**Chrome DevTools Protocol `Page.captureScreenshot` with an explicit clip
satisfies that reason strictly better, and lifts a hard ceiling:**

This machine has a single **1680×1050** screen (work area 1680×1002, ~971 px of
usable client height). **Six of the fourteen canvas frames cannot be
photographed at 1:1 as a window at all** — 1920×1080, 2560×1600, 1600×2560,
800×1280 and 900×1440 all exceed it. The window recipe would have silently
produced scaled or cropped references for nearly half the work.

**Size the viewport so the canvas's own zoom resolves to 1.0** before clipping:
the PC canvas uses `min(1, (innerWidth-36)/frameWidth)` and the tablet canvas
`min(1, (iw-40)/fw, (ih-150)/fh)`. At 1.0 the mapping is **1 canvas px = 1 dp =
1 Godot px = 1 captured px** (this desktop is 96 dpi at 100%, so no dpi
conversion enters anywhere).

## The shipped side: SubViewport probes, not a window

For the same ceiling reason, **"Godot windowed at a matching viewport" is not
possible for the large frames.** Use the committed `SubViewport` probes, which
are not clamped to the desktop:
`_ph412_probe.tscn -- --vp WxH --tag T [--force-touch]` for any frame, and
`_emptyphone_probe.tscn -- --force-touch --vp 412x892 [--dismiss|--world]
[--tab map|gen|plan|more]` for phone states.

**Which composition a frame lands in is decided by
`DccShell._compute_layout_mode()`**, and aspect alone decides it off a real
device: `_phone` needs `min/max < 0.6`; `is_tablet()` is `_touch and not
_phone`; `_is_tablet_sized()` returns false off-device; `_touch` is false
without `--force-touch`. So 1920×1080 and 1366×768 take **no flag** (desktop),
while every tablet and phone frame needs `--force-touch`.

**Two traps worth carrying.** The shipped desktop and tablet captures carry **no
world** — chrome, docks, rail, type and colour are all present, only the map is
empty. And `_ph412_probe` hides `open_project_dialog` but **not**
`phone_project_picker`, so at phone sizes all three of its shots are the picker;
only its first shot is the viewport.
