# Toolchain

Everything needed before writing Rust or Godot code. Written for the cloud
session that builds (`DECISIONS.md` §5); applies equally to a local setup.

**Verify every version before pinning it.** This document was written against a
knowledge cutoff, and Godot, `gdext`, `cargo-ndk`, `cargo-xwin`, and the NDK all
move. Where a version appears below, read it as "current when written," not as a
pin. Same discipline the HTML project enforces about stale assumptions.

## Rust

Install through `rustup`, not a distro package — you need targets on demand.

```bash
rustup target add x86_64-pc-windows-msvc     # Windows, via cargo-xwin
rustup target add aarch64-linux-android      # the target real devices use
rustup target add armv7-linux-androideabi
rustup target add x86_64-linux-android
rustup target add i686-linux-android
```

The last three cover older hardware and emulators. Check `gdext`'s own
minimum-supported Rust version at setup time.

## Godot

Install the latest stable Godot 4.x (`DECISIONS.md` §9) and download the matching
**export templates** — no platform export works without them.

Godot ships a **headless build** for CI-style exports without a display. Confirm
it can drive Windows and Android exports non-interactively before assuming the
GUI editor is needed for every step.

## `gdext`

Depend on it from `cartalith-godot` only (`ARCHITECTURE.md`).

**Prove the Android export in Phase 0.** gdext's own documentation has described
Android and WASM support as experimental with tooling still lacking
(`REFERENCES.md`). For a project whose goal includes an `.apk`, this is the single
highest-risk item in the toolchain — confirm it before investing further, not
after the engine is ported.

## Android

- **SDK and NDK.** Rust compiles against the NDK; the SDK plus a JDK is what
  Gradle and Godot's export pipeline need to assemble the `.apk`. Use the NDK
  version Godot pins, not the newest — a mismatch produces link errors that look
  like Rust problems.
- **`cargo-ndk`** handles the per-target clang and linker paths. Setting them by
  hand works and is easy to get subtly wrong.
- **No keystore yet.** Debug signing is enough to sideload (`DECISIONS.md` §6).

## Windows

Cross-compiling from Linux, two routes:

1. **`cargo-xwin`** targeting `x86_64-pc-windows-msvc` — fetches the Windows
   SDK and CRT pieces itself, and matches the ABI users expect. Try this first.
2. **`mingw-w64`** targeting `x86_64-pc-windows-gnu` — no MSVC SDK needed,
   different ABI. Keep as fallback.

Godot also needs its Windows export templates, and its exporter needs to be told
to cross-export from Linux. Godot has historically handled this well; verify
against the installed version.

**A build produced here is confirmed to compile and package, nothing more.**
Whether it runs is a question only Windows answers (`DECISIONS.md` §5).

## Crates for `cartalith-io`

`zip` reads the HTML app's saves directly; `serde` and `serde_json` parse
`params.json` (`SAVEFILE_COMPAT.md`, `PROVENANCE.md` §3).

## Phase 0 order

Confirm each step before the next, so a failure names its own layer:

1. A throwaway `cargo new --lib` compiles and tests here.
2. Add `gdext`; a minimal GDExtension class loads in the Godot editor.
3. A Windows cross-build of that class exports to a working `.exe`.
4. An Android cross-build (via `cargo-ndk`) exports to a working `.apk`.
   **Budget real time here** — see the gdext flag above.

Only then start Phase 1 (`ROADMAP.md`).
