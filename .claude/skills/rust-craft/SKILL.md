---
name: rust-craft
description: >
  Idiomatic Rust — ownership, error handling, iterators, API design, async,
  performance, unsafe, and the clippy/fmt/test workflow. Use this skill for ANY
  Rust work: writing a crate or a single function, reviewing or refactoring
  Rust, fixing a borrow-checker or lifetime error, choosing between String and
  &str or Box<dyn Trait> and impl Trait, designing a public API, picking an
  error type, adding a dependency, or setting up Cargo.toml. Also use when the
  user mentions Rust, cargo, clippy, crates.io, the borrow checker, lifetimes,
  traits, tokio or async, or unsafe — even when they only ask "does this look
  right?" Left alone, coding agents write average Rust: they clone to dodge the
  borrow checker, unwrap everything, and box what impl Trait would carry. This
  skill exists to stop that.
license: Original prose. Synthesized from MIT-licensed sources and official Rust documentation — see ATTRIBUTION.md.
---

# Rust craft

Average Rust compiles. Good Rust makes the compiler do the work: encode the
invariant in a type, borrow instead of copying, let the iterator drop the bounds
check. Every rule below earns its place by preventing a specific bug or a
specific allocation.

Read this file for the decisions you make in every function. Read a reference
file when you reach its subject:

| Reference | Read it when |
|---|---|
| `references/errors.md` | designing an error type, choosing thiserror vs anyhow, deciding whether to panic |
| `references/async.md` | writing async fns, spawning tasks, sharing state across `.await`, picking a channel |
| `references/performance.md` | a profiler says something is slow, or you are configuring a release build |

## Signatures first

The signature decides how much work your callers do. Take the loosest type that
does the job — deref coercion passes `&String` to `&str` for free, while the
reverse costs an allocation at every call site.

| To read | Take | Not |
|---|---|---|
| text | `&str` | `&String` |
| a sequence | `&[T]` | `&Vec<T>` |
| a path | `impl AsRef<Path>` | `String` |
| a value you keep | `T` or `impl Into<T>` | `&T`, then `.clone()` inside |

Return `impl Trait` when one type comes back, `Box<dyn Trait>` only when several
do. Boxing costs an allocation and a vtable hop, so pay it when you need the
choice, not by habit.

Implement `From`, never `Into`. The blanket impl gives you `Into` free, and `?`
converts errors through `From`.

```rust
impl From<io::Error> for ConfigError { /* ... */ }   // now `?` converts io::Error automatically
```

## Bodies: borrow, iterate, bind

**Borrow before you clone.** A `.clone()` added to quiet the borrow checker is
the most common Rust performance bug, and it hides the real question: what are
the two live borrows? Usually a narrower scope, a split borrow, or an index
resolves it. Clone when you genuinely need a second owner — and know that
cloning an `Rc` or `Arc` bumps a refcount rather than copying the data.

**Prefer iterators to index loops.** They drop the bounds check, they chain, and
they say what you mean.

```rust
let total: u64 = items.iter().filter(|i| i.active).map(|i| i.size).sum();
```

Keep the chain lazy. An intermediate `.collect()` in the middle of a pipeline
allocates a whole Vec to throw it away one line later.

**Bind early, nest little.** `let ... else` returns on the failure path and
leaves the happy path unindented. `matches!` tests a pattern without a `match`
block.

```rust
let Some(user) = lookup(id) else { return Err(NotFound(id)) };
```

**Look up once.** `HashMap::entry` replaces the `contains_key` + `insert` pair,
which hashes the key twice.

```rust
*counts.entry(word).or_insert(0) += 1;
```

## Errors

Return `Result` for anything a caller could reasonably handle; reserve panics
for broken invariants — a bug in your own code, not a bad input.

`unwrap()` and `expect()` belong in tests, in `main`, and behind an invariant you
have just proved in the surrounding lines. Everywhere else they convert a
recoverable error into a crash. When you do keep one, `expect("why this holds")`
documents the invariant and names the bug if it ever breaks.

