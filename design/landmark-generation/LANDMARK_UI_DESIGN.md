# Landmark generation — UI design

**What this is.** A design for the landmark generator's controls: where the
panel lives, what the per-type control is, and how the cap-versus-quota
distinction the owner asked for is made visible rather than merely explained.

**The ask, verbatim** (owner, 2026-08-30):

> design me layouts for the landmark (aka Point of Interest) generation and how
> you would make a submenu. I'd expect with per item a slider to make a type
> active and how much the generator could maximally place (not that this should
> be taken to the letter: a maximum number does not mean that the maximum number
> of a certain type should be placed. That is where the spacing calculation
> should give the restraint.

Inputs: `LANDMARK_GENERATION_RESEARCH.md` (owner-supplied, imported verbatim
2026-08-30 — §7, §8, §16, §17, §21, §22, §23, §29 are the sections this design
answers to), `DCC_SHELL_SPEC.md` (§2 menus, §3 domains, §5.1 pipeline, §11
tokens, §12 iconography, §13 touch), `UI_SHELL_DESIGN.md` (the disclosure
grammar), `design/Cartalith DCC Shell.dc.html` and
`design/android-2026-08-30/Cartalith Android.dc.html` (the visual vocabulary).

Artboards, laid out by `canvas.json` across three pages:

| File | Size | Shows |
|---|---|---|
| `Dock.dc.html` | 1920 × 1080 | the whole shell, CIVIL ▸ Landmarks open, PHYSICAL expanded, rows in all three states, exclusion rings drawn in the viewport |
| `Submenu.dc.html` | 1920 × 1080 | `Assets ▸ Landmark types ▸ Physical ▸` — three cascade levels in §2's style |
| `TypeRow.dc.html` | 900 × 820 | Waterfall and Mountain pass, both expanded to L5, annotated with what the engine can answer |
| `WhyFewer.dc.html` | 960 × 620 | the funnel popover — 1 284 candidates down to 11 |
| `Phone.dc.html` | 412 × 892 | half detent: six families, not forty-nine types |
| `PhoneTypes.dc.html` | 412 × 892 | full detent: one family's rows at 56 dp |

**Two disclosures before anything else.**

1. **`docs/ANDROID_UI_SPEC.md` and `docs/DCC_SHELL_SPEC.md` were not read from
   the design project.** The DesignSync tool is not present in this session's
   tool set, so the phone rules below are derived from the three newest things
   in this repository that *can* be read: the vendored `DCC_SHELL_SPEC.md` §13
   (2026-08-25, whose phone column is itself marked superseded),
   `design/android-2026-08-30/README.md` (which paraphrases the Android spec's
   locked decisions and carries the chosen chrome's exact values), and
   `design/phone-redesign/canvas.json`'s TARGETS annotation. Where those three
   disagree with the live spec, the live spec wins and this document is the
   stale party. Every phone figure below names which of the three it came from.
2. **`LANDMARK_GENERATION_SCOPE.md` was written concurrently with this document
   by another agent and is not cited here.** Every engine claim in §9's wiring
   table was re-derived by reading `cartalith-native/crates/` directly, so the
   two documents are independent readings of the same code rather than one
   quoting the other. If they disagree, that disagreement is information.

---

## 1 · Where this lives

### 1.1 The decision

**CIVIL ▸ Landmarks** — the existing v3 category `Points of interest`, renamed,
with its "Not built" stub replaced. One panel, one run, both physical and
cultural landmarks in it.

### 1.2 What was rejected, and why

**CARTO ▸ Assets & landmarks — rejected.** This is the tempting answer, because
the category is already called *Assets & landmarks* and
`civilization_workspace.gd:1640` already sends users there. But CARTO is
presentation, and `DCC_SHELL_SPEC.md` §4.5.5 states the exact test a control
must pass to live there: labels and icons are allowed as *"the exception §7's
prohibition allows, because they add nothing to and take nothing from the world
model."* A generated landmark fails that test in both directions. §22's object
model gives it `physical_basis`, `settlement_associations`,
`political_associations`, `causal_chain` and an emergent `importance` — it
*takes* from the world model at generation time and *adds* to it as something
other systems can read. Put it in CARTO and the rule that keeps the world model
out of the style workspace has its first hole. That rule is load-bearing:
`UI_SHELL_DESIGN.md`'s governing split is the whole reason the shell was
rebuilt.

What CARTO keeps: the *icon* a landmark type draws with, its label style, and
its zoom visibility. Those are presentation and they stay where presentation
lives.

**A new WORLD pipeline stage (an eleventh stage) — rejected, on a measured
consequence rather than on taste.** §21's pipeline reads exactly like WORLD's
dependency chain and physical landmarks are terrain-derived, so this is the
strongest rejected candidate. It fails on how WORLD parameters actually behave
in this shell. `world_workspace.gd:1113`'s `_on_float_row_released` calls
`_mark_stale_from` and then `_regenerate_live` (`:1140`), which calls
`bridge.generate(...)` — the **whole world, from stage 01**. That is not an
implementation shortcut; it is the reference's own `tparam()` behaviour,
verified with Playwright and recorded as correction 2 at the top of
`DCC_SHELL_SPEC.md`: *"`generate()` is monolithic (runs all ten stages,
unconditionally, every call — no branch skips any of them)"*. Put the landmark
caps in WORLD and nudging **Waterfall 40 → 41** re-runs tectonics, erosion,
hydrology and climate. With ~49 sliders in the panel that is not a slow UI, it
is an unusable one.

**Split — physical to WORLD, cultural to CIVIL — rejected**, for three
independent reasons, any one of which is sufficient:

- §35 is explicit that the chain is one chain: *tectonic uplift → high relief →
  steep river gradient → waterfall → visible natural feature → cultural
  significance → shrine*. A split panel puts the waterfall's cap in one dock and
  its shrine's cap in another, and the causal arrow between them crosses a
  domain boundary the user has to click through.
- §16's exclusion radius is **not per family**. `r = f(class, importance,
  terrain, region)` — a shrine and a waterfall compete for the same ground under
  the same constraint. A split panel has two spacing budgets over one field and
  no honest place to show the total.
- It inherits the WORLD regenerate problem for exactly half the types.

**Its own domain on the rail — rejected before it was considered.** §3: *"Nothing
else is a workspace."* Three domains, and the 2026-08-20 merge went the other
way — five to three.

### 1.3 Why CIVIL is right, positively

- **v3 already put it there.** `DCC_SHELL_SPEC.md` §3's v3 table lists CIVIL's
  fifth L2 category as *Points of interest*. This design fills a category the
  menu structure already declared, rather than opening a new one — and it
  replaces a stub whose own text (`civilization_workspace.gd:1628-1639`) says
  the concept is not built. That stub is currently one of the few places in this
  shell where a category exists with nothing behind it; closing it is a net
  reduction in unbacked surface.
- **CIVIL sits at the right point in the dependency order.** v3's own direction
  is WORLD → CIVIL, and both → CARTO. Landmarks consume terrain fields *and*
  settlements, routes, factions and resources — that is CIVIL's position
  exactly, and §23's fourth class is literally *"Cultural — dependent on
  civilization."*
- **CIVIL has the right recompute.** `recompute_civilisation`
  (`cartalith-godot/src/lib.rs:3360`, exposed as `engine_bridge.gd:1733`) is an
  explicit, button-driven pass — `civilization_workspace.gd:1096`'s
  `_build_recompute`. Its own tooltip measures it at *"about 1.0 s at 512², 1.6 s
  at 1024² and 4.2 s at 2048², roughly half the cost of a full Generate"*, and
  says why it is a button: *"That is why it is a button and not an automatic
  cascade after every brush stroke."* A landmark pass is the same shape of work
  and inherits the same, already-argued, already-shipped interaction model.
- **A button-driven pass is what makes the cap readout possible at all.** "11
  placed" is a fact about the *last run*. A panel whose every slider silently
  re-ran the pass would have no "last run" to report and no stable number to
  compare the cap against. The interaction the engine forces is also the one the
  design needs.

### 1.4 What the panel replaces, exactly

`civilization_workspace.gd:1625-1642`, `_build_poi` — a `Not built` section
carrying two notes and one jump button. The two notes go. The jump button's
*destination* stays true: hand-stamped icons remain Cartography's annotation.

**These two lists do not merge on the day the entity lands.** v3 wants *"an icon
on the map becomes a POI entity, not a decoration"* and the stub quotes it. The
honest reading is that a hand-stamped icon has no `causal_chain` and no emergent
`importance` — the two fields §22 and §24 say a landmark *is*. So the design
gives a hand-placed landmark an explicit `causal_chain: "placed by hand"` and
sorts it into its own `Placed by hand` group at the foot of the results list,
never mixed into a generated family's counts. A generated 11 and a hand-placed 3
are different claims and the panel never adds them together.

---

## 2 · The per-type control

The owner asked for *"a slider to make a type active and how much the generator
could maximally place."* One slider, two jobs.

### 2.1 One slider, zero means off — but the store keeps two fields

**The control is one gesture.** Dragging the track to its zero stop disarms the
type; dragging up from zero arms it. No checkbox column, no separate arm toggle.
The zero stop is detented and the readout at that position reads the word `off`,
not the number `0` — because "zero waterfalls" and "waterfalls disabled" are the
same outcome and there is no reason to make the user say it twice.

**The store is two fields**, and this is not a contradiction — it is what stops
a papercut this codebase has already met. `ScatterRule`
(`cartalith-assets/src/scatter.rs:133`) keeps `enabled: bool` and `density: f64`
as separate fields for the identical reason: a user who has tuned a value and
wants to *briefly* switch the thing off should get their number back. So the
model is `{ armed: bool, cap: u32 }`, the control writes both, and the retained
cap is made visible rather than hidden:

```
○  Waterfall                                 off · was 40
```

`was 40` is not a control. It is a two-word promise that the number survived,
and it costs one dim mono span. Drag up from `off` and the slider resumes at 40.

### 2.2 The resolved readout — the crux

Every armed row carries a second line, in `IBM Plex Mono` 9.5 px, `#5f6468`,
that says what the last run actually did:

