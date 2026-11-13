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
| 8 | **Harbour and piers** | `urban_bridge.rs::layout_dict` crosses `harbour_pt` and the `quay` street class; `growth.rs` builds piers and a mole (its doc says so) and `hinterland.rs` places a bollard per pier. **Piers are not crossed**; `urban_layout_draw.gd` never reads `harbour_pt` | Cross the pier polylines in the bridge (a new key, additive), draw planks as short cross-strokes and a moored boat as a stamp. **No generated value moves**, the geometry already exists | M | the draw is; the crossing is a Rust change, not a golden change |
| 9 | **Title** | The City Viewer's window title bar (`_title_label`); no title on the map or in an export | A serif stroked title as an overlay in the viewer (and optionally the export). Needs a serif font: the project bundles only Fira Sans, Fira Code and IBM Plex Mono (`godot-project/fonts/`); the mock used the OS's Georgia through `SystemFont`, which is not portable to Android | S (draw), S (a licensed bundled font) | **yes** |
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
| **CL-11** | **User-placed city pins and Poi** (only if Q5 is answered yes) | A `Poi` entity kind (a seventh `EntityKind`, which moves the guard count in `links.rs`'s doc), save round trip byte-identical on a real prior-format save, undo, a note bound by id |

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
| **Q7** | The serif font for the title | A bundled **OFL** face; two candidates to confirm the licence of before bundling: *Cinzel* and *IM Fell English*. Not checked in this pass |
| **Q8** | Decorative trees outside the wall, to match the reference's groves, drawn but not part of the generated world? | **Yes**, seeded from the layout, never saved, never read by any engine path, included in export so export follows screen |
| **Q9** | Cross the engine's pier geometry into the layout so piers can be drawn? | **Yes**: additive, no generated value moves |

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
