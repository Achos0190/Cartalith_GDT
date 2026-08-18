//! Urban morphology — the reference's script block 4 ("UME", lines 28166-31104
//! of `reference/Cartalith Gen1 v2.10.html`), ported.
//!
//! Block 4 is a self-contained IIFE with **no DOM references at all** (verified
//! by grep, not assumed: `document`/`window`/`canvas`/`ctx.`/`getElementById`/
//! `localStorage`/`requestAnimationFrame` produce zero hits inside its line
//! range) and **no asset-pack references** (confirming, independently, the
//! finding Phase 4's own milestone-1 investigation recorded). It also takes no
//! civ *types* — its whole input surface is scalars plus two plain rasters —
//! so this crate deliberately does **not** depend on `cartalith-civ`. See
//! `URBAN_MORPHOLOGY_SCOPE.md` for the full investigation and the milestone
//! plan this crate is being built along.
//!
//! Milestone 1 is the foundation every later milestone reads: the labelled RNG
//! substreams ([`rng`]) and the vector/polygon geometry kernel ([`geom`]).
//! Milestone 2 adds the [`graph`] the whole engine is built on — a planar
//! street graph with a uniform-grid spatial index, and the planar face
//! extraction that turns it into town blocks.
//!
//! **Not wired to anything.** Nothing in this crate is called from
//! `compute_civilisation()`, `cartalith-godot`, or the GUI — same standing
//! discipline as `cartalith-spatial` and every unwired subsystem port before it.

pub mod geom;
pub mod graph;
pub mod rng;

pub use geom::Vec2;
pub use graph::{Edge, Face, Graph, Node};
pub use rng::{Substream, fnv1a, stream};