```
●  Waterfall        ▓▓▓▓▓▓▓▓░░░░░░        40 max
   ▔▔▔▔                                   11 placed · spacing
```

Three parts, each doing one job:

**1 — the placed under-bar.** A second 2 px rule sits directly under the slider
track, its length the *placed* count as a fraction of the cap. Two bars, same
origin, different lengths. The gap between them **is** the cap-versus-quota
distinction, rendered, on every row, with no reading required. When placed
equals cap the two bars are flush and the row visibly "tops out". Colours are
already in the palette: cap fill `#e0a34a` (accent, the standard slider fill),
placed bar `#8d9296` (ink dim). No new token.

**2 — the count.** `11 placed`. Plain.

**3 — the limiting reason, and this is the part that matters.** One word after
the count, naming what actually stopped the generator:

| Token | Means | Does dragging the slider right help? |
|---|---|---|
| `at cap` | the cap was the binding constraint | **yes** |
| `spacing` | the exclusion radius rejected the rest | no — lower Crowding instead |
| `no terrain` | every remaining candidate failed this type's own constraints | no — relax the type's constraints |
| `candidates` | the candidate pool was exhausted before either | no — the world is too small or too coarse |

The design goal is not "explain the algorithm". It is: **the row answers "will
dragging this right change anything?" before the user drags it.** Every reason
other than `at cap` is the panel saying *the cap is not what is limiting you*,
which is the sentence the owner's brief is entirely about. `at cap` is drawn in
accent; the other three in ink-dim — so a panel where nothing is at cap has no
accent on any second line, and a user who has genuinely maxed something sees it
immediately.

