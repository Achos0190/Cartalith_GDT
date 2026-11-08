# VAULT_EDITOR_PROPOSAL.md — Markdown editing and linking, as close to a regular Markdown editor as the shell allows

**What this is.** A proposal of 2026-10-31 for the vault's note editor and its links, on desktop, tablet and phone. It
starts from what ships, read at the symbols. It measures that against what nine regular Markdown editors do (with
sources), and lays out the design, the Godot mapping, the Rust crate upgrade, milestones and owner questions.

**The owner's answers.** Ruling DY (2026-11-01) answered VQ1–VQ7 (§8). Two answers depart from the defaults:
- **VQ1** adds a session undo that reaches back to the note as it was first opened.
- **VQ4** keeps **Write** as the view a note opens in.

The design below carries both.

**What it is not.** A build or a status record. How far any of it is built is `cartalith-native/docs/STATUS.md`'s
answer. Routed from `OUTSTANDING_WORK.md` ("Phone and touch", the mobile shell row) and from `MARKDOWN_VAULT_SCOPE.md`,
whose milestones this extends.

**The owner's words (2026-10-31), verbatim:**

> Make sure you think we'll about the markdown linking and editing and how to fit this in a GUI. Make sure it comes
> close to any regular markdown editor.

> For the pc version everything is pretty much in place plus some extras such as the chronos timeline formatting for
> notes and using that for the timeline in cartalith

> You can upgrade the rust crate for the markdown vault accordingly I suppose.

**The mockup.** The vault is built into the mobile shell mockup (`design/mobile-shell-2026-10-31/`, live as the
claude.ai Design canvas named in that folder's `README.md`): MENU ▸ Records ▸ Markdown vault… (the World menu was renamed Records by Ruling DY), the Vault pin, or ⌕ with `n`. Captures are in
`cartalith-native/docs/evidence/mobile-shell-2026-10-31/`: `phone-note-preview.png`,
`phone-note-write-link-suggestions.png`, `tablet-vault-split.png`. The desktop form follows in the PC GUI refactor,
which reuses the same component.

**Sources read.**
- An inventory of the shipped vault, written by an agent that opened `shell/markdown_editor.gd`, `shell/vault_window.gd`
  and `shell/vault_store.gd`, and read the `cartalith-vault` crate, the bridge, the vault documents, Rulings BE, BF, AM,
  AV, DP and the STATUS rows in part. The author checked the editor's own header (`markdown_editor.gd`: toolbar, Write /
  Preview, Obsidian syntax, Ctrl+E, Insert template, Save and Cancel as signals) and `cartalith_vault::chronos`'s header.
- A survey of Obsidian, Typora, iA Writer, Bear, Logseq, Zettlr, Notion, Joplin and Markor (plus Craft and Ulysses
  lightly), from their documentation pages fetched on 2026-10-31. The URLs are in §2.

---

## 1. What ships

**Strong:**
- **Storage.** Plain `.md` files in a folder the user picks (`FsVault`), written to a temporary file and then renamed
  into place.
- **Write safety.** Save goes through `vault_write_file`'s hash guard, which refuses when the file changed on disk.
- **Entity binding.** `KnowledgeLink` records bind a note to a settlement, province, continent, faction, culture or
  landmark (`EntityKind`), with six link states.
- **Templates.** Read from the vault's own template folder, `.obsidian/templates.json` or Templater
  (`template::Config::read`).
- **Machine block.** `<!-- CARTALITH:BEGIN … -->` blocks (`block::upsert`).
- **Chronos.** Blocks are parsed in the Chronos Timeline plugin's exact syntax (`chronos::parse`, Ruling AM). The
  place editor adds events (`add_chronos_event`).

**The editor itself** (`markdown_editor.gd`, Rulings BE and BF):
- a header with Write / Preview, Cancel and Save;
- a toolbar: bold, italic, underline (`<u>`), strike, highlight, Normal/H1–H3, lists, task, quote, callout, code,
  `[[ ]]`, `#` and Insert template;
- Ctrl/Cmd+B, I, U and E;
- each transform is a pure function and one undo step.

**What a regular editor has that this lacks:**

