//! The Cartalith **project archive** — reader and writer for the tree
//! layout `SAVEFILE_COMPAT.md` specifies (owner decision 2026-08-25,
//! `DECISIONS.md` §7h).
//!
//! `SAVEFILE_COMPAT.md` is the authority, not this file: it is a normative
//! specification written for a second implementation in JavaScript, and
//! anything here that disagrees with it is a bug here. What this module
//! comments on is the reasoning that is *about the Rust*, which the
//! specification deliberately excludes.
//!
//! ## What this module owns, and what it refuses to own
//!
//! It owns the container, the tree's **slot registry**, the raster
//! encoding, the layout test, and §14's number handling. It owns **no
//! schema at all** for the documents under `entities/`, `annotations/`,
//! `history/`, `library/` or the root-level singles.
//!
//! That is deliberate and it is the same boundary [`crate::save`] already
//! draws for `params.json`: `cartalith-io` sits *below* every crate that
//! models a settlement, a label or a link, so it cannot name their types
//! without inverting the dependency graph. A document reaches the archive
//! as **JSON text against a registered slot name**, and each owning crate
//! keeps its own shape.
//!
//! The registry is the thing that makes "one concept, one home" a property
//! of the code. [`write_project`] refuses a document whose slot is not in
//! [`DOCUMENT_SLOTS`], so a new payload is one line here plus a section in
//! the specification — not a new top-level entry name invented at a call
//! site, which is exactly how the owner's `atlas`-versus-`cartography`
//! example happens.
//!
//! ## Two guards, both about silent wrongness
//!
//! 1. **Every raster's length is checked against `GW*GH` on the way in and
//!    on the way out** -- before any §8.2 un-shuffle, which reorders bytes
//!    and never changes their count. A raster entry carries no length of its own, so a
//!    short one is not a parse error; it is a truncated world. The same
//!    check covers the optional stored LOD tiles against their own
//!    `tile_w * tile_h * 3` (RGB, [`LodTiles`]), for the same reason.
//! 2. **Integral floats are coerced to integers before any schema sees a
//!    document** ([`coerce_integral_floats`]). `SAVEFILE_COMPAT.md` §14.2
//!    is the rule; `GUI_GAP_REGISTER.md` KV-04 is what forgetting it cost —
//!    every knowledge link a user ever made, discarded on each launch, for
//!    the shipped lifetime of the feature, because one integer came back as
//!    `1.0`. Doing this per-field would mean remembering it per-field.

use crate::{LoadError, SaveData, SaveError, SaveFields, SaveParams};
use std::collections::BTreeMap;
use std::io::{Read, Seek, Write};
use std::sync::Arc;

/// The entry whose **presence** selects the tree layout
/// (`SAVEFILE_COMPAT.md` §4). One central-directory lookup, no heuristics.
pub const PROJECT_MANIFEST: &str = "project.json";

/// `project.json`'s `format` member. A file whose `format` is present and
/// different is not a Cartalith project and is refused rather than guessed.
pub const PROJECT_FORMAT: &str = "cartalith-project";

/// `project.json`'s `format_version`. `2` since 2026-09-23 (owner Ruling AJ):
/// version 2 is version 1 plus `SAVEFILE_COMPAT.md` §8.2's byte-plane
/// shuffle. Both are read; only 2 is written.
///
/// The version is the *declaration*; it is not what decides whether a raster
/// is un-shuffled. That is [`SHUFFLED_INFIX`] in the entry name, so a
/// version-1 archive's bytes cannot be un-shuffled by mistake whatever its
/// manifest says.
pub const PROJECT_FORMAT_VERSION: i64 = 2;

/// Inserted before a 4-byte raster's extension when its bytes are stored
/// byte-plane shuffled (`SAVEFILE_COMPAT.md` §8.2):
/// `rasters/heightmap.f32` -> `rasters/heightmap.shuffled.f32`.
///
/// **The name is the fail-loud marker.** A reader that predates the shuffle
/// looks up `rasters/heightmap.f32`, does not find it, and refuses the archive
/// (§6.4) -- it never gets to put a plain-dump view over reordered bytes and
/// draw plausible noise, which a flag inside `project.json` could not have
/// prevented for a reader that did not check the flag. The same property runs
/// the other way: this reader un-shuffles an entry if and only if its name
/// carries this infix, so every archive written before 2026-09-23 reads
/// exactly as it always did.
pub const SHUFFLED_INFIX: &str = ".shuffled";

/// Which layout an archive turned out to be. Reported rather than inferred
/// by the caller, because "this came from an HTML export" is a real thing
/// for a UI to say and there is no second way to find out afterwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// The tree (`SAVEFILE_COMPAT.md` §5).
    Tree,
    /// The flat legacy layout (§15) — read-only.
    Flat,
}

/// A raster's element type. The archive carries it in the file extension
/// (`SAVEFILE_COMPAT.md` §8) because a reader must know the element width
/// before it can read a byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Element {
    F32,
    I32,
    U8,
}

impl Element {
    pub fn size(self) -> usize {
        match self {
            Element::F32 | Element::I32 => 4,
            Element::U8 => 1,
        }
    }

    pub fn ext(self) -> &'static str {
        match self {
            Element::F32 => "f32",
            Element::I32 => "i32",
            Element::U8 => "u8",
        }
    }
}

/// One grid payload, owning its values. `Vec` rather than a borrowed slice
/// because the read side has to allocate anyway and the write side is
/// handed rasters that mostly do not exist as a contiguous `Vec` in the
/// caller (a territory raster is `Vec<i32>`, a water-body classification is
/// `Vec<u8>`) — a borrowing enum would need three lifetimes to save one
/// clone of data that is about to be compressed.
#[derive(Debug, Clone, PartialEq)]
pub enum Raster {
    F32(Vec<f32>),
    I32(Vec<i32>),
    U8(Vec<u8>),
}

impl Raster {
    pub fn element(&self) -> Element {
        match self {
            Raster::F32(_) => Element::F32,
            Raster::I32(_) => Element::I32,
            Raster::U8(_) => Element::U8,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Raster::F32(v) => v.len(),
            Raster::I32(v) => v.len(),
            Raster::U8(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The entry's bytes as this writer stores them: byte-plane shuffled for a
    /// 4-byte element (`SAVEFILE_COMPAT.md` §8.2), verbatim for `u8`, which
    /// has one plane and nothing to reorder. Goes under
    /// [`raster_entry_name`], which agrees with this on which is which.
    fn write_to<W: Write>(&self, sink: &mut W) -> std::io::Result<()> {
        match self {
            Raster::U8(v) => sink.write_all(v),
            Raster::F32(v) => write_planes(sink, v, f32::to_le_bytes),
            Raster::I32(v) => write_planes(sink, v, i32::to_le_bytes),
        }
    }

    /// The inverse of [`write_planes`]: element `i`'s little-endian bytes are
    /// `bytes[i]`, `bytes[n+i]`, `bytes[2n+i]`, `bytes[3n+i]`. Decoded
    /// straight into the element vector, so un-shuffling costs no second
    /// `4n`-byte buffer on top of the one the entry inflated into. The caller
    /// has already checked `bytes.len() == n * 4`.
    fn from_planes(element: Element, bytes: &[u8]) -> Raster {
        let n = bytes.len() / 4;
        let word = |i: usize| [bytes[i], bytes[n + i], bytes[2 * n + i], bytes[3 * n + i]];
        match element {
            Element::F32 => Raster::F32((0..n).map(|i| f32::from_le_bytes(word(i))).collect()),
            Element::I32 => Raster::I32((0..n).map(|i| i32::from_le_bytes(word(i))).collect()),
            // One plane: the shuffle is the identity, and no `u8` entry is
            // ever stored under a shuffled name.
            Element::U8 => Raster::U8(bytes.to_vec()),
        }
    }

    /// Decodes explicitly rather than casting: the `Vec<u8>` a zip entry
    /// decompresses into is allocator-aligned, not guaranteed 4-byte
    /// aligned.
    fn from_bytes(element: Element, bytes: &[u8]) -> Raster {
        match element {
            Element::U8 => Raster::U8(bytes.to_vec()),
            Element::F32 => Raster::F32(
                bytes
                    .chunks_exact(4)
                    .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
                    .collect(),
            ),
            Element::I32 => Raster::I32(
                bytes
                    .chunks_exact(4)
                    .map(|c| i32::from_le_bytes(c.try_into().unwrap()))
                    .collect(),
            ),
        }
    }
}

/// One registered raster slot. The registry is the tree's guarantee that
/// two subsystems cannot each invent a home for the same grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RasterSlot {
    pub path: &'static str,
    pub element: Element,
}

/// Every raster path the format defines (`SAVEFILE_COMPAT.md` §8.1),
/// including the reserved ones — reserved paths are registered precisely so
/// that a later implementation fills the named slot rather than inventing
/// `rasters/biomes.u8` beside it.
pub const RASTER_SLOTS: &[RasterSlot] = &[
    RasterSlot {
        path: "rasters/heightmap.f32",
        element: Element::F32,
    },
    RasterSlot {
        path: "rasters/temperature.f32",
        element: Element::F32,
    },
    RasterSlot {
        path: "rasters/rainfall.f32",
        element: Element::F32,
    },
    RasterSlot {
        path: "rasters/volcanic_field.f32",
        element: Element::F32,
    },
    RasterSlot {
        path: "rasters/impact_field.f32",
        element: Element::F32,
    },
    RasterSlot {
        path: "rasters/strahler_order.u8",
        element: Element::U8,
    },
    RasterSlot {
        path: "rasters/territory.i32",
        element: Element::I32,
    },
    RasterSlot {
        path: "rasters/provinces.i32",
        element: Element::I32,
    },
    RasterSlot {
        path: "rasters/water_bodies.u8",
        element: Element::U8,
    },
    RasterSlot {
        path: "rasters/agrarian_density.f32",
        element: Element::F32,
    },
    RasterSlot {
        path: "rasters/biome.u8",
        element: Element::U8,
    },
    RasterSlot {
        path: "rasters/lithology.u8",
        element: Element::U8,
    },
    RasterSlot {
        path: "rasters/koppen.u8",
        element: Element::U8,
    },
    RasterSlot {
        path: "rasters/wildlife.u8",
        element: Element::U8,
    },
    // The world substrate (`SAVEFILE_COMPAT.md` §8.3, owner Ruling AR,
    // 2026-09-24): every grid of `cartalith_engine::WorldState` the six core
    // rasters above do not already carry, so a reopened project is the world
    // that was saved rather than a terrain-only stand-in for it. Written
    // together, under `project.json`'s `substrate` member ([`SUBSTRATE_MEMBER`]),
    // which is what says the set is complete; see [`SUBSTRATE_RASTERS`].
    RasterSlot {
        path: "rasters/flow_discharge.f32",
        element: Element::F32,
    },
    RasterSlot {
        path: "rasters/plate_id.i32",
        element: Element::I32,
    },
    RasterSlot {
        path: "rasters/boundary_mask.u8",
        element: Element::U8,
    },
    RasterSlot {
        path: "rasters/boundary_type.u8",
        element: Element::U8,
    },
    RasterSlot {
        path: "rasters/stress_field.f32",
        element: Element::F32,
    },
    RasterSlot {
        path: "rasters/shear_field.f32",
        element: Element::F32,
    },
    RasterSlot {
        path: "rasters/age_field.f32",
        element: Element::F32,
    },
    RasterSlot {
        path: "rasters/resistance_field.f32",
        element: Element::F32,
    },
    RasterSlot {
        path: "rasters/crust_field.f32",
        element: Element::F32,
    },
    RasterSlot {
        path: "rasters/channel_receiver.i32",
        element: Element::I32,
    },
    RasterSlot {
        path: "rasters/channel_mask.u8",
        element: Element::U8,
    },
    RasterSlot {
        path: "rasters/river_intensity.f32",
        element: Element::F32,
    },
    RasterSlot {
        path: "rasters/river_mask.u8",
        element: Element::U8,
    },
    RasterSlot {
        path: "rasters/river_floor.f32",
        element: Element::F32,
    },
];

/// The world-substrate rasters, in [`RASTER_SLOTS`] order (`SAVEFILE_COMPAT.md`
/// §8.3). A caller that writes any of them MUST also write the
/// [`SUBSTRATE_MEMBER`] manifest member describing the set, and a reader MUST
/// NOT treat the set as complete without it: a raster present with no member
/// is a payload an older build carried through a re-save it could not
/// describe.
pub const SUBSTRATE_RASTERS: [&str; 14] = [
    "rasters/flow_discharge.f32",
    "rasters/plate_id.i32",
    "rasters/boundary_mask.u8",
    "rasters/boundary_type.u8",
    "rasters/stress_field.f32",
    "rasters/shear_field.f32",
    "rasters/age_field.f32",
    "rasters/resistance_field.f32",
    "rasters/crust_field.f32",
    "rasters/channel_receiver.i32",
    "rasters/channel_mask.u8",
    "rasters/river_intensity.f32",
    "rasters/river_mask.u8",
    "rasters/river_floor.f32",
];

/// `project.json`'s top-level member describing the world substrate
/// (`SAVEFILE_COMPAT.md` §8.3). This crate carries it opaquely -- written from
/// [`ProjectWrite::substrate`], read into [`ProjectData::substrate`] -- because
/// its schema is the producer's: which optional grids the world had, and the
/// one scalar (`integrated_drainage`) that is a property of the drainage those
/// grids describe. Absent in every archive written before 2026-09-24.
pub const SUBSTRATE_MEMBER: &str = "substrate";

/// The six rasters [`SaveFields`] carries, in the order
/// `SAVEFILE_COMPAT.md` §8.1 lists them. Written by [`write_project`] from
/// `fields` rather than from the caller's extra-raster map, so a project
/// cannot be saved with a terrain that disagrees with its own manifest.
pub const CORE_RASTERS: [&str; 6] = [
    "rasters/heightmap.f32",
    "rasters/temperature.f32",
    "rasters/rainfall.f32",
    "rasters/volcanic_field.f32",
    "rasters/impact_field.f32",
    "rasters/strahler_order.u8",
];

/// Every JSON document path the format defines (`SAVEFILE_COMPAT.md`
/// §9-§13), reserved ones included. [`write_project`] refuses anything not
/// listed here — see this module's own doc comment for why that refusal is
/// the point rather than a nuisance.
///
/// `history/territory/<year>.i32` is **not** here: its name carries a year,
/// so it is validated structurally rather than by lookup (see
/// [`ProjectWrite::history_territory`]).
///
/// # `entities/landmarks.json` carries the dock's settings **and its results**
///
/// Registered 2026-09-01. Before that the landmark layer was in no slot at
/// all, and `write_project` rejects an unregistered slot outright, so the name
/// had to exist here before a writer could exist anywhere.
/// `cartalith-godot`'s `project_bridge.rs` supplies that writer
/// (`LandmarksDoc`, written from `WorldGen::landmark_store` and parsed back on
/// open), and lists the slot in its own `ENGINE_OWNED_SLOTS` so the shell
/// cannot overwrite a document it has no view of.
///
/// # `annotations/measurements.json` is the opposite case — caller-owned
///
/// Registered 2026-09-03 (`SAVEFILE_COMPAT.md` §11.4). A saved measurement is
/// a mode name, the grid points that were clicked and the reading they
/// produced; the engine models none of those as retained state — its measure
/// functions are stateless queries over points *the caller owns*
/// (`measure_bridge.rs`) — so this slot is written and read by the shell
/// exactly the way `entities/journeys.json` is, and it is deliberately absent
/// from `project_bridge.rs`'s `ENGINE_OWNED_SLOTS`.
///
/// It is under `annotations/` rather than `entities/` by §5.1's own test: a
/// measurement is a mark the author made on the sheet, not a thing in the
/// world with an id that other documents reference.
///
/// # `library/settlement_types.json` is caller-owned, same shape as the
/// measurements slot above
///
/// Registered 2026-09-21 (`lazy-riding-piglet.md` Batch D — the Settlement
/// Editor's artboard 1f, "Settlement types"). A settlement type is a named
/// bundle of fields a settlement already has (kind, specialisation, traits,
/// walls, age policy, name-pool source) plus a per-faction default — the
/// settlement-drop tool applies a bundle's fields via the same
/// `civ_edit_settlement`/`civ_settlement_toggle_trait` calls the place
/// editor already uses. Nothing in `WorldGen` models a "type": it is purely
/// authored data that travels with the project rather than global
/// preferences, so this slot is caller-owned (`cartalith-godot`'s
/// `project_bridge.rs` does **not** list it in `ENGINE_OWNED_SLOTS`) and the
/// shell writes and reads it directly, the same way it already does for
/// `annotations/measurements.json`.
///
/// **Both halves are written, and they are written for different reasons.**
/// The authored settings — per-kind caps, armed flags, crowding, the four
/// class radii — are hand-entered configuration that no recomputation brings
/// back, and until this slot existed they were never written *and* never
/// cleared, so they followed the user out of whichever project was open last
/// into the next one.
///
/// The retained *run* is written too, since owner ruling 10 (2026-09-06):
/// *"Landmark persistence? → PERSIST in `entities/landmarks.json`."* **This
/// paragraph used to say the opposite** — "the retained run is not written,
/// because `cartalith_civ::landmark::generate` is a pure function of the
/// world, the settings and the seed".
///
/// **That argument is now wrong, and this paragraph has been wrong about how
/// wrong it was.** It first read "superseded rather than wrong — re-running the
/// pass does reproduce the placement exactly", which was true when written and
/// stopped being true on 2026-09-06, when `landmark_run_inner` began assembling
/// `LandmarkInputs::manual_icons` under owner ruling 14. The pass takes a
/// **fourth** input, so a re-run reproduces the placement only if the
/// hand-placed icons are also unchanged — and a user can move one at any time.
/// `cartalith_civ::landmark::generate`'s own doc carries the four-term form and
/// flags this exact argument shape; read it there rather than trusting a
/// restatement here. What a re-run never reproduces, fourth term or not, is
/// anything a person or a later pass has since attached to a placement.
/// `LandmarksDoc.results` is `Option` and
/// `skip_serializing_if`, so a project whose pass has never run still writes
/// no run at all, and `load_save` reached on its own still invalidates —
/// `project_bridge.rs`'s restore puts the archive's own placements back on
/// top afterwards, over the field and grid that archive carried.
///
/// # `entities/conflicts.json` is engine-owned
///
/// Registered 2026-09-23 (`STORY_PLANNING_SCOPE.md` SP-4,
/// `SAVEFILE_COMPAT.md` §9.7). The payload is `WorldGen::conflicts`, which
/// GDScript has no view of, so `project_bridge.rs` writes and reads it and
/// lists it in `ENGINE_OWNED_SLOTS` -- the landmarks slot's arrangement. It
/// is under `entities/` because a conflict is a thing other documents refer
/// to by id (its settlement/province anchor points the other way, by `tid`).
pub const DOCUMENT_SLOTS: &[&str] = &[
    "entities/settlements.json",
    "entities/factions.json",
    "entities/ways.json",
    "entities/provinces.json",
    "entities/continents.json",
    "entities/journeys.json",
    "entities/landmarks.json",
    "entities/conflicts.json",
    "history/timeline.json",
    "annotations/labels.json",
    "annotations/icons.json",
    "annotations/regions.json",
    "annotations/measurements.json",
    "library/assets.json",
    "library/travel.json",
    "library/settlement_types.json",
    "drafts/paint.json",
    "drafts/sculpt.json",
    "appearance.json",
    "vault.json",
];

/// `history/territory/` — the prefix a per-year territory raster sits
/// under. Separate from `rasters/` because `rasters/` is *this* world's
/// grids and a snapshot is a past one (`SAVEFILE_COMPAT.md` §5.1).
pub const HISTORY_TERRITORY_PREFIX: &str = "history/territory/";

/// `cartography/tiles/` — the prefix the **optional** stored LOD tile
/// pyramid sits under (owner ruling 28, 2026-09-06: *"the LOD tiles should
/// be stored in the save, or at least optional to include"*).
///
/// The name already existed as the archive's canonical *foreign* example
/// (`SAVEFILE_COMPAT.md` §6.2's round-trip test uses
/// `cartography/tiles/0/0/0.png`), and this promotes one shape of it to a
/// first-class slot family. It is a **prefix** rather than a registered
/// slot for the same reason `history/territory/` is: the address —
/// `<z>/<col>/<row>` — is part of the name, so it is validated structurally
/// (by this module's own `lod_tile_id`) rather than by lookup in a fixed
/// list.
pub const LOD_TILE_PREFIX: &str = "cartography/tiles/";

/// The stored pyramid's own index — **written by [`write_project`] itself**,
/// never by a caller, because the part of it that matters
/// ([`LodTiles::source_key`]) has to be computed from the archive's own
/// heightmap or it is not a check, it is a claim.
///
/// Deliberately absent from [`DOCUMENT_SLOTS`], which is what makes a
/// caller-supplied `cartography/tiles/index.json` a
/// [`SaveError::UnknownSlot`] instead of a second, disagreeing index.
pub const LOD_TILE_INDEX: &str = "cartography/tiles/index.json";

/// The `.u8` element extension every stored tile carries, for the reason
/// §8 gives for the rasters: a payload with no header carries its element
/// width in its name.
///
/// **This is `u8` the element type, not "one byte per pixel."** A tile was
/// one byte per pixel — a relief-detail shade ratio — until
/// `LOD_DETAIL_SCOPE.md` LOD-D2 (2026-09-21) made a tile a colour picture:
/// `cartalith_godot::lod_bridge::tile_mask` now emits three `u8` bytes per
/// pixel (R, G, B). The extension still names the *element width* correctly
/// (one byte, same as the rasters' own `.u8` slots) — it was never a
/// per-pixel-count claim, and [`LodTiles::tile_w`]/[`LodTiles::tile_h`] carry
/// the pixel geometry a reader needs to turn bytes back into pixels.
const LOD_TILE_EXT: &str = ".u8";

/// Bytes per pixel in a stored tile — **`3`, RGB**, since LOD-D2
/// (2026-09-21). See [`LOD_TILE_EXT`]'s own doc comment for the one-channel
/// history; every per-tile length check in this module multiplies by this
/// rather than repeating the literal.
const LOD_TILE_CHANNELS: usize = 3;

/// One stored LOD tile pyramid: what it was made from, how big one tile is,
/// and the tiles themselves.
///
/// # This is a **cache**, and the archive treats it as one
///
/// `SAVEFILE_COMPAT.md` §16.1 refused to store a pyramid at all and gave
/// three reasons. Ruling 28 overrode the decision, not the reasons, and two
/// of them are still true and are handled here rather than argued away:
///
/// - **It is enormous, and more so since the tile format tripled.** The
///   figures immediately below this bullet until 2026-09-22 were the
///   *pre-LOD-D2* one-channel measurement (4.40-5.12 MiB at levels 0-5,
///   21.87-27.75 MiB at 0-6) and were stale from the day `LOD_TILE_CHANNELS`
///   became `3` — a shade-ratio mask compresses far better than a colour
///   picture, so the real numbers are not a small correction. Re-measured on
///   the real target grid (`cartalith_godot::lod_bridge::pyramid_mask_bytes`'s
///   own doc carries the exact command): levels 0-4 deflate to **18.10 MiB**,
///   0-5 to **49.60 MiB**, 0-6 to **131.02 MiB**, against a whole archive of
///   ~24.9 MiB for that grid (§18.1, itself pre-dating this feature and worth
///   re-checking rather than trusting). A five-level pyramid alone now
///   **roughly doubles the file**; a six-level one is **five times** it.
///   `cartalith_godot::lod_bridge::SAVE_PYRAMID_MAX_LEVEL` (`4`) is where the
///   one caller that writes this slot today draws that line, and its own doc
///   comment carries the reasoning. So the slot is optional, `None` is what
///   [`ProjectWrite::new`] builds, and the depth is the caller's to choose.
/// - **It goes stale invisibly** — §16.1's own words, *"a reader cannot
///   cheaply tell which is older"*. It can now: [`LodTiles::source_key`] is
///   computed by the writer from the heightmap it is writing, and
///   [`read_project`] recomputes it from the heightmap it just read.
///   A mismatch **drops every tile with a warning** rather than handing back
///   a pyramid drawn over a world that has since been re-sculpted. Ruling 28:
///   *"prefer dropping them to drawing them."*
///
/// The third reason — that a pyramid is derived and can always be rebuilt —
/// is exactly why dropping is safe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LodTiles {
    /// FNV-1a-64 hex over everything the tile synthesizer reads from the
    /// world: the heightmap, the grid, the seed and the sea level. See
    /// [`lod_source_key`] for why those five and nothing else.
    ///
    /// **Checked on both sides, and the write side is the half that was
    /// missing.** [`write_project`] computes the key it stores from the
    /// heightmap it is writing — but if this field is **non-empty** it is first
    /// read as a claim about which world these tiles came from, and a
    /// disagreement drops the pyramid instead of stamping it with the new
    /// world's key. Leaving it **empty** still means *"I built these for the
    /// world I am handing you"* and is trusted, which is the contract a fresh
    /// producer wants; carrying a key forward from a read is what makes the
    /// check fire.
    pub source_key: String,
    /// Whatever the producer calls itself and its constants. Stored verbatim
    /// and returned verbatim; this crate never interprets it.
    ///
    /// It exists because [`source_key`](Self::source_key) covers the *world*
    /// and nothing else. A tile is also a function of its producer's own
    /// constants — tile size, the encoding's fixed point, the octave
    /// schedule — and a build that changes one of those would otherwise
    /// decode last week's tiles with this week's shader. The container
    /// cannot know those values, so the producer names itself and **the
    /// caller compares** before using the tiles.
    pub producer: String,
    /// One tile's pixel dimensions, the same for every level of a pyramid
    /// (a level's tile *footprint* shrinks; its pixel size does not).
    ///
    /// Stored so that the length guard this format applies to every other
    /// headerless payload applies here too: a tile entry that is not exactly
    /// `tile_w * tile_h * 3` bytes (RGB, since LOD-D2 — see [`LOD_TILE_EXT`]'s
    /// own doc comment) is a truncated tile, not a parse error, and nothing
    /// else in the archive would catch it.
    pub tile_w: usize,
    pub tile_h: usize,
    /// `(z, col, row)` -> that tile's bytes, **R, G, B per pixel, row-major**
    /// (`tile_w * tile_h * 3` bytes; see [`LOD_TILE_EXT`]'s own doc comment
    /// for why a tile was one channel before LOD-D2 and is three now).
    ///
    /// [`cartalith_spatial::pyramid::ChunkId`] rather than a tuple for the
    /// reason `atlas.rs` already gives: the pyramid geometry, the atlas key
    /// and the portable manifest all address a tile with this exact value,
    /// and a second copy of it here would have to agree by convention.
    pub tiles: BTreeMap<cartalith_spatial::pyramid::ChunkId, Vec<u8>>,
}

/// `cartography/tiles/<z>/<col>/<row>.u8` -> its address, or `None` for
/// anything else under the prefix.
///
/// Structural, like the `history/territory/<year>.i32` branch beside it, and
/// strict for the same reason: an entry under this prefix that does not parse
/// is somebody *else's* payload — `cartography/tiles/0/0/0.png` is the entry
/// §6.2's round-trip fixture uses — so it must fall through to
/// [`ProjectData::foreign`] and be carried unchanged rather than be adopted
/// and then dropped.
fn lod_tile_id(name: &str) -> Option<cartalith_spatial::pyramid::ChunkId> {
    let rest = name.strip_prefix(LOD_TILE_PREFIX)?.strip_suffix(LOD_TILE_EXT)?;
    let mut parts = rest.split('/');
    let z = parts.next()?;
    let col = parts.next()?;
    let row = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    // `str::parse::<u32>` rejects a leading `+`, a leading `-` and a leading
    // zero is accepted -- so `00/0/0.u8` would parse to the same id as
    // `0/0/0.u8`. Refused rather than aliased: two entry names for one tile
    // is the duplicate `ZipWriter` refuses outright on the way back out.
    for part in [z, col, row] {
        if part.len() > 1 && part.starts_with('0') {
            return None;
        }
    }
    Some(cartalith_spatial::pyramid::ChunkId::new(
        z.parse().ok()?,
        col.parse().ok()?,
        row.parse().ok()?,
    ))
}

fn lod_tile_entry(id: cartalith_spatial::pyramid::ChunkId) -> String {
    format!("{LOD_TILE_PREFIX}{}/{}/{}{LOD_TILE_EXT}", id.z, id.col, id.row)
}

/// The cache key a stored pyramid is checked against — FNV-1a-64 over the
/// world the tiles were synthesized from.
///
/// # Why these five inputs and no others
///
/// Derived from the **signature** of the synthesizer, not from a guess about
/// what matters: `cartalith_godot::lod_bridge::synthesize_tile_rgba` takes
/// `(field, gw, gh, z, col, row, seed, sea)`. `z`/`col`/`row` are the tile's
/// own address and travel in its entry name, which leaves the heightmap, the
/// two grid dimensions, the seed and the sea level — all five here.
///
/// Everything else the synthesizer reads is a compile-time constant of the
/// producer (`TILE_PX`, `z_base()`, the shade-ratio fixed point, the sun
/// azimuth, `AmplifyOpts::default()`), which is what
/// [`LodTiles::producer`] covers. `map_width_km`, `wrap_x` and `origin` are
/// **not** inputs — no argument of `synthesize_tile_rgba` carries them and
/// `AmplifyOpts` has no field for either — so hashing them would invalidate
/// a valid cache on a metadata edit.
///
/// FNV-1a-64 because this workspace already hashes fields with it in a dozen
/// golden tests and it needs no dependency. It is not a cryptographic hash
/// and does not need to be: the thing it defends against is a *changed*
/// world, not a forged one.
pub fn lod_source_key(params: &SaveParams, heightmap: &[f32]) -> String {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut h = OFFSET;
    let mut eat = |bytes: &[u8]| {
        for &b in bytes {
            h ^= b as u64;
            h = h.wrapping_mul(PRIME);
        }
    };
    eat(&(params.gw as u64).to_le_bytes());
    eat(&(params.gh as u64).to_le_bytes());
    eat(&(params.seed as i64).to_le_bytes());
    eat(&params.sea_level.to_bits().to_le_bytes());
    for v in heightmap {
        eat(&v.to_le_bytes());
    }
    format!("{h:016x}")
}

fn raster_slot(path: &str) -> Option<RasterSlot> {
    RASTER_SLOTS.iter().copied().find(|s| s.path == path)
}

/// Whether `name` is an entry **this build produces and consumes itself** —
/// the manifest, `params.json`, the two human-facing extras, any registered
/// document slot, any registered raster under **either** of its names (§8.2's
/// shuffled one included), a `history/territory/<year>.i32`,
/// or the stored LOD pyramid's index and tiles.
///
/// One function because both directions must agree exactly: [`read_project`]
/// files everything else under [`ProjectData::foreign`], and [`write_project`]
/// re-emits that map while refusing to let a carried copy shadow an entry it
/// writes from the live model. Two hand-kept lists would drift, and the drift
/// would show up as a **failed save** -- `ZipWriter` refuses a duplicate entry
/// name rather than writing one.
///
/// The two LOD branches are deliberately the *narrow* shape and not the whole
/// `cartography/tiles/` prefix: `cartography/tiles/0/0/0.png` — the entry
/// §6.2's round-trip test uses — is still somebody else's payload and is
/// still carried foreign, unchanged.
fn is_own_entry(name: &str) -> bool {
    name == PROJECT_MANIFEST
        || name == "params.json"
        || name == "README.md"
        || name == "preview.png"
        || name == LOD_TILE_INDEX
        || DOCUMENT_SLOTS.contains(&name)
        || raster_slot(name).is_some()
        || RASTER_SLOTS.iter().any(|&s| raster_entry_name(s) == name)
        || (name.starts_with(HISTORY_TERRITORY_PREFIX) && name.ends_with(".i32"))
        || lod_tile_id(name).is_some()
}

/// One project, ready to be written.
///
/// Deliberately not `Default`-constructible in one step: `params` and
/// `fields` are the two things an archive cannot be valid without, so they
/// are required arguments of [`ProjectWrite::new`] rather than fields a
/// caller can forget.
pub struct ProjectWrite<'a> {
    pub params: &'a SaveParams,
    /// The six core terrain rasters. Their lengths are checked against
    /// `params.gw * params.gh` before anything is written.
    pub fields: &'a SaveFields,
    /// `params.json`'s `cartalith` view — a flat map of dotted parameter
    /// keys. `Value::Null` writes no such member.
    pub cartalith_params: serde_json::Value,
    /// `params.json`'s `reference` view — the same settings under the HTML
    /// app's nested names. `Value::Null` writes no such member.
    pub reference_params: serde_json::Value,
    /// Rasters beyond the six core ones, keyed by full registered path
    /// (e.g. `"rasters/territory.i32"`).
    pub rasters: BTreeMap<String, Raster>,
    /// Registered document slot -> JSON text. The text is written
    /// verbatim; this crate does not reformat or validate a caller's
    /// schema, only that it parses.
    pub documents: BTreeMap<String, String>,
    /// Recorded year -> that year's territory raster.
    pub history_territory: BTreeMap<i64, Vec<i32>>,
    /// The optional stored LOD tile pyramid (owner ruling 28). `None` writes
    /// no `cartography/tiles/` entries at all, which is what
    /// [`ProjectWrite::new`] builds and what the measurement behind ruling 28
    /// recommends as the default — see [`LodTiles`] for the bytes.
    ///
    /// **Omitting it on a re-save deletes a pyramid the archive carried**,
    /// exactly as leaving [`ProjectWrite::history_territory`] empty drops the
    /// recorded years. That is correct for a cache and would not be for
    /// authored data; it is the reason this slot holds only derived pixels
    /// and no setting a user typed.
    pub lod_tiles: Option<LodTiles>,
    pub preview_png: Option<Vec<u8>>,
    /// Entries this build does not model, carried through from
    /// [`ProjectData::foreign`] and re-emitted **verbatim** — the first of
    /// `SAVEFILE_COMPAT.md` §6.2's two options for a reader that writes an
    /// archive back ("retain the raw bytes of every entry it did not consume
    /// and re-emit them unchanged").
    ///
    /// Empty for a project this build authored, which is the only reason a
    /// `Default`-shaped writer stays correct: a caller that never opened an
    /// archive has nothing foreign to carry, and one that did opts in by
    /// assigning the map it was handed.
    ///
    /// **A name this writer produces itself always wins.** If a later build
    /// registers a slot that an earlier open filed as foreign, the entry is
    /// written from the live model and the carried copy is skipped. That is
    /// not a stylistic preference: `ZipWriter` refuses a duplicate name
    /// outright (`InvalidArchive("Duplicate filename: ...")`, measured), so
    /// without the skip the whole save fails rather than the stale copy
    /// losing quietly.
    pub foreign: BTreeMap<String, Vec<u8>>,
    /// `README.md`. `None` writes none; [`DEFAULT_README`] is what the
    /// Godot boundary passes.
    pub readme: Option<String>,
    /// `project.json`'s `generator`. Provenance only — no reader branches
    /// on it (`SAVEFILE_COMPAT.md` §7).
    pub generator: String,
    /// `project.json`'s `created`, an RFC 3339 UTC timestamp. `None` writes
    /// no member; this crate does not read a clock (it has no dependency
    /// that offers one, and inventing a timestamp is the caller's decision
    /// to make, not a file writer's).
    pub created: Option<String>,
    /// `project.json`'s [`SUBSTRATE_MEMBER`], verbatim. `Value::Null` writes
    /// no member -- which is what [`ProjectWrite::new`] builds, and what a
    /// world that is not saving its substrate rasters must leave it as.
    pub substrate: serde_json::Value,
}

