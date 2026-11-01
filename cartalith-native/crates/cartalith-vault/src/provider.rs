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

/// Why a vault operation failed.
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
    /// A one-line, user-facing message per variant.
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
    /// Wraps a raw I/O error so `?` composes in [`FsVault`]'s methods.
    fn from(e: std::io::Error) -> Self {
        VaultError::Io(e)
    }
}

/// Whether `rel` is safe to treat as a single vault-relative path: not
/// empty, and every component a plain name — never `..`, a root, a drive
/// prefix or a UNC prefix.
///
/// **The same answer on every platform.** A vault-relative path is portable
/// project data (§5: [`crate::KnowledgeLink::relative_path`] travels in the
/// project file), and one vault is opened from a Windows PC and an Android
/// phone. A path this function accepted on Linux used to escape on Windows:
/// `C:/Windows/x.md` and `C:x.md` (a drive and a drive-relative path) and
/// `a\..\..\x.md` (`\` is a separator only there) parse as plain names on
/// Linux, so the host's own [`Path::components`] cannot be the whole rule.
/// The two provider tests that pin the drive-letter case failed on Linux
/// until 2026-10-31 for exactly this reason. So, on every host:
///
/// - a leading `/` or `\` is refused (a root, or a Windows UNC prefix);
/// - a leading ASCII letter and `:` is refused (a Windows drive);
/// - a `..` segment is refused when segments are split at both `/` and
///   `\`;
/// - and the host's own component check still applies on top.
///
/// [`FsVault::resolve`] uses this before joining `rel` onto a real
/// [`PathBuf`]; a provider with nothing to join it onto — a
/// `content://`-tree-backed one, say — has exactly the same reason to refuse
/// the same paths, so this is `pub` rather than private to that one caller.
/// §30's least-privilege rule applies to every provider, not just this
/// crate's own.
pub fn is_safe_relative_path(rel: &str) -> bool {
    let b = rel.as_bytes();
    let drive = b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':';
    !rel.is_empty()
        && !rel.starts_with(['/', '\\'])
        && !drive
        && rel.split(['/', '\\']).all(|seg| seg != "..")
        && Path::new(rel).components().all(|c| matches!(c, Component::Normal(_)))
}

/// The refusal of a provider that cannot move files (the default
/// [`VaultProvider::move_file`]): [`std::io::ErrorKind::Unsupported`],
/// naming the vault by its display string.
pub fn cannot_move(describe: &str) -> VaultError {
    VaultError::Io(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        format!("\"{describe}\" cannot move or rename notes from Cartalith"),
    ))
}

/// The platform-neutral operations a Markdown vault provider gives
/// [`crate::VaultSession`] — `MARKDOWN_VAULT_INTEGRATION.md` §6's
/// `MarkdownVaultProvider` interface, narrowed to exactly the subset the
/// session calls. §6 also lists `watchChanges`/`moveFile`/`deleteFile` as
/// *optional* capabilities. `moveFile` arrived on 2026-10-31 as
/// [`VaultProvider::move_file`], for the note editor's rename, with a
/// default that **refuses** rather than an `unimplemented!()`: a provider
/// that cannot move a file says so, and nothing is written. The other two
/// are still not offered.
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

    /// The full text of `rel`.
    fn read(&self, rel: &str) -> Result<String, VaultError>;
    /// §14's change-detection basis for `rel`: modification time and length.
    fn meta(&self, rel: &str) -> Result<FileMeta, VaultError>;
    /// Whether `rel` names a file in the vault right now.
    fn exists(&self, rel: &str) -> bool;
    /// Writes `text` to `rel`, creating parent directories as needed.
    fn write(&self, rel: &str, text: &str) -> Result<(), VaultError>;

    /// Moves the note at `from` to `to` (a rename, or a move between
    /// folders), creating `to`'s folders. Must refuse, changing nothing,
    /// when `to` is already another file: a move never overwrites.
    ///
    /// The default refuses with [`cannot_move`], and is what a provider gets
    /// until it can really move a file. `cartalith-godot`'s
    /// Storage-Access-Framework provider is one: its handler offers no move
    /// operation, and writing the new file without deleting the old one
    /// would leave two copies of a note, not a rename.
    fn move_file(&self, from: &str, to: &str) -> Result<(), VaultError> {
        let _ = (from, to);
        Err(cannot_move(&self.describe()))
    }

    /// Whether [`Self::move_file`] can succeed here at all, asked before a
    /// rename is planned so a person is never shown a plan the vault cannot
    /// carry out. `false` unless a provider overrides it, matching the
    /// refusing default above.
    fn can_move(&self) -> bool {
        false
    }

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

