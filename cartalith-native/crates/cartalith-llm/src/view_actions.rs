//! View and layer actions the model may PROPOSE (`LLM_ASSISTANT_SCOPE.md` s.5b.8 item 12 and
//! the LLM-6 row; milestone LLM-6d; owner Rulings DD and DI): switch a map layer on or off,
//! pick a style preset, centre and zoom the map on a cell, show or hide a landmark type, open
//! a read-only window. The second tool of the Commands conversation, `propose_view_actions`.
//!
//! WHY a second tool and not more keys in `set_generation_params`: a setting is a number in a
//! registry with a range, validated by V1-V13; a view action is a verb with typed arguments
//! over the app's session state. They have different undo semantics (below) and different
//! failure modes, so they get a different card and a different rule set (`A1`-`A9`), and a
//! reply can carry both ("make it drier and show me the east coast").
//!
//! ## The tool catalogue is built from the real app, every request
//!
//! The shell hands over a JSON document of FACTS (`ViewFacts`): the layer ids it can switch
//! with their current state, the style presets, the landmark types, the windows, the grid
//! size and the camera's zoom range. The facts for `open_window` are the rows of
//! `CommandIndex` (`shell/command_index.gd`) that the index marks `pinnable` and `available`
//! AND that a curated table of read-only windows names (`shell/llm_view.gd`), so a menu row
//! that disappears from the build disappears from the enum. An action whose facts are absent
//! is not offered at all: its id is not in the `action` enum, and a call naming it is
//! refused with the shell's own reason (`withheld`), never run.
//!
//! The ACTION VOCABULARY is fixed in this file ([`ACTION_IDS`]) and each action's argument
//! names and types are fixed here, so a shell bug can add a window to the enum but cannot add
//! "generate" to the vocabulary, give an action a free-form command string or widen an
//! argument's type. [`FORBIDDEN_STEMS`] is a second wall: a fact whose id contains a word for
//! generating, saving, deleting, exporting, importing, closing, resetting or the network is
//! dropped (and counted in [`ViewCatalogue::dropped`]), whatever the shell said.
//!
//! ## What "undo" means here (the owner asked for it to be defined)
//!
//! * `set_style_preset` is the one action with an undo step. It is applied through the
//!   existing look history, inside the LLM-5 grouped undo, so Edit > Undo reverts it in ONE
//!   step. A preset that also hides overlay layers (`PRESET_LAYERS_OFF`) says so on its row;
//!   Edit > Undo restores the picture, not those switches (the same rule as pressing the tile).
//!   A style change (and its undo) redraws the whole map, and the app's redraw re-fits the camera
//!   (`ViewportHost.refresh`), measured 2026-10-26 by `_llmview_probe.gd`; so a proposal that
//!   carries a style AND a camera move applies the style first and the camera after it, and an
//!   undo of the style leaves the view re-fitted, not where the camera action put it.
//! * Layer visibility, landmark filters, camera moves and opened windows are SESSION STATE.
//!   They are instant and are NOT on the undo stack: Edit > Undo does not move them. The card
//!   says so beside Apply and lists each row's old value so the user can reverse it by hand.
//!   (Defined so on purpose: putting a camera move on the same stack as a heightmap edit
//!   would make Ctrl+Z after "zoom to the delta" undo the sculpt before it.)
//!
//! MUST NEVER: run anything (this module has no setter and no handle to the app); clamp,
//! round or coerce a model's value (a number outside its range, a `1` for `true` and a
//! `"true"` string are refused); accept an action id or an argument value that is not in the
//! offered set; turn the model's text into anything but inert display text.

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use crate::catalogue::{depth, edit_distance, inert_text};
use crate::openai::ToolSpec;

/// The tool's name. Its description says plainly that nothing happens until the user presses Apply.
pub const VIEW_TOOL: &str = "propose_view_actions";

/// The whole action vocabulary, fixed in code. A labelled design choice (not a measurement):
/// five verbs cover the owner's list (layer visibility, a render look, centre/zoom, landmark
/// filters, open a window) and none of them can change the world's data.
pub const ACTION_IDS: [&str; 5] = ["set_layer_visible", "set_style_preset", "zoom_to", "set_landmark_filter", "open_window"];

/// Most actions one proposal may carry. Labelled judgement: the largest sensible request
/// ("show only rivers and landmarks, centre on the delta, open the vault") is about six; a
/// longer list is refused whole, never truncated (a cut list is a different proposal).
pub const MAX_VIEW_ACTIONS: usize = 8;

/// A1: largest `arguments` text accepted, in bytes. Eight actions with 240-character reasons
/// are about 3 KB; 8 KB leaves room for the summary and no room for a payload.
pub const MAX_VIEW_ARGUMENT_BYTES: usize = 8 * 1024;

/// A1: deepest nesting accepted. The legal shape `{"actions":[{"args":{"x":1}}]}` is depth 4 as
/// [`depth`] counts it (the top object is 0); one more level is not part of the shape.
pub const MAX_VIEW_DEPTH: usize = 4;

/// Longest model-written reason or summary kept, in characters (the parameter card's bound).
pub const MAX_VIEW_REASON_CHARS: usize = 240;

/// Most rows of each fact list the shell may hand over. Labelled judgement: above the real
/// counts on 2026-10-26 (10 layers, 16 presets, about 45 landmark types, 10 windows) with room
/// to grow; a list over it is cut to it and the cut is counted in [`ViewCatalogue::dropped`].
pub const MAX_FACT_ROWS: usize = 64;

/// Longest id the shell may use for a layer, preset, type or window.
pub const MAX_FACT_ID_CHARS: usize = 40;

/// Words that may never appear in an offered id (see the module docs). Each stem is a
/// labelled judgement from the owner's list ("no generate, no open/close/save, no delete, no
/// export, no network"): `open` is deliberately NOT here (`open_window` is the point), but
/// `open_project` is, and so are `close`, `quit` and `new_world`.
pub const FORBIDDEN_STEMS: [&str; 22] = [
    "generat", "regenerat", "save", "delete", "remove", "erase", "export", "import", "close", "quit", "reset", "clear", "revert",
    "undo", "redo", "upload", "download", "network", "http", "open_project", "new_world", "finalize",
];