| Area | Today | Evidence |
|---|---|---|
| Rendering while typing | none; Preview renders only when switched to | `set_mode` |
| Source colouring | none; a plain `TextEdit` | — |
| Following a link | **impossible**; no rendered link can be clicked or hovered | no `meta_clicked` / `meta_hover_*` handler in the project |
| `[[` autocomplete | none; the `[[ ]]` button opens an unfiltered list of up to 2 000 notes | `note_link` |
| Backlinks | plain labels, from an index rebuilt only on Refresh | `_build_browse_backlinks` |
| Unlinked mentions | listed without a "link it" action | place editor `_build_backlinks` |
| Rename / move | does not exist; links are never rewritten | searched `rename`, `move_note`, `vault_rename` |
| Following an unresolved link | refused: the index says Cartalith will not create a note | `_build_index_report` |
| Tables, images, task ticking | not rendered / ☐ glyphs only | `to_bbcode` |
| Outline | read-only labels | — |
| Properties | inert chips; frontmatter read only | `_build_browse_preview`, `markdown::frontmatter_fields` |
| Autosave, word count, find/replace | none; Cancel discards with no prompt | `_save_note` |
| Undo history | lost whenever the window rebuilds (dragging across the 640 px fold) | `_rebuild` (inference) |
| Phone toolbar | at the top of the editor; Ruling BE asks for it above the keyboard | `EditorToolbarScroll` |
| Landmark notes | no route from a landmark to its note | every `open_vault(` / `vault_attach(` call |
| Notes in ⌕ search | no notes scope | `SEARCH_SCOPES` |
| Private notes | marker read by the assistant only; no badge | `private::private_marker` |

**Code and document disagreements the inventory found:**

1. `vault_window.gd`'s header says "no wikilink generation", while the editor it embeds writes `[[…]]`.
2. `cartalith-vault/src/lib.rs` says no vault-config directory is read, but `template::Config::read` reads
   `.obsidian/templates.json`.
3. `broken_links` and `backlinks_to` resolve path links differently. A wrong path link is therefore neither reported
   broken nor counted as a backlink.
4. The shell caps listing and backlinks at 2 000 notes, against the crate's 12 000 and a vault of 2 000–10 000 notes.
   The caps sit at six places, each read at its symbol on 2026-11-01:
   - four `vault_list_files(2000)` calls in `vault_window.gd`: the connection header's count
     (`_build_connection_header`), the Browse list (`_build_browse`), the link picker (`_build_editor_pane`) and the
     folder tree (`_refresh_browse_tree`);
   - `vault_refresh_backlinks(2000)` in `_build_index`;
   - the default argument of `engine_bridge.gd::vault_list_files`.

   Ruling DY's VQ7 raises all six to the crate's `LIST_CAP`.
5. Ruling AV's Android folder access (`vault_connect_saf`) is not wired. The in-app browser cannot see files another
   app wrote, so a vault written by Obsidian may show no notes on a phone. This is unverified on a device and is the
   first thing to check there.
6. Two `provider` tests fail on Linux ("C:/Windows/x.md must be unsafe"): a drive-letter path is only absolute on
   Windows. Android is Linux, so the rule must be portable. The crate upgrade (§6) fixes it.

## 2. What users expect from a regular Markdown editor

Counted over the nine main editors.

| # | Convention | How many | Note |
|---|---|---|---|
| 1 | Markdown styled as you type | 9/9 | live rendering is the norm; a separate preview is not (4/9 have one, 3/9 split) |
| 2 | A way back to the raw text | 8/9 | — |
| 3 | Ctrl/Cmd+B, Ctrl/Cmd+I | 9/9 | Ctrl/Cmd+K for a link: 7/9; Logseq and Notion also use K for search |
| 4 | Autosave, no save step | 9/9 | users want to *see* that it saved (an Obsidian forum complaint) |
| 5 | Undo/redo; find in note; replace | 9/9; 9/9; 7/9 | — |
| 6 | Search across notes; fuzzy open-by-name | 9/9; 6/9 | — |
| 7 | Outline with click-to-jump | 7/9 | two also drag a heading to move its section |
| 8 | Checkboxes ticked in place | 7/9 | — |
| 9 | `[[` opens a note autocomplete | 6/9, plus Craft | in Obsidian, `#` picks a heading and `\|` an alias |
| 10 | Links survive rename | 6/9 | rewritten on rename (Obsidian, Bear, Logseq) or linked by id |
| 11 | Back after following a link | 6/9 | — |
| 12 | Backlinks; unlinked mentions | 5/9; 3/9 | expected by anyone who links notes |
| 13 | A formatting row above the on-screen keyboard | 7 of 7 mobile apps | customisable in 4 |

**Touch patterns that matter here:**
- Hover is replaced, not emulated (Notion: "There are no hover states on mobile").
- One tap following a link in an editable view causes accidental navigation. Obsidian's answer, tap-and-hold, is called
  undiscoverable. Joplin shows an "open" tooltip when the caret is in a link (3.7.3).
- Long-press opens variants (Bear: H1–H4; iA Writer: hold I for a table).
- The keyboard and the system bars cover the toolbar unless it is docked to the keyboard (Joplin fixed both).

