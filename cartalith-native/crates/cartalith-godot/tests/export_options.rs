//! The export options (`EXPORT_SCOPE.md` §7, milestone E3) — the Godot-free
//! half in `export_options.rs`. The `Dictionary` walk, `export_image` and the
//! non-mutation of the session appearance need a live `WorldGen` and are
//! checked by `godot-project/_exportoptions_probe.gd`.
//!
//! Every band-count figure below is a **literal worked by hand** from
//! `ExportBandPlan::for_budget`'s documented rule, not read back from the
//! code, so a change to the budget, the apron or the plan arithmetic turns
//! these red rather than moving with them.

#[path = "../src/render.rs"]
mod render;

#[path = "../src/export_stream.rs"]
mod export_stream;

#[path = "../src/export_options.rs"]
mod export_options;

use export_options::{ExportContent, ExportStyle, SettlementTier, band_budget_px, band_peak_px};
use export_stream::StreamFormat;
use render::{ExportBandPlan, TerrainAppearance};

/// The app's own 2048 × 1311 grid, and `bake_dims` at each rung:
/// `round(W · 1311 / 2048)`.
const LADDER: [(usize, usize); 5] = [(2048, 1311), (4096, 2622), (8192, 5244), (16384, 10488), (32768, 20976)];

/// `8192 × 5244`: the whole raster at `UNGATED_MAX_WIDTH` on that grid.
const UNGATED_PX: u64 = 42_958_848;

fn plan(a: &TerrainAppearance, w: usize, h: usize) -> ExportBandPlan {
    ExportBandPlan::for_budget(a, w, h, band_budget_px(UNGATED_PX, None, 23))
}

/// The numbers `export_raster_estimate` reports as `bands` / `band_rows` /
/// `apron_rows` / `band_peak_bytes`, at the default appearance (local
/// contrast on, radius 0.010 of the width).
///
/// 16K: 42 958 848 / 16 384 = 2 622 rows fit; apron round(163.84) = 164;
/// 2 622 − 328 = 2 294 rows a band; ⌈10 488 / 2 294⌉ = 5.
/// 32K: 1 311 fit; apron round(327.68) = 328; 1 311 − 656 = 655; ⌈20 976 / 655⌉ = 33.
#[test]
fn the_band_plan_at_every_rung_is_the_hand_worked_one() {
    let a = TerrainAppearance::default();
    assert!(a.local_contrast > 0.0, "the fixture must have an apron to plan around");
    let want = [(1, 1311, 0), (1, 2622, 0), (1, 5244, 0), (5, 2294, 164), (33, 655, 328)];
    for ((w, h), (bands, rows, apron)) in LADDER.into_iter().zip(want) {
        let p = plan(&a, w, h);
        assert_eq!((p.band_count(), p.rows_per_band, p.apron), (bands, rows, apron), "{w} x {h}");
    }
}

/// 2K/4K/8K are one band with no apron — E1's monolithic path, not a banded
/// approximation of it — and 16K/32K cost **exactly** what an 8K export
/// always has: 42 958 848 px × 23 B = 988 053 504 bytes.
#[test]
fn a_banded_export_peaks_at_the_ungated_8k_figure() {
    let a = TerrainAppearance::default();
    for (w, h) in LADDER {
        let peak = band_peak_px(&plan(&a, w, h)) * 23;
        if w <= 8192 {
            assert_eq!(peak, w as u64 * h as u64 * 23, "{w}: one band is the whole raster");
        } else {
            assert_eq!(peak, 988_053_504, "{w}: the tallest band, apron included");
        }
    }
}

/// The apron comes from the appearance, so a style override that turns
/// local contrast off moves the band count: 16K 2 622 rows a band, exactly 4
/// bands; 32K 1 311, exactly 16.
#[test]
fn a_style_without_local_contrast_plans_fewer_bands() {
    let style = ExportStyle { tunables: vec![export_options::tunable_from("local_contrast", 0.0).unwrap()], ..Default::default() };
    let a = style.apply_edits(TerrainAppearance::default());
    assert_eq!(a.local_contrast, 0.0);
    let p16 = plan(&a, 16384, 10488);
    let p32 = plan(&a, 32768, 20976);
    assert_eq!((p16.band_count(), p16.rows_per_band, p16.apron), (4, 2622, 0));
    assert_eq!((p32.band_count(), p32.rows_per_band, p32.apron), (16, 1311, 0));
}

/// A device reporting less than the 8K figure gets thinner bands, not a
/// refusal: 500 000 000 B / 23 = 21 739 130 px, so 32K fits 663 rows and
/// 663 − 656 = 7 rows a band, ⌈20 976 / 7⌉ = 2 997 bands.
#[test]
fn a_small_memory_budget_thins_the_bands() {
    assert_eq!(band_budget_px(UNGATED_PX, Some(500_000_000), 23), 21_739_130);
    assert_eq!(band_budget_px(UNGATED_PX, Some(u64::MAX), 23), UNGATED_PX, "never above the ungated ceiling");
    assert_eq!(band_budget_px(UNGATED_PX, None, 23), UNGATED_PX);
    let a = TerrainAppearance::default();
    let p = ExportBandPlan::for_budget(&a, 32768, 20976, band_budget_px(UNGATED_PX, Some(500_000_000), 23));
    assert_eq!((p.band_count(), p.rows_per_band), (2997, 7));
}

