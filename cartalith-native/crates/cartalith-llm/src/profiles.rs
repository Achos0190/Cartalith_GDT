//! Assistant profiles: named bundles of the settings that differ between "a model on my
//! machine" and "a hosted model" (owner request 2026-10-26; Ruling DQ,
//! `LLM_ASSISTANT_SCOPE.md` s.5g).
//!
//! WHY: switching between a local LM Studio model and a hosted Claude model used to mean
//! re-picking the provider, address, model, context ticks and reply cap by hand, and a
//! half-finished switch (new address, old ticks) is exactly how context leaks to the wrong
//! endpoint. A profile is the whole bundle, stored per device, applied at once. Every decision
//! about a profile's *values* (what is valid, how many, what a duplicate is called, what a
//! delete hands back) is made here as pure functions, so the shell stays a thin display and
//! the rules are tested with literals.
//!
//! WHAT A PROFILE HOLDS: provider preset id, base URL, model, thinking mode, the six
//! "Send: ..." context ticks (separate per profile, default off), an optional pinned
//! `max_tokens`, the retry limit (0 or 1, never more: scope s.4 rule 7), the local-provider GPU
//! override (Ruling DF; meaningful only for a local preset) and `key_account`, the NAME of the
//! credential-store entry its key lives under.
//!
//! WHAT A PROFILE NEVER HOLDS: a key (there is no field for one, [`ProfileSet::parse`] drops
//! every field it does not know, and a test serialises a set built from hostile input and greps
//! it), the master switch, the Commands or Edit capability switches, or the local/remote
//! provider switches. Those stay global: a profile can describe an endpoint, it can never open
//! the feature, so applying one cannot turn anything on (Ruling DQ rule 5).
//!
//! MUST NEVER: encode "no value" as a plausible one (a phone's built-in Local profile has an
//! EMPTY base URL because the LAN address is the user's, not a guess; an unreadable `max_tokens`
//! is `None` = "automatic", not 1024), exceed [`MAX_PROFILES`], hand out an id twice, or be
//! reachable from the seed-deterministic path.

use crate::gpu::GpuOverride;
use crate::host::{HostClass, classify_host};
use crate::keystore::account_ok;
use crate::openai::endpoint;
use crate::presets;
use crate::settings::{CUSTOM_ANTHROPIC, CUSTOM_OPENAI};
use serde_json::{Map, Value, json};

/// The schema version this build writes. A stored version above it is read tolerantly (known
/// fields only) and flagged [`Parsed::newer`]; it is rewritten at this build's version on the
/// user's next edit, which drops fields this build does not know (labelled judgement: keeping
/// unknown fields could carry a future secret-like field into the file, which the "never a key
/// field" rule forbids).
pub const VERSION: u32 = 1;
/// At most this many profiles per device (owner answer; the two built-ins count).
pub const MAX_PROFILES: usize = 16;
/// Longest profile name, in characters (a labelled judgement: a dropdown row and a phone
/// header both have to show it).
pub const MAX_NAME_CHARS: usize = 40;
/// Longest stored base URL, in characters (labelled judgement; `models.rs` caps ids at 200).
pub const MAX_URL_CHARS: usize = 300;
/// Longest stored model id, in characters (the same 200 `models.rs` accepts from a server).
pub const MAX_MODEL_CHARS: usize = 200;
/// The pinned reply cap's range, the same `(16, 8192)` the stream bridge clamps to
/// (`llm_stream.rs::MAX_TOKENS_RANGE`); a value outside it is "not pinned", not clamped.
pub const MAX_TOKENS_RANGE: (u32, u32) = (16, 8192);
/// The most automatic retries a profile may allow (scope s.4 rule 7: no retry beyond one).
pub const MAX_RETRY: u8 = 1;
/// The built-in local profile's id.
pub const LOCAL_ID: &str = "local";
/// The built-in external profile's id.
pub const EXTERNAL_ID: &str = "external";

/// How a reasoning model is asked to think.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThinkingMode {
    /// Send nothing extra (the provider's own default).
    Default,
    /// Send the preset's known "no thinking" field on every request
    /// (`presets::request_hints(..).no_thinking`; nothing where no field is known).
    None,
}

impl ThinkingMode {
    /// The stored id.
    pub const fn id(self) -> &'static str {
        match self {
            ThinkingMode::Default => "default",
            ThinkingMode::None => "none",
        }
    }
    /// Parse a stored id; unknown is `None`, never a guess.
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "default" => Some(ThinkingMode::Default),
            "none" => Some(ThinkingMode::None),
            _ => None,
        }
    }
}

/// The six "Send: ..." context sources. Each defaults off; a profile's set is its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Context {
    /// Current entity.
    pub entity: bool,
    /// Selection.
    pub selection: bool,
    /// Named Vault notes.
    pub notes: bool,
    /// Generation parameters and summary statistics.
    pub params: bool,
    /// World data found by search.
    pub rag_world: bool,
    /// Vault notes found by search.
    pub rag_vault: bool,
}

/// One profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    /// Stable id, `[a-z0-9_-]` 1 to 32; never reused within a set.
    pub id: String,
    /// The name the user sees (1 to [`MAX_NAME_CHARS`] characters).
    pub name: String,
    /// A shipped profile: cannot be deleted or renamed, can be edited and reset.
    pub builtin: bool,
    /// A `presets::PRESETS` id, a custom id, or empty for "none chosen".
    pub preset_id: String,
    /// The base URL (may be empty: a phone's Local profile).
    pub base_url: String,
    /// Free-text model id (may be empty).
    pub model: String,
    /// Thinking mode.
    pub thinking: ThinkingMode,
    /// The context ticks.
    pub context: Context,
    /// A pinned reply cap, or `None` for "the task's own default".
    pub max_tokens: Option<u32>,
    /// Automatic retries allowed, 0 or 1.
    pub retry_limit: u8,
    /// The Ruling DF override; applied only while the preset is local.
    pub gpu_override: GpuOverride,
    /// The credential-store account of this profile's key (`keystore::account_ok`).
    pub key_account: String,
}

/// A problem with a profile's values that stops anything being SENT with it (the profile is still
/// stored and can still be switched to, so it can be repaired: the gate refuses it, Ruling DQ).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Problem {
    /// The preset id is not in the table.
    UnknownPreset,
    /// The base URL is unusable; the reason is static text.
    BadUrl(&'static str),
    /// A remote or unknown host over `http://` (Ruling DQ: a remote profile requires https).
    CleartextRemote,
}

