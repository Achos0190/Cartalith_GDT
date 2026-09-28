//! GF-7: the geological clock (`GEOLOGY_FIRST_SCOPE.md` §4.12; owner Ruling BJ,
//! *"add a simple clock"*).
//!
//! **What it is.** One dimensionless number, geological age τ
//! ([`crate::WorldParams::geo_age`]), that says how long the landform
//! processes have run relative to a default world. It acts **only on pass and
//! iteration counts, never on rate constants**: explicit kernels cannot take a
//! larger coefficient without going unstable (the geological-time scope's
//! stability wall, §1.4 point 2), so a longer duration means more passes.
//!
//! **Why it exists.** GF-2 and GF-3 built rock-aware stream power and a
//! threshold hillslope, and measured that nine iterations of transient
//! incision change the surface by metres against relief of hundreds of metres
//! (§5.6, §5.7). The scope names this clock as its own lever on landform scale.
//!
//! **Why dimensionless.** The pipeline has no calibrated time, and a slider in
//! Myr would claim a calibration nothing supports (§4.12).
//!
//! **What it must never do.**
//! - Act when [`GeoClock::is_scaling`] is false. At τ = 1, or with
//!   `geology_processes` off, every count is **today's expression, by control
//!   flow** (`if !scaling { today } else { scaled }`), so neither the parity
//!   boundary nor the app's world can move and no golden can.
//! - Scale a rate constant, the priming climate, the hydrology trace, the RV-1
//!   carve, volcanism or craters (§4.12 "The clock does not touch").
//! - Hide a floor: every count reports whether its floor bound ([`Count`]), so
//!   the UI can say so (§4.12 "Floors").

use cartalith_jsmath::js_round;

/// τ's default, and the identity: the value at which every process runs
/// today's count by control flow. Source: §4.12 ("default **1.0**").
pub const GEO_AGE_DEFAULT: f64 = 1.0;
/// τ's lower bound. Source: §4.12 ("range 0.25 to 4.0").
pub const GEO_AGE_MIN: f64 = 0.25;
/// τ's upper bound. Source: §4.12 ("range 0.25 to 4.0").
pub const GEO_AGE_MAX: f64 = 4.0;
/// The saturating response's rate `k`. Source: §4.12, where it is stated as a
/// **judgement**, not a measured value: it gives `f_sat(0.25) = 0.30` and
/// `f_sat(4) = 2.20`.
pub const SATURATION_K: f64 = 0.5;
/// The stream-power iteration floor. Source: today's own `max(4, …)` in
/// `evolveCoupled`/`carveRiverValleys` (reference `Math.max(4,
/// Math.round(iters*0.6))`), kept by §4.12 ("`N_min` is 4 for stream power").
pub const STREAM_POWER_FLOOR: i32 = 4;
/// The floor for every pass-counted process. Source: §4.12 ("1 for every
/// pass"): a scaled process never silently stops running.
pub const PASS_FLOOR: i32 = 1;
/// The weathered mantle's e-folding depth `h₀`, metres. Source: §4.12, citing
/// Heimsath, Dietrich, Nishiizumi & Finkel (1997), *Nature* 388:358. **Not
/// yet verified at the source by this lane** (the GF-7 search reached the
/// paper's abstract, not the fitted coefficient); the law is unwired for the
/// reason [`mantle_thickness_m`] gives, so no world depends on it.
pub const MANTLE_H0_M: f64 = 0.5;
/// The weathered mantle at τ = 1, `h₁`, metres. Source: §4.12, stated as a
/// **judgement**.
pub const MANTLE_H1_M: f64 = 2.0;

/// The linear response `f(τ) = τ` (§4.12), for processes whose kernel already
/// saturates on its own (stream power lowers its own slope; the threshold
/// hillslope moves nothing below `θc`) or whose column bounds them (karst).
/// Must never be given to a process whose slowdown the kernel does not model:
/// that is what [`f_saturating`] is for.
pub fn f_linear(tau: f64) -> f64 {
    tau
}

