# The mockup's source and tests

The **`Shell` component** is built from the parts in this folder. Its frame is `../Shell.dc.html`, and it draws the
phone, the tablet and the desktop. The other frames are written by hand, and you edit them directly: `Desktop`,
`Live`, `Main`, `Tablet` and `Rows`.

This folder was kept in the repository on 2026-11-01. Until then it lived only in a session's scratch folder, so the
mockup could be edited but neither rebuilt nor re-tested.

| Path | What it is |
|---|---|
| `build/shell.tpl.html` | The frame's template. The script goes where `/*__SCRIPT__*/` stands |
| `build/d_*.js`, `build/c_class.js` | The component's script, in parts, in the order `tools/assemble.py` lists |
| `engine/world.js`, `engine/render.js` | The stand-in world the mockup draws: noise terrain, biomes and shading. It is not Cartalith's engine |
| `tools/assemble.py` | Joins the parts into the template |
| `tests/*.js` | The render tests, in Playwright |

## Rebuild

From this folder:

```sh
mkdir -p canvas/project
python3 -I tools/assemble.py "$PWD"
node --check build/_shell_check.js
cp canvas/project/Shell.dc.html ../Shell.dc.html
rm -rf canvas build/_shell_check.js
```

- **The assembler writes two files:**
  - `canvas/project/Shell.dc.html`, the frame;
  - `build/_shell_check.js`, the script alone, wrapped so that `node --check` can parse it.

  Both are generated, so neither is kept.
- **Checked 2026-11-01:** rebuilding from this folder reproduces the committed `../Shell.dc.html` byte for byte.
- **Edit the parts, not the assembled frame.** A rebuild overwrites the frame.

## Test

**Serve the frames, with `support.js` beside them, on port 8770.** That is the test scripts' default, and `ds13.js`
and `ea.js` also read `BASE`:

```sh
mkdir -p /tmp/mockup-serve
cp ../*.dc.html ../canvas.json ../../support.js /tmp/mockup-serve/
python3 -I -m http.server 8770 --directory /tmp/mockup-serve &
node tests/ds13.js
```

**What each test does:**
- **These assert, and exit 1 on a failed check or a page error:**
  - `ds13.js`: the touch map's control row (`MOBILE_SHELL_PROPOSAL.md` §2.5), with Ruling EB's thumb zoom and zoom
    step;
  - `ea.js`: Ruling EA's three small designs, with Ruling EB's 3D height slider and Scale with the map;
  - `dz.js`: Ruling DZ's tool-options bar;
  - `targets.js`: every visible target is at least 44 × 44 dp.
- **These drive the shell and print what they found,** with every page error. They check nothing on their own, so
  compare their output before and after a change:
  - `desk.js`: the desktop shell;
  - `vault.js`: the vault and its editor;
  - `qr.js`: the quick bar and the cartwheel;
  - `tl.js`: the timeline strip;
  - `dy.js`, `dy_touch.js`, `dy_more.js`: Ruling DY.

**Where captures go:**
- `ds13`, `ea`, `dz` and the `dy` tests write theirs into `cartalith-native/docs/evidence/`, under
  `mobile-shell-2026-10-31/` and `desktop-shell-2026-10-31/`. `ds13.js` also reads `SHOT_DIR`.
- `desk`, `vault`, `qr` and `tl` write working images to `SHOT_DIR`, or to the system's temp folder.

## A hazard in the runtime

**A throw in the component's constructor is not a page error.** The Design runtime (`support.js`) catches it and
logs it as a console error, and the frame draws almost nothing.
- Measured 2026-11-01, with a throw added to a scratch copy: no `pageerror`, one console error, and 10 elements drawn
  against 102.
- So a test that listens only for `pageerror` passes a blank frame.
- Listen for console errors as well, and assert that the shell drew.
  - `ea.js` does both: its first check is that the bottom bar exists.
  - `ds13.js` and `dz.js` listen for console errors, and check content that a blank frame lacks.
  - The diagnostic tests print what they find, so a blank frame shows up only as missing output.
