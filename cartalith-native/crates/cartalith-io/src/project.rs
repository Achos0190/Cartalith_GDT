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
//!    on the way out.** A raster entry carries no length of its own, so a
//!    short one is not a parse error; it is a truncated world.
//! 2. **Integral floats are coerced to integers before any schema sees a
//!    document** ([`coerce_integral_floats`]). `SAVEFILE_COMPAT.md` §14.2
//!    is the rule; `GUI_GAP_REGISTER.md` KV-04 is what forgetting it cost —
//!    every knowledge link a user ever made, discarded on each launch, for
//!    the shipped lifetime of the feature, because one integer came back as
//!    `1.0`. Doing this per-field would mean remembering it per-field.

use crate::{LoadError, SaveData, SaveError, SaveFields, SaveParams};
use std::collections::BTreeMap;
use std::io::{Read, Seek, Write};

/// The entry whose **presence** selects the tree layout
/// (`SAVEFILE_COMPAT.md` §4). One central-directory lookup, no heuristics.
pub const PROJECT_MANIFEST: &str = "project.json";

/// `project.json`'s `format` member. A file whose `format` is present and
/// different is not a Cartalith project and is refused rather than guessed.
pub const PROJECT_FORMAT: &str = "cartalith-project";

/// `project.json`'s `format_version`. `1` is the only version defined.
pub const PROJECT_FORMAT_VERSION: i64 = 1;

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

    /// Little-endian, no header, no length prefix (`SAVEFILE_COMPAT.md`
    /// §8). Buffered rather than one `write_all` per value: at this port's
    /// 8192x8192 ceiling a raster is 67 million values and a per-value call
    /// into the DEFLATE encoder is the whole cost of the save.
    fn write_to<W: Write>(&self, sink: &mut W) -> std::io::Result<()> {
        const CHUNK: usize = 16 * 1024;
        let mut buf: Vec<u8> = Vec::with_capacity(CHUNK * 4);
        match self {
            Raster::U8(v) => return sink.write_all(v),
            Raster::F32(v) => {
                for chunk in v.chunks(CHUNK) {
                    buf.clear();
                    for &x in chunk {
                        buf.extend_from_slice(&x.to_le_bytes());
                    }
                    sink.write_all(&buf)?;
                }
            }
            Raster::I32(v) => {
                for chunk in v.chunks(CHUNK) {
                    buf.clear();
                    for &x in chunk {
                        buf.extend_from_slice(&x.to_le_bytes());
                    }
                    sink.write_all(&buf)?;
                }
            }
        }
        Ok(())
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
];

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
/// # `entities/landmarks.json` carries the dock's settings, not its results
///
/// Registered 2026-09-01. Before that the landmark layer was in no slot at
/// all, and `write_project` rejects an unregistered slot outright, so the name
/// had to exist here before a writer could exist anywhere.
/// `cartalith-godot`'s `project_bridge.rs` supplies that writer
/// (`LandmarksDoc`, written from `WorldGen::landmark_store.settings` and
/// parsed back on open), and lists the slot in its own `ENGINE_OWNED_SLOTS`
/// so the shell cannot overwrite a document it has no view of.
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
/// **The split is deliberate and the two halves are not symmetric.** The
/// authored settings — per-kind caps, armed flags, crowding, the four class
/// radii — are hand-entered configuration that no recomputation brings back,
/// and until this slot existed they were never written *and* never cleared,
/// so they followed the user out of whichever project was open last into the
/// next one. The retained *run* is not written, because
/// `cartalith_civ::landmark::generate` is a pure function of the world, the
/// settings and the seed: reproducing it is one click, and showing placements
/// taken over the previous field against a new one would be wrong, which is
/// why `load_save` invalidates it either way.
pub const DOCUMENT_SLOTS: &[&str] = &[
    "entities/settlements.json",
    "entities/factions.json",
    "entities/ways.json",
    "entities/provinces.json",
    "entities/continents.json",
    "entities/journeys.json",
    "entities/landmarks.json",
    "history/timeline.json",
    "annotations/labels.json",
    "annotations/icons.json",
    "annotations/regions.json",
    "annotations/measurements.json",
    "library/assets.json",
    "library/travel.json",
    "drafts/paint.json",
    "drafts/sculpt.json",
    "appearance.json",
    "vault.json",
];

/// `history/territory/` — the prefix a per-year territory raster sits
/// under. Separate from `rasters/` because `rasters/` is *this* world's
/// grids and a snapshot is a past one (`SAVEFILE_COMPAT.md` §5.1).
pub const HISTORY_TERRITORY_PREFIX: &str = "history/territory/";

