# EROSION_GEOLOGICAL_TIME_SCOPE.md — geological time as a forcing framework for erosion

**What this is:** the investigation behind the owner's request for geological
time in erosion — what erosion runs, where the kernels are blind to real extent,
the constraints any time axis must respect, and the owner's target architecture
set against them, measured against this repository's code and
`reference/Cartalith Gen1 v2.10.html` (whose line numbers these are). **What it
is not:** a milestone plan or status; its one build is `DECISIONS.md` §7m. It is
**partial by its own account** — §7 names what is missing.

Owner request, 2026-09-02: *"I think adding the geological years would also aid
the several erosion functions better"*, following the crater ruling
(`DECISIONS.md` §7l) and the observation that *"the timeline for civilisation is
different than a timeline for a geological scale."* The owner then supplied an
architecture paper, **Geological Time as a Forcing Framework for Procedural
Landscape Evolution**, which §6 treats as the target design; it is not vendored
here, so §6 is its only record in this repository. The same day the owner
ruled: *"let's only fix the hillslope extent blindness"* — the extent fix alone,
**no clock** (§8).

---

## 1. What erosion actually runs

**A default `generate_terrain` runs exactly one erosion kernel invocation**:
`stream_power_kernel` for `light_iters = max(4, round(stream.iters × 0.6))` = 9
iterations at the default `iters: 15`, then one `isostatic_rebound`, inside the
`carve_rivers` block. Checked at the call site (`cartalith-engine/src/lib.rs`).

Everything else is **off by default**. `ErosionPassParams::off()` sets every
toggle false and `evolve_cycles: 0`, so the whole second block — velocity,
glacial, coastal, hillslope, sediment routing, tidal — is skipped, and the app
boundary (`cartalith_godot::params::defaults`) turns none of them on.
`droplet_kernel` and `erode_thermal` are not in `generate_terrain` at all; they
exist only in `cartalith-engine/src/erode_op.rs`, the manual Erode op.

So the twelve-kernel API is largely dormant, and a geological clock would
attach to one nine-iteration loop. (Outside generation, `cartalith-erosion::tile`
re-runs the same kernel over a refined zoom tile — EF-3 of
`ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md`.)

## 2. Extent-blindness: real, and only half of it was a defect

When this investigation ran, `cartalith-erosion` contained **zero occurrences of
`map_width_km`**; no kernel signature carried a cell size. Measured then:
`StreamPowerParams` built at 5 km and at 40 000 km were field-for-field
identical, and `stream_power_kernel` output differed in **0 of 6144 cells** —
while in the same run `terrain_detail_k` gave 7.5 vs 1.0 and
`crater_min_diameter_km` 1.0 km vs 833.3 km.

**Stream power's blindness is dimensionally correct, and must not be "fixed".**
The kernel computes `ki · dt · area^0.5 / l` with `area` in cells and `l` a D8
distance in cells. In real units `area_cells^0.5 / l_cells = A_real^0.5 /
L_real` — the cell-size factors cancel. At `m = 0.5, n = 1` the coefficient has
units of 1/T with no length term, so a 5 km region and a 40 000 km world
genuinely *should* incise identically per unit time. Changing it would
introduce a bug.

**Hillslope diffusion was the crater defect exactly — since fixed.**
`hillslope_diffuse` computes `delta[i] = d · (l + r + u + dn − 4·fld[i])`, an
explicit Laplacian at a spacing of one cell. Physically `Δz = D·dt/Δx²·∇²z`, so
`d` must scale as `1/cell_km²`. At 2048 cells, 5 km gives 0.00244 km/cell and
40 000 km gives 19.53 km/cell; the ratio of cell **areas** is 8000² =
**64 000 000**, the figure §7l cites for the crater count, one layer up. The
same literal `diffuse_d = 0.15` applied at both extremes — latent (the pass is
off by default), and wrong by seven orders of magnitude once ticked. **Fixed by
`DECISIONS.md` §7m** (`hillslope_extent_scale`, anchored at 800 km / 2048 so
every golden is bit-identical, and capped at `HILLSLOPE_STABLE_D`, because
raising `d` past 0.25 detonates the explicit scheme).

**`erode_thermal`'s `talus` has the same shape and is not fixed**: a raw
normalised height difference across one cell, so `tan θ = talus · peak_m /
cell_km`. At the default `talus = 0.012` and `peak_m = 4000`: 800 km → 7.0°,
40 000 km → 0.14°, 5 km → 87.1°, against a real scree repose of 30–37°.
Manual-op only, which is why it waits. The velocity and glacial kernels have
not been audited for extent.

## 3. The reference collapsed a time axis rather than lacking one

The reference treats every erosion duration as a dimensionless iteration count.
Every time-like control is labelled "Iterations", "Passes" or "Cycles". **Every
occurrence of "million years" in all 31 107 lines of v2.10 is a loading-screen
joke** (four, 9971–10100).

The only per-year units in the file are in the biology and civilisation layer
(NPP in g·m⁻²·yr⁻¹, ore in kg/ha/yr). The geological layer has none — so **the
owner's "two timelines" observation is a split the reference already practised
without naming it.**

The machinery a duration term would attach to is already written, and
hardcoded:

- `dt = 1.0` as a literal (line 4137) — the Braun & Willett timestep, pinned
  to one.
- The hillslope section header states the PDE `∂z/∂t = D∇²z` (3871), then folds
  `dt/dx²` entirely into `D` (3878).
