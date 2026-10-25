//! Where the API key comes from, and the trait an OS credential store fills.
//!
//! WHY: Ruling DD item 4 and `LLM_ASSISTANT_SCOPE.md` s.1 rule 5 say the key
//! lives in the OS credential store where one exists, otherwise an environment
//! variable, otherwise the feature refuses a remote call. **No plaintext key
//! file, ever.** This module is the pure half: the [`KeyStore`] trait (the
//! Windows Credential Manager implementation lives in `cartalith-godot`, the
//! only crate allowed a platform dependency, because this crate is held to one
//! dependency by `tests/no_network.rs`), the resolution order, and the
//! [`KeyState`] vocabulary the settings screen turns into a named reason.
//!
//! MUST NEVER: write a key to a file or to anything but a [`KeyStore`], fall
//! back from an unavailable store to a file, return a key's characters to the
//! UI (only [`ApiKey::masked`]), or encode "no key" as an empty or dummy key.

use crate::secret::ApiKey;

/// Where the user chose to keep the key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySource {
    /// Read from an environment variable at call time. The default (scope s.5).
    EnvVar,
    /// The OS credential store (Windows Credential Manager; Android Keystore
    /// arrives with LLM-10).
    OsStore,
}

impl KeySource {
    /// The id stored in settings.
    pub const fn id(self) -> &'static str {
        match self {
            KeySource::EnvVar => "env",
            KeySource::OsStore => "store",
        }
    }

    /// Parse a stored id; unknown or empty is `None`, never a guess.
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "env" => Some(KeySource::EnvVar),
            "store" => Some(KeySource::OsStore),
            _ => None,
        }
    }
}

/// The default environment variable name (the LLM-0 mock's `CART_LLM_KEY`).
pub const DEFAULT_ENV_VAR: &str = "CART_LLM_KEY";

/// Why a store operation did not happen. Static text only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreError {
    /// No credential store exists on this platform or build.
    Unavailable,
    /// The store exists and refused or failed the operation.
    Failed(&'static str),
}

/// A credential store. Values cross this seam as plain text because that is
/// what every OS store takes; callers wrap them in [`ApiKey`] at once.
pub trait KeyStore {
    /// Whether the platform has a usable store at all. Must not touch it.
    fn available(&self) -> bool;
    /// The saved secret for `account`, or `Ok(None)` if nothing is saved.
    /// "Nothing saved" is `None`, never an empty string.
    fn read(&self, account: &str) -> Result<Option<String>, StoreError>;
    /// Save (or replace) the secret for `account`.
    fn write(&self, account: &str, secret: &str) -> Result<(), StoreError>;
    /// Remove the secret for `account`; removing nothing is not an error.
    fn delete(&self, account: &str) -> Result<(), StoreError>;
}

/// A store that never exists: the answer on a platform with no credential
/// store. Every method says so; nothing is kept anywhere.
pub struct NoStore;

impl KeyStore for NoStore {
    fn available(&self) -> bool {
        false
    }
    fn read(&self, _: &str) -> Result<Option<String>, StoreError> {
        Err(StoreError::Unavailable)
    }
    fn write(&self, _: &str, _: &str) -> Result<(), StoreError> {
        Err(StoreError::Unavailable)
    }
    fn delete(&self, _: &str) -> Result<(), StoreError> {
        Err(StoreError::Unavailable)
    }
}

/// An in-memory store for tests of the resolution order. Never used in the
/// shipped build.
#[derive(Default)]
pub struct MemoryStore {
    items: std::cell::RefCell<std::collections::BTreeMap<String, String>>,
}

impl KeyStore for MemoryStore {
    fn available(&self) -> bool {
        true
    }
    fn read(&self, account: &str) -> Result<Option<String>, StoreError> {
        Ok(self.items.borrow().get(account).cloned())
    }
    fn write(&self, account: &str, secret: &str) -> Result<(), StoreError> {
        self.items.borrow_mut().insert(account.to_owned(), secret.to_owned());
        Ok(())
    }
    fn delete(&self, account: &str) -> Result<(), StoreError> {
        self.items.borrow_mut().remove(account);
        Ok(())
    }
}