/// The validator's rules for a view proposal, numbered like the parameter rules (`V1`..).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewRule {
    /// Not JSON, not an object, too deep, too large.
    A1,
    /// `actions` missing, not an array, empty, or an unexpected top-level field.
    A2,
    /// More than [`MAX_VIEW_ACTIONS`] actions.
    A3,
    /// An entry is malformed (not an object, an unexpected field, `action` not a string, `args` not an object).
    A4,
    /// The action is not in the vocabulary, or is not offered right now.
    A5,
    /// An argument is unknown, or a required one is missing.
    A6,
    /// An argument has the wrong type (a number for a bool, a string for a number, ...).
    A7,
    /// An argument value is not one of the offered values, or a number is out of range.
    A8,
    /// The same target twice in one proposal.
    A9,
}

impl ViewRule {
    /// `"A1"` .. `"A9"`.
    pub fn id(self) -> &'static str {
        match self {
            ViewRule::A1 => "A1",
            ViewRule::A2 => "A2",
            ViewRule::A3 => "A3",
            ViewRule::A4 => "A4",
            ViewRule::A5 => "A5",
            ViewRule::A6 => "A6",
            ViewRule::A7 => "A7",
            ViewRule::A8 => "A8",
            ViewRule::A9 => "A9",
        }
    }
}

/// One fact row: an id the model may name, its label and, where it has one, its state now.
#[derive(Debug, Clone, PartialEq)]
pub struct FactRow {
    /// `[a-z0-9_]{1,40}`.
    pub id: String,
    /// Display label (inert).
    pub label: String,
    /// Current state for rows that have one (layers, landmark types: shown or hidden; presets: the active one).
    pub on: Option<bool>,
    /// One sentence the card prints under the row (a preset that hides layers), inert; may be empty.
    pub note: String,
}

/// The camera facts: the grid and the zoom range. Absent as a whole when no world is open.
#[derive(Debug, Clone, PartialEq)]
pub struct CameraFacts {
    /// Grid width in cells.
    pub w: u32,
    /// Grid height in cells.
    pub h: u32,
    /// Smallest zoom factor the viewport takes.
    pub zoom_min: f64,
    /// Largest zoom factor the viewport takes.
    pub zoom_max: f64,
    /// The zoom now, when readable (omitted otherwise, never 0).
    pub zoom_now: Option<f64>,
}

/// What the shell says about the app right now. See the module docs.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ViewFacts {
    /// Layers `ViewportHost.set_layer_visible` takes.
    pub layers: Vec<FactRow>,
    /// Style presets (the base look of each, no variants).
    pub presets: Vec<FactRow>,
    /// Landmark types the filter switches.
    pub kinds: Vec<FactRow>,
    /// Windows: the index rows that are pinnable, available and on the curated read-only list.
    pub windows: Vec<FactRow>,
    /// The camera, when a world is open.
    pub camera: Option<CameraFacts>,
    /// Why an action is not offered, by action id (the shell's own words).
    pub withheld: BTreeMap<String, String>,
}

/// The catalogue the validator and the tool spec read: the facts that survived the checks.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewCatalogue {
    facts: ViewFacts,
    /// Facts refused by the checks above (a bad id, a forbidden stem, a cut list), as text for
    /// the tests and the probe; empty on a healthy build.
    pub dropped: Vec<String>,
}

fn id_ok(id: &str) -> bool {
    !id.is_empty() && id.chars().count() <= MAX_FACT_ID_CHARS && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// The first forbidden stem inside `id`, if any. The word `preset` is removed first: it contains
/// the stem `reset` ("p-reset"), and a style preset is an offered thing, not a reset. Every other
/// match is a plain substring (`autosave` is caught by `save`, deliberately: a false positive
/// costs one dropped fact, a false negative costs a destructive row in the enum).
fn forbidden(id: &str) -> Option<&'static str> {
    let probe = id.replace("preset", "");
    FORBIDDEN_STEMS.iter().copied().find(|s| probe.contains(s))
}

fn rows_from(v: Option<&Value>, what: &str, dropped: &mut Vec<String>) -> Vec<FactRow> {
    let Some(arr) = v.and_then(Value::as_array) else { return Vec::new() };
    let mut out: Vec<FactRow> = Vec::new();
    for (i, r) in arr.iter().enumerate() {
        if i >= MAX_FACT_ROWS {
            dropped.push(format!("{what}: more than {MAX_FACT_ROWS} rows; the rest were cut"));
            break;
        }
        let Some(o) = r.as_object() else {
            dropped.push(format!("{what}: row {i} is not an object"));
            continue;
        };
        let Some(id) = o.get("id").and_then(Value::as_str) else {
            dropped.push(format!("{what}: row {i} has no id"));
            continue;
        };
        if !id_ok(id) {
            dropped.push(format!("{what}: id '{}' is not [a-z0-9_]{{1,{MAX_FACT_ID_CHARS}}}", inert_text(id, 48)));
            continue;
        }
        if let Some(s) = forbidden(id) {
            dropped.push(format!("{what}: id '{id}' contains the forbidden word '{s}'"));
            continue;
        }
        if out.iter().any(|x| x.id == id) {
            dropped.push(format!("{what}: id '{id}' twice"));
            continue;
        }
        let label = inert_text(o.get("label").and_then(Value::as_str).unwrap_or(id), 60);
        out.push(FactRow {
            id: id.to_owned(),
            label: if label.is_empty() { id.to_owned() } else { label },
            on: o.get("on").and_then(Value::as_bool),
            note: inert_text(o.get("note").and_then(Value::as_str).unwrap_or(""), 200),
        });
    }
    out
}

impl ViewCatalogue {
    /// Build from the shell's JSON facts. `Err` only when the text is not a JSON object; every
    /// bad row is dropped and listed in [`ViewCatalogue::dropped`] instead of failing the request.
    pub fn from_json_text(text: &str) -> Result<Self, String> {
        let v: Value = serde_json::from_str(text).map_err(|_| "the view facts are not JSON".to_string())?;
        Self::from_json(&v)
    }

