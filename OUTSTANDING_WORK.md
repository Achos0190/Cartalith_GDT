# Outstanding work

> **Relationship to `cartalith-native/docs/STATUS.md`** (added 2026-08-31, when
> STATUS.md became the single source of truth for progress)
>
> **This file answers "what is left, and where do I go to do it?"** It is a
> *routed backlog*: every row names the document that owns the work, carries a
> size, and sits in one of six buckets — in flight, ready, blocked, open
> decision, declined, or a defect in the project record.
>
> **`STATUS.md` answers "what state is this in?"** — verified against the code,
> per milestone. **Where the two disagree about a status, `STATUS.md` wins**,
> and the disagreement is a bug in this file to fix rather than a fact to
> reconcile.
>
> The division is deliberate: a ledger of what exists and a queue of what is
> next are different shapes, and merging them is how the last `STATUS.md` grew
> to 8 122 lines. Keep it that way — **this file must not acquire a "done"
> column.** A row that is finished is deleted from here and recorded there.
>
> *One correction to the paragraph below, which was written hours before the
> rewrite it describes: `STATUS.md` is no longer 8 122 lines and no longer
> disclaims its own purpose. It was rewritten from scratch on 2026-08-31 against
> the working tree and is **1 157 lines**. The rest of the reasoning stands —
> the twenty-odd scope documents still answer the question only for themselves,
> which is why this file exists.*

The single list of what is left, assembled 2026-08-31 from every scope document
in this repository plus a code check of the claims that mattered most. It exists
because the question "what is actually left?" was costing a session most of its
budget: `STATUS.md` was 8 122 lines with four lines over 15 000 characters and
said in its own header that it no longer met its purpose, `CHANGELOG.md` stopped
five days short of the working tree, and the twenty-odd scope documents each
answer the question only for themselves.

**This document is a router, not a replacement.** Every row names the document
that owns it. Where a row's owning document disagrees with the code, the code
won and §6 records the disagreement — those defects in the project record are
worth more than any single milestone row below, and a reader with ten minutes
should read §6 before §2.

## The count, honestly

**97 outstanding items across 24 subsystems** — **re-derived by counting table
rows mechanically, 2026-09-05 (thirty-second pass)**, after an earlier pass left four
different totals in this file at once (a headline of 142, a table summing to 143,
and a report claiming 145). That is §6.8's own "counts that disagree with
themselves", reintroduced; the fix is the count, and the lesson is that the
arithmetic here is not safe to delegate.

The figure is `§1 + §2 + §3 + §4`, with §5's declined entries deliberately
outside it.

**2026-09-05, the menu design-conformance audit and its first fixes — 96 → 97.**
Owner instruction: the GUI verification runs **before** the rest of the list.
Four Fable 5.1 auditors at Ultracode plus an adversarial cross-check, then six
fixes on Opus 5. `cargo test --workspace` holds at **3 133**, 0 failed — correct
for GDScript-only work.

**283 menu items, enumerated from the code rather than from the designs** —
125 conforms, 99 deviates, 37 no-design, 17 design-stale, 5 unreachable. The
enumeration direction was the point: walking the designs and ticking items off
can only find what the designs already list, and 37 surfaces have no design at
all.

*The cross-check earned its place, and its verdict was `partly-unsound`.* Seven
of 42 verdicts refuted, two materially: tool shortcuts V/M/R/B/F/L/I were
reported **unreachable** when `dcc_widgets.gd:948` binds them — a fix lane would
have worked against a defect that does not exist — and a phone-tab count of five
was wrong where two other auditors had four.

*Six fixes shipped, all confirmed.* Autosave defaulted **off** and to **10 min**,
a value **not on its own ladder**, so a fresh install opened the interval submenu
with every radio row unchecked; the settlement panel printed a faction **number**
where the design says the owning **name**; the paint row had Commit and no
Discard; the collapsed left dock showed WORLD's word in every domain; the
collapsed timeline strip measured **17 px** against a 24/34 spec. Migration was
decided deliberately — only the **absent** key moves, an explicit choice is never
rewritten.

*A refutation that prevented damage.* The lane proposed removing
`ROLE["h_timeline"]` and `H_TIMELINE` as dead; `_roleresolve_probe.gd`, a
committed file, reads both. It raised this rather than acting, and the verifier
confirmed the constants are live.

*Eighth consecutive batch with a false clause in prose written the same pass* —
a doc naming `_refresh_stage_rows()`, which exists nowhere (the caller is
`_paint_stage_rows()`), and a dash reason saying "generate a world first"
rendered on a fully generated world holding six factions. Both corrected.

**2026-09-04, twenty-seventh batch — 96 → 95.** Pack trait art is built end to
end and parked behind one line; the group-header row closes on a measurement.
`cargo test --workspace` **3 129 → 3 133**, 0 failed. Every verdict confirmed.

*The row that was filed as app-wide turned out to be two headers.* The batch that
found the `group()` header problem filed it with its blast radius unmeasured. It
is **2 headers over their floor out of 59 distinct headers across 141 surface
states** — and the change was taken anyway only because reverting all 57 live
headers to `AUTOWRAP_OFF` and diffing the framebuffer measured **0 px** of
difference, i.e. the fix costs nothing rather than being merely worth it. The
verifier widened the relationship check past the lane's own: **544 assertions
over 55 headers, 0 failures**, with headers identified structurally rather than
by label.

*The trait-art work is finished and deliberately not switched on.* Everything on
both sides exists — the resolver returns a whole pin's row from Rust so no
geometry is re-derived in GDScript, and the no-art path is byte-identical to the
committed file across a full frame. The single remaining line is **withheld on
purpose**: installing it puts trait art on the live map, which falsifies the
pack-import warning's `trait` clause, and removing that clause moves a golden
literal only the owner can authorise.

*Seventh consecutive batch with a false clause in a lane's own new prose.* Three
of them this time, all naming the same wrong precondition — "until a pack is
imported", when the real gate is that no resolver is installed. A world can hold
a pack full of trait art and still draw discs.

**2026-09-04, twenty-sixth batch — 98 → 96.** Trait badges reach the map and
labels now clear them; the GeoJSON parser is built and stops at the ruling it
needs. `cargo test --workspace` **3 087 → 3 129**, 0 failed. Every verifier
verdict confirmed.

*The label-clearance fix was falsified before it was believed.* Per-pixel
intersection of a settlement name's own ink with the badge row, three fixtures,
before → after: **70 → 0**, **236 → 0**, **75 → 0** px. The verifier re-measured
against the actual committed file rather than the lane's `_PreChange` subclass,
and confirmed the no-trait path byte-identical across the full 2 400×1 200 frame.

*And the port had not "fallen behind the reference" — it had dropped a parameter
present in the very lines it cites.* `lblCandidates` carries `drop` at both
v2.10:15716 and v2.11:16199; HEAD's port took five parameters where the
reference takes seven.

*A premise in the batch brief was wrong, and the lane caught it at the symbol.*
`composite_trait_badges` is a plain `pub fn` taking a raster buffer and a Rust
struct — **GDScript structurally cannot call it**, so the pack-art half was
never closable from `map_overlay.gd`. Four Rust doc comments asserting "the
caller is GDScript" were false and are corrected. What shipped instead is the
reference's own no-art branch, which is this port's only state for every pack.

*Sixth consecutive batch with a false clause in a lane's own new prose*, plus a
botched line-wrap that left five stray tabs mid-expression — it parsed, because
tabs are whitespace, and only a verifier reading the bytes found it.

**2026-09-04, twenty-fifth batch — 97 → 98.** The bounded paint upload closes
end to end; the trait-sprite Rust half closes and the row re-scopes to the one
GDScript caller it still needs; two cross-lane findings filed. `cargo test
--workspace` **3 075 → 3 087**, 0 failed.

*The win survived contact, and is quoted with its spread rather than as a ratio.*
Per dab in the shell, before → after: 512² **1.44 → 0.85 ms**, 1024²
**4.44 → 1.12 ms**, 2048² (the shipped default) **16.51 → 1.85 ms**. Sixteen
milliseconds per pointer-move sample is a dropped frame at 2K, and it is gone.
Proven byte-identical to the full re-upload after every one of 20 dabs, 8 of 8
mutants killed, and the verifier re-derived it on its own world with its own
brush path. The four boundary states are kept distinct — including a world
regenerated mid-stroke, where the new window would have *fitted* the stale
mirror and was refused anyway.

*Fifth consecutive batch with a false clause in a lane's own new prose, and this
time two.* One attributed the fallback dab to the "after" column's maximum
(2.55 ms) when the fallback measures **15.3 ms** — a cause asserted without
measuring the cause. The other said the sculpt and paint preview rasters share a
format; they do not (`RGB8` at `lib.rs:8334`, `RGBA8` at `:9284`), which makes
the opt-in flag load-bearing for a *stronger* reason than the sentence gave.
Both corrected, along with a `blit_sprite` doc that called the function
bottom-anchored three lines above the centre-anchored caller that had just
landed.

*Worth knowing for every future probe:* `ImageTexture.update()` is a **no-op
under `--headless`** — reproduced independently. A pixel probe against a texture
updated that way must run windowed or it proves nothing.

**2026-09-04, twenty-fourth batch — holds at 97.** The Journey panel width
closes; the bounded paint upload's Rust half closes and the row re-scopes to the
shell wiring. `cargo test --workspace` **3 068 → 3 075**, 0 failed.

*The row's own diagnosis was wrong, and it was written by the main loop.* It said
"the panel is dropdowns whose minimum is their widest item". There is **no
dropdown anywhere** in `build_results()` or its fourteen helpers — every dropdown
in the planner lives in the left dock. The real causes were an `HBoxContainer` of
action buttons demanding the **sum** of their sentence-length labels (209 + 202 +
8 = 419, +22 padding = **441**), and unbounded `_kv_row()` label/value pairs. Fix
was 12 lines: flow instead of box, and expand-plus-autowrap on whichever side is
naturally wider. **441 → 258 px body, 456 → 280 dock, identical on three seeds.**

*The open question from last batch is settled by measurement rather than left as
inference.* At `HEAD~1` the dock measured 441 / 441 / 441; at `HEAD` the same.
**Pre-existing, not caused by rule 8's append** — and the verifier re-derived it
on three seeds of its own choosing.

*A verifier stopped a false rule entering `MISTAKES.md`.* A lane reported that
the non-console Godot binary "writes nothing to a redirected stdout"; measured,
it writes 602 bytes. Recording that would have made a standing rule out of a
wrong observation.

*Fourth consecutive batch in which a lane shipped a false clause inside its own
newly-written prose* — this time "absolute figures move a few percent between
runs, the ratios do not", refuted by re-running the bench. Only the **byte**
ratio is stable, because it is `grid / window` arithmetic rather than a timing.

**2026-09-04, twenty-third batch — holds at 97.** First batch at two build
lanes. Rule 8 closes: the Journey planner appends instead of replacing, per the
owner's 2026-09-04 ruling. The pack-section re-derivation lands as the audit it
was asked to be. `cargo test --workspace` unchanged at **3 068**, 0 failed —
correct for a GDScript lane plus an audit.

*The hazard was carried across, and it was not where the ruling described it.*
The ruling, the row and `right_dock.gd`'s own doc all describe a conversion as
three artefacts — a `CTX_` constant, a `CTX_TITLES` row, a `_dispatch()` arm.
There was a **fourth**, in a different file: `journey_planner_view.gd::
build_results()` opened by clearing `right_dock_body` **itself** — harmless while
the planner replaced the dock, destructive the instant it appends, because the
selection lives in that same container. Mutation-verified: re-inserting only that
teardown turns the probe red at exactly the right check.

*A third premise about the pack warning failed, which is why the owner asked for
the measurement.* The backlog row said `composite_map_icons` draws settlement and
poi. It does not — it composites the `icons` family and nothing else, so those
two are undrawn as well. **True unused set: `seamarks`, `settlement`, `trait`,
`poi`, `custom`**; composited: `textures`, `biomes`, `terrains`, `icons`. Only
one section is emittable by the warning today.

*A verifier caught a single-sample measurement.* Lane A reported "no dock
overflow" from one world whose plan was empty; three worlds measure the results
panel at 351 / 385 / 441 px against a 280 px dock. Filed above, with what is
measured kept apart from what is inferred.

**2026-09-04, twenty-second batch — holds at 97.** Vault milestone 3 closes as
already done (shipped 2026-09-02 in `4ec07f5`; `STATUS.md:846` already said so);
right-dock rule 7 is built; the preview row is re-scoped by measurement rather
than reworded. `cargo test --workspace` **3 037 → 3 068**, 0 failed.

*Two counts in the record were wrong in the same direction — too high.* The
right-dock ladder has **8 reachable rules, not nine**: the ninth (`ROUTE`) is a
dead entry the spec itself flags, since `rdMode4()` returns `way` for both tools.
Of the eight, only **rule 7** was genuinely unbuilt; rule 5's remaining half asks
for a settlement inspector this dock already draws.

*A real data-loss bug, found by a verifier and fixed here.* `vault.json` was
written only when `links` was non-empty — one member of a three-member store — so
a project with a connected vault and a map snapshot but no knowledge links wrote
**no document at all** and lost the snapshot on save. The predicate that answers
for all three (`LinkStore::is_empty()`) was built and mutation-tested in the same
batch and simply was not wired. Now wired; the call-site guard is filed above,
because `project_save_with_documents` takes gdext types and no Rust test reaches it.

*The paint preview's decline was measured false.* Its prose called the saving
"negligible"; a full-grid rebuild costs **0.73 / 1.48 / 4.55 / 16.80 ms** per dab
at 512/1024/2048/4096 squared and re-uploads **1 MB to 64 MB** each time, while
`touched_bounds` covers **1.80%** of the grid at 2048. A second comment asserting
the preview *is* cheap per dab was corrected with those figures. The sculpt half's
decline, by contrast, still holds and is owned elsewhere.

**2026-09-04, twenty-first batch — 99 → 96.** DS-03 closes, the pack-manifest
re-baseline closes, the religion share export closes. `cargo test --workspace`
**3 024 → 3 037**, 0 failed.

*DS-03's premise was false and the lane found something worse.* The row (and my
brief) said the tablet deletes ~30% of desktop content. **The deletion set is
zero** — 52 `is_tablet()`/`_touch` branches in `shell/`, every one reflows, none
deletes, confirmed by an identical per-class control census at both densities
across all 10 (domain, mode) pairs. The "~30%" is a property of the *artboard*
(`GUI_GAP_REGISTER.md:9529`, tablet 2560 vs desktop 1920), never of this shell.
**What was really losing content was horizontal overflow through a
`SCROLL_MODE_DISABLED` axis** — the MISTAKES trap, third occurrence. Measured at
tablet: CARTO ▸ Labels forced the 400 px left dock to **1 589 px**, eating 1 189 px
of map; CIVIL ▸ Factions to 555; Landmarks to 417; with a world, WORLD ▸ Generate
to 783; and a fifth panel the lane's own report missed, CIVIL ▸ Infra at 597.
**Desktop was broken the same way** (472 and 1 212 px inside a 372 px slot), so
two desktop panels deliberately move — both from rendering outside their slot to
fitting it.

*The project's first authorised golden re-baseline landed, and held its scope.*
Exactly one string and three fixtures moved; a workspace-wide diff of every
golden and fixture confirms nothing else did. The permanent divergence is
disclosed at five sites a future parity run will actually meet, and all three
fixture sites are load-bearing — re-adding `unused.push("biomes")` is killed.

*The ruling's own premise did not survive being re-opened, and the lane did not
widen anyway.* `trait` is **not** the only true clause: `composite_map_icons`
draws settlement and poi sprites too. Lane B pinned the decision with a test
whose doc says changing it **is** the disclosure, and raised the question rather
than acting on it. That is the right call and it is an open owner question.

**2026-09-03, twentieth batch — holds at 99.** The save-format provenance gap
closes; the religion screens close and are replaced by the narrower engine gap
they exposed; two rows are added from verifier findings. `cargo test --workspace`
**3 015 → 3 024**, 0 failed.

