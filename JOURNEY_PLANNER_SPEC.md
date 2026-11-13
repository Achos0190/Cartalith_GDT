# Journey planner — implementation spec

> **Port note — not part of the vendored spec.** This file is the owner's
> layout spec for the rebuilt planner, imported verbatim on 2026-08-19
> (`c634110`) with `design/Journey Planner DCC.dc.html`, and its text is not
> edited. The marked port notes record where a later owner ruling or a newer
> design canvas supersedes it, or where the build departs from it; where two
> canvases disagree, the newer wins (`CLAUDE.md`). It is not status — that is
> `cartalith-native/docs/STATUS.md`'s — and it is not the engine: the `jp*` port,
> its boundary and its request keys are `JOURNEY_PLANNER_SCOPE.md`.

Rebuild of the Cartalith Gen1 v2.10 journey planner on the DCC shell. **Every field and
every computation of the v2.10 planner is kept**; only the layout and visual system change.
The planner itself is part of the shell (INFRA workspace); the Travel library that feeds it
is an addition, specified separately in `TRAVEL_LIBRARY_SPEC.md`.

Mockups: `Journey Planner DCC.dc.html` — `1a` distance spine (the chosen and only
maintained direction), `2a`–`2b` Travel library. The alternative "parameter bench"
direction and the table form of the library were reviewed and deleted.

## 1 · What was wrong with v2.10

The v2.10 planner (`#civPlannerSec`, `_jpRenderPartyForm`, `_jpRenderResults`) was a
full-screen modal over the map with a ~900 px scrolling wall of party controls, a second
scrolling wall of accordion stage cards (15 override fields × N stages), and a results
column below the fold. Three consequences, all layout:

1. The number you were tuning for (calendar days) was rarely on screen with the control
   you were tuning.
2. Per-stage overrides were invisible in aggregate — you could not see which stages had
   been touched without opening each accordion.
3. Blocked stages sorted in route order, so the one thing that made the journey
   impossible could sit at the bottom of the list.

Nothing about the model was wrong, so nothing about the model changed.

## 2 · Placement in the shell

- Domain rail: **INFRA**; tool label **JOURNEY** at the foot of the rail.
- Menu bar unchanged (File · Edit · Assets · Data · Preferences · Window · Help), and
  `Data ▸ Journey planner… ⇧J` opens it, matching `DCC_SHELL_SPEC.md` §2.4.
- Shell geometry is obeyed exactly: 34 px menu bar, 34 px tool options bar, 1 px
  hairlines at `rgba(255,255,255,.10)`, radius 0, accent `#e0a34a`, warn `#e0a840`,
  block `#b55950`, water `#7d9dae`, mono `IBM Plex Mono` for all labels and numerals.
- Tool options bar carries the journey-level controls: route picker, party preset,
  carriage auto/manual, "re-route for <mode>…", save journey, export table.
- Timeline bar carries the journey calendar: one band per day, coloured travel /
  water / weather hold / rest-layover.

> **Port note — placement superseded.** Owner ruling, 2026-09-05
> (`LARGE_ITEM_RULINGS.md`, item 4): *"Journey planner becomes a CIVIL rail node
> (v3/RP) rather than a Data menu row."* The newer DCC environment canvas
> (2026-08-31) draws it that way — CIVIL node `planner`
> (`design/dcc-environment-2026-08-31/spec/02-rail-and-domains.md`), its left
> dock in `04-left-dock.md` §6d, its results context and tool-options line in
> `05-right-dock-and-bars.md` §1.13 and §2.2.14. So there is no INFRA rail
> (INFRA merged into CIVIL on 2026-08-20) and no `Data ▸ Journey planner…` row;
> ⇧J survives as a global accelerator (`app.gd::_unhandled_key_input`), and
> every entry point calls `app.gd::open_journey_planner()`, which lights the
> rail node and arms the `journey` takeover. The timeline band this section
> describes borrows the shell's `timeline_row` while the planner is up.

## 3 · Direction 1a — distance spine

Route map (236 px) and terrain profile (150 px) share one horizontal distance axis, so
elevation, stage boundaries, settlements and water legs line up vertically. **The profile
is the stage selector** — click a band to inspect it; ⌥ click isolates; ⇧ drag trims.

