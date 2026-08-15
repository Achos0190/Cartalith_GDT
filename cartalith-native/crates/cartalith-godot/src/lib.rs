//! Boundary layer between Godot and the engine crates (ARCHITECTURE.md).
//!
//! `WalkingSkeleton` is the Phase 0 proof that a gdext-backed class loads in
//! the Godot editor and survives a Windows/Android export. `WorldGen`
//! (below) is Phase 1's real API surface (`ARCHITECTURE.md`: "a `WorldGen`
//! with `generate(seed, width_km, resolution)` and accessors returning
//! fields") — the only place in this crate (and the only crate in the
//! workspace) that touches `cartalith_engine::WorldState` and a Godot type
//! in the same function, exactly the boundary `ARCHITECTURE.md` describes:
//! "Rust never touches the scene tree... only `cartalith-godot` may depend
//! on `gdext`."

use cartalith_engine::{generate_terrain, WorldParams, WorldStructureParams};
use godot::classes::image::Format;
use godot::classes::{IRefCounted, INode, Image, ImageTexture, Node, RefCounted};
use godot::init::{ExtensionLibrary, gdextension};
use godot::prelude::*;

struct CartalithExtension;

#[gdextension]
unsafe impl ExtensionLibrary for CartalithExtension {}

/// Placeholder GDExtension class for the Phase 0 walking skeleton.
#[derive(GodotClass)]
#[class(base=Node)]
struct WalkingSkeleton {
    base: Base<Node>,
}

#[godot_api]
impl INode for WalkingSkeleton {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }

    fn ready(&mut self) {
        godot_print!("cartalith-godot: WalkingSkeleton ready (Phase 0)");
    }
}

#[godot_api]
impl WalkingSkeleton {
    /// Round-trips a value through Rust so GDScript can confirm the
    /// extension is actually loaded, not just present on disk.
    #[func]
    fn ping(&self) -> GString {
        GString::from("cartalith-godot: pong")
    }
}

/// Either a fresh `generate_terrain()` run or a loaded save
/// (`cartalith_io::load_save`, `MVP_SCOPE.md` point 12/criterion 7). A
/// loaded save only carries the terrain fields `SAVEFILE_COMPAT.md`
/// documents (no plate/stress/flexure substrate — those aren't part of the
/// save format), so this is a separate variant rather than trying to
/// backfill a full `WorldState`; `build_color_texture` reads through
/// `WorldGen`'s own small accessor methods below so it doesn't need to
/// know which source is active.
enum WorldSource {
    Generated(Box<cartalith_engine::WorldState>),
    Loaded(Box<cartalith_io::SaveData>),
}

/// `MVP_SCOPE.md` points 10-11: basic 2D rendering + minimal UI. Owns the
/// last `generate_terrain()` result (or loaded save); GDScript drives it via
/// `generate()`/`load_save()` then `build_color_texture()`. Square grid
/// (`gw == gh`) for MVP **generation** — a loaded save keeps whatever
/// `GW`/`GH` it was exported at, which need not be square (the reference
/// HTML's own `resW`/aspect-from-image handling is UI-layer scope this port
/// hasn't built yet, but a save's own stored resolution isn't that).
#[derive(GodotClass)]
#[class(base=RefCounted)]
struct WorldGen {
    base: Base<RefCounted>,
    source: Option<WorldSource>,
    gw: i32,
    gh: i32,
    sea_level: f64,
    /// Set via `set_experimental_flags`, applied by both `generate()` and
    /// `generate_world_structure()`. All four are now golden-verified
    /// (see each field's own doc comment in `cartalith-engine`/
    /// `cartalith-climate` -- `cartalith-native/docs/CHANGELOG.md` has the
    /// full extraction history). `dynamic_lithology` defaults `false`
    /// because that's JS's own real default; `volc_provinces`/
    /// `terrain_wind_deflection`/`ocean_currents` default `true` because
    /// JS's real defaults are `true` (unconditional, in wind deflection's
    /// case) -- this `WorldGen` wrapper's own defaults can match JS
    /// exactly regardless of what `cartalith_engine::WorldParams::defaults`
    /// itself defaults to, since every call site here overrides all four
    /// explicitly. Still exposed as toggles, not hardcoded: useful for
    /// comparing against the real HTML app with one turned off at a time.
    dynamic_lithology: bool,
    volc_provinces: bool,
    terrain_wind_deflection: bool,
    ocean_currents: bool,
}

