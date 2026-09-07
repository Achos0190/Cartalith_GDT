# Canvas fix to paste — the OFF switch track has no visible edge

**Owner ruling C, 2026-09-07:** *"Take it back to the designer."* This is the
change to make in the Claude Design project. **Nothing in the shell changes
until this lands and is re-imported** — `_cklight_probe` pins the current
behaviour so it cannot drift while the question is open.

Applies to **`Cartalith DCC Environment.dc.html`**. Check the Tablet and Android
canvases for the same construction before finishing.

---

## What is wrong

The switch track takes `var(--sur)` when OFF, and every surface it sits on is
`var(--pan)`. Those two tokens are one step apart in both palettes, so the track
disappears and the control reads as a floating dot rather than a switch.

| Palette | OFF track (`--sur`) | Ground (`--pan`) | Contrast |
|---|---|---|---|
| Light | `#f4f2ee` | `#fbfaf7` | **≈1.03:1** (measured off rendered pixels) |
| Dark | `#0d0e0f` | `#121314` | **≈1.04:1** (computed from the tokens) |

**This is not a light-palette bug.** It was reported on light only because that
is the palette the owner's machine boots. Both palettes are equally affected,
and a fix that only addresses light would be wrong.

The ON state is fine — `--acc` is `#a4650f` / `#e0a34a` and reads clearly. The
knob is fine — `var(--ink)` in both. **Only the OFF track is invisible.**

---

## The change — three lines, one identical edit each

**Append this to the track `<span>`'s style:**

```
;box-shadow:inset 0 0 0 1px var(--bor)
```

**Sites (all three are the same span; only the binding prefix differs):**

| Line | Binding |
|---|---|
| **ENV:76** | `r.togBg` |
| **ENV:365** | `f.togBg` |
| **ENV:846** | `f.togBg` |

**Before** (ENV:76 shown; ENV:365 and ENV:846 are identical with `f.` for `r.`):

```html
<span style="width:30px;height:17px;border-radius:999px;flex:none;position:relative;background:{{ r.togBg }}">
```

**After:**

```html
<span style="width:30px;height:17px;border-radius:999px;flex:none;position:relative;background:{{ r.togBg }};box-shadow:inset 0 0 0 1px var(--bor)">
```

---

## Why `box-shadow: inset`, and not `border`

**A `border` would change the geometry; an inset shadow does not.** The track is
declared `width:30px;height:17px`, and the knob is positioned against those
numbers (`top:2px`, `left:2px`/`15px`, `13px` square). A 1 px border under the
default `content-box` sizing makes the box **32×19** and shifts the knob's
resting inset from 2 px to 3 px on one side — so the switch stops being centred
and the shell, which ports these figures literally, inherits the drift.

`box-shadow: inset` draws **inside** the existing box and participates in no
layout, so **every measurement in the shell stays valid**: 30×17 track, 13 px
knob, inset 2.

## Why `var(--bor)` specifically

It is the canvas's own border token and it is already palette-aware:
`rgba(0,0,0,.20)` on light, `rgba(255,255,255,.16)` on dark. **So one
declaration fixes both palettes** and introduces no new value to keep in sync.

Using a *fill* instead does not work: the next token up, `--ins`, is `#eceae4`
on light, which is still only **≈1.15:1** against `--pan`. **No background token
in the palette gives a track a readable edge — that is why this needs a border,
not a different fill.**

---

## What this deliberately does NOT change

- **The two data lines are untouched** — ENV:1354 and ENV:1838 keep
  `togBg: v ? 'var(--acc)' : 'var(--sur)'`. The OFF fill is still `--sur`; it
  simply gains an edge.
- **The ON state keeps `--acc`.** It also receives the ring, which reads as a
  crisper edge on amber rather than a change. If you would rather the ring were
  OFF-only, that needs a fourth property (`togBord`) carried through both data
  sites and all three templates — five edits instead of three, for a difference
  most viewers will not see.
- **No geometry moves**, so no shell re-measure is needed beyond re-importing.

---

## After it lands

1. Re-import to `design/mcp-2026-09-07/`.
2. `_cklight_probe` will then **fail**, correctly — it pins the current
   edgeless track. Update its pin to the new declaration **in the same pass**,
   with the `ENV:` line cited, so the probe keeps failing on the old state.
3. Check the same construction in `Cartalith Tablet.dc.html` and
   `Cartalith Android.dc.html` — the shell draws one switch across all three.
