//! The action catalogue (`ASSISTANT_ACTIONS_PROPOSAL.md` s.3.1, milestone AC-1; Ruling DW): ONE
//! table of everything the assistant may be offered, GENERATED from the sources that exist and
//! never typed in parallel with them.
//!
//! ## Where each entry comes from
//!
//! | Source | Entries | Id | How it reaches here |
//! |---|---|---|---|
//! | generation parameters | one per offered registry row | `param:<key>` | [`Catalogue`] (the registry rows, already built per request) |
//! | view verbs | one per layer, the style preset, zoom, the landmark filter | `view.layer.<id>`, `view.style_preset`, `view.zoom_to`, `view.landmark_filter` | [`ViewCatalogue`] (`shell/llm_view.gd::facts`) |
//! | windows | the curated read-only windows | `window.<id>` | the same view facts |
//! | menu rows | every runnable `CommandIndex` row | `menu:<ID_ constant, lower-cased>` | [`ShellFacts`] (`shell/llm_actions.gd`) |
//! | context-card actions | the literal-labelled dotted ids | `civ.start_way` | `data/action_static_sources.json`, scanned from the shell source by `tests/ac1_static_sources.rs` (which also fails when it goes stale) |
//! | map tools | every `arm_tool("...")` literal | `tool.<id>` | the same data file |
//! | the year cursor | `timeline.set_year` | `timeline.set_year` | [`ShellFacts`] |
//!
//! The build is a pure function of those inputs: the same inputs in any order give the same
//! entries in the same order (entries are sorted by `(path, id)`), which is the AC-1 bar "the ids
//! are identical across two builds".
//!
//! ## What is refused by construction
//!
//! The vocabulary of [`Effect`] has NO `destructive` member and the build drops (and records in
//! [`ActionCatalogue::dropped`]) any entry whose id or title names generating, saving, deleting,
//! exporting, importing, closing, resetting, quitting or the network ([`refusal`]): AQ1 leaves
//! file actions out, and the owner's rule is that the assistant never does these. `pinnable` is
//! NOT a safety mark (`CommandIndex`: Save, Close, New world and Export are all pinnable), so a
//! menu row is offered as runnable only when a curated review row says what it does
//! ([`MENU_REVIEW`]); every other runnable row is listed with [`Effect::Unreviewed`] and
//! `available: false`, so it is visible (AC-1: every runnable row has an entry) but cannot be
//! proposed. The same rule covers context actions ([`CONTEXT_REVIEW`]).
//!
//! ## Descriptions
//!
//! At most [`MAX_DESC_CHARS`] characters (AC-1). A description comes from, in order: the registry's
//! own `desc` (parameters), the curated tables here (labelled judgements, read at the symbol they
//! describe), or a DERIVED sentence built from the title and menu (`desc_source` says which).
//! Derived ones are counted by [`ActionCatalogue::derived_desc_count`] and are AC-5's work list,
//! not placeholders to hide: a derived description states only what the source says.
//!
//! MUST NEVER: hold a handle to the app or the engine; offer an id the sources did not produce;
//! widen the offered set from model text (every input here is built by the app); mark an entry
//! `available` without the source saying so.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value, json};

use crate::catalogue::{Catalogue, Kind as PKind, edit_distance, inert_text};
use crate::view_actions::{FORBIDDEN_STEMS, ViewCatalogue};

/// AC-1 bar: the longest description, in characters.
pub const MAX_DESC_CHARS: usize = 120;

/// The tree is at most three levels deep (AC-2 bar). A path with more segments is dropped.
pub const MAX_PATH_DEPTH: usize = 3;

/// The longest entry id. A labelled bound above the longest real id (`param:` + a registry key).
pub const MAX_ID_CHARS: usize = 64;

/// The embedded static scan (context actions and tools). See `tests/ac1_static_sources.rs`.
pub const STATIC_SOURCES: &str = include_str!("../data/action_static_sources.json");

/// What kind of function an action is (the `kind` filter of `list_actions`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionKind {
    /// Reads something and shows it.
    Query,
    /// Changes how the map or app is shown.
    View,
    /// Changes the selection or the year cursor.
    Select,
    /// Changes a generation setting.
    Setting,
    /// Edits the world or arms an editing tool.
    Edit,
    /// Opens a window.
    Window,
}

impl ActionKind {
    /// The wire name.
    pub fn as_str(self) -> &'static str {
        match self {
            ActionKind::Query => "query",
            ActionKind::View => "view",
            ActionKind::Select => "select",
            ActionKind::Setting => "setting",
            ActionKind::Edit => "edit",
            ActionKind::Window => "window",
        }
    }

    /// Parses a wire name; `None` for anything else.
    pub fn parse(s: &str) -> Option<Self> {
        [ActionKind::Query, ActionKind::View, ActionKind::Select, ActionKind::Setting, ActionKind::Edit, ActionKind::Window]
            .into_iter()
            .find(|k| k.as_str() == s)
    }

    /// The names, for tool schemas.
    pub fn names() -> Vec<&'static str> {
        vec!["query", "view", "select", "setting", "edit", "window"]
    }
}

/// What running an action does. There is deliberately NO `destructive` member: v1 offers none
/// (AQ1), so a destructive effect cannot be written, and [`refusal`] drops the rows that would be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    /// Reads and reports; changes nothing.
    Read,
    /// Changes session state (a layer, the camera, an armed tool); not on the undo stack.
    View,
    /// Changes the project and is reverted by one Edit > Undo.
    Undoable,
    /// A setting that changes the world only when the world is regenerated or recomputed.
    Regenerates,
    /// Opens a window or panel and stops (AQ2: the model never works inside a dialog).
    OpensWindow,
    /// Nobody has reviewed what this does for assistant use. Listed, never proposable.
    Unreviewed,
}

impl Effect {
    /// The wire name.
    pub fn as_str(self) -> &'static str {
        match self {
            Effect::Read => "read",
            Effect::View => "view",
            Effect::Undoable => "undoable",
            Effect::Regenerates => "regenerates",
            Effect::OpensWindow => "opens-window",
            Effect::Unreviewed => "unreviewed",
        }
    }
}

/// AC-3: which existing click path runs an entry. Derived from the id's family, so a new family cannot
/// be run until it is named here; anything unnamed is [`Route::None`] and is refused at Apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// A generation setting, through `undo_group.write_param` (the `param:` id's key is the rest).
    Param,
    /// A view verb, through `LlmView.apply` (layers, the style preset, zoom, the landmark filter).
    View,
    /// A curated window, through `CommandIndex.run_row` (`window.<id>`).
    Window,
    /// A reviewed menu row, through `CommandIndex.run_row` (`menu:<name>`).
    Menu,
    /// The year cursor, through `DccShell.tl_set_year`.
    Year,
    /// A selection verb, through `DccApp.select`, `clear_selection` or `select_back`.
    Select,
    /// Arming a map tool, through `DccApp.arm_tool`.
    Tool,
    /// AC-5: Edit > Undo, through `DccApp.undo_last` (the one entry point Ctrl+Z, the Edit row and the
    /// toolbar square all use). It runs alone on its card (rule C8).
    Undo,
    /// No click path from a proposal (context-card actions, unreviewed rows).
    None,
}

impl Route {
    /// The wire name the card carries.
    pub fn as_str(self) -> &'static str {
        match self {
            Route::Param => "param",
            Route::View => "view",
            Route::Window => "window",
            Route::Menu => "menu",
            Route::Year => "year",
            Route::Select => "select",
            Route::Tool => "tool",
            Route::Undo => "undo",
            Route::None => "none",
        }
    }

    /// The inverse of [`Route::as_str`]; anything else, including the empty string, is [`Route::None`]
    /// (the refused route), never a plausible default.
    pub fn parse(s: &str) -> Route {
        [Route::Param, Route::View, Route::Window, Route::Menu, Route::Year, Route::Select, Route::Tool, Route::Undo].into_iter().find(|r| r.as_str() == s).unwrap_or(Route::None)
    }
}

/// Where a description came from (AC-5 reads this to find the work left).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescSource {
    /// The registry's own wording (a generation parameter's `desc`).
    Registry,
    /// A curated table in this file, a labelled judgement.
    Curated,
    /// Built from the title and menu only.
    Derived,
}

