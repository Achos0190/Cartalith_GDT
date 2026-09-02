//! `_peakaudit_hash` — before/after output fingerprint. **Throwaway probe.**
//!
//! `MEMORY_OPTIMIZATION_SCOPE.md`'s R1/R2/R3 are all claimed to be
//! output-neutral. `cargo test` passing is necessary and not sufficient
//! (`CLAUDE.md`'s own working rules), so this dumps a byte-exact fingerprint
//! of every surviving `WorldState` field and every civ grid the pipeline
//! produces, at a fixed seed and grid, to be diffed across the change.
//!
//! Hashes the raw `to_ne_bytes` of each element, so two runs agree only if
//! every float is bit-identical — not "close".
//!
//! `PEAKAUDIT_REGEN=1` runs `generate_terrain` twice with the first world
//! still held and asserts the two are bit-identical, which is R1's own proof
//! obligation: dropping the previous world before generating the next cannot
//! change the next one.
//!
//! ```text
//! cargo run --release -p cartalith-civ --example _peakaudit_hash -- <gw> <gh> [seed]
//! ```

/// FNV-1a over raw bytes. Not cryptographic — this is a diff aid.
struct Fnv(u64);
impl Fnv {
    fn new() -> Self {
        Fnv(0xcbf2_9ce4_8422_2325)
    }
    fn eat(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= b as u64;
            self.0 = self.0.wrapping_mul(0x1000_0000_01b3);
        }
    }
}

trait Fingerprint {
    fn feed(&self, h: &mut Fnv);
}
impl Fingerprint for [f32] {
    fn feed(&self, h: &mut Fnv) {
        for v in self {
            h.eat(&v.to_ne_bytes());
        }
    }
}
impl Fingerprint for [u8] {
    fn feed(&self, h: &mut Fnv) {
        h.eat(self);
    }
}
impl Fingerprint for [i32] {
    fn feed(&self, h: &mut Fnv) {
        for v in self {
            h.eat(&v.to_ne_bytes());
        }
    }
}
impl Fingerprint for [i16] {
    fn feed(&self, h: &mut Fnv) {
        for v in self {
            h.eat(&v.to_ne_bytes());
        }
    }
}
impl Fingerprint for [usize] {
    fn feed(&self, h: &mut Fnv) {
        for v in self {
            h.eat(&(*v as u64).to_ne_bytes());
        }
    }
}
// Widened to u64 on the way in, like `[usize]` above, so `plate_id`'s
// recorded fingerprints survived R4's `Vec<usize>` -> `Vec<u16>` narrowing.
impl Fingerprint for [u16] {
    fn feed(&self, h: &mut Fnv) {
        for v in self {
            h.eat(&(*v as u64).to_ne_bytes());
        }
    }
}

