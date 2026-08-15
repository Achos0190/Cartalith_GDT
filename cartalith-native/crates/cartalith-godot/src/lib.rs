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

mod render;
use render::RenderCtx;

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
    /// `latAt`'s inputs (`render.rs`) — `p.world`/`p.climate.lat_n`/`.lat_s`
    /// for a fresh `generate()`, or `save.params.world` + JS's own literal
    /// `climate` defaults (55/5) for a loaded save, whose format doesn't
    /// store latitude band at all (`SAVEFILE_COMPAT.md`).
    world: bool,
    lat_n: f64,
    lat_s: f64,
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
            world: false,
            lat_n: 55.0,
            lat_s: 5.0,
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
        self.world = p.world;
        self.lat_n = p.climate.lat_n;
        self.lat_s = p.climate.lat_s;
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
        self.world = p.world;
        self.lat_n = p.climate.lat_n;
        self.lat_s = p.climate.lat_s;
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
        self.world = save.params.world;
        // SaveParams carries no latitude band -- JS's own literal
        // `climate` defaults (reference HTML line 2287), same fallback
        // WorldParams::defaults uses.
        self.lat_n = 55.0;
        self.lat_s = 5.0;
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
    /// result. Ported from the reference HTML's own default-settings
    /// renderer (`render.rs`'s doc comment lists exactly what's ported vs.
    /// deliberately excluded) — no longer the MVP placeholder tint this
    /// method used before. A blue tint on channelized cells stands in for
    /// the reference's vector river overlay (`drawRiverWays`, not wired
    /// into this port), keeping "rivers visible" (`MVP_SCOPE.md`'s "done"
    /// checklist, point 2) satisfied. Returns `None` before the first
    /// `generate()` call.
    #[func]
    fn build_color_texture(&self) -> Option<Gd<ImageTexture>> {
        let (field, temperature, rainfall, flow, chan_mask) = match self.source.as_ref()? {
                WorldSource::Generated(ws) => (
                    &ws.field,
                    &ws.temperature,
                    &ws.rainfall,
                    Some(ws.flow_discharge.as_slice()),
                    ws.channels.as_ref().map(|c| c.chan.as_slice()),
                ),
                WorldSource::Loaded(save) => (
                    &save.fields.heightmap,
                    &save.fields.temperature,
                    &save.fields.rainfall,
                    None,
                    Some(save.fields.strahler_order.as_slice()),
                ),
            };
        let gw = self.gw as usize;
        let gh = self.gh as usize;
        let ctx = RenderCtx::new(field, temperature, rainfall, flow, gw, gh, self.sea_level, self.world, self.lat_n, self.lat_s);

        let mut bytes = Vec::with_capacity(gw * gh * 3);
        for y in 0..gh {
            for x in 0..gw {
                let i = y * gw + x;
                let (mut r, mut g, mut b) = render::cell_color(&ctx, x, y);

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