/// One argument's type, with its bounds, as the schema and the validator both read it.
#[derive(Debug, Clone, PartialEq)]
pub enum ArgType {
    /// `true` or `false` only (a `1` or `"true"` is refused).
    Bool,
    /// A whole number in `min..=max`.
    Int {
        /// Inclusive lower bound.
        min: i64,
        /// Inclusive upper bound.
        max: i64,
    },
    /// A finite number in `min..=max`.
    Num {
        /// Inclusive lower bound.
        min: f64,
        /// Inclusive upper bound.
        max: f64,
    },
    /// One of these strings.
    Choice(Vec<String>),
}

/// One named argument.
#[derive(Debug, Clone, PartialEq)]
pub struct ArgSpec {
    /// The argument's name.
    pub name: String,
    /// Its type.
    pub ty: ArgType,
    /// Whether the call must carry it.
    pub required: bool,
    /// AC-5: one sentence on what the value MEANS, shown in `describe_action`'s schema as its
    /// `description`. A small local model given only `{"type":"integer"}` for `select.settlement`'s `id`
    /// wrote the words `number` and `n` (the 2026-11-07 qwen3-14b run, AC-3's known gap); the hint says
    /// what the number is and that a word is never one. Fixed text written here, never from the world.
    pub hint: Option<&'static str>,
}

impl ArgSpec {
    fn new(name: &str, ty: ArgType, required: bool) -> Self {
        ArgSpec { name: name.to_owned(), ty, required, hint: None }
    }

    /// This argument with its meaning sentence (see [`ArgSpec::hint`]).
    fn hint(mut self, h: &'static str) -> Self {
        self.hint = Some(h);
        self
    }

    /// This argument as a JSON-schema property.
    pub fn schema(&self) -> Value {
        let mut v = match &self.ty {
            ArgType::Bool => json!({"type": "boolean"}),
            ArgType::Int { min, max } => json!({"type": "integer", "minimum": min, "maximum": max}),
            ArgType::Num { min, max } => json!({"type": "number", "minimum": min, "maximum": max}),
            ArgType::Choice(c) => json!({"type": "string", "enum": c}),
        };
        if let Some(h) = self.hint {
            v["description"] = json!(h);
        }
        v
    }

    /// The argument in one short phrase for a preloaded pack (AC-4): `year (whole number -400..1200)`.
    /// A choice lists at most six values, then says how many more, so a pack stays small.
    pub fn brief(&self) -> String {
        let ty = match &self.ty {
            ArgType::Bool => "true or false".to_owned(),
            ArgType::Int { min, max } => format!("whole number {min}..{max}"),
            ArgType::Num { min, max } => format!("number {min}..{max}"),
            ArgType::Choice(c) if c.len() <= 6 => format!("one of {}", c.join("/")),
            ArgType::Choice(c) => format!("one of {} (and {} more)", c[..6].join("/"), c.len() - 6),
        };
        format!("{} ({}{})", self.name, ty, if self.required { "" } else { ", optional" })
    }

    /// A format example value (not a recommendation): `true`, the lowest allowed number, the first choice.
    fn example(&self) -> Value {
        match &self.ty {
            ArgType::Bool => json!(true),
            ArgType::Int { min, max } => json!(0i64.clamp(*min, *max)),
            ArgType::Num { min, max } => json!(0.0f64.clamp(*min, *max)),
            ArgType::Choice(c) => json!(c.first().cloned().unwrap_or_default()),
        }
    }

    /// Checks one value: right JSON type, in range or in the set. Never clamps or coerces.
    fn check(&self, v: &Value) -> Result<(), String> {
        let n = &self.name;
        match &self.ty {
            ArgType::Bool => v.as_bool().map(|_| ()).ok_or_else(|| format!("{n} must be true or false (a real boolean)")),
            ArgType::Int { min, max } => {
                let x = v.as_i64().or_else(|| v.as_f64().filter(|f| f.is_finite() && f.fract() == 0.0 && f.abs() < 9.0e15).map(|f| f as i64));
                match x {
                    None => Err(format!("{n} must be a whole number")),
                    Some(x) if x < *min || x > *max => Err(format!("{n} {x} is outside {min}..{max}")),
                    Some(_) => Ok(()),
                }
            }
            ArgType::Num { min, max } => match v.as_f64().filter(|f| f.is_finite()) {
                None => Err(format!("{n} must be a number")),
                Some(x) if x < *min || x > *max => Err(format!("{n} {x} is outside {min}..{max}")),
                Some(_) => Ok(()),
            },
            ArgType::Choice(c) => match v.as_str() {
                None => Err(format!("{n} must be a string")),
                Some(s) if c.iter().any(|x| x == s) => Ok(()),
                Some(s) => {
                    let mut near: Vec<(usize, &String)> = c.iter().map(|x| (edit_distance(s, x), x)).collect();
                    near.sort();
                    let near: Vec<&str> = near.iter().take(3).map(|(_, x)| x.as_str()).collect();
                    Err(format!("{n} '{}' is not offered; nearest: {}", inert_text(s, 40), near.join(", ")))
                }
            },
        }
    }
}

/// One catalogue entry. See the module docs for each field's source.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionEntry {
    /// Stable id (see the module table).
    pub id: String,
    /// Display name.
    pub title: String,
    /// One line, at most [`MAX_DESC_CHARS`] characters.
    pub desc: String,
    /// Where `desc` came from.
    pub desc_source: DescSource,
    /// The longer text `describe_action` shows when there is one (a parameter's both-directions desc).
    pub detail: Option<String>,
    /// Category path, at most [`MAX_PATH_DEPTH`] segments, `/`-separated.
    pub path: String,
    /// The type of function.
    pub kind: ActionKind,
    /// What running it does.
    pub effect: Effect,
    /// The arguments (empty for most).
    pub args: Vec<ArgSpec>,
    /// Whether it can be proposed right now, evaluated when the catalogue was built.
    pub available: bool,
    /// Why not, when `available` is false.
    pub why: String,
    /// Its current state in words, when the app reported one ("shown", "hidden", "now 0.4").
    pub state: Option<String>,
    /// Extra search words (a labelled judgement; AC-5 grows it).
    pub aliases: String,
    /// The source family, for counts and the tests.
    pub source: &'static str,
}

impl ActionEntry {
    /// The click path that runs this entry (AC-3). Reads the id's family and the entry's own effect: an
    /// `Unreviewed` entry, a context action or an id of an unknown family is [`Route::None`].
    pub fn route(&self) -> Route {
        if self.effect == Effect::Unreviewed {
            return Route::None;
        }
        let id = self.id.as_str();
        if id.starts_with("param:") {
            Route::Param
        } else if id.starts_with("view.") {
            Route::View
        } else if id.starts_with("window.") {
            Route::Window
        } else if id.starts_with("menu:") {
            Route::Menu
        } else if id == "timeline.set_year" {
            Route::Year
        } else if id.starts_with("select.") {
            Route::Select
        } else if id.starts_with("tool.") {
            Route::Tool
        } else if id == "edit.undo" {
            Route::Undo
        } else {
            Route::None
        }
    }

    /// Whether Session mode refuses this entry on a card (AC-3). Session mode "keeps generation out of
    /// reach at the table" (`menus.gd::SESSION_MODE_TIP`) by hiding the WORLD domain, so a card may not
    /// change a generation setting or arm a WORLD tool (sculpt, paint, region) while it is on. A labelled
    /// judgement read from `SESSION_HIDDEN_DOMAINS` (`dcc_shell.gd`) and the tool homes in `statics`.
    pub fn session_gated(&self) -> bool {
        self.kind == ActionKind::Setting || self.path.starts_with("world/")
    }

    /// The arguments as a JSON-schema object.
    pub fn args_schema(&self) -> Value {
        let props: Map<String, Value> = self.args.iter().map(|a| (a.name.clone(), a.schema())).collect();
        let req: Vec<&str> = self.args.iter().filter(|a| a.required).map(|a| a.name.as_str()).collect();
        json!({"type": "object", "properties": props, "required": req})
    }

