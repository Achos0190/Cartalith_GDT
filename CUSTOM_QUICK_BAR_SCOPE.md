# Custom quick bar scope — user-pinned commands in the top bar and under the cartwheel

> **A proposal, written 2026-10-25 at the owner's request.** Owner, verbatim:
> *"in the top bar we have quick buttons to do measurements. And when an option
> that requires a brush is chosen we get brush settings. But I'd like to make the
> bar user adaptable as well so a user could put options there themselves like a
> quick button to access the planner, or the vault or other such functions that
> might often be used. Things we don't have below the cartwheel for example but
> are still relevant when used. We could also put an llm button under the
> cartwheel as soon as an api/llm has been added by the user."*
>
> Like every scope document here this one **defines** milestones and does not
> track them (no status column). Status is in `cartalith-native/docs/STATUS.md`.
> Evidence tags: **[verified]** opened at the named symbol on 2026-10-25;
> **[grep]** a text count that CQ-0 must re-measure live; **[judgement]** a
> labelled recommendation the owner may overrule. Nothing below is "measured";
> every number in a milestone is a bar to be met.
>
> **Per `DECISIONS.md` §7d this is divergence by addition**: the reference HTML
> has no user-configurable bar, so nothing of the reference is lost. It also
> **delivers part of `MAP_CONTEXT_SCOPE.md` fork F5** ("ring customisation:
> later", Ruling AX), which this scope asks the owner to lift for the dock and
> the shared sub-rings only (Q3).
>
> Read `MISTAKES.md`'s preflight first. The rows that bite here are named where
> they apply: *Move a command off the menu bar*, *Return/marshal a value that can
> be absent*, *Read a layout that overflows the screen*, *Add `clip_text`*,
> *Write a backward-compatibility test*, *Gate a write on "is there anything to
> save"*, *Declare the work green*, *Run `godot --headless --import`*.

---

## 0 · The recommendation in one screen

**One list of pinned commands, drawn in three places, edited in one.**

| | Desktop | Tablet | Phone |
|---|---|---|---|
| **Quick cluster** (the user's pins) | right end of the **tool palette bar**, after a rule | same bar, touch-scaled | a chip row at the top of the MAP sheet, and a *Quick* section first in MORE |
| **Overflow** | a `⋯` chip opens a `PopupMenu` of the pins that did not fit | same | the row scrolls horizontally; its last chip, `All…`, opens a list sheet |
| **Under the cartwheel** | a *ring dock*: up to 5 chips directly beneath the ring | same, at the ring's touch scale | no ring; the thumb fan (`MAP_CONTEXT_SCOPE.md` §8.2) is unchanged and not extended |
| **Assistant button** | first chip of the cluster and of the ring dock, **only** while the assistant is enabled and a provider is configured | same | same chip in the row and in MORE |
| **Customise** | `Preferences ▸ Quick bar…` window, `＋` chip, `⋯ ▸ Customise…` | same window at touch density | a sheet from MORE or the `＋`/`All…` chips |
| **Stored in** | `user://cartalith_settings.cfg`, a new `[quick_bar]` section. **Never in a world `.zip`** | same file on that device | same file on that device |

Three rules decide the rest:

1. **The pin is a pointer to an existing command, never a second implementation.**
   Pressing a chip runs the very handler the menu row runs. A command that is
   renamed, moved or removed shows as a greyed chip with a reason; it is never
   silently dropped and never silently re-pointed.
2. **Position is still meaning on the ring.** The compass keeps its eight slots
   and its four fixed cardinals. User items go *beside* the compass (the dock) or
   *inside a sub-ring that already exists*; they never move or replace a built-in
   slot in v1 (§4).
3. **No new surface the user has not asked for.** The shipped default is an empty
   cluster, so a user who ignores this feature sees today's bar unchanged apart
   from one small `＋` chip.

---

## 1 · What exists today (verified 2026-10-25, by opening the code)

| Thing | Where | State |
|---|---|---|
| The owner's "measurement quick buttons" | `tool_bar.gd` — `DccToolBar._build_measure_tools()`, `_measure_button()`, `MEASURE_GROUP_POINT`, `MEASURE_GROUP_VERTICAL`; the canvas is `design/Cartalith Measurement Toolbar.dc.html` | Drawn as **button groups inside the tool options bar** (`DccShell` `tool_options_row`). **They exist only while Measure is armed** — `DccToolBar.mode` follows `app.tool_armed` |
| The brush settings | `tool_bar.gd` — `_build_sculpt_options()`, `_build_paint_options()`, and the ET-D5 third row `build_brush_controls(parent, app, tool)` (spacing, pressure, scatter, symmetry; called by the Way tool's row too) | Appear only when Sculpt or Paint (or Way) is armed. Driven by `tool_armed`, `generation_finished`, `world_loaded` |
| The tool palette bar | `app.gd` — `_install_tool_palette_bar()`, `tool_palette_bar`, `_tool_palette_row`, `set_domain_tools()`, `_strip_entries()`, `_rebuild_tool_strip()`, `_sync_tool_strip()`; `dcc_widgets.gd::tool_strip()` | A bar **of its own above the options bar**, **off the phone**, rebuilt on `workspace_changed`. Its comment records why it is not a prefix on the options bar: the palette (135–287 px) plus the options bar's up-to-1 440 px pushed the window's minimum width past 1 600 px |
| The options bar's fit rules | `dcc_shell.gd` — `tool_options_row`; on a tablet the row sits in a horizontal scroller; `DccApp.set_tool_options(build)` wraps `DccShell.set_tool_options` | Heights from `DccTheme.role_px("h_tool_options")` (40 desktop, 56 tablet per `DESIGN_HANDOFF.md`). On the phone the options bar is the **bottom sheet** (MAP / GENERATE tabs drive its detent; PLAN and MORE open overlays) |
| The command index | `command_index.gd` — `CommandIndex.build()`, `_add_parameters()`, `_add_menu_commands()`, `_walk_popup()`, `EXTRAS`, `search()`, `groups()` | One searchable list assembled from the app: parameters (`kind: "param"`), every leaf of the live `MenuBar` (`kind: "menu"`, or `"readout"`), and `EXTRAS` (`kind: "action"`: Zoom to fit, Point sample, Theme). Each row has `title, blurb, group, kind, key, available, why`. **It carries no handle to run anything**: a row does not record its popup or its item id |
| Stable command identity | `menus.gd` — the `ID_*` constants (128 `const ID_… := n` lines **[grep]**), e.g. `ID_VAULT := 45` (World ▸ Markdown vault…), `ID_STORY_JOURNEY_PLANNER := 726`; `shortcuts_dialog.gd` and `DccSettings.shortcut_binding(context, action_id, default)` key the owner's rebinds on `action_id` | The id is already the project's persisted identity for a menu command. `menus.gd` says ids are dispatched **per `PopupMenu`** ("two popups sharing an id do not cross-fire"), so uniqueness *across* popups is not guaranteed — CQ-0 measures it |
| Where the two named examples live | `menus.gd::_story()` (`Story ▸ JOURNEYS ▸ Journey planner`, `ID_STORY_JOURNEY_PLANNER`; `app.gd::open_journey_planner()`); `ID_VAULT`, `ID_VAULT_BROWSE` (716); `app.gd::open_vault()` | Both are real menu rows, so the walk indexes them. The Journey planner also has a CIVIL rail node and the `⇧J` key |
| Per-user persistence | `dcc_settings.gd` — `DccSettings`, one `ConfigFile` at `CONFIG_PATH := "user://cartalith_settings.cfg"`, one section constant per concern (`_SEC_LAYOUT`, `_SEC_HANDEDNESS`, `_SEC_LLM`, …), `_ensure_loaded()`, `_save()`, `_string_list()`; `layouts()`/`save_layout()` store named dictionaries; `asset_favourites()`/`remember_asset()` store capped string lists | Per machine, never in a world. The LLM section is the in-tree precedent for an **allow-listed** section (`LLM_DEFAULTS`, `set_llm_value()` refuses an unknown key or a wrong type) |
| The ring and its slot providers | `radial_ring.gd` (`DIRS`, `ANGLES`, `RING_RADIUS 92`, `SLOT_SIZE 60`, `SUB_RING_RADIUS 84`, `PUSH_PAST 18`, `arm(pos, slots, q)`, `_apply_touch_scale()`); `context_broker.gd::ring_collect()` merges `GlobalTools.ring_cardinals()` (N Inspect, E Measure ▸, S Undo, W Region) with each workspace's `ring_slots(req)` (NW, NE, SE, SW); `_ring_clearance()` gives the card the ring's reach | **Eight slots, all used, in every domain.** A slot is `{label, glyph, shortcut, armed, enabled, reason, callable \| children}`. Leaf diagonals today include CARTO's Label (`cartography_workspace.gd::ring_slots`); the others carry `children` **[grep, CQ-0 re-reads]**. Nothing is drawn below the S slot |
| The sub-ring ceiling | `cartography_workspace.gd::ring_slots` comment on `STYLE_PRESETS` | A sub-ring of `n` items at radius 84 with 56 px slots clears its neighbours only up to **n = 9** (chord `2·84·sin(π/9)` = 57.5 px; n = 10 is 51.9 px, an overlap). CARTO's Style slot already uses an "All styles…" overflow wedge for this reason |
| The assistant | `dcc_settings.gd` — `LLM_DEFAULTS` (`enabled`, `preset_id`, `base_url`, `model`, `local_allowed`, `remote_allowed`, …), `llm_value()`, `llm_gate_dict()`; `llm_settings_window.gd` (`_gate()` calls `bridge.llm_authorize(gate, "", consented)`); `app.gd::open_assistant_settings()`; `menus.gd` `Preferences ▸ Assistant…` | The settings and the gate are built (LLM-2). **The assistant panel is not** (LLM-3): `llm_settings_window.gd` says "The assistant panel is not in this build yet". So there is nothing yet for an Assistant button to open |
| Touch floor | `DESIGN_HANDOFF.md`: "Minimum touch target 44 px … no exceptions"; Android's 48 dp, "take the larger"; phone chips `min-height:44px` (`ANDROID_UI_SPEC.md`) | The phone has no tool palette bar; its tools are chips in the MAP sheet |

**A correction to the request's premise, stated plainly.** The measurement
buttons are not always visible: they and the brush settings are *contextual*
rows that appear when their tool is armed (`DCC_SHELL_SPEC.md` §4: "one row,
always reflecting the active tool or workspace"). A pinned command, by
contrast, is **always visible whatever is armed**. That is the point of it, and
it is why the pins live in the *palette* bar rather than inside the contextual
options bar (§3.1).

---

## 2 · What other applications teach

| Application | Mechanism | Take / refuse |
|---|---|---|
| **Blender** — Quick Favorites | Right-click any menu item ▸ *Add to Quick Favorites*; the list opens from `Q` and is stored in the user's preferences | **Take** the add-from-anywhere gesture and per-user storage. **Refuse** a second hotkey: the ring and the bar are the two entries |
| **Photoshop / Office** — Quick Access Toolbar | A strip of user-chosen commands; *Customize* lists every command; reorder, separators, reset | **Take** the whole model: a searchable command catalogue on the left, an ordered list on the right, a reset. This is the closest match to the owner's sentence |
| **Krita / Procreate** | Favourite brushes and actions on the radial / a configurable gesture | **Take** that favourites sit *beside* the fixed layout rather than reshuffling it |
| **Maya** — shelves | Drag a menu item onto the shelf to make a button | **Take** drag-to-pin from the search results on desktop (an addition to, never instead of, the explicit Add button) |
| **VS Code** — command palette ▸ pin / `Customize Layout` | Any palette command can be bound or surfaced; the palette is the catalogue | **Take**: `CommandIndex` *is* the palette; the Customise window reuses its `search()` |
| **Android launchers / Google Maps** | A bounded dock of user items with an overflow folder | **Take** the bounded dock and the overflow; **refuse** free positioning, which breaks the fixed-slot grammar |

---

## 3 · The design

### 3.1 Where the quick cluster sits (desktop and tablet)

**At the right end of the tool palette bar** (`app.gd::_tool_palette_row`),
after a `DccTheme.rule(true)` separator, not in the options bar. Reasons, all
from §1: the options bar is contextual and rebuilt on every arm; its width is
already the project's known layout hazard; the palette bar is short (135–287 px)
and has the room; and a pin must be visible whatever is armed. The cluster is
rebuilt by the same `workspace_changed` hook, and re-lit by nothing else (a pin
is a command, not a toggle, in v1 — §3.6).

A chip is the same height as the palette's cells (`role_px("h_tool_options")`
minus the bar's padding), icon plus short label, tooltip = full title, the
accelerator if `DccSettings.shortcut_binding` has one, and — when disabled — the
`why`. A chip whose command is unavailable stays and says why (the
`CommandIndex` rule, "never a dead button").

**Fit and overflow.** The cluster measures the width left in the bar after the
palette cells and shows as many chips as fit; the rest go into a `⋯` chip's
`PopupMenu` in pin order. The shown count is computed from the bar's actual
width, **not** from the window width, and the cluster **never raises the shell's
minimum width** (CQ-2's first bar). Do not use `clip_text` on a chip label
(*MISTAKES.md*: it collapses the minimum width to 1); truncate the string in
code and put the full title in the tooltip.

**Tablet:** the same bar at `DccTheme.TOUCH_SCALE`; chips are ≥ 44 dp on both
axes (the existing tablet scale table is keyed by bare desktop integers —
`h_tool_options` uses `role_px`, not `_scaled()`, for exactly that collision).

### 3.2 Phone

The phone has no tool palette bar (`_install_tool_palette_bar()` returns early
on `is_phone()`), and its tool options are the bottom sheet. So:

- **A chip row at the top of the MAP sheet's content**, visible from the *half*
  detent up. The *peek* detent (66 dp, `ANDROID_UI_SPEC.md`) keeps carrying only
  what it carries today; the peek is not enlarged. The row scrolls horizontally;
  its final chip `All…` opens a full-height list sheet of every pin (46 dp rows,
  the phone spec's row height) with the same Customise entry.
- **A *Quick* section first in `PhoneMenu` (MORE)**, 46 dp rows, so a pin is two
  taps from anywhere even with the sheet closed.
- Every tappable is **≥ 44 px on both axes** (the floor); the recommended size is
  **48 dp** because `DESIGN_HANDOFF.md` says to take the larger of the two
  platform minimums. The row is a horizontal scroller whose vertical axis is
  *disabled*: walk it for `get_combined_minimum_size().x` as *MISTAKES.md*
  ("Read a layout that overflows the screen") requires, so its content cannot
  widen the sheet.
- Phone chips are labelled (icon-only is not enough at that size).
- Search results (`CommandIndex` on the phone's search field) gain a **pin /
  unpin star** on each row, the phone's equivalent of Blender's right-click. Long
  press on a chip enters reorder (§3.5).

### 3.3 The pin: identity, kinds, what is not pinnable

`CommandIndex` rows gain four fields, added in `_walk_popup()` and `build()`
where the row is made (the index stays built from the app, never a hand-kept
catalogue — its own header's principle):

| Field | Meaning |
|---|---|
| `pin_key` | the stable identity: `menu:<action_id>` for a menu row, `extra:<slug>` for an `EXTRAS` row. Stored in the config. The key follows the id `shortcuts_dialog.gd` already persists, so it survives a reword of the title |
| `run` | **not stored**; resolved at press time from the live index: the owning `PopupMenu` and item index, executed by emitting that popup's own `id_pressed(action_id)` — the signal its real handler (`_on_file`, `_on_edit`, `_on_assets`, `_on_landmarks`…) is connected to |
| `pinnable` | bool |
| `why_not` | required when `pinnable` is false |

**Pinnable in v1:** a leaf `menu` row that is an **action** (not a check or radio
item), has a stable `action_id`, whose popup has **no `about_to_popup` handler
with side effects**, and that is not one of the dynamic id ranges
(`ID_AUTOSAVE_FIRST + i`, the recent-projects list, the GPU-device list, the
pack list). And an `EXTRAS` row that registers a handler (CQ-0 decides which of
Zoom to fit / Point sample / Theme qualify; Theme is a pointer row and likely does
not). **Not pinnable, each with its reason shown in the Customise list:**

- parameters (`kind: "param"`): they apply on a generate and have a dock control;
- readouts (`kind: "readout"`);
- check and radio rows (state mirroring is CQ-8) — **45 `add_check_item` /
  `add_radio_*` / `set_item_as_checkable` mentions in `menus.gd` [grep]**;
- rows under the 36 `about_to_popup` handlers **[grep]** that evict baked chunks
  or enumerate GPU adapters — `command_index.gd`'s header forbids firing those
  from the index, and a chip cannot be allowed to do it either;
- dynamic rows (above).

**Availability of a pinned chip.** The index records the *built* state, and its
header is explicit that `Edit ▸ Undo`, `Redo` and `Reset generation parameters`
are decided only in `about_to_popup`. So a chip does **not** copy `available`
for a row like that: it stays enabled, and the shared handler reports "nothing
to undo" in the status line exactly as the menu does. Where a command has its
own availability function (`bridge.has_world`, `app.bridge.generating`), the
chip reads that function, not the index snapshot. CQ-0 lists, per pinnable row,
which of the two applies.

**Tools are not pins.** Arming a tool is the palette's job and already has a
home (§4.5 of `DCC_SHELL_SPEC.md`); pinning `arm_tool("measure")` would put a
second copy of the Measure cell in the same bar. If the owner wants it, it is
`tool:<id>` in CQ-8, lit from `app.tool_armed`.

### 3.4 Persistence (per user, never in a world)

A new `[quick_bar]` section in `cartalith_settings.cfg` via `DccSettings`
(`_SEC_QUICK_BAR := "quick_bar"`), written through an **allow-listed accessor
pair in the style of `llm_value()`/`set_llm_value()`**: an unknown key or a
wrong type is refused and writes nothing.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `version` | int | `1` | schema version; an unknown newer version is read as empty and **not overwritten** until the user edits |
| `bar` | Array of `{k: String, t: String}` | `[]` | ordered pins: `k` = `pin_key`, `t` = the title at pin time (display fallback for a pin whose command is gone) |
| `dock` | Array of `{k, t}` | `[]` | the ring dock's user pins, ordered |
| `share` | Dictionary `"<domain>.<DIR>" → Array of {k, t}` | `{}` | pins appended to a diagonal's existing sub-ring (§4.3) |
| `assistant_button` | bool | `true` | the user's switch for the conditional Assistant chip (§5); it can only *hide* the chip, never show it when the gate is closed |

Rules: **omit rather than default** a missing key (*MISTAKES.md* "Return,
marshal or default a value that can be absent" — a pin list of `[]` is real, a
missing list is "never configured", and the Customise window may say so). A
hand-edited file can make the bar *shorter*, never invent a command: unknown
`pin_key`s render as greyed "not in this build" chips. Caps are enforced on
read as well as write (bar 24, dock 4, share per §4.3). **Nothing from this
feature is written into a world `.zip`, a project archive or an export**; a
test lists the archive's entries and greps them (CQ-1). It is per device: the
phone and the desktop keep separate bars, because their pinnable sets and
densities differ (Q7).

*Gate a write on "is there anything to save"* (*MISTAKES.md*): clearing the last
pin must still write (the empty list), or an old list reappears on reload.

### 3.5 Customise mode

One window, three tabs, reached from `Preferences ▸ Quick bar…`, the `＋` chip,
the overflow's `Customise…`, and `MORE ▸ Quick ▸ Customise` on the phone. The
window is a menu row, so `command_index.gd` finds it with no `EXTRAS` row
(*MISTAKES.md* "Move a command off the menu bar": assert by **title**, not by
`search()` rank).

| Tab | Contents |
|---|---|
| **Bar** | left: the searchable catalogue (`CommandIndex.search()`, grouped by `groups()`, unpinnable rows shown disabled **with their `why_not`**); right: the ordered pins with a **drag handle, ▲ ▼ buttons, ✕ remove** and a count `n / 24`. Add = double-click, `＋ Add`, or drag. Footer: *Reset to empty* (confirmed) |
| **Ring dock** | same two columns; right column capped at 4, plus the fixed Assistant row shown dimmed with its current state (§5) |
| **Ring slots** | one row per domain × diagonal that has a sub-ring: its built-in children (read-only, from the live `ring_slots()`), the pins appended after them, and `n of 9` remaining; leaf diagonals show "no sub-ring — cannot share" |

Reorder and remove work with **no drag** (▲ ▼ ✕ at ≥ 44 px on touch), because a
drag-only reorder is unusable on a phone and invisible to a keyboard. The window
is fully keyboard operable (Tab order, Enter adds, Delete removes, Alt+↑/↓
reorders). A 25th bar pin is refused with the reason, not silently dropped.
Quick removal also lives on the chip itself: right-click (long-press on touch) ▸
*Remove from quick bar · Move left · Move right · Customise…*.

### 3.6 What a chip does when pressed

It resolves `pin_key` against the live index, checks the row is still present
and not disabled, and emits the popup's `id_pressed`. Result and side effects are
therefore *identical* to the menu click — CQ-2's first behavioural bar compares
them. It never reimplements a command, and never runs while the engine is busy
for a command the menu would refuse (the handler's own guard answers).

---

## 4 · The cartwheel: how user items and the eight slots share

The ring's grammar is "a slot's position is its meaning" and is **full**: all
eight slots are used in every domain, four global (N Inspect, E Measure ▸, S
Undo, W Region) and four domain-owned (`MAP_CONTEXT_SCOPE.md` §5.1). An
arbitrary pin cannot simply take a slot without breaking the muscle memory the
design exists to build, and F3 (Ruling AX) fixed the cardinals. So user items
reach the ring in two bounded ways, and a third is declined.

### 4.1 The ring dock — "under the cartwheel" (the owner's sentence, literally)

A row of up to **5 chips** drawn directly **beneath** the ring disc, outside the
eight slots, in the S direction beyond the disc margin. Contents, in order:
**conditional built-ins first** (today: the Assistant chip, §5), then the user's
dock pins (≤ 4). It is drawn only when non-empty, so the ring is pixel-identical
to today's for a user who has pinned nothing and has no assistant.

- **Selection.** On a held ring (sticky), a tap on a chip runs it. In a
  continuous drag (RMB drag on desktop, hold-then-slide on tablet), moving the
  pointer *below the disc's lower edge* into a chip's rectangle selects it on
  release. The hit test is **rectangle first, only when the pointer is outside
  the disc**; inside the disc the existing 45° sector rule is untouched. It does
  not collide with the `PUSH_PAST` sub-ring gate: the S slot is Undo, a leaf, so
  no sub-ring opens in that direction. **A no-wait flick never selects a dock
  chip** — the dock is not drawn during a flick and is not part of the compass
  (the expert path stays eight directions).
- **Readout.** Hovering a chip puts its full title, accelerator and `why` in the
  ring's centre caption, as a slot does.
- **Geometry.** Chip 40 px on desktop, scaled by `DccTheme.TOUCH_SCALE` on a
  tablet (the ring already scales slots from 60 px to 96 dp, i.e. ×1.6, giving
  64 px chips; ≥ 44 dp holds). `radial_ring.gd::_clamp_centre()` and
  `context_broker.gd::_ring_clearance()` must treat the dock as part of the
  ring's reach: the ring slides inward at an edge so the dock is not clipped, and
  the context card docks beside ring *plus* dock.
- **Disabled-with-reason**, never removed, exactly as a slot (§5.2 rule 1 of the
  map-context scope).

### 4.2 The cardinals and the diagonals

N, E, S, W stay global and fixed. The domain diagonals keep their built-in
families. **No user pin replaces a built-in slot in v1** (Q3 default). Reason:
the user would be trading a position that means *Uplift* in WORLD or
*Settlement* in CIVIL for one that means *Open vault*, in every domain at once,
and the displaced tool would then have no ring slot at all.

### 4.3 Sharing a diagonal's existing sub-ring

A diagonal that already has `children` (the `▸` slots) accepts user pins **after**
its built-in children, so a user can keep "Vault note" next to the Label tools.
The cap is the sub-ring ceiling: `built_in_children + user_pins ≤ 9`, so the
neighbour chord stays ≥ 56 px (§1). The built-in children never move or
disappear; the user's items are visually separated (a thin rule in the sub-ring
caption, a different border) so position-is-meaning holds for the built-ins. A
leaf diagonal (Label, Route and any other the live read finds — CQ-0 lists them)
**cannot** be shared: converting it to a `▸` slot would turn a one-flick tool
into a two-step one. Items are keyed per domain and direction (`"cartography.NE"`)
because a diagonal's meaning is per domain.

### 4.4 Declined for v1: replacing a diagonal

Letting a user overwrite NW/NE/SE/SW wholesale (Blender's and Procreate's
answer to power users, and what `MAP_CONTEXT_SCOPE.md` §5.2 rule 6 sketches as
"right-click a tool button ▸ Assign to ring slot"). It needs a way to restore
the built-in, a decision about the displaced family, and per-domain probes for
every combination. Q3 asks whether to schedule it later.

### 4.5 Counting the slots

| Surface | Capacity | Who owns it |
|---|---|---|
| Compass | exactly 8 (the existing probe asserts "exactly 8 slots with the §5.1 cardinals" — unchanged) | built-in only |
| Sub-ring of a `▸` diagonal | ≤ 9 children | built-ins first, user pins after |
| Ring dock | ≤ 5 chips | conditional built-ins first (Assistant), user pins after |

---

## 5 · The conditional Assistant button

Owner request: *"put an llm button under the cartwheel as soon as an api/llm
has been added by the user."* `LLM_ASSISTANT_SCOPE.md` and Rulings DC/DD do not
themselves say where a button goes; **this condition is the owner's 2026-10-25
request and should be recorded as a ruling when he confirms Q6** [judgement].

**It is a built-in conditional item, not a user pin.** It cannot be removed from
the pin list (it is not in it), only hidden by `assistant_button = false`.

| State | Condition (all read from `DccSettings`, no network) | Draws |
|---|---|---|
| **Hidden** | `enabled` is false, **or** no provider is configured, **or** `assistant_button` is false | nothing; no gap reserved |
| **Shown, disabled** | `enabled`, and a provider is configured (`preset_id` or `base_url` non-empty, **and** `model` non-empty), but `bridge.llm_authorize(DccSettings.llm_gate_dict(), "", DccSettings.llm_consented_origins())` refuses | the chip, greyed, tooltip = the gate's own reason (key missing, consent not given, local/remote not allowed) — never a bare "disabled" |
| **Shown, live** | the above, and the gate allows | the chip, enabled |
| **Missing build** | `bridge` lacks the binding (`llm_endpoint_info` etc.) | hidden, as the settings window's "this build has no assistant support" row does |
| **No panel yet** | LLM-3's assistant panel is not in the build | **hidden** — a button that opens nothing is exactly the dead control this project forbids. CQ-7 is gated on LLM-3 |

"Provider configured" deliberately does not require a key: a local server needs
none, and a missing key on a remote one is a *reason to show*, not a reason to
hide (the user just added a provider and should see why it is not live yet).

**It appears within one frame of the setting changing, without a restart.**
`DccSettings` is static and has no signal; the settings window's `_store()`
must emit one (`app.assistant_state_changed`), and the cluster and the ring
dock subscribe. **It opens** the assistant panel (LLM-3) at the desktop's docked
position, the tablet's right dock, or the phone's bottom sheet, and does nothing
else: no call is made by merely showing the chip. The zero-socket bar
(`LLM_ASSISTANT_SCOPE.md` §4.1) is asserted over a session in which the chip is
shown, hidden and toggled.

Placement: first chip of the quick cluster and first chip of the ring dock, so
it keeps one predictable place. Phone: first chip of the MAP-sheet row and first
row of MORE ▸ Quick.

---

## 6 · Forms at a glance

| | Desktop | Tablet (short side ≥ 900 dp) | Phone (< 900 dp) |
|---|---|---|---|
| Quick cluster | palette bar, right end; chips `h_tool_options − pad` high | same, touch-scaled, ≥ 44 dp | MAP-sheet row (half detent up) + MORE ▸ Quick; chips 48 dp, floor 44 |
| Overflow | `⋯` chip ▸ `PopupMenu` | same | horizontal scroll + `All…` ▸ list sheet |
| Ring dock | beneath the ring, 40 px chips | beneath the ring, 64 px chips (the ring's ×1.6) | none |
| Shared sub-rings | yes | yes | n/a (thumb fan unchanged) |
| Customise | window, three tabs | same, touch density | full-height sheet, tabs as segmented control |
| Reorder | drag, ▲▼, Alt+↑↓ | drag, ▲▼ | ▲▼ buttons, long-press then drag |
| Assistant chip | cluster + dock | cluster + dock | row + MORE |

---

## 7 · Milestones

Each *done means* is checkable in the tree. Status lives in `STATUS.md`. Effort:
S ≈ 1 day, M 2–4 days, L 1–2 weeks, one lane. IDs are unique to this scope.

| ID | Milestone | Done means (numeric / verifiable) | Effort |
|---|---|---|---|
| **CQ-0** | **Pin identity and the pinnable set** | `CommandIndex` rows carry `pin_key`, `pinnable`, `why_not` (a `run` handle is resolved, not stored). A probe walks the live index and prints: total rows, pinnable count, unpinnable count by reason (check/radio, `about_to_popup`, dynamic, param, readout). **Bars:** 0 duplicate `pin_key`s among pinnable rows (or the collision is resolved by a documented disambiguator); every unpinnable row has a non-empty `why_not`; the two named examples, *Journey planner* and *Markdown vault*, are pinnable and, when run, open their windows (`journey_planner_view` / `vault_window` visible); pressing a pinned row produces the **same** status/state as clicking the menu row for 10 sampled rows. Mutation: change `_walk_popup`'s check/radio filter, the probe goes red. Also lists which ring diagonals are leaves and which carry `children`, per domain | S–M |
| **CQ-1** | **The store** | `DccSettings` `[quick_bar]` accessors with an allow-list; round-trips bar, dock, share and flags; a wrong type, unknown key, unknown newer `version`, or a pin count above the cap reads as empty/clamped without throwing; an unknown `pin_key` survives a load/save (not dropped). **Bars:** a saved project `.zip`, an export and a project archive contain **0** `quick_bar` bytes (entry listing and grep, with a leak mutation that writes one and goes red); clearing the last pin persists the empty list; a backward-compat fixture built from a **real prior** `cartalith_settings.cfg` (`git show <sha>:`-style, not an empty one) opens, resolves and re-serialises byte-identically in its other sections | S |
| **CQ-2** | **Desktop cluster** | The cluster draws in `tool_palette_bar`; chips run their command; `⋯` overflow. **Bars:** shell root `get_combined_minimum_size().x` is **identical** with 0 and with 24 pins at 1280, 1600 and 1920 px (measured on three seeds/worlds, since width is content-dependent); shown + overflow counts equal the pin count at each width; a pin on an unavailable command is visible, disabled, with a non-empty tooltip; a pin whose command was removed renders as "not in this build" and is **not** deleted; rebuilding on `workspace_changed` loses no pin; tooltips carry the accelerator from `DccSettings.shortcut_binding`. The chip height equals the palette cells' | M |
| **CQ-3** | **Customise window** | Three-tab window per §3.5. **Bars:** add 3, remove 1, reorder 1 (by button and by keyboard), quit and reload — order equals; a 25th bar pin and a 5th dock pin are refused with a stated reason; every unpinnable catalogue row is disabled with its `why_not`; `Reset to empty` is confirmed and persists; fully operable with no pointer (probe drives Tab/Enter/Delete); the window's own entry is found by `CommandIndex` **by title**. Closes with `Preferences ▸ Quick bar…`, the `＋` chip and the overflow entry all opening the same window | M |
| **CQ-4** | **Ring dock and shared sub-rings** | Dock drawn only when non-empty; hit-test, hover caption, edge flip and card clearance include the dock; shared children appended after the built-ins. **Bars:** with no pins and no assistant the ring renders **pixel-identical** to today's (framebuffer diff 0 differing pixels, windowed — headless `ImageTexture.update()` is a no-op); `_ctxring_probe` still asserts exactly 8 slots with the N/E/S/W cardinals and stays green; a flick (≤ 150 ms) never selects a dock chip (positive control: a slow drag does); for each shared diagonal at its **maximum** child count (9) all pairwise slot distances ≥ 56 px, and a 10th pin is refused; a leaf diagonal offers no sharing and says why; the dock is not clipped at any of the four screen edges for a ring opened within 20 px of that edge | M–L |
| **CQ-5** | **Tablet** | Touch scale and gesture for the cluster, the dock and Customise. **Bars:** every chip and dock chip ≥ **44 dp** on both axes (measured, not read from a constant); hold → slide down into a dock chip → lift runs it; handedness (`DccSettings.dominant_hand`) does not put the card over the dock; the pen barrel button case follows `OUTSTANDING_WORK.md` §2.12's device check and is not re-claimed here | S–M |
| **CQ-6** | **Phone** | MAP-sheet chip row, `All…` list sheet, MORE ▸ Quick, search-result star, Customise as a sheet. **Bars:** every tappable ≥ **44 px** on both axes (target 48 dp) at three portrait viewports including 412 dp wide and one landscape; **no horizontal page scroll** (root `get_combined_minimum_size().x` ≤ viewport width, walked for the disabled-axis `ScrollContainer` trap); the peek detent's height is unchanged (66 dp); with 24 pins the row scrolls and `All…` lists all 24; the phone's `PHONE_TABS` behaviour is unchanged; tab switches lose no pin | M |
| **CQ-7** | **Assistant button** (**gated on LLM-3**) | The five states of §5 on all three forms. **Bars:** a table-driven test over (enabled × provider-configured × model × gate-allows × `assistant_button` × panel-present) = the expected state for every one of the 2⁶ = 64 rows; mutation: drop one conjunct, test red; appears/disappears within **1 process frame** of `set_llm_value()` with no restart; **0 outbound sockets** over a session that shows, hides and toggles the chip (`LLM_ASSISTANT_SCOPE.md` §4.1's test, extended); the key never appears in the new section or its tooltip; the disabled state shows the gate's own reason string | S–M |
| **CQ-8** | **Deferred extensions** (own milestones, not started by this scope) | Toggle-state mirroring for check/radio rows; `tool:<id>` pins lit from `tool_armed`; per-domain visibility; layout export/import; replacing a diagonal (§4.4). Each needs its own owner answer | — |

**Order:** CQ-0 → CQ-1 → CQ-2 → CQ-3 is the desktop path and each step ships
alone. CQ-4 needs CQ-1 and CQ-3. CQ-5 needs CQ-2 and CQ-4. CQ-6 needs CQ-1 and
CQ-3 and may run beside CQ-4. **CQ-7 needs LLM-3** (the panel) and CQ-2; it can
land before CQ-4 if the dock is not yet wanted. Nothing here changes generated
output; every milestone's "green" is `cargo test --workspace` **and** a
parse-check of every `.gd` touched plus `shell/app.gd` (*MISTAKES.md* "Declare
the work green"), with probe scenes for the shell, which `cargo test` cannot see.

Annotation (Ruling BK) applies to every file: each new function, constant and
branch cites this scope or its ruling.

---

## 8 · Risks

1. **A pin runs a handler that assumes its menu opened.** Some `PopupMenu`s
   prepare state in `about_to_popup` (undo/redo availability, atlas cache cap
   enforcement, GPU device enumeration). Running the id from a chip skips that.
   *Mitigation:* the pinnable filter (§3.3) excludes the destructive ones; CQ-0's
   10-row same-effect comparison includes at least one row per remaining
   handler family.
2. **Command identity drifts.** An `action_id` is stable but menu rows are
   re-homed (Journey planner moved to the rail, then back to `Story`). *Mitigation:*
   pins key on id, not title; a missing id greys the chip with the stored title;
   the id table is already the keymap's identity, so a re-homing that breaks it
   would break rebinds first.
3. **Width.** The palette bar was introduced *because* prefixing the options bar
   broke the window's minimum width at 1 600 px. *Mitigation:* CQ-2's
   identical-min-width bar at 0 and 24 pins; overflow is computed from available
   width.
4. **The ring grows crowded and stops being learnable.** *Mitigation:* the
   compass is untouched; the dock is bounded at 5 and absent when empty; shared
   sub-rings stop at the geometric ceiling of 9; flicks never reach the dock.
5. **Two sources for "what can this do".** A pin list is user state, not a
   catalogue; but a hand-maintained default list would be the stale-document trap
   `command_index.gd`'s header names. *Mitigation:* the shipped default is empty;
   the catalogue is the live index.
6. **Phone vertical budget.** The MAP sheet already carries tool options; a chip
   row costs ~48–56 dp at the half detent. *Mitigation:* peek is untouched; the
   row appears from half detent up; measured at CQ-6 on three viewports.
7. **Settings changes are silent.** `DccSettings` has no signal, so a cluster or
   dock built before the user enabled the assistant would stay stale.
   *Mitigation:* one new `app` signal emitted by the writers that matter; CQ-7's
   one-frame bar.
8. **Pins of the assistant leak intent.** A visible Assistant chip tells a
   shoulder-surfer an LLM is configured. Minor; `assistant_button = false` hides it.
9. **Stale prose.** `MAP_CONTEXT_SCOPE.md` §5.2 rule 6 and F5, `GUI_GAP_REGISTER.md`'s
   keymap-editor proposal and `shortcuts_dialog.gd` describe customisation as
   "later". *Mitigation:* when CQ-4 lands, fix those sentences in the same change
   (*MISTAKES.md* "Change behaviour").

---

## 9 · Owner questions

**All eleven answered 2026-10-25: every recommended default approved (Ruling DH, `LARGE_ITEM_RULINGS.md`).** Q6's answer also asked that the Assistant chip's condition be recorded as a ruling, which Ruling DH does. The table below is kept as the questions were asked; the "Recommended default" column is now the decision.

Defaults were recommendations, labelled [judgement]; answered by number.

| # | Question | Recommended default | Why |
|---|---|---|---|
| **Q1** | One global pin list, or one per domain (WORLD / CIVIL / CARTO)? | **Global**, with per-domain visibility as CQ-8 | Your two examples (planner, vault) are used in every domain; per-domain doubles the editing surface before anyone has asked |
| **Q2** | Pins in the palette bar (always visible) rather than in the contextual options bar? | **Yes, the palette bar's right end** | The options bar is contextual and already the width hazard; a pin must be visible whatever is armed |
| **Q3** | On the ring: dock plus sharing a `▸` diagonal's sub-ring — but **no replacement** of a built-in slot — in v1? This lifts F5 for those two only | **Yes** | Keeps position-is-meaning for all eight slots; "under the cartwheel" is the dock, literally |
| **Q4** | Ship the bar empty, or seeded with Journey planner and Markdown vault? | **Empty** (plus the `＋` chip) | A seeded bar changes every existing user's screen; you asked for users to choose |
| **Q5** | Caps: 24 bar pins, 4 dock pins (+ the Assistant chip), 9 children per shared sub-ring? | **As stated** | 9 is the geometric ceiling in the code; 24 and 4 are labelled judgements |
| **Q6** | Does the Assistant chip appear automatically once the assistant is enabled and a provider is configured (with a user switch to hide it), and should that condition be recorded as a ruling? | **Yes to both** | It is your request; a switch to hide it costs one boolean |
| **Q7** | Per-device storage only (no sync between desktop and phone)? | **Yes** | The pinnable sets and densities differ; sync is a separate feature |
| **Q8** | Toggle commands (check and radio rows — layers, autosave, theme) in v1? | **No, CQ-8** | A chip would have to mirror state it cannot read without opening the menu; the safe v1 is actions only |
| **Q9** | Phone: chip row in the MAP sheet **and** a section in MORE, or MORE only? | **Both** | Two taps from the map, but the row costs height; the peek is untouched |
| **Q10** | May a tool (Measure, Region…) be pinned, as well as a menu command? | **No, not in v1** | The palette already holds the tools; a second Measure cell is a duplicate |
| **Q11** | Should "Remove from quick bar" be offered from any menu row's own context (right-click a menu item), as Blender does? | **Add and remove from the Customise window and the chip; the menu-item gesture only if Godot's `PopupMenu` can report a right-click** | CQ-3 checks feasibility before promising it |

---

## 10 · Considered and declined

| Idea | Why not |
|---|---|
| Putting the pins inside the tool options bar | Contextual, rebuilt on every arm, and the project's recorded width hazard (§1) |
| A ninth ring slot | The compass is 45° sectors; nine breaks the sector rule and every learned flick |
| Replacing the diagonals by default | Breaks position-is-meaning in all domains at once (§4.4) |
| Free-positioning chips | Breaks the bounded-dock grammar and cannot be made accessible by keyboard |
| Storing pins in the world file | They are a preference of the person, not a fact about the world; a shared `.zip` would carry one user's bar to another |
| A hand-written catalogue of pinnable commands | The stale-document trap; the index is built from the app |
| Showing a dead Assistant chip before LLM-3 | A control that opens nothing is the defect this project's index exists to prevent |
| Auto-pinning "recently used" | Makes the bar move under the user's hand; position must be the user's own |


---

## 11 · CQ-0 measurements (2026-10-25, live, by `_cmdindex_probe.gd` section 7)

Re-measured against the tree rather than copied; these replace the **[grep]** counts above where they differ. Method: build a `CommandIndex` over the booted shell and classify each row.

- **`ID_*` constants:** 127 `const ID_... := n` lines in `menus.gd` (the scope's 128 came from a slightly different pattern). Ids are per-`PopupMenu`: **small ids (0-7, 10-13) repeat across eight data-built popups** (the icon families, the texture sets, six landmark-family type lists), and `731` (`Places without notes`) is one command listed in both World and Story. Rule built: an item numbered from 0 marks an index popup (unpinnable, "generated row"); an identical id and title is one command (the second row is an unpinnable alias); any remaining collision is disambiguated as `menu:<group>/<id>`, which fires 0 times on this tree.
- **Check/radio:** 45 mentions in `menus.gd` text, but **116 live rows** are checkable (ladders generate many); all refused, each with a reason.
- **`about_to_popup`:** 36 connections in `menus.gd` text. Live, **every popup carries two baseline connections** (Godot's `MenuButton::_popup_visibility_changed`, and `dcc_widgets.gd`'s `style_popup` lambda), so the scope's "has a handler, so unpinnable" rule would have made **0 rows pinnable** (measured). Refined: only handlers that evict or enumerate (`_refresh_atlas_cache_menu`, `_on_gpu_devices_about_to_popup`) refuse a row; a handler that only refreshes enabled state leaves it pinnable and sets `refreshes_on_open`, so a chip must not copy `available` (**64 of the 65** pinnable rows). A lambda handler cannot be read by name; `command_index.gd::_open_handlers` states that limitation.
- **Finding, not fixed here:** `_gather_menu_buttons` also collects the `MenuButton`s inside every embedded `ColorPicker`, so **264 of the index's 693 rows** have an empty group and are colour-preset rows. They pollute search; they are marked unpinnable with that reason. Excluding them from search is its own change.
- **Totals:** 693 rows; **65 pinnable**. Unpinnable: 110 parameters, 112 check/radio, 101 generated, 19 readouts, 18 unavailable (`_todo`), 3 `EXTRAS` rows (no registered handler yet: Zoom to fit, Point sample, Theme), 264 embedded picker rows, 1 alias. (A row takes the first reason that applies, so a picker row that is also a check row counts once, under the picker.)
- **The two named examples:** `Story > Journey planner` (`menu:726`) and `World > Markdown vault` (`menu:45`) are pinnable and, run through `CommandIndex.run_row()`, open `journey_planner_view` and `vault_window`. A pressed row emits the owning popup's own `id_pressed`, the signal a menu click emits, so the "same state as a click" bar holds by construction; no separate 10-row comparison was made.
- **Ring diagonals (read at the `ring_slots` symbols, not run):** leaves are CARTO NW `Label` and CIVIL SW `Route`. Every other diagonal carries `children`: CARTO NE `Icon`, SE `View`, SW `Style`; CIVIL NW `Settle`, NE `Territory`, SE `Way`; WORLD NW `Uplift`, NE `Carve`, SE `Freehand`, SW `Paint`. Sharing (Q3) therefore offers 10 of 12 diagonals across the three domains.
- **Q11:** a scratch probe in an embedded viewport showed a right-click does **not** emit `id_pressed` and leaves the popup open, but `PopupMenu.window_input` **does** receive the right-button event, and `get_focused_item()` names the hovered row after a motion event. Feasible in principle; the probe's first right-click read `focused=-1`, so CQ-3 must confirm item identification with a windowed check before promising the gesture.
- **Mutation:** weakening the check/radio filter in `_why_not_pinnable` turns the probe red (`no check/radio row is pinnable got=78`); restored, green.

---

## 12 · CQ-1 and CQ-2 measurements (2026-10-26, live, by `_quickbar_store_probe.gd` and `_quickbar_probe.gd`)

Produced by the two probes named, windowed, from a scratch worktree with its own `config/name` and a wiped user dir; the commands are in each probe's header. Numbers are what those runs printed on this machine; none is quoted from elsewhere.

- **Store (CQ-1).** `[quick_bar]` is written only through `DccSettings.set_quick_bar_value()`; the allow-list is `QUICK_BAR_DEFAULTS` (five keys, four user-settable; `version` is the format's). 22 refused writes (unknown key, `version`, wrong type, malformed or repeated pin, 25th bar pin, 5th dock pin, 10th share pin, cardinal or malformed share key) leave the file byte-identical. Reads are lenient and writes strict: a hand-edited file is cut to 24 / 4 / 9, malformed entries are dropped and a missing title reads as `""`. A stored `version` that is newer or not an int reads as an empty section, is not rewritten by reading, and the first user edit replaces the section (the newer build's other keys go with it; stated in `set_quick_bar_value()`'s comment). 114 checks.
- **Prior-format fixture.** `cartalith-native/docs/evidence/quick-bar-2026-10-26/prior_cartalith_settings.cfg` was produced by the pre-CQ-1 setters at `5047cc99` (hand, theme, units, recent, autosave, cpu, lod, atlas, landmark layer, a shortcut override, a named layout, asset favourites and recent, a window size, three `[llm]` keys, one lighting default: 15 sections). After quick-bar writes the text before `[quick_bar]` is byte-identical to it.
- **Leak (CQ-1).** With pins stored, a project saved from a generated world (39 zip entries, 1 124 926 bytes), the raw zip and a 2048 PNG export contain none of `quick_bar`, `QBLEAK`, the ghost pin's key or its title. The detector's positive control (the same zip with a `quick_bar` entry appended) is hit once. The scope's "leak mutation" would be a change to the Rust save path; none was made, so the control is the stand-in.
- **Width (CQ-2).** The shell's minimum width is the root `VBoxContainer`'s combined minimum (`_app` itself, a plain Control, reads 0). Identical with 0 and 24 pins at 1280 / 1600 / 1920 on seeds 1011, 2022 and 3033 (WORLD) and in CIVIL and CARTO on the first; run values 1067-1116 desktop, 975-1032 tablet, content-dependent as `MISTAKES.md` says. The cluster's own minimum is the `＋` chip's width whatever the pin count. Shown + overflow = 24 in every one of those cells; for example WORLD 10 + 14 at 1280, 12 + 12 at 1600, 17 + 7 at 1920.
- **Size (CQ-2).** Desktop: chip height equals the palette cell's, 39 as laid, floor 30. Tablet: 55 as laid, floor 44; every shown chip and the `＋` chip measured 44 x 44 or larger (`⋯` is not size-asserted by the probe).
- **Cost.** One `CommandIndex` build per refresh: median 7.7 ms (min 7.6, max 8.3) desktop, 7.8 (7.7 to 9.9) tablet, seven runs, 693 and 698 rows. A refresh runs on a workspace change, a world load and a generation finish; a press builds its own fresh index.
- **Divergences from sections 3.1-3.3, each deliberate.** (1) A chip is live exactly when the index calls its row pinnable *now*: a row of a menu that refreshes state on open and was built disabled stays disabled with its reason, where section 3.3 kept it enabled (a chip cannot run a disabled row, and `run_row()` refuses it). A refreshing row built *enabled* stays live and the handler answers. (2) Chips are text only: menu rows have no icons to reuse. (3) Until CQ-3, `＋` opens a plain menu of the pinnable commands grouped as `CommandIndex` groups them, and a chip's right-click menu offers Remove / Move left / Move right; `Customise…` is CQ-3's.
- **Not measured here.** Phone (CQ-6) and any on-glass pass; the Assistant chip (CQ-7). `menus.gd` now has an `Assistant panel` row (id 901, LLM-3), so CQ-7's gate on the panel is open; that is for its own milestone to re-verify.