fn raster_slot(path: &str) -> Option<RasterSlot> {
    RASTER_SLOTS.iter().copied().find(|s| s.path == path)
}

/// Whether `name` is an entry **this build produces and consumes itself** —
/// the manifest, `params.json`, the two human-facing extras, any registered
/// document slot, any registered raster, or a `history/territory/<year>.i32`.
///
/// One function because both directions must agree exactly: [`read_project`]
/// files everything else under [`ProjectData::foreign`], and [`write_project`]
/// re-emits that map while refusing to let a carried copy shadow an entry it
/// writes from the live model. Two hand-kept lists would drift, and the drift
/// would show up as a **failed save** -- `ZipWriter` refuses a duplicate entry
/// name rather than writing one.
fn is_own_entry(name: &str) -> bool {
    name == PROJECT_MANIFEST
        || name == "params.json"
        || name == "README.md"
        || name == "preview.png"
        || DOCUMENT_SLOTS.contains(&name)
        || raster_slot(name).is_some()
        || (name.starts_with(HISTORY_TERRITORY_PREFIX) && name.ends_with(".i32"))
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
            preview_png: None,
            foreign: BTreeMap::new(),
            readme: None,
            generator: format!("cartalith-native {}", env!("CARGO_PKG_VERSION")),
            created: None,
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
    appearance.json   how the map is drawn
    vault.json        links out to an external Markdown vault
    preview.png       a thumbnail; not map data

Every `rasters/*.f32`, `*.i32` and `*.u8` entry is a bare little-endian binary
dump with no header: exactly grid_width * grid_height elements, row-major,
index = y * grid_width + x.
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
    /// `project.json`'s `format_version`, or `0` for a flat archive.
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
pub fn write_project<W: Write + Seek>(
    sink: W,
    project: &ProjectWrite<'_>,
) -> Result<(), SaveError> {
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

    for (path, values) in [
        (CORE_RASTERS[0], &f.heightmap),
        (CORE_RASTERS[1], &f.temperature),
        (CORE_RASTERS[2], &f.rainfall),
        (CORE_RASTERS[3], &f.volcanic_field),
        (CORE_RASTERS[4], &f.impact_field),
    ] {
        writer.start_file(path, opts)?;
        write_f32_slice(&mut writer, values)?;
    }
    writer.start_file(CORE_RASTERS[5], opts)?;
    writer.write_all(&f.strahler_order)?;

    for (path, raster) in &project.rasters {
        writer.start_file(path.as_str(), opts)?;
        raster.write_to(&mut writer)?;
    }

    for (slot, text) in &project.documents {
        writer.start_file(slot.as_str(), opts)?;
        writer.write_all(text.as_bytes())?;
    }

    for (year, values) in &project.history_territory {
        writer.start_file(format!("{HISTORY_TERRITORY_PREFIX}{year}.i32"), opts)?;
        Raster::I32(values.clone()).write_to(&mut writer)?;
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
    Ok(())
}

fn write_f32_slice<W: Write>(sink: &mut W, values: &[f32]) -> std::io::Result<()> {
    const CHUNK: usize = 16 * 1024;
    let mut buf: Vec<u8> = Vec::with_capacity(CHUNK * 4);
    for chunk in values.chunks(CHUNK) {
        buf.clear();
        for &v in chunk {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        sink.write_all(&buf)?;
    }
    Ok(())
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
    root.insert(
        "world".into(),
        serde_json::json!({
            "grid_width": p.gw,
            "grid_height": p.gh,
            // `wrap_x`, not `world`: a member called `world` inside an
            // object called `world` is not a name (`SAVEFILE_COMPAT.md` §7).
            "wrap_x": p.world,
            "map_width_km": p.map_width_km,
            "sea_level": p.sea_level,
            "seed": p.seed,
        }),
    );
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
    for slot in RASTER_SLOTS {
        let Some(bytes) = read_entry_bytes(archive, slot.path) else {
            continue;
        };
        let bytes = match bytes {
            Ok(b) => b,
            // Present but unreadable. Fatal only for the terrain, exactly as
            // a wrong *length* is (§6.4) -- a heightmap this build cannot
            // decode is not a world, and reporting it as "missing" would
            // blame the wrong thing.
            Err(e) if slot.path == CORE_RASTERS[0] => return Err(LoadError::Io(e)),
            Err(e) => {
                warnings.push(format!("{}: skipped ({e})", slot.path));
                continue;
            }
        };
        let expected = n * slot.element.size();
        if bytes.len() != expected {
            let message = format!(
                "{}: expected {expected} bytes for this grid, got {}",
                slot.path,
                bytes.len()
            );
            if slot.path == CORE_RASTERS[0] {
                return Err(LoadError::RasterLength(message));
            }
            warnings.push(format!("{message} -- skipped"));
            continue;
        }
        rasters.insert(
            slot.path.to_string(),
            Raster::from_bytes(slot.element, &bytes),
        );
    }

    let heightmap = match rasters.remove(CORE_RASTERS[0]) {
        Some(Raster::F32(v)) => v,
        // §6.4: the one raster whose absence is fatal. Everything else has
        // an honest substitute; a project with no terrain has nothing.
        _ => return Err(LoadError::MissingEntry("rasters/heightmap.f32")),
    };
    let mut take_f32 = |path: &str, honest_zero: bool| -> Vec<f32> {
        match rasters.remove(path) {
            Some(Raster::F32(v)) => v,
            _ => {
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
        _ => vec![0u8; n],
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

    let preview_png = read_entry_bytes(archive, "preview.png").and_then(|r| r.ok());

    Ok(ProjectData {
        save: SaveData {
            params: SaveParams {
                gw,
                gh,
                seed,
                map_width_km,
                sea_level,
                world,
            },
            fields: SaveFields {
                heightmap,
                temperature,
                rainfall,
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
        preview_png,
        foreign,
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
        };
        let fields = SaveFields {
            heightmap: (0..n).map(|i| i as f32 * 0.25).collect(),
            temperature: (0..n).map(|i| 30.0 - i as f32 * 0.5).collect(),
            rainfall: (0..n).map(|i| (i % 13) as f32 * 0.125).collect(),
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
    fn is_own_entry_covers_all_seven_branches_and_no_more() {
        for owned in [
            "project.json",              // PROJECT_MANIFEST
            "params.json",
            "README.md",
            "preview.png",
            "entities/settlements.json", // DOCUMENT_SLOTS
            "drafts/paint.json",         // DOCUMENT_SLOTS, the slot this pass made restore
            "rasters/heightmap.f32",     // raster_slot
            "history/territory/1200.i32", // the prefix + extension pair
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
            "history/territory/1200.json", // right prefix, wrong extension
            "history/territory.i32",     // right extension, not under the prefix
            "vendor/notes.txt",          // plainly someone else's
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
        for name in [
            "entities/settlements.json",
            "rasters/heightmap.f32",
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
            "rasters/heightmap.f32",
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

        let odd = set_compression_method(&buf, "rasters/territory.i32", 1);
        let back = read_project(Cursor::new(&odd)).expect("one odd entry must not cost the world");
        assert!(back.raster("rasters/territory.i32").is_none());
        assert_eq!(back.warnings.len(), 1, "{:?}", back.warnings);
        assert!(
            back.warnings[0].contains("rasters/territory.i32") && back.warnings[0].contains('1'),
            "the warning must name the entry and the method: {:?}",
            back.warnings
        );

        // The terrain is the one entry whose undecodability is fatal -- and
        // it is reported as unreadable, not as missing, so the user is told
        // what is actually wrong.
        let odd = set_compression_method(&buf, "rasters/heightmap.f32", 1);
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
            read_project(Cursor::new(rebuild("rasters/heightmap.f32"))),
            Err(LoadError::MissingEntry("rasters/heightmap.f32"))
        ));

        let back = read_project(Cursor::new(rebuild("rasters/temperature.f32")))
            .expect("climate is not fatal");
        assert_eq!(back.save.fields.temperature, vec![0.0f32; 16]);
        assert_eq!(
            back.warnings.len(),
            1,
            "the zero-fill must be reported, never assumed: {:?}",
            back.warnings
        );

        // Volcanic/impact zero really is the true value, so no warning.
        let back = read_project(Cursor::new(rebuild("rasters/volcanic_field.f32"))).unwrap();
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
        fields.rainfall.pop();
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

    #[test]
    fn the_manifest_says_what_the_specification_says_it_says() {
        let (params, fields) = sample(11, 7);
        let mut p = ProjectWrite::new(&params, &fields);
        p.created = Some("2026-08-25T00:00:00Z".into());
        let m = manifest_json(&p);
        assert_eq!(m["format"], PROJECT_FORMAT);
        assert_eq!(m["format_version"], PROJECT_FORMAT_VERSION);
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
    fn a_project_written_twice_is_byte_identical() {
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
        assert_eq!(write_to_vec(&p), write_to_vec(&p));
    }
}
