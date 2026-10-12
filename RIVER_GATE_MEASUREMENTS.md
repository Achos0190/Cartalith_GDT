# River gate measurements (Ruling CF item 6)

Owner, 2026-10-12, Ruling CF item 6: *"Order-versus-discharge river gate and
River Transport discharge gate: measure first, report numbers, change nothing
yet. Ruling BV (no river density/threshold changes) stands meanwhile."*

This file is the measurement. **Nothing in generation, drawing or planning was
changed**, and no golden moved: the only code added is an `#[ignore]`d harness.
It records no status; `cartalith-native/docs/STATUS.md` does that.

## The two gates, opened at their symbols

| Gate | Where | Today's rule | The owner's open question |
|---|---|---|---|
| **1. Delta-fan mouths** ("order versus discharge as the gate for rivers in the map") | `cartalith-godot/src/river_delta.rs` `delta_fans`, constants `MIN_ORDER` (3) and `MIN_DISCHARGE_FRAC` (2e-4) | a drawn run whose last piece meets water gets a fan when `own_order >= 3` **and** its discharge `>= 2e-4 x gw x gh` | order, discharge, or both? (`STATUS.md` RIM-4 entry: *"the order >= 3 gate skips 155 mouths that clear the discharge bar, 3 of the top 5 by discharge being order 1"*) |
| **2. River Transport navigability** | `cartalith-civ/src/river_class.rs` `NAVIGABLE_ORDER` (3); read by the planner's channel test (`lib.rs`, `net.order[j] < river_class::NAVIGABLE_ORDER`) **and** the router's `civ_navigable_river_discount` | a land cell is a navigable river point when the planner network (`fresh_river_channels`) gives it Strahler order `>= 3`. **There is no discharge gate today** | gate on discharge as well as order? (`OUTSTANDING_WORK.md`: *"a 75x-threshold river reads order 1 to 2 on the 256-wide world"*) |

Gate 1 is draw-side only: it reads the drawn network and decides where extra
fan runs are painted. Gate 2 is the reference's own `order>=3`
(`_civNavigableRiverDiscount`, v2.11 line 21441), shared by the planner and the
Route tool's router so the two cannot drift (Ruling CD).

## Method

Harness: `cartalith-native/crates/cartalith-godot/tests/river_gate_measurement.rs`,
test `river_gate_measurement`, ignored. Command, run alone in release:

```text
cd cartalith-native
RG_OWNER=1 cargo test --release -j 2 -p cartalith-godot --test river_gate_measurement -- --ignored --nocapture --test-threads=1
```

- **Worlds.** App parameters (`params::defaults()`: integrated drainage on,
  geology model on, river density 1.0, no GPU), twelve fixed seeds (246371,
  483920, 912345, 24601, 404, 505, 808, 707, 71077345, 1001, 31337, 123456,
  chosen before any result was seen) at three shapes: **512x328 at 40 075 km**,
  **384x246 at 800 km** (the regional default), **1024x656 at 40 075 km**.
  Plus one calibration world, the owner's (seed 246371, 2048x1311, 40 075 km,
  density 1.55). 37 worlds, 26 s wall in total.
- **Gate 1** reads the drawn network the way `river_comb_harness.rs` does:
  `river_entities` then `river_draw_plan` (runs it hides are not counted), and
  a *mouth* is a drawn run whose outlet cell or a neighbour is water or at/below
  sea level. It applies four rules to every mouth: order only, discharge only,
  both (today), either; and reports the two disagreement sets.
