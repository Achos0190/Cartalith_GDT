# Attribution and sources

`godot-shell` is original prose. It exists because the Godot skill packs surveyed
below all answer a different question: how to build a game in Godot. This project
builds a native engine in Rust and uses Godot to draw it, take input, and produce
an `.exe` and an `.apk`.

## Why a narrow skill rather than a vendored pack

Ponytail's ladder, applied honestly:

- **Does this need to exist at all?** A general Godot game-development skill does
  not — this project has no physics bodies, no animation graph, no navigation,
  no multiplayer, no game loop. Installing 55 skills to use four would bury the
  four.
- **Does something already cover it?** Partly, and the overlap is named below
  rather than hidden. `GodotPrompter` in particular covers GDExtension, the
  export pipeline, mobile development, and multithreading properly.

So this skill covers only the shell subset, states the decisions this project has
already made (renderer, where logic lives, what to commit), and points at the
packs for everything else. **If GodotPrompter's `gdextension` and
`export-pipeline` skills turn out to cover `references/export-pipeline.md` better
once installed, delete that reference and keep the pointer.** Two documents
answering one question is the failure this repo's own CHANGELOG keeps recording.

## Packs surveyed

| Pack | Size | Godot | License | Fit here |
|---|---|---|---|---|
| **[jame581/GodotPrompter](https://github.com/jame581/GodotPrompter)** | 55 skills | 4.3+ (4.5/4.6/4.7 features) | MIT | **Best fit.** Has dedicated `gdextension` (explicitly godot-cpp *and* Rust), `export-pipeline`, `mobile-development`, and `multithreading` skills — the four this project actually needs. Recommended companion install. |
| **[vl4dt/godot-skills](https://github.com/vl4dt/godot-skills)** | 12 skills | 4.7 coverage | MIT | Game-oriented (physics, animation, networking). GDExtension appears only inside the C# skill; no dedicated export or renderer skill. |
| **[alexmeckes/godot-claude-skills](https://github.com/alexmeckes/godot-claude-skills)** | 5 skills | 4.x | MIT | Code-gen, scene design, shaders, plus live editor control via godot-mcp. Useful if you adopt godot-mcp; otherwise off-target. |
| **[Randroids-Dojo/Godot-Claude-Skills](https://github.com/Randroids-Dojo/Godot-Claude-Skills)** | 1 skill | 4.x | MIT | GdUnit4 testing, PlayGodot automation, CI/CD, **web and desktop** exports. Worth a look for CI later; note it does not cover Android, which is this project's harder target. |
| **[sickn33/antigravity-awesome-skills](https://github.com/sickn33/antigravity-awesome-skills)** | 1,400+ skills | — | see caution | Aggregator. `godot-gdscript-patterns` is 41 lines pointing at a playbook, with no GDExtension, texture, or export coverage. |

### Caution on the aggregator

`sickn33/antigravity-awesome-skills` re-hosts skills at scale, and the two
inspected (`rust-pro`, `godot-gdscript-patterns`) carry `source: community` with
**no original author and no license stated**. That is a licensing unknown, not a
permission — so nothing was copied from it. Its `rust-pro` also targets
"Rust 1.75+", behind the edition-2024 / 1.85 baseline `rust-craft` uses.

Install from it if you like; do not vendor from it into this repository without
tracing a skill back to its original author and license first.

## Technical claims and where they came from

- **`ImageTexture.update()` over `create_from_image()` for repeated updates** —
  the Godot documentation and community reporting both state `update` reuses the
  existing allocation while `create_from_image` allocates afresh. This is the
  project's hot path (regenerate → display), which is why it earned a place in a
  deliberately short skill.
- **Renderer trade-offs** — Godot 4 ships Forward+, Mobile, and Compatibility.
  Compatibility runs OpenGL ES for the widest Android reach and has **no compute
  shaders**, with `GPUParticles` silently doing nothing under it. Forward+ is
  documented as poorly optimised on mobile. Switching renderers later means
  reworking materials, shaders, and lighting — hence choosing before building.
- **GDExtension feature tags and the editor-works-export-fails failure** — the
  Godot documentation on the `.gdextension` file describes feature-tag filtering
  and separate editor / `template_debug` / `template_release` builds. The
  manifest example in `references/export-pipeline.md` shows the *shape*; verify
  exact tag spelling against the docs for your Godot version, because a
  misspelled tag matches nothing silently rather than erroring.
- **Android NDK pinning** — Godot pins a specific NDK (reported as 23.2.8568313
  at the time of writing). Treat the number as needing verification; treat the
  rule — use the version Godot pins, not the newest — as durable.
- **gdext Android maturity** — gdext's own documentation has described Android
  and WebAssembly support as experimental with tooling still lacking, which is
  why `../../REFERENCES.md` and `../../TOOLCHAIN.md` both flag proving the
  Android export in Phase 0.

`godot-rust.github.io` is blocked by this session's network egress proxy, so
gdext's own Android instructions could not be read directly. Read them before
attempting the Android build; the guidance here is assembled from secondary
sources and marked as such.

## Style

Same two constraints as `rust-craft` (see `../rust-craft/ATTRIBUTION.md`):
Ponytail decided what to include — a shell skill that grew to cover game
development would defeat its own purpose — and Strunk and White decided how to
phrase it: active voice, positive form, each rule's reason in a clause, and no
paragraph defending a rule the reader can simply follow.