*The batch's most valuable finding was a sentence that had been true of nothing.*
The religion roll-up read `Sun Cult — 9 816 people (8.0%), leads 20 settlements`.
Measured over a real world: **158 of 173 settlements have population 0** (village
add-ons, faithful to the reference), so their adherent dictionaries arrive empty
while their plurality is real. The 9 816 people were minorities inside the 15
populated towns Sun Cult does **not** lead, and all 20 it leads hold nobody —
**the two halves of one sentence shared no settlement.** Alongside it, 43
settlements the roll-up counted for a faith had rows that named no faith at all.
Both fixed, and the hover card now says *why* a pin has no share — proved by a
**framebuffer difference** (25 391 of 2 160 000 bytes, card 276 px vs 157 px,
the populated card byte-identical between arms), not by reading the scene graph.

*A verifier refuted the owner's own ruling being met.* Rule 1's conversion made
`_tool_section()` answer with exactly one id, and its `match` reached
paint/territory/label/icon **before** the draft clause — so arming any of those
four took Commit, Discard, Undo and Redo away from an uncommitted sculpt draft,
and Paint drew *its* Commit in the same slot. That is "nothing is yanked away"
breaking where it matters most. Fixed and measured: all four now read
`stack=true`. A second, distinct gap the same probe exposed is filed above.

**2026-09-03, nineteenth batch — 101 → 97.** Four rows close: the clipboard model
and all four Edit commands (step one had closed 10/10 a batch earlier); the APK
probe-scene exclusion; the coordinate-units probe; and §2.1's last delivery gap.
`cargo test --workspace` holds at 3 015, 0 failed.

*A fourth consecutive batch found a row describing work that was already built.*
DS-03's resolver — `DccTheme.ROLE`, `role_px()`, `is_tablet()` — has existed since
2026-08-31 with **87 live call sites across eight shell files**, and `is_tablet()`
already avoids the predicate `GUI_GAP_REGISTER.md` §57 refuted. The row called it
unstarted for three days. The lane guarded it instead (7/7 mutants killed) and
found a real defect while doing so: `ROLE["h_rail_head"]` read `[29, 34]` against
both the canvas and the shipped shell, now `[30, 44]`. **The reflow half is what
remains**, and it is blocked on nothing.

*The owner's scoped `export_presets.cfg` authorisation was needed and was
exercised* — exactly one line, only the `exclude_filter` key of the Android
preset: `"addons/godotsteam/*,addons/godot_ai/*"` gains `,_*`. Measured rather
than assumed: an export pack now stores 147 files, **0** of them beginning
`res://_`. (The main loop reported mid-batch that `_*` was already present. That
was a **torn read** of a file the lane was concurrently editing — see MISTAKES.md.)

*The main loop edited this file while a verifier was running* — the APK row below
— and the verifier caught the mid-run change as an unexplained diff. Recorded in
MISTAKES.md: a doc the main loop owns is still part of the verifier's baseline.

**2026-09-03, eighteenth batch — 101 → 100.** **Urban morphology is finished.**
Milestones 16 and 17 both closed, and **neither was open**: 16 shipped in
`cff1edc` a day before the row said it "remains … blocked by definition", and
17's stated blocker — "settlements carry no `specialisation` and no `traits`" —
was falsified **six minutes after it was written** and stood for eleven days. One
genuine gap surfaced, was filed in §2.1, and closed the next batch
(`settlement_layout` → `_with`).
Lane C corrected nine false shell claims. `cargo test --workspace` **3 010 →
3 015**, 0 failed.

*The main loop got this batch's premise wrong and the lane caught it.* The brief
asserted "crates/cartalith-urban has no `tests/` directory at all, so the
whole-subsystem golden genuinely does not exist" — the premise was true and the
inference false: this crate puts fixtures at `src/<module>/tests/golden.rs` by
convention, and the milestone-16 golden is 3 139 lines of it. **Absence of a
directory is not absence of a test.** The lane re-derived the golden from the
frozen reference anyway and proved it byte-identical, which is why the wrong
premise cost nothing.

**2026-09-03, seventeenth batch — 103 → 101.** Three §2.5 rendering rows closed
(geology microtexture / dune ripples; sky-view-factor and cast-shadow; SDF coast,
river and biome tinting), one new small row filed for the leg that is genuinely
unbuilt (the vector river overlay). `cargo test --workspace` **2 992 → 3 010**,
0 failed, 25 ignored; byte-identical at the default, no golden re-baseline.

*This batch is the clearest evidence yet for the preflight table's first row.
**Two of the three closed rows named a blocker that was false, and the renderer's
own module doc was the source of it** — each row cited `render.rs`'s "deliberately
excludes" list as evidence, in the file that had already implemented them. A third
lane re-opened six audit rows at their symbols and found all six still open, which
is the same discipline returning the opposite answer: re-opening is not a formality
that always closes something.*

*The audit lane also found **three new false claims of the most expensive kind** —
prose asserting a whole Rust module does not exist. `world_workspace.gd:159` said
Köppen classification is "not ported" (`cartalith-climate/src/koppen.rs` is
golden-tested and drives a live layer); `performance_window.gd:140` said no
per-device GPU enumeration exists (`multi.rs:378`); `civilization_workspace.gd:5405`
said cartalith-civ has no faction relation (`relations.rs` exists to create that
edge, and a surface 330 lines above the note already draws it). `git log -S` dates
all three as false for **fourteen to sixteen days**. They sit in panels that
otherwise work, so no disabled-control sweep reaches them — see MISTAKES.md.*

*Two of this document's own prior claims were retracted by the same lane: a
"CORRECTION" that asserted the opposite of the source comment whose line range it
cited, and a provenance exoneration refuted by `git log -S` on the same note.*

**2026-09-03, sixteenth batch — held at 103.** PH-15, the navpad hover tint and
the label-clipping residue all closed; PH-16 closed **in the state the panel
owns** and re-filed for the state it does not; two rows added.

*The lane's most valuable move was one nobody asked for: **it added a control
state.** The register had measured exactly one — planner open, no world — and read
the result as this panel's defect. Measured against planner *closed*, opening the
planner **removes 447 blank rows** (1 494 → 1 047). So most of that band is the
app with no world loaded, and filling it would have been decoration over a world
that does not exist. A number with nothing to compare against cannot say whose
defect it is.*

*The band the panel does own was real: `_RouteMapView._draw()` returned at
`pts.size() < 2` while `map_texture` already held the world render, and the
comment beside it asserted there was no texture to show — both halves false.
With a world and no route, **253 rows → 98**.*

*A defect the register never caught: `_route_map_wrap` laid out **1 437 px wide
on a 1 080 px screen**, because a `ScrollContainer` with an axis DISABLED folds
its child's minimum size into its own and the overflow propagates past
`PRESET_FULL_RECT` with no scrollbar to reveal it. Now 1 080.*

*And a self-inflicted regression it caught by re-measuring rather than assuming:
`clip_text` plus ellipsis collapses a Label's minimum width to **1**, so beside a
`SIZE_EXPAND_FILL` sibling the text vanished — removing a real line of text and
**raising** the blank-row count 1 047 → 1 072. Both are new `MISTAKES.md` rows.*

*The baseline itself had to be rebuilt first: the previous run's `blank_rows=0`
was 0 by construction, because this machine boots `mode="light"`. The probe now
forces dark **and refuses to run otherwise** — the mechanism the verification
brief demanded without supplying.*

**2026-09-03, fifteenth batch — 104 → 103.** The right dock now **appends** as the
owner ruled, and the unavailable-command row is re-cut against a measurement
rather than a memory.

*The dock lane's own probe went red on its first run and found a bug the fix
created: **the Paint section outlived its own tool**, because nothing called
`leave_paint_context()` when another tool armed inside WORLD — harmless while the
dock was a whole-panel takeover, a stale panel under a live selection once it
appends. It also caught the second-order hazard, that `armed_tool` survives a
domain switch, and gated Paint on WORLD for the reason the old code had.*

*The sharpest finding is about a **test**, not the code: `_rightdock5_probe.gd`
was green while pinning the design the ruling rejects — six checks asserted
`_context == "paint"` and friends, and a seventh was literally
`_check("...", true, ...)`, an unconditional pass. A probe can enforce the wrong
design as confidently as the right one.*

*Menu commands: the row claimed **21 unavailable of 356**; the probe measures
**374 total, 15 unavailable**. Two of the sixteen reasons were false — one a
description of what the command does, standing where its justification should be,
and `command_index.gd` reads exactly that field as the reason.*

*Both agent failures this batch were **infrastructure** — a `server_error` and a
`529 Overloaded` — not code. The phone lane was relaunched on a stronger model
and resumes rather than restarts; its predecessor's output was parse-clean and
test-green in the tree.*

*Four defects in the verification brief, one of them serious: it demanded a
dark-theme pixel count while naming **no mechanism for getting dark**, and this
machine boots light — reproducing, inside a brief that cites the rule, the exact
trap that rule exists to prevent. `MISTAKES.md` carries it as **citing a rule in
a brief is not satisfying it**.*

**2026-09-03, fourteenth batch — 103 → 104, and the verification was the batch's
real output.** Shell stages 5 and 6 landed what the design supplies; the count
went **up** because measuring properly turned two "done" claims into open rows.

*Stage 5's lane behaved well where it mattered: it **declined** `mapCursor`,
`layersBtnBg/Col` and the tool-options bar because §0 lists those bindings as
absent from the delivered prototype, and confirmed `statusMid` prints no invented
number. All 19 of §0's missing bindings were found in the re-export.*

*Five refutations, four fixed here. **The right dock replaces rather than
appends** — measured in a booted app, `settlement name SURVIVED=false` on arming
a tool — which is the naive merge the owner's ruling explicitly rejects, and a
lane had signed it off as satisfied. Now its own row. **The scale bar lost AA**:
`_chrome()` moved it from `text_faint` to `text_dim` while giving it no scrim, so
its background is the map — 3.14:1 over a white map on dark, 4.11:1 over a black
map on light, both from above the line to below it. The nine ratios the lane did
compute were all correct and none of them was this pair. **`vpContext` appended
`EDITED`/`RESOLVED` to every domain** where `ENV:1889` gives the verdict to WORLD
alone. And a **false rationale had shipped into source**: "leaving the map live
… buys back the one thing turning it off broke" — flipping that flag changes 0
of 288 000 pixels, because the panel is the opaque cover.*

*PH-16's own "blank_rows=0" proof was **0 by construction**: the probe borrowed
the register's dark-theme `>23` threshold and ran it on a light capture where
every background pixel is 251. Re-run in dark: **1 069 of 2 400 rows** blank
against the register's original 1 434 — reduced, not gone. The row now carries
that number.*

*The verifier found **three defects in the verification brief**, all correct —
including `git diff 0bba2f9 HEAD` where `0bba2f9` **is** HEAD, written while
anticipating the previous batch's version of the same error. That makes four
unfalsifiable checks shipped in briefs; `MISTAKES.md` now carries the rule as
**ask what result would refute the claim, then check the instruction can produce
it**, plus three new preflight rows the verifier proposed.*

**2026-09-03, thirteenth batch — shell stages 3 and 7, and the religion screens.**
Stage 3 (menus) came back **already done**: all 29 `PopupMenu.new()` sites route
through `DccWidgets.style_popup()`, which reads the tokens, so stage 1's re-base
*was* the restyle — zero edits, `_cmdindex_probe` PASS at 374 entries unchanged.
Stage 7 restyled the nine windows, and the religion screens shipped over the
belief engine that landed the same day.

*The stage-7 lane found **four defects the 2026-08-31 token re-base had caused
and nothing had checked**: an asset-library checkerboard whose contrast fell from
(7,8,8) to (2,3,4) — in the exact pair a comment two lines above recommended — a
trait-chip hover that became a darkening where it had been a lift, a drag preview
invisible on light because `raised` and `panel` had become byte-identical, and a
verdict green left as a raw literal at **1.96:1** on the light panel. A re-base is
verified against its sources; the properties that matter are the differences
between values, and no test covers those. New `MISTAKES.md` entry.*

*It also found the plan's own numbers disagree: §2's "nine windows" and
`STATUS.md` RP-S7's nine are **not the same nine**. RP-S7's list was used.*

*The verifier found three defects in **the verification brief itself**, all
correct. Its `project.godot` check stopped being evidence the moment the main
loop committed mid-verification — a clean tree makes `git diff` empty for every
file — re-checked properly as `git diff 8382744 HEAD` (unchanged, 75 comment
lines intact). Its probe-guard check could not discriminate, because another
lane's `menus.gd` rewiring landed before the baseline was taken. And its "count
settlements shown a default religion" is always 0 by construction, since
`religion` and `adherents` are emitted together for every settlement. Two are
now `MISTAKES.md` preflight rows.*

**2026-09-03, twelfth batch — headline holds at 103.** The layer-stack UI closed
(section 7's row list in CARTO, RD-10's right-dock Layers section **appended**
rather than replacing the selection, per the owner's dock ruling, plus WCAG 2.2
SC 2.5.7 Move up / Move down beside the drag). **PH-16 narrowed rather than
closed**, and one row was added, so the count did not move.

*PH-16's cause was real and is fixed: `journey_planner_view.gd` pre-scaled six
heights by `phone_scale()` **and** let the shared `phone_fit()` walk multiply the
same subtree again, so every pre-scaled row rendered at `phone_scale()`². But a
verifier measured the panel still reporting `(1080, 2400)` — a full-screen
takeover — with `_show()` still switching the viewport off. The register's
complaint is a pixel one; the fix so far is geometric.*

*Three defects the verifier found in the layer UI, two fixed here. **Reset to
quality tier never re-synced the Layers panel**, so the engine returned to the
default order while the panel kept drawing the user's arrangement — fixed. **The
Colour relief row is live over a layer that draws nothing** at the shipped
default, because `ramp_strength` is `0.0` and `composite` skips it: disclosed in
the left dock, and left as its own row because the honest end state is a
judgement about the default, not a patch. A comment's `phone_scale` arithmetic
was two rebases stale (2.748 where the probe prints 2.621) and is corrected.*

*The verifier also refuted **the verification brief itself**: "measure PH-16 at
393×852" cannot discriminate, because `phone_scale()` is exactly 1.0 at that
size. The lane's choice of 1080×2400 was correct and the brief had called it an
evasion. `MISTAKES.md`'s orchestration entry is now ×4 and carries the rule: check
your own verification instruction is discriminating before demanding a lane
satisfy it — a test condition that cannot fail is worse than none, because it
looks like rigour.*

**2026-09-03, eleventh batch — the first under the owner's new GUI order, which
puts the §3.2 rows blocked on *other work* first.** Both preconditions are built,
so **two rows moved §3.2 → §2.2** and only their UI halves remain. The headline
holds at 103; nothing closed, two things became startable.

- **The layer stack.** CA-04's stated reason was **wrong about the pixels**: it
  said `render.rs` bakes the categories into one pass needing an architecture
  change, but both composites already existed inside `land_color` — colour relief
  as a normal-over lerp, hillshade as a multiply — with the operator and slot in
  *source* rather than in *data*. The fix was register-composited, costing no
  allocation; N buffers were measured at **368.8 MiB** at the 8192 export ceiling
  and rejected. Byte-identity at the default is by control flow, proved with **8
  FNV digests taken before the change** and unchanged after.
- **Belief.** The row claimed `cartalith-civ::belief` does not exist. It was
  already 945 lines — compatibility tables with no diffusion, no callers, no
  bridge. That is the **twelfth** row this week that did not survive being
  re-opened at its symbol.

*`export_raster.rs` needed no change, and that was structural rather than lucky —
the stack lives on `TerrainAppearance`, which all three raster consumers already
fetch. The lane proved it with a test named for last week's failure,
`every_stack_control_moves_both_consumer_paths`, whose doc says it measures
rather than assumes **because `with_ground_tiles` did not**, moved no pixel at the
default, and left the suite green while every exported PNG diverged from the map.
That is `MISTAKES.md` being applied before the mistake rather than after it.*

*The verifier found the same class a third time anyway, in the other lane: the
belief staleness key covered `belief_seed`'s second argument and not its first,
so reassigning a settlement to a faction of another faith left it showing the old
religion while the guard reported itself current — **and that was the fix for the
identical miss on the religion column**. The key is now derived from the
function's signature, `culture` is documented as deliberately uncovered with the
reason, and `MISTAKES.md`'s entry is widened to "covering some inputs of a thing,
not all of them ×3".*

**2026-09-03, tenth batch, taking 111 → 103** — nine rows closed, the largest
single drop yet, and most of it came from *measuring* rather than building.