- **Gate 2** reads `cartalith_civ::fresh_river_channels` with the arguments
  `story_bridge::jp_world_parts` passes, on land cells
  (`build_water_bodies` class 0). Rules: order `>= 2/3/4`; discharge
  `Q >= k x T` on channel cells (T = `river_flow_thresh`, the river threshold
  the planner's `flow_thresh` is) alone, OR-ed with order 3 ("as well as"), and
  AND-ed with it. The anchor k = 8 is `river_class::DELTA_FLOW_MULT`, the
  "high flow" multiple the same module already uses; the sweep is 4, 6, 8, 10,
  12 (the +-25 % and +-50 % the brief asked for) plus 16, 32, 75 for context.
  Per rule: navigable cells, length (each cell's step to its receiver,
  wrap-aware, in km), connected pieces along the receiver tree, pieces under
  5 cells, the largest piece's share, and the share of navigable length whose
  chain reaches water without leaving the navigable set ("to-water").
- **Proxy, stated.** Gate 1 reads `River::order` (the run's highest order) and
  `River::discharge` (the run's maximum), not `DrawnRun::own_order` and the
  maximum over the last drawn piece that `delta_fans` reads, and "on a shore"
  is the comb harness's neighbourhood test, not `DrawnRun::reach`. The owner
  world calibration below shows how far that moves the counts.

## Gate 1: delta-fan mouths

Pooled over the twelve seeds of each shape. *medQ/T* is the median mouth
discharge as a multiple of that world's river threshold (scale-free);
*noTrib* is the share of runs with no tributary; *order 1* the share that are
order-1 runs.

| Shape | Shore mouths | Order >= 3 only | Discharge >= 2e-4 only | **Both (today)** | Either |
|---|---:|---:|---:|---:|---:|
| 512x328, 40 075 km | 5 712 | 1 126 | 3 309 | **967** | 3 468 |
| 384x246, 800 km | 1 553 | 117 | 1 479 | **115** | 1 481 |
| 1024x656, 40 075 km | 7 771 | 1 295 | 3 667 | **1 161** | 3 801 |
| owner, 2048x1311 (1 world) | 643 | 133 | 263 | **111** | 285 |

The disagreement sets:

| Shape | Set | n | medQ/T | median length (cells) | median order | no tributary | order 1 |
|---|---|---:|---:|---:|---:|---:|---:|
| 512x328 | today (both) | 967 | 100.5 | 11.2 | 3 | 27 % | 0 % |
| 512x328 | order, not discharge | 159 | 2.6 | 2.0 | 3 | 64 % | 0 % |
| 512x328 | discharge, not order | 2 342 | 27.8 | 6.4 | 2 | 62 % | 49 % |
| 384x246 | today (both) | 115 | 30.2 | 18.2 | 3 | 19 % | 0 % |
| 384x246 | order, not discharge | 2 | 0.3 | 1.0 | 3 | 100 % | 0 % |
| 384x246 | discharge, not order | 1 364 | 2.8 | 5.7 | 1 | 75 % | 70 % |
| 1024x656 | today (both) | 1 161 | 77.9 | 18.7 | 3 | 23 % | 0 % |
| 1024x656 | order, not discharge | 134 | 2.9 | 8.7 | 3 | 57 % | 0 % |
| 1024x656 | discharge, not order | 2 506 | 17.2 | 11.3 | 2 | 55 % | 45 % |
| owner | order, not discharge | 22 | 4.1 | 27.5 | 3 | 41 % | 0 % |
| owner | discharge, not order | 152 | 18.7 | 20.6 | 1 | 57 % | 52 % |

Of each world's ten largest-discharge mouths, today's rule admits **80 of
120** (512x328), **58 of 120** (384x246), **87 of 120** (1024x656) and **4 of
10** (owner world); order alone admits exactly the same mouths as today in
every case; discharge alone admits all of them.

The per-world 90th percentile of mouth discharge, as a share of the grid's
cells (the statistic `MIN_DISCHARGE_FRAC` was taken from, on the owner world
only): median **3.56e-3** (2.06e-3 to 8.59e-3) at 512x328, **7.70e-3**
(4.60e-3 to 1.35e-2) at 384x246, **2.04e-3** (6.94e-4 to 6.89e-3) at
1024x656, **2.65e-3** on the owner world. All are 3.5 to 67 times today's
2e-4 floor.

**Calibration against `STATUS.md`'s engine figures** (owner world, RIM-4
entry): engine 2 203 mouths, 138 order >= 3, 62 eligible, 155 skipped by the
order gate though they clear the discharge bar; this harness 643 mouths, 133
order >= 3, 111 both, 152 discharge-not-order. The order >= 3 count and the
discharge-not-order count agree to within 4 %; the mouth total and the
eligible count do not. The likely reason (a suspicion, not measured) is the
proxy: the engine counts every drawn run whose last drawn piece meets water,
including runs carried on by `river_draw_plan`'s extensions and bridges, and
reads discharge over that last piece only, so it counts more mouths and fewer
pass its discharge floor. The ratios between the rules, which is what the
recommendation rests on, are the same in both readings.

**What the numbers say.**

1. **The discharge half of today's rule does what it was added for**: what it
   removes from the order rule is small and stubby (median 2.6 to 4.1 x T,
   median length 1 to 27 cells, mostly without tributaries). On the 800 km
   world it removes 2 of 117.
2. **Discharge alone would fan about three times as many mouths** (3 309
   against 967 at 512x328; 1 479 against 115 at 800 km), and the added ones
   are mostly order 1-2 runs with no tributary. On the 800 km world the 2e-4
   floor is cleared by 95 % of all mouths, so a discharge-only rule there is
   practically "every mouth", median 2.8 x T. The floor is a share of the
   grid's cells, and its meaning moves with world size and km (the 90th
   percentiles above span 6.9e-4 to 1.35e-2).
3. **The real cost of the order rule is the biggest mouths**: 33 to 62 of 120
   top-ten mouths get no fan on the matrix worlds, 6 of 10 on the owner world.
   The large-discharge low-order runs carry a median 17 to 28 x T on the
   40 075 km worlds. Why a high-discharge run reads order 1 is documented at
   `River::discharge` (discharge is accumulated on the D8 tree, order on the
   channel tree; they are different trees); whether these runs look like
   rivers that deserve a delta was **not** checked by eye.

**Recommendation, gate 1: keep today's rule (order >= 3 AND the discharge
floor).** Discharge alone or either would triple the fans, mostly onto order-1
trickles, and on a regional map would fan nearly every mouth. If the owner
wants the largest rivers fanned regardless of order, the candidate is an
*additional* clause for the top of the distribution only (for example a
discharge in multiples of T, which is scale-free, rather than a share of the
grid), and it needs its own measurement and a look at the frames before it is
proposed; this harness does not measure it.