    /// An example call for `describe_action`: the format, never a suggestion of what to choose.
    pub fn example(&self) -> Value {
        let args: Map<String, Value> = self.args.iter().filter(|a| a.required).map(|a| (a.name.clone(), a.example())).collect();
        json!({"id": self.id, "args": args})
    }

    /// Checks `args` against this entry's arguments: unknown names, missing required ones, wrong
    /// types and out-of-range values are all refused with the reason. Returns the args as written.
    pub fn check_args(&self, args: &Map<String, Value>) -> Result<Map<String, Value>, String> {
        if let Some(k) = args.keys().find(|k| !self.args.iter().any(|a| &a.name == *k)) {
            let takes: Vec<&str> = self.args.iter().map(|a| a.name.as_str()).collect();
            return Err(format!("unknown argument {}; {} takes: {}", inert_text(k, 40), self.id, if takes.is_empty() { "no arguments".into() } else { takes.join(", ") }));
        }
        for a in &self.args {
            match args.get(&a.name) {
                None if a.required => return Err(format!("missing argument {}", a.name)),
                None => {}
                Some(v) => a.check(v)?,
            }
        }
        Ok(args.clone())
    }
}

/// One runnable `CommandIndex` row as the shell reports it.
#[derive(Debug, Clone, PartialEq)]
pub struct MenuRow {
    /// The `DccMenus` `ID_` constant's name, lower-cased, without the prefix (`factions`).
    pub name: String,
    /// The row's title.
    pub title: String,
    /// The menu it sits in (`STORY`).
    pub group: String,
    /// The curated window id this row opens, when `llm_view.gd::WINDOWS` names it (so the row is
    /// merged into `window.<id>` instead of listed twice).
    pub window: Option<String>,
    /// Whether the index marks it available now.
    pub available: bool,
    /// The index's reason when it is not.
    pub why: String,
}

/// What the shell reports beyond the view facts: the runnable rows, the world flag, the year.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellFacts {
    /// Runnable (pinnable) `CommandIndex` rows.
    pub menu_rows: Vec<MenuRow>,
    /// Whether a world is open; `None` when the shell did not say (then world-bound entries are
    /// unavailable with that reason, never assumed available).
    pub has_world: Option<bool>,
    /// The year cursor's track `(min, max)`, when there is one.
    pub years: Option<(i64, i64)>,
    /// AC-3: how many settlements the open world holds, when the shell said (`bridge.settlements().size()`).
    /// `None` is "not said": no selection entry is offered then, never `Some(0)` read as "none".
    pub settlements: Option<i64>,
    /// AC-3: whether `DccApp.select_back()` has an earlier selection to return to; `None` is "not said"
    /// and offers no `select.back`.
    pub can_select_back: Option<bool>,
    /// AC-5: whether Edit > Undo has a step to take and what it would take (`DccApp.can_undo_any`,
    /// `undo_target_label`); `None` is "not said" and offers no `edit.undo`, never "nothing to undo".
    pub undo: Option<UndoFact>,
}

/// What the shell says about Edit > Undo right now (see [`ShellFacts::undo`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndoFact {
    /// Whether an undo step exists.
    pub available: bool,
    /// The step's label ("Sculpt stamp"), already inert; empty when `available` is false or the app named none.
    pub label: String,
}

impl ShellFacts {
    /// Parses the shell's JSON. Bad rows are skipped (the build's `dropped` list does not see
    /// them: they are the shell's bug, and the count of rows is what the probe compares).
    pub fn from_json_text(text: &str) -> Result<Self, String> {
        let v: Value = serde_json::from_str(text).map_err(|_| "the action facts are not JSON".to_string())?;
        let o = v.as_object().ok_or("the action facts are not an object")?;
        let mut rows = Vec::new();
        for r in o.get("menu_rows").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]) {
            let Some(r) = r.as_object() else { continue };
            let s = |k: &str| r.get(k).and_then(Value::as_str).map(str::to_owned);
            let (Some(name), Some(title)) = (s("name"), s("title")) else { continue };
            if name.is_empty() || !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
                continue;
            }
            rows.push(MenuRow {
                name,
                title: inert_text(&title, 60),
                group: inert_text(&s("group").unwrap_or_default(), 30),
                window: s("window").filter(|w| !w.is_empty()),
                available: r.get("available").and_then(Value::as_bool).unwrap_or(false),
                why: inert_text(&s("why").unwrap_or_default(), 120),
            });
        }
        let years = o.get("timeline").and_then(Value::as_object).and_then(|t| Some((t.get("min")?.as_i64()?, t.get("max")?.as_i64()?)));
        Ok(ShellFacts {
            menu_rows: rows,
            has_world: o.get("has_world").and_then(Value::as_bool),
            years,
            settlements: o.get("settlements").and_then(Value::as_i64).filter(|n| *n >= 0),
            can_select_back: o.get("can_select_back").and_then(Value::as_bool),
            undo: o.get("undo").and_then(Value::as_object).and_then(|u| {
                Some(UndoFact { available: u.get("available")?.as_bool()?, label: inert_text(u.get("label").and_then(Value::as_str).unwrap_or(""), 60) })
            }),
        })
    }
}

/// An entry the build refused to offer, with the reason (a test and the probe read these).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dropped {
    /// The id it would have had.
    pub id: String,
    /// Why it was dropped.
    pub why: String,
}

/// The whole table for one request.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionCatalogue {
    entries: Vec<ActionEntry>,
    /// Entries refused by construction (forbidden words, depth, duplicate ids, over-long ids).
    pub dropped: Vec<Dropped>,
}

/// Words that may never appear (as a substring of the id) for an entry to be offered: the view
/// vocabulary's own list ([`FORBIDDEN_STEMS`], reused so there is ONE list), plus [`EXTRA_STEMS`].
/// Returns the first stem found. `preset` is stripped first (it contains `reset`).
pub fn refusal(id: &str, title: &str) -> Option<&'static str> {
    let slug = |s: &str| {
        let mut o = String::new();
        for c in s.to_lowercase().chars() {
            if c.is_ascii_alphanumeric() {
                o.push(c);
            } else if !o.ends_with('_') {
                o.push('_');
            }
        }
        o
    };
    let probe = format!("{}|{}", slug(id).replace("preset", ""), slug(title).replace("preset", ""));
    if let Some(s) = FORBIDDEN_STEMS.iter().chain(EXTRA_STEMS.iter()).copied().find(|s| probe.contains(s)) {
        return Some(s);
    }
    // Whole-word stems: short words that are substrings of harmless ones (`cut` in `execute`).
    let words: BTreeSet<&str> = probe.split(|c: char| !c.is_ascii_alphanumeric()).collect();
    EXTRA_WORDS.iter().copied().find(|w| words.contains(w))
}

/// More forbidden substrings, beyond the view list, for rows the index can carry that a view
/// action cannot: found by reading the 2026-10-31 menu rows (`Clear library`, `Clear caches`,
/// `Run pipeline`, `Delete`). A labelled judgement; a false positive drops one entry and says so.
pub const EXTRA_STEMS: [&str; 6] = ["wipe", "purge", "discard", "overwrite", "pipeline", "evict"];

/// Whole-word forbidden stems (see [`refusal`]).
pub const EXTRA_WORDS: [&str; 6] = ["cut", "paste", "exit", "restart", "run", "regen"];

