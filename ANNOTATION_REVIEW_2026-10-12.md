# Annotation review, 2026-10-12 (Ruling BK, independent verifier)

An independent check of the comment-only annotation commits made on 2026-10-12.
The reviewer did not see the authors' reasoning and did not rely on their commit
messages. **Findings only.** No source file was changed by this review. Every
correction below is proposed wording for whoever fixes the comment.

Commits reviewed (9): `9b8d82c4` (static-func slice 1), `745a515d` (slice 2),
`e021d504` (slice 3), `a85e4da9` (slice 4), `286f31f5` (slice 5), `3bfcf327`
(slice 6, workspaces), `67d715e5` (instance-func slice A), `eac7b3ff` (instance
funcs, remainder), `77372b65` (`context_broker.gd` plus 367 Rust test docs and
three generator sources).

## Part 1: mechanical result

**Method.** For every commit and every changed non-`.md` file, both sides
(`<c>^` and `<c>`) were read through `git show`, CRLF was normalised to LF, and
the following were removed: blank lines; full-line comments (`#` for `.gd`, `//`
for `.rs`/`.js`); trailing comments, cut by a quote-aware scanner so a `#` or `//`
inside a string literal is kept. The two stripped sequences had to be identical.
This is stricter than `git diff -U0` filtering because it also covers the
deletions in `e021d504` (75) and `a85e4da9` (57). Those deletions are trailing
`##` comments rewritten as leading ones, and the scanner sees through them.
No commit touched a `.tscn`, `project.godot` or any other non-text type.

| Commit | Files checked | Non-comment change |
|---|---|---|
| `9b8d82c4` | 1 | none |
| `745a515d` | 5 | none |
| `e021d504` | 3 | none (trailing comments moved; code identical) |
| `a85e4da9` | 12 | none (same) |
| `286f31f5` | 29 | none |
| `3bfcf327` | 5 | none |
| `67d715e5` | 5 | none |
| `eac7b3ff` | 16 | none |
| `77372b65` | 76 | **2 files: `tools/civ_tier_feedback_capture.js` and `tools/civ_want_counts_capture.js`** |

The two flagged files are **code changes by design**: they now *emit* the new
`///` lines (a `TEST_DOCS` table and `docFor()` in the first, five `out.push`
calls in the second). The commit message's "Comments only" is literally false
for these two, though it does name the generators. `um_block2_golden_header.js`
passes because its additions sit inside the emitted template text.

**Generated-file reproduction (item d).** All three generators write to stdout,
so each was run into the scratchpad and compared with `cmp`:

| Generator | Target | Result |
|---|---|---|
| `node tools/civ_tier_feedback_capture.js` | `cartalith-civ/tests/golden_parity_centrality_feedback.rs` | **byte-identical** to the working tree and to `HEAD` |
| `node tools/civ_want_counts_capture.js` | `cartalith-civ/tests/golden_parity_want_counts.rs` | **byte-identical** |
| `node tools/um_block2_capture.js` (uses `um_block2_golden_header.js`) | `cartalith-civ/tests/golden_parity_urban_adapter.rs` | **byte-identical** |

So the generator edits change no generated output beyond the docs that were
committed alongside them, and a regeneration will not lose those docs.

## Part 2: truth review

**Method.** Every added comment block was extracted from the nine diffs: 1 546
blocks, 1 177 in `.gd` and 367 in `.rs` (`scratchpad/rev/blocks.py`). Three sets
were reviewed:

1. **Stratified random sample, `.gd`:** 62 blocks from all 9 commits, seeded
   (`random.seed(20261012)`), 6 / 7 / 9 / 9 / 11 / 7 / 6 / 6 / 1 per commit in
   the order above.
2. **Every `intent unverified` / `labelled judgement` comment:** 58 blocks
   (56 `.gd`, 2 `.rs`). Four of them wrap the phrase across a line break, and a
   one-line grep misses those four.
