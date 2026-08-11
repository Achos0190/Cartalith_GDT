# Porting to Godot — directory instructions

Scoped to this folder. Supplements the repository root's `CLAUDE.md`, which governs
`Cartalith Gen1 v*.html` and is unrelated to this effort except as the source being
ported.

**This folder holds documentation and reference material only.** The Rust and Godot
project belongs in a new, separate repository (`DECISIONS.md` §8).

## Read `README.md` first

It carries the decision summary, the reading order, and the working discipline.
This file exists to load automatically and state the constraints below; it does
not replace reading `README.md` and `DECISIONS.md` properly.

## Constraints

- **Write no Rust or Godot project files here.** Starting the port means creating
  the new repository, not adding a `Cargo.toml` to `Cartalith_RC`.
- **Do not edit `reference/Cartalith Gen1 v2.10.html`.** It is the frozen snapshot
  every other document was written against. Re-freezing to a newer version is
  fine — regenerate `FUNCTION_INDEX.md` in the same pass, so the two never drift.
- **Do not deviate from `DECISIONS.md` silently.** Architecture decided before any
  code exists sometimes needs revision. Raise it, then record the new reasoning —
  the same way the HTML CHANGELOG discloses every deliberate re-baseline.
- **Expect these documents to age, and say so when they have.** Godot versions,
  gdext maturity, and crate specifics all move. Re-verify rather than trusting a
  version number written here.

## Contents

| Path | What it is |
|---|---|
| `README.md` | start here — decisions and reading order |
| `DECISIONS.md` | every choice, what it beat, and why |
| `MVP_SCOPE.md` | the first milestone's boundary and success criteria |
| `ARCHITECTURE.md` | the Rust↔Godot split and crate layout |
| `PROVENANCE.md` | sources, algorithms, formats; what must be hand-ported |
| `SAVEFILE_COMPAT.md` | the `.zip` format, verified against live code |
| `PARITY_TESTING.md` | golden-value testing against the JS engine |
| `REFERENCES.md` | external libraries and projects |
| `TOOLCHAIN.md` | setup, in order |
| `ROADMAP.md` | phases after the MVP |
| `SKILLS.md` | which skills to install, vendored or not, and why |
| `skills/ponytail/` | vendored anti-over-engineering skill (MIT) |
| `skills/rust-craft/` | general Rust craft, plus errors/async/performance references |
| `skills/cartalith-rust-conventions/` | only the rules this project overrides |
| `skills/godot-shell/` | Godot 4.x as a drawing and packaging shell |
| `skills/cartalith-porting-discipline/` | this port's own working rules |
| `reference/` | the frozen HTML snapshot and its function index |

## To start porting

1. Read `README.md`, `DECISIONS.md`, `MVP_SCOPE.md`, `ARCHITECTURE.md`.
2. Check whether the frozen snapshot is still current. If the repository root has
   moved on, decide with the owner whether to re-freeze — a newer version may have
   fixed something relevant.
3. Create the new repository and work through `TOOLCHAIN.md`'s Phase 0 before
   porting any engine logic.
4. Copy the five skills into that repository's `.claude/skills/`, and install
   GodotPrompter alongside them (`SKILLS.md`).
5. Copy `docs/research/` — at least the six documents `PROVENANCE.md` names.
   Constants without reachable derivations get "cleaned up" by someone who cannot
   see why they hold.