/// AC-5: the REVIEWED allow-list of ids that [`refusal`] would drop although they do nothing
/// destructive: `(exact id, fixed title, why it is safe)`. The guard itself ([`refusal`], its stems and
/// the view tool's [`FORBIDDEN_STEMS`]) is untouched; this list is consulted only by [`Builder::push`], by
/// EXACT id (never by stem or prefix), and it also REPLACES the entry's title with the fixed one here, so
/// a shell row that reuses the id under a hostile title keeps no hostile title. Found by the 2026-11-07
/// real-row run (AC-3's known gap: "Undo that" had no offered action): each row below was dropped for a
/// word in its LABEL that describes a harmless thing. Reviewed 2026-11-08 by reading each row's handler:
///
/// * `edit.undo` is Edit > Undo itself, the existing undo path (`DccApp.undo_last`, the one entry point
///   Ctrl+Z, the Edit row and the toolbar square share). It reverts the newest step and Redo brings it
///   back; it runs ALONE on its card (rule C8), so a card never undoes a step it just made.
/// * `menu:undo_history` opens the dock's History panel (`DccApp.open_undo_history` ->
///   `right_dock_ctrl.show_history`): a read-only list, it reverts nothing.
/// * `carto.region_start` arms the Region tool's first corner; "export" is the box's PURPOSE and nothing
///   is exported or written.
/// * `world.river_clear` removes a temporary highlight drawn over the map; no world data changes.
///
/// NOT here, on purpose: the draft discards (`civ.territory_discard`, `world.paint_discard`,
/// `world.sculpt_discard`), `world.paint_erase` and `world.sculpt_undo`. Each throws away the user's
/// unsaved draft or stamp, which is the kind of thing the stems exist to keep from a model; no click path
/// runs them from a proposal either, so listing them would only invite a model to propose what Apply must
/// refuse. Adding a row here needs a reason in this table and the test
/// `the_refusal_allow_list_is_exact_reviewed_and_no_wider`.
pub const REFUSAL_EXEMPT: &[(&str, &str, &str)] = &[
    ("edit.undo", "Undo the last change", "Edit > Undo through DccApp.undo_last; Redo reverses it; alone on its card"),
    ("menu:undo_history", "Undo history", "opens the History panel; a read-only list"),
    ("carto.region_start", "Start export region here", "arms the Region box's first corner; nothing is exported"),
    ("world.river_clear", "Clear river highlight", "removes a temporary highlight; no world data changes"),
];

/// Reviewed menu rows: `(constant name, kind, effect, path, description)`. A labelled judgement
/// from reading each row's handler and title on 2026-10-31: every one opens a window or panel and
/// changes no world data. The test `every_review_row_names_a_real_menu_constant` checks each name
/// against `menus.gd`, so a renamed constant fails there rather than silently un-reviewing a row.
/// Rows the refusal list drops (`Undo history`, `Start export region here`) are NOT here: a review
/// row that [`refusal`] would drop is dead, and `no_review_row_is_refused` fails on one.
pub const MENU_REVIEW: &[(&str, ActionKind, Effect, &str, &str)] = &[
    ("find_on_map", ActionKind::Window, Effect::OpensWindow, "window/edit", "Search the map for a place, label or landmark by name and jump to it."),
    ("places", ActionKind::Window, Effect::OpensWindow, "window/records", "Open the table of places: settlements and landmarks."),
    ("places_without_notes", ActionKind::Window, Effect::OpensWindow, "window/records", "Open the list of places that have no vault note yet."),
    ("story_political", ActionKind::Window, Effect::OpensWindow, "window/story", "Open the political history (who held it, when) of the selected place."),
    ("story_faction_history", ActionKind::Window, Effect::OpensWindow, "window/story", "Open the history of the selected faction."),
    ("story_assistant", ActionKind::Window, Effect::OpensWindow, "window/story", "Open the Assistant panel."),
    ("vault_browse", ActionKind::Window, Effect::OpensWindow, "window/records", "Browse and edit a Markdown note in the vault."),
    ("pref_assistant", ActionKind::Window, Effect::OpensWindow, "window/preferences", "Open the Assistant's settings: provider, model and what it may read."),
    ("pref_quick_bar", ActionKind::Window, Effect::OpensWindow, "window/preferences", "Open the settings for the buttons pinned to the top bar."),
    ("pref_storage", ActionKind::Window, Effect::OpensWindow, "window/preferences", "Show where Cartalith keeps its files."),
    ("pref_shortcuts", ActionKind::Window, Effect::OpensWindow, "window/preferences", "Open the list of keyboard shortcuts."),
    ("undo_history", ActionKind::Window, Effect::OpensWindow, "window/edit", "Show the undo ledger in the right dock: the steps that can be undone."),
    ("slicer", ActionKind::Window, Effect::OpensWindow, "window/assets", "Open the sprite-sheet slicer for cutting art into assets."),
    ("help_credits", ActionKind::Window, Effect::OpensWindow, "window/help", "Show the credits and the academic principles behind the generator."),
    ("help_about", ActionKind::Window, Effect::OpensWindow, "window/help", "Show the version and what Cartalith is."),
];

/// The curated windows' areas and descriptions (`llm_view.gd::WINDOWS` ids): `(id, path, description)`.
/// A window id the shell sends that is not here still gets an entry, with a derived description.
pub const WINDOW_REVIEW: &[(&str, &str, &str)] = &[
    ("asset_library", "window/assets", "Open the asset library: browse, tag and apply map art."),
    ("factions", "window/records", "Open the faction roster: every faction with its settlements and strength."),
    ("world_data", "window/records", "Open the world data tables: settlements, factions, ways and more, sortable."),
    ("vault", "window/records", "Open the Markdown vault: the notes written about this world."),
    ("timeline", "window/story", "Open the timeline: years, events and who held what."),
    ("journey_planner", "window/story", "Open the journey planner: routes, stops and travel times."),
    ("journeys_list", "window/story", "Open the saved journeys."),
    ("travel_library", "window/story", "Open the travel library: modes of travel and their speeds."),
    ("conflicts", "window/story", "Open the list of conflicts drawn on the map."),
    ("shortcuts", "window/help", "Open the list of keyboard shortcuts."),
];

/// What a reviewed context action is: `(id, kind, effect, takes a map point, description)`. A
/// labelled judgement from each action's handler in the workspace files: these are the ones that
/// act at a map point (or on the selection) and need no further input. Anything else the scan
/// finds (a draft's commit or discard, a radius, a class) is listed `unreviewed`.
pub const CONTEXT_REVIEW: &[(&str, ActionKind, Effect, bool, &str)] = &[
    ("global.centre", ActionKind::View, Effect::View, true, "Centre the map view on a point."),
    ("carto.region_start", ActionKind::Edit, Effect::View, true, "Arm the Region tool and start a box at a point; drag to the opposite corner."),
    ("world.river_clear", ActionKind::View, Effect::View, false, "Remove the river highlight drawn over the map; no world data changes."),
    ("global.measure_from", ActionKind::View, Effect::View, true, "Start a measurement from a point."),
    ("global.copy_coordinate", ActionKind::View, Effect::View, true, "Copy a point's coordinate to the clipboard."),
    ("civ.info_here", ActionKind::Query, Effect::Read, true, "Read the settlement and ecology at a point."),
    ("civ.landmark_why", ActionKind::Query, Effect::Read, true, "Explain why a landmark is at a point."),
    ("civ.drop_settlement", ActionKind::Edit, Effect::Undoable, true, "Place a new settlement at a point."),
    ("civ.start_way", ActionKind::Edit, Effect::View, true, "Start drawing a way (road or path) at a point."),
    ("civ.start_route", ActionKind::Edit, Effect::View, true, "Start drawing a travel route at a point."),
    ("civ.add_event", ActionKind::Edit, Effect::Undoable, true, "Add a timeline event at a point."),
    ("civ.journey_from", ActionKind::Edit, Effect::View, true, "Plan a journey starting from a point."),
    ("civ.journey_to", ActionKind::Edit, Effect::View, true, "Plan a journey ending at a point."),
    ("carto.add_label", ActionKind::Edit, Effect::Undoable, true, "Add a text label at a point."),
    ("carto.stamp_icon", ActionKind::Edit, Effect::Undoable, true, "Stamp the armed icon at a point."),
    ("world.river_inspect", ActionKind::Query, Effect::Read, true, "Read the river at a point: size, flow and name."),
    ("world.river_trace", ActionKind::Query, Effect::Read, true, "Trace the river at a point downstream."),
    ("world.river_catchment", ActionKind::Query, Effect::Read, true, "Show the catchment area that drains to a point."),
    ("world.section_from", ActionKind::View, Effect::View, true, "Start a cross-section profile from a point."),
    // 8fcab5fc (MS-5): the lead verb on empty ground. Handler read 2026-11-11: `app.arm_tool("sculpt")` only; it arms the tool and
    // draws nothing (a stamp needs a drag), like `carto.region_start`. No map point is taken.
    ("world.sculpt_here", ActionKind::Edit, Effect::View, false, "Arm the Sculpt tool; nothing changes until a drag stamps the terrain."),
];

