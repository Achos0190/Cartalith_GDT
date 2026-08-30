# Android + DCC shell designs, imported 2026-08-30

Imported from the owner's Claude Design project **UI mockups planning**
(`067f80e7-dbb7-4492-8e69-96aaa8050a4d`) over the design MCP, after
`/design-login`.

| File | What it is |
|---|---|
| `Cartalith Android.dc.html` | The interactive Android prototype (156 KB, complete). Tabs, sheet detents, live gestures, generator and planner flows. **This is the design being implemented.** |

## Where the specs live

Two markdown specs govern this work and live in the same design project rather
than here, because the owner edits them there:

- `docs/ANDROID_UI_SPEC.md` — the phone. Tabs `MAP · GENERATE · PLAN · MORE`,
  sheet detents peek/half/full, project-picker entry, landscape left-rail,
  staged generator readout, undo chip, app-bar search.
- `docs/DCC_SHELL_SPEC.md` — the Windows/desktop shell, control by control.
  §2 is all seven menus item by item; §13 states **tablet keeps full desktop
  parity** (same regions, menus and disclosure depth; targets 44–52 px, docks
  400 px) and that only the phone reorganises.
- `candidates/Android Chrome B.dc.html` — the chosen chrome, and the source of
  the exact values used in the bottom bar (66 px nav, active pill
  `rgba(224,163,74,.16)` at radius 14, sheet `22px 22px 0 0` on `#15171a`,
  handle 40×4).

Read them with the DesignSync tool (`get_file`) rather than assuming — the
owner has revised them across nine question rounds and they are newer than any
canvas in `design/`.

## Relationship to `design/`

`design/*.dc.html` holds the older DCC canvases (2026-08-17 → 08-24).
`Cartalith Android.dc.html` is **newer than all of them** and supersedes
`Cartalith Android Phone.dc.html` for phone layout, under
`DCC_SHELL_SCOPE.md`'s standing rule that the newer canvas wins.