    /// As [`ViewCatalogue::from_json_text`] over a parsed value.
    pub fn from_json(v: &Value) -> Result<Self, String> {
        let Some(o) = v.as_object() else { return Err("the view facts are not an object".into()) };
        let mut dropped = Vec::new();
        let mut facts = ViewFacts {
            layers: rows_from(o.get("layers"), "layers", &mut dropped),
            presets: rows_from(o.get("presets"), "presets", &mut dropped),
            kinds: rows_from(o.get("kinds"), "kinds", &mut dropped),
            windows: rows_from(o.get("windows"), "windows", &mut dropped),
            camera: None,
            withheld: BTreeMap::new(),
        };
        if o.get("camera").is_some_and(|c| !c.is_object()) {
            dropped.push("camera: not an object".into());
        }
        if let Some(c) = o.get("camera").and_then(Value::as_object) {
            let n = |k: &str| c.get(k).and_then(Value::as_f64).filter(|x| x.is_finite());
            let g = |k: &str| c.get(k).and_then(Value::as_u64).filter(|x| *x >= 1 && *x <= 1_000_000);
            match (g("w"), g("h"), n("zoom_min"), n("zoom_max")) {
                (Some(w), Some(h), Some(lo), Some(hi)) if lo > 0.0 && lo < hi => {
                    facts.camera = Some(CameraFacts { w: w as u32, h: h as u32, zoom_min: lo, zoom_max: hi, zoom_now: n("zoom_now") });
                }
                _ => dropped.push("camera: needs w, h >= 1 and 0 < zoom_min < zoom_max".into()),
            }
        }
        if let Some(w) = o.get("withheld").and_then(Value::as_object) {
            for (k, why) in w {
                if ACTION_IDS.contains(&k.as_str()) {
                    if let Some(s) = why.as_str() {
                        facts.withheld.insert(k.clone(), inert_text(s, 160));
                    }
                }
            }
        }
        Ok(ViewCatalogue { facts, dropped })
    }

    /// The facts that survived (tests, the prompt).
    pub fn facts(&self) -> &ViewFacts {
        &self.facts
    }

    /// The action ids offered now: those whose facts are non-empty, in vocabulary order.
    pub fn offered(&self) -> Vec<&'static str> {
        ACTION_IDS
            .iter()
            .copied()
            .filter(|id| match *id {
                "set_layer_visible" => !self.facts.layers.is_empty(),
                "set_style_preset" => !self.facts.presets.is_empty(),
                "zoom_to" => self.facts.camera.is_some(),
                "set_landmark_filter" => !self.facts.kinds.is_empty(),
                "open_window" => !self.facts.windows.is_empty(),
                _ => false,
            })
            .collect()
    }

    /// True when at least one action is offered (otherwise the tool is not in the request).
    pub fn is_offered(&self) -> bool {
        !self.offered().is_empty()
    }

    /// The tool, its `action` an enum of the offered ids and its description listing each
    /// offered action with its arguments and the values they take.
    pub fn tool_spec(&self) -> ToolSpec {
        let ids = self.offered();
        // Each value with its state NOW where the shell read one, so the model knows which layer
        // is on without a lookup tool: `roads (now shown)`. The state words are not part of the
        // value: `args` takes the bare id, and the enum of the call is the bare ids.
        let list = |rows: &[FactRow], words: [&str; 2]| {
            rows.iter()
                .map(|r| match r.on {
                    Some(true) => format!("{} (now {})", r.id, words[0]),
                    Some(false) => format!("{} (now {})", r.id, words[1]),
                    None => r.id.clone(),
                })
                .collect::<Vec<_>>()
                .join("|")
        };
        let plain = |rows: &[FactRow]| rows.iter().map(|r| r.id.as_str()).collect::<Vec<_>>().join("|");
        let mut lines: Vec<String> = Vec::new();
        for id in &ids {
            lines.push(match *id {
                "set_layer_visible" => format!("set_layer_visible {{layer: {}, visible: true|false}}", list(&self.facts.layers, ["shown", "hidden"])),
                "set_style_preset" => format!("set_style_preset {{preset: {}}}", list(&self.facts.presets, ["the style", "not the style"])),
                "zoom_to" => {
                    let c = self.facts.camera.as_ref();
                    let (w, h, lo, hi) = c.map(|c| (c.w, c.h, c.zoom_min, c.zoom_max)).unwrap_or((0, 0, 0.0, 0.0));
                    format!(
                        "zoom_to {{x: 0..{}, y: 0..{} (grid cells), zoom: optional {lo}..{hi}}}",
                        w.saturating_sub(1),
                        h.saturating_sub(1)
                    )
                }
                "set_landmark_filter" => format!("set_landmark_filter {{kind: {}, visible: true|false}}", list(&self.facts.kinds, ["shown", "hidden"])),
                "open_window" => format!("open_window {{window: {}}}", plain(&self.facts.windows)),
                _ => String::new(),
            });
        }
        ToolSpec {
            name: VIEW_TOOL.into(),
            description: format!(
                "Propose changes to how the map is shown. The user reviews and applies; nothing happens by this call. \
                 At most {MAX_VIEW_ACTIONS} actions. Each action is {{action, args, reason}}; args must be exactly the \
                 arguments listed, with exactly these values: {}.",
                lines.join("; ")
            ),
            parameters: json!({"type": "object", "properties": {
                "summary": {"type": "string"},
                "actions": {"type": "array", "maxItems": MAX_VIEW_ACTIONS, "items": {"type": "object", "properties": {
                    "action": {"type": "string", "enum": ids},
                    "args": {"type": "object"},
                    "reason": {"type": "string"}}, "required": ["action", "args"]}}},
                "required": ["actions"]}),
        }
    }

    /// The system-prompt paragraph about this tool. Added ONLY when the tool is in the request,
    /// so the prompt never names a tool the model cannot call. Any edit is a prompt edit:
    /// re-run the hostile corpus (scope security bar 3).
    pub fn prompt_paragraph() -> &'static str {
        "Requests about what the map SHOWS (\"hide the roads\", \"show only rivers\", \"centre on the\n\
delta\", \"switch to the satellite look\", \"open the vault\"): call propose_view_actions. It is\n\
separate from settings. You PROPOSE; the user presses Apply. You cannot generate, save, close,\n\
export or delete anything, and no action here does. Use only the listed actions and values.\n\
\n"
    }

    fn near(&self, got: &str, pool: &[&str]) -> String {
        let mut v: Vec<(usize, &str)> = pool.iter().map(|p| (edit_distance(got, p), *p)).collect();
        v.sort();
        v.iter().take(3).map(|(_, p)| *p).collect::<Vec<_>>().join(", ")
    }
}