/// AC-5: the descriptions of rows and context actions that are LISTED but not reviewed to run
/// (`Effect::Unreviewed`): `(id, description)`. Each was written 2026-11-08 from reading the row's
/// handler (named in the comment beside it), so a model reading the catalogue learns what the thing is
/// even though it cannot propose it. A description here makes NO claim that the row is safe to run:
/// that is what [`MENU_REVIEW`] and [`CONTEXT_REVIEW`] decide. The test
/// `ac5_no_entry_has_a_derived_description_on_the_real_rows` fails when a real row is missing here, and
/// `ac5_descriptions_are_real_sentences` fails on a placeholder shape.
pub const DESC_ONLY: &[(&str, &str)] = &[
    // menus.gd: ID_AP_PACK_META -> _open_pack_metadata (name, author, licence fields; written to pack.json on export).
    ("menu:ap_pack_meta", "Edit the asset pack's name, author and licence; they are written into pack.json on the next pack export."),
    // asset_library_window.gd::_on_validate (AssetValidator's ordered warning list).
    ("menu:ap_validate", "Check the asset pack for problems and show the ordered list of warnings."),
    // asset_library_window.gd::_on_apply_to_map (installs the library as the map's art).
    ("menu:apply_library", "Install the asset library as the map's art, as the library's Apply to map button does."),
    // app.gd::open_pack_compare_picker (AP-9's diff card with its three merge choices).
    ("menu:compare_pack", "Pick an asset pack and see how it differs from the library, with three ways to merge it."),
    // app.gd::open_pack_in_library_picker (AP-1: into the library only, the map is left as it is).
    ("menu:open_pack_library", "Pick an asset pack and add it to the library only; the map is left as it is."),
    // asset_library_window.gd::report_now (AP-2's report card).
    ("menu:pack_report", "Open the asset library and show its report card for the current pack."),
    // menus.gd::_copy_selection (snapshots the label and icon selection; never changes the map).
    ("menu:copy", "Copy the selected map labels and icons to the app's clipboard; nothing on the map changes."),
    // menus.gd: ID_DESELECT -> _host.clear_selection().
    ("menu:deselect", "Drop the current selection on the map; Previous selection brings it back."),
    // menus.gd::_select_all (scoped to the active domain).
    ("menu:select_all", "Select everything selectable in the active domain."),
    // app.gd::show_project_on_disk (reveals the project file in the file manager; a no-op unsaved or on touch).
    ("menu:show_on_disk", "Reveal the project file in the computer's file manager; nothing is changed or written."),
    // app.gd::open_storage_locations.
    ("menu:storage", "Open the dialog that shows, and changes, where Cartalith keeps its files."),
    // menus.gd: ID_HELP_DOCS (opens _docs_dir() in the OS shell).
    ("menu:help_docs", "Open Cartalith's documentation folder in the operating system's file manager."),
    // menus.gd::_on_relief_exag (forgets the stored value; applies on the NEXT Generate, nothing regenerates).
    ("menu:gfx_exag_engine", "Forget the stored relief exaggeration and use the engine's own value from the next Generate."),
    // menus.gd: ID_LOD_ENTER_NOW -> viewport request_lod_entry.
    ("menu:lod_enter_now", "Switch the map to deep-detail tiles now, if you are zoomed in far enough."),
    // menus.gd: ID_STORY_DRAW_CONFLICT (selects CIVIL and arms the Conflict tool).
    ("menu:story_draw_conflict", "Go to the CIVIL domain and arm the Conflict tool to draw a conflict on the map."),
    // menus.gd::_prompt_forget_layout (asks which saved layout, then a confirmation).
    ("menu:win_forget_layout", "Ask which saved window layout to forget, and delete it after a confirmation."),
    // app.gd::open_data_manager (DataManagerWindow.ROUTES: Import and Export groups).
    ("menu:data_manager", "Open the Data manager: import and export routes for maps, heightmaps, GIS data and world data."),
    // retired id 722; the row survives in older captures and opens the same timeline as window.timeline.
    ("menu:world_timeline", "Open the timeline: years, events and who held what."),
    // cartography_workspace.gd context rows.
    ("carto.style_preset", "Pick a style preset from the context card's submenu; it changes the map's whole look."),
    ("carto.view_field", "Choose which data field the map displays, from the context card's submenu."),
    // civilization_workspace.gd context rows.
    ("civ.open_city_layout", "Open the city viewer for the settlement under the pointer."),
    ("civ.settlement_class", "Step the settlement class that the Settlement tool will place next."),
    ("civ.territory_commit", "Write the territory outline being drawn into the territory map."),
    // world_workspace.gd context row (shown while the Paint tool is armed).
    ("world.paint_radius", "Step the Paint tool's brush radius, in cells."),
];

/// The [`DESC_ONLY`] description of `id`, when there is one.
fn desc_only(id: &str) -> Option<&'static str> {
    DESC_ONLY.iter().find(|(i, _)| *i == id).map(|(_, d)| *d)
}

/// Why a context-card action cannot be proposed yet (AC-3).
pub const CONTEXT_NOT_RUNNABLE: &str = "It runs from the map's right-click card at a point; the assistant cannot run it from a proposal yet.";

/// Curated tool descriptions: `(tool id, description)`. Each arming changes only the session (no
/// world change until the user acts with the tool), so every tool's effect is [`Effect::View`].
pub const TOOL_REVIEW: &[(&str, &str)] = &[
    ("inspect", "Arm Inspect: click the map to read what is under a point."),
    ("region", "Arm Region: drag a box on the map to choose an area to edit or export."),
    ("measure", "Arm Measure: click points on the map to measure distance, area or a section."),
    ("sculpt", "Arm Sculpt: stamp raise, lower or smooth brushes onto the terrain."),
    ("paint", "Arm Paint: paint colour marks over the map."),
    ("settlement", "Arm Settlement: click the map to place a settlement."),
    ("way", "Arm Way: click points to draw a road or path."),
    ("route", "Arm Route: click points to draw a travel route."),
    ("territory", "Arm Territory: draw a faction's territory outline."),
    ("journey", "Arm Journey: click stops to plan a journey."),
    ("conflict", "Arm Conflict: draw a conflict on the map."),
    ("label", "Arm Label: click the map to place a text label."),
    ("icon", "Arm Icon: click the map to stamp the chosen icon."),
];

/// Extra search words for entries whose own wording misses how people ask: `(id, words)`. The
/// AC-0 reach test found "Let me draw a new road" matched nothing ("Way" says no "road") and
/// "make the mountains higher" ranked its answer 7th ("Peak height" says no "mountain").
/// A labelled judgement; AC-5 grows it from the 30-task script.
pub const ALIASES: &[(&str, &str)] = &[
    ("tool.way", "road roads path street track"),
    ("tool.route", "road trade route travel line"),
    ("tool.settlement", "town city village place"),
    ("civ.drop_settlement", "town city village place found"),
    ("civ.start_way", "road path street"),
    ("window.factions", "kingdoms empires nations realms"),
    ("window.vault", "notes lore writing"),
    ("window.journey_planner", "travel trip itinerary"),
    ("window.shortcuts", "hotkeys keys keyboard"),
    ("window.world_data", "spreadsheet table statistics"),
    ("timeline.set_year", "date era time history"),
    ("view.style_preset", "look theme appearance antique parchment nautical satellite"),
    ("view.zoom_to", "pan centre focus camera"),
];

/// Search words added to every registry parameter by key stem: the registry's keys read like
/// `relief.peak_height`, whose words are not the player's. `(key fragment, words)`. Labelled judgement.
pub const PARAM_ALIASES: &[(&str, &str)] =
    &[("peak", "mountain mountains high tall"), ("sea_level", "ocean water flood coast"), ("rain", "wet dry drought"), ("temp", "hot cold warm cool climate")];