- The velocity docblock writes `v_old(x − v_old·Δt)` (3907) with `Δt` pinned at
  0.02 (3995).

Maximum freedom — no parity to break, no unit to honour — and minimum guidance:
un-pinning `1.0` returns a dimensionless `1.0`, not a calibration.

## 4. Three findings that constrain any design

**The stability wall.** Hillslope diffusion is explicit FTCS with a hard
`D ≤ 0.25` bound, and the reference's slider ceiling is 0.2 (`edD`, line
12927). **A duration term therefore cannot multiply the rate constants** — it
must scale pass counts. Implemented naively as "multiply D by elapsed years", it
diverges above a factor of ~1.67 and produces an exploded height field, which
will present as bad terrain rather than as a numerical bug.

**Stream power equilibrates.** Measured in this engine: per-iteration change
falls to 2e-5 by 360 iterations. Past that the landscape stops responding to
time, so "500 Myr" and "100 Myr" would look nearly identical.

**The default has no uplift.** `stream.uplift = 0.0`, the reference's own
literal. With `U = 0`, `∂z/∂t = −E` monotonically: **a geological clock wired to
erosion today would mean exactly one thing, a flatter world.** Uplift has to
become a real forcing before elapsed time produces landscape rather than mush.

## 5. Do not share the crater clock

`CraterParams::surface_age_myr` should not be reused for erosion, on physical
grounds: craters integrate a flux **linearly and indefinitely**, while stream
power **relaxes to a steady state and then stops caring about time**. The same
number means different things to the two systems.

This agrees with the owner's paper, which separates world age, feature age and
surface exposure age (its §7), and preserves §7l's three-clocks table, whose
purpose was to stop one control silently moving an unrelated system. What the
owner *did* couple is the **diffusivity**, not the clock: crater degradation
reads `ErosionPassParams::diffuse_d` (`DECISIONS.md` §7l-ii, ruling 2).

## 6. The target architecture, and where it meets the measurements

The owner's paper proposes time as an **integration domain** rather than a
multiplier, with discrete events modifying continuous forcing functions and the
existing coupled loop becoming the process engine. That framing is right,
matches landscape-evolution practice, and is compatible with everything
measured above. Four places the measurements bite:

1. **`z_{t+Δt} = z_t + Δt·F` is the exact form that explodes here** (§4). The
   integration must scale pass counts, not rate constants — or the schemes must
   change.
2. **The forcing framework needs `U(t)` to be real.** With `uplift = 0` the
   whole apparatus produces monotonic flattening (§4).
3. **Equilibration limits what a duration control can express** (§4). The
   paper's volcanic degradation sequence (its §5) assumes ongoing
   differentiation the fluvial system will not provide once at steady state.
4. **This reverses a recorded design position.** The reference's own research
   chose *"a natural-order single pass with one flow→climate→flow sandwich, not
   an iterated landscape-evolution model"* — recorded in
   `docs/research/system-coupling-audit.md` (cited at line 4264) as the choice
   of `docs/research/pipeline-order-audit.md`, whose anchors, Whipple & Tucker
   (1999) and Braun & Willett (2013), set process ordering while their
   timescales were left aside. Reversing that is a decision to raise and
   record, not to make quietly.

The paper's crater-degradation treatment (`t_diff ∝ L²/κ`) is the standard
diffusive model; `DECISIONS.md` §7l-i built it for craters.

## 7. What this document does not contain

- **The literature review.** A survey of landscape-evolution models
  (FastScape, Landlab, CHILD, Badlands); the numerical schemes that make
  geological time affordable — in particular whether Braun & Willett's
  implicit O(n) method is unconditionally stable in `dt`, which decides whether
  duration can be decoupled from compute cost; and defensible ranges for `K`,
  `D`, `m`, `n`, uplift and denudation rates. Commissioned, returned nothing,
  still owed.
- **The blast-radius measurement** — perturb an erosion constant, record which
  suites fail. It did not run. What is known: the light stream-power pass and
  its `isostatic_rebound` are on the **default** path, so every
  `cartalith-civ` golden suite that runs on generated terrain is downstream of
  any change to either (sixteen when the crater change measured them,
  2026-09-02) — the lesson the crater work paid for.
- **A dimensional analysis** of the kernels not covered in §2.

## 8. Open questions

The 2026-09-02 ruling settled one and deferred the rest:

1. **Is reversing the reference's "not an iterated LEM" position (§6, item 4)
   authorised?** Unruled. §7l authorised breaking parity *for craters*; it is
   not a blank cheque, and erosion's blast radius is larger.
2. ~~Should the extent-blindness in `hillslope_diffuse` be fixed first,
   independently of any clock?~~ **Ruled yes and built** — `DECISIONS.md` §7m.
3. **Is a second self-referential anchor acceptable?** Unruled. §7l anchored
   craters on the app's own default rather than the terrestrial rate, and said
   so. Doing it again for erosion ("N Myr ≡ the current iteration defaults") is
   circular by construction; it may still be right, but should be a decision.
4. **Should `isostatic_rebound` be per-pass or per-op?** Unruled. The
   reference's `eroFinish` (line 4260) gives it one call per op, which this
   port follows — but 34% of relief hangs on the choice, and a geological clock
   cannot leave it implicit.
5. **Does uplift become a real forcing?** Unruled. Without it, elapsed time
   only flattens (§4).

Questions 1 and 3–5 are prerequisites of a geological clock, which the same
ruling declined to build for now; they become live only if a clock is proposed
again.
