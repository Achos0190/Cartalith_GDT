//! The LLM assistant's settings surface for the Godot shell
//! (`LLM_ASSISTANT_SCOPE.md` milestone LLM-2).
//!
//! `cartalith-llm` owns every decision (address classification, the settings
//! gate, the GPU rule, key resolution); this file owns exactly the things it
//! cannot:
//!
//! 1. **Reading the real environment and the real credential store** and
//!    handing them to the pure functions (`std::env::var` and
//!    [`crate::llm_keystore::OsKeyStore`]).
//! 2. **Not borrowing `WorldGen`** (see [`LlmBridge`]).
//! 3. **Not panicking.** Every method returns a dictionary with an `ok` flag
//!    rather than unwrapping, because a panic crossing the gdext boundary takes
//!    the Godot process down (`cartalith-rust-conventions`).
//!
//! **NO NETWORK.** Nothing in this file opens a socket or resolves a name. LLM-3
//! owns the wire (Godot's `HTTPClient`).
//!
//! **THE KEY NEVER COMES BACK.** [`LlmBridge::llm_key_status`] answers whether a
//! key is present and its masked form (`****` plus the last four characters,
//! or `****` alone for a short key). No method here returns, logs or echoes a
//! key's characters; [`LlmBridge::llm_key_save`] takes the key in and says only
//! `ok` or a static reason.
//!
//! **ABSENT MEANS ABSENT.** An optional field (a port, a masked key, a target
//! device) is omitted from the dictionary when there is none; it is never `-1`,
//! `""` or `0` standing for "none". Callers test `has()`.

use cartalith_llm::gpu::{Adapter, GpuOverride, GpuPlan, plan, plan_phone};
use cartalith_llm::host::classify_host;
use cartalith_llm::keystore::{KeySource, KeyStore, StoreError, resolve_key};
use cartalith_llm::presets::{ApiStyle, KeyRequirement, PRESETS};
use cartalith_llm::settings::{
    CUSTOM_ANTHROPIC, CUSTOM_OPENAI, Capability, LlmSettings, authorize, origin_of, terms_url,
};
use cartalith_llm::{ApiKey, HostClass};
use godot::classes::{IRefCounted, RefCounted};
use godot::prelude::*;

use crate::llm_keystore::OsKeyStore;

/// Why a key cannot be saved where no OS store exists. On Android the
/// environment-variable route is not offered (an app's environment cannot be
/// set on a phone), so the text names the Keystore's requirement instead
/// (LLM-10). Never suggests a file: there is no file route (Ruling DD Q5).
#[cfg(target_os = "android")]
const NO_STORE_SAVE: &str = "The Android Keystore cannot be reached on this device (it needs Android 12 or later), so no key can be saved. Cartalith never saves a key to a file.";
#[cfg(not(target_os = "android"))]
const NO_STORE_SAVE: &str =
    "No OS credential store is available here. Use an environment variable; Cartalith never saves a key to a file.";

/// `{"ok": false, "error": <static reason>}`.
pub(crate) fn fail(message: &str) -> VarDictionary {
    vdict! { "ok" => false, "error" => message }
}

/// A string field of `d`, or `""` when absent or not a string. An absent text
/// field is "not set" and the gate refuses it, so `""` is safe here.
pub(crate) fn text(d: &VarDictionary, key: &str) -> String {
    d.get(key).and_then(|v| v.try_to::<GString>().ok()).map(|g| g.to_string()).unwrap_or_default()
}

/// A bool field of `d`, `false` when absent or not a bool (the off direction).
pub(crate) fn flag(d: &VarDictionary, key: &str) -> bool {
    d.get(key).and_then(|v| v.try_to::<bool>().ok()).unwrap_or(false)
}