`spacing` additionally makes the row's own spacing control the obvious next
thing to touch, because it is the word under the finger.

### 2.3 Why not a separate arm toggle

Considered and rejected. A checkbox plus a slider is two controls, two hit
targets, and one redundant state — an armed type with a cap of zero, which means
nothing and which a user *will* produce. Collapsing them removes an
unrepresentable state rather than hiding a real one. The owner asked for one
control and one control is also the correct answer.

The one thing lost is the ability to arm a type without committing to a number,
and that is not a real loss: arming a type *is* committing to a number, because
the number is what the generator reads.

### 2.4 The cap's scale

A linear 0–200 slider spends most of its travel in a range nobody wants. The
track is **perceptually spaced**: `off · 1 · 2 · 3 · 5 · 8 · 12 · 20 · 30 · 50 ·
80 · 120 · 200`, a rounded 1–2–3–5 ladder. Thirteen detents, each a real
decision, and the difference between 1 and 2 gets as much travel as the
difference between 120 and 200 — which is right, because for a Continental-class
type the difference between one and two is the whole design of the world and the
difference between 120 and 200 is meaningless. The exact number stays typeable
in the row's expanded numeric field (§3.2), so nothing is unreachable.

---

## 3 · Disclosure for ~50 types

§29 lists **49** types in six families: Physical 15, Transportation 8, Economic
6, Military 6, Religious/Cultural 8, Historical 6. §23 grades them four ways:
Continental, Regional, Local, Cultural. Those two schemes are **orthogonal** — a
waterfall is Physical by family and Regional-or-Local by class — and nesting
both would produce a six-level tree, which `UI_SHELL_DESIGN.md` forbids outright:
*"A sixth level means the L2 category is wrong and should be split."*

### 3.1 Family groups, class badges

**Family is the grouping. Class is a badge on the row.**

| Level | Here |
|---|---|
| L1 domain | CIVIL |
| L2 ▾ category | **Landmarks** (renames v3's *Points of interest*) |
| L3 § section | `PLACEMENT` · `TYPES` · `LAST RUN` — always expanded |
| L4 › group | the six §29 families, one open at a time by default |
| L5 + advanced | one type row's own constraints, closed by default |

Exactly five levels. The type row is a row inside L4, not a level of its own —
its `+` fold is L5, which is where §7/§8's type-specific constraints live.

The class shows as a three-letter mono badge in the row's left gutter — `CON` /
`REG` / `LOC` / `CUL` — at 9 px, `#5f6468`, and four filter chips at the head of
`§ TYPES` dim the non-matching rows. §23's *"This hierarchy should determine both
generation frequency and map visibility"* survives intact as a property of the
row rather than as a second tree the user has to navigate.

### 3.2 Group headers report while collapsed

A collapsed group is not silent:

```
›  PHYSICAL                          6 of 15 armed  ·  74 placed
```

So a user who opens the panel with everything collapsed can already see which
families produced their landmarks, and where the map's 187 markers came from,
without expanding anything. This is the same principle as `Assets ▸ Icon
families ▸ (24 families with filled/capacity counts)` — a count on a closed
container is worth more than the container.

Each group header also carries one bulk gesture: `arm all` / `off`. With 49
types, "turn the whole military family off for a moment" is a real thing to want
and stepping thirteen detents six times is not the way to do it. This is a
bulk *operation*, not a second copy of the row control — the same distinction
`Assets ▸ Asset pack ▸ Batch` already draws.

### 3.3 "I want more ruins" — three steps

1. **CIVIL** on the domain rail (skipped if already there)
2. **Landmarks** category
3. **Historical** group — `§ TYPES` is L3 and always expanded, so it costs
   nothing

The Ruin row is now visible. Three steps, or two from inside CIVIL.

**A fourth path costs one step and is free.** `command_index.gd` builds its
search list from `EngineBridge.param_keys()` / `param_info(key)` — *"a
hand-maintained catalogue of an app's own features is the most reliably stale
document a project can own. This one cannot disagree with the app, because it is
assembled from the app."* If the caps are `ParamSpec`-shaped rows (§9 argues
they should be), typing `ruin` into the search finds the row with no work at all,
on desktop and on the phone. This is the strongest single argument for the data
shape in §9.2, and it is the phone's answer to fifty types.

