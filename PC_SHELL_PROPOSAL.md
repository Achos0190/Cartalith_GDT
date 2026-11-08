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

**Every owner question is answered:** PQ1–PQ9 by Ruling DY and PQ10–PQ13 by Ruling DX, both of 2026-11-01
(`LARGE_ITEM_RULINGS.md`). The design below carries the answers; §7 keeps the questions as they were asked. Four
answers depart from the default:
- **PQ1:** the Context panel is modular, with tabs the user arranges (§3.3).
- **PQ4:** the planner opens in its own window (§3.7).
- **PQ5:** the conversation can be saved inside the world file (§3.5).
- **PQ6:** the World menu is renamed Records, and Generate world moves to File (§3.8).

**Ruling DZ** came later the same day, while the owner reviewed the mockup. The tool-options bar holds the armed tool's
settings only, so Generate world, New seed, Center landmasses and Bake ALL & finalize leave it (§3.12).

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
  CIVIL ▸ Timeline's controls. This is that design, and the owner approved it (Ruling DY, PQ2):
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

**Tabs.** Inspect, Note and Ask are the default tabs, and Ctrl+1, Ctrl+2 and Ctrl+3 switch between them. The
user can add ten more modules and arrange them (**Modules**, below).

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

**Modules: tabs you arrange (Ruling DY, PQ1).** The owner's answer was *"1, but it should be modular. Make
suggestions for modules."* It came with "Tabs you arrange" and ten of the eleven modules offered. The panel is a
host, and each module is a tab.

**Arranging the tabs:**
- **Adding.** `+` at the end of the strip lists the modules. Each one is checked when it is already in the panel.
- **Moving.** Dragging a tab reorders it. A tab's menu (right-click, or long-press on touch) also has **Move
  left** and **Move right**.
- **A window.** The tab's menu has **Open in a window**, which shows that module in a §3.6 window. The window
  follows the selection until pinned.
- **Removing.** The tab's menu has **Remove** on every tab but Inspect.
- **Fit.** A module that does not fit the subject's kind is hidden, not removed. It returns when a subject it fits
  is shown.
- **Where it is kept.** The arrangement is saved per user with the settings, as the quick bar's `[quick_bar]` is
  (`CUSTOM_QUICK_BAR_SCOPE.md`). It is never in the world file.
- **Bookmarks are the exception: they belong to the world.** The owner's answer was "In the world file" (Ruling DY).
  They are saved inside the world file, as a new optional slot in the project `.zip`. The recent list is
  session-only.
- **Keys.** Ctrl+1, Ctrl+2 and Ctrl+3 stay on Inspect, Note and Ask, wherever those tabs sit, and re-add one that
  was removed. Added modules have no default key.
- **Menus.** Window ▸ Context panel lists the ten modules as check rows, followed by **Reset panel**, which returns
  the panel to Inspect · Note · Ask. That gives every module a menu path, so a key can be bound to it (§3.8).

**Judgements in the ruling, open to the owner:**
- Inspect cannot be removed, because it anchors the panel.
- Open in a window is on the desktop only (§3.11).
- Ctrl+1, Ctrl+2 and Ctrl+3 are fixed.
- The menu rows are not on the phone.

**One builder, two hosts.** Each module calls the builder that already ships for its window. A module is never a
second implementation. Moving a builder into a module starts from that window's own builder (`MISTAKES.md`,
re-framing a window).

| Module | Shown for | Reuses (read at the symbols, 2026-11-01) | Shows |
|---|---|---|---|
| History | settlement, faction, landmark | `place_editor_window.gd::_build_timeline` and `::_build_political` (holders by period, the population trajectory, authored chronos events, journeys passing); `faction_roster_window.gd::_build_tab_history`; `right_dock.gd` `CTX_HISTORY` | holders by period and dated events, chronos from the note; every year a link that moves the strip |
| Economy | settlement, faction | `place_editor_window.gd::_build_economy` (Economy & notables); `faction_roster_window.gd`'s Economy tab (`_build_economy_readout`, `_build_currency`, `_build_tariffs`, `_build_gaps`) | production, trade and notables; a faction's readout, currency and tariffs |
| Military | faction, settlement | `faction_roster_window.gd::_build_military_block`, with `_build_military_figures`, `_build_garrison_block` and `_build_conflicts_block` | armies, garrisons and conflicts; a place's fortified flag and garrison |
| Faith, culture & relations | faction, culture | `faction_roster_window.gd::_build_tab_relations` (with `_build_faith_group`) and `::_build_culture_card` | relations with every faction, the faith group and the culture card |
| Territory | faction | `faction_roster_window.gd::_build_tab_territory` (claims, provinces, influence, terrain fit) | what the faction claims and holds |
| City layout | settlement | `place_editor_window.gd::_build_layout`; `city_viewer_window.gd` | the settlement's layout, and City viewer… |
| Here | settlement, landmark; the cursor when nothing is selected | `right_dock.gd` `CTX_SAMPLE` (`SAMPLE_FIELDS`, `on_cursor_sampled`) | elevation, climate, biome and the other sample rows |
| Routes & journeys | settlement, landmark | `right_dock.gd` `CTX_ROUTE` (`show_route`); `journey_planner_view.gd`; `saved_journeys_window.gd` | roads from the subject, saved journeys through it, and Plan a journey from here |
| Links | any subject with a note | the vault crate's link index (`notelinks.rs`: backlinks and unlinked mentions) | notes that link to the subject, and unlinked mentions with a **Link** button |
| Recent & bookmarks | always | the selection history (§3.1); bookmarks are new | the history as a list (session-only), and subjects the user bookmarks (saved in the world file) |

**Compare** was offered and not chosen. Flow F8's pin and banner remain the way to compare two subjects.

### 3.4 The Note tab

- **What it shows.** The tab shows the subject's own note, found through its entity binding, so renaming a place
  does not lose its note.
- **Write is the default; Preview is the toggle** (Ctrl+E). Ruling DY's VQ4 made the vault open notes in Write,
  and this tab follows it.
  - Write edits the Markdown in place, with the source coloured, autosave, and Ctrl+B, Ctrl+I, Ctrl+K and Ctrl+E.
  - The editor is `VAULT_EDITOR_PROPOSAL.md`'s, in a narrower frame. Applying VQ4 here as well is a labelled
    judgement in the ruling.
