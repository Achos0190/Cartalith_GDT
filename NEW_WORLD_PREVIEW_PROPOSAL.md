# New World seed preview: proposal

**A proposal of 2026-10-11, from an owner request. Not scheduled.** It defines
milestones (NWP-1…NWP-4) and the reasoning behind them; it carries no status
column. Status, once any of this is scheduled, lives in
`cartalith-native/docs/STATUS.md` and nowhere else.

**Owner request, verbatim (2026-10-11):**

> Can we also do a crude preview in the loading/new world screen. Nothing
> dressed up just quickly based on a number of plates (let's take 36 for world
> and 6 for region of 200km, scale the other size options) just gray on blue to
> have a rough idea of landmasses vs ocean in a 320,px resolution? Just to get
> an idea of the possible result of a seed? Write me a proposal and mockup of
> this New World/new project screen and how it compares to the current

**Mockup:** `design/new-world-preview-2026-10-11/mockup.html`. One static page,
no external files. It has a *Current / Proposed / Both* switch and a
*Desktop / Phone* switch. Its preview canvas is live, but **it is drawn by a
JavaScript stand-in, not by the engine.** The page says so twice. §4 explains
why the build should use the engine instead.

**Evidence base.** Each figure below says where it came from:

- **Screenshots:** a windowed Godot probe that I ran today on a scratch copy of
  `godot-project`. Its user dir is `NWPREV GD Scratch` and the live user dir was
  not touched.
- **Timings and overlap percentages:** an ignored Rust harness that I wrote and
  ran today, `crates/cartalith-godot/tests/nwprev_cost.rs`. I built it into a
  scratch target with `-j 2`.
- **Facts about the code:** each one names the symbol I opened. Nothing in this
  document is measured unless it says so.

---

## 1. The current screen

### 1.1 How it is built

The screen is `godot-project/shell/new_world_dialog.gd`, class `NewWorldDialog`,
an `AcceptDialog`. `app.gd::setup` builds it once at startup and reuses it.
File ▸ New world… opens it through `app.gd::open_new_world()`, as do the open-project
gallery tile and the phone picker's "new world" tile.

- **Desktop and tablet.** A 620 × 780 window (`setup()` sets
  `size = Vector2i(620, 780)`). It has two advisory notes (`NOTE_CREATION_ONLY` and
  `NOTE_APPEARANCE_KEPT`) above one scrolling form. The form's sections, in order:
  - **Seed:** a `SpinBox`, 0 to `SEED_MAX` = 2 000 000 000. It starts on
    `randi() % 1000000`.
  - **Extent:** an `OptionButton`, *Region* / *Whole world*. This is the live
    engine parameter `world`, so changing it is a `param_set`. A note follows,
    `EXTENT_NOTE_REGION` or `EXTENT_NOTE_WORLD`.
  - **Map width & resolution:**
    - *Map width* takes one of the `SIZE_PRESETS`: Local 200, Province 800,
      Region 2 000, Subcontinent 5 000, Continent 12 000 and Planet 40 075 km,
      plus Custom.
    - *Width (km)* is a free entry from 1 to 100 000. It is the same value as
      Map width; the two stay in step.
    - *Resolution* takes one of the `RESOLUTION_PRESETS`: 512, 1K, 2K, 4K or 8K.
      The default is 2K.
    - *Grid columns* runs from `GRID_MIN` 4 to `GRID_MAX` 8192.
    - *Aspect* takes one of the 7 `ASPECT_PRESETS`. Region defaults to 1.5625:1
      and Whole world moves to 2:1. Both come from `reference_grid_height`.
    - *Grid rows* is derived from Aspect unless the user types a value.
    - A derived panel shows Grid, Extent, Cell size and Aspect.
    - A warning appears at 4K and 8K.
  - **World structure:** an *Archetype* menu, Classic or one of the five
    `ARCHETYPE_LABELS`.
  - **Generation:**
    - three toggles: villages, metropolis and biome-K;
    - *Recovery phase*;
    - *Factions*, *Suitability floor* and *Packing*;
    - *Fixed settlement counts*, with five count boxes;
    - a closing note.

  The button row is `AcceptDialog`'s own: one **Create** button.
  `_on_create()` calls `generate_checked()`, which calls `bridge.generate(request())`.
- **Phone.** `DccWidgets.phone_window()` and `_phone_card()` give §6.7's card, at
  most 360 dp wide (`PHONE_CARD_MAX_W`). The card holds:
  - Seed, with a dice button. `_build_seed_dice()` is called only `if _phone`.
  - Two extent chips, REGION and WORLD, from `_build_extent_chips()`.
  - Size & resolution: Map width, Width (km) and Resolution.
  - The derived panel.
  - Archetype.
  - Both advisories.
  - CANCEL and CREATE WORLD pinned at the bottom of the card, from
    `_build_phone_actions()`.

  Everything else is built into a hidden `rest`, because `request()` still reads
  those controls.

**Generation itself:**

1. `EngineBridge.generate()` snapshots the parameter table.
2. It sets `generating` and emits `generation_started`.
3. It starts a worker `Thread`. That thread owns `WorldGen` until `_finish`.