/// The saturating response `f_sat(τ) = (1 − e^(−kτ)) / (1 − e^(−k))`, `k =`
/// [`SATURATION_K`] (§4.12), for processes that decelerate for a reason the
/// kernel does not model: a widening shore platform (cliff retreat, Sunamura
/// 1992, Trenhaile 2000) and a trough tending to a steady U-form (glacial,
/// Harbor 1992, MacGregor et al. 2000). `f_sat(1) = 1` by construction.
/// Must never be applied on top of a kernel that saturates by itself, which
/// would count the slowdown twice.
pub fn f_saturating(tau: f64) -> f64 {
    (1.0 - (-SATURATION_K * tau).exp()) / (1.0 - (-SATURATION_K).exp())
}

/// The weathered mantle `h(τ) = h₀ · ln(1 + τ·(e^(h₁/h₀) − 1))`, metres
/// (§4.12, from integrating Heimsath et al.'s exponential soil production
/// with no erosion, anchored at `h(1) = h₁`).
///
/// **Unwired in generation, deliberately** (GF-7, disclosed in §5.8): the law
/// gives `h(1) = h₁ = 2 m`, a mantle that does not exist in today's pipeline.
/// Writing it at τ = 1 would move every processes-on world at the default age,
/// which contradicts §4.12's own "at 1.0 every process takes today's
/// expressions" and B12's τ = 1 byte-identity; writing it only at τ ≠ 1 would
/// make τ = 0.95 and τ = 1 differ by a whole 1.9 m mantle. That is an owner
/// question, not a lane's choice. It also cannot move B1, B2 or B4: the
/// mantle's maximum (2.69 m at τ = 4) is below `R_EXPOSE` (5 m, §2.5), so it
/// never changes the exposed rock on its own.
pub fn mantle_thickness_m(tau: f64) -> f64 {
    MANTLE_H0_M * (1.0 + tau * ((MANTLE_H1_M / MANTLE_H0_M).exp() - 1.0)).ln()
}

/// One effective count, and whether its floor bound. `floored` is reported so
/// the UI can mark it (§4.12: "the clamp is not hidden").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Count {
    /// The count the process runs.
    pub n: i32,
    /// True when the unfloored `round(N·f(τ))` was below the floor.
    pub floored: bool,
}

/// `max(floor, js_round(base · f))`, with the floor's binding recorded.
/// `js_round` because today's own light-pass expression uses it (§4.12
/// "Floors").
fn scaled(base: f64, f: f64, floor: i32) -> Count {
    let raw = js_round(base * f) as i32;
    Count { n: raw.max(floor), floored: raw < floor }
}

/// A count that is today's `n` unchanged, floored at `floor` only if today's
/// expression floors it too (it does not, for the pass counts: they are
/// passed through).
fn unscaled(n: i32) -> Count {
    Count { n, floored: false }
}

/// The clock as the engine reads it: τ plus the gate. Constructed once per
/// generation, from `geology_processes` (and the column's presence) and
/// `geo_age`.
///
/// Must never scale while `on` is false: the clock is part of GF-2's gated
/// processes (`geology_processes`, off in the app), so the app's world does
/// not move until that switch is turned on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeoClock {
    on: bool,
    age: f64,
}

impl GeoClock {
    /// `on` is the rock gate (`geology_processes` and a column present);
    /// `age` is τ. A finite τ outside §4.12's range is clamped into it (the
    /// `PARAMS` row already clamps a GUI or save write the same way). A
    /// non-finite τ is not an age, so the clock **refuses to act** on it
    /// (today's counts) rather than panicking inside generation; `params::set`
    /// rejects non-finite input, so it is reachable only from a hand-built
    /// `WorldParams`.
    pub fn new(on: bool, age: f64) -> Self {
        if !age.is_finite() {
            return GeoClock { on: false, age: GEO_AGE_DEFAULT };
        }
        GeoClock { on, age: age.clamp(GEO_AGE_MIN, GEO_AGE_MAX) }
    }

    /// The τ this clock scales by (after the constructor's clamp).
    pub fn age(&self) -> f64 {
        self.age
    }