/// One validated action, ready to show and to apply.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewChange {
    /// One of [`ACTION_IDS`].
    pub action: &'static str,
    /// The validated arguments, exactly as the model wrote them (typed).
    pub args: Map<String, Value>,
    /// What the row is about, for the card ("Ways & routes", "Style preset", "Map view").
    pub label: String,
    /// The value before, when the shell could read it (omitted otherwise, never a default).
    pub from: Option<String>,
    /// The value after, in words.
    pub to: String,
    /// True when Edit > Undo reverts it (only `set_style_preset`).
    pub undoable: bool,
    /// A sentence the card prints under the row (a preset that hides layers).
    pub note: Option<String>,
    /// The model's reason, made inert; `None` when it gave none.
    pub reason: Option<String>,
}

/// One action the validator refused.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewRejected {
    /// The action id as written, when it was a string (inert).
    pub action: Option<String>,
    /// Which rule.
    pub rule: ViewRule,
    /// Why, in words.
    pub why: String,
}

/// What the validator made of one `propose_view_actions` call.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewVerdict {
    /// A whole-call failure (A1-A3); then `valid` and `rejected` are empty.
    pub structural: Option<(ViewRule, String)>,
    /// The model's one-line summary, inert.
    pub summary: Option<String>,
    /// Actions that passed, in the model's order.
    pub valid: Vec<ViewChange>,
    /// Actions refused.
    pub rejected: Vec<ViewRejected>,
    /// Actions that would change nothing (a layer already at that state), dropped, not errors.
    pub noop: Vec<String>,
}

impl ViewVerdict {
    fn structural(rule: ViewRule, why: impl Into<String>) -> Self {
        ViewVerdict { structural: Some((rule, why.into())), summary: None, valid: vec![], rejected: vec![], noop: vec![] }
    }

    /// The outcome rule (same as the parameter flow): a structural failure, or nothing valid
    /// with something rejected, sends the errors back once.
    pub fn wants_retry(&self) -> bool {
        self.structural.is_some() || (self.valid.is_empty() && !self.rejected.is_empty())
    }

    /// The tool result for the retry. Rule ids and nearest offered values only.
    pub fn feedback(&self) -> Value {
        if let Some((rule, why)) = &self.structural {
            return json!({"status": "rejected", "error": format!("{}: {why}", rule.id()),
                "hint": "Nothing was applied. Call propose_view_actions again with a corrected list."});
        }
        let errs: Vec<Value> = self.rejected.iter().map(|r| json!({"action": r.action, "rule": r.rule.id(), "error": r.why})).collect();
        json!({"status": "rejected", "valid": self.valid.len(), "errors": errs,
            "hint": "Nothing was applied. Use only the listed actions, arguments and values, with the right types."})
    }

    /// The undo statement the card prints beside Apply (the module docs define it): which
    /// valid rows Edit > Undo reverts and which are session state. Empty with no valid row.
    pub fn undo_note(&self) -> Vec<String> {
        let undoable = self.valid.iter().filter(|c| c.undoable).count();
        let instant = self.valid.len() - undoable;
        let mut out = Vec::new();
        if undoable > 0 {
            out.push(
                "Edit > Undo reverts the style preset in one step. A style change redraws the map, which re-fits the view, and so does its undo."
                    .to_owned(),
            );
        }
        if instant > 0 {
            out.push(format!(
                "{} view change{} take{} effect at once and {} not on the undo stack (the old values are listed above).",
                instant,
                if instant == 1 { "" } else { "s" },
                if instant == 1 { "s" } else { "" },
                if instant == 1 { "is" } else { "are" }
            ));
        }
        out
    }
}

fn num_text(x: f64) -> String {
    if (x - x.round()).abs() < 1e-9 { format!("{}", x.round() as i64) } else { format!("{x:.2}") }
}

