//! The Android half of `MARKDOWN_VAULT_SCOPE.md` milestone 4 — the Storage
//! Access Framework provider `cartalith-vault/src/provider.rs`'s module doc
//! names and does not itself implement, because that crate must not depend
//! on Godot (`cartalith-vault/src/lib.rs`'s own contract: "no engine crate,
//! no `gdext`"). This crate already does, so this is where it lives.
//!
//! ## Why this is a delegate, not an implementation
//!
//! **Rust cannot call Android's Java APIs directly.** There is no JNI
//! binding anywhere in this workspace, and reaching a `content://` tree URI
//! — opening it, listing its children, streaming bytes in and out of it —
//! is a Java-framework surface (`DocumentFile`/`DocumentsContract`,
//! `ContentResolver`) with no `std::fs` equivalent. `gdext` gives this crate
//! one bridge across that gap that does not require writing or vendoring any
//! Java: the Godot object system itself. A GDScript object already runs
//! inside the same process and — on the real device this milestone cannot
//! be verified from — has whatever access Godot's own Android platform
//! layer or a custom Android plugin gives it.
//!
//! So [`SafVaultProvider`] is honestly what the lane brief asked for it to
//! be if that turned out to be the honest answer: **a trait implementation
//! that delegates every operation to a GDScript-side handler**, supplied as
//! a single dispatcher [`Callable`]. It carries no Android-specific code of
//! its own — it cannot, from this crate — only the marshaling between
//! [`VaultProvider`]'s Rust shape and the `{op, args} -> {ok, ...}` shape the
//! handler speaks.
//!
//! ## The dispatcher contract
//!
//! `vault_connect_saf` takes a `Callable` of one GDScript signature:
//!
//! ```gdscript
//! func _saf_dispatch(op: String, args: Array) -> Dictionary
//! ```
//!
//! reused rather than invented: it is exactly the `{"ok": bool, "error":
//! String, ...}` shape every method in `vault_bridge.rs` already returns to
//! GDScript, now crossing the boundary in the other direction. `op` and
//! `args`, per [`VaultProvider`] method:
//!
//! | `op` | `args` | success carries |
//! |---|---|---|
//! | `"available"` | `[]` | `value: bool` |
//! | `"list"` | `[limit: int]` | `value: PackedStringArray` of vault-relative paths |
//! | `"read"` | `[rel: String]` | `value: String` |
//! | `"meta"` | `[rel: String]` | `modified: int` (Unix seconds, `0` if unknown), `len: int` |
//! | `"exists"` | `[rel: String]` | `value: bool` |
//! | `"write"` | `[rel: String, text: String]` | (no payload needed) |
//!
//! A call that cannot succeed returns `{"ok": false, "error": "..."}` and
//! nothing here ever panics on it — an unreachable grant, a revoked
//! permission or a missing file all become a normal [`VaultError`] the same
//! way a missing directory does for [`FsVault`](cartalith_vault::FsVault).
//! `rel` is validated with [`is_safe_relative_path`] on the Rust side before
//! it ever reaches the dispatcher — the same containment [`FsVault::resolve`]
//! (cartalith_vault::FsVault::resolve) enforces for a real path, applied
//! here because nothing guarantees the handler re-derives it for a document
//! tree walked by name segments instead.
//!
//! ## What this pass built, and what it did not
//!
//! Two layers of runnable proof, neither of which touches a real device:
//!
//! 1. **`cargo test --workspace`** — in `cartalith-vault`'s own suite, a
//!    proof that [`cartalith_vault::VaultSession`] performs the full
//!    attach/edit/write/conflict-refusal cycle through *any*
//!    [`VaultProvider`], not only [`FsVault`](cartalith_vault::FsVault).
//!    That is the strongest claim verifiable with no Godot process at all.
//! 2. **`godot-project/_vaultsaf_probe.gd`** — headless, run as
//!    `Godot_v4.7.1-stable_win64_console.exe --headless --path godot-project
//!    --script _vaultsaf_probe.gd`, the same convention every other
//!    `_*_probe.gd` in this project uses. It builds a fake dispatcher in
//!    GDScript (an in-memory map, no `content://` under it either) and drives
//!    `vault_connect_saf` through `EngineBridge`, proving the marshaling
//!    above actually round-trips through a live `Callable` — list, read, the
//!    full attach/edit/preview/write cycle, and the source-changed refusal —
//!    which `cargo test` alone cannot, since constructing a `Callable` or a
//!    `GString` needs a live Godot runtime (this file's own `#[cfg(test)]`
//!    module says the same about `guard`). Passed at the time this was
//!    written; re-run it after touching this file, `vault_bridge.rs`'s
//!    `err`/`ok`, or `cartalith-vault`'s `VaultProvider` trait shape.
//!
//! **Not built, and not this pass's to build**: the GDScript
//! `_saf_dispatch` implementation itself, and with it every real Android
//! behaviour milestone 4 was supposed to deliver. Concretely, unverified
//! from this environment and each a real question a device pass has to
//! answer, not assume:
//!
//! - **How the picker and the grant actually work on this Godot version.**
//!   Whether Godot 4.7's Android export can hand GDScript a persisted
//!   `content://` tree URI (via `DisplayServer.file_dialog_show` or
//!   otherwise) without a custom Android plugin, or whether one has to be
//!   written — a `.aar` registered in `export_presets.cfg`, which this pass
//!   does not touch (`CLAUDE.md`'s constraint list). Either answer is
//!   plausible; neither is checked here.
//! - **Whether `takePersistableUriPermission` survives** an app restart, a
//!   device reboot, and an Android version upgrade — SAF's own documented
//!   edge cases, not this port's.
//! - **Revocation.** A user can revoke folder access from Android's system
//!   settings at any time; `"available"` is where that has to surface, and
//!   only a real device can exercise the revoked path.
//! - **§35 criterion 2** (cross-device vault identity) has its *identity*
//!   half already done and unrelated to this file: [`VaultRef::new`]
//!   (cartalith_vault::links::VaultRef::new) derives `vault_id` from a hash
//!   of the trimmed display name, not from the binding, so the same logical
//!   vault connected on Windows and Android was already going to land on the
//!   same id before this pass existed. What this pass adds is the *binding*
//!   half the criterion also needs — a provider that can actually be
//!   reached on Android — and confirming the two halves agree on a real
//!   second device is still open.
//! - Filenames with characters SAF and NTFS disagree on, very large notes,
//!   and concurrent writes from another SAF-aware app landing mid-operation.

