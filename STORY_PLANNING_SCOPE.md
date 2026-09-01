# Story planning layer — settlement timelines, conflicts, journeys

> **This document defines SP-1…SP-5; it does not track them.** Where any of
> them stands today — built, blocked, or waiting on one of §6's open questions
> — is recorded only in `cartalith-native/docs/STATUS.md`. Read this file for
> what a milestone *is* and why it is shaped that way.

Owner-supplied direction, 2026-08-25. Three features asked for in one sentence,
and they are one subsystem rather than three: each is a different reading of the
same year cursor the Timeline subsystem already owns.

> "I think adding timeline information to settlements and maybe start an overlay
> in civil where a user can draw things for conflicts? And make a bridge to a
> traveling party so it is easier to use as a story planning aid."

The owner then chose the fullest option on all three (`AskUserQuestion`,
2026-08-25). Recorded here rather than paraphrased, because §5's milestones are
sized against these three answers and a later reader needs to know the question
was asked:

| Fork | Chosen |
|---|---|
| What drives the conflict overlay | **Both** — you draw it, and it reads each side's real manpower figures and war-duration estimate |
| What a settlement's timeline shows | **Both tracks on one strip** — simulated history from the existing snapshots, plus authored story events |
| How far the party bridge goes | **Full chain** — journey entity, supply consumption over distance, computed arrival dates, and the party surfacing in the timeline of each settlement it passes |

## 1. Why this is cheap in the middle and expensive at the ends

This subsystem is cheap in the middle because most of its engine work is
already someone else's finished milestone. What it asks for that nobody has
built is almost entirely *identity and display* — the connective tissue
between three subsystems that were each built correctly and never introduced
to each other.

**The inventory this scoping pass took, 2026-08-25 — the reason the milestones
below are sized the way they are.** It is a record of what the pass found, not
a claim about today; check anything load-bearing against the code.

- **The whole Timeline subsystem**, defined by `TIMELINE_SCOPE.md`.
  `cartalith-civ::timeline`
  carries `TimelineSnapshot { year, territory, settlements, ways }`,
  `civ_snapshot_save`/`civ_snapshot_load`, `civ_year_diff` → `YearDiff`, the
  collapse and recovery step functions, proximity adjacency and betweenness.
  `civ_assign_tid`/`civ_resync_next_tid` give settlements and ways **stable ids
  across a re-generation**, which is the single fact that makes a per-settlement
  history expressible at all.
- **`get_settlements()` exposes `tid`** (`crates/cartalith-godot/src/lib.rs`,
  confirmed 2026-08-23 in `PARITY_AUDIT.md` §7 — an earlier `STATUS.md` entry
  claiming otherwise had gone stale).
- **The Timeline UI**, as a sixth `DccWidgets.category()` in
  `civilization_workspace.gd`: year pills, Add year, a real-time-scale scrub,
  Play/Pause/Step, three filter checkboxes over a live `civ_year_diff()` count,
  and the collapse-simulation form.
- **Military manpower.** `cartalith-civ::manpower` computes the four outputs
  (standing / field / emergency armies, and sustainable war duration) from five
  variables, per the owner's verbatim specification in
  `MILITARY_MANPOWER_SCOPE.md`. Plus `military.rs` and `relations.rs`.
- **The Travel Library.** `travel_library.rs` holds `PartyPreset` alongside
  `AnimalDef`/`VehicleDef`/`VesselDef`, and the wiring is real, not parallel:
  `jp_capacity`/`jp_calc_land`/`jp_plan` each gained an `_ex` sibling taking a
  `JpAnimalResolver`, so a custom entry changes computed capacity and speed and
  can hard-block a terrain.
- **The Markdown Vault** (`cartalith-vault`), with section-span notes, a
  `CARTALITH:BEGIN/END` machine block and a backlink index — the natural home
  for authored prose, and the reason §3 below does **not** invent a second one.

**The one thing the pass could not find, and the reason the owner's third ask
is the keystone:** no persistent, referenceable journey existed anywhere in
this port. The Travel Library's own usage tracking said so honestly — it
reported `0` for "saved journeys" *by construction*, because
`route_get`/`infra.routes` are drawn polylines with no attached plan.
Everything in §4 hangs off fixing that, which is why SP-1 is first.

## 2. The shared spine

One year cursor, three readings. The cursor already exists (the Timeline
category's scrub); nothing here introduces a second clock.

- A **settlement** reads the cursor as *its own history up to this year*.
- A **conflict** reads it as *am I active in this year*.
- A **journey** reads it as *where is the party today*.

Everything below is keyed on `tid`, not on array index. This is not a
preference: a re-generation renumbers settlements, and this project has already
shipped two destructive bugs (`RF-02`/`RF-03`) from a window holding an index
across a generate. Any story record keyed by index is a data-loss bug waiting
for its first regenerate.

## 3. Authored content lives in the Vault, not in a new store

The obvious mistake here is to build a second prose store for story events and
end up with two half-populated ones. `cartalith-vault` already does
section-span Markdown with a machine block and backlinks, and the owner has
already ruled once this session that vault UI belongs in Data rather than in a
new menu.

So: an authored event is a dated entry in the subject's vault note, inside the
machine block, and the timeline strip *reads* it. New Rust surface is a typed
accessor over what the vault already stores, not a new persistence layer.

Consequence worth stating up front: an authored event survives a regenerate
exactly as well as the vault's `tid` binding does, and no better.

## 4. Milestones

Ordered so each one is useful on its own if the next never lands.

### SP-1 — The Journey entity

The keystone. A `Journey` in `cartalith-civ`: a `PartyPreset` reference, a route
(the existing polyline), a start year, and a name/id. Persisted in the save
alongside the timeline (`SAVEFILE_COMPAT.md` — additive, and the `.zip` entry
order is fixed, so verify against the live format rather than assuming).

Closes the Travel Library's honest `0`: its "saved journeys" usage count becomes
real for the first time.

**Done means** a journey survives save → load → reopen, and the Travel Library
reports a non-zero, correct reference count for a party preset a journey uses.

### SP-2 — Journey progression over the cursor

Distance along the route × the party's computed speed (through the existing
`jp_*_ex` path, so a custom Travel Library entry still governs) gives a position
per day. Supply consumption over distance, and a computed arrival date.

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
same tool vocabulary the Way and Route tools already establish.

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
- **No second prose store** — see §3.
- **No new clock.** If something needs a finer grain than the Timeline's year,
  raise it rather than adding a parallel cursor.
- **No golden-parity target.** None of this exists in the reference HTML, so per
  `DECISIONS.md` §7d this is all tagged as divergence-by-addition. The
  constraint that *does* bind: anything routed through `jp_*` must not change
  what the Journey Planner already computes, and the existing suite must prove
  it.

## 6. Open questions

- **Grain.** A year is the Timeline's unit, but a journey takes days and a
  battle takes hours. SP-2 needs sub-year positioning; whether that surfaces as
  a real date or as a fraction of a year is unresolved.
- **What a conflict is attached to.** Free geometry is simplest and matches
  "draw things". Whether a conflict should also be able to *reference* a
  settlement or a province — so that moving one moves the other — is a real
  fork and is not decided here.
- **Regenerate semantics.** A journey's route is a polyline in world space; a
  regenerate changes the world under it. Whether journeys are invalidated,
  re-snapped, or kept as-is with a staleness mark needs a ruling before SP-2
  ships.