    /// Whether any count differs from today's expression. False at τ = 1
    /// exactly, and whenever the gate is off: the whole of the identity.
    pub fn is_scaling(&self) -> bool {
        // Exact comparison on purpose: 1.0 is the identity by control flow,
        // and 1.0 is exactly representable, so a slider at 1.00 reads it.
        self.on && self.age != GEO_AGE_DEFAULT
    }

    /// The light pass's (and each `evolve_cycles` cycle's) stream-power
    /// iterations. Today: `max(4, js_round(iters·0.6))`, verbatim. Scaled:
    /// `max(4, js_round(iters·0.6·τ))`, linear (§4.12 table, incision depth).
    pub fn light_pass_iters(&self, iters: i32) -> Count {
        if !self.is_scaling() {
            // Today's statement, verbatim: `(js_round(iters·0.6) as i32).max(4)`.
            let raw = js_round(iters as f64 * 0.6) as i32;
            return Count { n: raw.max(STREAM_POWER_FLOOR), floored: raw < STREAM_POWER_FLOOR };
        }
        scaled(iters as f64 * 0.6, f_linear(self.age), STREAM_POWER_FLOOR)
    }

    /// The sediment fill's stream-power iterations. Today: `iters`, verbatim
    /// (depositSediment runs the full count, with no floor). Scaled:
    /// `max(4, js_round(iters·τ))`, linear (§4.12 table: "`iters·τ` for the
    /// sediment fill", with stream power's floor).
    pub fn sediment_fill_iters(&self, iters: i32) -> Count {
        if !self.is_scaling() {
            return unscaled(iters);
        }
        scaled(iters as f64, f_linear(self.age), STREAM_POWER_FLOOR)
    }

    /// GF-3's threshold hillslope passes. Linear: the stage moves only the
    /// excess over `θc`, so it saturates by construction (§4.12 table).
    pub fn hillslope_passes(&self, n: i32) -> Count {
        if !self.is_scaling() {
            return unscaled(n);
        }
        scaled(n as f64, f_linear(self.age), PASS_FLOOR)
    }

    /// The glacial pass count. Saturating (§4.12 table): `glacial_kernel` runs
    /// a fixed per-pass abrasion law with no steady-U-form or overdeepening
    /// deceleration.
    pub fn glacial_passes(&self, n: i32) -> Count {
        if !self.is_scaling() {
            return unscaled(n);
        }
        scaled(n as f64, f_saturating(self.age), PASS_FLOOR)
    }

    /// The coastal pass count. Saturating (§4.12 table): `coastal_process`
    /// has no shore-platform term.
    pub fn coastal_passes(&self, n: i32) -> Count {
        if !self.is_scaling() {
            return unscaled(n);
        }
        scaled(n as f64, f_saturating(self.age), PASS_FLOOR)
    }
}

/// Every count the clock sets, for one set of parameters at one τ -- the
/// stage-06 readout (§4.12 "The UI"). The GUI asks for this rather than
/// re-deriving the formulas, so there is one copy of them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EffectiveCounts {
    /// Whether the clock acts on this world at all (`geology_model` and
    /// `geology_processes` both on). When false every count below is today's.
    pub active: bool,
    /// Light-pass stream-power iterations (and per `evolve_cycles` cycle).
    pub stream_power: Count,
    /// Sediment-fill stream-power iterations.
    pub sediment_fill: Count,
    /// Threshold-hillslope passes.
    pub hillslope: Count,
    /// Glacial passes.
    pub glacial: Count,
    /// Coastal passes.
    pub coastal: Count,
}

/// The counts `generate_terrain` would run for `p` with its age set to `tau`.
/// `active` follows the same gate generation uses, except that it cannot see
/// the column (none exists before generating), so it reads `geology_model`
/// in its place, which is what builds the column.
pub fn effective_counts(p: &crate::WorldParams, tau: f64) -> EffectiveCounts {
    let active = p.geology_model && p.geology_processes;
    let c = GeoClock::new(active, tau);
    EffectiveCounts {
        active,
        stream_power: c.light_pass_iters(p.stream.iters),
        sediment_fill: c.sediment_fill_iters(p.stream.iters),
        hillslope: c.hillslope_passes(cartalith_erosion::THRESHOLD_HILLSLOPE_PASSES),
        glacial: c.glacial_passes(p.passes.glacial_passes),
        coastal: c.coastal_passes(p.passes.coastal_passes),
    }
}

