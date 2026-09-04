//! GeoJSON **import** — `FUNCTIONAL_CONTRACT.md` DM-03's other half.
//!
//! [`parse_geojson`] reads a GeoJSON `FeatureCollection` back into memory: the
//! documents `cartalith_engine::geojson::export_geojson` writes, and foreign
//! documents that happen to be shaped the same way. It is the mirror of that
//! module and nothing more — it produces geometry and properties, and it has
//! **no opinion about what an imported feature means for a world**. See "What
//! this deliberately does not decide" below.
//!
//! # Why the reader is here and the writer is in `cartalith-engine`
//!
//! Not because it had to be: `cartalith-engine` could perfectly well hold both
//! halves. It is here because assembling a document out of six subsystems'
//! state is orchestration, while *parsing* one is a pure transformation on text
//! that knows nothing about a world, and reading an outside file into memory is
//! what this crate does for every other format the app accepts.
//!
//! **That choice has one cost, and it is paid in the test suite.**
//! `cartalith-engine` depends on this crate and not the reverse, so no test
//! here can call `export_geojson`. `tests/reference_geojson_round_trip.rs`
//! therefore imports a *copy* of the exporter's own golden document — the bytes
//! the reference implementation itself emitted — rather than a live export, and
//! says so in its own header.
//!
//! # A parser's job is to reject
//!
//! Almost all of this module is refusal. A document-level refusal says what the
//! file is instead; a refusal inside a feature names that feature's index and
//! the path within it, down to the coordinate component. The rules, in the
//! order the document is walked:
//!
//! | Input | Result |
//! |---|---|
//! | not JSON | [`GeoJsonError::Json`], carrying serde_json's line and column |
//! | JSON that is not an object | [`GeoJsonError::NotAnObject`] |
//! | an object whose `type` is not `"FeatureCollection"` | [`GeoJsonError::NotAFeatureCollection`] |
//! | no `features` array | [`GeoJsonError::NoFeatureArray`] |
//! | a top-level `crs` member | [`GeoJsonError::ForeignCrs`] — see below |
//! | a feature with `"geometry": null`, or no `geometry` member | refused |
//! | a geometry type this port has no shape for (`GeometryCollection`) | refused, by name |
//! | a coordinate that is a string, a bool, or an object | refused, at its index |
//! | a position of fewer than two numbers | refused, at its own index |
//! | a coordinate literal too large for an `f64` | refused by serde_json, as a JSON error |
//! | a `LineString` of fewer than two positions | refused (RFC 7946 §3.1.4) |
//! | a ring of fewer than four positions, or whose ends differ | refused (§3.1.6) |
//! | a `Polygon` with no rings — including one inside a `MultiPolygon` | refused |
//!
//! An **empty** `MultiPoint`/`MultiLineString`/`MultiPolygon` coordinate array
//! is *accepted* and yields zero parts: RFC 7946 permits an empty multi-part
//! geometry, and a writer that emitted one meant "nothing here". A `Polygon`
//! with zero rings is a different thing — a polygon has no meaning without an
//! exterior ring — and is refused.
//!
//! Nothing here panics on any input. `serde_json`'s own recursion limit stops a
//! hostile nest before the stack does, and this module's walk over the
//! already-parsed tree is bounded at the four levels a `MultiPolygon` has.
//!
//! # Coordinates: the one thing that must not be guessed
//!
//! This project's documents are **local planar kilometres (east, north)**, not
//! WGS84 — a procedurally generated world has no georeference. RFC 7946 says
//! the opposite about a document that does not say otherwise: §4 makes crs-less
//! coordinates WGS84 by definition. So the question "is this file in
//! kilometres?" has three answers here and **only one of them is silence**:
//!
//! * the document carries [`CRS_NOTE`] verbatim in its top-level `properties`
//!   → [`CrsClaim::PlanarKm`]. This is what `export_geojson` writes.
//! * the document carries a top-level `crs` member, naming *any* reference
//!   system → refused with [`GeoJsonError::ForeignCrs`]. This importer cannot
//!   reproject, and quietly reading degrees as kilometres would put a whole
//!   world inside one cell.
//! * neither → [`CrsClaim::Unstated`], carried on the result and **never
//!   defaulted**. The caller decides; the caller is told it is deciding.
//!
//! [`grid_xy`] converts a position to grid cells for a caller that has settled
//! that question, and is the exact inverse of `cartalith_spatial::geo::geo_xy`
//! up to that function's own three-decimal rounding — which is lossy, and
//! bounded: `geo_xy` rounds kilometres to 1e-3, so a recovered cell coordinate
//! is within `0.0005 / cell_km` cells of the original.
//!
//! # What this deliberately does not decide
//!
//! Reading a document is settled; *applying* one to a world is not, and this
//! module stops at that boundary rather than inventing an answer. An imported
//! settlement names a `faction` and a `factionName` this world may not have; an
//! imported territory polygon is an outline, where `CivData::territory` is a
//! per-cell raster; an imported `way` has no `WayType` beyond a string. Each of
//! those is a product decision, not a parsing one.

use crate::{js_num, json_string};
use serde_json::{Map, Value};

