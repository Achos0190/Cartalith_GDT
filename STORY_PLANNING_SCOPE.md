# STORY_PLANNING_SCOPE.md — settlement timelines, conflicts and journeys over one year cursor

**What this is:** the definition of story planning's five milestones (SP-1…SP-5,
§4), the owner direction and choices that sized them, and the rules they share
(§2, §3, §5). **What it is not:** status. Where each milestone stands is
`cartalith-native/docs/STATUS.md`'s "Story planning" group; open work is routed
from `OUTSTANDING_WORK.md`.

Owner-supplied direction, 2026-08-25. Three features asked for in one sentence,
and they are one subsystem rather than three: each is a different reading of the
same year cursor the Timeline subsystem (`TIMELINE_SCOPE.md`) already owns.

> "I think adding timeline information to settlements and maybe start an overlay
> in civil where a user can draw things for conflicts? And make a bridge to a
> traveling party so it is easier to use as a story planning aid."

The owner then chose the fullest option on all three (`AskUserQuestion`,
2026-08-25). Recorded here rather than paraphrased, because §4's milestones are
sized against these three answers and a later reader needs to know the question
was asked:

| Fork | Chosen |
|---|---|
| What drives the conflict overlay | **Both** — you draw it, and it reads each side's real manpower figures and war-duration estimate |
| What a settlement's timeline shows | **Both tracks on one strip** — simulated history from the existing snapshots, plus authored story events |
| How far the party bridge goes | **Full chain** — journey entity, supply consumption over distance, computed arrival dates, and the party surfacing in the timeline of each settlement it passes |

Later owner rulings bind the details (`LARGE_ITEM_RULINGS.md`): **Ruling AM**
(authored events are a Chronos block, §3) and **Ruling AO** with its two
addenda (the calendar date, conflict attachment, journey re-snap — §6).

## 1. Why this is cheap in the middle and expensive at the ends

Most of the engine work this subsystem needs was already someone else's
milestone. What it asks for is almost entirely *identity and display* — the
connective tissue between subsystems that were each built correctly and never
introduced to each other. It builds on:

- **The Timeline subsystem** (`TIMELINE_SCOPE.md`): `cartalith_civ::timeline`'s
  `TimelineSnapshot`, `civ_snapshot_save`/`civ_snapshot_load`, `civ_year_diff`
  → `YearDiff`, the collapse and recovery steppers. `civ_assign_tid` /
  `civ_resync_next_tid` give settlements and ways **stable ids across a
  re-generation** — the single fact that makes a per-settlement history
  expressible at all — and `get_settlements()` exports that `tid`.
- **The year cursor's two views**: the CIVIL left dock's Timeline category
  (`civilization_workspace.gd::_build_timeline`) and the shell's timeline strip
  (`dcc_shell.gd` §10a), both reading and writing `CivData::year`.
- **Military manpower** (`cartalith_civ::manpower`, per the owner's verbatim
  specification in `MILITARY_MANPOWER_SCOPE.md`): standing / field / emergency
  armies and sustainable war duration. Plus `military.rs` and `relations.rs`.
- **The Travel Library** (`TRAVEL_LIBRARY_SPEC.md`): `PartyPreset` beside the
  animal/vehicle/vessel definitions, and a resolver path (`jp_capacity_ex`,
  `jp_calc_land_ex`, `jp_plan_ex`/`jp_plan_full`) through which a custom entry
  really changes computed capacity and speed and can hard-block a terrain.
- **The Markdown Vault** (`cartalith-vault`): section-span notes, a
  `CARTALITH:BEGIN/END` machine block and a backlink index — the natural home
  for authored prose, and the reason §3 does **not** invent a second one.

**The keystone gap the 2026-08-25 scoping found:** no persistent, referenceable
journey existed in this port — the Travel Library's "saved journeys" usage count
read `0` by construction, because `route_get`/`infra.routes` are drawn polylines
with no attached plan. Everything in the journey half hangs off fixing that,
which is why SP-1 is first.

## 2. The shared spine

One year cursor, three readings. Nothing here introduces a second clock (§5).

- A **settlement** reads the cursor as *its own history up to this year*.
- A **conflict** reads it as *am I active in this year*.
- A **journey** reads it as *where is the party today* — the cursor's year plus
  its day-of-year (`WorldGen::civ_day`, Ruling AO).

Everything is keyed on `tid`, not on array index. This is not a preference: a
re-generation renumbers settlements, and this project has already shipped two
destructive bugs (`RF-02`/`RF-03`) from a window holding an index across a
generate. Any story record keyed by index is a data-loss bug waiting for its
first regenerate.

## 3. Authored content lives in the Vault, not in a new store

The obvious mistake is to build a second prose store for story events and end
up with two half-populated ones. `cartalith-vault` already does section-span
Markdown with a machine block and backlinks, and the owner has already ruled
that vault UI belongs in Data rather than in a new menu.