#[godot_api]
impl IRefCounted for WorldGen {
    fn init(base: Base<RefCounted>) -> Self {
        Self {
            base,
            source: None,
            gw: 0,
            gh: 0,
            sea_level: 0.42,
            dynamic_lithology: false,
            volc_provinces: true,
            terrain_wind_deflection: true,
            ocean_currents: true,
        }
    }
}

#[godot_api]
impl WorldGen {
    /// Sets the four golden-verified subsystem flags this instance's
    /// `generate()`/`generate_world_structure()` calls apply from then on
    /// — see the `WorldGen` struct's own doc comment on the fields this
    /// writes.
    #[func]
    fn set_experimental_flags(
        &mut self,
        dynamic_lithology: bool,
        volc_provinces: bool,
        terrain_wind_deflection: bool,
        ocean_currents: bool,
    ) {
        self.dynamic_lithology = dynamic_lithology;
        self.volc_provinces = volc_provinces;
        self.terrain_wind_deflection = terrain_wind_deflection;
        self.ocean_currents = ocean_currents;
    }

    /// Runs the full ported pipeline (`cartalith_engine::generate_terrain`)
    /// at the given seed/real-km map width/grid resolution. `resolution`
    /// is clamped to a sane minimum (4) — a 0 or negative value from an
    /// unset GDScript `SpinBox` should not panic the extension.
    #[func]
    fn generate(&mut self, seed: i32, width_km: f64, resolution: i32) {
        let gw = resolution.max(4) as usize;
        let gh = gw;
        let mut p = WorldParams::defaults(gw, gh, seed);
        p.map_width_km = if width_km > 0.0 { width_km } else { 800.0 };
        p.tect.dynamic_lithology = self.dynamic_lithology;
        p.volc.provinces = self.volc_provinces;
        p.climate.terrain_wind_deflection = self.terrain_wind_deflection;
        p.climate.currents = self.ocean_currents;
        let ws = generate_terrain(&p);
        // Not p.sea_level -- World-Structure archetypes re-anchor it;
        // WorldState carries the value actually used.
        self.sea_level = ws.sea_level;
        self.gw = gw as i32;
        self.gh = gh as i32;
        self.source = Some(WorldSource::Generated(Box::new(ws)));
    }

    /// Named World-Structure archetype presets (reference HTML
    /// `ARCHETYPES`, lines 2521-2526) as
    /// `(continentality, fragmentation, tectonic_energy, ocean_depth,
    /// hotspot_density)`. `cartalith_engine::WorldParams::world_structure`
    /// itself takes raw knobs only, not named presets (its own doc
    /// comment: "a caller wanting 'Archipelago' passes that preset's own
    /// numbers") -- so the name -> knobs lookup lives here, in the
    /// boundary layer, rather than in GDScript
    /// (`ARCHITECTURE.md`: "Godot computes nothing beyond layout").
    #[func]
    fn generate_world_structure(&mut self, seed: i32, width_km: f64, resolution: i32, archetype: GString) -> bool {
        let (continentality, fragmentation, tectonic_energy, ocean_depth, hotspot_density) =
            match archetype.to_string().to_lowercase().as_str() {
                "earth" => (0.30, 0.50, 0.60, 0.60, 0.20),
                "supercontinent" => (0.60, 0.10, 0.50, 0.70, 0.10),
                "archipelago" => (0.15, 0.90, 0.80, 0.30, 0.50),
                "volcanic" => (0.05, 1.00, 0.90, 0.80, 1.00),
                "rift" => (0.40, 0.35, 0.75, 0.55, 0.30),
                other => {
                    godot_print!("cartalith-godot: unknown World-Structure archetype '{other}'");
                    return false;
                }
            };

        let gw = resolution.max(4) as usize;
        let gh = gw;
        let mut p = WorldParams::defaults(gw, gh, seed);
        p.map_width_km = if width_km > 0.0 { width_km } else { 800.0 };
        p.world_structure =
            WorldStructureParams { enabled: true, continentality, fragmentation, tectonic_energy, ocean_depth, hotspot_density };
        p.tect.dynamic_lithology = self.dynamic_lithology;
        p.volc.provinces = self.volc_provinces;
        p.climate.terrain_wind_deflection = self.terrain_wind_deflection;
        p.climate.currents = self.ocean_currents;

        let ws = generate_terrain(&p);
        self.sea_level = ws.sea_level;
        self.gw = gw as i32;
        self.gh = gh as i32;
        self.source = Some(WorldSource::Generated(Box::new(ws)));
        true
    }

