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
| `landmark-glyphs.js` | **NOT YET FETCHED** — `window.LM_GLYPHS`, needed by all three | |

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
- **`landmark-glyphs.js` is not on disk yet.** Fetch it with `DesignSync`
  (`method: get_file`, `path: landmark-glyphs.js`) and write it here. It
  defines `window.LM_GLYPHS`; `cartalith-dcc-parts.js` *references* that global
  but does not define it, so without the file every landmark icon is missing.

## An unresolved discrepancy — do not guess which canvas is authoritative

There are **two different Android canvases** in this repository:

| Path | Bytes | sha256 |
|---|---|---|
| `design/mcp-2026-09-07/Cartalith Android.dc.html` (live project) | 168 836 | `c1e4d5ec…` |
| `design/dcc-environment-2026-08-31/Cartalith Android.dc.html` | 168 836 | `c1e4d5ec…` |
| `design/Cartalith-Android-2026-09-07.dc.html` (owner attachment) | 170 327 | `5f626c41…` |

The live project's file is **byte-identical to the Aug-31 copy**. The 2026-09-07
attachment — which every GENERATE-screen decision so far was built against, and
which `ANDROID_UI_SPEC.md` cites throughout — is **a different, larger file**.

**Diff them before building, and say which one each conclusion rests on.** The
owner named the project file, and an owner instruction is newer than a vendored
copy; but the attachment is newer by date and is what the shipped work matched.
This is exactly the kind of ambiguity that has produced a wrong build from a
right-looking anchor in this project before.