impl Problem {
    /// The reason in words.
    pub fn reason(self) -> String {
        match self {
            Problem::UnknownPreset => "its provider is not one this build knows".into(),
            Problem::BadUrl(why) => format!("its address cannot be used: {why}"),
            Problem::CleartextRemote => "a remote address must use https:// so your key and text are not sent in the clear".into(),
        }
    }
    /// A stable code for tests and the bridge.
    pub const fn code(self) -> &'static str {
        match self {
            Problem::UnknownPreset => "unknown_preset",
            Problem::BadUrl(_) => "bad_url",
            Problem::CleartextRemote => "cleartext_remote",
        }
    }
}

/// Why an edit of the set was refused. Static text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpError {
    /// The name is empty after trimming.
    NameEmpty,
    /// The name is longer than [`MAX_NAME_CHARS`].
    NameTooLong,
    /// The name holds a control, bidi or zero-width character.
    NameChars,
    /// Another profile already has this name (compared ignoring case).
    NameTaken,
    /// The set already holds [`MAX_PROFILES`].
    TooMany,
    /// No profile has this id.
    Unknown,
    /// A built-in cannot be renamed or deleted.
    BuiltinFixed,
}

impl OpError {
    /// The reason in words.
    pub const fn reason(self) -> &'static str {
        match self {
            OpError::NameEmpty => "A profile needs a name.",
            OpError::NameTooLong => "That name is too long (40 characters at most).",
            OpError::NameChars => "That name has a character that cannot be shown safely.",
            OpError::NameTaken => "Another profile already has that name.",
            OpError::TooMany => "There are already 16 profiles. Delete one first.",
            OpError::Unknown => "That profile no longer exists.",
            OpError::BuiltinFixed => "The built-in profiles cannot be renamed or deleted. You can edit and reset them.",
        }
    }
}

/// A set of profiles with the last-used and default ids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileSet {
    /// Stored schema version (>= [`VERSION`] when read from a newer build).
    pub version: u32,
    /// The profiles, in display order; the two built-ins are always present.
    pub profiles: Vec<Profile>,
    /// The profile last switched to, restored at startup. Empty = none yet.
    pub last: String,
    /// The one default profile of this device: used when `last` is empty or gone.
    pub default: String,
}

/// What [`ProfileSet::parse`] found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parsed {
    /// The usable set (never empty: the built-ins are always present).
    pub set: ProfileSet,
    /// The text came from a newer build.
    pub newer: bool,
    /// Stored profiles that were dropped, each with a fixed-vocabulary reason (no stored text).
    pub dropped: Vec<&'static str>,
}

/// Whether `c` may not appear in a name or id shown to the user: a control, bidi or zero-width
/// character (the same classes `models.rs` drops from a model id).
fn bad_char(c: char) -> bool {
    c.is_control() || matches!(c, '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{FEFF}')
}

/// Validate and normalise a display name: trimmed, 1 to [`MAX_NAME_CHARS`] characters, no
/// unsafe characters.
pub fn clean_name(raw: &str) -> Result<String, OpError> {
    let t = raw.trim();
    if t.is_empty() {
        return Err(OpError::NameEmpty);
    }
    if t.chars().any(bad_char) {
        return Err(OpError::NameChars);
    }
    if t.chars().count() > MAX_NAME_CHARS {
        return Err(OpError::NameTooLong);
    }
    Ok(t.to_owned())
}

/// Whether `id` is a legal profile id: `[a-z0-9_-]`, 1 to 32.
pub fn id_ok(id: &str) -> bool {
    !id.is_empty() && id.len() <= 32 && id.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_' || c == b'-')
}

fn truncate_chars(s: &str, max: usize) -> String {
    s.trim().chars().filter(|c| !c.is_control()).take(max).collect()
}

/// Whether `preset_id` is a provider this build knows (or empty = none chosen).
fn preset_known(preset_id: &str) -> bool {
    preset_id.is_empty() || preset_id == CUSTOM_OPENAI || preset_id == CUSTOM_ANTHROPIC || presets::find(preset_id).is_some()
}

impl Profile {
    /// Problems that make applying this profile unsafe or pointless. Empty = applicable. An
    /// EMPTY address is not a problem (a phone's built-in Local profile starts without one;
    /// the gate says "no base URL is set" until the user enters it).
    pub fn problems(&self) -> Vec<Problem> {
        let mut out = Vec::new();
        if !preset_known(&self.preset_id) {
            out.push(Problem::UnknownPreset);
        }
        let url = self.base_url.trim();
        if !url.is_empty() {
            match (endpoint(url), classify_host(url)) {
                (Err(crate::error::LlmError::InvalidBaseUrl(why)), _) => out.push(Problem::BadUrl(why)),
                (_, Err(e)) => out.push(Problem::BadUrl(e.reason())),
                (_, Ok(info)) => {
                    let local = matches!(info.class, HostClass::SameMachine | HostClass::LocalNetwork);
                    if !local && url.to_ascii_lowercase().starts_with("http://") {
                        out.push(Problem::CleartextRemote);
                    }
                }
            }
        }
        out
    }

    /// Whether this profile's preset runs on the user's machine or network (Ruling DF applies
    /// only then). A custom or unknown preset is judged by its address.
    pub fn is_local(&self) -> bool {
        match presets::find(&self.preset_id) {
            Some(p) => p.is_local,
            None => classify_host(&self.base_url)
                .map(|i| matches!(i.class, HostClass::SameMachine | HostClass::LocalNetwork))
                .unwrap_or(false),
        }
    }

    /// The values applying this profile writes into the live `[llm]` settings: exactly these
    /// keys, none of them a switch and none a secret. `max_tokens` 0 means "not pinned".
    /// `gpu_override` is the profile's for a local profile and `automatic` otherwise (Ruling DF
    /// governs local providers only, so a remote profile never carries a local preference).
    pub fn live(&self) -> Map<String, Value> {
        let gpu = if self.is_local() { self.gpu_override } else { GpuOverride::Automatic };
        let mut m = Map::new();
        m.insert("preset_id".into(), json!(self.preset_id));
        m.insert("base_url".into(), json!(self.base_url));
        m.insert("model".into(), json!(self.model));
        m.insert("thinking".into(), json!(self.thinking.id()));
        m.insert("max_tokens".into(), json!(self.max_tokens.unwrap_or(0)));
        m.insert("retry_limit".into(), json!(self.retry_limit));
        m.insert("gpu_override".into(), json!(gpu.id()));
        m.insert("key_account".into(), json!(self.key_account));
        m.insert("ctx_entity".into(), json!(self.context.entity));
        m.insert("ctx_selection".into(), json!(self.context.selection));
        m.insert("ctx_notes".into(), json!(self.context.notes));
        m.insert("ctx_params".into(), json!(self.context.params));
        m.insert("ctx_rag_world".into(), json!(self.context.rag_world));
        m.insert("ctx_rag_vault".into(), json!(self.context.rag_vault));
        m
    }