---

## 4 · The spacing control

§16 asks for `r = f(class, importance, terrain, region)` via Poisson-disc
sampling. A user must be able to tune that without meeting the phrase
"Poisson-disc" or the letter `r`.

### 4.1 One dial named after its effect

`§ PLACEMENT` carries **Crowding**, a single slider from `sparse` to `dense`,
0.25× to 2.00× on every class radius at once. Its readout converts the
multiplier into the user's own units:

```
Crowding          ▓▓▓▓▓▓▓░░░░░░░       × 1.00
                  a regional landmark keeps 34 km clear
```

The second line is the whole point. `× 1.00` is arithmetic; `34 km` is a fact
about the map the user is looking at, in the units `Preferences ▸ Units` is
already set to. Nobody needs to know what the multiplier multiplies.

### 4.2 The four class radii, as L5

Under `+ advanced`, four rows in km — Continental, Regional, Local, Cultural —
which is §23's four classes and §16's own worked example (*"Minor landmark →
small exclusion radius … World landmark → very large exclusion radius"*). The
Crowding dial scales all four. One dial for everyone; four for the person who
wants a world where regional landmarks crowd and continental ones do not.

### 4.3 The one toggle that changes the meaning of the whole panel

```
☑  Types compete with each other
   Off lets a shrine sit beside a waterfall. On keeps every landmark
   clear of every other one.
```

This is the single most consequential decision in §16 and it has no jargon in
it. On: one disc field over all types, and a dense Physical family genuinely
crowds out the Religious one — which is §16's own intent (*"This prevents
procedural landmark saturation"*). Off: per-type fields, and the families stop
interacting. Both are legitimate worlds; the difference is enormous; the sentence
is nineteen words.

### 4.4 The headroom line — the panel-scale answer

At the head of `§ PLACEMENT`, one line, always visible:

```
caps total 640  ·  room for about 210 at this spacing  ·  last run placed 187
```

This is the arithmetic that explains everything below it. A user whose caps sum
to 640 and whose world placed 187 has the reason on screen *before* they run and
*before* they hunt for a broken slider. It is a packing estimate from land area
and the mean class radius, not a promise — the word `about` is doing real work
and stays.

Two scales, one message: **the headroom line stops "the generator is broken" at
the panel; the per-row reason token stops it at the row.**

### 4.5 Per-type spacing

Each row's L5 fold carries `Minimum separation`, in km, defaulting to its class
radius and showing that inheritance in place (`34 km · from class REG`). Editing
it detaches the row from the class and the readout says so (`18 km · overridden`).
This is §16's `f(class, …)` made editable at the one granularity where a user has
an opinion — "I want waterfalls closer together than other regional landmarks" —
without exposing the function.

---

## 5 · "Why fewer than I asked for" — the explainer

`WhyFewer.dc.html`. Clicking a row's reason token opens a small popover with the
funnel for that type, from the last run:

```
WATERFALL · LAST RUN

  candidates evaluated              1 284
  failed min flow accumulation      −  902        382 left
  failed min drop 40 m              −  247        135 left
  rejected by spacing               −  124         11 left
  cap 40                            not reached

  11 placed
```

Four rows, one arithmetic, no prose. It is deliberately the same shape as
`explain_settlement` (`cartalith-godot/src/lib.rs:4920`), which already returns
*"every weighted term, sorted most-decisive first"* plus an `excluded` reason,
and whose doc comment states the division this popover keeps: *"All wording is
left to the caller — this returns facts, not prose."* The engine returns the
five integers; the shell writes the five labels.

This is the design's answer to the brief's hardest sentence — *"a user who drags
Waterfall to 40 and gets 11 must be able to see why without asking."* The
under-bar shows it at a glance, the reason token names it in one word, and the
popover proves it in five numbers. Three depths, and the user picks how far they
want to go.

---

## 6 · The submenu

The owner asked *"how you would make a submenu"*, and the brief asks for the
menu-bar route drawn as a `PopupMenu` cascade. Drawing it surfaced a real
constraint, recorded rather than worked around.

### 6.1 What §2 forbids

> The menu bar holds *program* functions. World generation, simulation,
> rendering and map styling are workspaces reached through the domain rail (§3),
> **never menu items**.

So there is no top-level `Landmarks` menu, and **no `Run landmark pass` item**.
Running the generator from the menu bar is precisely the thing §2 exists to
prevent, and the rule has already survived one deletion pass on this evidence
(§2.4's Conversion group, removed 2026-08-20 for a structurally identical
reason: a route that was really a parameter).

### 6.2 What §2 permits, with a precedent to match

`Assets ▸ Icon families ▸` is the shape: *"Submenu listing the 24 families with
filled/capacity counts; picking one opens the library scoped to it."* A
navigation cascade with live counts, whose leaves are destinations rather than
controls. `menus.gd:685` builds it today.

So: **`Assets ▸ Landmark types ▸`**, three levels.

```
Assets ▸
  Landmark types ▸
      PHYSICAL              6 of 15 armed · 74 placed   ▸
      TRANSPORTATION        3 of 8  armed · 21 placed   ▸
      ECONOMIC              2 of 6  armed ·  9 placed   ▸
      MILITARY              4 of 6  armed · 33 placed   ▸
      RELIGIOUS · CULTURAL  5 of 8  armed · 41 placed   ▸
      HISTORICAL            3 of 6  armed ·  9 placed   ▸
      ─────
      Landmark icons…                    poi · 10 slots
      Landmark label style…              → Cartography
```

and the third level, per family:

```
      PHYSICAL ▸
          ● Peak                    12 max · 12 placed · at cap
          ● Waterfall               40 max · 11 placed · spacing
          ○ Cliff                          off · was 20
          ● Gorge                    8 max ·  6 placed · no terrain
          …
```

Picking any row calls `select_domain_category("civilization", "Landmarks")` and
scrolls to it — `dcc_shell.gd:1763` already does exactly this, and
`faction_roster_window.gd:682` is a shipped caller.

### 6.3 The third level is read-only, and that is the design

The leaves show `armed · cap · placed · reason` and do **not** toggle. This is
`UI_SHELL_DESIGN.md`'s own rule — *"The dropdown that opens a window is a
shortcut into it, never a second implementation of it"* — and it is also just
right: forty-nine numbers do not belong in a menu, and a menu row that arms a
type would be a second store to keep in sync with the dock's slider.

What the cascade *is* good for, and what the dock is bad at, is **the
world-at-a-glance read**. Every family, every armed type, every placed count, in
one hover, without scrolling a dock or expanding a group. That is a genuine
complement rather than a duplicate.

**Assets is the right menu**, not Data: the two non-family rows at the foot are
genuinely Assets business — which of `LIBRARY_POI_SLOTS`' ten `poi` icons a
landmark type draws with (`cartalith-assets/src/library.rs:70`), and where its
label style lives. The cascade sits beside `Icon families` because it is the same
kind of thing about the same kind of content.

---

## 7 · The phone

Per `design/android-2026-08-30/README.md`'s summary of `ANDROID_UI_SPEC.md`
(tabs `MAP · GENERATE · PLAN · MORE`, sheet detents peek/half/full) and its
chosen-chrome values (bottom nav 66 px, active pill `rgba(224,163,74,.16)` at
radius 14, sheet `22px 22px 0 0` on `#15171a`, handle 40×4).

**Forty-nine sliders cannot be a phone screen.** The half detent does not try.

### 7.1 Half detent — six rows, not forty-nine

```
   Landmarks                        187 on the map

   caps total 640 · room for ~210 · placed 187

   Crowding      ────●────────      × 1.00
                 a regional landmark keeps 34 km clear

   PHYSICAL             6/15 · 74 placed          ›
   TRANSPORTATION       3/8  · 21 placed          ›
   ECONOMIC             2/6  ·  9 placed          ›
   MILITARY             4/6  · 33 placed          ›
   RELIGIOUS            5/8  · 41 placed          ›
   HISTORICAL           3/6  ·  9 placed          ›

   [        Run landmark pass        ]
```

Nine rows. The headroom line and the Crowding dial are here because they are the
two controls that move *everything*, and on a phone the global dial is worth more
than any individual cap. Tapping a family pushes the full detent.

### 7.2 Full detent — one family

One family's types, one row each, 56 dp tall (two lines: name + slider, then the
resolved line). At 8 rows for the largest family after Physical, and 15 for
Physical, this is a normal scrolling list rather than a wall.

