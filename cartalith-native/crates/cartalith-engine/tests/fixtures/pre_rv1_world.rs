//! The generated worlds the JS-parity golden suites were captured against,
//! frozen as this port produced them before RV-1 (`LARGE_ITEM_RULINGS.md`
//! Ruling BD, 2026-09-29) replaced the river carve.
//!
//! **Why this exists.** Ruling BD changed `generate_terrain`'s carve on purpose
//! (`cartalith_hydrology::carve_channel_network`), so every generated world's
//! `field` -- and the four arrays computed from the carved field -- no longer
//! match the reference's. The golden suites downstream of the carve
//! (`cartalith-civ/tests/golden_parity_*.rs`: biome, lithology, settlement
//! placement, roads, faction aggregates and the rest) do not test the carve:
//! they test a civ port against a JS capture taken **on the reference's world**.
//! Re-recording their expected values from this port's new world would turn
//! each into a snapshot of itself -- the outcome `golden_parity_carve.rs`
//! already refuses for craters ("re-baselining them against our own output
//! would turn a parity test into a self-referential snapshot. Pinned instead").
//! So those suites keep generating their world with `generate_terrain`, and
//! then [`pin`] puts back the six arrays the carve changed, from this file's
//! capture, and they go on proving parity against the reference.
//!
//! **What was captured.** `pre_rv1_worlds.bin` was written by `generate_terrain`
//! built from commit `1750821` (the last commit before RV-1), for every
//! configuration a golden suite generates -- `WorldParams::defaults(gw, gh,
//! seed)` with `climate.w_iters = 12` and `world` set, which is every such
//! suite's own setup. Each world's other fields (plates, stress, age,
//! lithology inputs, the channel tree, stream order, the river-intensity
//! stamp) were compared by hash between that build and the RV-1 build and are
//! **identical**: the carve runs after all of them. Only these six differ.
//! [`pin`] asserts the configuration is one it holds, rather than quietly
//! leaving a new configuration unpinned.
//!
//! `golden_parity_carve.rs` checks the capture itself against the reference's
//! own carved arrays, within the tolerance that suite always used, so this
//! file is evidence of the reference world, not a second copy of an opinion.
//!
//! Layout: for each configuration in [`CONFIGS`] order, `n = gw * gh` little-
//! endian `f32`s each of `field`, `temperature`, `rainfall`, `flow_discharge`
//! and `river_floor`, then `n` bytes of `river_mask`.

#![allow(dead_code)]

use std::sync::Arc;

/// `(gw, gh, seed, world)`, in file order, each with the FNV-1a 64 hash of
/// that world's `stream_order` (little-endian `i16` bytes). Stream order is
/// computed before the carve, so it is the same in the captured build and in
/// this one, and it identifies the world [`pin`] is about to overwrite.
pub const CONFIGS: [(usize, usize, i32, bool, u64); 7] = [
    (14, 11, 24601, false, 0xc0fafb3e9b0257a4),
    (16, 12, 314159, true, 0xa7f8d6020f60c0c4),
    (48, 40, 777, false, 0xa322f122c7ab274e),
    (24, 18, 24601, false, 0xe4c3c4c8ec088cbc),
    (48, 36, 314159, true, 0xce88c9540158f814),
    (20, 16, 314159, true, 0x045296b2cd6c9ec6),
    (64, 48, 24601, false, 0x30105d96ffa1a45d),
];

fn fnv_i16(v: &[i16]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for x in v {
        for b in x.to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
    }
    h
}

const BYTES: &[u8] = include_bytes!("pre_rv1_worlds.bin");

// The file is exactly its layout: 21 bytes a cell over every configuration.
const _: () = {
    let mut total = 0;
    let mut k = 0;
    while k < CONFIGS.len() {
        total += 21 * CONFIGS[k].0 * CONFIGS[k].1;
        k += 1;
    }
    assert!(BYTES.len() == total, "pre_rv1_worlds.bin does not match CONFIGS");
};

/// The six carve-dependent arrays of one captured world.
pub struct Frozen {
    pub field: Vec<f32>,
    pub temperature: Vec<f32>,
    pub rainfall: Vec<f32>,
    pub flow_discharge: Vec<f32>,
    pub river_floor: Vec<f32>,
    pub river_mask: Vec<u8>,
}

/// The captured world for `(gw, gh, seed, world)`. Panics on a configuration
/// the capture does not hold.
pub fn frozen(gw: usize, gh: usize, seed: i32, world: bool) -> Frozen {
    let mut off = 0usize;
    for &(cw, ch, cs, cwo, _) in &CONFIGS {
        let n = cw * ch;
        if (cw, ch, cs, cwo) == (gw, gh, seed, world) {
            let f32s = |off: &mut usize| -> Vec<f32> {
                let v = BYTES[*off..*off + 4 * n]
                    .chunks_exact(4)
                    .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                    .collect();
                *off += 4 * n;
                v
            };
            let field = f32s(&mut off);
            let temperature = f32s(&mut off);
            let rainfall = f32s(&mut off);
            let flow_discharge = f32s(&mut off);
            let river_floor = f32s(&mut off);
            let river_mask = BYTES[off..off + n].to_vec();
            return Frozen { field, temperature, rainfall, flow_discharge, river_floor, river_mask };
        }
        off += 21 * n;
    }
    panic!("no pre-RV-1 world captured for {gw}x{gh} seed {seed} world={world}; add it to pre_rv1_worlds.bin");
}

/// Puts the pre-RV-1 carve's six arrays back into a world `generate_terrain`
/// just built for the same configuration. Asserts the world really is that
/// configuration's -- its grid size, and its `stream_order` hash, which the
/// carve cannot move -- so a pin can never land on the wrong world.
pub fn pin(ws: &mut cartalith_engine::WorldState, gw: usize, gh: usize, seed: i32, world: bool) {
    let f = frozen(gw, gh, seed, world);
    assert_eq!(ws.field.len(), gw * gh, "pin: world is not {gw}x{gh}");
    let order = ws.stream_order.as_ref().expect("pin: the frozen worlds all carved rivers");
    let want = CONFIGS.iter().find(|c| (c.0, c.1, c.2, c.3) == (gw, gh, seed, world)).unwrap().4;
    assert_eq!(fnv_i16(order), want, "pin: {gw}x{gh} seed {seed} is not the world that was captured");
    ws.field = Arc::new(f.field);
    ws.temperature = Arc::new(f.temperature);
    ws.rainfall = Arc::new(f.rainfall);
    ws.flow_discharge = Arc::new(f.flow_discharge);
    ws.river_floor = Some(f.river_floor);
    ws.river_mask = Some(f.river_mask);
}