/// Pins the clock's math against literals independent of this module's own constants.
#[cfg(test)]
mod tests {
    //! Pins the clock's math (both response shapes, the mantle law) against
    //! literals independent of this module's own constants, and pins the
    //! τ=1/gate-off identity and the clamp/non-finite refusal that keep the
    //! clock from ever moving a world it must not touch.
    use super::*;

    /// Protects the saturating law's shape: literals from §4.12's own
    /// arithmetic (0.30 and 2.20) and from an independent evaluation
    /// (`python -c`, recorded in §5.8), never from `SATURATION_K` itself.
    #[test]
    fn saturating_response_matches_hand_worked_literals() {
        // Protects: f_saturating's shape against literals independent of
        // SATURATION_K, so a mutated K could not pass by re-deriving its own check.
        assert!((f_saturating(1.0) - 1.0).abs() < 1e-15, "f_sat(1) must be exactly the identity");
        assert!((f_saturating(0.25) - 0.298_633_426_760_995_6).abs() < 1e-12);
        assert!((f_saturating(2.0) - 1.606_530_659_712_633_4).abs() < 1e-12);
        assert!((f_saturating(4.0) - 2.197_540_261_032_505_4).abs() < 1e-12);
        assert_eq!(f_linear(2.5), 2.5);
    }

    /// Protects the mantle law against §4.12/B12's literal values (1.333 m at
    /// τ = 0.25, 2.686 m at τ = 4, the scope's three-decimal figures; the
    /// 1e-9 literals are an independent `python -c` evaluation). Never
    /// asserted against the constants.
    #[test]
    fn mantle_law_matches_the_scopes_literals() {
        // Protects: mantle_thickness_m's closed form against the scope's own
        // hand-worked figures, independent of MANTLE_H0_M/MANTLE_H1_M.
        assert!((mantle_thickness_m(0.25) - 1.333_598_044_293_021_3).abs() < 1e-9);
        assert!((mantle_thickness_m(4.0) - 2.686_231_205_029_357_3).abs() < 1e-9);
        assert!((mantle_thickness_m(1.0) - 2.0).abs() < 1e-12);
        assert!((mantle_thickness_m(0.25) - 1.333).abs() < 1e-3);
        assert!((mantle_thickness_m(4.0) - 2.686).abs() < 1e-3);
    }

    /// Protects the τ = 1 identity by control flow: at the default age, and
    /// with the gate off at any age, every count is today's expression
    /// (literals for `iters: 15`, `glacial_passes: 8`, `coastal_passes: 4`,
    /// `N_h = 8`).
    #[test]
    fn tau_one_and_gate_off_are_todays_counts() {
        // Protects: the τ = 1 / gate-off identity by control flow -- every count
        // must equal today's unscaled expression, never a scaled one that
        // happens to round to the same value.
        for c in [GeoClock::new(true, 1.0), GeoClock::new(false, 4.0), GeoClock::new(false, 0.25)] {
            assert!(!c.is_scaling());
            assert_eq!(c.light_pass_iters(15).n, 9);
            assert_eq!(c.light_pass_iters(4).n, 4, "today's floor: round(2.4) = 2 -> 4");
            assert!(c.light_pass_iters(4).floored);
            assert_eq!(c.sediment_fill_iters(15).n, 15);
            assert_eq!(c.sediment_fill_iters(3).n, 3, "today's sediment fill has no floor");
            assert_eq!(c.hillslope_passes(8).n, 8);
            assert_eq!(c.glacial_passes(8).n, 8);
            assert_eq!(c.coastal_passes(4).n, 4);
        }
    }