impl<'a> ProjectWrite<'a> {
    pub fn new(params: &'a SaveParams, fields: &'a SaveFields) -> Self {
        ProjectWrite {
            params,
            fields,
            cartalith_params: serde_json::Value::Null,
            reference_params: serde_json::Value::Null,
            rasters: BTreeMap::new(),
            documents: BTreeMap::new(),
            history_territory: BTreeMap::new(),
            lod_tiles: None,
            preview_png: None,
            foreign: BTreeMap::new(),
            readme: None,
            generator: format!("cartalith-native {}", env!("CARGO_PKG_VERSION")),
            created: None,
            substrate: serde_json::Value::Null,
        }
    }

    /// Registers one document. Returns the previous text for that slot, if
    /// any — a caller writing the same slot twice is a bug worth seeing.
    pub fn document(&mut self, slot: impl Into<String>, json: impl Into<String>) -> Option<String> {
        self.documents.insert(slot.into(), json.into())
    }

    pub fn raster(&mut self, path: impl Into<String>, raster: Raster) -> Option<Raster> {
        self.rasters.insert(path.into(), raster)
    }
}

/// The `README.md` this port writes. Aimed at a human who opened the
/// archive in a zip tool and wants to know what the directories are, which
/// is the only audience it has (`SAVEFILE_COMPAT.md` §13.4: no program
/// reads it).
pub const DEFAULT_README: &str = "\
# Cartalith project

This archive is a Cartalith project, not a plain image export.
`project.json` says which format version it is.

    project.json      what this file is, and the grid every raster is measured against
    params.json       the settings the world was generated from
    rasters/          one value per grid cell -- elevation, climate, hydrology, territory
    entities/         settlements, factions, roads, provinces, continents
    history/          recorded past years
    annotations/      labels, icons and the selected region -- marks on the map
    library/          setting-level definitions that outlive any one world
    drafts/           uncommitted edits
    cartography/      optional cached zoom tiles -- derived from rasters/,
                      often absent, and safe to delete
    appearance.json   how the map is drawn
    vault.json        links out to an external Markdown vault
    preview.png       a thumbnail; not map data

Every raster decodes to a bare little-endian binary dump with no header:
exactly grid_width * grid_height elements, row-major,
index = y * grid_width + x. A `*.u8` entry is that dump as stored. A
`*.shuffled.f32` / `*.shuffled.i32` entry is byte-plane shuffled: all of the
elements' first bytes, then all their second bytes, then third, then fourth.
Byte k of element i is at offset k * (grid_width * grid_height) + i.
";

/// `SAVEFILE_COMPAT.md` §14.2, implemented once for the whole format.
///
/// Rewrites every JSON number that is stored as a float but has no
/// fractional part into an integer, recursively. `1.0` becomes `1`, `1e0`
/// becomes `1`, `1.5` is untouched, and a value too large for `i64` is
/// untouched (§14.1 forbids one that large anyway, and quietly mangling it
/// would be worse than leaving it visible).
///
/// **Why this is central and not per-field.** GDScript and JavaScript both
/// type every JSON number as a float, so any document that has passed
/// through either re-emits integers this way. `GUI_GAP_REGISTER.md` KV-04
/// is what a strict parser on the other side costs: the vault's link store
/// failed to deserialize on two integer fields and every link was silently
/// discarded on each launch, for the whole shipped life of the feature.
/// Per-field tolerance is per-field remembering.
pub fn coerce_integral_floats(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Number(n) => {
            // `is_f64()` first: a number already stored as an integer is
            // left exactly as it is, so this pass is a no-op on a document
            // written by a strict producer.
            if let Some(f) = n.as_f64().filter(|_| n.is_f64()) {
                // `f as i64` saturates in Rust, so the range test has to
                // come first or 1e30 would silently become i64::MAX.
                if f.fract() == 0.0 && f.abs() <= 9_007_199_254_740_991.0 {
                    *value = serde_json::Value::Number(serde_json::Number::from(f as i64));
                }
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                coerce_integral_floats(item);
            }
        }
        serde_json::Value::Object(map) => {
            for (_, v) in map.iter_mut() {
                coerce_integral_floats(v);
            }
        }
        _ => {}
    }
}

