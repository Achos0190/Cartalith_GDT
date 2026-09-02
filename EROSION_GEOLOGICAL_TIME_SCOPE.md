# Erosion and geological time — scope

**Status of this document: partial, and it says so rather than pretending.** Two of four
commissioned investigations completed before a session limit killed the rest; a separate
literature review of landscape-evolution modelling returned nothing at all. What is here is
**measured against this repository's own code and the frozen reference**, and is worth
recording on its own. What is missing is named in §7.

Owner request, 2026-09-02: *"I think adding the geological years would also aid the several
erosion functions better"*, following the crater ruling (`DECISIONS.md` §7l) and the
observation that *"the timeline for civilisation is different than a timeline for a
geological scale."* The owner subsequently supplied an architecture paper, **Geological Time
as a Forcing Framework for Procedural Landscape Evolution**, which §6 treats as the target
design.

---

## 1. What actually runs today

Measured, not read from documentation. An investigating agent added one temporary integration
test, ran it, deleted it, and confirmed no trace remained.

**The default generation runs exactly one erosion kernel invocation**: nine iterations of
`stream_power_kernel`, then one `isostatic_rebound`. `light_iters` is
`max(4, round(stream.iters * 0.6))` = 9 from the default `iters: 15`.

Everything else is **off by default**. `ErosionPassParams::off()` sets every toggle false and
`evolve_cycles: 0`, so the entire second block — velocity, glacial, coastal, hillslope,
sediment routing, tidal — is skipped. `droplet_kernel` and `erode_thermal` are not in
`generate_terrain` at all; they exist only in `erode_op.rs`, the manual Erode op.

So the twelve-kernel API is largely dormant, and a geological clock would today attach to one
nine-iteration loop.

## 2. Extent-blindness: real, but only half of it is a defect

`cartalith-erosion` contains **zero occurrences of `map_width_km`**. No kernel signature
carries a cell size or any real-extent term. Measured: `StreamPowerParams` built at 5 km and
at 40 000 km are field-for-field identical, and `stream_power_kernel` output differs in
**0 of 6144 cells**. For contrast, in the same run `terrain_detail_k` gives 7.5 vs 1.0 and
`crater_min_diameter_km` gives 1.0 km vs 833.3 km.

**But stream power's blindness is dimensionally correct, and must not be "fixed".** The kernel
computes `ki * dt * area^0.5 / l` with `area` in cells and `l` a D8 distance in cells. In real
units `area_cells^0.5 / l_cells = A_real^0.5 / L_real` — the cell-size factors cancel
identically. At `m = 0.5, n = 1` the stream-power coefficient has units of 1/T with no length
term, so a 5 km region and a 40 000 km world genuinely *should* incise identically per unit
time. Changing this would introduce a bug, not remove one.

**Hillslope diffusion is a different story, and it is the crater defect exactly.**
`hillslope_diffuse` computes `delta[i] = d * (l + r + u + dn - 4*fld[i])` — an explicit
Laplacian at a grid spacing of exactly one cell. Physically `Δz = D·dt/Δx²·∇²z`, so `d` must
scale as `1/cell_km²`. At 2048 cells, 5 km gives 0.00244 km/cell and 40 000 km gives 19.53
km/cell; the ratio of cell **areas** is 8000² = **64 000 000** — the identical figure §7l
cites for the crater count, one layer up. `diffuse_d = 0.15` is the same literal at both
extremes.

It is off by default, so this is latent rather than active. It is wrong by seven orders of
magnitude the moment a user ticks Hillslope on a region or a world.

`erode_thermal`'s `talus` has the same shape: a raw normalised height difference across one
cell, so `tan θ = talus · peak_m / cell_km`. At the default `talus = 0.012` and
`peak_m = 4000`: 800 km → 7.0°, 40 000 km → 0.14°, 5 km → 87.1°. Real scree repose is 30-37°.
Manual-op only, so lower priority — but the same class of error.

## 3. The reference collapsed a time axis rather than lacking one

The frozen reference treats every erosion duration as a dimensionless iteration count with no
time meaning. Every time-like control is labelled "Iterations", "Passes" or "Cycles". **Every
occurrence of "million years" in all 31 107 lines is a loading-screen joke.**

The only per-year units anywhere in the file are in the biology and civilisation layer (NPP in
g·m⁻²·yr⁻¹, ore in kg/ha/yr). The geological layer has none — so **the owner's "two timelines"
observation is a split the reference already practised without ever naming it.**

The sharper finding: the machinery a duration term would attach to is already written, and
hardcoded in exactly two places.

- `dt = 1.0` as a literal (reference line 4137) — the Braun & Willett timestep, pinned to one.
- The hillslope section header states the PDE `∂z/∂t = D∇²z` (3871) and then folds `dt/dx²`
  entirely into `D` (3878).
- The velocity docblock writes `v_old(x − v_old·Δt)` (3907) with `Δt` pinned at 0.02 (3995).

