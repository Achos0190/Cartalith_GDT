---
name: godot-shell
description: >
  Godot 4.x used as a presentation and packaging shell over a native engine —
  GDExtension wiring, showing generated data as a texture, renderer choice,
  Control-node UI, threading rules, and Windows/Android export. Use for ANY
  Godot work in this project: writing or reviewing GDScript, editing a .tscn or
  project.godot, writing the .gdextension manifest, displaying a heightmap or
  any generated field on screen, laying out UI, or debugging an extension that
  loads in the editor but not in an export. Trigger on Godot, GDScript,
  GDExtension, gdext, .tscn, ImageTexture, Control nodes, export presets, .apk,
  or "why does this work in the editor but not the exported build?" Godot 4.x
  ONLY — Godot 3 guidance is not merely dated but wrong here, since GDExtension
  replaced GDNative outright. For game systems this project does not have
  (physics, animation, multiplayer, AI), see SKILLS.md instead: this skill
  covers the shell, not game development.
license: Original — authored for this project, no external source.
---

# Godot as a shell

Godot draws, takes input, and builds the `.exe` and `.apk`. Rust owns
generation, simulation, and every number on screen (`../../ARCHITECTURE.md`).
That division is the whole design, and most Godot material found online assumes
the opposite — that Godot holds the game logic — so read advice with the
division in mind and take the parts about rendering, UI, and packaging.

Read `references/export-pipeline.md` when you set up or debug a build. Read this
file for everything you touch while writing the shell.

## Godot 4.x only

Godot 4 replaced GDNative with GDExtension. They are different systems, not
versions of one — a Godot 3 tutorial about native code describes an API that no
longer exists. Node names changed too (`Spatial` → `Node3D`, `KinematicBody` →
`CharacterBody3D`), so Godot 3 sample code fails on names before it fails on
concepts.

When you find guidance, check its Godot version first. If it does not say, and
it mentions GDNative or `Spatial`, discard it.

## Keep logic out of GDScript

Every line of GDScript that computes something is a line that escapes
`cargo test`, escapes golden-parity verification, and eventually disagrees with
the Rust that computes the same thing. GDScript belongs on the presentation
side: wiring a button to a call, positioning a node, formatting a label.

The test for whether code is in the wrong language: if you could get it wrong
numerically, it belongs in Rust.

## Showing a generated field

Rust hands over bytes; Godot wraps them in an `Image`, then an `ImageTexture`,
then draws that texture. For a `TextureRect` or `Sprite2D`, this is the whole
render path.

```gdscript
var img := Image.create_from_data(w, h, false, Image.FORMAT_RGBA8, bytes)
var tex := ImageTexture.create_from_image(img)
```

**On every regeneration after the first, call `update` rather than building a new
texture.** `ImageTexture.update(img)` reuses the existing GPU allocation;
`create_from_image` allocates a new one each time, which is the standard way to
turn a responsive viewer into a stuttering one.

```gdscript
tex.update(img)   # same size and format as before
```

`update` requires the image to match the original's size and format. A
resolution change means a new texture — which is correct, and rare.

Pick the format for the data: `FORMAT_RGBA8` for colour the Rust side has
already computed, `FORMAT_RF` for a single-channel float field you intend to
sample in a shader. Sending raw floats and colouring on the GPU keeps the CPU
out of the loop; sending finished RGBA keeps the colour logic in Rust where it
is testable. Choose deliberately — both are defensible, and mixing them is what
produces two colour paths that drift.

## Choose the renderer before you build the UI

Godot 4 offers three, and switching later means reworking materials, shaders,
and lighting:

| Renderer | Backend | Use |
|---|---|---|
| **Compatibility** | OpenGL ES | 2D, web, widest Android reach including older devices |
| **Mobile** | Vulkan / Metal | 3D on modern phones |
| **Forward+** | Vulkan | desktop 3D; poorly optimised on mobile |

The MVP is a 2D map on desktop and Android (`../../MVP_SCOPE.md`), which points
at **Compatibility** — it reaches the most Android hardware and asks the least
of it.

Know the cost before committing: Compatibility has no compute shaders, and
`GPUParticles2D`/`GPUParticles3D` silently do nothing under it. Neither matters
for a terrain viewer whose work happens in Rust. Both would matter if the later
3D phase (`../../ROADMAP.md`) wants GPU terrain work, so revisit the choice
there rather than inheriting it by default.

## UI

Build with `Control` nodes and let containers do the arithmetic. `VBoxContainer`,
`HBoxContainer`, and `MarginContainer` position their children; anchors handle
resizing. Hand-computed positions look right on your window and wrong on a phone.

Give every interactive control a minimum size that a thumb can hit. The MVP UI is
a seed field, a resolution picker, a width field, and a generate button
(`../../MVP_SCOPE.md` point 11) — small enough that getting the layout right
costs almost nothing and skipping it produces a build that is unusable on the
device it targets.

## Threading

**The scene tree is not thread-safe.** A background thread that touches a node
crashes or corrupts state, usually not immediately.

Generation runs off the main thread — it takes seconds, and a frozen window
during it is the difference between a tool and a toy. Hand the result back with
`call_deferred`, which runs on the main thread at a safe point:

```gdscript
func _on_generation_done(bytes: PackedByteArray) -> void:
    _apply_texture.call_deferred(bytes)
```

Godot's own signals are safe to emit from a worker only if the receiving code
defers its node access. Defer at the boundary and the rule stays simple.

## Version-control hygiene

Commit `project.godot`, `.tscn`, `.tres`, `export_presets.cfg`, and the
`.gdextension` manifest. Ignore `.godot/` — it is a rebuildable cache, and
committing it produces merge conflicts in generated files.

Do not commit export keystores or the compiled Rust libraries the `.gdextension`
points at. Both are build outputs; both belong in `.gitignore`.

## What this skill does not cover

This project has no physics bodies, no animation graph, no navigation mesh, no
multiplayer, and no game loop in the Godot sense. If one of those ever becomes
real, the Godot skill packs listed in `../../SKILLS.md` cover them properly and
this skill should stay narrow rather than grow to meet them.

For the Rust on the other side of the boundary, see `rust-craft` and
`cartalith-rust-conventions` — the latter carries the rule that matters most
here: a Rust panic crossing the GDExtension boundary can take down the whole
Godot process, so `cartalith-godot` converts errors rather than letting them
unwind.