- **Undo and revert are the vault's** (VQ1, VQ6). The session undo goes back to the note as it was first opened
  this session. **Revert to opened…** asks first.
- **Links.** A `[[link]]` selects what it names, so the map, Inspect and Ask follow.
  - In Write, a link is followed with Ctrl-click, or on touch with the `Open ›` chip.
  - A link to a missing note offers to create it, with a choice of template, never silently (VQ2).
- **Chronos.** A chronos row moves the year, and the row the year falls in is lit.
- **No note yet.** The tab offers **Create *name*.md** from the kind's template, already linked to the subject.
- **The full vault window.** ⧉ opens the note in the vault window for split view, the outline, backlinks and
  rename. The vault window is `Records ▸ Markdown vault…` (Ctrl+Shift+V), since Ruling DY renamed the menu.
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

**The conversation.** One conversation per session persists across selections. It is cleared only by **New**, so
switching places never loses it (A1).

**Saving it (Ruling DY, PQ5):**
- **The preference.** Preferences ▸ Assistant gains **Save the assistant conversation with the world**. It is off by
  default.
- **The first-use card.** The first Send while that preference is unanswered shows a card in the tab: *"Save this
  conversation with the world? It is stored inside the world file and travels with it when you share it."* Its
  buttons are **Save with the world** and **Not now**. The answer sets the preference, and the card is not shown
  again.
- **When it is on:**
  - Saving the world writes the conversation into the world file, as a new optional slot in the project `.zip`.
  - Opening the world restores it.
  - The tab's header says *saved with the world*.
- **What it must never do:**
  - write the key, which is never in a save, a log or an export (`LLM_ASSISTANT_SCOPE.md`);
  - save anything while the preference is off;
  - change what Ask is offered: private notes are still never offered (Ruling DP).
- **The save format.** `SAVEFILE_COMPAT.md` gains the slot when it is built, not before.

**Answers link back** (A3):
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
- **Opening on the right subject.** Records ▸ Factions…, Story ▸ Faction history… and CIVIL ▸ Open factions… open on
  the current faction, not the last one used (finding W6). The roster's "Capital: X" is a link that selects it
  (flow F3).
- **Window ▸ Open windows** lists every open window, the Context panel and pop-outs included (finding W5). While an exclusive dialog is open the menu bar takes no input; since PC-7 the five context windows are not exclusive on the desktop, so the list shows them (picking one raises it where the user left it).
- **Refresh.** Roster edits refresh the Context panel, and an assistant save refreshes the vault (finding W7).

**What stays modal.** The Data manager, the Asset library, New world, Export, Open project and file dialogs stay
modal. Ruling DK Q4 already keeps the first two and the file dialogs out of pop-out. A modal window there
protects a whole-world operation.

**Pop-out is unchanged** (Ruling DK: off by default, a global setting and a per-window button, roster first). The
two features are orthogonal: pop-out decides **where** a window draws, and this decides whether it **blocks**.
`PopoutHost.prepare` already clears `exclusive` for a native window. This makes the docked default match.

### 3.7 The planner in its own window

The owner's words (Ruling DY, PQ4): *"Planner should open in a separate window that doesn't block the map (that way a
user could put it on a second screen for example)"*. This replaces the section's first design, a wide workspace in
the left dock.

- **On the desktop, the planner opens in its own window.** It is one of §3.6's non-blocking windows, so the map stays
  visible, stays live and takes clicks (finding W4).
- **It has to become a window first.** Today `journey_planner_view.gd::_show` hides the map, and
  `POPOUT_WINDOWS_SCOPE.md` found that the planner is not a window at all. Re-hosting it in one is PC-8's work.
- **On a second screen.** Ruling DK's per-window **Pop out** button makes the window native, and a native window can
  be moved to another screen. DK Q5 already says *"keep the main map live while the planner is native."*
  - An embedded window cannot leave the main window, so this needs PW's planner step.
  - That step is PW-4. It is next in DK's order, because the roster and the other civilisation windows already pop
    out (PW-2, PW-3).
  - PC-8 and PW-4 re-host the same view, so they are best done as one change.
- **Its bar row moves into the window** (Ruling DZ, §3.12). Today `journey_planner_view.gd::_tool_options_journey()`
  puts the planner's journey picker, a carriage readout and three actions on the tool-options bar: re-route…, save
  journey and export table. They become the window's own toolbar.
- **The tablet and the phone** keep the planner as `MOBILE_SHELL_PROPOSAL.md` draws it.
- **The selection feeds the stops.** With a settlement selected, the planner offers **Start here**, **Add as stop**
  and **End here**, each one undo step (flow F2). The context card's "Plan journey from here" and "Journey to here"
  stay. The route's name is the same everywhere: the card and the planner currently disagree, "Journey 1" against
  "Route #0" (finding M2).

### 3.8 Menus

Each change names the audit finding it answers. The PC bar keeps its eight menus and their order. The phone's MENU
and the tablet's bar mirror these rows, as `MOBILE_SHELL_PROPOSAL.md` requires.

