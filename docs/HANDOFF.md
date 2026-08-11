# Cartalith Gen1 — Session Hand-off

**Read this first.** Start-here guide for a new session. Pairs with `CLAUDE.md` (architecture +
invariants + working rules) and `CHANGELOG.md` (per-version history).

## Where we are

- Repo **`Achos0190/Cartalith_RC`**. This repository was seeded as a single snapshot upload
  ("Add files via upload") — the pre-merge development history (the `elevation_foundation`
  v0.036–v0.144 lineage, its branches and PRs) lives in the older `cartalith-gen1` repository
  and in `CHANGELOG.md` here, not in this repo's git log.
- **Current tool file: `Cartalith Gen1 v2.10.html`.** One self-contained HTML file, four
  script blocks (generator engine / civ-politics layer / asset library / urban-morphology
  engine, new in v0.95 — see CLAUDE.md's "Merged-file architecture"). The merge is DONE —
  there is no build step; the file is hand-evolved. New version = new file. Older `v0.57`/
  `v0.6`/`v0.61`–`v2.09` are kept and never edited.
- **v2.10 — ocean current coastal deflection widened + LOD bake depth 6.** Two owner items from one
  message. (1) Owner: "part of ocean flow sometimes seems to focus on one part of the coast and
  doesn't deflect or curve from it" → follow-up: "Im seeing the directly bumping into land/shore
  scenario." Root-caused by measurement (synthetic straight coastline + uniform onshore wind swept
  through `deflectFlow`), not assumed: `computeOceanCurrent` passed `blockBlur:1`, so the coastline
  gradient that triggers redirection was only non-negligible in ~1-2 cells touching the coast —
  the flow ran essentially undeflected until the last few cells, then snapped tangential all at
  once (a last-instant correction, not a curve — exactly the reported symptom). Fixed by raising
  `blockBlur` to 6 (a one-time, not per-iteration, cost); swept 1/4/6/8 and confirmed the far-field
  (open-ocean, no land nearby) value stays bit-identical across every value tested, so the fix is
  confined to the near-coast band, never smears the open ocean. `buildWind`'s own separate
  `deflectFlow` call (terrain wind) untouched. (2) Owner: "let's export LOD tiles to level 6 as max
  render" — the `#bakeAllDepth` picker topped out at LOD 0–5 though the handler already clamped to
  8; added the missing `LOD 0–6 · 5461 tiles` option (pure markup, no logic change). Hash vs v2.09
  diverges at the default (`currents:true` feeds `field`/`temp`/`rain`/`flow`) — isolated via a
  pinned-seed A/B with `currents=false` on both sides: byte-identical, confirming the change is
  confined to the ocean-current path (same isolation precedent as v1.82). `tests/run.sh`
  1070/1070 (+4), `tests/run_um.sh` 852/852. See CHANGELOG/CLAUDE.md for the full writeup.
- **v2.09 — LOD/bake terrain checkerboard from coarse-cell-quantized curvature.** Owner: "Terrain
  rendering/Painting is quickly blockey/pixilated especially when zooming in with LOD."
  Root-caused by direct measurement (screenshots + instrumented render pipeline), confirmed
  PRE-EXISTING back through v2.04 — unrelated to this session's own v2.05–v2.08 LOD work, just
  never visually audited at this exact deep-zoom/large-map combination before. Ruled out in order:
  `addZoomDetail`, the raw amplified height buffer (sampled per-pixel — smooth), temp/rain fields,
  the grain noise term — all smooth/continuous, and the pattern was isolated to WITHIN a single
  pyramid tile, ruling out tile-seam causes. Root cause: `curvatureAt`/`aspectFactor` (written for
  the main map's own per-pixel loop, called there with exact integer coordinates) were instead
  called from `renderBiomeTileRGBA` (interactive LOD tiles) and `bakePixel` (refined-tile export)
  as `curvatureAt(Math.round(wx),Math.round(wy))` on a FINE fractional coordinate — every pixel
  rounding to the same coarse cell reads an identical curvature/aspect value while height/temp/
  hillshade stay continuous; `materialWeights`' ×300 curvature amplification turns that
  quantization into a stark wet/dry checkerboard the moment LOD zooms in far enough that one
  coarse cell spans many pixels. Fix: `curvatureAtF`/`aspectFactorF`, continuous siblings that
  bilinear-sample `field` at the fractional coordinate — bit-identical to the originals at any
  exact integer coordinate (asserted), so the main map's own untouched per-pixel render stays
  byte-for-byte identical. Verified with real before/after screenshots — checkerboard gone. Hash
  vs v2.08 ALL IDENTICAL (neither modified function is on the default render path). `tests/run.sh`
  1066/1066 (+4), `tests/run_um.sh` 852/852. See CHANGELOG/CLAUDE.md for the full writeup.
- **v2.08 — LOD full zoom-out was cover-cropped on mobile, with no escape valve.** Owner: "When
  using LOD the window on mobile doesn't allow a full zoom out anymore." Root-caused by direct
  measurement (Playwright mobile-viewport probe), not by guessing at the `_lodZoom` clamp: every
  zoom-out path (buttons/wheel/pinch/reset) already floored `_lodZoom` at exactly 1 — the camera
  was never broken. The real defect: `_lodFitCanvas()` always displays the LOD canvas in CSS
  "cover" mode (correct while genuinely zoomed in, the v0.87 fix it's built for), which crops it
  to the viewport's own aspect with no letterboxing — so "fully zoomed out" still showed only a
  cropped slice. Measured ~68% of map width cropped at a 390×844 (mobile) viewport vs ~15% at
  1400×900 (desktop) — same bug, mostly invisible on wide windows. The off-LOD camera already has
  an escape valve (`_viewClampFill`'s v1.13 fit-scale floor lets you keep zooming out past cover
  until the whole map fits); LOD's hard `_lodZoom` floor at 1 gave the crop no such exit. Fix:
  letterbox-FIT exactly at the zoom floor, cover above it; `_lodFitCanvas()` now also runs from
  `requestLodRender()` (every zoom-only button/wheel/pinch path), not just `applyView()` (which a
  zoom step alone never called — the fix would have silently never applied otherwise). Hash vs
  v2.07 ALL IDENTICAL (pure CSS sizing). `tests/run.sh` 1062/1062 (unchanged), `tests/run_um.sh`
  852/852, 5 new smoke assertions (`R.v208`) reproducing the crop through the real DOM/camera at a
  portrait viewport. See CHANGELOG/CLAUDE.md for the full writeup.
- **v2.07 — river channel width made real-km-aware.** Owner: "check the scaling from the base...
  when setting the width of the map to 1/5/10/100 km etc scales all features accordingly. So that a
  river becomes a bigger feature progressively." Root cause: `buildRiverNetwork`'s render half-width
  and `carveRiverValleys`' valley-carve half-width were both expressed and CAPPED purely in grid
  cells, no `cellKm`/`mapWidthKm` reference anywhere — the same channel always occupied the same
  fraction of the map regardless of its real km (v1.60's defect, in a subsystem
  `terrainDetailK`/`riverCoarseEase`/`lodDetailFreqK` never reached — those ease relief/detail
  FREQUENCY, never a channel's WIDTH). Measured: a fixed world's channel held at 85 cells across
  1/5/10km — never "bigger." Fix: `riverWidthScaleK(mapWidthKm)`, the fifth `terrainDetailK`-family
  sibling (shares its `TERRAIN_DETAIL_MAX_K` cap), but eased BOTH ways (unlike every one-sided
  sibling) since width is a pure geometric km↔cell conversion: `min(16,max(1/16,800/mapWidthKm))`,
  `mapWidthKm` alone (not blended with `gw`, matching `riverCoarseEase`'s own established low-res-
  test-preview safety reasoning), no-op at the literal default. Wired at both width computations,
  each with its existing cell cap scaled by the SAME factor so reference-scale behavior is exactly
  preserved. Engine-only, hash vs v2.06 ALL IDENTICAL at default (incl. `hash_gen1.js`'s own 512px
  battery). `tests/run.sh` 1062/1062 (+7), `tests/run_um.sh` 852/852. Verified via an isolated live
  sweep (one fixed world, only mapWidthKm varied) plus new unit tests covering the standalone
  function and a synthetic single-trunk-valley wiring check. Known scope cuts: `burnChannels`' LOD
  tile-refinement burn width and `drawRiverWays`' vector-overlay stroke (a deliberate v1.29 scale-
  invariant cartographic-symbol design) both untouched — opt-in, off by default. See CHANGELOG/
  CLAUDE.md for the full writeup.
- **v2.06 — LOD tile cache: shallow zoom levels are pinned, never evicted (zoom-out re-render
  fix).** Owner: "Zooming out seems to rerender all tiles. Which it shouldn't do as zoomed out
  tiles had already been rendered before. They should be stored and recalled." Measured before
  fixing: a wide view rendered, a deep-zoom dive PANNING across a real swath of the map (enough
  distinct tiles to exceed the ~72-tile canvas budget v1.74 set), then a return to the exact
  original wide view — 3 of 10 original tiles needed full recolorization (a narrower "zoom in/out
  at one fixed spot" test showed near-perfect caching, isolating the defect to genuine exploration
  exceeding the LRU budget). Fix: shallow pyramid levels (z=0 is 1 tile, z=1 is 4, z=2 is 16 — the
  exact levels a "zoom all the way back out" gesture returns to) are now held in
  `_lodCachePinned`/`_lodTileCanvasPinned`, OUTSIDE the ordinary LRU pool, never evicted; the
  deep-zoom LRU pool is otherwise unchanged (still capped, still evicts under genuinely deep
  exploration — no finite cache survives that, and this fix doesn't try to). New `lodPinMaxZ()`
  scales the pinned depth DOWN as `_lodTile` grows (≤30% of the same per-tile-size pixel budget
  `lodTileCanvasMax()` already draws from), so pinning can never itself exceed the budget it sits
  beside — z≤2 at the default 1024px tile, down to z≤0 at 4096px. `lodCacheClear()` now also
  clears both pinned pools (unlike the LRU pools, a pinned entry never self-evicts, so leaving it
  uncleared would leak stale-world tiles every regenerate). Re-measured: the return-to-wide-view
  step now needs zero recolorizations. Engine-only, hash vs v2.05 ALL IDENTICAL. `tests/run.sh`
  1055/1055 (+9), `tests/run_um.sh` 852/852, 2 new smoke assertions verified via an isolated
  Playwright probe through the real rendering pipeline. See CHANGELOG/CLAUDE.md for the full
  writeup.
- **v2.05 — LOD zoom-detail pipeline made real-km-aware (deep-zoom pixelation fix).** Owner,
  pasting a screenshot of blocky terrain under the `LOD6 12,38/par 6,19/cached` debug label:
  "There is still a certain pixilated quality to the map when we zoom. The graphics should be
  finer than that." Root cause, found by reading source before writing any fix: neither
  `amplifyRegion` (per-tile refine pass) nor `addZoomDetail` (extra fractal octaves as LOD depth
  increases) reference `cellKm`/`mapWidthKm`/`terrainDetailK` anywhere — added-noise frequency
  ("cycles per coarse cell") always defaulted to a flat `1.0`. Exact v1.60 defect, different
  subsystem: `terrainDetailK` only eases the BASE field (deliberately does nothing for world-scale
  maps), while the LOD viewer's whole job is showing texture finer than the base grid can — exactly
  what a huge-`cellKm` world (the owner's own: 20,000km/2048px, 9.77 km/cell) needed and never got.
  Fix: `lodDetailFreqK(mapWidthKm)`, the 4th sibling of `terrainDetailK`→`riverCoarseEase`→
  `_jpDrinkingCoarseEase` (same formula as `riverCoarseEase`, own name — a future retune of one
  must not silently retune the others), wired at `lodTileOpts()`'s one call site
  (`detailFreq: lodDetailFreqK(state.mapWidthKm)` — both consumers already read `opts.detailFreq`
  with a `1.0` fallback). No-op at the literal default `mapWidthKm=800`. Measured, not assumed: the
  same coarse field/seed/tile location showed ~10.5x more high-frequency energy post-fix (discrete-
  Laplacian proxy); a real screenshot at matching Tiled-LOD z=6 coordinates confirmed visibly finer
  grain — after catching a test-site trap (the world's highest point sits on a locally flat summit
  plateau where the amplitude taper suppresses detail regardless of frequency; a nearby sloped
  point was the correct site). Engine-only, hash vs v2.04 ALL IDENTICAL at default. `tests/run.sh`
  1046/1046 (+8), `tests/run_um.sh` 852/852. Known scope cut: the v1.29-disclosed per-tile seam
  residue and v1.22's supersample-backing mechanism are separate, untouched pieces of overall LOD
  render quality. See CHANGELOG/CLAUDE.md for the full writeup.
- **v2.04 — per-stage Journey Planner overrides expanded to the full travel-option set.** Owner:
  "Per stage override should be the full travel options. Per stage a lot can change." Before this,
  `stageOverrides[idx]` only had a UI for 6 fields (Travel mode/Group size/Cargo/Pace/Pack animal/
  Vehicle) even though `_jpEffectiveStagePlan`'s generic merge already threads EVERY plan field to
  `jpCalcLand`/`jpCalcWater`/`jpCapacity` per stage with zero plumbing changes. Added 10 new rows —
  Weather/Carry food (tri-state)/Road-water quality/Infrastructure on every stage; Hours/day/
  Supplies carried/Grazing/Foraging/Mount (Mounted-Rider-only)/Desert water (gated on the STAGE's
  own biome) on land stages only, since `jpCalcWater` never reads any of those five. `routeCond`'s
  option list is now stage-category-aware (`JP_ROUTE[s.cat]`, not always `.land`'s keys) — the
  party form's own control gets away with land-only options because `_jpDeriveStages` validates
  before applying; a per-stage override has no such net. Deliberately NOT given a row:
  `seasonalClosures` (a pass/season property) and `restCadence`/`seasonDrift`/`autoPromote`
  (whole-journey aggregates a per-stage value would be silently inert for). Civ-layer only, hash
  vs v2.03 ALL IDENTICAL. `tests/run.sh` 1038/1038, `tests/run_um.sh` 852/852, 11 new smoke
  assertions, independently verified via two isolated Playwright probes (22 land + 15 sea
  assertions) before trusting the full suite — caught and fixed one real test-authoring bug along
  the way (per-stage cards are trouble-sorted, so an unscoped `[data-jps="X"]` query can silently
  grab the wrong stage). See CHANGELOG/CLAUDE.md for the full writeup.
- **v2.03 — the ℹ️ Info button moved beside the persistent `#readout` panel.** Owner, pasting a
  screenshot of the always-visible `#readout` summary in the sidebar: "I want the generation info
  button here." The button/panel (v1.101) previously lived inside `#genWorld` (`#genInfoSec`),
  so it disappeared whenever the Explore or Asset-Library tab was active — exactly the moment a
  troubleshooting dump is most likely to be wanted, since `#readout` itself is a sibling of every
  tab panel inside `<aside>` (rendered after `#genWorld`/`#explorePanel`/`#assetsPanel` close,
  never inside any of them) and stays visible regardless of the active tab. Pure DOM relocation:
  the whole `#genInfoSec` block (`genInfoBtn`/`genInfoPanel`/`genInfoText`/`genInfoCopyBtn`/
  `genInfoCopyStatus`) moved into the SAME `.sec` that already holds `#readout`, right below it —
  every element id kept exactly, so `generationInfoText()` and its event-listener block (both
  `getElementById`-driven, indifferent to DOM position) needed zero changes. Owner also asked for
  the version to jump straight to **v2.03** (not the previously-planned v2.00) for this release —
  no significance beyond the version string itself. Hash vs v1.102 **ALL IDENTICAL** in every
  scenario (a static-markup move, zero logic touched). `tests/run.sh` 1038/1038, `tests/run_um.sh`
  852/852 (block 4 untouched — this is pure block-1 HTML). Verified via an isolated Playwright
  probe before trusting it in the full smoke suite: the button now sits in the same `.sec` as
  `#readout`, is outside `#genWorld`/`#explorePanel`/`#assetsPanel`, opens with real content on
  click, and stays visible/clickable under the Explore tab — the whole point of the move.
- **v1.102 — lakes were silently misclassified as rivers in the Journey Planner.** Owner: "Pathfinding
  and routes seem to make mistakes with lakes?" Root-caused on a real lake before fixing:
  `_jpDeriveStages` correctly distinguishes CART_BIOMES Lake (14) from Ocean (15) as raw indices, but
  only ever gave the ocean branch real treatment — every lake cell was unconditionally
  `cat:'river'`/`terrain:"Calm River"` regardless of size. Reproduced: a real lake crossing measured
  172.8m gain/183.6m loss over 85.9km — a lake bed is flat, so that's DEM noise being read by
  `_jpRiverCondition` as a real current, which can just as easily fabricate a fast "Strong
  Downstream/Upstream" reading on a different crossing. A pond and a Great-Lakes-scale crossing got
  identical treatment (calmest river terrain, river-only vessel eligibility — a real open-water hull
  crossing a big lake incorrectly read as ineligible). **Fix**: `nearestLandDist` (the ocean branch's
  own "distance to shore" measurement) reused for lakes too — near-shore (`d<=2`, same cutoff as
  "Sheltered Bay") stays river-like; a lake wide enough that its middle is genuinely far from any
  shore gets the SAME open-water terrain/vessel/wind-condition treatment the ocean branch already
  has (Coastal Waters/Open Sea — existing tables, no new vocabulary). Verified on a controlled
  synthetic lake (hand-carved `_cartBiome`/`field` on a real world): 3-cell-wide stays river; a
  30×31-cell lake produces a genuine sea-classified middle reaching Open Sea; a real ocean crossing
  is unaffected. Civ-layer only, hash vs v1.101 ALL IDENTICAL. 1038/1038, 852/852, 5 new smoke
  assertions (independently verified via isolated reproduction). See CHANGELOG/CLAUDE.md for the
  full writeup.
- **v1.101 — river/water detection eased for large-scale maps; a Generate → World troubleshooting
  info button.** Owner report right after v1.100: a Journey Planner stage on a 40,000km-wide,
  2048px world (19.53 km/cell) read hundreds of km with zero water in reach. Reproduced the exact
  scale and measured before fixing. **Root cause**: `riverFlowThresh` is keyed on `terrainDetailK`,
  which only eases the FINE side of the scale — a map coarser than the 800km/2048px reference sits
  flat at k=1 by design, so the river/water threshold never loosens for a big map. Measured: only
  ~34% of land (16% desert) was within the Journey Planner's own water-reach radius of a detected
  river on the reported world. **Fix A (engine)**: `riverCoarseEase`, a new companion to
  `terrainDetailK` that eases the COARSE side, feeding only into `riverFlowThresh` (not the height-
  formula/heterogeneity noise frequency, which stays untouched). Deliberately keyed on `mapWidthKm`
  ALONE, not blended with resolution the way `terrainDetailK`'s own `cellKm` is — a first cut that
  reused `cellKm` measurably re-baselined this file's own low-resolution test suite (most of it
  runs well under 2048px at the 800km default), caught by two real `test_tail.js` failures before
  the redesign. **Fix B (Journey Planner only)**: `_jpDrinkingCoarseEase`, uncapped past Fix A's own
  cartographic cap (which exists so a rendered world map isn't cluttered with faint rivers — not a
  constraint the Journey Planner's "can a party find a spring" check should inherit). **Measured net
  effect on the reported world**: drinkable-land 34%→96% (16%→95% desert); the reported worst-case
  874km stage (10 Mounted Riders, zero pack animals) dropped from 3742%→698% of capacity — still
  correctly flags this specific extreme configuration, not over-corrected to never block. **New**:
  a Generate → World "ℹ️ Info" button (owner: "would help in troubleshooting") — a plain-text dump
  led by the same fields the on-screen readout shows, then a full `JSON.stringify` of every
  generation-affecting state block, since the readout alone wasn't enough to reproduce the reported
  world this session (several tect/erosion sliders it never surfaced). Copies via
  `navigator.clipboard` with an `execCommand`/manual-select fallback chain (must degrade on
  `file://`). Hash vs v1.100 ALL IDENTICAL at the app's own default (mapWidthKm=800, any
  resolution); a deliberate, measured re-baseline above that (confirmed via a direct field-hash A/B
  at mapWidthKm=12,800). `tests/run.sh` 1038/1038 (+7), `tests/run_um.sh` 852/852, smoke +11
  (independently verified via isolated reproduction — see v1.100's own disclosed pre-existing
  `smoke_gen1.js` environmental crash, confirmed still unrelated to this version). See CHANGELOG/
  CLAUDE.md for the full writeup.
- **v1.100 — Journey Planner stage-blocking audit: a real remedy gap fixed, two other suspects
  cleared by measurement.** Owner asked to fix, together, three flagged concerns: frequent stage
  blocking, whether the route-drawing cost model over-prefers water, and whether settlement
  gravity ever produces pathologically bad detours. **Real fix**: `_stageTrouble` missed 2 of
  `_jpVesselWaterBlock`'s 4 verdicts ("cannot operate on..." mode mismatch, "No vessel selected")
  — both fell to the generic catch-all with no useful remedy. Both mean the party cannot make this
  water leg at all, so a new confirm()-gated "🔧 Re-route journey, land-only" button (reusing
  `_jpRerouteForMode`, which gained an optional `forceMode` param) is now offered — the one case
  where re-routing the whole journey is a genuine, deterministic fix. The land-side capacity hard-
  blocks (v1.63/v1.67) also now point at their own controls instead of the generic line (no
  button, per the existing "never applied automatically" precedent). **Suspects #1 and #2 were
  re-verified and CLEARED, not fixed**: the original sea-cost diagnosis was itself an artefact of
  comparing against an unfair (non-friction-aware) land baseline — re-measured fairly, the water
  preference is friction-driven and reasonably grounded, so `_CIV_SEA_COST` was left unchanged;
  settlement gravity + the existing-way discount measured a bounded ~20% worst-case detour,
  matching their documented "soft + capped" design. **A real, unrelated bug caught during
  verification**: the JS `VERSION` const still read `'1.99'` — the THIRD recurrence of this exact
  drift (v1.30, v1.52, now this); fixed. **Environmental note**: the full `smoke_gen1.js` run
  crashes the headless Chromium page near its own end in this session's execution environment —
  reproduced identically against an unmodified v1.99 with a no-op stub, confirming it predates
  this change; the new assertions were verified via direct, isolated Playwright reproduction
  instead. 1031/1031, 852/852, hash ALL IDENTICAL. See CHANGELOG/CLAUDE.md for the full writeup.
- **v1.99 — routing geometry can cut a corner across forbidden terrain; a ferry-crossing
  exception found along the way.** A live Journey-Planner audit (real world, land+sea routes
  between real settlements, several presets/party/vessel configs, checked against
  `docs/research/travel-speeds.md` §8) found land-mode routes reading partly as ocean and vice
  versa, then "Fix all bugs you found." **Root cause**: `_civRoutingGrid`'s downsampled coarse
  cell can read "passable" from one sampled pixel while other full-res pixels inside it are the
  forbidden terrain, AND `catmullRomSample`'s Catmull-Rom spline is not guaranteed to stay within
  its own control points' convex hull — reproduces even at 1:1 resolution, so it's the spline,
  not merely the downsample. Symptom: `_jpDeriveStages` correctly classifies the resulting
  phantom stage, but the stage itself shouldn't exist — either a confusing hold/porter-capacity
  hard-block, or (worse) a silently-accepted nonsensical leg (a solo walker "crossing" 230 km of
  open water at 17 km/day, no warning). **Fix**: `_civTerrainValidTest`/`_civNearestValidPt`, a
  full-resolution repair pass inside `_civSmoothPath` itself, wired at every 'land'/'water'-mode
  caller (`_civDijkstraPath`, `_civMstRoutes`, `_civHierarchicalNetwork`,
  `_civConnectPlaceToNetwork`, `_civConnectVillageAddons`) — 'mixed' mode is exempt by design
  (crossing water there is legitimate). **A first cut of the fix was itself wrong** — caught by
  measuring before AND after: water-fraction went UP (2%→32%) on one route, because
  `_civDijkstraPath`'s own land-mode cost grid already has a documented "existing sea lane is a
  traversable ferry crossing" exception (v1.53) the naive repair didn't know about, and was
  "fixing" a real 77-point ferry leg back onto dry land — a new `opts.allowSeaLanes` flag, scoped
  to the ONE cost-grid builder that actually has this exception, fixed it. Also: `_civCommitWay`
  now warns (non-blocking) instead of silently drawing a straight line through forbidden terrain
  when a manually-drawn Way's waypoints have no real connecting path — the one
  `_civDijkstraPath`-consuming caller besides `_jpRerouteForMode` that never checked `.reachable`.
  **The audit's third finding (solo-walker paved-road speed) turned out to be the audit's OWN
  comparison error**, not an app bug — it compared against travel-speeds.md's calendar-average
  column when this tool's `dailyKm` is explicitly the travel-day figure (v1.52); against the
  correct column the measured speeds fit. No code change there. Verified via a live A/B across 7
  seeds (25,652 land-mode path points): 0 genuinely-bad points after the fix, excluding
  legitimate ferry crossings; the auto-network/sea-lane-MST/village connectors (no ferry
  exception, held to a fully strict standard) also came back clean. Hash vs v1.98 ALL IDENTICAL.
  1031/1031, 852/852, smoke +16. See CHANGELOG for the full writeup.
- **v1.98 — sea-lane geometry from round-trip time, not uniform distance (routing-audit U4/U5).**
  The agreed second half of the routing work. **U4**: `roadDijkstra` gained an optional trailing
  `edgeCost(i,j,dx,dy)` — every cost model here was `cost(cell)`, but currents/wind/flow are
  `cost(from→to)`; omitting the argument takes the identical arithmetic path, asserted by diffing
  the whole `dist` array. **U5**: `_civSeaTimeEdgeCost` replaces the flat `cost=1` per ocean cell
  (which made lane geometry pure shortest-distance with the v1.77-v1.82 vector fields having zero
  influence) with sailing time — rig polar vs local TWA plus along-track current, fields resampled
  onto the routing grid ONCE since they are deliberately uncached (v1.86).
  **A Prim MST is UNDIRECTED**, so rather than invent a tie-break for an asymmetric cost, each edge
  costs the MEAN of its two directional times — which is also the right objective, since a permanent
  lane is sailed both ways. Symmetric ⇒ MST valid and deterministic. Not a no-op: the polar is
  non-linear, so an along-wind lane has a worse round trip than a cross-wind one.
  **The obvious aggregate test was too weak to report** (mean lane sailing quality +0.35% — noise,
  because a world has only ~1-6 lanes with fixed port endpoints); the controlled same-water
  comparison is the real evidence: **36 better, 0 worse, 1 tie**, up to 15.3% faster, accepting up
  to 24% longer paths — audit Test D passing. Hash vs v1.97 ALL IDENTICAL. Deliberate re-baseline of
  generated lane geometry. See CHANGELOG for the full writeup.
- **v1.97 — water route conditions derived from the real fields (routing-audit U1/U2/U3).**
  First build off `docs/research/routing-audit.md`; closes its two P0 rows. Owner picked
  U1+U2+U3-then-U4+U5 with auto+override via `AskUserQuestion`. `_jpDeriveStages` hardcoded
  `routeCond="Neutral"` for EVERY water stage, so JP_ROUTE's river-direction and favourable-sea
  bands were unreachable by derivation and A→B always cost what B→A cost. U1: river gradient from
  the `gain`/`loss` the chunker already accumulates in metres. U2: `_jpSeaCondition` samples the
  v1.77-v1.82 current/wind vector fields (resolved ONCE per route — they are deliberately uncached
  per v1.86). U3: `jpSailFactor` rig polars — zero dead upwind, peak on a broad reach, lower again
  dead downwind. **Two calibration errors caught by measurement**: a flat sail-neutral of 0.80 sat
  near a square rig's BEST value and made "Strong Headwind" ~50% of passages (fixed by deriving
  neutral/span per rig from the polar itself); river thresholds were an order of magnitude low
  (0.8/4.0 vs a measured p50 of 9.6-32.8 m/km → 8/35, deliberately absolute not world-relative).
  Measured after: all five bands reachable (35/27/17/11/11%), **directional asymmetry 100%**.
  Hash vs v1.96 ALL IDENTICAL. 1031/1031, 852/852, 704/706 smoke (+19). Changes reported TIME, not
  route GEOMETRY — that is U4+U5 next. See CHANGELOG for the full writeup.
- **v1.96 — a full-grid faction-aggregate pass ran on every `generate()`, into a hidden panel.**
  Owner: "check for optimisation. However small every ms I'll take it. Without degrading the
  fidelity of the data." Measure-first CPU profile; **root-causes the OPEN item v1.92 logged and
  could not explain** (see the resolved entry under Next/open). `_civSubTab` defaults to
  `'factions'` (v1.55) and `generate()`'s civ wrapper calls `_civRenderPlaceEditor()` →
  `_civRefreshActiveSubPage()` BEFORE `_origGenerate`, so `_civFactionAggregates()` built
  resource potentials + population density + biome raster + ocean DT and ran a full
  `GW·GH × CIV_RESOURCE_KEYS` accumulation against the world about to be destroyed, into a
  `display:none` panel, with every cache nulled moments later — **686 ms per generate() at 1024px
  with zero settlements and zero territory**. Fixed with a `_civSubPageVisible()` gate (reads
  `#genCiv`'s inline `style.display`, exact and layout-flush-free) plus a refresh-on-reveal in
  `#genSubBar`'s handler, which also closes a real pre-existing staleness bug (that handler never
  refreshed the page on open). **A first cut of the gate was wrong and the smoke suite caught it**:
  it swallowed `_civCloseFactionsModal()`/`_civCloseFactionDrawer()`, which are STATE RESETS, and
  `#civFactionsModal` is a full-viewport overlay OUTSIDE `#genCiv` — those now sit unconditionally
  above the gate. Also hoisted `computeFlow()`'s module-global `state.world` read out of its D8
  neighbour loop (~21M lookups at 2048px, twice per generate; same class as v1.92's hoist in that
  same loop). Measured: `generate()` median 8227→7830 ms (−4.8%) at 1024px, clean trials ≈ −430 ms.
  Hash vs v1.95 ALL IDENTICAL. 1031/1031, 852/852, 685/687 smoke. See CHANGELOG for the full
  writeup incl. what was investigated and deliberately left alone (GPU `readPixels`, the per-pixel
  colour loop, `computeResistance`).
- **v1.95 — duplicate-logic sweep: seven "two functions answering one question" instances
  consolidated.** Owner: audit for functions that should be standard-on or merged, then "Fix all
  7" of a dedicated agent sweep's findings — this file's own recurring lesson (v1.30 on) hit
  again. All seven verified by direct measurement (a probe reproducing the shared function
  against a hand-copy of the OLD formula, plus full-grid diffs), not assumed bit-identical from a
  small-looking diff: (1) `_civEnhancedTravelCost` (trunk-road builder) vs `_civMixedCostGrid`
  (Route tool) — the biome-friction table was a verbatim copy (now shared
  `_civBiomeFriction`, bit-identical), and the "navigable river bonus" curve had ALREADY drifted
  despite a comment claiming consistency (`_civMixedCostGrid` discounted any order≥1 flow via an
  ungated absolute floor; `_civEnhancedTravelCost` only ever discounted order≥3) — new shared
  `_civNavigableRiverDiscount(order)` makes them agree, measured: `_civEnhancedTravelCost`'s own
  output is untouched (0/41,984 test cells differ), `_civMixedCostGrid` changes on 12.4% of cells
  (mean cost +7.8%) — the one deliberate, disclosed behavioral consequence of the whole pass. (2)
  `_civSettlementPopulation` vs `_civPlaceCatchmentCeiling` — the latter's own comment admitted
  copying the former's formula rather than calling it; new shared `_civCatchmentPop`, verified
  provably consistent across all 18 test-world settlements. (3) Journey Planner's human
  water-consumption rate (4 sites) and animal water-carry-days (2 sites) — new
  `jpHumanWaterRate`/`jpAnimalWaterCarryDays`, siblings to v1.84's `jpHumanWaterCarryDays` which
  fixed the adjacent carry-duration duplication but left these two. (4) The mild-upland
  defensibility term at 3 sites (`buildSettlementSuitability`, `_civPlaceDefensibility`,
  `_umWallSpec`) — new `_civTerrainRuggednessD(r)` in block 1 (`_umWallSpec` still can't call
  `_civPlaceDefensibility` directly, real recursion, but shares the raw term now). (5) Base
  population-by-kind hardcoded at 3 sites with inconsistent `metropolis` coverage — new named
  `_CIV_BASE_POP_BY_KIND`/`_civBasePopForKind`. (6) A "0.60 coastal tolerance" hardcoded at 2
  sites whose comment wrongly claimed it matched `_civSnapToWaterEdge`'s default (actually 0.80)
  — new named `SETTLE_COAST_SWAP_TOLERANCE`. (7) The catchment-radius-in-cells conversion at 5
  sites — new `_civCatchmentRadiusRaw`/`_civCatchmentRadiusCells`. Hash vs v1.94 ALL IDENTICAL
  (civ-layer only). 1031/1031, 852/852, 685/687 smoke (2 known pre-existing environmental
  failures, unrelated). See CHANGELOG for the full writeup.
- **v1.94 — grain-yield wiring: connecting v1.31's orphaned formula surfaced a real overshoot
  bug.** Owner: "Let's build in the grainyield part and all that it connects to." v1.31 shipped
  `grainYieldRatio()`/`GRAIN_YIELD_RATIO_FLOOR`/`GRAIN_YIELD_RATIO_TYPICAL`/`GRAIN_SEED_KG_PER_HA`
  grounded in `docs/research/settlement-resources.md` §10.4 but never called them; v1.93's
  dead-code audit re-found the same orphaned code and correctly left it alone (v1.31's own
  CHANGELOG already discloses it as a deliberate, accepted scope cut — "exposed but only
  reported, not wired into food surplus"). This version does the wiring. Tracing the arithmetic
  before wiring it in surfaced a real formula bug: `grainYieldRatio(K)`'s original formula
  reached `TYPICAL` at K=0.3 and kept climbing to a flat **5.68** (31% past the documented
  historical maximum of 4.34) for any K≥0.6 — invisible until now because the function had zero
  callers. Fixed to a plain linear interpolation over K's own [0,1] range, matching the clamp
  convention its siblings `subsistenceModeAt`/`agrarianDensityKm2` already use. New
  `_civPlaceGrainYield(p)` (the established thin on-demand `_civPlace*` primitive convention,
  same idiom as `_civPlaceDefensibility`) samples `currentCarryingCapacity()` at the settlement's
  own cell and returns `{ratio, floor, deficit, kgPerHa}` — explicitly reported, not simulated,
  feeding no population/food-shed math (preserves the v1.34 ACYCLIC-chain rule: "nothing
  downstream may feed back"). Wired into the SHARED `_civFormatPlaceInsp(p)`, discovered to
  already feed BOTH the Settlement Inspector popup and the City Viewer's General section — one
  change satisfies "all that it connects to" with no second, competing display path.
  Deliberately not derived from `grainYieldKgHa(soil)` (calibrated for a different purpose,
  v1.34 population modeling — dividing by `GRAIN_SEED_KG_PER_HA` would produce ratios up to
  16.67, past any historically-grounded figure). Hash vs v1.93 ALL IDENTICAL (pure on-demand
  display, never reached from `generate()`/`renderNow()`). 1031/1031, 852/852, 685/687 smoke (the
  2 shortfalls are the long-standing pre-existing v0.92/v0.87 environmental canvas-sizing
  failures, reconfirmed unrelated). Known scope cuts: not surfaced in the Settlements table or
  Economy/Statistics pages (matches the `_civPlaceDefensibility`/`_civPlaceFoodSurplus`
  per-settlement-only precedent); no world-relative self-calibration of the K→ratio curve
  (judged unnecessary — zero simulation stakes, display-only). See CHANGELOG for the full
  writeup.
- **v1.93 — code streamlining: seven confirmed-dead functions/subsystems, three unused
  constants, and one real bug found along the way.** Owner: "Can we make the very code itself
  more streamlined? This by removing unnecessary bits or things that are double? Beware to not
  lose functionality!" A mechanical zero-reference sweep, not a rewrite: a script extracted every
  top-level `function`/`const` declaration across all 4 script blocks, counted occurrences of
  each across the whole file, and flagged anything with a count of 1 (nothing else ever
  references it) — then every flag was manually verified in context, cross-checked against BOTH
  headless test suites (`tests/test_tail.js`, `tests/um_test_tail.js`), before touching anything.
  Functions only called by a test suite (`featuresNear`, `collectVisibleTiles`, `chunkChildren`,
  `unpackRGB8`, `grainKgPerHaMedieval`) were correctly left alone as deliberately-kept, tested
  primitives, not cruft. **Seven genuinely dead functions/subsystems removed**, each because its
  job is already done elsewhere — the recurring "two things answering one question, one silently
  orphaned" shape, just with the orphaned side dead rather than drifted: `_civPopulateEntityInspector`
  (a v1.16-planned settlement/faction-editor dispatcher — both targets are called directly from
  their real call sites, nothing ever routed through it); `computeRainfall()` (an early orographic
  rain model fully superseded by `simulateWeather()`); the entire `dirty`/`invalidate()`/
  `flushDirty()` dependency-graph invalidation layer (superseded early on by direct calls +
  `_fieldGen`/`_climGen` generation counters everywhere — `flushDirty()` never had a caller, four
  of `dirty`'s five non-render flags never had a reader; `dirty` narrowed to just `{render:false}`,
  the genuinely load-bearing `scheduleRender()` rAF scheduler is untouched); `clearScratch()` (an
  unused optimization hook — the scratch-buffer pool already self-manages growth per call);
  `view3dSync()` (its body is duplicated inline at `enter3D()`, the only place that needed it);
  `_origRenderNow` (an abandoned capture — the real perf-overlay auto-refresh wiring exists under a
  different name, `_renderNow_orig`, ~3700 lines later); `polylineCrossings` (UME engine, block 4 —
  a thin `segInt` wrapper the ported engine never calls, and not part of UME's own `_test:{}`
  exposure either). Three unused data constants removed too (`ORGANIC`, `LANDFORM_KEYS`,
  `SCULPT_GLOBAL_KEYS`), each checked against its sibling to confirm only the unused half was
  orphaned. **One real bug found and FIXED, not removed**: `resource_index.json` was documented (a
  v1.31-era comment stated as present-tense fact that "these keys are written into
  `resource_index.json`") but never actually pushed into `exportZip()`'s entries, unlike its
  siblings `biome_index.json`/`lithology_index.json`. Wired `resourceIndexManifest()` in, matching
  the existing pattern exactly; verified via a real export probe showing all 15 resource keys now
  present. Deliberately left alone: `grainYieldRatio()`/`GRAIN_YIELD_RATIO_*` — also zero-caller,
  but v1.31's own CHANGELOG already discloses this as an intentional, accepted scope cut, not
  accidental cruft. Hash vs v1.92 ALL IDENTICAL. 1031/1031, 852/852, smoke matches the v1.92
  baseline exactly (no new assertions needed). Net ~60 fewer lines, identical behavior, one
  previously-silent export gap closed. See CHANGELOG for the full writeup.
- **v1.92 — generation-chain speed pass: a redundant Math.hypot() in every D8 neighbour loop,
  and plate-array object access in assignPlates().** Owner: "Check the full génération chain and
  rendering chain function by function and see if we can optimise." A genuine function-by-
  function CPU-profile audit at 2048px (CDP Profiler self-time + direct wall-clock A/B, the same
  methodology v1.87/v1.89 established), deliberately not re-chasing ground those two passes
  already covered (`buildWaterBodies`'s MinHeap, `streamPowerKernel`/`glacialKernel`'s MinHeap +
  incision-coefficient hoist, the per-pixel colour loop, `roadDijkstra`'s heap which regressed
  and was reverted). Found two real, fixable redundancies: (1) `Math.hypot(dx,dy)` recomputed
  fresh on every (cell, neighbour) pair across 7 D8 (`dx,dy∈{-1,0,1}`) loop sites in 4 functions
  (`streamPowerKernel` ×4, `glacialKernel` ×1, `computeFlow` ×1 — run twice per default
  `generate()` — `buildRiverNetwork` ×1), even though it only ever takes 2 distinct values;
  hoisted into a 9-entry lookup built once per call via the SAME `Math.hypot` call the inline
  version used (a pure hoist, bit-identical by construction — not a reformulation, so no risk of
  the precision drift v1.89 had to catch). Every OTHER `Math.hypot(dx,dy)`-shaped call in the
  file was checked and correctly left alone (droplet/velocity kernels, brush radii, settlement
  distances all pass genuinely varying dx,dy — nothing to hoist). (2) `assignPlates()`'s
  already-O(N log N) Jump-Flood-Algorithm Voronoi rasterisation dereferenced
  `plates[p].x`/`plates[p].y` (object property access) inside its innermost loop; hoisted into
  flat `Float64Array`s built once at function entry, `plates` itself untouched elsewhere.
  Measured via direct wall-clock A/B (profiler self-time alone is known to overstate real
  impact — v1.87/v1.89's own lesson): `generate()` total at 2048px **39438.8ms → 33912.8ms
  median (−14.0%)**, non-overlapping trial distributions across 10 trials/side;
  `assignPlates()` alone **3489.6ms → 3185.0ms median (−8.7%)**, also non-overlapping. The
  render chain was separately audited (a CPU profile of the hot-cache `renderNow()`/
  `drawCivLayer()` interactive path on a real 41-settlement/71-way populated world) and found
  already fast — under 1ms/call at 1024px, no redundant computation, no fix needed. Hash vs
  v1.91 ALL IDENTICAL. 1031/1031, 852/852, smoke suite matches the v1.91 baseline exactly (no
  new assertions — a performance-only change has nothing new to assert). Two items considered
  but not pursued, logged below in "Next / open": GPU `readPixels` cost (still the single
  largest CPU-profile self-time line item, still not independently actionable in this
  SwiftShader-only environment) and a small `buildResourcePotentials` cost inside `generate()`'s
  own trailing render whose trigger wasn't tracked down this pass.
- **v1.91 — asset pack persistence + the Splat-texture Library bridge.** Owner: "Make sure all
  shown functions in saving and loading assets are functional. And that saving a map saves
  everything ways, settlements, assetpack, painting everything." Audited every save/load surface
  before writing any fix — ways/settlements/labels/icons/factions/territory (`_civSyncToState`/
  `_civSyncFromState`) and hand-painted Cartography overrides (`state.cartoPaint`, v0.146) were
  already correctly round-tripped, reconfirmed with a fresh live probe. The asset pack was not: a
  real `exportZip()`→`loadZip()` probe confirmed `loadAssetPack()` (the header's own "Import asset
  pack…" button) set the runtime `assetPack` global directly but never touched the Asset Library's
  `AssetDB` — the one thing `_alExportEntries`/`_alImportProject` actually persist (invariant 6:
  `assetPack` is never serialized directly). Total, silent loss confirmed: 10 icon/7 texture slots
  before export, `null` after reload. Fixed with a new `window._alImportPackZip` bridge (mirrors
  the existing `_alExportEntries`/`_alImportProject` cross-block convention) that mirrors the same
  pack into `AssetDB` right after `loadAssetPack()` sets `assetPack` — the live map is untouched,
  only the NEXT save now captures the pack. Testing that fix surfaced three more real bugs in the
  pre-existing v1.26/v1.28 Library→runtime bridge (`syncToRuntime()`/`applyLibraryAssets()`): the
  "Splat channels" (`textures`) family was never wired into the bridge at all (v1.28's own "EVERY
  family" pass missed the one family that predates it — ground-material Library art only ever
  reached the map via a full pack export→re-import loop); `assetPack.texAny` (the gate every splat
  render call site checks) was never set by the bridge even once the textures slots themselves
  were restored — data present, invisible, caught only by a direct before/after probe of `texAny`
  itself; and pack name/author/license attribution never traveled through the bridge. All three
  fixed. A fifth, smaller gap: the Library's own "Import pack" button never called
  `syncToRuntime()` (needed a separate manual "Apply to map" click, unlike every other way art
  enters the Library) — fixed and confirmed via a real `page.setInputFiles()` drive of the actual
  file input, not a direct function call. Disclosed residual: a texture round-tripped through the
  Library gets resampled to that family's fixed 512×512 canvas size if it wasn't already that size
  (confirmed on the reference sample pack: 256×256 → 512×512, `inv` unchanged to four significant
  figures — same content, not corruption; a pre-existing property of `renderToCanvas` biomes/
  terrains have had since v1.28, not newly introduced). Hash vs v1.90 ALL IDENTICAL (default render
  untouched — every fix is reachable only once a pack is actually imported). 1031/1031, 852/852, 7
  new smoke assertions (`R.v191`). See CHANGELOG for the full writeup.
- **v1.90 — save files: DEFLATE-compress the project .zip; internal format unchanged.** Owner:
  "simplify save files and find a way to optimise the internal formatting of the save files and
  compression of the save files." Measured first: `exportZip()`'s `zipStore()` always wrote
  method-0 (STORE, zero compression) — a real populated 512px world's export measured 71.24MB raw,
  with the large `.f32` field exports (many of them sparse/mostly-zero, e.g. resource-potential
  layers) the dominant contributor. `unzipAny` (pre-existing, already used by `loadAssetPack`/the
  asset-pack importer) already reads BOTH stored and DEFLATE'd entries via the central directory —
  so the only missing half was on the WRITE side. `zipStore` is now `async` and DEFLATE-compresses
  each entry via the native, zero-dependency `CompressionStream('deflate-raw')` (raw DEFLATE
  bitstream — matches ZIP method 8 exactly, unlike `'deflate'`/`'gzip'`'s extra wrapper bytes),
  falling back to method 0 whenever compression doesn't shrink the entry, whenever
  `CompressionStream` is unavailable (this file's `file://`-degrades-gracefully convention), or for
  any `.png`-named entry (already-compressed raster data — not worth the CPU). `loadZip` switched
  its reader from the store-only `unzipStore` to `unzipAny` — the one place still on the old
  reader, so a v1.90+ compressed save silently failed every `.f32`/`.bin` lookup before this; a
  strict superset, every pre-v1.90 store-only save still loads unchanged. `exportZip`/
  `exportRegionTiles`/`ZipExporter.blob`/`.download`/`AssetLibrary.exportPack()` all threaded the
  new `await`; fixed one real latent bug surfaced by the same sweep —
  `AssetLibrary.applyToMap()` called `ZipExporter.blob()` without awaiting it, which would have
  passed a bare Promise into `loadAssetPack()` and thrown at runtime the first time that code path
  actually ran. Verified via a REAL end-to-end `exportZip()`→mutate-live-state→`loadZip()`
  round-trip (the actual production functions, not a reimplementation, Playwright-driven on a real
  generated+auto-populated world): perfect fidelity (field/temp/rain hashes, places, ways, labels,
  seaLevel, seed all identical after reload) and a measured 26.35MB→12.11MB (54%) real file-size
  reduction; isolated raw sparse-`.f32`-style entries compressed ~78%. Two test-writing mistakes
  caught before shipping: an initial smoke test assumed generic sine-wave float data would compress
  >2x and measured only ~9% (DEFLATE doesn't exploit float32 mantissa continuity) — replaced with a
  sparse/mostly-zero test array matching the real shape that drives the actual measured win, at a
  realistic >5x threshold; a hand-rolled backward-compatibility test constructing a pre-v1.90-style
  ZIP byte-for-byte omitted the central directory record's own separate trailing filename-bytes
  copy (distinct from the local header's copy), corrupting the parse — fixed by including it, per
  the ZIP spec. `exportRegionTiles`'s own separate `wantGzip`-driven per-file `.gz` mechanism for
  refined heightmap tiles is now largely redundant (the whole `.zip` compresses anyway) but was
  deliberately left alone — a separate, working, user-facing feature with its own naming
  convention, not in scope for a "don't touch things that work" pass. Internal SAVE FORMAT
  (`params.json` shape, field/layer naming, entry list) is unchanged — "simplify" resolved to
  "smaller and cleaner," not a schema rewrite; no evidence surfaced that the existing format itself
  is a real pain point beyond size. Hash vs v1.89 ALL IDENTICAL (compression is a
  write/read-time-only change, never touches `generate()`/render output). 1031 / 1031 (`run.sh`,
  +10 new assertions), 852 / 852 (`run_um.sh`, block 4 untouched). See CHANGELOG for the full
  writeup.
- **v1.89 — simulation-speed pass: erosion kernels' priority-flood heap.** Owner: "Again search
  for optimisation in the simulation, rendering, LOD, and way/route/PathFinding.js systems"
  (no separate PathFinding.js file exists — the in-file route/Dijkstra logic is what's meant).
  `streamPowerKernel` (runs synchronously inside `carveRiverValleys()` on every default
  `generate()` call) and `glacialKernel` (manual erosion button) got the same preallocated-
  typed-array `MinHeap` fix v1.87 shipped for `buildWaterBodies`. `streamPowerKernel`'s implicit-
  incision loop was ALSO recomputing a per-cell `Math.pow`-driven coefficient on every one of
  `P.iters` passes despite every input being fixed before the loop starts — hoisted into a
  `Float64Array` (a first cut used Float32 and was correctly caught as a real mismatch by
  `hash_gen1.js` — `field`/`temp`/`rain` differed while `flow`/`rgba` coincidentally still
  matched). Net: ~12% off `carveRivers`, ~4-5% off total `generate()` at 2048px, all measured via
  direct A/B (a CPU profile of these loops turned out to meaningfully overstate the win — profiler
  overhead, not a real cost, so every number in the CHANGELOG entry is wall-clock, not profiler
  self-time). The identical fix applied to `roadDijkstra` measured WORSE (up to -41% on a real
  "Generate Roads" run) because it allocates a fresh heap per SETTLEMENT rather than once per
  call — reverted, numbers left in a comment so it isn't re-attempted blind. LOD and the per-pixel
  render loop re-investigated, found already tight (same conclusion as v1.87). Hash vs v1.88 ALL
  IDENTICAL. See CHANGELOG for the full writeup including the GPU-readback finding left disclosed,
  not acted on (this environment's software-GPU numbers can't represent real-device behavior).
- **v1.88 — settlement pick priority + visibility-gate consistency.** Owner: "Settlements are
  clickable on any zoom level, making it hard to click a larger settlement when you're zoomed out
  as it often means you click one of the smaller ones that are only visible when zooming in." Two
  defects found at all five place-pick sites (`_civSelectPlaceAt`, `_civDropPlace`'s
  select-near-existing, both `_civInfoAt` radii, the right-click context menu): (1) every site
  picked purely by nearest-pixel with no regard for how prominently `drawCivLayer` actually draws a
  settlement (`(4+klass.rank)*lsc`) — a small, close pin could beat a much bigger one only slightly
  farther away, exactly the reported symptom; (2) only `_civSelectPlaceAt` respected the
  `villageAddon` visibility gate (v1.68/v1.70) — a village `drawCivLayer` refuses to draw below its
  reveal threshold could still be picked at the other four sites. Fixed with two shared helpers
  (`_civPlacePickVisible`, `_civPlacePickWeight` — the latter mirrors `drawCivLayer`'s own pin-size
  formula verbatim) applied everywhere, ranking in-range candidates by `d²/w²` instead of raw `d²`.
  The absolute pick radius is unchanged — only the tie-break among in-range candidates shifted.
  Verified directly against unmodified v1.87 in both directions (a realistic near-miss flips from
  hamlet→city; a hidden addon stops being selectable while zoomed out; an unambiguous click on a
  small settlement still picks it, guarding against over-correction). One near-miss caught during
  building: an early cut also weighted `_civInfoAt`'s tight City-Viewer pin-hit re-test, which isn't
  a competition between candidates — that inflated the hit radius up to 81× for a metropolis;
  reverted, left unweighted by design. Hash vs v1.87 ALL IDENTICAL. 6 new smoke assertions
  (`R.v188`). See CHANGELOG for the full writeup.
- **v1.87 — rendering-speed pass: buildWaterBodies()'s priority-flood heap.** Owner: "let's see
  if we can optimise the code again for rendering speed whilst we keep the fidelity and detail."
  Measured first via `tests/perf/perf_gen1.js`: the render "prologue" phase (everything before the
  per-pixel colour loop) cost nearly as much as the pixel loop at large resolutions. A CPU profile
  (CDP `Profiler`, real 2048px world) isolated it to `buildWaterBodies()`'s `MinHeap` (the
  priority-flood depression-fill backing lake classification, rebuilt once per terrain change) —
  ~53% of the function's own time was `MinHeap.pop`/`.push`, which was backed by dynamically-
  growing plain JS arrays even though every cell is enqueued at most once (a known upper bound
  `n=W·H`). Fixed by preallocating typed-array heap storage (identical sift-up/sift-down
  comparison logic ⇒ identical push/pop order ⇒ bit-identical output) and hoisting two neighbour-
  test closures that were being reallocated on every one of up to `n` loop iterations. Verified via
  `hash_gen1.js` (ALL IDENTICAL), a direct hash/sum comparison of `buildWaterBodies`'s own output
  arrays, and a controlled 6-trial A/B showing a real 16% reduction at 2048px (median
  1006ms→844ms) — the official harness itself under-reported this due to background noise from
  twelve preceding `generate()` calls. The per-pixel colour loop (the single largest render cost
  overall) was profiled too and deliberately left untouched — no redundant recomputation found;
  further speedup there would need either a fidelity-losing LUT approximation or a much larger,
  higher-risk dispatch restructuring. Hash vs v1.86 ALL IDENTICAL. See CHANGELOG for the full
  writeup including the CPU-profile numbers.
- **v1.86 — bug hunt + optimization pass: climate re-simulation silently left settlement
  suitability, biome classification and several debug views on stale data.** Owner: "Can you bug
  hunt and do a optimisation pass" — an audit pass, not an owner-reported symptom. Found by
  checking every cached field against this file's own established `_fieldGen`/`_climGen`
  convention (per `currentSlopeField`'s own v1.17 comment and `_climGen`'s module comment),
  confirmed by direct before/after reproduction on a real world BEFORE any fix shipped. Root
  cause: `computeFlow()`/`generate()` null a whole derived-cache family together (biome raster,
  soil, lithology, landform, resources, carrying capacity, settlement suitability, wildlife, NPP,
  population density, wetlands) — but `computeTemperature()`/`simulateWeather()`, independently
  reachable via a climate-slider drag or the standalone "Simulate weather" button, rewrite
  temp/rain WITHOUT going through computeFlow/generate, so none of that family was invalidated.
  Same defect class the sea-level slider's own handler already had to patch once (its own comment:
  "owner report: the geological Resources view stayed stale... NOT invalidated on a sea change") —
  never extended to climate. Consequential, not cosmetic: `currentFloodField()` feeds
  `buildSettlementSuitability`'s flood penalty AND `_civSnapToWaterEdge` directly, so the real
  workflow "generate once, try climate configs via Simulate weather, auto-populate" silently used
  the FIRST configuration's data regardless of what the map was showing. Two narrower, same-class
  bugs alongside it: `currentFloodField`/`currentWindThrowField` were keyed on `state.tect.seed`
  instead of `_fieldGen` (every sibling cache already uses it), so a same-seed regenerate or
  sculpt/erosion edit never invalidated them either. Fixed by adding the SAME verbatim
  family-invalidation line computeFlow/generate already use to both climate functions (a narrower
  hand-picked list nearly shipped missing two fields that turned out to depend on rainField
  transitively — verbatim over hand-trimmed), and switching the two cache keys to `_fieldGen`
  (`,_climGen` for wind-throw). Bundled a small, low-risk optimization: `buildWindThrowField` now
  reuses the cached `buildBiomeRaster()` instead of re-classifying per cell, also fixing a real
  mountain-lake misclassification. **Considered and deliberately NOT done**: caching
  `currentWindField()`/`currentOceanField()` (the only `current*Field()` accessors with zero
  caching) — investigated because it looked like the same gap, but these two recompute their
  `tSea` proxy directly from LIVE `state.climate`/`state.planet` (via `climEffectiveEquatorTemp()`,
  not the cached `tempField`), which is WHY they update instantly while dragging the tilt/rotation
  sliders — caching them would have reintroduced staleness there, trading one bug for another; left
  uncached, disclosed rather than shipped as a plausible-looking regression. Hash vs v1.85 ALL
  IDENTICAL (every fix changes only WHEN a cache recomputes, never its deterministic value). See
  CHANGELOG for the full writeup, including the exact before/after probe numbers.
- **v1.85 — ocean heating grounded in axial tilt + rotation; confirms climate→rendering
  interconnection.** Owner: "gravity, axial tilt and how long days are on the world all these
  things inform how much energy a sun sets in a world and how much it keeps (eg. the heating of
  the ocean and flow of it are influenced)... let's always assume a sun like star and quantify
  this in a simple way. All I want is that the heating of the ocean and resulting ocean currents
  and subsequent wind are all based in grounded values." Also asked to confirm the terrain-
  coupled wind/current work (v1.77–v1.82) reaches map rendering — confirmed by direct inspection,
  no code needed: `simulateWeather()` calls `oceanSSTAnomaly()`/`computeOceanCurrent()` before
  building its own wind field, the documented "Loop 2" shipped since v0.067. The real gap: the
  base equator-pole temperature gradient (six duplicate `tSea` formulas + the GPU shader) was
  completely disconnected from `state.planet`'s gravity/tilt/rotation sliders. New
  `climEffectiveEquatorTemp()` scales the equator-pole CONTRAST by two grounded, disclosed
  multipliers — axial tilt via the real North & Coakley (1979) 2nd-order energy-balance-model
  obliquity term (critical reversal at arccos(1/√3)≈54.7356°, Rose/Cronin/Bitz 2017), rotation via
  the SAME Ω=24/rotationHours `circulationCells()` already uses — both normalized to exactly 1.0
  at Earth defaults (23.4°, 24h), so the bit-identical invariant holds by construction (confirmed:
  `hash_gen1.js` v1.84→v1.85 ALL IDENTICAL at defaults). Gravity deliberately gets no new term —
  its established roles (lapse rate ~g, circulation-cell count) are already implemented, and no
  equally simple, citable formula exists for a further direct link. Measured live: real
  `generate()`+`refreshClimate()` at max tilt (45°) drops mean tempField 14.9°C→−16.2°C, in the
  predicted direction. Full derivation: `docs/research/solar-energy-budget.md`. See CHANGELOG for
  the full writeup.
- **v1.84 — Journey Planner: water only counts as carried weight in arid biomes.** Owner: "water
  should only become an actual weight in arid biomes/climates... for other journeys it should
  technically not be counted. Water is usually abundant and always collectable in meaningful
  quantities." Root cause: `humanWaterCarryDays` still charged a flat 2-day water reserve for
  ANY non-desert biome (animals already correctly charged zero there) — fixed with a new shared
  `jpHumanWaterCarryDays(biome,supplyDays)` helper redirecting three duplicate call sites, and
  `jpCalcLand`'s convergence-loop water term gated to `isDesert?(...):0`, which also **reverts
  v1.56's "auto water-crossing tier applies to any biome" widening** back to desert-only per this
  direct new instruction. Swept for consistency: `jpAssessResupply`'s cause-attribution, the
  hard-block wording, the formula trace ("assumed abundant... not counted as carried weight"),
  the Info panel's Water finding, and `_jpPlan`'s `waterL` route-summary total (which reads
  `cap.humanWaterRate` directly, bypassing the gated breakdown — the one site that would have
  silently kept reporting non-zero water for a non-desert stage). `jpCalcWater` (sea/river
  vessels) deliberately untouched — a ship can't detour to a stream mid-passage. Hash vs v1.83 ALL
  IDENTICAL (JP is interactive-only). See CHANGELOG for the full writeup.
- **v1.83 — a Mounted Rider party's own mounts now carry saddlebag capacity.** Owner pasted a
  real route with several `⛔ Carrying enough water...` blocks (up to 3659% over) and one
  `⛔ Overloaded 167%...` block: "Can you see what needs fixing?" Diagnosed: the reported 300kg
  capacity was exactly `people*JP_HUMAN_PORTER` — a Mounted Rider party got zero extra capacity
  from being mounted, identical to Walking, unless the same mounts were ALSO manually re-declared
  as pack animals (the "Lone courier" preset's own undocumented convention). `jpCapacity` now
  credits `max(0, people - plan.animals[mount]) * JP_ANIMALS[mount].cap *
  JP_MOUNT_SADDLEBAG_FRAC(0.3)`, avoiding double-count by construction. Verified against all four
  shapes: reported case 300kg→660kg; Walking party unaffected; Lone courier preset unchanged;
  partial declaration blends correctly. Does NOT rescue a genuine extreme water overload — a
  400km waterless-desert crossing for the same party still correctly blocks, verified directly.
  Hash vs v1.82 ALL IDENTICAL. See CHANGELOG for the full writeup, including a test-writing
  mistake (wrong `_jpEnsurePlan` calling convention) caught and fixed before shipping.
- **v1.82 — ocean current direction becomes heat-driven, not just wind-derived; windFx slowed
  65%.** Owner: "check how heat in an ocean originates and how flow direction is dictated by it.
  At the moment it just seems to base itself from right to left." Measured first: the meridional
  current was set solely by Ekman-rotating the latitude-band wind, zero basin-position dependence
  — ~94% of the equatorial trade band showed net-poleward flow, cold anomaly nearly absent,
  contradicting the file's own docstring. Fixed with a western/eastern-boundary bend in
  `computeOceanCurrent` (poleward pile-up on a basin's western edge — Gulf Stream-style; weaker
  equatorward on the eastern edge — Peru/Benguela-style upwelling), reusing the existing coastal-
  distance weights, zonal component untouched by construction. A single-pixel min/max test was
  tried and rejected (the bend can locally cancel one outlier cell); robust aggregates show the
  real improvement — mean-absolute SST anomaly 0.204→0.511, cold-cell fraction 4.5%→19.2%. windFx
  particle speed `0.9→0.315` (35%, "slow down by 65%" taken literally), verified against the real
  shipped function via rAF interception. Not bit-identical at defaults (`currents:true` feeds
  `field` via `carveRiverValleys`) but isolated: with `currents=false`, hash is ALL IDENTICAL. One
  test-isolation bug found and fixed during verification (a leaked debug-view/particle-loop state
  breaking a later pre-existing v1.78 assertion). See CHANGELOG for the full writeup.
- **v1.81 — wildlife-informed foraging gives a real water-range lever, not just food.** Owner:
  "any journey is only factually limited by the longest distance one is able to travers with the
  resources they can carry... we already have fauna information and therefore a good idea about
  how foraging could extend a route/travel distance." Measured before designing: a broad sample
  found terrain-legality/preset-bug dominated blocks (water rare); a TARGETED synthetic sweep
  (well-provisioned camel caravan, waterless desert) found the real cliff — `jpAssessResupply`
  blocks with zero elasticity past ~150-200 km, 269-506% over capacity, no lever to pull. Two
  design forks resolved via `AskUserQuestion`: extend the existing Foraging dropdown (not a new
  control), wire in real `currentWildlife()` regional richness (not the flat table alone). New
  `JP_BIOMES.waterForage` column (steeply biome-dependent, near-zero in true desert);
  `_jpWildlifeForageMod` compares a stage's sampled richness to the world's own mean (never an
  absolute cutoff); `jpForaging` gained optional trailing `mx,my` and a `waterReduction` return;
  `_jpDeriveStages` now carries each stage's own midpoint coordinate. `foraging="None"` stays an
  exact no-op. **Disclosed, not fixed this pass**: Active foraging's speed cost can outweigh its
  consumption benefit on an already-marginal single-carry stretch — measured, confirmed
  pre-existing (reproduces identically on unmodified v1.80), this version's water term improves it
  without resolving it; deferred as a materially larger change than what was asked. Two
  pre-existing environmental smoke-suite failures (`v0.92`/`v0.87` canvas-sizing) observed during
  verification, confirmed unrelated by reproducing identically against unmodified v1.80. Hash vs
  v1.80 ALL IDENTICAL. See CHANGELOG for the full writeup.
- **v1.80 — the wind/current streak animation (v1.78) never actually rendered.** Owner: "No
  animation in the flow layers." Root cause: `_windFxStart()` cleared the canvas's INLINE
  `display` style to `''`, expecting that to reveal it — but the CSS stylesheet rule for
  `#windFxCanvas` already sets `display:none`, and an empty inline value falls back to the
  stylesheet rather than overriding it, so the canvas stayed 0×0 on screen while the particle
  loop ran perfectly (and invisibly) underneath. Confirmed via real-screenshot frame-diffing
  (zero byte-diff across 5 animation frames pre-fix; ~90-95k bytes of diff per frame pair after).
  Fixed with an explicit `display='block'`. The v1.78 smoke assertion for this exact path checked
  the wrong DOM property (`cv.style.display`, not `getComputedStyle(cv).display`) and so passed
  despite the real bug — corrected in place. Hash vs v1.79 ALL IDENTICAL. See CHANGELOG for the
  full writeup.
- **v1.79 — addon villages now cluster with nearby siblings instead of each one individually
  beelining to the closest big settlement.** Owner, mid-session: *"the roads from the deeper
  settlement layers dont connect to their nearest siblings and individually connect to the
  closest big settlement. They probably just connected to the closest main road by the most
  efficient route."* Civ-layer only (`_civConnectVillageAddons`). Measured first (seed 31337/
  512px): v1.71-v1.78 could only ever target a real settlement, never another village — 79.5% of
  villages had a nearer sibling than the settlement they actually connected to, mean connector
  21.8 km vs. mean nearest-sibling distance 14.0 km. Three designs (single-shot Dijkstra with
  villages as extra targets; tap into the nearest road point; a genuine growing forest) were put
  to the owner via `AskUserQuestion` — growing forest chosen, the only one guaranteeing every
  village still traces back to a real settlement. Rebuilt as a BATCHED Prim's algorithm (one
  shared multi-source Dijkstra per round from settlements ∪ already-joined villages, attaching the
  cheapest batch before regrowing) rather than one Dijkstra per village, which v1.71's own comment
  already measured as a full extra order of magnitude too slow at 200 villages. Re-measured: mean
  connector 21.8→14.5 km, nearest-sibling-closer share 79.5%→36.5%; a separate chain-integrity
  probe confirmed zero villages left networked only among themselves. Hash vs v1.78 ALL IDENTICAL
  (interactive-only). See CHANGELOG for the full writeup.
- **v1.78 — terrain coupling made unconditional + Layer-view fix + animated wind/current
  streaks.** Owner, immediately after v1.77 shipped: *"Wind and current should always be coupled
  to terrain therefore the toggle is unneeded. Has the Layer view been updated accordingly? And
  can we also have the animation as in the PoC"* — three asks in one message. Engine only. **Not**
  bit-identical at defaults — a deliberate, measured re-baseline (same class as v1.36/v1.39/
  v1.46/v1.60). 1017 / 852 green; 618 smoke green; hash battery diverges on
  `field`/`temp`/`rain`/`flow`/`rgba` in every scenario, disclosed below as intended.
  - **Toggle removed, not defaulted true.** `state.climate.terrainWind` is deleted from the
    state literal; `buildWind`'s guard changed from `if(c.terrainWind && opts && opts.elev)` to
    `if(opts && opts.elev)`; `oceanSSTAnomaly` lost its `terrainOn` branch outright. `loadZip`'s
    v1.77 compat guard now deletes the stale field from a reopened v1.77 save instead of
    reintroducing it. Checkbox/sync/listener all removed.
  - **The Layer-view question had a real answer: no, it hadn't been.** v1.77's own CHANGELOG
    disclosed `currentWindField()`/`currentOceanField()` as unwired. Fixed:
    `currentWindField()` now threads elevation into `buildWind` like every real call site;
    `currentOceanField()` — which used to derive its "current" by zeroing the wind vector outside
    ocean cells — now calls the real `computeOceanCurrent()` and returns the genuine
    Ekman-rotated 2D field. Asserted: the Ocean Layer view's vectors are measurably distinct from
    a masked copy of the wind field.
  - **Animated streaks, ported from the PoC's own feature.** New `#windFxCanvas` in
    `.canvas-stack` (inherits the shared pan/zoom transform off-LOD; own reprojection under
    Tiled LOD via `_windFxProject`/`_windFxBounds`, matching `drawLODDebugOverlays`' `px()/py()`
    idiom). 260 wind / 200 ocean particles sample the same Layer-view data functions and redraw
    via `destination-out` compositing for a fading trail. `_windFxSync()` (called from the
    debug-view button handler + once on initial load) starts/stops the loop; the running rAF loop
    self-terminates by re-checking `state.debug` every tick — one reliable trigger needed, unlike
    the sculpt joystick's many `_sculptNavSync` call sites.
  - **Bug found and fixed before shipping: Wind→Ocean mid-animation crashed.**
    `_windFxStart()`'s original guard skipped reinitialization whenever anything was already
    running, so a kind switch kept sampling a stale, wrong-shaped field object — an ocean sampler
    reading a wind-shaped field with no `.ocean`, a hard `TypeError`. Root-caused via a
    Playwright `pageerror` handler capturing the full stack (a plain page-load probe found
    nothing — the crash needs a specific interaction). Fixed with `_windFxKind` tracking plus a
    mid-loop kind-change detection in `_windFxStep()` that reinitializes and explicitly
    reschedules.
  - **Disclosed consequence: `field` itself now differs, not just `temp`/`rain`/`flow`.**
    `generate()`'s default `carveRiverValleys()` pass carves the heightmap from traced river
    polylines, downstream of `flowField`, downstream of `rainField` — which the now-always-on
    coupling measurably changes. Confirmed via `hash_gen1.js`: `field` mismatches in every
    scenario. The correct closed-loop consequence of "always coupled," not a regression.
  - **Three test fixes, none touching app behavior**: the cold-current assertion's fixed
    threshold was too strict once real seed variance was exercised (redesigned around the raw
    `oceanSSTAnomaly()` field); the zonal-rain-belt-ratio assertion ran on an ambient random seed
    the coupling could occasionally push below threshold (pinned to reference seed 12345); and a
    v1.60-era isolated fuel-limited-settlement test's save/restore was missing
    `state.world_structure.enabled` — left `true` by an earlier unrelated smoke block, reshaping
    the pinned seed's geology enough to move every iron settlement off fuel-limited. Root-caused
    by instrumenting the real smoke run (a fresh reproduction passed and gave no clue).
  - **Known scope cuts**: the PoC's fuller moisture/cloud/snowpack/seasonal-ITCZ system remains
    deferred exactly as v1.77 scoped it. Streak particle counts/advection step are carried over
    from the PoC's own values, not independently retuned.
- **v1.77 — terrain-coupled wind & ocean currents, ported from the owner's PoC.** Owner: *"let's
  do the climate engine port"* — the go-ahead for work scoped in a prior session (via
  `AskUserQuestion`) to wind/current terrain-coupling + gyre/western-intensification, deferring the
  PoC's fuller moisture/cloud/snowpack/seasonal-ITCZ system, with two binding constraints: the world
  wrap must stay correct, and the new mechanic must genuinely feed rain/climate, not sit decoratively
  beside it. Opt-in (`state.climate.terrainWind`, default `false`) — bit-identical at defaults.
  - **Root cause, confirmed by reading the code before porting anything**: `buildWind` had zero
    direct terrain-blocking mechanism — wind blew straight through mountains, only reacting to
    temperature (itself only indirectly elevation-aware via lapse rate). `oceanSSTAnomaly` derived
    its sign/magnitude by reading `buildWind`'s own meridional component `wy` directly as a proxy
    for ocean current — no Ekman rotation, no distinct 2D current field, no gyre structure. Both gaps
    match exactly what the PoC's own method notes describe as missing from a naive wind-only model.
  - **New pure primitives**: `deflectFlow(u0,v0,block0,WW,WH,wrapX,opts)` (generic terrain
    deflection — damps the into-block flow component and redirects tangentially along the local
    block contour, iterated with blending; gap/strait acceleration from the block field's Laplacian;
    reuses the existing wrap-aware `blurCoarse` rather than re-implementing smoothing) and
    `computeOceanCurrent(wx,wy,elevC,WW,WH,wrapX,sea,latOf,opts)` (Ekman-rotated current run through
    the same `deflectFlow` against a hard coastline, plus shelf friction and a wrap-aware
    western-intensification scan — the PoC's own version wasn't wrap-aware since its demo domain
    never wraps, so this scan was rewritten using a two-pass priming technique for `westDist` and
    modulo-indexed `eastDist`).
  - **Wired into the real pipeline, not bolted beside it**: `buildWind` gained an optional `opts`
    param — when `terrainWind` is on and elevation is supplied, it runs `deflectFlow` against a
    land/mountain block field derived from real elevation, then blends the result back in damped by
    height (so high peaks deflect more than gentle slopes). `oceanSSTAnomaly` and `simulateWeather`
    (the actual rain-producing pass — semi-Lagrangian moisture advection along wind, orographic lift
    from real height gradients) both now pass real elevation through, so a real `refreshClimate()`
    pass measurably changes `rainField`/`tempField` when the toggle is on — not a debug-view-only
    field.
  - **Measured before shipping**: near-ridge deflection 3.7x the far-field effect (proves it's
    localized to terrain, not a global reweighting); World-mode wrap seam diff exactly zero when the
    test ridge is kept off the seam (the seam sitting AT a ridge peak legitimately splits flow —
    that's not a wrap bug, and the first version of this test conflated the two before being
    corrected); `rainField`/`tempField` mean deltas 0.040/0.268 through a real `refreshClimate()`
    pass (proves the "must feed the simulation" constraint, not decorative).
  - **A test-harness mistake caught before it became a false alarm**: an early probe pre-filled
    `wx`/`wy` with a synthetic uniform wind and expected `buildWind` to preserve/build on it — but
    `buildWind` always computes its own latitude-band base wind from scratch and overwrites whatever
    was pre-filled, so the apparent "wind reversal" was just the physically-correct trade-wind
    direction at that latitude, not a bug. Caught by diffing `buildWind`'s own off-vs-on output
    directly instead of trusting a pre-filled input to survive.
  - New UI checkbox "Terrain-coupled wind & currents" beside the existing "Ocean currents" toggle.
    `currentWindField()`/`currentOceanField()` debug-view functions deliberately left unwired to the
    new mechanism (disclosed scope cut). 1016 / 852 green; hash ALL IDENTICAL at defaults; +6 smoke
    assertions. See CHANGELOG for the full writeup.
- **v1.76 — village connectors: an unnecessary `.reverse()`, not a terrain problem.** Owner:
  *"how ways are done for the recently added Villages... it seems like a loopy bundle of spaghetti
  which is not how roads historically formed."* Root-caused by measurement, not inspection: median
  connector circuity was 2.62x straight-line, 54/199 self-intersected, and the worst offender's own
  path cost — recomputed from the same cost grid Dijkstra used — was 2.8x more expensive than the
  straight line despite near-flat terrain, proving the "shortest path" wasn't actually one. The
  Dijkstra `prev[]` walk already builds `raw` in village→…→settlement order, matching `aIdx`/`bIdx`
  — but `_civConnectVillageAddons` called an unnecessary `raw.reverse()` right after, flipping to
  settlement-first/village-last, then overwrote the endpoints as if it hadn't, corrupting every
  connector into "jump toward the destination, retrace the whole route backward almost to the
  village, then jump to the destination again." `aIdx`/`bIdx` were never wrong (only the drawn
  geometry), which is why existing endpoint-value smoke checks never caught it. Fix: delete the
  reverse — the two endpoint overwrites were already correct as originally written. (A first cut
  swapped which coordinate each overwrite targets instead; that produces the same geometry but the
  wrong point order, silently breaking the v1.71 `pts[0]`-is-the-village convention — caught by a
  full smoke run before shipping, not by inspection.) Re-measured: median circuity 1.12x, zero
  self-intersections (was 54). Civ-layer only; hash ALL IDENTICAL; +3 smoke assertions. See
  CHANGELOG for the full writeup.
- **v1.75 — `_civAutoRoutes` stamped way indices from two different arrays.** The HANDOFF-flagged
  latent defect left open by the v1.72 pass: `_civAutoRoutes`'s trunk-road ways carried
  `aIdx`/`bIdx` positions into its own filtered `settles` array while its village-connector ways
  (same call, `_civConnectVillageAddons`) carried positions into full `state.places` — two bases in
  one `civWays` list. Confirmed latent (the sole reader, `_civNetworkMetrics`, never consumes this
  particular output) but a real trap for any future/test reader. Fixed with a four-line remap
  (`settles.map(p=>state.places.indexOf(p))`) applied to the trunk ways right after they're built.
  Civ-layer only; hash ALL IDENTICAL; +2 smoke assertions using a deliberately-inserted POI to force
  the two index bases to collide and prove the remap (not just the absence of a crash). See
  CHANGELOG for the full writeup.
- **v1.74 — Tiled LOD zoom freeze: a colorized tile is a static image, and one composite per
  frame.** Owner: *"repeated quick zoom in-out actions cause a browser to freeze and become
  unresponsive."* Reproduced with real wheel events and root-caused by instrumenting call counts
  before writing any fix. Three scheduling defects; nothing about WHAT is drawn changed. 1016 / 852
  green; hash ALL IDENTICAL; +9 smoke assertions.
  - **Baseline measured (seed 31337 / 800km / 1024px, 64-tick wheel gesture):** worst rAF
    frame **13,217 ms**, p99 8,880 ms, 35 frames over 500 ms; `drawLODView` 350 calls / 226,950 ms;
    `renderBiomeTileRGBA` 491 calls / 243,382 ms (364 full tiles at ~609 ms + 127 overviews at
    ~171 ms). A 3-second gesture took 119 s of wall time.
  - **Result: worst frame 13,217 ms → 1,381 ms (9.6×); repeat-gesture colorizations 263 → 72,
    canvas misses 200 → 15.** A single 1024×655 colorization still costs ~618 ms on this headless
    box (the floor for any frame that colorizes at all — the budget can't preempt mid-tile); real
    GPU hardware should be faster.
  - **Defect 1 — the derived-pixel cache was smaller than the source-data cache.** 48 tile
    heightmaps kept, only 24 canvases. Since the canvas key carries no zoom/pan term, a
    re-colorization is always eviction, never invalidation. **`lodTileCanvasMax()` budgets by
    PIXELS**, sized from a MEASURED working set — a first cut at 48 MPx (my own back-of-envelope
    ~30-tile estimate) still thrashed; `probe_distinct.js`'s analytic sweep of the gesture's own
    zoom trajectory found the truth: **68 distinct tiles**, up to **16 visible at once** (not the 4
    a centred view shows). At 72 MPx the cache peaks at 68/72 and repeat-gesture misses drop to 15.
  - **Defect 2 — every high-frequency camera input composited inline**, one `renderNow()` per event
    (wheel / pinch / pan-drag / joystick / sculpt stroke): 350 composites for 128 wheel ticks.
    `requestLodRender()` coalesces to one per animation frame — pixel-identical output, plus a yield
    point between frames.
  - **Defect 3 — the single largest surviving stall wasn't a camera-input handler.**
    `_lodScheduleOverviewRebuild`'s own async completion callback carried its own inline
    `renderNow()`, firing 128 times in one gesture. Fixing Defects 1+2 alone cut total work 2.4× but
    left the worst frame at 12.2 s until this fifth call site was routed through
    `requestLodRender()` too — THAT is what actually moved `drawLODView`'s max from 11,162 ms to
    1,077 ms. Grep every `renderNow()` reachable from an LOD-camera-adjacent path, not just the
    obvious pointer handlers.
  - **Per-frame colorization budget (12 ms) on the interactive path only.** `drawLODView` stops
    colorizing past budget and lets the overview show through (what it already does for a tile whose
    height data isn't ready), always doing ≥1 so it converges. A direct `renderNow()` still
    composites the whole view. The debounced settle refine IS budgeted; the two `withBusy()` refine
    buttons are not.
  - **Ruled out by measurement — don't re-chase:** v0.93's stretch fast path works (streak peaked at
    1 of 4); `pyramidTile` ran **0 times** (not tile generation); `sharedSeaFields()` is properly
    cached.
- **v1.73 — label collision reserved one box and drew in another.** Bug-hunt continuation past
  the village layer. v1.28 added trait-badge clearance to the label DRAW path
  (`_civDrawSettlementPin` pushes a 'below' label clear of the badges) but not to v1.12's
  label-collision RESERVATION, so such a label painted outside its own reserved box and through a
  neighbour's. `_civTraitDrop()` is now the single definition both sides read; only the 'below'
  candidate moves and it returns 0 for a trait-less place, so everything else reserves exactly what
  it did before. Measured: one real overlap at deep zoom on the reference world, gone after the fix.
  1016 green; hash ALL IDENTICAL; +3 smoke assertions.
  - Ruled out during the same hunt (don't re-chase): the label WIDTH heuristic is conservative
    (never under-reserved across 35 labels, worst ratio 0.935); `jn.stops` is dropped on save but
    nothing reads it back; `_civBakeKey` omits civ state deliberately (separate overlay canvas);
    v1.71's connectors do render at their reveal zoom (14,840 px across 199).
- **v1.72 — bug hunt: three v1.71 village-connector defects, all one missing tag.** A
  deliberate hunt over the v1.68–v1.71 village layer, each defect measured on a real world
  before fixing. Civ-layer only. 1016 / 852 green; hash ALL IDENTICAL; smoke green (+7 new).
  - **A (HIGH, rendering):** the way serialization whitelist dropped `villageAddon`, so a
    save→reload left 199 connectors as ordinary `'ancient'` ways visible from zoom 0.7 while
    their villages stayed hidden until 2.4 — a web of roads to invisible settlements in any
    reopened project. Places kept the flag (state is deep-cloned wholesale); that asymmetry is
    what made it visible.
  - **B (HIGH):** "Generate Roads" kept only `w.manual` ways (destroying all 199 connectors)
    and then fed the villages to the trunk-network builder, producing 226 normally-visible
    ways to them. Villages are now excluded from the trunk network and connectors regenerated
    onto it. **Side effect: 3919 ms → 397 ms.**
  - **C (MEDIUM):** the way list is not virtualized; 199 unnamed connectors buried the ~53
    authored roads (252 cards). They now sit in a collapsed `<details>` — 54 top-level cards,
    all still reachable.
  - **Latent, deliberately unfixed:** `_civAutoRoutes` builds way `aIdx`/`bIdx` against a
    filtered `settles` array while the other paths use full-`state.places` indices. Divergent
    only when a POI exists, and `_civNetworkMetrics` (the sole reader) is never called on that
    path — so nothing misreads it today. Noted rather than refactored speculatively.
- **v1.71 — addon villages connected by a low-tier "Ancient route."** Owner, immediately after
  v1.70 shipped: "the new settlements on the deeper level also need to be connected. By a lower
  type road (ancient route for example) so it only shows when zoomed in." Civ-layer only.
  1016 / 852 green; hash battery ALL IDENTICAL; 581 smoke green (572 + 9 new); see CHANGELOG for
  the full writeup.
  - `_civConnectVillageAddons` connects every addon village to its nearest REAL SETTLEMENT with a
    new `type:'ancient'`, `villageAddon:true` way, gated on `CIV_VILLAGE_ADDON_LOD` via a new
    `_civWayLodMin` helper — the road reveals together with its village, not at the generic
    ancient-road threshold (0.7).
  - First cut targeted the nearest existing WAY cell (a T-junction) instead of a settlement, and
    measured broken: `_civNetworkMetrics` only recognises settlement-to-settlement edges, so a
    short spur's far endpoint fell back to nearest-place-by-coordinate — which usually found the
    VILLAGE ITSELF, an `aIdx===bIdx` self-loop silently dropped. All 200 test villages read
    isolated despite 161 having a drawn connector. Routing to the nearest settlement instead
    fixed it by construction (guaranteed distinct, valid `aIdx`/`bIdx`).
  - `roadDijkstra` gained an additive multi-source form (`sx` may be an array of cell indices) so
    every village's nearest-settlement connection is found in ONE shared full-grid search instead
    of one Dijkstra call per village (up to 200). Every pre-v1.71 scalar call site is bit-identical.
  - Measured (seed 31337, 800km/512px, 200 villages): 199 connectors, 1 genuinely unreachable
    island fragment, every connected village correctly non-isolated post-fix.
- **v1.70 — dense grid and roadside villages merged into one suitability-weighted, road-biased
  pass.** Owner, immediately after v1.69 shipped: "mix the dense village function and the
  roadside village function into something more nuanced? And only come into view when our zoom
  is at about 60% zoomed in." Owner picked "Suitability-weighted road bias" via
  `AskUserQuestion`. Civ-layer only. 1016 / 852 green; hash battery ALL IDENTICAL; 590 smoke
  green (572 + 18 new); see CHANGELOG for the full writeup.
  - v0.76's `villageMode` is retired from BASE placement entirely — it used to densify EVERY
    tier at once, which was the actual "waay too populated" cause. Base placement is now
    byte-identical regardless of the new toggle; only the additive layer responds to it.
  - `_civSeedVillages` draws candidates from the whole suitability grid (dense mode's own
    technique) but each candidate clears a SOFT probabilistic accept test —
    `_civVillageAcceptProb`, `max(roadProb, suitProb)` — instead of an unconditional accept.
    Road proximity raises odds with a smooth exponential falloff; genuinely great land
    (`suit>=SETTLE_SEED_THRESH`) qualifies regardless of distance; below the relaxed floor
    (`VILLAGE_SUIT_THRESH=0.32`) a cell is never even a candidate.
  - `CIV_VILLAGE_ADDON_LOD=2.4` (was v1.68's `CIV_ROADSIDE_VILLAGE_LOD=2.0`, "~50%"), scaled by
    6/5 for the new "~60%" ask. `p.roadsideVillage` renamed `p.villageAddon`.
  - One checkbox/flag now: `civVillagesChk`/`_civVillages` replaces both
    `civVillageDensityChk`/`_civVillageDensity` (v0.76) and `civRoadsideVillagesChk`/
    `_civRoadsideVillages` (v1.68).
  - Measured (seed 31337, 800km/512px): 35→235 settlements with the toggle on (200 addon,
    hit the cap); 200 accepted with real roads vs. 192 with none in a direct pure-function
    comparison; 76% of accepted villages sit within the road-proximity search window.
- **v1.69 — roadside villages now also factor in settlement suitability.** Owner, immediately
  after v1.68 shipped: "They should still also factor in settlement suitability." v1.68's
  seeding pass walked the road network and spaced candidates against existing settlements, but
  never consulted `currentSettlementSuitability()`. Civ-layer only. 1016 / 852 green; hash
  battery ALL IDENTICAL; 568 smoke green; see CHANGELOG for the full writeup.
  - `_civSeedRoadsideVillages` now takes the SAME `suit` field the rest of auto-populate reads
    (threaded in, not re-derived) and, at each arc-length step, searches a small local window for
    the highest-suitability dry cell instead of taking the raw interpolated point.
  - A window with no cell clearing `ROADSIDE_VILLAGE_SUIT_THRESH=0.32` (the same relaxed
    threshold dense-village mode already uses) is skipped outright.
  - Measured: 112 villages added (down from 120 — marginal sites v1.68 force-placed are now
    correctly skipped), min score 0.320, mean 0.428.
- **v1.68 — roadside villages: a sparser alternative to the dense grid, revealed only at
  deep zoom.** Owner: the existing "Dense village grid" option (v0.76) "sometimes feels waay
  to populated on the map" — keep it, but add a sparser alternative for when it's off, villages
  spaced along the settlement generator's own routes, visible only once zoomed in roughly
  halfway toward a close view. Confirmed the persistence-model fork (real settlements vs. a
  decorative overlay) with the owner via `AskUserQuestion` before building — real settlements
  chosen. Civ-layer only. 1016 / 852 green; hash battery ALL IDENTICAL; 566 smoke green; see
  CHANGELOG for the full writeup.
  - `_civSeedRoadsideVillages` walks the finished road network at `VILLAGE_SPACING_KM` arc-
    length steps, blue-noise-rejecting candidates via v1.26's own bucket-grid technique; runs
    after routing settles, before population assignment, so new villages are real hamlets
    swept into the same food-shed/economy math as any other settlement.
  - Zoom-gating reuses `drawCivLayer`'s existing `CIV_LOD_PLACE` convention (one raw zoom
    number compared directly for either camera) rather than inventing a new percentage system
    — a new `CIV_ROADSIDE_VILLAGE_LOD=2.0` threshold, deeper than every existing tier, with no
    dot-fallback below it (fully hidden, not just faded).
  - Known scope cuts: the reveal threshold isn't independently calibrated against a literal
    percentage (none exists in this file to calibrate against); `_civAutoRoutes` alone doesn't
    re-seed roadside villages; only the primary map-click pick site is zoom-gated.
- **v1.67 — a water-driven convergence loop could return a physically absurd, unblocked
  stage.** Owner pasted a full "Severe" Journey Planner verdict — an 18-month, 4253 km journey
  with stages at 525%-1475% of capacity and no hard block — "Somehow it feels like it is way
  too long for travel time." Root-caused by reproducing the report's own Stage 11 numbers before
  writing any fix: v1.63's `JP_LOAD_INVALID_RATIO` check only ever saw the UN-iterated `ratio0`
  (0.82, read as fine); the convergence loop's OWN water term is a real feedback loop (slower
  speed → longer gap in days → more water → more load → slower speed) that converged at 15.3×
  capacity while `jpLoadPenalty` silently floors at 0.45× past 150%. Civ-layer only
  (`jpCalcLand`). 1016 / 852 green; hash battery ALL IDENTICAL; 557 smoke green; see CHANGELOG
  for the full writeup.
  - Fix: the SAME `JP_LOAD_INVALID_RATIO` cutoff is now also checked on the post-loop
    `loadRatio`, not just `ratio0` — one `if`, right after the convergence loop.
  - Fixing this surfaced 3 pre-existing v1.56 smoke assertions that were themselves accidentally
    overloaded (a synthetic 3000 km dry gap for a 4-person Walking party with zero animals) — the
    same shape v1.63's own entry already named. Fixed by shrinking the scenario to a genuinely
    carriable 110 km, not by loosening the new check.
  - Known scope cuts: no change to the loop's iteration count, `jpLoadPenalty`'s curve, or any
    terrain/weather/infrastructure table — only the missing post-loop capacity check.
- **v1.66 — per-stage pack-animal + vehicle fine-tuning, with a swap advisory.** Owner: a
  2-person party travels moderate climate for 2/3 of a route then desert, wanting to swap
  mule+cart for camel+travois at the transition; "For now I cant make any such a finetunement."
  Investigated first — the water/food math was already species- and per-stage-correct; the real
  gaps were no per-stage vehicle control and no auto-detected advisory. Owner picked "auto-detect
  + advisory button" via `AskUserQuestion`. Civ-layer only. 1016 / 852 green; hash battery ALL
  IDENTICAL; 552 smoke green; see CHANGELOG for the full writeup.
  - `_jpBestPackageForStage` (species/vehicle twin of v1.53's `_jpBestLandTransportForStage`) +
    a new per-stage Vehicle override (None/Cart/Wagon/Travois/Sled, mirroring v1.50's Pack
    animal select) + a "Use here" advisory button writing both together.
  - Verified against the owner's literal two-stage scenario: moderate stage shows no advisory,
    desert stage recommends camel, applying it touches only that one stage.
  - Known scope cuts: Mounted Rider's mount species not covered; sleds manual-only; vehicle
    sizing/existence stays the whole-route auto-picker's job.
- **v1.65 — Journey Planner: one-click auto-fix buttons for stage bugs.** Owner: "when a stage
  gives a bug give a button to automate a fix." Extends v1.53's "Use here"/v1.47's "Re-route"
  advisory-button pattern to blocked-stage trouble cards. Civ-layer only. 1016 / 852 green; hash
  battery ALL IDENTICAL; 544 smoke green; see CHANGELOG for the full writeup.
  - Three subcases with a deterministic, side-effect-free fix get a button: winter-closed pass
    (turn off seasonal closures), mount-blocked/baggage-train-without-animals (switch to
    Walking), wheel-vehicle-present (clear carts/wagons + switch to Walking — clearing carts
    alone was tried first and just traded one wheel-block message for another, caught by testing
    the button end-to-end).
  - A vessel swap and any cargo/party-size change stay text-only — treated as the user's own call.
  - Every fix verified by clicking the actual rendered button and confirming the stage unblocked,
    not assumed from reading the code.
- **v1.64 — auto-generated roads preserve and prefer manually-drawn ways.** Owner: "on parts of
  routes and ways, when applicable always follow them as they are optimized." Investigated first
  (the phrase was ambiguous across three candidate mechanisms) — the manual Route/Way tools already
  had a real discount (v1.53); the auto-generated network never did, and silently destroyed manual
  ways on every "Generate Roads" run. Civ-layer only. 1016 / 852 green; hash battery ALL IDENTICAL;
  537 smoke green; see CHANGELOG for the full writeup.
  - `_civAutoRoutes()` used to open with `civWays=[]`, destroying every manual way. Now manual
    ways (`w.manual===true`) are preserved across a rebuild instead of discarded.
  - `_civHierarchicalNetwork` gained optional `opts.existingWays`: cells along a supplied way get
    the same `_CIV_EXISTING_WAY_DISCOUNT=0.25` the manual tools use (v1.53), via a shared helper so
    the two mechanisms can't drift apart. `_civAutoRoutes` feeds its preserved manual land ways in.
  - Measured: a settlement pair the base network didn't connect directly went from 30 to 134
    usage-count on the manual way's own cells and became a direct edge once discounted.
  - Known scope cuts: `_civMstRoutes` (sea-lane MST) not threaded with this; `_civIterativeAutoWorld`
    ("Auto World") regenerates settlements from scratch each run so preservation wasn't extended
    there (only the standalone "Generate Roads" button, which leaves settlements untouched).
- **v1.63 — Journey Planner: an impossible load is finally flagged, not silently crawled at
  45%; Small Caravan gets its own +20% back.** Owner supplied a research prompt diagnosing two
  findings from a fresh audit of the load-penalty/coordination chain, plus three lower-priority
  confirmations and two unverified-value flags. Civ-layer only. 1016 / 852 green; hash battery
  ALL IDENTICAL; 531 smoke green; see CHANGELOG for the full writeup.
  - **Finding 1**: `jpLoadPenalty`'s curve floored at a flat 0.45 for ANY ratio past 1.50 — a
    22×-166× overloaded stage read identically to a 1.51× one. `JP_LOAD_INVALID_RATIO=1.50`
    reuses the curve's own existing top boundary as an invalidation cutoff (checked on the
    UN-iterated `ratio0`, before the convergence loop) — above it the stage is `blocked`, not
    silently computed. Every graduated band at or below 1.50 is untouched.
  - **Finding 2**: Small Caravan (≤10) carried a neutral `coordMod:1.00` instead of the
    +15-25% bonus travel-speeds.md §5 actually describes. Set to 1.20. Individual and the three
    tiers above Small Caravan are unchanged.
  - Confirmed unchanged: `JP_GRAZING` scale ordering, sea-leg weather/biome linkage,
    `jpCalcWater` needs no changes. Flagged unverified in code comments: Snow/Ice terrain
    modifier (0.55), Galleon cruise speed (13 km/h).
  - Fixing this surfaced several pre-existing smoke-test scenarios that were themselves
    accidentally overloaded (synthetic test plans with a `cargoKg` sized for a different
    baseline) — fixed by giving each a baseline it can actually carry, not by loosening the
    new check.
- **v1.62 — settlements no longer land on top of each other, even across opposing factions.**
  Owner: "settlements being created on top of each other even from oposing factions now."
  Root-caused by ablation (disable one candidate mechanism at a time, re-measure) before any fix.
  1016 / 852 green; hash battery ALL IDENTICAL; 524 smoke green; see CHANGELOG for the full writeup.
  - **The v1.46 coastal-preference swap relocates a landmass's worst non-port settlement onto a
    fresh coastal candidate and never checked that candidate against settlements already standing
    on the landmass.** `byLandmass` groups by LANDMASS, not faction; since v1.58 let several
    factions share one landmass, this could drop one faction's settlement directly onto a rival's.
  - **Measured, not assumed**: an 8-seed sweep found overlapping pairs (several at 0 km) on 5 of 8
    seeds, including cross-faction pairs. Disabling `_civOceanDistField` (the v1.46 smoke test's own
    technique for turning the swap off) eliminated every overlap; disabling the water-edge snap did
    not — isolating the coastal swap as the sole cause.
  - **Fix**: reject a swap candidate within `suppR` (already computed, the same value every other
    placement pass in the function uses) of any OTHER settlement. Four lines, civ-layer only.
  - **Known scope cut**: the crossroads-settlement snap and the all-settlement water-edge snap
    share the same "move without checking siblings" shape in principle, but the ablation showed
    neither produces overlaps in practice on the sample tested — left alone, not fixed speculatively.
- **v1.61 — LOD tile refinement: one bad tile can no longer take its neighbours down with
  it.** Owner report (screenshot): a rectangular block of Tiled-LOD tiles permanently stuck on
  the coarse/unshaded overview — deep zoom, plain Biome view, no bake involved, panning away and
  back never fixed it. Investigated thoroughly before any fix: audited every LOD camera-move path
  for a missing `scheduleLodRefine()` call (none found), checked v1.60's new functions against the
  Worker's function whitelist (not the cause), ruled out stale baked tiles (owner confirmed no
  baking), attempted reproduction via Playwright across 6 seeds with forced refine cycles and
  aggressive fast zoom/pan (never triggered a stuck tile). **The exact trigger was not
  reproduced this session** — but the audit found a real structural gap worth fixing regardless.
  1016 / 852 green; hash battery ALL IDENTICAL; 521 smoke green; see CHANGELOG for the full writeup.
  - **Neither of `refineVisibleTiles()`'s two compute paths (Web Worker pool, sync main-thread
    fallback) isolated one tile's failure from its neighbours.** Worker side: an uncaught throw
    inside a job never reaches `postMessage` — it fires the main thread's `onerror` instead, which
    rejects the WHOLE batch dispatched to that worker (several tiles die together, jobs are
    round-robin split). Sync side: the fallback loop had no per-iteration try/catch, so a throw
    stopped it partway through, leaving every tile queued after the bad one uncached too. Both
    failures were silently swallowed by `scheduleLodRefine()`'s outer `catch(_){}` — matching the
    report's "no loading indicator, nothing visibly wrong except the stuck block" exactly, and
    matching "permanently stuck" since the same tile position recomputes identically every time.
  - **Fix**: per-job try/catch in the worker loop (pushes `null`, already the existing
    skip-on-falsy convention) + matching per-iteration try/catch in the sync fallback, both now
    `console.warn`ing instead of swallowing. A failed tile is simply left uncached — the ordinary
    "not yet refined" state, retried on the next debounced settle — instead of taking siblings
    down with it.
  - **`bakeVisibleTiles()`/`bakeAllTiles()` share the identical latent shape, deliberately not
    touched this pass** — baking wasn't implicated in the report, and a finalized-world baking
    failure (silently missing atlas chunks meant to be permanent) deserves its own dedicated look.
  - **Known scope cut, disclosed**: the root TRIGGER was never identified. This fix prevents it
    from ever manifesting as a silent, permanent, multi-tile block again, and the new
    `console.warn` makes it diagnosable (exact `z/col/row` + error message) if it recurs — but
    "why did `pyramidTile` throw at all" is genuinely still open.
- **v1.60 — real-km-aware relief and rivers: small regions get genuinely finer drainage
  detail.** Owner: "when choosing a smaller region rivers dont become more visible (i think its
  a scaling issue). I cant seem to find any rivers with the branching pattern or length you might
  expect... check any and all information on river formation and how simulations/terrain
  programs realistically generate them and apply them with proper scaling." Scope decided via
  `AskUserQuestion`: fix everything, including the underlying terrain relief, not just the river
  threshold. `docs/research/scale-invariant-terrain.md` (new). 1016 / 852 green; hash battery
  shows the disclosed Stage-A-only mismatch at defaults (see below); 516 smoke green; see
  CHANGELOG for the full writeup.
  - **Root cause, measured before any code changed**: (1) the relief pipeline
    (warp/heterogeneity/height-formula noise) samples at a frequency fixed to a fraction of grid
    width, never real km — a 50 km region and a 40,000 km world at the same resolution produced
    statistically identical relief; (2) crater/volcano radii convert real km to cells with only
    floor clamps, reaching **2.8× the grid width** at a 50 km region; (3) the river
    channel-initiation threshold (`GW*GH*0.0004`) is likewise a pure grid-cell fraction,
    independently reimplemented at ~18 call sites, ungrounded in any real km² (Montgomery &
    Dietrich's ~0.1-1 km² channel-initiation area).
  - **`terrainDetailK(gw,mapWidthKm)`** (new) — one-sided `Math.min(16,Math.max(1,
    REF_CELLKM/cellKm))` anchored at the app's own literal default (`mapWidthKm:800`,
    `resW:2048`), the same anchor-at-the-default discipline `_V3D_RATIO0` (v0.67) already uses.
    `k===1` exactly at/above the reference scale (world mode, or any region ≥800 km at ≤2048
    resolution — the common case), so it's provably bit-identical to v1.59 there. Threaded into
    `heightParams().nf` and `heteroParams().hf` only — warp frequency and `state.tect.blurR` stay
    grid-relative (disclosed scope cuts).
  - **`clampFeatureRadiusCells`** caps a single crater/volcano at 12% of the shorter grid axis —
    universal, not scale-gated (a crater covering the whole map is wrong at any resolution).
  - **`riverFlowThresh(gw,gh)`** consolidates ~18 inline recomputations into one function AND
    divides by the same `terrainDetailK` — added only after measurement showed Stage B alone made
    a 50 km region's drainage network measurably SPARSER (channel cells 4233→3345, finer relief
    fragments the long downhill runs flow accumulation needs). Dividing by `terrainDetailK`
    recovered it: channel cells →**6001** (+42%), polylines 718→**1497** (+109%) at 50 km; channel
    fraction **12×**/**28×** higher at 100 km/25 km on a 1024-res world vs. the unchanged default.
  - **Bit-identity, disclosed precisely**: the standard `hash_gen1.js` battery mismatches at every
    scenario vs v1.59 — this is the crater/volcano clamp ALONE (not scale-gated, fires at any
    resolution depending on the random roll, including the literal default). An isolated A/B with
    craters/volcanoes disabled proves `terrainDetailK` is bit-identical to v1.59 at reference
    scale. A deliberate, measured re-baseline — same class as v1.36/v1.39/v1.46's placement fixes.
  - **Two smoke assertions needed new fixtures**, both root-caused to the crater/volcano clamp
    reshaping a specific hardcoded seed's terrain/geology (verified by independent probe before
    touching the test, not assumed): the v0.94 routing-fix regression's two coordinate pairs, and
    the v1.31 §10.3 fuel-limited-settlement check (isolated onto its own dedicated fresh world in
    a save/restore block, matching v1.46/v1.58's test-isolation precedent, instead of depending on
    whatever world ~40 assertions of shared ambient smoke-suite state happened to leave behind).
  - **Known scope cuts**: warp frequency and `state.tect.blurR` stay grid-relative; erosion
    kernels, coastal/glacial passes, `carveRiverValleys`, and fjord masking were all audited and
    confirmed already resolution-relative, so none needed changes; `TERRAIN_DETAIL_MAX_K=16` and
    `FEATURE_RADIUS_MAX_FRAC=0.12` are reasoned, not independently historically calibrated.
- **v1.59 — Civilization menu reorder: faction creation leads into world generation.** Owner:
  "completely redesign and rethink the civilisation menu's under generate and make it a bottom up
  system that populates on the map as it does at the moment... Refactor and Consolidate the
  Generation → Civilization menu from the ground up putting the menu's in a logical order of
  faction creation that leads up to the autopopulate (auto routes and settlements) function." Pure
  civ-layer HTML/DOM reorganization — no engine/UME change, every id/handler unchanged, only
  physical placement moved. Hash vs v1.58 ALL IDENTICAL. 1001 / 852 green; hash battery ALL
  IDENTICAL; 516 smoke green; see CHANGELOG for the full writeup.
  - `#civSubBar` reordered: Generation moves from last to 2nd, right after Factions — **Factions →
    Generation → Settlements → Economy → Statistics** (was Factions → Settlements → Economy →
    Statistics → Generation, v1.55's ordering, which left the world-gen trigger buttons dead last
    behind three post-generation report pages that read empty until Auto-populate has run). No JS
    change needed — the tab click handler keys off `dataset.civsub`, not DOM position.
  - `#civSubGeneration` restructured into an explicit **Step 1 (populate) → Step 2 (roads) → Ways →
    Step 3 (territories) → Provinces → Display** sequence, dissolving the old "Advanced" catch-all
    `<details>` — Ways/Provinces promoted to always-visible sections (Provinces stays after Step 3
    on purpose, since `_civGenerateProvinces()` needs `civTerritory` populated first); the four
    map-styling sliders isolated into their own "Display" accordion.
  - The Territory-paint brush radius (`civTerRadius`) moved out of Generation entirely into a new
    contextual row (`civTerritoryToolRow`, beside `civPoiTypeRow`), shown only while the Territory
    tool is armed — it's a tool-brush parameter, not a generation setting.
- **v1.58 — political fragmentation on a single landmass.** Owner, on the v1.57 scope cut below:
  "if there is only 1 continent it should lead to a division of the continent, based on geography
  and industrial prowess." `docs/research/political-fragmentation.md` (new). Hash vs v1.57 ALL
  IDENTICAL. 1001 / 852 green; hash battery ALL IDENTICAL; 508 smoke green; see CHANGELOG for the
  full writeup.
  - `_civAssignLandmassFactions()` (new) replaces the flat one-faction-per-landmass assignment.
    Spare faction ids (`factionCount>landmassCount` — only ever nonzero when there's real unused
    capacity) are apportioned across landmasses by highest-averages (real seat-apportionment
    method) weighted by summed settlement suitability (already the file's own unified geography+
    resource signal). Extra seats become extra capitals, seeded by suitability + blue-noise spacing
    (v1.26's scatter idiom); every other candidate joins its nearest capital. The actual border is
    left to the EXISTING `_civAutoPolity` ("Recalculate Territories") flood-fill, unchanged — it
    already costs steep terrain expensive, so mountain ranges become real borders with zero
    engine/UME changes.
  - Byte-identical to the pre-fix code whenever `factionCount<=landmassCount` (verified directly,
    same seed) — a strict generalisation, not a special case. Measured on a real world (seed
    12345, 6 factions, 2 landmasses): before 27/2/0/0/0/0 (2 factions used), after 6/2/16/14/2/6
    (all 6 used, 5 capitals spread across both landmasses proportional to capacity).
  - A pre-existing v1.46 smoke assertion (coastal-preference pass, ON vs OFF across a fixed
    3-seed sample) needed to become statistically honest — v1.58 widens achievable capital count
    per landmass, which widens a pre-existing cross-run RNG-cascade noise source enough to
    occasionally flip one seed. Widened to 8 seeds, changed to an aggregate "never net-worse"
    comparison; `CIV_FACTIONS` pinned for the test's duration (same class of test-isolation gap as
    v1.24's BUG-3).
- **v1.57 — Factions pop-up + one editing surface per faction field.** Owner, right after
  reviewing v1.55: "I'd also very much love it to be in a pop-up menu." Also fixed a real
  duplicate-editing-surface bug the same review had already surfaced by inspection: Government/
  Culture/Religion/Ag. technology were editable BOTH inline in the quick-select pill row AND in
  the Faction Inspector drawer for the same four fields. Hash vs v1.56 ALL IDENTICAL. 1001 / 852
  green; hash battery ALL IDENTICAL; see CHANGELOG for the full writeup.
  - `#civFactionsModal` — the world overview/roster/detail-drawer moved into a full-screen pop-up,
    same shell contract as `#cityViewerModal`/`#routeEditorModal` (own `.open` toggle, own Escape
    handler, in `_overCanvasOverlay`'s and `_sculptNavSync`'s guard lists). The quick-select pills
    stayed in the sidebar (they drive `_civActiveFaction` for the map-authoring tools, which needs
    the canvas visible, not covered by a pop-up).
  - `_civBuildFactionPicker()` no longer builds the four per-pill selects — editing those fields
    now lives only in the Inspector drawer inside the pop-up. Two existing v1.07/v1.10 smoke
    assertions tested the removed selects directly and were rewritten (now assert zero selects on
    the pill + the Inspector's own selects still work and round-trip) rather than left broken.
- **v1.56 — water-constraint softening.** Owner: "for water now it quickly gives a hard
  constraint. Whilst in reality people often drank from streams/rivers/other smaller stops along
  a route... Suggest an adjustment to reflect this instead of the hard warning" — the two-part fix
  researched and presented in the prior session, now approved and built.
  `docs/research/water-access-travel.md` (new). Hash vs v1.55 ALL IDENTICAL. 1001 / 852 / 496 green.
  - Root cause: `_jpStageDryKm`'s freshwater test reused `flowThresh=GW*GH*0.0004`, the SAME
    constant `buildRiverNetwork` uses as its own order-1 channel-initiation bar (at default
    density, `channelThreshold` reduces to `thresh` exactly) — conflating "renders as a mapped
    river" with "a party can find a drink." `JP_DRINKING_FLOW_DIVISOR=16` (Horton's laws,
    Rb²≈9-36 two Strahler orders down; confirmed on a real generated world via new
    `tests/perf/probe_water_gap.js`: 59/60 sampled routes read "dry" at the old threshold, 28/60
    at ÷16) is applied ONLY inside `_jpStageDryKm` — rendering/`buildRiverNetwork` untouched.
  - The auto water-crossing tier (`JP_DESERT_WATER`) was `isDesert`-gated, so a non-desert biome
    with a genuinely long dry stretch got a flat, ungraduated 1.1× reserve and no speed penalty at
    any severity. The gate is removed for the AUTO path (any biome); the explicit override
    dropdown stays desert-only (its labels are desert-narrative, and it's only ever shown on a
    desert stage).
  - A pre-existing v1.51 smoke assertion relied on the ambient, heavily-mutated smoke-suite
    world's hydrology by chance to show a dry stretch — broken by the fix's own intended effect
    (fewer routes read as dry). Replaced with a controlled synthetic-`flowField` scenario that
    proves the same claim deterministically.
  - Known scope cuts: `jpAssessResupply`'s hard-block threshold/wording is untouched (fires less
    often, isn't loosened); sea/river water rules are untouched (land-stage-only change).
- **v1.55 — faction-first Civilization menu.** Owner: "I like the new civilization menu,
  implement it please in a logical fashion, maybe make it scroll into the screen from the left.
  Only showing a simplified version at first (that for examples only shows a global overview)" —
  approving and implementing the faction-first mockup/proposal from the prior session (which
  found, by audit, that faction culture had ZERO mechanical effect on settlement placement or
  territory — naming-flavor only). Hash vs v1.54 ALL IDENTICAL. 1001 / 852 / 488 green.
  - `#civSubBar` reordered Factions→Settlements→Economy→Statistics→Generation (was Generation-
    first); `_civSubTab` now defaults to `'factions'`.
  - `#civFactionsWrap` holds an always-in-flow **global overview** (world-summary line + quick-
    select pills + a richer roster) and a **detail drawer** (`#civFactionDrawer`, `.civ-drawer`)
    that slides in FROM THE LEFT on a row click — same `translateX`+`.22s ease` idiom the mobile
    `<aside>` panel already uses, mirrored to the opposite edge. `_civRefreshActiveSubPage()`
    closes the drawer on every entry into the tab, so re-visiting Factions always lands on the
    simplified overview first.
  - **Territory Fit** (new, in the drawer): `_civFactionAggregates()`'s existing single
    `O(GW·GH)` pass now also accumulates a per-faction terrain-mix (river/coastal/arid/forest/
    hills) + world-mean twin; `_civCultureTerrainFit()` compares a terrain-themed culture
    (highland/desert/riverlands/sylvan/maritime) against the world mean for a match/typical/
    mismatch verdict — `common`/`imperial` get composition-only, never a fabricated verdict.
  - **Found during verification, not by inspection**: making Factions the default tab surfaced a
    latent bug where `generate()`'s own wrapper reaches `_civFactionAggregates()` (via
    `_civRenderPlaceEditor()`→`_civRefreshActiveSubPage()`) BEFORE the real `generate()` body
    runs — crashed inside `plateCrust()` on an empty `plates` array (`generate()` must never
    throw — this file's own invariant). Fixed with a `plates.length` guard returning a safe
    all-zero shape, not cached.
  - Known scope cuts: Territory Fit is read-only (no placement bias); the roster row doesn't
    show Territory Fit at a glance (drawer-only); Settlements/Economy/Statistics pages unchanged.
- **v1.54 — agricultural technology as a per-faction axis.** Owner: the flat 9:1 farmer:urbanite
  ratio "doesn't sit against a civilisation having mastered the plow and sitting roughly at a
  level of industrial production, even so barely" — asked for research into how productivity
  actually scales and how to use it in auto-populate. `docs/research/agricultural-productivity.md`
  (new) grounds it in the England agricultural-labour-share series: 9:1/~90% is a *pre-improvement*
  baseline (closer to ancient/early-medieval than to "mastered the plow"); by the time a society
  genuinely masters rotation/drainage/selective breeding (England ~1700-1760) the ratio was already
  ~1:1; "barely industrial" (steam threshing, first chemical fertilizer, England ~1800) sat at
  ~0.54:1. Hash vs v1.53 ALL IDENTICAL. 1001 / 852 / 479 green.
  - **Scope, decided with the owner via `AskUserQuestion` before building**: per-faction (not one
    world setting), live-editable in the Faction Inspector (not setup-gate-only), read fresh every
    `_civFoodShed` call.
  - **New `AG_TECH_LEVELS` (6 rungs) + `civFactionAgTech`** (per-faction parallel array, same
    convention as Culture/Religion/Government — but NOT pure flavor, disclosed as such everywhere).
  - **Two real formula bugs found only by measuring, not by deriving on paper.** (1) The shipped
    `FOOD_BASE_SURPLUS_RATIO=1/FARMERS_PER_URBANITE` should be `1/(FARMERS_PER_URBANITE+1)` —
    population balance (R farmers + 1 urbanite, R yields must cover R+1 people) gives `1/(R+1)`, and
    `1/R` blows past 100% below R=2, which the fixed R=9 default never triggered. Pinned exactly at
    the historical constant for the default rung (every existing world untouched to the bit,
    asserted); corrected formula used for every other rung. (2) `FOOD_SURPLUS_RATIO_MAX=0.35` is a
    PRE-INDUSTRIAL ceiling, not a soil-quality ceiling — using it unscaled silently neutralised the
    whole feature on first real measurement (an "industrial" faction's food shed came out ~0.5%
    bigger than traditional's, because both saturated at the same flat cap). Fixed: the cap now
    scales with the rung's own base ratio (preserving the original best-soil-vs-median relationship),
    clamped at a new absolute 0.95. Re-measured: Early Industrial now gives **2.66×** Traditional
    Agrarian's food-shed capacity on a real settlement — the order-of-magnitude shift the research
    predicts, not the ~0.5% nudge the first cut actually shipped.
  - **Deliberately does not touch yield-per-hectare or carrying capacity** — the tech-level ratio
    governs what SHARE of the land's already-calibrated population ceiling must stay agricultural,
    which is exactly the real "agricultural labour share" metric this is calibrated from. Disclosed
    scope cut: no total-population-growth-from-industrialisation effect (nutrition/medicine/
    migration pulls) — would need to touch the carrying-capacity chain, deliberately left alone.
  - **UI**: Faction Inspector "Ag. technology" select (with live hint text) + a matching compact
    picker dropdown, same two-surfaces convention Government/Culture/Religion already use.
- **v1.53 — route drawing prioritizes existing infrastructure; a named per-stage transport
  advisory.** Owner audit: does each stage of a route get its own optimal transport, and does
  drawing a route between two settlements with an existing connecting way (e.g. a sea lane)
  actually follow it, rather than cutting across an intervening land strip? Travel times
  re-checked first (`probe_travel.js` — clean, unchanged from v1.52, not touched this pass).
  Both other claims were measured against a real generated+auto-populated world before any code
  changed. Hash vs v1.52 ALL IDENTICAL (civ-layer, interactive-only). 1001 / 852 green.
  - **Existing-infrastructure discount was dead code for `'mixed'` mode.** `_civDijkstraPath` has
    carried an "existing ways get discounted" comment since v0.6, but the sea-lane half was a
    `Math.min(cost,1.0)` cap — a no-op whenever the baseline was already ≤1.0, which is exactly
    `_civCommitRoute`'s `'mixed'`-mode open-water cost (`_CIV_SEA_COST=0.6`). Measured: two ports
    604 km apart by an existing sea lane instead drew a fresh 790 km, 97%-overland route — the
    lane lost by ~3.5% of path cost purely from getting zero infrastructure credit. Fixed to a
    real multiplicative ×0.25 discount (matching the land-way term); re-measured, the same route
    now follows the lane (601 km, 97% water). **Fourth time this file's `Math.min(cost,cap)`
    pattern was a no-op against an already-lower baseline** — default to a multiplicative discount
    next time unless a hard floor is genuinely the intent.
  - **Per-stage land transport: the UI's own hint text overclaimed "picked per stage from its own
    terrain."** True for water (vessel auto-substitutes on a blocked stage) but false for land —
    `_jpEffectiveStagePlan` plain-inherits `plan.transport`, confirmed by a synthetic Hills/Open-
    Plains route that stayed "Walking" on every stage. Not an oversight: `_jpPlan`'s own comment
    documents a silent per-stage auto-swap being tried and rejected (a party appearing to travel
    faster once one stage silently swapped Baggage Train for Walking). Fix respects that decision:
    `_jpBestLandTransportForStage` computes the fastest mode for one stage's own terrain (same
    equipment counts), surfaced as a named, dismissable "⚡ Mounted Rider would be ~63% faster
    here" advisory with a one-click "Use here" into the existing `stageOverrides` mechanism —
    never applied automatically. >10% margin to avoid noise. Hint text rewritten to match reality.
- **v1.52 — season slider self-enable + the last four travel cuts + V1.915 snapping.** Three owner
  requests. Hash vs v1.51 ALL IDENTICAL. 1001 / 852 / **464** green.
  - **The Cartography "Season (render)" slider "did nothing to change the map."** Root cause: not
    broken, INERT — `_seasonK` gates on Biome map view AND the "Seasons & Köppen climate" checkbox,
    which lives on a DIFFERENT tab (Generate→World) and defaults off. Measured pre-fix: 1 distinct
    render across all 5 slider positions. Fixed by making the slider self-enabling (turns its own
    prerequisite on, never off) plus a status note stating live/inert in words. Post-fix: 5/5.
  - **The four scope cuts v1.43/v1.49/v1.51 each deferred, all built from data already on hand**:
    rest days (§5's 1-in-3-to-5 cadence, reported SEPARATELY from travel days — never blurred, that
    blur is what made v1.43 hard to verify); season drift over a long journey (`jpSeasonAt`, each
    stage assigned its own **midpoint** season — start-day assignment let a 108-day trip never
    leave Spring); sea closure (*Mare Clausum* — Open Sea/Rough shut in Winter, Coastal/Bay stay
    open, sharing v1.51's `seasonalClosures` control); a cost model in **day-wages** (not an
    invented currency), with land:river:sea carriage ratios (0.055:0.011:0.002) from Diocletian's
    Price Edict — the same source v1.33 already uses for food logistics. `probe_travel.js` re-run
    clean: all 20 reference bands and the §9 242-day sample journey unaffected.
  - **V1.915 snap-to-place/way, reintroduced for the manual Way/Route tools.** They pushed the raw
    grid cell with zero attraction — confirmed regression against V1.915's own Route Editor. New
    `_civFindSnapTarget`/`_civSnapPoint`, grid-space via v1.23's `_civZoomPickR` (not V1.915's
    screen-pixel math), nearest-wins across places AND way curves with no place preference — same
    semantics V1.915's `findWaySnap` uses. Opt-out (`state.viz.snapWays`, default true), two synced
    checkboxes (Way tool in Civilization, Route tool in Explore), live hover ring on the canvas.
  - **Two bugs found only by verification, both self-inflicted in this same pass**: the desert-tier
    ordering bug (caught by syntax check) and a smoke test's `mk()` reusing `_jpEnsurePlan(jn)`'s
    SAME returned object across "variants" — v1.51's own HANDOFF entry names this exact trap.
  - **A third bug found only by taking the verification SCREENSHOT**: `const VERSION='1.50'`
    (block 1) had drifted stale again — two versions after v1.30's own comment on that exact line
    warned about this. Display/export metadata only (never gated on, so no test ever caught it),
    but the header chip reads from it, so the v1.51/v1.52 screenshots were showing "v1.50" until
    fixed. New smoke check derives the expected version from the target filename, not a literal.
- **v1.51 — constraints that were stated but never measured.** Owner asked whether max travel
  distances, travel-time calculations, dependencies and constraints are all properly factored in
  and adjustable. Audited by MEASUREMENT (a sensitivity sweep over every control + a constraint
  census), not by reading. Hash vs v1.50 ALL IDENTICAL. 1001 / 852 / **446** green.
  - **The time model was already sound** — Σ stage-days == `plan.days` exactly, 15 of 17 controls
    provably move the answer. Every defect was in the CONSTRAINT layer, where three inputs were
    constants standing in for data the world already carries.
  - **Headline**: the resupply requirement was never compared with the map. A 908 km route demanded
    a stop every 27 km while its 4 settlements averaged **303 km** apart — reported as a clean plan.
    And the verdict line *"cannot be resupplied from settlements in reach"* was driven by
    `totalMass > capacity` — it named settlements while measuring an overloaded pack. New
    `_jpResupplyReach` now reports "longest gap 545 km vs 29 km carried, 18.9× short". **Seventh
    occurrence** of a threshold never compared against the thing it describes.
  - **`supplyDays` was inert** (2/7/20/45 → identical days): the convergence loop divided its own
    load back out and used a hardcoded 7. Now the food interval IS `supplyDays`.
  - **Water was hardcoded at 1.5 days** while the world has rivers and lakes. `_jpStageDryKm`
    measures the longest waterless run; the gap is derived at the stage's own speed inside the
    convergence loop, so load lengthens it. `desertWater` defaults to `auto`.
  - **Party size had no road-capacity term** and bigger was monotonically faster — 100,000 people on
    a dirt track was the model's *fastest* configuration. `jpColumnFactor` damps the finished daily
    distance by the column's own passage time (never a speed multiplier; Haste doesn't exempt it):
    100k → 6.33 km/day. Caravan scale unaffected.
  - **Winter mountain passes now close** (`jpSeasonalClosure`, gated on terrain AND a cold biome,
    overridable via `plan.seasonalClosures`).
  - **Owner follow-ups, same version**: (a) a blocked/overloaded stage is now highlighted **where it
    can be edited** — `_stageTrouble` sorts problem stages to the top of the per-stage overrides,
    force-opens and tints them, and names the control that fixes it (not just the symptom);
    (b) **vessel information** — new pure `jpVesselDayKm`/`jpVesselMatrix` plus two panels ("Vessels
    on this route" ranked with blocking reasons, and the full hull × water-type reference). Cruise
    speed is the wrong number: Longship is fastest on every river (132 km/day), Caravel at sea (157),
    and the fastest hull is never simply the highest cruise speed.
  - Reusable lessons: **when a control has a warning attached to it, check that it moves the number
    the warning is about**; **a term that only ever helps will run away** (porter capacity);
    **"over capacity" is a symptom, not a cause** — name which constraint bound; **report a problem
    where it can be fixed**, and have the fix line name a control rather than restate the reason.
  - Two self-inflicted bugs caught only by verification (auto desert tier resolved after the speed it
    modifies; `waterGapDays` read from the wrong loop iteration) and one test trap:
    **`_jpEnsurePlan(jn)` returns the SAME object every call** — clone before building variants.
  - **Still open**: no rest-day/calendar tier split; `plan.season` uniform for a whole journey; no
    sea-closure (*Mare Clausum*/monsoon) analogue; no cost/toll model. New cuts: per-terrain file
    counts are a fixed table, not a road-width field; `JP_COLUMN_FLOOR = 0.35` is reasoned, not
    historically calibrated.
- **v1.50 — auto-selection audit + the bottleneck veto.** Owner asked whether auto-selection and
  promotion fit each biome/terrain/weight. Hash vs v1.49 ALL IDENTICAL. 1001 / 852 / **427** green.
  - **Audit cleared**: no dead table keys anywhere, full animal coverage, all 5 promotion paths
    correct. **Found**: `Hills` and `Mountain Pass` carried per-animal ratings but NO selection
    rule, so biome beat terrain where terrain matters most — a pass in an arid biome picked a camel
    (0.50) over a mule (0.85). Those two terrains are **29.8% of land route-km** on a real world.
    Fixed. `Forest Path` still disagrees with its own table but **on purpose** (capacity, not speed)
    and is now commented so it isn't re-found as a bug.
  - **Bottleneck veto**: a pack train is a whole-journey commitment, so one demanding stage
    (≥20% off the best animal, ≥10% of the route) switches the WHOLE route's animal to the
    total-time minimiser — flagged by name, and overridable via the new per-stage Pack animal
    control. Deliberately a veto rather than a global re-optimisation, which would have silently
    flipped the Forest Path capacity choice.
  - Reusable lesson: **a rule table disagreeing with its data table is a bug unless the
    disagreement is written down.** Two of three such cases here were oversights; the third was
    intent, and only a comment could tell them apart.
- **v1.49 — Route Editor: the answer comes first, and says how sure it is.** Owner-requested audit
  of the travel planner's layout + information density. Hash vs v1.48 ALL IDENTICAL. 1001 / 852 /
  **418** green.
  - **Layout, measured**: `#reResults` sat 295px BELOW the fold (top y=1295 in a 1000px viewport)
    while ~875px of dead space sat opposite the party form. Results + Stops moved into that gap as
    a **sticky** output column (`.re-col-out`) — Results top **1295 → 314**. `align-self:flex-start`
    is mandatory for the stick; sticky is off on the ≤900px stacked layout.
  - **Three pure readers, no new modelling**: `_jpVerdict` (levelled verdict that always names its
    reasons), `_jpConfidence` (asymmetric band that widens with duration — the model is a best case
    and its optimism grows with trip length), `_jpPackRange` (the wagon-equation carry ceiling shown
    *before* you cross it, computed from the same inputs as v1.48's guard so it's one source of
    truth).
  - Lesson worth keeping: **canvas tick density must be judged against DISPLAYED width**, not the
    internal backing resolution — the first cut used `cv.width` (640) for an element rendering at
    ~430px.
  - **Still open from that audit** (bigger than a UI pass): no cost/price/toll model, so a trade
    route never reports profit; `plan.season` is uniform for the whole journey; spoilage barely
    modelled; no return leg; no side-by-side plan comparison.
- **v1.48 — Pack-animal count: fodder-feedback divergence, reported honestly.** Owner: "250kg
  of cargo now necessitates roughly 213 mules." Hash vs v1.47 ALL IDENTICAL. 1001 / 852 /
  **409** green.
  - **Root cause: a fixed-point iteration with no fixed point.** The pack-animal count solver
    iterates against a fodder cost that scales with the count (each animal carries its own
    fodder). Past the breakeven day count for the picked species, adding animals costs more
    capacity than it gains — the iteration diverges, and the 6-step cap was silently returning
    whatever the divergent series had reached, a large but meaningless number.
  - Cross-checked against an owner-supplied independent reference tool with the line-for-line
    identical formula — confirmed this was never a version regression to bisect, and that the
    reference's own separate "recursive supply collapse" advisory shows the intent was always
    honest infeasibility reporting.
  - Fix: detect the infeasibility analytically (`animalFood*supplyDays*fodderFrac >= A.cap`)
    before iterating; fall back to a bounded honest floor with `infeasible`/`warn` flags and a
    hint pointing at the real fix (fewer supply days, more grazing, or a v1.44 Stops resupply) —
    not more animals. Verified: `supplyDays=200`/default grazing now reports 18, not 191,454.
  - Scope cut: `supplyDays` still isn't auto-shortened at a passed Stop — see CLAUDE.md's
    "Pack-animal count: fodder-feedback divergence, reported honestly (v1.48)".
- **v1.47 — Journey re-routing: mode-aware pathfinding.** HANDOFF's "Sea routes are never
  chosen; travel mode cannot re-bias a route" — the last of the three items open after v1.44.
  Hash vs v1.46 ALL IDENTICAL (civ-layer only). 1001 / 852 / **404** green.
  - **Re-reading the code first found the multi-modal cost graph already existed.**
    `_civDijkstraPath`'s `mode='water'`/`'mixed'` branches (v0.94) already let the Route tool
    cross open water when cheaper, rivers included. The real gap: `_jpDeriveStages` only
    SAMPLES an already-drawn path, so switching Transport never re-paths it.
  - New `_jpModeForRoute` (Sea Faring→`'water'`, River Transport→`'mixed'`, land modes→land) +
    `_jpRerouteForMode`, both thin — zero new pathfinding code, a bridge to the existing one.
  - `_civDijkstraPath` gained a purely-additive `reachable` field, since it always returned SOME
    path (even a straight line across impassable terrain) when the target was never reached —
    needed so a failed sea re-route between inland points reports honestly instead of drawing a
    ship through a continent.
  - Explicit "🧭 Re-route for `<mode>`…" button, never silent (v1.24 BUG-4 `confirm()`
    precedent) — a hand-drawn route is the user's own work.
  - Scope cuts: River Transport prefers rather than requires water; re-routing is point-to-point,
    not stop-preserving.
- **v1.46 — Coastal settlement preference.** HANDOFF's "Settlements with a port still sit
  inland" — v1.37 fixed coastal DETECTION, v1.40 tried raising the coast weight and reverted it
  (clusters seeds along the coastline, halves settlement count). Hash vs v1.45 ALL IDENTICAL
  (civ-layer placement only). 1001 / 852 / **397** green.
  - **A first cut ("swap in one coastal settlement per landmass if it has none") measured a
    no-op** — both landmasses in the reference world already had ≥1 port. Shipped version instead
    targets each landmass's coastal SHARE OF SETTLEMENTS to exceed its coastal SHARE OF LAND by
    `PORT_PREFERENCE_MULT` (3×) — scales with the landmass's own geometry rather than a flat count.
  - Candidates come from re-running the SAME `findSettlementSeeds` primitive (same threshold, same
    suppression radius) over the SAME `suit` field, masked to ocean-shore range — genuine local
    maxima, bounded and landmass-scoped so it can't repeat v1.40's clustering failure.
  - New `_civOceanDistField()` — the OCEAN-only twin of `_civCoastDistField()`, which wrongly
    sources from lakes too.
  - **Disclosed side effect**: moving a settlement before routing (the v1.39 ordering rule) can
    cascade into the crossroads pass — on the one tested seed where a swap fired with a large
    displacement, final settlement count moved 19→27. Real, not a bug (the water-edge snap has the
    same property, usually smaller). `PORT_PREFERENCE_MULT=3` is un-tuned, disclosed as a scope cut.
- **v1.45 — River deep-zoom fade: the second factor.** Closes the item v1.41 left open ("a real
  but PARTIAL recovery... a second factor is still unidentified"). Hash vs v1.44 ALL IDENTICAL.
  1001 / 852 / **393** green.
  - **Root cause: a glyph-sizing cap reused on a stroke-width law that didn't need one.** The
    river-ways call site in `drawLODView` capped `zk` at 8 (`Math.min(8,GW/span)`), copied from
    `drawLODDebugOverlays`' SEPARATE glyph-sizing `zk`, before handing it to `drawRiverWays`'s own
    self-damping `baseW*sqrt(zk)` width law. But `zk` is also the exact `GW/span` factor driving
    the (uncapped) `px`/`py` geometry reprojection — so past zoom 8 the stroke froze while the line's
    on-screen coordinates kept stretching apart, reading as a thinning/fading river.
  - **Fix**: one line, `const zk=GW/span;` (no cap), isolated to that one call site.
    `drawLODDebugOverlays`' own glyph `zk` is untouched.
  - Found by ablation (a controlled off-vs-on pixel-diff probe ruled out `riverSinuosity`,
    `rdpSimplify`, `catmullRomSample` before landing on the real cause). Measured: zoom 8 unaffected
    (already under the old cap); zoom 32 painted pixels 3,928 → 7,020 (+79%).
- **v1.44 — Route Editor: journey editing gets a full screen.** Owner: "when clicking a route I
  wish it to open a full screen menu so we can properly make edits" (route visual upper-left, edit
  stops/traveler options/carriage/season/weather, current suggestion system kept). Hash vs v1.43
  ALL IDENTICAL. 1001 / 852 / **390** green.
  - **Relocated the existing party/results forms into `#routeEditorModal`, not a second
    implementation** — same v1.18 City Viewer shell contract (`.open` class, own Escape, added to
    the scroll-fix and joystick-hide guard lists). Sidebar collapsed to a one-line summary + "Edit
    route…" button.
  - New: `_reDrawRouteMap` (static top-down thumbnail from real painted biome data, no camera); a
    Stops list with per-settlement rest/layover days (`jn.layovers`, additive on `totalDays`, kept
    out of the load/resupply convergence loop on purpose); a Weather override
    (`plan.weatherOverride`, default `"auto"` = the pre-existing seasonal-average suggestion,
    unchanged for every journey that doesn't touch it).
  - **Two bugs only manual browser screenshots caught** (not assertable headlessly): the modal's
    header summary and the Weather hint text both went stale after a non-structural edit. Fixed by
    running the summary render on every refresh and marking Weather `data-structural="1"`.
  - Scope cuts: no click-to-open from the route's line on the map (sidebar card only); route-map
    thumbnail doesn't handle antimeridian wrap; "stops" is layover days, not waypoint/path editing.
- **v1.43 — Journey Planner recalibrated against `docs/research/travel-speeds.md`.** Owner: "the
  current planner seems to roughly take 37% longer than historically recorded." Hash vs v1.42 ALL
  IDENTICAL (block 2 only). 1001 / 852 / **381** green.
  - **Measured before fixing, and the measurement moved the diagnosis.** The report's §7 critiques
    the speed tables; the largest single error was the auto-derived INFRASTRUCTURE tier, which §7
    never looks at. On a generated world at seed 12345/512px, 61% of an 830 km route's km auto-tiered
    as "Hostile / Dead Zone" (×0.50) and NO km ever reached "Stable Settlements" — mean 4.0 km/day.
    `JP_INFRA_TIERS` demanded 8 settlements per 100 km as an absolute count; the generator places ~30
    towns for a whole world. Now multiples of the world's own measured density. **After: 10.5 km/day.**
  - Four more structural faults, each with its own note in CLAUDE.md's v1.43 section: one 3.0 km/h
    "Baggage Train" bucket covering ox wagons, pack trains and porters alike; the per-animal terrain
    and weather tables unreachable for anything but a lone rider; sea distance driven by the LAND
    hours/day slider; and §5's small-caravan rule inverted into a penalty.
  - **Result: all 20 reference cases inside their §8 bands (was 9/20); the report's own §9 sample
    journey reproduces at 242 days against its 244 (was 366, +50%).**
  - Reproduce with `/tmp/.../probe_travel.js` — or re-derive it: the probe calls `jpCalcLand` /
    `jpCalcWater` directly over synthetic stages, no world needed, since both are pure over their
    (stage, plan) arguments. The infra half DOES need a generated world.
  - Scope cuts (unchanged from the report's §10 architecture): no rest-day tier split, no seasonal
    gate (monsoon lock / closed passes), no toll or tropical-attrition term, no relay-courier mode.
- **v1.39 — water-edge snap ENABLED; placement now runs before routing.** Hash vs v1.38 ALL
  IDENTICAL. 1001 / 852 / 365 green.
  - **The rule: nothing may move a settlement after `_civHierarchicalNetwork` has routed.** v1.36's
    end-of-pass snap broke v1.02 and v0.97; the snap now precedes the first routing pass and the
    crossroads settlements are snapped before their own re-route.
  - Measured: channel-bottom occupancy 6 → 1, none in water, 65.5% on the water edge (below v1.36's
    79.3%, which was measured in the unshippable end-of-pass configuration).
  - Settlement popup no longer overflows: long road/export lists now wrap.

- **OPEN, reported by owner, NOT yet addressed** (top of the queue):
  1. ~~Rivers disappear when zooming under LOD tiling~~ **FIXED v1.45.** v1.40's own probe (below,
     kept for the record) measured no wholesale disappearance, and v1.41 partially fixed a real
     de-emphasis effect (356→479 px recovery at zoom 32) but left "a second factor... unidentified."
     v1.45 found it: `drawLODView`'s river-ways call site capped `zk` at 8
     (`Math.min(8,GW/span)`, copied from `drawLODDebugOverlays`' separate glyph-sizing `zk`) before
     handing it to `drawRiverWays`'s own self-damping `baseW*sqrt(zk)` stroke-width law — freezing
     the stroke width past zoom 8 while the (uncapped) geometry reprojection kept stretching the
     line apart, so it read relatively thinner the deeper you zoomed. Fix: `const zk=GW/span;`
     (no cap), isolated to that one call site. Measured: zoom 8 unaffected; zoom 32 painted pixels
     3,928 → 7,020 (+79%). See CLAUDE.md's "River deep-zoom fade: the second factor (v1.45)".
     <details><summary>v1.40's original investigation (superseded, kept for the record)</summary>

     INVESTIGATED, NOT REPRODUCED (v1.40). Measured by diffing the `#view` canvas with
     `state.viz.riverWays` off vs on at seed 12345/256px: off-LOD 6,041 changed px; LOD z=1 20,334;
     z=2 18,272; z=4 25,116; z=8 14,926. Rivers therefore DO draw at every zoom the probe can reach,
     and the decline from z=4 to z=8 is simply less world on screen, not fading (the v1.29 width law
     is `base·√z` under LOD, which gets WIDER with zoom). Ruled out along the way: the draw gate
     (`state.viz.riverWays && dbg==='off' && biome`) is zoom-independent — `biome` is just
     `state.mode==='biome'`; and the `inView` cull keeps a polyline if ANY point is within pad 4, so
     it cannot drop them wholesale. (The baked-atlas path this note flagged as the likely culprit was
     later ruled out too — v1.29's own CHANGELOG entry: "85 tiles baked, zero difference.")
     </details>
  2. ~~Settlements with a port still sit inland~~ **FIXED v1.46.** v1.37 fixed coastal DETECTION,
     not PREFERENCE; raising the global coast weight (v1.40) clustered seeds and the suppression
     radius culled them net-negative (29 → 12). v1.46 instead does a bounded, landmass-scoped swap —
     each landmass's coastal share of settlements must exceed its coastal share of land by 3×,
     filled from genuine local-suitability maxima along the shore, never touching `suit` or the
     suppression radius. See CLAUDE.md's "Coastal settlement preference (v1.46)" for the measured
     effect (seed-dependent — correctly a no-op on an already-saturated landmass) and the disclosed
     crossroads-count side effect of repositioning a settlement before routing.
  3. ~~Settlements seeded on tiny islets~~ **FIXED v1.40** (`buildLandmassQuality`; islet occupancy
     → 0, settlement count unchanged). Coastal PREFERENCE — see #2 above, **FIXED v1.46**.
  4. **FIXED v1.42 + v1.47.** Road generation compares land against sea (`_civPreferSeaRoutes`,
     v1.42, Diocletian cost ratios, drops a redundant road only when it is not the sole overland
     link). Correction to the earlier diagnosis: journeys are USER-DRAWN, so this was never a
     journey-planner bug — the screenshot's line is a generated civWay. Travel-mode re-biasing —
     **FIXED v1.47**: the multi-modal cost graph (`_civDijkstraPath`'s `mode='water'`/`'mixed'`,
     land ways + sea lanes + navigable rivers with a real per-km cost, v0.94) turned out to
     already exist; the actual gap was `_jpDeriveStages` only sampling an already-drawn path.
     `_jpRerouteForMode` + an explicit "Re-route for `<mode>`…" button now re-path the journey's
     own start/end under the selected mode on request (never silently, per the v1.24 BUG-4
     precedent). See CLAUDE.md's "Journey re-routing: mode-aware pathfinding (v1.47)".

- **v1.38 — the City Viewer reports the settlement's own trade, not its faction's.** Hash vs v1.37 ALL
  IDENTICAL. 1001 / 852 / 365 green. The popup used `_civPlaceTrade`, the viewer used
  `_civFactionAggregates` — both correct for what they measured, but a settlement view leading with
  faction figures made the two contradict each other. **A scope mismatch is as damaging as a rule
  mismatch**: v1.33 unified the rule and missed this because the rule was fine and the subject was
  wrong. Viewer now makes the identical call; faction rows remain, labelled.

- **v1.37 — coastal settlements exist again; salt is not universal.** Hash vs v1.36 ALL IDENTICAL.
  1001 / 852 / 363 green.
  - `_umSiteKindFromTerrain` still used the pre-v1.35 near-radius, so the coastal box was one cell and
    the world had **zero** coastal settlements. With that and the estuary fix: **0 → 6 with sea access**.
  - `riverthrough` (estuary = sea AND river) now grants SEA access, not river.
  - `_civSaltAccess`: sea evaporation / rock salt / salt lake. Salt was an unmet critical need for all
    29 settlements because only the evaporite resource field was consulted.
  - **The checklist could never mark anything MET** — absolute `>0.25` against a windowed mean, the
    fourth occurrence of that mistake. Gaps now differentiate (mean 4.0 of 7 categories).

- **v1.36 — water-edge placement + corridors: GROUNDWORK, not delivering yet.** ← **next session
  starts here.** Hash vs v1.35 ALL IDENTICAL. 1001 / 852 / 359 green.
  - `_civSnapToWaterEdge` measurably works (79.3% on water edge vs 65.5%, flood occupancy 6 → 1, mean
    river distance 16.3 → 3.6 km) but is **gated off** (`state.civ.waterEdgeSnap`) because it runs
    after routing and leaves ways stopping short of moved settlements (breaks v1.02 and v0.97).
    **THE NEXT TASK: reorder `_civIterativeAutoWorld` so placement completes before routing**, then
    turn the snap on by default.
  - `buildRouteCorridors` (terrain-derived passes/fords/isthmuses, deliberately not road-derived so
    placement stays acyclic) is sound and sparse but **ineffective at `corridor:0.08`** — settlements
    read 0.72× the land average. Re-tune only after the snap stops confounding the measurement.
  - **Measurement trap**: an early 2.45× corridor preference was the snap's doing, not the corridor
    term's. Change one thing at a time.

- **v1.35 — "water access: none" for harbour towns.** All 29 settlements in the reference world
  reported `none`. Hash vs v1.34 ALL IDENTICAL. 1001 / 852 / 357 green.
  - **A km threshold below one cell width is unsatisfiable.** `coastDistKm < 3` against a 3.13 km cell
    could never fire. Use `_umWaterReachKm()` (≥1.5 cells), never a bare km literal.
  - **v1.32 broke `riverOrder` the same way** (gated at ~2.1 km, below a cell) — it read 0 everywhere,
    silently disabling navigability, food-shed transport mode and harbour validity.
  - Attached sea lanes and `_umSiteKindFromTerrain` were never consulted; both are authoritative.
    **Fifth instance** of two functions answering one question and drifting.
  - Every verdict now carries a `basis` string, so "none" can be told from a threshold bug.

- **v1.34 — hinterland surplus derived from soil + the 9:1 farmer ratio.** Hash vs v1.33 ALL
  IDENTICAL. 1001 / 852 / 352 green + new `tests/perf/probe_foodshed.js`.
  - v1.33's free `FOOD_MARKETED_FRACTION = 0.30` is replaced by the attested **9 farmers : 1
    urbanite** ratio, with per-cell surplus from soil fertility (470–1000 kg/ha). **Marginal soil
    yields zero surplus** — that is the brake on runaway city size.
  - **Structural fix**: v1.33 gave a town its whole catchment ceiling, i.e. the entire population of
    its own catchment with nobody left farming. It now gets only the surplus.
  - **The chain is acyclic and now asserted**: terrain → carrying capacity → rural pop → surplus →
    urban ceiling. Three smoke checks prove no feedback. Keep them green.
  - **Two calibration traps, both measured**: pinning subsistence to the yield-range midpoint assumes
    median soil = 0.5 and collapsed urban share to 0.86%; using the yield minimum as a floor gave
    barren ground a surplus. Calibrate to the world's measured median; scale yield from zero.
  - Clean world, seed 12345/256px: **urbanisation 13.98%**, all settlements within their sheds.
  - Parameter audit found and unified a **fourth** duplicate-constant pair (two grain yields).

- **v1.33 — export-reporting audit + the food-shed population ceiling.** Hash vs v1.32 ALL IDENTICAL.
  1001 / 852 / 344 green. Research: `docs/research/food-logistics.md`.
  - **The settlement inspector and the Economy page disagreed about exports.** v1.32 fixed the faction
    threshold and left `_civPlaceTrade`'s copy on the old absolute margin — with a comment claiming they
    matched. One rule now (`_civResourceTradeBalance`). **Third time this shape has appeared.**
  - **Populations are now capped by what can actually feed them.** A deficit used to mean "imports
    food" with no check that food existed in reach. Now `_civFoodShed()` = local catchment + countryside
    hinterland + long-range import, with delivery decaying as `2^(−d/D)` (160 km land / 880 river /
    8000 sea, from Diocletian's Price Edict). Seed 12345/256px: total pop 123,185 → 81,873, largest
    52,589 → 25,669, all settlements within their shed.
  - **Two traps worth remembering.** Counting only other settlements' surplus (not the countryside)
    crushes every capital to its own catchment disc. And the ceiling pass must iterate to a FIXED POINT
    — capping a supplier removes supply a consumer was counting on.
  - `FOOD_MARKETED_FRACTION = 0.30` is the one free parameter and it governs maximum city size.

- **v1.32 — owner bug batch: overlay scroll, faction exports, Explore popup, phantom coastlines.**
  Five reported issues, all UI/civ-layer — **hash vs v1.31 ALL IDENTICAL**. 1001 / 852 / 335 green.
  - Setup-gate/generating scroll was swallowed because `#onboard` is a `.canvas-wrap` child and that
    element's wheel handler `preventDefault()`s unconditionally. New `_overCanvasOverlay` guard.
  - Faction Exports were permanently empty: a fixed `>0.15` absolute margin against world means that
    v1.31's scarcity thinning pushed down to 0.02–0.16. Now a ratio test, like the archetype fix.
  - Explore's Info tool no longer drills into the full-screen City Viewer; it opens the same anchored
    popup Civilization mode uses (city card on top, editable parameters), with the viewer behind an
    explicit button.
  - **The grid-cell-radius bug class bit twice more.** `_umSiteKindFromTerrain` scanned `GW/128` CELLS
    (tens of km on a big world) so an inland town read as `bay` and drew a coastline inside its 1.7 km
    box; the profile's river search accepted a stem within `GW/8` CELLS, producing `river ord 1
    ~618.4 km`. Both are real-km now. **When you see a radius in grid cells, check what it means in
    km at the largest supported world.**
  - A first cut put `const UM_SITE_BOX_KM = …UME.SITE_WM…` at block-2 top level; `UME` is block 4, so
    it threw and aborted block 2. Lazy functions now — the cross-block rule is not optional.
  - The bathymetry-variance assertion was flaky (~1 in 3) on v1.30–v1.32 alike; now guarded + toleranced.
  - **Disclosed gap**: the faction-export fix is verified by reading only. The smoke harness never
    generates territory, so the threshold is never exercised there and that assertion is vacuous.

- **v1.31 — pre-industrial resource grounding.** Owner supplied
  `docs/research/settlement-resources.md` and asked for the tool to be updated with it; six of its
  sections are now built in. Resource vocabulary grew **6 → 15** (append-only), deposit footprints are
  thinned by **crustal abundance** (§10.1 — rank-based, only ever thins), **iron is gated by charcoal
  rather than ore** (§10.2/§10.3 — 2 of 3 iron settlements come out fuel-limited at seed 12345, the
  documented Elba case), trade runs §9's 7-category checklist and §8's composite archetypes, §6 adds
  the pastoral↔arable manure/competition coupling, §7 gates bulk goods on navigable water, and
  §10.7 makes population density vary by **subsistence mode** instead of one flat constant.
  Everything is a new key, a new opt-in `opts.*`, or block-2 derivation — **hash vs v1.30 is ALL
  IDENTICAL including `icons`**. 1001 / 852 / 326 green.
  - **Two silent latent bugs surfaced by probing, not by tests** (NaN compares false, so nothing
    threw): `mkResMap` was a frozen six-key literal indexed with fifteen keys, NaN-poisoning
    `worldMeanResource` — which the import/export rule and archetype thresholds both divide by; and
    `channelAtlasGroups` hand-listed six resources across two RGB files, so nine fields would have
    silently vanished from the channel atlas and manifest. The `.f32` export was hand-listed too and
    had been missing **tin** since v0.105. Anything indexed by the vocabulary must be built from it.
  - **Population scale was explicitly held** (owner's call): density redistributes by land use, but
    `currentAgrarianDensity` normalises per world so the land-integrated total exactly matches v1.30
    (1,326,919 at seed 12345/256px). The reference's 25–70% realized-vs-theoretical discount is
    available as a one-line change and is NOT applied. **Do not recalibrate by pinning one band** —
    doing that collapsed the ceiling 13× in a first cut, because almost no cell is annual cultivation.
  - Measured: settlements 18 → 29, total population 122,342 → 123,185 (+0.7%).
  - **Not built**: §5 (soil from parent rock — duplicates `buildSoilFertility`), §10.5/§10.6 (labour
    budgets, storage losses — need a labour model that does not exist), §10.4's seed-to-yield floor is
    defined and exposed but only reported, not wired into food surplus.

- **v1.30 — settlement suitability unified, flood wired in, per-settlement trade.** Owner asked whether
  exports could be auto-defined and whether suitability really uses all the underlying layers. The
  audit answered both, and found a correctness bug: there were **two** suitability functions.
  `buildSettlementSuitability` (block 1) backed the debug view, the `.f32` export and the seed list;
  `_civExtendedSuitability` (block 2) — richer terms AND a different seed threshold, 0.65 vs 0.42 — was
  what auto-populate actually placed from, so **the advisory gold dots were never the sites you got**.
  Now one function: the extended terms live in block 1 behind an optional `opts.ctx` (so it stays a
  pure primitive, still callable with the original 8 args), `_civExtendedSuitability` is deleted, and
  `SETTLE_SEED_THRESH` gives the view and the placer one threshold. **Flood** was absent from placement
  entirely and is now a penalty (plus the slope×flood buildability composite the `siteprofile` view
  already drew — that view is now a preview of a live term, not a post-hoc description). **`soil` was
  a declared-but-never-read parameter** and now reaches the score via soil × a rainfall optimum
  (deliberately not a second temperature bell — carryingCap is already soil × temp × water, so
  rainfall is the one new agronomic signal). **Water was double-counted** (`wW` and `wC` both read
  `water[i]`); the old "coast/trade" proxy is replaced by a real coast SDF. **`_civPlaceTrade`** gives
  each settlement its own exports/imports from specialisation + hinterland + food surplus, measured
  against the same world mean the faction rule uses. Verified **1001/1001** (+9), **852/852**, hash vs
  v1.29 **ALL IDENTICAL** (suitability never touches the terrain render), smoke **310/310** (+10).
- **v1.30's calibration lesson — read before touching `SUIT_W_FULL`.** The first cut redistributed all
  weights so they summed to 1, which is the obvious thing to do and is wrong. Measured, the field
  collapsed: median 0.314 → **0.124**, max 0.751 → **0.431**, advisory seeds above 0.42 from 102 to
  **ONE** — because the average cell lost weight that sat on food and water and gained weight on a
  coastline it does not have. The rule the code now states explicitly: **CORE** terms exist at every
  land cell (carrying capacity, freshwater, slope, elevation band, soil×rain, buildability) and sum to
  1.0, fixing where the sigmoid's 0.5 pivot falls; **OPPORTUNITY** terms (coast, river, lake, minerals)
  are zero for most of the map and are ADDED on top, never carved out. Post-fix: median 0.205, max
  0.730, 59 seeds at 0.42. **Placement measured** (seed 12345, 256px): mean floodplain exposure of
  placed settlements **0.617 → 0.437**, share on high-flood ground **58% → 28%**, 9 of 18 sites
  unchanged. **Disclosed side effect:** count rose 12 → 18 and the tier mix moved toward hamlets
  (2 villages → 12). Population comes from carrying capacity, not from the score, so this is not the
  score shrinking towns — sites pushed off floodplains onto drier ground have less fertile catchments
  and are legitimately smaller. Floodplains being both the best farmland and the worst foundations is a
  real tension; the flood weight (0.14) is the dial, left untuned pending a call on how hard it should
  bite.
- **v1.29 — eight owner-reported bugs.** Triaged against the code first; three of them (river-ways
  streaking across the map, the LOD tile seam, river-ways inside lakes) share one shape of cause — a
  per-tile or per-polyline computation each neighbour performs on a different truncated view of the
  same shared data. **The heightmap/climate pipeline is untouched:** hash vs v1.28 reports
  `field`/`temp`/`rain`/`flow` **IDENTICAL in every scenario**; `rgba` differs and that difference is
  *proven* to be entirely the requested river-way restyle — with `state.viz.riverWays=false` on both
  sides the rendered canvas is byte-identical (FNV `1404487302` each). **B1** the touch long-press
  callout suppression was scoped to `.row input[type=range]`; widened to every range input (the
  Library's v1.26 weight sliders live outside `.row`). **B2/B7a** `traceRiverPolylines` returns a
  RECEIVER CHAIN, not a drawable path: in world mode it wraps the antimeridian (one `lineTo` back
  across the map) and it walks straight over inland lakes (which pool above sea level, so the
  `fld[i]<sea` stop never fires). New pure `splitRiverPolylines(polys,W,skip)` cuts a chain wherever
  the next point isn't reachable by a straight stroke; applied at render + GeoJSON export only, so
  `carveRiverValleys` is untouched. **B3** river-way width grew 1:1 with zoom on BOTH camera paths;
  damped to √zoom (`base·√z` under LOD, `base/√z` off it — the two conventions carry the zoom factor
  in opposite places), exactly `base` at zoom 1. **B4** the `if(_lodOn)` wheel/pinch branches only
  scaled `_lodZoom` and never touched `_lodCx/_lodCy`, so LOD always zoomed about the camera centre;
  new `_lodZoomAt` restores zoom-to-cursor (`_lodCx = gx + regW'·(0.5−fx)`, which collapses to the old
  behaviour at fx=0.5, so the zoom buttons are unchanged). **B5** — see the seam note below. **B6**
  both 3D height paths flatten water with `h < sea ? sea : h`, which can only catch the OCEAN; an
  inland lake pools ABOVE sea level so its pre-flood terrain stayed in the mesh and lit up as relief.
  New `_v3dHeightSource()` substitutes `_lakeFill` per lake cell — CPU pre-pass, no shader change,
  returns `field` itself when either toggle is off. **B7b** a class-0 cell sitting lower than the lake
  next door reads dry at map scale but is under water once zoomed in (v1.05 floods lake shorelines
  sub-cell); new `_civLakeFlooded` applies the renderer's own predicate in both civ land tests.
  Verified **992/992**, **852/852**, smoke **300/300** (+12).
- **v1.29's LOD seam — read this before touching tile rendering.** Root-caused by measurement; the
  first two theories were both wrong. The tiles' HEIGHT data at a shared column is byte-identical
  (mean abs diff exactly **0** — `amplifyRegion` samples inclusively, so neighbours share their
  boundary column), and quantising the destination rect changed nothing. The real cause was
  `renderBiomeTileRGBA` box-blurring the sea floor **per tile**: a box blur clamps at the array edge,
  so two adjacent tiles smooth the boundary from opposite truncated neighbourhoods — identical height
  in, different colour out (shared-column RGB MAD **6.71** vs 0.3–0.5 for ordinary neighbours; in the
  live composite that column was the single largest colour discontinuity on screen, 22.3 against a 4.4
  local mean). Most of a map is ocean, so that was the seam. Now sourced from the world-wide coarse
  fields the main map already caches (`sharedSeaFields()` over `_seaHCache`/`_seaShadeCache`), sampled
  at world coordinates like `tempField` — which is what v0.092's own stated goal asked for, is what
  the PNG bake already did, and drops two full-tile blurs per refine. **After: shared-column MAD
  6.71 → 0.04.** Two smaller fixes alongside: tile colorisers clamped their central-difference index
  at the border (rendering that column at half slope — now `edgeL/edgeR/edgeU/edgeD` extrapolate), and
  the destination rect is quantised to whole device pixels. **Residue, disclosed:** the boundary column
  still measures ~2× its local neighbourhood, consistent with the shared world column being DRAWN
  TWICE — a one-pixel stutter, not a hairline. Removing it needs tiles rendered with a one-pixel apron
  and cropped, which changes `pyramidTile`'s output shape and the atlas format. The other per-tile
  neighbourhood passes (`aoB`/`crestB`/`coastB`/`riverB`/`biomeBD`) share the class of defect but are
  all opt-in and already documented as per-tile decoration. Separately, the v1.19 pan joystick and the
  zoom-reset button were the only two camera moves that never scheduled a tile refine — which is the
  "correct resolution only renders when the user zoomed in/out" half of the report; both now do.
- **v1.28 — owner: "all the things that aren't live I want you to wire them so they are."** The v1.26
  audit found **35 of the 71** non-custom Asset Library slots had storage, an inspector card and an
  export slot but no consumer — art could be authored and would never appear. All 35 now render, and
  every path is inert without a pack, so hash vs v1.27 is **ALL IDENTICAL**. **Biome (15) + Terrain
  (13) textures** bound to the two PAINTED Cartography layers: new `PACK_BIOME_SLOTS`/
  `PACK_TERRAIN_SLOTS` whose index order is 1:1 with the frozen `CART_BIOMES`/`CART_TERRAINS`
  (slot N = paint value N+1), manifest `biomes`/`terrains` sections (JSON + CSV), and
  `surfaceColor`'s existing paint-tint step samples the texture instead of the flat `CART_*_COLS`
  swatch — same 0.60 weight, same pipeline position, so relief still shows through. These two are
  sampled as **TRUE COLOUR**: they deliberately skip `finalizePackTexture`, whose per-channel
  `inv=1/mean` is what makes the splat family divide a texture's absolute hue out. **Traits (7)** were
  dead in three places at once — the manifest importer only handled `settlement`/`poi`, there was no
  `_traitSprite`, and nothing drew traits at all despite the civ layer's own comment claiming they
  were "drawn beside the marker"; all three added, badges drawn in a capped centred row under the pin
  with a glyph fallback. `administrative` **appended** to `CIV_TRAITS` — a genuine gap, since
  `_civNetworkMetrics` already assigned it and the Library already reserved its slot, but it had no
  vocabulary entry so it could never be toggled or given art. **Cache bug found while wiring:**
  `_lodRenderKey()` and the civ bake key had no asset-pack term, so an imported pack only appeared
  once something unrelated changed the key — the same bug class those keys already carry v0.86/v0.88
  comments about. New `_assetGen` in both keys, bumped on import/clear/Library-sync; **this also fixes
  the pre-existing splat-texture case**. Bridge extended to push ground textures + structures.
  **Block 3 needed no changes** — its importer and exporter already handled all three families, so the
  engine really was the only missing half. **Visible default change:** settlements with traits now
  draw badges where they previously drew nothing (no toggle added — ask if you want one). Verified:
  **992/992**, **852/852**, hash **ALL IDENTICAL**, smoke **288/288** (+7); browser-proved a magenta
  biome texture rendering `[166,51,155]` and cyan terrain `[51,166,155]` against flat-swatch baselines.
  **Probe gotcha worth remembering:** writing a paint array directly without bumping `_paintGen` leaves
  the render cached and shows no tint at all — that artefact is what surfaced the `_assetGen` gap.
- **v1.27 — owner: "clean up the code, annotate each block with what it does, remove old comments
  and check for bugs."** Senior-review pass over v1.26's scatter system. Six defects found by
  reading + one found by the verification probe; all inside code that only runs once rules are
  configured, so hash vs v1.26 is **ALL IDENTICAL including `icons`**. **FIX-1:** wetland/biome were
  ORed in scatter mode but ANDed in relief mode — ticking "Wetland only" silently discarded the
  user's biome picks; both now AND. **FIX-2:** `normalizeScatterRule` didn't reject non-finite
  input, and rules come from a user-supplied project zip — a `NaN` density scattered on EVERY land
  cell (`keep >= Math.min(1,NaN)` is always false) and a `NaN` spacing collapsed the relief bucket
  grid into an O(n²) scan; all numeric fields now clamp through a `num()` helper. **FIX-2b (probe,
  not reading):** `Object.assign(base,r)` mutates and returns `base`, so `out===base` and every
  `base.<field>` fallback read the garbage it was meant to replace — **this bug was in v1.26 too**,
  and was only caught because the probe asserted `Number.isFinite` instead of trusting the code.
  **FIX-3:** `spaceOf` falls back on the computed value so an un-normalised rule can't collapse the
  grid. **FIX-4 (data correctness):** deleting every variant of a Library asset left its bitmaps
  live on the map forever — `syncToRuntime` skipped empty slots and never cleared what it had
  written; it now tracks owned slots and passes `dropIcons`/`dropCustom`, never touching
  imported-pack art it didn't write. **FIX-5:** scatter priority depended on rule *insertion* order
  (the table is built by iterating an object); now most-specific-first, verified order-independent.
  **FIX-6:** the brush asked for ~15k darts per stamp at max radius×density, on every pointermove —
  capped at 1500 (437 icons in 3 ms at the maxima). **Comments:** removed one stale header (the
  `TREE_SLOT` note still called sprite packs "a later, optional upgrade" — they've been the primary
  draw path for many versions); historical rationale comments deliberately KEPT, since notes like
  v0.61's "deliberately NOT an async function" are what stop a shipped regression being
  reintroduced. Verified: engine **992/992**, UME **852/852**, hash **ALL IDENTICAL**, smoke
  **281/281** (+7). **Flake recorded, not reproduced:** one engine run reported 991/1 and never
  recurred across 16 further v1.27 runs / 3 v1.26 runs; the v1.27 edits are unreachable from the
  headless suite (it never passes `opts.rules`). Note the harness prints failures as `FAIL - <name>`
  on **stderr** — worth knowing when grepping.
- **v1.26 — owner: "replace my current SVG-based cartography icons with a rich, Nortantis-style
  raster asset scattering system"** (5 numbered requirements: pack autopopulation, per-asset
  inspector controls, a procedural scatterer, a manual brush, Y-sorted raster rendering).
  **Investigation changed the scope: three of the five already existed.** `drawMapIcons` has always
  drawn raster pack sprites (`ctx.drawImage(v.bmp,…)`, bottom-anchored via `spriteDrawRect`) with
  the vector glyphs as a *fallback*, already Y-sorted, and already picked random variants;
  `placeMapIcons` already did spacing-rejection placement (grid-bucket `fits()` — Poisson-disk in
  all but name); `_carPopulateIconGallery` already drew pack bitmaps, not SVG; pack import already
  auto-assigned art to the 10 frozen `PACK_ICON_SLOTS` via the manifest. The real gap was **per-asset
  control** — the biome→asset mapping was HARD-CODED in `placeMapIcons`, so behaviour was fixed by
  which frozen slot an asset occupied and nothing outside that list could scatter. Three design
  forks were put to the owner via `AskUserQuestion` (answers: Library-as-source-of-truth bridge;
  climate `BIOME_KEYS`; open vocabulary via custom sets). Shipped: **D1** a `ScatterRule` data layer
  whose presets reproduce v1.25's hard-coded behaviour exactly; **D3** `placeMapIconsRuled()`,
  reached only via a new OPTIONAL `opts.rules` so its absence is a guaranteed bit-identical
  fall-through (relief mode = elevation-ranked + shared blue-noise spacing grid; scatter mode =
  jittered grid + density-as-keep-probability); **D5** a unified Y-sorted `items[]` draw list, fixing
  a real occlusion bug where category (not latitude) decided overlap so a mountain always painted
  over a tree standing in front of it; **D2** the **Library→runtime bridge** — the missing link the
  request didn't account for, since block 3's AssetDB only reached the map via a project-zip export/
  re-import round-trip — plus a "Procedural scattering" section in `#alInsp` (enable/mode/13-biome
  grid/min-max size/density/elevation band/wetland-only/per-variant weights) that syncs live; **D4** a
  Cartography density brush that dart-throws with blue-noise rejection using the asset's OWN rule for
  size and variant weighting. Verified: engine **992/992** and UME **852/852** unchanged, hash vs
  v1.25 — `default`/`geoid`/`waves`/`ao` **ALL IDENTICAL** with the `icons` scenario intentionally
  diverging (`field`/`temp`/`rain`/`flow` all identical, only `rgba` differs ⇒ purely the Y-sort draw
  order, not terrain — same pattern as v1.20), smoke **274/274** (+8). Behaviour was proved in a real
  browser first: 244/244 icons inside a biome-restricted rule and none in water, density 581→3872,
  relief spacing exactly the requested 8.0, unweighted variant picks byte-matching `pickIconVariant`
  across 200 samples, one brush stamp painting 10 correctly sized/spaced/on-land icons.
  **Known scope cuts:** rules bind to climate biomes only (no `CART_BIOMES`/`CART_TERRAINS` painted-
  layer targeting — the owner's own choice); no slope/aspect/coast-distance rule terms; the brush has
  no eraser or per-stroke undo; settlement/trait/POI families deliberately get no scatter rules.
- **v1.25 — owner: "when selecting the preset worldshapes like volcanic, archiapello or islands
  the result isn't what is suggested."** Root-caused with a Playwright probe (measured land
  fraction + connected-component landmass fragmentation across all six archetypes at a fixed
  seed/resolution, BEFORE writing any fix). Root cause: `deriveFromWorldStructure()` derives
  plates/tectonicEnergy/volcanism from an archetype's bundle but never touches `state.seaLevel`
  — an independent user slider (default 0.42) that stays put regardless of archetype;
  `normalize()` is a pure min-max stretch, also continentality-independent. Combined with the
  height formula's disproportionate orogeny contribution for exactly the low-continentality
  archetypes (Archipelago tectonicEnergy 0.80, Volcanic 0.90), the fixed sea level landed in the
  wrong place for those worlds' own height distributions: measured pre-fix at seed 12345/512px,
  **Archipelago (continentality 0.15) rendered 71.5% land; Volcanic (continentality 0.05)
  rendered 60.6% land** — both MORE land than plain Classic's 56.5%, backwards from what those
  names promise. Fix: `applyWorldStructureSeaLevel()`, gated on `state.world_structure.enabled`,
  called inside `generate()` right after `normalize()` + the volcanism/craters clamp. Reuses
  `generateContinentalityField()`'s own O(N) histogram-percentile technique — measures the
  ACTUAL generated field's height histogram and re-anchors `state.seaLevel` to the threshold
  that yields exactly the archetype's promised `(1−continentality)` ocean fraction (clamped
  `[0.05,0.95]`), refreshing the `#sea`/`#seaV` sidebar slider via the existing `v()`/`lab()`
  globals (needed since `_suGenCommit()` calls `syncUI()` *before* `generate()`). Chosen over a
  predictive pre-generation formula because it's self-correcting regardless of how
  tectonicEnergy/oceanDepth reshape the distribution — same reasoning that already justifies
  `generateContinentalityField()`'s own percentile approach. Does NOT touch the height formula
  itself (invariant 8: no re-added γC term) — only the independent land/ocean threshold. No
  effect when `world_structure.enabled` is false (Classic/default), so bit-identity holds.
  **Re-measured post-fix** (same seed/resolution): land fraction now tracks each archetype's
  continentality almost exactly — Earth-like 0.300, Supercontinent 0.601, Archipelago 0.150,
  Volcanic 0.050, Rift 0.401. Landmass-dominance also improved incidentally (Volcanic's largest-
  landmass share 84.0%→44.2%, Archipelago's 96.8%→81.6%) though this wasn't the target of the
  fix. **Known scope cut, disclosed:** landmass SHAPE/fragmentation still doesn't fully track
  `fragmentation`'s intended noise frequency — `buildPlates()` samples the smooth
  `continentalField` at only each plate's single centroid, so achievable island count is capped
  by plate count (≤40), not fragmentation's frequency; Archipelago still concentrates most of
  its (now correctly small) land total in one dominant landmass rather than many similarly-sized
  islands. A real fix needs per-cell continentalField blending into the height formula's
  plate-base signal, which risks brushing invariant 8 — left as a candidate follow-up. Verified:
  engine **992/992** unchanged, UME **852/852** unchanged (fix lives entirely in block 1's
  `generate()`, gated off by default), hash vs v1.24 **ALL IDENTICAL** (default/WS-disabled
  scenario — the gate is airtight), smoke **266/266** (+7: per-archetype land-fraction sane
  bounds, cross-archetype land-fraction ordering, land fraction tracks continentality within a
  wide band for every archetype, sidebar sea-level slider DOM reflects the auto-derived value).
- **v1.24 — an external QA report** (headless jsdom + real-event harness against v1.21) flagged 8
  bugs; all 8 verified real against the current file (line numbers shifted, behavior hadn't) before
  fixing, all civ/UI-layer or pure accounting/escaping changes so engine/UME/hash are unchanged.
  **BUG-1 (HIGH):** World Structure sliders were completely dead — `segOn` is `const`-scoped inside
  `syncUI()`, the slider `change` handler (a separate closure) called it anyway and threw
  `ReferenceError` before regeneration ever ran; fixed by inlining the toggle. **BUG-2 (HIGH, data
  loss):** the global Delete/Escape keydown handler had no typing guard (unlike every sibling
  listener) — one stray Delete while editing a place's Name/Pop/History field silently deleted the
  settlement; added the same `INPUT`/`TEXTAREA`/`SELECT`/`isContentEditable` guard. **BUG-4 (MEDIUM,
  data loss):** Clear-labels/Clear-icons/Delete-place all mutated with no confirm, unlike the matching
  civ-tab clears; added matching `confirm()`. **BUG-5 (MEDIUM):** no `beforeunload` guard anywhere —
  new `_hasLiveWorld()` + listener (deliberately a blanket "world exists" check, not a fine-grained
  dirty flag — too many mutation sites to thread reliably without a silent gap). **BUG-3 (MEDIUM):**
  `showBusy`/`hideBusy` had no nesting counter, so two queued `withBusy` ops hid the overlay early;
  added a `_busyDepth` counter (pure internal accounting, every call site already balanced 1:1).
  **BUG-6 (LOW):** the asset-pack thumbnail gallery's `#packGrid` had real CSS but no HTML host
  element — restored it. **BUG-7 (LOW):** user names in several `innerHTML` CONTENT contexts weren't
  escaped (attributes/History textarea already were) — new shared `_escHtml()`, applied at the
  confirmed sites (place list row, faction dropdowns ×2, City Viewer header, settlements table row).
  **BUG-8 (LOW):** `spaceDown` only cleared on keyup — an Alt-Tab while holding Space left it stuck;
  added a `window` blur listener. Verified: 992/852 unchanged, hash ALL IDENTICAL, smoke **259/259**
  (+8). One test-only bug found during verification: the BUG-3 smoke assertion needed to force a
  clean `_busyDepth=0` baseline (an earlier queued op elsewhere in the long smoke run can still be in
  flight). Several report items were confirmed as non-issues and left alone (duplicate id inside an
  HTML comment, the function-reassignment-wrapper pattern, an intentional NaN check).
- **v1.23 — owner: "when zooming in, the area that is clickable to view a settlement stays fixed and
  thusly becomes relatively bigger, making panning near impossible"** + a Journey Planner review
  ("Coastal Waters faster than Open Sea — historically backwards"; "autoselect assigns a vessel to a
  leg it isn't fit for, only caught downstream"). Three fixes, all civ/Journey-Planner (block 2), so
  engine/UME suites and hash bit-identity are all unchanged. (1) **Settlement pick radius** — the
  place-pick radii were flat GRID-space (`GW/50`, `GW/35`) while the pins draw at a constant
  on-screen size, so zoomed in the clickable target ballooned and swallowed pan drags; new
  `_civZoomPickR(gridR0)` divides the zoom-1 radius by the live zoom (`viewT.scale` off-LOD, `_lodZoom`
  under LOD), applied at all five place-pick sites, unchanged at zoom 1. (2) **Sea speed ordering** —
  `JP_TERRAIN.sea` had Open Sea `0.85` < Coastal Waters `1.00` (wind is a separate axis in
  `JP_ROUTE.sea`; `1.00/0.85≈1.18` matched the reported 97/82), reordered to Sheltered Bay `0.95` <
  Coastal `1.00` < Open Sea `1.20` (Rough Open Sea `0.60` the weather outlier); measured Open Sea
  113→159.6 km/day (Cog/14h). (3) **Vessel↔terrain compat** — the selector (`_jpVesselFits`) and
  validator (`jpCalcWater`) had duplicate inline copies of the compat rules; both now call one shared
  `_jpVesselWaterBlock`, so an autoselected vessel can never be one the validator later rejects (the
  validator still fires for a genuinely infeasible manual pick). The Dhow stays `openSea:true`
  (historically correct — monsoon ocean trader); the task's "dhow restricted from open sea" premise
  doesn't apply here. Verified: 992/852 unchanged, hash ALL IDENTICAL, smoke **251/251** (+8), and an
  autoselection sweep across all nine water terrains showed zero invalid picks.
- **v1.22 — owner: "The joystick works in the opposite direction that we push, and on mobile/tablet
  i think we should just have it in all views" + "When using LOD pyramid tiling LOD0 seems to be of
  poor resolution (even if we pick 1k/2k) and the individual sub division of tiles below LOD0 should
  be tiles of 512px."** Three fixes in one pass. (1) **Joystick direction** — the v1.19 port of
  `Cartalith_V1.915.html`'s ANDROID NAV PAD dropped the source's velocity **negation** (`vx =
  -(dx/mag)*…`, comment "drag the knob right → reveal map to the right"); restoring that single sign
  in `_sculptNavSetKnob` makes push-right travel right, fixing both off-LOD (`viewT.panX += _svx`)
  and LOD (`_lodCx -= _svx`) branches at once. (2) **Joystick in all views** — `_sculptNavSync` no
  longer gates on `_sculptEditorActive()`; on any touch device it shows whenever the main map is the
  active surface, hidden only behind the setup gate (`#onboard`), in 3D (`_view3dOn`, its own
  camera), and in the City Viewer modal — re-synced from each of those transitions. (3) **LOD0
  resolution** — the LOD viewer composited into the GW×GH `#view` canvas, so fully zoomed out the
  map mapped 1:1 into GW pixels (the coarse-field resolution) and the pyramid's finer sub-tiles were
  downsampled straight back. New `_lodRenderW()` supersamples the LOD backing (2× field width, capped
  2560px so tablets stay smooth / big worlds never downsample), `lodViewRect` picks the pyramid level
  against that render width (LOD0 now composes from 2×2 finer tiles), and `drawLODView` maps its
  unchanged GW×GH draw math onto the larger backing via a context transform; `renderNow` restores
  `#view` to GW×GH on the non-LOD path so the default render is byte-identical, and `_v3dGrabColor`
  downscales the whole (possibly supersampled) canvas for the 3D drape instead of cropping. Verified:
  engine **992/992**, UME **852/852** (both unchanged), hash vs v1.21 **ALL IDENTICAL** (default +
  geoid/waves/ao/icons — all changes are opt-in LOD / touch-only), smoke **243/243** (the two v1.19
  joystick-direction assertions were flipped to assert the corrected direction). Browser-probed: LOD0
  at 1024px now picks z=1 (4 sub-tiles, each 1024px = 2× detail) into a 2048×1310 backing rendering
  crisp fine rivers/coastlines; joystick shows in Generate → World on a mobile UA, hides behind the
  gate/in 3D, push-right ⇒ `_svx<0` and `viewT.panX` decreases. Canvas/GPU/touch — flagged for manual
  on-device confirmation.
- **v1.21 — owner: "I'd like zoom and pan buttons and an option for the viewer to zoom. That way
  it should be easier to work accurately with larger resolution sheets."** Follow-up to a question
  about the Asset Library's sprite-sheet slicer (`SpriteSheetImporter`, script block 3): the
  slicer always scaled the *entire* sheet down to fit a fixed box, so a large/detailed sheet just
  shrank with no way to work at higher precision. Key finding: the slicer's canvas already sits
  inside `.al-slice-cv-wrap`, which is CSS `overflow:auto` — never triggered before because
  `redraw()` deliberately capped the canvas to always fit. Lifting that cap on zoom makes the
  canvas larger than its container and the **browser's own scrollbars/wheel-scroll/touch-scroll
  pan it for free** — no custom camera system needed (unlike the main map's `viewT`/`zoomAt` or
  the v1.18 City Viewer's `_cvCam`), and `evToSrc()` (every existing tool's pointer→source-
  coordinate conversion) needed zero changes since `getBoundingClientRect()` already reflects
  scroll position. New `−`/`+`/`Fit` zoom buttons + a live `%` readout + wheel-zoom-to-cursor
  (same "keep the point under the cursor fixed" trick as the main map's `zoomAt()`, via scroll
  offsets instead of a transform); new 4th "✋ Pan" mode alongside select/grid/pick, dragging
  adjusts the wrap's scroll by the drag delta (mirrors the main map's own `panDrag` idiom).
  Verified: engine **992/992** and UME **852/852** (both unchanged — block 3 only), hash vs v1.20
  **ALL IDENTICAL including `icons`** (pure Asset Library UI, zero map-render effect), smoke
  **243/243** (+7, driven purely through the DOM/real Playwright mouse input since
  `SpriteSheetImporter` is intentionally not exposed on `window` — including a check that
  cell-click-to-select still hits the right cell at a non-fit zoom, the real regression risk).
- **v1.20 — owner: "let's go up to 4/5 different possible tree types (and for other landscape
  types and features) that can be placed at relatively random."** Follow-up to a walkthrough of
  the Asset Library's coverage: the opt-in procedural map-icon layer (`state.viz.icons`, default
  `false`) only ever drew 2 tree styles (conifer/broadleaf) and nothing at all on grassland/
  steppe/desert/tundra. Scoped via `AskUserQuestion` to the broadest option. Trees grew to 5 kinds
  keyed off the frozen `BIOME_KEYS` biome index — conifer/broadleaf unchanged, + rainforest
  (temperate rainforest/jungle), + savanna (sparse, thinned by a hash-probability keep test), +
  wetland (a REAL terrain signal via `currentWetlandMask()`, checked first and overriding whatever
  biome sits underneath a marsh pocket). New ground-scatter category: shrub (grass/steppe),
  cactus (warm desert), boulder (tundra + cold desert, split by `tempField` at 10°C). `tempField`/
  `wetlandMask` are OPTIONAL additions to `placeMapIcons`'s `opts` — the function stays the "pure
  primitive, no globals" contract the headless suite already relies on. Mountains/hills gained
  variety in the **procedural fallback only** (snow cap below ~2°C, rockier hill outline when
  arid) — pack art for those two slots deliberately stays climate-unconditioned, a disclosed scope
  cut to avoid schema churn. Every new kind is both auto-scattered AND manually placeable via the
  Icon tool's "Feature icons" family (`PACK_ICON_SLOTS`/`CIV_FEATURE_ICON_TYPES` both 4→10). The
  sample-pack generator (`assets/make_sample_pack.py`) gained 6 matching procedural silhouette
  functions and `sample_pack.zip` was regenerated (21 icon sprites, up from 9). Verified: engine
  **992/992** (+8), UME **852/852 unchanged**, hash vs v1.19 — `default`/`geoid`/`waves`/`ao`
  **ALL IDENTICAL**, the `icons` scenario **intentionally diverges** for the first time (the
  feature is opt-in, so this is the expected effect, not a regression), smoke **236/236** (+5),
  the regenerated sample pack imports with zero warnings, fixed-seed whole-world screenshots
  confirm rendering across real biome diversity.
- **v1.19 — owner: "in generate - sculpt I have one small caveat. On mobile/android/ios when I
  need to drag the screen I'll paint at the same time. Can we put a small graphic joystick in the
  bottom right corner just as the cartalith v1.915 has."** A single-finger drag over the canvas is
  captured as a sculpt paint stroke, so on touch there was no gesture left to pan with while
  painting — the existing `#panBtn` ✋ toggle forces an either/or choice. Ported
  `Cartalith_V1.915.html`'s own "ANDROID NAV PAD" pan-joystick mechanics (never edited the source
  file — read-only reference, per the "kept as reference" rule) as a new small `#sculptNavpad`
  stick, stacked above `#zoomOverlay` in the same bottom-right corner (touch-only, shown only
  while `_sculptEditorActive()`), simplified from V1.915's full pad to just the stick since Gen1
  already has dedicated zoom buttons. `_sculptNavSetKnob`/`_sculptNavPanLoop`/
  `_sculptNavResetKnob` are a direct port of V1.915's own knob-to-velocity/rAF-loop functions; the
  loop drives whichever camera is active (`viewT.panX/panY` off-LOD, `_lodCx/_lodCy` under Tiled
  LOD), branching on `_lodOn` exactly like the existing drag handlers, so a nudge pans identically
  to a real drag, just relocated off the paintable canvas. Verified: engine **984/984** and UME
  **852/852** (both unchanged — script-block-1 UI/CSS only), hash vs v1.18 **ALL IDENTICAL**,
  smoke **231/231** (+8), fixed-seed mobile-viewport (Android UA + touch emulation) screenshots
  confirming placement and the dragging/accent-color knob state.
- **v1.18 — owner (2-part request): "making the religion system fully editable and... introducing
  a detailed interactive city view accessible from Explore mode."** Owner prioritized the City
  Viewer first (via `AskUserQuestion`): extend the existing UME engine rather than build a new
  one, reuse existing LOD/cache/viewport-cull patterns rather than new generic infra, and defer
  religion editing entirely to a later, separate effort (untouched here — still per-faction/
  categorical). Research established the decisive reframe: `UME.cityGen`'s model already carries
  named districts, real growth-stage history (`wall.history`/`parcel.age`/edge `.epoch`), and a
  full civic/religious/economic building roster (`churches`/`markets`/`civic`/`games`/`details`)
  that **neither existing renderer ever drew** — so this shipped almost entirely as a civ-layer
  rendering/camera/UI project, zero new `UME.cityGen` capability, zero new `opts.*`, UME suite
  untouched. Second finding: Explore mode's "Info" tool (`_civInfoAt`) only ever filled a plain
  text sidebar — the rich preview popup is a Civilization-mode (world-building) tool. **Entry
  point**: `_civInfoAt` gained a tight pin-hit test (reusing `_civSelectPlaceAt`'s pick radius) —
  a genuine settlement hit opens the new City Viewer instead of the plain summary; a miss falls
  through unchanged; the Civilization-mode editor (`_civOpenPlacePopup`) is untouched. **Shell**:
  new full-viewport modal with its own pan/zoom camera (`_cvCam`, mirroring `viewT`/`zoomAt`/
  `panDrag`'s math but fully self-contained), opened via the existing `_umModelForNow`. **LOD
  draw pipeline** (`_cvDrawCity`): extends `_umDrawLayoutPreview`'s layer stack, camera-scale-gated
  — parcel district fills/gates/plaza at "city," courtyard distinction at "neighbourhood," and
  civic/religious/market/clutter glyphs (viewport-culled via the same bbox-margin idiom
  `drawCivLayer` uses) at "max." **Info panel**: 7 sections sourcing only data confirmed to exist
  (`_umSiteProfile`, `_civFactionAggregates()`, the `_civPlace*` primitives, `wall.history`),
  faction-level figures explicitly labeled, genuinely unsimulated data (per-settlement religion,
  a structured event timeline) gets an honest "not yet modeled" note rather than a fabricated
  value. Edit button routes to the existing settlement editor — deep procedural-layout editing
  (rename a district, place a monument, edit an individual road/bridge) is explicitly out of
  scope (needs a new persisted per-city edit-overlay model, a separate future feature). Verified:
  engine **984/984**, UME **852/852** (both unchanged), hash vs v1.17 **ALL IDENTICAL**, smoke
  **223/223** (+7), fixed-seed screenshots (overview + max-detail tier).
- **v1.17 — owner: "Settlement Generation Audit & Refactor... make settlements emerge naturally
  from the world's geography instead of appearing as generic procedural stamps... The renderer
  should never invent geography."** Delivered in the owner's requested order: the full
  architectural audit FIRST (`docs/research/settlement-generation-audit.md` — 25 subsystems
  classified, weaknesses ranked; decisive findings: the UME engine modeled water but not land
  — 3 invented Gaussian hills drove every in-town terrain decision — and no generated
  settlement ever had a `specialisation`), then phases S1–S7, every engine capability keyed on
  a NEW `opts.*` (v0.98 guard pattern ⇒ synthetic path byte-identical). **S1** `_umSiteProfile(p)`
  (cached Site Profile from existing primitives; surfaced in the Inspector) + the model-cache
  water-fingerprint collision fix (mask bytes FNV-hashed, not counted). **S2** auto-populate
  derives `p.specialisation` from the profile (scored rules, 0.30 floor → honest 'none';
  `==null`-guarded so hand-set values survive); `mining` trait re-keyed off real hinterland ore
  (was elevation>0.55); faction culture passthrough. **S3** `_umTerrainCtx(p)` — the land twin
  of `_umWaterCtx` (22 m bilinear raster of real `field`); `buildSite.height()` reads it,
  every downstream consumer (street costs, market siting, bridgePt, slope rejection,
  suitability) just works; `terrainAware` on ⇒ steep/flooded parcels excluded. Latent
  2-point-riverPath crash in `addRiverBridges` fixed. **S4** `_umWallSpec(p)` none|ditch|
  palisade|stone ladder (fortress always stone; most hamlets honestly unwalled; `umWalls`
  override wins) threaded through `opts.wallStyle` to style-aware rendering; wall ring deflects
  (bounded, relief-relative) onto genuinely higher real ground — proven by a synthetic-ridge
  unit test + the `wallState.terrainDeflected` diagnostic. **S5** `_umWaterCtx` exports
  `riverOrder`/`seaLakeCells` (pre-river-stamp); decorative auto-bridges gone —
  `detectRiverCrossings` on the FINAL graph records road×centerline intersections as
  `site.bridges` (the road IS the bridge), crossing-less through-towns get a `site.ford`;
  `buildHarbour` requires navigability (sea/lake ≥40 cells or order ≥3) + non-cliff shore,
  stamping `site.harbourInvalid` otherwise (fixed-seed run: 26 stream "harbours" suppressed,
  16/16 bridges road-justified, 4/4 crossing-less through-towns got fords). **S6**
  `opts.economy={specialisation,oreBearing}` → bounded district overrides (oreyard/fishery/
  sawyard/granary/warehouse/paddocks) with physical predicates + scale-relative hamlet
  fallbacks, yard-shed grammar, per-economy details (spoil heaps/drying racks/log booms),
  renderer tints (36/38 economy towns carry their district; zero leakage). **S7** new
  `siteprofile` debug raster view (slope+flood buildability composite, full established
  pattern incl. `renderDebugTile` + new `currentSlopeField()`) + a `state.viz.civDiagnostics`
  vector overlay (footprint box + specialisation/wall-spec/river-class/bridge-ford-harbour
  validity card per settlement; peeks the model LRU by seed prefix, never triggers
  generation). Verified: engine **984/984**, UME **852/852** (+21), hash vs v1.16 **ALL
  IDENTICAL**, smoke **216/216** (+5), fixed-seed screenshots.
- **v1.16 — owner: "redesigning the Civilization interface... a UI and data architecture
  refactor, not a rewrite of the underlying simulation... generation tools at the top, followed
  by faction administration, settlement management, and world statistics."** `#genCiv` reorganized
  around **Generation → Factions → Settlements → Economy → Statistics** sub-pages (`#civSubBar`,
  mirroring `#genSubBar`'s own pattern), replacing the old flat Peoples/Settlements/Polity/
  Infrastructure `<details>` accordions. All work in script block 2 + HTML around `#genCiv`;
  blocks 1/3/4 untouched throughout. **Data layer** (`_civFactionAggregates()`, cached/gated on
  `[_civAggGen,_civTerrGen,_fieldGen,faction count]`, modeled on the pre-existing
  `_civRegionalPopulation()` single-pass shape): one GW×GH pass + one `state.places` pass per
  faction → population (Σ settlement pop, NOT the density-integral ceiling — that's kept
  separate as `foodProductionCapacity`), territory km², food surplus, trade/tax income, a
  5-way power breakdown (military/economic/political/cultural/religious + derived overall,
  explicitly labeled heuristic, never presented as simulated), imports/exports/strategic
  resources (territory-mean resource potential vs. world mean), and a craft-production share
  (non-primary-sector population, the one field with no direct backing signal — labeled
  "(approximate)"). Never re-runs `_civNetworkMetrics`' Brandes betweenness — faction
  economic/political numbers consume the persisted `economicImportance`/`tradeVolume` from the
  last Auto-Populate/Generate-Roads run (accepted staleness tradeoff). New `civFactionGovernment`
  parallel array (`CIV_GOVERNMENTS`, 9 types) follows the exact v1.07 culture/v1.10 religion
  convention (push/pop in add/remove-faction, old-save-compatible rebuild in
  `_civSyncFromState`). New `_civFactionBannerCanvas` draws a small deterministic per-faction
  shield glyph (pure rendering, cached, no new persisted bytes). **Generation page**: pure
  relocation of Auto-Populate/`civAutoPolityBtn` (relabeled "Recalculate Territories")/
  `civAutoRoutesBtn` (relabeled "Generate Roads") — same ids/handlers, zero logic change;
  provinces/territory-radius/scale-opacity sliders/way list parked under an "Advanced"
  disclosure. **Factions page**: existing pill picker kept (still drives `_civActiveFaction` for
  the Territory/Settlement map tools — a deliberately separate concern from Faction Inspector
  browsing) above a new richer faction list; clicking a faction opens `_civPopulateFactionEditor`
  inline (Name/Government/Culture/Religion editable, Capital/Population/Territory/Power/
  Economy/Diplomacy-placeholder read-only from the aggregate) with a lazy-loaded settlement
  sublist (`<details>` `toggle`, gated on `builtGen===_civAggGen` so an unchanged re-expand
  rebuilds nothing — node-identity verified). **Settlements page**: a virtual-scrolling
  sortable/filterable table (`ST_ROW_H=28`, fixed recycled row pool + overscan, positioned via
  `transform:translateY`) — search/faction/type/econ-role/pop-range filters plus 7 sort keys
  (Population/Prosperity/Importance/Economy/Age/Distance/Military value), `state.civTableFilter`
  (transient, not persisted). Verified against a synthetic 4,000-settlement world: DOM row count
  stays ~21-28 regardless of total count, filter/sort results cross-checked against an
  independent recomputation, row click opens the identical popup a map click would. The
  Settlement Inspector (`_civPopulatePlaceEditor`, still the map-anchored popup, not replaced)
  gained read-only Prosperity/Food-surplus/Defensibility/Connected-roads/Connected-rivers/
  Nearby-resources rows plus a Focus-camera button; all thin wrappers around already-cheap
  single-point/small-radius primitives (`_umInferWalls`, `_umSiteKindFromTerrain`,
  `buildSettlementSuitability`'s defensibility term, `currentResourcePotentials()`'s cached
  fields, `_civCatchmentDensityMean`) — no new full-grid pass anywhere. **Economy/Statistics
  pages**: render exclusively from `_civFactionAggregates()` + trivial extra tallies (way km/
  count, settlement-tier counts, mean prosperity), gated by `_civRefreshActiveSubPage()` so
  either page costs nothing while inactive (verified: switching away and back leaves the DOM
  byte-identical without an explicit refresh). Old `_civRenderSettlementList()`/
  `#civSettlementList`/`#civSettlementCount` (superseded by the virtualized table) deleted
  outright, all 3 call sites cleaned up — confirmed zero remaining references. One pre-existing
  smoke assertion ("Civilization → Polity no longer duplicates the timeline controls") checked a
  `<details>` literally named "Polity", which no longer exists after the redesign folded it into
  Generation → Territories; updated to check the whole `#genCiv` subtree instead (the invariant
  it guards still holds). Verified: engine **984/984** (block 1 untouched throughout), UME
  **831/831** (block 4 untouched), hash vs v1.15 **ALL IDENTICAL** at every phase (P0 data-layer-
  only, then each UI phase — nothing in the redesign touches any render path), smoke
  **211/211** (0 new assertions added to the suite file directly, but every phase was probed via
  standalone Playwright scripts: P0 numeric/cache-identity sanity, P1/P2 sub-tab shell + A/B
  button-relocation parity, P3 settlement-inspector cross-checks, P4 faction-list/inspector
  lazy-load + node-identity, P5 4,000-row virtual-scroll stress test, P6 economy/statistics
  cross-checks against independent recomputation).
- **v1.15 — owner: "integrate this sculpting tool and replace the one that's already in
  cartalith v1.14... follow the planning doc... goal is to complete this work completely."**
  Full port of `fractal-geology/Fractal Geology Painter v0.1.html`'s stamp-based,
  non-destructive terrain sculptor per `docs/SCULPT_EDITOR_INTEGRATION_PLAN.md`, **replacing
  the old "Manual Terrain" accordion entirely** (plotline feature brush + direct paint, both
  fully retired — HTML and JS). New 4th Generate sub-tab "Sculpt": paint intent with a
  13-feature registry (mountains/hills/ridge/plateau/cliff/canyon/valley/river/lake/basin/
  coastline/volcano/freehand, each with its own fractal edge-warp character) into a
  session-scoped DRAFT stamp stack (own undo/redo, never touches `field`, previewed as a
  translucent overlay) → **Commit** bakes the whole stack once, re-clamps any pre-existing
  locked river channel a non-river stamp raised (`enforceRiverChannels()`), carves+locks new
  River stamps, deposits Lake stamps into `lakeMask` (same mechanism the retired Water tool
  used), then one flow/climate recompute + one undo snapshot + one render. Brush size lives in
  grid cells (real-world/zoom-independent), pointer capture is LOD-aware throughout. **Two real
  bugs caught by the new test coverage and fixed:** the river-commit hook was passing `{x,y}`
  stamp points to a helper that expects `[x,y]` arrays (silently carving at grid cell (0,0)
  every time — a total river-tool failure that would have shipped unnoticed); and commit never
  re-protected a pre-existing locked river channel from being raised by an unrelated stamp.
  Verified: engine **984/984**, UME **831/831** (untouched), hash vs v1.14 **ALL IDENTICAL**
  (no stamps committed by default), smoke **211/211** (+8). See the CHANGELOG v1.15 entry for
  full detail on what old code was retired vs. kept.
- **Owner: "implement the top 6 borrow list from the research"** — `docs/research/azgaar-comparative-
  analysis.md` §4's ranked list, comparing against Azgaar's Fantasy Map Generator. **ALL 6 ITEMS
  SHIPPED**, one version per item (per the "finish one thing before starting the next" rule):
  (1) culture-flavored naming — v1.07. (2) setup-gate world archetype presets — v1.08. (3)
  GeoJSON/GIS export — v1.09. (4) province tier + religions layer — v1.10. (5) submap/resample UX —
  v1.11. (6) label placement + per-layer style editors — v1.12. The borrow-list arc is complete.
- **v1.14 — owner: "a multitude of rivers... in close proximity, as if two different engines are
  trying to achieve the very same thing... poor unnatural looking."** Confirmed literal: two river
  renderers really were both drawing the same network on top of each other on the main (non-LOD)
  Biome view — `surfaceColor()`'s per-pixel raster blend (Strahler/Rosgen width + Beer–Lambert
  depth off `_riverNet.intensity`/`depth`) AND `drawRiverWays()`'s Catmull-Rom+sinuosity vector
  spline over the SAME `_riverNet` (the v0.94 comment literally said "both render"). The vector
  path's smoothing/jitter wanders off the raster's exact cell-centerline, so away from dead-centre
  the two diverge into what reads as a second, parallel river — worst on terrain with many close
  channels. Fix: `surfaceColor`'s raster blend now skips itself whenever `state.viz.riverWays` is
  on (matching the Strahler debug view's existing vector-only precedent); off keeps the exact
  pre-v1.14 raster-only render. **Flagged, not fixed:** a separate, deeper "closely-spaced parallel
  channels" hatch pattern showed up in BOTH raster-only and vector-only renders during
  investigation on certain uniformly-sloped terrain — that's the underlying drainage network's own
  density (a flow-routing/channel-threshold question), not a render-duplication artifact; would
  need a separate engine-level pass if pursued (`state.viz.minRiverOrder` already thins it
  per-map). Verified: engine **923/923** (field/temp/rain/flow hashes identical to v1.13 — vctx-only
  change), UME **831/831**, hash vs v1.13 shows the intended default-rendering rgba diff (not
  neutral — this is a default-behavior bug fix), smoke **204/204** (+1).
- **v1.13 — owner: "3 fixes: labels give no visual results; let me zoom out until the full map
  WIDTH stays in view (furthest zoom-out currently uses map height, forcing L/R drag); clicked
  info keeps its coord at the original zoom, doesn't adapt."** Three civ/UI bug fixes, engine
  untouched. **(1) Region/area name labels stopped drawing:** the shared occupancy grid (v0.148)
  let a settlement auto-label claim the region label's cell — worsened by v1.12's extra label
  boxes — so §4's `if(!lblTest(...)) continue;` skipped it (owner: 0 of 2 drew). Fix: a pre-pass
  reserves each region label's box BEFORE the settlement loop (settlement labels now yield to
  user-authored names), and §4 draws region labels **unconditionally** (no collision skip → the
  selected label can't erase itself either). **(2) Zoom-out floored at COVER, not FIT:** the fill
  floor was `_viewCoverScale()` (max ratio — fills, but the other axis overflows). New
  `_viewFitScale()` (min ratio — whole map visible, letterbox on the overflow axis) is the new
  `zoomAt`/`_viewClampFill` floor; a fitting axis is **centred**. The default/reset view still
  FILLS (cover) via new `_viewFill()` (routed through reset button, post-generate/import, resize,
  load), so only deliberate zoom-out below cover reveals the full map. **(3) Clicked info kept the
  un-zoomed coord under LOD:** the left-click civ-tool `pointerdown` (and territory-paint
  `pointermove`) used plain `evtToGrid`, which assumes the world fills the canvas — wrong under the
  auto-entered tiled-LOD viewer (~260 cells off at deep zoom). Switched to LOD-aware `evtToGridLOD`
  (same inverse the v0.91 info/wildlife + v0.95 right-click paths use; off-LOD it falls through to
  `evtToGrid`). Verified: engine **923/923**, UME **831/831**, hash vs v1.12 **ALL IDENTICAL**,
  smoke **203/203** (+3: region label draws through a packed grid; zoom-out floor fits map W+H
  where cover overflowed; deep-LOD click reaches `_civInfoAt` on the right cell). Screenshots
  confirm labels at the filled default and both map edges visible at max zoom-out.
- **v1.12 — owner: "implement the top 6 borrow list from the research" (#6, and last, label
  placement + per-layer style editors).** Settlement/POI labels only ever tried one fixed spot
  (above the pin); a collision there silently dropped the label. `drawCivLayer`'s placement loop now
  tries above→below→right→left (new `lblTestBox`/`lblMarkBox`, sharing the same occupancy grid as
  the point-based `lblTest`/`lblMark` the region-name-label system still uses unchanged);
  `_civDrawSettlementPin`/`_civDrawPoiPin` gain `opts.labelPos` (default `'above'` — every pre-v1.12
  call site draws identically). Measured: 5 same-tier cities packed tighter than their own label
  width showed 1/5 labels pre-v1.12, **2/5** post (rescued via `below`). Two new per-layer style
  sliders (Settlements panel): **Territory fill opacity** (default `130/255`, the exact prior
  hardcoded alpha) and **Way opacity** (default `1`) — both fold into the existing per-pixel render
  passes, no new draw calls. Verified: engine **923/923**, UME **831/831**, hash vs v1.11 **ALL
  IDENTICAL**, smoke **200/200** (+2). Playwright A/B vs v1.11 on the identical packed-cities case
  confirms the concrete before/after (`positions:[null]` → `positions:['above','below']`).
- **v1.11 — owner: "implement the top 6 borrow list from the research" (#5 submap/resample UX).**
  Cartalith already had `amplifyRegion()` (seamless world-space heightmap upsampling), a region-select
  drag tool, and a tiled-.zip "Region export" pipeline — but it only produced FILES, never a live
  world. New **Extract as new world** button (same panel): reuses the same selection + `amplifyRegion`,
  replaces the live world with the amplified region, hands off to the EXISTING Import-heightmap
  calibrate→`inferTectonics()` pipeline (no new world-construction path). Deliberately skips
  `normalize()` (unlike `loadImage`'s raw-pixel path) — the amplified data is already real elevation
  in the parent's `[0,1]` space, and renormalizing would corrupt the sea-level/relative-height
  continuity that's the whole point. New `mapWidthKm` = parent width × the selection's true fraction
  (smaller region ⇒ higher-resolution close-up, not a rescale). Civ data (settlements/roads/territory/
  provinces) is cleared on extraction (old-extent coordinates, honest reset) behind a `confirm()`.
  Verified: engine **923/923**, UME **831/831**, hash vs v1.10 **ALL IDENTICAL**, smoke **198/198**
  (+5). Playwright-probed: a quarter-map region (256×164 of 512×328) extracted at 1024px produced a
  1024×656 world at exactly half the parent's km-width, fully valid after tectonic inference.
- **v1.10 — owner: "implement the top 6 borrow list from the research" (#4 province tier + optional
  religions layer).** New `civProvince` raster (parallel to `civTerritory`) subdivides each faction's
  territory via `_civGenerateProvinces()` (on-demand button): one province per city-tier+ settlement
  (rank ≥3), Voronoi-partitioned within that faction's own territory cells only (never crosses a
  faction boundary — verified), falling back to a single province seeded by the biggest settlement
  when there's no city+. Rendering (opt-in `state.viz.provinces`) folds a per-province lightness
  jitter into the SAME per-pixel territory-blit pass, not a second draw. `civProvince`/`CIV_PROVINCES`
  are deliberately not persisted (pure-derived, regenerate on demand). Religions scoped down to a
  per-faction categorical "state religion" (`civFactionReligion`, `CIV_RELIGIONS` fixed 8-entry list)
  rather than FMG's full spatial spread simulation — the research doc flags that half as optional;
  mirrors the v1.07 culture picker exactly, same persistence pattern. GeoJSON export (v1.09) gains a
  `province` layer sharing the territory tracer's boundary/hole-nesting code, and territory features
  gain a `religion` property. Verified: engine **923/923**, UME **831/831**, hash vs v1.09 **ALL
  IDENTICAL**, smoke **193/193** (+5). Playwright-probed on synthetic two-faction worlds: correct
  province counts (2 city-seeded + 1 fallback), zero cross-faction leakage, exported province area
  exactly tiles the parent territory (ratio 1.0000).
- **v1.09 — owner: "implement the top 6 borrow list from the research" (#3 GeoJSON/GIS export).** New
  **Export GeoJSON** button (File ▾, next to Export .zip): settlements/POIs, roads/sea-routes, rivers
  (Strahler ≥2) and faction territory outlines as one `.geojson` FeatureCollection, each feature tagged
  `layer` for filtering in a GIS tool. Coordinates are local planar km (east, north), NOT WGS84 lon/lat
  — a fantasy world has no real georeference (same call Azgaar's FMG makes); documented in a top-level
  `properties.note`. Territory outlines are the new algorithm: `_geoTraceMaskRings` walks a faction's
  `civTerritory` cell mask into closed boundary rings via oriented cell-edge tracing, classifies
  outer-shell-vs-hole by shoelace sign, nests holes into their smallest enclosing shell by point-in-
  ring — an enclave/lake renders as an actual `MultiPolygon` hole, not a spurious extra polygon. Lives
  in script block 1 (only ever called long after block 2 has run — same deferred cross-block pattern
  `exportZip` already uses for the Asset Library). Verified: engine **923/923**, UME **831/831**, hash
  vs v1.08 **ALL IDENTICAL**, smoke **188/188** (+2). Node-isolated ring-tracing unit tests (solid
  square, donut-with-hole, two disjoint blobs, empty mask) all pass exact area/classification checks.
  Playwright-probed on a real populated+territory-painted world: 225 features across all 4 layers,
  territory area ratio to painted cells = 1.000.
- **v1.08 — owner: "implement the top 6 borrow list from the research" (#2 setup-gate world archetype
  presets).** Cartalith already had the underlying system — `ARCHETYPES` (earth/supercontinent/
  archipelago/volcanic/rift) + `state.world_structure`'s continentality-field steering, exposed in the
  sidebar's Generate → World → World Structure panel — but only reachable AFTER a world existed, behind
  an "Enable continental steering" checkbox. Added a **World shape** preset row to the setup gate
  (`#suArchSeg`): Classic (default — `world_structure` disabled, bit-identical) plus Earth-like/
  Pangaea/Archipelago/Volcanic Isles/Rift Valleys, reusing the same `ARCHETYPES` data. Picking a preset
  calls the existing `deriveFromWorldStructure()` (invariant 5) before the upcoming commit; Classic
  restores true defaults exactly (not just `enabled=false`) so bouncing between presets and landing
  back on Classic reproduces the untouched default world. Verified: engine **923/923**, UME **831/831**,
  hash vs v1.07 **ALL IDENTICAL** (hash harness bypasses the gate entirely — structurally unaffected),
  smoke **186/186** (+3). Playwright-probed: untouched-default and explicit-Classic-click produce an
  identical field hash; Pangaea/Archipelago each differ materially from Classic and each other.
- **v1.07 — owner: "implement the top 6 borrow list from the research" (#1 culture-flavored naming).**
  `_civSettleName` was one global syllable/suffix generator for every faction. Added seven **naming
  cultures** (`CIV_CULTURES`: common/imperial/highland/desert/riverlands/sylvan/maritime), a parallel
  `civFactionCulture` array assigning each faction a culture (deterministic per-index default via
  `_civDefaultCulture`, so the six built-in factions read distinctly with zero setup), a naming-culture
  `<select>` next to each faction pill, `_civSettleName(rng,faction)` looking up the settlement's own
  faction's culture, a 🎲 re-roll button in the settlement editor (mirrors FMG's "regenerate burg
  name"), and `civFactionCulture` round-tripping through the same `state.civ` sync as `civFactionNames`
  (old-save compatible — missing ⇒ rebuilt from the deterministic default). Settlement naming isn't
  part of the hash battery, so free to change without touching cross-version neutrality. Verified:
  engine **923/923**, UME **831/831**, hash vs v1.06 **ALL IDENTICAL**, smoke **183/183** (+4).
  Playwright-probed: six factions pinned to six cultures produce visibly distinct names (Imperial:
  Novarcica; Highland: Kragandward; Desert: Ashqirspan).
- **v1.06 — owner: "maybe we should have the seed box back, and the random option there also."** The
  setup gate's generate form gains a World seed row: `#suSeedN` (prefilled with the boot-random seed on
  open) + `#suSeedRand` 🎲 (rolls a new value into the box; applied on Generate). `_suGenCommit` applies
  the typed seed to `state.tect.seed` (blank = fresh random, the old behaviour); sidebar `#seedN` syncs
  via `syncUI()`. Same seed + size + extent now reproduces the same world from the first generate.
  Playwright: same typed seed across two fresh boots → identical field hash; dice → different world;
  sidebar in sync. Smoke now seeds its boot world (31337) through this input (de-flakes the random-world
  assertions). Verified: engine **923/923**, UME **831/831**, hash vs v1.05 **ALL IDENTICAL**, smoke
  **179/179** (+1).
- **v1.05 — owner: "the blocky water" (#96, square lakes at LOD zoom — deferred since v0.96, FIXED).**
  Above-sea lakes are classified per coarse cell and were stamped by a NEAREST-cell test per pixel in
  the two sub-cell renderers (`renderBiomeTileRGBA`, `bakePixel`) → axis-aligned blue squares when the
  LOD zoom magnified past grid resolution. Fix: `buildWaterBodies` optionally exports its priority-flood
  pooled fill level (`opts.fillOut` → module cache `_lakeFill`, same lifetime as `_waterBody`); the
  samplers now flood the tile's own amplified terrain to the pool surface (interior all-lake cells
  water outright; boundary water where terrain < pool level AND inside a bilinear membership band
  `fq>0.35` that cuts smooth curves where the shelf is too flat for the terrain test to shape).
  Water-brush/flat lakes keep their painted cell shape. BASE per-cell loop untouched ⇒ default render
  bit-identical (hash ALL IDENTICAL). Same-world A/B (state.tect.seed=54869, 710-cell lake, eastern
  shore, 6 km LOD span): hard right-angle squares → smooth terrain-following shoreline. **Probe
  gotcha for future sessions: the setup gate has NO seed input — set `state.tect.seed` directly before
  generating, or every probe run is a different random world** (all pre-v1.05 probe runs were).
  Verified: engine **923/923**, UME **831/831**, hash vs v1.04 **ALL IDENTICAL**, smoke **178/178**.
- **v1.04 — owner: "harbour length + needle".** Root cause of the extreme wall needles: `buildWall`'s
  one-bank branch walks `townBank` between the landArc→bank projections, and on REAL water the bank is
  the real polyline spanning the whole box — degenerate classifications walked kilometres along the
  water (measured 2,210 m). Fix: if the bank walk exceeds max(1.6 × landArc, 500 m), fall back to the
  smooth curtain around the (v1.03-capped) hull; guarded on `usesRealWater` (UME byte-identical).
  Flood probe: max water-wall 2,210 m → 0, median ring aspect 1.1. Verified: engine **923/923**, UME
  **831/831**, hash vs v1.03 **ALL IDENTICAL**, smoke **178/178**.
- **v1.03 — owner (9 screenshots): island town wrongly "in open water", elongated port walls, square
  lakes at LOD.** (1) **Island rescue** (`_umWaterCtx`): the v1.00 mostly-water bail keyed on the whole
  box's water fraction (>0.72), suppressing island/coast towns; now measures the water fraction in a
  ~260 m disc around the settlement and bails only if that's >90% water (true mid-open-water). Islands
  build a small town on the island land. (2) **Elongated-wall cap** (`builtMassHull`): on real water, a
  pathologically elongated hull (needle along the shore/river) is compressed along its long axis to ≤2.4:1
  (guarded on `usesRealWater`; common ~1.3:1 walls untouched). Verified: engine **923/923**, UME
  **831/831**, hash vs v1.02 **ALL IDENTICAL**, smoke **178/178**. **Still flagged for v1.04+:** the
  owner's MOST-EXTREME wall needles + the long HARBOUR-QUAY extent along the shore couldn't be reproduced
  on local seeds — the aspect cap bounds any elongation but on-device confirmation is owed, and the
  harbour-length is a separate constraint; **square lakes at LOD** remain the pre-existing tile-renderer
  resolution limit (#96, deferred — needs procedural sub-cell coast detail in the tile pyramid).
- **v1.02 — owner: "sometimes ways don't connect — they stop just short of a location."** The land
  network (`_civHierarchicalNetwork`) consolidates shared corridors by claiming routing-grid cells
  busiest-first; an edge whose near-settlement cells were already claimed by a THROUGH road starts its
  visible run a routing-cell out, at a downsampled cell CENTRE offset from the pin, so the road stops
  short. v0.92's substitution only fixed the run reaching the edge's OWN endpoint cell; v1.02 adds a
  post-pass pulling any way endpoint landing near its `aIdx`/`bIdx` settlement exactly onto the pin
  (threshold scales with 1/sc AND claimed-corridor depth, bounded to ~45% of `GW/30` spacing so it can't
  reach a neighbour; only snaps to the way's own two settlements). Sea routes already anchored endpoints
  exactly. Verified: engine **923/923**, UME **831/831** (blocks 1 & 4 untouched), hash vs v1.01 **ALL
  IDENTICAL**, smoke **178/178** (+1 guard). Probe 8 seeds: **20 → 0** stop-short endpoints.
- **NEXT (v1.03) — owner bug report (9 screenshots, v1.01):** (1) **island town wrongly suppressed** —
  a settlement on a small island shows "No town layout — sits in open water" because the v1.00
  `mostlyWater` bail keys on the BOX water fraction (>72%), but the settlement IS on land; since v1.01
  snaps settlements onto land, the bail should test land NEAR the settlement, not the box, so
  islands/peninsulas/coasts build a (small) town. (2) **elongated wall slivers + long harbours** — port/
  coastal towns get a hugely stretched thin wall and warehouse/quay fabric strung far along the
  shoreline/river (`shoreFromMask` shoreline as `site.river` + harbour district + `builtMassHull`
  following the whole shore); constrain to a compact shore blob. (3) **square lakes at LOD zoom** —
  pre-existing #96 (coarse grid magnified past resolution at the tile renderer); still deferred.
- **v1.01 — owner: "settlements should not be in water — research and implement; also continue the
  outstanding points."** Three items. (1) **Settlements never stand in water** — research showed every
  placement path already refuses water (`_civSnapLand` checks sea + lakes, `_civDropPlace` refuses wet
  cells, crossroads promotion snaps); the actual root cause was that NOTHING re-validated pins when the
  terrain changed underneath them (erosion / sea-level recalibration / Water brush / imported save = the
  owner's "renders inside a lake"). New `_civSnapPlacesToLand()` reconcile: settlements on water snap to
  the nearest dry cell, dragging connected way endpoints along (v0.92 endpoint invariant); runs once per
  `_fieldGen`+sea-level change from the civ draw path + after auto-populate; POIs exempt (a shipwreck on
  water is legitimate). Probe: 40 placed / 0 wet → sea raised, 17 flooded → one redraw, 0 wet, way
  endpoint followed. (2) **Coastal wall no longer stretches along approach roads**: `builtMassHull`
  discounts bare degree-2 vertices of injected real-road primaries (`g._fromPaths` tag from
  `buildPrimariesFromPaths`; a vertex counts if it's a ≥3-way junction or any town street attaches);
  synthetic path never sets the tag ⇒ UME 831 byte-identical. Browser-verified: wall hugs the built
  fabric now. (3) **Fill mode — map always uses the full display**: minimum zoom is now the COVER scale
  (not letterbox fit) with pan clamped so no background band can show; one clamp in `applyView()`
  catches all input paths; `zoomAt` floor = cover; re-clamp on resize; `_lodFitCanvas` letterbox-COVERS.
  Input mapping untouched (evtToGrid is rect-based). Gotcha recorded in CHANGELOG: the clamp must
  measure against the LAST-APPLIED transform (`_viewApplied`), not pending `viewT`, or the bounds drift
  with the pan being clamped. Playwright portrait 720×1420: initial covered (floor 4.09), zoom-out
  holds, ±4000 px pan clamps to zero gap, centre-click in bounds, LOD covers, no errors. Verified:
  engine **923/923**, UME **831/831**, hash A/B vs v1.00 **ALL IDENTICAL**, smoke **177/177**. Manual
  browser pass still owed: real-device touch feel (pinch/rotate) for fill mode.
- **v1.00 — owner: "harbor at a coastline/river with the city on land, no roads over water from it";
  "tapping a city in explore → a popup with the city layout, a zoom in"; "[a settlement] renders inside
  a lake."** Four settlement-layout fixes + one explore feature, all opt-in / popup so render bit-
  identity to v0.99 holds. (1) **No town roads over open water:** `removeWaterCrossings` gains a real-
  water pass culling primaries/streets that cross open water away from the one bridge (`site.bridgePt`);
  `pruneLargest` drops orphaned far-bank fabric. Guarded on `usesRealWater` (UME suite byte-identical).
  Side benefit: removes the far junctions that inflated the coastal wall, so it hugs the built mass
  tighter. (2) **Town-on-land:** `generate()`'s market nudge searches the whole box (was 340 m) to land
  the centre on real shore. (3) **No floating lake-town:** `_umWaterCtx.mostlyWater` (box >72% water) +
  `_umModelFor` bail keep the bare pin for a settlement in open water. (4) **City-layout popup:** tapping
  a settlement in explore shows a zoomed, fit-to-built-mass render of its town at the top of the editor
  popup (`_umModelForNow` sync generate + `_umDrawLayoutPreview` + `_civOpenPlacePopup`); POIs/in-water
  settlements show none. Smoke's v0.95 crossfade assertion now picks the first settlement whose model
  actually renders (in-water ones legitimately render nothing). Verified: engine **923/923**, UME
  **831/831** (guard holds), hash A/B vs v0.99 **ALL IDENTICAL** (default-off), smoke **177/177** (an
  unrelated v0.73 routing-gravity assertion flakes on the unseeded smoke world — passes on re-run).
  **Still flagged:** coastal wall can still over-enclose along an arterial (deeper hull change); the map
  canvas does not yet fill a portrait/mobile display (letterboxes a landscape map — a core view/projection
  change needing interactive mobile verification).
- **v0.99 — owner: "Continue" (Stage 3 of the seamless refactor — coastal polish).** Two contained,
  safe improvements to v0.98's real-water settlement layouts, both on the opt-in path so render
  bit-identity to v0.98 holds. (1) **Smooth local coastline:** `_umWaterCtx` (civ adapter) now samples
  the height field **bilinearly** per 22 m mask cell instead of the nearest grid cell — at a coarse
  512px region ~70 mask cells collapsed onto one grid cell, so the whole ~1.7 km town box read as one
  blocky, axis-aligned land/water value (the owner's "solid block instead of smooth borders according
  to the heightmap"); the interpolated height crosses sea level smoothly across the box, so the town's
  coastline follows the real heightmap with sub-grid-cell detail. Adapter-only ⇒ engine/UME suites
  unaffected by construction. (2) **Coast orientation fix:** `townBank` (UME engine) hardcoded a `y−5`
  "town is north of the shoreline" offset — only right for the synthetic west→east coast; a real
  sea/lake can face any way, so it pushed the wall the wrong side on an E/W/S coast. Now offset toward
  the actual land (market side), **guarded on `site.usesRealWater`** so the synthetic path (UME suite)
  is byte-identical. Verified: engine **923/923**, UME **831/831** (guard holds), hash A/B vs v0.98
  **ALL IDENTICAL** (default-off), smoke **177/177**. Browser-verified seed 54869 (512px): a
  pure-coastal walled town (bay, ~5k) sits on the real headland behind a smooth curved coast (was a
  blocky block); a river-through estuary town builds entirely on land with the map's water through it.
  **Still flagged (next pass):** on a coastal town the enceinte is sized from the street-graph
  built-mass hull (`builtMassHull`), which folds in arterial junctions and can enclose empty land
  beyond the built fabric — the wall stretches inland along a road while the built mass sits in the
  seaward corner. Pre-existing (v0.97 `primaryPaths`), NOT introduced here; constraining the wall to
  the built fabric is a growth/hull redesign (blocks don't exist yet when `buildWall` runs inside
  `grow()`). "River through town" still reads best at 1K/2K.
- **v0.98 — owner (screenshots, seed 54869 512px): "sea, rivers, lake logic is all but correct" +
  "refactor ... to get a seamless whole ... same for rivers and lakes".** Stage 2 of the seamless
  region↔settlement refactor (Stage 1, v0.97, was roads): the town's WATER is now the map's water.
  Where v0.95/v0.96 gave `buildSite` a *synthetic* river/coast merely oriented to match, v0.98 feeds
  it the REAL map water so the town builds around the actual river/sea/lake. New `_umWaterCtx(p)`
  (civ adapter) packages the real water near a settlement into the layout's local box frame (orient
  forced to 0, referenced to the box centre C = the settlement's real position): (a) the nearest real
  river centerline (`traceRiverPolylines`' nearest stem, resolution-aware search radius — at a coarse
  512px region the ~1.7 km town box is barely one grid cell) and (b) a coarse local raster of ALL real
  water over the box (sea + sub-sea-level lakes, river band stamped in) plus its chamfer distance
  transform. `buildSite(seed,Wm,Hm,kind,opts)` gains `opts`: when `opts.water` is present,
  `isWater`/`riverDist` come from the mask/DT, `river` is the real centerline (or a shoreline extracted
  from the mask for a purely coastal town), and the synthetic water fill is dropped for coasts (the
  real sea is already on the map). The whole synthetic path (no `opts.water` — the headless UME suite)
  is untouched and bit-identical. `generate()` pins the market onto C (nudging off water if C is in the
  channel/sea) so town water AND roads land pixel-for-pixel on the map. A town whose nearest river is
  genuinely a couple of cells off correctly gets NO wrong synthetic river; a coastal town builds on the
  real headland with the sea respected. Verified: engine **923/923**, UME **831/831** (no-water path
  bit-identical), hash A/B vs v0.97 **ALL IDENTICAL** (default-off), smoke **177/177**. Browser-
  verified on seed 54869 (coastal town on real headland, sea not overlapped; river-2.8 km-off town
  draws no wrong river). **Known follow-up (flagged, not blocking):** coastal wall/harbour AESTHETICS
  are rough (market-nudge onto a peninsula → pointed wall + some warehouse sprawl past it); "river
  running through the town" reads best at 1K/2K (the 512px box is ~one cell). Water LOGIC is correct;
  polish is the next pass. **Next: Stage 3 (v0.99) — refine lake/coast shoreline into `buildSite`
  (largely subsumed by v0.98's water mask; a polish/refinement pass, plus the coastal aesthetics).**
- **v0.97 — owner: "build the city around the roads that connect the settlements" + "refactor ...
  to get a seamless whole" (+ "same for rivers and lakes").** Stage 1 of a staged, owner-approved
  refactor toward a seamless region↔settlement whole (Stage 2 = real river centerline into
  `buildSite`; Stage 3 = real lake/coast shoreline — both **pending**). The real inter-settlement
  roads reaching a settlement now ARE the town's arterial skeleton: `_umPrimaryPaths` resamples the
  connected `civWays` by arc length (~55 m — civWay vertices are km apart, which starved the first
  cut into 2-pt stubs and broke the wall), transforms them into the layout's local frame (exact
  inverse of the draw transform, so the injected road overlays the map road pixel-for-pixel), and
  passes them as `opts.primaryPaths` to `UME.cityGen`; the new `buildPrimariesFromPaths` adds them
  as the primary skeleton and `grow()`/`buildBlocks`/`buildWall` build around them. So the
  through-road enters at a gate, runs through the town as its high street, and exits at the far gate
  — seamless. Falls back to v0.96 aligned-bearings when nothing connects. Verified: engine
  **923/923**, UME **831/831** (fallback path unchanged), hash A/B vs v0.96 **ALL IDENTICAL**
  (default-off), smoke **175 → 177** (+2 guards: town-around-roads still forms a wall + full-extent
  primaries; paths densely resampled). Browser-verified (map road runs straight through a walled
  town via gates). **Next: Stage 2 (rivers).**
- **v0.96 — owner live-QA on v0.95's urban morphology + two map-render asks.** Batch of fixes
  from the owner testing v0.95 in-browser. Urban-morphology fixes (all still opt-in, default off):
  right-click a settlement works again under deep zoom (`evtToGridLOD` in the context menu — the
  Age/Fortifications fields were unreachable because of this); the town's main roads now lock to
  the map roads (`_umRouteEnds` matches on the way endpoint COORDINATE at the settlement, not the
  `aIdx`/`bIdx` that several split runs of one edge share); the layout is rotated to the real
  terrain (`_umTerrainOrient` — river axis from a PCA of nearby high-flow cells, or the sea
  direction, or 0 landlocked — so a river town's river runs the same way as the map river and a
  landlocked town has none, with road bearings pre-rotated to compensate); the city wall draws
  the CLOSED ring so it goes around the town; Age/Walls edits repaint; the layout is opaque at
  full zoom (solid fills, crossfade via the layer alpha only); harbour size scales with port
  population (`_umHarbourScale`, bit-identical to the PoC at default). Two map-render changes
  (intentional default-render adjustments, engine fields bit-identical, only `rgba` moves):
  rivers-as-ways redrawn in water-blue that deepens with order (the old hsl ramp went cyan→GREEN
  →orange) and de-"barcoded" by starting the vector ways at order 2 (the ~5,000 order-1 trickles
  stay in the raster water tint). **Deferred/known limitation:** blocky water borders at deep LOD
  zoom (coarse 512px field magnified past its resolution at the land/water threshold — pre-existing,
  same class as v0.92's blocky-lakes work; needs procedural sub-cell coast detail in the fragile
  tile renderer, scoped as a focused follow-up). Verified: engine **923/923**, UME **831/831**,
  smoke **173/173**, hash battery field/temp/rain/flow identical (rgba differs by the river
  restyle), fixed-seed screenshots (opaque walled towns, roads through gates, river aligned;
  global rivers clean blue).
- **v0.95 — owner request: refactor the `urban-morphology/` proof-of-concept (a standalone
  procedural city-layout generator) into Cartalith, with a deep-zoom reveal (settlement pin fades
  into its generated street layout, main roads locked to the region's own route network), a
  map-wide opt-in toggle, and settlement-popup Age/Fortifications controls inferred from
  population/tier by default.** Shipped as a new 4th `<script>` block (the PoC's pure, DOM-free
  engine, `UME.cityGen`) plus a civ-layer adapter/renderer on top of it, all opt-in
  (`state.viz.urbanLayouts`, default off) so render bit-identity to v0.94 holds at defaults
  (`hash_gen1.js` ALL IDENTICAL). Full detail — the `_umPlaceContext` adapter's age/wall
  inference and real-terrain site classification, the `routeEnds` road-locking hook, the
  `lodSpanKm()`-gated pin/layout crossfade renderer, the per-settlement generation cache/queue,
  and the popup fields — is in the CHANGELOG v0.95 entry (long, this was a full subsystem port).
  Verified: engine `tests/run.sh` **923/923** unaffected (script block 1 untouched); new
  `tests/run_um.sh` (ported PoC suite against the embedded block) **831/831**; `hash_gen1.js`
  A/B vs v0.94 **ALL IDENTICAL**; `smoke_gen1.js` **165 → 173** (+8 v0.95 assertions), **173/173**;
  fixed-seed Playwright screenshots confirm the crossfade reads correctly at 40/20/14/6 km spans
  (faint street web bleeding through a faded pin at mid-zoom → full walled-town layout with
  region roads visibly continuing into the settlement at deep zoom). **Deferred** (see CHANGELOG
  for the full list): faction→culture mapping (culture fixed to `'medieval'`), full terrain-
  sourced site geometry (currently type-classification only, not the river curve/bridge/harbour
  placement), the PoC's parcels layer + fine detail objects (trees/wells/crosses/etc.) in the
  canvas renderer, an era signal driving wall-vs-star-fort epochs over time.
- **v0.94 — owner /goal: "go on with the 4th proposal [colorization loop restructuring], draw
  rivers as ways as in the legacy cartalith app, and make route planning take sea-faring routes
  into account... when a split or partial [route] by sea or river is possible it opts to only use
  land based routes."** Three parts, plan confirmed with the owner beforehand (river-ways overlays
  on top of the raster blend and becomes the new default-ON; routing fix scoped to the interactive
  Route tool/journey planner, not the auto-network builder). (1) **Colorization-loop
  restructuring, re-scoped narrower.** v0.93 deferred this outright; fresh research proved buffer
  pooling is alias-safe (the RGBA output is never retained past its synchronous call anywhere) but
  showed allocation is 2-3 orders of magnitude cheaper than the per-pixel compute loop it sits
  inside, so pooling was evaluated and skipped as not worth it. Shipped instead: `sampleArr`
  row-hoisting (`sampleArrRowPrep`/`sampleArrRow`, eliminates redundant per-pixel recomputation of
  the row-only part of the bilinear sample, proven bit-identical, confirmed via the `--full`
  35-config hash battery). Palette-function scratch-ification (the project's own next roadmap
  item) was designed, then also deferred — it surfaced a genuine nested-call aliasing hazard
  (`grassCol` calls `ramp3` twice before consuming either result) that needs a proper multi-slot
  design, not a rushed single-buffer one. (2) **Rivers as ways.** The legacy `Cartalith_V1.915.html`
  drew every travel network (river/road/rail/sea) as one shared stroked-polyline "way"; Gen1 instead
  rendered rivers as a per-pixel raster blend, with true vector strokes existing only inside the
  opt-in Strahler debug view. That existing spline pipeline (previously duplicated between the
  main-canvas and LOD debug-overlay code) is now factored into one shared `drawRiverWays()` and
  exposed as a new default-on **"Draw rivers as ways"** checkbox, overlaid on top of the existing
  raster water blend on both the main canvas and Tiled LOD — a deliberate default-render change
  (`loadZip` back-compat guard keeps old saves on the old look, same pattern as v0.80's ocean-
  currents flip), verified via a targeted A/B (forcing `riverWays:false` reproduces v0.93's hash
  exactly, proving nothing else changed). Also closes a pre-existing LOD gap where the default
  Tiled-LOD Biome view never showed the river network's color at any zoom. (3) **Sea/river-aware
  routing.** Root-caused `_civMixedCostGrid` (the one function deciding both the Route tool's path
  and every journey): water cost (1.5) was tuned *above* flat land, backwards from the journey
  planner's own ~2.5× sea-speed model; land cost ignored biome friction other cost grids already
  use; real rivers carried zero cost information. Fixed by rebalancing water below land
  (`_CIV_SEA_COST=0.6`), sharing the biome-penalty table, and adding real river costing
  (`_CIV_RIVER_COST_BASE=0.85`, order-scaled, floor-only so a river never makes a cell worse).
  Scoped to the interactive tool only, per owner decision — the auto-generated world road network
  stays untouched, flagged as a possible follow-up. Verified via an independent Playwright A/B on
  six coastal detour-prone point pairs: v0.93 committed 5-6% water (essentially all-land) on two of
  them, v0.94 committed 35-50% water on the identical pairs — every pair showed equal-or-higher
  water usage. Render battery: `field`/`temp`/`rain`/`flow` identical everywhere (no engine change
  anywhere in this version); `rgba` differs at biome-mode configs by design (river-ways default).
  Headless **923** unchanged throughout; smoke **159 → 165** (6 new assertions across all three
  parts). Same-day, before v0.94: a **v0.93 hotfix** shipped first (live-testing report: lake edges
  blocky again, LOD tiles seemingly uncached) — see the v0.93 hotfix entry below for the full
  root-cause (a rapid continuous-zoom gesture let the progressive-overview stretch placeholder
  compound staleness without bound) and fix (`_lodOverviewStretchStreak`, bounding consecutive
  un-landed stretches instead of any single stretch's ratio, after a first ratio-cap attempt broke
  the legitimate single-big-jump case).
- **v0.93 — owner /goal: "make the proposed optimisations in a new version, keep a focus on
  graphic fidelity (no pixelated views or blockyness when zooming in on terrain)."** A prior
  session (on request) had produced a ranked list of 6 LOD-render/tile-pipeline optimization
  proposals (the engine's `generate()` stages were excluded from consideration — the bit-identity
  invariants make that path too risky to touch for speed alone). This version implements 3 of the
  6, all opt-in on the LOD path — render battery **ALL IDENTICAL to v0.92**, headless **923**
  unchanged, smoke **157/157** (+3 new assertions): (1) **progressive overview rebuild** — a zoom
  step with a previous overview in hand now stretches it and returns in <30ms instead of blocking
  on a full synchronous rebuild, with the real rebuild deferred to a background pass
  (`_lodScheduleOverviewRebuild`) guarded by `_fieldGen`/render-key checks against a regenerate
  landing mid-flight; (2) **GENPOOL extended to tile refinement** — a new task-parallel
  `runTiles()` dispatch mode (vs. the existing row-split `run()`) lets `refineVisibleTiles()`
  compute pool-eligible tiles across cores instead of one-by-one on the main thread (~3.1× faster,
  bit-identical output); found and fixed a genuine cold-Worker JIT cliff (~20× penalty on a
  fresh Worker's first call) with a `GENPOOL.warmup()` step at init; (3) **parallel atlas baking**
  — `bakeVisibleTiles()`/`bakeAllTiles()` batch each pyramid level's tile compute through the same
  pool before the still-sequential PNG-encode/write loop (~25% faster for a multi-level bake). A
  4th proposal (`renderBiomeTileRGBA` colorization-loop restructuring) was scoped, then
  deliberately **not shipped** — pooling its scratch/output buffers risks aliasing with cached
  tile data (a correctness bug that would manifest as exactly the visual corruption the owner's
  fidelity mandate exists to prevent) and per-pixel `sampleArr` hoisting risks floating-point
  reordering the bit-identity invariant doesn't tolerate; left for a future version with a
  narrower, independently-verifiable scope. A 5th, lower-risk proposal (lazy seasonal-field
  allocation — `tempJul/tempJan/rainJul/rainJan` now allocate on `computeSeasons()`'s first real
  call instead of unconditionally in `allocate()`, since seasons default off) shipped alongside the
  three LOD ones. **Fidelity explicitly verified** per the owner's stated concern: fixed-seed
  (424242) Playwright screenshots at overview / immediate-post-zoom (stretched placeholder) /
  settled (pooled-refined) / LOD zoom-cap (~1km scale) all show smooth, continuously-textured
  terrain, no blocky/quantized artifacts; a canvas pixel-diff confirmed refinement genuinely adds
  detail (not a no-op) rather than just being fast. See CHANGELOG for full profiling numbers,
  including two debugging arcs (a headless/SwiftShader canvas-GPU-contention red herring during
  pool profiling, and a same-page-first-call measurement artifact during bake-pool profiling) that
  turned out not to be real defects once isolated.
- **v0.92 — owner /goal: "carry out the [save-export audit's] reported fixes, then analyze why
  the program is so slow when zooming in even when tiles are baked"**, followed same-day by a
  bug report on the resulting fix (render battery ALL IDENTICAL to v0.91, headless **923**
  unchanged, smoke **137 → 148**). Four parts:
  (1) **audit fixes** (`docs/research/save-export-architecture-audit.md` §5, compatibility break
  accepted per owner): `exportZip()` now skips the redundant `map.png`/`tiles/*` flat bake when
  `state.finalized` (the Atlas pyramid already covers the whole map at every baked level — a
  precise signal, only ever true after `bakeAllTiles` finishes clean); the 4 `layers/*.png`
  preview PNGs are now opt-in (a new checkbox, same footing as the existing channel-atlas
  checkbox) instead of unconditional; the "Tiles & LOD" accordion (which buried "Atlas" two
  levels deep and bundled three unrelated features under one label) is split into three
  top-level sections — **Tiled LOD view** / **Atlas cache** / **Region export** — same element
  ids throughout, markup-only. (2) **the deeper question, root-caused with real profiling**: the
  reported "slow when zooming even with tiles baked" was real, but not the mechanism the owner
  suspected — the sharp TILE overlay correctly serves from the atlas when baked, but it draws on
  top of an "instant overview" backdrop that's rebuilt from scratch (full `GW×GH` canvas, the same
  expensive per-pixel colorization used for real tiles) on every zoom-level change, *always*,
  because it's built straight from `field` and never consults the atlas at all — measured **~940ms
  per zoom step at a modest 1024px world, unaffected by bake state** (baked vs. unbaked timings
  were the same order of magnitude). Since that backdrop is already documented as deliberately
  low-fidelity ("NO procedural detail"), rendering it at quarter resolution and letting the canvas
  stretch it (no signature changes needed — the resample/colorize functions already take
  independent output-resolution params) cut a full overview rebuild from **655ms → 53ms (12.3×)**,
  and the isolated backdrop-only cost (tile canvases + atlas already warm — the truest measure of
  a mid-zoom-gesture frame) from **1186ms → 71ms (16.6×)**. Visually verified unchanged character
  (screenshots of the same deep-zoom unrefined spot, before/after) since the backdrop was already
  a coarse placeholder standing in until the sharp tile overlay covers the same ground. Locked in
  with a permanent smoke-suite timing regression guard. See `CHANGELOG.md` for the full profiling
  numbers and reasoning. (3) **Follow-up bug report — "aspect ratio goes weird ... lakes are
  blocky/pixilated again on their edges."** The quarter-res downscale from part (2) starved small
  lakes (no `_coastSDF` smoothing — that only covers the ocean's sea-level threshold) of source
  samples: an 8-12px lake shrinks to 2-3px, and the 4× upscale turns an invisible native-res
  stair-step into a visibly faceted blob. "Aspect ratio weird" is the same artifact described
  differently — a round shape's *local* aspect getting mangled, not a canvas/CSS distortion (every
  measured aspect ratio checked out at ~1.56 consistently). A flat `/2` ratio fixes quality at
  1024px but re-regresses to ~420-440ms at 2048px, since a ratio's cost scales with the world.
  Fixed instead with a **512px fixed target output width** (`ovScale=min(1,512/GW)`), which bounds
  the overview's cost independent of world resolution (~100-130ms measured at both 1024px and
  2048px) while spending full resolution on worlds already ≤512px wide. Verified with fixed-seed
  screenshot A/Bs (lake shapes back to round, matching full-res) and two new smoke-suite
  regression guards (overview canvas width == 512px at the 1024px test world; aspect preserved).
  (4) **Second same-day follow-up — "Graphic fidelity seems to have degraded also."** The 512px
  cap fixed lakes/coastlines, but a broader softness remained at whole-map zoom (screenshots: v0.91's
  fine surface grain reads as a coarser blotch pattern in v0.92). Root cause wasn't the cap itself —
  every OTHER way into LOD (wheel-zoom, pan release, zoom buttons, auto-enter-on-zoom) already
  schedules `refineVisibleTiles()` after a 240ms settle, which draws a full-detail sharp tile over
  the coarse overview; a controlled before/after screenshot confirmed this genuinely restores full
  sharpness once triggered. But the **`lodChk` checkbox's own change handler never triggered it** —
  ticking "Tiled LOD view" and just looking left the user on the coarse overview indefinitely. This
  gap always existed but was invisible pre-v0.92 (the un-refined overview used to be full native
  resolution already); the resolution cap above is what finally exposed it. Fixed by having the
  checkbox call `withBusy('sharpening view…', refineVisibleTiles+renderNow)` immediately when
  checked (same pattern the explicit "Refine" button already uses), guarded by a re-check of
  `_lodOn` inside the deferred callback so a since-unchecked box can't race it. Verified with a real
  `.click()` on the checkbox confirming the visible tile populates with no further gesture. One new
  smoke-suite regression guard. Full battery green (923 headless, bit-identity ALL IDENTICAL,
  148/148 smoke).
- **v0.91 — owner /goal: "…how in explore the timeline should work. Currently it works, bit
  rather clunky"** (no engine changes — script block 2 civ-UI only; render battery ALL IDENTICAL
  to v0.90, headless **923** unchanged, smoke **123 → 130**): the second half of the v0.90 /goal
  request. Chosen direction (`AskUserQuestion`): **"one home, real time-scale."** (1) **One
  home** — timeline authoring (Add year + pills), scrubbing (slider + Animate) and the v0.85
  collapse/recovery simulator, previously split across Civilization → Polity (the controls) and
  Explore → Timeline (a second, synced read-only slider), now all live in Explore → Timeline;
  Simulate sits behind its own nested `<details>` so the common path stays uncluttered. Every
  moved element kept its id, so only markup moved — click/input handlers are untouched. The old
  duplicate slider (`#civTlSlider`) is retired; `#explTimelineSlider` is the only one now.
  (2) **Real time-scale** — `_civWireYearSlider()` used `slider.max = snapshotCount-1` and
  `slider.value = <array index>`, so uneven year gaps rendered as evenly-spaced ticks. Now
  `min`/`max`/`value` are the actual recorded years, with a `<datalist>` giving proportional tick
  marks (screenshot-verified at 3× zoom); dragging still snaps to the nearest recorded year
  (no interpolation model between discrete snapshots exists). (3) Fixed a latent gating bug while
  moving the markup: the Explore Timeline section used to hide itself until `civTimeline.length>0`,
  which would have made it impossible to add the *first* year from Explore — it's now always
  visible, only the slider+playback row gates on `length>1`.
  **Same-day follow-up (owner reports, live QA on the shipped v0.91)**, smoke **130 → 136**:
  (a) *"I dont see the timeline menu in explore"* — the first cut buried Timeline inside the filter
  funnel's collapsed popover alongside Polity/Settlements/Roads, which reads as a filter, not an
  editing surface, and was genuinely easy to miss. Moved to `#explTimelineSection`, a plain
  always-visible `.sec` with its own `<h2>`, same footing as Info/Journeys — no funnel click needed.
  (b) *"layer views arent responding to opacity anymore"* — a real regression from this session's own
  v0.89 work: generalizing `drawLODView()` to tile every debug view means `renderNow()`'s LOD early-
  return now always fires before the opacity-blend code, so the slider went inert under Tiled LOD for
  all ~29 views. Fixed by blending the affordance tile against the ordinary base tile
  (`renderBiomeTileRGBA`/`renderHeightTileRGBA`) inside `drawLODView()` itself, skipped at alpha=1 for
  zero added cost; `_lodRenderKey()` gained `debugOpacity` so the tile caches invalidate when the
  slider moves. (c) *"settlement/wildlife ones arent clickable...anymore"* — a gap explicitly flagged
  as a known follow-up in the v0.89 CHANGELOG: click-to-inspect was gated `!_lodOn` outright since
  `evtToGrid()` assumes the canvas shows the full world, untrue under LOD. New `evtToGridLOD(e)` (the
  inverse of v0.90's `_civPlaceScreenPos`) replaces the block with a correct reprojection. All three
  Playwright-verified via real interaction (dispatched clicks, opacity pixel-diff), render battery
  still ALL IDENTICAL to v0.90, headless 923 unchanged (canvas-interaction/LOD-render fixes — CLAUDE.md
  invariant #3, not headlessly testable).
  **Second same-day follow-up**, smoke **136 → 137**: (d) *"roads nearly miss settlements when zooming
  in"* — `_civSmoothPath()` (every way builder's shared smoothing chokepoint) `Math.round()`ed its own
  first/last point along with every interior point, up to half a grid cell of drift that Tiled LOD
  amplifies into a visible gap between a road and the pin it's meant to reach. Fixed by restoring full
  precision at each run's own endpoints post-round; `_civHierarchicalNetwork()` (the primary auto-
  route/auto-populate builder) had a second, compounding source — its raw points are downsampled
  routing-grid cell centers, not the settlement's real coordinate — fixed by substituting the true
  place coordinate in for each edge's actual first/last run (interior junction-to-junction runs are
  untouched, on purpose — they meet at a shared junction, not a settlement). Verified via a controlled
  probe (two real settlements, fractional coordinates, builders called directly): endpoint distance
  0.0 grid cells post-fix (was up to ~1). (e) *"[Clear places] leaves the routes"* — `civWays`/
  `civJourneys` carry no settlement-id, so deleting settlements left orphaned roads. `civClearPlacesBtn`
  (now labeled "Clear places & routes") clears both, mirroring `civClearRoadsBtn`. Civ layer (block 2)
  only; headless 923 unchanged, render battery ALL IDENTICAL to v0.90 (way geometry isn't in the
  battery).
- **v0.90 — owner request: "editing a settlement should open a pop-up in the viewscreen with the
  settlement properties and information"** (no engine changes — script block 2 civ-UI only; render
  battery ALL IDENTICAL to v0.89, headless **923** unchanged, smoke **120 → 123**): the settlement/POI
  editor moved out of the sidebar-pinned `#inspectorBody` (v0.65) into a floating `#placeEditPopup`
  anchored at the place's own screen position — the `showSettleInfo`/`showWildInfo` popup idiom, made
  editable, reusing `_civPopulatePlaceEditor`'s field logic unchanged. New `_civPlaceScreenPos(gx,gy)`
  projects world coords → screen px (both normal pan/zoom and `_lodOn`) so the popup opens correctly
  regardless of how the place was selected. Per the owner's chosen option, the sidebar "All settlements"/
  "All POIs" lists stay for browsing — their row-click/Edit button now calls `_civMoveViewTo` first (the
  existing "📍 Move viewer here" handler) so the popup opens already centered. Labels/icons are unchanged
  (still the sidebar inspector). Next queued: the Explore timeline rework (see "Next / open" below).
- **v0.89 — owner report: "tiled LOD info-layers don't scale properly"** (no engine changes; render
  battery ALL IDENTICAL to v0.88, headless **917 → 923**, smoke **117 → 120**): root cause was
  `drawLODView()` only tiling `state.debug` ∈ {off, lith, soil, water} — every other debug/info view
  (~26 of them: temp/rain/koppen/resources/wildlife/popdensity/tectonics/wind/ocean/rivers/…) fell through
  to the full un-zoomed `renderNow` pixel loop while the canvas stayed CSS-fitted for the current LOD zoom,
  so switching to e.g. Temperature while zoomed just stretched the whole world into the zoomed box.
  `renderAffordanceTileRGBA` is now generalized to cover every non-'off' debug value (samples the live
  coarse field at world coords — bilinear for continuous, nearest for categorical — and applies the exact
  main-map colour formula); new `debugTileContext(dbg)` builds precomputed fields once per render (not per
  tile); new `tileShade()` gives relief-lit views a tile-local hillshade. Per the owner's explicit "yes,
  overlays too": new `drawLODDebugOverlays()` reprojects wind/ocean arrows, plate-drift arrows, the T1
  boundary graph, Strahler river splines, and settle/wildlife markers onto the current LOD view rect, with
  zoom-scaled (capped at 8×) line/glyph sizes. Known non-regression: Settlement/Wildlife click-to-inspect
  stays `!_lodOn`-gated (pre-existing — `evtToGrid()` has no LOD-zoom awareness); flagged as a follow-up.
- **v0.88 — two owner-reported items** (no engine changes; render battery ALL IDENTICAL to v0.87,
  headless **911 → 917**, smoke **113 → 117**): (1) **LOD zoom capped too shallow** ("highest zoom stops
  at 20km, want 5km") — the ×64 zoom cap was fixed regardless of map width, and `updateScaleBar()` divided
  by the *full* map width even while LOD-zoomed in (the bar's reading never actually changed as you zoomed,
  reading as "stuck"). New `lodMaxZoom()` (scales to `mapWidthKm/5`, floor 64) replaces the ×64 cap at all
  three zoom sites (button/wheel/pinch); new `lodSpanKm()` (the real on-screen width — `mapWidthKm/_lodZoom`
  while LOD is on) now feeds the scale bar. (2) **Export/Import took the atlas separately** — retired the
  standalone **Export atlas…**/**Import atlas…** action pair (header + Tiles & LOD sidebar) and the
  **Embed baked atlas** checkbox; `exportZip()`/`loadZip()` already unconditionally embed/restore the atlas
  + asset library, so **File → Export .zip**/**Load project .zip** are now the sole 100%-round-trip actions.
  The Assets Library's own **Import pack…**/**Export pack .zip** stays as the dedicated asset-pack-only
  pair, per the owner's ask. Both items UI/browser-chrome only ⇒ bit-identical render.
- **v0.87 — two owner-reported UI items** (no engine changes; render battery ALL IDENTICAL to v0.86,
  headless **911** unchanged, smoke **111 → 113**): (1) **LOD/atlas viewport regression** — in LOD mode the
  CSS transform is identity, so the canvas sat at its intrinsic GW×GH size (small tile in a big viewport).
  New `_lodFitCanvas()` letterbox-fits the `#view` element to the `.canvas-wrap` content box while `_lodOn`
  (cleared on exit); transparent to LOD input since `evtToGrid`/LOD-pan read `getBoundingClientRect()`;
  called from `applyView()` + window resize. (2) **Import + Export merged into one "File ▾" header menu**
  (`#fileMenu`, Import section + Export form) — all element ids unchanged so wiring is untouched; the
  `#exportMenu` CSS retargeted to `#fileMenu`; single-shot Import rows + Export .zip close the menu, form
  clicks don't.
- **v0.86 — seven owner-reported fixes/additions** (no engine-simulation changes; render battery ALL
  IDENTICAL to v0.85, headless **909 → 911**, smoke **103 → 111**): (1) **climate redraw** — added a
  `_climGen` counter (bumped by the climate field-writers) to `_civBakeKey`/`_lodRenderKey`, so "Simulate
  weather" repaints instead of serving a stale bitmap; (2) **mobile Assets exit** — the header 🎨 button is
  now a toggle ("← Map", `_carExitAssetsMode`), always visible; (3) **mobile Export dropdown** — pinned to
  the viewport at ≤860px so it no longer clips off-screen; (4) **Layers-popover scroll** — `stopPropagation`
  on the popover's wheel so scrolling the list no longer zooms the map under it (all 31 views already have
  legends — now smoke-locked); (5) **Credits & academic-principles modal** (header ⓘ) — code sources
  studied + the tectonics/climate/civ academic principles, all cited; (6) **Light theme switch** (header
  ☀/🌙, ported from V1.915's Light theme) via `:root[data-theme="light"]` + localStorage, UI-chrome only so
  the map canvas is untouched; (7) **geological Resources layer** — the sea-level handler now invalidates
  the whole affordance/civ derived-cache set (was stale on sea change), and `buildResourcePotentials`
  computes over the **full map** (bedrock potential exists below sea too) instead of masking to exposed
  land. Bit-identity preserved throughout (all changes are UI/off-default-render or debug-view-only).
- **v0.85 — mechanistic collapse/recovery timeline simulator** (owner: "research the mathematics in regards
  to population mechanics (survival and migration rates…) and how to use this new function in regards to the
  timeline"). New `docs/research/collapse-timeline-dynamics.md` (network-robustness/gravity-migration/
  Black-Death-mortality-calibration sourcing) backs a year-stepped simulation: per-settlement stress (trade-
  dependency loss / density-connectivity exposure / undefended-violence exposure, weighted by a **character**
  — trade/disease/conflict/mixed, each fails settlements in a different order) drives mortality + out-migration
  each step; a Zipf/Ravenstein **gravity model** redistributes survivors to reachable settlements (headroom-
  capped, overflow = unplaced diaspora loss); shrunken nuclei demote/abandon past the v0.82 tier floors; a
  **recovery** mode instead compounds logistic regrowth toward each settlement's own catchment ceiling. All
  new functions are pure and deterministic (`_civProximityAdjacency`/`_civBetweennessFromAdjacency`/
  `_civSettlementStress`/`_civMortalityMigrationRates`/`_civGravityMigrate`/`_civCollapseStep`/
  `_civRecoveryGrowthStep`/`_civSimulateTimeline`); the impure `_civRunCollapseSimulation()` wiring reads
  `state.places`, runs the simulation, and writes **one `civTimeline` entry per step** — the *existing*
  timeline slider/pills scrub through simulated history with no new rendering code, since the entries are
  shaped exactly like `civAddYear`'s. New "Simulate collapse / recovery" UI under Civilization → Polity (mode/
  character/severity or regrowth-rate/start-year/duration-per-step + a Simulate button + a stats summary).
  **Never touches `state.places`/`civWays`** — same rule every other timeline write follows. Civ layer (block
  2) only; render battery ALL IDENTICAL to v0.84, headless **909** unchanged, smoke **86 → 98** (pure-function
  correctness + UI wiring, all via Playwright since these are block-2 functions the headless suite can't see).
  Browser-verified end to end on a real auto-populated world (screenshots: panel, configured run, post-
  simulate timeline pills/slider, Explore tab) — 0 console errors, correct entry count and stats. **Deferred**
  (doc §8): new-settlement founding from refugee overflow, true travel-cost distance (currently straight-line
  ×cellKm), regrowth-phase migration (recovery mode is currently single-settlement logistic only, no
  redistribution).
- **v0.85 audit pass (same day)** — five defects found and fixed in a post-ship review of the simulator:
  baseline-centrality map misaligned after any step-0 settlement failure (now `normBByTid` over INPUT
  settlements); annual rates applied once per multi-year step (now compounded `(1−m)^stepYears`, matching the
  recovery stepper and doc §4's Black Death calibration — collapses at default 10-yr steps are now an order of
  magnitude stronger and correctly calibrated); gravity overflow dropped at saturated destinations instead of
  re-flowing to open headroom (doc §5 semantics restored via bounded multi-pass allocation); phantom year-0
  era when simulating onto an empty timeline (civAddYear's v0.62 guard applied); silent overwrite of authored
  timeline years (now confirm()-gated). Smoke **98 → 103**; render battery still ALL IDENTICAL to v0.84,
  headless 909 unchanged. CHANGELOG has the full per-defect detail. NOTE for future sessions: block-2 (civ
  layer) functions are **not reachable from `tests/run.sh`** (it extracts script block 1 only) — put their
  assertions in `tests/perf/smoke_gen1.js`, and don't trust a green 909 to say anything about civ-layer code.
- **v0.84 — fix: restored the "Vertical" sublabel over Sea level/Peak** (owner report). v0.83's Map-width
  removal over-deleted a section heading unrelated to the ask. Pure markup restore; render battery ALL
  IDENTICAL, headless 909, smoke 86 unchanged.
- **Settlement-editor "disappeared" report — investigated, not reproduced.** Tested four realistic flows on
  v0.84 (direct `_civSelectedPlace` API, simulated canvas click via the Inspect tool, right-click "Edit"
  context-menu item from both the Civilization and Explore tabs) — all correctly open the pinned inspector
  editor with 0 console/page errors. Full smoke suite (86/86, including inspector-visibility assertions)
  green. **Next session: get exact repro steps from the owner** (which tab/tool, what preceded it — e.g. did
  it follow running Auto-populate with Recovery phase set, or a save/load, or a specific settlement count)
  before changing anything — no fix was applied since no failure could be triggered.
- **v0.83 — map width removed from the Generate → World sidebar; setup-gate-only** (owner request). Since
  v0.70 the sidebar carried a disabled, read-only `#mapw` copy ("shown for reference") duplicating the real
  editable width input in the setup gate. Removed the duplicate row + its reference legend entirely — map
  scale is now set in exactly one place (New-world/Import gate). Pure UI + dead-handler cleanup (removed the
  unreachable `bind('mapw',…)` that would have thrown, and the now-moot finalize-lock exemption); render
  battery ALL IDENTICAL to v0.82, headless **909**, smoke **86** (2 assertions updated for the removed
  elements). `state.mapWidthKm` semantics/usage sites unchanged — only where it can be entered changed.
- **v0.82 — post-collapse recovery model** (owner: "start it too"; `docs/research/settlement-emergence.md`
  §5–6). Auto-populate can model a world rebuilding after a demographic collapse, below the ecological ceiling.
  Recovery-phase selector under Civilization (`_civRecoveryPhase`: Stable · I Survival · II Subsistence · III
  Regional · IV Mature). Pure `_civApplyRecovery` scales populations by a phase fraction and applies the doc's
  **labour-shortage demotion** — a nucleus scaled below its tier floor drops to the tier its people support
  (former city → village), and a demoted urban site becomes a **fortified settlement in its ruins** (`p.ruins`
  + `fortified` trait); Survival/Subsistence abandon tiny unanchored nodes. Default Stable ⇒ auto-populate
  byte-identical; render battery ALL IDENTICAL, headless **909**, smoke **84 → 86**. Browser-verified the full
  Phase I–IV trajectory (Stable 93k → Survival 6% + ruins → … → Mature 81%). **Deferred follow-ups**: ruin-reuse
  *placement* bias (not just re-scoring the placed set) and surplus-gated *growth over time*.
- **v0.81 — capacity-grounded, map-size-dependent, automatic settlement populations** (owner design doc →
  `docs/research/settlement-emergence.md`). Auto-populate now derives population from the **energy-system
  model**: a settlement's population is what its catchment land sustains (carrying capacity K × the agrarian
  ceiling `AGRARIAN_MAX_KM2=200`, over per-tier real-km² catchments → **depends on map size**) plus, for
  exchange tiers (town/city/capital/metropolis), a centrality-weighted share of a regional urban pool
  (`_CIV_URBAN_SHARE=0.09`) — the option-1-vs-2 **synthesis** (capacity-first base + exchange-node
  concentration; the doc's §3 "a city is the exchange node"). Per-tier `_CIV_CATCHMENT_KM2` /
  `_CIV_SURPLUS_FRACTION` / `_CIV_TRADE_K` / `_CIV_POP_CAP`. The regional total is a **base calculation** run
  at the end of auto-populate (the v0.76 "Estimate" button is retired; the readout auto-updates). Civ layer
  only, render battery ALL IDENTICAL, headless **909**, smoke **83 → 84**. Browser-calibrated at 400/800/2000
  km (tiers in-band, scale with map size). **Next: v0.82 post-collapse recovery model** (Phase I–IV,
  ruin-reuse, labour caps, surplus-gated growth) — owner asked to start it; foundation is these
  capacity-grounded populations.
- **v0.80 — quality-default + persistence fixes + mobile header fix** (owner: "apply all fixes and
  optimisation; check the UX/UI on mobile"). Headless **909**, smoke **83**. (1) **Ocean currents ON by
  default** (`climate.currents` false→true) — cheap, integrated, adds warm/cold coastal-climate realism.
  Like `carveRivers` (v0.145) this is an intentional default flip: **default render no longer bit-identical
  to v0.79** (currents-off reproduces it); `loadZip`'s `==null?false` guard keeps pre-v0.80 saves exact.
  Albedo/seasons stay opt-in (albedo forces the CPU temp path; seasons is heaviest + changes annual→seasonal
  field meaning) — deliberately NOT flipped, to avoid a perf regression. (2) **LOD sculpt-edit persistence** —
  `_lodEdits` now save as sparse (index,value) deltas over the deterministic procedural base (reconstructed
  via `pyramidTile` on load); closes the "un-baked tile edits lost on save" gap. Nothing written when there
  are no edits. (3) **Mobile header fix** (≤860px): header was ~123px tall with **Export ▾ off-screen** on a
  ~390px phone — fixed with `flex-wrap` (buttons wrap to a reachable second row), `#undoMem` hidden, compact
  one-line `h1`; header **123→80px**, no clipped buttons/overflow. Rest of the mobile UX audited sound
  (slide-in drawer, no `aside` overflow, enlarged touch targets, 16px inputs). Browser-verified all three;
  the one rare headless flake is a pre-existing `Math.random()` noise test (test_tail.js:1707), not currents
  (deterministic per seed).
- **v0.79 — deep-zoom oxbow-lake pockets** — closes the last flagged river-morphology deferral (v0.72's
  "oxbow cut-off geometry, needs centerline curvature tracking"). Engine (block 1), opt-in (Burn-rivers
  toggle), never in `generate()`/default render ⇒ **render battery ALL IDENTICAL to v0.78**, headless
  **903 → 909**. `featureDetailPass` gains an oxbow term revealed only at **z≥9** (`zo` ramps across z9..z10;
  z≤8 byte-identical): a rare ridged-noise blob field gated to the floodplain band beside order≥3 channels,
  carved to a shallow water pocket. Seam Δ=0 (pure world-coord noise + shared LUT), carve-only, floor-bounded.
  True cut-off geometry (vector centerline tracking) still out of scope per LOD tile; this is the seam-safe
  LOD approximation. **Browser pass owed**: the pockets at z9–z10 on a real floodplain; noise-sample perf.
- **Settlement-density research deferrals — COMPLETE** (`docs/research/settlement-density.md`).
  v0.75 (metropolis §5) · v0.76 (village density §6 + regional-pop estimate §3) · v0.77 (wetlands
  carrying-capacity §2b) · v0.78 (transport transfer-overhead §5c) all shipped, each opt-in / default
  bit-identical. The last item — the **Mediterranean-scrub residual calibration** (§9 Q5) — is now resolved
  by a sourced follow-up: the Roman-Italy anchor (Beloch ~15–20/km²; Frier Latin West ~17/km²; Hin 2013)
  **confirms** `shrub`'s existing residual 0.95 + intensify 0.40 (dense but rain-fed Mediterranean dry-farming),
  so no code change was warranted — the doc's §9/§2a/References were updated with the citation instead of
  changing a well-calibrated number. All five §9 open questions are closed.
- **v0.78 — transport transfer/handling overhead** (settlement-density §5c, the "pathfinding for routes"
  strand). Civ layer (block 2) only, **engine bit-identical to v0.77** (headless **903**, render battery ALL
  IDENTICAL — journeys are transient, never in the battery), smoke **81 → 83**. Wiseman et al. 2024:
  transshipments (land↔water mode-changes) add ~5% cost each, compounding, independent of distance. New pure
  `_civTransshipments()` / `_civTransferOverhead()`; `_jpPlan` carries `transshipments` / `transferOverhead`
  / `handlingDays` (additive — travel `days` unchanged), and the journey inspector shows a **Transfers** row
  when the route changes mode (browser: a 95%-water port route → 1 transshipment, +5%, +0.5 d). **Browser
  pass owed**: the Transfers row on multi-leg land↔sea itineraries; whether 0.5 d/transfer feels right.
- **v0.77 — wetlands/marshes carrying capacity** (settlement-density §2b) — first density track to touch
  the **engine** (block 1, headless-testable). `buildBiomeRaster` (fed to K) had no wetlands class; Wetlands
  lived only in `buildCartBiome`'s `CART_BIOMES`. New pure `buildWetlandMask()` uses the exact same
  moisture+flatness+low condition, so the two pipelines finally agree (smoke asserts cell-for-cell match).
  `buildCarryingCapacity(opts.wetMask)` overrides a wetland cell's residual with `WETLAND_DENSITY_RESIDUAL=0.70`
  (productive but malaria/flood friction); `estimateRegionalDensityKm2(wetMask)` uses
  `WETLAND_INTENSIFY_ELIGIBLE=0.95` (managed-wetland/raised-field intensification). Rides the **Biome
  carrying-capacity** toggle (`_biomeK`, default off) ⇒ **default field + render bit-identical to v0.76**;
  headless **897 → 903**, smoke **79 → 81**. `_wetlandMask` invalidated in lockstep with `_carryCapField`.
- **v0.76 — dense village-grid placement mode + regional-population estimate** (settlement-density §6/§3).
  Civ layer (block 2) only, **engine bit-identical to v0.75** (headless **897**, render battery ALL
  IDENTICAL), smoke **75 → 79**. (1) **Dense village grid** (`_civVillageDensity`, checkbox, default off):
  wires the v0.69 `suppressionRadiusCells(VILLAGE_SPACING_KM,…)` helper into `_civIterativeAutoWorld` (when
  tier counts are blank) — seeds at the ~10 km site-catchment spacing instead of ~market-town, ~3–4× denser,
  capped at `_CIV_VILLAGE_CAP=200` pins (browser: 40 → 200). (2) **Regional-population estimate**
  (`_civRegionalPopulation()` + button): integrates the persons/km² field over land (+ per-faction over
  painted territory) for real totals without a pin per hamlet (browser: ~254k over ~190k km², ~1.33/km²).
  Both opt-in/read-only ⇒ auto-populate byte-identical when off. **Browser pass owed**: does the 200-pin cap
  feel right, and are the estimate's absolute numbers sensible across biome-K on/off?
- **v0.75 — imperial-seat (metropolis) tier** (settlement-density §5). Civ layer (block 2) only,
  **engine bit-identical to v0.74** (headless **897**, render battery ALL IDENTICAL), smoke **72 → 75**.
  Adds a rare **Metropolis ★** class above Capital, placed by the sourced ceiling-breaking rule (Lawrence
  et al. 2016: post-2000 BC growth = administrative/taxation capacity, proxied by betweenness centrality ×
  polity size). New pure `_civSelectMetropolises()` promotes a capital with normalised betweenness ≥ 0.85
  that is also the seat of a ≥6-settlement faction; ≤1/faction, ≤3 total; base pop 45,000, scaled by the
  existing centrality multipliers (browser probe: ~133k on a dominant hub). Gated behind the
  "Imperial-seat tier" checkbox (`_civMetropolis`, default off ⇒ auto-populate byte-identical; skipped when
  tier counts are fixed). Frozen pack-slot vocabs untouched (procedural ★ fallback). **Browser pass owed**:
  metropolis feel across seed variety — is one imperial seat per large polity the right rate, and is base
  pop 45,000 / the betweenness threshold 0.85 tuned to taste?
- **v0.74 — "Bake all levels & finalize world" promoted to the top of Generate → World** (owner
  request). The finalize button was buried two collapsed disclosures deep (*Tiles & LOD → Atlas*),
  so committing a world to the Atlas phase meant hunting for it. A new **Finalize world** section
  (`#finalizeSec`) is now the first block of Generate → World (above Geology), hosting the bake-depth
  picker + **🔒 Bake ALL levels & finalize world** / **🔓 Un-finalize** buttons. Pure DOM-position
  relocation: the moved elements keep their v0.62 ids (`bakeAllDepthRow`/`bakeAllDepth`/`bakeAllBtn`/
  `unfinalizeBtn`) so `applyFinalizedUI()` and every handler are unchanged; per-view bake / clear /
  export stay under *Tiles & LOD → Atlas*. Banner/chip/alert text re-pointed to "the top of Generate →
  World". **Engine bit-identical to v0.73** (render battery ALL IDENTICAL; headless **897** unchanged),
  smoke **71 → 72** (+1 asserting the bake button is the first `<button>` in `#genWorld`, in
  `#finalizeSec`, not behind a `<details>`). Verified in-browser (screenshot).
- **v0.73 — economic land/sea routing + settlement-waypoint pathfinding** (owner report: routes
  ignored a cheaper/more-direct sea leg and bypassed settlements they passed instead of stopping).
  Civ layer (block 2) only, **engine bit-identical to v0.72** (headless **897** unchanged), smoke
  **68 → 71**. Owner chose *both* systems + *soft-attract, capped detour*. (1) **Settlement gravity**
  (`_civApplySettlementGravity`) — a capped, radius-limited (~RW/80) cost discount around every
  settlement, applied to the Route-tool grid (`_civDijkstraPath`) and both auto-network passes
  (`_civHierarchicalNetwork`); a least-cost path now bends *through* settlements near its corridor
  (they become stops) but never detours far, and — only finite cells discounted — never carves water.
  (2) **Economic sea** — mixed-grid water cost 2.2 → **1.5** (`_CIV_WATER_COST`) so a >~1.5× land
  detour loses to the sea leg; a mostly-water committed route auto-flags a sea voyage
  (`_civPathWaterFrac`≥0.5) so the planner picks a vessel. (3) **Sea-net augmentation** — each port
  also gets a direct lane to its nearest sea-reachable port (not just the MST spine). (4) **Stops**
  row in the journey planner (`_civPassedSettlements`, derived/transient, not serialised).
  Verified in-browser (routing probe + before/after screenshot); smoke uses a deterministic
  settlement-*injection* gravity test. **Browser pass owed**: feel of the auto-network on varied
  worlds (are the coastal roads vs sea lanes sensible?), and tuning of `_CIV_WATER_COST`/gravity
  strength to taste.
- **v0.72 — deep-zoom river morphology (tributaries + local incision).** Finishes the river-lod
  brief's LOD10+ tier by extending `featureDetailPass` (z≥8, behind the Burn-rivers toggle): the trunk
  thalweg locally incises deeper with zoom, and a **dendritic tributary creek network** (ridged
  value-noise, catchment-gated to a trunk channel's valley influence `Rt=2.5+order`, land-only) reveals
  itself. The noise is a pure function of world coords + the coarse Strahler LUT, so **seam Δ=0** (even
  with the z≥7 meander wobble on); carve-only under the sea−0.06 floor (deep ocean never raised).
  Strictly gated above z=7 (`zt=clamp((z−7)/3)`) — z≤7 output is byte-identical to v0.71 even with the
  depths forced high. Engine bit-identical to v0.71 (opt-in; never in `generate()`), headless **890 →
  897** (+7), smoke **67 → 68**. Deferred: oxbow cut-off geometry (needs true centerline curvature
  tracking) and the Rust/WASM port (JS-first per owner). **Browser pass owed**: the tributary network
  and incision at z8–z10 on a real world; perf of the ridged-noise pass on 1024² tiles at deep zoom.
- **v0.71 — zoom-dependent feature rendering** (owner goal + the river-lod / rust-lod render briefs),
  three stages in one version, engine bit-identical to v0.70, headless **890** (+26), smoke **67**:
  (1) **persistent feature registry** — rivers as objects (Strahler polylines, discharge, hydrology
  width, length), fjord/canyon components, peaks; `featuresNear`/`riversInRect`/`featureSummary`
  query API; `features.json` export (features survive baking); cached as `_featureReg`, invalidated
  with `_riverNet`. (2) **LOD render caches** — per-tile canvas LRU keyed on `_lodRenderKey` +
  pan-reuse of the coarse overview; `_lodEditGen` guards edits; pixels identical, computed once.
  (3) **`featureDetailPass`** — zoom-revealed morphology on refined tiles behind the Burn-rivers
  toggle: river valley cross-sections ∝ Strahler order (z≥4), fjord wall steepening (z≥3), canyon
  incision (z≥4), meander wobble (z≥7, deterministic world-coord wave); seam-safe, opt-in (no
  grids ⇒ byte-identical), floor never raises terrain. Tributaries + local incision landed in v0.72;
  still deferred (briefs): oxbows, Rust port (JS-first per owner).
  **Browser pass owed**: LOD pan/zoom feel with the caches, the revealed valleys/fjords/canyons at
  deep zoom on a real world, cache memory pressure on 8K worlds.
- **v0.70 — bug-fix batch + map-scale locked at creation.** Four owner-reported bugs, each reproduced
  in a real browser before fixing (see `tests/perf/` probes), engine bit-identical to v0.69, headless
  **864**, smoke **61 → 65**: (1) **`roadDijkstra` crash on imported heightmaps** — `dist` was Float32 but
  priorities Float64, so the uniform imported cost grid re-pushed cells until the heap overflowed 2³²; fixed
  with a `visited` source-finalization array (output-identical, auto-populate 127 s→4 s). (2) **imported
  worlds had no rivers** — `inferTectonics` never ran `computeFlow`; now does climate→flow so `flowField`
  populates. (3) **~900 plates on import** — `pickPlateSeeds` capped at 40. (4) **sea level didn't move the
  coastline** — `_civBakeKey` omitted `state.seaLevel`, so the cached bitmap was reused; added it. Plus **map
  width locked** in the sidebar (`#mapw` disabled, exempt from the finalize re-enable) — set at creation in
  the gate. **Next track: zoom/scale-dependent feature rendering** (fjords/rivers/canyons/mountains by zoom)
  — the owner's larger ask, overlapping the river-LOD + LOD-perf roadmap; research/plan queued (task).
- **v0.69 — settlement density (sourced).** First of the three research-doc tracks the owner
  supplied (`docs/research/settlement-density.md`; river-lod + rust-wasm briefs also committed for
  later tracks, JS-first, Rust deferred). Pure/CPU-path additions, **engine bit-identical to v0.68**
  (biome term defaults off; density field additive, never in `generate()`); headless **864** (+12
  calibration), smoke **61**. Added: `foragerFloorKm2` (NPP→forager density), biome-residual
  `buildCarryingCapacity` behind `opts.biomeK` (default 0 = byte-identical; opt-in checkbox flips
  `_biomeK`), `estimateRegionalDensityKm2` (persons/km², water-gated agrarian ceiling) surfaced as
  the **"Pop density"** debug view + `population_density.f32` export, and `suppressionRadiusCells`
  spacing helper (not yet wired into placement). Deferred v0.70 candidates: metropolis tier,
  village-density placement mode, Wetlands carrying-capacity, Mediterranean-scrub calibration.
  Roadmap after density: JS LOD/renderer perf refactor → river-as-feature LOD → (later) Rust/WASM.
- **v0.68 — fix: sidebar was live during the v0.67 setup gate.** The gate modal lives inside
  `.canvas-wrap` so it only covered the canvas; the sidebar (a sibling `aside`) stayed clickable,
  and its Generate→World sliders (sea/climate/weather) acted on the empty pre-commit field — the
  "sea level/climate/weather seem broken" report. The committed sim was verified fine. Fix:
  `body.setup-gated` (toggled in `_setupOpen`/`_setupHide`) dims + `pointer-events:none` the
  sidebar until a world is committed/loaded. Engine bit-identical; headless **852**; smoke **59**.
- **v0.67 — setup gate + scale/height calibration.** The app no longer auto-generates on load;
  a **hard setup gate** blocks the canvas until the user commits base settings (the old
  once-per-browser `cartalith_onboarded` flag — why the card "didn't load on opening" for
  returning users — is retired). Boot: browser allocates + renders empty + opens the gate;
  **headless (no indexedDB) keeps the old auto-generate path verbatim**, so 852 + bit-identity
  are byte-unchanged. Gate (`_setupOpen`): intro (Generate/Load/Import, no Skip) → generate form
  (resolution, extent, center, scale & calibration with **km/mi** toggle + distance legend, peak)
  → **Commit** runs `generate()` once; heightmap Import → calibrate form → **Commit** auto-runs
  `inferTectonics()`. **Peak auto-suggest** `suggestPeakM(w)=round(8849·(1−e^(−w/1330)))` (800→4000
  preserved, caps ~Everest). **Scale-aware 3D** `_v3dEffExag()` normalizes the drape exaggeration
  by the true relief:width ratio (default look bit-identical; whole-world auto-flattens). Units are
  a localStorage pref (km + m canonical). Engine bit-identical to v0.66; headless **852**; smoke
  **50 → 57**. Also fixed the header chip that read v0.65 in v0.66. Browser pass owed: 3D feel
  across scales, live units toggle, import→infer with a real DEM.
- **v0.66 — IA CORRECTION (owner-directed): the Generate branch menu is restored.** v0.64 had
  retired the Generate sub-tab bar and moved Civilization + Cartography into Explore, following
  the research proposal's §3 — but contradicting the owner's intended IA. The shipped structure
  is now: **Generate** (authoring) = sub-tabs **World | Civilization | Cartography**, with the
  pinned Selection inspector under the sub-tab bar shared by Civ+Carto, and the tool palette
  split per branch (Civ: Inspect·Settlement·POI·Territory·Way; Carto: Inspect·Label·Icon) —
  all buttons drive the one `_civSetTool` machine; **Explore** (planning) = Info·Route tools,
  Journeys, Journey planner, canvas filter funnel + timeline. Entering Explore auto-arms Info.
  Paint re-gated to Generate→Cartography. Bundled fixes: Un-finalize button no longer disabled
  by the finalize lock (bug since v0.62); active sub-tab label no longer amber-on-amber; stale
  "Edit →" path strings updated. Engine bit-identical to v0.65 (checksums unbroken to v0.62);
  headless **852 green**; smoke suite rewritten for the corrected IA, **41 → 50** green.
  **`docs/research/ui-ux-upgrade.md` §Status carries a correction note superseding §3's
  re-homing — do NOT re-apply "Civilization and Cartography live in Explore".**
- **v0.65 — UI/UX overhaul, the remaining scope cuts closed out.** Engine bit-identical to v0.64
  (checksums byte-equal all the way back to v0.62; headless **852 green** throughout);
  `tests/perf/smoke_gen1.js` grew **27 → 41** Playwright assertions. `docs/research/ui-ux-upgrade.md`
  §Status now shows every stage genuinely complete, not just scoped-down. (1) **Full pinned
  inspector**: the settlement/POI/label edit forms (name/kind/pop/history/…) now render IN the
  pinned inspector itself, not inline in the lists — `_civRenderSettlementList`/`_civRenderPoiList`/
  `_civRenderLabelList` only render rows + selection highlight now; `_civSelectedRowRefs` preserves
  the old inline version's live-row-patching optimization (no full list rebuild per keystroke) by
  handing the currently-selected row's DOM refs to whichever editor the inspector renders. Extended
  to a third group, the Placed-Icons list's own per-instance editor, so selection is single across
  all three (place/label/icon instance) — picking one clears the others. Caught and fixed a stale
  bug along the way: the label list's delete handler only refreshed the label list itself, leaving
  a deleted label's editor stuck on screen. (2) **Per-layer hotkeys** (§4.10): bare-key shortcuts
  (B/T/F/S/W/R/0) for the Layers popover's most-reached-for views, badge shown in the popover,
  guarded against firing while typing in any input. (3) **Assets/Export promoted to header
  utilities**: the tab bar is now a genuine two-position Forge/Atlas phase switch (just Generate +
  Explore) — Export became a header dropdown (`#exportWrap`, mirrors Import ▾ but stays open across
  internal clicks since it's a form, not a one-shot action list) and Assets became a plain header
  button (`_carEnterAssetsMode`) that enters the same full-viewport Asset Library takeover as
  before; exiting is automatic (clicking Generate/Explore always restores the canvas — no `_activeTab`
  changes were needed since Assets/Export never touched that variable's only two remaining
  consumers). Browser pass owed: the relocated inspector's feel end-to-end, the hotkeys in daily
  use, and the header Export/Assets controls.
- **v0.64 — UI/UX overhaul completed** (the stages v0.63 deferred). Engine bit-identical to
  v0.63 (checksums byte-equal all the way back to v0.62; headless **852 green** throughout);
  `tests/perf/smoke_gen1.js` grew **12 → 27** Playwright assertions. Highlights: **Edit tab +
  Generate sub-tab bar retired** (Generate is World-only; Tiles & LOD moved into Generate →
  World; Undo moved to the header; Civilization + Cartography moved wholesale into Explore);
  **"Places & roads (terrain)" retired outright** (engine functions kept, UI gone — it shared
  `state.places` with civ settlements, so its "Clear places" could silently wipe them, a real
  landmine now closed); a unified 9-button tool palette replaces every scattered `data-civtool`
  control, with Label + Icon newly folded into `_civTool`; a lightweight pinned selection
  inspector (later made "full" in v0.65); danger accents + confirm-when-non-empty on 3 destructive
  Clear buttons that had none before.
- **v0.62 — civ-layer UX batch + finalize milestone (user request).** Engine bit-identical to
  v0.61 at defaults (checksums byte-equal; 848/848 green). (1) Economy+Politics merged into one
  **Polity** section + an **∅ Unclaimed** faction pill (paint to erase territory). (2) Timeline
  slider fixed (phantom "0 AD" era on first Add-year killed; mid-drag rebuild no longer resets
  the thumb via `_civTlDragSrc`) and **twinned** — `#civTlSlider` in Polity + the Explore slider
  share `_civWireYearSlider`. (3) Places gain a persistent **History** field; POIs get their own
  collapsible list (`#civPoiList`, expand-in-place editor like settlements); **right-click
  context menu** on the viewport (edit/move/delete nearest, drop settlement/POI, info) with
  `e.button` guards so the right button never sculpts/drops. (4) **Bake ALL levels & finalize**:
  `bakeAllTiles(depth)` bakes the whole LOD pyramid (select 2–5) into the atlas, then
  `state.finalized` locks Generate → World (3D dials exempt), banners the panel, and guards
  `generate()`/`confirmRegenerate()`/`_manualTerrainActive()` — the project becomes a
  cartographic LOD viewer/editor; un-finalize reverses. Headless-proven: finalized `generate()`
  is a byte-exact no-op. Also: `docs/research/ui-ux-upgrade.md` (researched UI/UX proposal,
  phase-based IA / layers popover / disclosure / inspector patterns, staged rollout).
  **Browser pass owed**: Polity flow, slider drag feel in both places, POI list + History
  editor, context menu, full bake → finalize → viewer flow.
- **v0.61 — sync-`generate()` contract restored (repo review fix).** v0.6's
  `async buildTectonicSubstrate()` refactor made `generate()` await unconditionally, breaking
  the v0.135 invariant that `generate()` completes synchronously when no worker pool is engaged.
  Headless fallout: flat `rainField` for unawaited callers → 32 suite failures + a crash that
  aborted ~200 assertions. Fixed by making `buildTectonicSubstrate` return `false` synchronously
  on the no-pool path (Promise only on the pool path); `generate()` awaits only a Promise.
  Proven: suite **848/848 green**; FIELD/TEMP/RAIN/FLOW FNV checksums bit-identical to
  v0.6-awaited at seed 12345/256px. Now **Invariant 12** in `CLAUDE.md`.
- Same batch (repo hygiene): settlement-seed test no longer hard-crashes the suite on an empty
  seed list; `tests/run.sh` defaults to the newest Gen1 file (exec bit restored); dead merge
  tooling swept into `legacy/` (see `legacy/README.md`); `CLAUDE.md` slimmed to architecture +
  invariants with the 108-entry version log moved to `CHANGELOG.md`; real `README.md`.
- **Browser passes owed** (headless can't see these — accumulate from recent versions):
  the v0.6 3D drape view (orbit/pinch camera, drape re-upload), the `renderNow(rect)` brush
  fast path feel, worker progress/parity for the erosion ops, GPU R32F path, LOD/atlas
  interaction, plus the visual passes listed in recent `CHANGELOG.md` entries.

## How to verify (the discipline we hold)

1. `tests/run.sh` must pass — the full assertion suite (984 as of v1.15), CPU paths of the engine block. Extend
   `tests/test_tail.js` when adding a stage; stubs in `tests/stub_head.js`. Script block 4 (urban
   morphology, v0.95+) is pure/DOM-free like block 1 and gets its own harness, `tests/run_um.sh`
   (852 assertions as of v1.17, ported from `urban-morphology/tests/`) — but the civ-layer
   adapter/renderer that calls it is block 2, so that half still needs `tests/perf/smoke_gen1.js`.
2. **Cross-version neutrality**: any additive/opt-in change must be proven byte-identical to the
   prior version at defaults — FNV checksums of field/temp/rain (and render where applicable) at
   seed 12345, 256px, region mode. `tests/perf/hash_gen1.js` is the Playwright A/B battery for
   render-path changes.
3. GPU shaders, Web Worker glue, and canvas interaction (zoom/pan/paint/3D) **cannot be verified
   headlessly** — implement, then flag explicitly for a manual browser pass.
4. Commit messages end with the session URL line (see existing commits). Push to the work
   branch; create a draft PR; ask the user if they want it watched.

## Key invariants (full list in CLAUDE.md)

- Don't renumber frozen vocabularies (`BIOME_KEYS`, `KOPPEN_KEYS`, `BTYPE_KEYS`, `LITH_KEYS`,
  `CART_BIOMES`/`CART_TERRAINS`) — save-format stability.
- Worker kernels stay self-contained (rebuilt from `toString()` in the suite — Invariant 11).
- `generate()` completes synchronously when no pool is engaged (Invariant 12 — the v0.6 lesson).
- Nullable fields (`geoidField`, `tideField`, `continentalField`, `orogenyField`, `warpX/Y`,
  `riverMask`) — every consumer null-checks.
- Keep CPU and GPU lapse (`uLapse`) in lockstep.
- World-seam invariant (avg wrap delta < 0.12) is seed-dependent — don't tighten it.

## Next / open

- **OPEN, considered not done: `tests/perf/smoke_gen1.js`'s full run (700+ assertions) crashes the
  headless Chromium page near its own end in this session's execution environment.** Found while
  verifying v1.100. Reproduced identically against a completely unmodified `Cartalith Gen1
  v1.99.html`, substituting a no-op stub for v1.100's own new test block — the crash fires at the
  exact same position regardless of what runs there, confirming it predates v1.100 and is not
  triggered by any specific test's logic. Most likely cumulative headless-Chromium renderer memory
  pressure from ~700 assertions' worth of `generate()` calls/canvases/WebGL contexts accumulating in
  one long-lived page across the whole suite (this environment renders WebGL via SwiftShader —
  software rasterization, already flagged elsewhere as heavier than real hardware). NOT fixed this
  pass — no `generate()`/`renderNow()` code was touched, and diagnosing a headless-browser resource
  ceiling is a different kind of work than the routing audit this session was asked to do. If picked
  up: try periodically closing/reopening the page (or splitting the suite into several page sessions)
  inside `smoke_gen1.js` itself; confirm first whether the crash point is stable (always the same
  assertion offset) or drifts with unrelated changes, which would point to a memory-size threshold
  rather than a specific leak.
- **OPEN, considered not done: the `VERSION` JS const (`Cartalith Gen1 v*.html`, near the top of
  script block 1) has now drifted stale THREE times** (v1.30, v1.52, v1.100) despite two prior
  fix-comments warning about exactly this. It's export/atlas metadata + the on-screen `#verTag`
  chip only — never gated on, confirmed harmless to every hashed/tested field — but still a real,
  repeatedly-recurring defect. A real fix needs one source of truth: either derive `VERSION` from
  the `<title>` tag at load time (or vice versa), or add a build-time/commit-time check outside the
  file itself, rather than relying on a human to remember a third hardcoded literal on every version
  bump. Not attempted this pass — a version-bump-hygiene change is a different scope than a version
  bump itself.
- **RESOLVED in v1.96** (was: "`buildResourcePotentials` costs ~500ms at 2048px inside a PLAIN
  `generate()`"). The v1.92 guess — block 2's `renderNow` wrapper lazily triggering it — was wrong.
  The real path is `generate()`'s civ-layer **wrapper**, not its render: it calls
  `_civRenderPlaceEditor()` → `_civRefreshActiveSubPage()` → (Factions, the v1.55 default sub-tab)
  → `_civFactionAggregates()`, which unconditionally builds `currentResourcePotentials()`,
  `currentPopulationDensity()`, `buildBiomeRaster()` and `_civOceanDistField()` plus a full
  `GW·GH × CIV_RESOURCE_KEYS` accumulation — **before** `_origGenerate` runs, against the world
  about to be destroyed, into `#genCiv` which is `display:none` until the user opens the
  Civilization sub-tab, with every cache it filled nulled moments later by `generate()`'s own
  invalidation. Measured 686 ms at 1024px with zero settlements and zero territory. Fixed with a
  `_civSubPageVisible()` gate on the expensive render half (state resets stay unconditional — see
  CHANGELOG for the regression that taught us the difference) plus a refresh-on-reveal in
  `#genSubBar`'s handler. See CLAUDE.md's "A full-grid faction-aggregate pass ran on every
  generate(), into a hidden panel (v1.96)".
- **OPEN, considered not done: the Library→runtime bridge still has no `dropTextures`/`dropBiomes`/
  `dropTerrains`/`dropStructures` tracking.** Found during the v1.91 asset-pack-persistence pass.
  v1.27 added `dropIcons`/`dropCustom` ownership tracking so deleting a Library-owned icon/custom
  asset correctly retracts it from `assetPack` on the next `syncToRuntime()` — but when v1.28 wired
  in `biomes`/`terrains`/structures (and v1.91 added `textures`), none of those four families got
  the equivalent tracking. Deleting a Library-owned biome/terrain/structure/splat-texture asset
  still leaves its stale pixels sitting in `assetPack` rather than retracting them — a real gap, but
  narrower and lower-risk than the v1.91 data-loss bug it was found alongside (nothing goes MISSING,
  a deleted asset just lingers visually until the next full reload). If picked up: extend the same
  `_pushedIcons`/`_pushedCustom` Set-diff pattern `syncToRuntime()` already uses to four more owned-
  key sets, and extend `applyLibraryAssets()`'s drop handling to match.
- **OPEN, considered not done: GPU `readPixels` synchronous readback cost in `gaussBlur`/
  `normalize()`/`computeTemperature()`.** Found during the v1.89 simulation-speed pass — a CPU
  profile of a full `generate()` call showed `gl.readPixels` (the GPU→CPU sync readback inside the
  `GPU._down` helper, used by `GPU.blurArr`/`GPU.norm`/`GPU.temperature`) as one of the single
  largest line items, comparable to `streamPowerKernel` itself. NOT investigated further or
  touched, for two reasons: (1) this session's headless-Chromium test environment renders WebGL2
  via SwiftShader (software rasterization — confirmed via `perf_gen1.js`'s own `env.gpu` probe),
  so "GPU path" timing measured here reflects CPU-emulated-GPU driver overhead, not real graphics-
  hardware behavior — CLAUDE.md's own `perf_gen1.js` header already carries this exact caveat for
  a reason. Any conclusion drawn from these numbers about whether the GPU path genuinely helps or
  hurts on a real device would be unfounded. (2) Fixing this properly would mean either disabling
  the GPU path when the CPU path is provably faster (a runtime GPU-vs-CPU benchmark — a materially
  bigger feature than a targeted fix) or restructuring `gaussBlur`'s dispatch logic, both of which
  need a REAL GPU to validate against, which this session cannot access. Worth a dedicated pass
  with actual browser/device access — start by re-profiling `generate()` on a real GPU to see
  whether `readPixels` is still disproportionate there before deciding anything.
- **OPEN, considered not done: `roadDijkstra`'s heap already tried and reverted (v1.89) — do not
  re-attempt the same typed-array technique blind.** The identical `MinHeap` fix that helped
  `buildWaterBodies`/`streamPowerKernel`/`glacialKernel` measured WORSE here (up to -41% on a real
  36-settlement "Generate Roads" run) because `roadDijkstra` allocates a fresh heap PER SETTLEMENT
  rather than once per call — see the v1.89 CHANGELOG entry for the full numbers. If this is ever
  revisited, the right shape of fix is different: e.g. a heap REUSED across the `for(let s=0;s<P;
  s++)` loop in `buildRoadNetwork` (reset between calls instead of reallocated), which would keep
  the typed-storage win without paying its allocation cost 36+ times — not attempted this pass, a
  materially different (and untested) change from what was tried and reverted.
- **OPEN, considered not done: a faster priority-queue for `buildWaterBodies`'s depression fill.**
  Found during the v1.87 rendering-speed pass — a CPU profile showed `MinHeap.pop`'s own O(log n)
  sift-down cost (~482ms of ~1050ms at 2048px) essentially unchanged by the typed-array fix (that
  fix cut `push`/`nb`/`visit`, not `pop`'s comparison count). A d-ary heap or a bucket/radix
  priority queue could plausibly cut the `log n` factor further, but risks popping equal-priority
  cells (common — the algorithm's own `filled[j]=filled[i]+EPS` tie-break cascades through flat
  depressions) in a DIFFERENT order than the current binary heap, which would change which cells
  inherit which fill height and thus the final lake classification — a real bit-identity risk, not
  attempted. If revisited, the verification bar is high: must reproduce the EXACT same `_waterBody`/
  `_lakeFill` output (hash comparison, not just "looks similar") across several seeds/resolutions
  with flat/depression-heavy terrain, not just the one reference seed.
- **OPEN, considered not done: the per-pixel colour loop (`surfaceColor`/`materialWeights`/
  `landColorCore`) is the single largest render cost and was left untouched in v1.87.** A CPU
  profile found no redundant recomputation — every expensive call is already made exactly once per
  pixel. Two paths were considered and rejected for this pass: (1) a lookup-table approximation of
  the repeated `Math.pow`/`Math.exp` calls — rejected because any interpolation scheme introduces
  quantization error, directly conflicting with the owner's explicit "keep fidelity and detail"
  constraint on this task; (2) restructuring `renderNow`'s per-pixel debug-view dispatch (the ~20
  `dbg==='...'` string-literal checks before reaching the biome-mode branch) — likely low-value
  since V8 typically compiles literal-string comparisons to pointer equality, and a much larger,
  higher-regression-risk change than the rest of this pass. Worth a dedicated look only if a future
  profile shows the dispatch chain itself (not the colour math it guards) as a measurable cost.
- **OPEN, found not fixed: Active foraging's speed cost can outweigh its consumption benefit on a
  single, already-marginal carry stretch.** Found while verifying v1.81 (below) —
  `probe_jp_forage_effect2.js` measured several scenarios where switching Foraging from None to
  Active INCREASED the load-ratio percentage rather than decreasing it, because
  `JP_FORAGING['Active'].speedMod` slows travel, which on a stage whose carry interval is capped by
  the stage's own duration (not a shorter resupply stop) directly increases the one-time mass that
  must be carried — sometimes by more than the foraging discount saves. Confirmed pre-existing (not
  introduced by v1.81) by re-running the identical scenario against unmodified v1.80 and finding
  the same pattern at equal-or-worse magnitude. v1.81's new water-foraging term measurably improves
  the outcome in every comparable case but doesn't resolve the underlying tension. Not fixed —
  retuning `JP_FORAGING` speed multipliers or the day-count/interval-capping logic is a materially
  larger, riskier change than v1.81's actual ask (extend Foraging to cover water; wire in real
  wildlife data), confirmed via two `AskUserQuestion` answers that didn't cover this. Pick this up
  as its own scoped pass — start from `probe_jp_forage_effect2.js` in the scratchpad convention,
  re-derive it fresh since scratchpad files aren't committed.
- **OPEN, found not fixed: two environmental smoke-suite failures, unrelated to any recent
  change.** `v0.92 follow-up fix: overview canvas is capped at 512px wide...` and `v0.87: LOD/atlas
  mode fills the viewport...` both fail on this headless-Chromium harness as of v1.80/v1.81/v1.82,
  confirmed identical across all three (so not a regression) — likely viewport/canvas-sizing
  flakiness in this environment rather than an app bug, but not root-caused. Worth a dedicated look
  if it starts flagging more assertions or if a real canvas-sizing bug is ever reported matching
  either description.
- **OPEN, considered not done: `currentWindField()`/`currentOceanField()` have no caching at all.**
  Found during the v1.86 bug hunt as the only `current*Field()`-style accessors in the file with zero
  caching — but they recompute their `tSea` proxy directly from live `state.climate`/`state.planet`
  (via `climEffectiveEquatorTemp()`), which is why they currently update instantly while dragging the
  axial-tilt/rotation sliders. A `_fieldGen`/`_climGen`-keyed cache (the obvious fix, by analogy with
  every sibling) would REINTRODUCE staleness there — the exact bug class v1.86 just fixed elsewhere —
  since it would only refresh those two views once a full climate recompute runs, not on every slider
  drag. Any future attempt needs a key built from the RAW slider values themselves (equatorTemp,
  poleTemp, lapseRate, latN, latS, windMode, pressK, axialTiltDeg, rotationHours, g, radiusRel,
  currentK for the ocean variant) rather than the two generation counters — more fragile, more
  values to keep in sync, which is why v1.86 left it uncached rather than rushing a plausible-looking
  fix. Worth a dedicated pass if these views are ever profiled as a real bottleneck during panning
  with a debug view open (not measured this session — the concern is theoretical/structural, not from
  observed slowness).
- **OPEN, found not fixed: `computeFlow(true)` is not idempotent across two calls even when `field`
  ends up bit-identical before each one.** Discovered chasing a genuinely confusing smoke-suite
  failure while verifying v1.86's own new tests — the pre-existing v1.78 "`refreshClimate()` is
  deterministic" check started failing only when the new `R.v186` terrain-edit test ran immediately
  before it in the sequential suite. Root-caused with a direct Playwright probe: carve a depression
  into `field[]`, run `computeFlow(true)`, restore `field[]` to its exact prior bytes (`fieldExact:
  true`, confirmed), run `computeFlow(true)` again — `flowField` differs by a mean of ~0.86/cell from
  a version that was never carved at all. Confirmed to reproduce IDENTICALLY on unmodified v1.85, so
  this predates v1.86 entirely and is untouched by any of this session's fixes — a real, pre-existing
  property of `computeFlow`'s own seeding/accumulation (possibly a scratch-buffer-pool reuse issue —
  see the "scratch buffer pool" module comment's own "caller must not reuse the same slot for two
  live arrays" warning, though this wasn't traced all the way to a specific line). **Not fixed** —
  out of scope for a test-harness fix, and worth its own dedicated root-cause pass rather than a
  rushed drive-by; `R.v186`'s own terrain-edit sub-test was rewritten to restore `field`/`flowField`
  directly via `.set()` instead of depending on `computeFlow()` being re-run-idempotent, sidestepping
  the need for it rather than fixing it. **Worth investigating**: this implies a real user doing a
  sculpt edit and then undoing it (if Undo restores `field[]` and re-runs `computeFlow()`, which is
  the natural implementation) could leave `flowField` — and everything downstream: rivers, settlement
  suitability, resources — subtly different from a world that was never touched, not just stale (the
  v1.86 debug-view finding) but actually WRONG. Start by checking whether `computeFlow`'s scratch
  buffers (`mbuf`/`ibuf`/`ubuf` slots) are fully overwritten before use on every call or only
  conditionally topped up.
- **v1.86 shipped**: bug hunt + optimization pass. `computeTemperature()`/`simulateWeather()` now
  invalidate the same 11-field derived-cache family `computeFlow()`/`generate()` already do (biome
  raster, soil, lithology, landform, resources, carrying capacity, settlement suitability, wildlife,
  NPP, population density, wetlands) — previously only a full regenerate or the sea-level slider's own
  patched handler cleared it, so a pure climate re-simulation (slider drag, or the "Simulate weather"
  button) silently left settlement suitability/placement reading STALE biome and flood data.
  `currentFloodField`/`currentWindThrowField` switched from a `state.tect.seed` cache key to
  `_fieldGen` (the file's own established convention), fixing the same defect class for same-seed
  regenerates and sculpt/erosion edits. Bundled: `buildWindThrowField` now reuses the cached
  `buildBiomeRaster()` instead of reclassifying per cell. Every fix verified by DIRECT reproduction
  (a Playwright probe showing the exact before/after numbers on a real seed), not just static
  reasoning. Hash vs v1.85 ALL IDENTICAL. See CHANGELOG for the full writeup and the
  considered-but-not-done wind/ocean-caching item (above).
- **v1.85 shipped**: ocean heating grounded in axial tilt + rotation via new `climEffectiveEquatorTemp()`
  (North & Coakley 1979 obliquity term + rotation-rate term, both normalized to 1.0 at Earth
  defaults), redirecting the six duplicate `tSea` formulas + the GPU shader to one source of truth;
  also confirmed (measurement, no code) that the terrain-coupled wind/current pipeline already
  reaches map rendering via `simulateWeather()`'s internal `oceanSSTAnomaly()` call. Bit-identical
  at defaults (`hash_gen1.js` v1.84→v1.85 ALL IDENTICAL); measured live divergence at non-default
  tilt/rotation. Gravity deliberately left without a new term (disclosed scope cut — see
  `docs/research/solar-energy-budget.md`). See CHANGELOG for the full writeup.
- **v1.84 shipped**: Journey Planner water is now zero carried weight outside `desertLike` biomes
  (new `jpHumanWaterCarryDays` helper; `jpCalcLand`'s water term gated to `isDesert`, reverting
  v1.56's "any biome" widening back to desert-only per direct new owner instruction); swept
  `jpAssessResupply`, block/trace wording, the Info panel, and `_jpPlan`'s `waterL` route-summary
  total (a second independent recomputation that needed its own gate) for consistency. Desert-stage
  behavior stays byte-identical throughout. Hash vs v1.83 ALL IDENTICAL. See CHANGELOG for the full
  writeup.
- **v1.83 shipped**: a Mounted Rider party's own mounts now carry saddlebag capacity —
  `jpCapacity` credits `max(0, people - plan.animals[mount]) * JP_ANIMALS[mount].cap *
  JP_MOUNT_SADDLEBAG_FRAC(0.3)`, closing a real gap (a mounted party's capacity was previously
  identical to Walking's unless the mount was ALSO manually re-declared as a pack animal) without
  double-counting or rescuing a genuinely unsurvivable water-driven crossing (verified directly).
  Hash vs v1.82 ALL IDENTICAL. See CHANGELOG for the full writeup, including a test-writing
  mistake (wrong `_jpEnsurePlan` calling convention) caught and fixed before shipping.
- **v1.82 shipped**: ocean current direction becomes heat-driven (a western/eastern-boundary bend
  in `computeOceanCurrent`, reusing the existing coastal-distance weights — poleward pile-up on a
  basin's western edge, weaker equatorward upwelling on its eastern edge) instead of a flat,
  latitude-band-uniform Ekman drift; windFx particle speed slowed to 35% of its prior rate. Not
  bit-identical at defaults (`currents:true` feeds `field` via `carveRiverValleys`) but isolated
  and proven scoped: with `currents=false`, hash vs v1.81 is ALL IDENTICAL. See CHANGELOG for the
  full writeup, including the single-pixel-min/max test design mistake caught and replaced with
  robust aggregates, and the test-isolation bug found and fixed during verification.
- **v1.81 shipped**: wildlife-informed foraging — a new `JP_BIOMES.waterForage` column plus real
  `currentWildlife()` regional richness feeding the existing Foraging dropdown, giving a
  well-provisioned, biome-appropriate party a genuine way to extend both food AND water range
  instead of hitting `jpAssessResupply`'s binary block with zero elasticity. Two design forks
  resolved via `AskUserQuestion` (extend the existing dropdown; wire in real wildlife data). Hash
  vs v1.80 ALL IDENTICAL. See CHANGELOG for the full writeup, including the two OPEN items found
  during verification (above) and disclosed rather than silently fixed or dropped.
- **v1.80 shipped**: the v1.78 wind/current streak animation never actually rendered — a CSS
  `display:none` stylesheet rule silently defeated the `style.display=''` reveal, so the particle
  loop ran perfectly into an invisible 0×0 canvas. Fixed with an explicit `display='block'`; the
  v1.78 smoke assertion that should have caught this checked the wrong DOM property (inline style,
  not computed) and was corrected too. Hash vs v1.79 ALL IDENTICAL. See CHANGELOG for the full
  writeup, including the frame-diffing verification technique.
- **v1.79 shipped**: addon-village connectors now grow as a batched Prim-style forest (settlements
  ∪ already-joined villages), so a village can attach to a nearby sibling instead of always
  beelining to the closest big settlement. Measured before and after (mean connector 21.8→14.5 km,
  nearest-sibling-closer share 79.5%→36.5%); a chain-integrity probe confirmed every village still
  traces back to a real settlement, zero village-only clusters. Three designs were put to the owner
  via `AskUserQuestion` before building (see CHANGELOG for all three and why growing-forest won).
  Known scope cut: `BATCH`'s round-count formula is reasoned, not independently tuned against a
  "correct" cluster size — a candidate follow-up if the clustering ever reads as too coarse/fine.
- **v1.78 shipped**: terrain coupling made unconditional (the `state.climate.terrainWind` toggle
  is gone — deflection now runs unconditionally wherever elevation is supplied), the Wind/Ocean
  Layer views fixed to actually show the terrain-deflected/Ekman-rotated fields (a disclosed
  v1.77 scope cut), and an animated wind/current particle-streak overlay ported from the owner's
  PoC. Not bit-identical at defaults — `field` itself now differs from v1.77 via the
  `carveRiverValleys()` feedback chain, a disclosed, deliberate consequence. See CHANGELOG for
  the full writeup, including the mid-animation kind-switch crash found and fixed before shipping
  and three pre-existing test fixes the now-always-on coupling required.
- **v1.77 shipped**: terrain-coupled wind & ocean currents, ported from the owner's PoC
  (`terrain_coupled_flow_poc_2.html`) at the previously-agreed middle scope (wind/current
  terrain-coupling + gyre/western-intensification; full moisture/cloud/snowpack/seasonal-ITCZ system
  still deferred). New `deflectFlow`/`computeOceanCurrent` primitives, wired into `buildWind`/
  `oceanSSTAnomaly`/`simulateWeather` behind opt-in `state.climate.terrainWind` (default off — bit-
  identical at defaults). Both owner constraints verified by measurement: World-mode wrap seam diff
  exactly zero (wrap-aware), and a real `refreshClimate()` pass shows real `rainField`/`tempField`
  deltas with the toggle on (genuinely feeds the simulation, not decorative). See CHANGELOG for the
  full writeup, including a self-caught test-harness mistake during validation.
- **v1.76 shipped**: village connector "spaghetti" root-caused to an unnecessary `.reverse()` in
  `_civConnectVillageAddons`, not a terrain/discount issue — median circuity 2.62x→1.12x, self-
  intersections 54→0. See CHANGELOG for the full writeup.
- **v1.75 shipped**: `_civAutoRoutes` aIdx/bIdx index-base divergence (the v1.72 HANDOFF follow-up)
  fixed — trunk-road ways now remap their `settles`-local indices to `state.places` positions before
  joining the village-connector ways in `civWays`. See CHANGELOG for the full writeup.
- **v1.74 shipped**: Tiled LOD zoom freeze — a colorized tile is a static image, one composite per
  frame. Owner: "repeated quick zoom in-out actions cause a browser to freeze and become
  unresponsive." Three scheduling defects (undersized/mis-measured tile-canvas cache, every camera
  input compositing inline, and the overview-rebuild's own completion callback compositing
  unbudgeted) — nothing about what's drawn changed. Worst rAF frame 13.2 s → 1.4 s (9.6×). See
  CHANGELOG for the full writeup, including the follow-up commit that fixed the overview-rebuild
  call site and a stale `VERSION` constant the smoke suite caught.
- **v1.73 shipped**: label-collision draw/reserve mismatch (trait-badge clearance) unified behind
  `_civTraitDrop()`. See CHANGELOG for the four leads investigated and ruled out in the same pass.
- **v1.72 shipped**: bug-hunt pass fixing three v1.71 connector defects (save/load flag loss,
  Generate Roads destroying + re-creating connectors as trunk roads, way-list flooding). See
  CHANGELOG. The `aIdx`/`bIdx` index-base divergence that pass flagged and left open was fixed in
  v1.75, above.
- **v1.71 shipped**: addon villages now get a low-tier 'ancient' way connecting each to its nearest
  real settlement, deep-zoom-gated together with the village (`_civConnectVillageAddons`). See
  CHANGELOG for the full writeup and the self-loop bug the first cut (connect to nearest ROAD
  junction, not settlement) measured and fixed. Known scope cuts: no cap on connector length; a
  village on a landmass with no real settlement gets no connector; village-to-village connections
  are not modelled (every addon village always spurs to its nearest SETTLEMENT, never to a
  neighbouring village even when that would be shorter).
- **v1.70 shipped**: dense-village-grid and roadside-villages merged into one suitability-weighted,
  road-biased pass, replacing both toggles with `civVillagesChk`/`_civVillages`. See CHANGELOG for
  the full writeup. Known scope cuts: `roadFalloff` reuses `VILLAGE_SPACING_KM` rather than an
  independently calibrated decay constant; the comparative road-bias smoke check uses generous slack
  (a spacing-rejection interaction can occasionally trade one accepted candidate for another nearby);
  the "~60%" zoom framing is a proportional bump of v1.68's own un-calibrated constant, still not
  independently measured against a literal percentage. Only the primary map-click pick site is
  zoom-gated (table/right-click reach addon villages regardless of zoom, unchanged from v1.68);
  `_civAutoRoutes` alone (Generate Roads without a full Auto-populate) doesn't re-seed them.
- **v1.69 shipped**: roadside villages now also factor in settlement suitability, not just road
  spacing + dry land. See CHANGELOG for the full writeup.
- **the rest of a cut-off owner message, never resolved.** The desert-transition
  request that shipped as v1.66 arrived mid-typing, cut off after "Also I'd like that the current
  [...]." Asked the owner whether to wait for the rest or proceed on the water/desert-swap part
  alone via `AskUserQuestion`; they said proceed as-is. The unfinished sentence was never followed
  up — if the owner returns to it, that's the loose thread, not a new report. v1.66 also
  deliberately did NOT touch `JP_DRINKING_FLOW_DIVISOR`/`_jpStageDryKm` (v1.56, the water-RANGE-
  from-hydrology measurement) — the owner's own answer to the clarifying question narrowed "water
  carry" specifically to the animal-species accounting (already correct, confirmed by investigation)
  rather than the water-range model, so that axis is untouched and still a candidate if raised again.
- **v1.66 shipped**: per-stage pack-animal + vehicle fine-tuning with a swap advisory — the
  mule+cart → camel+travois desert-transition scenario. See CHANGELOG for the full writeup.
- **v1.65 shipped**: "when a stage gives a bug give a button to automate a fix" — one-click
  auto-fix buttons for the three deterministic blocked-stage subcases (seasonal closure, mounted/
  baggage-train-no-animals, wheel-vehicle-present). See CHANGELOG for the full writeup.
- **Civilization menu reorder (v1.59) — shipped.** See CHANGELOG/above for the full writeup;
  `#civSubBar` now reads Factions → Generation → Settlements → Economy → Statistics, and
  `#civSubGeneration` is an explicit Step 1→2→3 sequence. Pure reorganization — no scope cuts, no
  functionality changed.
- **Settlement placement clusters onto one faction (v1.58) — shipped.** See CHANGELOG/above for
  the full writeup; `_civAssignLandmassFactions()` apportions spare faction capacity across
  landmasses by highest-averages, weighted by summed settlement suitability. Known scope cut: the
  territory-fill MAP colouring still needs an explicit "Recalculate Territories" run to reflect the
  new per-settlement faction split (pre-existing two-button workflow, unchanged).
- **Water-constraint softening (v1.56) — shipped.** See CHANGELOG/above for the full writeup;
  `JP_DRINKING_FLOW_DIVISOR=16` + the auto water-crossing tier generalized to every biome. Not
  independently calibrated against a real-world drainage-density figure for this engine's specific
  grid resolution — a reasoned mid-value in a theoretically- and empirically-grounded band, not a
  historical constant. Would be the natural next refinement if the owner wants it tightened further.
- **The LOD tile seam is reduced, not eliminated (v1.29).** After moving the sea-floor smoothing to
  the shared world-wide fields, two adjacent tiles now agree at their shared world column to a RGB
  MAD of 0.04 (interior 0.3–0.6, was 6.71) — the tiles themselves are seamless. In the live composite
  the boundary column drops from being the single strongest colour discontinuity on screen (5.05× its
  local neighbourhood) to ~2×, which is consistent with the remaining artifact being the shared world
  column DRAWN TWICE: `amplifyRegion` samples a tile's box with inclusive endpoints, so tile A's last
  and tile B's first column are the same world position, and both get a device pixel. That is a
  one-pixel stutter, not a hairline. Closing it properly means rendering tiles with a one-pixel apron
  and cropping — which changes `pyramidTile`'s output shape and therefore the atlas format, so it was
  deliberately left out of a bug batch. Also still open in the same family: the other per-tile
  neighbourhood passes (`aoB`/`crestB`/`coastB`/`riverB`/`biomeBD`) have the identical
  truncated-at-the-edge defect, but all are opt-in (default 0) and already documented as per-tile
  decoration.
- **v1.29's "villages don't render correctly … terrain/sea alignment" was only partly reproduced.**
  The mechanism found and fixed is real and specific: a class-0 land cell sitting below the pooled
  surface of an adjacent lake reads dry at map scale and floods once the LOD renderer draws that
  lake's shoreline sub-cell, so a pin there ends up standing in water. If the owner's screenshot was
  showing something else (e.g. a genuinely blocky lake outline, or a pin offset from its terrain
  rather than submerged by it), that half is NOT fixed — the civ overlay and the terrain canvas are
  structurally co-registered (both take the same `lodViewRect()` and CSS-fit the same box), so an
  offset would be a different bug and needs the screenshot re-shared to pin down.
- **Religion Manager (deferred from the v1.18 request) — NOT started.** The owner's original ask
  paired a fully editable religion system (CRUD/merge/split/holy cities/diffusion via trade/
  migration/conquest/missionaries, per-settlement dominant/minority religion %/tension/conversion/
  pilgrimage) with the Interactive City Viewer; the owner explicitly prioritized the viewer first
  and deferred religion to a separate later effort. Religion today is still exactly what v1.10
  shipped: one categorical `civFactionReligion` value per faction, editable only in the Faction
  editor. Nothing per-settlement exists. Next session on this thread should treat it as its own
  audit-then-plan effort (the same "research existing patterns first" discipline v1.17/v1.18 both
  used), likely starting from `_civFactionAggregates()`'s `religious` power term and the City
  Viewer's own Religion panel (which currently shows the faction's dominant faith + the model's
  own `churches` — real data that a diffusion model would need to reconcile with, not replace).
- **Interactive City Viewer (v1.18) — shipped.** Documented scope cuts, not forgotten: (a) deep
  procedural-layout editing (rename a district, place a monument/landmark at an exact spot, edit
  an individual road/bridge inside the generated fabric) — needs a new persisted per-city
  edit-overlay data model (analogous to the Sculpt editor's stamp stack over the terrain field),
  a genuinely separate future feature; today's Edit button routes to the existing settlement-level
  editor (name/age/walls/specialisation/traits/history) instead. (b) Literal contour-terracing of
  mountain-town street layout — not modeled; UME's real-terrain integration (v1.17 S3) already
  lets street costs/building suitability read real slope, but there's no dedicated "grow along
  contour lines" rule. (c) A distinct "pilgrimage city" archetype with ceremonial roads — not
  modeled; a monastic/religious specialisation exists (v1.17 S2) but doesn't yet drive a
  temple-complex-plus-processional-avenue layout. All three are genuinely new engine capabilities
  (UME.cityGen would need new logic), not data the model already computes — unlike almost
  everything else the City Viewer surfaces, which was already generated and simply never drawn.
- **Sculpt mobile pan joystick (v1.19) — shipped.** `#sculptNavpad` (stick only, no zoom slider —
  Gen1's `#zoomOverlay` already has dedicated `+`/`−`), touch-only, shown only while
  `_sculptEditorActive()`. Not scoped beyond the Sculpt editor — a future session could extend the
  same pattern to other touch/paint tools (e.g. Cartography's paint mode) if that friction is ever
  reported; not requested here, so left alone.
- **Expanded natural-feature vocabulary (v1.20) — shipped.** Documented scope cuts, not forgotten:
  (a) `mountain`/`hill` pack-art variant *selection* stays purely positional, not climate-aware —
  only the zero-asset procedural fallback varies by temperature/aridity; a pack author who wants
  a snow-mountain vs. bare-mountain distinction still can't target one directly, only supply more
  variants that get distributed randomly. (b) The desert cactus/boulder split and the mountain
  snow-cap both key off a single, un-tuned threshold (10°C, 2°C respectively) — reasonable
  defaults, not validated against the climate sim's actual real-world calibration. (c) Not
  extended to `structures.trait` badges or biome/terrain ground-texture art (both already parsed
  by schema 2 but not drawn anywhere — a pre-existing gap, unrelated to this feature, still open).
  (d) No new POI/settlement slots — this was scoped to trees/ground-scatter/mountain-hill variety
  specifically, not a broader POI-vocabulary expansion (the user's uploaded reference sheet had
  far more categories — ruins, standing stones, lighthouses, culture packs, etc. — than fit this
  request; those remain reachable today only via the Asset Library's existing free-form `custom`
  icon family + sprite-sheet slicer, manually placed one at a time, not auto-attached to anything).
- **External QA report fixes (v1.24) — shipped.** 8 bugs (2 HIGH, 3 MEDIUM, 3 LOW), all verified real
  against the current file before fixing: World Structure sliders were completely dead
  (`ReferenceError` from an out-of-scope `segOn`); Delete-while-typing deleted the selected settlement
  (missing typing guard); three destructive actions had no confirm; no `beforeunload` guard existed;
  the busy overlay could hide early with queued ops; the asset-pack thumbnail gallery's host element
  was missing; several settlement/faction names weren't HTML-escaped in content contexts; a stuck
  Space-pan on Alt-Tab. **Known scope note:** BUG-7 (escaping) was fixed at the confirmed reported
  sites, not swept file-wide — a broader audit of every `innerHTML` content interpolation is possible
  future work if more instances are found, but wasn't attempted here (low severity, local-tool-only
  impact, and the file is ~24k lines).
- **Settlement pick-radius zoom scaling + Journey Planner sea-speed/vessel fixes (v1.23) — shipped.**
  (1) The settlement clickable area was a flat grid-space radius that ballooned on screen when zoomed
  in and blocked panning near a settlement — now zoom-scaled (`_civZoomPickR`) to a constant on-screen
  size at all five place-pick sites. (2) `JP_TERRAIN.sea` had Open Sea slower than Coastal Waters
  (systemic; wind is a separate axis) — reordered so Open Sea is fastest. (3) The vessel autoselector
  and the leg validator shared no compatibility source of truth (two inline copies) — consolidated to
  `_jpVesselWaterBlock`, so the selector can never pick what the validator rejects. **Known
  decisions / follow-ups:** (a) the sea multiplier for Open Sea (1.20) is a conservative +20%
  continuous-sailing edge over coastal; the absolute km/day (159.6 for a Cog at 14 h) runs a touch
  above the task's ~80-150 reference band purely because of the out-of-scope base ship speed + hours,
  not the multiplier — the ORDERING is what the bug was about. (b) The Dhow is deliberately kept
  open-sea capable (historically correct — Indian-Ocean monsoon dhows were ocean traders); if a
  coastal-only dhow variant is ever wanted, flip its `JP_SHIPS` entry (`openSea:false` +
  `invalidWater:["Open Sea","Rough Open Sea"]`) and both the selector and validator pick it up from
  the one shared rule. (c) Settlement pan-near-pin feel is canvas/pointer interaction — flagged for
  manual on-device confirmation; the headless smoke only asserts the radius shrinks with zoom.
- **Joystick direction + all-views + LOD0 supersample (v1.22) — shipped.** Three owner-reported
  items. (1) The Sculpt pan joystick moved the view opposite to the push — the v1.19 port dropped
  `Cartalith_V1.915.html`'s velocity negation; restored in `_sculptNavSetKnob`. (2) The joystick is
  now shown across all main-map views on touch (`_sculptNavSync` gate), hidden only behind the setup
  gate / in 3D / in the City Viewer. (3) LOD0 read as low-resolution because the LOD compositor was
  locked to the GW×GH `#view` canvas — new `_lodRenderW()` supersamples the LOD backing (2× field
  width, capped 2560px) and `lodViewRect` picks a finer pyramid level against it, so LOD0 composes
  from finer sub-tiles into a crisp canvas; the non-LOD path resets `#view` to GW×GH so the default
  render stays byte-identical. **Known scope cuts / follow-ups worth knowing:** (a) the supersample
  cap is a flat 2560px render-width ceiling and the supersample factor is a fixed 2× (not display-DPI
  aware), so on a very large 4K+ display LOD0 can still be gently upscaled beyond 2560px — a bounded,
  low-risk default chosen so low-end tablets stay smooth; making it DPI-adaptive is a possible future
  refinement. (b) The instant LOD overview placeholder (shown for a beat before the sharp tiles land)
  is still built at the 512px `OV_TARGET_W` cap — it's covered by the sharp finer tiles after the
  toggle-on / settle refine, so this only affects the brief pre-refine frame, not the steady state.
  (c) The civ-layer overlay canvases (settlements/labels) are still GW×GH; under LOD they read
  slightly softer than the now-supersampled terrain beneath them — alignment is unaffected (both
  CSS-fit the same box), only their own crispness. Not requested, left for a future pass if noticed.
- **Sprite-sheet slicer zoom/pan (v1.21) — shipped.** Zoom buttons + wheel-zoom-to-cursor + a
  dedicated Pan mode, leveraging the slicer's pre-existing (previously dormant)
  `overflow:auto` wrap for native scroll-based panning. No scope cuts to speak of — the request
  was fully self-contained to this one tool. Worth knowing for later: the zoom cap is a flat
  ~6000px canvas-dimension ceiling (memory-bounded), not tied to the sheet's native resolution, so
  even a modest sheet can zoom in well past 1:1 if that's genuinely useful for precision.
- **Settlement generation refactor (v1.17) — shipped (audit + S1–S7).** Documented scope cuts,
  not forgotten: (a) per-culture town morphology — `civFactionCulture` now reaches
  `opts.culture`, but UME still ships 2 profiles ('medieval'/'venus'), so every faction resolves
  to 'medieval' morphology; (b) `model.details` (including the new per-economy spoil-heap/
  drying-rack/log-boom props) are generated but the Gen1 map/preview renderers have never drawn
  details — a pre-existing limitation; the economy reads via district tints + yard-shed/warehouse
  building fabric instead; (c) wall terrain-following is bounded vertex deflection (±60 m,
  relief-relative), not a full ridge-tracing solver (stated in the audit); (d) valley WIDTH has
  no engine primitive — `currentFloodField()` is the documented floodplain proxy; (e) richer
  diagnostics (colour-coded district fills, per-epoch growth-stage view) would extend
  `state.viz.civDiagnostics`; the raster + fact-card overlays shipped cover the owner's
  validation list's core (footprint, slope, river class, wall spec, bridge/harbour validity).
- **Sculpt editor (v1.15) — fully shipped, all 8 phases of `docs/SCULPT_EDITOR_INTEGRATION_PLAN.md`
  §9 complete (P0 noise/geometry/registry through P7 docs/tests), including the P3 acceptance test
  (painting + committing with a Resources/Carry-Cap/Settlement debug view already open updates that
  view in the same frame — reverified directly for v1.15, since `computeFlow(true)` inside
  `sculptCommit()` invalidates `_resourcePots`/`_carryCapField`/`_settleSuitField` exactly like any
  other terrain edit). Nothing from the plan was deliberately deferred; the plan's own §4 registry
  table, §6 edge-character table, and §7 water table are the reference for any future per-feature
  tuning. World-wrap (equirectangular seam) handling was out of scope for the PoC-ported features
  from the start (`applyFeatureAlongCurve`, the retired plotline's own primitive, never had it
  either — parity, not a regression) — a future pass could add it to `sculptApplyStamp` uniformly
  if a `state.world`-mode sculpt session needs it.
- **Seamless region↔settlement refactor — Stage 1 (roads, v0.97), Stage 2 (water, v0.98) and Stage 3's
  coastline pass (v0.99) SHIPPED; the coastal WALL SIZING is the one substantive item left.** The
  coordinate trick below is the crux and is realised in `_umWaterCtx` + `buildSite`'s `opts.water` branch:
  - **Stage 2 (water, v0.98) — DONE.** Instead of one river polyline, the adapter (`_umWaterCtx`) feeds
    `buildSite` a whole real-water package via `opts.water`: the nearest real river centerline
    (`traceRiverPolylines(_riverNet.order,_riverNet.recv,GW,GH,minOrder)` → nearest stem, clipped to a
    resolution-aware box radius) AND a coarse local raster mask of ALL real water (sea + sub-sea-level
    lakes + stamped river band) with a chamfer distance transform. `buildSite`'s `isWater`/`riverDist`
    read the mask/DT and `river` is the real centerline (or a mask-extracted shoreline for a purely
    coastal town), so `waterPoly`/`bridgePt`/`bridgeDir`/`bankSide`/`harbourIdx` all recompute against
    real geometry. The synthetic path (no `opts.water`) is byte-identical (UME suite 831/831).
  - **The coordinate trick (the crux — still in force):** for the drawn water to land EXACTLY on the
    map, `generate()` forces **`anchors.market = box centre C`** (nudged off water if C is in the
    channel/sea) and `_umPlaceContext` forces **orient=0** on the real-water path — real geometry needs
    no rotation. The draw transform is `grid = p + R(orient)·(local − market)·gridPerMeter`; with
    market=C and the water referenced as `C + R(0)·(realGrid − p)/gridPerMeter`, the `(C − market)` shift
    is zero, so water (and the roads, which already anchor at market) draw pixel-for-pixel on the map.
  - **Stage 3 coastline pass (v0.99) — DONE.** (a) `_umWaterCtx`'s local water mask samples `field`
    **bilinearly** per 22 m cell (not nearest grid cell), so the town's coast is a smooth heightmap-
    following curve rather than one blocky box at coarse resolution; (b) `townBank`'s water-following
    offset now points toward the actual land (market side) for any coast facing, not the synthetic
    `y−5` "north" (guarded on `site.usesRealWater` ⇒ UME suite byte-identical). `shoreFromMask` still a
    rough PCA chain, but it feeds off the smoother mask now.
  - **Coastal WALL SIZING (the item left, → v1.00):** on a coastal town the enceinte is sized from the
    street-graph built-mass hull (`builtMassHull`), which folds in bare junctions on the arterial roads
    entering the town, so the wall can enclose a wedge of empty land beyond the built fabric (built mass
    in the seaward corner, wall stretching inland along a road). `buildWall` runs inside `grow()`, BEFORE
    `buildBlocks`, so it has no block/parcel fabric to size against — the fix is to size the ring from
    the actual built density (either defer/rebuild the wall after blocks exist, or weight `builtMassHull`
    toward true intersections — degree ≥ 3 nodes — and away from degree-2 arterial pass-throughs). This
    is a growth/hull change in the synthetic-tested engine, so guard it on `usesRealWater` (or prove
    UME 831 byte-identical). Also still: "river through the town" reads best at 1K/2K (512px box ≈ one
    grid cell). Verify per change: engine 923, UME 831, hash ALL-IDENTICAL (default-off), smoke,
    fixed-seed screenshots.
- **Urban morphology (v0.95) — the requested feature is fully shipped; several scope cuts are
  documented, not forgotten.** Faction→culture/tradition mapping (the PoC ships 2 culture
  profiles; every settlement currently generates as `'medieval'` — worth revisiting once/if
  factions carry a culture concept of their own). Full terrain-sourced site GEOMETRY: the port
  currently classifies real terrain into a site TYPE (river/riverthrough/bay/coast/landlocked)
  but the river curve/bridge/harbour placement stays the PoC's own self-consistent synthetic
  generation (mixing synthetic geometry with real `isWater()` was judged unsafe without a full
  redesign — see the CHANGELOG v0.95 entry's reasoning). The PoC's parcels layer and fine detail
  objects (trees/wells/market crosses/cranes/bollards) aren't drawn in the canvas renderer yet
  (kept out of v1 for per-frame draw cost — blocks/walls/streets/buildings are). No era signal
  (`civYear`) drives wall-vs-star-fort epochs over simulated time yet. None of these were asked
  for explicitly; pick up only if the owner wants deeper fidelity.
- **LOD/render performance optimizations — 4 of 6 originally-proposed shipped across v0.93/v0.94**
  (progressive overview rebuild, pooled tile refine, pooled atlas bake, `sampleArr` row-hoisting —
  see the v0.93/v0.94 entries and CHANGELOG for full detail). **Explicitly deferred, not
  forgotten:** (a) palette-function scratch-ification (`snowCol`/`rockCol`/etc.) — designed in
  v0.94, surfaced a genuine nested-call aliasing hazard (`grassCol` calls `ramp3` twice before
  consuming either result — a shared single scratch buffer would silently corrupt colors); needs a
  proper multi-slot scratch design, not a rushed single-buffer one. This is the project's own
  next performance-audit-roadmap item, still open. (b) A 6th proposal from the original list was
  never detailed in this thread — re-derive or ask the owner if further LOD/render perf work is
  wanted. Neither is a known bug or regression.
- **Rivers-as-ways (v0.94) — shipped as an overlay + new default, auto-network builder untouched.**
  `drawRiverWays()` is reusable as-is if a future version wants it wired into more places (e.g. an
  export/print map style). Not queued as follow-up unless requested.
- **Sea/river-aware routing (v0.94) — scoped to the interactive Route tool/journey planner by
  owner decision.** The auto-generated world road network (`_civHierarchicalNetwork`+
  `_civMstRoutes`, used by auto-populate) is still architecturally two disjoint land-only/water-only
  Dijkstra passes and cannot produce a single mixed route between arbitrary settlements — a real,
  separate limitation from the one fixed this version, flagged in the v0.94 research but explicitly
  out of scope. Candidate for a future version if the owner wants the auto-generated network to
  also route mixed land+sea+river.
- **Save/export architecture restructuring — SHIPPED in v0.92.** `docs/research/save-export-architecture-
  audit.md` (read-only audit, 2026-07-13) found the real bloat was `exportZip()` writing overlapping map
  imagery from three independent code paths for the same terrain, and a separate, lower-risk naming/IA
  muddle in "Tiles & LOD" burying "Atlas" two levels deep. Owner's /goal ("carry out the reported fixes")
  shipped both: the audit's §5A (skip the redundant flat bake for a finalized world; make the 4
  `layers/*.png` previews opt-in) and §5B/C (split into Tiled LOD view / Atlas cache / Region export). See
  the v0.92 CHANGELOG entry for the full list. `biome_raster.bin` vs `biome_baked.bin` (the owner's own
  "double data" example) is intentionally **not** touched — the audit found they're two genuinely
  different classifiers for two different consumers, not duplication. Nothing queued from this thread.
- **LOD zoom performance — root-caused and fixed in v0.92, then quality-corrected same day.** Same
  /goal, second half: "why is zooming in slow even when tiles are baked." Real answer (found via
  `performance.now()` profiling, not guesswork): the sharp tile overlay *does* correctly serve from the
  atlas when baked — the actual bottleneck was the "instant overview" backdrop underneath it, rebuilt at
  full `GW×GH` resolution through the same expensive per-pixel colorizer on every zoom step, never
  consulting the atlas at all (~940ms/step measured, identical whether baked or not). First fix rendered
  that backdrop at a flat quarter resolution (12.3-16.6× faster) — but this starved small lakes (no
  `_coastSDF` smoothing outside the ocean coastline) of samples, reported back same-day as "lakes are
  blocky/pixilated again" + "aspect ratio goes weird" (the same artifact, described two ways). Replaced
  the flat ratio with a **512px fixed target output width**, which bounds the overview's cost
  independent of world resolution (~100-130ms at both 1024px and 2048px, vs. the rejected flat-`/2`
  alternative's ~420-440ms at 2048px) while giving small worlds full resolution. A same-day second
  report ("graphic fidelity seems to have degraded also") traced to a *pre-existing* gap the cap
  exposed rather than caused: the `lodChk` checkbox never scheduled the sharpen-on-settle pass every
  other LOD entry point already gets, so ticking it and just looking left the user on the coarse
  overview indefinitely. Fixed by triggering that pass immediately on checkbox-check. All three fixes
  locked in with permanent smoke-suite regression guards (timing + overview-canvas-size/aspect +
  checkbox-triggers-refine). Nothing queued — if the owner reports zooming still feels slow after this,
  the next place to look is the TILE overlay's own per-tile colorization cost (not yet profiled in
  isolation) or the debug-overlay vector drawing pass (`drawLODDebugOverlays`), not the overview
  backdrop these fixes already addressed.
- **The owner's 2026-07-12 /goal (settlement pop-up + Explore timeline rework) is now fully shipped**
  across v0.90 (settlement editor → map pop-up) and v0.91 (timeline: one home, real time-scale — see
  above). No queued follow-up on either; nothing else outstanding from that request.
- The queued work tracked at the end of the pre-merge era (browser passes above) plus whatever
  the user asks next. Check `docs/ROADMAP.md` for the long arcs; recent `CHANGELOG.md` entries
  state per-feature follow-ups (e.g. cross-tile seam editing is the one genuinely open LOD item).
- **Zoom/scale feature-rendering track (owner goal) is now JS-complete** through v0.72: registry
  (v0.71) → LOD render caches (v0.71) → per-zoom morphology valleys/fjords/canyons/meanders (v0.71) →
  tributaries + local incision (v0.72). What remains is explicitly deferred and needs a decision:
  **oxbow cut-offs** (a scalar-field carve can't do them — needs true centerline curvature tracking on
  the river polylines) and the **Rust/WASM engine port** (owner chose JS-first). A full browser pass on
  the deep-zoom morphology (does it read as rivers/fjords/canyons to the eye, and is the ridged-noise
  tributary pass fast enough on 1024² tiles at z8–z10) is owed before calling the visual side done.
