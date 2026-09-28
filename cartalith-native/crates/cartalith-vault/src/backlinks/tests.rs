//! `backlinks.rs`'s own tests.
//!
//! The filesystem half runs against a real temporary folder of real Markdown
//! files, because the whole design turns on `(modified, len)` being cheaper
//! than a read and a mocked provider would prove nothing about that.

use super::*;
use crate::provider::FsVault;

/// A fresh, process- and time-unique scratch directory under the system temp
/// dir for one test's fixture files.
fn tmp(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!(
        "cartalith_backlinks_{name}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Writes `text` to `rel` under `root`, creating parent directories as
/// needed, for building a fixture vault directly on disk.
fn write(root: &std::path::Path, rel: &str, text: &str) {
    let p = root.join(rel);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(p, text).unwrap();
}

// ------------------------------------------------------------------ parsing

#[test]
fn wikilinks_lose_their_alias_and_their_heading() {
    // Protects: `parse_links` strips a wikilink's `|alias` and `#heading`
    // down to the bare target, for all three wikilink shapes.
    let links = parse_links("see [[Kelvhold]], [[Kelvhold|the town]] and [[Nareth#History]].");
    let targets: Vec<&str> = links.iter().map(|l| l.target.as_str()).collect();
    assert_eq!(targets, vec!["Kelvhold", "Kelvhold", "Nareth"]);
    assert!(links.iter().all(|l| l.form == LinkForm::Wiki));
}

#[test]
fn markdown_links_are_taken_and_urls_are_not() {
    // Protects: `parse_links`/`is_note_path` accept a relative Markdown
    // link and a note-with-no-extension link, and reject a URL, a mailto,
    // an anchor-only link and an image.
    let links = parse_links(
        "[a](Settlements/Kelvhold.md) [b](../Notes/Thing) [c](https://example.com/x.md) \
         [d](mailto:someone@example.com) [e](#a-heading) [f](img/map.png)",
    );
    let targets: Vec<&str> = links.iter().map(|l| l.target.as_str()).collect();
    assert_eq!(targets, vec!["Settlements/Kelvhold.md", "../Notes/Thing"]);
    assert!(links.iter().all(|l| l.form == LinkForm::Markdown));
}

/// A `[[` with no `]]`, and a `[` with no `]`, must terminate rather than
/// scanning to the end of a large note — this runs over every file in a vault.
///
/// Protects: no hang/false-capture on either unterminated bracket form, and
/// that a real link after a stray `[[` is still found by restarting at the
/// inner opener.
#[test]
fn unterminated_brackets_do_not_hang_or_capture() {
    assert!(parse_links("[[never closed").is_empty());
    assert!(parse_links("[text( no close").is_empty());
    let mixed = parse_links("[[open [[Kelvhold]] more");
    assert_eq!(mixed.len(), 1);
    assert_eq!(mixed[0].target, "Kelvhold");
}

#[test]
fn entity_blocks_are_read_and_a_broken_one_is_not_an_error() {
    // Protects: `parse_entities` returns a well-formed block's entity key,
    // and returns nothing (rather than erroring) for an unterminated one —
    // the index must never refuse to open a vault over a malformed block.
    let good = format!(
        "# T\n\n{} entity=\"settlement:42\" version=\"1\" -->\nbody\n{}\n",
        block::BEGIN_PREFIX,
        block::END_MARKER
    );
    assert_eq!(parse_entities(&good), vec!["settlement:42".to_string()]);
    let broken = format!("# T\n\n{} entity=\"settlement:9\" -->\nno end\n", block::BEGIN_PREFIX);
    assert!(parse_entities(&broken).is_empty(), "a broken block must not stop the index");
}

// -------------------------------------------------------------- fingerprint

/// The one property the whole mention design rests on: a note that contains a
/// word is **never** filtered out. False positives are fine; false negatives
/// would make the feature quietly wrong.
///
/// Protects: no false negative for any of a set of distinct sample names,
/// singly or as a multi-word phrase requiring every token to be present.
#[test]
fn the_fingerprint_has_no_false_negatives() {
    let words = [
        "kelvhold", "nareth", "ostmere", "highfell", "verrun", "ashfoot", "sedge", "draumr",
        "veldmark", "korrath", "mirelle", "aurelia", "sythe",
    ];
    for w in words.iter() {
        let text = format!("The road from {w} was long and cold.");
        let bits = fingerprint_text(&text);
        let want = fingerprint(w);
        assert_eq!(bits & want, want, "{w} was filtered out of a note containing it");
    }
    // and a multi-word name, all of whose tokens must be present
    let text = "A charter of the Vale of Kelv, signed at Kelvhold.";
    let want = fingerprint("Vale of Kelv");
    assert_eq!(fingerprint_text(text) & want, want);
}

/// A name too short to fingerprint returns nothing rather than matching
/// everything — the difference between a bounded scan and an unbounded one.
///
/// Protects: `fingerprint` returns `0` for a too-short or empty name, and
/// `mention_candidates` returns no candidates for such a name rather than
/// the whole vault.
#[test]
fn a_name_with_no_usable_token_is_refused() {
    assert_eq!(fingerprint("Ai"), 0);
    assert_eq!(fingerprint("-"), 0);
    assert_eq!(fingerprint(""), 0);
    let idx = BacklinkIndex::new();
    assert!(idx.mention_candidates("Ai", &[]).is_empty());
}

/// No word can be read back out of a record. Asserted rather than asserted in
/// prose: the whole "never the prose" promise is one `u64`.
///
/// Protects: serialising a [`NoteRecord`] never emits any word from the
/// source text, while the fingerprint itself is non-zero.
#[test]
fn a_note_record_holds_no_prose() {
    let rec = parse_note("Kelvhold is a river town of four thousand souls, walled in stone.");
    let json = serde_json::to_string(&rec).unwrap();
    for word in ["Kelvhold", "river", "walled", "souls", "stone"] {
        assert!(!json.contains(word), "{word} survived into the stored record: {json}");
    }
    assert!(rec.word_bits != 0);
}

// ----------------------------------------------------------- the real index

#[test]
fn a_real_folder_indexes_backlinks_both_ways() {
    // Protects: a full `refresh` over a real folder finds both link forms,
    // counts multiple references from one source correctly, keeps each
    // source's own link form, and `mention_candidates` surfaces an
    // unlinked mention while excluding notes that already link properly.
    let root = tmp("both");
    write(&root, "Settlements/Kelvhold.md", "# Kelvhold\n\nA river town.\n");
    write(&root, "Factions/Veldmark.md", "Holds [[Kelvhold]] and [[Kelvhold|the town]].\n");
    write(&root, "People/Aldis.md", "Born in [a](Settlements/Kelvhold.md).\n");
    write(&root, "Journal/Thaw.md", "Rode down to Kelvhold before the river froze.\n");
    let vault = FsVault::new(&root);

    let mut idx = BacklinkIndex::new();
    let stats = idx.refresh(&vault, 500).unwrap();
    assert_eq!(stats.seen, 4);
    assert_eq!(stats.reread, 4, "a first build reads every note once");
    assert_eq!(stats.dropped, 0);
    assert!(idx.is_built());
    assert_eq!(idx.note_count(), 4);

    let back = idx.backlinks_to("Settlements/Kelvhold.md");
    let mut names: Vec<&str> = back.iter().map(|b| b.source.as_str()).collect();
    names.sort();
    assert_eq!(names, vec!["Factions/Veldmark.md", "People/Aldis.md"]);
    let veld = back.iter().find(|b| b.source.starts_with("Factions")).unwrap();
    assert_eq!(veld.count, 2, "two references from one source count as two");
    assert_eq!(veld.form, LinkForm::Wiki);
    let aldis = back.iter().find(|b| b.source.starts_with("People")).unwrap();
    assert_eq!(aldis.form, LinkForm::Markdown);

    // the journal mentions it and does not link it
    let linked: Vec<String> = back.iter().map(|b| b.source.clone()).collect();
    let cands = idx.mention_candidates("Kelvhold", &linked);
    assert!(
        cands.contains(&"Journal/Thaw.md".to_string()),
        "the unlinked mention is not a candidate: {cands:?}"
    );
    assert!(!cands.iter().any(|c| linked.contains(c)), "an excluded note came back");

    std::fs::remove_dir_all(&root).ok();
}

/// The whole point of the design: a second refresh over an unchanged folder
/// re-reads **nothing**, and one edited file costs exactly one read.
///
/// Protects: the `(modified, len)` change-detection basis itself -- a
/// second refresh over an unchanged vault reads zero files, and a change to
/// only the length (not necessarily the mtime) triggers exactly one reread
/// whose new links land in the index.
#[test]
fn a_refresh_reads_only_what_changed() {
    let root = tmp("incr");
    for i in 0..12 {
        write(&root, &format!("N{i}.md"), &format!("note {i} links [[N0]]\n"));
    }
    let vault = FsVault::new(&root);
    let mut idx = BacklinkIndex::new();
    assert_eq!(idx.refresh(&vault, 500).unwrap().reread, 12);
    assert_eq!(idx.refresh(&vault, 500).unwrap().reread, 0, "an unchanged vault costs no reads");

    // A changed *length* is enough — this test must not depend on the
    // filesystem's mtime resolution, which on some platforms is a second.
    write(&root, "N3.md", "note 3 links [[N1]] and [[N2]] now, and is longer than it was\n");
    let s = idx.refresh(&vault, 500).unwrap();
    assert_eq!(s.reread, 1, "one edited file, one read");
    assert_eq!(s.dropped, 0);
    assert_eq!(idx.backlinks_to("N1.md").len(), 1, "the new link is in the index");

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn a_deleted_note_leaves_the_index() {
    // Protects: a note removed from the vault is dropped from the index on
    // the next `refresh` (counted in `RefreshStats::dropped`), and a link
    // that pointed at it now shows up in `broken_links`.
    let root = tmp("del");
    write(&root, "A.md", "[[B]]\n");
    write(&root, "B.md", "b\n");
    let vault = FsVault::new(&root);
    let mut idx = BacklinkIndex::new();
    idx.refresh(&vault, 500).unwrap();
    assert_eq!(idx.note_count(), 2);
    std::fs::remove_file(root.join("B.md")).unwrap();
    let s = idx.refresh(&vault, 500).unwrap();
    assert_eq!(s.dropped, 1);
    assert_eq!(idx.note_count(), 1);
    // and A's link is now broken
    let broken = idx.broken_links();
    assert_eq!(broken.len(), 1);
    assert_eq!(broken[0], ("A.md".to_string(), "B".to_string()));
    std::fs::remove_dir_all(&root).ok();
}

/// An entity with no note of its own is still discoverable, which is the half
/// a note-to-note index alone would miss.
///
/// Protects: `notes_referencing_entity` finds the note carrying a Cartalith
/// block for the entity, returns nothing for an unrelated entity key, and
/// `entity_block_count` reflects the one block indexed.
#[test]
fn an_entity_is_found_through_a_block_in_someone_elses_note() {
    let root = tmp("entity");
    write(
        &root,
        "Provinces/Vale.md",
        &format!(
            "# Vale\n\n{} entity=\"settlement:118\" version=\"1\" -->\nrows\n{}\n",
            block::BEGIN_PREFIX,
            block::END_MARKER
        ),
    );
    write(&root, "Other.md", "nothing here\n");
    let vault = FsVault::new(&root);
    let mut idx = BacklinkIndex::new();
    idx.refresh(&vault, 500).unwrap();
    assert_eq!(idx.notes_referencing_entity("settlement:118"), vec!["Provinces/Vale.md"]);
    assert!(idx.notes_referencing_entity("settlement:9").is_empty());
    assert_eq!(idx.entity_block_count(), 1);
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn orphans_are_notes_nothing_points_at() {
    // Protects: `orphans` lists exactly the notes no other note links to,
    // by stem -- a linked leaf is excluded, an unlinked hub and an
    // unlinked lone note are both included.
    let root = tmp("orph");
    write(&root, "Hub.md", "[[Leaf]]\n");
    write(&root, "Leaf.md", "leaf\n");
    write(&root, "Lonely.md", "nobody links me\n");
    let vault = FsVault::new(&root);
    let mut idx = BacklinkIndex::new();
    idx.refresh(&vault, 500).unwrap();
    let mut o = idx.orphans();
    o.sort();
    assert_eq!(o, vec!["Hub.md".to_string(), "Lonely.md".to_string()]);
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn the_index_round_trips_through_json() {
    // Protects: `to_json`/`from_json` round-trip the whole index
    // byte-for-value, the reload is recognised as already built, and it
    // still costs zero rereads against an unchanged vault.
    let root = tmp("json");
    write(&root, "A.md", "[[B]] and [[C|see]]\n");
    write(&root, "B.md", "b\n");
    let vault = FsVault::new(&root);
    let mut idx = BacklinkIndex::new();
    idx.refresh(&vault, 500).unwrap();
    let round = BacklinkIndex::from_json(&idx.to_json()).unwrap();
    assert_eq!(round, idx);
    assert!(round.is_built());
    // and a reloaded index still costs no reads against an unchanged vault
    let mut round = round;
    assert_eq!(round.refresh(&vault, 500).unwrap().reread, 0);
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn clear_forces_a_full_rebuild() {
    // Protects: `clear` empties the index and resets `is_built` to false,
    // and the next `refresh` re-reads every note rather than trusting the
    // (now-gone) old state.
    let root = tmp("clear");
    write(&root, "A.md", "[[B]]\n");
    write(&root, "B.md", "b\n");
    let vault = FsVault::new(&root);
    let mut idx = BacklinkIndex::new();
    idx.refresh(&vault, 500).unwrap();
    idx.clear();
    assert!(!idx.is_built());
    assert_eq!(idx.note_count(), 0);
    assert_eq!(idx.refresh(&vault, 500).unwrap().reread, 2);
    std::fs::remove_dir_all(&root).ok();
}

/// A missing vault root is refused, not silently reported as an empty vault —
/// which would read on screen as "you have no notes".
///
/// Protects: `refresh` against a nonexistent root returns an error and
/// leaves the index unbuilt, rather than succeeding with zero notes.
#[test]
fn a_missing_root_is_an_error_not_an_empty_index() {
    let vault = FsVault::new(std::env::temp_dir().join("cartalith_no_such_vault_xyzzy"));
    let mut idx = BacklinkIndex::new();
    assert!(idx.refresh(&vault, 100).is_err());
    assert!(!idx.is_built());
}

/// A bare `[[Kelvhold]]` finds `Settlements/Kelvhold.md`, and a path link
/// finds it with or without the extension.
///
/// Protects: `target_matches`'s three resolution rules (exact path, path
/// without `.md`, bare-name-to-stem) each independently find the target,
/// and a similarly-named-but-different note (`Kelvhold Bridge`) does not.
#[test]
fn a_target_resolves_by_stem_and_by_path() {
    let root = tmp("resolve");
    write(&root, "Settlements/Kelvhold.md", "town\n");
    write(&root, "A.md", "[[Kelvhold]]\n");
    write(&root, "B.md", "[x](Settlements/Kelvhold.md)\n");
    write(&root, "C.md", "[[Settlements/Kelvhold]]\n");
    write(&root, "D.md", "[[Kelvhold Bridge]]\n");
    let vault = FsVault::new(&root);
    let mut idx = BacklinkIndex::new();
    idx.refresh(&vault, 500).unwrap();
    let mut src: Vec<String> =
        idx.backlinks_to("Settlements/Kelvhold.md").into_iter().map(|b| b.source).collect();
    src.sort();
    assert_eq!(src, vec!["A.md".to_string(), "B.md".to_string(), "C.md".to_string()]);
    std::fs::remove_dir_all(&root).ok();
}