While the worker runs, the busy overlay (`busy_overlay.gd`) covers the whole
viewport and swallows input. It shows a scrim and a card with one witty line per
stage. It also shows a *relief trace*, a mountain-profile progress bar, and a
line reading "stage NN of 10 · <name>". The WORLD dock's *Pipeline status*
section, `world_workspace.gd`, lists the ten stages with a state dot and the
elapsed time for each: 01 Planet, 02 Extent & scale, 03 World structure,
04 Tectonics, 05 Volcanism & impacts, 06 Erosion, 07 Hydrology, 08 Climate,
09 Ecology & biomes and 10 Resources & soils. The stage boundaries come from
`cartalith-engine/src/progress.rs`.

### 1.2 What the screenshots show

Probe: `_nwprev_probe.gd` in the scratch copy. It boots `app.tscn` in a
`SubViewport`, calls `app.open_new_world()` and saves the frame.

> **Disclosed:** it calls a function to open the dialog. So these are pictures
> of the screen. They are not a reachability proof (`MISTAKES.md`, "Reach a
> screen inside a probe").

It ran windowed. Both runs printed `RESULT … checks=5 fail=0`. The screenshots
are in the scratchpad (`…/scratchpad/nwprev/shots/`), not in the repository.

**Desktop, 1600 × 900** (`desk_01_dialog_top.png`, `desk_02_dialog_bottom.png`,
`desk_03_dialog_world_planet.png`).

- The dialog sits centred at (490, 60), 620 × 780. A world-less shell is behind
  it, with an empty map area. On a first run there is no map at all to judge a
  seed by.
- The two advisories take four lines of small, faint text before the first
  control.
- The form scrolls: the probe measured **954 px of content in a 659 px
  scroller**. About a third of the form is below the fold at open. That third is
  the Generation block: settlement toggles, Factions, two placement sliders and
  five count boxes, each reading `0`.
- The seed is a bare six-digit number in a 380 px field. **There is no dice on
  desktop.** The roll exists, but only as *New seed* in the GENERATE · WORLD tool
  bar and the WORLD dock, both outside this dialog.
- The only action is a centred **Create** with an accent outline. There is no
  Cancel; the title bar's ✕ is the way out.
- Choosing *Whole world* and *Planet* fills the derived panel correctly
  (2048 × 1024, 40 075 km × 20 038 km, 19.6 km per cell). The extent note
  changes to the world text.
- **Nothing on the screen shows what the seed will produce, and there is no
  plate count anywhere in the dialog.** `tect.plates` lives in World ▸ Generate ▸
  04 Tectonics. Its default is 14 for every map width
  (`GENERATION_PARAMETERS.md`, Group `tectonics`).

**Phone, 1080 × 2340 `--force-touch`** (`phone_01_dialog_top.png`,
`phone_02_dialog_bottom.png`; content scale 2.621).

- A full-screen sheet carries the `NEW WORLD / creation-time settings` head and
  §6.7's rounded card.
- Inside the card:
  - **Seed:** the field shows 902913. A 44 dp dice button sits beside it. Its
    icon is a small two-pip die, legible in a full-resolution crop.
  - **Extent:** REGION and WORLD chips. REGION is lit.
  - **Size & resolution:** Map width, Width (km) and Resolution.
  - The derived panel.
  - **Archetype**, with its note.
  - The two advisories, which begin at the bottom of the scroller.
- CANCEL and CREATE WORLD are pinned below the scroller.
- The card's scroller measured **653 dp of content in 592 dp**, so the
  advisories are partly below the fold at open.
- A band below the card stays empty in the screenshot.
- As on desktop, nothing previews the seed.

### 1.3 What is wrong with it for choosing a seed

1. **A seed is a number with no picture.** To learn what a seed gives, the user
   must Create, wait for the whole ten-stage pipeline, look, reopen the dialog,
   change the number and repeat. Measured today, a CPU generate at the dialog's
   default 2K width costs **about 4.1-4.2 s (medians 4 192 ms region, 4 118 ms world)** per try (§6). On a phone it is slower, and
   above 2048 × 1311 a memory question comes first (Ruling AS).
2. **The control that most decides the landmass count is not in the dialog.**
   `tect.plates` sets the number of Voronoi plates. Each plate is continental or
   oceanic: `build_plates` makes a plate oceanic when `rng.next_f64() < 0.45`.
   The plate count stays at 14 whether the map is 200 km or 40 075 km.
3. **On desktop the roll is outside the dialog.** The dice is phone-only
   (`_build_seed_dice()` is called `if _phone`).
4. **Terrain choices and settlement choices share one form.** The Generation
   block, about 300 px of settlement placement, sits in the same scroll as seed
   and size. It is creation-time only in the sense that Create reads it; it does
   not decide the land.

---

## 2. The proposed screen

The proposal keeps every control the dialog has today. It adds one preview, a
plate row and a seed history. It moves the settlement block behind a disclosure.
It does not delete a surface, so `MISTAKES.md`'s "Delete a surface" inventory
does not apply. Every control keeps a home.

### 2.1 Desktop (and tablet landscape)

The window is about **980 × 780**: the same height as today and wider. It has two
columns.

- **Left, 660 px: the preview.**
  - A header: `§ PREVIEW · LAND VS OCEAN · 320 × 205`. The height follows the
    extent: 205 for a region (`round(320 × 0.64)`) and 160 for a world (2:1).
  - The 320-pixel canvas drawn at 2×, with nearest-neighbour filtering.
  - A stats line: land %, plate count, extent, and the draw time.
  - The **Seed** row: the field, then a **dice** button, then a **lock** button.
  - A **history strip** of the last four previews. Each is a thumbnail with its
    seed and land %. Clicking one restores that seed, extent, width and plates.
- **Right, about 280 px: the settings.**
  - **Extent:** a segmented pair, as on the phone.
  - **Map width**, using the same presets as today.
  - **Plates:** − *n* +, with a source label, "auto from width" or "set by hand ·
    reset".
  - **Resolution.**
  - The derived panel.
  - **Archetype.**
  - **▸ More options:** Aspect, Grid columns, Grid rows, and today's whole
    Generation block, unchanged.
  - The two advisories, folded into one line with the full text on hover.
- **Footer:** **Cancel** and **Generate world**. Generate world is the accent
  fill, rightmost. It is still `AcceptDialog`'s OK button underneath, so
  `confirmed → _on_create()` is unchanged.

**Tablet.** In landscape it uses the same two columns, floored to the touch
targets: `tablet_fit()` floors height only, and controls that a rebuild creates
must be floored at the call site on both axes (`MISTAKES.md`). In portrait it
uses the phone's single column with the canvas at 2×.

### 2.2 Phone

§6.7's card, in this order:

1. **The preview canvas, full card width.** The card's inner width is 328 dp, so
   the 320-pixel preview maps to about 1 dp per preview pixel.
2. A one-line stats readout.
3. The Seed field, a 44 dp dice and a 44 dp lock.
4. The four-thumbnail history row.
5. The REGION and WORLD chips.
6. Map width, then Plates (− *n* + at 44 dp), then Resolution.
7. The derived panel and Archetype.
8. CANCEL and CREATE WORLD, pinned as today.

The preview takes about 210 dp of the 592 dp scroller. So the dialog's lower
rows scroll on most phones, and the pinned action row is unaffected. **Not
verified:** a phone probe cannot press controls inside an embedded
`AcceptDialog` (`MISTAKES.md`, "Plan a phone probe that must reach a DIALOG"),
so the phone legs are "it renders" and on-glass "it can be operated".

### 2.3 Before and after, at a glance

| | Current | Proposed |
|---|---|---|
| What a seed looks like before Create | nothing | 320 px grey-on-blue land/ocean mask, redrawn on every change |
| Roll a new seed (desktop) | not in the dialog (tool bar *New seed*) | dice beside the field |
| Roll a new seed (phone) | dice | dice, and the preview redraws |
| Compare seeds | from memory, one Create per seed | last 4 as thumbnails; click to restore |
| Keep a seed you like | write it down | lock: the dice, and *New seed* while the dialog is open, leave it alone |
| Plate count | not in the dialog; 14 at every width (04 Tectonics) | in the dialog; from map width (§5), overridable, the same engine parameter |
| Settlement placement block | inline, about ⅓ of the scroll | behind ▸ More options |
| Actions (desktop) | **Create** only, plus the ✕ | **Cancel** and **Generate world** |
| Dialog footprint (desktop) | 620 × 780; form 954 in 659 px | about 980 × 780; first screen without scrolling (to be measured, NWP-3) |
| Cost per look at a seed | one full generate (§6) | one preview (§6), 16.5 ms (region) and 38.8 ms (world) median |

---

## 3. The preview's behaviour

- **What it draws.** Grey (#8f9295) land on blue (#2b4d6b) ocean, 320 pixels
  wide, and nothing else: no relief, rivers, labels or ice. The owner said
  *"nothing dressed up."*
- **When it redraws.** On every change to an input it depends on, with a short
  debounce so that a held spinner does not queue thirty runs. Seed, extent, map
  width, plates and resolution are the dialog's own. The rest are the live
  parameters that the preview's stages read:

  > `sea_level`, `tect.warp`, `tect.blur_r`, `tect.vel`, `tect.alpha`,
  > `tect.beta`, `tect.age_inf`, `tect.flexure`, `tect.hetero`, `tect.ridged`,
  > `tect.lloyd`, `tect.narrow_plate_base_blur`, `world_structure.*`, and the
  > archetype.

  That list is derived from the arguments of the functions the preview calls
  (§4.1). `MISTAKES.md` says a staleness key must be derived from the
  definition, because a key written from memory misses inputs. If preview and
  Create ever read different values, the preview lies. They must read the same
  `param_get`.
- **Dice.** It rolls a new seed (`randomise_seed()`, unchanged), redraws, and
  pushes the previous preview into the history.
- **History.** The last 4 previews, in this session, in the dialog only. Each
  stores seed, extent, width, plates and the 320-wide mask; restoring one sets
  all four. A history entry is pushed only by a dice roll, a typed seed or a
  size change. Restoring an entry does not push one, so the strip does not churn.
- **Lock.** While it is on, the dice is disabled and `randomise_seed()` is a
  no-op, which also covers the tool bar's *New seed*. A typed seed still wins:
  the user typed it. Whether the lock outlives the dialog is open question Q4.
- **Generate.** It runs exactly today's `generate_checked()`. The preview does
  not change what Create sends, with one exception: if Q1 is answered yes,
  `tect.plates` is set from the width rule (§5), and that changes generated
  output for every width except 800 km (§7, risk 1).
- **Archetype.** When an archetype is chosen, the engine ignores `tect.plates`
  and uses `clamp(round(4 + fragmentation·24), 4, 40)`
  (`generate_terrain_inner`; `GENERATION_PARAMETERS.md` Group `world_structure`).
  The Plates row then shows that number read-only, labelled "from archetype".
  The preview must run the World-Structure path too: it needs
  `generate_continentality_field` and the plate re-basing in `build_plates`.
  **That path was not measured here** (§4.3).
- **Honesty label.** The canvas header says what it is: "land vs ocean · plate
  layout". The size of the disagreement with the finished world, measured in §4,
  goes in the tooltip, so the label is a claim the user can check.

---

## 4. Faithfulness: is it the real thing or a lookalike?

### 4.1 What the engine's early stages actually do

`generate_terrain_inner` (`cartalith-engine/src/lib.rs`), CPU path, World
Structure off, up to the `VOLCANISM` banner, runs these steps in order:

1. **`compute_warp`.** The warp amplitude is `warp·0.18·gw` cells and its
   frequency is `2.5/gw`, or `3/gw` for a world. That makes the warp fixed in
   fractions of the map: it does not depend on resolution.
2. **`build_plates(gw, gh, seed as u32, n, lloyd, world, ws)`.** It places `n`
   plates from one `Mulberry32` stream. Each plate is oceanic with probability
   0.45, at a position of `rng·gw` and `rng·gh`. That is the same fraction of the
   map at any resolution. Then it runs `lloyd` relaxation passes over the integer
   cell grid.
3. **`assign_plates`.** A jump-flood Voronoi through the warp produces
   `plate_id: Vec<u16>`.
4. **`compute_stress`.** It finds boundaries and their types and blurs stress at
   radius `tect.blur_r`. Then `compute_flexure` blurs at `3·blur_r`.
5. **The plate-base field.** It is `plates[pid].base`, blurred at
   `plate_base_blur_r = max(2, blur_r·K)`. K is 0.18 in the app.
6. **`build_age_field`, `compute_heterogeneity` and `compute_height`.**
   `compute_height` takes `nf = 5·terrain_detail_k(gw, km)`.
7. **`normalize_field`.**

Land is `field > sea_level`, which is 0.42 in `params::defaults()`.

**The resolution trap is in two places.**

- **`tect.blur_r` is a radius in cells.** Its default of 18 is 0.9 % of a
  2048-wide map and 5.6 % of a 320-wide one.
- **`terrain_detail_k(gw, km)` depends on cell size.** For a 200 km region it
  returns 1 at 320 wide, 2 at 1024 and 4 at 2048. So the preview's relief noise
  would run at a different frequency from the real world's.

Running the pipeline unchanged at 320 is therefore *not* the same world drawn
smaller. Correcting both is two multiplications:

- multiply `blur_r` by `320 / target_gw`;
- pass `km · 320 / target_gw` wherever `km` reaches `terrain_detail_k`, so that
  `nf` and the heterogeneity frequency are the target grid's.

Everything else in the list is already independent of resolution.

### 4.2 The measurement

**Harness and method.**

- **Harness:** `crates/cartalith-godot/tests/nwprev_cost.rs`, test
  `nwprev_overlap`. It is `#[ignore]`d. I ran it alone in release, once against
  1024-wide targets and once against `NWPREV_TARGET=2048`.
- **Parameters:** `params::defaults()`, the app's own world, on the CPU path.
  Plates are 6 for a 200 km region and 36 for a 40 075 km world. The five seeds
  are 1, 42, 1234, 51127 and 987654.
- **Metric:** the percentage of the 320-wide cells whose land/ocean class
  matches the target-width world's class at that cell's centre.
- **Controls:** each row is printed beside two references, because a bare
  percentage on a map that is half ocean proves nothing (`MISTAKES.md`, "Invent
  a screening metric").
  - *Chance:* the agreement that two unrelated masks with the same land
    fractions would reach. It came out at 50 to 60 %.
  - *Negative control:* this seed's preview against the *previous* seed's
    finished world.

**The arms.**

- *scaled*: the early stages at 320, with the two cell-unit corrections from
  §4.1. **This is recommendation (a).**
- *naive*: the early stages at 320, uncorrected.
- *plates only*: the sign of each cell's plate after `assign_plates`. This is
  the crudest form that is still faithful.
- *full320*: the whole `generate_terrain` at 320.

The reference in every row is the **finished world**, `generate_terrain`, at the
target width with the same seed.

**Agreement with the finished world, % of cells. Median (min..max) over 5 seeds:**

| Case | Target | **scaled (a)** | naive | plates only | full320 | chance | negative control |
|---|---|---|---|---|---|---|---|
| Region 200 km, 6 plates | 1024 × 655 | **97.7 (93.0..99.0)** | 93.9 (91.4..96.8) | 95.0 (87.3..97.0) | 93.9 (91.2..96.5) | 50.3..59.5 | 29.8..71.7 |
| Region 200 km, 6 plates | 2048 × 1311 | **97.7 (95.8..98.5)** | 90.7 (86.5..97.2) | 96.6 (93.7..98.1) | 91.5 (86.3..96.9) | 50.1..59.8 | 25.6..71.4 |
| World 40 075 km, 36 plates | 1024 × 512 | **98.2 (97.0..98.6)** | 89.9 (87.4..93.9) | 96.3 (93.2..97.2) | 89.8 (87.2..93.9) | 49.8..53.5 | 44.6..49.6 |
| World 40 075 km, 36 plates | 2048 × 1024 | **97.6 (96.9..98.3)** | 87.6 (86.9..93.1) | 97.9 (93.6..98.4) | 87.5 (86.7..93.0) | 49.7..54.5 | 44.8..51.0 |

**Reading the table.**

- **Scaled agreement is 93.0 to 99.0 %.** Land IoU for the scaled arm at 2048:
  91.5 to 96.8 % for the region and 94.2 to 96.9 % for the world.
- **The early stages already decide the coast.** The early stages at the target
  width agree with the finished world at 96.7 to 99.9 %. Everything after
  tectonics moves very little of the land/ocean split at these settings:
  volcanism, erosion, hydrology and the rest.
- **The same seed at two resolutions gives the same plates.** Plate-sign masks at
  320 and at the target agree at 98.9 to 99.9 %.
- **Running the whole pipeline at 320 is *less* faithful than the corrected
  early stages.** It reads 86.3 to 96.9 %. The uncorrected cell-unit parameters
  cost more than skipping erosion does.
- **The negative control sits at chance or below.** It reads 25.6 to 71.7 %
  against a chance of about 50 %. The 71 % cases are two seeds whose land
  fractions differ widely. So agreement in the high 90s is information about
  this seed, not a property of the metric.

**Not measured** (these remain claims for NWP-1 to settle):

- **The GPU path.** The shipped shell turns `use_gpu` on at boot. Its stage
  outputs are principled-equivalent, not bit-identical.
- **Archetypes and World Structure.** Their sea level is also re-anchored by
  `apply_world_structure_sea_level` after the early stages.
- **Other sea levels, extents, plate counts and resolutions.** Only these two
  cases and two target widths were run, and only five seeds of each.
- **Where the 2 to 4 % of disagreeing cells lie.** The expectation is
  coastlines and small islands. That is a suspicion, not a measurement.

### 4.3 Recommendation: (a), the real early stages at low resolution

**Recommend (a).** Run the engine's own early stages at 320 wide, with the two
cell-unit inputs rescaled to the grid that Create will build. Classify with the
same `sea_level`. With the evidence above, the preview can honestly be labelled
*"the plate layout and land/ocean split this seed gives"*. It is not the
finished map cell for cell. It agrees with it on about 97 to 98 % of cells, with
a worst case of 93 % in the cases measured. It will not show the following:

- coastline detail finer than 1/320 of the map;
- islands smaller than about one preview pixel: 0.6 km at 200 km and 125 km at
  40 075 km;
- lakes, rivers or relief.

**Why not (b), a separate lookalike** like the mockup's JavaScript? A separate
Voronoi with its own random stream would show *a* plausible map, not *this
seed's* map. It would also drift from the engine with every plate or warp change.
Its cost advantage does not exist: §6 measures the real stages at 320 at
16.5 ms (region) and 38.8 ms (world) median. The mockup uses a lookalike only because a static page cannot call
the engine.

**Fallback if NWP-1's measurement fails on the GPU path or for archetypes.**
Use the *plates only* mask (`build_plates` plus `assign_plates`). It needs no
blur, no noise and no GPU, and it still reached 87.3 to 98.4 % with seeds
agreeing across resolution. Label it "plate layout" rather than "land".

---

## 5. Plate count from map width

The owner's two anchors are **(200 km, 6)** and **(world, 36)**. The planet preset
is 40 075 km, so the world anchor is taken at 40 075 km. Any rule through two
points is a judgement. Two candidates, with every preset the dialog offers:

| Map width preset | km | **Log-linear (recommended)** | Power law | Today (`tect.plates`) |
|---|---|---|---|---|
| Local | 200 | **6** | 6 | 14 |
| Province (the dialog default) | 800 | **14** (13.85) | 10 | 14 |
| Region | 2 000 | **19** | 13 | 14 |
| Subcontinent | 5 000 | **24** | 18 | 14 |
| Continent | 12 000 | **29** | 24 | 14 |
| Planet | 40 075 | **36** | 36 | 14 |

- **Log-linear:** `plates = clamp(round(6 + 30 · ln(km/200) / ln(40075/200)), 4, 40)`.
  Each doubling of width adds about 3.9 plates.
- **Power law:** `plates = clamp(round(6 · (km/200)^0.338), 4, 40)`. The
  exponent is `ln 6 / ln 200.375`, which makes the curve pass through both
  anchors.

**Why log-linear:**

1. **It passes through today's default world.** 800 km gives 13.85, which rounds
   to 14, today's `tect.plates` default. So **a default Create is unchanged**.
   Only non-default widths change. The power law would change the default world
   from 14 plates to 10.
2. **It suits how the count reads on screen.** The count's job is "how many
   landmasses do I see", and a plate's share of the map shrinks multiplicatively
   with width. Both rules are labelled judgements. Neither is physics:
   - this engine's plates are Voronoi cells over the frame, not lithospheric
     plates;
   - six plates in a 200 km region is geologically nothing like Earth.

   The owner's anchors are the authority here.
- **Custom widths** use the same formula, clamped. Below about 129 km the count
  is 4. It reaches the cap of 40 only near 89 000 km, close to the field's
  100 000 km maximum.
- **The engine accepts every value in the table.** `tect.plates` is
  `Kind::Int`, 4 to 40, step 1 (`params.rs` `PARAMS`;
  `GENERATION_PARAMETERS.md`), so 6 to 36 fit. **Archetypes do not reach 36.**
  Their derived count is `round(4 + frag·24)`: 6 for supercontinent, 12 for rift,
  16 for earth, 26 for archipelago and 28 for volcanic. That is why the Plates
  row turns read-only under an archetype (§3).
- **The "Whole world" question.** Should *Whole world* always mean 36,
  whatever its width, or should the count follow the width so that a
  12 000 km planet gets 29? See Q2.

---

## 6. Cost

**Harness and method.** Same harness, test `nwprev_cost`, release, run alone:
run 1 started with no other `cargo`, `rustc` or Godot process on the machine (checked by `tasklist` immediately before and after); run 2 ran while another session's `rustc` was compiling and is shown only as a contention check. 16 rayon threads, this Windows desktop (Godot's log on it reports an AMD Radeon RX 7800 XT; the timed path is CPU only).

- **Early stages:** warp to `normalize_field`, as in §4.1. Each figure is the
  median of 10 runs over 5 seeds, with min..max.
- **Full generate:** `generate_terrain`, the whole CPU pipeline, median of 5.
- **Default width:** the full pipeline at the dialog's default 2048, median of 3.

| Case | Early stages @ 320 | Full `generate_terrain` @ 320 | Early stages @ 1024 | Full generate @ 2048 (today's cost per look) |
|---|---|---|---|---|
| Region 200 km, 6 plates (320 × 205) | **16.5 ms (15.9..17.2)**; run 2: 16.0 (15.3..16.4) | 193.2 ms (176.9..213.4); run 2: 190.7 | 181.6 ms (181.4..183.6) | 4 192 ms (3 793..4 264); run 2: 4 120 |
| World 40 075 km, 36 plates (320 × 160) | **38.8 ms (29.9..39.9)**; run 2: 31.0 (30.0..33.5) | 186.4 ms (177.5..201.4); run 2: 190.9 | 413.6 ms (348.5..436.3) | 4 118 ms (3 928..4 276); run 2: 4 198 |

So a corrected early-stage preview at 320 costs **about 16 ms for a region and 30-40 ms for a 36-plate world** on this desktop: roughly 100-250 times cheaper than one look at a seed today (about 4.1-4.2 s at 2K, CPU). The world case is slower mainly because `build_plates`' Lloyd relaxation is brute force, O(iterations x cells x plates), and 36 plates is six times 6 (a suspicion from the code, not a profile). Running the *whole* pipeline at 320 costs about 190 ms and is, per §4.2, less faithful, so it buys nothing. Previewing at the target width instead (early stages at 1024) would cost 180-420 ms and is not proposed.

**Where it runs.** The preview must not touch the `WorldGen` instance while a
generate's worker thread owns it. That would be the `Gd<T>::bind() failed` panic
that the busy overlay exists to prevent.

The binding should therefore be a **static** `#[func]`, which takes no `&self`,
or a pure function behind its own small `RefCounted`. Its inputs are a parameter
dictionary, the preview width and the target width. It returns a
`PackedByteArray` mask.

- At these costs it can run on the main thread behind a debounce of about
  120 ms.
- If NWP-1 measures a phone, or the GPU path, above one frame budget, it moves
  to `WorkerThreadPool` with a sequence number. A stale result is then dropped
  rather than drawn.

**Not measured:**

- **On a phone.** No handset was involved today.
- **With the GPU path.** It would add the device handshake that
  `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 8 measured at about a second per
  generation. A preview should simply use the CPU stages.

---

## 7. Risks

1. **Changing generated output (needs the owner).** If the width rule writes
   `tect.plates` (Q1, recommended yes), every Create at a non-800 km width
   produces a different world from today. No golden moves:
   - goldens compare `WorldParams::defaults`;
   - the dialog would write a live parameter, as it already does for `world`.

   It is still a behaviour change to the shipped product, so it needs the owner's
   explicit yes (`MISTAKES.md`, "Change generated output"). The alternative is to
   preview with the scaled count but generate with 14, a preview that lies.
   **This document rejects that alternative.**
2. **The preview's inputs drift.** If someone later adds a parameter to the
   early stages without adding it to the preview's key, the preview goes silently
   stale. Mitigation: NWP-1 builds the preview by *calling the same functions in
   the same order*. It does this through a shared, crate-private
   `tectonic_substrate()` that `generate_terrain_inner` also calls, not a copy.
   A test mutates each input and asserts that the mask moves.
3. **The two-copies trap during the refactor.** Extracting
   `tectonic_substrate()` out of `generate_terrain_inner` must be identity by
   control flow. Verify by hashing a full `generate_terrain` output before and
   after at three seeds, on the CPU and the GPU paths (`MISTAKES.md`, "Change
   generated output").
4. **Dialog size.** 980 × 780 fits 1600 × 900 and tablet landscape. It does not
   fit a 1280 × 720 desktop, but today's 780 px height does not fit that either.
   NWP-3 measures the minimum size. The left column can shrink the canvas to 1×
   (320 px) below about 1100 px of width.
5. **Users read the preview as a promise.** It is about 97 % right in the
   measured cases and fails at fine scale. The tooltip states the measured
   agreement, and the header says "land vs ocean", not "map".
6. **Archetype and World Structure behaviour is unmeasured.** NWP-1 measures it
   or the Plates row falls back to "plates only" for archetypes (§4.3).
7. **Phone reachability.** Probes cannot press inside an embedded dialog on a
   phone. Operation is verified on glass (§8, NWP-4).

---

## 8. Milestones

Each batch is independently verifiable and has at most 2 agents. The lanes are
file-disjoint, plus one verifier. Rules that must be in every brief, from
`MISTAKES.md`:

- read the preflight table;
- never encode "no value" as a plausible value; dash it with its reason;
- never assert a constant against itself;
- `cargo test` cannot see a broken shell;
- run pixel probes windowed and isolate them by a project copy with its own
  `config/name`;
- quote timings as a median with min..max, run alone;
- annotate every new symbol for a human auditor (Ruling BK).

### NWP-1: the engine preview (Rust; lane A)

- **Files:** `cartalith-engine/src/lib.rs`, a new `cartalith-engine/src/preview.rs`,
  and `crates/cartalith-godot/tests/nwprev_cost.rs`, promoted from today's
  measurement harness.
- **Work:**
  - Extract the CPU tectonic substrate, from warp to `normalize_field`, into one
    crate-private function. Both `generate_terrain_inner` and `preview` call it.
  - Add `preview::land_mask(p: &WorldParams, preview_w) -> Vec<u8>`, applying
    §4.1's two corrections against `p.gw`. Include the World-Structure path,
    covering the continentality field and the sea-level re-anchor.
- **Acceptance, with measurable bars:**
  - (i) `generate_terrain` output is byte-identical before and after the extract,
    at 3 seeds × 2 extents, CPU and GPU.
  - (ii) Median agreement with the finished world is at least 96 % and the
    minimum is at least 92 %. This holds at 1024 and 2048 targets, for both owner
    cases, 5 seeds each, and for each of the five archetypes at 800 km. The bars
    are a judgement set just under today's measured 93.0 / 97.6 to 98.2.
  - (iii) Every chance and negative-control row prints. The harness panics on an
    empty mask, as it does today.
  - (iv) Early stages at 320 have a median of at most 40 ms on the desktop,
    measured alone. This is a judgement for "instant at debounce". It is not
    measured on a phone.
- **Mutation:**
  - Remove either cell-unit correction, `blur_scale` or `km_scale`. The
    1024-target agreement must fall below the bar, or the bar is too loose and
    must be re-set.
  - Mutate the sea-level read. The mask must move.

### NWP-2: the binding (Rust; lane A, same batch, after NWP-1)

- **Files:** `cartalith-godot/src/lib.rs`, a static `#[func]
  preview_land_mask(params: Dictionary, preview_w: int, target_w: int) ->
  PackedByteArray`.
- **Work:** it reads the same `params.rs` `PARAMS` keys as `generate()` reads. A
  missing key is reported as missing, not defaulted silently.
- **Acceptance:**
  - A Rust test shows that a mask from the binding's parameter path equals
    `preview::land_mask` for the same `WorldParams`.
  - A failure path returns an empty array, not a plausible mask.
  - The binding never touches `self`, so it cannot panic during a generate.

### NWP-3: the dialog (GDScript; lane B, parallel to lane A)

- **Files:** `shell/new_world_dialog.gd`, and a new
  `_nwprev_probe.gd` / `.tscn`.
- **Work:** §2 and §3.
  - The canvas is a `TextureRect` over an `ImageTexture` (`FORMAT_L8` or
    `RGB8`), `TEXTURE_FILTER_NEAREST`.
  - The width rule is one `const`-table-free function `plates_for_km(km)`,
    the §5 formula.
  - Add the history strip, the lock (honoured inside `randomise_seed()`),
    Cancel/Generate on desktop and the More-options disclosure.
  - Guard on `_has("preview_land_mask")`. Without it the canvas shows a dash and
    its reason ("this engine build has no preview"). It must not draw an ocean
    rectangle.
- **Acceptance**, from a windowed probe at 1600 × 900, at 1600 × 1000
  `--force-touch` (tablet), and at 1080 × 2340 `--force-touch` (phone):
  - (i) The canvas pixels change when the seed changes. There is a positive
    control: same seed, no change.
  - (ii) Each input in §3's key list, set through `param_set`, moves the mask,
    or is listed as measured-inert.
  - (iii) The plates readout follows the width table for all 6 presets, against
    literals rather than the function.
  - (iv) The lock blocks the dice and `randomise_seed()`.
  - (v) Clicking a history thumbnail restores all four values.
  - (vi) The first screen on desktop holds preview, seed, extent, size, plates
    and Generate without scrolling (measure the scroller).
  - (vii) Every tappable control is at least 44 dp on both axes on the phone and
    tablet, pressed at its edges.
- **Not provable by probe:** phone operation inside the dialog. See NWP-4.

### NWP-4: verification and glass

The verifier re-runs NWP-1's harness alone and the three probe densities, and
opens every screenshot. The owner then checks on glass that the phone controls
can be operated: dice, chips, plates ±, thumbnails, CANCEL and CREATE WORLD.

**Two-agent shape.** Batch 1 runs lane A (NWP-1 then NWP-2) beside lane B
(NWP-3, against the guarded binding; the canvas shows its dash until the `.dll`
has it). Then a verifier. No lane writes another lane's files. If lane B needs
`lib.rs`, it asks.

---

## 9. Open questions for the owner, with recommended defaults

| # | Question | Recommended default |
|---|---|---|
| Q1 | Should Create use the scaled plate count, so `tect.plates` is set from width, changing worlds at every width except 800 km? | **Yes.** Otherwise the preview shows plates the world will not have. |
| Q2 | Does "36 for world" mean every *Whole world* gets 36, or the 40 075 km width? | **The width.** A whole world at Planet width gets 36; a smaller whole world follows the table. |
| Q3 | Log-linear or power-law table (§5)? | **Log-linear.** It keeps today's default 800 km world at 14 plates, unchanged. |
| Q4 | Does the seed lock outlive the dialog, so *New seed* in the tool bar respects it? | **Yes, for the session.** Not saved in the project. |
| Q5 | How many history thumbnails, and do they persist? | **4, session only**, as requested. |
| Q6 | Should the busy overlay's card show the preview of the seed being generated? | **Optional, later.** It is cheap and makes the wait legible, but it is a separate change. |
| Q7 | Preview colours. | **Grey #8f9295 on blue #2b4d6b**, independent of the map's look, as asked. |
| Q8 | Under an archetype, show the derived plate count read-only, or let the width rule override the archetype? | **Read-only.** Overriding would change what an archetype means. |

---

## 10. What was checked, and how

- **Searched** `OUTSTANDING_WORK.md` and `STATUS.md` for an existing seed-preview
  row. I found none. The matches were incidental uses of "preview" and "seed"
  far apart in one line. All searches stayed inside `C:\Users\Vincent\Cartalith_GDT`.
- **Opened at their symbols:**
  - `new_world_dialog.gd`: `setup`, `_build`, `SIZE_PRESETS`, `ASPECT_PRESETS`,
    `RESOLUTION_PRESETS`, `GRID_MAX`, `_build_seed_dice`, `_on_create`,
    `generate_checked`, `request`, `randomise_seed`.
  - `app.gd`: `open_new_world`, `_new_seed`.
  - `engine_bridge.gd`: `generate`.
  - `busy_overlay.gd`: the header.
  - `world_workspace.gd`: "Pipeline status".
  - `cartalith-terrain`: `compute_warp`, `build_plates`, `assign_plates`,
    `compute_height`, `terrain_detail_k`.
  - `cartalith-engine`: `generate_terrain_inner` (through to volcanism),
    `plate_base_blur_r`.
  - `cartalith-godot`: `params.rs` `defaults()` and `reference_grid_h`.
  - `GENERATION_PARAMETERS.md`: groups `world_structure` and `tectonics`.
- **Ran:**
  - the windowed probe, desktop and phone, `checks=5 fail=0` each;
  - `nwprev_overlap` twice, 1024 and 2048 targets;
  - `nwprev_cost` twice (run 1 alone; run 2 under another session's compile, shown as a contention check);
  - the mockup's generator in `node`: syntax checks of both scripts, the width
    table, determinism (the same seed twice gives 0 differing cells), and the
    mask dimensions 320 × 205 and 320 × 160;
  - headless Chrome screenshots of the mockup.
- **Observed in passing and not changed.** The WORLD dock's *New seed* tooltip
  (`world_workspace.gd`) says it "rolls a new seed in File ▸ New world and
  regenerates from it". `app.gd::_new_seed()` only calls `randomise_seed()`, and
  I found no regenerate on that path. **Reported as probably false prose; not
  verified on screen.**