impl ActionCatalogue {
    /// Builds the catalogue. See the module docs; pure and deterministic.
    ///
    /// `params` is the registry catalogue of this request (absent when the parameter source is
    /// off); `view` the view facts (absent or empty when no map is built); `shell` the runnable
    /// rows and flags. Never fails: a source that cannot be read contributes nothing and a
    /// refused entry is recorded in [`ActionCatalogue::dropped`].
    pub fn build(params: Option<&Catalogue>, view: Option<&ViewCatalogue>, shell: &ShellFacts) -> Self {
        let mut b = Builder { entries: BTreeMap::new(), dropped: Vec::new() };
        let no_world = match shell.has_world {
            Some(true) => None,
            Some(false) => Some("No world is open."),
            None => Some("The app did not say whether a world is open."),
        };
        if let Some(c) = params {
            b.params(c);
        }
        if let Some(v) = view {
            b.view(v, no_world);
        }
        b.menu_rows(shell);
        b.statics(no_world, view);
        b.selection(shell, no_world);
        b.history(shell);
        if let Some((lo, hi)) = shell.years {
            b.push(ActionEntry {
                id: "timeline.set_year".into(),
                title: "Go to year".into(),
                desc: "Move the shared timeline cursor to a year; the map and records follow it.".into(),
                desc_source: DescSource::Curated,
                detail: Some(format!("The track runs {lo} to {hi}; the cursor is clamped to it by the app, but a year outside it is refused here.")),
                path: "story/time".into(),
                kind: ActionKind::Select,
                effect: Effect::View,
                args: vec![ArgSpec::new("year", ArgType::Int { min: lo, max: hi }, true).hint("The year to go to, as a whole number such as 398 (never a word).")],
                available: no_world.is_none(),
                why: no_world.unwrap_or("").into(),
                state: None,
                aliases: String::new(),
                source: "timeline",
            });
        }
        let Builder { entries, dropped } = b;
        let mut entries: Vec<ActionEntry> = entries.into_values().collect();
        entries.sort_by(|a, b| (a.path.as_str(), a.id.as_str()).cmp(&(b.path.as_str(), b.id.as_str())));
        // `dropped` is a record, not an offer, but it is part of the build's output and the
        // determinism bar covers it: sort it so the input order of the shell's rows cannot show.
        let mut dropped = dropped;
        dropped.sort_by(|a, b| (a.id.as_str(), a.why.as_str()).cmp(&(b.id.as_str(), b.why.as_str())));
        ActionCatalogue { entries, dropped }
    }

    /// All entries, sorted by `(path, id)`.
    pub fn entries(&self) -> &[ActionEntry] {
        &self.entries
    }

    /// One entry by exact id.
    pub fn get(&self, id: &str) -> Option<&ActionEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// True when nothing at all is offered (the tools are then not in the request).
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Entries whose description was derived, not written (AC-5's work list).
    pub fn derived_desc_count(&self) -> usize {
        self.entries.iter().filter(|e| e.desc_source == DescSource::Derived).count()
    }

    /// Entries of one source family.
    pub fn count_source(&self, source: &str) -> usize {
        self.entries.iter().filter(|e| e.source == source).count()
    }
}

/// Accumulates entries, keyed by id so a duplicate is caught, applying the by-construction rules.
struct Builder {
    entries: BTreeMap<String, ActionEntry>,
    dropped: Vec<Dropped>,
}

impl Builder {
    /// The one door every entry goes through: forbidden words, id length, path depth, description
    /// length, duplicates. An entry that fails is recorded, never offered.
    fn push(&mut self, mut e: ActionEntry) {
        let id = e.id.clone();
        let mut drop = |why: String| self.dropped.push(Dropped { id: id.clone(), why });
        if e.id.is_empty() || e.id.chars().count() > MAX_ID_CHARS {
            drop(format!("id is empty or longer than {MAX_ID_CHARS} characters"));
            return;
        }
        // A parameter row is a VALUE the user changes and a later Generate applies; its label is the
        // registry's own wording and may say "generated" (`carve_rivers`: the 2026-10-31 real-row run
        // found it refused on its title). The id is still checked; every action-like source is checked
        // on its title too.
        let title = if e.source == "params" { "" } else { e.title.as_str() };
        // AC-5: an id on the reviewed allow-list skips the stem check, by exact id only, and takes the
        // fixed title from that list (so the title the stem check would have read is not the shell's).
        if let Some((_, fixed, _)) = REFUSAL_EXEMPT.iter().find(|(i, ..)| *i == e.id) {
            e.title = (*fixed).to_owned();
        } else if let Some(s) = refusal(&e.id, title) {
            drop(format!("contains the forbidden word '{s}'"));
            return;
        }
        if e.path.split('/').count() > MAX_PATH_DEPTH || e.path.is_empty() || e.path.split('/').any(str::is_empty) {
            drop(format!("path '{}' is empty or deeper than {MAX_PATH_DEPTH} levels", e.path));
            return;
        }
        if self.entries.contains_key(&e.id) {
            drop("the id appears twice".into());
            return;
        }
        e.title = inert_text(&e.title, 60);
        e.desc = inert_text(&e.desc, MAX_DESC_CHARS);
        if e.desc.is_empty() {
            e.desc = inert_text(&format!("{}.", e.title), MAX_DESC_CHARS);
            e.desc_source = DescSource::Derived;
        }
        if let Some((_, w)) = ALIASES.iter().find(|(i, _)| *i == e.id) {
            e.aliases = (*w).to_owned();
        }
        self.entries.insert(e.id.clone(), e);
    }

    /// One entry per offered registry row.
    fn params(&mut self, c: &Catalogue) {
        for r in &c.rows {
            let ty = match r.kind {
                PKind::Bool => ArgType::Bool,
                PKind::Int => ArgType::Int { min: r.min.ceil() as i64, max: r.max.floor() as i64 },
                PKind::Float => ArgType::Num { min: r.min, max: r.max },
            };
            let home = if r.group == "civ" { "civil/generation".to_owned() } else { format!("world/generation/{}", r.group) };
            let aliases: Vec<&str> = PARAM_ALIASES.iter().filter(|(f, _)| r.key.contains(f)).map(|(_, w)| *w).collect();
            let title = if r.unit.is_empty() { r.label.clone() } else { format!("{} ({})", r.label, r.unit) };
            // `describe_action` shows the registry's full sentence when the one-line form had to be
            // cut, plus the two notes the registry carries (a ruled switch, a setting that does
            // nothing right now), so nothing the parameter card shows is lost in the catalogue.
            let mut detail = if r.desc.chars().count() > MAX_DESC_CHARS { r.desc.clone() } else { String::new() };
            if r.advanced {
                detail.push_str(" Ruled behaviour: flipping it reproduces older output on purpose.");
            }
            if let Some(w) = &r.inert {
                detail.push_str(&format!(" Does nothing now: {w}."));
            }
            self.push(ActionEntry {
                id: format!("param:{}", r.key),
                title,
                desc: r.desc.clone(),
                desc_source: DescSource::Registry,
                detail: Some(detail.trim().to_owned()).filter(|d| !d.is_empty()),
                path: home,
                kind: ActionKind::Setting,
                effect: Effect::Regenerates,
                args: vec![ArgSpec::new("value", ty, true)],
                available: true,
                why: String::new(),
                state: r.current.map(|v| format!("now {}", v.to_json(r.kind))),
                aliases: aliases.join(" "),
                source: "params",
            });
        }
    }