### 7.3 Targets

Rows are drawn at **56 dp** and the two-line row's own slider hit box at
**48 dp**. This clears the brief's 44 px floor and Android's own 48 dp minimum
simultaneously, which matters because `design/phone-redesign/canvas.json`
records the discrepancy explicitly: *"Built to Android's 48dp touch minimum, not
the 44 the shell uses today — 44 is the iOS number and `DccTheme.PHONE_TAP_MIN`
carries it on an Android-only build."* Drawing to 48 satisfies both specs and
prejudges neither.

No painted status bar and no painted gesture handle — same canvas, same reason:
the OS draws its own and a painted one reads as doubled. The 20 dp gesture inset
holds no target.

### 7.4 The one thing the phone keeps that it could have cut

**The resolved line.** It is the most cuttable element on a 412 dp screen and it
is the thing this whole design is for. It stays at 9.5 px mono. A phone user who
sets Waterfall to 40 and gets 11 is *more* likely to conclude the app is broken
than a desktop user, not less, because they have less context on screen.

---

## 8 · Iconography

§12: no emoji, every glyph a bespoke inline SVG on a 16 × 16 viewBox,
`fill:none`, `stroke:currentColor`, `stroke-width:1.2`, round caps and joins.

**The category reuses an existing glyph.** §12 already assigns *POI — a diamond
with a centre dot*. The Landmarks category is that category renamed, so it takes
that glyph rather than a new one. The centre dot is §12's own sanctioned
exception (*"no fills except 0.7 px dots where a mark must survive at 12 px"*).