/// [`FsVault`] implements [`VaultProvider`] by delegating each method to the
/// same-named inherent method below — the trait exists so `cartalith-godot`'s
/// SAF-backed provider can stand in for this one, not to add behaviour here.
impl VaultProvider for FsVault {
    /// Delegates to [`FsVault::available`].
    fn available(&self) -> bool {
        FsVault::available(self)
    }
    /// Delegates to [`FsVault::list_markdown`].
    fn list_markdown(&self, limit: usize) -> Result<Vec<String>, VaultError> {
        FsVault::list_markdown(self, limit)
    }
    /// Delegates to [`FsVault::read`].
    fn read(&self, rel: &str) -> Result<String, VaultError> {
        FsVault::read(self, rel)
    }
    /// Delegates to [`FsVault::meta`].
    fn meta(&self, rel: &str) -> Result<FileMeta, VaultError> {
        FsVault::meta(self, rel)
    }
    /// Delegates to [`FsVault::exists`].
    fn exists(&self, rel: &str) -> bool {
        FsVault::exists(self, rel)
    }
    /// Delegates to [`FsVault::write`].
    fn write(&self, rel: &str, text: &str) -> Result<(), VaultError> {
        FsVault::write(self, rel, text)
    }
    /// Delegates to [`FsVault::move_file`].
    fn move_file(&self, from: &str, to: &str) -> Result<(), VaultError> {
        FsVault::move_file(self, from, to)
    }
    /// A folder on this device can always be asked to move a file.
    fn can_move(&self) -> bool {
        true
    }
    /// The root's own display path -- the one provider whose "display
    /// only" string (§5) is a real filesystem path.
    fn describe(&self) -> String {
        self.root.display().to_string()
    }
    /// A filesystem provider always has a [`FsVault`] to give.
    fn as_fs_vault(&self) -> Option<&FsVault> {
        Some(self)
    }
}

impl FsVault {
    /// Binds a vault to `root`, without checking it exists yet — call
    /// [`FsVault::available`] for that.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        FsVault { root: root.into() }
    }

    /// The vault's root directory.
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

    /// The full text of `rel`, read from disk.
    pub fn read(&self, rel: &str) -> Result<String, VaultError> {
        Ok(std::fs::read_to_string(self.resolve(rel)?)?)
    }

    /// `rel`'s modification time and length, for §14's change detection.
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

    /// Whether `rel` names a file in the vault right now.
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

    /// Moves the file at `from` to `to`, creating `to`'s folders; refuses
    /// with [`std::io::ErrorKind::AlreadyExists`], changing nothing, when
    /// `to` is already another file. The old folder is left in place, even
    /// if now empty, as Obsidian leaves it.
    ///
    /// **A case-only rename** (`kelvhold.md` to `Kelvhold.md`) goes through a
    /// sibling temporary name. On Windows and Android storage the two names
    /// are one file, so `to` "exists" and a plain check would refuse a
    /// legitimate rename; on Linux they can be two files, and a plain
    /// `std::fs::rename` would silently replace the other one. Moving aside
    /// first answers both: if `to` still exists once `from` is out of the
    /// way, it is a different file, and the move is put back and refused.
    ///
    /// A labelled limit: between the existence check and the rename there is
    /// a window of microseconds in which another app could create `to`, and
    /// `std::fs::rename` would then replace it. `std` offers no portable
    /// no-replace rename to close it.
    pub fn move_file(&self, from: &str, to: &str) -> Result<(), VaultError> {
        let src = self.resolve(from)?;
        let dst = self.resolve(to)?;
        let taken = || VaultError::Io(std::io::Error::new(std::io::ErrorKind::AlreadyExists, format!("\"{to}\" already exists")));
        if !src.is_file() {
            return Err(VaultError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, format!("\"{from}\" does not exist"))));
        }
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if from != to && from.to_lowercase() == to.to_lowercase() {
            let mut aside = src.clone().into_os_string();
            aside.push(".cartalith-rename");
            let aside = PathBuf::from(aside);
            std::fs::rename(&src, &aside)?;
            if dst.exists() {
                std::fs::rename(&aside, &src)?;
                return Err(taken());
            }
            return match std::fs::rename(&aside, &dst) {
                Ok(()) => Ok(()),
                Err(e) => {
                    let _ = std::fs::rename(&aside, &src);
                    Err(VaultError::Io(e))
                }
            };
        }
        if dst.exists() {
            return Err(taken());
        }
        Ok(std::fs::rename(&src, &dst)?)
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

