//! `notelinks.rs`'s own tests: link spans, the resolver, the shortest-link
//! writer and rename planning. Every expectation is a literal, written from
//! Obsidian's syntax rather than read back from this module's output.

use super::*;

/// A span as `(syntax, target, heading, block, alias)` — one readable row
/// per link instead of five assertions.
fn row(s: &LinkSpan) -> (&'static str, Option<&str>, Option<&str>, Option<&str>, Option<&str>) {
    (s.syntax.as_str(), s.target.as_deref(), s.heading.as_deref(), s.block.as_deref(), s.alias.as_deref())
}

/// The targets of every span in `text`, `None` for a same-note link.
fn targets(text: &str) -> Vec<Option<String>> {
    link_spans(text).into_iter().map(|s| s.target).collect()
}

/// `(path, text)` pairs from literals.
fn vault(files: &[(&str, &str)]) -> Vec<(String, String)> {
    files.iter().map(|(p, t)| (p.to_string(), t.to_string())).collect()
}

// ------------------------------------------------------------------- spans

/// Protects: every wikilink part lands in its own field -- target, heading,
/// `^block`, alias -- for a plain link, an aliased heading link, a block
/// link, an embed (whose `|300` is the alias slot) and a same-note heading
/// link (target `None`, not `""`).
#[test]
fn wikilink_parts_are_split_into_target_heading_block_and_alias() {
    let t = "see [[Kelvhold]], [[Kelvhold#History|the town]], [[Nareth#^abc123]], ![[map.png|300]] and [[#Local]].";
    let spans = link_spans(t);
    assert_eq!(
        spans.iter().map(row).collect::<Vec<_>>(),
        vec![
            ("wiki", Some("Kelvhold"), None, None, None),
            ("wiki", Some("Kelvhold"), Some("History"), None, Some("the town")),
            ("wiki", Some("Nareth"), None, Some("abc123"), None),
            ("embed", Some("map.png"), None, None, Some("300")),
            ("wiki", None, Some("Local"), None, None),
        ]
    );
}

/// Protects: CommonMark links -- a percent-encoded path and heading are
/// decoded, an angle-bracket destination keeps its spaces, a URL is
/// `external` and keeps its `#`, an image is `markdown_embed`, a title is
/// skipped, and an anchor-only link has no target.
#[test]
fn markdown_links_decode_their_target_and_keep_their_text() {
    let t = "[a](Settlements/Kelvhold%20Town.md#Old%20Quarter) [b](<Notes/Thing One.md>) [c](https://x.org/a#b) \
             ![m](img/map.png) [d](Note.md \"a title\") [e](#anchor)";
    let spans = link_spans(t);
    assert_eq!(
        spans.iter().map(row).collect::<Vec<_>>(),
        vec![
            ("markdown", Some("Settlements/Kelvhold Town.md"), Some("Old Quarter"), None, Some("a")),
            ("markdown", Some("Notes/Thing One.md"), None, None, Some("b")),
            ("markdown", Some("https://x.org/a#b"), None, None, Some("c")),
            ("markdown_embed", Some("img/map.png"), None, None, Some("m")),
            ("markdown", Some("Note.md"), None, None, Some("d")),
            ("markdown", None, Some("anchor"), None, Some("e")),
        ]
    );
    let external: Vec<bool> = spans.iter().map(|s| s.external).collect();
    assert_eq!(external, vec![false, false, true, false, false, false]);
}

/// Protects: offsets are Unicode scalar indices (what a GDScript `String`
/// indexes by), not bytes, and lines are 0-based. `é` is two bytes and `🜂`
/// four, so a byte-based offset would read 9, not 5.
#[test]
fn offsets_are_characters_and_lines_are_zero_based() {
    let t = "é🜂\n  [[Kelvhold]] x";
    let s = &link_spans(t)[0];
    assert_eq!((s.start, s.end, s.line), (5, 17, 1));
    assert_eq!(s.bytes, 9..21);
}

/// Protects: a link inside inline code, a backtick fence or a tilde fence
/// is an example, not a link.
#[test]
fn links_inside_code_are_not_links() {
    let t = "`[[A]]` and\n```\n[[B]]\n```\n~~~\n[x](C.md)\n~~~\n[[D]]";
    assert_eq!(targets(t), vec![Some("D".to_string())]);
}