- **`gpu_compute_height` is an undocumented decision, and the right one.** The
  blocker was never written down: `HEIGHT_LAYOUT` binds **9 storage buffers**
  against `REUSED_STAGE_MAX_STORAGE_BUFFERS = 8`, so there was never a device it
  could be built on. Worse, its recorded 5.17×/8.13×/4.84× speedups were against a
  *single-threaded f32 twin*; against the production `compute_height`, which is
  f64 and already `par_chunks_mut`, it wins **2.13× at 1024² and 1.15× at 2048²**
  — about 5 ms, against a handshake two orders of magnitude larger. Not wired, and
  now documented with what would overturn it.
- **The `gpu_height` throughput drop is upload-bound**, hypothesis tested and
  confirmed against a cross-kernel control: the two narrow kernels get *cheaper*
  per cell from 1024² to 2048² while only the nine-buffer bind group turns around.
- **`build_road_network` is parallelised — and has no production caller.** All six
  call sites are tests; the shipped path is `civ_road_network`, already parallel.
  Said out loud rather than filed as a speed win it is not.
- The `_peakaudit_*` probes are deleted, and the ocean lattice and hand-lettered
  glyph rows resolved.

*The verifier's finding is the one worth remembering: **the lane that closed the
benchmark-averaging row then committed that exact defect three times**, writing
single-sample figures into two doc comments and a scope document as measured fact.
None reproduced — 416 ms re-measured at 730, a "5× spread" at 1.4%, a "halving" at
1.35×. All three are now ranges or directions, and the residue is its own row.*

*Two claims were false rather than imprecise and are fixed: `render.rs`'s module
doc still listed `rockSlope` refinement and wetness darkening as **excluded** in
the same file that had just implemented them — and that doc is
`OUTSTANDING_WORK.md`'s own cited location for the row — and `STATUS.md` named
three deleted probes as "present and uncalled". A dangling
`examples/_peakaudit_peak.rs` citation in `landmark.rs` went with them.*

**2026-09-03 — four owner rulings on the GUI blockers, taking 112 → 111 and moving
four rows without closing any work.** The owner reprioritised GUI and answered the
four rows sitting in §3.1 *blocked on an owner decision and nothing else*. Full
text in `LARGE_ITEM_RULINGS.md`'s second section.

- **DS-03: keep everything, reflow only.** The tablet gets the full desktop
  inventory; nothing is removed. That retires the *content* question outright —
  there is no "which controls leave" list to build — and leaves the row as its
  architectural half alone: `DccTheme.TABLET`'s exhausted key space. → §2.2.
- **`rdExtraMode`: selection wins, the tool appends.** This answers
  `UNWIRED_FUNCTIONS.md`'s open question 1, which is deleted from §4 — the only
  reason the headline moved. → §2.2.
- **The APK probe scenes: excluded, under a scoped authorisation** to edit
  `export_presets.cfg`'s `exclude_filter` **and nothing else in that file**.
  `Cargo.toml`, `.gitignore` and `project.godot` remain off limits. → §2.2.
- **CV-24 / ED-02: both wait for a design pass.** `TIMELINE_SCOPE.md` §4's
  instruction to design the panel before guessing its region is upheld rather than
  overridden, so these move §3.1 → **§3.3**: still blocked, on a design rather than
  on the owner. Not closed and not startable.

*Owner question 3 — the WORLD left-dock A/B switch — was deliberately **not**
asked. It is doubly blocked: its captions and gate live in the truncated tail of
`02-rail-and-domains.md` §8, so there is no label to build the control with even
once the call is made, and an answer would not have been executable.*

**The owner's GUI order, standing:** the §3.2 rows blocked on other work first,
then the unblocked rows, then the rows blocked on a design that does not exist.

**2026-09-03, ninth batch, taking 115 → 112.** Four rows closed, and three of the
four were **already built** — the manual-icon tool's arming, rendering and
persistence all resolve at their symbols (`icon_arm` → `IconEditor::arm`;
`viewport_host.gd` drawing `icon_list()`; `SLOT_ICONS` round-tripping 53 icons
with per-instance scale intact), and so does **CA-05**, whose row said the icon
tool has no on-canvas resize handles while `#[func] icon_handles`
(`lib.rs:7759`) has a live caller at `cartography_workspace.gd:966`. The one
genuinely open gap was the density brush, which is built. The layer live-sync
signal and the `LIVE_LAYERS` assertion were likewise already done.

*The pack-import warning was **reclassified rather than fixed**, and the lane was
right to decline the brief's instruction: that string is pinned by golden fixtures
captured from the reference under Node, so editing it is a re-baseline
`DECISIONS.md` §7a protects. It also found the urgency lower than stated —
`PackManifest::warnings` reaches no user, so the false claim is false in code and
not on screen.*

*The verifier refuted four claims. **Four brush constants and the brush seed were
pinned by nothing** — the clamp test compared each constant against itself, the
same shape that let `MIN_REGION_WORLD_AXIS` survive `4 → 3`. Now pinned to the
reference's literal slider attributes (`#carIconBrushR` at 1656, `#carIconBrushD`
at 1657 over the `/100` at 13515) and all four die under mutation. **My own first
replacement for the seed test repeated the defect** — two fresh editors agree
whatever the seed is — and needed the literal added before the mutant died. A
`km²` readout in `world_workspace.gd` was still going through `_thousands` beside
the converted siblings, which is the half-fix `format_area`'s own doc names; and a
spliced sentence in new prose was corrected.*

*One row added: the Coordinate-system units fix is real but nothing would catch
its removal.*

**2026-09-03, eighth batch, taking 116 → 115.** **Saved measurements + CSV** ships
as the *fifth caller* of the slot path rather than a second mechanism, which the
ruling required: one line in `DOCUMENT_SLOTS` was the whole Rust change, because
the channel built last batch was shaped to take it. The document carries `gw`/`gh`
and **refuses on a grid mismatch, then clears** — staleness-marking was rejected
and the reason recorded, that a marked reading stays readable and plausible while
the points under it name different ground. The CSV is canonical km, verified by
re-exporting under mi and comparing bytes. **Pack biome/terrain decoding** landed
with the default output unchanged.

*Two verifier refutations, both real, both fixed here. The measurements **reader**
had a data-loss defect the write side hid: `float(<null>)` is a GDScript runtime
error rather than a conversion, so one null value aborted the whole reader, the
caller took the `ok == false` branch and cleared the in-memory list, and the user
lost every healthy reading beside it — silently, with no reason line. This build
writes that null itself for a NaN. Guarded, and the probe now proves it: reverting
the guard turns "3 of 3 survive" into "0 of 3". And pack decoding wired
`with_ground_tiles` into the on-screen builder only, so with a pack applied and
cells painted the map blended the pack tile while **every exported PNG blended the
flat swatch** — a new divergence, and one the reference does not have
(`landColorCore` reads the same `assetPack` global at 8168, 11730 and 11969).
Attaching it moves no pixel at the default, which is exactly why no golden caught
it.*

*One row added: the pack-import warning still names biomes and terrains as unused
sections, about two families the map now composites.*

**2026-09-03, seventh batch, taking 120 → 116 — and the duplicate is finally gone.**
Four rows closed. The **vault leak**: `import_heightmap` called `absorb` without
`release_world`, so links and snapshots taken against the old world survived into
the imported one. The clear moved **into `absorb`** rather than to the call site —
`absorb` is the funnel all four generated-world paths share and the function that
replaces `self.civ` wholesale, which is what makes every `entity_id` in the store
meaningless; a call-site fix leaves the fifth path free to repeat it. Proven by
revert: deleting the two lines fails 2 assertions, **on the import arm only**.
And the **four caller-owned save slots** now round-trip, which closes the last
duplicate-classification defect this section has carried: *Saved measurements +
CSV* was filed both as ready (§2.2) and as blocked on this work (§3.2), and the
blocker has landed, so the §3.2 half is deleted.

*The atlas row was narrowed rather than closed, on the lane's own honest finding:
the live collision is fixed and was measured real first — three different worlds
at one parameter tuple all hashed to `beffe825` — but the save format records no
provenance, so the discriminator cannot survive a reopen. Stated as a format gap
rather than worked around.*

*Two verifier refutations, both the "a fix nothing asserts is not a fix" class,
both closed here. `WorldGen::carried_foreign` was pinned by nothing — replacing
`std::mem::take` with `Default::default()` left the whole Rust suite green, and no
unit test can reach it because `WorldGen` is a cdylib `GodotClass`; `_savetree_probe.gd`
now drives a real open→re-save with a non-UTF-8 foreign payload and the mutant dies
("foreign entry was DROPPED by the re-save"). And `is_own_entry` has **seven**
branches where its fixture reached four — `params.json`, `README.md` and
`preview.png` each survived the entire workspace suite. Now table-driven over all
seven plus eleven near misses; all three mutants die.*

**2026-09-03, sixth batch, taking 119 → 120 — the count went UP, and that is the
batch's most useful output.** Two rows closed: **Units** (the Measure panel, the
Region-select extent, and — after a verifier caught the same function half
converted — the `radius`, `section` and `area` arms beside them, the last needing
a new `DccUnits.format_area`, since 100 km² is 38.6 mi² and not 62.1) and
**Region ▸ New world from selection** (`region_as_new_world` plus the `WorldGen`
state work the ruling insisted must not be folded into GUI work).

*Three rows were added because the region lane enumerated `WorldGen`'s fields
against `absorb` and `close_world()` instead of trusting the five the ruling
named, and found two pre-existing defects it correctly escalated rather than
silently repairing: `import_heightmap` leaks vault links and snapshots across a
world replacement, and the atlas `world_key` hashes parameters but never how the
field was produced, so a resampled, an imported and a generated world can collide
in one namespace. A verifier added the third. **A backlog that only ever shrinks
is not being read carefully**, and this is what looking properly costs.*

*The Layers row was narrowed rather than closed on the same evidence: the lane
claimed a build-time read covers a later cross-panel click, but `_register_workspaces()`
builds all five workspaces eagerly at launch, so CARTO's checkboxes exist before any
click can happen. And `MIN_REGION_WORLD_AXIS = 4` survived mutation at `4 → 3`,
because its test compared the constant against itself — now pinned to
`generate_sized`'s own `grid_w.max(4)`, and the mutant dies.*

**2026-09-03, fifth pass — two rows closed by re-reading, not by building, taking
121 → 119.** Both said "Built" and stayed open on a residue that later waves had
already fixed, so the *rows* were stale rather than the work. **Civilisation
authoring**: the `CivRebuild::Routes` unconditional tail is gated —
`civ_settle_staleness` clears `civ_dirty` only when `civ_merge` reports the layer
really was re-derived, with a test per mode. **The river entity**: the third
`f64::hypot` at `enforce_channel_descent` is `js_hypot`, the doc citation is
corrected to reference 4532-4537 *with the correction shown*, and all four
surviving mutants now have pinning tests — including
`enforce_channel_descent_carves_the_v8_hypot_disc`, which asserts the call site
the earlier divergence test only measured.

*This is the fourth time this file has carried a row whose blocker had already
lifted. Re-open a "Built … but" row at its cited symbol before scheduling work
against it.*

**2026-09-03, fourth batch, taking 123 → 121.** **Colour management** ships behind an
sRGB default that is byte-identical *by control flow*, not by arithmetic —
`ColorSpace::Srgb => return` fires before a single byte is read, so no matrix
constant, transfer function or rounding rule can move the shipped image. Proven
twice rather than asserted: an FNV hash of the finished render captured **before**
the feature was written and re-run unchanged after, and a real 2048×1312 round
trip measuring `0.000000 %` of bytes moved. The owner's stated cost was therefore
never paid — no golden was re-baselined. **Rebindable shortcuts** ship per-context
with same-context conflicts surfaced, cross-context collisions correctly *not*
flagged, live reapplication and a reset path.

