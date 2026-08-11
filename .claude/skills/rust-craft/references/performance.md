# Performance

## Measure first

Optimising unmeasured code wastes effort and usually makes the code worse. Profile
(`cargo flamegraph`, `perf`, `samply`), find the actual hot path, then read this
file. An intuition about which function is slow is wrong often enough that
checking is always cheaper than not checking.

Benchmark with `criterion`, which runs enough iterations to distinguish a real
change from noise. A single timed run measures your machine's mood.

Always profile a release build. Debug builds carry bounds checks, overflow
checks, and no inlining — their hot spots are not your hot spots.

## Allocation is usually the cost

Most Rust performance work is removing allocations from loops.

**Reserve when you know the size.** `Vec::with_capacity(n)` allocates once
instead of growing through log₂(n) reallocations, each copying everything.

**Reuse the buffer.** Hoist the allocation out and clear it per iteration:

```rust
let mut buf = String::with_capacity(256);
for record in records {
    buf.clear();
    write!(&mut buf, "{record}")?;
    sink.write_all(buf.as_bytes())?;
}
```

**Keep pipelines lazy.** An intermediate `.collect()` allocates a whole
collection to feed the next adaptor. Chain the adaptors instead and collect once
at the end.

**Freeze a finished Vec.** `into_boxed_slice()` drops the capacity field and the
ability to grow — a small win on memory, and a clear statement that the length
is now fixed.

**Avoid `format!` in a hot loop.** It allocates a fresh `String` every call.
`write!` into a reused buffer instead.

## Choose the collection for the access pattern

`Vec` beats `HashMap` for small n — often up to a few dozen entries — because a
linear scan of contiguous memory outruns hashing plus a pointer chase. `HashMap`
wins once n grows or lookups dominate.

`HashMap`'s default hasher is DoS-resistant, which costs speed. For keys that
never come from untrusted input, a faster hasher (`rustc-hash`, `ahash`) is a
one-line change with a real win.

Prefer contiguous layouts. Iterating a `Vec<Struct>` walks memory in order;
chasing a `Vec<Box<Struct>>` scatters it across the heap and defeats the
prefetcher.

## Inlining and codegen

Within a crate, LLVM inlines on its own. `#[inline]` matters mainly across crate
boundaries, where the compiler cannot see the body — so put it on small public
functions in a library, not on everything.

`#[inline(always)]` overrides the compiler's judgement. It is occasionally right
and usually not; measure both ways before keeping it.

## Release profile

```toml
[profile.release]
opt-level = 3
lto = "fat"           # cross-crate inlining; slower link
codegen-units = 1     # more optimisation, no parallel codegen
panic = "abort"       # no unwinding tables — only if you never catch_unwind
strip = true          # drop symbols from the binary
```

Each line trades compile time for run time, so apply them to release builds you
ship, not to the profile you iterate in. `panic = "abort"` also changes
behaviour, not just speed: it removes `catch_unwind` and any test that relies on
catching a panic.

## Bounds checks

Iterators eliminate bounds checks because the range is proven once. Indexing in a
loop re-checks each access. Where indexing is genuinely clearer, hoisting a slice
of the exact length often lets LLVM prove the check away:

```rust
let row = &grid[y * width..(y + 1) * width];   // checked once
for x in 0..width { use(row[x]); }             // provably in range
```

Reaching for `get_unchecked` to skip a bounds check is an `unsafe` decision. Do
it only with a profile showing the check matters and a `// SAFETY:` comment
proving the index is in range.