/// Why a key is, or is not, available. The settings screen words each variant
/// differently (scope s.5: "each distinct, never a dead button or a bare 0").
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyState {
    /// A valid key was found.
    Present,
    /// The environment variable is not set (or is empty).
    EnvMissing {
        /// The variable's name (not a secret).
        var: String,
    },
    /// The variable name is not a legal environment variable name.
    EnvNameInvalid,
    /// The variable is set but its value is not a legal key.
    EnvInvalid {
        /// The variable's name.
        var: String,
    },
    /// The user chose the OS store but this platform has none.
    StoreUnavailable,
    /// The store exists and holds nothing for this account.
    StoreEmpty,
    /// The store failed.
    StoreError(&'static str),
    /// The store held a value that is not a legal key.
    StoreInvalid,
}


impl KeyState {
    /// A stable code for the bridge and for tests (never shown).
    pub fn code(&self) -> &'static str {
        match self {
            KeyState::Present => "present",
            KeyState::EnvMissing { .. } => "env_missing",
            KeyState::EnvNameInvalid => "env_name_invalid",
            KeyState::EnvInvalid { .. } => "env_invalid",
            KeyState::StoreUnavailable => "store_unavailable",
            KeyState::StoreEmpty => "store_empty",
            KeyState::StoreError(_) => "store_error",
            KeyState::StoreInvalid => "store_invalid",
        }
    }

    /// What is wrong, in words that name the fix. `None` exactly when the key
    /// is present: there is no "problem" text for a good key, so a caller
    /// cannot show an empty string as if it were a reason.
    pub fn problem(&self) -> Option<String> {
        Some(match self {
            KeyState::Present => return None,
            KeyState::EnvMissing { var } => format!(
                "Key missing: the environment variable {var} is not set. Set it and restart Cartalith, or choose the OS credential store."
            ),
            KeyState::EnvNameInvalid => {
                "The environment variable name is not valid: use letters, digits and underscores.".to_owned()
            }
            KeyState::EnvInvalid { var } => {
                format!("The environment variable {var} does not hold a usable key (no spaces or control characters).")
            }
            KeyState::StoreUnavailable => "No OS credential store is available here. Use an environment variable; Cartalith never saves a key to a file."
                .to_owned(),
            KeyState::StoreEmpty => "No key is saved in the OS credential store. Enter one in Settings.".to_owned(),
            KeyState::StoreError(_) | KeyState::StoreInvalid => {
                "The OS credential store failed or holds an unusable value.".to_owned()
            }
        })
    }
}

/// Longest environment variable name accepted (labelled judgement; Windows
/// allows 32 767 characters in a value, names are far shorter in practice).
pub const MAX_ENV_NAME: usize = 128;

/// Whether `name` is a plain environment-variable name: ASCII letters, digits
/// and `_`, not starting with a digit, 1 to [`MAX_ENV_NAME`] characters.
pub fn env_name_ok(name: &str) -> bool {
    let b = name.as_bytes();
    !b.is_empty()
        && b.len() <= MAX_ENV_NAME
        && !b[0].is_ascii_digit()
        && b.iter().all(|c| c.is_ascii_alphanumeric() || *c == b'_')
}

/// Whether `account` is a legal store account id (`[a-z0-9_-]`, 1 to 64).
/// Accounts are provider ids, so a stray character cannot build an odd
/// credential target name.
pub fn account_ok(account: &str) -> bool {
    !account.is_empty()
        && account.len() <= 64
        && account.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_' || c == b'-')
}