/// The gate-relevant settings from the shell's dictionary. Anything missing or
/// of the wrong type reads as **off or empty**: a corrupt or partial settings
/// record can only make the feature more closed, never more open.
pub(crate) fn settings_from(d: &VarDictionary) -> LlmSettings {
    let mut s = LlmSettings::off();
    s.enabled = flag(d, "enabled");
    s.prose = flag(d, "prose");
    s.commands = flag(d, "commands");
    s.edit = flag(d, "edit");
    s.local_allowed = flag(d, "local_allowed");
    s.remote_allowed = flag(d, "remote_allowed");
    s.preset_id = text(d, "preset_id");
    s.base_url = text(d, "base_url");
    s.model = text(d, "model");
    // An unknown or absent source keeps `LlmSettings::off()`'s environment
    // variable default; the key is then looked up where the user never put it
    // and the gate reports it missing.
    if let Some(src) = KeySource::from_id(&text(d, "key_source")) {
        s.key_source = src;
    }
    // The opt-in token cap: a positive integer turns it on; absent, zero, negative or
    // of the wrong type is OFF (Ruling DD Q8: no cap by default, and a corrupt record
    // must not invent one -- nor a `Some(0)` that would refuse everything).
    s.token_cap = d
        .get("token_cap")
        .and_then(|v| v.try_to::<i64>().ok())
        .and_then(|n| u64::try_from(n).ok())
        .filter(|n| *n > 0);
    let env = text(d, "env_var");
    if !env.is_empty() {
        s.env_var = env;
    }
    s
}

/// The credential-store account for a provider: its id. Presets and the two
/// custom ids all satisfy `keystore::account_ok`.
pub(crate) fn account_for(preset_id: &str) -> String {
    preset_id.to_owned()
}

/// The key problem as the user should read it on this platform. On Android
/// (LLM-10) an environment-variable problem is replaced: an app's environment
/// cannot be set on a phone, so "set it and restart" is advice nobody can
/// follow there; the Keystore is the route. Elsewhere the text is unchanged.
fn phone_problem(src: KeySource, problem: String) -> String {
    if cfg!(target_os = "android") && src == KeySource::EnvVar {
        PHONE_ENV_PROBLEM.to_owned()
    } else {
        problem
    }
}

/// What [`phone_problem`] says on Android for the environment source.
const PHONE_ENV_PROBLEM: &str = "Key missing: an app's environment cannot be set on a phone. Choose Android Keystore as the key source and save the key there.";

/// A GPU plan as the dictionary `LlmBridge::llm_gpu_plan` documents: `target`
/// only when there is one, `class`/`class_label` only when the URL parsed.
fn plan_dict(p: GpuPlan, class: Option<HostClass>) -> VarDictionary {
    let mut d = vdict! {
        "ok" => true,
        "warn" => p.warn,
        "headline" => p.headline.as_str(),
        "detected" => p.detected.as_str(),
        "effect" => p.effect.as_str(),
        "switch" => p.switch,
    };
    if let Some(c) = class {
        put_class(&mut d, c);
    }
    if let Some(t) = p.target {
        d.set("target", t.as_str());
    }
    d
}

/// A class as the dictionary fields the shell reads.
fn put_class(d: &mut VarDictionary, c: HostClass) {
    d.set("class", c.id());
    d.set("class_label", c.label());
}

