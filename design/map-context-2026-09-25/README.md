# Map context prototypes, 2026-09-25

Interactive mockups of the right-click **context card** and the 8-slot **tool
ring** that `MAP_CONTEXT_SCOPE.md` proposes. They are for all three form factors
and were built at the owner's request (*"generate some mockups that are fully
interactive … the focus would be the mouse/wheel interface"*). The owner
reviewed them the same day. Their feedback was that contrast and the back
button were hard to see, so this copy is the refined second revision.

**These are prototypes, not the shell.** Nothing here is wired to the engine.
The map, the objects on it, the elevation and biome readouts and the object
names are hand-made placeholders. Rows such as *Open city layout* or *Journey
from here* only post a status message. `MAP_CONTEXT_SCOPE.md` §4.3 says which
engine call would back each row. What the port has actually built is in
`cartalith-native/docs/STATUS.md`, group *Map context*.

| File | Frame | What to try |
|---|---|---|
| `Main.dc.html` | Desktop, 1280 × 800, mouse | **RMB click** → card · **RMB drag** → ring (a fast flick selects before it draws) · **RMB hold** → ring + card · **hold Q** → ring at the cursor · **Ctrl+RMB drag** → brush radius/strength · push past a `▸` slot → sub-ring · MMB pans · Esc |
| `Tablet.dc.html` | Tablet, 1194 × 834, touch or pen | **Hold 0.5 s** → ring + card at the finger, with a progress ring and a pulse · keep the finger down and **slide + lift** to select · **lift in place** keeps both open · drag pans (or strokes, with a sculpt tool armed) · the pen barrel or RMB opens the card · the top bar toggles handedness (the card moves away from the palm) |
| `Phone.dc.html` | Phone, 390 × 844, touch | **Long-press 0.48 s** → sample pin + peek sheet with four action chips · drag the handle up for the full card · **drag the pin** to change the subject · the **tool pill**: tap toggles armed ↔ Inspect, press and slide up for the **thumb fan** |
| `canvas.json` | — | The canvas index: frame positions and the "how to try" sticky notes |

Every frame has the three rail buttons **WORLD / CIVIL / CARTO**. The owner
asked for three simple buttons rather than a likeness of the full GUI.
Switching domain re-resolves an open card, and changes the ring's four
diagonal slots (§5.1 of the scope).

## Things worth trying, because they exercise a rule

- Right-click between **Vhal Serai** and its label. Three objects stack there
  (settlement, label, route), so the card opens with **Select — 3 objects
  here**. That is Photoshop's Move-tool and QGIS's Identify pattern (§2.1).
- Arm **Uplift ▸ Mountains**, click the map a few times, then right-click. The
  card leads with **Commit N strokes**. A draft is the most time-sensitive
  state, so the *Draft* section is first (§4.1).
- Right-click the river in WORLD. *Inspect river* and *Trace downstream* are
  **shown greyed out with the reason**, not hidden (§4.2).
- Right-click a settlement in CARTO. You get CARTO's verbs plus one
  *Settlement actions in CIVIL ›* row that switches domain in place (fork F4).
- Open a `▸` family on the ring. The ring re-centres on that slot, and the
  **BACK** button sits at the sub-ring's own centre, filled in the accent.

## What the 2026-09-25 refinement changed

The owner reported that the contrast and the back button were hard to see.

- **Back** used to be a grey `‹` on the *main* ring's centre, where the
  sub-ring's own items could overlap it. It is now an accent-filled
  `‹ BACK` button at the **sub-ring's centre**, with a glow ring. The card's
  submenu back row is an accent `‹ BACK · …` band. The phone fan's back button
  is accent-filled and labelled too.
- **Ring:** a 30% scrim darkens the map behind an open ring. The disc is 88%
  opaque, up from 62%. Slot fills lifted `#191c1e` → `#262a2e`, borders
  `#34383b` → `#6b7277`, slot text → `#f4f5f6`. The hover state is a warm
  `#4a3717` fill with `#ffc86e` text.
- **Card:** section headers, hints and subtitles lifted from `#8d9296` to
  `#b0b5b9`/`#b8bdc1`. Disabled labels went from `#6f7478` (3.9 : 1 on the
  card, below the 4.5 : 1 text floor) to `#a0a6aa` (about 7 : 1), and their
  reasons to `#ebc080`. Dividers are
  `#3a3e42`.
- **Map text** (settlement names, labels) gets a halo, dark under light ink
  and light under dark ink, so the Antique/Ink/Print style presets stay
  readable.
- Rail and tab captions for inactive domains lifted to `#b8bdc1`.

## Opening them

The live canvas is the claude.ai Design artifact
`https://claude.ai/artifact/VFGFCHkWqchDynrpHFCjeK`. It is private to the
owner until it is shared from its Share menu. Press **Play** on a frame.

The files here are that canvas's own files, verbatim. Each loads
`./support.js`, the Design-component runtime, which lives one level up at
`design/support.js`. To open one outside the canvas, copy that file beside it.
It is not duplicated here, which is the same convention the other `design/`
imports follow.