    /// The view verbs and the curated windows, from the facts.
    fn view(&mut self, v: &ViewCatalogue, no_world: Option<&'static str>) {
        let f = v.facts();
        let withheld = |a: &str| f.withheld.get(a).cloned();
        for l in &f.layers {
            self.push(ActionEntry {
                id: format!("view.layer.{}", l.id),
                title: format!("{} layer", l.label),
                desc: format!("Show or hide the {} layer on the map.", l.label),
                desc_source: DescSource::Curated,
                detail: None,
                path: "carto/layers".into(),
                kind: ActionKind::View,
                effect: Effect::View,
                args: vec![ArgSpec::new("visible", ArgType::Bool, true)],
                available: true,
                why: String::new(),
                state: l.on.map(|b| if b { "shown".into() } else { "hidden".into() }),
                aliases: String::new(),
                source: "view",
            });
        }
        let verb = |id: &str, title: &str, desc: &str, path: &str, effect: Effect, args: Vec<ArgSpec>, act: &str, present: bool| {
            let why = if present { None } else { Some(withheld(act).unwrap_or_else(|| "nothing is available for it right now".into())) };
            (id.to_owned(), title.to_owned(), desc.to_owned(), path.to_owned(), effect, args, why)
        };
        let mut rows: Vec<(String, String, String, String, Effect, Vec<ArgSpec>, Option<String>)> = Vec::new();
        let preset_ids: Vec<String> = f.presets.iter().map(|p| p.id.clone()).collect();
        let preset_active = f.presets.iter().find(|p| p.on == Some(true)).map(|p| p.label.clone());
        if !f.presets.is_empty() || f.withheld.contains_key("set_style_preset") {
            rows.push(verb(
                "view.style_preset",
                "Style preset",
                "Switch the map's whole look to a named preset; Edit > Undo reverts it.",
                "carto/style",
                Effect::Undoable,
                vec![ArgSpec::new("preset", ArgType::Choice(preset_ids), true).hint("The look to switch to; one of the listed values exactly.")],
                "set_style_preset",
                !f.presets.is_empty(),
            ));
        }
        if f.camera.is_some() || f.withheld.contains_key("zoom_to") {
            let (w, h, lo, hi) = f.camera.as_ref().map(|c| (c.w, c.h, c.zoom_min, c.zoom_max)).unwrap_or((1, 1, 1.0, 1.0));
            rows.push(verb(
                "view.zoom_to",
                "Centre and zoom the map",
                "Move the view to a grid cell, optionally setting the zoom.",
                "map/view",
                Effect::View,
                vec![
                    ArgSpec::new("x", ArgType::Num { min: 0.0, max: f64::from(w.saturating_sub(1)) }, true).hint("The grid column to centre on, from 0 at the west edge."),
                    ArgSpec::new("y", ArgType::Num { min: 0.0, max: f64::from(h.saturating_sub(1)) }, true).hint("The grid row to centre on, from 0 at the north edge."),
                    ArgSpec::new("zoom", ArgType::Num { min: lo, max: hi }, false).hint("The zoom factor; 1 is the whole map."),
                ],
                "zoom_to",
                f.camera.is_some(),
            ));
        }
        if !f.kinds.is_empty() || f.withheld.contains_key("set_landmark_filter") {
            rows.push(verb(
                "view.landmark_filter",
                "Show or hide a landmark type",
                "Show or hide one type of landmark (peaks, fords and so on) on the map.",
                "carto/layers",
                Effect::View,
                vec![ArgSpec::new("kind", ArgType::Choice(f.kinds.iter().map(|k| k.id.clone()).collect()), true), ArgSpec::new("visible", ArgType::Bool, true)],
                "set_landmark_filter",
                !f.kinds.is_empty(),
            ));
        }
        for (id, title, desc, path, effect, args, why) in rows {
            let state = (id == "view.style_preset").then(|| preset_active.clone().map(|a| format!("now {a}"))).flatten();
            self.push(ActionEntry {
                id,
                title,
                desc,
                desc_source: DescSource::Curated,
                detail: None,
                path,
                kind: ActionKind::View,
                effect,
                args,
                available: why.is_none(),
                why: why.unwrap_or_default(),
                state,
                aliases: String::new(),
                source: "view",
            });
        }
        for w in &f.windows {
            let (path, desc, src) = match WINDOW_REVIEW.iter().find(|(i, _, _)| *i == w.id) {
                Some((_, p, d)) => ((*p).to_owned(), (*d).to_owned(), DescSource::Curated),
                None => ("window/other".to_owned(), format!("Open the {} window.", w.label), DescSource::Derived),
            };
            self.push(ActionEntry {
                id: format!("window.{}", w.id),
                title: format!("Open {}", w.label),
                desc,
                desc_source: src,
                detail: None,
                path,
                kind: ActionKind::Window,
                effect: Effect::OpensWindow,
                args: vec![],
                available: true,
                why: String::new(),
                state: None,
                aliases: String::new(),
                source: "window",
            });
        }
        let _ = no_world;
    }

    /// Runnable `CommandIndex` rows. A row that opens a curated window is merged into it.
    fn menu_rows(&mut self, shell: &ShellFacts) {
        for r in &shell.menu_rows {
            if let Some(w) = &r.window {
                // Covered by `window.<id>`; recorded so the probe can account for every row.
                self.dropped.push(Dropped { id: format!("menu:{}", r.name), why: format!("merged into window.{w}") });
                continue;
            }
            let id = format!("menu:{}", r.name);
            let reviewed = MENU_REVIEW.iter().find(|(n, ..)| *n == r.name);
            let group = r.group.to_lowercase();
            let (kind, effect, path, desc, src) = match reviewed {
                Some((_, k, e, p, d)) => (*k, *e, (*p).to_owned(), (*d).to_owned(), DescSource::Curated),
                None => (
                    ActionKind::View,
                    Effect::Unreviewed,
                    format!("more/{}", if group.is_empty() { "menus".to_owned() } else { group.replace(|c: char| !c.is_ascii_alphanumeric(), "_") }),
                    // AC-5: a written description when there is one; the derived sentence is only the
                    // fallback for a row that appears after the last review (the real-row test fails on it).
                    desc_only(&id).map_or_else(|| format!("{} menu: {}.", if r.group.is_empty() { "App" } else { &r.group }, r.title), str::to_owned),
                    if desc_only(&id).is_some() { DescSource::Curated } else { DescSource::Derived },
                ),
            };
            let (available, why) = if effect == Effect::Unreviewed {
                (false, "Not yet reviewed for assistant use; the user can run it from the menu.".to_owned())
            } else {
                (r.available, r.why.clone())
            };
            self.push(ActionEntry {
                id,
                title: r.title.clone(),
                desc,
                desc_source: src,
                detail: None,
                path,
                kind,
                effect,
                args: vec![],
                available,
                why,
                state: None,
                aliases: String::new(),
                source: "menu",
            });
        }
    }

