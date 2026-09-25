# Map context scope — the right-click card, the tool ring, and their touch forms

> **A proposal, written 2026-09-25 at the owner's request** (*"think about the
> interface under a rightclick or wheel for the mouse … per sub system (World /
> Cartho / Civil) … and how to implement such an interface on a touch device
> Tablet … and Smartphone"*). **Not scheduled.** Five forks in §10 need owner
> answers first. Two of them decide what gets built.
>
> Like every scope document here, this one **defines** milestones and does not
> track them. Status is in `cartalith-native/docs/STATUS.md`, group *Map
> context*.
>
> Per `DECISIONS.md` §7d this is **divergence by addition**. The reference's
> `_civCtxShow` has six operations, and every one survives in §4.3. The rest is
> new interaction design. "Would a user of the HTML app find this feature
> present and its result equivalent or better?" — yes, by construction.

---

## 0 · The recommendation in one screen

**Two surfaces for two questions, opened by one button.**

| | **Context card** | **Tool ring** |
|---|---|---|
| Answers | *"What can I do with **this**, **here**?"* | *"What do I do **next**?"* |
| Grammar | noun first: the object or cell under the pointer | verb first: tools and modes |
| Shape | a titled vertical list in fixed sections. Rows can be disabled with a reason. No length limit | 8 slots on a compass. Positions never move. At most one sub-ring |
| Contents vary by | what was hit × domain × armed tool × draft state | domain only (4 of the 8 slots) |
| Desktop | **RMB click** (press and release without moving) | **RMB drag**: flick a direction and release. Or hold **Q** |
| Tablet (≥ 900 dp) | **long-press, lift in place**: the ring and card open together at the finger | **long-press, then slide** to a slot and lift |
| Phone (< 900 dp) | **long-press** drops the sample pin, and the pin's chip expands into a **peek sheet** of verbs | **press-and-slide on the tool chip** opens a thumb fan above the bottom nav |

The load-bearing rule is from Kurtenbach & Buxton's marking menus (Maya), and
Blender's pies inherit it: **a slot's position is its meaning.** Once a user
has flicked north-west for Settlement fifty times, they stop looking at the
ring. That only works if north-west is Settlement every time the CIVIL ring
opens: disabled when unavailable, never removed and never reordered. The card
has the opposite property. It is a list, it is read, and it can be as
situational as the hit requires.

---

## 1 · What exists today (verified 2026-09-25, by opening the code)

| Thing | Where | State |
|---|---|---|
| RMB signal | `map_overlay.gd` — `signal map_right_clicked(gx, gy, hit, screen_pos)`, emitted from the `MOUSE_BUTTON_RIGHT` branch of `_gui_input` **on press** | Carries **one** hit: the nearest settlement, from `_hit_test_settlement`. Nothing else is hit-tested |
| Broadcast | `app.gd::_on_map_right_clicked` → every workspace with `on_map_right_clicked` | **Only `civilization_workspace.gd` implements it.** Right-click in WORLD or CARTO does nothing |
| The CIVIL menu | `civilization_workspace.gd::on_map_right_clicked` / `_on_ctx_id` | A `PopupMenu`, styled by `DccWidgets.style_popup` (MN-19), gated on `app.active_domain() == "civilization"`. Five of the reference's six ops: Edit · Move viewer to · Delete · Drop settlement here · Info here. Drop POI is omitted, per CV-01 |
| Touch hold | `map_overlay.gd` — `_TOUCH_HOLD_MS := 500`, `_TOUCH_SLOP`, `_touch_armed` / `_touch_swallow_up`, `_process` | PH-02. A hold emits the same `map_right_clicked`. The withheld-press design stops the hold from also firing the armed tool. That design is good, and §8 keeps it |
| Phone presentation | `dcc_shell.gd::phone_present_popup` → `phone_menu.gd::open_sheet` | The same `PopupMenu`, re-presented as an L4 sheet |
| Haptics | `dcc_shell.gd` — `_HAPTIC_MS` (`sample`, `detent`, `tool_arm`, `verdict`, `back`, `blocked`) and `_haptic(kind)` | Built. No context or ring pulse yet |
| Pickers the engine already has | `engine_bridge.gd` — `civ_pick_place_at`, `label_hit_test`, `icon_hit_test` | Settlements, labels and icons can be hit. **Rivers, ways, routes, landmarks and sculpt stamps cannot**; each needs its own pick (§9.2) |
| River inspector | `right_dock.gd` — `CTX_RIVER` | The context exists, but no map gesture selects a river |

**A collision nobody has recorded yet.** `design/dcc-environment-2026-08-31/
spec/06-phone.md` §7 defines the phone's long-press as *"one pointer held
**480 ms** without moving — samples the terrain, drops the pin, shows the chip"*.
The shipped code defines the same gesture, at **500 ms**, as *"open the context
menu"*. Both were built from owner-supplied inputs, and they cannot both be
true. STATUS lists RP-S6 (building that phone spec) as *not started*, so the
collision will surface the day RP-S6 begins. §8 resolves it by making the pin
the context's subject, so the gesture does both. That is fork **F1** in §10.

---

## 2 · What other applications teach

Each row gives what the application does, then what this port takes from it
or refuses.

### 2.1 Graphics and DCC