/// One project, as read. Everything past `save` is tree-only: a flat
/// archive carries no entities, history or annotations, and the empty maps
/// are the honest report of that rather than a failure.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectData {
    /// The grid, the six core rasters, and the parameter state — the same
    /// shape [`crate::load_save`] has always returned, so every existing
    /// caller keeps working against either layout.
    pub save: SaveData,
    pub layout: Layout,
    /// `project.json`'s `format_version`, or `0` for a flat archive. `1` and
    /// `2` read identically -- see [`SHUFFLED_INFIX`] for why the raster
    /// decode does not branch on this.
    pub format_version: i64,
    /// Registered rasters beyond the six core ones, keyed by full path.
    pub rasters: BTreeMap<String, Raster>,
    /// Registered documents, parsed and integer-coerced.
    pub documents: BTreeMap<String, serde_json::Value>,
    /// The same documents' JSON text, **verbatim** — the archive's own bytes
    /// with only a byte-order mark stripped (§14). Same keys as
    /// [`ProjectData::documents`] exactly: a document that was skipped
    /// appears in neither.
    ///
    /// Kept because re-serializing the parsed [`serde_json::Value`] is not
    /// the same text. It sorts object members (`Value`'s map is a
    /// `BTreeMap`), it drops whitespace, and §14.2's coercion pass has
    /// already rewritten `1.0` to `1` inside it. For a document *this*
    /// crate's callers parse against a schema none of that matters. For a
    /// document handed **back to whoever wrote it** — the caller-owned
    /// slots of §5, which no schema in this workspace models — all of it
    /// does: the round trip is only lossless if the text is the text.
    ///
    /// The cost is one extra copy of the documents in memory for the
    /// lifetime of a `ProjectData`, which is bounded by the same JSON the
    /// parsed map already holds and is small beside the rasters beside it.
    pub document_text: BTreeMap<String, String>,
    pub history_territory: BTreeMap<i64, Vec<i32>>,
    /// The stored LOD tile pyramid, **only if it still describes this
    /// world** (owner ruling 28). `None` when the archive carried none, and
    /// also `None` when it carried one whose [`LodTiles::source_key`] does
    /// not match the heightmap that was just read — a stale pyramid is
    /// dropped, with a line in [`ProjectData::warnings`] saying so, because
    /// ruling 28's own instruction is to *"prefer dropping them to drawing
    /// them"*.
    ///
    /// **Matching the key is necessary and not sufficient.** It says the
    /// tiles were made from this terrain; it says nothing about whether they
    /// were made by *this build's* synthesizer. Compare
    /// [`LodTiles::producer`] against your own before drawing one — this
    /// crate cannot, because the constants that go into it live in the
    /// producer.
    pub lod_tiles: Option<LodTiles>,
    pub preview_png: Option<Vec<u8>>,
    /// Entries this build does not know (`SAVEFILE_COMPAT.md` §6.3), keyed
    /// by archive entry name, **with their raw bytes**. Not an error and not
    /// a warning — an unknown entry is normal.
    ///
    /// **This was `foreign_entries: Vec<String>`, a names-only census, and
    /// the names alone were not enough.** §6.2's "without data loss" gives a
    /// writing reader exactly two options: re-emit every entry it did not
    /// consume, or refuse to overwrite the archive. A list of names supports
    /// neither — it only lets a caller warn before dropping — and
    /// `SAVEFILE_COMPAT.md` §17 named that as the format's one open
    /// conformance gap. Hand this map to [`ProjectWrite::foreign`] and the
    /// gap closes: a project written by a newer build and re-saved by an
    /// older one keeps the payloads the older one could not read.
    ///
    /// The keys are still the census — iterate them for the "N entries this
    /// build does not understand" sentence — so nothing that only wanted the
    /// names lost anything.
    ///
    /// The cost is honest and bounded: these bytes stay in memory for the
    /// lifetime of the `ProjectData`, next to the rasters, which are larger.
    pub foreign: BTreeMap<String, Vec<u8>>,
    /// `project.json`'s [`SUBSTRATE_MEMBER`], verbatim and integer-coerced, or
    /// `Value::Null` when the manifest had none (every archive written before
    /// 2026-09-24, and every flat one).
    pub substrate: serde_json::Value,
    /// The core rasters (`CORE_RASTERS` after the heightmap) this read did
    /// **not** find, and filled in [`SaveData::fields`] with §8.1's
    /// substitute instead -- zeros, for all five. `Some(empty)` is a tree
    /// archive that carried every one; `None` is a flat archive, whose reader
    /// does not report it.
    ///
    /// Needed because the substitute is indistinguishable from data once it
    /// is in `fields`: an all-zero `strahler_order` is both "no channels" and
    /// "the raster was missing". A consumer that rebuilds state *from* one of
    /// these grids -- the world substrate (§8.3) rebuilds `stream_order` from
    /// `strahler_order` -- must be able to tell which it holds.
    pub core_substituted: Option<Vec<String>>,
    /// Everything that was skipped and why (`SAVEFILE_COMPAT.md` §6.4).
    ///
    /// A damaged optional entry must not cost the user their world, so it
    /// is skipped rather than fatal — but "skipped silently" is how a
    /// format loses data without anyone noticing, so every skip lands here
    /// for the caller to surface.
    pub warnings: Vec<String>,
}

impl ProjectData {
    /// One document by slot, or `None` if the archive did not carry it.
    pub fn document(&self, slot: &str) -> Option<&serde_json::Value> {
        self.documents.get(slot)
    }

    /// One document's text exactly as the archive carried it, or `None` if
    /// the archive did not carry it. See [`ProjectData::document_text`] for
    /// why the verbatim text is kept alongside the parsed value.
    pub fn text_of(&self, slot: &str) -> Option<&str> {
        self.document_text.get(slot).map(String::as_str)
    }

    /// One document by slot, deserialized. `None` when the slot is absent;
    /// `Some(Err(..))` when it is present and does not fit `T`, which the
    /// caller should turn into a warning rather than a failed load.
    pub fn parse<T: serde::de::DeserializeOwned>(
        &self,
        slot: &str,
    ) -> Option<Result<T, serde_json::Error>> {
        self.documents
            .get(slot)
            .map(|v| serde_json::from_value(v.clone()))
    }

    pub fn raster(&self, path: &str) -> Option<&Raster> {
        self.rasters.get(path)
    }
}

// =============================== writing ===============================

fn zip_opts() -> zip::write::SimpleFileOptions {
    // DEFLATE (method 8). Named rather than inherited from
    // `SimpleFileOptions::default()` because it is a format decision
    // (`SAVEFILE_COMPAT.md` §3) and not a default worth taking silently.
    zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated)
}

/// Writes one project archive in the tree layout.
///
/// Takes a seekable sink rather than a path so a round-trip test can hand
/// it a `Cursor<Vec<u8>>`; `SAVEFILE_COMPAT.md` §3.2's atomicity
/// requirement is the *caller's* — building into memory and moving the
/// result into place is a filesystem decision this function has no
/// business making.
///
/// Returns the warnings produced while writing — today, only ever *"a
/// stale LOD pyramid was dropped"*, mirroring [`ProjectData::warnings`] on
/// the read side. Empty is the normal case; a caller that only cares about
/// success can still just `?`/`.unwrap()` the `Result` and ignore the `Ok`
/// payload.
pub fn write_project<W: Write + Seek>(
    sink: W,
    project: &ProjectWrite<'_>,
) -> Result<Vec<String>, SaveError> {
    let n = project.params.gw * project.params.gh;
    let f = project.fields;

    for (entry, got) in [
        (CORE_RASTERS[0], f.heightmap.len()),
        (CORE_RASTERS[1], f.temperature.len()),
        (CORE_RASTERS[2], f.rainfall.len()),
        (CORE_RASTERS[3], f.volcanic_field.len()),
        (CORE_RASTERS[4], f.impact_field.len()),
        (CORE_RASTERS[5], f.strahler_order.len()),
    ] {
        if got != n {
            return Err(SaveError::FieldLength {
                entry,
                expected: n,
                got,
            });
        }
    }

    // Validate every caller-supplied payload BEFORE opening the writer, so
    // a rejected save never produces a partial archive even when the caller
    // ignored §3.2 and handed us a file.
    for (path, raster) in &project.rasters {
        let Some(slot) = raster_slot(path) else {
            return Err(SaveError::UnknownSlot(path.clone()));
        };
        if raster.element() != slot.element {
            return Err(SaveError::RasterElement {
                entry: path.clone(),
                expected: slot.element,
                got: raster.element(),
            });
        }
        if raster.len() != n {
            return Err(SaveError::RasterLength {
                entry: path.clone(),
                expected: n,
                got: raster.len(),
            });
        }
        if CORE_RASTERS.contains(&path.as_str()) {
            // The six core rasters come from `fields`, and only from
            // there. Accepting a second copy here would let an archive
            // carry a terrain that disagrees with its own manifest -- the
            // duplication the tree exists to remove, one layer down.
            return Err(SaveError::UnknownSlot(path.clone()));
        }
    }
    for (slot, text) in &project.documents {
        if !DOCUMENT_SLOTS.contains(&slot.as_str()) {
            return Err(SaveError::UnknownSlot(slot.clone()));
        }
        if let Err(e) = serde_json::from_str::<serde_json::Value>(text) {
            return Err(SaveError::DocumentJson {
                entry: slot.clone(),
                message: e.to_string(),
            });
        }
    }
    for (year, raster) in &project.history_territory {
        if raster.len() != n {
            return Err(SaveError::RasterLength {
                entry: format!("{HISTORY_TERRITORY_PREFIX}{year}.i32"),
                expected: n,
                got: raster.len(),
            });
        }
    }
    // The same guard the rasters get, for the same reason: a tile carries no
    // length of its own, so a short one is not a parse error, it is a
    // truncated picture. `tile_w * tile_h * LOD_TILE_CHANNELS` is checked
    // here rather than inferred from the first tile, so a pyramid of
    // uniformly-wrong tiles still fails.
    if let Some(lod) = &project.lod_tiles {
        let per_tile = lod
            .tile_w
            .checked_mul(lod.tile_h)
            .and_then(|px| px.checked_mul(LOD_TILE_CHANNELS))
            .unwrap_or(0);
        if per_tile == 0 {
            return Err(SaveError::RasterLength {
                entry: LOD_TILE_INDEX.to_string(),
                expected: 1,
                got: 0,
            });
        }
        for (id, bytes) in &lod.tiles {
            if bytes.len() != per_tile {
                return Err(SaveError::RasterLength {
                    entry: lod_tile_entry(*id),
                    expected: per_tile,
                    got: bytes.len(),
                });
            }
        }
    }

    let mut writer = zip::ZipWriter::new(sink);
    let opts = zip_opts();

    // `project.json` first: a partially transferred archive is then
    // diagnosable (`SAVEFILE_COMPAT.md` §3.1).
    writer.start_file(PROJECT_MANIFEST, opts)?;
    writer.write_all(
        &serde_json::to_vec_pretty(&manifest_json(project)).expect("a Value always serializes"),
    )?;

    if !project.cartalith_params.is_null() || !project.reference_params.is_null() {
        let mut params = serde_json::Map::new();
        if !project.cartalith_params.is_null() {
            params.insert("cartalith".into(), project.cartalith_params.clone());
        }
        if !project.reference_params.is_null() {
            params.insert("reference".into(), project.reference_params.clone());
        }
        writer.start_file("params.json", opts)?;
        writer.write_all(
            &serde_json::to_vec_pretty(&serde_json::Value::Object(params))
                .expect("a Value always serializes"),
        )?;
    }

    // `.as_slice()` uniformly: `heightmap`/`temperature`/`rainfall` are
    // `Arc<Vec<f32>>` and `volcanic_field`/`impact_field` are plain
    // `Vec<f32>` (`SaveFields`'s own doc), so a literal array of `&f.<name>`
    // would mix two reference types and fail to compile.
    //
    // Every 4-byte raster goes out byte-plane shuffled under its
    // `.shuffled.` name (`SAVEFILE_COMPAT.md` §8.2, owner Ruling AJ).
    for (path, values) in [
        (CORE_RASTERS[0], f.heightmap.as_slice()),
        (CORE_RASTERS[1], f.temperature.as_slice()),
        (CORE_RASTERS[2], f.rainfall.as_slice()),
        (CORE_RASTERS[3], f.volcanic_field.as_slice()),
        (CORE_RASTERS[4], f.impact_field.as_slice()),
    ] {
        let slot = raster_slot(path).expect("every core raster is a registered slot");
        writer.start_file(raster_entry_name(slot), opts)?;
        write_planes(&mut writer, values, f32::to_le_bytes)?;
    }
    writer.start_file(CORE_RASTERS[5], opts)?;
    writer.write_all(&f.strahler_order)?;

    for (path, raster) in &project.rasters {
        let slot = raster_slot(path).expect("validated above");
        writer.start_file(raster_entry_name(slot), opts)?;
        raster.write_to(&mut writer)?;
    }

    for (slot, text) in &project.documents {
        writer.start_file(slot.as_str(), opts)?;
        writer.write_all(text.as_bytes())?;
    }

    for (year, values) in &project.history_territory {
        writer.start_file(format!("{HISTORY_TERRITORY_PREFIX}{year}.i32"), opts)?;
        write_dump(&mut writer, values, i32::to_le_bytes)?;
    }

    // The optional pyramid. Its index is written from here and not from the
    // caller's `documents` map on purpose: `source_key` is computed from the
    // heightmap this very call is writing, so the archive cannot leave with a
    // key that describes some other world's terrain.
    //
    // **That guarantee is necessary and was not sufficient, and this block used
    // to claim it was.** Computing the key here makes the index agree with the
    // heightmap *whatever bytes the caller handed over* — so a caller holding
    // pre-sculpt tiles did not get caught, it got its stale tiles **stamped
    // with the new world's key** and read back later with an empty warning
    // list. A verifier built exactly that archive on 2026-09-06 and the reader
    // accepted it, because the reader can only catch an index that disagrees
    // with its own archive's heightmap, which this writer made impossible to
    // produce.
    //
    // So a **non-empty** `source_key` is now read as a claim and checked. Empty
    // still means "I built these for the world I am handing you" and is
    // trusted, which is the documented contract for a fresh producer. A
    // disagreement **drops the pyramid** rather than failing the save — ruling
    // 28's *"prefer dropping them to drawing them"*, and the pyramid is derived,
    // so nothing is lost that cannot be rebuilt.
    //
    // The drop used to be silent -- `write_project` returned `Result<(),
    // SaveError>` and had nowhere to put a warning, unlike `read_project`'s
    // `ProjectData::warnings`. `OUTSTANDING_WORK.md`'s
    // "a dropped pyramid is silent" row: nothing assigns
    // `ProjectWrite::lod_tiles` yet, so no caller can hit this today, but the
    // channel exists now so the save path is safe once one does.
    let mut warnings: Vec<String> = Vec::new();
    if let Some(lod) = &project.lod_tiles {
        let live_key = lod_source_key(project.params, &f.heightmap);
        if lod.source_key.is_empty() || lod.source_key == live_key {
            writer.start_file(LOD_TILE_INDEX, opts)?;
            writer.write_all(
                &serde_json::to_vec_pretty(&serde_json::json!({
                    "source_key": live_key,
                    "producer": lod.producer,
                    "tile_w": lod.tile_w,
                    "tile_h": lod.tile_h,
                }))
                .expect("a Value always serializes"),
            )?;
            for (id, bytes) in &lod.tiles {
                writer.start_file(lod_tile_entry(*id), opts)?;
                writer.write_all(bytes)?;
            }
        } else {
            // Same detection as `read_project`'s stale-pyramid branch, mirrored
            // rather than shared: the read side checks the key it just decoded
            // from the archive against the heightmap it just decoded too, and
            // has no `ZipWriter` or `SaveError` in scope to share a helper with.
            warnings.push(format!(
                "{LOD_TILE_PREFIX}: the tiles being written were made from a \
                 different world (key {}, this one is {live_key}) -- dropped \
                 rather than written",
                lod.source_key
            ));
        }
    }

    if let Some(png) = &project.preview_png {
        writer.start_file(
            "preview.png",
            zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored),
        )?;
        writer.write_all(png)?;
    }

    if let Some(readme) = &project.readme {
        writer.start_file("README.md", opts)?;
        writer.write_all(readme.as_bytes())?;
    }

    // Last, and deliberately: everything above is written from the live
    // model, so a carried entry that has since become one of ours loses to
    // the model rather than being emitted twice (see
    // [`ProjectWrite::foreign`]). §6.2's first option, in one loop.
    for (name, bytes) in &project.foreign {
        if is_own_entry(name) {
            continue;
        }
        writer.start_file(name.as_str(), opts)?;
        writer.write_all(bytes)?;
    }

    writer.finish()?;
    Ok(warnings)
}

/// Buffered, like every raster write here: at this port's 8192x8192 ceiling
/// a raster is 67 million values and a per-value call into the DEFLATE
/// encoder is the whole cost of the save.
const WRITE_CHUNK: usize = 64 * 1024;

/// §8's plain dump: each element's four little-endian bytes in turn. Only
/// `history/territory/<year>.i32` is still written this way (§10.2 -- the
/// shuffle does not reach that directory).
fn write_dump<W: Write, T: Copy>(sink: &mut W, values: &[T], le: fn(T) -> [u8; 4]) -> std::io::Result<()> {
    let mut buf: Vec<u8> = Vec::with_capacity(WRITE_CHUNK * 4);
    for chunk in values.chunks(WRITE_CHUNK) {
        buf.clear();
        buf.extend(chunk.iter().flat_map(|&v| le(v)));
        sink.write_all(&buf)?;
    }
    Ok(())
}

/// `SAVEFILE_COMPAT.md` §8.2's byte-plane shuffle: every element's byte 0,
/// then every byte 1, then 2, then 3 -- `out[k*n + i] = le(v[i])[k]`.
/// Streamed a plane at a time, so the shuffled entry never exists as a whole
/// second copy of the raster in memory; the price is four passes over
/// `values`, which is cheap beside deflating them.
fn write_planes<W: Write, T: Copy>(sink: &mut W, values: &[T], le: fn(T) -> [u8; 4]) -> std::io::Result<()> {
    let mut buf: Vec<u8> = Vec::with_capacity(WRITE_CHUNK);
    for plane in 0..4 {
        for chunk in values.chunks(WRITE_CHUNK) {
            buf.clear();
            buf.extend(chunk.iter().map(|&v| le(v)[plane]));
            sink.write_all(&buf)?;
        }
    }
    Ok(())
}

/// The archive entry this writer stores a registered raster under:
/// [`SHUFFLED_INFIX`] before the extension for a 4-byte element, the slot's
/// own path for `u8`. [`Raster::write_to`] makes the same split, so the name
/// and the bytes cannot disagree.
fn raster_entry_name(slot: RasterSlot) -> String {
    if slot.element.size() == 1 {
        return slot.path.to_string();
    }
    let stem = &slot.path[..slot.path.len() - slot.element.ext().len() - 1];
    format!("{stem}{SHUFFLED_INFIX}.{}", slot.element.ext())
}

/// `project.json` for one project (`SAVEFILE_COMPAT.md` §7). Public so a
/// caller can inspect or test exactly what would be written without writing
/// a file, the same way [`crate::save::params_json`] already can.
pub fn manifest_json(project: &ProjectWrite<'_>) -> serde_json::Value {
    let p = project.params;
    let mut root = serde_json::Map::new();
    root.insert("format".into(), serde_json::json!(PROJECT_FORMAT));
    root.insert(
        "format_version".into(),
        serde_json::json!(PROJECT_FORMAT_VERSION),
    );
    root.insert("generator".into(), serde_json::json!(project.generator));
    if let Some(created) = &project.created {
        root.insert("created".into(), serde_json::json!(created));
    }
    let mut world = serde_json::json!({
        "grid_width": p.gw,
        "grid_height": p.gh,
        // `wrap_x`, not `world`: a member called `world` inside an
        // object called `world` is not a name (`SAVEFILE_COMPAT.md` §7).
        "wrap_x": p.world,
        "map_width_km": p.map_width_km,
        "sea_level": p.sea_level,
        "seed": p.seed,
    });
    // `world.origin` (`SAVEFILE_COMPAT.md` §7) — **written only when the
    // caller has one**. Every other member of this object is MUST and is
    // always here; this one is the archive's answer to a question it may
    // genuinely not know, and the absent key is that answer. Writing
    // `"gen"` for a `None` would make an unknown provenance
    // indistinguishable from a recorded one, which is the whole defect the
    // member exists to close: it would let a re-saved import claim to be a
    // generated world and share its atlas namespace again.
    if let Some(origin) = &p.origin {
        world
            .as_object_mut()
            .expect("just built as an object")
            .insert("origin".into(), serde_json::json!(origin));
    }
    // `world.name` (`SAVEFILE_COMPAT.md` §7) — the world's own generated
    // display name, written only when the caller has one, the same MAY
    // shape as `origin` immediately above and for the same reason: an
    // absent key is how this format says "unknown", so writing a name for
    // a `None` would assert one the caller never had.
    if let Some(name) = &p.name {
        world
            .as_object_mut()
            .expect("just built as an object")
            .insert("name".into(), serde_json::json!(name));
    }
    root.insert("world".into(), world);
    // `substrate` (`SAVEFILE_COMPAT.md` §8.3) -- written only when the caller
    // saved the substrate rasters it describes.
    if !project.substrate.is_null() {
        root.insert(SUBSTRATE_MEMBER.into(), project.substrate.clone());
    }
    serde_json::Value::Object(root)
}

// =============================== reading ===============================

/// `None` means **the entry is not in the archive**, and nothing else.
///
/// Every other failure is `Some(Err(..))`, because the two are not the same
/// thing and the call sites above treat them differently: an absent optional
/// raster is normal, an unreadable one is a hole in the user's project. The
/// case that forced the distinction is a **compression method this build
/// cannot decode** (`SAVEFILE_COMPAT.md` §3.3) — `zip` reports it at
/// `by_name` time, and swallowing it with `.ok()?` reported an intact entry
/// as one that was never written. A re-save would then have dropped it in
/// silence, which is §6.2's failure mode and KV-04's shape all over again.
fn read_entry_bytes(
    archive: &mut zip::ZipArchive<impl Read + Seek>,
    name: &str,
) -> Option<Result<Vec<u8>, std::io::Error>> {
    let mut entry = match archive.by_name(name) {
        Ok(entry) => entry,
        Err(zip::result::ZipError::FileNotFound) => return None,
        Err(e) => return Some(Err(std::io::Error::other(e.to_string()))),
    };
    let mut buf = Vec::with_capacity(entry.size() as usize);
    Some(entry.read_to_end(&mut buf).map(|_| buf))
}

fn json_num(v: &serde_json::Value, path: &[&str]) -> Option<f64> {
    let mut cur = v;
    for &seg in path {
        cur = cur.get(seg)?;
    }
    cur.as_f64()
}