3. **Rust test docs:** a random sample of 42 `///` blocks across 9 crates, plus
   all 30 docs written fresh rather than promoted (3 overlap the random sample),
   so 69 unique docs. All 305 promoted docs were also compared mechanically
   (below).

For each block the reviewer opened the code at `HEAD` and grepped the callers or
readers it names. Constants were checked against the reference HTML, the design
quotes in the file, or the engine source. Verdicts:

- **TRUE:** correct as written. Minor incompleteness counts as TRUE when it does
  not mislead.
- **UNVERIFIABLE:** a rationale the code cannot confirm or refute.
- **MISLEADING:** literally defensible, but a reader would come away wrong.
- **FALSE:** contradicted by the code or by a source in the tree.

### Counts

| Set | Reviewed | TRUE | UNVERIFIABLE | MISLEADING | FALSE |
|---|---|---|---|---|---|
| `.gd` random sample | 62 | 58 | 1 | 2 | 1 |
| `.gd` judgement / intent-unverified (all) | 56 | 37 | 0 | 15 | 4 |
| `.rs` random sample | 42 | 42 | 0 | 0 | 0 |
| `.rs` freshly written, not in sample | 27 | 25 | 0 | 1 | 1 |
| `.rs` judgement (all) | 2 | 1 | 0 | 1 | 0 |
| **Total** | **189** | **163** | **1** | **19** | **6** |

The random sample is the fair estimate of the pass's quality: about 94 % of
`.gd` comments and 42 of 42 Rust test docs are TRUE. The judgement and
intent-unverified set is where the errors concentrate: 19 of its 58 are not
TRUE. **In 14 of those, the comment calls a value a judgement or "no source
stated" when a source exists in the tree:** the reference HTML (8), a design
canvas quoted in the block directly above (3), the file's own header (1), or the
engine default (2).

### Promoted `// Protects:` docs (item c)

The 367 Rust docs split three ways:

- **305 promoted.** In 304 of them the whitespace-normalised `///` text is
  identical to the in-body `// Protects:` block that follows. The 305th,
  `golden_parity_labels.rs::case9_a_long_name_at_a_shallow_arc`, rewrites a
  `# heading` inside the comment into a sentence and reads coherently.
- **32 written fresh** in `golden_parity_library.rs`, where the body already had
  a terser `Protects:`.
- **30 written fresh** with no body comment: 20 in the generated centrality
  file, 7 in the urban adapter, 1 in want_counts and 2 in `lattice_diag.rs`.

That matches the commit's "305 + 32 + 30". A scan for unbalanced backticks or
parentheses, stray `#` headings and hyphen-split words found **no garbled
wrapping**.

Two side effects are worth knowing:

- **Redundant joins.** In 10 promoted docs the in-body block was a `Protects:`
  summary followed by an older paragraph saying the same thing, so the promoted
  `///` repeats itself. Nothing is false, only padded:
  - `cartalith-terrain/tests/golden_parity_zoom_detail.rs`: the docs above
    `a_shallow_level_is_a_byte_identical_no_op_and_a_deep_one_is_not` (line 191)
    and the octave-ceiling test (doc at line 210).
  - `cartalith-civ/tests/golden_parity_faction_aggregates.rs`: doc at line 869.
  - `golden_parity_labels.rs`: docs at lines 121 and 337.
  - `golden_parity_smelting_salt.rs`: doc at line 491.
  - `golden_parity_settlement_placement.rs`: doc at line 180.
  - `golden_parity_pack_manifest.rs`: doc at line 159.
  - `golden_parity_tiles.rs`: doc at line 63.
  - `golden_parity_sculpt.rs`: doc at line 525.

  (The line numbers are where the doc block starts in `77372b65`.)
- **Two copies.** All 305 promoted tests now carry the same text twice: the
  `///` and the in-body `// Protects:`. The next edit that updates one will
  leave the other stale. A follow-up should delete the in-body copy.

### Every non-TRUE finding, with proposed wording

Paths are under `cartalith-native/godot-project/shell/` unless they name a crate.
The line is the declaration at `HEAD`, and the comment sits directly above it.

**FALSE (6)**