Libraries define their own error enum so callers can match on it; applications
collapse everything into one type because they only log it. `thiserror` writes
the first, `anyhow` writes the second. See `references/errors.md`.

## Types that make bugs unrepresentable

**Newtype the meaningful values.** `struct UserId(u64)` and `struct OrderId(u64)`
stop the compiler from letting you swap them, and a bare `u64` never will.

**Parse, don't validate.** Turn the check into a type once, at the boundary,
instead of re-checking the same string in every function that receives it.

```rust
struct Email(String);
impl Email {
    fn parse(s: &str) -> Result<Self, InvalidEmail> { /* ... */ }
}
```

**Mark public enums `#[non_exhaustive]`** so adding a variant later stays a minor
release, and **mark builders and pure functions `#[must_use]`** so a dropped
result is a compile-time warning rather than a silent no-op.

## Ownership across threads

`Send` moves between threads, `Sync` shares by reference. `Arc<Mutex<T>>` is the
default shared-mutable pattern; `std::thread::scope` lets threads borrow from the
stack, which removes most reasons to reach for `Arc` at all.

```rust
std::thread::scope(|s| {
    for chunk in data.chunks_mut(1024) {
        s.spawn(|| process(chunk));   // borrows `data`, no Arc, no clone
    }
});
```

For data parallelism over a collection, `rayon` turns `iter()` into `par_iter()`
and handles the splitting. For async, the rules differ enough to deserve their
own file — see `references/async.md`.

## Numbers

Integer overflow panics in debug builds and wraps in release. That difference
bites when a test suite passes and production silently wraps, so state your
intent where overflow is possible: `checked_*` returns `Option`, `saturating_*`
clamps, `wrapping_*` wraps on purpose.

Floating point is not associative. Reordering `a + b + c` changes the result, so
treat any float computation whose exact output matters as order-sensitive.

## Unsafe

Reach for `unsafe` only after a safe design has actually failed, and keep the
block as small as the operation. Every `unsafe` block carries a `// SAFETY:`
comment naming the invariant that makes it sound — the comment is the proof, and
a block you cannot write one for is a block you do not understand yet.

Run `cargo miri test` over unsafe code. Miri catches undefined behaviour that
ordinary tests pass straight through.

## Modules and crates

Split a file when it starts doing two things; past roughly 500 lines, check
whether it already has. Prefer several small crates in a workspace to one large
crate — compilation parallelises, and a crate boundary is the clearest way to
say "this does not depend on that."

Add a dependency when it does something genuinely hard: a parser, a
cryptographic primitive, an async runtime. Write the ten lines yourself when it
is ten lines. Every dependency is a supply chain, a compile-time cost, and a
version you will one day have to bump.

## Documentation

Document public items with `///`, and put an example in it. Doc examples compile
and run under `cargo test`, so an example is a test that cannot rot silently.
Add `# Errors` when the function returns `Result`, `# Panics` when it can panic,
and `# Safety` when it is `unsafe` — these are the three things a caller cannot
infer from the signature.

## Tests

Unit tests live beside the code in `#[cfg(test)] mod tests`, where they can reach
private items. Integration tests live in `tests/` and see only the public API,
which makes them the honest check on whether that API is usable.

Structure each test as arrange, act, assert, and name it for the behaviour it
pins: `parse_rejects_empty_input`, not `test_parse_2`. Reach for `proptest` when
a property holds across a whole input range and enumerating cases by hand would
miss the interesting one.

## The loop before you call it done

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

Clippy is a teacher, not a nag: when it fires, read the lint's explanation before
silencing it. Suppress with a narrow `#[allow(...)]` and a comment saying why —
a crate-wide allow deletes the lint for code you have not written yet.

`clippy::pedantic` is worth reading through once for the education, but it is off
by default because it produces false positives; enable it lint by lint, not as a
group.

## New project defaults

```toml
[package]
edition = "2024"
rust-version = "1.85"      # edition 2024 requires it

[lints.clippy]
unwrap_used = "warn"       # in libraries; noisy and unnecessary in tests
```

Check the current stable Rust and edition before pinning these in a real project
— this file ages, and the toolchain does not stop moving.
