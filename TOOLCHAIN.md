# Toolchain

Everything needed before writing Rust or Godot code. Written for the cloud
session that builds (`DECISIONS.md` §5); applies equally to a local setup.

**Run `setup.sh` first.** It installs the Rust targets, `cargo-xwin`, and
`cargo-ndk` — the parts that are just tool installs with no licence to accept
— and prints a checklist for the two parts that stay manual: Godot itself and
the Android SDK/NDK, both gated behind a licence only the owner can accept,
and both version-sensitive in ways a script shouldn't guess at. Re-run it any
time; every step is idempotent. (This paragraph was restored 2026-08-19 from
the `Cartalith_RC` copy of this document, whose `setup.sh` is byte-identical
to this repo's — the script existed here with no doc mentioning it.)

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

### Which `.dll` a Godot run actually loads, and the build failure that follows

`godot-project/cartalith.gdextension` maps two Windows entries:

```
windows.debug.x86_64   = "res://../target/debug/cartalith_godot.dll"
windows.release.x86_64 = "res://../target/release/cartalith_godot.dll"
```

Godot picks between them by whether the **Godot binary doing the loading** is a
debug-featured build, not by anything in this repository. The editor is always
debug-featured, so **the editor itself, and Play/Run Project (F5) launched from
it, both load `target/debug/`.** So does `Godot_..._console.exe --headless
--path godot-project ...`, which is how every scripted headless drive in
`docs/CHANGELOG.md` runs. `target/release/` is loaded only by an exported build
that used a release export template. In practice: *everything a developer or the
owner runs day to day is the debug DLL*, and `cargo build -p cartalith-godot
--release` alone verifies nothing they will see.

**Windows holds an open file handle on a loaded DLL, and cargo cannot overwrite
it.** With the editor open on this project, `cargo build -p cartalith-godot`
fails:

```
error: failed to remove file `...\target\debug\cartalith_godot.dll`
Caused by:
  Access is denied. (os error 5)
```

That is a hard error, not a warning — but it is one line, it scrolls past in a
long build, and *the previous DLL is still sitting on disk and still loads*. The
result is a Godot session running native code older than the source tree, which
this shell degrades into silently: `engine_bridge.gd` guards every binding with
`has_method()` (correctly — it is what lets an older binary boot at all), so a
missing `#[func]` is not an error, it is a control that exists, responds to
clicks, and does nothing. Reproduced live 2026-08-20 against a running editor.

So, when a native change does not appear to take effect:

1. **Close the Godot editor**, then rebuild. Confirm cargo actually printed
   `Finished`, and check the DLL's timestamp moved.
2. Build **both** profiles when the change is meant to reach an export as well:
   `cargo build -p cartalith-godot && cargo build -p cartalith-godot --release`.
   The release DLL goes stale in the opposite way — nothing routine loads it, so
   nothing routine reveals that it is months behind.
3. Grep the DLL for a string only the new code contains (`grep -a -F "Ocean
   currents" target/debug/cartalith_godot.dll`) before concluding the *shell* is
   at fault. String literals from `#[func]` names and `const` tables survive into
   the binary; Rust field names generally do not, so pick a literal.

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
