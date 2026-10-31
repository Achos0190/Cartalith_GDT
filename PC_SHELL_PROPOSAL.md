# PC shell proposal: one selection, one year, one Context panel

**A proposal of 2026-10-31.** It defines; it does not track. What has been built is
`cartalith-native/docs/STATUS.md`'s answer. It has no status column.

**The interactive mockup** is the `Desktop.dc.html` frame in `design/mobile-shell-2026-10-31/`. It is the
same shell component the phone and tablet frames draw, now also at mouse density. The folder's `README.md`
lists what to try. It is also live in the claude.ai Design canvas named there. Screenshots are in
`cartalith-native/docs/evidence/desktop-shell-2026-10-31/`.

**The owner's request, verbatim (2026-10-31):** *"For the pc version everything is pretty much in place plus
some extras such as the chronos timeline formatting for notes and using that for the timeline in cartalith
[...] I'd like you to put the same effort in refractoring the PC GUI and check which menu's and functions could
be optimised so each feature flows naturally and the context windows are all properly locked together. Again
make me an interactive mockup. And don't forget the assistant function."*

It was followed the same evening by: *"Do all GUI's keep the loading screen?"* §3.10 answers that.

**Related documents:**
- `MOBILE_SHELL_PROPOSAL.md` is the phone and tablet half. The tablet is the desktop shell at touch density, so
  most of this document reaches the tablet too; §3.11 says exactly what.
- `VAULT_EDITOR_PROPOSAL.md` owns the note editor; the Note tab here is a view of it.
- `LLM_ASSISTANT_SCOPE.md` owns the assistant; the Ask tab is the docked panel its §6 already specifies.
- Making every Cartalith action reachable by the assistant without flooding its context is the owner's next
  question. It is a separate document, not this one.

---

## 1. What ships, read at the symbols

**Method.** A read-only audit (an agent, 2026-10-31) walked the shell code. The counts in the table below were
then re-run in this session, in `cartalith-native/godot-project/shell/`, with the command shown. Anything the
audit read but this session did not re-run is marked *(audit)*.

| Fact | Evidence |
|---|---|
| 23 shell windows extend `AcceptDialog`, which is exclusive: while one is open, the map, menus, docks and timeline take no input | `grep -l "^extends AcceptDialog" *.gd \| wc -l` → 23. `popout_host.gd` clears `exclusive` only for a native pop-out |
| No window, no dialog, no assistant file, no planner and no right dock listens to the year | `grep -l timeline_changed *_window.gd *_dialog.gd llm_*.gd journey_planner_view.gd right_dock.gd \| wc -l` → 0 |
| The year has a writer that skips the signal: CIVIL ▸ Timeline's pills, scrub, ▶ Animate and Step call `bridge.civ_goto_year` directly | `workspaces/civilization_workspace.gd::_tl_goto_year`; the strip's `dcc_shell.gd::tl_set_year` is the only writer that emits `timeline_changed` |
| The strip clamps the year to −400…1200 | `dcc_shell.gd::TL_YEAR_MIN`, `TL_YEAR_MAX`. The CIVIL dock allows −9999…9999 and the Place editor ±100 000 *(audit)* |
| The strip shows only in CIVIL | `app.gd::_on_workspace_changed` sets `timeline_bar.visible` *(audit)* |
| The selected settlement is pinned into the right dock from 8 places, outside any selection model | `grep -rn "right_dock_ctrl.on_settlement_selected(" .` → 8 (7 in `workspaces/civilization_workspace.gd`, 1 in `workspaces/infrastructure_workspace.gd`). 3 of them skip `CivilizationWorkspace._selected_index`, and Edit ▸ Deselect clears only that index *(audit)* |
| "The current place" means a settlement only; viewing its faction drops it | `right_dock.gd::current_settlement_index` returns −1 unless the dock's context is a settlement *(audit)* |
| Opening the assistant starts a new conversation | `llm_assistant_window.gd::open` calls `_rebuild()` |
| The assistant's place is a snapshot taken at open, with no year, faction, conflict or open note | `llm_assistant_window.gd::_current_place`, `::_args` *(audit)* |
| The planner replaces the map | `journey_planner_view.gd::_show` sets `app.viewport.visible = false` *(audit)* |
| The desktop bar is File · Edit · Assets · World · Story · Preferences · Window · Help; World ▸ Generate has no PC row | `menus.gd::build`'s non-tablet branch; Ctrl+R Run pipeline exists in the tablet's `_world()` only |
| 130 menu ids | `grep -cE '^const ID_[A-Z_0-9]+ := ' menus.gd` → 130 |
| Accelerators in use | Ctrl+A C D F N O R S V W X Z; Ctrl+Shift+E M N P S T Z; Shift+A D L; Shift+J, Q and Menu/Shift+F10 in `app.gd::_unhandled_key_input`; Delete (`grep -o "KEY_[A-Z0-9_]*[^)]*" menus.gd`) |