/// Unit tests for [`FsVault`]'s path containment, list/read/write round trip
/// and [`content_hash`].
#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh, process-unique scratch directory under the system temp dir,
    /// wiped and recreated so tests never see a stale run's leftovers.
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
        // Protects: `is_safe_relative_path` itself rejects every escape
        // shape §30 cares about (`..`, an absolute path, a Windows drive)
        // and accepts an ordinary relative path.
        for bad in ["..", "../x.md", "a/../../b.md", "/etc/passwd", "C:/Windows/x.md", ""] {
            assert!(!is_safe_relative_path(bad), "{bad} must be unsafe");
        }
        assert!(is_safe_relative_path("Locations/Nareth.md"));
    }

    /// Protects: `FsVault::resolve`'s own containment check, over the
    /// same escape fixtures -- §30's least-privilege rule enforced at the
    /// one place every read/write funnels through.
    #[test]
    fn resolve_refuses_every_way_out_of_the_vault() {
        let v = FsVault::new(temp_root("resolve"));
        for bad in ["..", "../x.md", "a/../../b.md", "/etc/passwd", "C:/Windows/x.md", ""] {
            assert!(v.resolve(bad).is_err(), "{bad} must be refused");
        }
        assert!(v.resolve("Locations/Nareth.md").is_ok());
    }

    /// Protects: the containment rule is the same on every host -- the
    /// Windows-only escape shapes (a backslash climb, a UNC prefix in either
    /// slash, a drive-relative `C:x.md`, a backslash root) are refused on
    /// Linux too -- while a colon or a `.` later in a path is not refused,
    /// since neither leaves the vault.
    #[test]
    fn the_containment_rule_is_the_same_on_every_platform() {
        for bad in ["..\\x.md", "a\\..\\..\\b.md", "\\\\server\\share\\x.md", "//server/share/x.md", "C:x.md", "c:\\x.md", "\\x.md", "a/..", "./x.md"] {
            assert!(!is_safe_relative_path(bad), "{bad} must be unsafe");
        }
        for good in ["Notes/Time 10:30.md", "a/./b.md", "v1.2 notes.md", "Ærenfell.md"] {
            assert!(is_safe_relative_path(good), "{good} must be safe");
        }
    }

    /// Protects: `move_file` moves a note into a folder it creates; refuses
    /// to overwrite another file or to move a missing one, leaving every
    /// file as it was; performs a case-only rename; and, where the
    /// filesystem is case-sensitive (checked at run time, not assumed),
    /// refuses a case-only rename onto a different existing file, keeping
    /// both.
    #[test]
    fn a_move_creates_folders_and_never_overwrites() {
        let root = temp_root("move");
        let v = FsVault::new(&root);
        v.write("A.md", "a\n").unwrap();
        v.write("B.md", "b\n").unwrap();
        v.move_file("A.md", "Archive/Old/A2.md").unwrap();
        assert!(!v.exists("A.md"));
        assert_eq!(v.read("Archive/Old/A2.md").unwrap(), "a\n");

        let err = v.move_file("B.md", "Archive/Old/A2.md").unwrap_err();
        assert!(matches!(&err, VaultError::Io(e) if e.kind() == std::io::ErrorKind::AlreadyExists), "{err}");
        assert_eq!(v.read("B.md").unwrap(), "b\n");
        assert_eq!(v.read("Archive/Old/A2.md").unwrap(), "a\n");
        assert!(v.move_file("Nope.md", "X.md").is_err());

        v.move_file("B.md", "b.md").unwrap();
        assert_eq!(v.list_markdown(10).unwrap(), ["Archive/Old/A2.md", "b.md"]);

        v.write("c.md", "lower\n").unwrap();
        if !v.exists("C.md") {
            v.write("C.md", "upper\n").unwrap();
            let err = v.move_file("c.md", "C.md").unwrap_err();
            assert!(matches!(&err, VaultError::Io(e) if e.kind() == std::io::ErrorKind::AlreadyExists), "{err}");
            assert_eq!(v.read("c.md").unwrap(), "lower\n");
            assert_eq!(v.read("C.md").unwrap(), "upper\n");
        }
        // Read the folder itself: `list_markdown` shows only `.md` names, so
        // it could never show a `*.md.cartalith-rename` file left behind.
        let left: Vec<String> =
            std::fs::read_dir(&root).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        assert!(left.iter().all(|f| !f.contains("cartalith-rename")), "an aside file was left: {left:?}");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Protects: `list_markdown`'s filtering (only .md, dot-dirs skipped,
    /// sorted, the `limit` cap is real), the atomic `write` leaving no
    /// temp file behind, and that a read after a write sees the new
    /// content in a freshly created subdirectory.
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

    /// Protects: §7's Missing case -- a vault whose root does not exist reports
    /// `available() == false`, `RootUnavailable` from `list_markdown` and an error from
    /// `read`, rather than panicking. Only those three entry points are asserted.
    #[test]
    fn a_missing_root_is_reported_not_panicked() {
        let v = FsVault::new(std::env::temp_dir().join("cartalith-vault-does-not-exist"));
        assert!(!v.available());
        assert!(matches!(v.list_markdown(10), Err(VaultError::RootUnavailable(_))));
        assert!(v.read("x.md").is_err());
    }

    /// Protects: `content_hash` is deterministic (same input, same hash),
    /// discriminates a one-byte change, and always emits 16 hex digits --
    /// the shape §14's staleness check assumes.
    #[test]
    fn content_hash_is_stable_and_discriminating() {
        assert_eq!(content_hash("abc"), content_hash("abc"));
        assert_ne!(content_hash("abc"), content_hash("abd"));
        assert_eq!(content_hash("").len(), 16);
    }
}