/// Reads a project archive in **either** layout (`SAVEFILE_COMPAT.md` §1:
/// readers accept both, writers produce only the tree).
///
/// The layout test is the presence of `project.json` and nothing else (§4).
pub fn read_project<R: Read + Seek>(reader: R) -> Result<ProjectData, LoadError> {
    let mut archive = zip::ZipArchive::new(reader)?;
    match read_entry_bytes(&mut archive, PROJECT_MANIFEST) {
        Some(bytes) => read_tree(&mut archive, bytes?),
        None => read_flat(&mut archive),
    }
}

/// One document's JSON text, read straight out of an archive **without
/// decoding the world it describes**.
///
/// The whole-archive path is [`read_project`], and it is the right one when
/// the caller is opening the project. This exists for the other question —
/// *"what does that file on disk say about X?"* — where paying for six
/// raster decompressions and every recorded year to reach one small JSON
/// document would be the only cost of asking.
///
/// The text is verbatim, on the same terms as [`ProjectData::document_text`]:
/// the archive's own bytes, byte-order mark stripped, no reformatting and no
/// §14.2 coercion. It is parsed once and the parse discarded, so a caller
/// can rely on the returned text being valid JSON without this crate
/// pretending to know its schema.
///
/// `Ok(None)` means *the archive does not carry that document*, and covers
/// three cases a caller has no reason to tell apart: the entry is absent,
/// the archive is the flat layout (§15 — it carries no documents at all), or
/// `slot` is not one of [`DOCUMENT_SLOTS`]. That last one is deliberate
/// rather than an error: an unregistered name must not be able to pull an
/// arbitrary entry out of the archive, and a caller that wants to tell a
/// typo from an absent document should check the name against
/// [`DOCUMENT_SLOTS`] itself before calling.
pub fn read_document<R: Read + Seek>(
    reader: R,
    slot: &str,
) -> Result<Option<String>, LoadError> {
    if !DOCUMENT_SLOTS.contains(&slot) {
        return Ok(None);
    }
    let mut archive = zip::ZipArchive::new(reader)?;

    // The manifest is checked for the same reason `read_project` checks it:
    // §4 makes its presence the layout test, and an entry called
    // `entities/journeys.json` inside an unrelated zip is not this format's
    // journeys document. No manifest at all is the flat layout, which
    // carries no documents -- absent, not an error.
    let Some(manifest_bytes) = read_entry_bytes(&mut archive, PROJECT_MANIFEST) else {
        return Ok(None);
    };
    let manifest: serde_json::Value =
        serde_json::from_slice(strip_bom(&manifest_bytes.map_err(LoadError::Io)?))
            .map_err(LoadError::Json)?;
    match manifest.get("format").and_then(|v| v.as_str()) {
        Some(PROJECT_FORMAT) => {}
        Some(other) => return Err(LoadError::NotAProject(other.to_string())),
        None => return Err(LoadError::NotAProject(String::new())),
    }

    let Some(bytes) = read_entry_bytes(&mut archive, slot) else {
        return Ok(None);
    };
    let bytes = bytes.map_err(LoadError::Io)?;
    let text = String::from_utf8(strip_bom(&bytes).to_vec())
        .map_err(|e| LoadError::Io(std::io::Error::other(e.to_string())))?;
    serde_json::from_str::<serde_json::Value>(&text).map_err(LoadError::Json)?;
    Ok(Some(text))
}