/// The Godot class the shell calls for everything in this file. **Deliberately not
/// methods on `WorldGen`**: a `#[func]` on `WorldGen` fails its own `bind()` while a
/// generation holds the engine on its worker thread (`engine_bridge.gd`'s multi-GPU
/// note: 360 panics measured), and the settings window must stay usable during a
/// generate. This class holds no state and never borrows `WorldGen`, the same shape
/// as `progress_bridge.rs`'s `GenerationProgress`.
#[derive(GodotClass)]
#[class(base=RefCounted)]
struct LlmBridge {
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for LlmBridge {
    /// Constructs the stateless class; `Base` is the only field.
    fn init(base: Base<RefCounted>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl LlmBridge {
    /// The provider picker's rows: every preset from `cartalith-llm`'s table,
    /// then the two custom entries. Each: `id`, `name`, `base_url`, `style`
    /// (`"openai"` / `"anthropic"`), `is_local`, `key_required`, `custom`;
    /// `port` and `terms_url` only where one exists. The custom rows have an
    /// empty `base_url` (the user types it) and no port.
    #[func]
    fn llm_presets(&self) -> Array<VarDictionary> {
        let mut out: Array<VarDictionary> = PRESETS
            .iter()
            .map(|p| {
                let mut d = vdict! {
                    "id" => p.id,
                    "name" => p.display_name,
                    "base_url" => p.base_url,
                    "style" => if p.api_style == ApiStyle::OpenAiCompatible { "openai" } else { "anthropic" },
                    "is_local" => p.is_local,
                    "key_required" => p.key == KeyRequirement::Required,
                    "custom" => false,
                };
                if let Some(port) = p.default_port {
                    d.set("port", port as i64);
                }
                if let Some(u) = terms_url(p.id) {
                    d.set("terms_url", u);
                }
                d
            })
            .collect();
        for (id, name, style) in [
            (CUSTOM_OPENAI, "Custom (OpenAI-compatible)", "openai"),
            (CUSTOM_ANTHROPIC, "Custom (Anthropic Messages)", "anthropic"),
        ] {
            out.push(&vdict! {
                "id" => id, "name" => name, "base_url" => "", "style" => style,
                "is_local" => false, "key_required" => style == "anthropic", "custom" => true,
            });
        }
        out
    }

    /// `classify_host` for the shell: `{ok, class, class_label, host, port?}`
    /// or `{ok: false, error}`. Pure; resolves nothing.
    #[func]
    fn llm_classify_host(&self, input: GString) -> VarDictionary {
        match classify_host(&input.to_string()) {
            Ok(h) => {
                let mut d = vdict! { "ok" => true, "host" => h.host.as_str() };
                put_class(&mut d, h.class);
                if let Some(p) = h.port {
                    d.set("port", p as i64);
                }
                d
            }
            Err(e) => fail(e.reason()),
        }
    }

    /// What the shell needs to know about the chosen endpoint:
    /// `{ok, origin, class, class_label, is_local, key_required}`. `is_local` is
    /// the GPU-rule predicate (a local preset, or a custom endpoint classed
    /// same-machine or LAN); `key_required` follows the gate (a required-key
    /// preset, or any endpoint that is not same-machine or LAN).
    #[func]
    fn llm_endpoint_info(&self, preset_id: GString, base_url: GString) -> VarDictionary {
        let id = preset_id.to_string();
        let url = base_url.to_string();
        let info = match classify_host(&url) {
            Ok(i) => i,
            Err(e) => return fail(e.reason()),
        };
        let origin = match origin_of(&url) {
            Ok(o) => o,
            Err(e) => return fail(e.reason()),
        };
        let lanish = matches!(info.class, HostClass::SameMachine | HostClass::LocalNetwork);
        let (preset_local, preset_required) = match cartalith_llm::presets::find(&id) {
            Some(p) => (p.is_local, p.key == KeyRequirement::Required),
            None => (lanish, id == CUSTOM_ANTHROPIC),
        };
        let mut d = vdict! {
            "ok" => true,
            "origin" => origin.as_str(),
            "is_local" => preset_local,
            "key_required" => preset_required || !lanish,
        };
        put_class(&mut d, info.class);
        d
    }

    /// The local-provider GPU rule (Ruling DF) as a plan:
    /// `{ok, warn, headline, detected, effect, switch, class?, class_label?,
    /// target?}`. `devices` is `gpu_enumerate_devices()`'s array, typed or not (only
    /// `key` and `kind` are read; an element that is not a dictionary is skipped); an empty array is a normal answer. `target`
    /// exists only when `switch` is true. An unknown `override_id` is an error,
    /// not a guess.
    #[func]
    fn llm_gpu_plan(
        &self,
        base_url: GString,
        local_selected: bool,
        override_id: GString,
        devices: VarArray,
    ) -> VarDictionary {
        let Some(ov) = GpuOverride::from_id(&override_id.to_string()) else {
            return fail("unknown GPU override");
        };
        let adapters: Vec<Adapter> = devices
            .iter_shared()
            .filter_map(|v| v.try_to::<VarDictionary>().ok())
            .map(|d| Adapter { key: text(&d, "key"), kind: text(&d, "kind") })
            .filter(|a| !a.key.is_empty())
            .collect();
        let class = classify_host(&base_url.to_string()).ok().map(|h| h.class);
        plan_dict(plan(local_selected, class, ov, &adapters), class)
    }

    /// The same rule on a phone (LLM-10, `cartalith_llm::gpu::plan_phone`):
    /// the same dictionary shape, `switch` always false and no `target`. The
    /// shell calls this when `OS.has_feature("android")`; no override is read
    /// because a phone has one GPU and nothing to switch to.
    #[func]
    fn llm_gpu_plan_phone(&self, base_url: GString, local_selected: bool) -> VarDictionary {
        let class = classify_host(&base_url.to_string()).ok().map(|h| h.class);
        plan_dict(plan_phone(local_selected, class), class)
    }

    /// The settings gate. `settings` carries `enabled`, `prose`, `commands`,
    /// `edit`, `local_allowed`, `remote_allowed`, `preset_id`, `base_url`,
    /// `model`, `key_source`, `env_var`; anything missing reads as off or
    /// empty. `capability` is `""` (the status line: every check except one
    /// capability's own switch), `"prose"`, `"commands"` or `"edit"`.
    /// `consented` is the list of origins the user has accepted.
    ///
    /// Answers `{ok: true, origin, class, class_label, local, key_required}` or
    /// `{ok: false, code, reason}` (plus `origin` for `consent_needed`). Reads
    /// the real environment and credential store for the key check and returns
    /// nothing about the key.
    #[func]
    fn llm_authorize(&self, settings: VarDictionary, capability: GString, consented: PackedStringArray) -> VarDictionary {
        let cap = match capability.to_string().as_str() {
            "" => None,
            "prose" => Some(Capability::Prose),
            "commands" => Some(Capability::Commands),
            "edit" => Some(Capability::Edit),
            _ => return fail("unknown capability"),
        };
        let s = settings_from(&settings);
        let consented: Vec<String> = consented.as_slice().iter().map(GString::to_string).collect();
        let (key_state, _key) =
            resolve_key(s.key_source, &s.env_var, &account_for(&s.preset_id), &|n| std::env::var(n).ok(), &OsKeyStore);
        match authorize(&s, cap, &key_state, &consented) {
            Ok(a) => {
                let mut d = vdict! {
                    "ok" => true, "origin" => a.origin.as_str(), "local" => a.local, "key_required" => a.key_required,
                };
                put_class(&mut d, a.class);
                d
            }
            Err(r) => {
                let mut d = vdict! { "ok" => false, "code" => r.code(), "reason" => r.reason().as_str() };
                if let cartalith_llm::Refusal::ConsentNeeded(o) = &r {
                    d.set("origin", o.as_str());
                }
                d
            }
        }
    }

    /// Whether a key is available for `account` from `source` (`"env"` or
    /// `"store"`), without revealing it: `{ok, code, present, store_available,
    /// problem?, masked?}`. `masked` exists only when present. `problem` is
    /// the words naming the fix and exists only when not present.
    #[func]
    fn llm_key_status(&self, source: GString, env_var: GString, account: GString) -> VarDictionary {
        let Some(src) = KeySource::from_id(&source.to_string()) else { return fail("unknown key source") };
        let store = OsKeyStore;
        let (state, key) = resolve_key(
            src,
            &env_var.to_string(),
            &account.to_string(),
            &|n| std::env::var(n).ok(),
            &store,
        );
        let mut d = vdict! {
            "ok" => true,
            "code" => state.code(),
            "present" => key.is_some(),
            "store_available" => store.available(),
        };
        if let Some(k) = &key {
            d.set("masked", k.masked().as_str());
        }
        if let Some(p) = state.problem() {
            d.set("problem", phone_problem(src, p).as_str());
        }
        d
    }

    /// Saves `key` for `account` in the OS credential store. Refuses (never
    /// falls back to a file) where there is no store. The reason is static
    /// text and never contains the key.
    #[func]
    fn llm_key_save(&self, account: GString, key: GString) -> VarDictionary {
        let store = OsKeyStore;
        if !store.available() {
            return fail(NO_STORE_SAVE);
        }
        let raw = key.to_string();
        let raw = raw.trim();
        let Ok(valid) = ApiKey::new(raw) else {
            return fail("That is not a usable key: no spaces or control characters, and not empty.");
        };
        drop(valid);
        match store.write(&account.to_string(), raw) {
            Ok(()) => vdict! { "ok" => true },
            Err(StoreError::Failed(why)) => fail(&format!("The OS credential store refused to save the key: {why}.")),
            Err(StoreError::Unavailable) => fail(NO_STORE_SAVE),
        }
    }

    /// Removes the saved key for `account`. Removing nothing is success.
    #[func]
    fn llm_key_forget(&self, account: GString) -> VarDictionary {
        let store = OsKeyStore;
        if !store.available() {
            return fail("No OS credential store is available here.");
        }
        match store.delete(&account.to_string()) {
            Ok(()) => vdict! { "ok" => true },
            Err(StoreError::Failed(why)) => fail(&format!("The OS credential store refused to remove the key: {why}.")),
            Err(StoreError::Unavailable) => fail("No OS credential store is available here."),
        }
    }
}