## Gate 2: River Transport navigability

Summed over the twelve seeds of each shape; *largest* and *to-water* are
per-world means. *+ / -* are cells the rule adds to, or removes from, today's
order >= 3 set.

### 512x328, 40 075 km

| Rule | Cells | km | Pieces | Pieces < 5 cells | Largest | To-water | + | - |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| order >= 2 | 163 112 | 14 934 791 | 8 941 | 3 193 | 5 % | 35 % | 113 741 | 0 |
| **order >= 3 (today)** | **49 371** | **4 505 890** | **2 736** | **804** | **6 %** | **31 %** | 0 | 0 |
| order >= 4 | 11 360 | 1 028 413 | 543 | 124 | 13 % | 25 % | 0 | 38 011 |
| Q >= 4 x T (-50 %) | 246 083 | 22 054 133 | 14 097 | 5 901 | 6 % | 39 % | 196 716 | 4 |
| Q >= 6 x T (-25 %) | 191 097 | 17 181 599 | 11 304 | 4 642 | 5 % | 38 % | 141 762 | 36 |
| Q >= 8 x T (anchor) | 159 809 | 14 404 734 | 9 515 | 3 807 | 5 % | 37 % | 110 594 | 156 |
| Q >= 10 x T (+25 %) | 138 266 | 12 484 710 | 8 280 | 3 210 | 5 % | 36 % | 89 303 | 408 |
| Q >= 12 x T (+50 %) | 122 664 | 11 088 088 | 7 316 | 2 735 | 5 % | 36 % | 74 200 | 907 |
| order 3 OR Q >= 4 x T | 246 087 | 22 054 544 | 14 098 | 5 902 | 6 % | 39 % | 196 716 | 0 |
| order 3 OR Q >= 6 x T | 191 133 | 17 185 065 | 11 306 | 4 644 | 5 % | 38 % | 141 762 | 0 |
| order 3 OR Q >= 8 x T | 159 965 | 14 419 116 | 9 518 | 3 807 | 5 % | 37 % | 110 594 | 0 |
| order 3 OR Q >= 10 x T | 138 674 | 12 522 091 | 8 299 | 3 217 | 5 % | 36 % | 89 303 | 0 |
| order 3 OR Q >= 12 x T | 123 571 | 11 171 270 | 7 365 | 2 758 | 5 % | 36 % | 74 200 | 0 |
| order 3 AND Q >= 4 x T | 49 367 | 4 505 480 | 2 735 | 803 | 6 % | 31 % | 0 | 4 |
| order 3 AND Q >= 8 x T | 49 215 | 4 491 508 | 2 733 | 806 | 6 % | 31 % | 0 | 156 |
| order 3 AND Q >= 12 x T | 48 464 | 4 422 708 | 2 687 | 785 | 6 % | 31 % | 0 | 907 |
| Q >= 16 x T | 100 591 | 9 109 883 | 5 993 | 2 187 | 5 % | 35 % | 53 614 | 2 394 |
| Q >= 32 x T | 61 265 | 5 551 492 | 3 699 | 1 228 | 6 % | 34 % | 22 237 | 10 343 |
| Q >= 75 x T | 31 461 | 2 847 630 | 1 853 | 567 | 8 % | 31 % | 7 647 | 25 557 |

Context: the median over worlds of the median Q/T of order-3 land cells is
**53.1**, of order-1/2 channel cells **3.2**; **7 647** order-1/2 land cells
carry at least 75 x T.

### The other shapes, the rows that matter

