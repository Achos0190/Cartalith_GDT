# Porting Cartalith to Godot — start here

Documentation and reference material for a ground-up native port of Cartalith
Gen1 — the single-file HTML worldbuilding tool — to a Rust engine inside Godot,
targeting a Windows `.exe` and an Android `.apk`.

**No code lives here.** This is a new project, not a refactor: nothing in
`Cartalith Gen1 v*.html` changes, and none of its working rules are altered.

## The decisions

Made through structured Q&A with the owner. Reasoning in `DECISIONS.md` — read it
before revisiting any of these.

| | |
|---|---|
| **First milestone** | Terrain-only MVP: the full pipeline, no civ, assets, sculpt, or journey planner |
| **MVP boundary** | Includes erosion and hydrology, not just heightmap and climate |
| **Architecture** | Godot + Rust via `gdext` — Rust owns logic, Godot owns rendering, UI, and packaging |
| **Rendering** | 2D only; the 3D drape is a later phase |
| **Build environment** | This cloud session builds; the owner verifies on real hardware |
| **Distribution** | Personal/hobby — no signing certificates or store work yet |
| **Correctness** | Golden-value parity against the JS engine, within a documented tolerance |
| **Code location** | A new, separate repository once porting begins |
| **Godot version** | Latest stable 4.x at setup time, pinned there rather than here |
| **Save files** | The port reads an existing HTML `.zip`'s terrain; writing comes later |
| **Build structure** | A Cargo workspace, one crate per subsystem |

## Reading order

| Document | Covers |
|---|---|
| `DECISIONS.md` | every choice, what it beat, and why |
| `MVP_SCOPE.md` | what the first milestone includes, excludes, and what "done" means |
| `ARCHITECTURE.md` | the Rust↔Godot split and the crate layout |
| `PROVENANCE.md` | academic sources, algorithms, and formats — what must be hand-ported and what a crate may replace |
| `SAVEFILE_COMPAT.md` | the `.zip` format, verified against the live code |
| `PARITY_TESTING.md` | extracting golden data and testing against it |
| `REFERENCES.md` | libraries and projects worth using or reading |
| `TOOLCHAIN.md` | setup, in the order to do it |
| `ROADMAP.md` | phases after the MVP |
| `SKILLS.md` | which Claude Code skills to install, and why |

Also here: `reference/` holds a frozen copy of the HTML app with a generated index
of all 1,094 top-level functions, and `skills/` holds the skills themselves.

## Working discipline

The HTML project's discipline — measure before fixing, test everything, finish one
thing before starting the next, document the reasoning — is the reason it survived
200+ versions. Carry it over.

- **Read before porting.** Study the JS source and the CHANGELOG entries that
  explain *why* a formula looks as it does, and extract golden values, before
  writing the Rust. Then you have something to test against as you go rather than
  at the end.
- **One subsystem at a time.** Port in pipeline order and verify each stage before
  the next. Porting four stages and debugging the combination is how a week
  disappears.
- **Faithful, not literal.** Idiomatic Rust is expected — ownership, iterators,
  `rayon` where Web Workers were. Same algorithms, constants, and formulas is also
  expected. **A deviation that changes the numbers is a decision to raise, not to
  make quietly.**
- **Document as you go.** Each milestone gets a changelog entry recording what was
  ported, how it was verified, and what is still open — so the next session can
  start cold.
- **Say what you verified.** GPU rendering, touch input, and on-device performance
  cannot be checked from a headless session. "Compiles and passes tests" is not
  "works."

## Status

Planning complete; no code written. Next is Phase 0 (`ROADMAP.md`): a walking
skeleton proving the toolchain builds for all three targets before any engine
logic is ported — in the new repository, not here.
