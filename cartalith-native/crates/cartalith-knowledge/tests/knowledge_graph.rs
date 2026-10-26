//! RK-4's graph bars on a hand-built vault and world (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.8 RK-4):
//! the Vault's `links_to` / `linked_from` edges equal `BacklinkIndex::backlinks_to`, a dangling link
//! is not an edge, a note with an `entity=` block is joined to its world document in BOTH
//! directions, a private note has no edge at all, and expansion order puts the entity join first.
//!
//! The world half of RK-4 (trade, ownership, faith, government, province edges equal the getters)
//! runs on three REAL generated worlds in `cartalith-godot`'s `knowledge_bridge.rs` tests.
//!
//! The vault is written to a scratch directory and read back through `FsVault`, so the notes are
//! parsed by the real code. Nothing here is a model or a socket.

use std::collections::BTreeSet;
use std::path::PathBuf;

use cartalith_knowledge::search::{KnowledgeIndex, WorldSide};
use cartalith_knowledge::vault::{KNOWLEDGE_NOTE_CAP, VaultIndex};
use cartalith_knowledge::world::{FactionIn, SettlementIn, TradeFlowIn, WorldInput};
use cartalith_vault::{BacklinkIndex, FsVault};

fn scratch(tag: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("cartalith-knowledge-graph-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn put(root: &std::path::Path, rel: &str, text: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

fn world() -> WorldInput {
    let st = |id: i64, name: &str| SettlementIn { id, name: name.into(), kind: "town".into(), population: 100 * id as u32, faction: 1, ..Default::default() };
    WorldInput {
        seed: 3,
        settlements: vec![st(11, "Kelvhold"), st(12, "Rhenmark"), st(13, "Dunmere")],
        factions: vec![FactionIn { id: 1, name: "Order of Dawn".into(), culture: "nordic".into(), religion_label: "Old Ways".into(), religion_key: "animism".into(), government_key: "monarchy".into(), government_label: "Monarchy".into(), ag_tech: "iron".into(), ..Default::default() }],
        trade: Some(vec![TradeFlowIn { from: 11, to: 12, good: "grain".into(), mode: "land".into(), volume: 2.0, distance_km: 10.0 }]),
        ..Default::default()
    }
}

/// The vault: links in both forms, a dangling link, a link to a private note, an alias, an entity
/// block that joins a world settlement and one that names a settlement that does not exist.
fn vault(root: &std::path::Path) {
    put(root, "Places/Kelvhold.md", "---\naliases: [The Grey Port]\n---\n# Kelvhold\nHarbour. See [[Rhenmark]], [[Missing Place]], [[DM/Plot]] and [Dunmere](Places/Dunmere.md).\n\n<!-- CARTALITH:BEGIN entity=\"settlement:11\" version=\"1\" -->\nsummary\n<!-- CARTALITH:END -->\n");
    put(root, "Places/Rhenmark.md", "# Rhenmark\nBack to [[The Grey Port]] (an alias link only).\n");
    put(root, "Places/Dunmere.md", "# Dunmere\nMarsh near [[Kelvhold]]. <!-- CARTALITH:BEGIN entity=\"settlement:99\" version=\"1\" -->\nghost\n<!-- CARTALITH:END -->\n");
    put(root, "DM/Plot.md", "# Plot\nthe secret, links to [[Kelvhold]]\n");
}

fn build(tag: &str) -> (KnowledgeIndex, VaultIndex, PathBuf) {
    let root = scratch(tag);
    vault(&root);
    let mut vi = VaultIndex::new();
    vi.refresh(&FsVault::new(&root), KNOWLEDGE_NOTE_CAP).unwrap();
    let ws = WorldSide::from_input(&world());
    (KnowledgeIndex::build(Some(&ws), Some(&vi)), vi, root)
}

/// Protects RK-4's Vault bar: for every non-private note the set of notes linking to it, from this
/// crate's edges, equals `BacklinkIndex::backlinks_to` (the vault's own resolver) once the private
/// note is removed from the sources. Aliases are the one addition (`[[The Grey Port]]` reaches
/// Kelvhold only here), so that row is asserted separately. A dangling link is no edge.
#[test]
fn vault_link_edges_equal_the_backlink_index() {
    let (_, vi, root) = build("links");
    let mut bi = BacklinkIndex::new();
    bi.refresh(&FsVault::new(&root), KNOWLEDGE_NOTE_CAP).unwrap();
    let edges = vi.link_edges();
    for target in ["Places/Kelvhold.md", "Places/Rhenmark.md", "Places/Dunmere.md"] {
        let theirs: BTreeSet<String> = bi.backlinks_to(target).into_iter().map(|b| b.source).filter(|s| s != "DM/Plot.md").collect();
        let mut ours: BTreeSet<String> = edges.iter().filter(|(_, to)| to == target).map(|(from, _)| from.clone()).collect();
        if target == "Places/Kelvhold.md" {
            // Rhenmark links Kelvhold ONLY through the alias `[[The Grey Port]]`: this index follows
            // it, the backlink index (which resolves by path and stem only) does not.
            assert!(!theirs.contains("Places/Rhenmark.md"));
            assert!(ours.remove("Places/Rhenmark.md"), "the alias link is followed");
            assert_eq!(ours, theirs, "{target}");
        } else {
            assert_eq!(ours, theirs, "{target}");
        }
        assert!(!theirs.is_empty(), "the check must see real links: {target}");
    }
    assert!(!edges.iter().any(|(_, to)| to.contains("Missing")), "a dangling link is not an edge");
    assert!(!edges.iter().any(|(a, b)| a.starts_with("DM/") || b.starts_with("DM/")), "a private note has no edge");
    assert_eq!(edges.len(), 4, "{edges:?}");
    let _ = std::fs::remove_dir_all(&root);
}

/// Protects the entity join both ways: the world settlement's neighbours include the note whose
/// `entity="settlement:11"` block names it (read as `documented_by`), the note's neighbours include
/// the world document (read as `describes`), and a block naming a settlement with no document
/// (`settlement:99`) joins nothing.
#[test]
fn an_entity_block_joins_its_world_document_in_both_directions() {
    let (idx, _, root) = build("entity");
    let note_first = "V:Places/Kelvhold.md#Kelvhold";
    assert!(idx.get(note_first).is_some(), "the note's first chunk id");
    let from_world = idx.neighbours("W:settlement:11", Some("describes"), &[]);
    assert_eq!(from_world.iter().map(|r| (r.id.as_str(), r.relation.as_str())).collect::<Vec<_>>(), vec![(note_first, "documented_by")]);
    let from_note = idx.neighbours(note_first, Some("describes"), &[]);
    assert_eq!(from_note.iter().map(|r| (r.id.as_str(), r.relation.as_str())).collect::<Vec<_>>(), vec![("W:settlement:11", "describes")]);
    // The ghost entity: Dunmere's note has a block for settlement 99, which has no document.
    let dunmere_note = idx.neighbours("V:Places/Dunmere.md#Dunmere", Some("describes"), &[]);
    assert!(dunmere_note.is_empty(), "a block naming nothing joins nothing: {dunmere_note:?}");
    // Settlement 13 (Dunmere) has no block: no note is joined to it.
    assert!(idx.neighbours("W:settlement:13", Some("describes"), &[]).is_empty());
    let _ = std::fs::remove_dir_all(&root);
}

/// Protects expansion order: with a query hint naming trade, the entity join is STILL first (the note
/// that documents the settlement), then the hinted trade partner, then the rest. MUTATION: demote
/// `describes` below the hinted group and the first assert goes red.
#[test]
fn the_entity_join_comes_before_a_hinted_relation() {
    let (idx, _, root) = build("order");
    let n = idx.neighbours("W:settlement:11", None, &["trades_with"]);
    assert_eq!(n[0].base, "describes", "{n:?}");
    let trade_pos = n.iter().position(|r| r.base == "trades_with").expect("the trade partner is a neighbour");
    let owner_pos = n.iter().position(|r| r.base == "owned_by").expect("the owner is a neighbour");
    assert!(trade_pos < owner_pos, "a named relation sorts before an unnamed one: {n:?}");
    // The links between notes read forward and back.
    let rh = idx.neighbours("V:Places/Rhenmark.md#Rhenmark", Some("links_to"), &[]);
    let kinds: BTreeSet<&str> = rh.iter().map(|r| r.relation.as_str()).collect();
    assert!(kinds.contains("links_to") && kinds.contains("linked_from"), "{rh:?}");
    let _ = std::fs::remove_dir_all(&root);
}
