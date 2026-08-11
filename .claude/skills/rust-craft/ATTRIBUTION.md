# Attribution and sources

`rust-craft` is original prose, written for this project. It is not a copy of any
single source. The rules were selected and phrased after studying the material
below, cross-checked against my own knowledge of Rust, and cut down to what
earns its place.

## Sources studied

Reachable and read in full or in summary:

- **[leonardomso/rust-skills](https://github.com/leonardomso/rust-skills)** (MIT) —
  265 rules across 26 categories, current through Rust 1.96 / edition 2024. The
  most substantive source found. Its category structure (`own-`, `err-`, `api-`,
  `async-`, `anti-`) and its framing of the problem — *"out of the box, coding
  agents write average Rust — they clone to dodge the borrow checker, `.unwrap()`
  everything, and reach for `Box<dyn Trait>` when `impl Trait` would do"* — shaped
  this skill's scope and its description.
- **[actionbook/rust-skills](https://github.com/actionbook/rust-skills)** (MIT) —
  a router-plus-modules architecture over 40+ Rust skills. Its defaults
  (edition 2024, `rust-version = "1.85"`, clippy lints on by default) are
  reflected in the "New project defaults" section. Its mandatory-router pattern
  was deliberately **not** adopted: a gatekeeper skill that must be consulted
  before answering adds a hop without adding knowledge.
- **[onsails/cc](https://github.com/onsails/cc)** (MIT) — `rust-dev`, "strict Rust
  standards with FAIL FAST error handling." Source of the split-at-500-lines
  module heuristic. Its own `SKILL.md` returned 404 on direct fetch, so only the
  repository-level description informed this — noted rather than papered over.
- **Official documentation** — the Rust API Guidelines, the Rustonomicon, the
  Rust Performance Book, and the clippy lint index, consulted for the specific
  claims about `From`/`Into` blanket impls, `#[inline]` across crate boundaries,
  overflow behaviour by profile, and `clippy::pedantic` being off by default.

## Sources requested but unreachable

These were requested and could not be fetched — this session's network egress
proxy blocks the domains. Nothing from them is represented here, and no claim in
this skill should be attributed to them:

- `mcpmarket.com/tools/skills/rust-best-practices`
- `mcpmarket.com/tools/skills/rust-development-workflow`
- `mcpmarket.com/tools/skills/rust-developer-intelligence`
- `lib.rs/crates/claude-rust`
- `lobehub.com/pl/skills/laurigates-claude-plugins-rust-development`

If any of these carries a rule worth adopting, it can be folded in later — the
skill is structured so a new rule slots into an existing section rather than
requiring a rewrite.

## Style

Two influences, both about restraint:

**Ponytail** ([DietrichGebert/ponytail](https://github.com/DietrichGebert/ponytail),
MIT — vendored in `../ponytail/`) supplied the discipline for *what to include*.
Its ladder — does this need to exist, is it already here, will one line do —
applies to a skill's contents as much as to code. The result is a short
`SKILL.md` carrying the decisions made in every function, with three reference
files for subjects you reach only sometimes. A long skill about writing less
would refute itself.

**Strunk and White**, *The Elements of Style*, supplied the discipline for *how
to phrase it*: omit needless words; use the active voice; put statements in
positive form; use definite, specific, concrete language; express coordinate
ideas in parallel form. In practice this meant cutting hedges, giving each rule
its reason in a clause rather than a paragraph, and stating what to do before
what to avoid.

Both point the same way, which is why the skill reads as it does: rules in the
imperative, one short example where an example is worth more than a sentence, and
no prose defending a rule that a reader can simply follow.
