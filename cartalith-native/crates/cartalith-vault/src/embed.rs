//! Embeds (`![[note]]`, `![[note#Heading]]`, `![[pic.png|300]]`, `![alt](pic.png)`): what the
//! Preview shows in place of the markup (`VAULT_EDITOR_PROPOSAL.md` s.3.4, milestone VE-7).
//!
//! This module decides, in Rust and with literal-pinned tests, everything about an embed that can go
//! wrong, so the Godot side only draws the answer ([`Embed`]) and never re-derives a rule:
//!
//! - **A cycle or a runaway chain never renders.** The caller passes the chain of notes it is
//!   already drawing (`stack`, the open note first). A target already on it is [`Embed::Cycle`]; a
//!   chain with [`MAX_EMBED_DEPTH`] embedded levels already open is [`Embed::TooDeep`]. Both are
//!   answers, not errors: Preview draws a clear placeholder and the recursion ends there.
//! - **An image never leaves the vault.** [`provider::is_safe_relative_path`] refuses `..`, a root, a
//!   drive and a UNC prefix before any path is joined, and the joined file is canonicalised and must
//!   still lie under the canonical vault root (which is what catches a symlink inside the vault that
//!   points out of it). The refusal is [`Embed::Escape`]. A remote address (`https:`) is never
//!   fetched: [`Embed::Unsupported`].
//! - **An embed is a read-only view.** Nothing here writes; the embedded note is edited by opening
//!   it, as in Obsidian.
//! - **A note is data.** The target, heading and size come out of free text; every one is parsed
//!   defensively and a value that does not parse is "not a size", never a guess.
//!
//! Divergence from Obsidian, disclosed: an image is found as a vault-relative path (from the note's
//! own folder first, then from the vault root), not by bare file name anywhere in the vault, because
//! the provider lists Markdown notes only. A bare name that is not at either place reads as missing.

use std::path::Path;

use crate::notelinks::Resolution;
use crate::{Error, VaultSession, markdown, provider};

/// How many embedded levels may be open below the note being read: the open note embeds A (1),
/// A embeds B (2), B embeds C (3), and C's own embeds are [`Embed::TooDeep`]. A labelled judgement:
/// three covers a map note that embeds a region note that embeds a place note, and a chain of any
/// real length is a cycle or a mistake; Obsidian itself stops a self-embed rather than recursing.
pub const MAX_EMBED_DEPTH: usize = 3;

/// The most characters of an embedded section handed to Preview. A labelled judgement: about two
/// screens of prose; an embed is a view, and the note is one click away for the rest.
pub const EXCERPT_MAX_CHARS: usize = 4000;

/// The largest image read, in bytes. A labelled judgement: a 16 MiB PNG is already a 4k map; a
/// larger file in a note's Preview would stall the editor and the memory is not recoverable.
pub const IMAGE_MAX_BYTES: u64 = 16 * 1024 * 1024;

/// Image extensions Godot's `Image.load_from_file` decodes (checked against its documented list:
/// png, jpg, webp, bmp, svg and others). `gif` and `avif` are Obsidian images that Godot cannot
/// decode, so they are [`Embed::Unsupported`] with that reason rather than a silent blank.
pub const IMAGE_EXTENSIONS: [&str; 6] = ["png", "jpg", "jpeg", "webp", "bmp", "svg"];

/// What follows the `#` of an embed target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fragment {
    /// No `#`: the note's first section.
    None,
    /// `#Heading` (or `#Parent#Child`, which reads as its last segment): that heading's section.
    Heading(String),
    /// `#^id`: the block (paragraph or list) ending in ` ^id`.
    Block(String),
}

/// An embed's inner text, parsed. `![[Note#Heading|300]]` has target `Note`, fragment
/// `Heading`, width 300.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parsed {
    /// The note or file part, trimmed, without fragment or size.
    pub target: String,
    pub fragment: Fragment,
    /// The width an image is asked to show at (`|300` or `|300x200`), in pixels.
    pub width: Option<u32>,
}