**Six new family glyphs**, drawn as one family the way the thirteen sculpt
features are:

| Family | Glyph |
|---|---|
| Physical | A peak with a contour arc beneath it |
| Transportation | A saddle between two rises, with a line through the notch |
| Economic | A trapezoidal ingot with one strike mark |
| Military | A tower with three crenellations |
| Religious · cultural | A stepped arch, open at the base |
| Historical | A broken column — two shafts, the right one snapped |

One weight throughout, nothing inside a glyph smaller than 1 px at 12 px render,
legible in both themes.

**One §12 correction, carried forward.** `DCC_SHELL_SPEC.md`'s own top-of-file
notice records that IBM Plex Mono is missing seven of §12's "stays text" symbols,
checked against the font's cmap: **✕ ● ○ ▾ ▸ ▶ ＋**. The state dots `●` / `○` are
among them, and this design's row state depends on them. **The shipped rows
should draw the state dot as a 0.7 px-dot SVG, not as text** — which §12's own
fill exception already permits, and which is the alternative that notice
explicitly names (*"Drawing them as glyphs is the alternative and is a question
for the design"*). The mockups use the text characters, because the browser falls
them back cleanly and the artboards are about layout; the note is here so the
implementation does not inherit the mockup's shortcut.

---

## 9 · Wiring table

Every control drawn, and what it binds to. `exists` means the binding is
reachable today. `owed` means it is not, and the design draws it anyway with the
gap named — the standing rule being that a drawn control with nothing behind it
is a defect unless it is *disclosed* as one.

All engine claims were verified by reading `cartalith-native/crates/` directly.

### 9.1 Controls

| # | Control | Artboard | Binds to | State |
|---|---|---|---|---|
| 1 | Category `Landmarks` (renames *Points of interest*) | Dock | `civilization_workspace.gd:1626`'s `DccWidgets.category(self, "Points of interest", categories)` — a rename plus a body | **exists** (the shell), **owed** (the body) |
| 2 | Category accordion, groups, sections, `+ advanced` | Dock | `dcc_widgets.gd:25` `category`, `:201` `section`, `:225` `group`, `:263` `advanced` | **exists** |
| 3 | Per-type cap slider | Dock, TypeRow, Phone | `dcc_widgets.gd:349` `slider(…, on_change, tooltip, on_release)` — the `on_release` split `world_workspace.gd:1096` already uses | **exists** (widget), **owed** (the value it writes) |
| 4 | The cap value itself, 49 of them | Dock | Nothing. There is no landmark parameter anywhere. See §9.2 | **owed** |
| 5 | Placed under-bar (the second 2 px rule) | Dock, TypeRow, Phone | Nothing — needs a per-type placed count from a run that does not exist | **owed** |
| 6 | Reason token (`at cap` / `spacing` / `no terrain` / `candidates`) | Dock, TypeRow, WhyFewer, Phone | Nothing yet. **The shape exists**: `stale_stages()` (`lib.rs:3279`, `engine_bridge.gd:1751`) already returns per-stage `reason`/`origin` strings the shell renders as prose (`civilization_workspace.gd:1130-1147`) | **owed** (precedent exists) |
| 7 | Class badge `CON`/`REG`/`LOC`/`CUL` | Dock, TypeRow | §23's hierarchy. Nothing in the engine grades anything four ways | **owed** |
| 8 | Class filter chips | Dock | `dcc_widgets.gd:922` `chip` | **exists** (widget), **owed** (the classes) |
| 9 | Group `arm all` / `off` | Dock | Bulk write over #4 | **owed** |
| 10 | Crowding slider | Dock, Phone | Nothing. Nearest shipped analogue is `ScatterRule::density` (`scatter.rs:153`, `0..3`, *"Above 1 packs tighter: in Relief mode a smaller derived spacing"*) — the same idea, on the icon scatterer, not exposed to any UI | **owed** (analogue exists) |
| 11 | Four class radii (L5, km) | Dock | Nothing | **owed** |
| 12 | `Types compete with each other` | Dock | Nothing | **owed** |
| 13 | Per-type `Minimum separation` (L5, km) | TypeRow | Nothing. Analogue: `ScatterRule::spacing` (`scatter.rs:156`, *"Explicit minimum separation in grid cells; None derives it from density"*) and `spacing_cells` (`:210`) | **owed** (analogue exists) |
| 14 | Headroom line (`caps total · room for · placed`) | Dock, Phone | Land area exists; the packing estimate does not | **owed** |
| 15 | `Run landmark pass` button + busy relabel | Dock, Phone | `civilization_workspace.gd:1154` `_recompute_civ` is the exact pattern (relabel → disable → two frames → blocking call → result note). `bridge.civ_recompute()` = `engine_bridge.gd:1733` → `lib.rs:3360` | **exists** (pattern + host call), **owed** (the pass) |
| 16 | `LAST RUN` result note | Dock | `dcc_widgets.gd:865` `note`; the wording pattern is `civilization_workspace.gd:1169-1173` | **exists** (widget), **owed** (the numbers) |
| 17 | Stale badge above the run button | Dock | `stale_stages()` → `civilization_workspace.gd:1130` `_refresh_staleness`, incl. its 1 s poll timer | **exists**, needs a `landmarks` key |
| 18 | `Why fewer` funnel popover | WhyFewer | Nothing. Shape precedent: `explain_settlement` (`lib.rs:4920`, `engine_bridge.gd:647`) — sorted terms + `excluded` reason, *"returns facts, not prose"* | **owed** (precedent exists) |
| 19 | `Assets ▸ Landmark types ▸` cascade | Submenu | `menus.gd:639` `_assets`, `:685` `add_submenu_item("Icon families", …)` as the precedent; `:726` `_refresh_family_counts` is the live-count pattern | **exists** (mechanism), **owed** (content) |
| 20 | Cascade leaf → dock row | Submenu | `dcc_shell.gd:1763` `select_domain_category(id, category)`; caller precedent `faction_roster_window.gd:682` | **exists** for the category; **owed** for scroll-to-row |
| 21 | `Landmark icons…` menu row | Submenu | `LIBRARY_POI_SLOTS` (`cartalith-assets/src/library.rs:70`, ten slots) and `PACK_POI_SLOTS` (`slots.rs:99`, eight) | **exists** (vocabulary), **owed** (49→10 mapping) |
| 22 | `Landmark label style… → Cartography` | Submenu | `select_domain_category("cartography", "Labels")` | **exists** |
| 23 | Landmark markers drawn on the map | all | `icon_place` (`lib.rs:5816`), `icon_list` (`:5934`), `icon_delete` (`:5923`), `icon_clear_all` (`:5952`) draw *manual* icons. A generated landmark is not one | **owed** (renderer path exists) |
| 24 | `Placed by hand` group in the results list | Dock | `bridge.icon_list()` (`engine_bridge.gd:1644`) already enumerates hand-stamped icons | **exists** |
| 25 | Phone family drill (half → full detent) | Phone | Sheet detents are the Android canvas's own; `dcc_widgets.gd:1218` `phone_window`, `:1270` `phone_present`, `:1041` `phone_pill`, `:1081` `phone_slider` | **exists** |
| 26 | Search finds a type row | — (behaviour) | `command_index.gd`, built from `param_keys()`/`param_info()` | **exists** *if* #4 lands as `ParamSpec` rows — see §9.2 |

### 9.2 The one data-shape recommendation

**Make the 49 caps `ParamSpec`-shaped rows.**
`cartalith-godot/src/params.rs` is a flat, dotted-key table over
`WorldParams`, and its own header argues the case better than this document
can: emitting one `#[func]` per field *"would make the GDScript side hardcode 58
names, 58 ranges, 58 steps and 58 labels a second time — the exact duplication
that lets a slider silently drift from the range the reference actually
shipped … Adding a parameter is one row here, and no GDScript change at all."*

Forty-nine caps is where that argument becomes overwhelming. It also buys #26 for
nothing: `command_index.gd` reads the same table, so every landmark type becomes
searchable by name on both platforms with no further work.

**One caveat, stated rather than glossed.** `params.rs` is a table over
`cartalith_engine::WorldParams`, and `param_set` on a WORLD parameter is what
`world_workspace.gd:1113` turns into a full `generate()`. Landmark caps must
**not** inherit that. They need either a sibling table with its own
`landmark_param_*` accessors, or a `landmark.*` key prefix that the shell binds
without an `on_release` regenerate. The shape is proven; the trigger must not be
copied.

### 9.3 Engine capabilities the design depends on

Verified by direct grep over `cartalith-native/crates/`.

| §  | Capability | State | Where |
|---|---|---|---|
| §6.1 | Flow accumulation | **exists** | `cartalith-hydrology/src/lib.rs:136` `compute_flow`; threshold `:218` `river_flow_thresh`; order `:425` `strahler_from_receivers`. Both are shipped debug layers (`flow`, `strahler`) |
| §8 | Mountain pass / saddle detection | **exists, golden-ported** | `cartalith-civ/src/lib.rs:1601` `build_route_corridors`, from reference line 5903, with `CORRIDOR_KNEE = 0.45` (`:1583`). Its own doc states §8's test exactly: *"a MIN across the two flanking maxima, not a MAX — one steep side is a hillside, two is a pass"* |
| §12 | Least-cost movement | **exists** | The route cost model in `cartalith-civ/src/tools.rs`; `civ_pass_relief` (`:519`) folds the corridor field into the slope term at the reference's own 0.40 pass factor |
| §14 | Resource-driven landmarks | **exists** | `cartalith-civ/src/lib.rs:1234` `build_resource_potentials` — 15 minerals; shipped as the `rsrc` layer |
| §11 | Settlement geography | **exists** | `build_settlement_suitability`, shipped as the `settle` layer; decomposed per settlement by `explain_settlement` (`lib.rs:4920`) |
| §4 | TPI | **exists twice, neither reusable** | `cartalith-civ/src/wildlife.rs:418-440` — a real 4-neighbour Weiss-2001 TPI, but inline inside the ecoregion flood-fill and private to it. `cartalith-godot/src/render.rs:1741` `build_ao` — a **two-scale, RMS-normalised cavity map**, which is a better TPI than §4 asks for (*"compare each cell's height against a blurred version of the same field. Sitting below the local mean means sitting in a hollow"*), but it is `pub(crate)` in the **renderer**. Lifting its cavity computation into a shared field is a small, well-defined job |
| §5 | Curvature | **owed** | Exists only as a *render* term (`curve_shade`, `render.rs:1541`). No shared field, no debug layer |
| §9 | Viewshed / visibility | **owed — and it is the big one** | Nothing anywhere. No `LAYER_GROUPS` entry (`sample_bridge.rs:539`), no line-of-sight, no horizon march, no SVF. The one mention of SVF (`cartalith-engine/src/region_export.rs:29`) is a list of what the reference's *unported* biome tile renderer needs. §18's castle model puts `F_visibility` at **0.20** — the joint-largest term. Six of the 49 types lean on it — **Peak**, **Volcanic feature**, **Watchtower**, **Fort**, **Sacred mountain**, **Border marker** — and until it exists, all six score on a model missing its largest term, and **the panel must say so on the row, not in a footnote** |
| §16 | Poisson-disc (Bridson) | **owed as named; probably should not be built** | No general sampler. But `place_map_icons_ruled` (`cartalith-assets/src/placement.rs:201`) already does bucketed minimum-separation rejection over a candidate set, hardened in reference v1.27 against exactly the NaN-spacing bucket-collapse this would hit (`scatter.rs:56-63`). Landmarks have a **fixed** candidate set — §30 step 5 generates candidates, step 7 scores them, step 8 spaces them — so the correct algorithm is *rejection over a suitability-sorted candidate list*, not dart-throwing into empty space. Bridson generates points where none exist; that is not this problem. Recommend reusing the shipped bucketed test and **not** building Bridson |
| §7 | Waterfall drop + channel confinement | **owed, cheaply** | Gradient and flow exist. "Vertical drop across a channel" and "channel confinement" do not, and both derive from `field` + `flow` with no new retention |
| §22 | The landmark record | **owed** | No POI/landmark record type exists. `civ_tools_bridge.rs:26` states it as a decision: *"Territory only — no `civ_drop_poi`, no fabricated POI record type."* `cartalith-vault/src/links.rs:55`'s `EntityKind` has five variants and a comment saying why POI is not the sixth: *"POIs are not a ported concept in this port at all … building one would be inventing an entity to hang a feature on"* |
| — | Landmark icon vocabulary | **exists** | `LIBRARY_POI_SLOTS` (`library.rs:70`) ten slots — ruin, landmark, mountain_peak, lake, named_forest, battlefield, shrine, cave, bridge, other. §29's 49 types map many-to-one onto these ten; the mapping is a design job, not an engine one |

### 9.4 The honest split

**26 controls drawn. 5 fully exist (rows 2, 22, 24, 25, and 17 once
`stale_stages()` learns a `landmarks` key), 9 exist as a mechanism awaiting
content (1, 3, 8, 15, 16, 19, 20, 21 — and 26, which exists *conditionally* on
§9.2's data shape), and 12 are owed outright (4, 5, 6, 7, 9, 10, 11, 12, 13, 14,
18, 23).**

The panel's *chrome* — accordion, groups, sliders, chips, notes, the run
button's busy pattern, the staleness badge and its poll, the menu cascade
mechanism, the cross-domain jump, the phone sheet — is entirely shipped and
reusable. Nothing in the layout needs a new widget.

Every *value* the panel reads or writes is owed, because the landmark record
does not exist. That is **one gap, not twelve**: all twelve owed controls are
downstream of `#4` and `§22`. Build the record, the cap table and one pass, and
all twelve resolve at once — along with the nine mechanisms, which are only
waiting for something to display.

**Two gaps sit outside that: viewshed and curvature**, in §9.3 rather than in
the control table, because they are capability gaps no amount of UI work
touches. A landmark panel can ship complete and correct with both still
missing; what it cannot do is pretend they are there. Viewshed is the more
serious
by a distance: six landmark types have it as a dominant term, and until it lands
the design's obligation is to mark those rows in place. `Dock.dc.html` draws
that mark — a bracketed `no viewshed` tag beside the type's name — on **Peak**
and **Volcanic feature**, with one line at the head of `§ TYPES` saying how many
carry it and what it costs them. The panel never presents a score it cannot
honestly compute.
