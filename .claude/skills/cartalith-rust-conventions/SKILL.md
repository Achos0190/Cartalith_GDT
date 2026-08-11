---
name: cartalith-rust-conventions
description: >
  The four Rust rules that are specific to THIS project and would be wrong
  advice anywhere else: float precision must match the JS engine rather than
  improve on it, floating-point operations must not be reordered without
  re-running golden-parity tests, NaN comparison differs between JS and Rust,
  and a panic crossing the gdext boundary can take down the Godot process. Use
  when writing or reviewing Rust in the cartalith-native workspace, choosing
  f32 vs f64, sorting or comparing float arrays, or writing anything callable
  from Godot. Trigger on "which float type", "is this safe to unwrap here",
  "gdext boundary", "parity test failing", or an unexplained golden-test
  failure after a refactor. For general Rust craft use `rust-craft`; for which
  crate code belongs in use `cartalith-porting-discipline`.
license: Original — authored for this project, no external source.
---

# Cartalith Rust conventions

Four skills divide this work; none repeats another:

- **`rust-craft`** — how to write good Rust anywhere.
- **`ponytail`** — whether to write it at all, and how little.
- **`cartalith-porting-discipline`** — which crate it goes in, and whether it is
  golden-parity verified.
- **This skill** — the handful of rules where this project's constraints
  override ordinary Rust practice.

Everything here follows from one fact: the engine's value is reproducing the JS
engine's numbers within a documented tolerance (`../../PARITY_TESTING.md`). That
makes some ordinary Rust improvements into bugs.

## Match the original precision. Do not improve it.

JS numbers are `f64`, but a value that round-trips through a `Float32Array` is
`f32` in practice — and this codebase's heightmap, climate, and flow fields are
all `Float32Array`. Check the reference file for each field rather than assuming.

Porting an `f32` field to `f64` because Rust makes it easy produces a value that
is *more accurate* and *fails the parity test*. For this project, matching beats
improving; raise a precision change as a decision, do not make it silently.

## Do not reorder float operations without re-running parity tests

Floating-point addition and multiplication are not associative, and LLVM
reorders under some optimisation settings. A refactor that looks purely
structural can move a parity test off its tolerance.

When a golden test fails after a change that "cannot" have affected the maths,
suspect operation order before suspecting logic.

## NaN compares differently in Rust than in JS

Both languages propagate NaN through arithmetic identically, so arithmetic is
not the hazard. Comparison is: in JS every comparison against NaN is simply
`false`, while in Rust `partial_cmp` returns `None` and the common
`sort_by(|a, b| a.partial_cmp(b).unwrap())` panics.

Any Rust code that sorts or compares a float array needs a stated NaN policy.
Read what the JS original assumes — usually "NaN cannot occur here, because X" —
and encode that assumption rather than picking a Rust default that quietly
changes behaviour.

## A panic must not cross the gdext boundary

Elsewhere, an unexpected panic ends a task. In `cartalith-godot`, a panic
unwinding through a GDExtension callback can take down the whole Godot process —
worse than the JS engine, where a thrown exception in one render call leaves the
app running.

Every `cartalith-godot` function callable from Godot converts errors from the
crates beneath it into something Godot can see: a return code, a signal, a
logged error. Confirm what `gdext` itself does about panics at the boundary when
you implement this — that behaviour is version-specific, and this file states
the risk rather than an API (`../../TOOLCHAIN.md`).

Inside the engine and subsystem crates, ordinary `rust-craft` error rules apply:
`Result` for what a caller can handle, a panic only for a broken invariant, and
each crate owning its own error type rather than sharing one workspace-wide enum.

## Tests

Golden-parity tests live in each subsystem crate's own `tests/` and run under
plain `cargo test` with no Godot dependency — that independence is what makes
them runnable in CI and in a headless session.

They verify end-to-end numbers, which is not a substitute for ordinary unit
tests. A bounds-clamping helper still deserves a test that it clamps at both
ends.