use cartalith_vault::provider::{FileMeta, VaultError, VaultProvider, is_safe_relative_path};
use godot::prelude::*;

use crate::WorldGen;
use crate::vault_bridge::{err, ok};

/// Delegates every [`VaultProvider`] operation to a GDScript-supplied
/// [`Callable`] — see this module's own doc for the dispatcher contract and,
/// just as importantly, for what using it still leaves unverified.
#[derive(Debug)]
pub struct SafVaultProvider {
    tree_uri: String,
    dispatch: Callable,
}

impl SafVaultProvider {
    pub fn new(tree_uri: String, dispatch: Callable) -> Self {
        SafVaultProvider { tree_uri, dispatch }
    }

    /// Invokes the dispatcher with `(op, args)` — the two positional
    /// arguments this module's own doc specifies, `args` nested as its own
    /// `Array` rather than spliced flat, so the GDScript side really does
    /// see the `func _saf_dispatch(op: String, args: Array)` signature
    /// documented above and not a variadic call with `op` as the first
    /// element — and unwraps the `{ok, error, ...}` result.
    ///
    /// Godot's own contract for an invalid `Callable` is to print nothing
    /// and hand back `NIL` (`Callable::callv`'s doc comment) — silence that
    /// would otherwise read as "the file does not exist" rather than "the
    /// handler is gone". Checked explicitly rather than trusted, along with
    /// a malformed reply (wrong type, no `ok` key), so every failure this
    /// method can hit becomes a real [`VaultError`] and never a panic
    /// crossing the `gdext` boundary (`cartalith-rust-conventions`).
    fn call(&self, op: &str, args: VarArray) -> Result<VarDictionary, VaultError> {
        if !self.dispatch.is_valid() {
            return Err(unavailable("the Android storage handler is not connected"));
        }
        let result = self.dispatch.callv(&varray![op, &args]);
        let Ok(dict) = result.try_to::<VarDictionary>() else {
            return Err(unavailable("the Android storage handler returned something other than a result"));
        };
        if dict.get("ok").and_then(|v| v.try_to::<bool>().ok()).unwrap_or(false) {
            Ok(dict)
        } else {
            let msg = dict.get("error").map(|v| v.to_string()).unwrap_or_else(|| "the operation failed".to_string());
            Err(unavailable(msg))
        }
    }
}

fn unavailable(msg: impl std::fmt::Display) -> VaultError {
    VaultError::Io(std::io::Error::other(msg.to_string()))
}

/// Every method that carries a `rel` checks it before it ever reaches the
/// dispatcher — see this module's own doc for why that is not redundant
/// with whatever the handler does on its side.
fn guard(rel: &str) -> Result<(), VaultError> {
    if is_safe_relative_path(rel) { Ok(()) } else { Err(VaultError::Escapes(rel.to_string())) }
}