/// The `note` `export_geojson` writes into every document's top-level
/// `properties`, and the only positive evidence a document is in this project's
/// own planar kilometres.
///
/// **Duplicated deliberately**, and this is the copy that is *not* the source
/// of truth: the original is `cartalith_engine::geojson::CRS_NOTE`, which this
/// crate cannot reach — `cartalith-engine` depends on `cartalith-io`, not the
/// other way round. `the_reference_document_is_recognised_as_planar_km` pins
/// this copy against a verbatim excerpt of what the *reference implementation
/// itself* emitted, so a drift between the two crates fails a test rather than
/// silently reclassifying every Cartalith document as [`CrsClaim::Unstated`].
pub const CRS_NOTE: &str = "Coordinates are local planar kilometres (east, north) from the map's own scale, not real-world WGS84 longitude/latitude.";

/// One GeoJSON position: east, north. A third element (elevation) is dropped —
/// this world is 2D — and the fact that one was present is recorded once on
/// [`GeoJsonDoc::elevation_ignored`] rather than thrown away silently.
pub type Position = [f64; 2];

/// A linear ring. Closed (first position equals last) with at least four
/// positions, because [`parse_geojson`] refuses one that is not.
pub type Ring = Vec<Position>;

/// What the document says about its own coordinate reference system.
///
/// There is no `Wgs84` variant: a document that names a reference system is
/// refused at parse time rather than carried, because this importer has no
/// reprojection and a caller holding such a value would have nothing useful to
/// do with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrsClaim {
    /// The document carries [`CRS_NOTE`] verbatim: local planar kilometres.
    PlanarKm,
    /// The document said nothing. **Not** an assertion that it is in
    /// kilometres, and not an assertion that it is in degrees.
    Unstated,
}

/// The geometry types this port has a shape for. `GeometryCollection` is not
/// one of them and is refused by name.
#[derive(Debug, Clone, PartialEq)]
pub enum Geometry {
    Point(Position),
    MultiPoint(Vec<Position>),
    /// At least two positions.
    LineString(Vec<Position>),
    MultiLineString(Vec<Vec<Position>>),
    /// At least one ring.
    Polygon(Vec<Ring>),
    MultiPolygon(Vec<Vec<Ring>>),
}

impl Geometry {
    /// The GeoJSON `type` string this geometry came from, for a message or a
    /// summary readout.
    pub fn type_name(&self) -> &'static str {
        match self {
            Geometry::Point(_) => "Point",
            Geometry::MultiPoint(_) => "MultiPoint",
            Geometry::LineString(_) => "LineString",
            Geometry::MultiLineString(_) => "MultiLineString",
            Geometry::Polygon(_) => "Polygon",
            Geometry::MultiPolygon(_) => "MultiPolygon",
        }
    }

    /// Every position in the geometry, in document order.
    pub fn visit_positions(&self, f: &mut impl FnMut(Position)) {
        match self {
            Geometry::Point(p) => f(*p),
            Geometry::MultiPoint(ps) | Geometry::LineString(ps) => ps.iter().for_each(|p| f(*p)),
            Geometry::MultiLineString(ls) | Geometry::Polygon(ls) => {
                ls.iter().flatten().for_each(|p| f(*p))
            }
            Geometry::MultiPolygon(polys) => polys.iter().flatten().flatten().for_each(|p| f(*p)),
        }
    }
}

/// One `Feature`.
#[derive(Debug, Clone, PartialEq)]
pub struct GeoFeature {
    pub geometry: Geometry,
    /// The feature's `properties` **as written**, or `None` when the member was
    /// absent or `null`. Not flattened to an empty map: "this writer emits no
    /// properties" and "this feature happens to have none" are different facts,
    /// and only the reader can decide whether the difference matters.
    pub properties: Option<Map<String, Value>>,
}

impl GeoFeature {
    /// One property, or `None` when the feature has no properties at all or
    /// none by that name. Callers `match`; nothing is defaulted.
    pub fn prop(&self, key: &str) -> Option<&Value> {
        self.properties.as_ref()?.get(key)
    }

    /// `properties.layer`, the tag `export_geojson` puts on every feature
    /// (`settlement`, `poi`, `way`, `river`, `territory`, `province`). `None`
    /// for a foreign document that carries no such convention — which is
    /// normal, not an error.
    pub fn layer(&self) -> Option<&str> {
        self.prop("layer")?.as_str()
    }
}

/// A parsed `FeatureCollection`.
#[derive(Debug, Clone, PartialEq)]
pub struct GeoJsonDoc {
    /// See [`CrsClaim`]. Always known — the two variants are "it said km" and
    /// "it said nothing", and a document that said something else never gets
    /// this far.
    pub crs: CrsClaim,
    /// At least one position carried a third (elevation) component, which was
    /// dropped. A UI that wants to warn about lost data has this to warn on.
    pub elevation_ignored: bool,
    /// The collection's own top-level `properties`, or `None` when it had none.
    /// `export_geojson` always writes one (`generator`, `version`, `seed`,
    /// `mapWidthKm`, `note`); RFC 7946 does not require it, so its absence is
    /// information about the writer rather than a defect.
    pub properties: Option<Map<String, Value>>,
    pub features: Vec<GeoFeature>,
}