/// Parses the text between `![[` and `]]`. The part after the first `|` is a size when it is
/// `N` or `NxM` (digits only, `N` > 0), and otherwise an alias, which an embed does not use.
pub fn parse(inner: &str) -> Parsed {
    let (left, right) = match inner.split_once('|') {
        Some((l, r)) => (l, Some(r)),
        None => (inner, None),
    };
    let width = right.and_then(|r| {
        let w = r.trim().split(['x', 'X']).next().unwrap_or("");
        let h_ok = r.trim().split(['x', 'X']).nth(1).is_none_or(|h| !h.is_empty() && h.bytes().all(|b| b.is_ascii_digit()));
        if !w.is_empty() && w.bytes().all(|b| b.is_ascii_digit()) && h_ok {
            w.parse::<u32>().ok().filter(|n| *n > 0)
        } else {
            None
        }
    });
    let (note, frag) = match left.split_once('#') {
        Some((n, f)) => (n, Some(f)),
        None => (left, None),
    };
    let fragment = match frag.map(str::trim) {
        None | Some("") => Fragment::None,
        Some(f) if f.starts_with('^') => Fragment::Block(f[1..].trim().to_string()),
        Some(f) => Fragment::Heading(f.rsplit('#').next().unwrap_or(f).trim().to_string()),
    };
    Parsed { target: note.trim().to_string(), fragment, width }
}

/// Whether `target` is an address with a URL scheme (`https:`, `data:`, `file:`...), which an embed
/// never fetches or reads. A Windows drive (`C:`) is not a scheme: it is one letter, and is refused
/// by the path check instead.
pub fn is_external(target: &str) -> bool {
    match target.split_once(':') {
        Some((scheme, _)) => {
            scheme.len() >= 2
                && scheme.as_bytes()[0].is_ascii_alphabetic()
                && scheme.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'+' || b == b'.' || b == b'-')
        }
        None => false,
    }
}

/// The file extension of `target`, lowercased, or `""`.
fn ext_of(target: &str) -> String {
    let last = target.rsplit(['/', '\\']).next().unwrap_or(target);
    last.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase()).unwrap_or_default()
}

/// Whether `target` is an image Preview can draw.
pub fn is_drawable_image(target: &str) -> bool {
    IMAGE_EXTENSIONS.contains(&ext_of(target).as_str())
}

/// The places an attachment named `target` is looked for, in order: beside the note that embeds it,
/// then at the vault root. Both are vault-relative; `target` must already have passed
/// [`provider::is_safe_relative_path`], so neither can climb out.
pub fn candidates(from: &str, target: &str) -> Vec<String> {
    let t = target.replace('\\', "/");
    let mut v = Vec::new();
    if let Some((dir, _)) = from.rsplit_once('/') {
        v.push(format!("{dir}/{t}"));
    }
    v.push(t);
    v
}

/// Why an embed's chain may not go on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate {
    Cycle,
    TooDeep,
}

/// Whether the note at `path` may be drawn inside the chain `stack` (the open note first, then each
/// note being drawn within it). A path already on the chain, compared ignoring case as the resolver
/// does, is a [`Gate::Cycle`]; otherwise a chain that already has [`MAX_EMBED_DEPTH`] embedded levels
/// open (`stack.len() > MAX_EMBED_DEPTH`) is [`Gate::TooDeep`]. The cycle test comes first so the
/// reason given is the true one.
pub fn gate(stack: &[String], path: &str) -> Result<(), Gate> {
    if stack.iter().any(|s| s.eq_ignore_ascii_case(path)) {
        return Err(Gate::Cycle);
    }
    if stack.len() > MAX_EMBED_DEPTH {
        return Err(Gate::TooDeep);
    }
    Ok(())
}

/// A section handed to Preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Excerpt {
    pub text: String,
    /// True when the section was longer than the cap and the end was left off.
    pub truncated: bool,
    /// The heading that was found (for a heading fragment), as written in the note.
    pub heading: Option<String>,
}

/// Why a fragment names nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FragmentError {
    HeadingMissing(String),
    BlockMissing(String),
}

/// The part of `text` an embed with `fragment` shows, at most `max_chars` characters.
///
/// - **Heading**: that heading's section, heading line included, to the next heading at the same or
///   a shallower level. Headings match ignoring case (Obsidian's rule for `[[Note#heading]]`).
/// - **Block**: the lines around the one ending in ` ^id`, out to the nearest blank lines.
/// - **None**: the note's first section: what precedes the first heading when that is not blank,
///   else the first heading's section. Front matter is never shown.
pub fn excerpt(text: &str, fragment: &Fragment, max_chars: usize) -> Result<Excerpt, FragmentError> {
    let text = text.replace("\r\n", "\n");
    let fm = markdown::frontmatter_end(&text);
    let secs = markdown::sections(&text);
    let (slice, heading): (String, Option<String>) = match fragment {
        Fragment::Heading(h) => {
            let want = h.trim();
            match secs.iter().find(|s| s.title.eq_ignore_ascii_case(want)) {
                Some(s) => (text[s.span.clone()].to_string(), Some(s.title.clone())),
                None => return Err(FragmentError::HeadingMissing(want.to_string())),
            }
        }
        Fragment::Block(id) => match block_of(&text[fm..], id) {
            Some(b) => (b, None),
            None => return Err(FragmentError::BlockMissing(id.clone())),
        },
        Fragment::None => {
            let first_heading = secs.first().map(|s| s.heading.start).unwrap_or(text.len());
            let pre = &text[fm..first_heading.max(fm)];
            if !pre.trim().is_empty() {
                (pre.to_string(), None)
            } else if let Some(s) = secs.first() {
                (text[s.span.clone()].to_string(), Some(s.title.clone()))
            } else {
                (String::new(), None)
            }
        }
    };
    let slice = slice.trim_matches('\n').trim_end().to_string();
    if slice.chars().count() > max_chars {
        let cut: String = slice.chars().take(max_chars).collect();
        return Ok(Excerpt { text: cut.trim_end().to_string(), truncated: true, heading });
    }
    Ok(Excerpt { text: slice, truncated: false, heading })
}

