# City map look: closer to the reference, and a marker that opens its card and note

**A scope of 2026-11-13, from the owner's request:** *"Do you see how this city map has certain markers and how it
graphically looks? I'd like you to closely study how to get closer to it. Left clicking a marker should give the
information card and a note."* The reference is a WorldAnvil city map ("The City of ...dgard", a screenshot of a
promotional video; not stored in this repository, because it is third-party art).

This document **defines** milestones and holds the design reasoning. It carries no status column: what is built is
`cartalith-native/docs/STATUS.md`'s answer (CLAUDE.md, "One place holds state").

**What was done in this pass:** research and this scope, plus one scratch before/after render that proves the look is
reachable from the **real** generated layout. **Nothing shipped changed.** The left-click behaviour was not built (section
4.3 says why). The only code added is the evidence harness `cartalith-native/godot-project/_citylook_probe.gd` (with its
`.tscn`), which draws and asserts nothing that the app uses.

**A note on Ruling EO.** Ruling EO exists (`LARGE_ITEM_RULINGS.md`): on touch a tap shows the action chips and a
long-press opens the full card. This scope was first drafted against a checkout 163 commits behind origin, which
lacked rulings EJ–EV; the coordinator corrected this note on 2026-11-13. Re-open every symbol cited below against
origin/main before building (for example, the Note tab now binds labels, icons and landmarks too: 8206f6cb, 7039afa7).

---

## 1. The example (the owner asked for one first)

`cartalith-native/docs/evidence/city-look-2026-11-13/example_before_after.png`, 2622 x 958.

| File (same folder) | What it is |
|---|---|
| `before_0.png`, `before_1.png` | **The shipped City Viewer, unmodified**, canvas cropped from a 1600 x 1000 windowed frame. Velielshade (pop 17 981) and Torvtorgskaltorvbay (pop 17 614), seed 483920, 2400 km, 384 x 288, fit-to-window zoom |
| `after_0.png`, `after_1.png` | **A scratch restyle**: `_citylook_probe.gd`'s `_After` class redraws the *same* `urban_layouts()` dictionary from the engine with the reference's palette and primitives. Not shipped code, not a product feature |
| `example_before_after.png` | the pair for town 0 with captions |

**What is real and what is not, so nobody mistakes the mock for a build.**

- **Real:** every building footprint, ridge line, tone, street segment, block, plaza, wall ring and gate, tree, well,
  field polygon and the market anchor is read from the engine's layout dictionary (`building_*`, `streets`, `blocks`,
  `plaza`, `wall_ring`, `wall_gates`, `detail_*`, `farmland`, `market`, `citadel_wall`, `worship_sites`).
- **Palette and primitives, mine:** the colours are read off the reference by eye (section 2.1 has the sampled
  numbers) and are labelled judgements. Roof slope hatching, the wall band, tower discs, the furrow rows, the canopy
  highlight, the grass mottling, the serif title and the pin pictograms are drawn from `draw_*` calls in the harness.
- **Illustrative pins:** the citadel pin, the place-of-worship pin, the market "?" pin and the round well markers come from
  layout keys. **The two tavern pins and any anchor or vineyard pin are placed by the harness** to show where a
  kind-to-pin mapping would put them; the engine has no tavern (section 4.1).
- **Not in the example:** a harbour. No walled `curtain` town of this world has a `water_poly` (11 candidates scored, none
  with water), so water, shoreline and piers are not shown. They are in section 3 as gaps measured on code, not on a frame.
- **Only the fit zoom was captured.** The bars in section 5 are judged at several zooms; those frames are the milestone work,
  not claimed here.

---

## 2. The reference, element by element

Read from the screenshot (a phone capture at 1440 x 3168, the map filling the width).

| Element | What the reference does |
|---|---|
| Roofs | Hand-drawn top-down pitched roofs, **dark brown** with visible shingle stripes parallel to the ridge, a light ridge line, a dark outline and a drop shadow; many L and T shaped footprints packed along streets with almost no gaps |
| Wall | A **stone band** several metres wide, outlined, with a crenellated top; **round towers** at every angle; paved **gatehouses** where roads cross it |
| Streets and squares | **Tan dirt**, no dark casing; squares are open dirt; a **fountain or well** with a blue basin in the plaza |
| Trees | **Clumps of round canopies**, each with a dark outline and a lighter top-left highlight, merging into groves; dense inside the wall in gardens and outside it |
| Fields | **Rows**: bands of furrows (dark and light alternating) in strips outside the wall; a farm hut at the field edge |
| Harbour and sea | A **deep blue sea**, a **lighter shore band** and a sandy beach line; **wooden piers** with plank texture reaching into the water and a boat |
| Ground | Green grass with large soft colour variation, **not flat**; dirt inside the wall |
| Title | "THE CITY OF" over the name in a **serif face with a dark stroke**, cream fill |
| Markers | **Teardrop pins** with the tip on the place: **orange** with a white pictogram (tower, tavern mug, "?") or **white/grey** with a dark pictogram (castle, anchor, vineyard); **small round orange point markers** with a dark ring. Screen-sized: they do not scale with the map |
| Interaction | The hand cursor is over the white castle pin: the pin is the click target |

### 2.1 Sampled colours (mine, one run, 24 x 24 px patches picked by eye on the phone capture)

Roof mass lum 0.15 to 0.23 (RGB about 58,32,30 and 74,56,43); grass lum 0.23 to 0.38 (44,67,31 and 76,109,50);
dirt lum 0.49 to 0.69 (139,124,100 and 187,176,135); sea lum 0.11 to 0.14 (about 17,38,64). These are a screenshot of a
compressed video, not the art. They set the *order* of the bars in section 5, not their last digit.