impl VaultProvider for SafVaultProvider {
    fn available(&self) -> bool {
        self.call("available", varray![]).ok().and_then(|d| d.get("value")).and_then(|v| v.try_to::<bool>().ok()).unwrap_or(false)
    }

    fn list_markdown(&self, limit: usize) -> Result<Vec<String>, VaultError> {
        let dict = self.call("list", varray![limit as i64])?;
        let value = dict.get("value").ok_or_else(|| unavailable("\"list\" carried no value"))?;
        let arr = value.try_to::<PackedStringArray>().map_err(|_| unavailable("\"list\" did not return a string array"))?;
        Ok(arr.to_vec().into_iter().map(|s| s.to_string()).collect())
    }

    fn read(&self, rel: &str) -> Result<String, VaultError> {
        guard(rel)?;
        let dict = self.call("read", varray![rel])?;
        let value = dict.get("value").ok_or_else(|| unavailable("\"read\" carried no value"))?;
        value.try_to::<GString>().map(|s| s.to_string()).map_err(|_| unavailable("\"read\" did not return text"))
    }

    fn meta(&self, rel: &str) -> Result<FileMeta, VaultError> {
        guard(rel)?;
        let dict = self.call("meta", varray![rel])?;
        let modified = dict.get("modified").and_then(|v| v.try_to::<i64>().ok()).unwrap_or(0).max(0) as u64;
        let len = dict.get("len").and_then(|v| v.try_to::<i64>().ok()).unwrap_or(0).max(0) as u64;
        Ok(FileMeta { modified, len })
    }

    fn exists(&self, rel: &str) -> bool {
        if guard(rel).is_err() {
            return false;
        }
        self.call("exists", varray![rel]).ok().and_then(|d| d.get("value")).and_then(|v| v.try_to::<bool>().ok()).unwrap_or(false)
    }

    fn write(&self, rel: &str, text: &str) -> Result<(), VaultError> {
        guard(rel)?;
        self.call("write", varray![rel, text])?;
        Ok(())
    }

    fn describe(&self) -> String {
        self.tree_uri.clone()
    }
}

#[godot_api(secondary)]
impl WorldGen {
    /// Connects a Storage-Access-Framework vault: `tree_uri` is the
    /// document tree URI Android granted (already persisted on the
    /// GDScript/Java side — this call does not request or persist a
    /// permission, only binds one that already exists), and `dispatch` is
    /// the handler this module's own doc specifies. Same `{ok, error,
    /// vault_id}` shape as [`Self::vault_connect`], and every other vault
    /// call (`vault_info`, `vault_list_files`, `vault_read_file`, `vault_attach`,
    /// …) works on the result exactly as it does for a filesystem vault —
    /// [`cartalith_vault::VaultSession`] does not know which one it is
    /// holding.
    ///
    /// Refuses immediately, connecting nothing, when `dispatch` is not a
    /// callable at all or when the handler itself reports the grant
    /// unreachable right now (mirrors [`Self::vault_connect`]'s refusal of a
    /// missing directory).
    #[func]
    fn vault_connect_saf(&mut self, tree_uri: GString, display_name: GString, dispatch: Callable) -> VarDictionary {
        if !dispatch.is_valid() {
            return err("no Android storage handler was given to connect through");
        }
        let uri = tree_uri.to_string();
        if uri.trim().is_empty() {
            return err("no folder was granted");
        }
        let provider = SafVaultProvider::new(uri, dispatch);
        let name = display_name.to_string();
        let name = if name.trim().is_empty() { None } else { Some(name) };
        match self.vault.connect_provider(Box::new(provider), name.as_deref()) {
            Ok(id) => {
                let mut d = ok();
                d.set("vault_id", id);
                d
            }
            Err(e) => err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `SafVaultProvider` cannot be exercised here — building a `Callable`
    /// or a `GString` needs a live Godot runtime, the same reason
    /// `vault_bridge.rs`'s own tests give for not constructing one either.
    /// What plain Rust *can* prove is `guard`'s delegation, since
    /// `is_safe_relative_path` itself is already covered in
    /// `cartalith-vault`.
    #[test]
    fn guard_rejects_exactly_what_is_safe_relative_path_rejects() {
        assert!(guard("Locations/Nareth.md").is_ok());
        for bad in ["..", "../x.md", "/etc/passwd", ""] {
            assert!(guard(bad).is_err(), "{bad} must be refused");
        }
    }
}
