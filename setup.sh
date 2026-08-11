#!/usr/bin/env bash
# Cartalith native port — Phase 0 toolchain setup (TOOLCHAIN.md).
#
# Automates what's safe to automate: Rust targets, cargo-xwin, cargo-ndk.
# Checks for, but does not install, Godot and the Android SDK/NDK — both are
# downloads gated behind a licence the owner has to accept, and the right
# NDK version depends on whichever Godot build gets installed (TOOLCHAIN.md's
# own "use the NDK version Godot pins, not the newest" warning). Silently
# fetching either would be a decision this script has no business making.
#
# Idempotent — safe to re-run after a partial setup or a toolchain bump.
set -euo pipefail

say()  { printf '\n== %s ==\n' "$1"; }
ok()   { printf '  [ok] %s\n' "$1"; }
todo() { printf '  [ ]  %s\n' "$1"; }

say "Rust"
if ! command -v rustup >/dev/null 2>&1; then
  todo "rustup not found. Install it from https://rustup.rs, then re-run this script."
  exit 1
fi
ok "rustup found: $(rustup --version | head -1)"

TARGETS=(
  x86_64-pc-windows-msvc     # Windows, via cargo-xwin
  aarch64-linux-android      # the target real devices use
  armv7-linux-androideabi    # older hardware
  x86_64-linux-android       # emulator
  i686-linux-android         # emulator, 32-bit
)
for target in "${TARGETS[@]}"; do
  if rustup target list --installed | grep -qx "$target"; then
    ok "target already installed: $target"
  else
    rustup target add "$target"
    ok "target added: $target"
  fi
done

say "cargo-xwin (Windows cross-compile)"
if command -v cargo-xwin >/dev/null 2>&1; then
  ok "already installed"
else
  cargo install cargo-xwin
  ok "installed"
fi

say "cargo-ndk (Android cross-compile)"
if command -v cargo-ndk >/dev/null 2>&1; then
  ok "already installed"
else
  cargo install cargo-ndk
  ok "installed"
fi

say "Godot"
if command -v godot4 >/dev/null 2>&1 || command -v godot >/dev/null 2>&1; then
  ok "a Godot binary is on PATH — confirm it's current stable 4.x (DECISIONS.md §9)"
else
  todo "no Godot binary on PATH."
  todo "Download latest stable 4.x + its export templates from"
  todo "https://godotengine.org/download — not fetched here; the version is"
  todo "pinned at setup time (DECISIONS.md §9), not hardcoded into this script."
fi

say "Android SDK / NDK"
if [ -n "${ANDROID_NDK_HOME:-}" ]; then
  ok "ANDROID_NDK_HOME=$ANDROID_NDK_HOME"
else
  todo "ANDROID_NDK_HOME is not set."
  todo "Install the SDK + the NDK version Godot pins (not the newest one —"
  todo "a mismatch produces link errors that look like Rust problems),"
  todo "accept its licence yourself, then export ANDROID_NDK_HOME."
fi
if [ -n "${ANDROID_SDK_ROOT:-}${ANDROID_HOME:-}" ]; then
  ok "Android SDK root is set"
else
  todo "ANDROID_SDK_ROOT (or ANDROID_HOME) is not set — Gradle and Godot's"
  todo "export pipeline both need it to assemble the .apk."
fi

say "Summary"
echo "  Handled by this script : Rust targets, cargo-xwin, cargo-ndk."
echo "  Still yours to do      : any [ ] line above (Godot + Android licences)."
echo "  Next                   : TOOLCHAIN.md's Phase 0 order — a throwaway"
echo "                           crate compiles, then a minimal gdext class"
echo "                           loads in the Godot editor, then a Windows"
echo "                           export, then an Android export."
