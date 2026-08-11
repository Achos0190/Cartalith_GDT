---
name: cartalith-porting-discipline
description: >
  Enforces the working discipline for porting Cartalith Gen1's JS/Canvas
  procedural-generation engine to Rust + Godot. Use whenever porting a
  function or subsystem from the reference HTML file, writing or reviewing a
  crate boundary, writing a golden-parity test, adding a new crate to the
  workspace, or deciding whether something belongs in cartalith-engine vs a
  subsystem crate vs cartalith-godot. Also trigger on phrases like "port this
  function", "which crate should this go in", "is this parity-safe", "golden
  test", or "new subsystem crate". Do NOT use for work on the original HTML
  file itself (Cartalith_RC's own root CLAUDE.md governs that) or for
  general Rust/Godot questions unrelated to this specific port.
license: Original — authored for this project, no external source.
---

# Cartalith porting discipline

This project is porting `Cartalith Gen1 v*.html` (a ~200-version-deep, single-file
JS/Canvas/WebGL procedural world generator) to a Rust engine core inside Godot. The full
plan lives in the `Porting to Godot/` folder this skill ships alongside
(`DECISIONS.md`, `MVP_SCOPE.md`, `ARCHITECTURE.md`, `PARITY_TESTING.md`,
`SAVEFILE_COMPAT.md`, `ROADMAP.md`, `REFERENCES.md`) — read the relevant one before acting,
this skill is a set of reflexes, not a replacement for those documents.

## Before touching any port work: locate ground truth

1. **The reference HTML file is ground truth, not this skill's memory of it.** Use
   `reference/Cartalith Gen1 v*.html` (the frozen copy in this folder) and
   `reference/FUNCTION_INDEX.md` (a mechanically-generated name→line index — grep it or the
   HTML file directly rather than guessing where something lives). If the live
   `Cartalith_RC` repo has moved past the frozen reference version, say so explicitly rather
   than silently porting from a stale copy.
2. **Read the CHANGELOG entry for the function/subsystem you're about to port, if one
   exists** (root `CHANGELOG.md` in the main repo, or the frozen reference's own history via
   the root `CLAUDE.md`). This codebase's own CHANGELOG is unusually rich at explaining
   *why* a formula is what it is — many constants and thresholds were tuned across several
   owner-reported bugs, not chosen once and left alone. Porting the *current* formula
   without knowing *why* it looks the way it does is how a rewrite silently reintroduces a
   bug that was already fixed once.

## The ladder for "where does this code go"

Before writing a line of Rust, place it correctly (see `ARCHITECTURE.md` for the full
crate list and reasoning):

1. Is this pure generation/simulation logic (tectonics, height, climate, erosion,
   hydrology, noise, RNG)? → the matching subsystem crate (`cartalith-noise`,
   `cartalith-terrain`, `cartalith-climate`, `cartalith-erosion`, `cartalith-hydrology`).
2. Does it orchestrate/sequence subsystem crates, or own the `WorldState` struct? →
   `cartalith-engine`, and ONLY orchestration — if you're about to write a formula inside
   `cartalith-engine`, stop, it belongs in a subsystem crate instead.
3. Does it read/write the HTML app's save format? → `cartalith-io`.
4. Does it touch a Godot type, the scene tree, or GDExtension APIs at all? → the boundary
   layer only (`cartalith-godot`). No other crate may depend on `gdext` — if you're tempted
   to reach for a Godot type inside `cartalith-engine` or a subsystem crate, that's a sign
   the abstraction is leaking; stop and re-route the data instead.
5. Is it civ/politics, urban morphology, or asset-library logic? Those are **out of MVP
   scope** (`MVP_SCOPE.md`) — confirm this is actually the right phase before writing it at
   all (`ROADMAP.md`).

## Every ported formula needs a golden-parity check before it's considered done

Per `PARITY_TESTING.md`: porting a formula and eyeballing that the output "looks
reasonable" is not sufficient for this project. Before marking a port complete:

1. Does a golden test exist for this stage? If not, extract one from the reference HTML
   file (or from a real `.zip` save, per `SAVEFILE_COMPAT.md`) before writing more Rust on
   top of an unverified stage — verifying stage N with stage N−1 already unverified just
   compounds an unknown error.
2. Run the port against the golden data. A mismatch means: re-read the JS source for that
   exact stage again — check operation order, constant values, and whether an earlier stage
   (especially the RNG/noise primitives, see `PARITY_TESTING.md`'s own emphasis on this)
   is actually the real source of the drift, not the stage you're staring at.
3. **Never widen a tolerance to make a failing test pass without understanding why it
   failed first.** A tolerance exists to absorb genuine floating-point/language differences,
   not to paper over an actual formula mismatch. If a real, understood, deliberate deviation
   from the JS engine's exact behavior is being made, that's a decision to log (in the new
   repo's own `CHANGELOG.md`, the same way the HTML project's own CHANGELOG discloses every
   deliberate re-baseline), not a tolerance to quietly loosen.

## One subsystem at a time

Do not port tectonics, climate, erosion, and hydrology together and debug the combination
at the end. Finish and golden-verify one pipeline stage before starting the next — the
crate-per-subsystem structure in `ARCHITECTURE.md` exists specifically to make this the
natural way to work, not an extra discipline layered on top of it.

## When adding a genuinely new capability (not a straight port)

If a Rust-native rewrite naturally suggests a better approach than the JS engine's own
(different parallelism strategy, different internal data structure) — that's fine and
expected, per `ARCHITECTURE.md`'s note that "a rewrite is not required to be a literal
line-for-line translation." What's NOT fine is silently changing the *numerical result* of
something the golden-parity discipline is supposed to cover. Internal restructuring that
preserves output: proceed. Anything that changes output: flag it, explain why, get it
confirmed rather than assumed correct.

## Flag what can't be verified in this environment

Real GPU rendering, real Android touch input, real on-device performance, and real Windows
execution cannot be confirmed from a headless/cloud session. Say so explicitly in any status
report rather than claiming something "works" when only "compiles and passes headless
tests" has actually been confirmed — matching the carve-out the original HTML project's own
`CLAUDE.md` already documents for WebGL/Worker/touch interaction, extended here to native
builds (`DECISIONS.md` §5, `TOOLCHAIN.md`).
