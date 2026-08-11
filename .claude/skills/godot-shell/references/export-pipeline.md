# Build and export pipeline

Two build systems run here and must agree: `cargo` produces a native library per
platform, and Godot's exporter packages that library into an `.exe` or `.apk`.
Most failures are a disagreement between them about where a file is.

Version numbers below age quickly. Verify the current Godot release, the NDK
version Godot pins, and gdext's own Android instructions before trusting a
number written here — `../../../TOOLCHAIN.md` sets the same rule for the same
reason.

## The `.gdextension` manifest

This file tells Godot which library to load for which platform and build type.
Its shape:

```ini
[configuration]
entry_symbol = "gdext_rust_init"
compatibility_minimum = 4.2

[libraries]
linux.debug.x86_64     = "res://../target/debug/libcartalith.so"
linux.release.x86_64   = "res://../target/release/libcartalith.so"
windows.debug.x86_64   = "res://../target/debug/cartalith.dll"
windows.release.x86_64 = "res://../target/release/cartalith.dll"
android.debug.arm64    = "res://../target/aarch64-linux-android/debug/libcartalith.so"
android.release.arm64  = "res://../target/aarch64-linux-android/release/libcartalith.so"
```

The keys are feature tags — platform, build type, architecture — and Godot loads
the first line whose tags all match. Confirm the exact spelling of each tag
(`arm64` versus `aarch64`, `release` versus `template_release`) against the
Godot documentation for your version and against gdext's own template, because a
misspelled tag does not error: it silently matches nothing.

## "It works in the editor but not in the export"

This is the standard GDExtension failure, and it has one usual cause: the editor
loaded a debug library, and the export needs a release entry the manifest does
not have.

Work through it in order:

1. **Is there a release line for that platform?** The editor runs a debug build,
   so a manifest with only debug paths works until the moment you export.
2. **Did you build that target in release?** `cargo build --release` produces a
   different path than `cargo build`, and the manifest names both.
3. **Did the library reach the package?** Check the exported `.apk` (it is a zip)
   or the `.exe`'s data for the `.so`/`.dll`.
4. **Does `compatibility_minimum` exceed the Godot running it?** An older engine
   refuses the extension rather than loading it partially.

Reproduce with an export before assuming it works. A GDExtension project that has
only ever been run from the editor has not been tested.

## Windows

Cross-compiling from Linux, the two routes are `cargo-xwin` targeting
`x86_64-pc-windows-msvc` (matches what Windows users expect) and `mingw-w64`
targeting `x86_64-pc-windows-gnu` (no MSVC SDK needed, different ABI). Try
`cargo-xwin` first; keep mingw as the fallback.

Godot exports Windows from Linux given the Windows export templates. A build
produced this way is confirmed only to compile and package — that it runs
correctly is something only a Windows machine can tell you
(`../../../DECISIONS.md` §5).

## Android

The highest-risk step in this project's toolchain. gdext's own documentation has
described Android support as experimental with tooling still lacking
(`../../../REFERENCES.md`), so prove it early with a trivial extension rather
than after the engine is ported.

What has to line up:

- **Rust targets.** `rustup target add aarch64-linux-android` covers modern
  devices; add `armv7-linux-androideabi`, `x86_64-linux-android`, and
  `i686-linux-android` for older hardware and emulators.
- **NDK version.** Godot pins a specific NDK — reported as 23.2.8568313 at the
  time of writing. Install that one. A newer NDK is the usual cause of link
  errors that look like Rust problems and are not.
- **Toolchain environment.** `cargo-ndk` sets the clang path and per-target
  linker for you (`CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER` and friends).
  Setting them by hand works and is easy to get subtly wrong.
- **One build per architecture.** Each target produces its own `.so` at its own
  path, and each needs its own line in the manifest.
- **Debug signing is enough.** Personal distribution needs no keystore
  (`../../../DECISIONS.md` §6); Godot's debug keystore installs fine by
  sideloading.

## Verifying a build from a headless session

A cloud session can confirm that `cargo` compiled every target, that Godot's
exporter ran without error, and that the `.so`/`.dll` is inside the package
(unzip the `.apk` and look).

It cannot confirm the app runs, renders correctly, or responds to touch. Say
which of the two you did. Reporting "the Android build works" when you have only
confirmed it packaged is the specific claim to avoid — this project's own
headless carve-out exists for exactly this gap.