That means maximum freedom — no parity to break and no unit to honour — and minimum guidance:
un-pinning `1.0` returns a dimensionless `1.0`, not a calibration.

## 4. Three findings that constrain any design

**The stability wall.** Hillslope diffusion is explicit FTCS with a hard `D ≤ 0.25` bound and
a shipped slider ceiling of 0.2. **A duration term therefore cannot multiply the rate
constants** — it must scale pass counts. Implemented naively as "multiply D by elapsed years",
it diverges above a factor of ~1.67 and produces an exploded height field. That will not
present as a numerical bug; it will present as bad terrain.

**Stream power equilibrates.** Measured in this engine: per-iteration change falls to 2e-5 by
360 iterations. Past that the landscape stops responding to additional time. So "500 Myr" and
"100 Myr" would look nearly identical, and a duration control is not doing what a naive
reading assumes.

**The default has no uplift.** `stream.uplift = 0.0` — the reference's own literal. With
`U = 0`, `∂z/∂t = −E` monotonically. **A geological clock wired to erosion today would mean
exactly one thing: a flatter world.** Uplift has to become a real forcing before elapsed time
produces landscape rather than mush.

## 5. Do not share the crater clock

`CraterParams::surface_age_myr` should not be reused for erosion, on physical grounds rather
than architectural taste: craters integrate a flux **linearly and indefinitely**, while stream
power **relaxes to a steady state and then stops caring about time**. The same number means
different things to the two systems.

This agrees with the owner's own architecture paper, which separates world age, feature age
and surface exposure age (§7 there). It also preserves §7l's three-clocks table, whose purpose
was precisely to stop one control silently moving an unrelated system.

## 6. The target architecture, and where it meets the measurements

The owner's paper proposes time as an **integration domain** rather than a multiplier, with
discrete events modifying continuous forcing functions and the existing coupled loop becoming
the process engine. That framing is right, matches landscape-evolution practice, and is
compatible with everything measured above. Four places where the measurements bite:

1. **`z_{t+Δt} = z_t + Δt·F` is the exact form that explodes here** (§4, stability wall). The
   integration must scale pass counts, not rate constants — or the schemes must change.
2. **The forcing framework needs `U(t)` to be real.** With the shipped `uplift = 0`, the whole
   apparatus produces monotonic flattening (§4).
3. **Equilibration limits what a duration control can express** (§4). The paper's volcanic
   degradation sequence (§5 there) assumes ongoing differentiation the fluvial system will not
   provide once at steady state.
4. **This reverses a recorded design position.** The reference's own research
   (`system-coupling-audit.md`, cited at reference line 4264) deliberately chose *"not an
   iterated landscape-evolution model"*, citing Whipple & Tucker (1999) for process ordering
   while pointedly ignoring its timescales. Per `CLAUDE.md`, reversing that is a decision to
   raise and record, not to make quietly.

The paper's crater-degradation treatment (`t_diff ∝ L²/κ`) is the standard diffusive model and
explains directly why small craters vanish first — which is the preservation point §7l left
open, where the terrestrial record is a preserved subset rather than a census.

## 7. What this document does not yet contain

Named rather than glossed:

- **The literature review.** A survey of landscape-evolution models (FastScape, Landlab, CHILD,
  Badlands), the numerical schemes that make geological time affordable — in particular
  whether Braun & Willett's implicit O(n) method is genuinely unconditionally stable in `dt`,
  which decides whether duration can be decoupled from compute cost — and defensible ranges
  for `K`, `D`, `m`, `n`, uplift and denudation rates. **It returned nothing; it is still owed.**
- **The blast-radius measurement.** The commissioned empirical audit — perturb an erosion
  constant, record exactly which suites fail — did not run. What is known: the light
  stream-power pass and its `isostatic_rebound` are on the **default** path, so
  `cartalith-civ`'s 16 golden suites are downstream of any change to either. That is the
  lesson the crater work paid for.
- **A dimensional analysis per kernel** for the remaining kernels.

## 8. Open questions for a ruling

1. **Is reversing `system-coupling-audit.md`'s "not an iterated LEM" position authorised?**
   §7l authorised breaking parity *for craters*; it is not a blank cheque, and erosion's blast
   radius is larger.
2. **Should the extent-blindness in `hillslope_diffuse` be fixed first, independently of any
   clock?** It is the crater defect exactly, it is cheaper, and it needs no timeline. The
   measurements suggest yes.
3. **Is a second self-referential anchor acceptable?** §7l anchored craters on the app's own
   default rather than the terrestrial rate, and said so honestly. Doing it again for erosion
   ("N Myr ≡ the current iteration defaults") is circular by construction. It may still be the
   right call; it should be a decision rather than a habit.
4. **Should `isostatic_rebound` be per-pass or per-op?** The reference's `eroFinish` gives it
   one call per op, which this port follows faithfully — but 34% of relief hangs on the choice,
   and a geological clock cannot leave it implicit.
5. **Does uplift become a real forcing?** Without it, elapsed time only flattens (§4).