/// The style's edits: a ramp preset keeps the mode in force (as
/// `load_ramp_preset` does), tunables clamp to the published range, and
/// `relief_lights` rounds. The input is taken by value — the caller's copy
/// is not reachable from here — and a second application from the same base
/// gives the same answer.
#[test]
fn the_style_edits_layer_on_a_copy() {
    let mut base = TerrainAppearance::default();
    base.ramp.set_mode(render::RampMode::Step);
    let before = serde_json::to_value(&base).unwrap();
    let style = ExportStyle {
        ramp: export_options::ramp_from_name(render::RAMP_PRESETS[1].0),
        tunables: vec![export_options::tunable_from("exag", 99.0).unwrap(), export_options::tunable_from("relief_lights", 3.6).unwrap()],
        ..Default::default()
    };
    let a = style.apply_edits(base.clone());
    assert_eq!(a.exag, 12.0, "clamped to TUNABLE's 0..12");
    assert_eq!(a.relief_lights, 4);
    assert_eq!(a.ramp.mode(), render::RampMode::Step, "the mode survives a ramp preset");
    let mut preset = render::ElevationRamp::preset(render::RAMP_PRESETS[1].0).unwrap();
    preset.set_mode(render::RampMode::Step);
    assert_eq!(a.ramp, preset);
    assert_ne!(a.ramp, base.ramp, "the ramp actually changed");
    assert_eq!(serde_json::to_value(&base).unwrap(), before, "the base is untouched");
    assert_eq!(serde_json::to_value(style.apply_edits(base)).unwrap(), serde_json::to_value(&a).unwrap());
    assert!(ExportStyle::default().is_empty());
}

#[test]
fn names_are_validated_not_defaulted() {
    assert_eq!(export_options::look_from_name("antique parchment"), Some(render::LOOK_ANTIQUE));
    assert_eq!(export_options::look_from_name("Sepia"), None);
    assert_eq!(export_options::ramp_from_name("no such ramp"), None);
    assert!(export_options::tunable_from("exagg", 1.0).is_err());
    assert!(export_options::tunable_from("exag", f64::NAN).is_err());
    assert_eq!(export_options::format_from_name("PNG"), Some(StreamFormat::Png));
    assert_eq!(export_options::format_from_name("bigtiff"), Some(StreamFormat::BigTiff));
    assert_eq!(export_options::format_from_name("webp"), None);
    assert_eq!(export_options::format_name(StreamFormat::BigTiff), "bigtiff");
}

/// The preset parser `load_appearance_preset` and the export now share:
/// `save_appearance_preset`'s own document shape round-trips, and each
/// malformed shape is an error naming its fault.
#[test]
fn a_saved_preset_parses_and_a_bad_one_is_refused() {
    let mut a = TerrainAppearance::default().with_look(render::LOOK_ANTIQUE);
    a.exag = 7.25;
    let doc = serde_json::json!({ "format": "cartalith-appearance", "v": 1, "name": "x", "appearance": a });
    let back = export_options::parse_appearance_preset(&doc.to_string()).unwrap();
    assert_eq!(serde_json::to_value(&back).unwrap(), serde_json::to_value(&a).unwrap());
    let err = |t: &str| export_options::parse_appearance_preset(t).err().expect("must be refused");
    assert!(err("{").starts_with("parse failed"));
    assert_eq!(err(r#"{"format":"other","appearance":{}}"#), "not a Cartalith appearance preset");
    assert_eq!(err(r#"{"format":"cartalith-appearance"}"#), "no appearance block");
    assert!(export_options::read_appearance_preset(std::path::Path::new("Z:/no/such/preset.json")).err().expect("must be refused").starts_with("open failed"));
}

/// §5's explicit tier, largest first: a tier draws itself and everything
/// above it; only `all` reaches the additive villages.
#[test]
fn a_settlement_tier_includes_everything_above_it() {
    let t = SettlementTier::from_name("town").unwrap();
    for (kind, drawn) in [("metropolis", true), ("capital", true), ("city", true), ("town", true), ("village", false), ("hamlet", false), ("village_addon", false)] {
        assert_eq!(t.includes(kind), drawn, "town tier, {kind}");
    }
    assert!(SettlementTier::Hamlet.includes("hamlet"));
    assert!(!SettlementTier::Hamlet.includes("village_addon"));
    assert!(SettlementTier::All.includes("village_addon"));
    assert!(!SettlementTier::All.includes("castle"));
    assert_eq!(SettlementTier::from_name("all").map(SettlementTier::name), Some("all"));
}

/// The default content is what `export_raster_png` draws — terrain and
/// rivers, nothing pending — and every overlay asked for is reported by name.
#[test]
fn overlays_are_reported_not_dropped() {
    let mut c = ExportContent::default();
    assert!(c.rivers);
    assert!(c.overlays().is_empty());
    c.settlements = Some(SettlementTier::City);
    for key in ExportContent::OVERLAY_KEYS {
        assert!(c.set_overlay(key, true));
    }
    assert!(!c.set_overlay("rivers", true), "rivers is drawable, not an overlay");
    assert_eq!(c.overlays(), ["settlements", "routes", "ways", "labels", "territory", "icons"]);
}