impl GeoJsonDoc {
    /// `(min_east, min_north, max_east, max_north)` over every position in the
    /// document, or `None` when it holds no positions at all.
    ///
    /// `None` rather than a zeroed box: an empty collection's extent is not the
    /// origin, and a readout printing `0, 0, 0, 0` would be indistinguishable
    /// from a real world one point wide at the corner.
    pub fn bounds(&self) -> Option<(f64, f64, f64, f64)> {
        let mut acc: Option<(f64, f64, f64, f64)> = None;
        for feat in &self.features {
            feat.geometry.visit_positions(&mut |[e, n]| {
                acc = Some(match acc {
                    None => (e, n, e, n),
                    Some((mine, minn, maxe, maxn)) => {
                        (mine.min(e), minn.min(n), maxe.max(e), maxn.max(n))
                    }
                });
            });
        }
        acc
    }
}

/// Everything that can stop an import, each carrying enough to act on.
#[derive(Debug, Clone, PartialEq)]
pub enum GeoJsonError {
    /// Not JSON. Carries serde_json's own message, which names line and column.
    Json(String),
    /// Valid JSON whose top level is an array, a number, a string, `true` or
    /// `null` — none of which can be a `FeatureCollection`.
    NotAnObject,
    /// A JSON object that is not a `FeatureCollection`. Carries what `type`
    /// said; **empty when the member was absent or was not a string**, the same
    /// encoding [`crate::LoadError::NotAProject`] uses for the same question,
    /// and unambiguous because `""` is not a GeoJSON type.
    NotAFeatureCollection(String),
    /// No `features` member, or one that is not an array.
    NoFeatureArray,
    /// A top-level `crs` member. Carries a compact rendering of whatever it
    /// said, so the message can name the projection the user has to convert
    /// from.
    ForeignCrs(String),
    /// Something inside one feature. `at` is a path within the feature, e.g.
    /// `geometry.coordinates[0][3]`.
    Feature { index: usize, at: String, reason: String },
}

impl std::fmt::Display for GeoJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeoJsonError::Json(m) => write!(f, "not JSON: {m}"),
            GeoJsonError::NotAnObject => write!(
                f,
                "the top level is not a JSON object, so this is not a GeoJSON FeatureCollection"
            ),
            GeoJsonError::NotAFeatureCollection(found) if found.is_empty() => write!(
                f,
                "the top level has no \"type\" member; a GeoJSON FeatureCollection must say \"type\":\"FeatureCollection\""
            ),
            GeoJsonError::NotAFeatureCollection(found) => write!(
                f,
                "the top level is a \"{found}\", not a \"FeatureCollection\"; import a whole collection, not a single feature or geometry"
            ),
            GeoJsonError::NoFeatureArray => write!(
                f,
                "no \"features\" array; a FeatureCollection must carry one (RFC 7946 section 3.3), even when it is empty"
            ),
            GeoJsonError::ForeignCrs(said) => write!(
                f,
                "this document declares a coordinate reference system ({said}); Cartalith GeoJSON is local planar kilometres and this importer cannot reproject"
            ),
            GeoJsonError::Feature { index, at, reason } => {
                write!(f, "feature {index} at {at}: {reason}")
            }
        }
    }
}

impl std::error::Error for GeoJsonError {}

/// Reads a GeoJSON `FeatureCollection`. See the module documentation for every
/// rule this refuses on, and for what it does **not** decide.
///
/// Never panics and never returns a partial document: either every feature
/// parsed, or the first fault comes back with its feature index.
pub fn parse_geojson(text: &str) -> Result<GeoJsonDoc, GeoJsonError> {
    // `serde_json::from_str` enforces its own 128-level recursion limit, so a
    // hostile nest is an error here rather than a blown stack.
    let root: Value = serde_json::from_str(text).map_err(|e| GeoJsonError::Json(e.to_string()))?;
    let Value::Object(root) = root else {
        return Err(GeoJsonError::NotAnObject);
    };

    match root.get("type").and_then(Value::as_str) {
        Some("FeatureCollection") => {}
        Some(other) => return Err(GeoJsonError::NotAFeatureCollection(other.to_string())),
        None => return Err(GeoJsonError::NotAFeatureCollection(String::new())),
    }

    if let Some(crs) = root.get("crs") {
        return Err(GeoJsonError::ForeignCrs(compact(crs)));
    }

    let Some(Value::Array(raw)) = root.get("features") else {
        return Err(GeoJsonError::NoFeatureArray);
    };

    let properties = object_member(&root, "properties");
    let crs = match properties.as_ref().and_then(|p| p.get("note")).and_then(Value::as_str) {
        Some(note) if note == CRS_NOTE => CrsClaim::PlanarKm,
        _ => CrsClaim::Unstated,
    };

    let mut elevation_ignored = false;
    let mut features = Vec::with_capacity(raw.len());
    for (index, item) in raw.iter().enumerate() {
        features.push(
            read_feature(item, &mut elevation_ignored)
                .map_err(|(at, reason)| GeoJsonError::Feature { index, at, reason })?,
        );
    }

    Ok(GeoJsonDoc { crs, elevation_ignored, properties, features })
}

/// The inverse of `cartalith_spatial::geo::geo_xy`: a planar-kilometre position
/// back to fractional grid cells.
///
/// `None` when `cell_km` is not a positive finite number, or the position is
/// not finite — a caller with no scale gets no coordinate rather than a
/// division by zero rendered as `inf`.
///
/// **Lossy, and bounded.** `geo_xy` rounds each kilometre value to three
/// decimals on the way out, so the recovered cell coordinate is within
/// `0.0005 / cell_km` cells of the one that was exported. It is not the
/// caller's job to remember that, so it is stated here and pinned by
/// `grid_xy_inverts_geo_xy_within_the_rounding_it_cannot_undo`.
pub fn grid_xy(east_km: f64, north_km: f64, gh: usize, cell_km: f64) -> Option<(f64, f64)> {
    if !(cell_km.is_finite() && cell_km > 0.0) || !east_km.is_finite() || !north_km.is_finite() {
        return None;
    }
    Some((east_km / cell_km, gh as f64 - north_km / cell_km))
}