/// Protects: the bracket edge cases terminate and read as an author sees
/// them -- an escaped opener is text, an unclosed `[[` is text, a wikilink
/// does not span lines, a stray `[[` before a real link yields the real one,
/// a failed `[[` still lets its second `[` open a Markdown link, and an
/// escaped `!` makes an embed a plain link.
#[test]
fn escaped_unclosed_and_multiline_brackets_are_text() {
    assert!(link_spans("\\[[x]]").is_empty());
    assert!(link_spans("[[never closed").is_empty());
    assert!(link_spans("[[a\nb]]").is_empty());
    assert_eq!(targets("[[open [[Kelvhold]] more"), vec![Some("Kelvhold".to_string())]);
    assert_eq!(link_spans("[[x](y.md)").iter().map(row).collect::<Vec<_>>(), vec![("markdown", Some("y.md"), None, None, Some("x"))]);
    assert_eq!(link_spans("\\![[x]]").iter().map(row).collect::<Vec<_>>(), vec![("wiki", Some("x"), None, None, None)]);
}

/// Protects: inside a Markdown table Obsidian writes the alias separator as
/// `\|`; the backslash belongs to the separator, so the target is clean and
/// a rename would rewrite exactly `Kelvhold`.
#[test]
fn a_table_pipe_alias_keeps_the_target_clean() {
    let t = "| [[Kelvhold\\|the town]] |";
    let s = &link_spans(t)[0];
    assert_eq!(row(s), ("wiki", Some("Kelvhold"), None, None, Some("the town")));
    assert_eq!(&t[s.target_bytes.clone()], "Kelvhold");
}

/// Protects: `link_at` finds the link under a caret -- inside it, at its
/// first character, just past its end -- picks the second of two adjacent
/// links at their shared boundary, and finds nothing outside.
#[test]
fn link_at_finds_the_link_under_the_caret() {
    let t = "ab [[One]][[Two]] cd";
    let at = |o: usize| link_at(t, o).and_then(|s| s.target);
    assert_eq!(at(4).as_deref(), Some("One"));
    assert_eq!(at(3).as_deref(), Some("One"));
    assert_eq!(at(10).as_deref(), Some("Two"));
    assert_eq!(at(17).as_deref(), Some("Two"));
    assert_eq!(at(1), None);
    assert_eq!(at(18), None);
}

/// Protects: outgoing links are notes only -- a URL, an attachment (by
/// Obsidian's accepted-format list) and a same-note link are left out,
/// while a note with a dot in its name stays in.
#[test]
fn outgoing_links_are_notes_only() {
    let t = "[[Kelvhold]] [[Mr.Smith]] ![[map.png]] [x](https://e.org) [y](Notes/Thing.md) [[#Local]] [z](doc.pdf)";
    let out: Vec<String> = outgoing_links(t).into_iter().filter_map(|s| s.target).collect();
    assert_eq!(out, vec!["Kelvhold", "Mr.Smith", "Notes/Thing.md"]);
}

/// Protects: the attachment test is the extension list, case-insensitive,
/// and a dot inside a note's name does not make it an attachment.
#[test]
fn attachments_are_recognised_by_extension_only() {
    assert!(is_attachment("map.png"));
    assert!(is_attachment("Folder/Doc.PDF"));
    assert!(!is_attachment("Mr.Smith"));
    assert!(!is_attachment("Notes/v1.2 notes"));
    assert!(!is_attachment("Kelvhold.md"));
    assert!(!is_attachment("Kelvhold"));
}

// --------------------------------------------------------------- resolving