1. `place_editor_window.gd:2048` `_build_economy` (eac7b3ff). The doc lists
   "imports and exports groups and the water, food-shed, smelting and salt
   notes" and says "a missing row ... is shown as a dash with its reason". The
   function builds only the Population number field and the Economy
   (specialisation) choice. Imports, Exports, food shed, smelting and salt are
   `_build_trade` (line 2101). **Proposed:** "Population and economy section:
   the population number field and, when the engine supplies a specialisation
   vocabulary, the Economy choice, which is stored on this settlement and not
   fed back into faction aggregates (ED-03). Imports, exports, water, food shed,
   smelting and salt are `_build_trade`'s."
2. `dcc_icons.gd:41` `STROKE` (286f31f5). "Why it is white: intent unverified."
   The file header (lines 8-11) says why: Godot's SVG rasteriser has no
   `currentColor`, so glyphs are drawn white and the host `modulate`s them.
   **Proposed:** "...white so the host can tint it with `modulate`, which is how
   this file stands in for the spec's `currentColor` (see the header)."
3. to 5. `workspaces/civilization_workspace.gd:4840 / 4842 / 4844`
   `COL_TYPE_PX`, `COL_SHARE_PX`, `COL_TREND_PX` (3bfcf327). Each says
   "labelled judgement per the block above". The block above sources them from
   `Religion.dc.html`'s inline widths (TYPE 56, SHARE 44, TREND 40).
   **Proposed:** "Width in px of the Religion table's TYPE column: the design's
   inline `width:56px` (`Religion.dc.html`, per the block above). No theme role
   carries it." Do the same for 44 and 40.
6. `cartalith-civ/tests/golden_parity_urban_adapter.rs:496`
   `infer_age_matches_reference` (77372b65). The doc says the cases cover "both
   ends of its 30..1000 clamp". The reference (`_umInferAge`, v2.10 line 22096)
   floors `p/100` at 1, so the result is never below 60 and the `30` arm cannot
   be reached. The captured ages run from 60 to 1000. **Proposed:** "...at the 22
   captured populations, including the 1000 cap, the 60 floor that
   `max(1, pop/100)` produces (the `max(30, ...)` arm is unreachable), and zero,
   negative and NaN inputs." **Fix it in the generator,
   `tools/um_block2_golden_header.js`, then regenerate.**

**MISLEADING (19)**

7. to 11. Five simulation defaults. All say "labelled judgement, no source
   stated", but each equals the reference's own control default (v2.11): the
   `value` attributes at lines 1940, 1942 and 1944-1947, read at 25420-25421 as
   `/100` and `/1000`.
   - `place_editor_window.gd:225` `_sim_severity` 0.5 = `#civSimSeverity`
     50/100.
   - `place_editor_window.gd:228` `_sim_rate` 0.01 = `#civSimRate` 10/1000.
   - `place_editor_window.gd:231 / 234 / 237` `_sim_start_year` 0,
     `_sim_duration` 100, `_sim_step_years` 10 = `#civSimStartYear`,
     `#civSimDuration`, `#civSimStepYears`.

   **Proposed:** "Default N: the reference's `#civSimX` `value` (v2.11
   1940-1947)."
12. to 14. `workspaces/civilization_workspace.gd:135 / 137 / 139`
    `_tl_sim_start_year`, `_tl_sim_duration`, `_tl_sim_step_years` (3bfcf327).
    Same finding and same correction: 0 / 100 / 10 are the reference's
    `#civSimStartYear` / `#civSimDuration` / `#civSimStepYears` values.
15. `new_world_dialog.gd:446` `_plates_value` (a85e4da9). "Written to
    `tect.plates` on Create. The initial 14 is a placeholder; its source is not
    stated." Create writes it only while `_plates_manual` is set; otherwise
    `_classic_plates()` uses `plates_for_km`. 14 is also the engine default
    (`cartalith-engine` `WorldParams::defaults()`, `plates: 14`). **Proposed:**
    "The hand-set plate count, written to `tect.plates` on Create only while
    `_plates_manual` is set (else `plates_for_km`). Initial 14 = the engine
    default `plates: 14`; not cited at the declaration."