// --- the walk ---------------------------------------------------------------

/// `JSON.stringify(v)` for an error message, through this crate's own writers
/// so a number in a diagnostic reads the way the same number does in the file.
fn compact(v: &Value) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.as_f64().map(js_num).unwrap_or_else(|| n.to_string()),
        Value::String(s) => json_string(s),
        Value::Array(a) => {
            let inner: Vec<String> = a.iter().map(compact).collect();
            format!("[{}]", inner.join(","))
        }
        Value::Object(o) => {
            let inner: Vec<String> =
                o.iter().map(|(k, v)| format!("{}:{}", json_string(k), compact(v))).collect();
            format!("{{{}}}", inner.join(","))
        }
    }
}

/// An object-valued member, or `None` when it is absent, `null`, or something
/// other than an object. RFC 7946 explicitly allows `properties` to be `null`.
fn object_member(o: &Map<String, Value>, key: &str) -> Option<Map<String, Value>> {
    match o.get(key) {
        Some(Value::Object(m)) => Some(m.clone()),
        _ => None,
    }
}

/// A path inside one feature, and the rule that path broke. Becomes
/// [`GeoJsonError::Feature`] once the caller knows the feature's index.
type Fault = (String, String);

fn fault(at: impl Into<String>, reason: impl Into<String>) -> Fault {
    (at.into(), reason.into())
}

fn read_feature(v: &Value, elevation_ignored: &mut bool) -> Result<GeoFeature, Fault> {
    let Value::Object(obj) = v else {
        return Err(fault(".", "a features array may only hold Feature objects"));
    };
    match obj.get("type").and_then(Value::as_str) {
        Some("Feature") => {}
        Some(other) => return Err(fault("type", format!("is \"{other}\", not \"Feature\""))),
        None => {
            return Err(fault("type", "is missing; every member of features must be a Feature"))
        }
    }

    let geometry = match obj.get("geometry") {
        None => return Err(fault("geometry", NO_GEOMETRY_MISSING)),
        Some(Value::Null) => return Err(fault("geometry", NO_GEOMETRY_NULL)),
        Some(g) => read_geometry(g, elevation_ignored)?,
    };

    // `"properties": null` and no `properties` member both become `None`;
    // anything that is not an object is a defect, not an absence.
    let properties = match obj.get("properties") {
        None | Some(Value::Null) => None,
        Some(Value::Object(m)) => Some(m.clone()),
        Some(other) => {
            return Err(fault(
                "properties",
                format!("is {}, which is neither an object nor null", compact(other)),
            ))
        }
    };

    Ok(GeoFeature { geometry, properties })
}

const NO_GEOMETRY_MISSING: &str = "is missing. RFC 7946 allows a Feature to carry a null geometry, but a feature with no shape cannot be placed on a map, so this importer refuses it rather than dropping it silently";
const NO_GEOMETRY_NULL: &str = "is null. RFC 7946 allows that, but a feature with no shape cannot be placed on a map, so this importer refuses it rather than dropping it silently";
const NO_GEOMETRY_COLLECTION: &str = "is \"GeometryCollection\", which this importer does not read; split it into one Feature per geometry";

fn read_geometry(v: &Value, elev: &mut bool) -> Result<Geometry, Fault> {
    let Value::Object(g) = v else {
        return Err(fault("geometry", format!("is {}, not an object", compact(v))));
    };
    let Some(kind) = g.get("type").and_then(Value::as_str) else {
        return Err(fault("geometry.type", "is missing or is not a string"));
    };
    if kind == "GeometryCollection" {
        return Err(fault("geometry.type", NO_GEOMETRY_COLLECTION));
    }
    let Some(coords) = g.get("coordinates") else {
        return Err(fault("geometry.coordinates", "is missing"));
    };
    let at = "geometry.coordinates";

    Ok(match kind {
        "Point" => Geometry::Point(read_position(coords, at, elev)?),
        "MultiPoint" => Geometry::MultiPoint(read_positions(coords, at, elev)?),
        "LineString" => Geometry::LineString(read_line(coords, at, elev)?),
        "MultiLineString" => {
            let mut out = Vec::new();
            for (i, part) in as_array(coords, at)?.iter().enumerate() {
                out.push(read_line(part, &format!("{at}[{i}]"), elev)?);
            }
            Geometry::MultiLineString(out)
        }
        "Polygon" => Geometry::Polygon(read_rings(coords, at, elev)?),
        "MultiPolygon" => {
            let mut out = Vec::new();
            for (i, poly) in as_array(coords, at)?.iter().enumerate() {
                out.push(read_rings(poly, &format!("{at}[{i}]"), elev)?);
            }
            Geometry::MultiPolygon(out)
        }
        other => {
            return Err(fault(
                "geometry.type",
                format!(
                    "is \"{other}\", which is not a GeoJSON geometry. Expected Point, MultiPoint, LineString, MultiLineString, Polygon or MultiPolygon"
                ),
            ))
        }
    })
}