The shipped viewer's own constants, for contrast: `GROUND` is rgb(194,179,148) everywhere (so the grass and the dirt are
the same colour and the wall reads as a line on paper), roofs are `ROOF_V` 0.60 with `ROOF_S` 0.46 (tan and yellow, lum
well above the reference's), the wall is a 4.5 m dark stroke, and a tower is 3.6 m of disc.

---

## 3. Gap analysis, element by element

Each row: what the shipped viewer shows (in `before_*.png`), what exists at its symbol, what it would take, cost
(**S** under a day, **M** a few days, **L** a week or more) and whether it is **render-only** (no generated value moves, so no
golden changes and no owner ruling under the "Change generated output" row of `MISTAKES.md`).

Ranked by how much each changes the first impression, biggest first.

| # | Element | Shipped, at its symbol | What it would take | Cost | Render-only |
|---|---|---|---|---|---|
| 1 | **Ground** (grass outside, dirt inside) | One flat parchment: `city_viewer_window.gd::_draw_canvas` fills `DRAW.GROUND` under everything; `draw_layout` has no grass and no wall-interior fill | A grass fill with deterministic mottling outside, the wall ring (or the blocks, on an unwalled town) filled dirt. The mock did this in 30 lines | S | **yes** |
| 2 | **Roof palette and hatching** | `urban_layout_draw.gd::_roof_color` and `_draw_roofs`: warm tan palette by the engine's `building_tone`; a roof ink pass and a ridge line only at `ROOF_INK_MIN_PX` 4.5 and `ROOF_DETAIL_MIN_PX` 9 px lots, so at fit zoom there is no hatching at all | Darker palette (the tone still comes from `building_tone`, so the variety is the engine's), a light ridge, 2 to 4 slope lines parallel to `building_ridge`, a stronger shadow. Slope lines need a per-roof clip to stay inside L-shaped footprints (the mock overshoots them slightly at corners) | M | **yes** |
| 3 | **Wall: stone band, towers, gatehouses** | `_draw_wall`: one `draw_polyline` of `WALL_W` 4.5 m, towers as `WALL_TOWER_R_M` 3.6 m discs, gates as small dots (`wall_gates`) | Two-pass band (dark casing then stone), a crenel dash, tower discs with a dark ring and a darker core, gate pavings with flanking towers. All from `wall_ring` and `wall_gates`; bastioned and palisade styles keep their own branches | M | **yes** |
| 4 | **Streets and plaza** | `draw_layout`: casing then fill, `W_FILL` per class; plaza fill and outline; wells and the market cross are 2.6 m glyphs in `_draw_props` | Tan fill with a soft edge instead of a dark casing; wider plaza paving; the well drawn as a basin with water and a stone ring (the mock does this at screen scale) | S | **yes** |
| 5 | **Trees as clumps** | `_draw_details(..., trees=true)`: rim circle then fill, already merged "as the target image's clumps"; radius `rr * m_scale` (about 1 to 2 m), so at fit zoom a tree is 2 to 3 px | A larger canopy with a highlight, and a shadow. **The reference has dense groves outside the wall; the engine's hinterland emits few trees there**, so a faithful look needs either decorative scatter seeded from the layout hash (render-only, never saved) or an engine change to `hinterland.rs` (moves generated output) | S (look), M (scatter), L (engine) | look and scatter yes; engine change **no** |
| 6 | **Fields as rows** | `_draw_farmland`: `FARM_FIELD` fill, one 0.22-alpha outline per strip (`FARM_LINE`); the frames show pale stripes | Furrow rows across each strip (dark and light alternating, clipped with `Geometry2D.intersect_polyline_with_polygon`, as the mock does), a farm hut where the engine already has `detail_kind` huts | S | **yes** |
| 7 | **Water and shoreline** | `draw_layout` fills `water_poly` flat `WATER` and `_draw_water_mask` re-paints it last; a coastal site draws only the river centreline | A deep-blue fill, a lighter shore band and a beach line from the polygon's own outline (the mock does the band in two strokes) | S | **yes** |
| 8 | **Harbour and piers** | `urban_bridge.rs::layout_dict` crosses `harbour_pt` and the `quay` street class; `growth.rs` builds piers and a mole (its doc says so) and `hinterland.rs` places a bollard per pier. **Piers are not crossed** (as of 2026-11-13; crossed as `harbour` on 2026-11-14, see the fourth build note); `urban_layout_draw.gd` never reads `harbour_pt` | Cross the pier polylines in the bridge (a new key, additive), draw planks as short cross-strokes and a moored boat as a stamp. **No generated value moves**, the geometry already exists | M | the draw is; the crossing is a Rust change, not a golden change |
| 9 | **Title** | The City Viewer's window title bar (`_title_label`); no title on the map or in an export (as of 2026-11-13; the map look gained one on 2026-11-14, see the fourth build note) | A serif stroked title as an overlay in the viewer (and optionally the export). Needs a serif font: the project bundles only Fira Sans, Fira Code and IBM Plex Mono (`godot-project/fonts/`); the mock used the OS's Georgia through `SystemFont`, which is not portable to Android | S (draw), S (a licensed bundled font) | **yes** |
| 10 | **Markers** | Section 4 | Section 4 | M to L | the glyphs yes; the click path is a behaviour change |
| 11 | **Hand-drawn stroke quality** (wobble, ink pooling, paper grain) | None; every edge is a straight `draw_*` | Out of reach for a vector draw; the reference's roofs are painted art. The look in the mock stops short of it by design. A line-art or sprite route exists (`ASSET_PACK_TOOLS_PROPOSAL.md` section 9, AP-10) but is not part of this scope | L, not proposed | yes |

**What the mock proves:** rows 1 to 7 and 9 were done over the real layout in one harness of under 500 lines, with no
engine change. The look is a **draw** change. The ranks above are by impression, not by effort.

**Where the shipped viewer is ahead of the reference:** it draws places of worship in their rite's built form, a citadel with a
gate gap, districts, markets by rank, intramural versus extramural roof tint and 16 detail kinds. None of that
should be flattened by a restyle; the restyle must keep each as a separate pass.

---

## 4. Markers

### 4.1 What markers exist today, and what the engine can name

| Where | Marker | Symbol | Picks |
|---|---|---|---|
| Map | **Settlement pin**: a faction-coloured disc, an ink ring, a capital ring, a coastal badge, trait badges, a glyph per class (`hamlet`, `village`, `town`, `city`, `capital`, `metropolis`: `⌂ ● ◉ ⬣ ✦ ★`) and a name | `map_overlay.gd::_draw_settlement_pin`, `SETTLEMENT_CLASS` | `_hit_test_settlement`: a **disc** around the anchor, `radius + HOVER_RADIUS_PAD` |
| Map | **Landmark ring** with a glyph by kind | `_draw_landmark_ring`, `_landmark_radius` | `_hit_test_landmark` |
| Map | **Manual icon** (families `settlement`, `feature`, `poi`, `custom`): a square, a triangle, a diamond or a circle; pack art if a pack is loaded (AP-3) | `_draw_icon_glyph`, `set_marker_art_resolver` | `_hit_test_icon` |
| Map | Sample pin (phone), journey markers, timeline halos | `_draw_sample_pin`, `_draw_journey_markers` | CM-5 |
| **City Viewer** | **None.** The canvas draws the layout and a legend; a left press only starts a pan | `city_viewer_window.gd::_on_canvas_input` | nothing |

**Kinds the engine can name, to map pictograms onto** (nothing below is a new entity):

| Source | Kinds |
|---|---|
| `SETTLEMENT_CLASS` | hamlet, village, town, city, capital, metropolis |
| `landmark_kinds()` (`cartalith-civ/src/landmark.rs`) | 5 families, 40+ keys: Physical (peak, ridge, saddle, cliff, gorge, cave, waterfall, spring, lake, delta, river_confluence, volcanic_feature, rock_formation, glacial_feature, ancient_forest), Transport (mountain_pass, river_crossing, ford, bridge_site, road_junction, caravan_station, portage, **harbour**), Economic (mine, quarry, salt_works, resource_extraction_site, market_site, trade_depot), Military (**fort**, **watchtower**, fortified_pass, fortified_crossing, battlefield, border_marker), Cultural (**shrine**, **temple**, sacred_grove, sacred_mountain, pilgrimage_site) |
| Layout dictionary (inside one city) | `citadel_*` (castle), `worship_sites` (rite: spire, minaret, dome), `harbour_pt`, `market_centers` and `market_names`, `detail_kind` (well, cross, crane, bollard, spoilheap, dryingrack, logboom), `farmland` |
| Asset library `poi` slots | ruin, landmark, mountain_peak, lake, named_forest, battlefield, shrine, cave, bridge, other; `trait` slots: fortified, mining, port, administrative, trade_hub, military, religious; `seamarks`: lighthouse, beacon, buoy, anchorage, shipwreck, reef |