16. `tool_overlay.gd:82` `symmetry_count` (286f31f5). "Default 4 is a labelled
    judgement." 4 is the engine's own `radial_count` default (`cartalith-godot`
    `lib.rs` brush-option table) and `tool_bar.gd`'s `RADIAL_DEFAULT`, which
    says so. **Proposed:** "Default 4, the engine's `radial_count` default
    (mirrors `DccToolBar.RADIAL_DEFAULT`)."
17. `adjust_panel.gd:158` `_gest_label` (286f31f5). "set by `_changed_live`,
    passed to `_end_gesture` by `_end_idle`". It is also set by `_structural`
    and `_on_ramp_begin`, and read by `_structural`, `_on_layer_picked`,
    `_on_generation_started`, `_on_ramp_edited` and the slider release in
    `_slider`. **Proposed:** "...set by `_changed_live`, `_structural` and
    `_on_ramp_begin`; every `_end_gesture` call passes it (idle timeout, layer
    change, generation start, ramp edit, slider release, structural edit)."
18. and 19. `urban_layout_draw.gd:284` `ROOF_V` and `:281` `ROOF_S` (286f31f5).
    "`_roof_color` adds `tone * ROOF_V_SWING`" (or "subtracts `tone *
    ROOF_S_SWING`"). The code uses `t = clamp(tone, 0, 1) * 2 - 1`, so the
    swing is ±SWING around the base and not `0..SWING`. **Proposed:** "...adds
    `(2*tone - 1) * ROOF_V_SWING` (tone clamped to 0..1) and clamps to
    0.30-0.86." Write the same for `ROOF_S` with "subtracts" and 0.18-0.62.
20. `right_dock.gd:6027` `_build_stops` (67d715e5). "The Compare action is a
    no-op (`func(): pass`); intent unverified." The button is built
    **disabled**, and its tooltip states the intent: the design's "(mock)" A/B
    toggle, with no store behind it. **Proposed:** "...The Compare action is drawn disabled
    with its reason as the tooltip (§1.9's own '(mock)' caption; no previous-ramp
    store exists)."
21. `tool_bar.gd:510` `_paint_state` (67d715e5). "those literals are fallbacks,
    not the workspace's own initial values". They are identical to
    `world_workspace.gd`'s initial `_paint_layer` ("biome") and `_paint_brush`.
    **Proposed:** "...returns fixed defaults identical to `world_workspace.gd`'s
    initial `_paint_layer`/`_paint_brush`. Nothing keeps the two in step."
22. `vault_window.gd:2464` `_build_reader` (eac7b3ff). It describes the field and
    "a save-local-copy action. Does not write to the vault folder itself." It
    leaves out the "Insert updated section into source..." action, which writes
    the section back to the vault file through `_confirm_section_write` (preview,
    then a hash-checked confirm), and the "what this note holds" group.
    **Proposed:** "...a Save local copy action, an 'Insert updated section into
    source...' action that writes the section back to the vault file after a
    preview and a hash-checked confirmation (`_confirm_section_write`), and the
    'what this note holds' group. Building it writes nothing."
23. `engine_bridge.gd:604` `_read_param_table` (eac7b3ff). "Called from `_ready`
    and again after a project open or a world load". The callers are `_ready`,
    `load_save` and `close_world`. **Proposed:** "Called from `_ready`,
    `load_save` and `close_world`, because loading or closing a world moves the
    dials."
24. `cartalith-civ/tests/golden_parity_wildlife.rs:323`
    `current_wildlife_matches_the_reference_roster_for_roster`, promoted
    verbatim. "The last assert checks `wild_region_color` agrees". The last
    assert is `total_species > 40`. The `wild_region_color` check is inside the
    per-region loop. **Proposed:** "...An assert in the per-region loop checks
    `wild_region_color` agrees with what was stored." The same sentence is in
    the in-body comment.
25. `cartalith-civ/tests/golden_parity_centrality_feedback.rs:47` `path`, a
    generated doc. "the two interior places at ratio 1.0 and ~0.667 (just over
    0.65) are both promoted". The path has **four** interior places, with ratios
    0.667, 1.0, 1.0 and 0.667, and all four are promoted Town to City.
    **Proposed:** "...the four interior places (ratios ~0.667, 1.0, 1.0, ~0.667,
    the lowest just over 0.65) are all promoted, and both zero-betweenness ends
    are demoted." **Fix it in `tools/civ_tier_feedback_capture.js`'s
    `TEST_DOCS.path`, then regenerate.**

**UNVERIFIABLE (1)**

26. `tool_bar.gd:361` `_note` (67d715e5). "The label is clipped, so it must stay
    the last child of its row." The function does set `clip_text` and add a
    spacer first. The ordering rule is a rationale the code does not show.
    `MISTAKES.md` notes that `clip_text` collapses the minimum width to 1 beside
    an expanding sibling. **Proposed:** keep it, and add the measured reason or
    the incident that produced the rule.

**Noted as TRUE but worth a second look.** `dcc_shell.gd:1317`
`_build_menu_bar` lists the wordmark, the menu host row and undo/redo, but not
the readout cluster (world, pass state, res/cpu/gpu/mem) it also builds. This
is incomplete rather than wrong.

## Systemic observations

- **"labelled judgement" applied after the fact.** About 40 comments label a
  value as a judgement on behalf of an author who never did. Ruling BK asks
  every non-obvious constant to name its source, and "labelled judgement" reads
  as that source. When the annotator simply found no citation, "no source found
  at this line" is honest. "Labelled judgement" is not. The 14 corrections
  above came from checking whether a source existed: four places in the tree
  (the reference HTML, the design file quoted in the same comment block, the
  file header, the engine defaults) would have caught every one of them.
- **Caller lists.** Most were complete (roughly 40 caller and reader claims
  were grepped). The two incomplete ones (items 17 and 23) are both "set or called
  by X" claims that named the first hit and stopped.
- **The generated-test docs live in the generators** and survive regeneration.
  That is verified above. The two generated-file findings (items 6 and 25) have
  to be fixed in the generator, not in the `.rs` file.

## Reproduce

The scripts are in the session scratchpad (`rev/`), not committed:
`mech2.py` (the comment-strip comparison), `blocks.py` (block extraction),
`promo.py` (promoted-text comparison), and the sampling one-liner seeded with
`20261012`. `cargo` was not run, because another agent owns builds. Nothing in
this review depends on a build.

## Resolution (2026-10-12, applied after this review)

Every finding above was re-checked against the code before it was applied.
Comments only (plus the two generators' comment-emitting strings). No `cargo
test` was run; `cargo check --tests -j 1` passed for `cartalith-assets`,
`-civ`, `-climate`, `-io`, `-spatial` and `-terrain`.

**Fixed as written:** 1 (`_build_economy`; the wording also keeps the `s` /
`details` sentence), 2 (`STROKE`), 3 to 5 (`COL_*_PX`), 7 to 11 and 12 to 14
(the simulation defaults now cite `#civSimSeverity` / `Rate` / `StartYear` /
`Duration` / `StepYears`, v2.11 lines 1940-1947), 15 (`_plates_value`), 16
(`symmetry_count`), 18 and 19 (`ROOF_V` / `ROOF_S`: the swing is now
stated as `(2*tone - 1)`, plus or minus), 20 (Compare
action), 21 (`_paint_state`), 22 (`_build_reader`), 23 (`_read_param_table`;
verified: `_ready`, `load_save`, `close_world`), 24 (wildlife; the in-body copy
was removed with the others), `dcc_shell.gd` `_build_menu_bar` (the "noted as
TRUE but incomplete" one now names the readout cluster).

**Fixed in the generator and regenerated:** 6 (`infer_age_matches_reference`,
in `tools/um_block2_golden_header.js`; the same false "both `_umInferAge`
clamps" also sat in the generated file's module doc, so that line was fixed
too) and 25 (`path`, in `tools/civ_tier_feedback_capture.js` `TEST_DOCS`).
`node` output went to a scratch path and was diffed against the committed
tests: the only differences were those comment lines. `want_counts` re-ran
byte-identical.

**Changed from the review's wording:**
- 17 (`_gest_label`): the review's setter list was itself incomplete.
  `_slider`'s drag start also sets it, so that is named too.
- 26 (`tool_bar.gd` `_note`): the ordering rule has no measurement or incident
  in the code or in `MISTAKES.md`, and none was invented. The comment now cites
  the `clip_text` row of `MISTAKES.md` as the mechanism and says plainly that no
  measurement of why the order matters is recorded.

**Judgement / "no source stated" comments:** the eight reference defaults, the
three design widths, `STROKE`, `_plates_value` and `symmetry_count` got their
real source (above). Where an annotator, not an author, had applied the label
and no source was found, the wording is now "no source found at this line":
`ROOF_S`, `ROOF_V`, `ROOF_S_SWING`, `SHADOW`'s alpha, `ROUTE_END`,
`FARM_PASTURE`, `WALL_TOWER_R_M`, `TREE_RIM`, `ROW_LABEL_W`, `_FIELD_LABEL_W`,
`LOG_TAIL_LINES` and `LOG_TAIL_BYTES`, `_selected` (faction roster),
`LIGHT_ALT/AMB/NUM_STEPS`, `_PEEK_CARD_ANIM`, ramp bar `BAR_H` / `HANDLE_H`,
`STAGE_LOG_MAX`, the cartography drag floors and `_icon_gen_spacing`,
`_settlement_faction`, `THUMB_SIZE_PHONE`, and the two wildlife thresholds (more
than 40, more than 500). The four wrapped-phrase comments the grep missed
(`ROOF_S`, `SHADOW`, the start-year default, the roof prose) are among these.
**Left as written, because an author made the call in the tree:**
`adjust.rs::GAMMA_RANGE` ("a labelled judgement" in its own doc, so the two
`GAMMA_MAX` mirrors are correct), `tool_bar.gd`'s ladder bounds (the block above
says "labelled JUDGEMENT"), `busy_overlay.gd`'s card geometry ("Judgements" in
the line above), `tool_overlay.gd`'s "by eye" values. The remaining
"intent unverified" comments (`dcc_settings` save, `tool_changed`,
`atlas_api`, `_matrix_problem_count`, `ID_PREF_QUALITY`, `_PEEK_CARD_MIN_DRAG_DP`,
the `dcc_theme` sizes, `_way_type`, the family chip default) state a real
absence of evidence and were kept.