So an authored event is a dated entry in the subject's vault note, and the
timeline *reads* it. **The entry format is Ruling AM's** (2026-09-23): a
` ```chronos ` block in the Chronos Timeline Obsidian plugin's exact syntax, so
the same note draws a working timeline in Obsidian with no Cartalith involvement.
It sits anywhere in the note **except inside the machine block**, which is
replaced wholesale on every Cartalith write and would lose the author's events.
(Ruling AM superseded this section's earlier "inside the machine block"
wording.) Authors edit the note directly, or use the Settlement Editor's
**Add event** form, which composes the Chronos line so nobody types the syntax
(Ruling AM's addendum). The Rust surface is a typed accessor over what the
vault already stores (`cartalith_vault::chronos`), not a new persistence layer.

Consequence worth stating up front: an authored event survives a regenerate
exactly as well as the vault's `tid` binding does, and no better.

## 4. Milestones

Ordered so each one is useful on its own if the next never lands.

### SP-1 — The Journey entity

The keystone. A `Journey` in `cartalith-civ`: a `PartyPreset` reference, a route
(the existing polyline), a start year, and a name/id. Persisted in the save
alongside the timeline — `SAVEFILE_COMPAT.md` §9.6, `entities/journeys.json`;
additive, and the `.zip` entry order is fixed, so verify against the live format
rather than assuming.

It makes the Travel Library's "saved journeys" usage count real for party
presets. A journey names a preset, not an animal, so an animal's journey usage
stays `0` by construction.

**Done means** a journey survives save → load → reopen, and the Travel Library
reports a non-zero, correct reference count for a party preset a journey uses.

### SP-2 — Journey progression over the cursor

Distance along the route × the party's computed speed (through the existing
`jp_*_ex` path, so a custom Travel Library entry still governs) gives a position
per day. Supply consumption over distance, and a computed arrival date. The date
is Ruling AO's calendar (§6).

**Done means** scrubbing the year cursor moves the party marker along its route
and the supply readout falls as it goes, with the arrival date derived rather
than typed. Golden-testable against `jp_plan`'s existing figures — the party's
speed must not diverge from what the Journey Planner already computes for the
same party and route.

### SP-3 — The settlement timeline strip

Per-settlement history from the snapshots already captured: population, tier,
`ruins`/`fortified` flags per year — read, not recomputed. Authored events from
§3 on the same track. Journeys from SP-2 that pass through the settlement appear
as a third mark.

This is the first time the collapse/recovery simulation becomes legible *per
place* rather than as an aggregate count.

**Done means** opening a settlement shows its real simulated trajectory across
every stored year, with authored entries interleaved by date.

### SP-4 — The conflict overlay

A drawn conflict in CIVIL: fronts, arrows, sieges, battle markers, each with a
name, a year range, the sides involved and an outcome. Free-form geometry, the
same tool vocabulary the Way and Route tools already establish, optionally
anchored to a settlement or province (§6).

Then the half the owner specifically asked for: it **reads the numbers**. A
conflict naming two factions pulls each side's real standing/field/emergency
figures and the sustainable war-duration estimate from `manpower.rs`, as
annotation on the drawn thing.

**Done means** a drawn conflict shows both sides' real manpower, the drawing
persists through save/load, and it appears and disappears as the year cursor
crosses its range.

### SP-5 — The planning aid, joined up

The three readings referring to each other: a journey that passes a settlement
during a conflict's active years says so; a settlement's strip shows the
conflicts that touched it.

Deliberately last. Each of SP-1 to SP-4 is worth having alone, and this one is
worth nothing until at least two of them exist.

## 5. Out of scope, said explicitly

- **No combat resolution.** The overlay annotates and reads manpower; it does
  not decide who wins. `MILITARY_MANPOWER_SCOPE.md` models capacity, not
  outcomes, and inventing an outcome model would be a silent deviation from it.
  (For the same reason a conflict's outcome is free text, not an enum — Ruling
  AO's SP-4 addendum.)
- **No second prose store** — see §3.
- **No new clock.** The cursor is `CivData::year`. Ruling AO gave it a
  day-of-year for SP-2 (`WorldGen::civ_day`, 0…364, beside the year): the same
  cursor at a finer grain, not a parallel one, and only journeys read it.
  Anything else that needs a finer grain raises it rather than adding a cursor.
- **No golden-parity target.** None of this exists in the reference HTML, so per
  `DECISIONS.md` §7d it is divergence-by-addition. The constraint that *does*
  bind: anything routed through `jp_*` must not change what the Journey Planner
  already computes, and the existing suite must prove it.

## 6. The three questions this document left open — ruled

Each was ruled twice. The later ruling is the one that stands.

- **Grain (SP-2).** Ruling AO (2026-09-23): **a real calendar date, reusing the
  year/month/day that Chronos already writes** (Ruling AM) rather than a second
  date system. The calendar, from AO's SP-2 addendum:
  `cartalith_vault::chronos::MONTH_DAYS` — the Gregorian month lengths with
  February fixed at 28, 365 days, no leap year. A saved journey departs on
  1 January of `Journey::start_year`; rest days are spread evenly; arrival is
  departure + `ceil(total_days)` from `jp_plan`'s own figure.
- **What a conflict is attached to (SP-4).** Ruling 8 (2026-09-06): geometry
  with optional references — a deleted referenced entity leaves the geometry
  and dashes the reference (never a sentinel id), and *"which conflicts touch
  this settlement?"* must be answerable. Ruling AO (2026-09-23) confirmed it:
  **a conflict can reference a settlement or a province**, by stable id, so
  moving the place moves the conflict. The builder's calls on the points it
  left open — a province keyed by its seed settlement's `tid`, deletion keeps
  the anchor, auto-populate detaches, a new world empties the store — are
  AO's SP-4 addendum.
- **Regenerate semantics for a journey's route (SP-2).** Ruling 9 (2026-09-06)
  chose *invalidate*; **Ruling AO (2026-09-23) replaced it with re-snap onto
  the new terrain.** A re-snap must name its edge cases rather than assume the
  happy path; the policy is AO's SP-2 addendum: an endpoint is re-bound by
  `tid` and name together, a vanished stop **drops** the journey (reported by
  name), and an unroutable leg is **kept and flagged**.