*Two defects the verifier found, both fixed here. `File ▸ Close project` runs
`close_world()`, which does `world_gen = WorldGen.new()` and re-inits
`color_space` to sRGB — so the picker was left reading Display P3 over an sRGB
engine. The comment above that control asserted the opposite ("opening a project
therefore cannot leave this row stale, because nothing underneath it moved"):
nothing in the **document** moves, which is true and is not the question, because
the **engine** moved. And `menus.gd`'s Colour management `_todo` still read "the
renderer is sRGB-only end to end", false in every clause within the same session
— now a signpost to the real control rather than a `_todo`, which would have made
`command_index.gd` count a shipped feature as missing.*

*Both lanes also mis-attributed their test-count deltas, in opposite directions,
while both absolute figures were right. Noted because the absolute number is the
one this project checks; the deltas were never load-bearing.*

**Earlier the same day, taking 124 → 123.** The two stranded funnel chips are
wired (*Show rejected* draws the real capped list and says so in its label;
*Raise crowding to × N* computes the figure, snaps it to the dial's step, and
refuses off-dial answers by printing them rather than clamping), and **Report an
issue** is now a local diagnostic dump with no endpoint, as ruled.

*The chip lane found something larger than its own task: **the crowding
direction was inverted in three places** in `civilization_workspace.gd`. The
engine divides — `LandmarkSettings::radius_km` is `base / crowding_in_force()`,
pinned by `landmark.rs::crowding_higher_packs_tighter` — while the panel
multiplied. The two agreed at the default × 1.00 and nowhere else, so at × 2.00
the panel read "keeps 68 km clear" where the pass kept 17, and the note printed
directly above the chips told the user to move the dial the wrong way.*

*The verifier then refuted two claims, both fixed here rather than deferred. A
replacement comment asserted "neither chip is ever enabled onto a no-op";
measured, the one kind that reached `ok` placed its promised candidate exactly
as computed and that candidate's own new ring rejected another, for a net gain
of zero — the control's "floor" wording was honest, the comment's was not. And
the diagnostic report printed `VRAM budget: 0.0 GB` where `0` is the sentinel
for **no cap** and is the shipping default, so every stock-install report would
have told its reader the GPU path was budget-refused. **That is the fourth
instance in two days of encoding "no value" as a plausible value**, after
`needs_crowding`, `harbour_scale` and `wall_spec`.*

**Earlier the same day, taking 126 → 124.** Three rows closed, each
verified independently rather than on its lane's word: **CPU worker threads**
(the setter that returned `true` while changing nothing now returns the truth —
root cause was `ACTIVE_THREADS` never being written by Rayon's *implicit* global
init, so the honesty check was reading a counter only the explicit path touched;
the settings value is wired and restored first in `_ready()`, and `menus.gd`'s
`_todo` is a real menu), **Settlement diagnostics** (0 bare zeros over a real
world, 20 fields dashed with their reason, and an audit that found **five more**
fields defaulting a value and printing it as though measured), and the
**Landmark funnel** (both halves, with rejection *reasons* rather than bare
coordinates). One row replaced them: the two funnel chips left stranded on
reasons that the funnel work itself made false.

*A boundary defect was fixed here too, and it is the same defect three times in
one day: `rejects.rs` marshalled `Option<f64>::None` as `0.0`, and `0.0` is a
plausible Crowding, so 44 of 614 spacing rows read as genuine measurements. The
key is now absent and callers use `has()` — the idiom the diagnostics card
settled on for `harbour_scale` and `wall_spec` the same day.*

**Earlier the same day, taking 128 → 126.** Two rows deleted, both verified 10/10 by an
independent adversarial pass rather than on their lane's word: **CARTO ▸ Icons**
(§2.2 — a sea-marks asset family, a generated placement pass, and the coastline
snap test the ruling names, reusing the label culler rather than growing a
second one) and **Label collision culling** (§2.2 — now genuinely wired, see
below). Three more rows were **narrowed, not closed, because their verifiers
refuted them**: CPU worker threads (a setter that returns `true` while changing
nothing), Settlement diagnostics (19 of 203 cards print an undashed `order 0`,
breaking the owner's one binding condition), and Cut · Copy · Paste (step one
only, as the ruling sequences it).

*That wave also caught a regression this file's own headline had no way to see:
commit `0f0fe55` used `_label_cull` twice in `cartography_workspace.gd` and
declared it nowhere, so `class_name CartographyWorkspace` failed to register and
`shell/app.gd` — the application root — would not compile. **The Godot shell was
unbootable while `cargo test` reported 2 821 green.** Fixed, and both files now
parse clean. A separate latent hazard was closed at the same time: the label
comparator used `partial_cmp(..).unwrap_or(Equal)`, which is intransitive on a
NaN weight and makes Rust's sort panic — a panic that crosses the gdext boundary
takes the whole Godot process down. Now `total_cmp`, pinned by a test that
panics if the line is reverted.*

An earlier five-lane wave closed five rows on 2026-09-02 and
they are deleted: CARTO ▸ Labels, all three steps (§2.2, built and verified
against the code); the reference re-freeze to v2.11 (§2.8, done — `reference/`
now holds v2.11 and a regenerated `FUNCTION_INDEX_v2.11.md`); and
`_civPlaceSmelting`, `_civSaltAccess`, and `_civFactionAggregates`'s resource-
and density-fed half (all three §2.3, all found already built). One row was
added — whether the committed v2.11 is `Cartalith_RC`'s actual live head,
unresolved and unverifiable from this machine (§3.3). **A sixth row went in the
same pass**: *Label collision culling*'s §3.2 entry, one half of a
duplicate-classification defect this section has carried for days. It was filed
as blocked on the labelling pass; that pass landed in this wave, so the row was
not merely misclassified any more, it was false. The §2.2 half stands. Net,
taking 133 → **128**.
*Earlier the same day, a separate pass closed three rows taking 136 → 133: the
urban **17a caveat** (§2.1, which is now empty), and
`TERRAIN_APPEARANCE_SCOPE.md` **§16** and **§19** (§2.5).*

| | Count | Meaning |
|---|---:|---|
| In flight | 3 | Code exists, committed but partial (§1) |
| Ready to start | 54 | Nothing blocks them; someone has to pick them up (§2) |
| Blocked | 28 | A named blocker, listed in §3 |
| Open decisions | 18 | Not work yet — the owner owes an answer first (§4) |
| Declined / shelved | 25 entries | §5, kept so nobody re-proposes them |

Of the 30 blocked, **10 are blocked on an owner decision and nothing else** —
still the largest single category of stalled work, and §4 remains the shortest
path to unsticking it. That 10 is checkable and checks out: §3.1 holds exactly 10
rows. *It was 14 until 2026-09-03, when four GUI blockers were put to the owner
and answered — three became startable and one moved to §3.3, blocked on a design
rather than on a decision.*

**Every count above was re-derived by counting table rows mechanically,
2026-09-02,** and the per-section figures are: §1 **3**; §2 0+16+5+6+11+22+9+3 =
**72**; §3 14+10+11 = **35**; §4 **19**. *The previous version of this paragraph
gave §2 as 98 and §3 as 33 while the table beside it said 80 and 34 — the
document reproducing, in its own count section, the exact defect §6.8 exists to
record. Both are now derived by the same script that produced the table, so they
cannot disagree.*

**Nothing is now listed twice.** This section carried a classification defect for
days — a row cannot be both ready and blocked, and two were: *Saved measurements +
CSV* and *Label collision culling* each appeared in §2.2 **and** §3.2. Both are
resolved rather than reclassified, and in the same way: their blockers shipped, so
the §3.2 halves became false rather than merely misfiled. Culling went on
2026-09-03 when the labelling pass landed; measurements went the same day when the
four caller-owned save slots it was waiting on began to round-trip. The unique
count and the headline are therefore the same number, **103**, for the first time
since this file was written.

Four caveats on that number, stated rather than buried:

1. **It counts rows, not effort.** Urban milestone 10 is one row and ~407
   reference lines; "delete three probe files" is also one row. Sizes are on
   every row for this reason. The **85** rows that carry a size (everything
   except §4's 18 decisions — and 103 − 18 = 85, so the split is checkable
   against the headline) divide **20 large, 41 medium, 24 small**, re-derived
   2026-09-02 by the same script that counts the rows. *This caveat has now been
   overstated twice: it read "142 rows, 42/56/44" until 2026-09-01 and "134 rows,
   40/54/40" until today, both times because the sizes were counted by hand
   separately from the rows.*
2. **The `UNWIRED_FUNCTIONS.md` backlog is one row of the 3 "in flight" above,
   not many** — that document is itself a live backlog with a `file:line` per
   row, and re-counting it here would guarantee the two drift (this
   corrects an earlier version of this caveat, which pointed at "the 106
   ready" — the row has only ever lived in §1). It carries **21** open rows
   as of the 2026-09-01 third pass (22 after the second pass, 23 after the
   morning re-cut, 75 before it), **re-verified unchanged at 21 by a second
   full re-cut on 2026-09-02**. Counted individually the true total is
   **123** — this one row swapped for its 21 (103 − 1 + 21). (The figures here
   were "177, not 155" until 2026-09-01, "173" until 2026-09-02, and "153"
   against the 133 headline earlier the same day; each was arithmetic against a
   headline that has since moved, which is why the working is shown.)
3. **Six surveyors returned 487 rows; roughly 300 were `done` or `declined`,**
   and the rest deduplicated heavily — the urban milestones, the landmark
   viewshed and the vault's §26 each arrived from two or three surveys
   independently. The compression is real, not a sampling gap.
4. **Nobody ran the test suite.** "Done" for `UNIFIED_TOOL_PLAN.md` milestones
   A–E means the named crate modules and bridges exist and the commit reported
   green, not that `cargo test` passed this pass. §7 says what else is
   uncovered.
## The three that matter

If you stop reading here:

1. ~~**Urban morphology milestone 16**~~ — **closed 2026-09-03. Urban morphology
   has nothing outstanding.** Milestones 8-15 shipped in `4ec07f5`; the three
   `_um*` adapters and their wiring landed in `cff1edc`; and **milestone 16
   itself shipped in `cff1edc` too** — `generate.rs`, `generate/tests.rs`,
   `generate/tests/golden.rs` and `tools/um_capture.js` all enter the tree in
   that commit (`git log --diff-filter=A`). This entry claimed 16 "remains …
   blocked by definition" for a day after it had already shipped. The golden was
   independently re-derived: `node tools/um_capture.js` reproduces
   `generate/tests/golden.rs` **byte-identically** (md5
   `cf6487380773a5e13c1fdf2c5d54ff94`, 29 cases) from the frozen reference, and
   12 of the 13 stage modules are proven mutation-covered by
   `whole_subsystem_matches_reference`. **Measured limit, not a defect:**
   `hash_model` hashes five loops (edges, nodes, blocks, parcels, buildings) and
   none is an amenity, so amenity *placement* is covered by count and presence
   but not position — `MARGIN 25.0 → 200.0` survives. `rules.rs` is the one
   stage module with no mutation coverage.
2. **The GUI/shell replacement, stages 3, 5, 6 and 7** — `00-REPLACEMENT-PLAN.md`
   still opens with a truncated-prototype blocker that was resolved the same
   day (`BUILD_ANSWERS.md` §1). Stages 1, 2 and (as of 2026-09-01, second
   pass) 4 have landed; **stages 3, 5, 6 and 7 are unblocked and unstarted**,
   and anyone reading only the plan will believe stage 5 is still blocked.
3. **The project record itself** — largely **actioned on 2026-08-31**:
   `CHANGELOG.md` is now retired (frozen and marked, not backfilled — the 51
   commits since `bcabd5a` stay in `git log`), and `STATUS.md` was rewritten
   from scratch against the working tree. What §6 records below is therefore
   history plus whatever has not yet been swept; re-verify a §6 row against the
   file it names before acting on it.
   **The "commit the two untracked documents" item this entry used to lead
   with is done** (corrected 2026-09-01): `LARGE_ITEM_RULINGS.md` and
   `cartalith-native/docs/3D_TERRAIN_RENDER_RESEARCH.md` are both in `HEAD` —
   `git cat-file -e HEAD:<path>` succeeds for each, and so does the same check
   on this file. They landed in `fd9de7c`, along with 235 others. Nothing on
   this list is lost by a clean checkout any more; what remains here is
   whatever §6 records that a later pass has not yet swept.

---

## 1. In flight right now

Code exists for these; they are neither startable nor blocked, they are
half-landed. Each needs finishing. (The "and, in most cases, committing" this
sentence carried until 2026-09-01 is gone — see the correction below.)

**2026-09-01: eight agents worked this section's original eight rows in
parallel; each claim was independently re-verified against the code (compiled,
tested, or parse-checked — not re-read from the report) before being recorded
here.** Four rows closed outright — Milestone F's closeout, the `statusMid`
composite, Vault §14 Compare, and route corridors/travel cost as an analysis
field — and were deleted, their status moved to `STATUS.md`. The other four
were rewritten to describe what actually remained, which in three cases was
substantially narrower than what they said that morning.

**Same-day second pass: three more agents dispatched on three of those four
residuals, independently re-verified against the code (not the reports)
before being recorded here** — `cargo test -p cartalith-spatial --lib`
148/148, `--test golden_parity_paint` 7/7, `cargo test -p cartalith-godot
--lib` 409/409 and `cargo test -p cartalith-civ --lib` 513/513 all re-run
clean after a fresh `cargo build -p cartalith-godot` (the dll was stale
against exactly the files this pass touched), `cargo check --workspace`
clean, every touched `.gd` file `--headless --check-only` clean, and both
`_railfold_probe.tscn` and `_deadwire_probe.tscn` re-run passing. **GUI
replacement stage 4 closed outright** and is deleted below, its status moved
to `STATUS.md` (RP-S4). **Economy milestone 2 narrowed further** — Godot
wiring is now real; what remains is rewritten below. **Paint brush falloff**
was not its own §1 row (it lived inside the `UNWIRED_FUNCTIONS.md` backlog
row and, individually, in §2.2) but closes outright too; both are updated
below.

**Correction (2026-09-01, later the same day): "nothing in this document is
committed" is no longer true, and every "still uncommitted" qualifier below
is history.** Commit `fd9de7c` — *"Three rounds finishing in-flight work,
then two bugs found by hand"* — landed **237 files, 90 718 insertions**,
which is all three of the passes described above plus the documents they
wrote. `git status --short` now shows one modified tracked file (an
unrelated in-flight `journey_planner_view.gd` change) and two untracked
probe scenes. The re-verification those qualifiers asked for once the tree
committed is therefore **owed now**, not later — that is the live half of
the claim, and it is what §7's "the uncommitted working tree" bullet has
been reduced to.

| Item | Owns it | Size | Where it stands / next step |
|---|---|---|---|
| **The `UNWIRED_FUNCTIONS.md` backlog** — 21 open rows (1 small · 3 medium · 17 large), down from 75, plus **2** dangerous-class entries (1 real, 1 documented non-defect kept for contrast) | `UNWIRED_FUNCTIONS.md` | large | **Re-cut again 2026-09-02** — all 21 rows re-opened at their cited symbol and independently re-verified against `cff1edc`; **0 closed, the count holds at 21**, which is itself the finding. One **new dangerous-class entry**: the Settlement diagnostics overlay's tooltip is now a *false reason* — it disables the control citing a blocker that no longer holds, the class `tools/audit_wiring.py` structurally cannot see because the binding *is* called and it is the prose that lies. The right-dock "follow the armed tool" Medium row narrowed by four newly-landed tool-driven contexts. Re-cut from scratch 2026-09-01 morning, not patched: all 75 previously-open rows re-opened at their cited symbol and independently re-verified; 52 closed that pass (17 of 17 trivial, 24 of 25 small, 13 of 17 medium). **Same-day second pass**: one more of the 18 Large rows — Paint brush falloff, the row the morning cut named highest-severity — independently re-verified as built and closed, taking Large to 17 open (of `LARGE_ITEM_RULINGS.md`'s eighteen 2026-08-31 **build** rulings, tracked individually in §2.2 below) and the dangerous class from 3 entries to 1 (the 2 genuinely-dangerous Paint entries close; 1 documented non-defect remains, kept for contrast). **Same-day third pass**: one more Medium row — "Manual road tool / `road_edges` never retained" — independently re-verified as already false (`CivData::road_edges` genuinely retains `civ_hierarchical_network_topology`'s output) and closed alongside the wider journey/route cluster in §2.3. **Committed in `fd9de7c`** (this row said "still uncommitted" until 2026-09-01); the re-verification of every closed row that the commit was the precondition for is now due |
| **Landmark M8 residual** — 30 of 50 declared kinds still ship `buildable:false` (was 35) | `LANDMARK_GENERATION_SCOPE.md` | large | **Twenty** generate today (`landmark.rs::kinds()`, each unbuilt kind carrying a `not_built:` reason). *Two counting errors in this row are corrected 2026-09-02 against the code, not against the previous report: the denominator is **50**, never 49 (`grep -c "KindSpec {"`), and the buildable count at `HEAD` was **15**, so "fourteen" was low by one.* **2026-09-02: the five way-graph kinds landed** — `market_site`, `trade_depot`, `caravan_station`, `bridge_site`, `road_junction` — taking buildable 15 → 20. The `LandmarkInputs::ways` thread a prior lane left dead is now read end to end, from the `WayGrid` through five detectors to the gdext caller; each kind was verified placing on a real `generate_terrain` world rather than by flipping its flag. `JUNCTION_MIN_WAYS` was corrected 3 → 2 in the same pass, its inherited rationale shown false. `resource_extraction_site` went buildable 2026-09-01 — it reads the three resource-potential fields (`timber`, `sulfur`, `alum`) that Mine and Quarry's own resource lists don't, through their identical already-validated detector, so it claims no cell either of them already does. The other 30 reasons were individually re-verified against the code this pass, not just re-read; six were rewritten for precision (`volcanic_feature`, `rock_formation`, `glacial_feature`, `salt_works`, `ruin`, `abandoned_settlement`) with no change to their blocked conclusion. Six still need M7's viewshed; several need §13's route load; the military family is downstream of Fort |
| **Economy milestone 2** — the food-surplus cluster | `ECONOMY_SCOPE.md` | small | **Crate-complete and Godot-wired as of 2026-09-01 second pass; the remaining gap is a UI surface, not a binding or a port.** `civ_trade_bridge.rs`'s `food_shed_rows()` builds one shared `RoadComponents`, resolves each settlement's `farmers_per_urbanite` via the `civ_ag_tech_by_key` route the manpower model already uses, and calls `civ_food_shed` once per settlement; the `#[func] civ_food_shed` reads it out; `engine_bridge.gd` and `trade_store.gd` (caching alongside `civ_trade_flows`) complete the chain, triggered by the existing "Match trade flows" button — no new UI entry point was needed for the data to compute and cache. **What remains:** no dock or window calls `TradeStore.food_shed_for(index)` — confirmed by direct search, `place_editor_window.gd:385` still reads only `navigability`; the natural landing spot is right beside it, in the Trade tab. Two small residuals disclosed but not fixed this pass: `food_shed_rows()` recomputes `lithology`/`soil` per call rather than reading a `CivData` field (an efficiency nicety for whoever next touches `lib.rs`'s `compute_civilisation`, not a correctness gap); and a stale self-claim in the crate's own docs — **half of that second residual is now closed** (2026-09-01): `roster.rs`'s module doc did assert *"nobody at the `cartalith-godot` boundary calls `civ_food_shed`"* and has been corrected against `civ_trade_bridge.rs::food_shed_rows`, while the `trade.rs` half was re-checked line by line and **no such claim is there** — the citation was wrong, not the file |

---

## 2. Committed and scheduled, not started

Nothing blocks these. They are ordered largest-first within each group, and the
groups are ordered by how much of the remaining project they represent.

### 2.1 Urban morphology — what remains

Phase 5. Milestones 8-15 are **built and committed** in `4ec07f5`; **milestone 16
shipped in `cff1edc`** and milestone 17's five `_um*` are all built and golden-
covered (both verified 2026-09-03, batch 18 — see §3.2's closure note). **Every
urban milestone is now built, and the last delivery gap closed 2026-09-03**
(batch 19): `urban_layouts` now calls `settlement_layout_with`, so a per-settlement
wall/age override reaches the layout. Proven by delivery, not plumbing — an
independent probe measured `umWalls=off` taking the wall ring from 41 points to
absent, edges 1 060 → 1 336 and parcels 3 296 → 3 850, with `auto` restoring a
byte-identical signature. **This section is empty.**

**The section's earlier row closed as *wrong* rather than merely stale.** The 17a caveat — golden-verify the block-2 `_um*`
adapter — recorded its blocker as *"needs a block-2 capture harness that can run
`_um*` inside the host's full civ scope; the existing harness slices block 4
only"*. That premise is disproved by a running counter-example:
`cartalith-native/tools/um_block2_capture.js` drives the unmodified reference
under Node and `crates/cartalith-civ/tests/golden_parity_urban_adapter.rs` now
holds the extracted fixtures. Mutation matrix **22/22 killed**, and an
independent verifier confirmed the fixtures are genuinely reference-extracted
rather than replayed from the Rust port.

**The pass found two real port bugs the 11 synthetic-field unit tests could not
see** — which is the argument for golden-verification, stated concretely:

- `slope_at` used `f64::hypot` where the reference uses `Math.hypot`. This is
  the exact V8-libm divergence `CLAUDE.md` warns about and `geom::js_hypot`
  exists for.
- `um_site_profile` clamped the resource-context centre where the reference does
  not.

A third defect was in the fixture itself (`background_pots` never mirrored the
harness's one iron deposit), caught before it could be committed as truth.

*The "built and uncommitted — `git log 4ec07f5..HEAD` returns nothing" note this
section carried is history: `cff1edc` landed the downstream wiring.*

> **Before executing any ruling that says "add `cartalith-urban` as a dependency
> of `cartalith-godot`": the substance is already done.** `urban_bridge.rs`
> ("the first consumer `cartalith-urban` has ever had") reaches the crate
> through `cartalith_civ::urban_adapter`, which is the layering
> `cartalith-civ/Cargo.toml:18-22` explicitly defends. The "zero consumers"
> sentence in `URBAN_MORPHOLOGY_SCOPE.md:1761-1766` is a **quotation of what
> `PARITY_AUDIT.md` §3.4 found before milestone 17a**, and the same paragraph
> describes closing it. What is missing is the generator stages above, not the
> Cargo edge; adding a direct edge would buy nothing and violate the layering.

### 2.2 The owner's 2026-08-31 Large rulings

Eighteen rows, all ruled **build** on 2026-08-31 in `LARGE_ITEM_RULINGS.md`.
**Sixteen remain not started; two have closed and are deleted from the table
below.** Paint brush falloff closed 2026-09-01, second pass, verified against
the code (`paint.rs`, `paint_bridge.rs`, the two touched `.gd` files,
`DECISIONS.md` §7k; full evidence in `UNWIRED_FUNCTIONS.md`'s Large section
and `STATUS.md`). CARTO ▸ Labels closed 2026-09-02, all three steps verified
against the code: `LabelClass` (5 variants) and `label_class` on `LabelDto`
(`crates/cartalith-godot/src/project_bridge.rs`, `#[serde(default)]` so an
older archive still opens), `labels::label_candidates`/`generate_labels`
(`crates/cartalith-civ/src/labels.rs`) over five sources kept beside the
hand-placed list, and `LABEL_TYPOGRAPHY_DEFAULTS` transcribed from
`parts.js:363` (26/2.5/.28, 18/2/.20, 13/1.5/.06, 15/1.5/.14 italic,
11/1.2/.06), drawn by `map_overlay.gd::_draw_labels`. The Water class had no
entity at all; `labels::lake_features()` fills it. **One correction on the
way out**: the ruling's own "halo and tracking do not exist in the engine
today" was half true — tracking, yes; halo was not, since
`labels::arc_label_line_width` already existed, golden-pinned; what was
missing was a halo any label class could actually *set*, which
`LabelTypography::halo_px` now is. Two of the sixteen still carry costs the
build must honour, and one more is excluded pending an owner answer (§3.1).
*`LARGE_ITEM_RULINGS.md` was untracked when this section was written; it is
tracked in `HEAD` as of `fd9de7c` — see §6.1.*

| Item | Size | Note |
|---|---|---|

### 2.3 Civilisation, economy and journeys

| Item | Owns it | Size | Next step |
|---|---|---|---|
| Story planning **SP-3** — the settlement timeline strip (simulated history + authored vault events + journey passes) | `STORY_PLANNING_SCOPE.md` | large | No per-settlement history accessor in `timeline.rs`; `civilization_workspace.gd:1633` is the world-level strip, not a per-settlement one |
| Story planning **SP-4** — the conflict overlay in CIVIL, reading real manpower figures | `STORY_PLANNING_SCOPE.md` | large | Blocks landmark M9. Its attachment model is undecided (§4) |
| **CV-23** — historical territorial occupation over time | `STATUS.md` | large | Timeline work, not territory work |
| **VA-01** — the vault scan *index* (not the scan) | `STATUS.md` | medium | |
| The `wantCounts` / user-fixed-tier-count branch of `_civIterativeAutoWorld` | `PHASE2_SCOPE.md` m8 | small | Deferred at the time as "separate future work"; `cartalith-godot/src/lib.rs:911` records its absence |

### 2.4 Vault, project archive and save format

| Item | Owns it | Size | Next step |
|---|---|---|---|
| Vault **milestone 2** — the map snapshot (§21, §22) at immediate/local/regional radii | `MARKDOWN_VAULT_SCOPE.md` | medium | Its own record: "blocked on nothing — `export_raster.rs` already crops" |
| Project archive remainder — project-layer panels, the `library/` and `drafts/` slots, a `preview.png` producer, foreign-entry preservation | `STATUS.md`, `SAVEFILE_COMPAT.md` §17 | medium | Nothing draws any of it; `preview.png` has a writer and no producer; foreign entries are reported rather than preserved |
| Story planning **SP-1** — the `Journey` entity proper | `STORY_PLANNING_SCOPE.md` | medium | Half met, and the half that landed was built outside this document's plan: journeys persist as GDScript-owned state (`journey_planner_view.gd:3125` → `entities/journeys.json`). Not met: no `Journey` type in `cartalith-civ`, and the doc's own acceptance test still fails — `travel_bridge.rs:252` returns a hardcoded `0` |

### 2.5 Rendering, terrain appearance and export-adjacent

| Item | Owns it | Size | Next step |
|---|---|---|---|
| The stage-by-stage `WorldParams`-field audit against every stage-01…11 slider | `GUI_FEATURE_PARITY_SCOPE.md` | large | The document's own closing "honest size statement": the Generate pipeline's ~60-80 individual stage sliders, "none of which are individually scoped anywhere yet". No such audit document exists |
| §20 — the high-precision display pipeline | `TERRAIN_APPEARANCE_SCOPE.md` | medium | `render.rs` still composites into a `u8` RGB buffer (`apply_local_contrast(… rgb: &mut [u8] …)`, `:3646`) |
| The vector river overlay | `FUNCTIONAL_CONTRACT.md` cap. 6 | small | The one leg of the old SDF row still genuinely unbuilt: `map_overlay.gd` has no `drawRiverWays` equivalent (`grep -n river map_overlay.gd` returns only settlement-badge prose and the new faith lines) |
| Re-derive which pack sections the live map actually composites | `FUNCTIONAL_CONTRACT.md` cap. 6 | small | **Owner ruling 2026-09-04: measure before ruling again.** For every section name the pack-import warning can emit, establish whether `pack.rs` composites it, and report the true unused set. Two premises about this warning have already failed on contact — first that biomes/terrains were unused (false), then that `trait` was the only remaining true clause (also false: `composite_map_icons`, `pack.rs:470`, draws settlement and poi). **Audit-only, closes nothing by itself**; the owner rules once it lands |
| Pack trait art is built end to end and **gated on the owner's pack-warning ruling** | `FUNCTIONAL_CONTRACT.md` cap. 6 | small | **Built 2026-09-04.** `WorldGen::civ_trait_badge_row()` resolves a whole pin's badge row in Rust — layout, variant pick and sprite rect all stay reference-side — and `map_overlay.gd::set_trait_art_resolver()` receives it; the fixture pack's art reaches the pin (115 / 186 / 89 px on three fixtures) and the no-art path is byte-identical to the committed file across a full 2 400×1 200 frame. **What remains is one guarded line** in `viewport_host.gd::refresh_settlement_traits()`. **It is deliberately not written**: installing it makes trait art reach the live map, which falsifies the pack-import warning's `trait` clause, and removing that clause moves a golden literal the owner has not authorised. The ruling releases the line |
| GeoJSON import — the parser is built, applying it needs a ruling | `FUNCTIONAL_CONTRACT.md` DM-03 | medium | **Partial 2026-09-04.** `cartalith_io::parse_geojson` refuses malformed input with an actionable reason and never panics — verified independently against a 25-case table plus a 20 000-level nest — and `WorldGen::geojson_inspect` survives hostile input across the gdext boundary (24/24 from GDScript). **What remains needs an owner decision**, and the parser stopped there rather than choosing: what *applying* a document means when an imported feature names a faction this world does not have (create it, remap by name, or import unclaimed). Also owed: the Data-manager Import route in GDScript |
| Slippy-map tile addressing (XYZ/TMS/WMTS, a zoom ladder, retina variants) | `FUNCTIONAL_CONTRACT.md` cap. 6/9 | medium | Tile *export* exists; addressing is the remainder |

**Closed from this table 2026-09-03** (batch 17, verified): *Geology microtexture /
dune ripples* and *Sky-view-factor and cast-shadow fields* — `tests/geology_micro_and_sky_fields.rs`,
8 tests; *SDF coast tinting, river bands and biome blend* — `tests/sdf_river_and_biome.rs`,
10 tests, wired into all three `with_appearance` consumers (`lib.rs:6561`, `:8045`,
`export_raster.rs:100`). Byte-identical at the default: `color_space.rs`'s
`FINISHED_RENDER_FNV1A = 0x6154_1058_49e7_10d6` is unmodified and still passes, so
no golden re-baseline was taken. **Two of the three named a blocker that was false**
— `render.rs`'s own "deliberately excludes" list was each row's cited evidence, and
it described the file that had already implemented them.

### 2.6 GPU, threading and memory

| Item | Owns it | Size | Next step |
|---|---|---|---|
| `compute_stress` gather reformulation on GPU | `GPU_LAYER_INTEGRATION_SCOPE.md` | large | Deferred at milestones 5, 6 and 9 in turn. Needs a scatter→gather rewrite plus its own float-equivalence re-verification |
| Erosion's per-cell parts (thermal, stream-power) on GPU | `GPU_LAYER_INTEGRATION_SCOPE.md` | large | Feasibility table rates it "Good"; no erosion shader among the ten `.wgsl` files |
| Phase 2 per-cell affordance fields on GPU (biome, carrying capacity, resource potentials, settlement suitability) | `GPU_LAYER_INTEGRATION_SCOPE.md` | large | "Directly comparable to climate/erosion's per-cell case" |
| Water-body priority-flood (`build_water_bodies`) on GPU | `GPU_LAYER_INTEGRATION_SCOPE.md` | large | Half tractable, half genuinely hard — the above-sea depression fill is a global priority queue, and parallel Planchon-Darboux is a research task. ~92 ms at 1024² |
| Rendering / colour synthesis on GPU (`render.rs`) | `GPU_LAYER_INTEGRATION_SCOPE.md` | medium | The feasibility table calls it "best fit, no golden-parity tension at all", and the pilot named it the natural next target. Distinct from §21's beachhead argument in §5 |
| `cartalith-godot`'s own sequential orchestration | `CPU_MULTITHREADING_SCOPE.md` | medium | Named explicitly as untouched, and as "the real ceiling left" alongside the hard-hazard functions |
| World-wrap support for the milestone 1-5 kernels (warp, heterogeneity) | `GPU_LAYER_INTEGRATION_SCOPE.md` | medium | Both stages fall back to CPU whenever `world=true` (`cartalith-engine/src/lib.rs:778`, `:894`) |
| Full `ComputeTier` capability classifier | `GPU_COMPUTE_PILOT_SCOPE.md` §4 | medium | `crates/cartalith-gpu/src` contains only `lib.rs` and `multi.rs`; grep for `ComputeTier` returns nothing |
| Performance telemetry system | `GPU_COMPUTE_PILOT_SCOPE.md` §24 | medium | Deferred until more than one workload needs monitoring; nine kernels exist now |
| GPU memory pooling across persistent fields | `GPU_COMPUTE_PILOT_SCOPE.md` §14 | medium | |
| Hardware diagnostics panel (§23) | `GPU_COMPUTE_PILOT_SCOPE.md` | medium | Partly delivered by the multi-GPU work (`performance_window.gd:78`, `menus.gd:1663`); no §23 panel as specified |
| Tiled / chunked GPU compute (§18) | `GPU_COMPUTE_PILOT_SCOPE.md` | large | Partly unblocked by the LOD pyramid. `multi.rs` ships a band split covering exactly one kernel (`gpu_warp`), 1.22-1.54× at 4096² and a loss at 2048² and below |
| Per-segment culling for one long way whose bounding box crosses the window | `MEMORY_OPTIMIZATION_SCOPE.md`, `GUI_GAP_REGISTER.md` §54 | medium | The zoom-bound overlay lever shipped (-87.5% gfx dev); this residue did not |
| Integrate `QuadTree` and `TiledField` into a real caller, or retire them | `LOD_TILING_BASE_SCOPE.md` | medium | **Two of the crate's three data structures are unconsumed** three weeks and six dependent crates later — every external reference is a doc comment, and `lod_bridge.rs:54-63` argues at length why using `QuadTree` there "would be strictly worse than not using it". `DirtyTracker` does have real callers. Also leaves the deferred `tile_size` benchmark with no workload |
| GPU device reuse across generations — **the real item, re-scoped by measurement** | medium | **Replaces "per-pipeline caching across repeated `generate_terrain` calls", whose premise was backwards.** Measured 2026-09-03: six pipeline builds total **2.60 ms** (0.24-0.71 ms each) against a device handshake of *several hundred milliseconds* — so caching pipelines targets the smaller half by roughly two orders of magnitude. The device is where the value is. **Needs an owner decision, not just work**: holding a `wgpu::Device` alive between generations changes lifetime and failure semantics around the `lost` flag, which this project has already measured losing on `forward_plus`/vulkan. *No point estimate is quoted here on purpose — see the row below* |
| GPU timing measurements are single-sample and the device is noisy | small | **Found 2026-09-03 by a verifier, and the lane that found it committed it three times.** The benchmark-averaging row was closed by taking medians of 3 — then the same lane wrote three fresh single-sample figures into two doc comments and a scope document as measured fact, and none reproduced: a cold handshake of 416 ms re-measured at **730 ms**; a 512² spread quoted as **5×** re-measured at **1.4%** (the original was contaminated by parallel `cargo test` contention); an upload bandwidth said to **halve** at 9.24 → 5.65 GiB/s re-measured at 7.67 → 5.67. All three are now stated as ranges or directions rather than points, but **the other nine pre-existing `measured_*` timing tests in `cartalith-gpu` are still single-sample.** Give them medians, and make the harness refuse to run under a parallel suite |

### 2.7 Android and on-device verification

No Android pass has run since 2026-08-25. All six items below are live.

| Item | Owns it | Size | Next step |
|---|---|---|---|
| The Android adaptive icon had **no background layer** | `ANDROID_BUILD_SCOPE.md` | small | **Owner-reported 2026-09-03 ("on the 6t the icon is a dull weird grey scale"); root-caused and fixed the same day, unverified on device.** `icons/android_adaptive_background_432.png` was an 804-byte blank — **one distinct colour, `(0,0,0,0)`, fully transparent**. An adaptive icon's background layer must be opaque; when it is empty the launcher substitutes its own neutral plate, which is the reported grey and happens under any theme. Now opaque `rgb(0,24,48)`, the dominant band of the owner's own `Cartalith icon.png` rather than an invented colour. **Second change, owner ruling same day: thicken `cartalith icon2.png` into the monochrome layer.** Measured first and the measurement changed the plan — icon2 converted straight across gives **2.4-6.4%** ink against the shipped **5.1%**, i.e. a *fainter* themed icon, because icon2 is fine line art on black. Dilated at source resolution (MaxFilter 19) then fitted: **17.05% ink**, inside the band Android's own themed icons occupy, with **100% of ink inside both the 66dp safe circle and the 72dp visible circle** so no launcher mask clips it. **Verify on the handset with the next APK** — launcher behaviour for a transparent background is launcher-dependent and cannot be checked headlessly |
| Menu design conformance — the held groups needing an owner ruling | `DESIGN_HANDOFF.md` | large | **The audit ran 2026-09-04/05 and is verified** (283 items enumerated from code, four Fable auditors + an adversarial cross-check: 125 conforms, 99 deviates, 37 no-design, 17 design-stale, 5 unreachable; cross-check `partly-unsound`, 7 of 42 verdicts refuted). **Six unambiguous fixes shipped.** What remains is **not conformance work but decisions**, grouped: (a) the **phone MORE shell** — 7 findings where the shell re-presents desktop popups and `06-phone.md` specifies bespoke screens; `phone_menu.gd` cites `docs/ANDROID_UI_SPEC.md`, **which is not in this repository**; (b) the **left-dock body structure** — 12 mode-gated blocks in the spec vs 9 categories per domain shipped; (c) three **structural moves** (Journey planner as a Data row vs a CIVIL rail node; Atlas Refine under Preferences vs the WORLD rail; the Asset-pack submenu's 4 bands vs the newest 9-row shape); (d) **Δ vertical measure** live in 2D where the canvas says 3D-only — the shell may be better, so deleting working code needs a ruling; (e) the **37 no-design surfaces**, concentrated in windows/dialogs, incl. 8 stock Godot file pickers where the canvas has a bespoke browser |
| **Menu-by-menu design-conformance audit — owner request, 2026-09-04** | `DESIGN_HANDOFF.md` | medium | **Trigger: when GUI work is done, alongside the APK build, not before.** Owner instruction: verify that **every single menu** conforms to the latest designs. **Fable 5.1 at Ultracode, minimum 2 agents.** Read the 2026-08-25 ruling first — *when two design canvases disagree the newer one wins; where none exists, derive from the DCC canvases' own vocabulary* — and note an owner decision is newer than any canvas (`Data ▸ Conversion` is still drawn and was removed by decision 2026-08-20, so the canvas is the stale party there). Enumerate menus from the code, not from the design set, or the audit can only find what the designs already list. `GUI_GAP_REGISTER.md`'s menu-naming audit is prior art. **Owner ruling 2026-09-04 on ordering and models:** the audit runs **before** the rest of the outstanding list, not after GUI work completes; the audit itself is **Fable 5.1 at Ultracode** (4 auditors + an adversarial cross-check, dispatched `w9cjb2jog`); **the fixes that follow are Opus 5 at Ultracode, with Fable 5.1 where design judgment is needed** rather than mechanical conformance |
| **Rebuild the APK and drop it on D: — owner request, 2026-09-03** | `ANDROID_BUILD_SCOPE.md` | small | **Trigger: when GUI work is done, not before.** Recipe in memory `cartalith-apk-build-and-drop`, verified: `cargo ndk -t arm64-v8a build --release -p cartalith-godot`, then `--export-release "Android"` (**expect it to fail at signing — there is no release keystore and never has been; the unsigned APK it leaves behind is good**), then sign with Godot own debug keystore via `apksigner`. **Two things must be verified, not assumed:** that the `.so` inside the APK is the one just built — a shipped APK once carried a library 25 commits stale, so features were live in the tree and dead on the handset, and Godot accepts a wrong `.gdextension` key with no error — and the md5 at the destination after copying. Destination `D:\Users\Vincent\Documents\Vincent\Persoonlijk\Writing\Tools & writing hacks\`, which already holds `Cartalith.apk` (57 021 549 bytes, 2026-09-01 14:10, sha256 `6c70b414...`). **Preserve that build rather than overwriting it blind** — date-stamp the old one aside first |
| Six features never driven on device since the 2026-08-24 USB disconnect — paint visibility, save/undo, the debug views, GeoJSON export, hand-drawn ways, civ-recompute | `ANDROID_BUILD_SCOPE.md` | medium | Recorded as *unverified on device*, not as verified. The 2026-08-25 pass drove a different list and did not pick these up |
| The phone shell with **no world open** scans 1 494 of 2 400 rows blank | medium | **Re-filed 2026-09-03 from PH-16, which attributed it to the wrong surface.** The register measured one state — planner open, no world — and read the result as the Journey Planner's. With a control state added, opening the planner **removes 447 blank rows**: closed 1 494, open 1 047. So the band belongs to the app with no world loaded, not to this panel, and there is nothing honest to draw into a world that does not exist. Whatever the empty shell should show is a design question `06-phone.md` does not answer |
| The phone inspector's widest rows demand 1 408 px on a 1 080 px screen | small | **Found 2026-09-03 while closing PH-16; the register never caught it.** Contained rather than removed: those rows now sit inside `SCROLL_MODE_AUTO` containers so they are reachable by horizontal scroll and no ancestor exceeds the screen. The row *widths* are the remaining question and they are a design one. *The container half was a real defect and is fixed — `inspector_scroll` had its horizontal axis DISABLED, and a `ScrollContainer` folds its child's minimum size into its own on a disabled axis, so 1 436 px propagated up to `_center_panel` and Godot clamped it past `PRESET_FULL_RECT`. `_route_map_wrap` 1 437 → 1 080* |
| The `vault.json` write gate has no test at its call site | `MARKDOWN_VAULT_SCOPE.md` | small | **The bug is fixed, the guard is not built.** The gate read `!store.links.is_empty()` — one member of a three-member store — so a project with a connected vault and a map snapshot but **no knowledge links** wrote no `vault.json` at all and lost the snapshot on save. Now `!store.is_empty()`. `LinkStore::is_empty()` itself is mutation-tested (2 of 3 conjuncts killed), but **the call site is not**: `project_save_with_documents` takes gdext types on a `GodotClass`, so no Rust unit test can reach it — this needs a probe scene that saves a snapshot-only project and reopens it |
| A sculpt draft that appears without a tool-arm does not rebuild the right dock | `UNWIRED_FUNCTIONS.md` | small | **Found 2026-09-03 by a verifier probe, measured pre-existing at HEAD before crediting it.** `app.arm_tool()` early-returns when the tool is already armed, so no `tool_armed` fires; nothing else signals a stamp-count change to the dock. A draft created while Inspect is already armed leaves the dock showing a body built when the count was 0 — `_tool_section()` answers `stamps` while the body reads `["SAMPLE"]`. Distinct from the append bug fixed the same day, which was about arming a *different* tool |
| The Colour relief layer row is live over a layer that draws nothing | small | **Disclosed 2026-09-03, not fixed.** `TerrainAppearance::ramp_strength` ships at `0.0` and `LayerStack::composite` skips Colour relief entirely when the ramp contributes nothing (`None => continue`), so at the shipped default that row's dot, opacity, blend and reorder are live controls over an invisible layer — a verifier measured a default-state hillshade/colour-relief swap as byte-identical. The left dock now says so in a note; **the right dock's Layers section does not**, and the honest end state is probably that the ramp gets a non-zero default or the row is folded away until it has one. A judgement, not a patch |
| The default 2048×1311 new world costs ~878 MB peak on the phone | `STATUS.md` | medium | The "no progress indication" half is stale — a staged 10-stage readout ships off `cartalith-engine::progress`. The memory cost stands |
| Prove `push_warning` reaches Android's `logcat` (a positive control) | `ANDROID_BUILD_SCOPE.md` | small | Owed by two consecutive passes; the second explicitly declined it, noting the alternative "rests on an argument, not a measurement" |
| The left-panel sheet retains its scroll offset across close/reopen and will not scroll back up | `ANDROID_BUILD_SCOPE.md` | small | Six swipe attempts at three x positions failed. Not investigated |
| Exercise **R1**'s Godot-side hunk inside a running Godot process on the handset | `MEMORY_OPTIMIZATION_SCOPE.md` | small | The case for R1 is four arguments, not a screenshot |
| Bottom-docked controls do not ride above the IME | `UNWIRED_FUNCTIONS.md` | small | Zero `get_virtual_keyboard_height` hits in `shell/`, re-verified 2026-08-31 |
| The Android debug `.so` residue — 156 MB, 207 MB APK | `STATUS.md` | small | Reduced from 400 MB; still not the 18 MB a full strip gives. See §5 for why it stays |

### 2.8 Discipline debts

Small, cheap, and each one the kind of thing that silently invalidates a later
measurement.

| Item | Owns it | Size | Next step |
|---|---|---|---|
| **15 menu commands unavailable**, each carrying a true reason | `STATUS.md` | small | **Re-cut 2026-09-03, and the previous figures were wrong in both halves** — this row claimed *21 unavailable of 356 total*; the probe measures **374 total, 15 unavailable, 15 of 15 with a reason**. All 16 were opened at their symbols and **two were false**: `Clear atlas cache now` kept a build-time sentence describing what the command *does* while disabled, and `command_index.gd` reads a disabled row's tooltip as its stated reason — so the searchable index carried a description masquerading as a justification; and `No GPU detected` was minted with raw `add_item` + `set_item_disabled`, bypassing the `_todo`/`_readout`/`_signpost` vocabulary and defaulting to `_todo` when it is an empty-list placeholder. Both fixed, taking 16 → 15. The remaining 15 are genuinely blocked — mostly on the absent 3D viewport, the clipboard step of Cut/Copy/Paste, and stage groups that expose no parameters |
| Copy in the two upstream owner notes the research briefs cross-reference (`Gravity influence.md`, `Weather Model.md`) | `PROVENANCE.md` | small | They live only in the upstream `Cartalith_RC` / `Cartalith-Gen1` repositories. The alternative the doc itself allows is keeping the paragraph so the dangling reference is a known one |
| Five "left undetermined" questions from the unwired re-cut — light-theme inertness of the CARTO panels, the phone measure strip / label bar / way card, the 44 vs 48 dp target sweep, whether `sculpt_stroke_point` can reject an appended point, landscape composition beyond the sheet handle, and whether any `_todo` reason cites a `PARITY_AUDIT.md` section number that has moved | `UNWIRED_FUNCTIONS.md` | small | Three of the six need a handset or a light-theme capture, not a read |

---

## 3. Blocked, with the blocker named

A row is here only if something concrete stops it. Where the blocker is an
owner answer, the question itself is in §4.

### 3.1 Blocked on an owner decision

| Item | Owns it | Size | Blocker |
|---|---|---|---|
| **Landmark M7 — viewshed / line-of-sight** | `LANDMARK_GENERATION_SCOPE.md` | large | Open question 5: the accuracy/cost budget (observer count, radius cap, grid resolution). §5 states the complexity honestly and deliberately does not choose a number. **Gates six of the 49 landmark kinds**, and `needs_viewshed` already ships as a declared flag with no implementation behind it. 8192² is 67 M cells, so it is not a naive all-pairs proposition |
| **IN-13 — trade flows**: who trades with whom (bipartite match, network flow), prices, tariffs, caravans as entities | `STATUS.md` | large | Needs a decision about what a currency is in this world. `TradeBalance` names *what*, never *who* |
| Resolution-range policy — 4096 needs 2.41 GiB and 8192 needs 9.65 GiB, so 2048×1311 is the last Android-viable preset | `MEMORY_OPTIMIZATION_SCOPE.md` §8 | small | A product decision. The doc twice refuses to change `RESOLUTION_PRESETS` unilaterally, and now has the numbers to support whichever way it goes |
| Save compression — the byte-plane shuffle (27-36% smaller, writes faster) | `STATUS.md` | medium | Needs a `format_version` bump and a fail-loud marker; **it ends `SAVEFILE_COMPAT.md` §8's bare-dump promise** |
| Save compression — quantising saved rasters to `u16` | `STATUS.md` | medium | Lossy. `PARITY_TESTING.md` and `DECISIONS.md` §7a bar it without a ruling |
| **CA-19** — a writable biome colour table | `STATUS.md`, `PARITY_AUDIT.md` | medium | Buildable today, but **costs a golden re-baseline** that `DECISIONS.md` §7a protects |
| Delete the seven uncalled `cartalith-gpu` public functions (~70 lines) | `GPU_LAYER_INTEGRATION_SCOPE.md` | small | The ponytail pass declined to delete public API on its own authority. Verified today: `heterogeneity_grid_gpu`, `gauss_blur_grid_gpu`, `assign_plates_grid_gpu`, `flow_accumulation_gpu_with`, `gpu_resistance_grid_cpu` and `init_gpu_f64` have zero callers; `warp_grid_gpu`'s only external hit is a doc comment. `init_gpu_f64` is separately owner question 8 |
| The flaky GPU determinism test `generate_terrain_gpu_path_is_deterministic_and_valid` | `STATUS.md` F1 | small | Fails ~1 run in 3 under full-workspace parallel load, by ~1 ulp. The decision is whether an `assert_eq!` on a whole f32 field is the right bar for a path §7a holds only to principled equivalence |
| Military manpower **finding 2** — standing armies land at Imperial Rome's ratio, not the era table's standing column | `MILITARY_MANPOWER_SCOPE.md` | medium | Correcting it means recalibrating outputs currently validated against the owner's worked example. Reported, not tuned |
| Shrink `STATUS.md` | `STATUS.md` own header | medium | An editorial decision for the owner, declined twice by audit passes as correctly out of their remit. Still not made — but **the size that motivated it is gone**: this cell said "8 122 lines with four lines over 15 000 characters" until 2026-09-01, contradicting this document's own header three paragraphs in. `wc -l` gives **1 445** today (1 157 at the 2026-08-31 rewrite, so it is growing again). The decision is open; the emergency is not |

### 3.2 Blocked on other work in this list

| Item | Owns it | Size | Blocker |
|---|---|---|---|
| **Landmark M9** — cultural interpretation and temporal state | `LANDMARK_GENERATION_SCOPE.md` | large | `STORY_PLANNING_SCOPE.md` **SP-4**, which is not started and whose attachment model is undecided, plus open questions 1-2. **Two documents' largest remaining milestones sit behind one unasked question** |
| Story planning **SP-2** — journey progression over the cursor | `STORY_PLANNING_SCOPE.md` | large | §6's regenerate-semantics question explicitly gates it: whether a journey's route polyline is invalidated, re-snapped, or kept with a staleness mark "needs a ruling before SP-2 ships". The grain question (real date vs fraction of a year) is also unresolved |
| Story planning **SP-5** — the planning aid, joined up | `STORY_PLANNING_SCOPE.md` | medium | Deliberately last: worth nothing until at least two of SP-1…SP-4 exist. Only SP-1 is partly real |
**Closed from this table 2026-09-03** (batch 18, verified): **milestone 16** —
shipped in `cff1edc`, golden byte-reproducible from the frozen reference, 12 of 13
stage modules mutation-covered. **Milestone 17's five `_um*`** — all five exist,
all five are golden-covered, and all five survive mutation of a constant each
(`um_wall_spec` `age >= 260.0 → 261.0` KILLED; `um_site_profile` `gw/70 → gw/71`
KILLED). **Both rows' stated blockers were false.** Milestone 17's — "settlements
carry no `specialisation` and no `traits`" — was falsified **six minutes after it
was written**: `be2d5f7` 19:31:09 added the `economy: None` hardcode, `e63d5d9`
19:37:15 added the `PlaceExtras` that supplies it, and it stood for eleven days.
One genuine gap remains and is filed under §2.1: `urban_bridge.rs` still calls
`settlement_layout()` (which supplies `PlaceOverrides::default()`) rather than
`settlement_layout_with()`, so a per-settlement wall/age override is stored but
never reaches the layout.

| The **GUI_GAP_REGISTER §3** A/B/C/D open/closed split, never re-derived | `GUI_GAP_REGISTER.md` | medium | Recovering each dropped class letter is "a judgment per row, not arithmetic" — declined by three consecutive audit passes. The register cannot currently say how many of its 300 IDs are open. `UNWIRED_FUNCTIONS.md` is the live successor; read the register as history |

### 3.3 Blocked on hardware, or on a design that does not exist

| Item | Owns it | Size | Blocker |
|---|---|---|---|
| **CV-24 / ED-02** — the year scrubber as program scope; the undo-history panel | `STATUS.md`, `TIMELINE_SCOPE.md` §4 | medium | **Reclassified 2026-09-03 from "blocked on an owner decision" to this section.** Put to the owner and ruled **both wait for a design pass** — §4's standing instruction to design the panel first rather than guess its region is upheld rather than overridden. Not closed, not startable; the ruling is that guessing the region is worse than waiting for the design |
| The phone overflow menu — re-present the seven desktop menus as a touch-sized drill-down | `ANDROID_BUILD_SCOPE.md` §5 | large | A mobile menu design is being produced separately; the pass was instructed to diagnose only. Four compounding causes including 15 hover-opened submenus and ~12 physical-px rows |
| **BUILD_ANSWERS §3** — the Data-manager window and 13 of 24 asset families are absent from the new Environment prototype | `design/…/BUILD_ANSWERS.md` | medium | Awaiting a decision: build them against the older `Cartalith DCC Shell.dc.html` canvas, or have the design project add the window to the Environment file. **A standing offer to supply it exists.** Easy to lose, because it sits in an answers file rather than in the plan |
| **BUILD_ANSWERS §4** — phone generation-failure and storage-full states are undesigned; content descriptions and dynamic type are absent; the 48 dp target sweep is partly done | `design/…/BUILD_ANSWERS.md` | small | The design does not exist; the design project has offered to produce it on request. Feeds stage 6 |
| **DS-13** — the phone viewport control column (zoom/pan/navpad) redesign | `GUI_GAP_REGISTER.md` §57 | medium | Three registered, nothing built — and four high-severity refutations of the proposed design, including three colour equalities that were arithmetically false |
| Observe the §13 phone **landscape** composition on the device | `ANDROID_BUILD_SCOPE.md` | small | `adb` cannot force it: Godot's `orientation="sensor"` sets `SCREEN_ORIENTATION_SENSOR`, which follows the accelerometer and overrides `settings put system user_rotation`. **Needs the owner to physically rotate the handset.** Every measurement in §50 is portrait |
| **§47** — hi-DPI blur confirmed only to `_phone_scale` 2.748, not the owner's 3.664 | `STATUS.md` | small | Needs the OnePlus 12, which this project has not had on the bench. §47 is confirmed *in kind* and not at that scale |
| **GPU §21** — thermal / mobile-adaptive GPU scheduling | `GPU_COMPUTE_PILOT_SCOPE.md` | medium | **No Android GPU compute path exists to adapt.** The handset runs the CPU pipeline entirely; the device passes treat "zero `wgpu` lines in logcat" as a *pass* condition. Both `project.godot` renderer keys are `gl_compatibility` |
| Owner question 3 — should the WORLD left-dock A/B switch come back? | `UNWIRED_FUNCTIONS.md` | small | Doubly blocked: an owner call, *and* the captions and gate (`ldSwitch`/`ldSwA`/`ldSwB`) are in the truncated tail of `02-rail-and-domains.md` §8, so there is no label to build it with |
| The 3D research's three commissioned questions (`gl_compatibility` rationale; wgpu/Godot GPU coexistence; what a raised device floor buys) | `3D_TERRAIN_RENDER_RESEARCH.md` | medium | Parked with the 3D viewport. Question 2 is named the highest-value unanswered question and gates `RenderingDevice`, compute shaders and GPU-driven culling. Resuming is cheap — the research is complete at 1 530 lines |
| Vault **milestone 4** — device pass verifying the Android SAF provider (folder picker, persisted grant, revocation) | `MARKDOWN_VAULT_SCOPE.md` | large | Needs a real Android device |
| Whether the root `Cartalith Gen1 v2.11.html` (now also mirrored into `reference/`) is `Cartalith_RC`'s live head, or a copy that repository has since moved past | `CLAUDE.md`, `REFERENCE_DRIFT_v2.10_to_v2.11.md` | small | `Cartalith_RC` is not present on this machine and is not a remote of this repository, so it cannot be checked without opening it — do not assert either way without doing so. Left open by the 2026-09-02 re-freeze pass, which froze `reference/` to v2.11 without resolving it |

---

## 4. Open decisions the owner still owes

Not work. Each of these has to be answered before the row it gates becomes a
task. Ordered by how much they unblock.

| # | Question | Owns it | Gates |
|---|---|---|---|
| 1 | **What is a conflict attached to** — free geometry, or a reference to a settlement/province? | `STORY_PLANNING_SCOPE.md` §6 Q2 | SP-4, and through it landmark M9. The single highest-leverage unanswered question in the project |
| 2 | **The viewshed cost budget** — observer count, radius cap, grid resolution | `LANDMARK_GENERATION_SCOPE.md` OQ 5 | Landmark M7 and six landmark kinds |
| 3 | **Regenerate semantics for a journey's route polyline** — invalidated, re-snapped, or kept with a staleness mark? | `STORY_PLANNING_SCOPE.md` §6 Q3 | SP-2, explicitly |
| 4 | **Does the landmark set live in the save tree (`entities/landmarks.json`) or regenerate on load?** | `LANDMARK_GENERATION_SCOPE.md` OQ 1 | The record's shape — research §25's state transitions cannot be recomputed. Storage is in memory today and the save format is untouched |
| 5 | **Does a landmark become a `cartalith_vault::EntityKind`?** | `LANDMARK_GENERATION_SCOPE.md` OQ 2 | A `Landmark` template exists in `design/vault-templates/` and is recognised by `template.rs:155`, but `links.rs:81-84` has no variant. `MARKDOWN_VAULT_SCOPE.md` §4's identity-strength table needs a new row first |
| 6 | **Does `DECISIONS.md` §7a/§7d's parity contract apply to landmarks at all?** | `LANDMARK_GENERATION_SCOPE.md` OQ 3 | `FUNCTION_INDEX.md` returns nothing for "landmark", so there is nothing to match. `landmark.rs` was built assuming divergence-by-addition; no ruling is recorded |
| 8 | **What should `statusMid`'s `repaint NN ms` measure** — frame time, texture-upload time, or `_refresh_map()` wall time? | `UNWIRED_FUNCTIONS.md` Q2 | One field of a composite that otherwise shipped |
| 9 | **Should a committed sculpt stamp re-evaluate when sea level moves?** | `UNWIRED_FUNCTIONS.md` Q6 | Also a parity question: the reference re-reads `state.seaLevel` live, this port snapshots. `sculpt.rs:1076 with_sea_level` exists and nothing calls it |
| 10 | **Should generated worlds be denser relative to carrying capacity?** (`civ_settlement_population`'s surplus fractions) | `ECONOMY_SCOPE.md`; also `MILITARY_MANPOWER_SCOPE.md` finding 3 | Raised independently by two documents. `ecological_factor` saturates at its 2.0 ceiling on 5 of 6 real factions, which is the symptom |
| 11 | **Is `init_gpu_f64` kept or deleted?** | `UNWIRED_FUNCTIONS.md` Q8 | Part of the seven-dead-functions row. The pilot recorded no disposition for its own residue — `GPU_COMPUTE_PILOT_SCOPE.md` has no `f64`/`SHADER_F64` mention at all |
| 12 | **Is the phone app bar's ☰ / ▤ pair now stale?** | `UNWIRED_FUNCTIONS.md` Q9 | Scopes stage 3 of the shell rebuild. The 2026-08-31 Android canvas's app bar is [world pill] · ⌕ · ⋮ |
| 13 | **`--good` and `--accH`** — declared and never used | `UNWIRED_FUNCTIONS.md` Q10 | The prototype records both as declared-and-never-used itself, so a shell with no consumer may be fidelity rather than a gap |
| 14 | Where do landmarks live in the crate graph? | `LANDMARK_GENERATION_SCOPE.md` OQ 4 | **Answered de facto, never formally**: the code landed in `cartalith-civ/src/landmark.rs` and `cartalith-terrain/src/analysis.rs` rather than a new crate. §4.4 called it "a real architectural fork, not just a filing question" and the fork is not recorded as decided |
| 15 | How does a generated landmark relate to the manual icon tool (`annotations/icons.json`)? | `LANDMARK_GENERATION_SCOPE.md` OQ 6 | Affects the save format, the renderer, and M6's spacing inputs. Partially touched — 49 glyphs were drawn — but a rendering vocabulary is not the decision |
| 16 | Should the 16K/32K export be un-shelved? | `EXPORT_SCOPE.md` | See §5 for the four things an un-shelve costs |
| 17 | Store distribution and signing (`DECISIONS.md` §6) | `ROADMAP.md` | "Things the architecture permits and nobody has committed to." Not work until someone commits. `export_presets.cfg` has only Windows Desktop and Android |
| 18 | A WASM target sharing `cartalith-engine` (`DECISIONS.md` §2) | `ROADMAP.md` | Same status. Zero `wasm32` hits in any `Cargo.toml` |
| 19 | Should `STATUS.md` be shrunk, and how? | `STATUS.md` header | Listed in §3.1 as a blocker too, because it taxes every session that follows `CLAUDE.md` literally |

**One question left this section on 2026-09-01 and is not renumbered into
it.** *"Which unit systems are offered? (**PR-15**)"* — formerly #16, owned by
`GUI_GAP_REGISTER.md` §10 — is **answered**: `LARGE_ITEM_RULINGS.md` rules it
*"Build, and add nautical miles"*, naming km / mi / nautical miles outright.
An answered question is not an open decision, so it is deleted here rather
than carried with a note, per this document's own no-"done"-column rule. The
work it gated is the Units row in §2.2, which now carries the residual
`_todo` at `menus.gd`. Questions 17-20 shifted up by one; a citation of
"question 16" written before this date means Units, not the export shelf.

---

## 5. Declined and shelved, and why

Kept so nobody re-proposes them. Nothing here is a gap.

**Owner-parked or shelved, reversible by a word**

- **The 3D viewport, and all 3D work.** Parked 2026-08-31, the same day the
  research landed: *"On part of the 3D let's keep that for later at this
  moment, it will be implemented later on."* `DECISIONS.md` §4 continues to
  stand. The research is complete and parked at 1 530 lines. The two menu rows
  and the phone 2D/3D FAB stay drawn and disclosed; the FAB's toast becomes
  honest only when the Small relief-exaggeration row lands. This is why
  `ROADMAP.md` Phase 3's "3D drape" is **not** listed as outstanding above.
- **16K/32K single-image export, E1-E5.** Shelved 2026-08-25 at the owner's
  request. Un-shelving costs four things in order: (a) lifting the shelf;
  (b) reversing the documented "rendered once, tiled and single are the same
  pixels" decision in `export_raster.rs` — a `DECISIONS.md`-grade change whose
  "same pixels, no seams" guarantee **has already been earned** by E1's
  byte-identity tests, which were built, proven at five band heights, and then
  deliberately reverted; (c) a ruling on the codec/size trade, since §6.3 is
  blunt that at 32K no codec makes this small (500 MB - 1 GB lossless, the one
  lossy option ruled out by AGPL licensing on `jxl-encoder`, WebP eliminated
  at 16 383 px); and (d) accepting that E4 — overlays into a `SubViewport`
  across frames under a synthetic per-band camera — is new work with **no
  reference behaviour to port against**, because the reference's own bake
  draws terrain and nothing else.

**Declined on measurement or architecture**

- **§21, the GPU rendering path for appearance.** Milestone 6 measured the case
  at ~5% of a generate+render and declined to start; a second renderer would
  diverge from the golden-verified one under `DECISIONS.md` §7c. Treat this as
  a decision to confirm, not a task to schedule.
- **Tile-scoped (incremental) recomputation of hydrology/climate/civ stages.**
  A separate re-architecture, only worth taking if lazy whole-recompute proves
  too slow — and it has not: 76.5 ms @512², 188.9 ms @2048², 18.8× cheaper
  than the generation it replaces.
- **Per-stage re-execution of the ten-stage pipeline.** The capability exists in
  neither this engine nor the reference app; verified by Playwright against the
  real reference (WW-11).
- **Orogeny graph-tracing and Dijkstra/MST road networks on GPU.** The first
  needs genuine algorithmic redesign; the second was confirmed as
  should-stay-on-CPU because predecessor ties are settle-order-dependent and
  roads would visibly move.
- **The hard-hazard sequential functions** — CPU flow accumulation,
  priority-flood, `compute_stress`'s scatter, `erode_thermal`'s delta scatter,
  `droplet_kernel`, the stream-power main loop, orogeny tracing,
  `road_dijkstra`'s traversal. Confirmed unsafe per function with the hazard
  named, not assumed. **Three separate documents defer the same four
  algorithms for the identical reason**; they are the shared ceiling on both
  the GPU and the Rayon efforts, and none has an owner.
- **Landmark M1's consolidation** — consolidate the three duplicate slope/TPI/curvature copies onto the canonical field. M1's own "Done when" demands `build_ao`'s output be proven **bit-identical** before and after refactoring. `DECISIONS.md` §7a protects the rendered output, and `cartalith-terrain/src/analysis.rs` module doc explains the reasoning: refactoring `build_ao` would put a golden-protected render path at risk to share four lines of box blur. Declined rather than scheduled.
- **R6** — stop reserving grid-sized capacity in the two heaps (42.96 + 32.2 MiB). Declined as low-value, with its own note already recording that the saving is small on Android.
- **A bounded thread pool** — declined as "this port has no interactive editing
  mid-generation to protect against". The Sculpt/paint tool system has since
  landed, so the premise is worth re-checking.
- **`ComputeBackend` trait abstraction** — "premature with one kernel". Nine
  kernels exist now, so the stated reason has partly expired.
- **Overlay lever 1** (collapse the dash loop into one `draw_multiline`) —
  measured a no-op to the digit, verified pixel-identical, reverted rather than
  shipped. `_dashbatch_probe` is kept as the reason not to retry.
- **hi-DPI mitigations** — font oversampling 1 152 KB, icon re-rasterisation
  424 KB. "There is no trade-off here to make."
- **The Android `.so` at 156-171 MB** with `debug = "line-tables-only"` rather
  than the 18 MB a full strip gives. If size ever becomes the binding
  constraint, drop `debug` and set `strip = "debuginfo"` together.

**Declined because the engine has no counterpart**

- **AS-14** user-picked "active variant" (variant choice is weighted and seeded);
  **AS-15** per-slot Anchor (`Anchor` is a *family* property); **AS-16** the
  24-family rail (owner decision, disclosed in the window's header).
- **Vault §35 criteria 6-7** — POIs and "regions" as entity kinds. Recorded as
  unsatisfiable rather than faked.
- **Vault §11 TextRange/MarkdownBlock selections** — a correctness decision: a
  byte offset stops pointing at the right paragraph. **§19** continent field on
  a settlement's export block — `civ_continents` deliberately keeps no raster
  (268 MB at the 8192² ceiling). **Two-way sync, `obsidian://` links, the Data
  manager vault block** — §33's explicit V1 non-goal. **Setext headings** — ATX
  only, because that is what all four of the owner's real templates use.
  **Feeding the imported note copy back into world state** — §36 forbids a
  second source of truth.
- **Journey Planner**: six DOM render functions (Godot's job), `_jpLayovers` and
  `_jpSettlements` as Rust functions, and the `JpParty` widening — re-examined
  and deliberately declined, because `TRAVEL_LIBRARY_SPEC.md` §3.1 carries no
  seasonal-physiology or desert fields.
- **Military**: per-settlement garrisons (a placement rule nothing implies),
  campaigns / unit movement / combat (each needs a clock, a map objective and
  an opposed force), change over time, and leaving `power.military` as the
  reference's golden-verified composite.
- **Urban**: `_umDrawLayout`/`_umDrawLayoutPreview`/`_umLayoutAlpha` and the
  block-1 LOD hook (canvas rendering is Godot's job — built as GDScript
  instead); the `_umModelCache` LRU and one-per-frame `setTimeout` queue (a
  workaround for the browser's single thread; this port has real threads); the
  17 removed culture profiles (only `medieval` and `venus` are live);
  `buildGridStreets` and the palimpsest mode (removed upstream, no live caller).
- **Asset library authoring-side conveniences** the reference itself calls
  authoring-only.
- **`state.erosion` is not written to saves** — only 2 of 16 keys are modelled
  by `loadZip()`, so it is deliberately not written rather than written
  partially.
- **Warfare, Narrative/Scenario, year-by-year historical playback, and a
  coordinate system / projection.** The first three need a product decision
  nobody has made; projection is declined outright because Cartalith's world is
  a flat, non-georeferenced procedural grid with no real-world CRS.
- **`DECISIONS.md` §7b's simulated historical territorial expansion** —
  considered and deferred, not rejected: revisit only if the static weighted-
  Voronoi result feels wrong once actually seen, not preemptively.
- **The Data manager's five silent navigation rows** — re-checked and left
  alone twice; each opens a pane that explains itself.

---

## 6. Contradictions in the project record

These are defects in the record, not milestones, and they are worth more than
any single row above: each one costs a future session either re-derived work or
a wrong plan. They are ordered by what they cost.

### 6.1 Two documents exist only in the working tree — ~~open~~ **closed 2026-09-01**

**This defect is fixed, and the paragraph below is kept only because §6 is a
record of what the project record got wrong.** `LARGE_ITEM_RULINGS.md` and
`cartalith-native/docs/3D_TERRAIN_RENDER_RESEARCH.md` are both in `HEAD` —
verified with `git cat-file -e HEAD:<path>` on each, not by reading a
document — having landed in `fd9de7c` with 235 other files. No clean checkout
loses either. The stale wording survived in three places at once (here, "The
three that matter" #3, and §2.2's footnote), which is itself the pattern this
section exists to name: one fact asserted in three files ages in three
places.

*What it said, for the record:* `git status` → `?? LARGE_ITEM_RULINGS.md` and
`?? cartalith-native/docs/3D_TERRAIN_RENDER_RESEARCH.md`. The first carries the
owner's rulings on all eighteen Large rows, **including two scoped
authorisations that override standing rules** — editing
`crates/cartalith-godot/Cargo.toml`, and diverging from the reference on paint
falloff — plus the accepted cost on colour management. The second is 1 530
lines of commissioned research. A clean checkout loses both. *This is the
cheapest thing on this page to fix.*

### 6.2 `CHANGELOG.md` is five days behind the repository

Last heading: `## 2026-08-26 (12)`. A grep for `2026-08-3` returns **zero
matches**, while `git log` shows eleven commits dated 2026-08-30/31. Missing
entirely: landmark generation end to end (`a6feec3`), the 49 landmark glyphs,
`DESIGN_HANDOFF.md`, the prototype import, the GUI replacement spec, the
INFRA→CIVIL / RENDER→CARTO ruling, stages 1-2 (`c03b43c`), and the unwired
re-cut (`5543ef3`). A grep for "landmark" across all 29 534 lines returns two
unrelated hits. `CLAUDE.md` tells every session this file records what actually
landed; for the last week it does not, and anyone reconstructing state from it
will conclude the GUI replacement has not begun.

### 6.3 `STATUS.md` contradicts the code it summarises, on the same day

Its newest section header (`:133`, dated 2026-08-30) reads *"Landmark
generation catalogued, **nothing built** — no viewshed, no Poisson-disc…"*.
Landmark generation shipped that same day:
`cartalith-civ/src/landmark.rs` is **3 730 lines** with 49 kind specs, a
`Landmark` struct carrying `causal_chain`, and `pub fn generate`; plus
`landmark_bridge.rs`, 49 glyphs, `_landmark_probe.gd`, and a CIVIL ▸ Landmarks
panel. Thirteen of the 49 kinds generate today. The genuinely-absent parts are
narrower than the header: viewshed, persistence, and the vault entity kind.

Three further `STATUS.md` defects:

- Its `Last updated:` line (`:330`) says **2026-08-25** while sections dated
  2026-08-30 sit above it, and there is **no 2026-08-31 section at all** — so
  the largest structural change since the shell was written (the five→three
  rail fold) is invisible in the authoritative living status.
- It reports the unwired backlog as *"44 → 21 open"*; the 2026-08-31 re-cut
  says **77**. A session trusting `CLAUDE.md`'s "authoritative status is
  `STATUS.md`" gets a number 56 rows low.
- It has no section for `MILITARY_MANPOWER_SCOPE.md` (built 2026-08-25) or for
  `ECONOMY_SCOPE.md` at all.

### 6.4 Six scope documents are stale in the same direction

Every one understates progress. The pattern is consistent and worth naming: the
gap registers get re-verified, the scope documents do not.

| Document | What it still says | What is true |
|---|---|---|
| `TERRAIN_APPEARANCE_SCOPE.md:1113` "Still open" | Nine items, incl. "the GUI editing panel (all UI work on hold)", "§17 colour vibrancy", and milestone 1's elevation-ramp question | **Three of nine are stale.** The UI hold lifted 2026-08-18 and the panel shipped as `render_workspace.gd`; §17 shipped as the "Colour grade" and "Grade field influence" groups; the ramp question is answered in code both ways (`lib.rs:1873`, `list_ramp_presets`). §19 is half-done rather than open |
| `ASSET_LIBRARY_SCOPE.md:1068`, `:1179` | AS-07, AS-12 and AS-17 are "still honestly a gap" | All three closed **2026-08-23** (`as_set_item_transform` at `lib.rs:11060`, `as_collections`, `SliceGrid::move_line`) and are recorded closed in `GUI_GAP_REGISTER.md` |
| `UNIFIED_TOOL_PLAN.md:2265`, `:2268`; `STRANDED_TOOLS.md:34` | "all UI work is on hold"; milestone F is the only work left and is unwired | **Resolved 2026-09-01, not just historical.** The hold-lifted claim itself was already stale — `CLAUDE.md` records having already corrected this exact class of error once — and the second half is now fixed rather than merely diagnosed: `UNIFIED_TOOL_PLAN.md` carries a verified "Milestone F as built" section, and `STRANDED_TOOLS.md`'s "44 methods... not one wired" claim is annotated false in place. See `STATUS.md`'s Tool system row |
| `MARKDOWN_VAULT_SCOPE.md:247`, `:259`, `:439` | Milestone 3 "blocked"; milestone 6 "engine half done", "the UI half is not built" | Milestone 3's blocker lifted 2026-08-25 (it is not-started, not blocked). Milestone 6's UI landed 2026-08-26 — `vault_window.gd` 641 → 1 140 lines, with `_build_search` and the "confirm always" checkbox |
| `GUI_FEATURE_PARITY_SCOPE.md` status box | Twelve items open, incl. heightmap import, GeoJSON export, the appearance GUI, the faction roster, the Journey Planner GUI, the Asset Library UI, the LOD viewport, light theme, opacity, measurement, quality tiers, PopupMenu theming | **Fully discharged as of 2026-09-01.** Every milestone item now exists in code — the one survivor, route corridors/travel cost as a selectable analysis field, shipped this pass (`sample_bridge.rs`'s `corridor`/`travel_cost` ids, tested). Only the never-attempted per-stage slider audit the document names in its own closing paragraph remains, and that was never a milestone item. The document should be closed out |
| `DCC_SHELL_SCOPE.md` | "Milestone 2 and milestone 3+ remain not yet dispatched"; "still deferred: light theme, responsive breakpoints, all tool functionality" | Both dispatched and completed the same day the sentence was written; all three deferrals closed. It is a 2026-08-18 snapshot wearing a milestone-plan title |
| `design/…/00-REPLACEMENT-PLAN.md` §0 | Opens with "the desktop prototype we received is truncated"; stages 5 and parts of 2 blocked | The split re-export landed the same day (Environment 239 712 B + `cartalith-dcc-parts.js`), `BUILD_ANSWERS.md` §1 confirms everything is present, and stage 2 completed. **Stage 5 is not blocked; it is not started** |

### 6.5 `FUNCTIONAL_CONTRACT.md` disagrees with itself in four places

Its bodies were not updated when its summary table and absent-list were. The
document explains why — it is a summary no feature commit is obliged to touch —
and it has now gone stale three times in eight days, with its own header
recording corrections on 2026-08-23, -24 and -25.

- Capability 3's body says slider-triggered live re-tuning is absent; `:578` and
  `:644-651` both record **SG-03 closed 2026-08-24**, citing `set_params`' own
  doc comment.
- Capability 6's body says the atlas/tile cache and the bake lock "remain
  unbuilt"; the absent-list strikes that bullet as landed
  (`cartalith_engine::bake::AtlasStore`).
- Capability 6 lists AO toggles as absent; AO shipped and `render.rs:1515`
  exposes `"ao_strength"`.
- Capability 13's body says urban milestones 8-17 "remain entirely unbuilt";
  8a and 12 landed 2026-08-24.

### 6.6 The reference freeze has actually drifted — the re-freeze itself closed 2026-09-02, the stale sentence survives

`FUNCTIONAL_CONTRACT.md:16-24` asserts the frozen v2.10 is the live repository's
latest and there is *"no re-freeze question to raise"*. That sentence is still
there, unedited — scope documents were deliberately not mass-edited in the
re-freeze pass — and it is still wrong.

**What is fixed:** `reference/` now holds `Cartalith Gen1 v2.11.html` (2 374 691
bytes) alongside v2.10 (untouched, byte-unmodified), `reference/FUNCTION_INDEX_v2.11.md`
was generated mechanically, and `REFERENCE_DRIFT_v2.10_to_v2.11.md` records the
drift. This was §2.8's row; it is done and deleted from that list, so
`FUNCTIONAL_CONTRACT.md`'s capability tags now have a v2.11 index to be checked
against, even though nothing has re-checked them yet.

**What is not fixed:** `FUNCTIONAL_CONTRACT.md`'s own sentence — a documentation
defect independent of the index it was excused by. And a question the re-freeze
raised rather than closed: whether the root `Cartalith Gen1 v2.11.html` is
`Cartalith_RC`'s actual live head, or a copy that repository has since moved
past, is unverified and unverifiable from this machine — see §3.3.

### 6.7 Five documents claim a blocker that has already lifted

Each of these will cause someone to skip real, startable work:

**A fifth instance was found and fixed 2026-09-02, and it is the most expensive
one yet recorded — it cost a dispatched wave.** Three live test headers stated
that golden fixtures *"could not be extracted because the environment has no JS
runtime"*. `node` is v24.19.0 and has been available for weeks; the retired
`CHANGELOG.md` even notes at its line 2000 that *"Node is installed now"*. Worse,
everything the claim gated had **already shipped on 2026-08-15**:
`stamp_volcanoes_provinces` was golden-verified in `713e0b1`, and `555b753` /
`3fd2fef` flipped `volc.provinces`, `terrain_wind_deflection` and `currents` to
`true`. Nothing in this port was ever off-by-default because of it; only the
prose lagged, for eighteen days. `tools/jsruntime_probe.js` now settles it two
ways — it proves the runtime executes the frozen reference *and* that the
committed fixtures are genuinely its output rather than the Rust port's, with
4/4 mutants killed. Three further stale claims in the same family
(`cartalith-climate/src/lib.rs`, `golden_parity_weather.rs`, and
`cartalith-engine/src/lib.rs`'s own `WorldParams::defaults` comment) are
recorded but **not yet swept**.

| Claim | Reality |
|---|---|
| `journey_bridge.rs:70` and `JOURNEY_PLANNER_SCOPE.md`: the ecoregion/species-richness subsystem "is unported and on no milestone anywhere" | Ported 2026-08-23 (`b7a46a7`) — `wildlife.rs:367` `build_ecoregions`, `:550` `region_richness`, `:588` `assign_wildlife`. The remaining work is wiring, and smaller than either document says |
| `ANDROID_BUILD_SCOPE.md`: the ~19 MB of `godotsteam`/`godot_ai` addons is "flagged, not fixed", including in a Done-means table row | Fixed 2026-08-20 in `d044af9` (`export_presets.cfg:56 exclude_filter`), with no `CHANGELOG` entry — so the fix is invisible to the docs and the doc actively misreports it |
| `GPU_LAYER_INTEGRATION_SCOPE.md` m6: `use_gpu` is deliberately "unexposed in the UI until a real UI/UX pass adds the §7c messaging" | The shell exposes it (`menus.gd:2792`) **and defaults it on at boot** (`engine_bridge.gd:170`). The engine default is still `false`, so both statements are locally true and the conclusion is stale. §7c's messaging requirement *was* met (`menus.gd:1629-1632`), but nowhere in that document |
| `CPU_MULTITHREADING_SCOPE.md`: using the integrated GPU alongside the dedicated one is a "separate, lower-priority idea recorded, not scoped" | Shipped 2026-08-20 as `crates/cartalith-gpu/src/multi.rs` — enumeration, selection, split tiles and a VRAM cap |

### 6.8 Counts that disagree with themselves

Small, but this is the document set that exists because countable claims drift.

- `ROADMAP.md` Phase 4 says "all seven milestones"; `ASSET_LIBRARY_SCOPE.md`
  §11 records an **eighth** (the sprite-sheet slicer, 2026-08-20). The count is
  stale low, not the work.
- `ROADMAP.md`'s "Not a phase: LOD and large worlds" still says *"revisit when
  a concrete need appears rather than building it speculatively"*, while
  `STATUS.md` lists shipped "LOD levels 0-8, Tiled LOD auto/manual" and
  `LOD_TILING_BASE_SCOPE.md` exists.
- `URBAN_MORPHOLOGY_SCOPE.md` gives the `_um*` adapter's denominator as **20**
  at `:2098` and **28** at `:1770`. The 20-item list is the one that enumerates
  names, so it is the checkable one: 13 ported, 5 blocked on milestones
  9/10/13/15, `_umPt` typed away, `_umCacheKey` out of scope.
- `UNWIRED_FUNCTIONS.md`'s headline **77** double-counted two rows its own
  "fixed during the audit" section closed (State religion,
  `_refresh_phone_bar_lit()`); 75 were genuinely open at the 2026-08-31 cut.
  Its Large section heading read "(16)" where the intro said 18. **Both
  historical as of the 2026-09-01 re-cut**, which was written from scratch
  against the tree rather than patched, and carries internally-consistent
  counts (18 Large in both the heading and the running total) — see
  `STATUS.md`.
- `LARGE_ITEM_RULINGS.md` says the 3D research "stands complete at 1 486
  lines"; `wc -l` gives **1 530** — drift inside the same day.
- `LARGE_ITEM_RULINGS.md` answered owner questions 4, 5 and (by implication) 7;
  the 2026-08-31 cut of `UNWIRED_FUNCTIONS.md` still listed all ten as open.
  **Fixed in the 2026-09-01 re-cut**, which marks 4, 5 and 7 "Answered" by
  name (7 only partly executed: the fifth save slot the ruling called for is
  still unbuilt) and leaves 1, 2, 3, 6, 8, 9 and 10 genuinely open.

### 6.9 One claim that would misdirect a ruling

`URBAN_MORPHOLOGY_SCOPE.md:1761-1766` contains the sentence *"the crate is not a
dependency of `cartalith-godot`"*. It is a **quotation of what `PARITY_AUDIT.md`
§3.4 found before milestone 17a**, and the same paragraph goes on to describe
closing it. Read out of context it will produce a ruling to add a Cargo edge
that would buy nothing and violate the layering `cartalith-civ/Cargo.toml:18-22`
defends. The full correction is in §2.1.

---

## 7. What this document does not cover

- **Test status.** No surveyor ran `cargo test`. Every "done" and every
  "golden-verified" here is the owning document's claim carried forward, plus a
  structural code check that the named module or binding exists. The known
  intermittent failure (`generate_terrain_gpu_path_is_deterministic_and_valid`)
  is listed in §3.1 as a decision, not a result.
- **The re-verification that commit was the precondition for.** This bullet
  used to read *"the uncommitted working tree… **126 tracked files** now
  differ from `HEAD` (16 488 insertions, 10 499 deletions)… Every such row
  needs re-verification once that work commits."* **That work committed**
  (`fd9de7c`, 237 files / 90 718 insertions), so the precondition is met and
  what is left is the debt, not the tree: `git diff --shortstat` re-run
  2026-09-01 gives **1 file changed** (an in-flight `journey_planner_view.gd`),
  and the only untracked paths are two `_routecutout_probe.*` scenes and a
  `tools/__pycache__/`. Nothing in this document has re-verified a closed row
  against the committed tree yet.
- **`UNWIRED_FUNCTIONS.md`'s 21 rows individually** (22 after the same-day
  second pass, 23 after the 2026-08-31 cut's morning pass, 75 before it).
  They are one row here because that document is the live backlog with its
  own `file:line` per row, and forking it would guarantee the two drift.
- **`GUI_GAP_REGISTER.md` as a working list.** Its ID total was re-counted three
  times (123 → 215 → 300) and its A/B/C/D open/closed split was never
  re-derived once; a class marker survives on only 54 of 215 rows. Read it as
  history. `UNWIRED_FUNCTIONS.md` is the successor, re-cut 2026-08-31 against
  the three-domain shell.
- ~~**The stray root files.**~~ `518.86`, `518.92` and `66.0` — accidental
  shell-redirect artefacts from the memory-measurement work — **are gone**
  (`ls` finds none of the three, 2026-09-01). Kept as a struck line rather
  than deleted so nobody re-investigates the same three filenames.