fn as_array<'a>(v: &'a Value, at: &str) -> Result<&'a Vec<Value>, Fault> {
    match v {
        Value::Array(a) => Ok(a),
        other => Err(fault(at, format!("is {}, not an array", compact(other)))),
    }
}

fn read_positions(v: &Value, at: &str, elev: &mut bool) -> Result<Vec<Position>, Fault> {
    as_array(v, at)?
        .iter()
        .enumerate()
        .map(|(i, p)| read_position(p, &format!("{at}[{i}]"), elev))
        .collect()
}

fn read_line(v: &Value, at: &str, elev: &mut bool) -> Result<Vec<Position>, Fault> {
    let pts = read_positions(v, at, elev)?;
    if pts.len() < 2 {
        return Err(fault(
            at,
            format!("a LineString needs two or more positions, found {}", pts.len()),
        ));
    }
    Ok(pts)
}

/// One polygon's ring list. A polygon with no exterior ring is refused; an
/// empty *multi-part* coordinate array is the caller's business, not this
/// function's.
fn read_rings(v: &Value, at: &str, elev: &mut bool) -> Result<Vec<Ring>, Fault> {
    let raw = as_array(v, at)?;
    if raw.is_empty() {
        return Err(fault(at, "a Polygon needs an exterior ring; this one has no rings at all"));
    }
    let mut out = Vec::with_capacity(raw.len());
    for (i, r) in raw.iter().enumerate() {
        let at = format!("{at}[{i}]");
        let ring = read_positions(r, &at, elev)?;
        if ring.len() < 4 {
            return Err(fault(
                &at,
                format!(
                    "a linear ring needs four or more positions, found {} (RFC 7946 section 3.1.6)",
                    ring.len()
                ),
            ));
        }
        let (first, last) = (ring[0], ring[ring.len() - 1]);
        if first != last {
            return Err(fault(
                &at,
                format!(
                    "a linear ring must close: it starts at [{},{}] and ends at [{},{}]",
                    js_num(first[0]),
                    js_num(first[1]),
                    js_num(last[0]),
                    js_num(last[1])
                ),
            ));
        }
        out.push(ring);
    }
    Ok(out)
}