**What the engine does not have: a tavern, an inn, a shop, a guildhall.** `cartalith-vault/src/links.rs` records that
**POIs are not a ported concept** (CV-01, a deliberate decision; `EntityKind` has `Settlement`, `Province`, `Continent`,
`Faction`, `Culture`, `Landmark`). The reference's tavern-mug pins therefore have no data to hang on. This is the one place
the look reaches past rendering; Q5.

### 4.2 The pin design

**Shape.** A teardrop: a head circle of radius `r` and a tip at the anchor, so the anchor is exactly the place and the head
sits above it, as the reference draws it (the mock builds it from a 25-point arc plus the tip and outlines it). One shared
draw function in a new file (`shell/pin_marker.gd`, preloaded, no `class_name`, so no `--import` pass is needed), taking
`(canvas, tip, kind, state, scale)`, called by both the map overlay and the City Viewer; never a second copy.

**Size.** Screen-constant, like the settlement pin (`PIN_SCALE_REF_PX`, `_civ_zoom_k()` already invert the camera's zoom for
this), with a floor of 44 dp for the touch target on tablet and phone and an 18 px head on desktop.

**Pictograms**, drawn as vector strokes, not font glyphs (the settlement pin's own header notes that Godot's default font
may lack its dingbats). The first set, one per kind family, with the engine source:

| Pictogram | For | Source |
|---|---|---|
| Castle | citadel; capital and metropolis settlements; `fort` | `citadel_wall`; `SETTLEMENT_CLASS`; landmark |
| Tower | place of worship (spire, minaret, dome keep their own head); `watchtower` | `worship_sites`; landmark |
| Anchor | harbour, port trait | `harbour_pt`; landmark `harbour`; trait `port` |
| Wheat | a farm field; `farmland` | `farmland` |
| Pick | mine, quarry, salt works | landmark Economic |
| Mug | tavern: **no engine entity** (Q5) | none |
| "?" | a marker with no name or no note yet (Q6) | derived |
| Peak, cave, spring, bridge, swords | Physical, Transport, Military landmark kinds | landmark |
| Round dot | wells, market squares, gates: the reference's small round point markers | `detail_kind` well, `market_centers`, `wall_gates` |

**Colour (the one meaning the reference does not tell us).** The reference has orange and white pins with no legend. The
recommendation (Q6): **orange when the marker's subject has a linked note, white when it has none**, so the pin itself says
"there is a story here", which is what the owner's next sentence is about. Faction colour stays on the settlement ring, not
the pin.

**Where it is drawn.** On the map, replacing the settlement disc only where the owner chooses it (Q1); landmarks and manual
icons get the same shape. In the City Viewer, as a **screen-space pass after** `draw_layout`, in the viewer's own
`_draw_canvas` -- not inside `draw_layout`, which the map's town layer and the export share and which is screen-agnostic.

**Picking a teardrop.** The shipped hit tests are discs around the anchor. A teardrop's body is **above** the anchor, so a
click on the head would miss. The hit test must be the head circle plus the stem, and the pad rule
(`HOVER_RADIUS_PAD`) must be re-derived. `hits_at`'s settlement and landmark entries share this logic and must move
together (`MISTAKES.md`: derive the list from the definition, every consumer).

### 4.3 Left click: what it does today, and the change

**Today (read at the symbols, not run in this pass; `MISTAKES.md`: say what you opened).**

- **Map, desktop, left press on a settlement:** `map_overlay.gd::_gui_input` -> `_pick_mark` -> emits `settlement_selected`,
  `landmark_selected`, `icon_selected` and `map_clicked`. `app.gd::_wire_selection` turns the settlement signal into
  `select("settlement", index)`, and the landmark signal into `select("landmark", index)` when no settlement was hit. `select`
  records history, calls `_apply_selection` and `RightDock.on_selection`, so the **Context panel follows** and shows its
  *current tab*: **Inspect by default** (`RightDock._tab`, `CONTEXT_PANEL_DEFAULT` = inspect, note, ask). The **card does not
  open**; the toolbar hint says "Click selects - right-click opens the card".
- **The card** (`context_card.gd`, `context_broker.gd`) opens on right-click (release, below the slop), on a touch hold, on
  `Q`'s ring, through `context_requested` -> `ContextBroker.resolve` -> `present`.
- **The note** is the Context panel's **Note tab** (`context_note.gd`): one tab away from Inspect, bound for **settlement and
  faction only** (`BINDABLE`); a landmark says "bound through its key, which this tab does not make yet".
- **Phone, tap:** the withheld press is released as a tap (`_release_touch_press`), the same signals fire and
  `RightDock._show_on_phone()` raises the right sheet; the card is the peek sheet (`phone_present_peek_card`) on a hold.
- **City Viewer:** a left press starts a pan. Nothing is picked.

**The change asked for.** A left click, or a tap, on a marker opens **the information card and the note**, through the paths
that already exist:

1. The press still goes through `_pick_mark` and `select(kind, id)`: **one selection, one history step**, as today. Nothing
   new produces selection.
2. After `select`, the overlay emits `context_requested` with `request_at(pin_screen_pos, "click")` narrowed to that one hit
   (`ContextBroker.reselect(hit)` already does the narrowing for Select ▸). `present` opens the **same** card (desktop, tablet)
   or peek sheet (phone). `"click"` is a third value for `source`, which `present` and `_ring_clearance` currently read as
   `mouse` or `touch`; the dock side rule for a tablet touch must stay as it is.
3. The card gains one **Note section**, drawn from `ContextNote`'s own resolver (the `no_note` / `note` / `novault` states its
   `resolve` already computes for a subject), **not a copy of it**: the state function becomes public and the card calls it.
   It shows the note's title and first lines when there is one, with `Open note` (focuses the dock's Note tab through
   `RightDock.show_tab`, the existing path) and `Create note` when there is none (the template choice stays in the Note tab,
   VQ2); for a subject the vault cannot bind it says why, never a blank (`MISTAKES.md`, "no value as a plausible value").
4. The element chips the SM-4 phone card already carries (Open note / Attach note, Add event, Journey from here, City viewer)
   stay the card's rows; they are not re-implemented.
5. **Gate:** only while the pointer (select) tool is the armed tool and no draft is open. A left click means *place* or
   *paint* under any other armed tool, and that must not change (`MISTAKES.md`: "arm-another-tool path").