/// The note paths most resolver tests share.
fn paths() -> Vec<String> {
    ["Settlements/Kelvhold.md", "Kelvhold Bridge.md", "Old/Nareth.md", "New/Nareth.md", "Index.md", "People/Aldis.md", "People/Bran.md"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Protects: the resolver's three steps in order -- exact path, path
/// without `.md`, bare name -- each case-insensitive, and an ambiguous bare
/// name reporting every candidate, sorted.
#[test]
fn the_resolver_tries_the_path_then_the_path_without_md_then_the_name() {
    let r = Resolver::new(paths());
    let u = |p: &str| Resolution::Unique(p.to_string());
    assert_eq!(r.resolve("Settlements/Kelvhold.md", "Index.md"), u("Settlements/Kelvhold.md"));
    assert_eq!(r.resolve("settlements/kelvhold", "Index.md"), u("Settlements/Kelvhold.md"));
    assert_eq!(r.resolve("Kelvhold", "Index.md"), u("Settlements/Kelvhold.md"));
    assert_eq!(r.resolve("kelvhold.md", "Index.md"), u("Settlements/Kelvhold.md"));
    assert_eq!(r.resolve("Kelvhold Bridge", "Index.md"), u("Kelvhold Bridge.md"));
    assert_eq!(r.resolve("Old/Nareth", "Index.md"), u("Old/Nareth.md"));
    assert_eq!(
        r.resolve("Nareth", "Index.md"),
        Resolution::Ambiguous(vec!["New/Nareth.md".to_string(), "Old/Nareth.md".to_string()])
    );
    assert_eq!(r.resolve("Missing", "Index.md"), Resolution::Unresolved);
}

/// Protects: the defect this resolver replaced -- a path link to the wrong
/// folder is unresolved (so it is broken AND not a backlink), never quietly
/// matched by file name.
#[test]
fn a_path_link_never_falls_back_to_a_file_name() {
    let r = Resolver::new(paths());
    assert_eq!(r.resolve("Wrong/Kelvhold", "Index.md"), Resolution::Unresolved);
    assert_eq!(r.resolve("Wrong/Kelvhold.md", "Index.md"), Resolution::Unresolved);
}

/// Protects: `./` and `../` resolve against the linking note's folder and
/// never climb above the vault root; a relative link that names no file is
/// not rescued by the name step; a leading `/` is the vault root.
#[test]
fn relative_links_resolve_against_the_linking_note() {
    let r = Resolver::new(paths());
    let u = |p: &str| Resolution::Unique(p.to_string());
    assert_eq!(r.resolve("../Settlements/Kelvhold.md", "People/Aldis.md"), u("Settlements/Kelvhold.md"));
    assert_eq!(r.resolve("./Bran", "People/Aldis.md"), u("People/Bran.md"));
    assert_eq!(r.resolve("../../x", "People/Aldis.md"), Resolution::Unresolved);
    assert_eq!(r.resolve("./Kelvhold", "People/Aldis.md"), Resolution::Unresolved);
    assert_eq!(r.resolve("/Index", "People/Aldis.md"), u("Index.md"));
}

/// Protects: an exact-case match wins a case-insensitive tie (Android and
/// Linux allow `Kel` beside `kel`), a target matching neither exactly is
/// ambiguous, and an empty target names the linking note itself.
#[test]
fn exact_case_wins_a_tie_and_an_empty_target_is_the_note_itself() {
    let r = Resolver::new(["a/Kel.md", "b/kel.md"]);
    assert_eq!(r.resolve("Kel", ""), Resolution::Unique("a/Kel.md".to_string()));
    assert_eq!(r.resolve("KEL", ""), Resolution::Ambiguous(vec!["a/Kel.md".to_string(), "b/kel.md".to_string()]));
    assert_eq!(r.resolve("", "a/Kel.md"), Resolution::Unique("a/Kel.md".to_string()));
    assert_eq!(r.resolve("", "zzz.md"), Resolution::Unresolved);
}

/// Protects: the shortest-link writer writes the bare name when it is
/// unique (what the editor's picker writes), and the folder path when the
/// name would resolve elsewhere or nowhere unique -- including the case
/// where a root-level note of the same name wins the bare name.
#[test]
fn the_shortest_link_is_the_name_when_unique_and_the_path_otherwise() {
    let r = Resolver::new(["Settlements/Aldenmoor.md", "Kelvhold.md", "Old/Kelvhold.md", "A/K.md", "B/K.md"]);
    assert_eq!(shortest_link_target("Settlements/Aldenmoor.md", "x.md", &r), "Aldenmoor");
    assert_eq!(shortest_link_target("Kelvhold.md", "x.md", &r), "Kelvhold");
    assert_eq!(shortest_link_target("Old/Kelvhold.md", "x.md", &r), "Old/Kelvhold");
    assert_eq!(shortest_link_target("A/K.md", "x.md", &r), "A/K");
}

// ---------------------------------------------------------------- renaming

/// Protects: a rename rewrites every link form to the renamed note --
/// wikilink, aliased, with a heading, embed, Markdown path, Markdown path
/// with a heading -- keeps aliases, headings and CRLF line endings, leaves
/// both kinds of code alone, splices correctly after a non-ASCII prefix, and
/// counts what it rewrote.
#[test]
fn a_rename_rewrites_every_form_and_keeps_aliases_headings_and_code() {
    let a = "Ærenfell road: [[Kelvhold]], [[Kelvhold|the town]], [[Kelvhold#History]], ![[Kelvhold]].\r\n\
             [x](Settlements/Kelvhold.md) and [y](Settlements/Kelvhold.md#Old%20Quarter).\r\n\
             `[[Kelvhold]]` stays.\r\n```\r\n[[Kelvhold]]\r\n```\r\n";
    let files = vault(&[("Settlements/Kelvhold.md", "# Kelvhold\n"), ("A.md", a)]);
    let plan = rename_plan("Settlements/Kelvhold.md", "Settlements/Kelvhold Town.md", &files).unwrap();
    assert_eq!(
        plan.files,
        vec![FileRewrite {
            path: "A.md".to_string(),
            text: "Ærenfell road: [[Kelvhold Town]], [[Kelvhold Town|the town]], [[Kelvhold Town#History]], ![[Kelvhold Town]].\r\n\
                   [x](Settlements/Kelvhold%20Town.md) and [y](Settlements/Kelvhold%20Town.md#Old%20Quarter).\r\n\
                   `[[Kelvhold]]` stays.\r\n```\r\n[[Kelvhold]]\r\n```\r\n"
                .to_string(),
            links: 6,
        }]
    );
    assert_eq!(plan.link_count(), 6);
    assert!(plan.ambiguous.is_empty());
}

/// Protects: the moved note's own relative links are rewritten from its new
/// folder (`../` deeper, `./Bran.md` becoming `../Bran.md`), a relative
/// wikilink stays relative, and a link by unique name is left alone.
#[test]
fn the_moved_note_keeps_its_relative_links_working() {
    let files = vault(&[
        ("People/Aldis.md", "[c](../Settlements/Kelvhold.md) [b](./Bran.md) [[Bran]] [[../Settlements/Kelvhold]]\n"),
        ("People/Bran.md", "b\n"),
        ("Settlements/Kelvhold.md", "k\n"),
    ]);
    let plan = rename_plan("People/Aldis.md", "People/Archive/Aldis.md", &files).unwrap();
    assert_eq!(
        plan.files,
        vec![FileRewrite {
            path: "People/Aldis.md".to_string(),
            text: "[c](../../Settlements/Kelvhold.md) [b](../Bran.md) [[Bran]] [[../../Settlements/Kelvhold]]\n".to_string(),
            links: 3,
        }]
    );
}

/// Protects: the collateral case -- a new name that collides with another
/// note's name would make links to THAT note ambiguous, so they are
/// rewritten to its path too, and the renamed note's links get its path.
#[test]
fn a_new_name_that_collides_rewrites_the_other_notes_links_too() {
    let files = vault(&[("Places/Kelvhold.md", "p\n"), ("Drafts/Old Town.md", "d\n"), ("A.md", "[[Kelvhold]] and [[Old Town]]\n")]);
    let plan = rename_plan("Drafts/Old Town.md", "Drafts/Kelvhold.md", &files).unwrap();
    assert_eq!(
        plan.files,
        vec![FileRewrite { path: "A.md".to_string(), text: "[[Places/Kelvhold]] and [[Drafts/Kelvhold]]\n".to_string(), links: 2 }]
    );
}

/// Protects: a link that was already ambiguous and may mean the renamed
/// note is reported (where, what, which candidates) and left alone, never
/// guessed at.
#[test]
fn an_ambiguous_link_is_reported_and_left_alone() {
    let files = vault(&[("A/K.md", ""), ("B/K.md", ""), ("N.md", "x\n[[K]]\n")]);
    let plan = rename_plan("A/K.md", "A/L.md", &files).unwrap();
    assert!(plan.files.is_empty());
    assert_eq!(
        plan.ambiguous,
        vec![AmbiguousLink {
            path: "N.md".to_string(),
            line: 1,
            target: "K".to_string(),
            candidates: vec!["A/K.md".to_string(), "B/K.md".to_string()],
        }]
    );
}

/// Protects: a move between folders leaves a link by unique name
/// byte-for-byte alone and rewrites the path links (with or without `.md`)
/// to the shortest form.
#[test]
fn a_folder_move_rewrites_only_the_links_that_would_break() {
    let files = vault(&[("A/K.md", "k\n"), ("N.md", "[[K]] [[A/K]] [[A/K.md]]\n")]);
    let plan = rename_plan("A/K.md", "B/K.md", &files).unwrap();
    assert_eq!(plan.files, vec![FileRewrite { path: "N.md".to_string(), text: "[[K]] [[K]] [[K]]\n".to_string(), links: 2 }]);
}

/// Protects: the renamed note's own self-link by name follows it (keyed by
/// the OLD path, which is the file the caller read), and its same-note link
/// is untouched.
#[test]
fn the_renamed_notes_own_self_link_follows_it() {
    let files = vault(&[("K.md", "see [[K#History]] and [[#Local]]\n")]);
    let plan = rename_plan("K.md", "L.md", &files).unwrap();
    assert_eq!(plan.files, vec![FileRewrite { path: "K.md".to_string(), text: "see [[L#History]] and [[#Local]]\n".to_string(), links: 1 }]);
}

/// Protects: a Markdown link keeps its style -- `<…>` stays unencoded, a
/// bare name without `.md` stays a bare name without `.md` (encoded), and a
/// vault-root `/` path keeps its slash.
#[test]
fn markdown_targets_keep_their_style() {
    let files = vault(&[
        ("Settlements/Kelvhold.md", ""),
        ("A.md", "[a](<Settlements/Kelvhold.md>) [b](Kelvhold) [c](/Settlements/Kelvhold.md)\n"),
    ]);
    let plan = rename_plan("Settlements/Kelvhold.md", "Settlements/Kelvhold Town.md", &files).unwrap();
    assert_eq!(
        plan.files,
        vec![FileRewrite {
            path: "A.md".to_string(),
            text: "[a](<Settlements/Kelvhold Town.md>) [b](Kelvhold%20Town) [c](/Settlements/Kelvhold%20Town.md)\n".to_string(),
            links: 3,
        }]
    );
}

/// Protects: every refusal, with its literal payload -- same path, missing
/// source, an escaping or drive-letter path, a non-note extension, each name
/// rule, and a case-insensitive collision -- while a case-only rename of the
/// note itself is allowed.
#[test]
fn a_rename_refuses_what_it_cannot_do_safely() {
    let files = vault(&[("Index.md", ""), ("K.md", "")]);
    let plan = |new: &str| rename_plan("K.md", new, &files).map(|_| ());
    assert_eq!(plan("K.md"), Err(RenameError::SamePath));
    assert_eq!(rename_plan("Nope.md", "X.md", &files).map(|_| ()), Err(RenameError::SourceMissing("Nope.md".to_string())));
    assert_eq!(plan("../x.md"), Err(RenameError::UnsafePath("../x.md".to_string())));
    assert_eq!(plan("C:/x.md"), Err(RenameError::UnsafePath("C:/x.md".to_string())));
    assert_eq!(plan("K.txt"), Err(RenameError::NotANote("K.txt".to_string())));
    assert_eq!(
        plan("K#1.md"),
        Err(RenameError::BadName { path: "K#1.md".to_string(), reason: "a link cannot reach a name containing # ^ [ or ]" })
    );
    assert_eq!(
        plan("Bad:Name.md"),
        Err(RenameError::BadName {
            path: "Bad:Name.md".to_string(),
            reason: "names cannot contain \\ : * ? \" < > | on Windows, where this vault may be opened"
        })
    );
    assert_eq!(
        plan("Folder./K.md"),
        Err(RenameError::BadName { path: "Folder./K.md".to_string(), reason: "Windows drops a trailing space or dot from a name" })
    );
    assert_eq!(
        plan("Folder/ .md"),
        Err(RenameError::BadName { path: "Folder/ .md".to_string(), reason: "the note needs a name before .md" })
    );
    assert_eq!(
        plan(".obsidian/K.md"),
        Err(RenameError::BadName {
            path: ".obsidian/K.md".to_string(),
            reason: "a name starting with . is hidden, so the note would vanish from every list"
        })
    );
    assert_eq!(plan("index.md"), Err(RenameError::TargetExists("index.md".to_string())));
    assert_eq!(plan("k.md"), Ok(()), "a case-only rename of the note itself is a rename, not a collision");
}

/// Protects: when a note changes folder, its relative links that name no
/// note -- an image, a note not written yet -- are re-pointed to reach the
/// same place (a Markdown target re-encoded, a wikilink written literally),
/// while a bare attachment link, the same kind of relative link in a note
/// that did not move, and a link by unique name are left alone.
#[test]
fn a_moved_notes_relative_attachment_links_still_reach_the_same_place() {
    let files = vault(&[
        ("People/Aldis.md", "![p](./img/aldis%20old.png) [[./Not Yet]] ![[map.png]]\n"),
        ("People/Bran.md", "![p](./img/bran.png) [[Aldis]]\n"),
    ]);
    let plan = rename_plan("People/Aldis.md", "People/Archive/Aldis.md", &files).unwrap();
    assert_eq!(
        plan.files,
        vec![FileRewrite {
            path: "People/Aldis.md".to_string(),
            text: "![p](../img/aldis%20old.png) [[../Not Yet]] ![[map.png]]\n".to_string(),
            links: 2,
        }]
    );
}

/// Protects: `rename_candidates` picks the notes the planner would touch
/// from link targets alone -- a path link that would break is in, a link by
/// unique name that survives a folder move is out, an unrelated note is
/// out, an already ambiguous link that may mean the renamed note is in, and
/// the renamed note is always in -- and it refuses exactly as the planner
/// does.
#[test]
fn rename_candidates_are_the_notes_the_planner_would_touch() {
    let p = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<String>>();
    let notes = [("N1.md", vec!["K"]), ("N2.md", vec!["A/K"]), ("N3.md", vec!["Elsewhere"])];
    let pick = |old: &str, new: &str, paths: &[String], notes: &[(&'static str, Vec<&'static str>)]| {
        rename_candidates(old, new, paths, &[], notes.iter().map(|(n, t)| (*n, t.iter().copied())))
    };
    assert_eq!(pick("A/K.md", "Z/K.md", &p(&["A/K.md", "N1.md", "N2.md", "N3.md"]), &notes), Ok(p(&["A/K.md", "N2.md"])));
    assert_eq!(pick("A/K.md", "A/L.md", &p(&["A/K.md", "B/K.md", "N4.md"]), &[("N4.md", vec!["K"])]), Ok(p(&["A/K.md", "N4.md"])));
    assert_eq!(pick("Nope.md", "X.md", &p(&["A/K.md"]), &notes), Err(RenameError::SourceMissing("Nope.md".to_string())));
}

/// Protects: a vault with nothing linking to the renamed note plans no
/// writes at all -- the plan is computed, and it is empty, not padded.
#[test]
fn a_rename_nothing_links_to_plans_no_writes() {
    let files = vault(&[("K.md", "alone\n"), ("Other.md", "[[Elsewhere]]\n")]);
    let plan = rename_plan("K.md", "L.md", &files).unwrap();
    assert_eq!((plan.from.as_str(), plan.to.as_str()), ("K.md", "L.md"));
    assert!(plan.files.is_empty() && plan.ambiguous.is_empty());
}

// ------------------------------------------------------------ folder renames

/// The vault the folder tests share: a folder with a nested folder, an
/// image beside a nested note, and a note outside that links in by path and
/// by name.
fn folder_vault() -> Vec<(String, String)> {
    vault(&[
        ("Places/Kelvhold.md", "Seat of the north.\n"),
        ("Places/Deep/Ford.md", "[k](../Kelvhold.md) [o](../../Journal/J.md) ![i](../img/x.png)\n"),
        ("Journal/J.md", "[[Places/Kelvhold]] [[Places/Deep/Ford|the ford]] [[Ford]]\n"),
    ])
}

fn paths_of(files: &[(String, String)]) -> Vec<String> {
    files.iter().map(|(p, _)| p.clone()).collect()
}

/// Protects: moving a folder rewrites the links INTO it exactly as a note
/// rename would -- a path link that would break is rewritten (to the
/// shortest unique name, alias kept), a link by unique name is not touched
/// -- and the plan lists every note that moves, nested ones included, in
/// path order.
#[test]
fn a_folder_rename_rewrites_links_into_it_and_lists_every_note_that_moves() {
    let files = folder_vault();
    let plan = folder_rename_plan_in("Places", "Realms", &paths_of(&files), &[], &files).unwrap();
    assert_eq!(
        plan.moves,
        vec![
            ("Places/Deep/Ford.md".to_string(), "Realms/Deep/Ford.md".to_string()),
            ("Places/Kelvhold.md".to_string(), "Realms/Kelvhold.md".to_string()),
        ]
    );
    let j = plan.files.iter().find(|f| f.path == "Journal/J.md").expect("the linking note is rewritten");
    assert_eq!(j.text, "[[Kelvhold]] [[Ford|the ford]] [[Ford]]\n");
    assert_eq!(j.links, 2, "the by-name link was left alone");
    assert_eq!((plan.from.as_str(), plan.to.as_str()), ("Places", "Realms"));
}

/// Protects: links OUT of a moving folder. When the folder goes one level
/// deeper, a relative link to a note outside it is re-pointed so it still
/// reaches that note; a relative link to a note inside the same folder, and
/// to an image beside it (which moves too), stay byte for byte.
#[test]
fn a_folder_moved_deeper_keeps_its_links_out_and_its_internal_links() {
    let files = folder_vault();
    let plan = folder_rename_plan_in("Places", "Atlas/Places", &paths_of(&files), &[], &files).unwrap();
    let ford = plan.files.iter().find(|f| f.path == "Places/Deep/Ford.md").expect("the nested note is rewritten");
    assert_eq!(ford.text, "[k](../Kelvhold.md) [o](../../../Journal/J.md) ![i](../img/x.png)\n");
    assert_eq!(ford.links, 1, "only the link out of the folder changed");
    assert!(plan.files.iter().all(|f| f.path != "Places/Kelvhold.md"), "a note with no links is not rewritten");
}

/// Protects: the folder candidates are every note inside it, plus notes
/// whose written links would change, and nothing else -- an unrelated note
/// and a link by unique name stay out.
#[test]
fn folder_candidates_are_the_moving_notes_and_the_notes_that_link_in() {
    let files = folder_vault();
    let paths = paths_of(&files);
    let notes = [("Journal/J.md", vec!["Places/Kelvhold"]), ("Other.md", vec!["Elsewhere", "Ford"])];
    let got = folder_rename_candidates("Places", "Realms", &paths, &[], notes.iter().map(|(n, t)| (*n, t.iter().copied())));
    assert_eq!(got, Ok(vec!["Journal/J.md".to_string(), "Places/Deep/Ford.md".to_string(), "Places/Kelvhold.md".to_string()]));
}

/// Protects: every folder refusal names its reason before anything is
/// planned -- a folder into itself and into its own descendant, a name that
/// already holds notes, a folder with no notes, the same path, a path out of
/// the vault and a name Windows cannot hold -- and a case-only rename is NOT
/// refused (`Places` to `places` is a real rename on a case-insensitive disk).
#[test]
fn a_folder_rename_refuses_itself_a_taken_name_and_a_bad_path() {
    let mut files = folder_vault();
    files.push(("Atlas/Old.md".to_string(), "x\n".to_string()));
    let paths = paths_of(&files);
    let try_it = |a: &str, b: &str| folder_rename_plan_in(a, b, &paths, &[], &files).map(|_| ());
    let itself = |a: &str, b: &str| Err(RenameError::IntoItself { from: a.to_string(), to: b.to_string() });
    assert_eq!(try_it("Places", "Places/Sub/Places"), itself("Places", "Places/Sub/Places"));
    assert_eq!(try_it("Places", "places/Deep"), itself("Places", "places/Deep"), "case-insensitively");
    assert_eq!(try_it("Places/Deep", "Places"), itself("Places/Deep", "Places"), "out over its own ancestor");
    assert_eq!(try_it("Places", "Atlas"), Err(RenameError::FolderExists("Atlas".to_string())));
    assert_eq!(try_it("Nowhere", "Z"), Err(RenameError::FolderMissing("Nowhere".to_string())));
    assert_eq!(try_it("Places", "Places"), Err(RenameError::SamePath));
    assert_eq!(try_it("Places", "../Out"), Err(RenameError::UnsafePath("../Out".to_string())));
    assert!(matches!(try_it("Places", "Bad:Name"), Err(RenameError::BadName { .. })));
    assert_eq!(try_it("Places", "places"), Ok(()), "a case-only rename is allowed");
    let msg = try_it("Places", "Places/Sub").unwrap_err().to_string();
    assert!(msg.contains("into itself"), "{msg}");
}

// ------------------------------------------------- attachments in a folder move

/// The attachments of a fixture, from path literals.
fn atts(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

/// Plans moving `Places` to `Realms` over `files` with `attachments` listed,
/// returning each rewritten note's new text keyed by path.
fn moved_texts(files: &[(&str, &str)], attachments: &[&str]) -> Vec<(String, String)> {
    let v = vault(files);
    let plan = folder_rename_plan_in("Places", "Realms", &paths_of(&v), &atts(attachments), &v).unwrap();
    plan.files.into_iter().map(|f| (f.path, f.text)).collect()
}

/// Protects: an outside note's embeds of a picture inside the moved folder
/// are re-pointed -- the path-qualified wiki embed, the vault-absolute and
/// the relative markdown image -- the wiki embed regenerated shortest (as a note link is: the name is unique, so it goes bare) and the markdown paths each keeping their own form, with the alias
/// (`|300`) and alt text untouched.
/// MUTATION: skip attachment targets in `plan_with` (`|| is_attachment(t)`)
/// and every assertion here fails.
#[test]
fn an_outside_note_embedding_a_picture_in_a_moved_folder_is_rewritten() {
    let got = moved_texts(
        &[
            ("Places/K.md", "Seat.\n"),
            ("Places/img/x.png", ""),
            ("Journal/J.md", "![[Places/img/x.png|300]] ![a](/Places/img/x.png) ![b](../Places/img/x.png)\n"),
        ],
        &["Places/img/x.png"],
    );
    assert_eq!(
        got,
        vec![(
            "Journal/J.md".to_string(),
            "![[x.png|300]] ![a](/Realms/img/x.png) ![b](../Realms/img/x.png)\n".to_string()
        )]
    );
}

/// Protects: a bare-name embed that still resolves to the one file after the
/// move is not touched (the planner's own keep-it-bare rule), and a link
/// inside the moved set -- note and picture move together -- is left
/// byte-for-byte alone.
#[test]
fn a_bare_embed_stays_bare_and_an_inside_link_is_untouched() {
    let got = moved_texts(
        &[
            ("Places/K.md", "![[x.png]] ![i](img/x.png) ![j](./img/x.png)\n"),
            ("Places/img/x.png", ""),
            ("Journal/J.md", "![[x.png]]\n"),
        ],
        &["Places/img/x.png"],
    );
    assert!(got.is_empty(), "nothing needs rewriting: {got:?}");
}

/// Protects: the writer of a rewritten attachment link. A bare name that
/// names two files after the move cannot stay bare, so a wiki embed is
/// path-qualified and a markdown image's bare name falls back to the path
/// (no `.md` is ever appended to a picture). A folder move alone cannot make
/// a bare name ambiguous (names do not change), so this drives the writer
/// directly with the resolver it would be handed.
/// MUTATION: drop the `att` branch in `rewritten_target` and the markdown
/// case comes back as `x.png.md`'s sibling or the wrong candidate.
#[test]
fn an_ambiguous_bare_attachment_name_is_written_path_qualified() {
    let r = Resolver::new(["A/x.png", "B/x.png", "J.md"]);
    let wiki = link_spans("![[x.png]]").remove(0);
    assert_eq!(rewritten_target(&wiki, "x.png", "B/x.png", "J.md", &r), "B/x.png");
    let md = link_spans("![](x.png)").remove(0);
    assert_eq!(rewritten_target(&md, "x.png", "B/x.png", "J.md", &r), "B/x.png");
    // Unique: a bare markdown name stays bare, with no `.md` added.
    let r = Resolver::new(["B/x.png", "J.md"]);
    assert_eq!(rewritten_target(&md, "x.png", "B/x.png", "J.md", &r), "x.png");
}