impl ViewCatalogue {
    /// A1-A9 over one parsed `propose_view_actions` call. Pure: reads `self` and `args`, writes nothing.
    pub fn validate(&self, args: &Value) -> ViewVerdict {
        // A1
        let Some(obj) = args.as_object() else { return ViewVerdict::structural(ViewRule::A1, "arguments are not a JSON object") };
        if args.to_string().len() > MAX_VIEW_ARGUMENT_BYTES {
            return ViewVerdict::structural(ViewRule::A1, format!("arguments are larger than {MAX_VIEW_ARGUMENT_BYTES} bytes"));
        }
        if depth(args, 0) > MAX_VIEW_DEPTH {
            return ViewVerdict::structural(ViewRule::A1, format!("arguments are nested deeper than {MAX_VIEW_DEPTH} levels"));
        }
        // A2
        if let Some(k) = obj.keys().find(|k| !matches!(k.as_str(), "summary" | "actions")) {
            return ViewVerdict::structural(ViewRule::A2, format!("unexpected top-level field {}", inert_text(k, 40)));
        }
        let list = match obj.get("actions").and_then(Value::as_array) {
            Some(a) if !a.is_empty() => a,
            _ => return ViewVerdict::structural(ViewRule::A2, "actions is missing, not an array, or empty"),
        };
        // A3
        if list.len() > MAX_VIEW_ACTIONS {
            return ViewVerdict::structural(
                ViewRule::A3,
                format!("{} actions is more than the {MAX_VIEW_ACTIONS} one proposal may carry", list.len()),
            );
        }
        let summary = obj.get("summary").and_then(Value::as_str).map(|s| inert_text(s, MAX_VIEW_REASON_CHARS)).filter(|s| !s.is_empty());
        let offered = self.offered();
        let mut v = ViewVerdict { structural: None, summary, valid: vec![], rejected: vec![], noop: vec![] };
        let mut targets: Vec<String> = Vec::new();
        for item in list {
            let name = item.get("action").and_then(Value::as_str).map(|s| inert_text(s, 40));
            let mut reject = |rule: ViewRule, why: String| v.rejected.push(ViewRejected { action: name.clone(), rule, why });
            // A4
            let Some(e) = item.as_object() else {
                reject(ViewRule::A4, "the action is not an object".into());
                continue;
            };
            if let Some(k) = e.keys().find(|k| !matches!(k.as_str(), "action" | "args" | "reason")) {
                reject(ViewRule::A4, format!("unexpected field {}", inert_text(k, 40)));
                continue;
            }
            let Some(id) = e.get("action").and_then(Value::as_str) else {
                reject(ViewRule::A4, "action is missing or not a string".into());
                continue;
            };
            let Some(a) = e.get("args").and_then(Value::as_object) else {
                reject(ViewRule::A4, "args is missing or not an object".into());
                continue;
            };
            // A5
            let Some(&action) = ACTION_IDS.iter().find(|x| **x == id) else {
                reject(ViewRule::A5, format!("unknown action; the offered actions are: {}", offered.join(", ")));
                continue;
            };
            if !offered.contains(&action) {
                let why = self.facts.withheld.get(action).cloned().unwrap_or_else(|| "nothing is available for it right now".into());
                reject(ViewRule::A5, format!("{action} is not available: {why}"));
                continue;
            }
            let reason = e.get("reason").and_then(Value::as_str).map(|s| inert_text(s, MAX_VIEW_REASON_CHARS)).filter(|s| !s.is_empty());
            match self.check(action, a) {
                Err((rule, why)) => reject(rule, why),
                Ok(Checked::Noop(text)) => v.noop.push(text),
                Ok(Checked::Change(mut c, target)) => {
                    if targets.contains(&target) {
                        reject(ViewRule::A9, "the same target appears twice in one proposal".into());
                    } else {
                        targets.push(target);
                        c.reason = reason;
                        v.valid.push(c);
                    }
                }
            }
        }
        v
    }

    /// One action's arguments against its fixed shape. The shape of each action lives here.
    fn check(&self, action: &'static str, a: &Map<String, Value>) -> Result<Checked, (ViewRule, String)> {
        let allowed: &[&str] = match action {
            "set_layer_visible" => &["layer", "visible"],
            "set_style_preset" => &["preset"],
            "zoom_to" => &["x", "y", "zoom"],
            "set_landmark_filter" => &["kind", "visible"],
            _ => &["window"],
        };
        let required: &[&str] = if action == "zoom_to" { &["x", "y"] } else { allowed };
        if let Some(k) = a.keys().find(|k| !allowed.contains(&k.as_str())) {
            return Err((ViewRule::A6, format!("unknown argument {}; {action} takes: {}", inert_text(k, 40), allowed.join(", "))));
        }
        if let Some(k) = required.iter().find(|k| !a.contains_key(**k)) {
            return Err((ViewRule::A6, format!("missing argument {k}")));
        }
        let text = |k: &str| -> Result<&str, (ViewRule, String)> {
            a.get(k).and_then(Value::as_str).ok_or((ViewRule::A7, format!("{k} must be a string")))
        };
        let boolean = |k: &str| -> Result<bool, (ViewRule, String)> {
            a.get(k).and_then(Value::as_bool).ok_or((ViewRule::A7, format!("{k} must be true or false, not a number or text")))
        };
        let number = |k: &str| -> Result<f64, (ViewRule, String)> {
            a.get(k).and_then(Value::as_f64).filter(|x| x.is_finite()).ok_or((ViewRule::A7, format!("{k} must be a number")))
        };
        let pick = |k: &str, rows: &'_ [FactRow]| -> Result<usize, (ViewRule, String)> {
            let got = text(k)?;
            rows.iter().position(|r| r.id == got).ok_or_else(|| {
                let pool: Vec<&str> = rows.iter().map(|r| r.id.as_str()).collect();
                (ViewRule::A8, format!("{} is not an offered {k}; nearest: {}", inert_text(got, 40), self.near(got, &pool)))
            })
        };
        let mut args = Map::new();
        match action {
            "set_layer_visible" | "set_landmark_filter" => {
                let (key, rows, prefix) = if action == "set_layer_visible" {
                    ("layer", &self.facts.layers, "layer")
                } else {
                    ("kind", &self.facts.kinds, "kind")
                };
                let i = pick(key, rows)?;
                let vis = boolean("visible")?;
                let row = &rows[i];
                args.insert(key.into(), json!(row.id));
                args.insert("visible".into(), json!(vis));
                let target = format!("{prefix}:{}", row.id);
                if row.on == Some(vis) {
                    return Ok(Checked::Noop(format!("{} is already {}", row.label, if vis { "shown" } else { "hidden" })));
                }
                let word = |b: bool| if b { "shown" } else { "hidden" };
                Ok(Checked::Change(
                    ViewChange {
                        action,
                        args,
                        label: row.label.clone(),
                        from: row.on.map(|b| word(b).to_owned()),
                        to: word(vis).to_owned(),
                        undoable: false,
                        note: None,
                        reason: None,
                    },
                    target,
                ))
            }
            "set_style_preset" => {
                let i = pick("preset", &self.facts.presets)?;
                let row = &self.facts.presets[i];
                args.insert("preset".into(), json!(row.id));
                if row.on == Some(true) {
                    return Ok(Checked::Noop(format!("{} is already the style", row.label)));
                }
                let from = self.facts.presets.iter().find(|r| r.on == Some(true)).map(|r| r.label.clone());
                Ok(Checked::Change(
                    ViewChange {
                        action,
                        args,
                        label: "Style preset".into(),
                        from,
                        to: row.label.clone(),
                        undoable: true,
                        note: Some(row.note.clone()).filter(|n| !n.is_empty()),
                        reason: None,
                    },
                    "preset".into(),
                ))
            }
            "zoom_to" => {
                let cam = self.facts.camera.as_ref().ok_or((ViewRule::A5, "no world is open".to_string()))?;
                let (x, y) = (number("x")?, number("y")?);
                if !(0.0..=f64::from(cam.w - 1)).contains(&x) {
                    return Err((ViewRule::A8, format!("x {} is outside 0..{}", num_text(x), cam.w - 1)));
                }
                if !(0.0..=f64::from(cam.h - 1)).contains(&y) {
                    return Err((ViewRule::A8, format!("y {} is outside 0..{}", num_text(y), cam.h - 1)));
                }
                args.insert("x".into(), json!(x));
                args.insert("y".into(), json!(y));
                let mut to = format!("centre on cell ({}, {})", num_text(x), num_text(y));
                if a.contains_key("zoom") {
                    let z = number("zoom")?;
                    if z < cam.zoom_min || z > cam.zoom_max {
                        return Err((ViewRule::A8, format!("zoom {} is outside {}..{}", num_text(z), num_text(cam.zoom_min), num_text(cam.zoom_max))));
                    }
                    args.insert("zoom".into(), json!(z));
                    to.push_str(&format!(", zoom {}x", num_text(z)));
                } else {
                    to.push_str(", zoom unchanged");
                }
                Ok(Checked::Change(
                    ViewChange {
                        action,
                        args,
                        label: "Map view".into(),
                        from: cam.zoom_now.map(|z| format!("zoom {}x", num_text(z))),
                        to,
                        undoable: false,
                        note: None,
                        reason: None,
                    },
                    "camera".into(),
                ))
            }
            _ => {
                let i = pick("window", &self.facts.windows)?;
                let row = &self.facts.windows[i];
                args.insert("window".into(), json!(row.id));
                Ok(Checked::Change(
                    ViewChange {
                        action,
                        args,
                        label: row.label.clone(),
                        from: None,
                        to: "opened".into(),
                        undoable: false,
                        note: None,
                        reason: None,
                    },
                    format!("window:{}", row.id),
                ))
            }
        }
    }
}