    /// This profile with its values replaced by `live` (the inverse of [`Profile::live`]), so a
    /// change made in the settings window or the panel is written through to the active profile.
    /// Identity (`id`, `name`, `builtin`, `key_account`) is never changed here. A field that is
    /// absent or of the wrong type keeps the profile's value; text is trimmed and capped; a
    /// `max_tokens` outside the range is "not pinned"; a `retry_limit` above [`MAX_RETRY`] is
    /// the maximum. A remote profile keeps its stored GPU preference (the live value is
    /// `automatic` there by construction and must not erase it).
    pub fn with_live(&self, live: &Value) -> Profile {
        let mut p = self.clone();
        let s = |k: &str| live.get(k).and_then(Value::as_str);
        let b = |k: &str, old: bool| live.get(k).and_then(Value::as_bool).unwrap_or(old);
        if let Some(v) = s("preset_id") {
            p.preset_id = truncate_chars(v, 64);
        }
        if let Some(v) = s("base_url") {
            p.base_url = truncate_chars(v, MAX_URL_CHARS);
        }
        if let Some(v) = s("model") {
            p.model = truncate_chars(v, MAX_MODEL_CHARS);
        }
        if let Some(t) = s("thinking").and_then(ThinkingMode::from_id) {
            p.thinking = t;
        }
        if let Some(n) = live.get("max_tokens").and_then(Value::as_i64) {
            p.max_tokens = u32::try_from(n).ok().filter(|n| (MAX_TOKENS_RANGE.0..=MAX_TOKENS_RANGE.1).contains(n));
        }
        if let Some(n) = live.get("retry_limit").and_then(Value::as_i64) {
            p.retry_limit = n.clamp(0, i64::from(MAX_RETRY)) as u8;
        }
        if p.is_local()
            && let Some(g) = s("gpu_override").and_then(GpuOverride::from_id)
        {
            p.gpu_override = g;
        }
        p.context = Context {
            entity: b("ctx_entity", p.context.entity),
            selection: b("ctx_selection", p.context.selection),
            notes: b("ctx_notes", p.context.notes),
            params: b("ctx_params", p.context.params),
            rag_world: b("ctx_rag_world", p.context.rag_world),
            rag_vault: b("ctx_rag_vault", p.context.rag_vault),
        };
        p
    }

    /// The stored form. There is deliberately no key field.
    fn to_json(&self) -> Value {
        let c = &self.context;
        json!({
            "id": self.id, "name": self.name, "builtin": self.builtin,
            "preset_id": self.preset_id, "base_url": self.base_url, "model": self.model,
            "thinking": self.thinking.id(),
            "ctx": { "entity": c.entity, "selection": c.selection, "notes": c.notes,
                     "params": c.params, "rag_world": c.rag_world, "rag_vault": c.rag_vault },
            "max_tokens": self.max_tokens.unwrap_or(0),
            "retry_limit": self.retry_limit,
            "gpu_override": self.gpu_override.id(),
            "key_account": self.key_account,
        })
    }

    /// Read one stored profile tolerantly. `Err` carries the fixed reason it was dropped.
    fn from_json(v: &Value) -> Result<Profile, &'static str> {
        let obj = v.as_object().ok_or("not an object")?;
        let id = obj.get("id").and_then(Value::as_str).ok_or("no id")?;
        if !id_ok(id) {
            return Err("illegal id");
        }
        let name_raw = obj.get("name").and_then(Value::as_str).ok_or("no name")?;
        // An over-long name is shortened (the data is the user's); an empty or unsafe one drops
        // the profile, because inventing a name would be encoding "no value" as a value.
        let t = name_raw.trim();
        if t.is_empty() || t.chars().any(bad_char) {
            return Err("unusable name");
        }
        let name: String = t.chars().take(MAX_NAME_CHARS).collect();
        let st = |k: &str| obj.get(k).and_then(Value::as_str).unwrap_or("");
        let ctx = obj.get("ctx").and_then(Value::as_object);
        let cb = |k: &str| ctx.and_then(|c| c.get(k)).and_then(Value::as_bool).unwrap_or(false);
        let key_account = match obj.get("key_account").and_then(Value::as_str) {
            Some(a) if account_ok(a) => a.to_owned(),
            _ => default_account(id),
        };
        Ok(Profile {
            id: id.to_owned(),
            name,
            builtin: id == LOCAL_ID || id == EXTERNAL_ID,
            preset_id: truncate_chars(st("preset_id"), 64),
            base_url: truncate_chars(st("base_url"), MAX_URL_CHARS),
            model: truncate_chars(st("model"), MAX_MODEL_CHARS),
            thinking: ThinkingMode::from_id(st("thinking")).unwrap_or(ThinkingMode::Default),
            context: Context {
                entity: cb("entity"),
                selection: cb("selection"),
                notes: cb("notes"),
                params: cb("params"),
                rag_world: cb("rag_world"),
                rag_vault: cb("rag_vault"),
            },
            max_tokens: obj
                .get("max_tokens")
                .and_then(Value::as_i64)
                .and_then(|n| u32::try_from(n).ok())
                .filter(|n| (MAX_TOKENS_RANGE.0..=MAX_TOKENS_RANGE.1).contains(n)),
            retry_limit: obj.get("retry_limit").and_then(Value::as_i64).map_or(1, |n| n.clamp(0, i64::from(MAX_RETRY)) as u8),
            gpu_override: GpuOverride::from_id(st("gpu_override")).unwrap_or(GpuOverride::Automatic),
            key_account,
        })
    }
}

/// The credential account a custom profile's key lives under: `pf_<id>`, so two profiles never
/// share an entry (owner answer: one entry per profile account). `id` is at most 32 characters,
/// so the account is at most 35, inside `account_ok`'s 64.
pub fn default_account(id: &str) -> String {
    format!("pf_{id}")
}

