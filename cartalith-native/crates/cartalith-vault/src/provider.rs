//! The vault provider trait, and the filesystem implementation of it
//! (`MARKDOWN_VAULT_INTEGRATION.md` §6).
//!
//! The spec asks for a platform-neutral provider with Windows and Android
//! implementations behind it. [`VaultProvider`] is that neutral shape: a
//! vault is a **root plus relative paths**, never an absolute path passed
//! around. §5's rule — "the absolute path or Android document URI is a
//! platform binding, not the semantic identity of the file" — is enforced by
//! the trait itself: every method takes a relative path, and the one method
//! that can hand back something filesystem-specific
//! ([`VaultProvider::as_fs_vault`]) is opt-in and `None` unless a provider
//! overrides it.
//!
//! [`FsVault`] is the Windows/desktop implementation and the only one this
//! crate ships. It refuses a relative path that escapes the root via
//! [`is_safe_relative_path`], kept as its own function so another provider
//! can reuse the same containment check rather than re-derive it.
//!
//! ## Android is a different crate's implementation, not a missing one
//!
//! `MARKDOWN_VAULT_SCOPE.md` milestone 4. Android needs the Storage Access
//! Framework (a `content://` tree URI and a persisted permission grant),
//! which is a Java-adjacent surface `std::fs` cannot reach — and this crate
//! must not learn to reach either; its own contract (`lib.rs`'s module doc)
//! is "no engine crate, no `gdext`". So the seam is [`VaultProvider`] itself,
//! not a `cfg(target_os = "android")` branch in this file: the
//! Storage-Access-Framework-backed provider lives in `cartalith-godot`
//! (`vault_saf::SafVaultProvider`), which *can* depend on Godot and holds a
//! `Callable` a GDScript handler supplies, so every operation this trait
//! defines is delegated to whatever platform code that handler is backed by.
//! That module's own doc states plainly what a real device pass still has to
//! verify — nothing about real SAF behaviour is or can be confirmed from a
//! crate this one exercises in a `cargo test`.

use std::path::{Component, Path, PathBuf};

/// A vault bound to a directory on this device.
#[derive(Debug, Clone)]
pub struct FsVault {
    root: PathBuf,
}

/// What [`FsVault`] knows about one file, and the whole basis of §14's
/// change detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileMeta {
    /// Seconds since the Unix epoch. `0` when the platform will not report a
    /// modification time — in which case the content hash is the only
    /// staleness signal, which is exactly why §14 recommends carrying both.
    pub modified: u64,
    pub len: u64,
}

#[derive(Debug)]
pub enum VaultError {
    /// A relative path that climbs out of the vault, is absolute, or names a
    /// Windows drive/UNC prefix. §30's least-privilege rule: Cartalith gets
    /// the directory the user chose and nothing above it.
    Escapes(String),
    /// The vault root is gone or is not a directory (§27 "Missing").
    RootUnavailable(PathBuf),
    Io(std::io::Error),
}