| Shape | Rule | Cells | km | Pieces | < 5 cells | Largest | To-water | + | - |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 384x246, 800 km | order >= 2 | 19 968 | 48 581 | 1 555 | 531 | 8 % | 33 % | 15 988 | 0 |
| | **order >= 3 (today)** | **3 980** | **9 631** | **312** | **104** | **18 %** | **27 %** | 0 | 0 |
| | order >= 4 | 391 | 954 | 25 | 8 | 66 % | 5 % | 0 | 3 589 |
| | Q >= 4 / 6 / 8 / 10 / 12 x T | 26 815 / 18 939 / 14 638 / 12 011 / 10 179 | 65 329 / 46 045 / 35 497 / 29 090 / 24 614 | 2 186 / 1 608 / 1 278 / 1 056 / 898 | | | 34 / 32 / 30 / 30 / 29 % | | |
| | order 3 OR Q >= 8 x T | 14 715 | 35 674 | 1 284 | 505 | 9 % | 30 % | 10 735 | 0 |
| | order 3 AND Q >= 8 x T | 3 903 | 9 454 | 306 | 103 | 19 % | 27 % | 0 | 77 |
| 1024x656, 40 075 km | order >= 2 | 338 575 | 15 357 610 | 12 000 | 3 658 | 6 % | 30 % | 225 918 | 0 |
| | **order >= 3 (today)** | **112 657** | **5 105 126** | **3 221** | **745** | **8 %** | **25 %** | 0 | 0 |
| | order >= 4 | 32 758 | 1 493 702 | 742 | 135 | 14 % | 21 % | 0 | 79 899 |
| | Q >= 4 / 6 / 8 / 10 / 12 x T | 439 817 / 336 610 / 278 725 / 240 035 / 212 273 | 19 758 951 / 15 161 788 / 12 566 417 / 10 825 680 / 9 574 867 | 16 974 / 12 794 / 10 357 / 8 749 / 7 570 | | | 32 / 31 / 30 / 30 / 29 % | | |
| | order 3 OR Q >= 8 x T | 279 573 | 12 606 850 | 10 404 | 3 430 | 7 % | 30 % | 166 916 | 0 |
| | order 3 AND Q >= 8 x T | 111 809 | 5 064 693 | 3 171 | 728 | 8 % | 25 % | 0 | 848 |
| owner, 2048x1311 | order >= 2 | 77 816 | 1 759 035 | 1 347 | 372 | 7 % | 18 % | 47 071 | 0 |
| | **order >= 3 (today)** | **30 745** | **696 632** | **397** | **103** | **9 %** | **12 %** | 0 | 0 |
| | order >= 4 | 11 375 | 258 766 | 120 | 30 | 12 % | 7 % | 0 | 19 370 |
| | Q >= 4 / 6 / 8 / 10 / 12 x T | 73 880 / 57 867 / 49 189 / 43 671 / 39 211 | 1 666 235 / 1 308 559 / 1 113 201 / 988 326 / 887 672 | 1 384 / 1 038 / 853 / 724 / 623 | | | 18 / 16 / 15 / 14 / 14 % | | |
| | order 3 OR Q >= 8 x T | 50 208 | 1 135 759 | 889 | 304 | 9 % | 15 % | 19 463 | 0 |
| | order 3 AND Q >= 8 x T | 29 726 | 674 075 | 361 | 82 | 9 % | 12 % | 0 | 1 019 |

Context medians of Q/T, order-3 cells against order-1/2 channel cells: 32.9
against 2.4 (384x246), 40.6 against 2.7 (1024x656), 24.8 against 1.5 (owner).

### A finding outside the question: the navigable network breaks

