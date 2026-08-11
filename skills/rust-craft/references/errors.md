# Errors

## Choosing the shape

The question is not "which crate" but "does my caller branch on this?"

A caller who handles `NotFound` differently from `PermissionDenied` needs an enum
with those variants. A caller who logs the message and moves on needs one opaque
type. Libraries usually face the first caller, applications the second — which is
the whole of the "thiserror for libraries, anyhow for applications" rule, and why
the rule bends whenever your situation does.

## Libraries: an enum per crate

`thiserror` writes the `Display` and `Error` impls from attributes. `#[from]`
generates the `From` impl that makes `?` convert automatically.

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config file not found: {0}")]
    NotFound(PathBuf),
    #[error("invalid syntax on line {line}")]
    Syntax { line: usize },
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
```

Keep the enum shallow and name variants for what went wrong, not for where. Mark
it `#[non_exhaustive]` so adding a variant later is not a breaking change.

Do not leak a dependency's error type in your public API. `Io(#[from]
std::io::Error)` is fine — `std` is stable — but re-exporting some crate's error
type ties your semver to theirs.

## Applications: one type, plus context

`anyhow::Result<T>` carries any error and a chain of context strings. The context
is the point: it turns "file not found" into the story of what you were trying
to do.

```rust
let config = std::fs::read_to_string(&path)
    .with_context(|| format!("reading config from {}", path.display()))?;
```

Use `with_context` (a closure) rather than `context` when building the message
allocates, so you only pay on the error path.

## When to panic

Panic for a broken invariant — something that indicates a bug in your own code,
which no caller could have avoided or handled:

- an index you have just bounds-checked
- a `match` arm the type system cannot prove unreachable
- a failed assertion about your own data structure

Return `Result` for everything that comes from outside: input, files, network,
parsing, user configuration. "The file was missing" is not a bug in your code.

In a library, panicking on bad input takes the choice away from the caller. In
`main`, panicking is a fine way to exit — the process was going to stop anyway.

## Preserving the cause

Implement `source()` (thiserror does it for you via `#[from]` and
`#[error(transparent)]`) so error chains stay walkable. A message that discards
its cause loses the only information that would have located the bug.

## Collecting fallible work

`collect` into `Result<Vec<_>, _>` stops at the first error:

```rust
let parsed: Result<Vec<Config>, _> = paths.iter().map(parse_config).collect();
```

To keep going and report every failure, partition instead of collecting — the
right call when validating user input, where reporting one error at a time makes
for a miserable loop.
