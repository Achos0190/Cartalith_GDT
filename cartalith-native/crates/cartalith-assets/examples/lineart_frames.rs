//! AP-10's frame writer: runs `cartalith_assets::lineart` on every
//! `src_<name>.png` in a folder and writes what the acceptance measurement
//! (`cartalith-native/docs/evidence/lineart-ap10-2026-10-19/ap10_measure.py`)
//! judges, so the numbers in `ASSET_PACK_TOOLS_PROPOSAL.md` §10 come from the
//! Rust pipeline itself and not from the Python lab it was ported from.
//!
//! Usage: `cargo run --release -p cartalith-assets --example lineart_frames -- <src_dir> <out_dir>`
//!
//! Writes per sample: `<name>_a<H>.png` (L8 coverage at H = 40, 160, 640 and
//! 2560: the research's ×1, ×4, ×16 and ×64 of a 2048-grid icon), the
//! 256² variant bake `<name>_bake256.png`, every SDF level
//! `<name>_sdf<R>.png` (L8, for bar (g)'s size), and one JSON line per
//! sample in `stats.jsonl`. It never writes outside `<out_dir>` and refuses
//! an unknown argument count rather than guessing (`MISTAKES.md`, "Write a
//! probe's usage header").

use cartalith_assets::lineart::{GapClosing, LineArtOptions, SdfPyramid, clean_ink, convert_line_art, render_coverage, working_image};
use cartalith_assets::{ItemTransform, decode_png, encode_png};
use std::path::Path;
use std::time::Instant;

/// The output heights the research judged at (×1, ×4, ×16, ×64).
const SCALES: [u32; 4] = [40, 160, 640, 2560];

/// Encode one channel of coverage as an L8 PNG.
fn write_l8(path: &Path, side: u32, a: &[u8]) {
    let img = image::GrayImage::from_raw(side, side, a.to_vec()).expect("side*side bytes");
    img.save(path).expect("write L8 png");
}

/// Entry point; see the module doc for the arguments.
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: lineart_frames <src_dir> <out_dir>");
        std::process::exit(2);
    }
    let (src, out) = (Path::new(&args[1]), Path::new(&args[2]));
    std::fs::create_dir_all(out).expect("create out dir");
    let mut names: Vec<String> = std::fs::read_dir(src)
        .expect("read src dir")
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|n| n.starts_with("src_") && n.ends_with(".png"))
        .collect();
    names.sort();
    if names.is_empty() {
        eprintln!("no src_*.png in {}", src.display());
        std::process::exit(3);
    }
    let mut stats = String::new();
    for file in names {
        let name = file.trim_start_matches("src_").trim_end_matches(".png").to_string();
        let img = decode_png(&std::fs::read(src.join(&file)).expect("read src")).expect("decode src");
        let t0 = Instant::now();
        let work = working_image(&img, &ItemTransform::default(), 256);
        let (mask, clean) = clean_ink(&work, GapClosing::Auto);
        let t_clean = t0.elapsed().as_secs_f64() * 1000.0;
        let t1 = Instant::now();
        let pyr = SdfPyramid::build(&mask);
        let t_sdf = t1.elapsed().as_secs_f64() * 1000.0;
        for h in SCALES {
            let a: Vec<u8> = render_coverage(&pyr, h, 1.0).iter().map(|v| (f64::from(*v) * 255.0).round() as u8).collect();
            write_l8(&out.join(format!("{name}_a{h}.png")), h, &a);
        }
        let mut sdf_bytes = Vec::new();
        for l in &pyr.levels {
            let p = out.join(format!("{name}_sdf{}.png", l.size));
            write_l8(&p, l.size, &l.q);
            sdf_bytes.push(format!("\"{}\":{}", l.size, std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0)));
        }
        let t2 = Instant::now();
        let r = convert_line_art(&work, &LineArtOptions::default(), 256).expect("convert");
        let t_bake = t2.elapsed().as_secs_f64() * 1000.0;
        std::fs::write(out.join(format!("{name}_bake256.png")), encode_png(&r.image).expect("encode")).expect("write bake");
        stats.push_str(&format!(
            "{{\"name\":\"{name}\",\"work\":{},\"threshold\":{:.4},\"illumination\":{:.4},\"grain\":{:.4},\"closed\":{},\"specks\":{},\"pinholes\":{},\"components\":{},\"stroke_p2_mask_px\":{:.3},\"sdf_top\":{},\"sdf_png_bytes\":{{{}}},\"floor_px_256\":{:.3},\"ms_clean\":{:.0},\"ms_sdf\":{:.0},\"ms_convert\":{:.0}}}\n",
            work.w,
            clean.threshold,
            clean.evidence.illumination,
            clean.evidence.grain,
            clean.closed,
            clean.specks_removed,
            clean.pinholes_filled,
            clean.components,
            pyr.stroke_p2,
            pyr.levels[0].size,
            sdf_bytes.join(","),
            r.floor_px,
            t_clean,
            t_sdf,
            t_bake
        ));
    }
    std::fs::write(out.join("stats.jsonl"), stats).expect("write stats");
}