fn read_tree(
    archive: &mut zip::ZipArchive<impl Read + Seek>,
    manifest_bytes: Vec<u8>,
) -> Result<ProjectData, LoadError> {
    let mut warnings: Vec<String> = Vec::new();

    let mut manifest: serde_json::Value =
        serde_json::from_slice(strip_bom(&manifest_bytes)).map_err(LoadError::Json)?;
    coerce_integral_floats(&mut manifest);

    match manifest.get("format").and_then(|v| v.as_str()) {
        Some(PROJECT_FORMAT) => {}
        Some(other) => return Err(LoadError::NotAProject(other.to_string())),
        // A `project.json` with no `format` at all is refused rather than
        // assumed: §4 makes this entry's presence the layout test, so an
        // unrelated file called `project.json` must not be read as a world.
        None => return Err(LoadError::NotAProject(String::new())),
    }
    let format_version = manifest
        .get("format_version")
        .and_then(|v| v.as_i64())
        .ok_or(LoadError::MissingField("format_version"))?;
    if format_version > PROJECT_FORMAT_VERSION {
        // Read it anyway (§4): the unknown-member rule is what makes a
        // newer archive partially legible, and refusing outright would
        // lose more than it protects.
        warnings.push(format!(
            "project.json says format_version {format_version}; this build knows {PROJECT_FORMAT_VERSION}. Parts of the archive may not have been understood."
        ));
    }

    let gw = json_num(&manifest, &["world", "grid_width"])
        .ok_or(LoadError::MissingField("world.grid_width"))? as usize;
    let gh = json_num(&manifest, &["world", "grid_height"])
        .ok_or(LoadError::MissingField("world.grid_height"))? as usize;
    // `as i32` here SATURATED silently: a conforming archive whose seed is
    // above 2^31 loaded with a different seed, and the only symptom was that
    // pressing Generate afterwards produced a world other than the one on
    // screen. Checked and refused instead -- see `LoadError::OutOfRange`.
    let seed_f = json_num(&manifest, &["world", "seed"]).ok_or(LoadError::MissingField("world.seed"))?;
    if !seed_f.is_finite() || seed_f < i32::MIN as f64 || seed_f > i32::MAX as f64 {
        return Err(LoadError::OutOfRange { field: "world.seed", value: seed_f });
    }
    let seed = seed_f as i32;
    let map_width_km = json_num(&manifest, &["world", "map_width_km"])
        .ok_or(LoadError::MissingField("world.map_width_km"))?;
    let sea_level = json_num(&manifest, &["world", "sea_level"])
        .ok_or(LoadError::MissingField("world.sea_level"))?;
    let world = manifest
        .get("world")
        .and_then(|w| w.get("wrap_x"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    // MAY, and absent in every archive written before the member existed.
    // Kept verbatim — an origin this build does not recognise is a value
    // from a newer writer, and §4's unknown-member rule says carry it, not
    // flatten it into one of the three this build knows.
    let origin = manifest
        .get("world")
        .and_then(|w| w.get("origin"))
        .and_then(|v| v.as_str())
        .map(str::to_string);
    // Same MAY, verbatim-or-absent shape as `origin` immediately above.
    let name = manifest
        .get("world")
        .and_then(|w| w.get("name"))
        .and_then(|v| v.as_str())
        .map(str::to_string);
    if gw == 0 || gh == 0 {
        return Err(LoadError::MissingField("world.grid_width"));
    }
    let n = gw * gh;

    // --- params.json: two views, either or both absent -------------------
    let mut cartalith = serde_json::Value::Null;
    let mut reference = serde_json::Value::Null;
    if let Some(bytes) = read_entry_bytes(archive, "params.json") {
        match bytes.map_err(LoadError::Io).and_then(|b| {
            serde_json::from_slice::<serde_json::Value>(strip_bom(&b)).map_err(LoadError::Json)
        }) {
            Ok(mut params) => {
                coerce_integral_floats(&mut params);
                cartalith = params
                    .get("cartalith")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                reference = params
                    .get("reference")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
            }
            // §6.4: only `project.json` and `heightmap` are fatal.
            Err(e) => warnings.push(format!("params.json skipped: {e}")),
        }
    }
    // `SaveData::state` keeps the shape every existing caller reads: the
    // reference-named object with the port's own dotted block nested inside
    // it under `cartalith`, exactly as the flat layout carried it. The tree
    // splits the two for legibility; this rejoins them so no consumer of
    // `SaveData` had to change.
    let mut state = match reference {
        serde_json::Value::Object(map) => serde_json::Value::Object(map),
        _ => serde_json::json!({}),
    };
    if !cartalith.is_null() {
        state
            .as_object_mut()
            .expect("just built as an object")
            .insert("cartalith".into(), cartalith);
    }

    // --- rasters ---------------------------------------------------------
    let mut rasters: BTreeMap<String, Raster> = BTreeMap::new();
    for &slot in RASTER_SLOTS {
        // §8.2: the shuffled name first, then the plain one. Which decode
        // applies is decided by **which name was found** and by nothing else
        // -- not by `format_version` -- so a plain entry is never
        // un-shuffled, in an archive of any version.
        let shuffled_name = raster_entry_name(slot);
        let shuffled = shuffled_name != slot.path;
        let (name, found, planar) = match shuffled.then(|| read_entry_bytes(archive, &shuffled_name)).flatten() {
            Some(found) => {
                if archive.index_for_name(slot.path).is_some() {
                    warnings.push(format!(
                        "{} and {shuffled_name} are both present; read {shuffled_name} and ignored the other \
                         (SAVEFILE_COMPAT.md §8.2), which a re-save will not keep",
                        slot.path
                    ));
                }
                (shuffled_name.as_str(), found, true)
            }
            None => match read_entry_bytes(archive, slot.path) {
                Some(found) => (slot.path, found, false),
                None => continue,
            },
        };
        let bytes = match found {
            Ok(b) => b,
            // Present but unreadable. Fatal only for the terrain, exactly as
            // a wrong *length* is (§6.4) -- a heightmap this build cannot
            // decode is not a world, and reporting it as "missing" would
            // blame the wrong thing.
            Err(e) if slot.path == CORE_RASTERS[0] => return Err(LoadError::Io(e)),
            Err(e) => {
                warnings.push(format!("{name}: skipped ({e})"));
                continue;
            }
        };
        let expected = n * slot.element.size();
        if bytes.len() != expected {
            let message = format!(
                "{name}: expected {expected} bytes for this grid, got {}",
                bytes.len()
            );
            if slot.path == CORE_RASTERS[0] {
                return Err(LoadError::RasterLength(message));
            }
            warnings.push(format!("{message} -- skipped"));
            continue;
        }
        let raster = if planar {
            Raster::from_planes(slot.element, &bytes)
        } else {
            Raster::from_bytes(slot.element, &bytes)
        };
        rasters.insert(slot.path.to_string(), raster);
    }

    let heightmap = match rasters.remove(CORE_RASTERS[0]) {
        Some(Raster::F32(v)) => v,
        // §6.4: the one raster whose absence is fatal. Everything else has
        // an honest substitute; a project with no terrain has nothing.
        _ => return Err(LoadError::MissingEntry("rasters/heightmap.f32")),
    };
    // Which core rasters were substituted rather than read -- see
    // [`ProjectData::core_substituted`] for why a caller needs the list.
    let mut core_substituted: Vec<String> = Vec::new();
    let mut take_f32 = |path: &str, honest_zero: bool| -> Vec<f32> {
        match rasters.remove(path) {
            Some(Raster::F32(v)) => v,
            _ => {
                core_substituted.push(path.to_string());
                if !honest_zero {
                    // Zero is a *lie* for temperature and rainfall (§8.1),
                    // so the substitution is reported rather than assumed.
                    warnings.push(format!(
                        "{path}: absent -- this project carries no such field; zero-filled"
                    ));
                }
                vec![0.0; n]
            }
        }
    };
    let temperature = take_f32(CORE_RASTERS[1], false);
    let rainfall = take_f32(CORE_RASTERS[2], false);
    let volcanic_field = take_f32(CORE_RASTERS[3], true);
    let impact_field = take_f32(CORE_RASTERS[4], true);
    let strahler_order = match rasters.remove(CORE_RASTERS[5]) {
        Some(Raster::U8(v)) => v,
        _ => {
            core_substituted.push(CORE_RASTERS[5].to_string());
            vec![0u8; n]
        }
    };

    // --- documents -------------------------------------------------------
    // Each document is kept twice: parsed and coerced for the schemas that
    // consume it, and verbatim for the slots no schema here models. The two
    // maps are populated together so they can never disagree about which
    // documents an archive carried.
    let mut documents: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    let mut document_text: BTreeMap<String, String> = BTreeMap::new();
    for slot in DOCUMENT_SLOTS {
        let Some(bytes) = read_entry_bytes(archive, slot) else {
            continue;
        };
        match bytes.map_err(|e| e.to_string()).and_then(|b| {
            let text = String::from_utf8(strip_bom(&b).to_vec()).map_err(|e| e.to_string())?;
            let value =
                serde_json::from_str::<serde_json::Value>(&text).map_err(|e| e.to_string())?;
            Ok((text, value))
        }) {
            Ok((text, mut v)) => {
                coerce_integral_floats(&mut v);
                documents.insert((*slot).to_string(), v);
                document_text.insert((*slot).to_string(), text);
            }
            Err(e) => warnings.push(format!("{slot}: skipped ({e})")),
        }
    }

    // --- history/territory/<year>.i32, and the foreign-entry census ------
    // Enumerated rather than looked up, because the year is part of the
    // name. Names are collected first: `by_name` borrows the archive.
    let all_names: Vec<String> = (0..archive.len())
        .filter_map(|i| archive.by_index_raw(i).ok().map(|e| e.name().to_string()))
        .collect();
    let history_names: Vec<String> = all_names
        .iter()
        .filter(|name| name.starts_with(HISTORY_TERRITORY_PREFIX) && name.ends_with(".i32"))
        .cloned()
        .collect();
    // A directory entry is not a payload (§3), and neither is anything
    // `is_own_entry` names. The **bytes** are kept, not just the names: see
    // [`ProjectData::foreign`] for why the census alone could not satisfy
    // §6.2. A foreign entry whose bytes cannot be decompressed is dropped
    // with a warning rather than failing the archive (§6.4) -- it is one
    // payload lost, and losing the world with it would be worse.
    let mut foreign: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for name in all_names
        .iter()
        .filter(|name| !name.ends_with('/') && !is_own_entry(name))
    {
        match read_entry_bytes(archive, name) {
            Some(Ok(bytes)) => {
                foreign.insert(name.clone(), bytes);
            }
            Some(Err(e)) => warnings.push(format!("{name}: unreadable, not carried ({e})")),
            None => {}
        }
    }
    let mut history_territory: BTreeMap<i64, Vec<i32>> = BTreeMap::new();
    for name in history_names {
        let stem = &name[HISTORY_TERRITORY_PREFIX.len()..name.len() - 4];
        let Ok(year) = stem.parse::<i64>() else {
            warnings.push(format!("{name}: skipped (not a year)"));
            continue;
        };
        let Some(bytes) = read_entry_bytes(archive, &name) else {
            continue;
        };
        let bytes = match bytes {
            Ok(b) => b,
            Err(e) => {
                warnings.push(format!("{name}: skipped ({e})"));
                continue;
            }
        };
        if bytes.len() != n * 4 {
            warnings.push(format!(
                "{name}: expected {} bytes for this grid, got {} -- skipped",
                n * 4,
                bytes.len()
            ));
            continue;
        }
        match Raster::from_bytes(Element::I32, &bytes) {
            Raster::I32(v) => {
                history_territory.insert(year, v);
            }
            _ => unreachable!("from_bytes(I32) returns I32"),
        }
    }

    let params = SaveParams { gw, gh, seed, map_width_km, sea_level, world, origin, name };

    // --- cartography/tiles/ — the optional pyramid, checked before it is
    // handed back -------------------------------------------------------
    //
    // Absent is the normal case and is not a warning. Present-but-stale is a
    // warning and costs the tiles, never the world: ruling 28's "prefer
    // dropping them to drawing them", implemented where a caller cannot skip
    // it rather than documented where a caller has to remember it.
    let lod_tiles = read_entry_bytes(archive, LOD_TILE_INDEX).and_then(|bytes| {
        let index = match bytes.map_err(|e| e.to_string()).and_then(|b| {
            serde_json::from_slice::<serde_json::Value>(strip_bom(&b)).map_err(|e| e.to_string())
        }) {
            Ok(v) => v,
            Err(e) => {
                warnings.push(format!("{LOD_TILE_INDEX}: skipped ({e}) -- stored LOD tiles dropped"));
                return None;
            }
        };
        let stored_key = index.get("source_key").and_then(|v| v.as_str()).unwrap_or("");
        let live_key = lod_source_key(&params, &heightmap);
        if stored_key != live_key {
            warnings.push(format!(
                "{LOD_TILE_PREFIX}: the stored tiles were made from a different world \
                 (key {stored_key}, this one is {live_key}) -- dropped rather than drawn"
            ));
            return None;
        }
        let dim = |k: &str| index.get(k).and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let (tile_w, tile_h) = (dim("tile_w"), dim("tile_h"));
        let per_tile = tile_w
            .checked_mul(tile_h)
            .and_then(|px| px.checked_mul(LOD_TILE_CHANNELS))
            .unwrap_or(0);
        if per_tile == 0 {
            warnings.push(format!(
                "{LOD_TILE_INDEX}: no usable tile size ({tile_w}x{tile_h}) -- stored LOD tiles dropped"
            ));
            return None;
        }
        let mut tiles = BTreeMap::new();
        for name in all_names.iter() {
            let Some(id) = lod_tile_id(name) else { continue };
            let bytes = match read_entry_bytes(archive, name) {
                Some(Ok(b)) => b,
                Some(Err(e)) => {
                    warnings.push(format!("{name}: skipped ({e})"));
                    continue;
                }
                None => continue,
            };
            if bytes.len() != per_tile {
                warnings.push(format!(
                    "{name}: expected {per_tile} bytes for a {tile_w}x{tile_h} tile, got {} -- skipped",
                    bytes.len()
                ));
                continue;
            }
            tiles.insert(id, bytes);
        }
        Some(LodTiles {
            source_key: live_key,
            producer: index
                .get("producer")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            tile_w,
            tile_h,
            tiles,
        })
    });

    let preview_png = read_entry_bytes(archive, "preview.png").and_then(|r| r.ok());
    let substrate = manifest.get(SUBSTRATE_MEMBER).cloned().unwrap_or(serde_json::Value::Null);

    Ok(ProjectData {
        save: SaveData {
            params,
            fields: SaveFields {
                heightmap: Arc::new(heightmap),
                temperature: Arc::new(temperature),
                rainfall: Arc::new(rainfall),
                volcanic_field,
                impact_field,
                strahler_order,
            },
            state,
        },
        layout: Layout::Tree,
        format_version,
        rasters,
        documents,
        document_text,
        history_territory,
        lod_tiles,
        preview_png,
        foreign,
        substrate,
        core_substituted: Some(core_substituted),
        warnings,
    })
}

fn read_flat(archive: &mut zip::ZipArchive<impl Read + Seek>) -> Result<ProjectData, LoadError> {
    let save = crate::load_from_archive(archive)?;
    Ok(ProjectData {
        save,
        layout: Layout::Flat,
        format_version: 0,
        rasters: BTreeMap::new(),
        documents: BTreeMap::new(),
        document_text: BTreeMap::new(),
        history_territory: BTreeMap::new(),
        // The flat layout has no pyramid and no place to put one -- §15 is
        // read-only and predates the slot entirely.
        lod_tiles: None,
        preview_png: None,
        // The flat layout has always carried entries no reader wanted (a
        // baked atlas, `map.png`, a README) and §6.3 has always said to
        // ignore them. **Deliberately still empty**, and this is the one
        // place §6.2 is knowingly not met: a flat archive re-saved comes out
        // as a *tree*, so carrying its entries would mean mixing one
        // layout's payloads into the other's namespace, where a reader of
        // either would not know what to do with them. The shell's own answer
        // is the honest one -- `app.gd` tells the person the file was read as
        // the older format and that saving converts it -- and the
        // conversion, not the census, is what a user has to decide about.
        foreign: BTreeMap::new(),
        // §15 predates the substrate (§8.3) and has no manifest to carry it.
        substrate: serde_json::Value::Null,
        core_substituted: None,
        // Not a warning: a flat archive carrying no project layer is the
        // format working as specified (`SAVEFILE_COMPAT.md` §15), not
        // something the reader failed at.
        warnings: Vec::new(),
    })
}

/// A UTF-8 BOM is not part of the JSON and `serde_json` will not skip it
/// (`SAVEFILE_COMPAT.md` §14). Editors on Windows add one; a reader that
/// choked on it would blame the wrong thing.
fn strip_bom(bytes: &[u8]) -> &[u8] {
    bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_spatial::pyramid::ChunkId;
    use std::io::Cursor;

    fn sample(gw: usize, gh: usize) -> (SaveParams, SaveFields) {
        let n = gw * gh;
        let params = SaveParams {
            gw,
            gh,
            seed: 4242,
            map_width_km: 1234.5,
            sea_level: 0.37,
            world: true,
            // `None` deliberately: every test built on this helper then
            // exercises the pre-provenance shape, which is the one an
            // existing archive on a user's disk has.
            origin: None,
            // Same reasoning as `origin` immediately above, and the same
            // pre-existence shape: an archive written before `world.name`
            // existed.
            name: None,
        };
        let fields = SaveFields {
            heightmap: Arc::new((0..n).map(|i| i as f32 * 0.25).collect()),
            temperature: Arc::new((0..n).map(|i| 30.0 - i as f32 * 0.5).collect()),
            rainfall: Arc::new((0..n).map(|i| (i % 13) as f32 * 0.125).collect()),
            volcanic_field: (0..n).map(|i| (i % 5) as f32 * 0.5).collect(),
            impact_field: (0..n).map(|i| (i % 3) as f32 * 0.75).collect(),
            strahler_order: (0..n).map(|i| (i % 251) as u8).collect(),
        };
        (params, fields)
    }

    fn write_to_vec(project: &ProjectWrite<'_>) -> Vec<u8> {
        let mut buf = Vec::new();
        write_project(Cursor::new(&mut buf), project).expect("write_project should succeed");
        buf
    }

    /// Everything about an archive the format promises is deterministic,
    /// with the one part it promises is **not** deliberately left out.
    ///
    /// `SAVEFILE_COMPAT.md` §16 records that this writer populates every zip
    /// entry's timestamp **from the clock** rather than leaving the 1980 DOS
    /// epoch, and treats that stamp as one of the two mechanisms answering
    /// *"when was this saved"* -- it even quotes a measured example. So two
    /// saves of one project are not byte-identical and were never meant to
    /// be, and the two determinism tests that compared raw bytes were
    /// **flaky by construction**: DOS time has two-second resolution, so a
    /// pair of writes that straddles a tick differs at the local header's
    /// mod-time field, byte 10. It stood because a write takes microseconds,
    /// which makes the window small rather than absent. It was caught in the
    /// wild on 2026-09-07, where it also cost more than one test: the
    /// failure fail-fasts the workspace run, skipping 57 targets, and the
    /// truncated total (100 result lines, 2 387 passed) reads exactly like a
    /// healthy one.
    ///
    /// **This is not a new decision, it is an existing one applied one level
    /// down.** `a_project_written_twice_has_identical_content` already
    /// excludes JSON's `created` on the stated grounds that *"a timestamp is
    /// the one member that must differ between two saves"*. The container's
    /// own stamp is the same member wearing the container's clothes.
    ///
    /// What remains compared is entry **order**, **name**, **CRC-32** and
    /// both sizes.
    ///
    /// **What that does and does not buy, measured rather than asserted.**
    /// Mutating the CRC out of the tuple leaves both tests green -- and so
    /// would mutating any other field out. That is not a hole opened here:
    /// a test that compares two runs of one writer can only catch a writer
    /// that is *unstable between runs*, never one that is *stably wrong*,
    /// and the raw-byte comparison this replaces had exactly the same blind
    /// spot. Correctness of the ordering is pinned separately and directly,
    /// by `a_stored_pyramid_writes_in_a_stable_order`'s own
    /// `assert_eq!(tiles, sorted)` against a re-sorted copy. Say what this
    /// helper covers -- run-to-run stability -- and not a word more.
    /// The non-emptiness guard is not decoration. Mutated to return
    /// `Vec::new()`, this helper leaves **both** determinism tests green:
    /// `assert_eq!` over two empty vectors is a tautology, and neither test
    /// looks at anything else. That is the silently-empty-golden-output trap
    /// this repository has been bitten by in four subsystems, so the shape
    /// is asserted here, once, where both callers get it.
    fn content_fingerprint(buf: &[u8]) -> Vec<(String, u32, u64, u64)> {
        let mut r = zip::ZipArchive::new(Cursor::new(buf)).expect("the writer produces a zip");
        let out: Vec<(String, u32, u64, u64)> = (0..r.len())
            .map(|i| {
                let e = r.by_index_raw(i).expect("every index is readable");
                (e.name().to_string(), e.crc32(), e.size(), e.compressed_size())
            })
            .collect();
        assert!(
            !out.is_empty(),
            "an archive this writer produced always carries entries -- an empty \
             fingerprint would make both determinism tests pass vacuously"
        );
        assert!(
            out.iter().any(|(n, ..)| n == "project.json"),
            "project.json is written unconditionally; its absence means the \
             fingerprint is reading something other than a project archive"
        );
        out
    }

    #[test]
    fn an_empty_project_round_trips() {
        let (params, fields) = sample(7, 5);
        let buf = write_to_vec(&ProjectWrite::new(&params, &fields));
        let back = read_project(Cursor::new(&buf)).expect("read_project should succeed");

        assert_eq!(back.layout, Layout::Tree);
        assert_eq!(back.format_version, PROJECT_FORMAT_VERSION);
        assert_eq!(back.save.params, params);
        assert_eq!(back.save.fields, fields);
        assert!(back.documents.is_empty());
        assert!(back.rasters.is_empty());
        assert!(back.history_territory.is_empty());
        assert!(
            back.warnings.is_empty(),
            "an empty project is not a damaged one: {:?}",
            back.warnings
        );
    }

    #[test]
    fn every_payload_kind_round_trips() {
        let (params, fields) = sample(6, 4);
        let n = 24;
        let mut p = ProjectWrite::new(&params, &fields);
        p.cartalith_params = serde_json::json!({ "tect.seed": 4242, "use_gpu": false });
        p.reference_params =
            serde_json::json!({ "tect": { "seed": 4242, "plates": 9 }, "seaLevel": 0.37 });
        p.raster(
            "rasters/territory.i32",
            Raster::I32((0..n).map(|i| (i % 4) as i32).collect()),
        );
        p.raster(
            "rasters/water_bodies.u8",
            Raster::U8((0..n).map(|i| (i % 3) as u8).collect()),
        );
        p.raster(
            "rasters/agrarian_density.f32",
            Raster::F32((0..n).map(|i| i as f32 * 0.5).collect()),
        );
        p.document(
            "entities/settlements.json",
            r#"{"next_id":3,"settlements":[{"id":1,"x":2,"y":3}]}"#,
        );
        p.document(
            "annotations/labels.json",
            r#"{"labels":[{"x":1.5,"y":2.5,"name":"Here"}]}"#,
        );
        p.document("vault.json", r#"{"version":1,"links":[]}"#);
        p.history_territory
            .insert(0, (0..n).map(|i| i as i32).collect());
        p.history_territory.insert(-120, vec![7i32; n]);
        p.preview_png = Some(b"\x89PNG not really".to_vec());
        p.readme = Some(DEFAULT_README.to_string());
        p.created = Some("2026-08-25T00:00:00Z".to_string());

        let buf = write_to_vec(&p);
        let back = read_project(Cursor::new(&buf)).expect("read_project should succeed");

        assert!(back.warnings.is_empty(), "{:?}", back.warnings);
        assert_eq!(back.save.params, params);
        assert_eq!(back.save.fields, fields);
        assert_eq!(back.rasters.len(), 3);
        assert_eq!(
            back.raster("rasters/territory.i32"),
            Some(&Raster::I32((0..n).map(|i| (i % 4) as i32).collect()))
        );
        assert_eq!(
            back.raster("rasters/water_bodies.u8"),
            Some(&Raster::U8((0..n).map(|i| (i % 3) as u8).collect()))
        );
        assert_eq!(
            back.document("entities/settlements.json").unwrap()["next_id"],
            3
        );
        assert_eq!(
            back.document("annotations/labels.json").unwrap()["labels"][0]["name"],
            "Here"
        );
        assert!(back.document("vault.json").is_some());
        assert_eq!(back.history_territory.len(), 2);
        assert_eq!(back.history_territory[&-120], vec![7i32; n]);
        assert_eq!(
            back.history_territory[&0],
            (0..n).map(|i| i as i32).collect::<Vec<_>>()
        );
        assert_eq!(
            back.preview_png.as_deref(),
            Some(&b"\x89PNG not really"[..])
        );
        // Both parameter views survive, rejoined into the one `state`
        // shape every existing consumer of `SaveData` reads.
        assert_eq!(back.save.state["tect"]["plates"], 9);
        assert_eq!(back.save.state["cartalith"]["tect.seed"], 4242);
    }

    /// Backward compatibility for `preview_png: None` (`ProjectWrite::new`'s
    /// default, and what every call site got before the Godot bridge learned
    /// to populate it): the archive must carry no `preview.png` entry at
    /// all, not an empty one -- SAVEFILE_COMPAT.md's damage ladder treats a
    /// present-but-empty thumbnail differently from an absent one.
    #[test]
    fn no_preview_is_the_default_and_writes_no_entry() {
        let (params, fields) = sample(4, 4);
        let buf = write_to_vec(&ProjectWrite::new(&params, &fields));
        let mut r = zip::ZipArchive::new(Cursor::new(&buf)).unwrap();
        let names: Vec<String> = (0..r.len())
            .map(|i| r.by_index_raw(i).unwrap().name().to_string())
            .collect();
        assert!(
            !names.iter().any(|n| n == "preview.png"),
            "no preview_png must write no entry: {names:?}"
        );
        let back = read_project(Cursor::new(&buf)).expect("the archive reads");
        assert!(back.preview_png.is_none());
        assert!(back.warnings.is_empty(), "{:?}", back.warnings);
    }

    /// The landmark slot end to end at *this* crate's boundary: registered,
    /// writable, and read back as a document rather than counted a foreign
    /// entry. `cartalith-godot` owns the payload's shape and pins that
    /// separately; what is asserted here is the gate it depends on, since
    /// un-registering the slot would make `write_project` refuse a landmark
    /// document outright and the failure would surface over there instead.
    #[test]
    fn the_landmarks_slot_is_open_and_round_trips() {
        assert!(
            DOCUMENT_SLOTS.contains(&"entities/landmarks.json"),
            "the landmarks slot must stay registered; unregistering it makes              write_project reject a landmark document outright"
        );

        let (params, fields) = sample(6, 4);
        let mut p = ProjectWrite::new(&params, &fields);
        p.document(
            "entities/landmarks.json",
            r#"{"settings":{"crowding":1.25},"sites":[{"kind":"shrine","x":2,"y":3}]}"#,
        );

        let buf = write_to_vec(&p);
        let back = read_project(Cursor::new(&buf)).expect("read_project should succeed");

        assert!(back.warnings.is_empty(), "{:?}", back.warnings);
        let doc = back
            .document("entities/landmarks.json")
            .expect("a registered slot must come back as a document");
        assert_eq!(doc["settings"]["crowding"], 1.25);
        assert_eq!(doc["sites"][0]["kind"], "shrine");
        assert!(
            !back.foreign.contains_key("entities/landmarks.json"),
            "a registered slot must never be reported as a foreign entry"
        );
    }

    #[test]
    fn a_world_with_no_civ_layer_round_trips() {
        // The case the register calls out explicitly: a generated world
        // nobody has edited must save and reload without inventing an
        // empty settlement list, an empty faction roster or a warning.
        let (params, fields) = sample(5, 5);
        let buf = write_to_vec(&ProjectWrite::new(&params, &fields));
        let back = read_project(Cursor::new(&buf)).unwrap();
        assert!(back.document("entities/settlements.json").is_none());
        assert!(back.document("entities/factions.json").is_none());
        assert!(back.raster("rasters/territory.i32").is_none());
        assert!(back.warnings.is_empty());
        assert_eq!(back.save.fields, fields);
    }

    #[test]
    fn a_flat_legacy_archive_still_reads() {
        // Written with the interoperability writer, read with the project
        // reader -- `SAVEFILE_COMPAT.md` §1's "readers accept both".
        let (params, fields) = sample(9, 4);
        let mut buf = Vec::new();
        crate::write_save(
            Cursor::new(&mut buf),
            &crate::SaveWrite {
                params: &params,
                state: serde_json::json!({ "tect": { "plates": 9 } }),
                fields: &fields,
            },
        )
        .unwrap();

        let back = read_project(Cursor::new(&buf)).expect("a flat archive must read");
        assert_eq!(back.layout, Layout::Flat);
        assert_eq!(back.format_version, 0);
        assert_eq!(back.save.params, params);
        assert_eq!(back.save.fields, fields);
        assert_eq!(back.save.state["tect"]["plates"], 9);
        assert!(back.documents.is_empty());
    }

    #[test]
    fn an_unknown_entry_is_ignored_not_an_error() {
        // §6.3, the rule that lets two implementations add payloads without
        // breaking each other. Tested on the tree layout; `lib.rs` already
        // tests it on the flat one.
        let (params, fields) = sample(4, 4);
        let buf = write_to_vec(&ProjectWrite::new(&params, &fields));

        let mut with_extra = Vec::new();
        {
            let mut r = zip::ZipArchive::new(Cursor::new(&buf)).unwrap();
            let mut w = zip::ZipWriter::new(Cursor::new(&mut with_extra));
            for i in 0..r.len() {
                w.raw_copy_file(r.by_index_raw(i).unwrap()).unwrap();
            }
            let opts = zip_opts();
            w.start_file("cartography/tiles/0/0/0.png", opts).unwrap();
            w.write_all(b"a payload from some future version").unwrap();
            w.start_file("entities/dragons.json", opts).unwrap();
            w.write_all(br#"{"dragons":[{"id":1}]}"#).unwrap();
            w.start_file("map.png", opts).unwrap();
            w.write_all(b"the legacy thumbnail name").unwrap();
            w.finish().unwrap();
        }

        let back = read_project(Cursor::new(&with_extra))
            .expect("unknown entries must not fail the archive");
        assert_eq!(back.save.fields, fields);
        assert!(
            back.warnings.is_empty(),
            "an unknown entry is normal, not a warning: {:?}",
            back.warnings
        );
        // ...but it is *censused*, so a caller can name what it holds (§6.2).
        assert_eq!(
            back.foreign.keys().cloned().collect::<Vec<String>>(),
            vec![
                "cartography/tiles/0/0/0.png".to_string(),
                "entities/dragons.json".to_string(),
                "map.png".to_string(),
            ]
        );
        // ...and *retained*, which is the half a name list could not do.
        assert_eq!(
            back.foreign["entities/dragons.json"],
            br#"{"dragons":[{"id":1}]}"#.to_vec()
        );

        // A project this build wrote itself has nothing foreign in it --
        // otherwise the census would cry wolf on every save.
        let clean = read_project(Cursor::new(&buf)).unwrap();
        assert!(
            clean.foreign.is_empty(),
            "{:?}",
            clean.foreign.keys().collect::<Vec<_>>()
        );
    }

    /// Every branch of [`is_own_entry`], and a near miss for each.
    ///
    /// The round-trip test below exercises this function through a fixture,
    /// and a fixture reaches whichever branches its entries happen to name.
    /// A verifier measured that: the fixture covered **four** of the seven,
    /// and mutating any of the other three -- `"params.json"`, `"README.md"`,
    /// `"preview.png"` -- left the entire workspace suite green. An entry this
    /// build owns but does not recognise would be carried as foreign and then
    /// written back *beside* the real one, so a silent branch here duplicates
    /// data rather than losing it, which is the harder failure to notice.
    ///
    /// Table-driven and asserting literals, not the constants: comparing
    /// `PROJECT_MANIFEST` against `PROJECT_MANIFEST` is the self-referential
    /// shape that let `MIN_REGION_WORLD_AXIS` survive its own mutation.
    #[test]
    fn is_own_entry_covers_all_nine_branches_and_no_more() {
        for owned in [
            "project.json",              // PROJECT_MANIFEST
            "params.json",
            "README.md",
            "preview.png",
            "entities/settlements.json", // DOCUMENT_SLOTS
            "drafts/paint.json",         // DOCUMENT_SLOTS, the slot this pass made restore
            "rasters/heightmap.f32",     // raster_slot
            "rasters/heightmap.shuffled.f32", // raster_entry_name, §8.2
            "rasters/territory.shuffled.i32", // the same, for an i32 slot
            "history/territory/1200.i32", // the prefix + extension pair
            "cartography/tiles/index.json", // LOD_TILE_INDEX, ruling 28
            "cartography/tiles/7/106/93.u8", // lod_tile_id
        ] {
            assert!(is_own_entry(owned), "{owned} is this build's own entry");
        }

        for foreign in [
            "project.jsonx",             // near miss on the manifest
            "params.json.bak",
            "README.mdx",
            "preview.pngx",
            "entities/settlements.jsonx",
            "drafts/paint.jsonx",
            "rasters/heightmap.f64",     // registered name, wrong extension
            "rasters/unknown.f32",       // right shape, unregistered slot
            "rasters/unknown.shuffled.f32", // shuffled shape, unregistered slot
            "rasters/water_bodies.shuffled.u8", // a u8 slot has no shuffled name
            "rasters/heightmap.shuffled.i32", // registered stem, wrong element
            "history/territory/1200.json", // right prefix, wrong extension
            "history/territory.i32",     // right extension, not under the prefix
            "vendor/notes.txt",          // plainly someone else's
            // The `cartography/tiles/` prefix is shared, not claimed, and
            // §6.2's own round-trip fixture is exactly the first of these.
            "cartography/tiles/0/0/0.png", // right prefix, another encoding
            "cartography/tiles/index.jsonx",
            "cartography/tiles/2/1.u8",   // too few address parts
            "cartography/tiles/2/1/0/0.u8", // too many
            "cartography/tiles/00/0/0.u8", // a second name for tile 0/0/0
            "cartography/tiles/-1/0/0.u8", // not a level
            "cartography/tiles/2//0.u8",  // an empty part
            "cartography/tiles.u8",       // right extension, not under the prefix
        ] {
            assert!(!is_own_entry(foreign), "{foreign} must be carried as foreign");
        }
    }

    /// §6.2's first option, end to end: an archive from a build that knows
    /// more than this one, opened here and written back, keeps the payloads
    /// this build could not read -- byte for byte, under their own names.
    ///
    /// The whole point of the row: an older build must not be the reason a
    /// newer build's data disappears.
    #[test]
    fn a_foreign_entry_survives_an_open_and_a_re_save() {
        let (params, fields) = sample(4, 4);
        let buf = write_to_vec(&ProjectWrite::new(&params, &fields));

        // Deliberately *not* valid JSON and not valid UTF-8: a carried entry
        // is bytes, and a reader that tried to parse or transcode one would
        // pass a text-only fixture and corrupt a real tile.
        let tile: Vec<u8> = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0xFF, 0x00, 0xC3];
        let mut newer = Vec::new();
        {
            let mut r = zip::ZipArchive::new(Cursor::new(&buf)).unwrap();
            let mut w = zip::ZipWriter::new(Cursor::new(&mut newer));
            for i in 0..r.len() {
                w.raw_copy_file(r.by_index_raw(i).unwrap()).unwrap();
            }
            w.start_file("cartography/tiles/0/0/0.png", zip_opts()).unwrap();
            w.write_all(&tile).unwrap();
            w.finish().unwrap();
        }

        let opened = read_project(Cursor::new(&newer)).expect("a newer archive still opens");
        assert_eq!(opened.foreign["cartography/tiles/0/0/0.png"], tile);

        // The re-save an older build would do: everything from the live
        // model, plus what it was handed and did not understand.
        let mut write = ProjectWrite::new(&params, &fields);
        write.foreign = opened.foreign.clone();
        let again = write_to_vec(&write);

        let back = read_project(Cursor::new(&again)).expect("the re-save opens");
        assert_eq!(
            back.foreign["cartography/tiles/0/0/0.png"], tile,
            "an entry this build cannot read must survive being re-saved by it"
        );
        assert_eq!(back.save.fields, fields, "and the world must be unharmed");
    }

    // -- cartography/tiles/: owner ruling 28's optional stored pyramid ----

    /// **The hole a verifier opened on 2026-09-06, now closed and pinned.**
    ///
    /// `write_project` computes the stored `source_key` from the heightmap it
    /// is writing. On its own that made the *archive* self-consistent and made
    /// stale tiles **undetectable**: hand the writer pre-sculpt tiles beside a
    /// sculpted heightmap and it stamped them with the new world's key, so the
    /// reader — which can only catch an index disagreeing with its own
    /// archive — read them back with an **empty warnings list**.
    ///
    /// A non-empty `source_key` is now a claim, and a false one costs the
    /// pyramid rather than the map. Empty stays trusted: that is the contract a
    /// fresh producer writes against, and `a_pyramid` above relies on it.
    #[test]
    fn stale_tiles_are_dropped_by_the_writer_not_restamped() {
        let (params, fields) = sample(6, 4);
        let mut sculpted = fields.clone();
        Arc::make_mut(&mut sculpted.heightmap)[0] += 1.0; // one cell is a different world

        let stale_key = lod_source_key(&params, &fields.heightmap);
        let live_key = lod_source_key(&params, &sculpted.heightmap);
        assert_ne!(stale_key, live_key, "the fixture must actually be two worlds");

        let mut pyr = a_pyramid(4, 3, 1);
        pyr.source_key = stale_key.clone();

        let mut write = ProjectWrite::new(&params, &sculpted);
        write.lod_tiles = Some(pyr.clone());
        let back = read_project(Cursor::new(&write_to_vec(&write))).expect("the save still opens");

        assert!(
            back.lod_tiles.is_none(),
            "tiles claiming another world must not be written; restamping them is what made              this undetectable on the read side"
        );
        assert_eq!(back.save.fields, sculpted, "and dropping them must not touch the world");

        // The trusted path still works, so the check costs a fresh producer
        // nothing: an empty key means "built for the world you are handing me".
        let mut fresh = ProjectWrite::new(&params, &sculpted);
        fresh.lod_tiles = Some(a_pyramid(4, 3, 1));
        let kept = read_project(Cursor::new(&write_to_vec(&fresh))).expect("opens");
        let kept = kept.lod_tiles.expect("an empty key is trusted and the pyramid is kept");
        assert_eq!(kept.source_key, live_key, "and it is stamped with the world it shipped beside");
    }

    /// **The other half of the 2026-09-06 hole.** Dropping a stale pyramid on
    /// write used to be silent -- `write_project` returned `Result<(),
    /// SaveError>` and had nowhere to put a warning, unlike `read_project`'s
    /// `ProjectData::warnings` (`OUTSTANDING_WORK.md`'s "a dropped pyramid is
    /// silent" row). `write_project` now reports the drop the same way the
    /// read side does.
    #[test]
    fn write_project_reports_a_dropped_pyramid() {
        let (params, fields) = sample(6, 4);
        let mut sculpted = fields.clone();
        Arc::make_mut(&mut sculpted.heightmap)[0] += 1.0; // one cell is a different world

        let stale_key = lod_source_key(&params, &fields.heightmap);
        let mut pyr = a_pyramid(4, 3, 1);
        pyr.source_key = stale_key;

        let mut write = ProjectWrite::new(&params, &sculpted);
        write.lod_tiles = Some(pyr);
        let mut buf = Vec::new();
        let warnings = write_project(Cursor::new(&mut buf), &write)
            .expect("a stale pyramid must not fail the save");

        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(
            warnings[0].contains(LOD_TILE_PREFIX) && warnings[0].to_lowercase().contains("dropped"),
            "{warnings:?}"
        );

        // And the archive really did drop them, same as the read-side test
        // above -- the warning describes what actually happened, not just
        // what almost did.
        let back = read_project(Cursor::new(&buf)).expect("the save still opens");
        assert!(back.lod_tiles.is_none());
    }

    /// The common case must stay quiet, not just the stale case loud: a
    /// matching `source_key`, an empty (trusted) one, and no pyramid at all
    /// all produce zero warnings.
    #[test]
    fn write_project_is_quiet_when_the_pyramid_matches() {
        let (params, fields) = sample(6, 4);

        // Empty key: the fresh-producer contract, trusted outright.
        let mut fresh = ProjectWrite::new(&params, &fields);
        fresh.lod_tiles = Some(a_pyramid(4, 3, 1));
        let mut buf = Vec::new();
        let warnings =
            write_project(Cursor::new(&mut buf), &fresh).expect("a fresh pyramid must save");
        assert!(warnings.is_empty(), "{warnings:?}");

        // Explicit, correct key: the same world, named rather than implied.
        let mut pyr = a_pyramid(4, 3, 1);
        pyr.source_key = lod_source_key(&params, &fields.heightmap);
        let mut write = ProjectWrite::new(&params, &fields);
        write.lod_tiles = Some(pyr);
        let mut buf2 = Vec::new();
        let warnings =
            write_project(Cursor::new(&mut buf2), &write).expect("a matching pyramid must save");
        assert!(warnings.is_empty(), "{warnings:?}");

        // No pyramid at all is the ordinary case and must stay ordinary.
        let none_write = ProjectWrite::new(&params, &fields);
        let mut buf3 = Vec::new();
        let warnings = write_project(Cursor::new(&mut buf3), &none_write)
            .expect("a save with no pyramid must save");
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    /// **RGB fixture bytes** — `tile_w * tile_h * LOD_TILE_CHANNELS` per
    /// tile, matching what a real producer writes since LOD-D2 (see
    /// `LOD_TILE_EXT`'s own doc comment for the one-channel history this
    /// helper used to fix at).
    fn a_pyramid(tile_w: usize, tile_h: usize, levels: i32) -> LodTiles {
        let mut tiles = BTreeMap::new();
        for z in 0..=levels {
            let n = 1u32 << z;
            for col in 0..n {
                for row in 0..n {
                    // Distinct content per tile, so a test that mixed two
                    // addresses up could not pass on identical bytes.
                    let seed = (z as u8).wrapping_mul(37).wrapping_add(col as u8 * 11 + row as u8);
                    tiles.insert(
                        ChunkId::new(z as u32, col, row),
                        (0..tile_w * tile_h * LOD_TILE_CHANNELS).map(|i| seed.wrapping_add(i as u8)).collect(),
                    );
                }
            }
        }
        LodTiles {
            source_key: String::new(), // the writer computes it
            producer: "lod/rgb/256/2".to_string(),
            tile_w,
            tile_h,
            tiles,
        }
    }

    /// The promotion itself: a pyramid written into the archive comes back
    /// byte for byte, under its own addresses, with its producer intact.
    #[test]
    fn a_stored_pyramid_round_trips() {
        let (params, fields) = sample(8, 6);
        let mut p = ProjectWrite::new(&params, &fields);
        let lod = a_pyramid(4, 3, 2);
        p.lod_tiles = Some(lod.clone());
        let back = read_project(Cursor::new(&write_to_vec(&p))).expect("the archive reads");

        let got = back.lod_tiles.expect("the pyramid survived");
        assert_eq!(got.tiles.len(), 1 + 4 + 16, "every tile of levels 0..=2");
        assert_eq!(got.tiles, lod.tiles, "the bytes must come back unchanged");
        assert_eq!((got.tile_w, got.tile_h), (4, 3));
        assert_eq!(got.producer, "lod/rgb/256/2");
        assert_eq!(got.source_key, lod_source_key(&params, &fields.heightmap));
        assert!(back.warnings.is_empty(), "{:?}", back.warnings);
        // And the tiles are the writer's, not the foreign carrier's.
        assert!(back.foreign.is_empty(), "{:?}", back.foreign.keys().collect::<Vec<_>>());
    }

    /// No pyramid is the default and is not a warning: [`ProjectWrite::new`]
    /// writes none, and an archive without one opens silently.
    #[test]
    fn no_pyramid_is_the_default_and_writes_no_entries() {
        let (params, fields) = sample(4, 4);
        let buf = write_to_vec(&ProjectWrite::new(&params, &fields));
        let mut r = zip::ZipArchive::new(Cursor::new(&buf)).unwrap();
        let names: Vec<String> = (0..r.len())
            .map(|i| r.by_index_raw(i).unwrap().name().to_string())
            .collect();
        assert!(
            !names.iter().any(|n| n.starts_with(LOD_TILE_PREFIX)),
            "an off-by-default slot must write nothing: {names:?}"
        );
        let back = read_project(Cursor::new(&buf)).expect("the archive reads");
        assert!(back.lod_tiles.is_none());
        assert!(back.warnings.is_empty(), "{:?}", back.warnings);
    }

    /// Ruling 28's actual requirement: *"silently drawing a stale tile over a
    /// re-sculpted world is the failure to design against -- prefer dropping
    /// them to drawing them."*
    ///
    /// The fixture is the exact user gesture -- same project, same seed, same
    /// grid, one sculpted cell -- so the *only* thing that can catch it is
    /// the heightmap term of the key.
    #[test]
    fn a_pyramid_from_another_world_is_dropped_not_drawn() {
        let (params, fields) = sample(8, 6);
        let mut p = ProjectWrite::new(&params, &fields);
        p.lod_tiles = Some(a_pyramid(4, 3, 1));
        let buf = write_to_vec(&p);

        // Re-write the same archive with one cell of terrain moved, carrying
        // the tiles across verbatim -- what a build that stored the pyramid
        // and forgot to re-synthesize it would produce.
        let mut sculpted = fields.clone();
        Arc::make_mut(&mut sculpted.heightmap)[17] += 0.25;
        let stale = {
            let mut w = Vec::new();
            let mut r = zip::ZipArchive::new(Cursor::new(&buf)).unwrap();
            let fresh = {
                let mut q = ProjectWrite::new(&params, &sculpted);
                q.lod_tiles = None;
                write_to_vec(&q)
            };
            {
                let mut zw = zip::ZipWriter::new(Cursor::new(&mut w));
                let mut base = zip::ZipArchive::new(Cursor::new(&fresh)).unwrap();
                for i in 0..base.len() {
                    zw.raw_copy_file(base.by_index_raw(i).unwrap()).unwrap();
                }
                for i in 0..r.len() {
                    let e = r.by_index_raw(i).unwrap();
                    if e.name().starts_with(LOD_TILE_PREFIX) {
                        zw.raw_copy_file(e).unwrap();
                    }
                }
                zw.finish().unwrap();
            }
            w
        };

        let back = read_project(Cursor::new(&stale)).expect("the world still opens");
        assert!(
            back.lod_tiles.is_none(),
            "tiles made from a different heightmap must not be handed back"
        );
        assert_eq!(back.save.fields, sculpted, "and the world itself is unharmed");
        assert!(
            back.warnings.iter().any(|w| w.contains("different world")),
            "dropping must be reported, not silent: {:?}",
            back.warnings
        );
    }

    /// The key's inputs, derived from `synthesize_tile_rgba`'s signature and
    /// then exercised **one at a time** -- the preflight rule for a cache
    /// key. A term left out of the hash is invisible until a user's map is
    /// drawn with the wrong relief, and nothing else in the suite would see
    /// it.
    #[test]
    fn every_input_the_synthesizer_reads_moves_the_key() {
        let (params, fields) = sample(6, 5);
        let base = lod_source_key(&params, &fields.heightmap);

        let mut gw = params.clone();
        gw.gw += 1;
        let mut gh = params.clone();
        gh.gh += 1;
        let mut seed = params.clone();
        seed.seed += 1;
        let mut sea = params.clone();
        sea.sea_level += 0.001;
        for (what, p) in [("gw", &gw), ("gh", &gh), ("seed", &seed), ("sea_level", &sea)] {
            assert_ne!(
                lod_source_key(p, &fields.heightmap),
                base,
                "{what} is an input of the tile synthesizer and must move the key"
            );
        }

        // `.as_ref().clone()`, not `.clone()` -- the latter is a cheap `Arc`
        // clone that would still alias `fields.heightmap`'s own storage, so
        // mutating `moved`/`swapped` below would corrupt the fixture `base`
        // was hashed from.
        let mut moved = fields.heightmap.as_ref().clone();
        moved[7] += 0.0001;
        assert_ne!(lod_source_key(&params, &moved), base, "a sculpted cell must move the key");
        // A cell *swapped* with another, not changed in value: an order-blind
        // hash (a sum, an xor of whole words) passes everything above and
        // fails here.
        let mut swapped = fields.heightmap.as_ref().clone();
        swapped.swap(3, 9);
        assert_ne!(swapped, *fields.heightmap, "the fixture must actually differ");
        assert_ne!(lod_source_key(&params, &swapped), base, "the key must depend on cell order");

        // And the two things that are NOT inputs: no argument of
        // `synthesize_tile_rgba` carries them, so hashing them would throw a
        // valid cache away on a metadata edit.
        let mut km = params.clone();
        km.map_width_km += 100.0;
        let mut wrap = params.clone();
        wrap.world = !wrap.world;
        for (what, p) in [("map_width_km", &km), ("wrap_x", &wrap)] {
            assert_eq!(
                lod_source_key(p, &fields.heightmap),
                base,
                "{what} is not an input of the tile synthesizer and must not invalidate a cache"
            );
        }
    }

    /// The length guard the rest of the format's headerless payloads get. A
    /// tile carries no length of its own, so a short one is a truncated
    /// picture rather than a parse error.
    #[test]
    fn a_tile_of_the_wrong_size_is_refused_at_write_time() {
        let (params, fields) = sample(4, 4);
        let mut p = ProjectWrite::new(&params, &fields);
        let mut lod = a_pyramid(4, 3, 0);
        lod.tiles.insert(ChunkId::new(1, 0, 0), vec![0u8; 11]);
        p.lod_tiles = Some(lod);
        let mut sink = Cursor::new(Vec::new());
        match write_project(&mut sink, &p) {
            Err(SaveError::RasterLength { entry, expected, got }) => {
                assert_eq!(entry, "cartography/tiles/1/0/0.u8");
                assert_eq!((expected, got), (36, 11), "4*3*LOD_TILE_CHANNELS(3) = 36");
            }
            other => panic!("a short tile must be refused: {other:?}"),
        }
    }

    /// A tile size of zero would make every tile "the right length" and turn
    /// the guard above into a no-op, so it is refused before the archive is
    /// opened rather than checked per tile.
    #[test]
    fn a_pyramid_with_no_tile_size_is_refused_at_write_time() {
        let (params, fields) = sample(4, 4);
        for (w, h) in [(0, 3), (4, 0), (0, 0)] {
            // The empty map is the case the guard exists for: with tiles
            // present, the per-tile length check would reject a zero size
            // anyway (it fails on the first tile), so a fixture that always
            // carries tiles cannot tell the two apart -- measured, the guard
            // survived its own mutation until this loop covered both.
            for tiles in [true, false] {
                let mut p = ProjectWrite::new(&params, &fields);
                let mut lod = a_pyramid(4, 3, 0);
                lod.tile_w = w;
                lod.tile_h = h;
                if !tiles {
                    lod.tiles.clear();
                }
                p.lod_tiles = Some(lod);
                let mut sink = Cursor::new(Vec::new());
                match write_project(&mut sink, &p) {
                    Err(SaveError::RasterLength { entry, .. }) => {
                        assert_eq!(entry, LOD_TILE_INDEX, "a {w}x{h} size is the index's fault");
                    }
                    other => panic!("a {w}x{h} tile size must be refused: {other:?}"),
                }
            }
        }
    }

    /// An index this reader cannot use costs the tiles and nothing else — and
    /// says so, because the alternative is a pyramid silently half-read.
    #[test]
    fn an_unusable_index_drops_the_tiles_and_reports_it() {
        let (params, fields) = sample(8, 6);
        let mut p = ProjectWrite::new(&params, &fields);
        p.lod_tiles = Some(a_pyramid(4, 3, 1));
        let buf = write_to_vec(&p);

        // The expected warning is asserted per case, not just "something
        // mentioning tiles": a reader that fell through an unparseable index
        // would still drop the tiles, on the *key* check, and a laxer
        // assertion could not tell a real guard from that accident.
        for (what, index, expect) in [
            ("unparseable", "{not json", "index.json: skipped"),
            (
                "no tile size",
                r#"{"source_key":"x","producer":"p"}"#,
                "no usable tile size (0x0)",
            ),
            (
                "zero tile size",
                r#"{"source_key":"x","producer":"p","tile_w":0,"tile_h":3}"#,
                "no usable tile size (0x3)",
            ),
        ] {
            // The key is written by `write_project`, so a fixture that wants
            // to reach the *size* checks has to carry the real one.
            let real = lod_source_key(&params, &fields.heightmap);
            let text = index.replace("\"x\"", &format!("\"{real}\""));
            let mut edited = Vec::new();
            {
                let mut r = zip::ZipArchive::new(Cursor::new(&buf)).unwrap();
                let mut w = zip::ZipWriter::new(Cursor::new(&mut edited));
                for i in 0..r.len() {
                    let e = r.by_index_raw(i).unwrap();
                    if e.name() == LOD_TILE_INDEX {
                        continue;
                    }
                    w.raw_copy_file(e).unwrap();
                }
                w.start_file(LOD_TILE_INDEX, zip_opts()).unwrap();
                w.write_all(text.as_bytes()).unwrap();
                w.finish().unwrap();
            }
            let back = read_project(Cursor::new(&edited)).expect("the world still opens");
            assert!(back.lod_tiles.is_none(), "{what}: the tiles must be dropped");
            assert!(
                back.warnings.iter().any(|w| w.contains(expect)),
                "{what}: expected a warning containing {expect:?}, got {:?}",
                back.warnings
            );
            // And **only** that reason. Every one of these fixtures carries a
            // heightmap the tiles really were made from, so telling the user
            // their world changed would be a wrong cause dressed as a
            // freshly-checked one -- and it is what a reader that dropped the
            // early return would report, since an index it could not parse
            // produces an empty key that matches nothing.
            assert!(
                !back.warnings.iter().any(|w| w.contains("different world")),
                "{what}: a broken index must not be reported as a changed world: {:?}",
                back.warnings
            );
            assert_eq!(back.save.fields, fields, "{what}: and the world is unharmed");
        }
    }

    /// The same guard from the other side: a tile that was truncated *after*
    /// the archive was written costs itself and nothing else (§6.4).
    #[test]
    fn a_truncated_tile_costs_only_itself() {
        let (params, fields) = sample(8, 6);
        let mut p = ProjectWrite::new(&params, &fields);
        p.lod_tiles = Some(a_pyramid(4, 3, 1));
        let buf = write_to_vec(&p);

        let mut damaged = Vec::new();
        {
            let mut r = zip::ZipArchive::new(Cursor::new(&buf)).unwrap();
            let mut w = zip::ZipWriter::new(Cursor::new(&mut damaged));
            for i in 0..r.len() {
                let e = r.by_index_raw(i).unwrap();
                if e.name() == "cartography/tiles/1/0/1.u8" {
                    continue;
                }
                w.raw_copy_file(e).unwrap();
            }
            w.start_file("cartography/tiles/1/0/1.u8", zip_opts()).unwrap();
            w.write_all(&[0u8; 5]).unwrap();
            w.finish().unwrap();
        }

        let back = read_project(Cursor::new(&damaged)).expect("the world still opens");
        let got = back.lod_tiles.expect("the rest of the pyramid survives");
        assert_eq!(got.tiles.len(), 1 + 4 - 1, "only the damaged tile is missing");
        assert!(!got.tiles.contains_key(&ChunkId::new(1, 0, 1)));
        assert!(
            back.warnings.iter().any(|w| w.contains("cartography/tiles/1/0/1.u8")),
            "{:?}",
            back.warnings
        );
    }

    /// The foreign path stays the fallback, which is what makes this a
    /// promotion rather than a land grab: `cartography/tiles/0/0/0.png` --
    /// the entry §6.2's round-trip fixture uses -- is still somebody else's
    /// payload, carried unchanged **beside** a stored pyramid of this build's
    /// own.
    #[test]
    fn a_foreign_tile_under_the_same_prefix_is_still_carried() {
        let (params, fields) = sample(8, 6);
        let mut p = ProjectWrite::new(&params, &fields);
        p.lod_tiles = Some(a_pyramid(4, 3, 1));
        let buf = write_to_vec(&p);

        let alien: Vec<u8> = vec![0x89, b'P', b'N', b'G', 0xFF, 0x00];
        let mut newer = Vec::new();
        {
            let mut r = zip::ZipArchive::new(Cursor::new(&buf)).unwrap();
            let mut w = zip::ZipWriter::new(Cursor::new(&mut newer));
            for i in 0..r.len() {
                w.raw_copy_file(r.by_index_raw(i).unwrap()).unwrap();
            }
            for name in [
                "cartography/tiles/0/0/0.png", // a different encoding
                "cartography/tiles/2/1.u8",    // too few address parts
                "cartography/tiles/00/0/0.u8", // a second name for tile 0/0/0
            ] {
                w.start_file(name, zip_opts()).unwrap();
                w.write_all(&alien).unwrap();
            }
            w.finish().unwrap();
        }

        let back = read_project(Cursor::new(&newer)).expect("the archive opens");
        for name in [
            "cartography/tiles/0/0/0.png",
            "cartography/tiles/2/1.u8",
            "cartography/tiles/00/0/0.u8",
        ] {
            assert_eq!(back.foreign.get(name), Some(&alien), "{name} must be carried, not adopted");
        }
        assert_eq!(back.lod_tiles.as_ref().expect("ours survives too").tiles.len(), 5);

        // And re-saving keeps both halves, with no duplicate name.
        let mut again = ProjectWrite::new(&params, &fields);
        again.foreign = back.foreign.clone();
        again.lod_tiles = back.lod_tiles.clone();
        let round = read_project(Cursor::new(&write_to_vec(&again))).expect("the re-save opens");
        assert_eq!(round.foreign, back.foreign);
        assert_eq!(round.lod_tiles, back.lod_tiles);
    }

    /// The index is the writer's, not a caller's: it carries a key computed
    /// from the heightmap being written, so a hand-supplied one would be a
    /// claim rather than a check.
    #[test]
    fn the_pyramid_index_is_not_a_caller_writable_slot() {
        let (params, fields) = sample(4, 4);
        let mut p = ProjectWrite::new(&params, &fields);
        p.document("cartography/tiles/index.json", "{}");
        let mut sink = Cursor::new(Vec::new());
        assert!(matches!(
            write_project(&mut sink, &p),
            Err(SaveError::UnknownSlot(s)) if s == "cartography/tiles/index.json"
        ));
    }

    /// The determinism rule [`a_project_written_twice_has_identical_content`]
    /// asserts, extended over the slot that adds thousands of entries: a
    /// `BTreeMap` keyed by `ChunkId` iterates in `(z, col, row)` order, so
    /// two saves of one pyramid carry the same content in the same order.
    #[test]
    fn a_stored_pyramid_writes_in_a_stable_order() {
        let (params, fields) = sample(5, 3);
        let mut p = ProjectWrite::new(&params, &fields);
        p.lod_tiles = Some(a_pyramid(3, 2, 2));
        assert_eq!(
            content_fingerprint(&write_to_vec(&p)),
            content_fingerprint(&write_to_vec(&p))
        );

        let buf = write_to_vec(&p);
        let mut r = zip::ZipArchive::new(Cursor::new(&buf)).unwrap();
        let tiles: Vec<String> = (0..r.len())
            .map(|i| r.by_index_raw(i).unwrap().name().to_string())
            .filter(|n| n.ends_with(".u8") && n.starts_with(LOD_TILE_PREFIX))
            .collect();
        let mut sorted = tiles.clone();
        sorted.sort_by_key(|n| lod_tile_id(n).expect("every written tile parses"));
        assert_eq!(tiles, sorted, "tiles must be written in address order");
        assert_eq!(tiles.first().map(String::as_str), Some("cartography/tiles/0/0/0.u8"));
    }

    /// The collision rule in [`ProjectWrite::foreign`]: a name the writer
    /// produces from the live model is written once, from the model.
    ///
    /// The scenario is real rather than hypothetical -- a build opens an
    /// archive, a *later* build registers one of its foreign names as a slot,
    /// and the carried copy is then a stale duplicate. Without the skip the
    /// save does not merely prefer the wrong copy, it **fails**: dropping the
    /// guard turns this test into
    /// `Zip(InvalidArchive("Duplicate filename: entities/settlements.json"))`.
    #[test]
    fn a_carried_entry_never_shadows_one_this_build_writes() {
        let (params, fields) = sample(4, 4);
        let mut write = ProjectWrite::new(&params, &fields);
        write.document("entities/settlements.json", r#"{"settlements":[]}"#);
        // Every category `is_own_entry` covers, not just the document slot:
        // a single-case fixture would pass with three of the four branches
        // deleted.
        //
        // Both of the heightmap's names (§8.2): the shuffled one is what this
        // writer produces, and a carried *plain* copy -- from a version-1
        // archive -- would be a second heightmap beside it.
        for name in [
            "entities/settlements.json",
            "rasters/heightmap.f32",
            "rasters/heightmap.shuffled.f32",
            "project.json",
            "history/territory/1200.i32",
        ] {
            write.foreign.insert(name.to_string(), b"stale".to_vec());
        }
        let buf = write_to_vec(&write);

        let mut r = zip::ZipArchive::new(Cursor::new(&buf)).unwrap();
        let names: Vec<String> = (0..r.len())
            .map(|i| r.by_index_raw(i).unwrap().name().to_string())
            .collect();
        for name in [
            "entities/settlements.json",
            "rasters/heightmap.shuffled.f32",
            "project.json",
        ] {
            assert_eq!(
                names.iter().filter(|n| n.as_str() == name).count(),
                1,
                "{name} must appear exactly once, from the live model"
            );
        }
        assert!(
            !names.iter().any(|n| n == "history/territory/1200.i32"),
            "a carried history raster this project has no year for must not be resurrected"
        );
        assert!(
            !names.iter().any(|n| n == "rasters/heightmap.f32"),
            "a carried plain heightmap must not ride along beside the shuffled one"
        );

        let back = read_project(Cursor::new(&buf)).expect("the archive reads");
        assert_eq!(
            back.text_of("entities/settlements.json"),
            Some(r#"{"settlements":[]}"#),
            "the model's document won, not the carried `stale`"
        );
        assert!(back.foreign.is_empty(), "{:?}", back.foreign.keys().collect::<Vec<_>>());
    }

    #[test]
    fn a_damaged_optional_document_costs_only_itself() {
        // §6.4: a corrupt labels file must not cost the user their world.
        let (params, fields) = sample(4, 4);
        let mut p = ProjectWrite::new(&params, &fields);
        p.document("entities/settlements.json", r#"{"settlements":[]}"#);
        let buf = write_to_vec(&p);

        let mut damaged = Vec::new();
        {
            let mut r = zip::ZipArchive::new(Cursor::new(&buf)).unwrap();
            let mut w = zip::ZipWriter::new(Cursor::new(&mut damaged));
            for i in 0..r.len() {
                w.raw_copy_file(r.by_index_raw(i).unwrap()).unwrap();
            }
            w.start_file("annotations/labels.json", zip_opts()).unwrap();
            w.write_all(b"{ this is not json").unwrap();
            w.finish().unwrap();
        }

        let back =
            read_project(Cursor::new(&damaged)).expect("a bad label file must not fail the load");
        assert_eq!(back.save.fields, fields);
        assert!(back.document("entities/settlements.json").is_some());
        assert!(back.document("annotations/labels.json").is_none());
        assert_eq!(back.warnings.len(), 1);
        assert!(
            back.warnings[0].contains("annotations/labels.json"),
            "{:?}",
            back.warnings
        );
    }

    /// Rewrites one entry's compression-method field, in both the local
    /// header and the central directory. Done by hand because the `zip`
    /// crate refuses to *write* a method it cannot also compress with, so
    /// there is no other way to build an archive this build cannot decode.
    fn set_compression_method(buf: &[u8], target: &str, method: u16) -> Vec<u8> {
        let mut out = buf.to_vec();
        let name = target.as_bytes();
        let mut patched = 0;
        // (signature, name-length offset, name offset, method offset)
        for (sig, nlen_at, name_at, method_at) in [
            (b"PK\x03\x04", 26usize, 30usize, 8usize),
            (b"PK\x01\x02", 28, 46, 10),
        ] {
            for p in 0..out.len().saturating_sub(name_at) {
                if &out[p..p + 4] != sig {
                    continue;
                }
                let nlen = u16::from_le_bytes([out[p + nlen_at], out[p + nlen_at + 1]]) as usize;
                if p + name_at + nlen <= out.len() && &out[p + name_at..p + name_at + nlen] == name
                {
                    out[p + method_at..p + method_at + 2].copy_from_slice(&method.to_le_bytes());
                    patched += 1;
                }
            }
        }
        assert_eq!(
            patched, 2,
            "expected a local header and a central entry for {target}"
        );
        out
    }

    #[test]
    fn an_undecodable_entry_is_reported_and_never_looks_absent() {
        // `SAVEFILE_COMPAT.md` §3.3: an entry compressed with a method the
        // reader has no decoder for is intact and unreadable, which is not
        // the same thing as absent. Reporting it as absent would let the
        // next save drop a real payload in silence -- §6.2's failure mode.
        //
        // Method 1 (Shrink), not 93 (Zstandard): `zip` is pulled in with
        // its default features, so this build *can* decode zstd, bzip2,
        // LZMA, XZ, PPMd and Deflate64 (see §17). The methods it cannot
        // decode are the legacy PKZIP ones, so the test uses one of those.
        let (params, fields) = sample(4, 4);
        let mut p = ProjectWrite::new(&params, &fields);
        p.raster("rasters/territory.i32", Raster::I32(vec![3; 16]));
        let buf = write_to_vec(&p);

        // The entry names are the §8.2 shuffled ones this writer produces;
        // the warning names the entry as stored, which is what a person
        // opening the archive in a zip tool will find.
        let odd = set_compression_method(&buf, "rasters/territory.shuffled.i32", 1);
        let back = read_project(Cursor::new(&odd)).expect("one odd entry must not cost the world");
        assert!(back.raster("rasters/territory.i32").is_none());
        assert_eq!(back.warnings.len(), 1, "{:?}", back.warnings);
        assert!(
            back.warnings[0].contains("rasters/territory.shuffled.i32") && back.warnings[0].contains('1'),
            "the warning must name the entry and the method: {:?}",
            back.warnings
        );

        // The terrain is the one entry whose undecodability is fatal -- and
        // it is reported as unreadable, not as missing, so the user is told
        // what is actually wrong. Undecodable under its shuffled name must
        // NOT fall back to looking for the plain one and report "missing".
        let odd = set_compression_method(&buf, "rasters/heightmap.shuffled.f32", 1);
        let err =
            read_project(Cursor::new(&odd)).expect_err("an undecodable heightmap is not a world");
        assert!(
            matches!(&err, LoadError::Io(e) if e.to_string().contains("compression method")),
            "{err}"
        );
    }

    #[test]
    fn a_missing_heightmap_is_fatal_and_a_missing_climate_is_not() {
        let (params, fields) = sample(4, 4);
        let buf = write_to_vec(&ProjectWrite::new(&params, &fields));

        let rebuild = |without: &str| -> Vec<u8> {
            let mut out = Vec::new();
            {
                let mut r = zip::ZipArchive::new(Cursor::new(&buf)).unwrap();
                let mut w = zip::ZipWriter::new(Cursor::new(&mut out));
                for i in 0..r.len() {
                    let e = r.by_index_raw(i).unwrap();
                    if e.name() == without {
                        continue;
                    }
                    w.raw_copy_file(e).unwrap();
                }
                w.finish().unwrap();
            }
            out
        };

        assert!(matches!(
            read_project(Cursor::new(rebuild("rasters/heightmap.shuffled.f32"))),
            Err(LoadError::MissingEntry("rasters/heightmap.f32"))
        ));

        let back = read_project(Cursor::new(rebuild("rasters/temperature.shuffled.f32")))
            .expect("climate is not fatal");
        assert_eq!(*back.save.fields.temperature, vec![0.0f32; 16]);
        assert_eq!(
            back.warnings.len(),
            1,
            "the zero-fill must be reported, never assumed: {:?}",
            back.warnings
        );

        // Volcanic/impact zero really is the true value, so no warning.
        let back = read_project(Cursor::new(rebuild("rasters/volcanic_field.shuffled.f32"))).unwrap();
        assert_eq!(back.save.fields.volcanic_field, vec![0.0f32; 16]);
        assert!(back.warnings.is_empty(), "{:?}", back.warnings);
    }

    #[test]
    fn a_truncated_raster_is_refused_not_believed() {
        let (params, fields) = sample(4, 4);
        let mut p = ProjectWrite::new(&params, &fields);
        p.raster("rasters/territory.i32", Raster::I32(vec![1; 15]));
        let mut buf = Vec::new();
        let err = write_project(Cursor::new(&mut buf), &p)
            .expect_err("a short raster must not be written");
        assert!(
            matches!(&err, SaveError::RasterLength { entry, expected: 16, got: 15 } if entry == "rasters/territory.i32"),
            "{err}"
        );
    }

    #[test]
    fn a_short_core_field_is_refused() {
        let (params, mut fields) = sample(6, 5);
        Arc::make_mut(&mut fields.rainfall).pop();
        let mut buf = Vec::new();
        let err = write_project(Cursor::new(&mut buf), &ProjectWrite::new(&params, &fields))
            .expect_err("a short field must not be written");
        assert!(matches!(
            err,
            SaveError::FieldLength {
                entry: "rasters/rainfall.f32",
                expected: 30,
                got: 29
            }
        ));
    }

    #[test]
    fn an_unregistered_slot_is_a_write_error() {
        // The registry is what keeps "one concept, one home" a property of
        // the code rather than of good intentions.
        let (params, fields) = sample(3, 3);
        let mut p = ProjectWrite::new(&params, &fields);
        p.document("cartography/tiles.json", "{}");
        let mut buf = Vec::new();
        assert!(matches!(
            write_project(Cursor::new(&mut buf), &p).expect_err("an invented slot must be refused"),
            SaveError::UnknownSlot(s) if s == "cartography/tiles.json"
        ));

        let mut p = ProjectWrite::new(&params, &fields);
        p.raster("rasters/elevation.f32", Raster::F32(vec![0.0; 9]));
        let mut buf = Vec::new();
        assert!(matches!(
            write_project(Cursor::new(&mut buf), &p).expect_err("an invented raster must be refused"),
            SaveError::UnknownSlot(s) if s == "rasters/elevation.f32"
        ));

        // ...and a second copy of a core raster is refused for the same
        // reason: the terrain has one home.
        let mut p = ProjectWrite::new(&params, &fields);
        p.raster("rasters/heightmap.f32", Raster::F32(vec![0.0; 9]));
        let mut buf = Vec::new();
        assert!(matches!(
            write_project(Cursor::new(&mut buf), &p).expect_err("a duplicate terrain must be refused"),
            SaveError::UnknownSlot(s) if s == "rasters/heightmap.f32"
        ));
    }

    /// `SAVEFILE_COMPAT.md` §8.3: the `substrate` member travels verbatim in
    /// `project.json`, is absent when the writer had none, and every
    /// substrate raster is a registered slot the reader keeps (not foreign).
    #[test]
    fn the_substrate_member_and_rasters_round_trip() {
        let (params, fields) = sample(3, 3);
        let mut p = ProjectWrite::new(&params, &fields);
        let mut buf = Vec::new();
        write_project(Cursor::new(&mut buf), &p).unwrap();
        let back = read_project(Cursor::new(&buf)).unwrap();
        assert!(back.substrate.is_null(), "no member written, none read");
        assert_eq!(back.core_substituted.as_deref(), Some(&[][..]), "all six core rasters were read");

        p.substrate = serde_json::json!({"version": 1, "integrated_drainage": true});
        for (i, path) in SUBSTRATE_RASTERS.iter().enumerate() {
            let slot = raster_slot(path).expect("registered");
            p.raster(*path, match slot.element {
                Element::F32 => Raster::F32(vec![i as f32 + 0.5; 9]),
                Element::I32 => Raster::I32(vec![i as i32 - 3; 9]),
                Element::U8 => Raster::U8(vec![i as u8; 9]),
            });
        }
        let mut buf = Vec::new();
        write_project(Cursor::new(&mut buf), &p).unwrap();
        let back = read_project(Cursor::new(&buf)).unwrap();
        assert_eq!(back.substrate, serde_json::json!({"version": 1, "integrated_drainage": true}));
        assert!(back.foreign.is_empty(), "{:?}", back.foreign.keys().collect::<Vec<_>>());
        for path in SUBSTRATE_RASTERS {
            assert_eq!(back.raster(path), p.rasters.get(path), "{path}");
        }
    }

    /// A core raster that was absent is reported, not only zero-filled --
    /// the substrate rebuilds stream order from `strahler_order.u8` and must
    /// tell "no channels" from "no raster".
    #[test]
    fn a_missing_core_raster_is_named_in_core_substituted() {
        let (params, fields) = sample(3, 3);
        let p = ProjectWrite::new(&params, &fields);
        let mut buf = Vec::new();
        write_project(Cursor::new(&mut buf), &p).unwrap();
        let mut src = zip::ZipArchive::new(Cursor::new(&buf)).unwrap();
        let mut out = Vec::new();
        {
            let mut w = zip::ZipWriter::new(Cursor::new(&mut out));
            for i in 0..src.len() {
                let e = src.by_index_raw(i).unwrap();
                if e.name() == "rasters/strahler_order.u8" {
                    continue;
                }
                w.raw_copy_file(e).unwrap();
            }
            w.finish().unwrap();
        }
        let back = read_project(Cursor::new(&out)).unwrap();
        assert_eq!(back.core_substituted, Some(vec!["rasters/strahler_order.u8".to_string()]));
        assert_eq!(back.save.fields.strahler_order, vec![0u8; 9]);
    }

    #[test]
    fn a_raster_of_the_wrong_element_type_is_refused() {
        let (params, fields) = sample(3, 3);
        let mut p = ProjectWrite::new(&params, &fields);
        p.raster("rasters/territory.i32", Raster::F32(vec![0.0; 9]));
        let mut buf = Vec::new();
        assert!(matches!(
            write_project(Cursor::new(&mut buf), &p)
                .expect_err("an f32 in an i32 slot must be refused"),
            SaveError::RasterElement {
                expected: Element::I32,
                got: Element::F32,
                ..
            }
        ));
    }

    #[test]
    fn an_unparseable_document_is_refused_at_write_time() {
        let (params, fields) = sample(3, 3);
        let mut p = ProjectWrite::new(&params, &fields);
        p.document("vault.json", "{ not json");
        let mut buf = Vec::new();
        assert!(matches!(
            write_project(Cursor::new(&mut buf), &p)
                .expect_err("invalid JSON must not reach the archive"),
            SaveError::DocumentJson { .. }
        ));
    }

    #[test]
    fn integral_floats_are_coerced_everywhere_kv04() {
        // `SAVEFILE_COMPAT.md` §14.2 / `GUI_GAP_REGISTER.md` KV-04: a
        // document that has passed through a language with one number type
        // comes back with `1.0` where `1` was written, and a strict parser
        // on the other side loses the user's data.
        let mut v: serde_json::Value = serde_json::from_str(
            r#"{"entity_id":1.0,"source_modified":1787605785.0,"nested":{"deep":[3.0,4.5,-7.0]},
                "kept":1.5,"huge":1e30,"text":"1.0","flag":true,"nothing":null}"#,
        )
        .unwrap();
        coerce_integral_floats(&mut v);

        assert_eq!(v["entity_id"], serde_json::json!(1));
        assert!(
            v["entity_id"].is_i64(),
            "must be an integer, not a float that prints as one"
        );
        assert_eq!(v["source_modified"], serde_json::json!(1787605785i64));
        assert!(v["source_modified"].is_i64());
        assert_eq!(v["nested"]["deep"][0], serde_json::json!(3));
        assert!(v["nested"]["deep"][0].is_i64());
        assert_eq!(v["nested"]["deep"][2], serde_json::json!(-7));
        // A genuine fraction is untouched.
        assert_eq!(v["nested"]["deep"][1], serde_json::json!(4.5));
        assert_eq!(v["kept"], serde_json::json!(1.5));
        // Past the safe-integer range (§14.1) it is left visible rather
        // than quietly saturated.
        assert!(v["huge"].is_f64());
        assert_eq!(v["text"], serde_json::json!("1.0"));
        assert_eq!(v["flag"], serde_json::json!(true));
        assert!(v["nothing"].is_null());

        // And it really runs on the read path, not only in this test.
        let (params, fields) = sample(3, 3);
        let mut p = ProjectWrite::new(&params, &fields);
        p.document(
            "vault.json",
            r#"{"version":1.0,"links":[{"entity_id":42.0}]}"#,
        );
        let buf = write_to_vec(&p);
        let back = read_project(Cursor::new(&buf)).unwrap();
        let doc = back.document("vault.json").unwrap();
        assert!(doc["version"].is_i64());
        assert!(doc["links"][0]["entity_id"].is_i64());
        // Deserializing into a strict integer type is the whole point.
        #[derive(serde::Deserialize)]
        struct Link {
            entity_id: i64,
        }
        #[derive(serde::Deserialize)]
        struct Store {
            version: u32,
            links: Vec<Link>,
        }
        let store: Store =
            serde_json::from_value(doc.clone()).expect("KV-04 must not be reproducible here");
        assert_eq!(store.version, 1);
        assert_eq!(store.links[0].entity_id, 42);
    }

    /// `world.origin` (`SAVEFILE_COMPAT.md` §7) is the tree's half of
    /// `SaveParams::origin`, and this pins the case an existing `.zip` on a
    /// user's disk is in: **no member at all**, distinguishable from a
    /// recorded `"gen"`.
    #[test]
    fn an_unknown_origin_writes_no_member_to_project_json() {
        let (params, fields) = sample(4, 4);
        assert_eq!(params.origin, None, "the sample is the pre-provenance shape");
        let m = manifest_json(&ProjectWrite::new(&params, &fields));
        assert!(m["world"].get("origin").is_none(), "wrote an origin for a world that had none: {m}");
        // The six MUST members are unaffected by the addition.
        assert_eq!(m["world"]["grid_width"], 4);
        assert_eq!(m["world"]["grid_height"], 4);
        assert_eq!(m["world"]["wrap_x"], true);
        assert_eq!(m["world"]["seed"], 4242);
        assert_eq!(m["world"]["sea_level"], 0.37);
        assert_eq!(m["world"]["map_width_km"], 1234.5);
    }

    #[test]
    fn a_known_origin_is_a_member_of_the_world_object() {
        let (mut params, fields) = sample(4, 4);
        params.origin = Some("region".to_string());
        let m = manifest_json(&ProjectWrite::new(&params, &fields));
        assert_eq!(m["world"]["origin"], "region");
    }

    /// Round-trips through a real archive for each origin this port writes,
    /// an unrecognised fourth (§4's unknown-member rule says carry it, not
    /// flatten it), and absence.
    #[test]
    fn every_origin_survives_a_tree_round_trip_including_absence() {
        for origin in [None, Some("gen"), Some("import"), Some("region"), Some("sculpt")] {
            let (mut params, fields) = sample(4, 4);
            params.origin = origin.map(str::to_string);
            let mut buf = Vec::new();
            write_project(Cursor::new(&mut buf), &ProjectWrite::new(&params, &fields))
                .expect("write_project should succeed");
            let back = read_project(Cursor::new(&buf)).expect("read back");
            assert_eq!(back.save.params.origin.as_deref(), origin, "origin did not survive");
        }
    }

    /// `world.name` (`SAVEFILE_COMPAT.md` §7) — the same MAY shape as
    /// `world.origin` above, added for `OUTSTANDING_WORK.md`'s "Recent
    /// worlds leaves show a filename where the canvas shows the world" row.
    /// Pins the case every native save written before this member existed
    /// is in: no member at all, distinguishable from a recorded name.
    #[test]
    fn an_unknown_name_writes_no_member_to_project_json() {
        let (params, fields) = sample(4, 4);
        assert_eq!(params.name, None, "the sample is the pre-name shape");
        let m = manifest_json(&ProjectWrite::new(&params, &fields));
        assert!(m["world"].get("name").is_none(), "wrote a name for a world that had none: {m}");
        // The six MUST members are unaffected by the addition.
        assert_eq!(m["world"]["grid_width"], 4);
        assert_eq!(m["world"]["grid_height"], 4);
        assert_eq!(m["world"]["wrap_x"], true);
        assert_eq!(m["world"]["seed"], 4242);
        assert_eq!(m["world"]["sea_level"], 0.37);
        assert_eq!(m["world"]["map_width_km"], 1234.5);
    }

    #[test]
    fn a_known_name_is_a_member_of_the_world_object() {
        let (mut params, fields) = sample(4, 4);
        params.name = Some("The Vharen Reach".to_string());
        let m = manifest_json(&ProjectWrite::new(&params, &fields));
        assert_eq!(m["world"]["name"], "The Vharen Reach");
    }

    /// Round-trips through a real archive: a named world, and absence — the
    /// exact "save, then open, and the name must be the one that was saved"
    /// check `SAVEFILE_COMPAT.md`'s own writer/reader contract calls for.
    #[test]
    fn a_world_name_survives_a_tree_round_trip_including_absence() {
        for name in [None, Some("The Vharen Reach"), Some("Kessa")] {
            let (mut params, fields) = sample(4, 4);
            params.name = name.map(str::to_string);
            let mut buf = Vec::new();
            write_project(Cursor::new(&mut buf), &ProjectWrite::new(&params, &fields))
                .expect("write_project should succeed");
            let back = read_project(Cursor::new(&buf)).expect("read back");
            assert_eq!(back.save.params.name.as_deref(), name, "world name did not survive");
        }
    }

    /// An old save on disk -- written before `world.name` existed -- must
    /// keep opening exactly as it did before this member was added: no
    /// `MissingField` refusal (`name` is MAY, not MUST) and every other
    /// member reads back unchanged. `manifest_json` never writes a `name`
    /// key for `params.name == None`, so this is the literal bytes an
    /// archive predating this change has.
    #[test]
    fn an_archive_from_before_world_name_existed_still_opens() {
        let (params, fields) = sample(4, 4);
        assert_eq!(params.name, None, "the fixture is the pre-name shape");
        let mut buf = Vec::new();
        write_project(Cursor::new(&mut buf), &ProjectWrite::new(&params, &fields))
            .expect("write_project should succeed");
        let back = read_project(Cursor::new(&buf)).expect("an archive with no world.name must still open");
        assert_eq!(back.save.params.name, None);
        assert_eq!(back.save.params.gw, params.gw);
        assert_eq!(back.save.params.gh, params.gh);
        assert_eq!(back.save.params.seed, params.seed);
    }

    #[test]
    fn the_manifest_says_what_the_specification_says_it_says() {
        let (params, fields) = sample(11, 7);
        let mut p = ProjectWrite::new(&params, &fields);
        p.created = Some("2026-08-25T00:00:00Z".into());
        let m = manifest_json(&p);
        assert_eq!(m["format"], PROJECT_FORMAT);
        // A literal, not `PROJECT_FORMAT_VERSION`: this test's name is a claim
        // about the specification, and §7 says `2`. Asserting the constant
        // against itself would hold for every value of it.
        assert_eq!(m["format_version"], 2);
        assert_eq!(m["world"]["grid_width"], 11);
        assert_eq!(m["world"]["grid_height"], 7);
        assert_eq!(m["world"]["wrap_x"], true);
        assert_eq!(m["world"]["seed"], 4242);
        assert_eq!(m["world"]["sea_level"], 0.37);
        assert_eq!(m["world"]["map_width_km"], 1234.5);
        assert_eq!(m["created"], "2026-08-25T00:00:00Z");
        // No grid duplication anywhere else (§13.1).
        assert!(m.get("GW").is_none());
    }

    #[test]
    fn a_foreign_project_json_is_refused_rather_than_guessed() {
        let mut buf = Vec::new();
        {
            let mut w = zip::ZipWriter::new(Cursor::new(&mut buf));
            w.start_file("project.json", zip_opts()).unwrap();
            w.write_all(br#"{"format":"some-other-tool","format_version":1}"#)
                .unwrap();
            w.finish().unwrap();
        }
        assert!(
            matches!(read_project(Cursor::new(&buf)), Err(LoadError::NotAProject(s)) if s == "some-other-tool")
        );
    }

    #[test]
    fn a_newer_format_version_warns_and_still_reads() {
        let (params, fields) = sample(4, 4);
        let buf = write_to_vec(&ProjectWrite::new(&params, &fields));
        let mut bumped = Vec::new();
        {
            let mut r = zip::ZipArchive::new(Cursor::new(&buf)).unwrap();
            let mut w = zip::ZipWriter::new(Cursor::new(&mut bumped));
            for i in 0..r.len() {
                let e = r.by_index_raw(i).unwrap();
                if e.name() == PROJECT_MANIFEST {
                    continue;
                }
                w.raw_copy_file(e).unwrap();
            }
            let mut m = manifest_json(&ProjectWrite::new(&params, &fields));
            m["format_version"] = serde_json::json!(99);
            w.start_file(PROJECT_MANIFEST, zip_opts()).unwrap();
            w.write_all(&serde_json::to_vec(&m).unwrap()).unwrap();
            w.finish().unwrap();
        }
        let back =
            read_project(Cursor::new(&bumped)).expect("a newer archive must not be discarded");
        assert_eq!(back.format_version, 99);
        assert_eq!(back.save.fields, fields);
        assert!(
            back.warnings.iter().any(|w| w.contains("format_version")),
            "{:?}",
            back.warnings
        );
    }

    #[test]
    fn a_bom_on_a_document_does_not_defeat_the_reader() {
        let (params, fields) = sample(3, 3);
        let buf = write_to_vec(&ProjectWrite::new(&params, &fields));
        let mut with_bom = Vec::new();
        {
            let mut r = zip::ZipArchive::new(Cursor::new(&buf)).unwrap();
            let mut w = zip::ZipWriter::new(Cursor::new(&mut with_bom));
            for i in 0..r.len() {
                w.raw_copy_file(r.by_index_raw(i).unwrap()).unwrap();
            }
            w.start_file("vault.json", zip_opts()).unwrap();
            w.write_all(&[0xEF, 0xBB, 0xBF]).unwrap();
            w.write_all(br#"{"version":1}"#).unwrap();
            w.finish().unwrap();
        }
        let back = read_project(Cursor::new(&with_bom)).unwrap();
        assert_eq!(back.document("vault.json").unwrap()["version"], 1);
        assert!(back.warnings.is_empty(), "{:?}", back.warnings);
    }

    #[test]
    fn every_registered_slot_is_reachable_and_unique() {
        // A registry with a typo silently creates a slot nothing can ever
        // write to, which is the failure this whole mechanism exists to
        // prevent -- so it is asserted rather than trusted.
        let mut seen = std::collections::HashSet::new();
        for slot in DOCUMENT_SLOTS {
            assert!(seen.insert(*slot), "duplicate document slot: {slot}");
            assert!(
                slot.ends_with(".json"),
                "a document slot must be JSON: {slot}"
            );
            assert!(
                !slot.starts_with('/') && !slot.contains(".."),
                "unsafe entry name: {slot}"
            );
        }
        let mut seen = std::collections::HashSet::new();
        for slot in RASTER_SLOTS {
            assert!(
                seen.insert(slot.path),
                "duplicate raster slot: {}",
                slot.path
            );
            assert!(
                slot.path.starts_with("rasters/"),
                "a raster lives under rasters/: {}",
                slot.path
            );
            assert!(
                slot.path.ends_with(slot.element.ext()),
                "the extension must name the element type: {}",
                slot.path
            );
        }
        for core in CORE_RASTERS {
            assert!(
                raster_slot(core).is_some(),
                "core raster {core} is not registered"
            );
        }
    }

    #[test]
    fn a_project_written_twice_has_identical_content() {
        // Not cosmetic: a save that differs run to run defeats every
        // version-control and sync workflow the owner might put a project
        // directory into.
        let (params, fields) = sample(5, 3);
        let mut p = ProjectWrite::new(&params, &fields);
        p.document("entities/settlements.json", r#"{"settlements":[]}"#);
        p.raster("rasters/territory.i32", Raster::I32(vec![2; 15]));
        p.history_territory.insert(3, vec![1; 15]);
        // `created` is provenance and is deliberately excluded: a
        // timestamp is the one member that must differ between two saves.
        // The zip container's own per-entry stamp is that same member one
        // level down -- see `content_fingerprint`, which is why this
        // compares content and not raw bytes, and why the test was renamed
        // rather than left claiming more than it checks.
        assert_eq!(
            content_fingerprint(&write_to_vec(&p)),
            content_fingerprint(&write_to_vec(&p))
        );
    }

    // ---- SAVEFILE_COMPAT.md §8.2: the byte-plane shuffle (owner Ruling AJ) ----

    /// The value generator behind `tests/fixtures/project_v1_7x5.ctl`,
    /// spelled exactly as the one-off program that wrote that file spelled it.
    /// A multiplicative hash of the cell index, so every byte of every element
    /// varies -- including exponent bytes that make NaNs, infinities and
    /// subnormals -- and a stride or plane error cannot hide in a smooth ramp.
    fn fixture_bits(i: usize, k: u32) -> u32 {
        (i as u32).wrapping_add(k).wrapping_mul(0x9E37_79B9)
    }

    fn bits_of(v: &[f32]) -> Vec<u32> {
        v.iter().map(|x| x.to_bits()).collect()
    }

    fn entry_names(buf: &[u8]) -> Vec<String> {
        let mut r = zip::ZipArchive::new(Cursor::new(buf)).unwrap();
        (0..r.len()).map(|i| r.by_index_raw(i).unwrap().name().to_string()).collect()
    }

    fn raw_entry(buf: &[u8], name: &str) -> Vec<u8> {
        let mut r = zip::ZipArchive::new(Cursor::new(buf)).unwrap();
        let mut out = Vec::new();
        r.by_name(name).unwrap().read_to_end(&mut out).unwrap();
        out
    }

    /// **The load-bearing property of the whole change: an archive on a
    /// user's disk today still opens, to the same bits.**
    ///
    /// The fixture is not built here. It was written on 2026-09-23 by the
    /// **unmodified version-1 writer** at `064b724` (a one-off program calling
    /// `write_project` with the values [`fixture_bits`] gives), before a line
    /// of the shuffle existed, and committed as bytes -- so it is the format
    /// an existing save actually has, not this build's idea of it. SHA-256
    /// `9a7998bf3d017541173d66d0ebdd61fed9d2a8b033a16f1b76f50725b2c9a2b8`.
    #[test]
    fn a_version_1_archive_reads_exactly_as_it_always_did() {
        let buf: &[u8] = include_bytes!("../tests/fixtures/project_v1_7x5.ctl");
        let n = 7 * 5;
        let f32s = |k: u32| (0..n).map(|i| fixture_bits(i, k)).collect::<Vec<u32>>();
        let i32s = |k: u32| (0..n).map(|i| fixture_bits(i, k) as i32).collect::<Vec<i32>>();
        let u8s = |k: u32| (0..n).map(|i| fixture_bits(i, k) as u8).collect::<Vec<u8>>();

        // The fixture is what it claims to be: plain names only, and the
        // heightmap entry is the plain dump, checked from the bytes rather
        // than through the reader under test.
        let names = entry_names(buf);
        assert!(names.iter().any(|n| n == "rasters/heightmap.f32"), "{names:?}");
        assert!(!names.iter().any(|n| n.contains(SHUFFLED_INFIX)), "{names:?}");
        let dump: Vec<u8> = f32s(1).iter().flat_map(|b| b.to_le_bytes()).collect();
        assert_eq!(raw_entry(buf, "rasters/heightmap.f32"), dump);

        let back = read_project(Cursor::new(buf)).expect("a version-1 archive opens");
        assert_eq!(back.format_version, 1);
        assert!(back.warnings.is_empty(), "{:?}", back.warnings);
        assert!(back.foreign.is_empty(), "{:?}", back.foreign.keys().collect::<Vec<_>>());
        let f = &back.save.fields;
        assert_eq!(bits_of(&f.heightmap), f32s(1));
        assert_eq!(bits_of(&f.temperature), f32s(2));
        assert_eq!(bits_of(&f.rainfall), f32s(3));
        assert_eq!(bits_of(&f.volcanic_field), f32s(4));
        assert_eq!(bits_of(&f.impact_field), f32s(5));
        assert_eq!(f.strahler_order, u8s(6));
        assert_eq!(back.raster("rasters/territory.i32"), Some(&Raster::I32(i32s(7))));
        match back.raster("rasters/agrarian_density.f32") {
            Some(Raster::F32(v)) => assert_eq!(bits_of(v), f32s(8)),
            other => panic!("agrarian_density: {other:?}"),
        }
        assert_eq!(back.raster("rasters/water_bodies.u8"), Some(&Raster::U8(u8s(9))));
        assert_eq!(back.history_territory.get(&7), Some(&i32s(10)));
        assert_eq!(back.text_of("vault.json"), Some(r#"{"version":1,"links":[]}"#));

        // Opening an old save and saving it writes version 2 -- and loses
        // nothing on the way: the re-saved archive reopens to the same bits.
        let mut again = ProjectWrite::new(&back.save.params, &back.save.fields);
        again.rasters = back.rasters.clone();
        again.history_territory = back.history_territory.clone();
        let buf2 = write_to_vec(&again);
        let back2 = read_project(Cursor::new(&buf2)).unwrap();
        assert_eq!(back2.format_version, 2);
        assert_eq!(bits_of(&back2.save.fields.heightmap), f32s(1));
        assert_eq!(bits_of(&back2.save.fields.rainfall), f32s(3));
        assert_eq!(back2.raster("rasters/territory.i32"), Some(&Raster::I32(i32s(7))));
        assert_eq!(back2.history_territory.get(&7), Some(&i32s(10)));
    }

    /// Round trip **and** layout. A round trip alone cannot tell a correct
    /// shuffle from a writer and reader that are wrong in the same way, and a
    /// second implementation reads the bytes, not this crate -- so the stored
    /// entry is checked against §8.2's own formula, `stored[k*n + i] ==
    /// le(v[i])[k]`, written out here independently of `write_planes`.
    #[test]
    fn a_shuffled_raster_round_trips_bit_for_bit_and_is_stored_as_planes() {
        let (params, mut fields) = sample(7, 5);
        let n = 35;
        let specials = [
            0.0f32,
            -0.0,
            f32::NAN,
            f32::from_bits(0x7fc0_1234), // a NaN with a payload
            f32::from_bits(0xffa0_0001), // a negative signalling-shaped NaN
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::from_bits(1), // the smallest subnormal
            f32::MIN_POSITIVE,
            f32::MAX,
            -f32::MAX,
            0.42,
            1.0,
        ];
        let hm: Vec<f32> = (0..n)
            .map(|i| specials.get(i).copied().unwrap_or_else(|| f32::from_bits(fixture_bits(i, 11))))
            .collect();
        fields.heightmap = Arc::new(hm.clone());
        let terr: Vec<i32> = (0..n)
            .map(|i| match i {
                0 => i32::MIN,
                1 => i32::MAX,
                2 => -1,
                3 => 0,
                _ => fixture_bits(i, 12) as i32,
            })
            .collect();
        let mut p = ProjectWrite::new(&params, &fields);
        p.raster("rasters/territory.i32", Raster::I32(terr.clone()));
        p.raster("rasters/water_bodies.u8", Raster::U8((0..n).map(|i| i as u8).collect()));
        p.history_territory.insert(9, terr.clone());
        let buf = write_to_vec(&p);

        // Every 4-byte raster under its shuffled name and NOT its plain one --
        // the absence of `rasters/heightmap.f32` is what makes a version-1
        // reader refuse this archive instead of drawing noise. `u8` and
        // history keep their plain names (§8.2, §10.2).
        let names = entry_names(&buf);
        for slot in RASTER_SLOTS.iter().filter(|s| {
            CORE_RASTERS.contains(&s.path) || s.path == "rasters/territory.i32" || s.path == "rasters/water_bodies.u8"
        }) {
            let shuffled = slot.element.size() == 4;
            let stored = if shuffled { raster_entry_name(*slot) } else { slot.path.to_string() };
            assert!(names.contains(&stored), "{stored} missing from {names:?}");
            if shuffled {
                assert!(!names.iter().any(|x| x == slot.path), "{} written plain as well", slot.path);
            }
        }
        assert!(names.iter().any(|x| x == "history/territory/9.i32"), "{names:?}");
        assert_eq!(
            raster_entry_name(raster_slot("rasters/heightmap.f32").unwrap()),
            "rasters/heightmap.shuffled.f32"
        );

        let raw = raw_entry(&buf, "rasters/heightmap.shuffled.f32");
        assert_eq!(raw.len(), n * 4);
        for (i, v) in hm.iter().enumerate() {
            for k in 0..4 {
                assert_eq!(raw[k * n + i], v.to_le_bytes()[k], "heightmap byte {k} of cell {i}");
            }
        }
        let raw = raw_entry(&buf, "rasters/territory.shuffled.i32");
        for (i, v) in terr.iter().enumerate() {
            for k in 0..4 {
                assert_eq!(raw[k * n + i], v.to_le_bytes()[k], "territory byte {k} of cell {i}");
            }
        }
        // History stays the plain dump.
        let dump: Vec<u8> = terr.iter().flat_map(|v| v.to_le_bytes()).collect();
        assert_eq!(raw_entry(&buf, "history/territory/9.i32"), dump);

        let back = read_project(Cursor::new(&buf)).unwrap();
        assert_eq!(back.format_version, 2);
        assert!(back.warnings.is_empty(), "{:?}", back.warnings);
        assert!(back.foreign.is_empty(), "{:?}", back.foreign.keys().collect::<Vec<_>>());
        assert_eq!(bits_of(&back.save.fields.heightmap), bits_of(&hm));
        assert_eq!(bits_of(&back.save.fields.temperature), bits_of(&fields.temperature));
        assert_eq!(bits_of(&back.save.fields.volcanic_field), bits_of(&fields.volcanic_field));
        assert_eq!(back.raster("rasters/territory.i32"), Some(&Raster::I32(terr.clone())));
        assert_eq!(back.history_territory.get(&9), Some(&terr));
    }

    /// The other half of §8.2's marker: the reader decides by NAME. A plain
    /// entry in a version-2 archive (a writer that skipped the shuffle, which
    /// §8.2 permits) is read as a dump, never un-shuffled; and an archive
    /// carrying both names for one slot reads the shuffled one and says so.
    #[test]
    fn a_plain_entry_is_never_unshuffled_and_two_copies_are_reported() {
        let (params, fields) = sample(7, 5);
        let buf = write_to_vec(&ProjectWrite::new(&params, &fields));
        let rebuild = |drop_shuffled: bool, plain: &[f32]| -> Vec<u8> {
            let mut out = Vec::new();
            {
                let mut r = zip::ZipArchive::new(Cursor::new(&buf)).unwrap();
                let mut w = zip::ZipWriter::new(Cursor::new(&mut out));
                for i in 0..r.len() {
                    let e = r.by_index_raw(i).unwrap();
                    if drop_shuffled && e.name() == "rasters/heightmap.shuffled.f32" {
                        continue;
                    }
                    w.raw_copy_file(e).unwrap();
                }
                w.start_file("rasters/heightmap.f32", zip_opts()).unwrap();
                w.write_all(&plain.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>())
                    .unwrap();
                w.finish().unwrap();
            }
            out
        };

        let only_plain = read_project(Cursor::new(rebuild(true, &fields.heightmap))).unwrap();
        assert_eq!(only_plain.format_version, 2);
        assert!(only_plain.warnings.is_empty(), "{:?}", only_plain.warnings);
        assert_eq!(bits_of(&only_plain.save.fields.heightmap), bits_of(&fields.heightmap));

        let decoy = vec![9.5f32; 35];
        let both = read_project(Cursor::new(rebuild(false, &decoy))).unwrap();
        assert_eq!(
            bits_of(&both.save.fields.heightmap),
            bits_of(&fields.heightmap),
            "the shuffled copy wins"
        );
        assert_eq!(both.warnings.len(), 1, "{:?}", both.warnings);
        assert!(
            both.warnings[0].contains("rasters/heightmap.f32")
                && both.warnings[0].contains("rasters/heightmap.shuffled.f32"),
            "{:?}",
            both.warnings
        );
    }
}
