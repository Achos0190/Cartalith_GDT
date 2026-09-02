# Reference drift — `Cartalith Gen1 v2.10.html` → `v2.11.html`

Produced 2026-09-02, in the pass that added `reference/Cartalith Gen1 v2.11.html` beside the v2.10
freeze and generated `reference/FUNCTION_INDEX_v2.11.md`. `CLAUDE.md` requires the reference and its
index to move together; this file is the third piece — **the map between the two, so a later pass can
correct scope-document line ranges without re-deriving the diff.**

**Both HTML files are in `reference/` and both stay.** v2.10 was deliberately not deleted: every line
range in every scope document resolves against it. Nothing in this document changes a scope document.

| | v2.10 | v2.11 |
|---|---|---|
| Path | `reference/Cartalith Gen1 v2.10.html` | `reference/Cartalith Gen1 v2.11.html` |
| Bytes | 2 363 410 | 2 374 691 |
| Lines | 31 107 | 31 755 (**+648**) |
| Top-level functions | 1 094 | 1 108 (**+14**) |
| Per script block | 633 / 350 / 19 / 92 | 641 / 356 / 19 / 92 |
| Index | `reference/FUNCTION_INDEX.md` | `reference/FUNCTION_INDEX_v2.11.md` |

Provenance of the v2.11 file: added to **this** repository's root by the owner on 2026-08-26 in
`4b2c95a` ("HTML v2.11: reads the tree format and the vault, writes neither"), modified the same day
in `b576d56` ("v2.11 ponytail: a per-frame world-wide river trace, a cache that could not invalidate,
and 50 fallbacks defending against a legal value"). `reference/Cartalith Gen1 v2.11.html` is a byte
copy of that root file. **The root copy stays where it is** — this pass did not move or delete it.

---

## 0. Is the root v2.11 the live `Cartalith_RC` head? — still unknown, and here is what was checked

`CLAUDE.md` records this as unresolved and says not to assert either way without opening
`Cartalith_RC`. It could not be opened, so the answer is unchanged. What was actually run:

- `Cartalith_RC` is **not on this machine**: absent from `C:\Users\Vincent\`, and a depth-3 directory
  search of `C:\` and `D:\` for a folder of that name returns nothing.
- It is **not a remote of this repository**: `git remote -v` lists only
  `origin  https://github.com/Achos0190/Cartalith_GDT.git`.
- **One new fact, and it cuts against "a copy":** `b576d56` is an *edit to the HTML made in this
  repository* — the ponytail pass that removed the `||0.42` fallbacks and re-keyed the river cache
  (§4 below). So this file is not merely a snapshot dropped in; at least one v2.11 change was authored
  here. That makes this copy equal to or **ahead of** whatever `Cartalith_RC` had on 2026-08-26, and
  says nothing about what `Cartalith_RC` has done since.

Verdict: **unknown, leaning "this repository holds work `Cartalith_RC` may not have."** Do not assert
otherwise without opening that repository.

---

## 1. Scale, in one paragraph

76 diff hunks: **75 lines deleted, 723 inserted, net +648**. Two of those hunks are the version string
(line 6's `<title>`, line 510's `#verTag`). **57 hunks are one mechanical edit repeated** — 49 sites
dropping a `state.seaLevel||0.42` fallback and 8 sites replacing an inlined cache-null list with a
call. That leaves **17 hunks of real change**, and they are two features (reading the port's project
tree, reading the Markdown Vault's link store) plus one cache-correctness fix.

**Nothing was removed and nothing was renamed.** Every one of v2.10's 1 094 functions still exists in
v2.11 under the same name in the same script block — which also settles renames, since a rename would
show as a removal paired with an addition and there are no removals.

---

## 2. The number that matters for scope documents

**762 explicit `line NNNN` / `lines NNNN–NNNN` citations, across 36 markdown files, point into the
reference's script range.** (A looser regex that also counts bare 4–5 digit numbers finds 861 across
54 files; 762 is the defensible figure. This document is excluded from both counts.) Every one of the
762 falls inside a §3 segment, so every one has an exact offset:

| Offset needed | Citations |
|---|---|
| +0 | 4 |
| **+26** | 228 |
| **+39** | 125 |
| +476 | 3 |
| +483 | 60 |
| +484 | 1 |
| +489 | 1 |
| **+490** | 103 |
| **+518** | 117 |
| **+648** | 120 |

Two things shrink that number before anyone starts:

- **336 of the 762 are in `cartalith-native/docs/CHANGELOG.md`, which is retired** — frozen at
  2026-08-26 and explicitly not to be appended to or trusted as current. Its citations are history and
  must stay pointing at v2.10. That leaves **426 in live documents**, the largest holdings being
  `URBAN_MORPHOLOGY_SCOPE.md` (62), `PHASE2_SCOPE.md` (49), `UNIFIED_TOOL_PLAN.md` (41),
  `GUI_GAP_REGISTER.md` (39) and `ASSET_LIBRARY_SCOPE.md` (34).
- **353 of the 762 (46%) need only +26 or +39.**

Correcting them is a separate pass; this document exists so that pass is arithmetic against §3 rather
than a re-diff. **A citation left pointing at v2.10 is still correct** — that file was kept — so the
correction is optional per document, not a repository-wide obligation.

---

## 3. The line map — 16 segments with an exact, constant offset

Every line in a segment below maps 1:1. Lines *between* segments were rewritten and have no
correspondence (they are §4's real changes). Derived from the `difflib` equal-blocks of a full
line-level diff, not estimated.

| v2.10 lines | offset | v2.11 lines |
|---|---|---|
| 1–2098 | +0 | 1–2098 |
| 2100–6848 | **+26** | 2126–6874 |
| 6849–12622 | **+39** | 6888–12661 |
| 12623–12628 | +436 | 13059–13064 |
| 12631–12644 | +470 | 13101–13114 |
| 12645–12719 | +476 | 13121–13195 |
| 12720–16214 | **+483** | 13203–16697 |
| 16216–16291 | +484 | 16700–16775 |
| 16292–16722 | +489 | 16781–17211 |
| 16723–22310 | **+490** | 17213–22800 |
| 22314–22315 | +487 | 22801–22802 |
| 22318–22459 | **+501** | 22819–22960 |
| 22464 | +518 | 22982 |
| 22466–22469 | +517 | 22983–22986 |
| 22470–26305 | **+518** | 22988–26823 |
| 26306–31107 | **+648** | 26954–31755 |

Lines with **no** v2.11 counterpart (rewritten or deleted): 2099, 12629–12630, 16215, 22311–22313,
22316–22317, 22460–22463, 22465.

**The table is checked against both indexes, not just asserted.** For all **1 094** functions the two
versions share, `FUNCTION_INDEX.md`'s line **plus this table's offset** equals
`FUNCTION_INDEX_v2.11.md`'s line — 1 094 of 1 094, zero mismatches. Applying an offset from this table
to a v2.10 citation is therefore known to land, not hoped to.

Script-block boundaries moved with it:

| Block | v2.10 | v2.11 |
|---|---|---|
| 1 — engine + app shell | 2083–14557 | 2083–15040 |
| 2 — civilization | 14562–26721 | 15045–27369 |
| 3 — Asset Library | 26722–28162 | 27370–28810 |
| 4 — urban morphology (UME) | 28166–31104 | 28814–31752 |

**Lines 1–2082 are byte-identical at every line but two** — line 6 and line 510, both the version
string. The static markup `FUNCTION_INDEX.md` Part 0 indexes did not move at all, so **every DOM `id`
and every Part 0 line range is still correct, unshifted.**

---

## 4. What actually changed

### 4.1 Added: 14 functions, none removed, none renamed

| v2.11 line | Block | Function | What it is |
|---|---|---|---|
| 6884 | 1 | `invalidateDerived` | The derived-affordance-cache null list, in one place |
| 12712 | 1 | `_tText` | UTF-8 decode of a tree entry, BOM-tolerant |
| 12717 | 1 | `_tInt` | Integer-valued JSON number or `null`; enforces the 2⁵³ range and the 1.0-reads-as-1 rule |
| 12720 | 1 | `_tNum` | Float member with a default; `null` means absent, not zero |
| 12723 | 1 | `_tStr` | Free text; empty string preserved |
| 12727 | 1 | `_tDoc` | Parse one document, warn-and-skip on damage |
| 12733 | 1 | `_tSparse` | i32 raster → the sparse `[index,value,…]` pairs territory/province use |
| 12751 | 1 | `_treeRead` | **307 lines.** Translates the port's project tree into the `{GW,GH,state}` shape `loadZip` already gets from a flat `params.json` |
| 26851 | 2 | `_treeRestore` | Fourth `loadZip` monkey-patch link: restores tree-carried provinces and author faction colours |
| 26903 | 2 | `_vaultStore` | `state.vault` if it holds a link array |
| 26904 | 2 | `_vaultLinksFor` | Vault links for one entity: by id, then by name |
| 26915 | 2 | `_vaultLinkHtml` | One link's imported frontmatter/template fields |
| 26927 | 2 | `_vaultLinksHtml` | The "Vault notes" block for an inspector |
| 26937 | 2 | `_vaultSummaryHtml` | Factions-overview counts, including links that no longer resolve |

Four new top-level constants come with them and are *not* in either index, which by construction
indexes function-valued declarations only: `TREE_MODELLED` (12744), `TREE_SETTLE_KINDS` (12748) and
`TREE_ROAD_CLASSES` (12749) in block 1, and `CIV_BASE_COLORS` (26850) in block 2. All four are new;
none appears in v2.10.

### 4.2 Bodies: 57 of the 1 094 carried-forward functions changed; 1 037 are byte-identical

Every one of the 1 037 moved and nothing else — same normalised text, new line. Of the 57:

| Cause | Functions |
|---|---|
| `state.seaLevel\|\|0.42` → `state.seaLevel` and nothing else | **44** |
| inlined cache-null list → `invalidateDerived()` and nothing else | **7** |
| substantive | **6** (two of which also carry the `seaLevel` edit) |

**The `seaLevel` edit: 51 occurrences gone.** 50 were rewritten in place at 49 hunk sites — line
20451 carried two of them on one line — spread over 46 top-level functions plus one site at top level
(v2.10 line 14301, a GL uniform outside any function). The 51st vanished with the block deleted in
§4.4. Of the 46 functions, 44 changed for this reason and nothing else; the other two are
`_umWaterCtx` and `_civRiverPolylines`, which also carry §4.3's real changes. The
**This was a live bug, not tidying, and 0.42 was never the point.** The file's own comment at v2.11
line 13196 names it: 50 sites carried the fallback against ~130 that did not, and **`0` is a legal
slider position** (`min="0"`), which `||` treats as absent. So at Sea level 0% the two halves of the
file disagreed about where the sea was — the renderer's `isWater(v){return v<state.seaLevel}`
correctly drew an all-land map while `_civDropPlace` computed `sea=0.42` and refused to place a
settlement on most of the land you could see. `state.seaLevel` is set in the state literal and
re-established by every load, *"so it is never actually undefined and the fallback was only ever
defending against its own zero."*

**Porting note.** Rust has no `||`-truthiness, so this cannot be inherited mechanically — only by
someone writing `unwrap_or(0.42)` or an `if sea == 0.0` guard by hand. Checked: no such construct
exists in `crates/*/src`. The `sea: 0.42` in `cartalith-assets::placement`'s `Default` is a different
thing and is correct — it mirrors the reference's own `Object.assign({sea:0.42, …})` options default,
which its doc comment already cites. **The exposure to check is the port's own sea-level-0 behaviour,
not a constant.**

**`invalidateDerived()` — 8 call sites**, seven inside functions and one at top level:

| v2.10 → v2.11 | In |
|---|---|
| 3194 → 3220 | `centerLandmasses` |
| 3355 → 3381 | `generate` (block 1) |
| 4865 → 4891 | `computeFlow` |
| 4911 → 4937 | `invalidateFieldCaches` |
| 5137 → 5163 | `computeTemperature` |
| 5676 → 5702 | `simulateWeather` |
| 6780 → 6806 | `inferTectonics` |
| 12732 → 13215 | top level (the sea-level slider handler) |

The eight sites nulled the same eight caches: `_resourcePots`, `_carryCapField`, `_settleSuitField`,
`_wildlife`, `_nppField`, `_triField`, `_popDensityField`, `_wetlandMask`. The file notes the
`_biomeK` toggle deliberately keeps its own narrower four-cache list — *"that narrower list is
correct, not drift"*.

### 4.3 The six substantive body changes

| v2.10 → v2.11 | Function | Change |
|---|---|---|
| 12623 → 13059 | `loadZip` | 25 → 65 lines. Detects the project tree by a single `project.json` lookup and routes through `_treeRead`; collects a `notes[]` array and reports skipped/unrecognised content in one `alert` at the end instead of dropping it |
| 16202 → 16685 | `_civRenderFactionsWorldOverview` | Appends `_vaultSummaryHtml()` on both the empty-world and populated paths |
| 16247 → 16731 | `_civPopulateFactionEditor` | Injects `_vaultLinksHtml('faction', …)` and `_vaultLinksHtml('culture', …)` |
| 16694 → 17183 | `_civPopulatePlaceEditor` | Injects `_vaultLinksHtml('settlement', …)` |
| 22300 → 22790 | `_umWaterCtx` | 90 → 101 lines. **Its duplicated inline river trace is deleted; it now calls `_civRiverPolylines()`** |
| 22464 → 22982 | `_civRiverPolylines` | **Cache key changed from `_fieldGen` to the identity of `_riverNet`** |

### 4.4 The one change with parity consequences: the river-polyline cache

Worth reading in full at v2.11 lines 22961–22980, because it corrects a comment that was itself wrong.
Two coupled defects:

1. **`_umWaterCtx` re-traced the whole world's rivers on every call.** The v2.10 comment claimed this
   was fine because "its result is itself cached per settlement". That was false: `_umModelCache` sits
   *behind* `_umWaterCtx`, not in front of it — the model-cache key fingerprints the water mask, so
   the water context must be built before the cache can be consulted. Nothing memoised the trace.
2. **`_civRiverPolylines` was keyed on `_fieldGen`, which was wrong in both directions.** Of the five
   sites that null `_riverNet`, three bump `_fieldGen` and two do not (`riverDensR`'s slider, and the
   post-carve `computeFlow(true)`) — so the cache served stale channels after a river-density change.
   And two sites bump `_fieldGen` without touching `_riverNet` (the biome/terrain-paint reset, the
   geoid offsets) — so it also re-traced when no river had moved.

v2.11 keys on `_riverNet` itself: `let _civRiverPolys=null, _civRiverPolysSrc=null;` and
`if(_civRiverPolys&&_civRiverPolysSrc===_riverNet) return _civRiverPolys;`. **No call site was
edited.** It does not change what a cold computation produces — only whether a *cached* one is
correct.

**The Rust port did not inherit either defect, and this was checked rather than assumed.**
`cartalith-civ::urban_adapter` has no `_fieldGen`-keyed polyline cache at all: `UrbanWorld::river_polys`
is a caller-supplied `&'a [Vec<(f64, f64)>]`, and its own doc comment already gives v2.11's reason —
*"a run of towns would repeat it once per town for an identical answer. The call is unchanged and so
is its result — only where it is made."* The port hoisted the trace out of `um_water_ctx` to the
caller; v2.11 memoised it behind `_civRiverPolylines`. Different mechanisms, same defect closed. Only
two Rust files mention `_fieldGen` at all, both in doc comments describing the reference
(`urban_adapter.rs` at `civ_coast_dist_field`, `cartalith-civ/src/lib.rs` on the aggregate cache key)
— **neither implements it.**

### 4.5 The reference documents its own re-baseline

v2.11 lines 2099–2124 are a new `CHANGELOG — v2.11 (re-baseline, disclosed)` block above
`const VERSION='2.11'`. It states the two capabilities are **read-only**, that `exportZip()` still
writes the flat layout byte for byte, and that closing that half is a separate change. It cites
`SAVEFILE_COMPAT.md` and `DECISIONS.md` §7h — *this repository's* documents. Read it before treating
any of §4 as a surprise.

---

## 5. How this was produced, and how to re-run it

- **The mechanical scan was recovered, not reinvented.** `FUNCTION_INDEX.md` says its name→line scan
  is mechanical but no generator was kept. A scanner was written and validated by requiring it to
  reproduce v2.10's Part 2 **exactly**: 1 094 rows, 633/350/19/92, every name and every line number,
  zero extra and zero missing. Its rule is top-level `^function` / `^async function` /
  function-valued `^const` (including the four IIFE forms `BIOME_INDEX`, `KOPPEN_INDEX`, `CRC_T`,
  `UME`) inside a `<script>` block. Part 2's sort key was recovered the same way, by exact
  reproduction: case-insensitive with `_` ordering below digits, tie-broken on block.
- **Body identity** is a SHA-1 of the whitespace-normalised brace-matched body (JS-aware: strings,
  template literals, comments and regex literals skipped). Twelve brace-less one-line arrows in each
  file are clamped to the next declaration — identically in both, so the comparison stays fair.
- **`FUNCTION_INDEX_v2.11.md` Part 1's purpose column is the 2026-08-23 analyst pass's work, carried
  forward.** All 1 094 lines are re-lined and otherwise untouched; only the 14 new functions have
  purposes written in this pass, from reading each body.

Checks run on the new reference, so it is known usable and not merely present:

- All four v2.11 script blocks pass `node --check`.
- `deflectFlow` (v2.10 5315–5357) and `blurCoarse` (5543–5548) — the slices
  `cartalith-native/tools/jsruntime_probe.js` pins — are **byte-identical** in v2.11 at 5341–5383 and
  5569–5574, i.e. the `+26` offset and nothing more.
- `node tools/jsruntime_probe.js` still passes against the untouched v2.10 file:
  `PROBE PASSED: node v24.19.0 runs the frozen reference, and the committed fixtures are its real output.`

---

## 6. What this pass deliberately did not do

- **No scope document was edited.** Correcting the 426 live citations (§2) is a separate pass and
  would have collided with three lanes slicing v2.10 concurrently. §2 and §3 are that pass's input.
  Nothing is broken meanwhile: v2.10 is still there, so every existing citation still resolves.
- **The recovered scanner was not committed.** It lives only in this pass's scratch. §5 states its
  rule and its sort key precisely enough to rebuild, and the validation bar to rebuild it against is
  "reproduces v2.10's Part 2 exactly". Committing it beside the indexes would close this loop for the
  next re-freeze and is a one-file change nobody has asked for yet.
- **`FUNCTION_INDEX.md` (v2.10) was not modified**, and neither was
  `reference/Cartalith Gen1 v2.10.html`.
- **`FUNCTIONAL_CONTRACT.md` was not re-tagged.** Its capability tags are still measured against
  v2.10. The v2.11 delta for it is small and specific — the project-tree reader and the read-only
  Vault surfaces of §4.1 are new legacy-side capabilities that no tag covers.
- **`OUTSTANDING_WORK.md` §2.8, `STATUS.md` and the retired `CHANGELOG.md` were not touched**, per the
  standing instruction that a later pass owns them.