| Menu | Change | Why |
|---|---|---|
| File | **Generate world, Ctrl+R**, right after New world… (Ruling DY, PQ6). The first draft had it as World's first row. The tablet's Run pipeline row is the same command, under one label (Ruling DX, PQ11) | M1: no PC row or key today |
| Records | Factions… gains **Ctrl+Shift+F** and opens on the current faction; Markdown vault… gains **Ctrl+Shift+V** | W6; M8 (no keys for either) |
| Records | The second Timeline row becomes the signpost *"Timeline is in Story ▸ Timeline"* | M3: two rows, one action |
| Records (name) | **World is renamed Records** (Ruling DY, PQ6), reopening Ruling CK Q3. `command_index.gd` `EXTRAS` gains the pointer row **"Records (was World)"**, so a search for the old name finds the menu. An alias titled plain "World" would be hidden by Window ▸ Workspace ▸ World, under the alias skip that draft 07 adds (`_has_menu_title()`) | M4: the menu holds records while the WORLD domain is terrain |
| Tablet bar, touch, under 1 180 dp | **Four menus: File · World · Records · Story** (Ruling DY, PQ6), reopening Ruling CK Q2. World keeps only the WORLD domain's rows: Generate world, the two resets, Pipeline / Sculpt and Seed & size…. Records is the desktop's menu. On this bar File leaves out Generate world, because World holds it (a judgement). The bar's width at 600 dp is a build bar | CK Q2 merged Data's rows into the tablet's World menu, below its pipeline rows |
| Edit | Deselect clears the one selection. **Previous selection (Alt+←)** and **Next selection (Alt+→)** are new | S1; §3.1 |
| Story | "Political history of this place…" becomes **"…of the selection"** (the Inspect page). Faction history… opens on the current faction | S2, W6 |
| Story | "Notes for this place" becomes **"Note for the selection"** (Ctrl+Shift+N, the Note tab), for any kind | S2 |
| Story | "Assistant panel…" becomes **"Ask the assistant about the selection" (Ctrl+J)** | A1, A4 |
| Story | Its "Places without notes…" row becomes a signpost to Records ▸ Places without notes…, where the same id stays | M3 |
| Window | "Right dock" becomes **"Context panel"**, as two rows: the check that shows or hides it, then a submenu of the same name (Inspect, Note and Ask on Ctrl+1, Ctrl+2 and Ctrl+3; Pin to the current subject; Follow the selection; then a MODULES band of ten check rows and **Reset panel**, per Ruling DY, PQ1, and not on the phone). A Godot `PopupMenu` row cannot be both a check and a submenu | §3.3 |
| Edit | **Context card (Shift+F10)** and **Cartwheel at the pointer (Q)**, after Find on map…, so both keys can be rebound and are listed (PQ10) | M8 |
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

*(Built 2026-11-08 for four of the five (every path but Landmark types): each third level is a captioned band of its parent popup with the same ids; Assets ▸ Landmark types ▸ family ▸ type became one `Landmark types…` row that opens a list window, by Ruling EH item 2 (2026-11-08), because its type rows are engine-sized state rows. `STATUS.md` has the measurements.)*

**Desktop menus: what is new and what Godot already does.** Godot's submenus already open beside their row, so the
whole path stays visible, as the mockup's columns draw it. Rows are one line, with the accelerator on the right. What
is missing is that **moving across the bar while a menu is open switches menus**: draft 10, `switch_on_hover`.

