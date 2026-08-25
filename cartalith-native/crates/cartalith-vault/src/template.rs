//! Creating a note from one of the author's own templates —
//! `GUI_GAP_REGISTER.md` **VA-02**.
//!
//! ## Why this is small, and why it must stay small
//!
//! `MARKDOWN_VAULT_SCOPE.md` milestone 1 drew a hard boundary: Cartalith
//! attaches to notes that already exist, and refuses a heading that does not.
//! That boundary was about *editing* — the machine block is the only thing
//! Cartalith rewrites unattended (§23), and a tool that reshapes an author's
//! prose is the failure mode the whole design is arranged against.
//!
//! Creating a file that was not there is a different act, and a safe one:
//! nothing is overwritten (a target that exists is refused outright), the
//! body is the author's own template copied verbatim, and the only thing
//! Cartalith substitutes is the entity's **name**. Everything else in the
//! template — `[Optional]`, `[If applicable]`, the prose placeholders the
//! owner's own README calls "a human overwrites" — is left exactly as
//! written. Filling those is `field_fill`'s job, behind its own preview, and
//! it stays there.
//!
//! ## Where templates come from
//!
//! **The vault, not this crate.** `design/vault-templates/` holds the owner's
//! real authoring templates, and they are *reference material for this
//! repository*, not content to ship: the integration targets a generic
//! Markdown vault, and a template registry compiled into the binary would be
//! Cartalith telling an author how to write their notes.
//!
//! So [`discover`] finds them the way the author already names them. Every
//! one of the owner's files says so in its filename — `Settlement
//! Template.md`, `Landmark template.md`, `Regional Overview.md` inside a
//! `Region Template/` folder — so a file is a template when "template"
//! appears in its **path**, case-insensitively. That is a convention, not a
//! guess, and it costs nothing: the walk it filters is `list_markdown`'s,
//! which reads directory entries and never opens a file.

/// One candidate template: its vault-relative path and the name to show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    pub rel: String,
    pub label: String,
}

/// The templates among `files` — see this module's own doc comment for why
/// the filename is the whole test. Order is `files`' order, which
/// `FsVault::list_markdown` has already sorted.
pub fn discover(files: &[String]) -> Vec<Template> {
    files
        .iter()
        .filter(|f| f.to_lowercase().contains("template"))
        .map(|f| Template { rel: f.clone(), label: label_for(f) })
        .collect()
}

/// `Region Template/Landmarks/Landmark template.md` -> `Landmarks ▸ Landmark
/// template`. The folder matters here in a way it does not elsewhere: the
/// owner's corpus has two byte-identical `Landmark template.md` files, one at
/// the root and one nested, and a picker that showed both as "Landmark
/// template" would be offering the same word twice.
fn label_for(rel: &str) -> String {
    let stem = rel.rsplit('/').next().unwrap_or(rel).trim_end_matches(".md").trim_end_matches(".MD");
    match rel.rsplit_once('/') {
        Some((dir, _)) => {
            let parent = dir.rsplit('/').next().unwrap_or(dir);
            format!("{parent} \u{25B8} {stem}")
        }
        None => stem.to_string(),
    }
}

/// The vault-relative path a new note for `kind` named `name` goes to —
/// v3's own `Settlements/{name}.md` convention, generalised to the four
/// entity kinds this port can address.
///
/// The folder is plural and capitalised because that is what the owner's own
/// vault looks like and what every Markdown-vault convention does; the file
/// stem is the entity's name with the characters a filesystem refuses
/// replaced by `-` rather than dropped, so two names that differ only in
/// punctuation do not collide.
pub fn suggested_path(kind: crate::EntityKind, name: &str) -> String {
    let folder = match kind {
        crate::EntityKind::Settlement => "Settlements",
        crate::EntityKind::Province => "Provinces",
        crate::EntityKind::Continent => "Continents",
        crate::EntityKind::Faction => "Factions",
    };
    format!("{folder}/{}.md", sanitise(name))
}