**What that means in use** (the audit's flows, re-told; *(audit)*):

- **F1, write a place's history and see it on the timeline.** Attaching a note from the Place editor stacks the
  vault over the editor. A chronos event added there never reaches the strip, the map or CIVIL ▸ Timeline. The
  strip cannot reach a year outside −400…1200.
- **F2, plan a journey between two selected places.** The selection is never used. The planner hides the map, so
  stops are picked from a map you cannot see.
- **F3, a faction, then its capital, then its note.** Every hop closes the surface before it: "Capital: X" is plain
  text, and the roster hides itself to open the note.
- **F4, ask the assistant about the selected place and act on the answer.** Asking about another place loses the
  conversation. Citations are dead text, and saving means picking the note by hand from up to 400 files.
- **F5, scrub the year and read a place.** The map moves; the right dock and CIVIL ▸ Timeline do not. Press a CIVIL
  pill instead and the strip and the journey markers stay behind.
- **F6, find a faction and open it.** Find only moves the camera; World ▸ Factions… opens on the last-used faction.
- **F7, review a conflict in its year.** Picking one leaves the year where it was, so its campaign may not draw.
- **F8, edit a place while comparing a neighbour.** The editor blocks the map; popped out, it stays on the first
  place while the dock follows each click.

The diagnosis is one sentence: **each surface keeps its own copy of "what" and "when", and nothing tells the
others when one changes.** Everything below follows from removing the copies.

## 2. The rule

Three things are shared, and each has exactly one writer:

1. **The selection**: `{kind, id}`, where kind is a settlement, landmark, faction, culture, conflict or note.
2. **The year**: one value, one range, one writer.
3. **The note focus**: the selection's own note, found through its entity binding (`KnowledgeLink`), not by name.

Every surface reads them and listens for their change. No surface keeps a copy. A surface that must stay on
something (a pinned panel, a pinned window) stores its own subject **as a pin**: an explicit, visible state,
never a stale copy.

## 3. The design

### 3.1 One selection

- **Producers.** Every producer calls one path: a map click, a Find result, the context card, a row in any
  window, a link in Inspect, a `[[link]]` in a note, a citation in an answer. All of them select. Find (Ctrl+F)
  selects its hit as well as moving the camera (finding S4).
- **History.** Alt+← and Alt+→ walk back and forward through the selection, like a browser. Edit ▸ Previous
  selection and Next selection carry the same keys.
- **Deselect** (Edit ▸ Deselect, Ctrl+D, or Esc on the map) clears the one selection. Every surface empties
  together, and Alt+← brings it back.
- **Delete** acts on the selection the panel shows, and its confirmation names it. It can no longer act on a
  second, hidden holder (finding S1).
- **The keyboard card.** Shift+F10 and the Menu key open the card at the selection, as `MAP_CONTEXT_SCOPE.md`
  §4.2 specifies, not at the mouse (finding S3).
- **On the map.** The selected place has a solid ring and a pinned place a dashed one. A selected faction rings
  every place it holds in the shown year. A selected conflict draws its front.

### 3.2 One year

- **One writer.** Every move of the year goes through the strip's `tl_set_year`, which emits
  `timeline_changed`: CIVIL ▸ Timeline's pills, scrub, ▶ Animate and Step, the Place editor's year chips, a chronos
  row in a note, a year in an assistant answer, a conflict picked from a list (finding T1).
- **Every reader listens.** The right dock's garrison and siege rows, CIVIL ▸ Timeline's pills and "Active year",
  the roster's History tab, the conflicts window, the Place editor and the assistant's context all redraw on
  `timeline_changed` (finding T2).
- **One range**, taken from the world: the earliest founding, chronos entry or recorded year to the latest. It
  replaces the strip's fixed −400…1200 (finding T3).
- **One transport.** Play, ×1 / ×10 / ×100 and Step live in the strip. CIVIL ▸ Timeline's ▶ Animate becomes
  "play through recorded years", a mode of the same transport.
- **The strip in every domain.** This is `GUI_GAP_REGISTER.md` CV-24, *"the year scrubber as program scope"*,
  which the owner ruled on 2026-09-03 must wait for a design pass, because the fixed-height strip has no room for
  CIVIL ▸ Timeline's controls. This is that design, and it goes to the owner as PQ2:
  - **Collapsed**, the strip is one line: the year, play, the slider, and ticks under the slider. The ticks are
    the world's events, its conflicts as spans, and the selection's own chronos entries in the accent colour.
  - **Expanded**, it adds two rows.
    - The first holds CIVIL ▸ Timeline's controls at program scope: the recorded years as pills, **+ Record year**
      (one undo step), the three filters (Exist only, Ghost removed, Highlight new) and the speeds.
    - The second is the event list: world events, conflicts and the selection's chronos entries, each labelled
      with its source note. A row moves the year; a conflict row also selects the conflict.
- **What the year moves** (each shown in the mockup):
  - The map colours each settlement by who held it that year, and draws places not yet founded as outlines.
    Exist only hides those outlines.
  - Inspect says who held the place then.
  - The chronos row the year falls in lights up in the note.
  - The assistant's year chip changes.
  - Picking a conflict moves the year to its first year, so its campaign draws (finding F7).
- **Chronos reaches the map's timeline** (finding T5). The owner's chronos formatting (`cartalith_vault::chronos`,
  Ruling AM) becomes a source for the strip's ticks and events, for the selected entity, faction notes included.
  The Place editor's "Add event" and the card's "Add event" become one verb, defaulting to the shown year, not to
  year 0 (finding T6).

### 3.3 The Context panel (the right dock)

The right dock becomes the **Context panel**. It shows one subject: the selection, or what the panel is pinned to.

**The header carries:**
- back and forward, which are the selection history;
- the subject's kind, name and one line of facts;
- **FOLLOWING** or **PINNED**, with a pin button;
- hide (Window ▸ Context panel).

**Pinning.** Pin the panel and click something else, and a banner says *"selection is now X"*, with **PIN TO IT**
and **FOLLOW**. That is how a place is compared with a neighbour without a second window (flow F8).

**Three tabs.** Ctrl+1, Ctrl+2 and Ctrl+3 switch between them.

- **Inspect.** One builder per kind (settlement, landmark, faction, culture, conflict, note).
  - **Every name is a link.** It selects that thing: a faction, a capital, a neighbour, a conflict, a faith. Every
    year is a link too; it moves the timeline.
  - **A settlement shows:**
    - an AT YEAR block, giving who held it in the shown year or when it is founded;
    - its facts;
    - its political history as dated periods, each with its holder and a "go to" year;
    - the conflicts fought over it, its nearest neighbours and its roads;
    - **City viewer…** and **Plan a journey from here**.
  - **A faction shows:**
    - at year: the settlements it held and whether it was at war;
    - identity, faith and capital, as links;
    - relations with every other faction;
    - its conflicts;
    - its settlements in the shown year.
  - **A conflict shows** its years, sides and places, all links.
  - **Edit is Inspect's edit mode** for a settlement: name, class, faction and fortified, applied as one undo step.
    The Place editor window keeps all six of its tabs (`place_editor_window.gd::TAB_ORDER`: Overview, Economy &
    notables, Timeline, Political history, Vault notes, Layout). It is opened from edit mode, and it follows the
    selection until pinned (§3.6). Nothing leaves the editor; the panel adds a quick path to the four most-used
    fields. (Per `MISTAKES.md`, re-framing a window starts from its own builder.)
  - **An armed tool appends its section below the subject, never above or instead of it.** That is the ruling
    *"Selection wins; the tool appends a section"* (`LARGE_ITEM_RULINGS.md`, the `rdExtraMode` row).
- **Note** (§3.4).
- **Ask** (§3.5).

### 3.4 The Note tab

- **What it shows.** The tab shows the subject's own note, found through its entity binding, so renaming a place
  does not lose its note.
- **Preview** is the default. **Write** edits the Markdown in place, with the source coloured, autosave, and
  Ctrl+B, Ctrl+I, Ctrl+K and Ctrl+E. The editor is `VAULT_EDITOR_PROPOSAL.md`'s, in a narrower frame.
- **Links.** A `[[link]]` selects what it names, so the map, Inspect and Ask follow. A link to a missing note
  creates it, as in any Markdown vault.
- **Chronos.** A chronos row moves the year, and the row the year falls in is lit.
- **No note yet.** The tab offers **Create *name*.md** from the kind's template, already linked to the subject.
- **The full vault window.** ⧉ opens the note in the vault window for split view, the outline, backlinks and
  rename. The vault window remains `World ▸ Markdown vault…` (now Ctrl+Shift+V).
- **Private notes** (Ruling DP) show as private, and are never offered to Ask.

### 3.5 The Ask tab: the assistant, docked

`LLM_ASSISTANT_SCOPE.md` §6 specifies a docked panel; the build shipped a floating exclusive window (finding A1).
The Ask tab is that panel. Everything the scope and Rulings DC, DD, DI, DM, DN, DO, DP, DQ, DR and DS settled
stays: off by default, every capability behind its own switch, consent, profiles, proposals that change nothing
until Apply, grouped undo, and regenerate on Apply. What changes is where the assistant lives and what it can see
and touch.

**Context chips.** A row of chips shows exactly what goes with the next message: the subject, the year and the
subject's note. The chips follow the panel, pinned or not, and a click leaves one out. The context is read **at
every Send**, not when the panel opened (finding A2). Each sent message records the chips it went with.

**The conversation.** One conversation persists across selections. It is cleared only by **New**, so switching
places never loses it (A1). Answers link back (A3):
- a name selects that thing;
- a year moves the timeline;
- a note citation opens the note in the Note tab, at the heading.

Each answer carries a "used" line, naming its tool calls, and a "sources" line.

**Cards.** A proposal is a card, and nothing changes until Apply:
- a setting card regenerates per Ruling DS, behind the loading screen;
- a note card appends under a marked heading, never touching the user's own text;
- a view card applies at once.

Each Apply is one undo step. A note card goes to the subject's linked note, preselected, instead of a pick from a
flat list of 400.

**Every shipped control has a home in the tab:**

| Shipped (`llm_assistant_window.gd`) | In the Ask tab |
|---|---|
| Task chips `_TASKS`: Describe place, Story, Note (Summarise, Suggest links, Clean up), plus Commands and Edit | The prompt row above the input. It offers the subject's own tasks, so a place offers Describe, History, Suggest links and Draft a chronos entry. Commands and Edit stay as modes; Edit keeps its painted region |
| The knowledge box (`search_knowledge`, `read_source`, `related_sources`) | Unchanged; its citations become links |
| Sources card | The answer's "sources" line, and citations in the text |
| Save to a note, picked from a list | The card's Apply; the linked note is preselected, and "another note…" keeps the list |
| Consent, settings, profiles (Rulings DD, DQ) | The tab's header: provider line, ⚙, **New**. The off state says nothing is sent and links to Preferences ▸ Assistant |
| `assistant_chip.gd::open_panel` and the ring-dock item | Open the Ask tab, scoped to the subject (finding A4) |

**Entry points.** Ctrl+J opens Ask from anywhere and focuses the input. Every Inspect page and the context card get
**Ask about this**. Story ▸ NOTES keeps a row, *"Ask the assistant about the selection"*, and Window ▸ Context panel
▸ Ask is its menu home.

**Out of scope here.** The mockup's answers are scripted over the stand-in world: no model is called. Which actions
the assistant can reach, and how they are described to the model without filling its context, is the owner's
next question and gets its own document.

### 3.6 Windows that do not block the map

The context windows become **non-exclusive in-app windows**:
- the faction roster;
- the Place editor;
- the City viewer;
- Conflicts;
- Saved journeys.

**How they behave:**
- **Movable.** A window is dragged by its title.
- **Above the map, not in front of everything.** It never blocks the map, the menus, the docks or the strip.
  Find (Ctrl+F) opens above it.
- **One host with a back stack.** One window opening another pushes onto it instead of closing the first or
  stacking a second exclusive window (finding W2).
- **Following.** Each follows the selection until its **FOLLOW** chip is pressed to pin it (finding W3):
  - the roster shows the selection's faction, or the holder in the shown year when a settlement is selected;
  - the Place editor and the City viewer show the selected settlement;
  - Conflicts marks the selected conflict.
- **Opening on the right subject.** World ▸ Factions…, Story ▸ Faction history… and CIVIL ▸ Open factions… open on
  the current faction, not the last one used (finding W6). The roster's "Capital: X" is a link that selects it
  (flow F3).
- **Window ▸ Open windows** lists every open window, the Context panel and pop-outs included (finding W5).
- **Refresh.** Roster edits refresh the Context panel, and an assistant save refreshes the vault (finding W7).

**What stays modal.** The Data manager, the Asset library, New world, Export, Open project and file dialogs stay
modal. Ruling DK Q4 already keeps the first two and the file dialogs out of pop-out. A modal window there
protects a whole-world operation.

**Pop-out is unchanged** (Ruling DK: off by default, a global setting and a per-window button, roster first). The
two features are orthogonal: pop-out decides **where** a window draws, and this decides whether it **blocks**.
`PopoutHost.prepare` already clears `exclusive` for a native window. This makes the docked default match.

### 3.7 The planner beside the map

- **On the desktop**, the journey planner opens as a wide workspace in the left dock, and the map stays visible and
  live (finding W4). Ruling DK Q5 says the same for its native pop-out: *"keep the main map live while the planner
  is native."*
- **The selection feeds the stops.** With a settlement selected, the planner offers **Start here**, **Add as stop**
  and **End here**, each one undo step (flow F2). The context card's "Plan journey from here" and "Journey to here"
  stay. The route's name is the same everywhere: the card and the planner currently disagree, "Journey 1" against
  "Route #0" (finding M2).

### 3.8 Menus

Each change names the audit finding it answers. The PC bar keeps its eight menus and their order. The phone's MENU
and the tablet's bar mirror these rows, as `MOBILE_SHELL_PROPOSAL.md` requires.

| Menu | Change | Why |
|---|---|---|
| World | **Generate world, Ctrl+R**, first. The tablet's Run pipeline row is the same command | M1: no PC row or key today |
| World | Factions… gains **Ctrl+Shift+F** and opens on the current faction; Markdown vault… gains **Ctrl+Shift+V** | W6; M8 (no keys for either) |
| World | The second Timeline row becomes the signpost *"Timeline is in Story ▸ Timeline"* | M3: two rows, one action |
| World (name) | **Keeps its name.** Ruling CK named it, and renaming it is PQ6, asked by name | M4: the menu holds records while the WORLD domain is terrain |
| Edit | Deselect clears the one selection. **Previous selection (Alt+←)** and **Next selection (Alt+→)** are new | S1; §3.1 |
| Story | "Political history of this place…" becomes **"…of the selection"** (the Inspect page). Faction history… opens on the current faction | S2, W6 |
| Story | "Notes for this place" becomes **"Note for the selection"** (Ctrl+Shift+N, the Note tab), for any kind | S2 |
| Story | "Assistant panel…" becomes **"Ask the assistant about the selection" (Ctrl+J)** | A1, A4 |
| Story | Its "Places without notes…" row becomes a signpost to World ▸ Records, where the same id stays | M3 |
| Window | "Right dock" becomes **"Context panel"**, with a submenu: Inspect, Note and Ask (Ctrl+1, Ctrl+2, Ctrl+3), Pin to the current subject, Follow the selection | §3.3 |
| Window | Open windows lists everything open | W5 |
| Help | "Keyboard shortcuts…" becomes **"Keyboard reference…"**, read-only; the editable one stays in Preferences | M2: one label, two behaviours |
| Assets | "Import image…" is removed; it opened the Asset library under a second name | M2 |

**Search keeps every moved title.** Per `MISTAKES.md` (*"Move a command off the menu bar"*), every title that stops
being a menu row gets a `command_index.gd` `EXTRAS` alias **in the same change**, so search still finds it. "Import
image" becomes an alias of the Asset library. Every accelerator moved out of a menu gets a
`shortcuts_dialog.gd::UNLISTED` row, so the reference lists it.

**Three-level nesting is flattened** (finding M7):
- Assets ▸ Landmark types ▸ family ▸ type;
- Preferences ▸ Lighting rig defaults ▸ Azimuth ▸ value;
- Preferences ▸ Tile size · LOD levels ▸ LOD levels;
- Preferences ▸ Atlas cache ▸ Size cap;
- Window ▸ 3D capability ▸ Force tier.

**Desktop menus open as side-by-side columns.** An open submenu is a second column beside its parent, so the whole
path stays visible. Moving across the bar while a menu is open switches menus. Rows are one line, with the
accelerator on the right.

**Desktop search runs commands.** The phone already runs a pinnable row through `DccQuickBar.run_pin_on`; the desktop
draws the same rows disabled (finding S4). They become runnable on the desktop too.

### 3.9 Keys

- **New:**
  - **Ctrl+R** generate (the PC gains the tablet's key);
  - **Ctrl+1**, **Ctrl+2** and **Ctrl+3** for Inspect, Note and Ask;
  - **Ctrl+J** to ask;
  - **Ctrl+Shift+F** for Factions and **Ctrl+Shift+V** for the vault;
  - **Alt+←** and **Alt+→** for selection history.
- **Free.** None of them is in the shipped list (§1). Alt+← and Alt+→ are not used by the shell; on a platform where
  the window manager takes them, Edit ▸ Previous and Next selection remain.
- **Rebindable.** Q (the cartwheel at the pointer) and Menu or Shift+F10 (the card) move out of
  `app.gd::_unhandled_key_input` into the rebindable table, and into the reference (finding M8).
- **Esc** closes the topmost transient: a menu, the card, the ring, Find, the topmost window, the edit mode. With
  nothing open, it deselects.

### 3.10 The loading screen, on every form factor

**Yes: every GUI keeps it.** The loading screen is the busy overlay, `busy_overlay.gd` (owner, 2026-10-11). It is a
scrim that takes every input while the engine works, with a card holding:
- the action;
- the relief trace, a ridge line that advances only to the stages the engine has reported;
- the stage line;
- one line of wit per stage (`loading_messages.gd`).

`app.gd::_mount_busy_overlay` mounts it once, for the desktop, the tablet and the phone alike. On a phone the card
scales by `DccTheme.phone_scale()`.

**Nothing here removes it or narrows it:**
- the assistant's Apply → regenerate runs behind it (Ruling DS);
- the non-exclusive windows of §3.6 are muted by it the way pop-outs already are (`busy_overlay.gd::_mute_subwindows`
  and its window mirror);
- MS-1's live phone/tablet switch waits for it to clear (`MOBILE_SHELL_MS1_SPEC.md`).

**The mockup now draws it on all three form factors.** It shows the shipped loading lines, two per stage, verbatim
from `loading_messages.gd` `NEW_MSGS`, and holds every key and click until the run ends. It has no cancel, as
shipped.

### 3.11 What reaches the tablet and the phone

- **The tablet** is the desktop shell at touch density, so it gets:
  - the Context panel, with 44 dp targets (measured in the mockup at 1194 × 834);
  - the strip's expanded row;
  - the menu rows.

  Its right dock changes from `MOBILE_SHELL_PROPOSAL.md`'s single inspector to the three tabs. Its windows stay
  modal sheets, because a touch user cannot keep a floating window and the map both in reach. That is a change to
  the tablet proposal, and it is PQ1's second half.
- **The phone** gets the model, not the panel: one selection, one year, the chronos ticks and the menu rows. Its
  sheet keeps showing the selection. On the phone, Note opens the vault at the note and Ask opens the assistant's
  sheet.

## 4. The flows, before and after

| Flow | Today | Proposed (as the mockup does it) |
|---|---|---|
| F1 Write a place's history and see it on the timeline | Editor → Vault tab → attach (stacks) → Add event (year 0) → seen in the editor only | Select → Ctrl+2 → Write the chronos line → its tick appears on the strip and the row lights at its year |
| F2 Journey between two selected places | Logistics → empty planner, map hidden → leave → card on A → card on B | Select A → *Plan a journey from here* → planner beside the live map → select B → **End here** |
| F3 Faction → capital → note | Politics → roster (blocks) → capital is text → close → click map → reopen roster → note (roster hides) | Select a place → click its faction → click "Capital" → Ctrl+2. Alt+← back at any point |
| F4 Ask about the place, act on it | Select → Story ▸ Assistant panel (snapshot) → Send → pick the note from a list → Save | Select → Ctrl+J → *Describe…* → click a citation → Apply → the Note tab shows it |
| F5 Scrub and read a place | Strip moves the map only; a CIVIL pill moves the rest but not the strip | Drag the strip: map, Inspect, chronos row and Ask's chip move together |
| F6 Find a faction, open it | Ctrl+F (camera only) → World ▸ Factions… (last used) → pick again | Ctrl+F → Enter (selected) → Ctrl+Shift+F opens on it |
| F7 A conflict in its year | Story ▸ Conflicts… → pick (window closes, year unchanged) → scrub by hand | Story ▸ Conflicts… → pick: selected and the year moves to its start; the front draws |
| F8 Edit a place beside a neighbour | Editor blocks the map; popped out, it never follows | Pin the panel on A, click B: the banner offers PIN TO IT or FOLLOW; or edit A in Inspect while a pinned Place editor stays on B |

## 5. Mapping onto the code

These are the symbols a build would change, named so the build can start from them. This proposal changes no code.

| Concern | Today | Proposed |
|---|---|---|
| Selection | `RightDock` contexts plus `CivilizationWorkspace._selected_index`; 8 direct pins (§1) | `DccApp.selection` (`{kind, id}`) and `DccApp.selection_changed`; one `DccApp.select(kind, id, opts)`; the 8 pins call it; `_selected_index` reads it; `clear_selection` clears it; a selection history on `DccApp` |
| Year | `DccShell.tl_set_year` plus `civilization_workspace.gd::_tl_goto_year` and the Place editor's direct `civ_goto_year` | `tl_set_year` is the only caller of `bridge.civ_goto_year`; every reader in §3.2 connects `timeline_changed`; the range comes from the world |
| Strip | `dcc_shell.gd` `timeline_bar`, CIVIL only | Shown in every domain; an expanded row holding the controls `civilization_workspace.gd::_build_timeline_years` and `::_build_timeline_playback` build today; chronos ticks from `vault_entity_chronos` |
| Right dock | `right_dock.gd` (`RightDock`) | `ContextPanel`: a header, the tabs, `RightDock`'s builders as Inspect, `markdown_editor.gd` as Note, `llm_assistant_window.gd`'s builders as Ask |
| Assistant | `LlmAssistantWindow` (an `AcceptDialog`); `open()` → `_rebuild()` | Its content inside the Ask tab, built once per session; `_current_place` and `_args` read `DccApp.selection` and the year at Send; `note_saved` connected to the vault |
| Windows | 23 `AcceptDialog`s, `exclusive` | The five context windows: `exclusive = false`, one host with a back stack, a follow/pin state; `busy_overlay.gd` mutes them as it mutes pop-outs |
| Planner | `journey_planner_view.gd::_show` hides `app.viewport` | Drawn in the left dock region, wide; reads the selection |
| Menus | `menus.gd` | The rows in §3.8; `command_index.gd` `EXTRAS` and `shortcuts_dialog.gd` `UNLISTED` in the same change |

## 6. Milestones

Each bar is something a probe or a grep can show. The probes run windowed where pixels or input routing matter, per
`MISTAKES.md`.

| ID | Delivers | Bar |
|---|---|---|
| PC-1 | One selection | `grep -rn "right_dock_ctrl.on_settlement_selected(" shell/` → 0 outside `DccApp.select`. A probe selects a place three ways (map, Find, a roster row), and each time the dock, CIVIL's index and the assistant context name the same place. Deselect empties all three. Alt+← restores |
| PC-2 | One year | `grep -rn "civ_goto_year(" shell/` → only inside `tl_set_year`. A probe moves the year from a CIVIL pill, and the strip, the journey markers, the dock's garrison row and CIVIL's pills all show it |
| PC-3 | The strip at program scope (CV-24) | The strip is visible in WORLD, CIVIL and CARTO. Recorded years, + Record, the three filters and the speeds work from every domain. Chronos ticks for the selection's note are drawn |
| PC-4 | The Context panel: header, pin, Inspect | Every name in Inspect selects; the pin holds while selecting elsewhere; an armed tool's section is below the subject |
| PC-5 | The Note tab | A `[[link]]` selects; a chronos row moves the year; Write autosaves; a missing note is created and bound to the subject |
| PC-6 | The Ask tab | Opening, closing or reselecting never empties the transcript. The context at Send includes the year. A citation selects. A note card's Apply goes to the linked note in one undo step. The hostile-string corpus (scope §4 bar 3) re-runs green, because the request changed |
| PC-7 | Non-exclusive context windows | A windowed probe clicks the map while the roster is open, and the map takes the click. The roster follows a new selection until pinned. The busy overlay mutes it during a generate |
| PC-8 | The planner beside the map | With the planner open, the map draws and takes input. Start / Add / End from the selection each add one undo step |
| PC-9 | Menus and keys | Every row in §3.8. `command_index.gd` finds each moved title by TITLE. `shortcuts_dialog.gd` lists every new key. No accelerator collides (`grep` of `menus.gd` and `app.gd`) |

PC-1 and PC-2 come first: everything else reads them. PC-4 and PC-6 can run in parallel after them. PC-7 shares
`popout_host.gd` with Ruling DK's PW-2, so the two should not run as concurrent lanes.

## 7. Owner questions

Each has a default; the mockup draws the default.

- **PQ1.** Make the right dock the Context panel (Inspect · Note · Ask, following the selection, with a pin), on
  the desktop **and on the tablet**? The tablet half changes `MOBILE_SHELL_PROPOSAL.md`'s single-inspector right
  dock. *Default: yes, both.*
- **PQ2.** Approve this as CV-24's design: the strip in every domain, with CIVIL ▸ Timeline's recorded years, + Record,
  filters and speeds in its expanded row. The owner ruled on 2026-09-03 that it waits for a design. *Default: yes.*
- **PQ3.** Make the five context windows non-exclusive and following-until-pinned, with the Data manager, the Asset
  library, New world, Export, Open project and file dialogs staying modal? *Default: yes.*
- **PQ4.** Open the planner beside the map on the desktop, instead of replacing it? Ruling DK's pop-out stays.
  *Default: yes.*
- **PQ5.** Is the assistant one persistent conversation per session, cleared only by **New**, rather than one per
  opening? *Default: one per session, not saved with the world.*
- **PQ6.** Rename the **World** menu? Ruling CK named it, when Data became World. The audit found it collides with
  the WORLD domain, which means terrain. *Default: keep "World". The question is asked by name because it reopens a
  ruled value.*
- **PQ7.** The collapse/recovery simulator exists twice, with separate state: the Place editor's Timeline tab and
  CIVIL ▸ Timeline's Simulate (finding D2). *Default: keep both entry points over one shared state; do not delete
  the editor's tab without an inventory.*
- **PQ8.** Take the new keys in §3.9? *Default: yes.*
- **PQ9.** Should Inspect's edit mode cover only name, class, faction and fortified, with everything else staying in
  the Place editor window, which follows the selection? *Default: yes.*

## 8. Limits of this proposal

- **The mockup's world is a stand-in.** The ownership periods are invented, but shaped like the engine's snapshots:
  Korrath's places belonged to Orentamwash until the split of 210, and a third of the League's were taken in the
  Sevjuni campaign. So are the three conflicts and the relations table.
- **The assistant is scripted; no model is called.** The "used" line under each answer illustrates the kind of call
  the shipped assistant makes; it is not the shipped tool list.
- **Windows show their main sections**, not their whole contents. The Place editor's six tabs are named in §3.3, not
  drawn.
- **Measured in Chromium (Playwright)** at 1600 × 900, at 1194 × 834 for the tablet's 44 dp targets, and at the
  phone sizes (`MOBILE_SHELL_PROPOSAL.md`). No page errors. Not run in Godot.
- **Mixed sources.** The audit's findings marked *(audit)* were read by an agent and not re-run here. Every count in
  §1 without that mark was re-run in this session with the command shown.