**Rows that work today keep their shipped form until their milestone.** For example, "Notes for this place", "Assistant
panel…" and "Political history of this place…" keep their current titles and behaviour until the Context panel exists.
Each keeps its id, so a rebinding or a quick-bar pin follows the row to its new title. Rows with nothing behind them yet
are `_todo`, with the reason. Building them as `_todo` instead would remove a working command (`MISTAKES.md`, "Delete
a surface").

**The GDScript drafts** are in `cartalith-native/drafts/pc-shell/`. Its README gives:
- the order, and the drafts that must land together;
- which form each row takes before and after its milestone;
- the collision check, and the `EXTRAS` and `UNLISTED` rows;
- the probes to update;
- what was not drafted, and why;
- where the code and this section disagreed. Those are corrected here.

**Desktop search runs commands.** The phone already runs a pinnable row through `DccQuickBar.run_pin_on`; the desktop
draws the same rows disabled (finding S4). They become runnable on the desktop too. *(Built 2026-11-08:
`dcc_shell.gd::_run_search_command()` is shared; the reason a row cannot run is the dialog's count line.)*

### 3.9 Keys

The owner took all of them (Ruling DY, PQ8).

- **New:**
  - **Ctrl+R** generate (the PC gains the tablet's key);
  - **Ctrl+1**, **Ctrl+2** and **Ctrl+3** for Inspect, Note and Ask;
  - **Ctrl+J** to ask;
  - **Ctrl+Shift+F** for Factions and **Ctrl+Shift+V** for the vault;
  - **Alt+←** and **Alt+→** for selection history.
- **Free.** None of them is in the shipped list (§1). Alt+← and Alt+→ are not used by the shell; on a platform where
  the window manager takes them, Edit ▸ Previous and Next selection remain.
- **Rebindable.**
  - The only rebindable table is the menu accelerators (`menus.gd::_bind_accelerator`). So Q (the cartwheel at the
    pointer) and Shift+F10 (the card) become Edit rows (PQ10) and leave `app.gd::_unhandled_key_input`.
  - The Menu key stays in `app.gd`, because a menu row holds one accelerator; it is listed in the reference through
    `shortcuts_dialog.gd::UNLISTED` (finding M8).
  - **On the phone no accelerator fires**, because its menu bar is a hidden model (`PhoneMenuModel`). So a phone with
    a hardware keyboard loses Q and Shift+F10, and keeps the Menu key and the long-press card.
- **Two scoped exceptions to "free":**
  - Ctrl+1, Ctrl+2 and Ctrl+3 pick a layer while the Layers popover is open, because its key match ignores modifiers.
  - On Windows and Linux, Alt+← and Alt+→ reach the menu while a text field has focus (PQ12).
- **Esc** closes the topmost transient: a menu, the card, the ring, Find, the topmost window, the edit mode. With
  nothing open, it deselects. *(Built 2026-11-08 in `app.gd::_unhandled_key_input`: Escape reaches the selection only
  when Inspect was already armed with no escape handler and no text field has focus; menus, the ring and windows take
  it first because they consume it before the app's unhandled-key path. The phone's back does not deselect.)*

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

  Its right dock changes from `MOBILE_SHELL_PROPOSAL.md`'s single inspector to the Context panel and its tabs. That
  was PQ1's second half, and the owner took both halves (Ruling DY). Its windows stay modal sheets, because a
  touch user cannot keep a floating window and the map both in reach. So a tab's **Open in a window** is not
  offered there.
- **The phone** gets the model, not the panel: one selection, one year, the chronos ticks and the menu rows. Its
  sheet keeps showing the selection. On the phone, Note opens the vault at the note and Ask opens the assistant's
  sheet.

### 3.12 The tool-options bar holds tool settings only (Ruling DZ)

The owner, verbatim: *"The buttons in the top rail new seed, generate and bake can be removed. Please see to only keep
the paint/sculpt brush settings here."* Asked about the rest, the owner chose "Tool settings only".

**What the bar holds:**
- **The armed tool's settings, and nothing else.** For sculpt and paint, that is the brush's mode, size and
  strength. Every other tool keeps its own settings: Measure, Way, Route, Territory, Settlement, Icon, Label and
  Conflict.
- **A tool's Commit and Discard stay.** They finish that tool's draft, and `DCC_SHELL_SPEC.md` §4 lays the row out
  as "context label → parameters → spacer → commit/discard". Keeping them is a labelled judgement in the ruling.
- **With no tool armed, the context name and one hint.** For example, *GENERATE · WORLD · Click selects ·
  right-click opens the card · Q opens the cartwheel*. A touch tablet's hint is *Tap selects · hold opens the
  cartwheel and the card*.
- **The quick bar's pins** at the bar's right end stay.

**What leaves it, and where each item stays.** Today `app.gd::_tool_options_generate()` builds WORLD's idle row.
Per `MISTAKES.md`, "Delete a surface", each item has a home:

| Leaves the bar | Stays in |
|---|---|
| Generate world | the WORLD dock; File ▸ Generate world (Ctrl+R); on the narrow touch tablet, World ▸ Generate world |
| New seed | the WORLD dock's RUN section (`app._new_seed()`) |
| Center landmasses | the WORLD dock's RUN section (`app._center_landmasses()`) |
| the "generating…" readout | the busy overlay and the status bar's `pass` cell, which the same `generation_started` handler already writes |
| Bake ALL & finalize | the WORLD dock's Finalize foot. The bar's copy only pressed that button, and needed `set_bake_shortcut` to stay in step |
| the planner's row (journey picker, carriage, re-route…, save journey, export table) | the planner's window (§3.7, PC-8) |

**Where the work is drafted:** `cartalith-native/drafts/pc-shell/11_tool_options_bar.gd`. It holds the replacement
function, the deletions in `app.gd` and `world_workspace.gd`, and the two probes that change (`_structmove_probe.gd`
and `_vfy_batch0905_probe.gd`). The menu parity check is unaffected, because no menu changes.

**The phone is unchanged.** Its tool strip appears only while a tool is armed, and then holds that tool's settings.

## 4. The flows, before and after

| Flow | Today | Proposed (as the mockup does it) |
|---|---|---|
| F1 Write a place's history and see it on the timeline | Editor → Vault tab → attach (stacks) → Add event (year 0) → seen in the editor only | Select → Ctrl+2 → Write the chronos line → its tick appears on the strip and the row lights at its year |
| F2 Journey between two selected places | Logistics → empty planner, map hidden → leave → card on A → card on B | Select A → *Plan a journey from here* → the planner opens in its own window, the map still live → select B → **End here** |
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
| Right dock | `right_dock.gd` (`RightDock`) | `ContextPanel`: a header, the tabs, `RightDock`'s builders as Inspect, `markdown_editor.gd` as Note, `llm_assistant_window.gd`'s builders as Ask. A module registry (id, title, the kinds it fits, the builder its window also calls, an empty state) and the user's arrangement in the user's settings (§3.3); bookmarks in a new optional slot of the world file |
| Tool-options bar | `app.gd::_tool_options_generate()` puts Generate world, New seed, Center landmasses, a busy readout and Bake ALL & finalize on WORLD's idle row; `journey_planner_view.gd::_tool_options_journey()` puts the planner's actions there | The armed tool's settings only; WORLD's idle row is `_tool_options_simple()` with the context name and one hint (§3.12, draft 11); the planner's row moves into its window |
| Assistant | `LlmAssistantWindow` (an `AcceptDialog`); `open()` → `_rebuild()` | Its content inside the Ask tab, built once per session; `_current_place` and `_args` read `DccApp.selection` and the year at Send; `note_saved` connected to the vault; with the preference on, the conversation in a new optional slot of the world file (§3.5) |
| Windows | 23 `AcceptDialog`s, `exclusive` | The five context windows: `exclusive = false`, one host with a back stack, a follow/pin state; `busy_overlay.gd` mutes them as it mutes pop-outs |
| Planner | `journey_planner_view.gd::_show` hides `app.viewport` | Its own non-exclusive window, one of §3.6's hosts, with Ruling DK's per-window Pop out; reads the selection. It must be an `AcceptDialog`: `PopoutHost.add_toggle_button(dlg: AcceptDialog, …)` takes only one (shipped), and Window ▸ Open windows, as draft 05 rebuilds it (`_visible_windows()`), walks only those. Found by the drafts |
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
| PC-10 | The modules (Ruling DY, PQ1) | Each of the ten modules shows for the kinds in §3.3's table and hides for the others. Each calls the same builder function its window calls: one definition, two call sites (`grep`). Add, drag, Move, Remove and Open in a window work. The arrangement survives a restart and is absent from the world file (the saved `.zip` is listed). A bookmark survives a save and reopen, inside the world file; the recent list does not. Ctrl+1/2/3 reach Inspect, Note and Ask wherever they sit |
| PC-11 | The tool-options bar holds tool settings only (Ruling DZ) | `_tool_options_generate()` builds no `DccWidgets.action()`. A windowed probe arms Inspect in WORLD and finds the context label, the hint and the quick bar's pins, and no Button of the bar's own. The WORLD dock's Generate world, New seed, Center landmasses and Bake ALL levels & finalize each still act. `grep -rn set_bake_shortcut shell/` and `grep -rn bake_and_finalize shell/` both → 0. Drafted: `cartalith-native/drafts/pc-shell/11_tool_options_bar.gd` |
| PC-5 | The Note tab | A note opens in Write (VQ4); a `[[link]]` selects, by Ctrl-click in Write; a chronos row moves the year; Write autosaves; Revert to opened… asks first and is one undo step; a missing note is offered with a template choice, then created and bound to the subject |
| PC-6 | The Ask tab | Opening, closing or reselecting never empties the transcript. The context at Send includes the year. A citation selects. A note card's Apply goes to the linked note in one undo step. The hostile-string corpus (scope §4 bar 3) re-runs green, because the request changed. The save preference is off by default and asked once at the first Send. With it on, a saved world reopens with the conversation, and the saved slot never holds the key (`grep` of the `.zip`); with it off, the slot is absent |
| PC-7 | Non-exclusive context windows | A windowed probe clicks the map while the roster is open, and the map takes the click. The roster follows a new selection until pinned. The busy overlay mutes it during a generate |
| PC-8 | The planner in its own window (Ruling DY, PQ4) | The planner is a window: `_show` no longer hides `app.viewport`. With it open, the map draws and takes input. Start / Add / End from the selection each add one undo step. Its Pop out button is present; reaching a second screen is PW's planner step |
| PC-9 | Menus and keys (drafted: `cartalith-native/drafts/pc-shell/`) | Every row in §3.8. `command_index.gd` finds each moved title by TITLE. `shortcuts_dialog.gd` lists every new key. No accelerator collides (`grep` of `menus.gd` and `app.gd`). The World menu reads Records, and Generate world sits in File after New world…. On a touch tablet 600 dp wide, File · World · Records · Story fit without overlap (measured). Fully met only after PC-1, PC-4, PC-5 and PC-6, whose behaviour four rows need; until then those rows are `_todo` or keep their shipped form |

**Build notes (2026-11-07; how far each milestone has got is `STATUS.md`'s answer).** PC-9 and PC-11 were built from
the drafts unchanged, with two departures found while building: the bar's Center landmasses button was the only home of
the pass's full disclosure, so the WORLD dock's button now reads `DccApp.CENTRE_TOOLTIP` (an item §3.12's inventory
missed); and the menu parity extractor had to learn that a draft's EDIT can already be in the tree. The PC-9 probe is
`_pc9menus_probe`; PC-1's is `_pc1select_probe`. PC-1 applies settlement selections to the surfaces; the other kinds are
recorded and wait for PC-4's Inspect builders. MS-1 shipped ids 930-933 first, so Reset panel is 934.

PC-2 (2026-11-07) was built as specified: `tl_set_year` is the one writer, with an `exact` argument the bar did not
name, because the CIVIL dock records years in -9999..9999 and the Place editor accepts +-100 000, and the strip's
clamp to -400..1200 would have loaded the wrong year's snapshot for them (PC-3's range from the world retires the
clamp's constant). The readers T2 lists that were missing were the right dock, the Place editor, CIVIL's pills and
the territory texture; the assistant's context carries no year until PC-6. The probe is `_pc2year_probe`.

PC-4 (2026-11-08) is the right dock itself, not a new node: `right_dock.gd` (`RightDock`) draws the header (◀ ▶ over
the selection history, the subject's name and one line of facts, FOLLOWING or PINNED) and a tab strip holding INSPECT
alone at the top of its body, and its existing context builders are Inspect. Note and Ask are absent, not drawn
disabled: their tabs arrive with PC-5 and PC-6. Departures and judgements found while building: the dock is told the
whole selection (`RightDock.on_selection()`), so faction, conflict and landmark selections are shown, and the map's
landmark pick and CIVIL's faction and conflict rows became `select()` producers; a context that is not a selection (a
route, a river, a measurement, a region, an ecoregion, the history ledger, an icon) releases a pin, because a pinned
panel would otherwise hide a tool's reading; Reset panel shows the panel on Inspect, since the arrangement has nothing
to reset until PC-10 (`context_module_registry()` stays empty, and the per-user `[context_panel]` store lands with the
first module); the settlement page gained Political history (periods, holder and year links), Conflicts here and the
three nearest neighbours, with a link to the Place editor's full tab so Story ▸ Political history of the selection
loses nothing the shipped row opened; the faction page gained its capital, its settlements (twelve, largest first) and
its conflicts; the conflict page gained Years and sides. The §3.3 page items not built are the AT YEAR block's "when
it is founded", roads, **City viewer…** and **Plan a journey from here** as named actions (the dock's City layout link
and Logistics already open both), edit mode (PQ9's quick fields), and moving the year when a conflict is picked
(finding F7). The phone's sheet is unchanged (§3.11). The probe is `_pc4context_probe`.

**Cultures are selectable** (2026-11-08, the owner's "Culture selection is unreachable from the GUI"). A culture's
selection id is its index in `bridge.get_cultures()` (the handle `context_modules.gd::build_faith` already read), and
`DccApp.culture_index(key)` gives it from a faction's `culture` key: -1 for an empty or unknown key, so unowned land has
no culture and gets no row. Producers, all through `DccApp.select("culture", i)`: a Find hit (a new `culture` entity in
`place_search.gd`, scope prefix `c`; its row carries no `x`/`y`, and `_select_search_hit()` pans only a row that has
them), a "Culture" link on the settlement page (beside Faction) and on the faction page, a "Select culture" row at the top
of the Faith module's Culture section, and a "Select culture" row on the settlement's map card (CIVIL, INFO section).
Inspect's culture page (`RightDock._build_culture()`, `CTX_CULTURE`) is the Faith module's own builder through the
registry, so `_build_culture_card` stays the one implementation, plus the factions that hold the culture as links. The
header, history, pin, bookmarks, the Links module and the assistant's chip read it through the generic paths. The Note
tab still says a culture has no binding (a vault note binds to a place or a faction). The phone's sheet is unchanged
(the new rows are desktop and tablet only). The probe is `_culturesel_probe`.

PC-3 (2026-11-08) was built with five departures from the text above, each found at the symbol. (1) **The range's
sources are the recorded years and the conflicts' start and end years**, not "the earliest founding, chronos entry or
recorded year": the engine records no founding year (the settlement dictionary has no such field), and the chronos
entries of the selection are ticks, not a range source, so selecting a place never moves the axis. (2) **A world with
no recorded year and no conflict has no range**: `tl_range()` answers the old -400..1200 axis with `from_world` false,
the end literals read `YEAR -400 (default)`, and the strip stays scrubbable. `TL_YEAR_MIN`/`MAX` survive only as that
fallback. A one-year world is widened to `TL_MIN_SPAN` (100 years) around it; the cursor is not folded in, so a cursor
outside the range pins the playhead to the nearest end. (3) **The collapsed strip keeps its shipped composition**
(TIMELINE, the year, the state word, `expand`): CV-24's collapsed line with a play button, a slider and ticks is not
built, and the ticks live on the expanded strip's track. (4) **+ Record year records the shown year** (scrub, then
record) through the dock's own `civ_add_year`; the engine keeps no undo step for a recorded year, here or in the dock,
so the proposal's "one undo step" is open. (5) ~~**A conflict row in the event list moves the year but does not select
the conflict**: a conflict is a selection kind only PC-4's Inspect shows.~~ Built 2026-11-08, after PC-4: the row moves
the year, then selects the conflict (`_confevent_probe`). The strip's pills show the 12 recorded years
nearest the cursor. The phone keeps its CIVIL-only strip. The probe is `_pc3strip_probe`; the phone guard is
`_pc3phone_probe`.

PC-6 (2026-11-08) docks the shipped assistant, not a rebuilt one: `LlmAssistantWindow` keeps every rule and owns the
content, built once per session, and `present_in()` moves it into the Context panel's Ask tab (`RightDock.show_tab()`,
a frame beside Inspect's scroll, so Inspect stays built underneath). `open()` routes every entry point to
`DccApp.open_context_tab("ask")` on the desktop and tablet; the phone keeps its sheet (§3.11). Departures and
judgements found while building: (1) **the selection and the year ride as a `shown` block in their own fence**
(`cartalith_llm::prompt::shown_block`, on the prose and Commands paths, never on Edit), gated by the existing,
previously unread "Send: selection" source (`ctx_selection`, off by default, now labelled "Send: selection and year"),
which Rust re-checks; the proposal's chips sit on that source rather than replacing it, so the scope's opt-in per
source holds. (2) **Two chips, not three**: the note chip is not wired (PC-5 built the Note tab; the chip is still PC-6's to add). (3) **New** clears the
conversation; on the phone it joins the status row so the sheet gains no row. (4) **Open in a window** (desktop) is the
dialog form, where Ruling DK's Pop out still applies; the dialog is still exclusive (PC-7), so the tab says "close that
window to bring it back" instead of offering a button a click could not reach. (5) A reading that is not a selection
(a route, a river, a measurement, a region, an ecoregion, the history ledger, an icon) switches the panel to Inspect;
a selection does not. (6) A verified world citation selects what it names (settlements by `tid`, factions, landmarks);
Vault citations stay text (PC-5 built the tab and did not wire them). Not built, stated: the save preference, its first-use card and the world-file
slot (PQ5); a note card going to the subject's linked note; names and years in an answer as links; per-subject task
chips; "Ask about this" on Inspect pages and the context card. The probe is `_pc6ask_probe`.

PC-7 (2026-11-08) converts the five windows §3.6 lists, all in one change, on the desktop only: one shared file,
`shell/context_window.gd` (`ContextWindow`, preloaded, no `class_name`), and two calls per window. Departures and
judgements found while building: (1) **the tablet keeps its modal windows**, as §3.11 says, so `ContextWindow.applies()`
is false on the tablet and the phone and neither changes. (2) **Measured on Godot 4.7.2, an embedded window that has the
focus takes every mouse event and every key**: only a left press outside it takes the focus away. So with a window
focused, a wheel notch, a right press and a middle drag over the map went to the window, not the map, and no menu
accelerator worked (Ctrl+F and Q measured; the menu bar never saw the key). Two relays close both holes: a press other than a left one outside the window's frame
takes its focus and is pushed into the main viewport (`ContextWindow._on_window_input`), and an unused Ctrl/Cmd or Alt
chord is run on the menu bar as its accelerator would (`shell/context_key_relay.gd`, through
`PopupMenu.activate_item_by_event`; a field's own chords and every plain key stay with the window, so a key typed into
a field never reaches the map's accelerators). Moving the pointer off a window does not take its focus, so hovering the
map with a window focused samples nothing until a press. (3) **Held changes pin instead of following.** A window
answers `held_changes()` with what it holds that has no commit path (the roster's open religion editor, the Place
editor's unsubmitted event); a new selection then pins it, with the reason on the chip and the status line, and
pressing PINNED asks before discarding. A field being typed in is committed against the subject it was typed for, the
path a re-open already took, and a year move waits for it to give up the focus before redrawing. (4) **The chip reads FOLLOW / PINNED** (the mockup's window header; the Context panel keeps FOLLOWING), and sits at
the right end of the button row, in the Pop out button's balance slot, so Close does not move (measured pixel-identical
outside the chip, `_popout_windows_id_probe` and `_popout_roster_id_probe` with `--chip-only`); it lives in a
non-Button holder because `AcceptDialog` resets the minimum width of every Button in that row, which is also why Pop
out lays out at 81 px and not the 84 its builder asks. (5) **Saved journeys
has no chip**: it lists every journey and nothing in it depends on the year; a row closed it while the planner replaced the map; since PC-8 it stays open (see PC-8's note). (6) Conflicts stays open when a row is pressed, and marks the selected conflict.
(7) The roster, the City viewer and Conflicts now redraw on a year move (one redraw per frame); the City viewer keeps the
user's zoom. Every open follows again. Not built, stated: §3.6's one host with a back stack (finding W2), ~~the roster's
"Capital: X" as a link (flow F3)~~ (built 2026-11-08: the Territory tab's "Capital: X ›" row selects the capital; the
note above it keeps its text, and the link closes the window only where it is modal), the refresh items (finding W7), and Find drawing above an open window (Ctrl+F works
from a window; whether Find's results draw above a window that overlaps them was not measured). The assistant's window is not one of the five and stays
exclusive, so PC-6's "close that window to bring it back" still holds; the vault, the Data manager and the rest stay
modal. The probe is `_pc7windows_probe`.

PC-8 (2026-11-08), with PW-4 as one change: on the desktop (`DccApp.planner_in_window()`, true where PC-7's
`ContextWindow.applies()` is) the planner draws in its own window, `shell/planner_window.gd` (an `AcceptDialog`,
preloaded, no `class_name`; its `PopoutHost` key `planner_window` is in `LIVE_KEYS`, with its Pop out / Dock button and
`prepare()` before each open). `JourneyPlannerView` builds its panels as before and reparents them into the window:
the Journeys list and the party form in a left column, the route map, profile, stops, inspector and matrix in the
centre, the results `build_results()` drew in the right dock's appended Journey section in a right column, the day
band in the window's own row, and the bar row (Ruling DZ) as the window's toolbar. Nothing it computes changed: the
same request gives the same `jp_compute` answer (six routes on three seeds hash equal to the pre-PC-8 tree), and the
79 controls the planner draws are the same 79. Departures and judgements found while building: (1) **opening it no
longer switches to CIVIL or arms Journey** on the desktop, so the map keeps its domain and tool and a click selects;
the CIVIL rail's Journey planner node still lights itself, because its handler writes the mode before opening.
(2) **Start here / Add as stop / End here** sit in a row under the toolbar, each disabled with its reason;
End here commits the stops as a route through the Route tool's own three calls and opens the planner on it, and
refuses while the Route tool is armed (the engine keeps one route draft). Each press is one step of Edit ▸ Undo,
on a stop history of the planner's own that the app's undo routes to first when the other histories are at the
depth they had when the step was taken; undoing End here deletes its route only while it is still the last route
and no saved journey names it, and otherwise drops the step with the reason. (3) **The window's floor is its content's
minimum width** (`_fit_floor()`, measured 1 196 px at seed 4242, the party form alone 466 px), because
`phone_window()` turns `wrap_controls` off and a narrower window cut the results column off; the toolbar and stop
rows scroll sideways. At 1 600 px that floor is wider than the map, so the map is usable beside the window only once it
is dragged aside or popped out, which is the ruling's own use case; an owner may want a narrower layout.
(4) A reopen uses `show()`, not `popup()`, which moved an on-screen window 22 px. (5) Saved journeys stays open when
a row is pressed. (6) The right dock's collapsed Journey header readout ("2 d · 432 km") now heads the window's
title. Not built, stated: one route name everywhere (finding M2: the card's "Journey 1" against the picker's
"Route #0"); an undo of End here that also restores the old route's per-stage edits (they were cleared when End
opened the new route); the tablet and the phone are unchanged (§3.11), proved by pixel identity at 393 × 852. The
probe is `_pc8planner_probe`.

PC-10 (2026-11-08) builds the ten modules as tabs of the Context panel, on the desktop and the tablet. The registry
is `shell/context_modules.gd` (preloaded, no `class_name`); `DccApp.context_module_registry()` answers it, so the ten
MODULES rows of Window ▸ Context panel are live check rows from the first boot. **One builder, two hosts, as the bar
asks:** each window-backed module calls its window's own builder on a detached instance of the window's script whose
subject is the panel's (the instance is a hidden child of the module's box, freed with it; an edit through its
controls refreshes the module through a `module_refresh` hook both windows read first in their rebuild paths, and their
signals are forwarded to the real window's, so CIVIL's listeners hear the edit). Two window tab bodies were lifted into
named functions so the call is one call, not a copy: `place_editor_window.gd::_build_tab_political` and
`faction_roster_window.gd::_build_tab_economy`; every reused builder has exactly two call sites (the window and
`context_modules.gd`; `_build_culture_card` three, the Identity tab and the module's two kinds). The arrangement is
`[context_panel]` in `cartalith_settings.cfg` (`DccSettings.context_panel_read()` / `set_context_panel_tabs()`, the
`[quick_bar]` pattern), never in the world file. Departures and judgements found while building: (1) **a stored
arrangement this build cannot use falls back to Inspect · Note · Ask with a notice**, and the settings file is first
copied to `cartalith_settings.context_panel.bak.cfg`; the bad section stays as it is until the user arranges the
panel. A settings file that does not parse at all is copied to `cartalith_settings.unreadable.bak.cfg` before any
write can replace it (`DccSettings._ensure_loaded()`; that is a change for every setting, made here because a corrupt
file used to be overwritten silently by the next save). (2) **Keys a build does not know are kept where they were**:
Note's key, which the default holds before PC-5 builds the tab, stays second, and a newer build's module survives an
edit here. (3) **Ask can be removed** (Ctrl+3 puts it back, as §3.3 says); Inspect cannot. (4) **Open in a window is
on module tabs only**, not on Inspect or Ask (Ask has its own window); a module window is a PC-7 window
(`shell/context_module_window.gd`) without Ruling DK's Pop out button, because it is not one of `PopoutHost.LIVE_KEYS`.
(5) **The hold that opens a tab's menu is 500 ms**, the cartwheel's. (6) **Military for a settlement** shows the Place
editor's Walls override and the garrison at the shown year (`CivilizationWorkspace.garrison_row`), not "a fortified
flag": the settlement record has none. (7) **Routes' "roads from the subject" are the roads whose first or last point
lies within two cells of it** (`ROAD_END_CELLS`, a labelled judgement: `bridge.roads()` carries no endpoint
settlement). (8) **History for a landmark is a sentence**: the chronos index reads settlement and faction notes only.
(9) **Bookmarks are a new caller-owned slot, `annotations/bookmarks.json`** (`SAVEFILE_COMPAT.md`), written only when
there is one; a generate drops them, an open restores the project's. (10) A window builder laid out for its window's width could widen the dock: measured on the touch tablet, the City
layout page pushed the dock over most of the map, so a module page sits in its own sideways-scrolling box
(`CtxModuleScroll`). (11) ~~No producer selects a culture yet, so the culture branches of Faith and Links are reached
only by code.~~ Closed 2026-11-08 (see "Cultures are selectable" under PC-4 below). Not built, stated: Pop out for module
windows; a module's own key (Note, which this note once listed, is PC-5's). The probe is `_pc10modules_probe`.

PC-5 (2026-11-08) builds the Note tab on the desktop and the tablet: `shell/context_note.gd` (preloaded, no `class_name`)
and the vault window's own editor, hosted in a frame the tab owns (`RightDock._note_frame`, a sibling of Inspect's
scroll as Ask's is, because the editor holds the user's unsaved text and cannot live in a body every `_rebuild()`
frees). **One note focus, one writer, defined:** the focus is the pair (`VaultWindow._browse_path`, the editor built for
it); both exist once, and the editor is in exactly one of three places, the tab's frame (`VaultWindow._dock_host`), the
vault window's body, or nowhere. The tab never builds an editor: `VaultWindow.dock_note(rel, host)` does, through the
same `_build_editor_pane(…, docked)`, the same `_load_note_state()` read and the same `_write_now()` writer (hash guard,
autosave, banner, session undo, Revert to opened…, `flush_for_exit`). `dock_note` answers `window` while the window is
shown, and `VaultWindow._clear()` frees a docked editor before the window builds its own (`dock_released`), so two
editors for one note are never alive together, which is VE-4's hazard (`_editor_path == _browse_path` at every read and
write) closed by construction. **No typed text is lost:** a selection that moves flushes the note through `_write_now()`
first; a refused save (conflict, deleted or read-only note, vanished vault) keeps the old editor on screen with its
banner and HOLDS the switch (the tab says what the selection is and shows its note once a save lands or a banner choice
settles it, `dock_saved`); leaving the tab keeps the editor alive and autosaving underneath; opening the vault window
flushes first; the quit routes are the window's. Departures and judgements found while building: (1) **only settlements
and factions are bound**: a landmark's entity id is made from its key by `vault_landmark_entity_id`, which the tab does
not call yet, and a culture or a conflict has no binding; each says why in the tab instead of drawing an empty editor.
(2) **A subject with several linked notes shows the first** and says how many more there are. (3) **A missing note is
created from the vault's template list, with a blank note as the first choice**, a labelled judgement: Cartalith ships no
templates (Ruling BF), so a vault with none would otherwise offer nothing; creating is a button, never a side effect, and
the note is bound to the subject (`vault_attach`) in the same press. (4) **A followed `[[link]]`** to a note that belongs
to a place or a faction selects it (the panel follows, so the map, Inspect and Ask do); pinned, or when no place owns the
note, it opens in the tab with the window's Back trail. (5) **The dated entries sit under the editor** (up to 12 rows
from `vault_entity_chronos`): pressing one moves the year through `tl_set_year()`, and the row the year falls in is lit
(a ranged entry while the year is inside it, of the single-year entries the latest at or before the year). (6) **The
side panel (backlinks, outline, properties) is not drawn in the tab**: the header's ⧉ opens the note in the vault window,
which keeps it. (7) **The phone** keeps its sheet: `open_context_tab("note")` there opens the vault at the subject's note
(`DccApp._phone_open_note()`, the call Story's "Notes for this place" made before), so the phone's Story and Window rows
lose nothing; the phone shell is unchanged. (8) A selection that changes while another tab is shown does not re-dock: the
note on screen stays docked and autosaving until the tab is shown again. Not built, stated: private notes shown as private
(Ruling DP); the Ask tab's note chip and a note card going to the subject's note (PC-6's deferred items); a note citation
opening the tab at its heading; the tab on a landmark; Pop out. The probe is `_pc5note_probe`.

PC-1 and PC-2 come first: everything else reads them. PC-4 and PC-6 can run in parallel after them. PC-10 follows PC-4.
PC-11 stands alone and can land at any time; the planner's bar row moves with PC-8. PC-7 changes
`popout_host.gd`, which Ruling DK's PW-2 built. PC-8 and PW-4 re-host the planner, so they are one change (§3.7).

## 7. Owner questions

**All answered.** PQ1–PQ9 by Ruling DY, PQ10–PQ13 by Ruling DX (owner, 2026-11-01). Each question had a default;
the mockup now draws the answers. Ruling DY, in short:
- **PQ1:** desktop and tablet, modular, as tabs the user arranges, with ten modules (§3.3). Bookmarks are saved in
  the world file, a follow-up answer.
- **PQ2:** approved as CV-24's design.
- **PQ3:** yes, those five.
- **PQ4:** the planner opens in its own window, which can go to a second screen through DK's Pop out (§3.7).
- **PQ5:** one conversation per session, which can be saved inside the world file. The preference is off by
  default and asked at first use (§3.5).
- **PQ6:** the World menu is renamed Records. Generate world goes to File after New world…. The narrow touch tablet
  shows File · World · Records · Story (§3.8).
- **PQ7:** both entry points, one state.
- **PQ8:** all the keys.
- **PQ9:** quick fields only.

The questions are kept below as they were asked.

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

Raised by the GDScript drafts (their K1–K3), 2026-11-01. **PQ10 to PQ13 are answered by Ruling DX** (owner,
2026-11-01):
- **PQ10:** the two rows go in Edit, on the desktop and the tablet, and not on the phone.
- **PQ11:** one label, "Generate world", everywhere.
- **PQ12:** no; Alt+arrows do nothing in a text field.
- **PQ13:** decided under the owner's delegation: the phone follows the mockup.

The questions are kept below as they were asked.

- **PQ10.** Where do the card's and the cartwheel's rows live, now that their keys need rows to be rebindable?
  *Default: Edit, after Find on map…: "Context card" (Shift+F10) and "Cartwheel at the pointer" (Q), on the desktop
  and the tablet.*
- **PQ11.** Should the regenerate command have one label on every form factor? *Default: keep two: "Generate world"
  on the PC (§3.8), "Run pipeline" on the tablet (its canvas). `_tabletparity_probe.gd` is taught the pair. The
  question is asked because it is finding M2's own complaint about one command under two names.*
- **PQ12.** Should Alt+← and Alt+→ move the selection while a text field has focus, as a browser does? *Default: yes.
  `_typed_into_field()` is the guard if PC-1 decides otherwise.*

Raised by the menu parity check, 2026-11-01 (`cartalith-native/docs/evidence/menu-parity-2026-11-01/REPORT.md`, P2):

- **PQ13.** What should the phone's Window menu show? The mockup's menus are one shared tree, so its phone shows
  "Context panel" and its submenu, as the desktop does. The drafts keep the phone's shipped "Right dock", because §3.11
  gives the phone the model, not the panel (9 rows). *Default: as drafted; the phone keeps "Right dock" until a phone
  form of the panel is designed.*

## 8. Limits of this proposal

- **The mockup's world is a stand-in.** The ownership periods are invented, but shaped like the engine's snapshots:
  Korrath's places belonged to Orentamwash until the split of 210, and a third of the League's were taken in the
  Sevjuni campaign. So are the three conflicts and the relations table.
- **The assistant is scripted; no model is called.** The "used" line under each answer illustrates the kind of call
  the shipped assistant makes; it is not the shipped tool list.
- **Windows show their main sections**, not their whole contents. The Place editor's six tabs are named in §3.3, not
  drawn.
- **The modules draw the stand-in world's data** where it has some. Where it has none (economy and military figures),
  the mockup shows small stand-in values labelled *stand-in*. They are not the engine's numbers.
- **Measured in Chromium (Playwright)** at 1600 × 900, at 1194 × 834 for the tablet's 44 dp targets, and at the
  phone sizes (`MOBILE_SHELL_PROPOSAL.md`). No page errors. Not run in Godot.
- **Mixed sources.** The audit's findings marked *(audit)* were read by an agent and not re-run here. Every count in
  §1 without that mark was re-run in this session with the command shown.