/// The block whose last line ends in ` ^id` (or is only `^id`), as the lines from the blank line
/// above it to the blank line below, with the id marker left off the last line.
fn block_of(body: &str, id: &str) -> Option<String> {
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
        return None;
    }
    let lines: Vec<&str> = body.lines().collect();
    let marker = format!("^{id}");
    let at = lines.iter().position(|l| {
        let t = l.trim_end();
        t == marker || t.ends_with(&format!(" {marker}"))
    })?;
    let mut lo = at;
    while lo > 0 && !lines[lo - 1].trim().is_empty() && !is_heading_line(lines[lo - 1]) && !is_heading_line(lines[lo]) {
        lo -= 1;
    }
    let mut hi = at;
    while hi + 1 < lines.len() && !lines[hi + 1].trim().is_empty() && !is_heading_line(lines[hi + 1]) && !is_heading_line(lines[hi]) {
        hi += 1;
    }
    let mut out: Vec<String> = lines[lo..=hi].iter().map(|s| s.to_string()).collect();
    let last = out.last_mut()?;
    let t = last.trim_end().to_string();
    *last = t.strip_suffix(&marker).unwrap_or(&t).trim_end().to_string();
    Some(out.join("\n"))
}

/// An ATX heading line (`#` to `######` then a space): a block of its own, so a paragraph never
/// runs into the heading above or below it.
fn is_heading_line(l: &str) -> bool {
    let t = l.trim_start();
    let n = t.bytes().take_while(|b| *b == b'#').count();
    (1..=6).contains(&n) && t[n..].starts_with(' ')
}

/// What an embed turns into. Every variant but [`Embed::Note`] and [`Embed::Image`] is a placeholder
/// the Preview draws with a reason; none of them is an error the user must dismiss.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Embed {
    /// A note (or a section of it). `path` is its vault path, pushed onto the chain by the caller
    /// before it draws the excerpt.
    Note { path: String, excerpt: Excerpt },
    /// An image inside the vault; read its bytes with [`VaultSession::read_image`]. `stamp` changes
    /// whenever the file does (`modified-seconds:length`), for the caller's texture cache.
    Image { path: String, stamp: String, width: Option<u32> },
    /// Nothing by that name.
    Missing { target: String },
    /// Several notes could mean it; nothing picks one.
    Ambiguous { target: String, candidates: Vec<String> },
    /// The note is already being drawn above this embed. `chain` is the notes in order, ending
    /// with `path`.
    Cycle { path: String, chain: Vec<String> },
    /// [`MAX_EMBED_DEPTH`] levels are already open.
    TooDeep { path: String, limit: usize },
    /// The note exists and the heading or block does not.
    FragmentMissing { path: String, fragment: String },
    /// A path that reaches outside the vault, or a file that resolves to somewhere outside it.
    Escape { target: String },
    /// Something Preview does not draw, with the reason in words.
    Unsupported { target: String, why: String },
    /// An image over [`IMAGE_MAX_BYTES`].
    TooLarge { path: String, bytes: u64, limit: u64 },
}

