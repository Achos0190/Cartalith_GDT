//! Ruling CN's colour-paint layer on `WorldGen` (`COLOUR_PAINT_SCOPE.md`
//! CP-2): the binding the shell's CARTO ▸ Colours ▸ Paint colour tool drives.
//!
//! # What it is
//!
//! `WorldGen::colour_paint` holds the layer ([`ColourPaint`], marks in map
//! kilometres, flat colours or CP-1M's emissive materials) behind an `Arc`,
//! so the deep-zoom worker's snapshot and the export snapshot take a
//! refcount rather than a copy. Every edit builds a new layer (the layer is
//! immutable once built) and moves `colour_paint_epoch`, which the tile and
//! grid cache keys read (`lod_key_with`), so no cached tile outlives an edit,
//! an undo or a redo.
//!
//! # Why the whole document crosses the boundary
//!
//! The shell's undo is stroke-granular on its own stack (§8): a step is the
//! document before the change, restored with [`WorldGen::colour_paint_set_document`].
//! Marks are small (§9: 5 000 x 40-point marks are 3.98 MB of JSON), so a
//! document per step is the simple, exact answer.
//!
//! # What it must never do
//!
//! - Touch world data or generation: this file only replaces the layer.
//! - Encode "no layer" as an empty one: `None` is the absent state, and the
//!   renderer's `with_colour_paint` filters inert layers by control flow.
//! - Accept a document the layer refuses (caps, non-finite values, unknown
//!   version or blend): a refused edit changes nothing and says why.

use std::sync::Arc;

use cartalith_spatial::colour_paint::{ColourMark, ColourPaint, ColourPaintDoc, DOC_VERSION};
use godot::prelude::*;

use crate::WorldGen;

/// The bucket index's exact-answer margin, in km: two grid cells, the widest
/// pixel footprint the screen raster asks for (`cell_color` asks one cell; a
/// zoomed-out tile may ask more, and falls back to the brute-force path,
/// never to a wrong answer -- `colour_paint::BucketIndex`). `0` with no scale
/// or no grid, which sends every query through the fallback.
pub(crate) fn index_margin_km(map_width_km: f64, gw: usize) -> f64 {
    if map_width_km.is_finite() && map_width_km > 0.0 && gw > 0 {
        2.0 * map_width_km / gw as f64
    } else {
        0.0
    }
}

impl WorldGen {
    /// The layer as the renderers attach it (`RenderCtx::with_colour_paint`
    /// filters a hidden or empty one), or `None`.
    pub(crate) fn colour_paint_layer(&self) -> Option<&ColourPaint> {
        self.colour_paint.as_deref()
    }

    /// The layer's `Arc`, for the worker and export snapshots. `None` for no
    /// layer **and for an inert one** (hidden or no marks): the snapshot's
    /// fingerprint then reads exactly what it read before CP-2.
    pub(crate) fn colour_paint_for_snapshot(&self) -> Option<Arc<ColourPaint>> {
        self.colour_paint.as_ref().filter(|l| !l.is_inert()).cloned()
    }

    /// The tile/grid cache key's colour-paint term: empty with no visible
    /// marks (so every key without paint is the pre-CP-2 string), else the
    /// edit epoch.
    pub(crate) fn colour_paint_key_term(&self) -> String {
        match self.colour_paint.as_ref().filter(|l| !l.is_inert()) {
            Some(_) => format!(";cp{}", self.colour_paint_epoch),
            None => String::new(),
        }
    }

    /// Replaces the layer with `doc` (`None` removes it), rebuilding the
    /// index at this world's scale, and moves the epoch. `Err` leaves the
    /// layer as it was.
    pub(crate) fn colour_paint_install(&mut self, doc: Option<ColourPaintDoc>) -> Result<usize, String> {
        let layer = match doc {
            None => None,
            Some(d) => Some(Arc::new(ColourPaint::new(d, index_margin_km(self.map_width_km, self.gw.max(0) as usize)).map_err(|e| e.to_string())?)),
        };
        let n = layer.as_ref().map_or(0, |l| l.len());
        self.colour_paint = layer;
        self.colour_paint_epoch = self.colour_paint_epoch.wrapping_add(1);
        Ok(n)
    }

