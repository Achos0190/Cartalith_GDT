//! Per-faction currencies — Ruling R (2026-09-21), kept by Ruling AR and
//! shaped by **Ruling AU** (2026-09-24, `LARGE_ITEM_RULINGS.md`): *"Each
//! faction carries its own currency; the user types its rate in the faction
//! roster and the engine only converts. No rate is derived from the
//! economy."*
//!
//! So this module is conversion and nothing else. It holds no state (the
//! rates live on the roster, `cartalith-godot`'s `civ_roster_bridge.rs`),
//! derives no rate, and nothing in the simulation reads it: the trade match
//! ([`crate::trade::trade_flows`]) prices every good in Ruling AB's world
//! scarcity index and is untouched by any rate. A rate changes what a
//! readout *says*, never what a flow *is*.
//!
//! ## The unit a rate is quoted against
//!
//! A rate is **units of the faction's currency per one unit of the world
//! price index** ([`crate::trade::scarcity_price`]'s unit, which is `1.0`
//! for a good whose world demand and supply balance). A faction at rate `12`
//! pays 12 of its own units for a balanced good; a flow worth `3.5`
//! index-units costs its importer `42`. Quoting every rate against the one
//! world index is what makes any two factions' currencies exchangeable
//! without a pairwise table: `a → b` is `amount / rate_a · rate_b`.
//!
//! ## A bad rate is refused, never replaced
//!
//! A rate is valid only when it is finite and strictly positive. Zero would
//! make every price free and every exchange out of it a division by zero;
//! a negative rate has no reading at all; `NaN`/`inf` poison whatever they
//! touch. Every function here returns `None` for an invalid rate rather
//! than falling back to `1.0` — a silent `1.0` would print a plausible
//! number in a currency the user never agreed to.

/// Whether `rate` can be used: finite and `> 0`.
pub fn valid_rate(rate: f64) -> bool {
    rate.is_finite() && rate > 0.0
}

/// An amount in world price-index units expressed in a currency quoted at
/// `rate` (units per index unit). `None` for an invalid rate, and for a
/// result that is not finite -- a `NaN`/infinite amount, or an overflow.
pub fn to_currency(index_amount: f64, rate: f64) -> Option<f64> {
    if !valid_rate(rate) {
        return None;
    }
    let v = index_amount * rate;
    v.is_finite().then_some(v)
}

/// The inverse of [`to_currency`]: an amount in a currency quoted at `rate`,
/// back in world price-index units. Same refusals.
pub fn to_index(amount: f64, rate: f64) -> Option<f64> {
    if !valid_rate(rate) {
        return None;
    }
    let v = amount / rate;
    v.is_finite().then_some(v)
}

/// `amount` of the currency quoted at `from_rate`, in the currency quoted at
/// `to_rate`: through the world index, `amount / from_rate · to_rate`.
/// `None` when either rate is invalid.
pub fn exchange(amount: f64, from_rate: f64, to_rate: f64) -> Option<f64> {
    to_currency(to_index(amount, from_rate)?, to_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_to_currency_multiplies_by_the_rate() {
        assert_eq!(to_currency(3.5, 12.0), Some(42.0));
        assert_eq!(to_currency(1.0, 0.25), Some(0.25));
        assert_eq!(to_currency(0.0, 7.0), Some(0.0), "a zero amount is a real zero");
        assert_eq!(to_index(42.0, 12.0), Some(3.5));
        assert_eq!(to_index(0.25, 0.25), Some(1.0));
    }

    #[test]
    fn exchange_goes_through_the_world_index() {
        // 10 units at rate 4 are 2.5 index units, which at rate 6 are 15.
        assert_eq!(exchange(10.0, 4.0, 6.0), Some(15.0));
        assert_eq!(exchange(15.0, 6.0, 4.0), Some(10.0));
        // Same currency: identity.
        assert_eq!(exchange(7.75, 3.0, 3.0), Some(7.75));
        assert_eq!(exchange(1.0, 0.5, 2.0), Some(4.0));
    }

    #[test]
    fn a_round_trip_returns_the_amount() {
        // Rates chosen to be exact in binary, so the round trip is exact.
        let there = exchange(123.0, 0.5, 8.0).unwrap();
        assert_eq!(there, 1968.0);
        assert_eq!(exchange(there, 8.0, 0.5), Some(123.0));
        let c = to_currency(1.25, 16.0).unwrap();
        assert_eq!(c, 20.0);
        assert_eq!(to_index(c, 16.0), Some(1.25));
    }

    #[test]
    fn an_invalid_rate_is_refused_not_replaced_by_one() {
        for bad in [0.0, -0.0, -2.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(!valid_rate(bad), "{bad}");
            assert_eq!(to_currency(5.0, bad), None, "{bad}");
            assert_eq!(to_index(5.0, bad), None, "{bad}");
            assert_eq!(exchange(5.0, bad, 2.0), None, "from {bad}");
            assert_eq!(exchange(5.0, 2.0, bad), None, "to {bad}");
        }
        assert!(valid_rate(1e-9) && valid_rate(1e9));
        assert_eq!(to_currency(f64::NAN, 2.0), None, "a NaN amount is not converted");
        assert_eq!(to_currency(1e308, 1e10), None, "overflow is refused, not inf");
    }
}