**Linking patterns worth copying:**
- One popup refined in place: `[[`, then `#` for a heading, `|` for a label (Obsidian; Bear uses `/`).
- A second source that is not notes (Notion's `@` for people and dates). Here that source is the map's entities.
- Create from the picker (Obsidian's quick switcher, Craft).
- Rewrite links on rename with a prompt option (Obsidian).
- Unlinked mentions with one-click linking (Obsidian's outgoing-links pane).
- Backlinks at the bottom of the note as well as in a pane (Obsidian's option, Logseq, Craft). This works on a phone
  with no sidebar.

**Pitfalls:**
- Silent note creation from a missing link.
- Unresolved links that look like resolved ones.
- Rename gaps: heading links (Bear) and queries (Logseq).
- Merge sync that duplicates text (Obsidian's own troubleshooting page).

Sources (fetched 2026-10-31): obsidian.md/help (links, aliases, properties, edit-and-read, mobile, hotkeys,
find-and-replace, the backlinks, outgoing-links, page-preview, quick-switcher and outline plugins, sync troubleshooting);
support.typora.io; ia.net/writer/support (wikilinks, shortcuts, version history); bear.app/faq (linking, info panel,
tables, conflicts); the logseq/docs repository pages; docs.zettlr.com (autocomplete, related files, search, FAQ);
notion.com/help (links and backlinks, shortcuts, mobile); joplinapp.org/help (markdown, linking, conflicts, changelogs);
github.com/gsantner/markor (CHANGELOG, NEWS). Forum reports are marked as such where they are used.

## 3. The design

### 3.1 One note, three views

- **Write** is the view a note opens in, whether from the map, a link or a search (Ruling DY, VQ4). The default asked
  was Preview, and the owner kept today's behaviour.
  - Write is the source, coloured as you type: headings, dimmed markers, links, tags and code. The caret and the
    colours line up.
  - A link is followed with Ctrl/Cmd-click, or on touch with the `Open ›` chip shown when the caret sits in a link
    (§3.2).
  - On a phone, opening a note in Write does not raise the keyboard. The keyboard comes up only when the text is
    tapped.
- **Preview** renders every construct in §3.4. Links follow on a tap or click, peek on a hold (touch) or a 650 ms
  hover (pointer), and tasks tick in place.
- **Split** puts Write and Preview side by side when the note pane is at least 760 dp wide.
- Ctrl/Cmd+E flips between Write and Preview, as shipped.

Live rendering inside the text, Obsidian's Live Preview, is the one baseline item this design does not promise. Godot's
`TextEdit` and `CodeEdit` colour text but do not change size or weight per span (to confirm on 4.7.1 in VE-0). The
coloured source plus Split is the honest substitute, and it is also what iA Writer and Markor do.

### 3.2 Linking

- **Typing `[[`** auto-pairs to `[[]]` and opens suggestions, ranked exact, prefix, word-start, substring, then
  subsequence. There are three sources:
  - notes, by name and frontmatter aliases;
  - world entities without a note: settlements, provinces, continents, factions, cultures and landmarks, marked "no note
    yet";
  - "Link to new note “…”" when nothing matches.
- **Refining in place:** `#` after a name lists that note's headings; `|` keeps an alias. With text selected, the link
  button writes `[[…|selected text]]`.
- **Three kinds of link, each drawn differently:**

  | Kind | Looks like | Following it |
  |---|---|---|
  | to a note | accent, solid underline | opens the note; Back returns |
  | to a world entity with no note | accent, dotted underline, ⌖ | offers "Create note from the Settlement (or Landmark…) template" and "Centre on map" |
  | unresolved | grey, dashed | offers "Create note" with a template choice; **never silently** (the pitfall) |

- **Following:**
  - Preview: tap or click follows.
  - Write: Ctrl/Cmd-click follows.
  - Phone Write: when the caret sits in a link, the keyboard row shows an `Open ›` chip. This is Joplin's
    discoverable fix, not Obsidian's hidden tap-and-hold.
  - Every follow pushes onto the note's back stack.
- **Peek:** hold (touch) or hover (pointer; Ctrl-held in Write) shows a card with the target's first lines, its kind,
  Open, and Centre on map for an entity.
- **Backlinks and unlinked mentions:**
  - Pointer and wide tablet: a side panel, Links · Outline · Properties.
  - Phone and narrow tablet: a sheet, and the same list at the end of Preview.
  - Each linked mention opens the note at that line. Each unlinked mention has a **Link** button that writes
    `[[Name]]`, or `[[Name|as written]]` when the case differs.
  - The index refreshes on save and on rename, not only on a manual Refresh.
- **Rename** (Properties ▸ Rename…) says how many links in how many notes will change, then rewrites them, keeping
  aliases and headings. It also moves the `KnowledgeLink` path, so the binding to the map survives.
- **From the map:**
  - Every entity kind reaches its note (landmarks gain the route they lack).
  - A note bound to an entity shows ⌖ *name* · **Centre on map** under its title.
  - ⌕ finds notes by name and text (`n` prefix).

### 3.3 Editing comfort

- **Toolbar.** The shipped order (bold, italic, underline, strike, highlight, heading, link, tag, task, callout,
  template), then bulleted and numbered lists, quote, code, table, divider, undo and redo.
- **Desktop and tablet:** the toolbar sits under the header, and each button's tooltip names its shortcut.
- **Phone:** the toolbar is a scrolling row **docked to the top of the soft keyboard**, as Ruling BE asks. It ends
  with **Done**, which closes the keyboard and leaves the note in Write (VQ4). Long-press variants: hold H for H1–H3, hold the table button for a 3×3.
- **Shortcuts:**

  | Shortcut | Action |
  |---|---|
  | Ctrl/Cmd+B, I, U | bold, italic, underline |
  | Ctrl/Cmd+K | link |
  | Ctrl/Cmd+L | task |
  | Ctrl/Cmd+E | Write / Preview |
  | Ctrl/Cmd+S | save now |
  | Ctrl/Cmd+F | find |
  | Ctrl/Cmd+H | find and replace |
  | Ctrl/Cmd+Z, Ctrl/Cmd+Shift+Z | undo, redo |

  Bold, italic, underline and Write / Preview already ship. The rest are new.
- **Smart keys.** Enter continues a list, a task (unticked) or a quote; Enter on an empty item ends the list. Tab and
  Shift+Tab indent. `[[` and `**` auto-pair.
- **Saving.** Autosave about a second after typing stops, and on leaving the note, through the same hash guard. The
  footer says **editing… / saved · time** so the save is visible. Ctrl/Cmd+S still saves at once.
- **Conflicts.** If the file changed outside Cartalith, a banner says so and nothing is overwritten. As built (VE-4,
  §7a) its choices are **Keep mine**, **Take theirs** and **Keep both**; the designed **Compare** (a two-column read
  of both versions) is not built.
- **Undo** survives a window rebuild: the history is held by the vault session, not the `TextEdit`.
- **The session undo** (Ruling DY, VQ1). The owner's words: *"keep an undo that reverts untill the opened state.
  This should be session only."* The follow-up answer: "First open this session".
  - **How far back.** Each note's history reaches back to the note as it was the first time it was opened since
    Cartalith started.
  - **What does not move that point.** Closing and reopening the note.
  - **When it ends.** Quitting clears it. None of it is written to disk.
  - **Autosave.** It saves the latest text and does not cut the history.
- **Revert to opened…** replaces Cancel (Ruling DY, VQ6).
  - It says what will be lost and asks first.
  - The revert is one undo step, so Ctrl/Cmd+Z brings the edits back during the session.
  - It is written through the same hash guard as any save, so a change made outside Cartalith raises the conflict
    banner instead of being overwritten.
- **Footer:** word count and the outgoing link count.

### 3.4 What Preview renders

- **Already rendered:**
  - headings 1–6, bold, italic, `__bold__`, `<u>`, `~~`, `==`;
  - inline and fenced code;
  - bulleted, numbered and task lists, quotes and callouts;
  - rules, links, tags, and frontmatter (dim).
- **Added:**
  - **tables**;
  - **images** (`![](…)`, including the map snapshots the vault already writes);
  - **embeds** (`![[note]]` shows the embedded note's first section, collapsed);
  - **task ticking**;
  - `_italic_`;
  - callout colours by type;
  - **chronos blocks** as a small timeline: one axis, then one row per entry. Tapping an entry moves Cartalith's
    timeline to its year. A note's history and the map's timeline stay one thing, as Ruling AM intended.

### 3.5 Structure

- **Properties.** An editable form over the frontmatter: aliases, tags and any `key: value`. Writes go through
  `frontmatter_set` and `frontmatter_remove`, which leave every other line byte-identical. It also shows the entity
  binding (kind, name, link state from `LinkStatus`) with Change… and Unlink, and the **Private** switch with its
  meaning ("kept out of exports and the assistant", Ruling DP).
- **Outline.** Headings, indented by level; a click jumps there in both views.
- **Tags.** An index over the vault. Tapping a tag searches for it.
- **Quick open.** The vault's search field filters names as you type and searches text from three letters.

## 4. Per form factor

| | Desktop | Tablet | Phone |
|---|---|---|---|
| Vault | one window: tree, note, side panel; pop-out as shipped | the same window; the side panel joins Preview below 900 dp | list, then note (push navigation) |
| Default view | Write (Ruling DY, VQ4) | Write | Write; the keyboard comes up only when the text is tapped |
| Write | toolbar under the header; Split from 760 dp | the same | keyboard row; one view at a time |
| Links, outline, properties | side panel | side panel or the end of Preview | sheet, and the end of Preview |
| Peek | hover 650 ms (Ctrl in Write) | hold 480 ms | hold 480 ms |
| From the map | the inspector's Note section (the PC refactor's linked panels) | the same | the selection sheet's note row |

## 5. How it maps onto Godot

- **Write:** `CodeEdit` with a Markdown `SyntaxHighlighter` (colour by column).
  - The suggestion popup is placed at `get_caret_draw_pos()`. On the phone it is docked above the keyboard row instead.
  - The keyboard row sits on the inset the shell already tracks (`_phone_kb_height`).
- **Preview:** `RichTextLabel` BBCode, as `to_bbcode` does today, with these additions:
  - links become `[url=link:…]` and tasks `[url=task:N]`;
  - `meta_clicked` follows a link or ticks a task, and `meta_hover_started` / `meta_hover_ended` drive the pointer peek;
  - tables use `[table]`.

  On touch, a hold on a link reads the meta under the finger. This is unverified on 4.7.1: VE-0 measures whether
  `meta_hover_started` fires under touch emulation.
- **Saving:** the autosave timer calls the existing `vault_write_file` path. The conflict banner replaces today's
  "Reload from disk" line.
- **The session undo:** a per-note history held by the vault session.
  - It is seeded with the text the note had the first time it was opened this session.
  - It follows a rename.
  - It is dropped when the app quits.
  - Revert to opened… writes that first text through `vault_write_file`'s hash guard.
- **Data:** everything the views need (link spans, link under the caret, suggestions, headings, tags, task toggling,
  frontmatter edits, unlinked mentions, rename plans) comes from the `cartalith-vault` crate through the bridge (§6).
  GDScript keeps no second parser.

## 6. The Rust crate upgrade

The owner allowed it on 2026-10-31. It is pure functions in `cartalith-vault`, plus `#[func]` wrappers in
`vault_bridge.rs`:

- **Link spans:** offsets, syntax, target, heading, block and alias, skipping code. Also `link_at` and outgoing links.
- **One resolver**, shared by broken links and backlinks.
- **Rename planning**, and a rename that applies the plan through the hash guard.
- **Suggestion ranking** and **headings**.
- **Markdown helpers:** `toggle_task`, `tags`, `word_count`, aliases, `frontmatter_set` and `frontmatter_remove`.
- **Unlinked mentions**, with **`link_mention`**.
- **The portable drive-letter rule.**

The commit that adds these, and what was tested, is recorded in STATUS.md, not here. The GDScript rewiring is the
milestones below. Where to find them:
- **Modules:** `notelinks.rs` (spans, the `Resolver`, rename planning), `noteedit.rs` (headings, tasks, tags, words,
  mentions), `frontmatter.rs`, `suggest.rs`, and the private `textscan.rs`.
- **The bridge:** 17 `vault_*` functions in `vault_bridge.rs`.

One gap the build found and this design must carry: **on Android a rename needs a `move` operation** in the Storage
Access Framework handler (`vault_saf.rs`). It has none, so until it does, rename refuses there rather than half-moving
a note.

## 7. Milestones

| # | Milestone | Bar |
|---|---|---|
| VE-0 | Measure on 4.7.1: `SyntaxHighlighter` capabilities, `meta_hover_started` under touch emulation, and the Android folder access of Ruling AV on the 6T | a written answer to each; MQ-like follow-ups if one fails |
| VE-1 | Links work: `meta_clicked` follows, Back returns, the three link kinds look different, and peek works | every link in the four sample notes followed and returned from; unresolved and entity links never create silently |
| VE-2 | `[[` suggestions with headings, aliases and entities, plus auto-pair | the picker is never unfiltered; Enter, Tab, Escape and arrows work; at least 44 dp rows on touch |
| VE-3 | Write colouring, Split, smart keys, find and replace, word count | caret and colours align at 3 widths; Split appears exactly from 760 dp |
| VE-4 | Autosave, the conflict banner, the session undo and Revert to opened… (VQ1, VQ6) | an edit outside Cartalith is never overwritten, by a save or by a revert; "saved" is visible within 2 s. Undo reaches the first-opened text after the note is closed and reopened, and nothing of it is on disk after a quit. Revert asks first and is one undo step |
| VE-5 | Backlinks pane, unlinked-mention Link button, outline jump, properties form, tags index | every listed mention opens its note at its line; a frontmatter edit leaves the other lines byte-identical |
| VE-6 | Rename with link rewriting, and create-from-link with a template choice | links survive the rename in every note that had one, aliases and headings kept, and the `KnowledgeLink` follows |
| VE-7 | Tables, images, embeds, task ticking, chronos rendering, callout colours | each renders in Preview; a chronos row moves the timeline |
| VE-8 | Phone: keyboard-docked row, `Open ›` chip, sheet, Done | the row stays above the keyboard on the 6T at both orientations |
| VE-9 | Map routes: landmark notes, a notes scope in ⌕, private badge, the 2 000-note caps raised to the crate's (all six sites of §2, VQ7) | a landmark reaches its note in one step; search finds a note by a word in its body; `grep -n "2000" shell/vault_window.gd` finds no cap |

Each closes with `cargo test --workspace` green and every touched `.gd` parse-checked, plus `shell/app.gd`
(`MISTAKES.md`).

## 7a. Build notes

*What was built is `cartalith-native/docs/STATUS.md`'s answer ("VE-0 and VE-1 built", "VE-2 built", "VE-3 built", "VE-4 built", "VE-5 built"); this records what the build learned that the design should carry.*

- **VE-0's answers.** (1) A `SyntaxHighlighter` honours colour only, so Live Preview stays out of scope and §3.1's substitute stands. (2) A still finger sends no motion event, so `meta_hover_started` does not fire under touch; the touch peek is a 500 ms hold the editor builds (§3.2, §5). (3) Android folder access on the 6T was not measured (no device); the question stays open and `move` in `vault_saf.rs` is still needed.
- **VE-1 departs from §3.2's table in one way:** BBCode has no dotted or dashed underline, so the three link kinds differ by colour, underline, italic and a glyph. A live-drawn dotted line would need a custom draw; not scheduled.
- **Following a link from a note with unsaved edits asks first** (the existing Discard / Cancel card). It does not save for the user; that is VE-4's autosave.
- **Create from a link is a menu row, never a side effect** (VQ2, VQ5). Its template choice is VE-1's minimum; VE-6 still owns the `KnowledgeLink` and a proper chooser.
- **VE-2 departs from §3.2 in four ways.** (1) The card lists at most 8 rows (6 on the phone), and an empty query shows the first 8 alphabetically, notes before places: "never unfiltered" is read as "never the whole vault". (2) Places are the four kinds the link follower already classes (settlement, province, continent, faction); cultures and landmarks wait for that classifier (VE-9). (3) The link button no longer opens a list: it writes `[[]]` (or `[[|selected]]`) and the card does the rest. (4) The card is a top-level panel, not a `PopupMenu`, because a popup would take the keyboard focus the text needs.
- **Build facts carried forward.** `AcceptDialog`'s Escape property is `dialog_close_on_escape` in 4.7.2, and the vault window needs it off while the card is open or Escape raises the unsaved-edits prompt. `TextEdit.text_changed` is deferred, so the editor's own multi-character writes cannot be told from typing by a flag; the auto-pair keys on a `+1` length step. Aliases come from the backlink index, so they appear only after a refresh. A probe that types into a vault beside a generated world must not choose fixture names a place name can share (a settlement called `Mar...` outranked a note called `Marren` alphabetically).

- **VE-3 departs from the design in these ways.** (1) **Write is colour only**, as VE-0 found; headings are accent, markers dim, links water, code the block red, tags green, `==` warm. Text roles are held to 4.5:1 and the dimmed marker to 3:1 against the field ground in both themes by nudging a token toward `text_bright` (`legible`); neither this document nor `DESIGN_HANDOFF.md` states a numeric bar, so those are the project's own, recorded in `markdown_highlighter.gd`. The raw light `accent` is 3.93:1 and `good` 4.39:1, which is why a floor is needed at all. (2) **Links are coloured from the crate** (`vault_link_spans`, per line); the lexical rest (markers, tags, fences, front matter) is a line scan in GDScript, because the crate does not model colour. (3) **Split is two panes in a row, not a draggable `SplitContainer`**: no divider to drag, and no scroll sync between the panes (Preview keeps its own scroll across refreshes). It is offered when the editor is at least 760 control units wide (which are dp) and never on the phone; a note squeezed below 760 falls back to Write and returns when the room does, unless the user chose Write meanwhile. (4) **Enter, Tab, `**`**: a numbered item counts up but later items are not renumbered; Tab indents with a tab character; `**` pairs, the closing `*`s are typed over, a third `*` withdraws the pair, and Backspace in an empty `**|**` removes it. (5) **Find** has a match-case toggle only (no whole word, no regex); hits are shown by colouring their text (a `SyntaxHighlighter` cannot tint a background), the current one is selected. Replace all is one undo step. (6) **Ctrl/Cmd+S emits `save_requested`**: the user's own save through the window's hash-guarded write; nothing saves by itself until VE-4. (7) **The footer** shows words and outgoing links from the crate, and leaves a figure out when the crate cannot answer.
- **Build facts carried forward from VE-3.** (a) On Godot 4.7.2 `TextEdit.get_rect_at_line_column(line, col)` answers for the grapheme **before** `col` (column 1 reports column 0's x), so it cannot say where a coloured run starts; the probe uses the hit test (`get_line_column_at_pos`) on the drawn pixels. (b) The `TextEdit` redraws an edited line before its deferred `text_changed` arrives, so a highlighter that compares line state "since the last refresh" sees nothing; it does not need to, because the `TextEdit` drops the cached colours of the edited line and every line below it itself (a line below a newly typed fence turned to code colour with no clear of ours). Nor did changing the find query need a clear: the hits were redrawn on lines nobody edited. (c) The caret and the focus border are drawn in the accent, a heading's own colour: a pixel probe of heading ink must park the caret and start inside the border. (d) `SyntaxHighlighter` is a script class here, held without a `class_name`, as the editor is.
- **VE-4 departs from the design in these ways.** (1) **The conflict banner offers Keep mine, Take theirs and Keep both, not Reload / Keep mine / Compare** (the milestone brief named the first three; Compare, a two-column read, is not built). Keep mine writes against the hash of the version the banner described, not a fresh read, so a second outside change after the banner refuses again and the banner returns. Take theirs loads the disk version as ONE undo step, so Ctrl+Z brings the user's text back. Keep both saves the user's text as `Name (my edits YYYY-MM-DD HH-MM-SS).md` beside the note (never over an existing file), then takes theirs. (2) **Other refusals are banners too, not conflicts:** a deleted note (Recreate with mine / Close note; it is never recreated by itself), a read-only or locked file, and a vanished vault folder (Try again). Each is told apart by a `kind` the crate gives (`Error::write_kind`), never by matching English. (3) **The session undo is checkpoints, not keystrokes.** The window keeps, per note, the first-opened text and one checkpoint per save or rebuild (at most 100, and 4 million characters, dropping the oldest after the first-opened one); an editor built later replays them as one undo step each, so Ctrl+Z walks back to the first-opened text after a close and reopen or a rebuild, but not keystroke by keystroke across one. Memory only; cleared on connect and disconnect; `history_follow_rename` is there for VE-6 to call. (4) **A CRLF note stays CRLF.** The editor holds LF; the window converts back on write when the file used CRLF, and compares dirtiness on LF, so opening a Windows note and leaving it alone writes nothing. (5) **Save keeps its label and Ctrl+S**; Cancel became **Revert to opened…**; the "Reload from disk" button is gone (the banner replaced it). (6) The crate's `write_file` now **skips a write whose text equals the file** (the file's time does not move) and refuses a read-only file on every host; `create_file` is new and never overwrites.
- **Build facts carried forward from VE-4.** (a) `_open_editor` assigns the new note's path and text **before** `_rebuild` frees the old editor, so for that moment the old note's `TextEdit` is alive under the new note's name; anything that reads the editor must check `_editor_path == _browse_path` (the probe's first run found `Qconf1.md` holding `Qalpha.md`'s text before this was added). (b) An embedded dialog that closes leaves no embedded window holding the keyboard, so after the Revert dialog the window and then the text must be given focus again, or typed keys and Ctrl+Z reach nothing. (c) A rename over a read-only file is refused by Windows but **succeeds on POSIX**, which is why the crate checks the permission itself. (d) `FileAccess.set_read_only_attribute` and a second process holding the file with `FileShare.Read` are how a probe makes a real read-only and a real locked file.
- **Not done in VE-4, honestly:** Compare; a check when the window regains focus (an outside change is found when the next save is refused, not before); the Android SAF handler's atomicity (the Java side writes the file; not measured, no device). *(Closed afterwards: a flush on quitting the whole app, and the `mark_world_dirty` that fired on every autosave, see the next bullet.)*
- **Quitting and the world's dirty mark (a later session).** The app's exits flush the editor: `VaultWindow.flush_for_exit` runs before `DccApp._close_requested` and `_back_exhausted` decide anything, on `NOTIFICATION_APPLICATION_PAUSED` (with the rescue) and on window focus-out (the plain guarded save). A refused save is not dropped and not forced: the user's text is written beside the note as `Name (my edits <stamp>).md` (never over a file) and the exit goes on; when even that fails the first close request is held and the window is shown, the second goes through. `vault_write_file` no longer marks the world unsaved: a note is not world data (`VaultSession::write_file` touches no link, and `SAVEFILE_COMPAT.md` s.13.3.3 derives a link's staleness at read time).
- **VE-5 departs from the design in these ways.** (1) **One panel, four tabs** (Links holds both the backlinks and the unlinked mentions), shown beside the editor from a 1 100 wide window, below it on a narrower one, and on the phone as a sheet that replaces the editor with a "‹ Note" button; the copy at the end of Preview and the tablet's 900 dp rule are not built. (2) **Link links the first mention of the note it is pressed on**, through the hash guard, and the note then leaves the list (it is a backlink now); the list is capped at 40. (3) **A backlink opens at the line of its first link to this note**, found from the source's own link spans resolved to this note; a mention opens at its first mention. (4) **Tapping a tag lists its notes from the vault index** (counted by note, case-folded, scanned up to 2 000 notes and said to be partial when it was) instead of running a text search for `#tag`. (5) **The properties form** edits scalars, lists, adds and removes, as one undo step each on the editor's live text; the entity binding with Change… and Unlink and the Private switch are not built. (6) The editor header is an `HFlowContainer` so the new **Panel** button wraps instead of cutting Save off.
- **Build facts carried forward from VE-5.** (a) A `Variant` holding a `String` converts to a `PackedStringArray` (empty) with gdext's `try_to`, so a bridge that tries conversions in order writes `key: []`; dispatch on `Variant::get_type()`. (b) A `Node` name cannot hold `.`, `:`, `@`, `/`, `"` or `%`, which every note path does: use `validate_node_name()` for any name made from data, in the code and in the probe. (c) Freeing a focused `LineEdit` emits `focus_exited`; a handler that commits on it must stand down while its panel is torn down, and the rebuild that follows a commit should be deferred. (d) Preview joins consecutive text lines into one paragraph, so a probe that needs a tall Preview must use list items or blank lines. (e) A multi-line string literal written through a script that changes line endings turns its `\n` into CRLF: write escapes, not line breaks.
- **Not done in VE-5, honestly:** the entity binding and Private switch in Properties; the end-of-Preview copy and the tablet form; a text search from a tag; one mention per press; scrolling Preview to a heading with inline markup; a mutant for the commit re-entrancy guard; the Android pause path; and VE-6 onward.
- **Not done in VE-3, honestly:** the toolbar's table, divider, undo and redo buttons and the phone's long-press variants (not named by VE-3's milestone line; the phone row is VE-8); scroll sync in Split; whole-word and regex find; renumbering after a list edit.

## 8. Owner questions, with defaults

**All answered by Ruling DY** (owner, 2026-11-01, `LARGE_ITEM_RULINGS.md`).

| # | Question | Default | Answer |
|---|---|---|---|
| VQ1 | Autosave (about 1 s after typing stops, and on leaving) instead of explicit Save; Ctrl/Cmd+S kept? | Yes | **Yes, plus a session undo back to the note as first opened this session** (§3.3) |
| VQ2 | Following an unresolved link offers to create the note with a template choice, reversing today's refusal? Never silently | Yes | **Yes** |
| VQ3 | Rename rewrites links in other notes, after showing the count? | Yes | **Yes** |
| VQ4 | Notes open in Preview, with one tap to Write? | Yes | **No: notes open in Write**, with Preview a toggle (§3.1) |
| VQ5 | A link to a world entity with no note shows ⌖ and offers that kind's template? | Yes | **Yes** |
| VQ6 | Cancel with unsaved edits asks first, like every other way out? | Yes | **Yes**, as **Revert to opened…**, which asks first and is one undo step (§3.3) |
| VQ7 | Raise the shell's 2 000-note caps to the crate's 12 000? | Yes | **Yes**: all six sites of §2. The question's list was not exact: search is not one of them, and it missed the header's count, the folder tree and the bridge's default |

## 9. What the mockup does not settle

- Its Write view is an HTML textarea over coloured text. Godot's equivalent (§5) is to be measured in VE-0.
- Its notes are eight samples written from the stand-in world, not a vault on disk. Rename, conflicts and templates are
  simulated.
- It does not show Compare, find and replace, or image attachments.
- It was driven in Chromium only.