    /// Rebuilds the layer's index at the current grid's scale after a
    /// generate or a load changed `gw` or the map width. The document is
    /// untouched: marks keep their kilometres across a regenerate (§5).
    pub(crate) fn colour_paint_rescale(&mut self) {
        if let Some(doc) = self.colour_paint.as_ref().map(|l| l.doc().clone()) {
            // A document that built once builds again: the caps and values
            // do not depend on the scale. Should it ever not, the layer is
            // dropped rather than kept at a stale index (never a wrong answer).
            if self.colour_paint_install(Some(doc)).is_err() {
                self.colour_paint = None;
            }
        }
    }
}

/// `{ok: true, marks}` or `{ok: false, error}`.
fn reply(r: Result<usize, String>) -> VarDictionary {
    match r {
        Ok(n) => vdict! { "ok" => true, "marks" => n as i64 },
        Err(e) => vdict! { "ok" => false, "error" => e.as_str() },
    }
}

#[godot_api(secondary)]
impl WorldGen {
    /// The layer's state: `marks`, `visible`, `emissive` (how many marks
    /// carry a material), `revision` (the edit epoch), and `km_per_cell` --
    /// **absent** when the world has no map width or no grid (no scale, no
    /// paint: a mark in km cannot be placed). With no layer, `marks` is 0 and
    /// `visible` true (a new stroke starts a visible layer).
    #[func]
    fn colour_paint_info(&self) -> VarDictionary {
        let (marks, visible, emissive) = match self.colour_paint.as_ref() {
            Some(l) => (l.len() as i64, l.doc().visible, l.doc().marks.iter().filter(|m| m.emissive.is_some()).count() as i64),
            None => (0, true, 0),
        };
        let mut d = vdict! { "marks" => marks, "visible" => visible, "emissive" => emissive, "revision" => self.colour_paint_epoch as i64 };
        if self.map_width_km.is_finite() && self.map_width_km > 0.0 && self.gw > 0 {
            d.set("km_per_cell", self.map_width_km / self.gw as f64);
        }
        d
    }

    /// The layer as its saved JSON (`ColourPaintDoc`, what CP-3's
    /// `appearance_paint.json` holds), or `""` with no layer.
    #[func]
    fn colour_paint_document(&self) -> GString {
        match self.colour_paint.as_ref() {
            Some(l) => GString::from(serde_json::to_string(l.doc()).unwrap_or_default().as_str()),
            None => GString::new(),
        }
    }

    /// Replaces the layer with the document `json`; `""` removes it. The
    /// shell's undo and Clear go through here. A refused document (bad JSON,
    /// unknown version or blend, a cap, a non-finite value) changes nothing.
    #[func]
    fn colour_paint_set_document(&mut self, json: GString) -> VarDictionary {
        let s = json.to_string();
        if s.trim().is_empty() {
            return reply(self.colour_paint_install(None));
        }
        match serde_json::from_str::<ColourPaintDoc>(&s) {
            Ok(doc) => reply(self.colour_paint_install(Some(doc))),
            Err(e) => reply(Err(format!("not a colour-paint document: {e}"))),
        }
    }

    /// Appends one mark (`ColourMark` JSON) on top of the others, starting a
    /// visible layer if there is none. Refused whole like a document.
    #[func]
    fn colour_paint_add_mark(&mut self, json: GString) -> VarDictionary {
        let mark: ColourMark = match serde_json::from_str(&json.to_string()) {
            Ok(m) => m,
            Err(e) => return reply(Err(format!("not a colour-paint mark: {e}"))),
        };
        let mut doc = self.colour_paint.as_ref().map(|l| l.doc().clone()).unwrap_or(ColourPaintDoc { version: DOC_VERSION, visible: true, marks: Vec::new() });
        doc.marks.push(mark);
        reply(self.colour_paint_install(Some(doc)))
    }

    /// Shows or hides the layer. `true` when the flag changed; `false` with
    /// no layer (there is nothing to show or hide).
    #[func]
    fn colour_paint_set_visible(&mut self, visible: bool) -> bool {
        let Some(mut doc) = self.colour_paint.as_ref().map(|l| l.doc().clone()) else { return false };
        if doc.visible == visible {
            return false;
        }
        doc.visible = visible;
        self.colour_paint_install(Some(doc)).is_ok()
    }
}