    /// Context actions and tools, from the embedded static scan.
    fn statics(&mut self, no_world: Option<&'static str>, view: Option<&ViewCatalogue>) {
        let data: Value = serde_json::from_str(STATIC_SOURCES).unwrap_or(Value::Null);
        let (gw, gh) = view.and_then(|v| v.facts().camera.as_ref().map(|c| (c.w, c.h))).unwrap_or((65_536, 65_536));
        let point_args = || {
            vec![
                ArgSpec::new("x", ArgType::Num { min: 0.0, max: f64::from(gw.saturating_sub(1)) }, true),
                ArgSpec::new("y", ArgType::Num { min: 0.0, max: f64::from(gh.saturating_sub(1)) }, true),
            ]
        };
        for c in data["context"].as_array().map(Vec::as_slice).unwrap_or(&[]) {
            let (Some(id), Some(label)) = (c["id"].as_str(), c["label"].as_str()) else { continue };
            let dom = id.split('.').next().unwrap_or("map");
            let home = match dom {
                "global" => "map/here",
                "civ" => "civil/here",
                "carto" => "carto/here",
                "world" => "world/here",
                _ => "map/here",
            };
            let title = label.replace('\u{2026}', "");
            let reviewed = CONTEXT_REVIEW.iter().find(|(i, ..)| *i == id);
            let (kind, effect, args, desc, src, avail, why) = match reviewed {
                // AC-3: no click path exists to RUN a context-card action from a proposal (each is
                // reached by a right-click at a point and the card's own handler), and the rule is that
                // a card never offers what Apply would refuse. So they stay listed, searchable and
                // describable, and `propose_actions` refuses them (B7) with this reason.
                Some((_, k, e, pt, d)) => (*k, *e, if *pt { point_args() } else { vec![] }, (*d).to_owned(), DescSource::Curated, false, CONTEXT_NOT_RUNNABLE.to_owned()),
                None => (
                    ActionKind::Edit,
                    Effect::Unreviewed,
                    vec![],
                    desc_only(id).map_or_else(|| format!("Context-card action: {title}."), str::to_owned),
                    if desc_only(id).is_some() { DescSource::Curated } else { DescSource::Derived },
                    false,
                    "Not yet reviewed for assistant use; the user can run it from the context card.".to_owned(),
                ),
            };
            self.push(ActionEntry {
                id: id.to_owned(),
                title,
                desc,
                desc_source: src,
                detail: None,
                path: home.into(),
                kind,
                effect,
                args,
                available: avail,
                why,
                state: None,
                aliases: String::new(),
                source: "context",
            });
        }
        for t in data["tools"].as_array().map(Vec::as_slice).unwrap_or(&[]) {
            let Some(t) = t.as_str() else { continue };
            let home = match t {
                "sculpt" | "paint" | "region" => "world/tools",
                "measure" | "inspect" => "map/tools",
                "settlement" | "way" | "route" | "territory" | "journey" => "civil/tools",
                "conflict" => "story/tools",
                "label" | "icon" => "carto/tools",
                _ => "map/tools",
            };
            let (desc, src) = match TOOL_REVIEW.iter().find(|(i, _)| *i == t) {
                Some((_, d)) => ((*d).to_owned(), DescSource::Curated),
                None => (format!("Arm the {t} tool."), DescSource::Derived),
            };
            self.push(ActionEntry {
                id: format!("tool.{t}"),
                title: format!("{} tool", capitalise(t)),
                desc,
                desc_source: src,
                detail: None,
                path: home.into(),
                kind: ActionKind::Edit,
                effect: Effect::View,
                args: vec![],
                available: no_world.is_none(),
                why: no_world.unwrap_or("").into(),
                state: None,
                aliases: String::new(),
                source: "tool",
            });
        }
    }
}

impl Builder {
    /// AC-3: the selection verbs over `DccApp.select` (PC-1): `select.settlement`, `select.none`,
    /// `select.back`. Offered only when the shell said the counts (`ShellFacts::settlements`,
    /// `can_select_back`); a missing fact offers nothing. They are `Effect::View`: the selection is session
    /// state with its own back history (Alt+Left), not on the undo stack. `select.none` is named for what
    /// it does, never `clear` (a forbidden stem). Settlements are the only kind offered: `DccApp` records a
    /// faction or landmark selection but shows it nowhere until PC-4.
    fn selection(&mut self, shell: &ShellFacts, no_world: Option<&'static str>) {
        let Some(n) = shell.settlements else { return };
        let why_world = no_world.map(str::to_owned);
        let has = n > 0;
        let (avail, why) = match (&why_world, has) {
            (Some(w), _) => (false, w.clone()),
            (None, false) => (false, "This world has no settlements to select.".to_owned()),
            (None, true) => (true, String::new()),
        };
        let mk = |id: &str, title: &str, desc: &str, args: Vec<ArgSpec>, available: bool, why: String, aliases: &str| ActionEntry {
            id: id.into(),
            title: title.into(),
            desc: desc.into(),
            desc_source: DescSource::Curated,
            detail: None,
            path: "map/selection".into(),
            kind: ActionKind::Select,
            effect: Effect::View,
            args,
            available,
            why,
            state: None,
            aliases: aliases.into(),
            source: "select",
        };
        self.push(mk(
            "select.settlement",
            "Select a settlement",
            "Select one settlement by id (its position in the settlement list, from 0); the dock and CIVIL follow.",
            vec![ArgSpec::new("id", ArgType::Int { min: 0, max: (n - 1).max(0) }, true)
                .hint("The settlement's position in the world's settlement list, counting from 0: a whole number such as 17, never a word. If the user named a place and you do not know its number, say so instead of proposing.")],
            avail,
            why.clone(),
            "pick choose town city village inspect",
        ));
        self.push(mk("select.none", "Select nothing", "Drop the current selection; Previous selection brings it back.", vec![], no_world.is_none(), why_world.clone().unwrap_or_default(), "deselect unselect"));
        let (bavail, bwhy) = match (&why_world, shell.can_select_back) {
            (Some(w), _) => (false, w.clone()),
            (None, Some(true)) => (true, String::new()),
            (None, _) => (false, "There is no earlier selection to return to.".to_owned()),
        };
        self.push(mk("select.back", "Previous selection", "Return to the selection before this one, like a browser's Back.", vec![], bavail, bwhy, "back return earlier"));
    }
}

impl Builder {
    /// AC-5: `edit.undo`, offered only when the shell said whether an undo step exists
    /// ([`ShellFacts::undo`]); a missing fact offers nothing. Effect [`Effect::Undoable`] (it changes the
    /// project, and Redo reverts it). `available` is the shell's answer now, with the step's label in
    /// `state` so the card can say what it would revert.
    fn history(&mut self, shell: &ShellFacts) {
        let Some(u) = &shell.undo else { return };
        self.push(ActionEntry {
            id: "edit.undo".into(),
            title: "Undo the last change".into(),
            desc: "Undo the newest step, exactly as Edit > Undo does; Redo brings it back.".into(),
            desc_source: DescSource::Curated,
            detail: Some("Runs alone on its card. It reverts whatever the newest undo step is, which the state field names when the app said.".into()),
            path: "edit/history".into(),
            kind: ActionKind::Edit,
            effect: Effect::Undoable,
            args: vec![],
            available: u.available,
            why: if u.available { String::new() } else { "There is nothing to undo.".into() },
            state: (u.available && !u.label.is_empty()).then(|| format!("would undo: {}", u.label)),
            // Search words (a labelled judgement): "undo that" must put THIS entry above `menu:undo_history`,
            // whose title also says undo; the repeated words are the weight, as the other alias rows have.
            aliases: "undo undo that revert take back last mistake".into(),
            source: "history",
        });
    }
}

fn capitalise(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// MS-5 (8fcab5fc) added the context row `world.sculpt_here`. Its handler only calls `app.arm_tool("sculpt")` (arms a
    /// tool, draws and deletes nothing), so the destructive guard must NOT drop it and the allow-list must NOT list it:
    /// it passes on its own words. Literal ids, so a loosened guard or a stem added for `sculpt` fails here.
    #[test]
    fn the_sculpt_here_context_row_passes_the_guard_unexempted() {
        assert!(refusal("world.sculpt_here", "Sculpt here").is_none());
        assert!(!REFUSAL_EXEMPT.iter().any(|(id, _, _)| *id == "world.sculpt_here"));
        assert!(refusal("world.sculpt_discard", "Discard sculpt draft").is_some());
    }

    #[test]
    fn refusal_catches_the_destructive_words_and_spares_harmless_ones() {
        for (id, t) in [
            ("menu:save", "Save"),
            ("menu:close_project", "Close project"),
            ("menu:new_world", "New world"),
            ("menu:export_png", "Export"),
            ("menu:import_pack", "Import asset pack"),
            ("menu:delete", "Delete"),
            ("menu:pref_clear_caches", "Clear caches"),
            ("menu:world_run_pipeline", "Run pipeline"),
            ("menu:edit_cut", "Cut"),
            ("menu:edit_paste", "Paste"),
            ("x", "Generate world"),
            ("x", "Discard draft"),
        ] {
            assert!(refusal(id, t).is_some(), "{id} / {t} must be refused");
        }
        for (id, t) in [("view.style_preset", "Style preset"), ("window.vault", "Open Markdown vault"), ("menu:execute_thing", "Execute summary"), ("tool.way", "Way tool")] {
            assert_eq!(refusal(id, t), None, "{id} / {t} must not be refused");
        }
    }

    #[test]
    fn an_arg_is_never_clamped_or_coerced() {
        let a = ArgSpec::new("n", ArgType::Int { min: 1, max: 5 }, true);
        assert!(a.check(&json!(3)).is_ok() && a.check(&json!(3.0)).is_ok());
        for bad in [json!(3.5), json!("3"), json!(6), json!(0), json!(true), json!(null)] {
            assert!(a.check(&bad).is_err(), "{bad} must be refused");
        }
        let b = ArgSpec::new("b", ArgType::Bool, true);
        assert!(b.check(&json!(1)).is_err() && b.check(&json!("true")).is_err() && b.check(&json!(false)).is_ok());
    }
}