| region | contents |
| --- | --- |
| left dock, 340 px | journeys list, then the whole party form (§5) |
| centre top | route map + route totals panel |
| centre spine | terrain profile with stage bands, km/day track, distance/day axis |
| centre strip, 32 px | stops · layover days, laid out along the distance axis |
| centre lower left | stage inspector — all 15 override fields for the selected stage (§6) |
| centre lower right, 642 px | stage matrix (§7) |
| right dock, 312 px | results — verdict card then collapsible groups (§8) |

## 4 · Rejected alternative

A "parameter bench" direction was built and rejected: results as a persistent KPI band
under the tool options bar, the party form opened out into a three-column bench, the matrix
full width, and reference panels in a bottom strip. It read as a spreadsheet rather than a
map tool and lost the spine's spatial anchor. Kept here only so the decision is on record.

## 5 · Party form — fields (all 26, unchanged from v2.10)

**Traveler** — group size (people) · pace (Easy / Steady / Forced) · hours per day (land)
· trade cargo (kg) · supplies carried (days) · carry food (on = carried, off = live off
the land) · grazing · foraging.

**Season & weather** — season · weather (auto = weighted by the season's own odds for the
biome, or a forced condition) · season drift during the journey · rest days (auto · 1 in N).

**Carriage** — auto / manual · transport mode · mount · vessel · donkeys · mules · camels
· horses · carts · wagons · travois · sleds · auto-promote Walking → Baggage Train when
overloaded. In auto, counts are computed (terrain × biome, km-weighted) and read-only.

**Route conditions** — road quality · infrastructure · desert water · respect seasonal
closures.

**Stops** — layover days per settlement the route threads.