6. **The City Viewer** gets the same behaviour on its new pins: a click calls the same `select` and the same
   `request_at`-shaped request (anchored to the pin, in the viewer's coordinates), then the card. The viewer's drag-to-pan keeps
   working: a press that travels past the slop is a pan, as the map's tap-versus-drag test already decides.
7. Prose to fix in the same change: the toolbar hint ("Click selects - right-click opens the card"), the Select tool's
   tooltip, `MAP_CONTEXT_SCOPE.md`'s statement of the left click, and any shortcuts-dialog or command-index text that names it.

**Why this was not built in this pass, in one list** (the brief allowed building it only if it was a few lines):

- the card has **no Note section** and `ContextNote` has **no public state function** (steps 2 and 3);
- which surface should open (the card, the dock's Note tab, or both) is a design fork, and showing the Note tab on every click
  would replace Inspect's facts, which is a loss the owner did not ask for (Q3);
- landmarks have no Note binding yet (`BINDABLE`), so "a note" for a landmark pin needs `vault_landmark_entity_id` wiring first;
- the armed-tool gate, the touch tap-versus-hold timing (`_TOUCH_HOLD_MS`) and the double-tap zoom detector in `viewport_host.gd`
  all touch the same press, and `cargo test` cannot see any of them (`MISTAKES.md`): it needs windowed probes on desktop, tablet
  and phone, which are in CL-9's bars.

The smallest change that *looks* like the ask (`select` then `right_dock_ctrl.show_tab("note")` in the settlement lambda)
was rejected for the reason above.

---

## 5. Milestones, with numeric bars

Order follows the owner's wording: the marker and the click first, then the look. Every bar is judged at the City Viewer's
zoom levels **x1 (fit), x4, x16 and x64** and on **at least three seeds and six towns** (`MISTAKES.md`: one world is one
sample), through `_citylook_probe.gd`'s harness extended for the sweep. **Pixel bars name their palette and run windowed**
(`MISTAKES.md`: palette-bound thresholds, and `--headless` makes `ImageTexture.update()` a no-op). Every number below is a
**labelled judgement**, set from section 2.1's reading, to be re-set by CL-0's measurement of the reference.

| # | Milestone | Bars |
|---|---|---|
| **CL-0** | **Harness and baseline.** Extend `_citylook_probe` to the zoom sweep and three seeds; save frames to `docs/evidence/city-look-2026-11-13/`; record the shipped viewer's measured roof luminance, tower radius, wall band width and tree radius per zoom | Frames at four zooms for six towns; the baseline table in the file; `var_to_bytes(layout)` equal before and after any CL step (**render-only proof**) |
| **CL-1** | **Marker draw**: `pin_marker.gd`, the teardrop, the first pictogram set, the orange/white state. In the City Viewer (derived pins only: citadel, worship, harbour, farm, market) | Tip within 1 px of the anchor at every zoom; head diameter 18 to 22 px on desktop, 44 dp touch target on tablet and phone; pin size within 1 px across x1 to x64; 0 overlaps of two pins' heads by more than 25 % (pins nudge, the tip never moves); one frame per pictogram at x1 and x16 |
| **CL-2** | **Marker pick**: head-plus-stem hit test in the map overlay and the viewer; `hits_at` and `_pick_mark` agree | A click on the head, on the stem and 1 px outside the head picks; 4 px outside the head does not; the same answer from `hits_at`, `_hit_test_*` and `_pick_mark` on 200 random points |
| **CL-3** | **Left click -> card and note** (section 4.3) | Windowed probes on desktop 1600x1000, tablet 2560x1600 `--force-touch`, phone 1080x2340: a left press or tap on a settlement, a worship site and a landmark opens the card or the peek sheet **and** shows the note state (note, no note, cannot bind) with its reason; exactly one history step and no map edit; with the Route, Settlement or Sculpt tool armed the click places or paints exactly as before (0 cards); a pan that travels past the slop opens nothing; the City Viewer drag still pans; the phone's tap still raises the sheet; `_ctxcard`, `_ctxbroker`, `_ctxpicks`, `_ctxphone` and `_ctxring` still pass, changed only in their literals for the new section |
| **CL-4** | **Ground, streets, plaza, water** (rows 1, 4, 7) | Outside-wall ground mean luminance 0.20 to 0.40, inside-wall dirt 0.45 to 0.70 (so they differ by at least 0.15); water 0.08 to 0.20 with a shore band 4 to 12 px; no `triangulate` failures (`_fill_ground_polygon`'s guard stays); `draw_layout` redraw time within 1.25x of the shipped median, run alone, three runs with the spread |
| **CL-5** | **Roofs** (row 2) | Mean roof luminance 0.12 to 0.28; at x4 and deeper at least 2 slope lines on at least 90 % of roofs wider than 12 px; the engine's tone spread kept (the 5th to 95th percentile tone range at least 60 % of the shipped one); ridge visible; no roof ink at x1 where a lot is under 4.5 px (the existing gate stays, so the town does not turn to soot) |
| **CL-6** | **Wall, towers, gatehouses** (row 3) | At x1, wall band 5 to 9 px, tower diameter at least 7 px, towers at 100 % of ring vertices (the shipped rule); every `wall_gates` point drawn as a paved gate; bastioned, palisade and ditch styles unchanged (their frames byte-equal to the shipped ones) |
| **CL-7** | **Trees, fields** (rows 5, 6) | Canopy radius at least 3.5 px at x1; at least 1 merged clump per 40 trees; furrow rows at 4 to 6 px spacing, alternating, on 100 % of strip fields at x4; decorative scatter (if Q8 is yes) never changes the saved layout and is seeded: two runs give identical pixels |
| **CL-8** | **Title** (row 9) | Serif face with a 6 to 10 px stroke, readable (stroke versus fill contrast at least 4.5:1) over grass, dirt and water; the font is bundled with its licence file, not the OS's |
| **CL-9** | **Harbour and piers** (row 8), only on a town with a harbour | Pier geometry crossed additively (no existing key changes, `urban_layouts` shipped keys byte-equal); piers drawn with planks, at least 2 piers on a harbour of rank 3 or above; none on a town with no `harbour_pt` |
| **CL-10** | **The map's own markers** get the same shape (settlement, landmark, manual icon), behind Q1 | The settlement pin's class, capital ring, coastal badge and trait badges all still drawn and distinguishable at x1 and x16; `_pinlabelovr_probe` (labels clear of pins) unchanged |
| **CL-11** | **User-placed city pins and Poi** (Q5 answered yes, Ruling EW item 4; built, see the 2026-11-14 build note) | A `Poi` entity kind (a seventh `EntityKind`, which moves the guard count in `links.rs`'s doc), save round trip byte-identical on a real prior-format save, undo, a note bound by id |
| **CL-12** | **Pin declutter**: generated landmark pins thinned by a screen-space rank-and-spacing rule so the zoomed-out map is not swamped (built 2026-11-15, see its build note) | At the fit zoom no two shown heads overlap, the set is identical under a 1 px pan, 2x zoom shows more, selected and noted pins are always shown, a hidden pin is not picked (`_declutter_probe.gd`) |

**Build note, 2026-11-13 (the first build step; status is `STATUS.md`'s).** The restyle rows CL-4 (ground, streets, plaza, water),
CL-5 (roofs), CL-6 (curtain wall, towers, gates) and CL-7 (trees, fields) were built together as the second selectable style in
`shell/urban_layout_look.gd`, with Ruling EW's two additions: the ground comes from the settlement's terrain
(`EngineBridge.settlement_ground_colour`, a `ground_color` key on every layout) and the citadel is drawn as a full fortification.
The bars above that quote luminance and pixel widths were NOT measured; the probe `_citylook1_probe.gd` checks the claims named in
`STATUS.md` instead. Numbering note: the milestone table's CL-1 is the marker draw, and this step is not it. The wall
that hugs the built area (Ruling EW item 1) was **built 2026-11-14** as `cartalith_urban::wallhug` behind `WorldParams::urban_wall_hug`
(`DECISIONS.md` section 7p; measured numbers and the open pockets in `STATUS.md`; frames `evidence/city-look-2026-11-13/wallhug_before_after.png`).
Still open: ~~CL-10 and the choice of default look~~ (both closed 2026-11-15 by Ruling FF items 1 and 2: see the last build note). (CL-1's head-overlap nudge and the City Viewer's click card were built 2026-11-15, the fifth build note below.) (CL-1's draw, CL-2 and CL-3 are the second build note; CL-11 the third; CL-8 and CL-9 the fourth, below.)

**Build note, 2026-11-13 (the second build step: CL-1's shared draw, CL-2 and CL-3; status is `STATUS.md`'s).**
`shell/pin_marker.gd` is the one teardrop (tip on the anchor, head 2.1 radii above it, vector pictograms for the engine's kinds:
castle, spire, tower, anchor, wheat, pick, peak, cave, drop, bridge, tree, swords, well and the round dot as the stated fallback),
orange with a linked vault note, white without. The map draws **generated landmarks** through it (replacing the class ring; the
class and importance still set the size, `_landmark_pin_r`), the City Viewer draws derived pins through it (citadel, worship,
harbour, market, farm; wells are mapped, not pinned). `PinMarker.hit` (head circle plus stem triangle) is the ONE footprint:
`_hit_test_landmark`, `hits_at` and `_pick_mark` all read it, and the City Viewer's `pin_at` too; a touch form widens the pad to
a 44 dp target (`touch_pad`). A left click (desktop, pointer tool only) on a settlement, landmark pin or icon opens the existing
card for that one marker through `ContextBroker.resolve` (`DccApp._open_marker_card`); the card's NOTE section is broker rows
from `ContextNote.card_rows` (the Note tab's own `resolve`, never a copy): *Open note · title* with the note's first lines,
*Create note...* (a dialog with the 4df2db45 folder chooser, one create-and-bind path shared with the Note tab), or a disabled row
that says why (no vault, cannot bind).
**Reconciling Ruling EO with Ruling EW on touch (a judgement, for the owner to overturn in one line).** EO is the later word on
the touch gesture itself and is explicit (a tap selects and shows chips, a long-press keeps the card), while EW's "(tap on touch)"
was written beside a desktop-click question. So on touch the TAP keeps EO's chips, whose Note chip is the existing state-aware
*Open note for X* / *Attach note to X...* route into the vault window (it has the same folder chooser), and the LONG-PRESS card
(the card on a tablet, the peek sheet on the phone) carries the new Note section. Both owner texts hold: the card with the Note
section is what a touch hold opens, and the one-tap route to the note is the chip. **Not built in this step:** the City Viewer's
click opens no card (it selects the town and the Context panel follows; the card there needs an anchor in the viewer's own
window), the CL-1 head-overlap nudge, the settlement pin and the manual icons keep their shapes (CL-10, behind Q1's map half), and
a landmark's card header still reads `(unnamed)` because `hits_at` carries no landmark name (the Note row below it does).

**Build note, 2026-11-14 (the third build step: CL-11, user-placed points of interest, and the marker-style option; status is `STATUS.md`'s).**
Ruling EW item 4 (*"Yes, user-placed POIs"*) is built, reversing CV-01 in the code.
*The entity.* `cartalith-civ/src/poi.rs`: `Poi {id, x, y, kind, name}` and `PoiKind`, the owner's twelve kinds (tavern or inn, shop, smithy,
temple, shrine, guild, gate, well, ruin, camp, mine, other). The `id` is **stored and never reissued** (a counter that only grows and is
saved), unlike a label's or an icon's cell-hash identity, so a move or a rename leaves the note bound and a delete leaves an absent
entity that no later POI can inherit. `EntityKind::Poi` is the ninth kind (`links.rs`'s guard count moved), with a vault folder
`Points of interest/`.
*The store.* A fourth annotation store, `AnnoStore::Pois` (`annotation_undo.rs`, `annotation_bridge.rs`): every `poi_*` call
(`poi_bridge.rs`) goes through `anno_scope`, so a placement, a move, a rename, a kind change and a delete are **one undo step each** and a
refused call records nothing; the id counter is outside the snapshot, so an undone placement never lets a later POI reuse its id.
*The file.* The optional `annotations/pois.json` (`SAVEFILE_COMPAT.md` section 11.2a), written only when a POI exists or one ever did, so every older save
re-serialises unchanged; unreadable rows (unknown kind, off-grid or non-finite point, id 0, repeated id) are dropped, counted and reported.
*The tool.* CARTO's tools block gains **Point of interest** (beside Label and Icon, which it shares the ledger and the card's rows with; CIVIL's
tools place simulated things, which a POI is not): a click asks for the name (`DccWidgets.prompt`; an empty name places nothing), the options row
holds the kind picker. `shell/pin_marker.gd` gains seven pictograms (mug, shop, anvil, banner, gate, ruin, tent) and maps the twelve kinds
(temple, shrine, well and mine share the landmark pictograms; `other` is the stated dot).
*A first-class marker.* The overlay draws POIs through the shared `PinMarker` (orange with a linked note, white without), picks them through
the same head-plus-stem footprint (`_hit_test_poi`, `hits_at`, `_pick_mark`, `marker_at`; a POI takes a pick only when its anchor is strictly nearer than
the settlement, landmark or icon, so no earlier pair resolves differently), and a left click opens the card with its NOTE section and, through
`GlobalTools._poi_rows`, **Rename, Move, Change kind (a submenu; on the phone a jump to a dialog) and Delete**; the Context panel has a
Point of interest page and the Note tab binds through `ContextNote` (`vault_poi_entity_id`). Find on map lists them by name or kind and selects the hit. The City Viewer draws the
POIs inside the town (within 850 m of the layout's market anchor, by the map town transform inverted) and a click on one selects it.
*The marker-style option (owner, 2026-11-14: "Can you keep the old circle markers as an option").* `DccSettings.marker_style()`, `pins` (default) or `circles`, in
Preferences > Graphics beside Town look and in Assets beside Landmark types; per user, never in a world save. `circles` draws the landmark class ring
**restored from `d2a8b6a3^`** (`map_overlay.gd::_draw_landmark_ring`) with the ring's own footprint (`_landmark_footprint`), and POIs and the City Viewer's derived markers as a
disc with the same pictogram (`PinMarker.draw_ring`, `hit_ring`); `PinMarker.draw_styled` / `hit_styled` / `rank_distance` are the one branch every drawn
marker and every pick goes through, so the picked shape is the drawn one in either style.
**Judgements, for the owner to overturn:** the tool lives in CARTO; a POI is sized like a Local-class landmark and is always shown (the Landmarks switch is the
generated landmarks'); a name is required to place one; in circles mode a POI is a disc centred on its anchor, not a ring; the City Viewer click selects the POI (the
Context panel follows) and opens no card, as for its derived pins. **Not built:** a POI category or a faction or way snap in CIVIL (CV-01's own design), a
settlement-to-POI conversion, hover tooltips for POIs, the tablet frames of the new tool, and POIs drawn into the town plan's own geometry.

**Build note, 2026-11-14 (the fourth build step: CL-8 the title and CL-9 the harbour, in the map look; status is `STATUS.md`'s).**
CL-9's Rust half is the one additive `harbour` key (Q9 above). The draw is `urban_layout_look.gd::_draw_harbour` (a dark-edged timber deck
with a cross-plank every 1.1 m, coarser when the planks would be under 2 px apart; a stone-and-outline breakwater when `mole` is
present) and `_draw_shore_band` (the lighter shallow band drawn on the WATER side of the polygon's edge, after the mask that repaints
flat deep water). Piers draw wherever the layout is drawn, the map's town layer included (where, as for the quay street, the layout's
water is not the base map's, so a pier could stand on land the raster shows as water: not measured). CL-8's title is
`urban_layout_look.gd::draw_title` / `_draw_map_title`: "THE CITY OF <NAME>" in capitals, the word following the settlement's kind
(CITY for city, capital and metropolis; TOWN; VILLAGE; HAMLET; none and the name alone for an unknown kind), in bundled Cinzel at
weight 700 with a cream fill over a dark 6 to 10 px stroke. **Where it sits (the choice the task left to the builder):** in the City
Viewer it is anchored to the top-left of the town's fit box and clamped inside the canvas, so it is on screen at every zoom and pan
(a title pinned to the town would leave the screen at x6); in the map's town layer and the export it is pinned to the top-left of the
town's screen-space bounding box and drawn only when that box is at least 480 px wide (`TITLE_MIN_BOX_PX`), so an overview map
carries no titles and a deep-zoom or large export does; a rotated town (the map rotates every plan by its terrain orientation) keeps
its title upright at the corner of the rotated bounding box. The name and kind reach the draw as two GDScript-side keys,
`place_name` and `place_kind`, added by `EngineBridge.urban_layouts` beside `ground_color`; absent when the settlement has no name. The
ink plan draws no title and is otherwise untouched. **Not built here:** a moored boat, the quay as a drawn timber edge, and the
mole on a frame (the engine builds one on an open `coast` site only, and no walled coast town was found; its draw is proven on a
labelled synthetic mole).

CL-0 to CL-3 are the owner's literal asks; CL-4 to CL-8 are render-only and need no ruling; CL-9 adds a Rust key; CL-11 reverses
a documented decision and needs one.

---

## 6. Owner questions, each with a default

| # | Question | Recommended default |
|---|---|---|
| **Q1** | Does the new look apply to the City Viewer only, or also to the map's own town layer at deep zoom and the export? | **All three through the one `UrbanLayoutDraw`**, the City Viewer first. The three already share `draw_layout`; two looks would be a dual path (the standing rule is to scrub the superseded approach, `DECISIONS.md` section 7p) |
| **Q2** | Keep today's ink-and-parchment plan as an alternative style, or replace it? | **Replace**; keep today's line plan only as the export's "plan" legend if the owner wants one. Style presets exist (`MAP_STYLE_RESEARCH.md`) if a second look is wanted later |
| **Q3** | On a left click, what opens: the card, the dock's Note tab, or both? | **The card, with a Note section** (title, excerpt, Open or Create note); the dock follows the selection as it does today and `Open note` focuses its Note tab. Never replace Inspect on every click |
| **Q4** | Single click or double click? | **Single**, as asked, only under the pointer tool; a double tap on a phone stays the zoom (`_double_tap_zoom`) |
| **Q5** | Taverns, inns, shops: add a place-of-interest entity users can place in a city and bind a note to? | **Not now.** Ship derived pins first (CL-1 to CL-3); decide CL-11 after seeing them. It reverses CV-01, so it is yours to rule |
| **Q6** | What does pin colour mean? | **Orange = has a linked note, white = none**, and a "?" head when the subject has no name or note yet. The reference gives no legend |
| **Q7** | The serif font for the title | A bundled **OFL** face; two candidates to confirm the licence of before bundling: *Cinzel* and *IM Fell English*. Not checked in this pass. **Answered 2026-11-14 by the agent that built CL-8, on this default (not by the owner; overturn in one line): Cinzel**, bundled as `godot-project/fonts/Cinzel-Variable.ttf` (the variable-weight file from `https://github.com/google/fonts/tree/main/ofl/cinzel`, page `https://fonts.google.com/specimen/Cinzel`) with its licence beside it, verbatim, as `Cinzel-OFL.txt` (`https://github.com/google/fonts/blob/main/ofl/cinzel/OFL.txt`: "Copyright 2020 The Cinzel Project Authors", SIL Open Font License 1.1, no Reserved Font Name declared; the file is shipped unmodified). Chosen over IM Fell English for its capital-led display letterforms (the title is in capitals) and its weight axis; the bold sans behind it covers glyphs Cinzel lacks |
| **Q8** | Decorative trees outside the wall, to match the reference's groves, drawn but not part of the generated world? | **Yes**, seeded from the layout, never saved, never read by any engine path, included in export so export follows screen |
| **Q9** | Cross the engine's pier geometry into the layout so piers can be drawn? | **Yes**: additive, no generated value moves. **Answered 2026-11-14 by the agent that built CL-9, on this default (not by the owner): done** as ONE new key, `harbour`, on a layout that already has `harbour_pt` (`urban_bridge.rs::layout_dict`): a dictionary with `piers` (two points a pier) and, only where the engine built a breakwater, `mole` (four points); absent when `harbour_pt` is. `UrbanLayout::harbour` carries `buildHarbour`'s whole works from `generate()` untouched; `urban_adapter/tests.rs::the_harbour_works_cross_additively_and_move_no_generated_value` pins a digest of every earlier field captured on the commit before the key existed |

---

**Owner answers, 2026-11-13: Ruling EW** (`LARGE_ITEM_RULINGS.md`). Q1 everywhere, with three corrections: the ground follows the local terrain, the citadel/castle is drawn in full as a walled fortification, and the wall hugs the built area (no wide empty band inside it; a §7p layout change). Q2 keep both styles. Q3 card with a Note section. Q5 user-placed POIs (reverses CV-01). Q4, Q6–Q9 take their defaults. The milestones below must absorb these.

## 7. Sources

External, retrieved 2026-11-13 by search snippet only (the pages were not opened); treat as pointers.

- World Anvil, interactive map pins (pins link to articles or maps; a Nov 2024 changelog says a pin's link opens the article):
  <https://www.worldanvil.com/learn/beginner-tutorials/get-started-maps>, <https://blog.worldanvil.com/worldanvil/dev-news/interactive-maps-upgrades>,
  <https://www.worldanvil.com/learn/map-making/maps>. **The search found nothing that confirms the teardrop shape or the orange/white
  colour meaning**; both are read off the screenshot alone.
- Watabou, *Medieval Fantasy City Generator*, devlog 0.11.0 "roof details and district view" (Plain, Hip and Gable roof options):
  <https://watabou.itch.io/medieval-fantasy-city-generator/devlog/711706/0110-roof-details-and-district-view>. No source found for an
  ink or hatching technique.

In-repository: `MAP_STYLE_RESEARCH.md` (the render vocabulary and ranked presets), `URBAN_MORPHOLOGY_SCOPE.md` (the layout model
and its renderers), `ASSET_LIBRARY_SCOPE.md` and `ASSET_PACK_TOOLS_PROPOSAL.md` (pack art for markers), `MAP_CONTEXT_SCOPE.md` and
`PC_SHELL_PROPOSAL.md` (the card, the Context panel and the Note tab), `VAULT_EDITOR_PROPOSAL.md` (notes).

## 8. Hazards for whoever builds this (`MISTAKES.md` rows that apply)

- **Do not restyle inside the shared pass without proving render-only.** CL-0's `var_to_bytes(layout)` equality is the proof;
  `_fill_ground_polygon`'s triangulation guard must stay on every new fill (42 failures at x64 once).
- **Windowed probes only for pixels**, with a positive control that must move; run each of desktop, tablet and phone.
- **Every consumer of a hit**: `_pick_mark`, `hits_at`, `_hit_test_*`, the hover signals and the context broker must move
  together when the pin's footprint changes.
- **Put the Ruling BK annotation in every new function and constant** (what it does, why, what it must never do; each colour's
  source is the reference screenshot, labelled a judgement).
- **A palette change re-bases every contrast pair**: the legend swatches in `city_viewer_window.gd::_swatch` use the old
  colours and must be recomputed.

**Build note, 2026-11-15 (the fifth build step: the City Viewer's card and CL-1's head-overlap nudge; status is `STATUS.md`'s).**
*The viewer click.* `city_viewer_window.gd::_select_pin` selects the pin's town or POI and, on a desktop click or a touch long press, calls `DccApp.open_viewer_marker_card(kind, id, anchor, source)`, which builds a one-hit request from the engine's row (`bridge.settlements()` or `poi_get`) and calls `ContextBroker.resolve`, the map's path. The request's `anchor` is the pin's place in the shell viewport (the window position plus the canvas transform), and `present()` uses it instead of the overlay's `screen_pos`. A tap on touch keeps Ruling EO (select, no card); the hold is the map's 480/500 ms. Judgements, for the owner to overturn: the viewer's card is not gated on the armed tool; a viewer popped out into a native window selects and opens no card.
*The nudge.* `PinMarker.fan_offsets` is the one algorithm for the map (landmarks and POIs, both marker styles) and the viewer. Heads overlapping by more than 25 % of the radii sum cluster; a cluster of up to 18 is fanned (pins in rows of six with half-spacing stagger so no tip lands on a neighbour's head, which the first ring layout did; circles on a ring), a larger knot is left (zoom resolves it). A fan that lands on a neighbouring cluster widens the link distance (four passes, fewest residual overlaps wins). Offsets read only relative positions, so a pan changes nothing (the map computes them on a canonical rect). Every pick reads the nudged position. Leaders and the true-point dot are drawn under the markers.

**Build note, 2026-11-15 (the City Viewer popped out: its card; status is `STATUS.md`'s).** The fifth note's viewer click assumed a docked viewer: the anchor was the dialog's `position` plus the canvas transform, in the main window's space, and the card an embedded child of the main window. Popped out (`POPOUT_WINDOWS_SCOPE.md`, `PopoutHost`) `position` is a screen position and the card would have opened in the main window, on the wrong monitor at a wrong place. Chosen: the card opens **inside the pop-out**, as `PopoutHost.dialog_host` already does for dialogs. `open_viewer_marker_card` takes the asking window, a request naming a native pop-out carries `host_window`, and `ContextBroker.present` moves its one `ContextCard` under that window before opening it (a docked, closed or non-pop-out request falls back to the app, so docked, tablet and phone are unchanged). The anchor is then the canvas transform alone, the card's bounds are its parent's viewport (so it flips and clamps inside the pop-out), the click-through hook is connected on the pop-out's own `window_input`, and the Create note dialog opens in the pop-out too. Probe `_popcard_probe.gd`; seven mutants killed. Rejected: the card in the main window with the main window raised (steals focus from the monitor the user is looking at and needs the anchor translated across windows).

**Build note, 2026-11-15 (the sixth build step: Ruling FA, overlapping pins tilt; status is `STATUS.md`'s).** *Supersedes the fifth note's nudge for the Pins style.* `PinMarker.tilt_angles` keeps every pin's tip on its exact place and rotates each overlapping pin about it: a pair at -45/+45 degrees (stems 90 apart, a heart), a cluster of three or more evenly across a spread of 120 degrees for three, plus 60 per extra pin up to a 180 degree cap (+/-90). The 45 is the owner's; the spread, the 60 step (two heads on one tip clear when their stems are 2*asin(1/2.1) = 56.8 degrees apart) and the cap are labelled judgements. The hit footprint is the rotated pin (head and stem), the pictogram stays upright and a pin has no label to turn. `fan_offsets` (slide plus leaders) survives for the Circles style only. The map's settlements are discs and glyphs, not teardrops, so they are outside the rule until CL-10. The CL-1 bar "pins nudge, the tip never moves" now reads: pins tilt, the tip never moves.

**Build note, 2026-11-16 (pin rings, the Ruling FA follow-up; status is `STATUS.md`'s).** The three leftovers of the sixth note: more than four pins on one tip overlapped like a hand of cards, a knot over `FAN_MAX` (18) stayed upright, and a tilted fan could land on a neighbouring cluster. Design: `PinMarker.fan_layout` keeps the heart, the three-fan and the four-pin half circle and, from five, puts the rest on outer rings: the same pin lifted `FAN_RING_STEP` = 2.3 head radii (one head diameter plus a 15 % gap, a labelled judgement) per ring along its tilt axis, on a stalk from the true place, four rings (4 + 7 + 11 + 14 slots; a ring's slots are `max(2 asin(FAN_CLEAR / S), 180 / n)` degrees apart over a half circle for its stem `S` in head radii), `FAN_PIN_MAX` = 36 (circles keep `FAN_MAX` = 18). Neighbouring fans: clusters are laid out, the fanned heads compared across clusters, and colliding clusters merged and re-laid out. Rejected, with the reason: a "+N" badge (hides pins at rest, adds an open state to every pick path), a spiral (a pin no longer points at its place), spread tips with leaders (moves the tip, which Ruling FA keeps exact). Picking follows the drawn pin through `lifted_tip` (`draw`, `hit_styled`, `rank_distance` take a `lift`), and an outer-ring pin's anchor distance against other marks is that of its drawn point. Stalks are drawn before every pin so a head always covers a stalk. Open: a stalk is not itself a hit target; the merge is measured on stacks, not on every spread of tips; a knot over 36 stays upright (the declutter thins the map first). Probe `_fanring_probe.tscn`, frames `evidence/city-look-2026-11-13/pinrings/`.

**Build note, 2026-11-15 (the seventh build step: the owner's icon sets and Ruling FC; status is `STATUS.md`'s).** *Icon sets:* the owner's two sheets (and a third, painted-only, for the kinds that had none) are cut into `icons/pins/{glyph,painted}/`, 128 px tall with the tip at the bottom-centre pixel, and offered as the Marker styles "Pins: icons" and "Pins: painted". Design choices: the texture is the same HEIGHT as the drawn pin, so a style change does not move the tip; each picture's own head radius and tip distance are measured from its alpha (the pins differ, 1.4 to 1.9 radii), and that, not one constant, is the hit footprint and the tilt-collision shape; a set only shows kinds it has a picture for (a file named like the kind, else a close alias), never another picture ("Fort can't borrow"); a linked note is a small orange badge. *Ruling FC:* a pin belongs to a town by lying within half the site box (0.85 km) of a settlement, because the engine ties none to one, and it waits for `_urban_plan_update`'s own zoom gate; the City Viewer is always shown.

**Build note, 2026-11-15 (Sea and lake draw smooth; status is `STATUS.md`'s).** The owner's *"Sea and lake should not draw as squares"*: `shell/urban_shore.gd` draws the engine's `water_mask_runs` (about 22 m map-cell rows) as smoothed, cached polygons in both looks (the ink plan's `_draw_water_mask` and the map look, which calls it), and smooths the `water_poly`/`river` outlines of a coastal site when they are the traced staircase, so the map look's shore band and the ink river edge follow the same curve. Judgements, all labelled in the file: 8 averaging passes held within 0.3 cell of the traced boundary, then 2 Chaikin passes; the half-cell bound is the owner's and is asserted both ways by `_shorepoly_probe.gd`. The look's colours and band widths are untouched.

**Build note, 2026-11-15 (CL-12: the pin declutter; status is `STATUS.md`'s).** From the coordinator's frame of the phone at its fit zoom, where generated landmark pins stood in stacks of dozens. Pin size is equal in dp on every form (`bc32d352`), so the cause is density and the cure is a screen-space rule in `map_overlay.gd` (the "Pin declutter" block): rank by always-keep (selected, linked note), class, importance, score, index; walk the order and keep a pin unless a kept pin's head is within 1.5 x the radii sum or a tip sits on its head. It is a distance rule, so it is stable under a pan and the same on every form (the phone gets fewer pins because its window holds fewer head-widths); zooming in shrinks the head in plane units and pins return. One predicate (`_lm_hidden`, with Ruling FC's town gate) hides a pin from the draw, the hit tests, `hits_at` and Ruling FA's tilt. Measured at the fit zoom on seed 1200: phone 463 -> 155 drawn (50 in the window, was 135), tablet 463 -> 138, desktop 450 -> 131; no two shown heads overlap. Not applied to the Circles style, an export, the City Viewer or user-placed POIs. No count badge. Probe `_declutter_probe.gd`, eight literal mutants killed, one equivalent; frames in `cartalith-native/docs/evidence/pin-declutter-2026-11-15/`.

**Build note, 2026-11-16 (CL-12's leftovers: the export and the Circles style; status is `STATUS.md`'s).** The rule's gate (`_declutter_active`) no longer excludes an export or the Circles style. In an export the walk reads the copy's own sizes, which are output pixels (landmark heads carry `_sym_gain` with no floor, settlement heads take the tier size), so the image keeps what a screen of that size would; in Circles it walks the same order with disc footprints (head = tip) at `DECLUTTER_DISC_SPACING` 1.15 x the radii sum, a labelled judgement, and a thinned disc becomes the faction dot. At 2048 x 1536 the desktop export goes 457 landmarks + 240 settlements to 110 + 173 with worst overlap 1.00 to 0.00; Circles at the fit zoom 457 + 19 to 176 + 17. Found on the way: the cached set did not see a note linked in Circles (`_pin_notes_rev` moves only when something reads the notes), now read in `_declutter_sig`. Probe `_declutter_probe.gd` legs C and X; frames in `cartalith-native/docs/evidence/pin-declutter-circles-export-2026-11-16/`.

**Build note, 2026-11-15 (CL-10 and the default look: Ruling FF items 1 and 2; status is `STATUS.md`'s).** *Settles Q1's map half and the default-look call.* In the three Pins styles (drawn Pins, Pins: icons, Pins: painted) the map's towns and villages are teardrop pins at every zoom, in place of the disc, glyph and LOD dot; the Circles style keeps its discs byte for byte. Design choices, each a labelled judgement: the pin is filled with the **faction colour** (the disc's identity), with the pictogram ink picked by contrast (an icon set's picture carries its own colours, so the faction is a small badge on its head instead); the drawn pin's pictograms are a castle (capital, metropolis), a skyline (city), a house (town), a hut (village) and a dot (hamlet), and the icon sets use `KIND_ICON`'s tier rows; the head is the reference's own tier size (`(4 + rank) * sc`) or the readable floor `PIN_HEAD_MIN_PX` grown 12 % per tier, whichever is larger. The capital ring, faith ring, coastal badge and the new-settlement halo are drawn around the head, the trait badges under the whole pin, and the name beside the pin (right, left, above, below, in that order) in a box that starts a gap outside the pin's own box, which the pin also reserves for later names. Settlement pins tilt with their neighbours (Ruling FA, `_fan`'s `s_a`) and are thinned by CL-12's walk, now one walk over landmarks and settlements on one rank scale: capital and metropolis 6 (always kept), city 5, town 3 (level with continental), village 1.5, hamlet 1.1 (above local and cultural), a selected settlement always kept; a thinned settlement stays as the faction dot (the road anchor), not picked. Picking is the drawn pin (head and stem, tilted) in `_hit_test_settlement` and `hits_at`; the cross-kind comparison with a landmark or icon still uses the anchor (tip) distance. Ruling FC's town-pin hiding is unchanged and applies to a town's own landmark and POI pins, never to the settlement's pin. The ghost pass and the hover card keep their shapes. `PinMarker.draw` gained `fill_col`, `ink_col` and `min_line` (a screen-pixel stroke floor, since a settlement head is under 3 local px at x4 and a one-unit floor ate it). **Item 2:** `DccSettings.city_look()` now defaults to `"map"`; a stored `"ink"` is read back and kept, a corrupt value reads `"map"`, and both stay selectable (Preferences, the City Viewer's picker). Probed by `_townpins_probe.gd` (desktop 1600x900 and phone 1080x2340, all three Pins styles) with named mutants; frames in `cartalith-native/docs/evidence/town-pins-2026-11-15/`.