| Application | Mechanism | Take / refuse |
|---|---|---|
| **Maya** — marking menus | Press, and a radial menu appears after a delay. Or flick a direction without waiting and the selection happens with no menu drawn. The novice path trains the expert path | **Take**: the whole ring model, including the no-wait flick on desktop. Research on marking menus found experts selecting several times faster than from linear menus, once positions were stable |
| **Blender** — pie menus + RMB context menu | Pies (`Z` shading, `` ` `` view, `Tab` mode) run on a key hold, with release-to-select. The RMB context menu is a separate list that depends on the editor and mode. Two surfaces, never merged | **Take**: the split itself, and the hold-key pie (`Q` here). **Refuse**: Blender's many pies. One ring per domain is enough |
| **Photoshop** | RMB with a brush armed opens the brush picker at the cursor. RMB with the Move tool **lists every layer under the cursor** | **Take**: brush parameters in the card's tool section (§4.1), and the **stacked-hit list** (*"Select ▸"*, §4.1). Map objects overlap constantly: a label on a town on a road on a river |
| **Krita** — pop-up palette | RMB opens a ring of favourite brushes and a colour selector around the cursor | **Take**: a ring that carries *current tool state* as well as tools. Here, the armed feature's sub-ring shows which feature is armed |
| **Procreate** — QuickMenu, touch & hold | A configurable gesture opens a six-slot radial at the finger. Touch-and-hold on the canvas is the eyedropper | **Take**: ring at the finger on tablet, and the **eyedropper as a context verb** (*"Paint this biome"*, *"Claim for this faction"*) |
| **Nomad Sculpt** (mobile sculpting) | Brush size and strength are vertical sliders on the screen edge, reachable by the thumb without opening anything | **Take** for tablet as an option (§7.3). It is the only good answer for continuous parameters on touch |
| **Figma / iPadOS** | Long-press or two-finger tap is the secondary click. Apple Pencil squeeze opens a tool palette at the pen tip | **Take**: the pen barrel button as RMB (§7.2). **Refuse**: two-finger tap as the context gesture, because its location is ambiguous (the midpoint of two fingers) and two fingers already mean pinch and rotate here |

### 2.2 Terrain and map editors

| Application | Mechanism | Take / refuse |
|---|---|---|
| **World Creator / Gaea / World Machine** | RMB on the graph opens a searchable node palette. RMB in the viewport is navigation | **Refuse** RMB-as-navigation. Pan here is Space/MMB (§4.5.1), so RMB is free. **Take** search: the card on desktop gets a type-to-filter field, the same command index the menus already have (`command_index.gd`) |
| **Unreal / Unity terrain tools** | Tool state lives in a panel. The viewport has only a brush ring, with size on a modifier drag | **Take**: Ctrl+RMB-drag as brush size (x) and strength (y), as a Photoshop-style HUD, only while a brush is armed (§6) |
| **Wonderdraft / Inkarnate / Dungeondraft** (fantasy maps, closest in audience) | Almost no contextual UI. Tool state is in side panels, and RMB mostly cancels | The gap this proposal fills. Users of these tools expect RMB to *do little*, so **Esc and RMB-on-empty-with-a-draft must never destroy a draft** (§4.4) |
| **QGIS** | RMB on the map: *Identify features* lists every feature at the point, grouped by layer, plus *Copy coordinate* in a CRS | **Take**: the identify list and the coordinate copy. `world_crs()` is already bound, so *Copy coordinate* can offer project km and the world CRS |
| **Google Earth / Google Maps** | Right-click, or long-press on a phone, drops a pin with **What's here? · Directions from / to here · Measure distance**. On a phone the pin opens a **bottom card**, and swiping it up shows more | **Take** the whole phone pattern (§8). Also *Journey from / to here* and *Measure from here* as place-verbs in every domain, since both need nothing but a point |

### 2.3 Strategy games

| Game | Mechanism | Take / refuse |
|---|---|---|
| **Crusader Kings III / Europa Universalis IV** | RMB on a province or character gives an interaction list scoped to *your relationship to it*. Disabled interactions show **why** on hover | **Take**: a disabled row always carries its reason (§4.2). This port already writes honest disabled reasons into tooltips (CV-12's disabled control), and the card follows that |
| **Civilization VI / Anno 1800 (console)** | On a controller, the build menu is a radial on the stick. On PC, RMB means *move/act on target* | **Take** the radial-as-build-menu for CIVIL (Settlement / Territory / Way / Route). **Refuse** RMB-as-direct-action. This is an editor, and an accidental act is worse than a menu |
| **Mass Effect / GTA weapon wheel** | Hold opens the wheel, direction plus release selects, and time slows while it is open | **Take**: *the world pauses while the ring is open*. A running timeline (▶) should not advance under an open ring or card (§9.4) |
| **The Sims — build mode pie** | RMB on an object opens a pie of that object's verbs | **Refuse** for this port. Object verbs vary in number (2 for an icon, 9 for a settlement), which breaks fixed positions. They belong in the card |

---

## 3 · The model: one request, many providers, two presenters

```
            ┌───────────── map_overlay.gd ──────────────┐
 RMB / hold │  gesture classifier (§6, §7, §8)           │
 / Q-hold   │  → ContextRequest {point, screen, hits[],  │
            │     source: mouse|pen|touch, intent: card|ring|both}
            └──────────────────────┬─────────────────────┘
                                   ▼
                     app.gd · ContextBroker.resolve(req)
       ┌───────────────┬───────────┴──────┬────────────────┐
       ▼               ▼                  ▼                ▼
  GlobalTools     WorldWorkspace   CivilizationWorkspace  CartographyWorkspace
  .context_actions(req) → Array[Action]    (each returns rows for its own
  .ring_slots(domain)   → 8 slots           hits and, if active, its tools)
                                   │
                     ┌─────────────┴──────────────┐
                     ▼                            ▼
          ContextCard (desktop/tablet)     PhoneContextSheet (phone)
          RadialRing  (desktop/tablet)     ThumbFan         (phone)
```

**`ContextRequest`**: `gx, gy` (grid), `screen_pos`, `hits: Array` (every
object under the point, nearest first, each `{kind, id, label, dist}`),
`domain`, `armed_tool`, `selection`, `draft: {kind, count}` (sculpt strokes,
paint cells, an open way or route, measure points), `finalized: bool`,
`source`, `form: desktop|tablet|phone`.

**`Action`**: `id`, `label`, `glyph`, `section` (§4.1), `enabled`,
`reason` (required when disabled), `shortcut` (shown right-aligned, so the
card teaches the keyboard), `danger: bool` (Delete-class rows sort last in
their section, get a separator, and keep the existing confirmation),
`callable`.

**Why providers rather than one big menu builder.** The existing
`on_map_right_clicked` broadcast is already this shape: each workspace owns
its own rows. Generalising it keeps each domain's verbs next to the code they
call. It also removes the CIVIL-only gate, so a settlement right-clicked in
CARTO still gets a card. That card has CARTO's verbs plus a single
*"Settlement actions in CIVIL ›"* row, which switches the domain and keeps the
selection. §3 of `DCC_SHELL_SPEC.md` says switching domains keeps the
selection, so that row costs nothing. (Fork **F4**: whether to show
cross-domain verbs inline instead.)

---

## 4 · The context card

### 4.1 Section grammar — fixed order, every domain

Empty sections are omitted. The order never changes. It is the card's
equivalent of fixed ring positions.

| # | Section | Contains | Why here |
|---|---|---|---|
| 0 | **Header** | Hit name and kind (`Vhal Serai · town`), or `Here` for a bare cell. Second line: elevation, biome, grid → km. A **Select ▸** row lists every hit when there are two or more (Photoshop Move-tool RMB, QGIS Identify) | Tells the user what the card is about before offering verbs on it |
| 1 | **Draft** | Commit · Discard · Undo last stroke/point, with counts (`Commit 3 strokes`) | Only when a draft exists. A draft is the most time-sensitive state in the app |
| 2 | **Tool** | The armed tool's 2–4 most-changed parameters, inline. Brush: radius / strength / mode. Settlement: class. Way: type | Photoshop's brush picker at the cursor. Saves a trip to the options bar |
| 3 | **Object** | Verbs on the hit, in the owning domain (§4.3) | The noun the user pointed at |
| 4 | **Place here** | Verbs that create something at this point | Domain-scoped |
| 5 | **Go / measure** | Centre view here · Measure from here · Cross-section from here · Journey from / to here | Needs only a point, so it works in every domain |
| 6 | **Info** | Pin sample here · Copy coordinate ▸ (km / world CRS) · Open vault note / Attach vault note (for a vault-kind hit) | Always last and always present, so there is one stable place to look |

### 4.2 Rules

- **Disabled rows stay, with a reason.** `Delete Vhal Serai` greys out on a
  finalized world with *"world is finalized — unlock in File ▸ Finalize"*.
  This matches `DCC_SHELL_SPEC.md` §4.5.6's lock list, and the CK3 precedent.
- **No sixth level.** A card row may open **one** submenu, and never two.
  `UI_SHELL_DESIGN.md`'s five-level disclosure grammar applies to the card as
  a unit.
- **Only engine-backed rows ship.** The port's discipline applies unchanged: a
  row with no engine call behind it is *omitted*, as Drop POI already is. It
  is not drawn and wired to nothing. The tables below mark every row.
- **Keyboard.** The Menu key / Shift+F10 opens the card for the current
  selection, anchored at the selection, or at the cursor when nothing is
  selected. Typing filters rows. Enter runs a row, Esc closes.

### 4.3 Per-domain contents

Backing key: **B** = `engine_bridge.gd` already binds it · **S** = shell-only
(GDScript, over bound calls) · **P** = needs a new pick (§9.2) · **E** = needs
new engine work · **—** = excluded, reason in the row.

#### WORLD — *what exists*

| Hit / state | Section | Row | Backing |
|---|---|---|---|
| any cell | Header | elevation · slope · biome · plate + boundary type (`sample_cell`) | B |
| Sculpt draft | Draft | Commit N strokes · Discard · Undo stroke | B (`sculpt_commit`, `undo_label`) |
| Sculpt armed | Tool | radius · feature param 1 · mode (add / set) · **Feature ▸** (the armed feature's family, §5.3) | S |
| Sculpt stamp under cursor | Object | Select · Hide / Show · Move up · Move down · Delete stamp | B (`sculpt_select_stamp`, `sculpt_set_stamp_hidden`, `sculpt_move_stamp_up/down`, `sculpt_delete_stamp`) + **P** + **E-small** (stamp footprint pick; `sculpt_list_stamps` returns `point_count`, not the points) |
| Paint draft | Draft | Commit N cells · Discard | B (`paint_commit`, `paint_discard`, `paint_draft_count`) |
| Paint armed | Tool | target (Biome / Terrain / Splat, from `get_paint_layers`) · radius · erase | B |
| any cell, paint armed | Object | **Paint with this cell's biome** — the eyedropper | S (`sample_cell` → the palette index in `get_paint_palette`) |
| river cell (`river_order > 0`) | Object | Inspect river → `CTX_RIVER` | **P** + E (the river's identity along its stem, §9.2) |
| river cell | Object | Trace downstream · Show catchment | E. The receiver tree exists in hydrology (`RC_ENGINE_CHANGES.md` §6n) but has no bound trace |
| any cell | Go | Cross-section from here | B (arms Measure in `section` mode with the first point placed; `measure_section`) |
| any cell | Info | Pin sample here · Copy coordinate ▸ | B (`sample_cell`, `world_crs`) |
| — | — | *Regenerate from this stage* | **—** `generate()` is monolithic, as recorded in `DCC_SHELL_SPEC.md` correction 2. A card must not bring back the fiction the spec removed |

#### CIVIL — *who occupies it*

| Hit / state | Section | Row | Backing |
|---|---|---|---|
| Settlement | Object | Edit… · Move viewer to · **Delete** (danger) | B. Today's CX-01 ops 0-2, unchanged |
| Settlement | Object | Open city layout… | B (`app.open_city_viewer`) |
| Settlement | Object | Journey from here / Journey to here | S over `jp_default_plan` / `jp_compute` + `app.open_journey_planner` |
| Settlement | Info | Open vault note / Attach vault note… | B (`vault_links_for("settlement", id)`, `app.open_vault`) |
| Territory under cell | Object | Open faction in roster · **Claim for this faction** (arms Territory with that faction: the eyedropper) | S over `app.open_faction_roster`; **E-small** for a per-cell faction read. `sample_cell` does not return the controlling faction, and `civ_territory_influence()` needs checking |
| Territory draft | Draft | Commit territory · Discard | B (`civ_territory_commit`, `civ_territory_discard`) |
| Route under cursor | Object | Open in Journey Planner · Rename · **Delete** | B (`jp_plan_for_route`, `route_set_name`, `route_delete`) + **P** (polyline pick over `route_get(i).points`) |
| Way under cursor | Object | Inspect way · **Delete** | **P** + **E**. Only `way_begin/append/commit/discard` are bound, and there is no way list or delete |
| Way / route in progress | Draft | Commit (Esc does the same) · Undo point · Discard | B (`way_commit`, `route_commit`, `…_discard`) |
| Landmark under cursor | Object | Inspect · Why here? (its placement funnel) | **P** over `landmarks()`; `landmark_funnels()` is B |
| any cell | Place here | Drop settlement here ▸ (the five classes; default = the tool's current class) · Start way here · Start route here | B (`civ_drop_settlement` via `_settlement_click`, `way_begin`, `route_begin`) |
| any cell | Info | Info here (settlement & ecology) | B. Today's CX-01 op 4 |
| — | — | Drop POI here | **—** CV-01 stands. POI is not a ported concept (`civ_tools_bridge.rs`) |

#### CARTO — *how it is shown*

| Hit / state | Section | Row | Backing |
|---|---|---|---|
| Label | Object | Edit text · Reset arc · Duplicate · **Delete** | B (`label_select`, `label_set`, `label_delete`, `label_create`) |
| Icon | Object | Properties · **Delete** | B (`icon_get`, `icon_delete`) |
| any named hit (settlement, landmark) | Place here | **Label this: "Vhal Serai"** — a label pre-filled with the hit's name | B (`label_create`) + S |
| any cell | Place here | Add label here · Stamp armed icon here (disabled with the reason *"no icon armed — Assets ▸ library"* when none is) | B (`label_create`, `icon_place`, `icon_armed`) |
| any cell | Object | View field ▸ (the eight views of the Layers popover, same order and hotkeys 1–8) · Style preset ▸ (the six `STYLE_PRESETS`) | B (`debug_layers`) + S (`render_workspace.gd`'s preset apply) |
| any cell | Go | Start export region here | B (`region_set`) |
| settlement | Object | Settlement actions in CIVIL › | S (domain switch; fork F4) |

---

## 5 · The tool ring

### 5.1 The compass

The **cardinals are global** and identical in every domain. The **diagonals
belong to the domain**. Each domain has exactly four tool families, which is
the reason for this split rather than a coincidence it exploits.

```
                         N  Inspect (V)
            NW ·                                · NE
                 domain 1        ╭───╮        domain 2
     W  Region (R)               │ ✕ │               E  Measure ▸ (M)
                 domain 4        ╰───╯        domain 3
            SW ·                                · SE
                         S  Undo  (label: "Undo stroke 3")
```

| Slot | WORLD | CIVIL | CARTO |
|---|---|---|---|
| **N** | Inspect | Inspect | Inspect |
| **E** | Measure ▸ | Measure ▸ | Measure ▸ |
| **S** | Undo | Undo | Undo |
| **W** | Region select | Region select | Region select |
| **NW** | **Uplift ▸** Mountains · Hills · Ridge · Plateau · Cliff · Volcano | **Settlement ▸** metropolis · city · town · village · hamlet | **Label** |
| **NE** | **Carve & water ▸** Canyon · Valley · River · Lake · Basin · Coastline | **Territory ▸** add · subtract | **Icon ▸** Settlement · Feature · POI · Custom (`ManualIconFamily`) |
| **SE** | **Freehand ▸** Raise · Lower · Smooth · Cliff · Ridge · Canyon · Mesa · Volcano | **Way ▸** road · track · trail · bridge | **View field ▸** the eight Layers views |
| **SW** | **Biome paint ▸** Biome · Terrain · Splat | **Route** | **Style preset ▸** Natural Vibrant · Default · Antique · Ink · Watercolor · Print |

Measure ▸ holds the five modes `global_tools.gd` already defines (segment /
path, area, radius, cross-section, Δ vertical).

**Why each family fits:**

- **WORLD's twelve sculpt features split cleanly into two sets of six** —
  those that raise and those that carve or hold water. Each set fits a
  sub-ring with room to spare. Freehand's eight sub-modes are exactly eight.
  Biome paint's three targets are exactly the three `PaintStamp` layers the
  engine has (`DCC_SHELL_SPEC.md` §4.5.2's own ruling).
  **`SCULPT_FUNCTION_CHART.md` §2 permits this regrouping and forbids a
  renumbering**: `FEATURE_KEYS`' index is a seed input. The ring maps a slot
  to a feature *key* and never to a position.
- **CIVIL's four tools are its four tools.** Journey is not a drag/click tool
  (`DCC_SHELL_SPEC.md` §4.5.4: *"no map gesture is bound to it"*), so it is
  in the card as *Journey from / to here*, not on the ring.
- **CARTO has only two tools**, Label and Icon. Its other two diagonals take
  its two most-used *view* choices, which the Layers popover and the Map style
  category otherwise need a dock trip for. The Layers popover already has
  exactly eight hot-keyed views.
- **Undo is south** because *pull back* is the natural flick for *go back*.
  An accidental undo is recoverable (Ctrl+Y / Redo), which is the test for
  putting an action on a flick. **Commit is deliberately not on the ring.**
  It is irreversible in the sense that matters (a sculpt commit marks
  tiles stale), so it belongs in the card, where it is read, not flicked.

### 5.2 Rules

1. **Position is meaning.** A slot never moves and never disappears. If it is
   unavailable (finalized world, no world loaded), it is drawn at 35% opacity
   with its reason in the centre readout. §4.5.6's lock list decides which
   slots grey when finalized: Inspect, Measure, Region, Label and Icon stay
   live.
2. **At most one sub-ring.** A `▸` slot opens its sub-ring **in place**: the
   ring re-centres on that slot, as in Maya's nested marking menus. Two levels
   and no more. This matches the card's one-submenu rule.
3. **The armed state is visible.** The armed tool's slot is filled with the
   accent, using §11's reversed-type-on-accent rule and MN-21's precedent. A
   sub-ring marks the armed feature. The ring shows *where you are* as well as
   *where you can go* (Krita's pop-up palette).
4. **Dead zone and cancel.** The centre disc (radius 24 px desktop, 36 dp
   touch) is cancel. Releasing inside it closes the ring and does nothing. The
   centre shows the hovered slot's full label and shortcut, so labels on the
   ring can stay short.
5. **Arming from the ring is arming.** It goes through the same
   `app.tool_group` path the TOOLS block uses. The ring is a second
   presentation of §4.5, never a second implementation. That is the same rule
   `UI_SHELL_DESIGN.md` applies to menus that open windows.
6. **Customisation is later, not never** (fork F5). v1 ships the fixed layout
   above. Letting users re-slot the diagonals is Blender's and Procreate's
   answer to power users. It belongs in the keymap editor `GUI_GAP_REGISTER.md`
   already proposes, where *"right-click a tool button → Assign to ring slot"*
   would sit beside *Assign shortcut*.

---

## 6 · Desktop input (mouse, pen with barrel button, keyboard)

| Input | Result |
|---|---|
| **RMB press → release**, under 8 px travel and under 300 ms | Context card at the cursor |
| **RMB press → travel ≥ 8 px** | Ring, drawn centred on the press point. Release on a slot selects it. **A fast flick selects without the ring being drawn** (Maya's expert path): the ring appears only if the button is still held after 150 ms |
| **RMB press → hold ≥ 300 ms, still** | Ring *and* card: the ring at the cursor, the card docked beside it on the side with room. Same as touch (§7), so the three platforms share one mental model |
| **Q hold** (key down → aim → key up) / **Q tap** | Ring at the cursor. A tap leaves it open, sticky, so a click selects (Blender's pie behaviour). `Q` is free in `DCC_SHELL_SPEC.md` §4.5's hotkey list, but check `DccSettings`' shortcut table before binding it |
| **Menu key / Shift+F10** | Card for the selection, or at the cursor |
| **Ctrl+RMB drag**, brush armed | HUD: horizontal drag = radius, vertical = strength. Shows live values on the brush ring and commits on release (Photoshop Alt+RMB, Unreal) |
| Space / MMB | Pan, unchanged, always available. The ring and card never capture MMB |

**One behaviour change to existing code, stated so it is not a surprise.**
`map_overlay.gd` emits `map_right_clicked` on **press** today. Its comment says
the reference's `contextmenu` *"fires on press"*. That is platform-dependent.
Browsers fire `contextmenu` on mouse-*down* on macOS and Linux, but on mouse-*up*
on Windows, which is this port's desktop target. Telling a click from a drag
requires waiting for the release, so the card moves to release. On Windows
that is also closer to how the owner has always seen the reference behave.
**Verify this on the owner's Windows browser before relying on the parity
half of that sentence.**

---

## 7 · Tablet (shortest width ≥ 900 dp — the desktop-parity shell)

The tablet keeps desktop parity (`UI_SHELL_DESIGN.md`), so it gets both
surfaces in desktop geometry at touch density. A finger has one button, so the
two surfaces share one gesture.

### 7.1 The gesture

| Finger does | Result |
|---|---|
| Hold **500 ms** without moving past `_TOUCH_SLOP` | `sample` haptic pulse (the table has it). **Ring and card open together** at the finger. The ring is centred on the touch point. The card docks on the side *away from the dominant hand* (a preference, default right-handed → card on the left) so the palm does not cover it |
| …then **slides** to a slot and lifts | Selects the slot, with a `tool_arm` pulse on crossing into a slot. The marking path: after a few weeks users hold, slide and lift without reading |
| …then **lifts in place** | Both stay open as tappable surfaces. Tap outside to dismiss |
| Immediate drag (no hold) | Unchanged: pan, or the armed tool's stroke. Touch has no no-wait flick, and that is the one expert path it gives up |

The withheld-press machinery PH-02 built (`_touch_armed`, `_release_touch_press`,
`_touch_swallow_up`) is kept exactly. It is what stops the hold from dropping a
settlement first. What changes is that after the hold fires, the finger is
**still tracked** instead of swallowed until lift, so the slide can select.

### 7.2 Occlusion, reach and the pen

- **Ring radius scales with the finger.** Slots sit at 96 dp on touch (60 px
  on desktop), so the thumb's contact patch does not cover the slot being
  aimed at. At 44 dp per slot, eight slots need a circumference of about
  352 dp, which is a radius of 56 dp. 96 dp leaves generous gaps and a clear
  angular sector per slot.
- **Edge flipping.** Near a screen edge the ring slides inward rather than
  clipping. The spec's *"reorganises rather than truncates"* applies to menus
  as well as docks.
- **The pen barrel button is RMB.** On Android the stylus primary button is
  reported as a secondary click. **Verify it arrives in Godot as
  `MOUSE_BUTTON_RIGHT`** on the owner's device before promising it. If it
  does, the pen gets the full desktop table in §6, flick included.
- **A hardware keyboard** gets the desktop bindings unchanged.

### 7.3 Optional — the edge sliders (Nomad Sculpt)

While a brush is armed on a tablet, two thin vertical sliders on the screen
edge nearest the non-dominant hand give radius and strength, always visible
and thumb-reachable. This is the touch equivalent of §6's Ctrl+RMB HUD. It
does not replace the options bar. Fork **F5b**.

---

## 8 · Phone (shortest width < 900 dp)

A ring at the fingertip does not fit a 412 dp screen with a 64 dp nav and a
sheet. It would cover a third of the map, and the thumb covers the rest. The
phone therefore **separates the surfaces by location**. The *noun* surface
lives where the finger pointed. The *verb* surface lives where the thumb
rests.

### 8.1 Long-press → pin → peek sheet (the noun surface)

This reconciles the collision in §1: **the long-press does what
`spec/06-phone.md` §7 says, and the pin becomes the context's subject.**

1. Hold **480 ms** (the phone spec's figure; see fork F1 for 480 vs 500)
   without moving. The `sample` haptic fires, the **sample pin** drops, and
   the sample chip appears exactly as `06-phone.md` §6.2 specifies.
2. At the same moment the sheet opens at the **peek** detent (66 dp,
   `06-phone.md` §5.2). Its header is the card's §4.1 header. Below it is **one
   horizontal row of up to four action chips**: the first four enabled rows of
   the card in section order, so the *Draft* chips (Commit / Discard) win when
   a draft exists and the *Object* verbs win otherwise. This is the Google Maps
   pin card (*Directions · Save · Share*).
3. **Swipe the sheet to half** and the full card appears, in the same
   sections, as 46 dp rows (`06-phone.md`'s row height). It is one provider
   list and one definition in two presentations, which is the rule PH-02
   already wrote for the current menu.
4. **The pin is draggable.** Finger-down on the pin drags it, and the card
   re-resolves on release. This answers the phone's worst context problem, a
   fingertip that is 40 cells wide at world zoom. The user adjusts the subject
   instead of retrying the gesture. A multi-hit point shows *Select ▸* as the
   first chip.
5. The chip's own *"tap for inspector ›"* is unchanged. The chip is the pin's
   **readout**, the sheet is its **verbs**, and the edge-swipe inspector is its
   **detail**. Three depths of one subject.

### 8.2 The thumb fan (the verb surface) — fork F2

A fan of the same eight slots opens **above the bottom nav**, anchored on the
armed-tool chip. The phone spec draws tool chips in the MAP sheet. This
proposes an **armed-tool pill** at bottom centre, above the nav, shown while
the sheet is closed.

- **Tap** the pill: toggle between the armed tool and Inspect. This is the
  most common phone switch (draw, then look), done without the sheet.
- **Press and slide up**: a 180° fan opens above the pill. A half-fan
  cannot point south, so **distance stands in for the south half**. The
  outer arc (radius 150 dp) carries the five north-half slots **W · NW · N ·
  NE · E**, left to right, at about 118 dp spacing. The inner arc (radius
  80 dp) carries **SW · S · SE**, at about 84 dp spacing. Every slot keeps its
  compass *side*, and Undo is still the short pull back toward the thumb. The
  outer arc spans 348 dp including slot size, which fits 412 dp. Transfer
  between tablet and phone is imperfect, but this avoids inventing a second
  layout.
- A `▸` slot opens its sub-fan in place of the fan, and back-swipe returns.
  No deeper.
- The fan sits in the lower 45% of the screen, inside the thumb's natural
  reach on a 6.4″ handset, and above the 20 dp gesture inset, which holds no
  targets (§13's rule).

If F2 is declined, the phone keeps the MAP sheet's TOOLS chips as its only
tool switch, which works today. The fan is the smaller half of this proposal.

### 8.3 What the phone does *not* get

- No ring at the fingertip (occlusion, as above).
- No two- or three-finger tap gestures for undo/redo (Procreate style). The
  phone spec has the undo chip, and two fingers already mean pinch and rotate.
- No edge sliders. The phone's edges are the inspector swipe (right) and the
  system back gesture (left).

---

## 9 · Implementation in this codebase

### 9.1 Files

| File | Change |
|---|---|
| `map_overlay.gd` | Replace the single-hit `map_right_clicked` with a **gesture classifier** that emits `context_requested(req: Dictionary)`. RMB: track press → release / travel / hold (§6). Touch: extend the existing hold path so the finger stays tracked after the hold (§7.1). **Keep the old signal** as a thin shim until every consumer has moved; `_menuconf_probe.gd` calls `on_map_right_clicked` directly |
| `map_overlay.gd` | `hits_at(pos) -> Array`: settlements (`_hit_test_settlement`), labels (`label_hit_test`), icons (`icon_hit_test`), then the new picks in §9.2. Nearest first, capped at 8 |
| new `shell/context_broker.gd` | Builds the `ContextRequest` (reads `app.active_domain()`, `armed_tool`, draft counts, finalized), asks every provider, sorts into §4.1's sections, and chooses the presenter by form. Owned by `app.gd`, like the other brokers there |
| `global_tools.gd` + the three workspaces | Each gains `context_actions(req) -> Array` and `ring_slots() -> Array`. `civilization_workspace.gd::on_map_right_clicked` becomes its `context_actions`. Its five rows move over **unchanged in id and behaviour**, which is CM-1's regression test |
| new `shell/context_card.gd` | A `PopupPanel` + `VBoxContainer` built from `DccWidgets` rows, **not** a `PopupMenu`. A `PopupMenu` cannot hold §4.1's inline parameter rows, a header readout, or a disabled row's reason. Keyboard navigation and type-to-filter included |
| new `shell/radial_ring.gd` | A `Control` with a custom `_draw()`: arcs, glyphs from `DccIcons`, dead zone, sub-ring. Slot from `atan2` of the pointer offset, snapped to 45° sectors. Sectors do not need hit rectangles, so it stays a few hundred lines. `faction_banner.gd`'s `_draw()` is the in-tree precedent for a hand-drawn `Control` |
| `phone_menu.gd` | A `peek_card(req)` variant of `open_sheet`: chip row at peek, full card at half |
| `dcc_shell.gd` | Two new `_HAPTIC_MS` kinds only if `sample`/`tool_arm` prove wrong on the device. Otherwise reuse them |

### 9.2 New picks, cheapest first

| Pick | How | Cost |
|---|---|---|
| Landmark | Nearest `landmarks()` entry within the glyph radius, in GDScript | small |
| Route | Point-to-polyline distance over `route_get(i).points`, in GDScript. Routes are few | small |
| Sculpt stamp | The stamp's stroke polyline or radial centre plus its radius. **`sculpt_list_stamps()` returns `point_count` and not the points**, so this needs one bound accessor for a stamp's points first | small–medium, **E-small** |
| Way | Needs a bound list of committed ways with geometry, and a delete. Neither exists | medium, **E** |
| River | `sample_cell().river_order > 0` says *a river is here*. *Which river* (identity along the main stem, for `CTX_RIVER`) needs the stem assembly `RC_ENGINE_CHANGES.md` §6j describes | medium, **E** |

### 9.3 Presenter choice

`form` comes from the existing `DccTheme.is_phone()` / the 900 dp threshold
(`_TABLET_MIN_DP`). Do not re-derive it. `source` comes from the event:
`device < 0` marks emulated-from-touch, the rule PH-02 already relies on.

### 9.4 Cross-cutting

- **The world pauses under an open surface.** If the timeline is playing, an
  open card or ring holds the tick and resumes on close (the weapon-wheel
  rule).
- **An open surface never commits or discards a draft by being dismissed.**
  Esc and tap-outside close the surface only. Commit and Discard are rows.
- **Probes.** One new `_ctxmenu_probe.gd` that walks every domain × {empty
  cell, settlement, label, icon} × {no tool, brush armed, draft present}. It
  asserts the card is **non-empty**, every disabled row has a non-empty
  `reason`, and the ring always has exactly 8 slots with the §5.1 cardinals.
  The *"assert non-emptiness and shape"* rule in `CLAUDE.md` exists because
  four subsystems were bitten by silent emptiness.

---

## 10 · Owner forks

| # | Question | Recommendation | Why |
|---|---|---|---|
| **F1** | On the phone, does long-press **(a)** drop the sample pin *and* open the verbs (§8.1), **(b)** only drop the pin, as `06-phone.md` says, or **(c)** only open the menu, as the code does today? And is the hold 480 ms or 500 ms? | **(a), 480 ms** | Satisfies both owner inputs instead of choosing between them. 480 is the newer figure and the only one written in a spec |
| **F2** | Build the phone's thumb fan (§8.2)? | **Yes, after CM-5** | Tool switching without the sheet is the phone's most frequent unmet need. It is optional because the sheet chips already work |
| **F3** | The ring's cardinals: global **Inspect / Measure / Undo / Region** as in §5.1? | **Yes** | The only assignment where every domain's diagonals are exactly its four tool families |
| **F4** | A settlement right-clicked in CARTO: show CIVIL's verbs **inline**, or one *"…in CIVIL ›"* row? | **One row** | Keeps each domain's charter (*what exists / who occupies it / how it is shown*) visible in the UI. Inline makes the card as long as all three domains together |
| **F5** | Ring customisation and the tablet edge sliders (§5.2.6, §7.3) — v1 or later? | **Later** | Both are power-user features that need the fixed layout to exist first, so there is something to customise |

---

## 11 · Milestones

Each milestone's *done means* is checkable in the tree. Status lives in
`STATUS.md`, not here.

| ID | Milestone | Done means | Size |
|---|---|---|---|
| **CM-1** | **The request and the broker** | `context_requested` emitted with a multi-hit `hits[]`; `context_broker.gd`; the three workspaces and `GlobalTools` implement `context_actions`; **CIVIL's five CX-01 rows arrive through it unchanged**, still in a styled `PopupMenu`. WORLD and CARTO produce their **B**-backed rows. The probe in §9.4 is green | medium |
| **CM-2** | **The context card** | `context_card.gd` replaces the `PopupMenu` on desktop and tablet: §4.1's sections, header readout, *Select ▸*, disabled-with-reason, inline Tool rows, keyboard and filter. RMB moves from press to release (§6) | medium |
| **CM-3** | **The desktop ring** | `radial_ring.gd`; RMB-drag with the no-wait flick, RMB-hold ring+card, Q hold/tap; §5.1's table for all three domains; sub-rings; armed-state fill; finalized greying | medium |
| **CM-4** | **Tablet** | Hold → ring+card with continued tracking and slide-to-select; haptics; handedness preference; edge flipping. **The pen barrel button is verified on the owner's device**, one way or the other, and the result is written down | medium |
| **CM-5** | **Phone noun surface** | Long-press → pin + peek chips → half-detent card; draggable pin that re-resolves. Lands **with or before RP-S6** so the phone spec and the code never disagree about the gesture again. Blocked on **F1** | medium |
| **CM-6** | **Phone thumb fan** | §8.2. Blocked on **F2** | small–medium |
| **CM-7** | **New picks and verbs** | §9.2's rows, one at a time, each landing with its card row: landmark and route picks (small), stamp pick, then the two **E** rows (way list and delete; river identity and trace downstream / catchment). Each **E** row is its own engine task and moves its row from *omitted* to *shipped* | large, divisible |

**Order**: CM-1 → CM-2 → CM-3 is the desktop path, and each step ships alone.
CM-4 needs CM-3. CM-5 needs CM-1 and F1, and its schedule is tied to RP-S6's.
CM-7 can run beside any of them.

---

## 12 · Considered and declined

| Idea | Why not |
|---|---|
| A ring of *object* verbs (The Sims) | Verb counts vary per object (2 to 9), which destroys fixed positions. That is the card's job |
| RMB = act directly (move to / attack, as in strategy games) | This is an editor. An unrequested write to the world is the most expensive mistake it can make |
| Hover-to-open (no click) radial | Opens by accident during every pan, and has no touch equivalent |
| Two-finger tap as the touch context gesture | Its location is ambiguous, and two fingers are already pinch and rotate (`06-phone.md` §7) |
| Three-level sub-rings | Marking-menu accuracy falls off sharply past two levels, and the card's one-submenu rule should match |
| Commit on the ring | Commit marks stages stale. It is read in the card, not flicked (§5.1) |
| *Regenerate from here* | `generate()` is monolithic. Re-introducing per-stage runs through a menu would bring back the fiction `DCC_SHELL_SPEC.md` correction 2 removed |
| Drop POI | CV-01, re-checked twice, stands |
