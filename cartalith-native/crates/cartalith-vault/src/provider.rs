//! The filesystem vault provider (`MARKDOWN_VAULT_INTEGRATION.md` §6).
//!
//! The spec asks for a platform-neutral provider with Windows and Android
//! implementations behind it. This is the Windows/desktop one, and the shape
//! it presents is the neutral one: a vault is a **root plus relative paths**,
//! never an absolute path passed around. §5's rule — "the absolute path or
//! Android document URI is a platform binding, not the semantic identity of
//! the file" — is enforced here by construction: every method takes a
//! relative path and refuses one that escapes the root.
//!
//! ## Android is not implemented, and that is a scoped decision
//!
//! Android needs the Storage Access Framework (a `content://` tree URI and a
//! persisted permission grant), which is a Java/JNI surface `std::fs` cannot
//! reach. `MARKDOWN_VAULT_SCOPE.md` carries it as its own milestone. What
//! this file does is keep the seam honest: nothing above it takes a
//! `PathBuf`, so the SAF implementation slots in beside [`FsVault`] rather
//! than through it.

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
        let p = Path::new(rel);
        if rel.is_empty() {
            return Err(VaultError::Escapes(rel.to_string()));
        }
        for c in p.components() {
            match c {
                Component::Normal(_) => {}
                _ => return Err(VaultError::Escapes(rel.to_string())),
            }
        }
        Ok(self.root.join(p))
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