fn read_position(v: &Value, at: &str, elev: &mut bool) -> Result<Position, Fault> {
    let a = as_array(v, at)?;
    if a.len() < 2 {
        return Err(fault(
            at,
            format!("a position needs at least an east and a north value, found {}", a.len()),
        ));
    }
    if a.len() > 2 {
        *elev = true;
    }
    let mut out = [0.0f64; 2];
    for (i, slot) in out.iter_mut().enumerate() {
        let Some(n) = a[i].as_f64() else {
            return Err(fault(format!("{at}[{i}]"), format!("is {}, not a number", compact(&a[i]))));
        };
        // Unreachable through serde_json as this workspace builds it: the
        // parser refuses an out-of-range literal (`1e400`) before a Value is
        // made, and `Number::from_f64` will not hold a non-finite one. Kept
        // because `as_f64` *can* return one if serde_json's
        // `arbitrary_precision` feature is ever unified in by another crate,
        // and an infinite coordinate reaching a caller would be worse than a
        // dead branch. No crate enables it today.
        if !n.is_finite() {
            return Err(fault(format!("{at}[{i}]"), "is not a finite number"));
        }
        *slot = n;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal well-formed document, so every rejection test below differs
    /// from an accepted one in exactly the thing it is testing.
    fn doc(features: &str) -> String {
        format!(r#"{{"type":"FeatureCollection","features":[{features}]}}"#)
    }

    fn feat(geometry: &str) -> String {
        format!(r#"{{"type":"Feature","geometry":{geometry},"properties":{{}}}}"#)
    }

    fn refusal(text: &str) -> String {
        parse_geojson(text).expect_err("must be refused").to_string()
    }

    #[test]
    fn the_minimal_document_this_suite_perturbs_is_itself_accepted() {
        let ok = parse_geojson(&doc(&feat(r#"{"type":"Point","coordinates":[1,2]}"#))).unwrap();
        assert_eq!(ok.features.len(), 1);
        assert_eq!(ok.features[0].geometry, Geometry::Point([1.0, 2.0]));
        assert_eq!(ok.crs, CrsClaim::Unstated, "it carries no note, and none is invented");
        assert!(!ok.elevation_ignored);
    }

    #[test]
    fn text_that_is_not_json_is_refused_with_a_position() {
        let e = refusal("{not json");
        assert!(e.starts_with("not JSON:"), "{e}");
        assert!(e.contains("line 1"), "serde_json names where: {e}");
    }

    #[test]
    fn json_that_is_not_an_object_cannot_be_a_collection() {
        for text in ["[1,2,3]", "42", r#""a string""#, "null", "true"] {
            assert_eq!(parse_geojson(text), Err(GeoJsonError::NotAnObject), "{text}");
        }
    }

    #[test]
    fn valid_json_that_is_not_geojson_says_so_rather_than_returning_nothing() {
        // The case the backlog row names explicitly: a file that parses cleanly
        // and is simply a different kind of document.
        let e = refusal(r#"{"name":"config","values":[1,2,3]}"#);
        assert!(e.contains("no \"type\" member"), "{e}");
        assert_eq!(
            parse_geojson(r#"{"name":"config"}"#),
            Err(GeoJsonError::NotAFeatureCollection(String::new()))
        );
    }

    #[test]
    fn a_bare_feature_or_geometry_is_refused_and_named() {
        let e = refusal(r#"{"type":"Feature","geometry":null}"#);
        assert!(e.contains("is a \"Feature\", not a \"FeatureCollection\""), "{e}");
        let e = refusal(r#"{"type":"Point","coordinates":[1,2]}"#);
        assert!(e.contains("is a \"Point\""), "{e}");
        // A non-string `type` is the same absence as no `type` at all.
        assert_eq!(
            parse_geojson(r#"{"type":7,"features":[]}"#),
            Err(GeoJsonError::NotAFeatureCollection(String::new()))
        );
    }

    #[test]
    fn a_collection_with_no_features_array_is_refused() {
        assert_eq!(
            parse_geojson(r#"{"type":"FeatureCollection"}"#),
            Err(GeoJsonError::NoFeatureArray)
        );
        assert_eq!(
            parse_geojson(r#"{"type":"FeatureCollection","features":{}}"#),
            Err(GeoJsonError::NoFeatureArray)
        );
        // An *empty* array is a real, valid document and is not refused.
        let empty = parse_geojson(r#"{"type":"FeatureCollection","features":[]}"#).unwrap();
        assert!(empty.features.is_empty());
        assert_eq!(empty.bounds(), None, "no positions, so no extent -- not a zeroed box");
    }

    #[test]
    fn a_declared_coordinate_reference_system_is_refused_rather_than_read_as_kilometres() {
        // The whole point of the module: degrees silently read as kilometres
        // would put an entire world inside one cell.
        let text = r#"{"type":"FeatureCollection","crs":{"type":"name","properties":{"name":"urn:ogc:def:crs:OGC:1.3:CRS84"}},"features":[]}"#;
        let e = refusal(text);
        assert!(e.contains("declares a coordinate reference system"), "{e}");
        assert!(e.contains("urn:ogc:def:crs:OGC:1.3:CRS84"), "the message names it: {e}");
        assert!(e.contains("cannot reproject"), "{e}");
    }

    #[test]
    fn a_document_without_the_note_is_unstated_and_not_assumed_to_be_kilometres() {
        let with = format!(
            r#"{{"type":"FeatureCollection","properties":{{"note":{}}},"features":[]}}"#,
            json_string(CRS_NOTE)
        );
        assert_eq!(parse_geojson(&with).unwrap().crs, CrsClaim::PlanarKm);

        // Every other shape of silence is Unstated, and none of them is an error.
        for props in [r#"{}"#, r#"{"note":"something else"}"#, r#"{"note":7}"#, "null"] {
            let text =
                format!(r#"{{"type":"FeatureCollection","properties":{props},"features":[]}}"#);
            assert_eq!(parse_geojson(&text).unwrap().crs, CrsClaim::Unstated, "{props}");
        }
    }

    #[test]
    fn a_feature_with_no_geometry_is_refused_rather_than_dropped() {
        let null_geometry = doc(r#"{"type":"Feature","geometry":null,"properties":{}}"#);
        let no_member = doc(r#"{"type":"Feature","properties":{}}"#);
        for text in [null_geometry, no_member] {
            let e = refusal(&text);
            assert!(e.starts_with("feature 0 at geometry:"), "{e}");
            assert!(e.contains("cannot be placed on a map"), "{e}");
            assert!(e.contains("rather than dropping it silently"), "{e}");
        }
    }

    #[test]
    fn a_features_array_holding_something_other_than_a_feature_is_refused() {
        assert!(refusal(&doc("7")).contains("may only hold Feature objects"));
        let e = refusal(&doc(r#"{"type":"Point","coordinates":[1,2]}"#));
        assert!(e.contains("feature 0 at type: is \"Point\", not \"Feature\""), "{e}");
        let e = refusal(&doc(r#"{"geometry":{"type":"Point","coordinates":[1,2]}}"#));
        assert!(e.contains("type: is missing"), "{e}");
    }

    #[test]
    fn a_geometry_collection_is_refused_by_name_with_a_way_forward() {
        let e = refusal(&doc(&feat(r#"{"type":"GeometryCollection","geometries":[]}"#)));
        assert!(e.contains("\"GeometryCollection\""), "{e}");
        assert!(e.contains("one Feature per geometry"), "{e}");
    }

    #[test]
    fn an_unknown_geometry_type_lists_the_ones_that_would_work() {
        let e = refusal(&doc(&feat(r#"{"type":"Sphere","coordinates":[1,2]}"#)));
        assert!(e.contains("is \"Sphere\", which is not a GeoJSON geometry"), "{e}");
        assert!(e.contains("MultiPolygon"), "{e}");
    }

    #[test]
    fn a_coordinate_that_is_not_a_number_is_refused_at_its_own_index() {
        let e = refusal(&doc(&feat(r#"{"type":"Point","coordinates":["12",3]}"#)));
        assert_eq!(e, "feature 0 at geometry.coordinates[0]: is \"12\", not a number");
        let e = refusal(&doc(&feat(r#"{"type":"Point","coordinates":[12,null]}"#)));
        assert_eq!(e, "feature 0 at geometry.coordinates[1]: is null, not a number");
        let e = refusal(&doc(&feat(r#"{"type":"LineString","coordinates":[[0,0],[1,{"x":2}]]}"#)));
        assert_eq!(e, "feature 0 at geometry.coordinates[1][1]: is {\"x\":2}, not a number");
    }

    #[test]
    fn a_short_position_is_refused_with_its_length() {
        let e = refusal(&doc(&feat(r#"{"type":"Point","coordinates":[12]}"#)));
        assert!(e.contains("a position needs at least an east and a north value, found 1"), "{e}");
        let e = refusal(&doc(&feat(r#"{"type":"Point","coordinates":[]}"#)));
        assert!(e.contains("found 0"), "{e}");
        let e = refusal(&doc(&feat(r#"{"type":"Point","coordinates":7}"#)));
        assert!(e.contains("is 7, not an array"), "{e}");
    }

    #[test]
    fn a_non_finite_coordinate_is_refused_rather_than_carried_as_infinity() {
        // `1e400` is legal JSON syntax and has no finite f64. Whatever the JSON
        // layer makes of it, it must not reach a caller as a position.
        // Measured, not assumed: serde_json refuses the literal outright, so
        // the fault surfaces as a JSON error and not as a position fault. That
        // is why `read_position`'s own `is_finite` guard is documented as
        // unreachable rather than as the thing that catches this.
        let e = refusal(&doc(&feat(r#"{"type":"Point","coordinates":[1e400,0]}"#)));
        assert_eq!(e, "not JSON: number out of range at line 1 column 105");
    }

    #[test]
    fn a_one_point_linestring_is_refused() {
        let e = refusal(&doc(&feat(r#"{"type":"LineString","coordinates":[[0,0]]}"#)));
        assert!(e.contains("a LineString needs two or more positions, found 1"), "{e}");
        // And inside a MultiLineString, named by part.
        let e = refusal(&doc(&feat(
            r#"{"type":"MultiLineString","coordinates":[[[0,0],[1,1]],[[2,2]]]}"#,
        )));
        assert!(e.contains("geometry.coordinates[1]: a LineString needs two or more"), "{e}");
    }

    #[test]
    fn a_ring_that_does_not_close_is_refused_and_both_ends_are_quoted() {
        let e =
            refusal(&doc(&feat(r#"{"type":"Polygon","coordinates":[[[0,0],[4,0],[4,4],[0,4]]]}"#)));
        assert_eq!(
            e,
            "feature 0 at geometry.coordinates[0]: a linear ring must close: it starts at [0,0] and ends at [0,4]"
        );
    }

    #[test]
    fn a_ring_of_fewer_than_four_positions_is_refused() {
        let e = refusal(&doc(&feat(r#"{"type":"Polygon","coordinates":[[[0,0],[4,0],[0,0]]]}"#)));
        assert!(e.contains("needs four or more positions, found 3"), "{e}");
        assert!(e.contains("3.1.6"), "the message cites the rule: {e}");
    }

    #[test]
    fn a_polygon_with_no_rings_is_refused_wherever_it_appears() {
        let e = refusal(&doc(&feat(r#"{"type":"Polygon","coordinates":[]}"#)));
        assert!(e.contains("needs an exterior ring"), "{e}");
        // The case the backlog row names: a MultiPolygon holding one ringless
        // polygon.
        let e = refusal(&doc(&feat(r#"{"type":"MultiPolygon","coordinates":[[]]}"#)));
        assert_eq!(
            e,
            "feature 0 at geometry.coordinates[0]: a Polygon needs an exterior ring; this one has no rings at all"
        );
    }

    #[test]
    fn an_empty_multi_part_geometry_is_accepted_because_rfc_7946_permits_one() {
        // The distinction the previous test depends on: `[]` at the top of a
        // MultiPolygon is "no polygons", which is legal; `[[]]` is "a polygon
        // with no exterior ring", which is not.
        for (kind, want) in [
            ("MultiPolygon", Geometry::MultiPolygon(vec![])),
            ("MultiLineString", Geometry::MultiLineString(vec![])),
            ("MultiPoint", Geometry::MultiPoint(vec![])),
        ] {
            let text = doc(&feat(&format!(r#"{{"type":"{kind}","coordinates":[]}}"#)));
            let d = parse_geojson(&text).unwrap_or_else(|e| panic!("{kind}: {e}"));
            assert_eq!(d.features[0].geometry, want);
            assert_eq!(d.bounds(), None, "{kind} holds no positions");
        }
    }

    #[test]
    fn a_polygon_with_a_hole_keeps_the_rings_in_the_order_they_were_written() {
        let text = doc(&feat(
            r#"{"type":"Polygon","coordinates":[[[0,0],[9,0],[9,9],[0,9],[0,0]],[[3,3],[6,3],[6,6],[3,3]]]}"#,
        ));
        let Geometry::Polygon(rings) = &parse_geojson(&text).unwrap().features[0].geometry else {
            panic!("a Polygon");
        };
        assert_eq!(rings.len(), 2);
        assert_eq!(rings[0][1], [9.0, 0.0], "the exterior ring stays first");
        assert_eq!(rings[1][0], [3.0, 3.0]);
    }

    #[test]
    fn a_third_coordinate_is_dropped_and_the_document_says_that_it_was() {
        let d = parse_geojson(&doc(&feat(r#"{"type":"Point","coordinates":[1,2,850]}"#))).unwrap();
        assert_eq!(d.features[0].geometry, Geometry::Point([1.0, 2.0]));
        assert!(d.elevation_ignored, "dropping a value silently is what this flag exists to stop");
    }

    #[test]
    fn properties_absent_and_properties_null_are_both_absent_and_neither_is_empty() {
        for props in ["", r#","properties":null"#] {
            let text = doc(&format!(
                r#"{{"type":"Feature","geometry":{{"type":"Point","coordinates":[0,0]}}{props}}}"#
            ));
            let d = parse_geojson(&text).unwrap();
            assert_eq!(d.features[0].properties, None, "{props:?}");
            assert_eq!(d.features[0].prop("name"), None);
            assert_eq!(d.features[0].layer(), None);
        }
        // Something that is neither an object nor null is a defect, not absence.
        let text = doc(
            r#"{"type":"Feature","geometry":{"type":"Point","coordinates":[0,0]},"properties":7}"#,
        );
        assert!(refusal(&text).contains("neither an object nor null"));
    }

    #[test]
    fn a_repeated_property_key_keeps_the_last_one_as_json_parse_does() {
        let text = doc(
            r#"{"type":"Feature","geometry":{"type":"Point","coordinates":[0,0]},"properties":{"name":"first","name":"second"}}"#,
        );
        let d = parse_geojson(&text).unwrap();
        assert_eq!(d.features[0].prop("name").and_then(Value::as_str), Some("second"));
    }

    #[test]
    fn a_fault_names_the_feature_it_is_in_not_just_the_first() {
        let good = feat(r#"{"type":"Point","coordinates":[0,0]}"#);
        let bad = feat(r#"{"type":"Point","coordinates":[0,"north"]}"#);
        let text = doc(&format!("{good},{good},{bad}"));
        let Err(GeoJsonError::Feature { index, .. }) = parse_geojson(&text) else {
            panic!("must be refused");
        };
        assert_eq!(index, 2);
    }

    #[test]
    fn a_deeply_nested_document_errors_rather_than_exhausting_the_stack() {
        // A hostile file, and the reason this module does not hand-roll a JSON
        // reader: serde_json carries its own recursion limit.
        let text = format!("{}{}", "[".repeat(20_000), "]".repeat(20_000));
        assert!(matches!(parse_geojson(&text), Err(GeoJsonError::Json(_))));
    }

    #[test]
    fn bounds_spans_every_geometry_in_the_document() {
        let a = feat(r#"{"type":"Point","coordinates":[10,-4]}"#);
        let b = feat(r#"{"type":"LineString","coordinates":[[0,0],[-3,50]]}"#);
        let c = feat(r#"{"type":"MultiPolygon","coordinates":[[[[1,1],[2,1],[2,2],[1,1]]]]}"#);
        let d = parse_geojson(&doc(&format!("{a},{b},{c}"))).unwrap();
        assert_eq!(d.bounds(), Some((-3.0, -4.0, 10.0, 50.0)));
    }

    #[test]
    fn grid_xy_refuses_a_scale_it_cannot_divide_by() {
        for bad in [0.0, -50.0, f64::NAN, f64::INFINITY] {
            assert_eq!(grid_xy(100.0, 300.0, 9, bad), None, "cell_km {bad}");
        }
        assert_eq!(grid_xy(f64::NAN, 0.0, 9, 50.0), None);
        assert_eq!(grid_xy(0.0, f64::INFINITY, 9, 50.0), None);
    }

    #[test]
    fn grid_xy_inverts_geo_xy_within_the_rounding_it_cannot_undo() {
        // `geo_xy` rounds kilometres to three decimals, so the bound is
        // 0.0005 / cell_km cells -- asserted against that arithmetic, not
        // against a number read off a run.
        let gh = 37usize;
        for &cell_km in &[0.7_f64, 12.5, 50.0] {
            let tol = 0.0005 / cell_km;
            for &(gx, gy) in &[(0.0, 0.0), (1.0 / 3.0, 2.0 / 7.0), (11.25, 36.5), (123.456, 0.75)] {
                let [e, n] = cartalith_spatial::geo::geo_xy(gx, gy, gh, cell_km);
                let (bx, by) = grid_xy(e, n, gh, cell_km).expect("a positive finite scale");
                assert!((bx - gx).abs() <= tol, "x {gx} -> {e} -> {bx} (tol {tol})");
                assert!((by - gy).abs() <= tol, "y {gy} -> {n} -> {by} (tol {tol})");
            }
        }
    }

    #[test]
    fn every_geometry_kind_reports_its_own_type_name() {
        let cases = [
            (r#"{"type":"Point","coordinates":[0,0]}"#, "Point"),
            (r#"{"type":"MultiPoint","coordinates":[[0,0]]}"#, "MultiPoint"),
            (r#"{"type":"LineString","coordinates":[[0,0],[1,1]]}"#, "LineString"),
            (r#"{"type":"MultiLineString","coordinates":[[[0,0],[1,1]]]}"#, "MultiLineString"),
            (r#"{"type":"Polygon","coordinates":[[[0,0],[1,0],[1,1],[0,0]]]}"#, "Polygon"),
            (
                r#"{"type":"MultiPolygon","coordinates":[[[[0,0],[1,0],[1,1],[0,0]]]]}"#,
                "MultiPolygon",
            ),
        ];
        for (json, name) in cases {
            let d = parse_geojson(&doc(&feat(json))).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(d.features[0].geometry.type_name(), name);
        }
    }
}