enum Checked {
    Noop(String),
    Change(ViewChange, String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts() -> Value {
        json!({
            "layers": [{"id": "roads", "label": "Ways & routes", "on": true}, {"id": "rivers", "label": "Rivers", "on": false},
                       {"id": "landmarks", "label": "Landmarks", "on": true}],
            "presets": [{"id": "natural_vibrant", "label": "Natural Vibrant", "on": true},
                        {"id": "satellite_relief", "label": "Satellite relief", "on": false, "note": "Also hides six overlay layers; Edit > Undo does not bring them back."}],
            "kinds": [{"id": "peak", "label": "Peak", "on": true}, {"id": "ford", "label": "Ford", "on": false}],
            "windows": [{"id": "vault", "label": "Markdown vault"}, {"id": "shortcuts", "label": "Keyboard shortcuts"}],
            "camera": {"w": 2048, "h": 1311, "zoom_min": 0.4, "zoom_max": 64.0, "zoom_now": 1.0},
            "withheld": {}
        })
    }

    fn cat() -> ViewCatalogue {
        ViewCatalogue::from_json(&facts()).unwrap()
    }

    fn call(actions: Value) -> Value {
        json!({"summary": "s", "actions": actions})
    }

    #[test]
    fn a_healthy_build_offers_all_five_actions_and_drops_nothing() {
        let c = cat();
        assert_eq!(c.offered(), vec!["set_layer_visible", "set_style_preset", "zoom_to", "set_landmark_filter", "open_window"]);
        assert!(c.dropped.is_empty());
        let spec = c.tool_spec();
        assert_eq!(spec.name, "propose_view_actions");
        assert_eq!(spec.parameters["properties"]["actions"]["items"]["properties"]["action"]["enum"].as_array().unwrap().len(), 5);
        assert_eq!(spec.parameters["properties"]["actions"]["maxItems"], 8);
        assert!(spec.description.contains("nothing happens by this call"));
        assert!(spec.description.contains("layer: roads (now shown)|rivers (now hidden)|landmarks (now shown)"), "{}", spec.description);
        assert!(spec.description.contains("preset: natural_vibrant (now the style)|satellite_relief (now not the style)"));
        assert!(spec.description.contains("window: vault|shortcuts"), "a window has no state");
        // The enum of the call stays the bare action ids, and no state word leaks into a value.
        assert!(!spec.parameters.to_string().contains("(now"));
        assert!(spec.description.contains("x: 0..2047, y: 0..1310"));
    }

    /// Protects "absent, never a default": a layer whose state the shell could not read is listed by
    /// its bare id (no "now ..." word), and a validated change to it has no `from`. MUTATION: write a
    /// state word for an unknown state and the first assertion goes red.
    #[test]
    fn an_unreadable_state_is_not_invented() {
        let c = ViewCatalogue::from_json(&json!({"layers": [{"id": "roads", "label": "Roads"}, {"id": "rivers", "label": "Rivers", "on": true}]})).unwrap();
        assert!(c.tool_spec().description.contains("layer: roads|rivers (now shown),"), "{}", c.tool_spec().description);
        let v = c.validate(&call(json!([{"action": "set_layer_visible", "args": {"layer": "roads", "visible": false}}])));
        assert_eq!((v.valid.len(), v.valid[0].from.clone()), (1, None), "no state read: not a no-op, and no invented before-value");
    }

    #[test]
    fn a_valid_mixed_proposal_yields_its_changes_with_old_and_new_values() {
        let c = cat();
        let v = c.validate(&call(json!([
            {"action": "set_layer_visible", "args": {"layer": "roads", "visible": false}, "reason": "declutter"},
            {"action": "set_style_preset", "args": {"preset": "satellite_relief"}},
            {"action": "zoom_to", "args": {"x": 1200, "y": 600.5, "zoom": 8}},
            {"action": "set_landmark_filter", "args": {"kind": "ford", "visible": true}},
            {"action": "open_window", "args": {"window": "vault"}}
        ])));
        assert!(v.structural.is_none() && v.rejected.is_empty(), "{v:?}");
        assert_eq!(v.valid.len(), 5);
        assert_eq!((v.valid[0].from.as_deref(), v.valid[0].to.as_str()), (Some("shown"), "hidden"));
        assert_eq!(v.valid[0].reason.as_deref(), Some("declutter"));
        assert_eq!((v.valid[1].from.as_deref(), v.valid[1].to.as_str(), v.valid[1].undoable), (Some("Natural Vibrant"), "Satellite relief", true));
        assert!(v.valid[1].note.as_deref().unwrap().contains("hides six overlay layers"));
        assert_eq!(v.valid[2].to, "centre on cell (1200, 600.50), zoom 8x");
        assert_eq!(v.valid[2].from.as_deref(), Some("zoom 1x"));
        assert_eq!((v.valid[3].from.as_deref(), v.valid[3].to.as_str()), (Some("hidden"), "shown"));
        assert_eq!(v.valid[4].to, "opened");
        // Only the preset is on the undo stack.
        assert_eq!(v.valid.iter().filter(|c| c.undoable).count(), 1);
        assert_eq!(v.undo_note().len(), 2);
        assert!(v.undo_note()[0].starts_with("Edit > Undo reverts the style preset in one step.") && v.undo_note()[0].contains("re-fits the view"));
        assert!(v.undo_note()[1].starts_with("4 view changes take effect at once and are not on the undo stack"));
    }

    #[test]
    fn a_noop_is_dropped_not_an_error() {
        let v = cat().validate(&call(json!([{"action": "set_layer_visible", "args": {"layer": "roads", "visible": true}}])));
        assert!(v.valid.is_empty() && v.rejected.is_empty());
        assert_eq!(v.noop, vec!["Ways & routes is already shown".to_string()]);
        assert!(!v.wants_retry());
    }

    fn rule_of(args: Value) -> (ViewRule, String) {
        let v = cat().validate(&args);
        if let Some(s) = v.structural {
            return s;
        }
        let r = &v.rejected[0];
        (r.rule, r.why.clone())
    }

    #[test]
    fn every_rule_has_a_literal_trigger() {
        assert_eq!(rule_of(json!([1])).0, ViewRule::A1);
        assert_eq!(rule_of(json!({"actions": []})).0, ViewRule::A2);
        assert_eq!(rule_of(json!({"actions": "x"})).0, ViewRule::A2);
        assert_eq!(rule_of(json!({"actions": [{}], "extra": 1})).0, ViewRule::A2);
        let nine: Vec<Value> = (0..9).map(|_| json!({"action": "open_window", "args": {"window": "vault"}})).collect();
        assert_eq!(rule_of(call(json!(nine))).0, ViewRule::A3);
        assert_eq!(rule_of(call(json!([7]))).0, ViewRule::A4);
        assert_eq!(rule_of(call(json!([{"action": "open_window", "args": {"window": "vault"}, "cmd": "x"}]))).0, ViewRule::A4);
        assert_eq!(rule_of(call(json!([{"action": 5, "args": {}}]))).0, ViewRule::A4);
        assert_eq!(rule_of(call(json!([{"action": "open_window", "args": []}]))).0, ViewRule::A4);
        assert_eq!(rule_of(call(json!([{"action": "generate_world", "args": {}}]))).0, ViewRule::A5);
        assert_eq!(rule_of(call(json!([{"action": "open_window", "args": {"window": "vault", "cmd": "rm"}}]))).0, ViewRule::A6);
        assert_eq!(rule_of(call(json!([{"action": "open_window", "args": {}}]))).0, ViewRule::A6);
        assert_eq!(rule_of(call(json!([{"action": "set_layer_visible", "args": {"layer": "roads", "visible": 0}}]))).0, ViewRule::A7);
        assert_eq!(rule_of(call(json!([{"action": "set_layer_visible", "args": {"layer": "roads", "visible": "false"}}]))).0, ViewRule::A7);
        assert_eq!(rule_of(call(json!([{"action": "zoom_to", "args": {"x": "5", "y": 5}}]))).0, ViewRule::A7);
        assert_eq!(rule_of(call(json!([{"action": "open_window", "args": {"window": "save_project"}}]))).0, ViewRule::A8);
        assert_eq!(rule_of(call(json!([{"action": "zoom_to", "args": {"x": 2048, "y": 5}}]))).0, ViewRule::A8);
        assert_eq!(rule_of(call(json!([{"action": "zoom_to", "args": {"x": 5, "y": -0.5}}]))).0, ViewRule::A8);
        assert_eq!(rule_of(call(json!([{"action": "zoom_to", "args": {"x": 5, "y": 5, "zoom": 64.1}}]))).0, ViewRule::A8);
        assert_eq!(rule_of(call(json!([{"action": "zoom_to", "args": {"x": 5, "y": 5, "zoom": 0.39}}]))).0, ViewRule::A8);
        let dup = cat().validate(&call(json!([
            {"action": "open_window", "args": {"window": "vault"}}, {"action": "open_window", "args": {"window": "vault"}}])));
        assert_eq!((dup.valid.len(), dup.rejected[0].rule), (1, ViewRule::A9));
    }

    #[test]
    fn range_edges_are_literal_and_never_clamped() {
        let c = cat();
        let ok = |x: f64, y: f64, z: f64| c.validate(&call(json!([{"action": "zoom_to", "args": {"x": x, "y": y, "zoom": z}}]))).valid.len();
        assert_eq!(ok(0.0, 0.0, 0.4), 1);
        assert_eq!(ok(2047.0, 1310.0, 64.0), 1);
        assert_eq!(ok(2047.5, 0.0, 1.0), 0);
        assert_eq!(ok(0.0, 1310.5, 1.0), 0);
        // A refusal leaves the number in the message as the model wrote it, not a clamped one.
        let (_, why) = rule_of(call(json!([{"action": "zoom_to", "args": {"x": 99999, "y": 5}}])));
        assert!(why.contains("99999") && why.contains("0..2047"), "{why}");
    }

    #[test]
    fn an_unknown_action_names_the_offered_ones_and_a_withheld_one_names_why() {
        let (_, why) = rule_of(call(json!([{"action": "regenerate", "args": {}}])));
        assert_eq!(why, "unknown action; the offered actions are: set_layer_visible, set_style_preset, zoom_to, set_landmark_filter, open_window");
        let mut f = facts();
        f.as_object_mut().unwrap().remove("camera");
        f["withheld"] = json!({"zoom_to": "No world is open.", "bogus": "x"});
        let c = ViewCatalogue::from_json(&f).unwrap();
        assert!(!c.offered().contains(&"zoom_to"));
        let v = c.validate(&call(json!([{"action": "zoom_to", "args": {"x": 1, "y": 1}}])));
        assert_eq!(v.rejected[0].why, "zoom_to is not available: No world is open.");
        // `bogus` is not a vocabulary id, so it was not even kept.
        assert!(!c.facts().withheld.contains_key("bogus"));
    }

    #[test]
    fn forbidden_words_and_bad_ids_are_dropped_from_the_facts() {
        let f = json!({
            "windows": [{"id": "vault", "label": "Vault"}, {"id": "save_project", "label": "Save"}, {"id": "export_map", "label": "x"},
                        {"id": "Vault2", "label": "x"}, {"id": "a b", "label": "x"}, {"id": "", "label": "x"}, {"id": "vault", "label": "dup"},
                        {"id": "regenerate_all", "label": "x"}, {"id": "close_world", "label": "x"}, {"id": "open_project_recent", "label": "x"},
                        {"id": "http_fetch", "label": "x"}],
            "layers": []
        });
        let c = ViewCatalogue::from_json(&f).unwrap();
        let ids: Vec<&str> = c.facts().windows.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, vec!["vault"]);
        assert_eq!(c.dropped.len(), 10, "{:?}", c.dropped);
        assert!(c.dropped.iter().any(|d| d.contains("forbidden word 'save'")));
        assert!(c.dropped.iter().any(|d| d.contains("forbidden word 'generat'")));
        // The vocabulary cannot be extended by facts: a window named like an action is still a window.
        assert_eq!(c.offered(), vec!["open_window"]);
        // `preset` contains `reset` but is not one; `autosave` and `reset_all` are caught.
        let c3 = ViewCatalogue::from_json(&json!({"windows": [{"id": "preset_gallery"}, {"id": "autosave_settings"}, {"id": "reset_all"}, {"id": "my_preset"}]})).unwrap();
        let ids3: Vec<&str> = c3.facts().windows.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids3, vec!["preset_gallery", "my_preset"]);
        // An id of 41 characters is refused, 40 is kept.
        let long41 = "a".repeat(41);
        let long40 = "a".repeat(40);
        let c2 = ViewCatalogue::from_json(&json!({"windows": [{"id": long41}, {"id": long40}]})).unwrap();
        assert_eq!(c2.facts().windows.len(), 1);
    }

    #[test]
    fn a_fact_list_over_the_cap_is_cut_and_says_so() {
        let rows: Vec<Value> = (0..70).map(|i| json!({"id": format!("w{i}")})).collect();
        let c = ViewCatalogue::from_json(&json!({"windows": rows})).unwrap();
        assert_eq!(c.facts().windows.len(), 64);
        assert!(c.dropped[0].contains("more than 64 rows"));
    }

    #[test]
    fn a_bad_camera_is_dropped_not_defaulted() {
        for cam in [json!({"w": 0, "h": 5, "zoom_min": 0.4, "zoom_max": 64.0}), json!({"w": 5, "h": 5, "zoom_min": 64.0, "zoom_max": 0.4}),
                    json!({"w": 5, "h": 5, "zoom_min": 0.0, "zoom_max": 1.0}), json!({"w": 5, "h": 5}), json!("x")] {
            let c = ViewCatalogue::from_json(&json!({"camera": cam})).unwrap();
            assert!(c.facts().camera.is_none() && !c.dropped.is_empty());
            assert!(!c.is_offered());
        }
        // No zoom_now: the row has no "from" (omitted, never 0).
        let c = ViewCatalogue::from_json(&json!({"camera": {"w": 5, "h": 5, "zoom_min": 0.4, "zoom_max": 64.0}})).unwrap();
        let v = c.validate(&call(json!([{"action": "zoom_to", "args": {"x": 1, "y": 1}}])));
        assert_eq!(v.valid[0].from, None);
    }

    #[test]
    fn model_text_is_inert_and_size_and_depth_are_bounded() {
        let v = cat().validate(&json!({"summary": "a\u{0}b\n[c](http://x)", "actions": [
            {"action": "open_window", "args": {"window": "vault"}, "reason": "x\ty\u{7}".to_string() + &"z".repeat(400)}]}));
        assert_eq!(v.summary.as_deref(), Some("a b [c](http://x)"));
        let r = v.valid[0].reason.as_deref().unwrap();
        assert!(!r.contains('\t') && !r.contains('\u{7}') && r.chars().count() == 240 && r.ends_with('\u{2026}'));
        let big = json!({"actions": [{"action": "open_window", "args": {"window": "vault"}, "reason": "q".repeat(9000)}]});
        assert_eq!(cat().validate(&big).structural.unwrap().0, ViewRule::A1);
        let deep = json!({"actions": [{"action": "open_window", "args": {"window": {"a": 1}}}]});
        assert_eq!(cat().validate(&deep).structural.unwrap().0, ViewRule::A1);
    }

    #[test]
    fn the_retry_rule_matches_the_parameter_flow() {
        let c = cat();
        assert!(c.validate(&json!([])).wants_retry());
        assert!(c.validate(&call(json!([{"action": "nope", "args": {}}]))).wants_retry());
        let mixed = c.validate(&call(json!([{"action": "nope", "args": {}}, {"action": "open_window", "args": {"window": "vault"}}])));
        assert!(!mixed.wants_retry() && mixed.valid.len() == 1);
        let fb = c.validate(&call(json!([{"action": "open_window", "args": {"window": "vaul"}}]))).feedback();
        assert_eq!(fb["errors"][0]["rule"], "A8");
        assert!(fb["errors"][0]["error"].as_str().unwrap().contains("nearest: vault"));
    }

    #[test]
    fn the_prompt_paragraph_names_the_tool_and_forbids_the_dangerous_verbs() {
        let p = ViewCatalogue::prompt_paragraph();
        assert!(p.contains("propose_view_actions") && p.contains("cannot generate, save, close"));
    }
}