impl VaultSession {
    /// Resolves the embed whose inner text is `inner` (the text between `![[` and `]]`, or a Markdown
    /// image's address) as written in the note `from`, drawn inside the chain `stack` (the open note
    /// first). Reads at most one note and one file's metadata; writes nothing.
    ///
    /// Order, each step a refusal that ends the embed: an external address; a path that is not safely
    /// inside the vault ([`Embed::Escape`]); an image or other attachment; then a note, which is
    /// resolved like any link, gated against a cycle and the depth cap, read, and cut to its
    /// excerpt. A same-note embed (`![[#Heading]]`) is the note `from` itself, and so is a cycle
    /// unless `from` is not yet on `stack`.
    ///
    /// Errors only when no vault is bound or a note that resolved cannot be read.
    pub fn embed_open(&self, from: &str, inner: &str, stack: &[String]) -> Result<Embed, Error> {
        let v = self.bound()?;
        let p = parse(inner);
        if is_external(&p.target) {
            return Ok(Embed::Unsupported {
                target: p.target,
                why: "an address on the internet is never fetched here".into(),
            });
        }
        // A plain name has no separator, no `:` and is not `.` or `..`; anything else must pass the
        // vault's own containment rule before any path is built from it. `C:x.png`, `..` and a UNC
        // path all fail it, and so do `a/../../b`.
        let plain = !(p.target.contains(['/', '\\', ':']) || p.target == ".." || p.target == ".");
        if !p.target.is_empty() && !plain && !provider::is_safe_relative_path(&p.target) {
            return Ok(Embed::Escape { target: p.target });
        }
        let ext = ext_of(&p.target);
        if !ext.is_empty() && crate::notelinks::is_attachment(&p.target) {
            if !IMAGE_EXTENSIONS.contains(&ext.as_str()) {
                let why = if matches!(ext.as_str(), "gif" | "avif") {
                    format!("a .{ext} image is not drawn here (Godot cannot decode it)")
                } else {
                    format!("a .{ext} file is not drawn in a note's Preview")
                };
                return Ok(Embed::Unsupported { target: p.target, why });
            }
            for rel in candidates(from, &p.target) {
                if !provider::is_safe_relative_path(&rel) {
                    return Ok(Embed::Escape { target: p.target });
                }
                if v.exists(&rel) {
                    let Some(fs) = v.as_fs_vault() else {
                        return Ok(Embed::Unsupported {
                            target: p.target,
                            why: "this vault is not on the device's filesystem, so an image in it cannot be read yet".into(),
                        });
                    };
                    return Ok(match contained_file(fs, &rel) {
                        None => Embed::Escape { target: p.target },
                        Some(full) => {
                            let m = std::fs::metadata(&full).map_err(|e| Error::Vault(e.into()))?;
                            if m.len() > IMAGE_MAX_BYTES {
                                Embed::TooLarge { path: rel, bytes: m.len(), limit: IMAGE_MAX_BYTES }
                            } else {
                                let meta = v.meta(&rel)?;
                                Embed::Image { path: rel, stamp: format!("{}:{}", meta.modified, meta.len), width: p.width }
                            }
                        }
                    });
                }
            }
            return Ok(Embed::Missing { target: p.target });
        }
        // A note. `![[#Heading]]` names the open note itself.
        let path = if p.target.is_empty() {
            from.to_string()
        } else {
            match self.resolve_link(from, &p.target)? {
                Resolution::Unique(path) => path,
                Resolution::Ambiguous(candidates) => return Ok(Embed::Ambiguous { target: p.target, candidates }),
                Resolution::Unresolved => return Ok(Embed::Missing { target: p.target }),
            }
        };
        if let Err(g) = gate(stack, &path) {
            return Ok(match g {
                Gate::Cycle => {
                    let mut chain = stack.to_vec();
                    chain.push(path.clone());
                    Embed::Cycle { path, chain }
                }
                Gate::TooDeep => Embed::TooDeep { path, limit: MAX_EMBED_DEPTH },
            });
        }
        let text = v.read(&path)?;
        match excerpt(&text, &p.fragment, EXCERPT_MAX_CHARS) {
            Ok(ex) => Ok(Embed::Note { path, excerpt: ex }),
            Err(FragmentError::HeadingMissing(h)) => Ok(Embed::FragmentMissing { path, fragment: format!("#{h}") }),
            Err(FragmentError::BlockMissing(b)) => Ok(Embed::FragmentMissing { path, fragment: format!("#^{b}") }),
        }
    }

    /// The bytes of an image inside the vault, for Preview to draw. `rel` is a path
    /// [`Self::embed_open`] returned; it is checked again here, because this is the function that
    /// touches the disk: a safe relative path, a drawable extension, a file whose canonical path
    /// lies under the canonical vault root, and at most [`IMAGE_MAX_BYTES`].
    pub fn read_image(&self, rel: &str) -> Result<Vec<u8>, Error> {
        let v = self.bound()?;
        let bad = |why: &str| Error::Vault(std::io::Error::new(std::io::ErrorKind::InvalidInput, why.to_string()).into());
        if !provider::is_safe_relative_path(rel) {
            return Err(Error::Vault(provider::VaultError::Escapes(rel.to_string())));
        }
        if !is_drawable_image(rel) {
            return Err(bad("not an image Preview can draw"));
        }
        let fs = v.as_fs_vault().ok_or_else(|| bad("this vault is not on the device's filesystem"))?;
        let full = contained_file(fs, rel).ok_or_else(|| Error::Vault(provider::VaultError::Escapes(rel.to_string())))?;
        let len = std::fs::metadata(&full).map_err(|e| Error::Vault(e.into()))?.len();
        if len > IMAGE_MAX_BYTES {
            return Err(bad("the image is larger than Preview reads"));
        }
        std::fs::read(&full).map_err(|e| Error::Vault(e.into()))
    }
}