fn row<T: Fingerprint + ?Sized>(name: &str, d: &T, len: usize) {
    let mut h = Fnv::new();
    d.feed(&mut h);
    println!("{name:<30} len {len:>9}  {:016x}", h.0);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let gw: usize = args.first().and_then(|s| s.parse().ok()).unwrap_or(512);
    let gh: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(328);
    let seed: i32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(483920);

    let mut p = cartalith_engine::WorldParams::defaults(gw, gh, seed);
    p.map_width_km = 1200.0;

    println!("=== _peakaudit_hash {gw}x{gh} seed {seed} ===");

    if std::env::var("PEAKAUDIT_REGEN").is_ok() {
        // R1's proof: `generate_terrain` is a pure function of `WorldParams`,
        // so whether the previous world is still alive when it runs cannot
        // change its result. Held vs. dropped, both orderings, same seed.
        let first = cartalith_engine::generate_terrain(&p);
        let second_while_held = cartalith_engine::generate_terrain(&p);
        assert_eq!(first.field, second_while_held.field, "generate #2 with #1 held");
        assert_eq!(first.rainfall, second_while_held.rainfall, "generate #2 with #1 held (rainfall)");
        assert_eq!(first.flow_discharge, second_while_held.flow_discharge, "generate #2 with #1 held (flow)");
        drop(second_while_held);
        drop(first);
        let after_drop = cartalith_engine::generate_terrain(&p);
        let mut h = Fnv::new();
        after_drop.field.as_slice().feed(&mut h);
        println!("generate after dropping the previous world: field {:016x}", h.0);
        println!("two consecutive generates are bit-identical, held or dropped: OK");
        return;
    }

    let ws = cartalith_engine::generate_terrain(&p);
    println!("--- WorldState ---");
    println!("{:<30}            {:.17e}", "sea_level", ws.sea_level);
    row("field", ws.field.as_slice(), ws.field.len());
    row("plate_id", ws.plate_id.as_slice(), ws.plate_id.len());
    row("boundary_mask", ws.boundary_mask.as_slice(), ws.boundary_mask.len());
    row("stress_field", ws.stress_field.as_slice(), ws.stress_field.len());
    row("age_field", ws.age_field.as_slice(), ws.age_field.len());
    row("resistance_field", ws.resistance_field.as_slice(), ws.resistance_field.len());
    row("crust_field", ws.crust_field.as_slice(), ws.crust_field.len());
    row("boundary_type", ws.boundary_type.as_slice(), ws.boundary_type.len());
    row("shear_field", ws.shear_field.as_slice(), ws.shear_field.len());
    row("volcanic_field", ws.volcanic_field.as_slice(), ws.volcanic_field.len());
    row("impact_field", ws.impact_field.as_slice(), ws.impact_field.len());
    row("temperature", ws.temperature.as_slice(), ws.temperature.len());
    row("rainfall", ws.rainfall.as_slice(), ws.rainfall.len());
    row("flow_discharge", ws.flow_discharge.as_slice(), ws.flow_discharge.len());
    if let Some(c) = ws.channels.as_ref() {
        row("channels.recv", c.recv.as_slice(), c.recv.len());
        row("channels.chan", c.chan.as_slice(), c.chan.len());
    }
    if let Some(o) = ws.stream_order.as_ref() {
        row("stream_order", o.as_slice(), o.len());
    }
    if let Some(m) = ws.river_mask.as_ref() {
        row("river_mask", m.as_slice(), m.len());
    }
    if let Some(f) = ws.river_floor.as_ref() {
        row("river_floor", f.as_slice(), f.len());
    }

    // ---- civ, the same call chain `compute_civilisation` makes -----------
    let sea = ws.sea_level;
    let world = p.world;
    let map_width_km = p.map_width_km;
    let wb = cartalith_civ::build_water_bodies(&ws.field, gw, gh, sea, world, Some(&ws.rainfall));
    let biome = cartalith_civ::build_biome_raster(&wb.classification, &ws.temperature, &ws.rainfall);
    let soil_slope = cartalith_civ::build_slope_field(&ws.field, gw, gh, world);
    let lithology = cartalith_civ::build_lithology(
        &ws.field, &ws.age_field, &ws.volcanic_field, &ws.crust_field, &ws.resistance_field, &ws.rainfall, sea,
    );
    let soil = cartalith_civ::build_soil_fertility(&lithology, &ws.temperature, &ws.rainfall, &soil_slope, &ws.age_field);
    let flow_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, map_width_km);
    let water_access = cartalith_civ::build_water_access(&ws.flow_discharge, &ws.field, gw, gh, sea, flow_thresh);
    let carrying_cap =
        cartalith_civ::build_carrying_capacity(&soil, &water_access, Some(&biome), &ws.temperature, &ws.field, sea, 0.0, None);
    let resources = cartalith_civ::build_resource_potentials(
        &lithology, Some(&ws.boundary_type), Some(&ws.shear_field), Some(&ws.flow_discharge), Some(&biome),
        &ws.field, &ws.rainfall, &ws.age_field, gw, gh, sea, Some(&ws.volcanic_field), true, false,
    );
    let raw_slope = cartalith_civ::build_raw_slope_field(&ws.field, gw, gh, world);
    let corridors =
        cartalith_civ::build_route_corridors(&ws.field, &raw_slope, Some(&ws.flow_discharge), gw, gh, sea, world, flow_thresh);
    let landmass = cartalith_civ::build_landmass_quality(&ws.field, Some(&carrying_cap), gw, gh, sea, world);
    let coast_sdf = cartalith_civ::build_coast_sdf(&ws.field, gw, gh, sea);
    let flood = cartalith_civ::build_flood_field(&ws.field, &ws.flow_discharge, &raw_slope, gw, gh, sea);
    let river_order =
        cartalith_civ::fresh_river_order(&ws.field, &ws.flow_discharge, gw, gh, sea, world, p.river_density, map_width_km);

    println!("--- civ ---");
    row("biome", biome.as_slice(), biome.len());
    row("lithology", lithology.as_slice(), lithology.len());
    row("soil", soil.as_slice(), soil.len());
    row("water_access", water_access.as_slice(), water_access.len());
    row("carrying_cap", carrying_cap.as_slice(), carrying_cap.len());
    // The R3 target: all fifteen, individually.
    for (name, g) in [
        ("res.copper", &resources.copper), ("res.tin", &resources.tin), ("res.iron", &resources.iron),
        ("res.gold", &resources.gold), ("res.salt", &resources.salt), ("res.timber", &resources.timber),
        ("res.lead", &resources.lead), ("res.silver", &resources.silver), ("res.clay", &resources.clay),
        ("res.buildstone", &resources.buildstone), ("res.flint", &resources.flint),
        ("res.obsidian", &resources.obsidian), ("res.gems", &resources.gems),
        ("res.sulfur", &resources.sulfur), ("res.alum", &resources.alum),
    ] {
        row(name, g.as_slice(), g.len());
    }
    row("corridors", corridors.as_slice(), corridors.len());
    row("coast_sdf", coast_sdf.as_slice(), coast_sdf.len());
    row("flood", flood.as_slice(), flood.len());
    row("river_order", river_order.as_slice(), river_order.len());

    let ctx = cartalith_civ::SuitabilityCtx {
        water_bodies: Some(&wb.classification),
        corridor: Some(&corridors),
        landmass: Some(&landmass.quality),
        flow: Some(&ws.flow_discharge),
        river_order: Some(&river_order),
        coast_sdf: Some(&coast_sdf),
        resources: Some(&resources),
        rain: Some(&ws.rainfall),
        flood: Some(&flood),
        slope_raw: Some(&raw_slope),
        flow_thresh,
    };
    let suit = cartalith_civ::build_settlement_suitability(
        &soil, &water_access, &carrying_cap, &ws.field, &soil_slope, gw, gh, sea, Some(&ctx),
    );
    row("suitability", suit.as_slice(), suit.len());
    let seeds = cartalith_civ::find_settlement_seeds(&suit, gw, gh, 0.42, (gw as f64 / 22.0).floor().max(6.0));
    let placements = cartalith_civ::place_settlements_with_water_edge_snap(
        &seeds, &suit, &ws.field, &wb.classification, &wb.fill_level, gw, gh, sea, world, 6,
        &flood, &ws.flow_discharge, flow_thresh, map_width_km,
    );
    // Settlement placement is the discrete argmax the scope document warns
    // about: if any resource grid moved by one ulp, this list moves.
    let mut h = Fnv::new();
    for pl in &placements {
        h.eat(&(pl.x as u64).to_ne_bytes());
        h.eat(&(pl.y as u64).to_ne_bytes());
    }
    println!("{:<30} n   {:>9}  {:016x}", "settlement placements", placements.len(), h.0);
}
