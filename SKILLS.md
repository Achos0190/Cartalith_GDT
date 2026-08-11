# Skills for this project

Skills vendored into `skills/`, and skills researched but left where they are.
Everything below was checked by live search, including licence status for anything
copied into this repository.

## How installation works

A skill is a folder containing `SKILL.md` — YAML frontmatter (`name`,
`description`, optionally `license` and `argument-hint`) plus the instructions.
Claude Code loads any skill folder placed in a project's `.claude/skills/`.

**`Porting to Godot/skills/` is not a live skill directory.** Copy what you want
into the new repository's `.claude/skills/` when it exists — or into
`Cartalith_RC`'s own now, since nothing here has to wait for the port.

## Vendored

### `ponytail/` — write less

**[DietrichGebert/ponytail](https://github.com/DietrichGebert/ponytail)**, MIT
(verified; the notice travels in the vendored copy). Climb a ladder before writing
code: does this need to exist, is it already here, does the standard library do
it, does a native feature, does an installed dependency, will one line do — and
only then write something. Independent benchmarking measured −15% code, −10%
cost, −11% time on a real repository, more modest than the project's own claim but
real.

It fits because it formalises a rule the root `CLAUDE.md` already states: don't
add abstractions beyond what the task requires. A blank Rust workspace is exactly
where that discipline slips, since the JS engine's restraint came from growing one
measured change at a time.

Vendored is the core skill only; the upstream repo also ships `ponytail-audit`,
`-debt`, `-gain`, `-review`, and `-help`.

### `rust-craft/` — Rust, written after studying the field

Original prose. A lean `SKILL.md` covering what you decide in every function —
signatures, borrowing, iterators, errors, types that make bugs unrepresentable,
numeric overflow, unsafe, modules, docs, tests, the clippy loop — plus references
on errors, async, and performance, loaded only when their subject comes up.

Written under two constraints: Ponytail's ladder decided what to include, since a
long skill about writing less would refute itself; Strunk and White decided how to
phrase it.

`skills/rust-craft/ATTRIBUTION.md` records what was studied and rejected. In
short: [leonardomso/rust-skills](https://github.com/leonardomso/rust-skills) (MIT,
265 rules, through Rust 1.96) was the richest source;
[actionbook/rust-skills](https://github.com/actionbook/rust-skills) supplied the
edition-2024 defaults but its mandatory-router pattern was rejected — a gatekeeper
adds a hop without adding knowledge; [onsails/cc](https://github.com/onsails/cc)
supplied the split-at-500-lines heuristic.

**Unreachable, so unrepresented:** `mcpmarket.com`, `lib.rs`, `lobehub.com`, and
`composio.dev` are blocked by this session's egress proxy. Their picks are not
reflected — paste any you want folded in.

### `godot-shell/` — Godot 4.x as a shell

Original, deliberately narrow. Keeping logic out of GDScript, showing a generated
field as a texture (including `update()` over `create_from_image()`, which governs
regeneration performance), renderer choice, `Control`-node UI, the scene tree's
thread rules, what to commit — plus a reference on the `.gdextension` manifest and
the Windows/Android builds.

**Godot 4.x only.** Godot 4 replaced GDNative with GDExtension outright, so Godot
3 native-code guidance describes an API that no longer exists.

Narrow because every pack surveyed teaches game development, and this project has
no physics, animation, navigation, multiplayer, or game loop. The five-pack survey
is in `skills/godot-shell/ATTRIBUTION.md`.

### `cartalith-rust-conventions/` — only what this project overrides

Match the JS engine's float precision rather than improving it; don't reorder
float operations without re-running parity tests; state a NaN policy wherever
floats are sorted (JS comparison against NaN is `false`, Rust's
`partial_cmp().unwrap()` panics); keep panics off the gdext boundary, where they
can take down the Godot process.

Trimmed when `rust-craft` was written — its first draft carried general advice
that now lives there. Keeping both copies would have been the "two things
answering one question" failure the CHANGELOG keeps recording.

### `cartalith-porting-discipline/` — this port's own rules

Encodes `DECISIONS.md`, `MVP_SCOPE.md`, `ARCHITECTURE.md`, and `PARITY_TESTING.md`
as reflexes: which crate does this belong in, has it been parity-verified. Update
it when those documents change — a skill enforcing a stale rule is worse than none.

## Researched, not vendored

### Coding

- **Superpowers** — the largest community framework found (243,000+★): plan-first
  design, TDD, root-cause debugging, verification before completion. Its
  principles match this project's discipline closely enough to be worth real
  evaluation, but it is a multi-skill framework and that is a decision for whoever
  does the coding.
- **Anthropic's official skills repository** — the reference for how a well-formed
  skill is structured.
- **Focused single-purpose skills** — `code-reviewer`, `git-commit-writer`,
  `changelog-generator`, `pr-description-writer`, `env-doctor`. None essential; a
  changelog generator earns its place once the new repo's `CHANGELOG.md` is real.

### Godot

Full survey in `skills/godot-shell/ATTRIBUTION.md`.

- **[jame581/GodotPrompter](https://github.com/jame581/GodotPrompter)** (55 skills,
  MIT, Godot 4.3+) — **install this.** Its `gdextension`, `export-pipeline`,
  `mobile-development`, and `multithreading` skills are the four this project
  needs, and it is the only pack whose GDExtension coverage names Rust.
- **[vl4dt/godot-skills](https://github.com/vl4dt/godot-skills)** (12, MIT, 4.7) —
  game-oriented; GDExtension only inside the C# skill.
- **[alexmeckes/godot-claude-skills](https://github.com/alexmeckes/godot-claude-skills)**
  (5, MIT) — pairs with godot-mcp for live editor control.
- **[Randroids-Dojo/Godot-Claude-Skills](https://github.com/Randroids-Dojo/Godot-Claude-Skills)**
  (MIT) — GdUnit4 testing, CI/CD, web and desktop exports. Not Android.

### UX/UI

- **[nextlevelbuilder/ui-ux-pro-max-skill](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill)**
  (MIT, ~23k★) — **the first to install, at Phase 3.** Not vendored for a
  structural reason rather than a licensing one: it is a generator (CSV databases,
  Python, a CLI) and `npx skills add` builds the files for your platform. Copying
  a generated snapshot here would freeze a build artifact and cut it from
  upstream.
- **[szilu/ux-designer-skill](https://github.com/szilu/ux-designer-skill)** —
  ~11k lines across 24 references, WCAG 2.2 AA, design systems, cited sources.
  The deeper option; licence unconfirmed.
- **[ceorkm/mobile-app-ui-design](https://github.com/ceorkm/mobile-app-ui-design)**
  — compact (~320 lines): thumb zone, 8-point grid, 60/30/10 colour, peak-end
  design. Small enough to vendor, but **no LICENSE file was found**, so it is
  linked rather than copied.
- **[phazurlabs/ux-ui-mastery](https://github.com/phazurlabs/ux-ui-mastery)** and
  **[HermeticOrmus/LibreUIUX](https://github.com/HermeticOrmus/LibreUIUX-Claude-Code)**
  — both far larger than this project's UI surface warrants.

**One caveat across all of them:** every UI/UX skill found targets React, Next.js,
Tailwind, or SwiftUI. Godot is none of those. Their design reasoning transfers;
their code output does not.

### A caution about aggregators

Both skills inspected from
**[sickn33/antigravity-awesome-skills](https://github.com/sickn33/antigravity-awesome-skills)**
(`rust-pro`, `godot-gdscript-patterns`) carry `source: community` with **no
original author and no licence**. That is an unknown, not a permission, so nothing
was copied. Its `rust-pro` also targets Rust 1.75+, behind `rust-craft`'s
edition-2024 baseline, and its Godot skill is a 41-line pointer with no
GDExtension, texture, or export coverage.

Install from an aggregator if convenient; trace a skill to its author and licence
before vendoring it into a repository you own.

## What to install, when

Copy these five into the new repository's `.claude/skills/` at setup:

| Skill | Job |
|---|---|
| `ponytail` | whether to write it at all |
| `rust-craft` | how to write good Rust anywhere |
| `cartalith-rust-conventions` | where this project overrides that |
| `godot-shell` | Godot as the drawing and packaging layer |
| `cartalith-porting-discipline` | which crate, and is it verified |

They do not overlap, and each names which of the others owns an adjacent question.

Add **GodotPrompter** at the same time for Godot depth beyond the shell, and
**UI/UX Pro Max** at Phase 3, when the interface outgrows four controls.
