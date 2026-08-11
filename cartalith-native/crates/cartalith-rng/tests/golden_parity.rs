//! Golden-parity test for `mulberry32` (PARITY_TESTING.md).
//!
//! Golden values extracted by running the JS engine's own `mulberry32`
//! (reference HTML, line 2291) under real Node.js (v24.19.0) — not
//! hand-derived. `mulberry32` is pure 32-bit integer arithmetic plus one
//! exact power-of-two division, so unlike later pipeline stages this one
//! is held to bit-for-bit equality rather than a tolerance: any drift here
//! means an operation, constant, or precedence mistake, not float noise.

use cartalith_rng::Mulberry32;

struct Case {
    seed: u32,
    vals: [f64; 8],
}

const CASES: &[Case] = &[
    Case {
        seed: 0,
        vals: [
            0.26642920868471265,
            0.0003297457005828619,
            0.2232720274478197,
            0.1462021479383111,
            0.46732782293111086,
            0.5450490827206522,
            0.6152513844426721,
            0.6489853798411787,
        ],
    },
    Case {
        seed: 1,
        vals: [
            0.6270739405881613,
            0.002735721180215478,
            0.5274470399599522,
            0.9810509674716741,
            0.9683778982143849,
            0.281103502959013,
            0.6128388606011868,
            0.7207431411370635,
        ],
    },
    Case {
        seed: 42,
        vals: [
            0.6011037519201636,
            0.44829055899754167,
            0.8524657934904099,
            0.6697340414393693,
            0.17481389874592423,
            0.5265925421845168,
            0.2732279943302274,
            0.6247446539346129,
        ],
    },
    Case {
        seed: 12345,
        vals: [
            0.9797282677609473,
            0.3067522644996643,
            0.484205421525985,
            0.817934412509203,
            0.5094283693470061,
            0.34747186047025025,
            0.07375754183158278,
            0.7663964673411101,
        ],
    },
    // state.tect.seed ^ 0x5bf03635 — the volcanism RNG's actual seed derivation.
    Case {
        seed: 0x5bf03635,
        vals: [
            0.2687344925943762,
            0.3486885984893888,
            0.7327678145375103,
            0.6295806602574885,
            0.0043168345000594854,
            0.3800378171727061,
            0.20431688730604947,
            0.7880281419493258,
        ],
    },
    // state.tect.seed ^ 0x27d4eb2f — the crater RNG's actual seed derivation.
    Case {
        seed: 0x27d4eb2f,
        vals: [
            0.6837094351649284,
            0.4740110815037042,
            0.4100087385158986,
            0.9908855312969536,
            0.0951343416236341,
            0.5865072789601982,
            0.24998902366496623,
            0.6635485957376659,
        ],
    },
    Case {
        seed: 2166136261, // FNV offset basis — an actual seed-derivation input elsewhere in the engine
        vals: [
            0.6112444521859288,
            0.4935242917854339,
            0.7740248835179955,
            0.4122861116193235,
            0.8122657814528793,
            0.05720820324495435,
            0.9159039182122797,
            0.19360002595931292,
        ],
    },
    Case {
        seed: u32::MAX,
        vals: [
            0.8964226141106337,
            0.189478256739676,
            0.7156526781618595,
            0.9440599093213677,
            0.8452364315744489,
            0.5391399988438934,
            0.6804977387655526,
            0.4755720964167267,
        ],
    },
    Case {
        seed: 999999937,
        vals: [
            0.4222431951202452,
            0.7476034415885806,
            0.033541144570335746,
            0.15489679691381752,
            0.4536716346628964,
            0.2804461740888655,
            0.856395430630073,
            0.9628198486752808,
        ],
    },
];

#[test]
fn matches_js_mulberry32_bit_for_bit() {
    for case in CASES {
        let mut rng = Mulberry32::new(case.seed);
        for (i, &expected) in case.vals.iter().enumerate() {
            let actual = rng.next_f64();
            assert_eq!(
                actual, expected,
                "seed {} step {}: got {actual}, expected {expected} (JS mulberry32)",
                case.seed, i
            );
        }
    }
}