    /// Protects each process's response shape and floor: linear for stream
    /// power and hillslope, saturating for glacial and coastal, and the
    /// floors 4 and 1. Literals from §4.12's arithmetic ("36 iterations
    /// instead of 9"; "any τ below 0.5 runs 4") and `python -c`.
    #[test]
    fn scaled_counts_follow_the_scope_table() {
        // Protects: each process's response shape (linear vs. saturating) and
        // floor, against the scope's own worked table -- a swapped shape or a
        // dropped floor would still pass a golden that never varies τ.
        let c = |t| GeoClock::new(true, t);
        // Light pass: max(4, round(9τ)).
        assert_eq!(c(4.0).light_pass_iters(15), Count { n: 36, floored: false });
        assert_eq!(c(2.0).light_pass_iters(15), Count { n: 18, floored: false });
        assert_eq!(c(0.5).light_pass_iters(15), Count { n: 5, floored: false }, "round(4.5) is 5 (half up)");
        assert_eq!(c(0.25).light_pass_iters(15), Count { n: 4, floored: true });
        // Sediment fill: max(4, round(15τ)).
        assert_eq!(c(4.0).sediment_fill_iters(15).n, 60);
        assert_eq!(c(0.25).sediment_fill_iters(15), Count { n: 4, floored: false }, "round(3.75) = 4, not floored");
        assert_eq!(c(0.25).sediment_fill_iters(10), Count { n: 4, floored: true });
        // Hillslope: linear, floor 1.
        assert_eq!(c(4.0).hillslope_passes(8).n, 32);
        assert_eq!(c(0.5).hillslope_passes(8).n, 4);
        assert_eq!(c(0.25).hillslope_passes(1), Count { n: 1, floored: true });
        // Glacial: round(8·f_sat(τ)).
        assert_eq!(c(4.0).glacial_passes(8).n, 18);
        assert_eq!(c(2.0).glacial_passes(8).n, 13);
        assert_eq!(c(0.5).glacial_passes(8).n, 4);
        assert_eq!(c(0.25).glacial_passes(8).n, 2);
        // Coastal: round(4·f_sat(τ)).
        assert_eq!(c(4.0).coastal_passes(4).n, 9);
        assert_eq!(c(2.0).coastal_passes(4).n, 6);
        assert_eq!(c(0.25).coastal_passes(4), Count { n: 1, floored: false }, "round(1.19) = 1");
        assert_eq!(c(0.25).coastal_passes(1), Count { n: 1, floored: true });
    }

    /// Protects the constructor's refusals: out-of-range τ clamps to the
    /// scope's range; a non-finite τ is not an age and scales nothing.
    #[test]
    fn constructor_clamps_and_refuses_non_finite() {
        // Protects: GeoClock::new's clamp to [GEO_AGE_MIN, GEO_AGE_MAX] and its
        // refusal to scale on a non-finite τ (NaN/infinity), rather than
        // panicking inside generation.
        assert_eq!(GeoClock::new(true, 9.0).light_pass_iters(15).n, 36);
        assert_eq!(GeoClock::new(true, 0.0).age(), 0.25);
        assert!(!GeoClock::new(true, f64::NAN).is_scaling());
        assert!(!GeoClock::new(true, f64::INFINITY).is_scaling());
    }

    /// Protects the readout's gate: it reports today's counts while either
    /// switch is off, and the scaled ones only with both on.
    #[test]
    fn effective_counts_follow_the_gate() {
        // Protects: effective_counts's gate -- both geology_model AND
        // geology_processes must be on for the clock to act, since a column
        // (built by geology_model) is what geology_processes reads.
        let mut p = crate::WorldParams::defaults(64, 40, 1);
        let off = effective_counts(&p, 4.0);
        assert!(!off.active);
        assert_eq!(off.stream_power.n, 9);
        // Processes on without the model: no column, so no clock either.
        p.geology_processes = true;
        assert!(!effective_counts(&p, 4.0).active, "geology_model off: inactive");
        p.geology_processes = false;
        p.geology_model = true;
        assert!(!effective_counts(&p, 4.0).active, "geology_processes off: inactive");
        p.geology_processes = true;
        let on = effective_counts(&p, 4.0);
        assert!(on.active);
        assert_eq!((on.stream_power.n, on.hillslope.n, on.glacial.n, on.coastal.n, on.sediment_fill.n), (36, 32, 18, 9, 60));
    }
}