    /// `MVP_SCOPE.md` point 12 / criterion 7: opens a real HTML-app `.zip`
    /// and renders that save's terrain. `path` is a native OS filesystem
    /// path (e.g. from a GDScript `FileDialog` in native/desktop mode) --
    /// `cartalith_io::load_save` only needs `Read + Seek`, so a plain
    /// `std::fs::File` satisfies it without any Godot `FileAccess`
    /// involvement. Returns `false` on any read/parse error and leaves the
    /// previous `source` untouched, matching `generate()`'s own
    /// fail-quietly-check-the-console shape (`main.gd`'s doc comment).
    #[func]
    fn load_save(&mut self, path: GString) -> bool {
        let file = match std::fs::File::open(path.to_string()) {
            Ok(f) => f,
            Err(e) => {
                godot_print!("cartalith-godot: load_save open failed: {e}");
                return false;
            }
        };
        let save = match cartalith_io::load_save(std::io::BufReader::new(file)) {
            Ok(s) => s,
            Err(e) => {
                godot_print!("cartalith-godot: load_save failed: {e}");
                return false;
            }
        };
        self.gw = save.params.gw as i32;
        self.gh = save.params.gh as i32;
        self.sea_level = save.params.sea_level;
        self.source = Some(WorldSource::Loaded(Box::new(save)));
        true
    }

    #[func]
    fn get_width(&self) -> i32 {
        self.gw
    }

    #[func]
    fn get_height(&self) -> i32 {
        self.gh
    }