/// The real path of `rel` under `fs`, only if it still lies under the real vault root once symlinks
/// are followed. `None` for a path that does not exist, cannot be canonicalised, or leaves the root.
fn contained_file(fs: &provider::FsVault, rel: &str) -> Option<std::path::PathBuf> {
    let joined = fs.resolve(rel).ok()?;
    let real = joined.canonicalize().ok()?;
    let root = canonical_root(fs)?;
    is_under(&real, &root).then_some(real)
}

/// Whether the real path `real` lies inside the real folder `root`, compared by whole path components (so a
/// sibling folder that merely shares a prefix, `/v2` beside `/v`, is outside).
fn is_under(real: &Path, root: &Path) -> bool {
    real.starts_with(root)
}

/// The canonical vault root.
fn canonical_root(fs: &provider::FsVault) -> Option<std::path::PathBuf> {
    let root: &Path = fs.root();
    root.canonicalize().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Protects: the inner text splits into note, fragment and width exactly as Obsidian writes them,
    /// and an alias is not mistaken for a size.
    #[test]
    fn parse_splits_target_fragment_and_width() {
        let p = parse("Aldenmoor");
        assert_eq!((p.target.as_str(), p.fragment.clone(), p.width), ("Aldenmoor", Fragment::None, None));
        let p = parse("Aldenmoor#History");
        assert_eq!(p.fragment, Fragment::Heading("History".into()));
        let p = parse("Aldenmoor#Parent#Child|shown");
        assert_eq!(p.fragment, Fragment::Heading("Child".into()));
        assert_eq!(p.width, None, "an alias is not a size");
        let p = parse("Aldenmoor#^abc-1");
        assert_eq!(p.fragment, Fragment::Block("abc-1".into()));
        let p = parse("maps/pic.png|300");
        assert_eq!((p.target.as_str(), p.width), ("maps/pic.png", Some(300)));
        assert_eq!(parse("pic.png|300x200").width, Some(300));
        assert_eq!(parse("pic.png|0").width, None, "a zero width is no width");
        assert_eq!(parse("pic.png|300x").width, None);
        assert_eq!(parse("pic.png|12a").width, None);
        assert_eq!(parse("#Only").target, "");
    }

    /// Protects: a URL scheme is external; a drive letter and a plain name are not.
    #[test]
    fn external_means_a_scheme() {
        for s in ["https://x.test/a.png", "http://x", "data:image/png;base64,AA", "file:///etc/passwd", "mailto:a@b.c"] {
            assert!(is_external(s), "{s}");
        }
        for s in ["C:/a.png", "C:a.png", "a.png", "Notes/a:b.png", "x"] {
            assert!(!is_external(s), "{s}");
        }
    }

    /// Protects: which extensions are drawn; `gif` is an attachment Preview cannot decode.
    #[test]
    fn drawable_images_are_the_decodable_ones() {
        for s in ["a.png", "A.PNG", "x/y.jpg", "z.jpeg", "w.webp", "b.bmp", "s.svg"] {
            assert!(is_drawable_image(s), "{s}");
        }
        for s in ["a.gif", "a.avif", "a.pdf", "a", "a.png.txt", "png"] {
            assert!(!is_drawable_image(s), "{s}");
        }
    }

    /// Protects: an image is looked for beside the note first, then at the root.
    #[test]
    fn candidates_prefer_the_notes_own_folder() {
        assert_eq!(candidates("Places/Mar.md", "map.png"), vec!["Places/map.png", "map.png"]);
        assert_eq!(candidates("Mar.md", "img/map.png"), vec!["img/map.png"]);
        assert_eq!(candidates("a/b/c.md", "x\\y.png"), vec!["a/b/x/y.png", "x/y.png"]);
    }

    /// Protects: the cycle and depth rules, with literals. The cap is 3 embedded levels: a chain of
    /// the open note plus three embedded notes may not open a fourth.
    #[test]
    fn gate_stops_a_cycle_and_a_runaway_chain() {
        let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        assert_eq!(gate(&s(&["A.md"]), "B.md"), Ok(()));
        assert_eq!(gate(&s(&["A.md"]), "A.md"), Err(Gate::Cycle), "a self-embed");
        assert_eq!(gate(&s(&["A.md", "B.md"]), "A.md"), Err(Gate::Cycle), "A embeds B embeds A");
        assert_eq!(gate(&s(&["A.md", "B.md"]), "a.MD"), Err(Gate::Cycle), "ignoring case, as the resolver does");
        assert_eq!(gate(&s(&["A.md", "B.md", "C.md"]), "D.md"), Ok(()), "three levels open is still allowed");
        assert_eq!(gate(&s(&["A.md", "B.md", "C.md", "D.md"]), "E.md"), Err(Gate::TooDeep));
        assert_eq!(gate(&s(&["A.md", "B.md", "C.md", "D.md"]), "A.md"), Err(Gate::Cycle), "cycle wins over depth");
    }

    const NOTE: &str = "---\ntitle: x\n---\nIntro line.\n\n# One\nbody one\n## Sub\nsub body\n# Two\nbody two ^blk\n\nafter\n";

    /// Protects: the first section is the preamble when there is one, else the first heading's section;
    /// front matter is never shown.
    #[test]
    fn first_section_is_the_preamble_or_the_first_heading() {
        let e = excerpt(NOTE, &Fragment::None, 4000).unwrap();
        assert_eq!(e.text, "Intro line.");
        let e = excerpt("---\na: 1\n---\n# One\nbody one\n## Sub\nsub\n# Two\nt\n", &Fragment::None, 4000).unwrap();
        assert_eq!((e.text.as_str(), e.heading.as_deref()), ("# One\nbody one\n## Sub\nsub", Some("One")));
        assert_eq!(excerpt("", &Fragment::None, 10).unwrap().text, "");
    }

    /// Protects: a heading embed takes that section to the next of the same level, case-blind, and
    /// a missing heading is a named refusal.
    #[test]
    fn heading_fragment_takes_its_own_section() {
        let e = excerpt(NOTE, &Fragment::Heading("one".into()), 4000).unwrap();
        assert_eq!(e.text, "# One\nbody one\n## Sub\nsub body");
        let e = excerpt(NOTE, &Fragment::Heading("Sub".into()), 4000).unwrap();
        assert_eq!(e.text, "## Sub\nsub body");
        assert_eq!(excerpt(NOTE, &Fragment::Heading("Nope".into()), 4000), Err(FragmentError::HeadingMissing("Nope".into())));
    }

    /// Protects: a block embed takes the paragraph around ` ^id`, without the marker.
    #[test]
    fn block_fragment_takes_its_paragraph() {
        let e = excerpt(NOTE, &Fragment::Block("blk".into()), 4000).unwrap();
        assert_eq!(e.text, "body two");
        assert_eq!(excerpt(NOTE, &Fragment::Block("zzz".into()), 4000), Err(FragmentError::BlockMissing("zzz".into())));
        assert_eq!(excerpt(NOTE, &Fragment::Block("bl k".into()), 4000), Err(FragmentError::BlockMissing("bl k".into())));
        // Obsidian's block ids are letters, digits and dashes; anything else is not an id, even when the note spells it.
        assert_eq!(excerpt("para ^a.b\n", &Fragment::Block("a.b".into()), 4000), Err(FragmentError::BlockMissing("a.b".into())));
        assert_eq!(excerpt("para ^a-b\n", &Fragment::Block("a-b".into()), 4000).unwrap().text, "para");
    }

    /// Protects: the cap cuts at the character limit and says so; a section exactly at the cap is
    /// whole.
    #[test]
    fn excerpt_is_capped_and_says_so() {
        let e = excerpt("abcdefghij", &Fragment::None, 4).unwrap();
        assert_eq!((e.text.as_str(), e.truncated), ("abcd", true));
        let e = excerpt("abcd", &Fragment::None, 4).unwrap();
        assert_eq!((e.text.as_str(), e.truncated), ("abcd", false));
        let e = excerpt("ééééé", &Fragment::None, 2).unwrap();
        assert_eq!(e.text, "éé", "a cut never splits a character");
    }

    /// Makes a throw-away vault under the temp dir.
    fn vault(tag: &str, files: &[(&str, &[u8])]) -> (std::path::PathBuf, VaultSession) {
        let root = std::env::temp_dir().join(format!("cartalith-embed-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (rel, bytes) in files {
            let p = root.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, bytes).unwrap();
        }
        let mut s = VaultSession::default();
        s.connect(root.to_str().unwrap(), None).unwrap();
        (root, s)
    }

    /// Protects: every outcome of `embed_open` over a real folder, with literals: a note, a section, a
    /// missing target, a cycle (A embeds B embeds A), the depth cap, an image and an image that is
    /// not there.
    #[test]
    fn embed_open_answers_each_case() {
        let (root, s) = vault(
            "cases",
            &[
                ("A.md", b"a text\n"),
                ("B.md", b"# Sec\nb body\n"),
                ("Dir/C.md", b"c text\n"),
                ("Dir/pic.png", b"\x89PNG"),
                ("top.png", b"\x89PNG"),
                ("note.gif", b"GIF"),
            ],
        );
        let top = vec!["A.md".to_string()];
        match s.embed_open("A.md", "B", &top).unwrap() {
            Embed::Note { path, excerpt } => assert_eq!((path.as_str(), excerpt.text.as_str()), ("B.md", "# Sec\nb body")),
            o => panic!("{o:?}"),
        }
        assert!(matches!(s.embed_open("A.md", "Nope", &top).unwrap(), Embed::Missing { .. }));
        assert!(matches!(s.embed_open("A.md", "A", &top).unwrap(), Embed::Cycle { .. }), "a note embedding itself");
        let chain = vec!["A.md".to_string(), "B.md".to_string()];
        match s.embed_open("B.md", "A", &chain).unwrap() {
            Embed::Cycle { chain, .. } => assert_eq!(chain, vec!["A.md", "B.md", "A.md"]),
            o => panic!("{o:?}"),
        }
        let deep: Vec<String> = ["A.md", "B.md", "Dir/C.md", "x.md"].iter().map(|x| x.to_string()).collect();
        assert!(matches!(s.embed_open("A.md", "Dir/C", &deep).unwrap(), Embed::Cycle { .. }));
        assert!(matches!(s.embed_open("A.md", "B#Sec", &top).unwrap(), Embed::Note { .. }));
        assert!(matches!(s.embed_open("A.md", "B#Gone", &top).unwrap(), Embed::FragmentMissing { .. }));
        match s.embed_open("Dir/C.md", "pic.png|120", &["Dir/C.md".to_string()]).unwrap() {
            Embed::Image { path, width, .. } => assert_eq!((path.as_str(), width), ("Dir/pic.png", Some(120))),
            o => panic!("{o:?}"),
        }
        match s.embed_open("Dir/C.md", "top.png", &["Dir/C.md".to_string()]).unwrap() {
            Embed::Image { path, .. } => assert_eq!(path, "top.png", "falls back to the vault root"),
            o => panic!("{o:?}"),
        }
        assert!(matches!(s.embed_open("A.md", "gone.png", &top).unwrap(), Embed::Missing { .. }));
        assert!(matches!(s.embed_open("A.md", "note.gif", &top).unwrap(), Embed::Unsupported { .. }));
        assert!(matches!(s.embed_open("A.md", "https://x.test/a.png", &top).unwrap(), Embed::Unsupported { .. }));
        let _ = std::fs::remove_dir_all(root);
    }

    /// Protects: depth, with a real four-deep chain: the open note plus three embedded levels is the
    /// limit, so the fourth level is `TooDeep` and not a note.
    #[test]
    fn embed_open_stops_at_the_depth_cap() {
        let (root, s) = vault("deep", &[("n1.md", b"1"), ("n2.md", b"2"), ("n3.md", b"3"), ("n4.md", b"4"), ("n5.md", b"5")]);
        let st = |n: usize| (1..=n).map(|i| format!("n{i}.md")).collect::<Vec<_>>();
        assert!(matches!(s.embed_open("n3.md", "n4", &st(3)).unwrap(), Embed::Note { .. }), "level 3 opens");
        assert!(matches!(s.embed_open("n4.md", "n5", &st(4)).unwrap(), Embed::TooDeep { limit: 3, .. }), "level 4 does not");
        let _ = std::fs::remove_dir_all(root);
    }

    /// Protects: no path escape, with literals. Every one of these names a real file outside the
    /// vault, or a drive, a root or a UNC path, and none may be read.
    #[test]
    fn embed_open_and_read_image_refuse_every_escape() {
        let outside = std::env::temp_dir().join(format!("cartalith-embed-outside-{}.png", std::process::id()));
        std::fs::write(&outside, b"\x89PNG secret").unwrap();
        let (root, s) = vault("esc", &[("A.md", b"a"), ("ok.png", b"\x89PNG")]);
        let top = vec!["A.md".to_string()];
        let name = outside.file_name().unwrap().to_string_lossy().to_string();
        for bad in [
            format!("../{name}"),
            format!("..\\{name}"),
            format!("a/../../{name}"),
            outside.display().to_string(),
            "/etc/passwd.png".to_string(),
            "C:/Windows/x.png".to_string(),
            "C:x.png".to_string(),
            "\\\\server\\share\\x.png".to_string(),
            "//server/share/x.png".to_string(),
            "..".to_string(),
        ] {
            let r = s.embed_open("A.md", &bad, &top).unwrap();
            assert!(matches!(r, Embed::Escape { .. }), "{bad} -> {r:?}");
            assert!(s.read_image(&bad).is_err(), "read_image {bad}");
        }
        // A good one still works, and read_image returns its bytes.
        assert_eq!(s.read_image("ok.png").unwrap(), b"\x89PNG");
        assert!(s.read_image("A.md").is_err(), "not an image");
        assert!(s.read_image("nope.png").is_err());
        let _ = std::fs::remove_dir_all(root);
        let _ = std::fs::remove_file(outside);
    }

    /// Protects: containment is by whole components. A file in a sibling folder that shares the vault folder's
    /// name as a prefix, or one beside the vault, is outside; the root itself and anything under it is inside.
    #[test]
    fn is_under_compares_whole_components() {
        let r = Path::new("/data/vault");
        assert!(is_under(Path::new("/data/vault/a.png"), r));
        assert!(is_under(Path::new("/data/vault/sub/a.png"), r));
        assert!(!is_under(Path::new("/data/vault2/a.png"), r), "a sibling that shares a prefix");
        assert!(!is_under(Path::new("/data/a.png"), r));
        assert!(!is_under(Path::new("/other/vault/a.png"), r));
    }

    /// Makes `link` a symlink to the file `target`; `false` when this host or user may not create one (Windows
    /// needs Developer Mode or elevation), in which case the caller skips rather than pretend.
    fn try_symlink(target: &std::path::Path, link: &std::path::Path) -> bool {
        #[cfg(unix)]
        let r = std::os::unix::fs::symlink(target, link);
        #[cfg(windows)]
        let r = std::os::windows::fs::symlink_file(target, link);
        r.is_ok()
    }

    /// Protects: a symlink inside the vault that points out of it is refused, in the embed's answer and in
    /// `read_image`. This is the only test that reaches `contained_file`'s `starts_with` (everything else is
    /// stopped earlier by the lexical path rule), so it is skipped, said aloud, where a symlink cannot be made.
    #[test]
    fn a_symlink_out_of_the_vault_is_refused() {
        let outside = std::env::temp_dir().join(format!("cartalith-embed-link-{}.png", std::process::id()));
        std::fs::write(&outside, b"\x89PNG secret").unwrap();
        let (root, s) = vault("link", &[("A.md", b"a"), ("inside.png", b"\x89PNG")]);
        if !try_symlink(&outside, &root.join("evil.png")) {
            eprintln!("SKIPPED a_symlink_out_of_the_vault_is_refused: this host cannot create a symlink");
            let _ = std::fs::remove_dir_all(root);
            let _ = std::fs::remove_file(outside);
            return;
        }
        let top = vec!["A.md".to_string()];
        assert!(matches!(s.embed_open("A.md", "evil.png", &top).unwrap(), Embed::Escape { .. }));
        assert!(s.read_image("evil.png").is_err());
        assert!(matches!(s.embed_open("A.md", "inside.png", &top).unwrap(), Embed::Image { .. }), "a real file beside it still draws");
        let _ = std::fs::remove_dir_all(root);
        let _ = std::fs::remove_file(outside);
    }

    /// Protects: an image over the cap is `TooLarge`, one byte under is an image.
    #[test]
    fn a_huge_image_is_refused() {
        let (root, s) = vault("big", &[("A.md", b"a")]);
        let f = std::fs::File::create(root.join("big.png")).unwrap();
        f.set_len(IMAGE_MAX_BYTES + 1).unwrap();
        let g = std::fs::File::create(root.join("fits.png")).unwrap();
        g.set_len(IMAGE_MAX_BYTES).unwrap();
        let top = vec!["A.md".to_string()];
        assert!(matches!(s.embed_open("A.md", "big.png", &top).unwrap(), Embed::TooLarge { .. }));
        assert!(matches!(s.embed_open("A.md", "fits.png", &top).unwrap(), Embed::Image { .. }));
        assert!(s.read_image("big.png").is_err());
        assert_eq!(s.read_image("fits.png").unwrap().len() as u64, IMAGE_MAX_BYTES, "exactly the cap is read");
        let _ = std::fs::remove_dir_all(root);
    }
}