**Redundant copies.** The in-body `// Protects:` block was deleted from the 304
tests whose text was identical to the `///` above, plus
`case9_a_long_name_at_a_shallow_arc` (whose doc had rewritten the `#` headings),
305 in all. The 32 in `golden_parity_library.rs` were kept: their bodies hold
information the doc lacks. The generated test files had no in-body copy, so no
generator needed that change. The ten docs that repeated themselves were
deduplicated: `golden_parity_zoom_detail.rs` (two), `golden_parity_labels.rs`
(two), `golden_parity_faction_aggregates.rs`, `golden_parity_smelting_salt.rs`,
`golden_parity_settlement_placement.rs`, `golden_parity_pack_manifest.rs`,
`golden_parity_tiles.rs` and `golden_parity_sculpt.rs`.

**Proof.** For every touched `.rs` and `.gd` file the content with all comments
removed (line, trailing and doc forms, line endings ignored) equals `HEAD`. The
`.js` generators differ only inside the strings that emit comments. Every
touched `.gd` file and `shell/app.gd` passes `godot --headless --check-only` in
a scratch copy, and a deliberately broken control script fails.

**Disagreements:** none with the verdicts. One note on method: item 1's review
text names "ED-03" and it was kept because the code comment next to the
`Economy` choice cites the same row.
