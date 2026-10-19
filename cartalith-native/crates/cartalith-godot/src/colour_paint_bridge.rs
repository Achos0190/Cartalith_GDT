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
//! The layer is organised into named **paint layers** (Ruling CS,
//! 2026-10-19: `PaintLayers`, bottom first, each with its own visibility and
//! lock). The `colour_paint_layer_*` funcs edit them; each edit is a whole new
//! document like a stroke, so the shell's document-per-step undo covers layer
//! edits too. Only visible layers' marks reach the renderer
//! (`ColourPaint::from_layers`), in layer order.
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

use cartalith_spatial::colour_paint::{ColourMark, ColourPaint, ColourPaintDoc, PaintLayers};
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
        self.colour_paint_install_layers(doc.map(PaintLayers::from_v1))
    }

    /// [`WorldGen::colour_paint_install`] for a layered document (Ruling CS):
    /// every layer edit, undo, redo and open goes through here, so the epoch
    /// moves on each and no cached tile outlives one. Returns the marks in
    /// every layer.
    pub(crate) fn colour_paint_install_layers(&mut self, doc: Option<PaintLayers>) -> Result<usize, String> {
        let layer = match doc {
            None => None,
            Some(d) => Some(Arc::new(ColourPaint::from_layers(d, index_margin_km(self.map_width_km, self.gw.max(0) as usize)).map_err(|e| e.to_string())?)),
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
        if let Some(doc) = self.colour_paint.as_ref().map(|l| l.layers().clone()) {
            // A document that built once builds again: the caps and values
            // do not depend on the scale. Should it ever not, the layer is
            // dropped rather than kept at a stale index (never a wrong answer).
            if self.colour_paint_install_layers(Some(doc)).is_err() {
                self.colour_paint = None;
            }
        }
    }

    /// Applies one layer edit (Ruling CS) to a copy of the document -- the
    /// empty document when there is none -- and installs it. `Err` (the
    /// edit's own refusal, or the layer's caps) leaves everything as it was.
    fn colour_paint_edit(&mut self, edit: impl FnOnce(&mut PaintLayers) -> Result<(), String>) -> Result<usize, String> {
        let mut doc = self.colour_paint.as_ref().map(|l| l.layers().clone()).unwrap_or_else(PaintLayers::empty);
        edit(&mut doc)?;
        self.colour_paint_install_layers(Some(doc))
    }
}