Auto-valued fields show `auto · <resolved value>` so the resolved value is never hidden.
The fodder-ceiling advisory ("a mule carries at most ~N days of its own fodder at this
grazing setting") stays attached to the supplies field, where it is caused.

> **Port note — the count.** The engine's plan carries **28** fields, not 26
> (`journey_bridge::plan_to_pairs`, which `jp_default_plan()` returns). Carriage
> *auto / manual* is not a plan field but a request flag (`jp_compute`'s
> `auto_carriage`), and layovers are a separate map keyed by stop (`layovers`).
> The newer canvas (`04-left-dock.md` §6d) draws these fields as five groups:
> Traveler, Season, Carriage, Route, and Stops · Layovers.

## 6 · Stage inspector — the 15 override fields

Travel mode · group size · cargo kg · pace · hours/day · weather · carry food · supplies
days · grazing · foraging · road quality · infrastructure · mount · desert water · vessel.

Rules: a blank field inherits the party form and reads `Inherit (value)`; an auto field
reads `Auto (resolved)`; a field that cannot apply to the stage (vessel on a land stage,
mount on a train) is disabled with `—` and the reason. Overridden fields carry the accent
border. Header shows terrain, distance, ascent, biome and the override count; footer shows
this stage's days, km/day, load, ascent and arrival day. Actions: clear overrides · copy to
all land stages · isolate stage. Faster-mode advisories appear here with a **use here**
action.

## 7 · Stage matrix

One row per stage, two column groups, so the journey reads as a progression rather than a
set of cards:

- **OVERRIDES** (editable, lit cell = set, blank = inherits): mode · pace · hours, with
  the remaining fields behind "N more fields".
- **MODEL · PER STAGE** (read-only, dim): terrain · biome, weather, cargo kg remaining,
  supply days remaining, km/day, days. Cargo and supply run cumulatively down the route,
  so consumption, climate and terrain change are visible stage over stage.

Problem stages are pinned to the top (blocked first, then warnings), then route order.
Column tools: clear column · fill down.

## 8 · Results

Verdict card first: state (feasible / feasible—strained / impossible), calendar days as
the headline number, travel · rest · layover split, confidence band, and the reason in
prose with the actions that resolve it. Then collapsible groups:

- **Time** — travel days · rest days · layovers · mean/best/worst · arrival season.
- **Load** — cargo · supplies · capacity · carriers · speed penalty, with a capacity bar
  marking the overload threshold.
- **Supply reach** — carried days and km · longest gap and where · water/arid runs ·
  foraging offset · per-leg bar with resupply ticks.
- **Cost** — food/fodder · wages · tolls/ferry · animal upkeep · total · per km and per day.
- **Vessels** — per water leg: vessel, hold used, sailing window.
- **Calculation trace** — opens in its own window (`⧉`).

> **Port addition — River stretches and On ways (Ruling EU items 2 and 3,
> 2026-11-13).** Two groups after *Resupply stops*, on every form (the desktop
> planner window, CIVIL ▸ Journeys on the tablet, the phone's results sheet).
> *River stretches* lists each navigable stretch along the route that the
> engine measured as quicker (`jp_compute`'s `river_offers`: direction, length
> along the route, days saved by an exact re-plan, the water types and currents,
> and a hull menu with each hull's days, the recommended one starred and a hull
> that cannot work the stretch disabled with the engine's reason); **use river**
> sails it as a river leg whatever the journey's transport, **decline** hides its
> prompts for the session, an accepted leg offers **walk instead** and its own
> hull menu, and a saved leg the world no longer has is named with its reason.
> Offered stretches draw dashed in the water colour on the route map with their
> number. *On ways* lists the stretches the route rides a way (`way_spans`: name
> or class, km) and what the way join did when it was committed; the route map
> draws those stretches with a casing. A river leg's stage greys its own Vessel
> override (§6) with that reason: the leg's hull is chosen in *River stretches*.

> **Port note — the trace is inline.** Built as an inline collapsible group over
> the selected stage, not a `⧉` window, as `GUI_GAP_REGISTER.md` §7.12
> recommended; it draws every land/water factor from the engine's structured
> trace and reconciles `∏ factor == daily_km`. No owner ruling amends this line
> of the spec.

## 9 · Blocked and strained states

A blocked stage sets the verdict to impossible, colours the stage in the list, matrix,
profile band and map, and offers its resolutions inline (turn off closures, re-route
land-only, depart earlier). Strain — load above ~90% of capacity, a supply gap longer than
the carry, a confidence band widening past a season change — is warned but not blocking.

## 9a · Stale overrides (Ruling EA 10)

*Added 2026-11-01. A design for the owner's approval, drawn in the mobile mockup
(`design/mobile-shell-2026-10-31/`); nothing of it is built. The build row is `OUTSTANDING_WORK.md`'s "A saved
journey's per-leg overrides can land on a different leg".*

**Ruling EA item 10, "Detect, drop and tell me":**
- each override saves a fingerprint of the leg it was set on;
- an override whose leg no longer matches is dropped, with a notice naming it;
- nothing is ever applied to a different leg.

The fingerprint is the build's to define. This section designs the notice.

- **When.** The notice appears only when something was dropped, at the recompute that dropped it. That happens on
  opening a saved journey after a regenerate, a sculpt or an engine change, or when one of those happens while the
  planner is open. A one-line toast says it at that moment, for example *1 leg change dropped: stage 3 was rebuilt
  after the regenerate*.
- **Where.** At the top of the planner, above the selection rows and the verdict card, in every form:
  - the desktop's planner window (Ruling DY, PQ4);
  - the touch tablet's CIVIL ▸ Journeys dock;
  - the phone's STORY ▸ Journeys.
- **What it says.** One warning row, coloured `warn` (`dcc_theme.gd`: `#e0a840` dark, `#9a6a12` light), naming each
  dropped change by stage, its leg's two ends and the field:

  > 1 leg change no longer fits this world and was dropped: stage 3, Thornwood → High Saddle (pace override "forced
  > march"). The legs were rebuilt after the world changed; nothing was applied to a different leg.

  Several drops are listed in one row, separated by semicolons, with "N leg changes … were dropped".
- **Actions.**
  - **Show stage N** opens the first named stage's inspector (§6), so the user can set the value again on the leg
    they mean.
  - **Dismiss** removes the notice.

  Neither restores anything: a dropped override is gone, which is the ruling.
- **Not a blocked state.** The verdict and the stages are computed without the dropped override, and nothing is
  coloured as blocked (§9). The notice reports what the planner did; it does not stop the journey.
- **In the mockup:** Story ▸ Saved journeys… ▸ the third journey ("before the regenerate"), on the desktop and the
  phone. The test `ea.js` checks the text word for word, the two actions, the notice's place at the top and that
  Dismiss removes it. Its captures are `ea-10-desktop-planner-stale.png` (desktop-shell evidence) and
  `ea-10-phone-planner-stale.png` (mobile-shell evidence).

## 10 · Still to build

Light theme, blocked-stage inspector state, journey list/picker, and the 2560 tablet
breakpoint — all on direction 1a.

> **Port note.** This list is the design project's own to-do for the mockup as
> of 2026-08-19, not a record of what the port has built — that is
> `STATUS.md`'s.