/// The two shipped profiles (owner answer 2026-10-26). `phone` leaves Local's address empty:
/// on a phone the model server is on the user's computer and its LAN address is theirs to enter.
/// Both ship with every context tick off. Their key accounts are `lm_studio` and `anthropic`,
/// the preset-id accounts keys were stored under before profiles existed, so an existing saved
/// key still resolves.
pub fn builtin_profiles(phone: bool) -> Vec<Profile> {
    vec![
        Profile {
            id: LOCAL_ID.into(),
            name: "Local".into(),
            builtin: true,
            preset_id: "lm_studio".into(),
            base_url: if phone { String::new() } else { "http://127.0.0.1:1234/v1".into() },
            model: "qwen3.8-27b".into(),
            thinking: ThinkingMode::None,
            context: Context::default(),
            max_tokens: None,
            retry_limit: 1,
            gpu_override: GpuOverride::Automatic,
            key_account: "lm_studio".into(),
        },
        Profile {
            id: EXTERNAL_ID.into(),
            name: "External".into(),
            builtin: true,
            preset_id: "anthropic".into(),
            base_url: "https://api.anthropic.com/v1".into(),
            model: "claude-sonnet-5-5".into(),
            thinking: ThinkingMode::Default,
            context: Context::default(),
            max_tokens: None,
            retry_limit: 1,
            gpu_override: GpuOverride::Automatic,
            key_account: "anthropic".into(),
        },
    ]
}

impl ProfileSet {
    /// A fresh set: the built-ins, nothing used yet, `local` the default.
    pub fn fresh(phone: bool) -> ProfileSet {
        ProfileSet { version: VERSION, profiles: builtin_profiles(phone), last: String::new(), default: LOCAL_ID.into() }
    }

