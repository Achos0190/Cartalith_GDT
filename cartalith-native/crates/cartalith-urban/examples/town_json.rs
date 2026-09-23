//! Write one `generate()` town as JSON, in the key shape
//! `urban_layout_draw.gd`'s `draw_layout` reads, so a windowed probe can draw
//! a town the shipping app cannot yet ask for.
//!
//! Exists for the Venus/radial plan: `cartalith-civ`'s urban adapter never
//! passes a `culture` (`urban_adapter.rs`' header — this port has no
//! faction-culture table), so `WorldGen::urban_layouts` only ever produces
//! medieval towns, and `_radialwall_probe.gd` needs a real radial one drawn by
//! the real renderer.
//!
//! ```text
//! cargo run -p cartalith-urban --example town_json -- <seed> <culture> <pop> <site|-> <out.json>
//! ```
//!
//! A subset of `cartalith-godot`'s `layout_dict`: the street graph, blocks,
//! lots, buildings, the wall and the water. Details and farmland are left out.

use cartalith_urban::generate::{GenOpts, generate};
use cartalith_urban::geom::Vec2;
use std::fmt::Write as _;

fn pts(v: &[Vec2]) -> String {
    let inner: Vec<String> = v.iter().map(|p| format!("[{},{}]", p.x, p.y)).collect();
    format!("[{}]", inner.join(","))
}

fn strs<'a>(v: impl Iterator<Item = &'a str>) -> String {
    let inner: Vec<String> = v.map(|s| format!("\"{s}\"")).collect();
    format!("[{}]", inner.join(","))
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() != 6 {
        eprintln!("usage: town_json <seed> <culture> <pop> <site|-> <out.json>");
        std::process::exit(2);
    }
    let seed: u32 = a[1].parse().expect("seed: u32");
    let pop: f64 = a[3].parse().expect("pop: f64");
    let opts = GenOpts {
        culture: Some(a[2].clone()),
        pop: Some(pop),
        site: (a[4] != "-").then(|| a[4].clone()),
        ..GenOpts::default()
    };
    let t = generate(seed, &opts);

    let mut s = String::from("{");
    let _ = write!(s, "\"culture\":\"{}\",\"wm\":{},\"hm\":{},", t.culture, t.wm, t.hm);
    let _ = write!(s, "\"market\":[{},{}],", t.anchors.market.x, t.anchors.market.y);
    s.push_str("\"streets\":{");
    let classes: Vec<String> = ["lane", "street", "quay", "ringroad", "primary"]
        .iter()
        .filter_map(|cls| {
            let v: Vec<Vec2> = t
                .graph
                .edges
                .iter()
                .filter(|e| e.cls == *cls)
                .flat_map(|e| [t.graph.nodes[e.a].pt(), t.graph.nodes[e.b].pt()])
                .collect();
            (!v.is_empty()).then(|| format!("\"{cls}\":{}", pts(&v)))
        })
        .collect();
    s.push_str(&classes.join(","));
    s.push_str("},");
    let blocks: Vec<String> = t.blocks.iter().map(|b| pts(&b.poly)).collect();
    let _ = write!(s, "\"blocks\":[{}],", blocks.join(","));
    let bp: Vec<&str> = t.blocks.iter().map(|b| if b.plaza { "1" } else { "0" }).collect();
    let _ = write!(s, "\"block_plaza\":[{}],", bp.join(","));
    if let Some(p) = &t.plaza {
        let _ = write!(s, "\"plaza\":{},", pts(&p.poly));
    }
    let parcels: Vec<String> = t.parcels.iter().map(|p| pts(&p.par.poly)).collect();
    let _ = write!(s, "\"parcels\":[{}],", parcels.join(","));
    let _ = write!(s, "\"parcel_district\":{},", strs(t.parcels.iter().map(|p| p.district)));
    let tones: Vec<String> = t.parcels.iter().map(|p| p.par.tone.to_string()).collect();
    let _ = write!(s, "\"parcel_tone\":[{}],", tones.join(","));
    let bld: Vec<String> = t.buildings.iter().map(|b| pts(&b.poly)).collect();
    let _ = write!(s, "\"buildings\":[{}],", bld.join(","));
    let ridge: Vec<Vec2> = t.buildings.iter().flat_map(|b| b.ridge).collect();
    let _ = write!(s, "\"building_ridge\":{},", pts(&ridge));
    let _ = write!(s, "\"building_district\":{},", strs(t.buildings.iter().map(|b| b.district)));
    let tone_of: std::collections::HashMap<&str, f64> =
        t.parcels.iter().map(|p| (p.par.id.as_str(), p.par.tone)).collect();
    let bt: Vec<String> = t
        .buildings
        .iter()
        .map(|b| tone_of.get(b.parcel.as_str()).copied().unwrap_or(0.5).to_string())
        .collect();
    let _ = write!(s, "\"building_tone\":[{}],", bt.join(","));
    if let Some(ring) = &t.wall.ring {
        let _ = write!(s, "\"wall_ring\":{},\"wall_style\":\"{}\",", pts(ring), t.wall.style);
        let land: Vec<Vec2> = t.wall.gates.iter().filter(|g| !g.water).map(|g| g.pt).collect();
        let _ = write!(s, "\"wall_gates\":{},", pts(&land));
        if let Some(c) = t.wall.centroid {
            let _ = write!(s, "\"wall_centroid\":[{},{}],", c.x, c.y);
        }
    }
    let _ = write!(s, "\"river\":{},\"river_w\":{},", pts(&t.site.river), t.site.river_w);
    let _ = write!(s, "\"water_poly\":{}", pts(&t.site.water_poly));
    s.push('}');
    std::fs::write(&a[5], s).expect("write");
}