/// Resolve the key for a provider.
///
/// * `env` looks a variable up (the caller passes `std::env::var(..).ok()`;
///   a test passes a closure, so no test touches the real environment).
/// * Surrounding whitespace on an environment value is trimmed (a trailing
///   newline from `echo` is the usual cause); an interior space is not.
///
/// Returns the state and, only for [`KeyState::Present`], the key.
pub fn resolve_key(
    source: KeySource,
    env_name: &str,
    account: &str,
    env: &dyn Fn(&str) -> Option<String>,
    store: &dyn KeyStore,
) -> (KeyState, Option<ApiKey>) {
    match source {
        KeySource::EnvVar => {
            if !env_name_ok(env_name) {
                return (KeyState::EnvNameInvalid, None);
            }
            match env(env_name).map(|v| v.trim().to_owned()).filter(|v| !v.is_empty()) {
                None => (KeyState::EnvMissing { var: env_name.to_owned() }, None),
                Some(raw) => match ApiKey::new(&raw) {
                    Ok(k) => (KeyState::Present, Some(k)),
                    Err(_) => (KeyState::EnvInvalid { var: env_name.to_owned() }, None),
                },
            }
        }
        KeySource::OsStore => {
            if !store.available() {
                return (KeyState::StoreUnavailable, None);
            }
            if !account_ok(account) {
                return (KeyState::StoreError("the account id is not valid"), None);
            }
            match store.read(account) {
                Ok(None) => (KeyState::StoreEmpty, None),
                Ok(Some(raw)) => match ApiKey::new(&raw) {
                    Ok(k) => (KeyState::Present, Some(k)),
                    Err(_) => (KeyState::StoreInvalid, None),
                },
                Err(StoreError::Unavailable) => (KeyState::StoreUnavailable, None),
                Err(StoreError::Failed(why)) => (KeyState::StoreError(why), None),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_env(_: &str) -> Option<String> {
        None
    }

    /// Protects: an environment key is found, trimmed, and masked to four
    /// characters; a missing, blank or illegal one gets its own state.
    #[test]
    fn env_resolution_states() {
        let env = |n: &str| (n == "K").then(|| "  sk-test-abcdef1234\n".to_owned());
        let (st, key) = resolve_key(KeySource::EnvVar, "K", "ollama", &env, &NoStore);
        assert_eq!(st, KeyState::Present);
        assert_eq!(key.unwrap().masked(), "****1234");

        let (st, k) = resolve_key(KeySource::EnvVar, "OTHER", "x", &env, &NoStore);
        assert_eq!(st, KeyState::EnvMissing { var: "OTHER".into() });
        assert!(k.is_none());

        let blank = |_: &str| Some("   ".to_owned());
        assert_eq!(resolve_key(KeySource::EnvVar, "K", "x", &blank, &NoStore).0, KeyState::EnvMissing { var: "K".into() });

        let spaced = |_: &str| Some("two words".to_owned());
        assert_eq!(resolve_key(KeySource::EnvVar, "K", "x", &spaced, &NoStore).0, KeyState::EnvInvalid { var: "K".into() });

        for bad in ["", "1ABC", "A B", "A=B", "A-B", &"X".repeat(129)] {
            assert_eq!(resolve_key(KeySource::EnvVar, bad, "x", &env, &NoStore).0, KeyState::EnvNameInvalid, "{bad:?}");
        }
    }

    /// Protects: with no OS store the answer is `StoreUnavailable`, and the
    /// resolver does NOT fall back to the environment or to a file (Ruling DD
    /// item 4). MUTATION: make `OsStore` fall through to `env` and the
    /// `still_unavailable` assertion goes red.
    #[test]
    fn no_store_means_unavailable_never_a_fallback() {
        let env_has_it = |_: &str| Some("sk-in-env-1234567".to_owned());
        let (st, key) = resolve_key(KeySource::OsStore, "K", "openai", &env_has_it, &NoStore);
        let still_unavailable = st == KeyState::StoreUnavailable;
        assert!(still_unavailable && key.is_none());
    }

    /// Protects: the store path reads what was written, reports empty and
    /// invalid distinctly, and a delete empties it again.
    #[test]
    fn store_round_trip_states() {
        let s = MemoryStore::default();
        assert_eq!(resolve_key(KeySource::OsStore, "K", "openai", &no_env, &s).0, KeyState::StoreEmpty);
        s.write("openai", "sk-live-9999888877").unwrap();
        let (st, key) = resolve_key(KeySource::OsStore, "K", "openai", &no_env, &s);
        assert_eq!((st, key.unwrap().masked()), (KeyState::Present, "****8877".to_owned()));
        s.write("openai", "has space").unwrap();
        assert_eq!(resolve_key(KeySource::OsStore, "K", "openai", &no_env, &s).0, KeyState::StoreInvalid);
        s.delete("openai").unwrap();
        assert_eq!(resolve_key(KeySource::OsStore, "K", "openai", &no_env, &s).0, KeyState::StoreEmpty);
        assert!(matches!(
            resolve_key(KeySource::OsStore, "K", "Bad Account", &no_env, &s).0,
            KeyState::StoreError(_)
        ));
    }

    /// Protects: a short key shows no characters at all (the `masked` rule the
    /// resolver relies on), and the ids round-trip without a plausible default.
    #[test]
    fn ids_and_short_key_mask() {
        let env = |_: &str| Some("ollama".to_owned());
        assert_eq!(resolve_key(KeySource::EnvVar, "K", "x", &env, &NoStore).1.unwrap().masked(), "****");
        for s in [KeySource::EnvVar, KeySource::OsStore] {
            assert_eq!(KeySource::from_id(s.id()), Some(s));
        }
        assert_eq!(KeySource::from_id(""), None);
        assert_eq!(KeySource::from_id("file"), None);
        assert!(env_name_ok(DEFAULT_ENV_VAR));
    }
}