impl std::fmt::Display for VaultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VaultError::Escapes(p) => write!(f, "\"{p}\" is not inside this vault"),
            VaultError::RootUnavailable(p) => write!(f, "vault folder unavailable: {}", p.display()),
            VaultError::Io(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for VaultError {}

impl From<std::io::Error> for VaultError {
    fn from(e: std::io::Error) -> Self {
        VaultError::Io(e)
    }
}

/// Whether `rel` is safe to treat as a single vault-relative path: not
/// empty, and every component a plain name — never `..`, a root, or (on
/// Windows) a drive/UNC prefix.
///
/// [`FsVault::resolve`] uses this before joining `rel` onto a real
/// [`PathBuf`]; a provider with nothing to join it onto — a
/// `content://`-tree-backed one, say — has exactly the same reason to refuse
/// the same paths, so this is `pub` rather than private to that one caller.
/// §30's least-privilege rule applies to every provider, not just this
/// crate's own.
pub fn is_safe_relative_path(rel: &str) -> bool {
    !rel.is_empty() && Path::new(rel).components().all(|c| matches!(c, Component::Normal(_)))
}

/// The platform-neutral operations a Markdown vault provider gives
/// [`crate::VaultSession`] — `MARKDOWN_VAULT_INTEGRATION.md` §6's
/// `MarkdownVaultProvider` interface, narrowed to exactly the subset the
/// session calls. (§6 also lists `watchChanges`/`moveFile`/`deleteFile` as
/// *optional* capabilities; V1 offers none of the three on either platform,
/// so they are not here to implement as `unimplemented!()` on one side.)
///
/// [`FsVault`] is this crate's own implementation. `cartalith-godot`'s
/// `vault_saf::SafVaultProvider` is the other one this port ships, and lives
/// in a different crate on purpose — see this file's module doc.
pub trait VaultProvider: std::fmt::Debug {
    /// §7's Connected-vs-Missing, asked cheaply enough to ask on every panel
    /// open. A directory check for [`FsVault`]; whatever confirms the
    /// permission grant is still live for anything else.
    fn available(&self) -> bool;

    /// Relative paths of the Markdown files in the vault, bounded at `limit`
    /// (§31) — see [`FsVault::list_markdown`].
    fn list_markdown(&self, limit: usize) -> Result<Vec<String>, VaultError>;

    fn read(&self, rel: &str) -> Result<String, VaultError>;
    fn meta(&self, rel: &str) -> Result<FileMeta, VaultError>;
    fn exists(&self, rel: &str) -> bool;
    fn write(&self, rel: &str, text: &str) -> Result<(), VaultError>;

    /// Where this vault points, for **display only** — never stored in
    /// project data (§5). A filesystem path for [`FsVault`]; a tree URI for
    /// a Storage-Access-Framework-backed one.
    fn describe(&self) -> String;

    /// Down-casts to the filesystem provider, for the one caller
    /// (`vault_snapshot` in `cartalith-godot`) that needs a real [`PathBuf`]
    /// to hand an image writer. `None` unless a provider overrides it.
    ///
    /// This is not a temporary gap a future milestone closes — it is what
    /// §5's "the absolute path or Android document URI is a platform
    /// binding" means concretely: a `content://` tree has no [`PathBuf`] to
    /// give, so the one caller that needs one must ask first and handle
    /// `None` rather than assume every provider can answer.
    fn as_fs_vault(&self) -> Option<&FsVault> {
        None
    }
}

impl VaultProvider for FsVault {
    fn available(&self) -> bool {
        FsVault::available(self)
    }
    fn list_markdown(&self, limit: usize) -> Result<Vec<String>, VaultError> {
        FsVault::list_markdown(self, limit)
    }
    fn read(&self, rel: &str) -> Result<String, VaultError> {
        FsVault::read(self, rel)
    }
    fn meta(&self, rel: &str) -> Result<FileMeta, VaultError> {
        FsVault::meta(self, rel)
    }
    fn exists(&self, rel: &str) -> bool {
        FsVault::exists(self, rel)
    }
    fn write(&self, rel: &str, text: &str) -> Result<(), VaultError> {
        FsVault::write(self, rel, text)
    }
    fn describe(&self) -> String {
        self.root.display().to_string()
    }
    fn as_fs_vault(&self) -> Option<&FsVault> {
        Some(self)
    }
}

impl FsVault {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        FsVault { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Whether the root is currently a readable directory — §7's
    /// Connected-vs-Missing distinction, asked cheaply enough to ask on
    /// every panel open.
    pub fn available(&self) -> bool {
        self.root.is_dir()
    }

    /// Relative paths of the Markdown files in the vault, `/`-separated and
    /// sorted, capped at `limit` entries.
    ///
    /// **Bounded on purpose** (§31: "do not load the entire vault into
    /// memory", "do not parse every Markdown file at startup"). This walk
    /// reads directory entries and file *names* only — it never opens a
    /// file — and stops at `limit` so a 20,000-note vault cannot stall the
    /// UI thread. Dot-directories are skipped, which also means Cartalith
    /// never lists the `.obsidian/` config it has no business reading.
    pub fn list_markdown(&self, limit: usize) -> Result<Vec<String>, VaultError> {
        if !self.available() {
            return Err(VaultError::RootUnavailable(self.root.clone()));
        }
        let mut out: Vec<String> = Vec::new();
        let mut queue: Vec<(PathBuf, String)> = vec![(self.root.clone(), String::new())];
        while let Some((dir, prefix)) = queue.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else { continue };
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with('.') {
                    continue;
                }
                let rel = if prefix.is_empty() { name.clone() } else { format!("{prefix}/{name}") };
                match entry.file_type() {
                    Ok(t) if t.is_dir() => queue.push((entry.path(), rel)),
                    Ok(t) if t.is_file() && name.rsplit('.').next().is_some_and(|e| e.eq_ignore_ascii_case("md")) => {
                        out.push(rel)
                    }
                    _ => {}
                }
            }
            if out.len() >= limit {
                break;
            }
        }
        out.sort();
        out.truncate(limit);
        Ok(out)
    }

    pub fn read(&self, rel: &str) -> Result<String, VaultError> {
        Ok(std::fs::read_to_string(self.resolve(rel)?)?)
    }

    pub fn meta(&self, rel: &str) -> Result<FileMeta, VaultError> {
        let m = std::fs::metadata(self.resolve(rel)?)?;
        let modified = m
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Ok(FileMeta { modified, len: m.len() })
    }

    pub fn exists(&self, rel: &str) -> bool {
        self.resolve(rel).map(|p| p.is_file()).unwrap_or(false)
    }

    /// Writes `text` to `rel`, creating parent directories.
    ///
    /// **Write to a sibling temp file, then rename.** A vault is a person's
    /// worldbuilding corpus; a partial write from a power cut or a full disk
    /// would truncate a note they spent an evening on. `std::fs::rename` over
    /// an existing file is atomic on NTFS and on POSIX, so the note is either
    /// the old one or the new one and never half of each. The temp file is a
    /// sibling rather than in the system temp directory because a rename
    /// across volumes is a copy, and a copy is not atomic.
    pub fn write(&self, rel: &str, text: &str) -> Result<(), VaultError> {
        let path = self.resolve(rel)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut tmp = path.clone().into_os_string();
        tmp.push(".cartalith-tmp");
        let tmp = PathBuf::from(tmp);
        std::fs::write(&tmp, text)?;
        match std::fs::rename(&tmp, &path) {
            Ok(()) => Ok(()),
            Err(e) => {
                let _ = std::fs::remove_file(&tmp);
                Err(VaultError::Io(e))
            }
        }
    }

    /// A vault-relative path resolved against the root, refusing anything
    /// that would reach outside it.
    ///
    /// `..` is rejected outright rather than normalised away: a normalising
    /// resolver still lets `a/../../b` through when `a` is a symlink, and
    /// Cartalith has no need to support a path that climbs at all.
    pub fn resolve(&self, rel: &str) -> Result<PathBuf, VaultError> {
        if !is_safe_relative_path(rel) {
            return Err(VaultError::Escapes(rel.to_string()));
        }
        Ok(self.root.join(rel))
    }
}