/// A filename stem that Windows, NTFS and POSIX all accept. Leading and
/// trailing dots and spaces go too — Windows silently strips a trailing dot
/// and the file would then not be at the path we recorded.
pub fn sanitise(name: &str) -> String {
    let mut out: String = name
        .chars()
        .map(|c| if matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') || (c as u32) < 0x20 { '-' } else { c })
        .collect();
    let trimmed = out.trim_matches(|c: char| c == '.' || c.is_whitespace()).to_string();
    out = trimmed;
    if out.is_empty() { "Untitled".to_string() } else { out }
}

/// The template's text with the entity's name substituted, and nothing else
/// touched.
///
/// Two placeholder conventions live in the owner's corpus and the README
/// separates them: `{{Landmark_Name}}`/`{{Region_Name}}` are Templater-style
/// substitutions, and `[Name]` is prose a human overwrites. Both name the
/// same thing — the title of the note — so both are filled, and **only**
/// where the placeholder is about the name:
///
/// - any `{{...Name}}` / `{{...name}}` token, whatever the prefix, and
/// - the exact literal `[Name]`.
///
/// `[If applicable]`, `[Optional]` and every other bracketed prompt survive
/// verbatim. They are instructions to the author, and rewriting them would
/// be Cartalith answering a question it was not asked.
pub fn fill_title(template: &str, name: &str) -> String {
    let mut out = String::with_capacity(template.len() + name.len());
    let bytes = template.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"{{") {
            if let Some(end) = template[i + 2..].find("}}") {
                let token = &template[i + 2..i + 2 + end];
                if token.to_lowercase().ends_with("name") {
                    out.push_str(name);
                    i += 2 + end + 2;
                    continue;
                }
            }
        }
        if bytes[i..].starts_with(b"[Name]") {
            out.push_str(name);
            i += 6;
            continue;
        }
        let ch = template[i..].chars().next().expect("i is a char boundary");
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EntityKind;

    #[test]
    fn discover_finds_the_owners_own_files_and_nothing_else() {
        let files: Vec<String> = [
            "Index.md",
            "Landmark template.md",
            "Locations/Nareth.md",
            "Region Template/Landmarks/Landmark template.md",
            "Region Template/Regional Overview.md",
            "Settlement Template.md",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let got = discover(&files);
        let rels: Vec<&str> = got.iter().map(|t| t.rel.as_str()).collect();
        assert_eq!(
            rels,
            [
                "Landmark template.md",
                "Region Template/Landmarks/Landmark template.md",
                "Region Template/Regional Overview.md",
                "Settlement Template.md",
            ],
            "case-insensitive, and a file inside a Template folder counts"
        );
        // The two identically-named landmark templates are distinguishable.
        assert_eq!(got[0].label, "Landmark template");
        assert_eq!(got[1].label, "Landmarks \u{25B8} Landmark template");
        assert_ne!(got[0].label, got[1].label);
    }

    #[test]
    fn suggested_paths_follow_v3s_convention_for_every_kind() {
        assert_eq!(suggested_path(EntityKind::Settlement, "Nareth"), "Settlements/Nareth.md");
        assert_eq!(suggested_path(EntityKind::Province, "Lower Vale"), "Provinces/Lower Vale.md");
        assert_eq!(suggested_path(EntityKind::Continent, "Vantharis"), "Continents/Vantharis.md");
        assert_eq!(suggested_path(EntityKind::Faction, "Draumr League"), "Factions/Draumr League.md");
    }

    #[test]
    fn a_name_that_is_not_a_filename_still_becomes_one() {
        assert_eq!(sanitise("Kel/Var: the Deep"), "Kel-Var- the Deep");
        assert_eq!(sanitise("  ..  "), "Untitled");
        assert_eq!(sanitise("Trailing."), "Trailing");
        assert_eq!(sanitise(""), "Untitled");
        // Two names differing only in punctuation must not collide.
        assert_ne!(sanitise("A:B"), sanitise("A?"));
    }

    /// The whole safety claim of this module in one test: the name goes in,
    /// the author's own prompts do not move.
    #[test]
    fn only_the_name_placeholders_are_filled() {
        let t = "## Settlement Profile: [Name]\n\n\
                 **Former Names:** [If applicable]\n\
                 **Era:** [Optional]\n\
                 # {{Landmark_Name}} and {{Region_Name}}\n\
                 keep {{date}} and {{title}} alone\n";
        let got = fill_title(t, "Nareth");
        assert!(got.starts_with("## Settlement Profile: Nareth\n"));
        assert!(got.contains("[If applicable]"), "author prompts survive");
        assert!(got.contains("[Optional]"));
        assert!(got.contains("# Nareth and Nareth"));
        assert!(got.contains("{{date}}"), "a non-name token is not ours to fill");
        assert!(got.contains("{{title}}"), "and neither is this one -- only ...Name");
        assert!(!got.contains("[Name]"));
    }

    #[test]
    fn fill_title_handles_multibyte_and_unclosed_tokens() {
        assert_eq!(fill_title("Ré{{Name}}ém", "X"), "RéXém");
        assert_eq!(fill_title("{{Name", "X"), "{{Name", "an unclosed token is left alone");
        assert_eq!(fill_title("", "X"), "");
    }
}