Only **12 to 31 %** of today's navigable length reaches water without leaving
the navigable set, and the largest connected piece holds only 6 to 18 % of it
(66 % only on the 800 km world's tiny order >= 4 set). The `RG_DIAG=1` switch
counts where an order >= 3 land cell's receiver goes. On 512x328, seed 246371:
of 5 206 such cells, 3 749 + 1 037 + 138 hand on to order 3 / 4 / 5 land, 95
to water, 20 end with no receiver, and **167 hand on to a land cell of order
0, 1 or 2**. The other diagnosed worlds show the same pattern (seed 404 at
512x328: 126 of 4 371; 1024x656: 186 and 183). A chain that drops order
downstream breaks the navigable run, and the planner then portages. This is
a property of the network `fresh_river_channels` builds, not of the gate: no
gate in the table raises the to-water share by more than about 8 points. Why
the order drops downstream is measured in **Chain breaks** at the end of this
file: the channel tree's receiver is never the cell the water flows to.

### Timing

`fresh_river_channels`, the network the planner builds on every call (the
"unmeasured cost" in the River Transport row), five calls each on seed 246371,
release build, single process, nothing else running, two separate runs:

| Grid | Run 1: median (min..max) | Run 2: median (min..max) |
|---|---|---|
| 384x246 | 0.005 s (0.005..0.006) | 0.005 s (0.005..0.005) |
| 512x328 | 0.010 s (0.010..0.011) | 0.010 s (0.010..0.010) |
| 1024x656 | 0.038 s (0.037..0.039) | 0.038 s (0.037..0.043) |
| 2048x1311 | 0.177 s (0.146..0.201) | 0.146 s (0.146..0.156) |

So roughly 0.15 to 0.2 s per planner call at the owner's size. The gates
themselves cost nothing that matters (gate 1, the whole entity and draw-plan
pass: 0.05 s at 512x328, 0.8 s at 2048x1311).

**What the numbers say.**

1. **A discharge gate OR-ed with order 3 is, in size, about the same as
   dropping to order >= 2.** At the anchor (8 x T) it gives 3.2x the cells
   (512x328), 3.7x (384x246), 2.5x (1024x656), 1.6x (owner world), and every
   added cell is an order 1-2 channel. The +-25 % and +-50 % steps move the
   cell count by about -13 % / +20 % and -23 % / +54 % (512x328): a
   continuous knob where order is a 3x step.
2. **AND-ing discharge with order 3 is almost a no-op**: it removes 0.3 to 3 %
   of today's cells at 8 x T, because order-3 cells already carry a median 25 to
   53 x T.
3. **Neither fixes the audit's symptom.** The audit river was mostly portage
   because most of its cells read order 1-2; a discharge OR would make those
   cells navigable, but the network stays in hundreds of pieces per world
   (pieces grow roughly with cells), and to-water rises only 3 to 8 points.
   The breaks above are the larger cause of portage stages.

**Recommendation, gate 2: keep `NAVIGABLE_ORDER = 3` alone for now; do not
add a discharge gate yet.** If the owner wants more of a river sailable, a
discharge OR at about 8 x T is the measured candidate (one constant, scale-free
because T scales with the world), but it should follow, not precede, a look at
why order drops downstream on this network, since that, not the gate, is what
fragments the navigable set.

## What would move under each option

Nothing below moved in this pass. Verified with
`git diff --stat` after the harness ran: only the harness, this file and the
two documentation rows changed.

| Option | Generation goldens | Other tests and pins that would change |
|---|---|---|
| Gate 1, keep | none | none |
| Gate 1, order only | none (draw-side) | `river_delta.rs` tests `the_tuned_constants_are_pinned_by_literals_not_by_themselves` (pins the 2e-4 floor) and `only_a_large_river_that_ends_in_water_gets_a_fan`; the comb harness's `fan_mouths` definition (`river_comb_harness.rs` `stats`); the RIM-4/RIM-7 fan pixel counts in `STATUS.md` (screen and export frames move) |
| Gate 1, discharge only or either | none (draw-side) | the same, plus `comb_thinning_never_hides_a_fan_mouth`, whose premise is `MIN_ORDER > COMB_MAX_ORDER` (with no order clause, an order-1 fan mouth is protected only by `COMB_DISCHARGE_FRAC <= MIN_DISCHARGE_FRAC`); about 3x the fans on screen, in tiles and in export |
| Gate 2, keep | none | none |
| Gate 2, discharge OR / AND, planner only | none (the reference-verified planner goldens m5/m6/sp2 run with no river network, `STATUS.md` Ruling CD batch 2) | `cartalith-civ/tests/ruling_cd_rivers.rs` fixtures where a cell's discharge crosses the new gate; and the planner would then disagree with the router about what is navigable, which Ruling CD set out to prevent |
| Gate 2, discharge OR / AND, planner and router | `cartalith-civ/tests/golden_parity_civ_tools.rs` mixed-domain router cases (reference-verified; the router's `civ_navigable_river_discount` reads `NAVIGABLE_ORDER`) | as above, plus `river_class.rs` `navigable_order_is_the_reference_gate` and `channel_terrain`'s `River with Shallows` rule (keyed on `order <= NAVIGABLE_ORDER`) |
| Gate 2, order >= 2 or >= 4 | as the row above | as the row above |

A change to gate 2 that reaches the router is a divergence from the
reference's `order>=3` and is a DECISIONS §7p change with a disclosed
re-baseline; gate 1 has no reference ancestor.

## Owner questions, with defaults

1. **Gate 1: keep "order >= 3 AND discharge floor" for delta fans?** Default:
   **yes, keep.** Discharge alone or either roughly triples the fans, mostly
   onto order-1 runs.
2. **Gate 1: should the very largest mouths get a fan whatever their order?**
   Default: **not now**; if yes, a separate measured clause for the top of the
   distribution in multiples of T, shown on frames before it ships.
3. **Gate 1: express the discharge floor in multiples of the river threshold
   instead of a share of the grid?** Default: **not now**; the floor is nearly
   inert on these worlds (it removes 2 to 17 % of order-3 mouths), so the unit
   matters only if question 2 is answered yes.
4. **Gate 2: add a discharge gate to River Transport?** Default: **no, keep
   order >= 3 alone for now.**
5. **Gate 2: investigate why the planner network's order drops downstream
   (the breaks that fragment it) before any gate change?** Default: **yes**, as
   a measurement row; no generation change without its own ruling (Ruling BV).
6. **If a discharge gate is wanted later, planner only or planner and
   router?** Default: **both**, so one definition of navigable stands (Ruling
   CD), with the router goldens re-baselined and disclosed.

## Chain breaks

Owner question 5 above, answered as a measurement (2026-10-12). **Nothing in
generation, drawing or planning was changed and no golden moved**: the only
code added is a second `#[ignore]`d test in the same harness. Command, run
alone in release, twice in separate processes with identical output (about
22 s each):

```text
cd cartalith-native
RG_OWNER=1 cargo test --release -j 2 -p cartalith-godot --test river_gate_measurement river_chain_breaks -- --ignored --nocapture --test-threads=1
```

Same worlds as above (twelve seeds x three shapes, plus the owner's world).
A **break** is what the `RG_DIAG` count above counted: an order >= 3 land
cell of the planner network (`fresh_river_channels`) whose receiver is a land
cell of order 0, 1 or 2. The per-world counts reproduce that count exactly
(seed 246371 at 512x328: 167; seed 404: 126).

### The mechanism, read at its symbols

The planner network is two steps, both the reference's own code
(`buildRiverNetwork` and `strahlerFromReceivers`, v2.11 line 4480, ported
exactly):

1. `cartalith_hydrology::build_channels_routed` (`build_channels_core`) marks
   a cell as channel when its discharge clears the threshold, then picks its
   **one receiver by aspect steering**: among neighbours lower on the routing
   surface, the best of drop x alignment with the real ground's gradient
   direction. It is **not** the steepest-descent (D8) receiver that
   `compute_flow` accumulated the discharge along, except inside a filled
   depression, where the doc of `build_channels_routed` already records the
   same failure and takes the D8 receiver instead.
2. `strahler_from_receivers` visits channel cells in **ascending discharge**,
   which is a valid upstream-before-downstream order only if a receiver always
   carries at least its donor's discharge. On the steered tree it does not, so
   a low-discharge receiver is finalised before its large donor reports in.

### The breaks, by cause

Classes partition the breaks. *A*: the receiver is a channel cell
(order 1-2). *B*: the receiver is not a channel cell (order 0), so it also has
no receiver and the chain stops there. Every other class tested was **zero on
every world**: a receiver with equal discharge (tie), a receiver with *more*
discharge than its donor (would be a Strahler bug), an off-network receiver
below sea level, and a receiver across the wrap seam.

| Shape | Order >= 3 cells | Breaks (per world) | A: channel receiver with less discharge | B: off-network receiver | Receiver = the D8 receiver | Donor in a filled depression |
|---|---:|---:|---:|---:|---:|---:|
| 512x328, 40 075 km | 49 371 | 1 467 (97 to 167) | 652 (44 %) | 815 (56 %) | **0** | 0 |
| 384x246, 800 km | 3 980 | 193 (9 to 26) | 42 (22 %) | 151 (78 %) | **0** | 0 |
| 1024x656, 40 075 km | 112 657 | 1 867 (117 to 219) | 474 (25 %) | 1 393 (75 %) | **0** | 0 |
| owner, 2048x1311 | 30 745 | 232 | 43 (19 %) | 189 (81 %) | **0** | 0 |

- **In all 3 759 breaks the receiver is not the D8 receiver**, and no donor
  is in a filled depression. Both classes are one cause: aspect steering sends
  the river's channel to a neighbour the water does not go to. Class A is
  where that neighbour happens to be a small channel; class B where it is
  not a channel at all.
- **Not an edge effect**: 9 to 12 % of breaks lie in the top or bottom 5 % of
  rows, against 10.6 to 10.8 % of order >= 3 cells.
- **What follows a break** (walking on from the receiver), pooled
  512x328 / 384x246 / 1024x656 / owner: navigable again at the next step
  58 / 0 / 34 / 3; within 2-3 more steps 137 / 5 / 68 / 3; within 4-10 more
  232 / 13 / 150 / 9; later 58 / 5 / 111 / 17; reaches water first
  97 / 4 / 42 / 5; **the chain stops** 885 / 166 / 1 462 / 195 (every class B
  break, plus a few class A chains that end).
- **Topological ordering alone is a small part of it.** Re-running the same
  per-cell Strahler rule in true upstream-before-downstream order on the same
  tree (`strahler_topological`, harness-only) leaves 927 / 171 / 1 507 / 199
  breaks: it removes 37 / 11 / 19 / 14 % of them.
- **The drawn network has the same breaks.** Counted the same way on
  `ws.channels` / `ws.stream_order` (generation's network, the one the map
  draws, built mid-carve by the same two functions): 1 624 / 168 / 2 844 / 519
  breaks, 934 / 143 / 2 053 / 427 of them to an off-network cell. On screen,
  `river_draw_plan`'s downhill extension (2026-10-05, `STATUS.md` "Rivers that
  stopped on dry land now run on") already bridges most class-B ends; the
  planner network gets no such bridge.

### Five examples

*Q/T* is discharge in multiples of the world's river threshold. Coordinates
are (x, y) cells. Chosen as each world's largest-discharge break of each
class (an input, not the result), then five picked across seeds and shapes.

| World | Donor | Receiver it is given | The D8 receiver, where the water goes | What follows |
|---|---|---|---|---|
| seed 404, 512x328 | (366,324), order 6, Q/T 7 896 | (366,325), channel order 2, Q/T 12.3 (class A) | (365,325), order 3, Q/T 8 228 | navigable again one step on |
| seed 912345, 512x328 | (480,311), order 4, Q/T 8 195 | (480,312), channel order 2, Q/T 10.9 (class A) | (481,312), order 2, Q/T 8 210 | navigable again 5 steps on |
| seed 483920, 512x328 | (346,304), order 4, Q/T 1 694 | (347,304), not a channel, Q/T 0.8 (class B) | (347,303), order 1, Q/T 1 696 | the chain stops |
| seed 24601, 1024x656 | (418,318), order 3, Q/T 2 816 | (419,318), not a channel, Q/T 0.3 (class B) | (419,319), order 1, Q/T 2 816 | the chain stops |
| owner seed 246371, 2048x1311 | (1592,712), order 6, Q/T 2 493 | (1591,711), not a channel, Q/T 0.0 (class B) | (1593,711), order 0, Q/T 2 493: water (inferred: the guard below re-points this donor there and leaves no break, which needs a non-land cell) | the chain stops one cell short of water |

The pattern in every example: the D8 receiver carries the donor's discharge
plus a little, the given receiver carries almost none, and the D8 receiver's
own order is low because the river's order was accumulated along the steered
path, not along the water's.

### Judgement: a defect, not hydrology

- **Not distributaries.** The tree has one receiver per cell and discharge
  never splits. In the 74 printed examples (two per world) the given receiver
  carries 0.0 to 99.6 x T against donors of 12.6 to 9 868 x T, and the D8
  receiver carries at least the donor's discharge in every one. The water
  does not go where the channel goes.
- **Not a Strahler artefact in the first place.** Ordering accounts for 11 to
  37 % of breaks; the rest survive a correct topological ordering.
- **Not seams, sea level or the fill**: zero breaks in each. Nor
  rasterisation: the planner reads the cell tree directly, and nothing is
  traced or drawn between it and the order raster.
- **The consequence is the reported one**: River Transport portages on rivers
  the map draws continuous, because the drawn network is bridged after the
  fact and the planner's is not. Whether the order drop also shows on screen
  (a big river drawn as order 1-2 past a break, so thinner or thinned by the
  comb rule) was **not** checked by eye.

Not measured: whether the reference path (integrated drainage off, where the
reference-verified goldens run) breaks as often. The two functions are the
reference's, so it is expected to; that would make this an inherited defect
that the port's routing did not cause.

### Candidate fixes, sized on a copy of the tree (nothing shipped)

The **receiver guard**: a channel cell whose steered receiver carries *less*
discharge than itself (an off-network cell always does) takes its D8 receiver
instead, the rule `build_channels_routed` already applies inside a filled
depression and for the reason its doc gives. Ordered with the shipped
flow-sorted `strahler_from_receivers`, unchanged. Also sized: topological
Strahler alone, and topological Strahler with a D8 fallback for class B only.

| Shape | Today: cells / pieces / to-water | Topological Strahler | Topo + D8 for class B | **Receiver guard** | Cells the guard re-points |
|---|---|---|---|---|---:|
| 512x328 | 49 371 / 2 736 / 31 % | 56 086 / 2 376 / 44 % | 74 645 / 1 846 / 65 % | **74 250 / 1 844 / 66 %, 0 breaks** | 28 111 of 524 922 channel cells (5.4 %) |
| 384x246 | 3 980 / 312 / 27 % | 4 648 / 309 / 30 % | 8 181 / 245 / 59 % | **8 016 / 242 / 59 %, 0 breaks** | 5 542 of 78 342 (7.1 %) |
| 1024x656 | 112 657 / 3 221 / 25 % | 119 943 / 2 974 / 28 % | 159 266 / 2 165 / 61 % | **158 559 / 2 140 / 61 %, 0 breaks** | 39 752 of 1 088 328 (3.7 %) |
| owner | 30 745 / 397 / 12 % | 31 895 / 378 / 13 % | 38 631 / 259 / 45 % | **38 449 / 257 / 50 %, 0 breaks** | 5 008 of 265 819 (1.9 %) |

Under the guard **no break is left on any world, and no navigable chain ends
inland**: every chain that does not reach water ends on the map's border
(248 / 70 / 322 / 49 chain ends, all on the top or bottom row or a non-wrapping
side). So the remaining 34 to 50 % of navigable length is rivers running off
the map edge, which "to-water" counts as not reaching water; it is a property
of the world's shape, not of the network. The navigable set grows by 25 to
101 % because orders that the steered tree lost are kept.

### Fixes per cause, and what each would move

| Fix | Cures | Moves | Does not move |
|---|---|---|---|
| **F1. Receiver guard in `build_channels_core`, integrated-drainage path only** (gated like the slope-ease divergence, so the reference path is identical by control flow) | A and B: every break on every world | every app world's channel network, hence generation's carve, `river_mask`, `stream_order`, the drawn rivers, and everything that reads them (settlements, roads, territory, landmarks, fans, the Route tool's river discount, the planner); every app-parameter figure in `STATUS.md` and the river harnesses (gap, comb, RIM-4 fans, this file's gate tables); any app-parameter test that pins a count or hash, to be enumerated by a workspace run with the change | `golden_parity_river` (3 cases), `tile_hydrology`, `golden_parity_civ_tools` and the other reference-path goldens, which run with integrated drainage off |
| **F1b. The same guard as a post-pass inside `fresh_river_channels` / `fresh_river_network` only** | A and B on the planner network | every `fresh_river_order` consumer (suitability, roads, territory, the Route tool's discount, the planner) and their app-parameter tests; `cartalith-civ/tests/ruling_cd_rivers.rs` fixtures if any cell's receiver changes | generation and the drawn map. But it leaves the drawn and the planned network built by two different rules, which DECISIONS §7p (scrub the superseded approach, do not keep it alongside) argues against |
| F2. Topological Strahler | 11 to 37 % of breaks | `strahler_from_receivers`, a reference-ported function pinned cell for cell by `golden_parity_river` if the order differs on those fixtures | not needed once F1 holds: with the guard, ascending discharge is already an upstream-first order and the shipped function leaves 0 breaks |

Either F1 change is a deliberate divergence from the reference's
`buildRiverNetwork`, disclosed per DECISIONS §7p, and changes generated output,
so it needs an owner ruling (Ruling BV covers density and threshold; this
changes neither, but it does change the rivers). F1 also changes what the map
draws, so frames at every zoom touched must be looked at before it ships: the
D8 receiver is a steeper, more grid-aligned step than the steered one, and a
staircase look is the risk to check.

### Recommendation

**F1, on the integrated path only, measured and looked at before it ships.**
One network definition for generation, drawing and planning (Ruling CD's point
and §6g's "both trees need the same surface"); it removes every break on these
37 worlds by re-pointing 2 to 7 % of channel cells, and it needs no change to
the reference-ported Strahler. F1b is the smaller fallback if the owner does
not want the drawn rivers to move. Gate 2's discharge question (owner question
4) should be re-measured after either, because the navigable set it was
measured against grows by a quarter to a double.

### Owner questions, with defaults

7. **Fix the chain breaks at all?** Default: **yes**; they are a defect (the
   channel goes where the water does not), and they are the larger cause of
   River Transport portages.
8. **F1 (generation and planner, one network) or F1b (planner only)?**
   Default: **F1**, gated to the integrated-drainage path, with the
   re-baseline disclosed and frames reviewed before commit.
9. **After the fix, keep `river_draw_plan`'s downhill extension?** Default:
   **keep**, and re-measure its loose ends with `river_gap_harness.rs`; it
   still bridges ends the guard does not touch (map-border and lake cases).
10. **Re-run gate 2 (and gate 1's fan counts) after the fix before deciding
    the discharge gate?** Default: **yes**.
11. **Should "to-water" count a river that runs off the map border as
    reaching an outlet?** Default: **yes, in the measurement only**; it does
    not change what the planner does.