/// FNV-1a 64, hex. Not a cryptographic hash and not asked to be: §14 wants
/// "optional content hash" as a cheap second opinion beside the modification
/// timestamp, for the case where a file is edited twice inside one
/// filesystem timestamp tick or restored from a backup with its mtime
/// preserved. A 64-bit non-cryptographic hash answers that at a byte a
/// nanosecond with no dependency added to the workspace.
pub fn content_hash(text: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(tag: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("cartalith-vault-test-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    /// Same fixtures `resolve`'s own test uses, checked directly against the
    /// helper it now delegates to — proof the extraction changed nothing.
    #[test]
    fn is_safe_relative_path_matches_resolves_own_boundary() {
        for bad in ["..", "../x.md", "a/../../b.md", "/etc/passwd", "C:/Windows/x.md", ""] {
            assert!(!is_safe_relative_path(bad), "{bad} must be unsafe");
        }
        assert!(is_safe_relative_path("Locations/Nareth.md"));
    }

    #[test]
    fn resolve_refuses_every_way_out_of_the_vault() {
        let v = FsVault::new(temp_root("resolve"));
        for bad in ["..", "../x.md", "a/../../b.md", "/etc/passwd", "C:/Windows/x.md", ""] {
            assert!(v.resolve(bad).is_err(), "{bad} must be refused");
        }
        assert!(v.resolve("Locations/Nareth.md").is_ok());
    }

    #[test]
    fn list_read_write_round_trip() {
        let root = temp_root("rt");
        let v = FsVault::new(&root);
        assert!(v.available());
        std::fs::create_dir_all(root.join("Locations")).unwrap();
        std::fs::create_dir_all(root.join(".obsidian")).unwrap();
        std::fs::write(root.join("Locations/Nareth.md"), "# Nareth\n").unwrap();
        std::fs::write(root.join("Index.md"), "# Index\n").unwrap();
        std::fs::write(root.join("notes.txt"), "not markdown").unwrap();
        std::fs::write(root.join(".obsidian/app.json"), "{}").unwrap();

        let files = v.list_markdown(100).unwrap();
        assert_eq!(files, ["Index.md", "Locations/Nareth.md"], "sorted, .md only, dot-dirs skipped");
        assert_eq!(v.list_markdown(1).unwrap().len(), 1, "the cap is real");

        assert_eq!(v.read("Locations/Nareth.md").unwrap(), "# Nareth\n");
        v.write("Locations/Nareth.md", "# Nareth\n\nedited\n").unwrap();
        assert_eq!(v.read("Locations/Nareth.md").unwrap(), "# Nareth\n\nedited\n");
        assert!(v.meta("Locations/Nareth.md").unwrap().len > 0);
        assert!(!v.exists("Nope.md"));
        // No temp file left behind by the atomic write.
        assert!(v.list_markdown(100).unwrap().iter().all(|f| !f.contains("cartalith-tmp")));

        // A new file in a new subdirectory.
        v.write("Regions/North.md", "# North\n").unwrap();
        assert!(v.exists("Regions/North.md"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_missing_root_is_reported_not_panicked() {
        let v = FsVault::new(std::env::temp_dir().join("cartalith-vault-does-not-exist"));
        assert!(!v.available());
        assert!(matches!(v.list_markdown(10), Err(VaultError::RootUnavailable(_))));
        assert!(v.read("x.md").is_err());
    }

    #[test]
    fn content_hash_is_stable_and_discriminating() {
        assert_eq!(content_hash("abc"), content_hash("abc"));
        assert_ne!(content_hash("abc"), content_hash("abd"));
        assert_eq!(content_hash("").len(), 16);
    }
}
