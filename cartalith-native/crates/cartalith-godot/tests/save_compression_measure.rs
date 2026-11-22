//! GX measurement harness (`SAVEFILE_COMPAT.md` §18.7, Ruling GX): which raster
//! compression format 3 should use. `#[ignore]`d: it prints, it does not gate.
//!
//! Run alone, release:
//! `CARTALITH_SAVE_SIZES=2048x1311,4096x4096 CARTALITH_SAVE_SEEDS=24601,7,424242 cargo test -p cartalith-godot --release --test save_compression_measure -- --ignored --nocapture`
//!
//! It zips the six core raster payloads of a generated world (the entries that
//! hold essentially all of an archive's bytes, §18.1) under each candidate,
//! reads them back, and asserts every f32 bit pattern identical, so a codec
//! that loses a bit cannot print a number. Protects: only that exactness.
#![allow(dead_code)]

#[path = "../src/params.rs"]
mod params;

use cartalith_engine::generate_terrain;
use std::io::{Cursor, Read, Write};

/// One candidate: zip method, level, and whether to delta before the shuffle.
#[derive(Clone, Copy)]
struct Cand {
    name: &'static str,
    zstd: bool,
    level: i64,
    delta: bool,
    shuffle: bool,
}

const CANDS: &[Cand] = &[
    Cand { name: "deflate, no shuffle (format 1)", zstd: false, level: 6, delta: false, shuffle: false },
    Cand { name: "shuffle+deflate (format 2)", zstd: false, level: 6, delta: false, shuffle: true },
    Cand { name: "shuffle+zstd-3", zstd: true, level: 3, delta: false, shuffle: true },
    Cand { name: "shuffle+zstd-9", zstd: true, level: 9, delta: false, shuffle: true },
    Cand { name: "shuffle+zstd-15", zstd: true, level: 15, delta: false, shuffle: true },
    Cand { name: "delta+shuffle+zstd-3", zstd: true, level: 3, delta: true, shuffle: true },
    Cand { name: "delta+shuffle+zstd-9", zstd: true, level: 9, delta: true, shuffle: true },
    Cand { name: "delta+shuffle+zstd-15", zstd: true, level: 15, delta: true, shuffle: true },
];

/// Integer difference of the f32 bit patterns along each row (row start kept raw), wrapping: exact and invertible.
fn delta_rows(bits: &[u32], gw: usize) -> Vec<u32> {
    let mut out = bits.to_vec();
    for row in out.chunks_mut(gw) {
        for x in (1..row.len()).rev() {
            row[x] = row[x].wrapping_sub(row[x - 1]);
        }
    }
    out
}
fn undelta_rows(d: &mut [u32], gw: usize) {
    for row in d.chunks_mut(gw) {
        for x in 1..row.len() {
            row[x] = row[x].wrapping_add(row[x - 1]);
        }
    }
}
fn shuffle(bits: &[u32]) -> Vec<u8> {
    let n = bits.len();
    let mut o = vec![0u8; n * 4];
    for (i, b) in bits.iter().enumerate() {
        let le = b.to_le_bytes();
        for k in 0..4 {
            o[k * n + i] = le[k];
        }
    }
    o
}
fn unshuffle(bytes: &[u8]) -> Vec<u32> {
    let n = bytes.len() / 4;
    (0..n).map(|i| u32::from_le_bytes([bytes[i], bytes[n + i], bytes[2 * n + i], bytes[3 * n + i]])).collect()
}

/// Median of three runs of `f`, in milliseconds.
fn med3(mut f: impl FnMut()) -> f64 {
    let mut t: Vec<f64> = (0..3)
        .map(|_| {
            let s = std::time::Instant::now();
            f();
            s.elapsed().as_secs_f64() * 1000.0
        })
        .collect();
    t.sort_by(|a, b| a.total_cmp(b));
    t[1]
}

fn encode(c: Cand, fields: &[&[f32]], strahler: &[u8], gw: usize) -> Vec<u8> {
    let mut buf = Vec::new();
    let method = if c.zstd { zip::CompressionMethod::Zstd } else { zip::CompressionMethod::Deflated };
    let opts = zip::write::SimpleFileOptions::default().compression_method(method).compression_level(Some(c.level));
    let mut w = zip::ZipWriter::new(Cursor::new(&mut buf));
    for (i, f) in fields.iter().enumerate() {
        let bits: Vec<u32> = f.iter().map(|v| v.to_bits()).collect();
        let bits = if c.delta { delta_rows(&bits, gw) } else { bits };
        w.start_file(format!("f{i}"), opts).unwrap();
        if c.shuffle {
            w.write_all(&shuffle(&bits)).unwrap();
        } else {
            w.write_all(&bits.iter().flat_map(|b| b.to_le_bytes()).collect::<Vec<u8>>()).unwrap();
        }
    }
    w.start_file("u8", opts).unwrap();
    w.write_all(strahler).unwrap();
    w.finish().unwrap();
    buf
}

fn decode(c: Cand, zipped: &[u8], n_fields: usize, gw: usize) -> Vec<Vec<u32>> {
    let mut a = zip::ZipArchive::new(Cursor::new(zipped)).unwrap();
    let mut out = Vec::new();
    for i in 0..n_fields {
        let mut bytes = Vec::new();
        a.by_name(&format!("f{i}")).unwrap().read_to_end(&mut bytes).unwrap();
        let mut bits = if c.shuffle {
            unshuffle(&bytes)
        } else {
            bytes.chunks_exact(4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect()
        };
        if c.delta {
            undelta_rows(&mut bits, gw);
        }
        out.push(bits);
    }
    let mut sink = Vec::new();
    a.by_name("u8").unwrap().read_to_end(&mut sink).unwrap();
    out
}

#[test]
#[ignore]
fn measure_raster_codecs() {
    let sizes = std::env::var("CARTALITH_SAVE_SIZES").unwrap_or_else(|_| "2048x1311".into());
    let seeds = std::env::var("CARTALITH_SAVE_SEEDS").unwrap_or_else(|_| "24601".into());
    let mib = |b: usize| b as f64 / (1024.0 * 1024.0);
    for size in sizes.split(',') {
        let (w, h) = size.split_once('x').unwrap();
        for seed in seeds.split(',') {
            let mut p = params::defaults();
            p.gw = w.trim().parse().unwrap();
            p.gh = h.trim().parse().unwrap();
            p.tect.seed = seed.trim().parse().unwrap();
            let ws = generate_terrain(&p);
            let n = p.gw * p.gh;
            let vf = ws.volcanic_field.as_ref().clone();
            let fields: [&[f32]; 5] = [&ws.field, &ws.temperature, &ws.rainfall, &vf, &ws.impact_field];
            let strahler: Vec<u8> = match ws.stream_order.as_ref() {
                Some(o) => o.iter().map(|&v| v.clamp(0, 255) as u8).collect(),
                None => vec![0; n],
            };
            println!("== {}x{} seed {}", p.gw, p.gh, seed);
            for c in CANDS {
                let mut z = Vec::new();
                let wms = med3(|| z = encode(*c, &fields, &strahler, p.gw));
                let mut back = Vec::new();
                let rms = med3(|| back = decode(*c, &z, 5, p.gw));
                for (f, b) in fields.iter().zip(&back) {
                    assert!(f.iter().zip(b).all(|(a, b)| a.to_bits() == *b), "{} lost a bit", c.name);
                }
                println!("ROW|{}x{}|{}|{}|{:.2}|{:.0}|{:.0}", p.gw, p.gh, seed, c.name, mib(z.len()), wms, rms);
            }
        }
    }
}