/// A GDScript layer index as a `usize`, or why not (a negative index is
/// refused, never wrapped onto a real layer).
fn layer_index(i: i64) -> Result<usize, String> {
    usize::try_from(i).map_err(|_| format!("there is no layer {i}"))
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
    /// The layer's state: `marks` (in every paint layer, hidden ones
    /// included), `drawn` (those in visible layers), `layers` (how many paint
    /// layers), `visible` (the master switch), `emissive` (how many marks
    /// carry a material), `revision` (the edit epoch), and `km_per_cell` --
    /// **absent** when the world has no map width or no grid (no scale, no
    /// paint: a mark in km cannot be placed). With no layer, `marks` is 0 and
    /// `visible` true (a new stroke starts a visible layer).
    #[func]
    fn colour_paint_info(&self) -> VarDictionary {
        let (marks, visible, emissive) = match self.colour_paint.as_ref() {
            Some(l) => (l.len() as i64, l.doc().visible, l.layers().layers.iter().flat_map(|x| &x.marks).filter(|m| m.emissive.is_some()).count() as i64),
            None => (0, true, 0),
        };
        let (layers, drawn) = self.colour_paint.as_ref().map_or((0, 0), |l| (l.layers().layers.len() as i64, l.drawn_len() as i64));
        let mut d = vdict! { "marks" => marks, "drawn" => drawn, "layers" => layers, "visible" => visible, "emissive" => emissive, "revision" => self.colour_paint_epoch as i64 };
        if self.map_width_km.is_finite() && self.map_width_km > 0.0 && self.gw > 0 {
            d.set("km_per_cell", self.map_width_km / self.gw as f64);
        }
        d
    }

    /// The layered document as JSON (`PaintLayers`: version 1 while it is
    /// one default layer, else version 2 -- the form CP-3's
    /// `appearance_paint.json` holds), or `""` with no layer. The shell's
    /// undo keeps these strings, so it covers layer edits as well as strokes.
    #[func]
    fn colour_paint_document(&self) -> GString {
        match self.colour_paint.as_ref() {
            Some(l) => GString::from(serde_json::to_string(l.layers()).unwrap_or_default().as_str()),
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
        match serde_json::from_str::<PaintLayers>(&s) {
            Ok(doc) => reply(self.colour_paint_install_layers(Some(doc))),
            Err(e) => reply(Err(format!("not a colour-paint document: {e}"))),
        }
    }

    /// Appends one mark (`ColourMark` JSON) to the **top** paint layer,
    /// starting the default layer if there is none. Refused whole like a
    /// document, and refused when the top layer is locked.
    #[func]
    fn colour_paint_add_mark(&mut self, json: GString) -> VarDictionary {
        let top = self.colour_paint.as_ref().map_or(0, |l| l.layers().layers.len().saturating_sub(1)) as i64;
        self.colour_paint_add_mark_to(top, json)
    }

    /// Appends one mark (`ColourMark` JSON) to paint layer `layer` (0 =
    /// bottom; with no layers, 0 starts the default one). Refused on a locked
    /// or missing layer.
    #[func]
    fn colour_paint_add_mark_to(&mut self, layer: i64, json: GString) -> VarDictionary {
        let mark: ColourMark = match serde_json::from_str(&json.to_string()) {
            Ok(m) => m,
            Err(e) => return reply(Err(format!("not a colour-paint mark: {e}"))),
        };
        reply(layer_index(layer).and_then(|i| self.colour_paint_edit(|d| d.add_mark(i, mark))))
    }

    /// The paint layers, **bottom first**: `[{name, visible, locked, marks}]`;
    /// empty with no layer. The shell lists them top first.
    #[func]
    fn colour_paint_layers(&self) -> VarArray {
        let mut out = VarArray::new();
        if let Some(l) = self.colour_paint.as_ref() {
            for x in &l.layers().layers {
                out.push(&vdict! { "name" => x.name.as_str(), "visible" => x.visible, "locked" => x.locked, "marks" => x.marks.len() as i64 }.to_variant());
            }
        }
        out
    }

    /// Adds an empty paint layer on top (`""` names it the next free
    /// `Paint N`). The reply's `layer` is its index.
    #[func]
    fn colour_paint_layer_add(&mut self, name: GString) -> VarDictionary {
        let name = name.to_string();
        let mut at = 0usize;
        let r = self.colour_paint_edit(|d| {
            at = d.add_layer(Some(name.as_str()))?;
            Ok(())
        });
        let ok = r.is_ok();
        let mut d = reply(r);
        if ok {
            d.set("layer", at as i64);
        }
        d
    }

    /// Renames paint layer `layer` (trimmed; blank refused; allowed locked).
    #[func]
    fn colour_paint_layer_rename(&mut self, layer: i64, name: GString) -> VarDictionary {
        let name = name.to_string();
        reply(layer_index(layer).and_then(|i| self.colour_paint_edit(|d| d.rename(i, &name))))
    }

    /// Moves paint layer `from` to index `to` (0 = bottom).
    #[func]
    fn colour_paint_layer_move(&mut self, from: i64, to: i64) -> VarDictionary {
        reply(layer_index(from).and_then(|f| layer_index(to).and_then(|t| self.colour_paint_edit(|d| d.move_layer(f, t)))))
    }

    /// Shows or hides paint layer `layer` (a hidden layer is not composited).
    #[func]
    fn colour_paint_layer_set_visible(&mut self, layer: i64, visible: bool) -> VarDictionary {
        reply(layer_index(layer).and_then(|i| self.colour_paint_edit(|d| d.set_layer_visible(i, visible))))
    }

    /// Locks or unlocks paint layer `layer` (a locked layer's marks cannot
    /// be added to, cleared or deleted).
    #[func]
    fn colour_paint_layer_set_locked(&mut self, layer: i64, locked: bool) -> VarDictionary {
        reply(layer_index(layer).and_then(|i| self.colour_paint_edit(|d| d.set_layer_locked(i, locked))))
    }

    /// Deletes paint layer `layer` and its marks (refused when locked).
    #[func]
    fn colour_paint_layer_delete(&mut self, layer: i64) -> VarDictionary {
        reply(layer_index(layer).and_then(|i| self.colour_paint_edit(|d| d.delete_layer(i))))
    }

    /// Empties paint layer `layer`, keeping it (refused when locked).
    #[func]
    fn colour_paint_layer_clear(&mut self, layer: i64) -> VarDictionary {
        reply(layer_index(layer).and_then(|i| self.colour_paint_edit(|d| d.clear_layer(i))))
    }

    /// Shows or hides the layer. `true` when the flag changed; `false` with
    /// no layer (there is nothing to show or hide).
    #[func]
    fn colour_paint_set_visible(&mut self, visible: bool) -> bool {
        let Some(mut doc) = self.colour_paint.as_ref().map(|l| l.layers().clone()) else { return false };
        if doc.visible == visible {
            return false;
        }
        doc.visible = visible;
        self.colour_paint_install_layers(Some(doc)).is_ok()
    }
}