    /// Builds a colour + hillshade texture from the last `generate()`
    /// result — `MVP_SCOPE.md` point 10: "colour and hillshade, enough to
    /// verify what was generated," explicitly **not** the JS engine's full
    /// renderer (no multi-octave grain, NPR styles, splat textures, or LOD
    /// pyramid — none of that is MVP scope, so this doesn't reach for
    /// pixel-for-pixel colour parity with the reference HTML's own
    /// renderer either). Land/water split at `sea_level`, a three-stop
    /// hypsometric ramp above it, a simple analytic hillshade from the
    /// height gradient, and a blue tint on channelized cells when
    /// `carve_rivers` produced topology — "land and water distinct, biome
    /// colouring plausible, rivers visible" (`MVP_SCOPE.md`'s own "done"
    /// checklist, point 2). Returns `None` before the first `generate()`
    /// call.
    #[func]
    fn build_color_texture(&self) -> Option<Gd<ImageTexture>> {
        let (field, chan_mask): (&[f32], Option<&[u8]>) = match self.source.as_ref()? {
            WorldSource::Generated(ws) => (&ws.field, ws.channels.as_ref().map(|c| c.chan.as_slice())),
            WorldSource::Loaded(save) => (&save.fields.heightmap, Some(save.fields.strahler_order.as_slice())),
        };
        let gw = self.gw as usize;
        let gh = self.gh as usize;
        let sea = self.sea_level;

        let mut bytes = Vec::with_capacity(gw * gh * 3);
        for y in 0..gh {
            for x in 0..gw {
                let i = y * gw + x;
                let h = field[i] as f64;
                let (mut r, mut g, mut b) = color_for_height(h, sea);

                let xl = if x > 0 { x - 1 } else { 0 };
                let xr = if x + 1 < gw { x + 1 } else { gw - 1 };
                let yu = if y > 0 { y - 1 } else { 0 };
                let yd = if y + 1 < gh { y + 1 } else { gh - 1 };
                let gx = (field[y * gw + xr] as f64 - field[y * gw + xl] as f64) * 0.5;
                let gy = (field[yd * gw + x] as f64 - field[yu * gw + x] as f64) * 0.5;
                let shade = hillshade(gx, gy);
                r *= shade;
                g *= shade;
                b *= shade;

                if let Some(mask) = chan_mask
                    && mask[i] != 0
                {
                    r *= 0.5;
                    g = (g * 0.5 + 0.3).min(1.0);
                    b = (b * 0.5 + 0.45).min(1.0);
                }

                bytes.push((r.clamp(0.0, 1.0) * 255.0) as u8);
                bytes.push((g.clamp(0.0, 1.0) * 255.0) as u8);
                bytes.push((b.clamp(0.0, 1.0) * 255.0) as u8);
            }
        }

        let packed = PackedByteArray::from(bytes);
        let image = Image::create_from_data(gw as i32, gh as i32, false, Format::RGB8, &packed)?;
        ImageTexture::create_from_image(&image)
    }
}

/// Bathymetric/hypsometric tint at `[0,1]` height `h` relative to
/// `sea_level` — a simplified stand-in for the reference HTML's own biome
/// colouring (deliberately not attempted here, see `build_color_texture`'s
/// doc comment).
fn color_for_height(h: f64, sea_level: f64) -> (f64, f64, f64) {
    if h < sea_level {
        let depth = ((sea_level - h) / sea_level.max(1e-6)).clamp(0.0, 1.0);
        lerp3((0.55, 0.75, 0.85), (0.02, 0.08, 0.25), depth)
    } else {
        let t = ((h - sea_level) / (1.0 - sea_level).max(1e-6)).clamp(0.0, 1.0);
        if t < 0.3 {
            lerp3((0.22, 0.42, 0.14), (0.55, 0.47, 0.28), t / 0.3)
        } else if t < 0.7 {
            lerp3((0.55, 0.47, 0.28), (0.5, 0.48, 0.46), (t - 0.3) / 0.4)
        } else {
            lerp3((0.5, 0.48, 0.46), (0.97, 0.97, 0.98), (t - 0.7) / 0.3)
        }
    }
}

fn lerp3(a: (f64, f64, f64), b: (f64, f64, f64), t: f64) -> (f64, f64, f64) {
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t, a.2 + (b.2 - a.2) * t)
}

/// A simple analytic hillshade from the height gradient `(gx, gy)` — a
/// synthetic normal (`z` fixed, not derived from real map-width-aware
/// relief) lit from the upper-left, returned as a multiplicative factor
/// clamped away from pure black/blown-out white so shaded terrain stays
/// readable at any slope.
fn hillshade(gx: f64, gy: f64) -> f64 {
    let (nx, ny, nz) = (-gx, -gy, 0.15);
    let nlen = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-6);
    let (nx, ny, nz) = (nx / nlen, ny / nlen, nz / nlen);
    let (lx, ly, lz): (f64, f64, f64) = (0.5, 0.5, 0.7);
    let llen = (lx * lx + ly * ly + lz * lz).sqrt();
    let (lx, ly, lz) = (lx / llen, ly / llen, lz / llen);
    let dot = nx * lx + ny * ly + nz * lz;
    (0.55 + 0.55 * dot).clamp(0.35, 1.3)
}