    /// The profile with `id`.
    pub fn get(&self, id: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.id == id)
    }

    fn index(&self, id: &str) -> Result<usize, OpError> {
        self.profiles.iter().position(|p| p.id == id).ok_or(OpError::Unknown)
    }

    /// The profile to restore at startup: `last` if it still exists, else `default`, else the
    /// first. Never empty (the built-ins are always there).
    pub fn startup_id(&self) -> &str {
        [self.last.as_str(), self.default.as_str()]
            .into_iter()
            .find(|id| self.get(id).is_some())
            .unwrap_or(&self.profiles[0].id)
    }

    /// Read stored text. Empty, unreadable or hostile text yields [`ProfileSet::fresh`]; a
    /// readable one keeps what is valid, drops what is not (reasons in [`Parsed::dropped`]),
    /// ignores every field it does not know (so a stray `api_key` is never kept), caps the
    /// count at [`MAX_PROFILES`], and restores a missing built-in (the shipped one) so the set
    /// is never without Local and External.
    pub fn parse(text: &str, phone: bool) -> Parsed {
        let mut out = Parsed { set: ProfileSet::fresh(phone), newer: false, dropped: Vec::new() };
        let Ok(v) = serde_json::from_str::<Value>(text) else { return out };
        let Some(obj) = v.as_object() else { return out };
        let version = obj.get("version").and_then(Value::as_u64).unwrap_or(0);
        out.newer = version > u64::from(VERSION);
        out.set.version = u32::try_from(version).unwrap_or(VERSION).max(VERSION);
        let mut profiles: Vec<Profile> = Vec::new();
        for pv in obj.get("profiles").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]) {
            match Profile::from_json(pv) {
                Ok(p) if profiles.iter().any(|q| q.id == p.id) => out.dropped.push("duplicate id"),
                Ok(p) => profiles.push(p),
                Err(why) => out.dropped.push(why),
            }
        }
        // The shipped profiles are always present: a stored one wins (the user edited it), a
        // missing one comes back as shipped.
        for b in builtin_profiles(phone).into_iter().rev() {
            if !profiles.iter().any(|p| p.id == b.id) {
                profiles.insert(0, b);
            }
        }
        if profiles.len() > MAX_PROFILES {
            profiles.truncate(MAX_PROFILES);
            out.dropped.push("over the 16 profile cap");
        }
        out.set.profiles = profiles;
        let pick = |k: &str| obj.get(k).and_then(Value::as_str).unwrap_or("").to_owned();
        let last = pick("last");
        out.set.last = if out.set.get(&last).is_some() { last } else { String::new() };
        let default = pick("default");
        out.set.default = if out.set.get(&default).is_some() { default } else { LOCAL_ID.into() };
        out
    }

    /// The stored text. Contains no key and no field outside the schema.
    pub fn to_json(&self) -> String {
        json!({
            "version": self.version,
            "last": self.last,
            "default": self.default,
            "profiles": self.profiles.iter().map(Profile::to_json).collect::<Vec<_>>(),
        })
        .to_string()
    }

    fn name_free(&self, name: &str, except: Option<&str>) -> bool {
        let lower = name.to_lowercase();
        !self.profiles.iter().any(|p| Some(p.id.as_str()) != except && p.name.to_lowercase() == lower)
    }

    fn new_id(&self) -> String {
        (1u32..).map(|n| format!("p{n}")).find(|id| self.get(id).is_none()).expect("a free id exists")
    }

    /// A free name `stem` + `tail`, then `stem` + `tail` + " 2", " 3"...: the stem is shortened
    /// so the whole fits the cap, and the tail and the number are never cut.
    fn free_name(&self, stem: &str, tail: &str) -> String {
        for n in 1u32.. {
            let num = if n == 1 { String::new() } else { format!(" {n}") };
            let room = MAX_NAME_CHARS.saturating_sub(tail.chars().count() + num.chars().count());
            let cand = stem.chars().take(room).collect::<String>().trim_end().to_owned() + tail + &num;
            if self.name_free(&cand, None) {
                return cand;
            }
        }
        unreachable!("an unbounded range has a free name")
    }

    /// Add a custom profile named `name`, started from `from` (an existing profile id: its
    /// values are copied, its KEY is not, the new profile gets its own account) or, with `None`,
    /// from the shipped External values with an empty model and every tick off. Returns the new
    /// id.
    pub fn add(&mut self, name: &str, from: Option<&str>) -> Result<String, OpError> {
        let name = clean_name(name)?;
        if self.profiles.len() >= MAX_PROFILES {
            return Err(OpError::TooMany);
        }
        if !self.name_free(&name, None) {
            return Err(OpError::NameTaken);
        }
        let id = self.new_id();
        let mut p = match from {
            Some(src) => self.get(src).cloned().ok_or(OpError::Unknown)?,
            None => Profile {
                id: String::new(),
                name: String::new(),
                builtin: false,
                preset_id: String::new(),
                base_url: String::new(),
                model: String::new(),
                thinking: ThinkingMode::Default,
                context: Context::default(),
                max_tokens: None,
                retry_limit: 1,
                gpu_override: GpuOverride::Automatic,
                key_account: String::new(),
            },
        };
        p.key_account = default_account(&id);
        p.id = id.clone();
        p.name = name;
        p.builtin = false;
        self.profiles.push(p);
        Ok(id)
    }

    /// Duplicate `id` under a free name ("<name> copy", "<name> copy 2", ...). The copy has its
    /// own key account and NO key (a key is never copied: it lives only in the credential
    /// store, under the original's account). Returns the new id.
    pub fn duplicate(&mut self, id: &str) -> Result<String, OpError> {
        let src = self.get(id).ok_or(OpError::Unknown)?.clone();
        if self.profiles.len() >= MAX_PROFILES {
            return Err(OpError::TooMany);
        }
        let name = self.free_name(&src.name, " copy");
        self.add(&name, Some(id))
    }

    /// Rename a custom profile.
    pub fn rename(&mut self, id: &str, name: &str) -> Result<(), OpError> {
        let i = self.index(id)?;
        if self.profiles[i].builtin {
            return Err(OpError::BuiltinFixed);
        }
        let name = clean_name(name)?;
        if !self.name_free(&name, Some(id)) {
            return Err(OpError::NameTaken);
        }
        self.profiles[i].name = name;
        Ok(())
    }

    /// Delete a custom profile. Returns the key account it used, so the caller can ASK before
    /// forgetting the stored key (a delete never removes a key by itself). `last` and `default`
    /// pointing at it fall back (`last` to none, `default` to Local).
    pub fn delete(&mut self, id: &str) -> Result<String, OpError> {
        let i = self.index(id)?;
        if self.profiles[i].builtin {
            return Err(OpError::BuiltinFixed);
        }
        let gone = self.profiles.remove(i);
        if self.last == id {
            self.last.clear();
        }
        if self.default == id {
            self.default = LOCAL_ID.into();
        }
        Ok(gone.key_account)
    }

    /// Write live values through to `id` (see [`Profile::with_live`]).
    pub fn sync_live(&mut self, id: &str, live: &Value) -> Result<(), OpError> {
        let i = self.index(id)?;
        self.profiles[i] = self.profiles[i].with_live(live);
        Ok(())
    }

    /// Mark `id` as the last one switched to. An EMPTY id clears the mark ("no profile is
    /// active": the settings are the user's own, not a profile's), which a declined switch uses
    /// to put things back exactly.
    pub fn set_last(&mut self, id: &str) -> Result<(), OpError> {
        if !id.is_empty() {
            self.index(id)?;
        }
        self.last = id.to_owned();
        Ok(())
    }

    /// Make `id` the device's default profile.
    pub fn set_default(&mut self, id: &str) -> Result<(), OpError> {
        self.index(id)?;
        self.default = id.to_owned();
        Ok(())
    }

    /// Put a built-in back to its shipped values (its key account and name are unchanged by
    /// construction: they are the shipped ones).
    pub fn reset_builtin(&mut self, id: &str, phone: bool) -> Result<(), OpError> {
        let i = self.index(id)?;
        let shipped = builtin_profiles(phone).into_iter().find(|b| b.id == id).ok_or(OpError::BuiltinFixed)?;
        self.profiles[i] = shipped;
        Ok(())
    }

    /// Adopt values that existed before profiles did (a user's pre-profile `[llm]` settings)
    /// as a custom profile, so the first switch does not lose them. `key_account` is the
    /// account their key was saved under. Returns the new id, or `Unknown`-free errors from
    /// [`ProfileSet::add`].
    pub fn adopt_legacy(&mut self, live: &Value, key_account: &str) -> Result<String, OpError> {
        let name = self.free_name("Previous settings", "");
        let id = self.add(&name, None)?;
        let i = self.index(&id)?;
        let account = if account_ok(key_account) { key_account.to_owned() } else { default_account(&id) };
        self.profiles[i] = self.profiles[i].with_live(live);
        self.profiles[i].key_account = account;
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set() -> ProfileSet {
        ProfileSet::fresh(false)
    }

    /// Protects: the two shipped profiles equal the owner's answer, as literals (not the
    /// constants they are built from). MUTATION: change the Local URL, either model, Local's
    /// thinking mode, or an account.
    #[test]
    fn built_ins_are_the_owners_two() {
        let s = set();
        let l = s.get("local").unwrap();
        assert_eq!((l.name.as_str(), l.preset_id.as_str(), l.base_url.as_str()), ("Local", "lm_studio", "http://127.0.0.1:1234/v1"));
        assert_eq!((l.model.as_str(), l.thinking, l.key_account.as_str()), ("qwen3.8-27b", ThinkingMode::None, "lm_studio"));
        let e = s.get("external").unwrap();
        assert_eq!((e.name.as_str(), e.preset_id.as_str(), e.base_url.as_str()), ("External", "anthropic", "https://api.anthropic.com/v1"));
        assert_eq!((e.model.as_str(), e.thinking, e.key_account.as_str()), ("claude-sonnet-5-5", ThinkingMode::Default, "anthropic"));
        assert!(l.builtin && e.builtin);
        assert_eq!(s.default, "local");
        assert_eq!(s.last, "");
    }

    /// Protects: a phone's Local has NO address (never a plausible one) and the other built-in is
    /// unchanged. MUTATION: give the phone Local a localhost URL.
    #[test]
    fn phone_local_has_no_address() {
        let p = ProfileSet::fresh(true);
        assert_eq!(p.get("local").unwrap().base_url, "");
        assert_eq!(p.get("external").unwrap().base_url, "https://api.anthropic.com/v1");
        assert!(p.get("local").unwrap().problems().is_empty(), "empty is not a problem, the gate says it is unset");
    }

    /// Protects: every built-in and every new profile has every context tick off (owner answer).
    /// MUTATION: default one tick to true.
    #[test]
    fn context_ticks_default_off() {
        let mut s = set();
        for p in &s.profiles {
            assert_eq!(p.context, Context::default());
        }
        let id = s.add("Mine", None).unwrap();
        let c = s.get(&id).unwrap().context;
        assert_eq!((c.entity, c.selection, c.notes, c.params, c.rag_world, c.rag_vault), (false, false, false, false, false, false));
    }

    /// Protects: the limits, by literal. MUTATION: change 16 or 40.
    #[test]
    fn caps_are_sixteen_profiles_and_forty_characters() {
        assert_eq!((MAX_PROFILES, MAX_NAME_CHARS, MAX_RETRY), (16, 40, 1));
        let mut s = set();
        for n in 0..14 {
            s.add(&format!("P{n}"), None).unwrap();
        }
        assert_eq!(s.profiles.len(), 16);
        assert_eq!(s.add("One more", None), Err(OpError::TooMany));
        assert_eq!(s.duplicate("local"), Err(OpError::TooMany));
        let mut t = set();
        assert!(t.add(&"x".repeat(40), None).is_ok());
        assert_eq!(t.add(&"y".repeat(41), None), Err(OpError::NameTooLong));
        assert_eq!(t.add("   ", None), Err(OpError::NameEmpty));
        assert_eq!(t.add("bad\u{202E}name", None), Err(OpError::NameChars));
        assert_eq!(t.add("tab\there", None), Err(OpError::NameChars));
        // 40 multi-byte characters are 40 characters, not 40 bytes.
        assert!(t.add(&"é".repeat(40), None).is_ok());
    }

    /// Protects: names are unique ignoring case and surrounding space.
    #[test]
    fn names_are_unique_ignoring_case() {
        let mut s = set();
        assert_eq!(s.add("local", None), Err(OpError::NameTaken));
        assert_eq!(s.add("  EXTERNAL ", None), Err(OpError::NameTaken));
        let id = s.add("Work", None).unwrap();
        assert_eq!(s.rename(&id, "work"), Ok(()), "renaming to its own name in other case is allowed");
        assert_eq!(s.rename(&id, "Local"), Err(OpError::NameTaken));
    }

    /// Protects: built-ins cannot be renamed or deleted; customs can; ids are never reused while
    /// live. MUTATION: allow deleting `local`.
    #[test]
    fn built_ins_are_fixed_customs_are_not() {
        let mut s = set();
        assert_eq!(s.rename("local", "Mine"), Err(OpError::BuiltinFixed));
        assert_eq!(s.delete("external"), Err(OpError::BuiltinFixed));
        assert_eq!(s.delete("zzz"), Err(OpError::Unknown));
        let a = s.add("A", None).unwrap();
        let b = s.add("B", None).unwrap();
        assert_ne!(a, b);
        assert_eq!(a, "p1");
        s.delete(&a).unwrap();
        assert!(s.get(&a).is_none());
        assert_eq!(s.add("C", None).unwrap(), "p1", "a freed id may come back, but never while live");
    }

    /// Protects: a duplicate copies values but not the key account (one account per profile),
    /// takes a free name, and starts as a custom profile. MUTATION: copy the account.
    #[test]
    fn duplicate_gets_its_own_account_and_a_free_name() {
        let mut s = set();
        let a = s.duplicate("local").unwrap();
        let b = s.duplicate("local").unwrap();
        assert_eq!(s.get(&a).unwrap().name, "Local copy");
        assert_eq!(s.get(&b).unwrap().name, "Local copy 2");
        let (pa, pb) = (s.get(&a).unwrap(), s.get(&b).unwrap());
        assert_eq!((pa.key_account.as_str(), pb.key_account.as_str()), ("pf_p1", "pf_p2"));
        assert_ne!(pa.key_account, s.get("local").unwrap().key_account);
        assert!(!pa.builtin);
        assert_eq!((pa.preset_id.as_str(), pa.model.as_str()), ("lm_studio", "qwen3.8-27b"));
    }

    /// Protects: a long name's duplicate still fits the cap, and its suffix survives.
    #[test]
    fn duplicate_of_a_long_name_fits() {
        let mut s = set();
        let long = "n".repeat(40);
        let a = s.add(&long, None).unwrap();
        let d = s.duplicate(&a).unwrap();
        let n = &s.get(&d).unwrap().name;
        assert!(n.chars().count() <= 40 && n.ends_with(" copy"), "{n}");
    }

    /// Protects: a delete hands back the key account (so the shell can ASK about forgetting it)
    /// and repairs `last` and `default`. MUTATION: return an empty account, or leave `default`.
    #[test]
    fn delete_returns_the_account_and_repairs_pointers() {
        let mut s = set();
        let a = s.add("A", None).unwrap();
        s.set_last(&a).unwrap();
        s.set_default(&a).unwrap();
        assert_eq!(s.delete(&a), Ok("pf_p1".to_owned()));
        assert_eq!((s.last.as_str(), s.default.as_str()), ("", "local"));
        assert_eq!(s.startup_id(), "local");
    }

    /// Protects: startup restores the last used profile, falls back to the default, then to the
    /// first. MUTATION: swap the order.
    #[test]
    fn startup_restores_last_then_default() {
        let mut s = set();
        assert_eq!(s.startup_id(), "local");
        s.set_default("external").unwrap();
        assert_eq!(s.startup_id(), "external");
        s.set_last("local").unwrap();
        assert_eq!(s.startup_id(), "local");
        assert_eq!(s.set_last("nope"), Err(OpError::Unknown));
        assert_eq!(s.set_last(""), Ok(()), "an empty id clears the mark");
        assert_eq!(s.last, "");
        assert_eq!(s.startup_id(), "external", "then the default decides");
    }

    /// Protects: serialisation round-trips every field exactly. MUTATION: drop a field from
    /// `to_json` or `from_json`.
    #[test]
    fn json_round_trips() {
        let mut s = set();
        let id = s.add("Cloud 2", None).unwrap();
        let live = json!({"preset_id": "openai", "base_url": "https://api.openai.com/v1", "model": "gpt-x",
            "thinking": "none", "max_tokens": 777, "retry_limit": 0, "gpu_override": "never",
            "ctx_entity": true, "ctx_selection": false, "ctx_notes": true, "ctx_params": true, "ctx_rag_world": false, "ctx_rag_vault": true});
        s.sync_live(&id, &live).unwrap();
        s.set_last(&id).unwrap();
        s.set_default("external").unwrap();
        let back = ProfileSet::parse(&s.to_json(), false);
        assert_eq!(back.set, s);
        assert!(!back.newer && back.dropped.is_empty());
        let p = back.set.get(&id).unwrap();
        assert_eq!((p.max_tokens, p.retry_limit, p.thinking), (Some(777), 0, ThinkingMode::None));
        assert_eq!((p.context.entity, p.context.selection, p.context.notes, p.context.params, p.context.rag_world, p.context.rag_vault),
            (true, false, true, true, false, true));
        // `gpu_override` is kept only for a local preset: openai is remote, so it stayed automatic.
        assert_eq!(p.gpu_override, GpuOverride::Automatic);
    }

    /// Protects: a profile NEVER holds a key. A hostile document with key-like fields, at the top
    /// and in a profile, parses; the re-serialised text holds none of those names or values.
    /// MUTATION: make `from_json` copy unknown fields through.
    #[test]
    fn no_key_field_survives_a_round_trip() {
        let hostile = r#"{"version":1,"api_key":"sk-TOPSECRET","last":"","default":"local","profiles":[
            {"id":"p1","name":"Evil","api_key":"sk-SECRET-1","key":"sk-SECRET-2","secret":"sk-SECRET-3",
             "authorization":"Bearer sk-SECRET-4","token":"sk-SECRET-5","password":"sk-SECRET-6",
             "preset_id":"openai","base_url":"https://api.openai.com/v1","model":"m","key_account":"pf_p1"}]}"#;
        let p = ProfileSet::parse(hostile, false);
        assert!(p.set.get("p1").is_some());
        let out = p.set.to_json();
        for needle in ["sk-", "SECRET", "api_key", "password", "authorization", "Bearer", "\"token\"", "\"secret\"", "\"key\""] {
            assert!(!out.contains(needle), "{needle} leaked into {out}");
        }
        // The only key-ish thing stored is the account NAME.
        assert!(out.contains("\"key_account\":\"pf_p1\""));
        // The shipped set and a with_live round trip hold none either.
        let mut s = set();
        s.sync_live("local", &json!({"api_key": "sk-X", "model": "m"})).unwrap();
        assert!(!s.to_json().contains("sk-X"));
    }

    /// Protects: unknown fields are ignored and a NEWER version is read tolerantly and flagged.
    /// MUTATION: reject a newer version, or crash on an unknown field.
    #[test]
    fn unknown_fields_and_newer_versions_are_tolerated() {
        let doc = r#"{"version":7,"future":{"a":1},"last":"p9","default":"external","profiles":[
            {"id":"p9","name":"Future","novel":[1,2,3],"preset_id":"lm_studio","base_url":"http://localhost:1234/v1","model":"m",
             "ctx":{"entity":true,"holo":true},"max_tokens":512,"thinking":"hyper","retry_limit":9,"gpu_override":"turbo"}]}"#;
        let p = ProfileSet::parse(doc, false);
        assert!(p.newer);
        assert_eq!(p.set.version, 7);
        let f = p.set.get("p9").unwrap();
        assert!(f.context.entity);
        assert_eq!(f.max_tokens, Some(512));
        assert_eq!(f.thinking, ThinkingMode::Default, "an unknown thinking id is the default, not a guess");
        assert_eq!(f.retry_limit, 1, "above the maximum is the maximum");
        assert_eq!(f.gpu_override, GpuOverride::Automatic);
        assert_eq!((p.set.last.as_str(), p.set.default.as_str()), ("p9", "external"));
        assert_eq!(p.set.profiles.len(), 3, "built-ins restored beside the stored one");
    }

    /// Protects: unreadable input is the shipped set, never a panic or a half set.
    #[test]
    fn garbage_is_the_shipped_set() {
        for text in ["", "not json", "[]", "null", "{}", "{\"profiles\":3}", "\u{0}"] {
            let p = ProfileSet::parse(text, false);
            assert_eq!(p.set.profiles.len(), 2, "{text:?}");
            assert_eq!(p.set.default, "local");
        }
    }

    /// Protects: bad stored profiles are dropped with a reason, an over-cap file is cut to 16, a
    /// duplicate id is dropped, and a missing built-in comes back as shipped. MUTATION: raise the cap.
    #[test]
    fn parse_drops_the_unusable_and_caps_the_count() {
        let mut list = String::new();
        for n in 0..20 {
            list += &format!(r#"{{"id":"x{n}","name":"N{n}"}},"#);
        }
        list += r#"{"id":"BAD ID","name":"a"},{"id":"ok","name":"   "},{"id":"x0","name":"dup"},{"name":"noid"},5"#;
        let p = ProfileSet::parse(&format!(r#"{{"version":1,"profiles":[{list}]}}"#), false);
        assert_eq!(p.set.profiles.len(), 16);
        assert_eq!(p.set.profiles[0].id, "local", "built-ins restored in front");
        assert!(p.dropped.contains(&"illegal id") && p.dropped.contains(&"unusable name") && p.dropped.contains(&"duplicate id"));
        assert!(p.dropped.contains(&"over the 16 profile cap"));
        assert!(p.dropped.contains(&"no id") && p.dropped.contains(&"not an object"));
    }

    /// Protects: a stored `last` or `default` that names nothing falls back, and an over-long
    /// stored name is shortened while an empty one drops the profile.
    #[test]
    fn dangling_pointers_and_long_names() {
        let doc = format!(r#"{{"version":1,"last":"ghost","default":"ghost","profiles":[{{"id":"a","name":"{}"}}]}}"#, "w".repeat(90));
        let p = ProfileSet::parse(&doc, false);
        assert_eq!((p.set.last.as_str(), p.set.default.as_str()), ("", "local"));
        let a = p.set.get("a").unwrap();
        assert_eq!(a.name.chars().count(), 40);
        // A stored profile with no other field reads as the SAFE defaults: one retry, nothing
        // pinned, default thinking, automatic GPU, every tick off, an account derived from its id.
        assert_eq!((a.retry_limit, a.max_tokens, a.thinking, a.gpu_override), (1, None, ThinkingMode::Default, GpuOverride::Automatic));
        assert_eq!((a.context, a.key_account.as_str(), a.builtin), (Context::default(), "pf_a", false));
    }

    /// Protects: a remote profile requires https (Ruling DQ), a local one may use http, and an
    /// unknown provider or unusable address is named. MUTATION: drop the cleartext arm.
    #[test]
    fn remote_requires_https() {
        let mut s = set();
        let id = s.add("R", None).unwrap();
        let mut check = |preset: &str, url: &str| {
            s.sync_live(&id, &json!({"preset_id": preset, "base_url": url})).unwrap();
            s.get(&id).unwrap().problems()
        };
        assert_eq!(check("openai", "http://api.openai.com/v1"), vec![Problem::CleartextRemote]);
        assert_eq!(check("openai", "https://api.openai.com/v1"), vec![]);
        assert_eq!(check("lm_studio", "http://127.0.0.1:1234/v1"), vec![]);
        assert_eq!(check("lm_studio", "http://192.168.1.20:1234/v1"), vec![]);
        assert_eq!(check("lm_studio", "http://example.com:1234/v1"), vec![Problem::CleartextRemote]);
        assert_eq!(check("lm_studio", "http://host.docker.internal:1234/v1"), vec![Problem::CleartextRemote], "unknown hosts are not local");
        assert_eq!(check("nonesuch", ""), vec![Problem::UnknownPreset]);
        assert!(matches!(check("openai", "ftp://x/v1").as_slice(), [Problem::BadUrl(_)]));
        assert!(matches!(check("openai", "https://user:pw@api.openai.com/v1").as_slice(), [Problem::BadUrl(_)]));
        assert_eq!(check("", ""), vec![]);
    }

    /// Protects: `live()` writes exactly the fourteen profile keys, none of them a switch or a
    /// secret, and the Ruling DF override is carried for a local profile only. MUTATION: add
    /// `enabled` to the map, or carry the override for a remote profile.
    #[test]
    fn live_values_are_exactly_the_profile_keys() {
        let mut s = set();
        s.sync_live("local", &json!({"gpu_override": "never"})).unwrap();
        let live = s.get("local").unwrap().live();
        let mut keys: Vec<&str> = live.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(keys, ["base_url", "ctx_entity", "ctx_notes", "ctx_params", "ctx_rag_vault", "ctx_rag_world", "ctx_selection",
            "gpu_override", "key_account", "max_tokens", "model", "preset_id", "retry_limit", "thinking"]);
        for forbidden in ["enabled", "prose", "commands", "edit", "local_allowed", "remote_allowed", "key_source", "env_var", "token_cap"] {
            assert!(!live.contains_key(forbidden), "{forbidden}");
        }
        assert_eq!(live["gpu_override"], "never");
        s.sync_live("external", &json!({"gpu_override": "never"})).unwrap();
        assert_eq!(s.get("external").unwrap().live()["gpu_override"], "automatic");
        // A profile that WAS local keeps its stored preference while it points at a remote
        // provider, but carries `automatic` live; pointing it back restores the preference.
        s.sync_live("local", &json!({"preset_id": "openai", "base_url": "https://api.openai.com/v1"})).unwrap();
        assert_eq!(s.get("local").unwrap().gpu_override, GpuOverride::NeverChange);
        assert_eq!(s.get("local").unwrap().live()["gpu_override"], "automatic");
        s.sync_live("local", &json!({"preset_id": "lm_studio", "base_url": "http://127.0.0.1:1234/v1"})).unwrap();
        assert_eq!(s.get("local").unwrap().live()["gpu_override"], "never");
        assert_eq!(live["max_tokens"], 0);
        assert_eq!(live["thinking"], "none");
    }

    /// Protects: a pinned max_tokens is kept inside the range and is "not pinned" outside it.
    /// MUTATION: widen the range.
    #[test]
    fn max_tokens_is_pinned_only_inside_the_range() {
        assert_eq!(MAX_TOKENS_RANGE, (16, 8192));
        let p = set().get("local").unwrap().clone();
        for (v, want) in [(16, Some(16)), (8192, Some(8192)), (15, None), (8193, None), (0, None), (-5, None), (4096, Some(4096))] {
            assert_eq!(p.with_live(&json!({"max_tokens": v})).max_tokens, want, "{v}");
        }
        assert_eq!(p.with_live(&json!({"max_tokens": "big"})).max_tokens, None, "a wrong type is 'not pinned'");
    }

    /// Protects: the retry limit never exceeds one and may be zero.
    #[test]
    fn retry_limit_is_zero_or_one() {
        let p = set().get("local").unwrap().clone();
        for (v, want) in [(0, 0u8), (1, 1), (2, 1), (99, 1), (-3, 0)] {
            assert_eq!(p.with_live(&json!({"retry_limit": v})).retry_limit, want, "{v}");
        }
    }

    /// Protects: identity is never changed by a write-through (name, id, built-in flag, key
    /// account). MUTATION: let `with_live` read `key_account`.
    #[test]
    fn write_through_never_changes_identity() {
        let p = set().get("external").unwrap().clone();
        let q = p.with_live(&json!({"id": "x", "name": "Hacked", "builtin": false, "key_account": "other", "model": "m2"}));
        assert_eq!((q.id.as_str(), q.name.as_str(), q.builtin, q.key_account.as_str()), ("external", "External", true, "anthropic"));
        assert_eq!(q.model, "m2");
    }

    /// Protects: text fields are trimmed and capped on write-through.
    #[test]
    fn write_through_trims_and_caps_text() {
        let p = set().get("local").unwrap().clone();
        let q = p.with_live(&json!({"model": format!("  {}  ", "m".repeat(500)), "base_url": "  http://a/v1\n "}));
        assert_eq!(q.model.chars().count(), 200);
        assert_eq!(q.base_url, "http://a/v1");
    }

    /// Protects: resetting a built-in restores the shipped values and leaves customs alone.
    #[test]
    fn reset_restores_shipped_values() {
        let mut s = set();
        s.sync_live("local", &json!({"model": "other", "ctx_notes": true, "max_tokens": 300})).unwrap();
        assert_eq!(s.get("local").unwrap().model, "other");
        s.reset_builtin("local", false).unwrap();
        assert_eq!(s, set());
        let c = s.add("C", None).unwrap();
        assert_eq!(s.reset_builtin(&c, false), Err(OpError::BuiltinFixed));
    }

    /// Protects: pre-profile settings are adopted as a custom profile that keeps its old key
    /// account (so an existing saved key still resolves) and a free name. MUTATION: use `pf_` for it.
    #[test]
    fn legacy_settings_are_adopted_with_their_old_account() {
        let mut s = set();
        let live = json!({"preset_id": "openai", "base_url": "https://api.openai.com/v1", "model": "gpt-x", "ctx_notes": true});
        let id = s.adopt_legacy(&live, "openai").unwrap();
        let p = s.get(&id).unwrap();
        assert_eq!((p.name.as_str(), p.key_account.as_str(), p.model.as_str()), ("Previous settings", "openai", "gpt-x"));
        assert!(p.context.notes && !p.builtin);
        let id2 = s.adopt_legacy(&live, "Bad Account!").unwrap();
        assert_eq!(s.get(&id2).unwrap().name, "Previous settings 2");
        assert_eq!(s.get(&id2).unwrap().key_account, format!("pf_{id2}"));
    }

    /// Protects: every account any operation hands out is a legal credential account.
    #[test]
    fn every_account_is_legal() {
        let mut s = set();
        for n in 0..14 {
            let id = s.add(&format!("A{n}"), None).unwrap();
            assert!(account_ok(&s.get(&id).unwrap().key_account), "{id}");
        }
        for p in &s.profiles {
            assert!(account_ok(&p.key_account));
        }
        assert!(account_ok(&default_account(&"z".repeat(32))));
    }

    /// Protects: the thinking ids are stable.
    #[test]
    fn thinking_ids_are_stable() {
        assert_eq!((ThinkingMode::Default.id(), ThinkingMode::None.id()), ("default", "none"));
        assert_eq!(ThinkingMode::from_id("none"), Some(ThinkingMode::None));
        assert_eq!(ThinkingMode::from_id(""), None);
    }
}
