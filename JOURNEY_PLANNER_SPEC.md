# Journey planner — implementation spec

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

## 9 · Blocked and strained states

A blocked stage sets the verdict to impossible, colours the stage in the list, matrix,
profile band and map, and offers its resolutions inline (turn off closures, re-route
land-only, depart earlier). Strain — load above ~90% of capacity, a supply gap longer than
the carry, a confidence band widening past a season change — is warned but not blocking.

## 10 · Still to build

Light theme, blocked-stage inspector state, journey list/picker, and the 2560 tablet
breakpoint — all on direction 1a.
