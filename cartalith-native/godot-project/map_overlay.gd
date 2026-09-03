extends Control
## Phase 2 civilisation-layer overlay (cartalith-civ): settlements + roads
## drawn on top of `%MapView`'s terrain texture. Godot computes nothing
## here beyond screen-space placement -- every value drawn (position,
## faction, tier, name, population) comes straight from `WorldGen.
## get_settlements()`/`get_roads()` (ARCHITECTURE.md: "Godot computes
## nothing beyond layout").
##
## Sibling of `%MapView` under the same `MarginContainer` (`MapMargin`),
## so both share an identical laid-out rect -- this control's own `size`
## is what `%MapView`'s texture is drawn into. `%MapView` uses
## `stretch_mode = 5` (`STRETCH_KEEP_ASPECT_CENTERED`), so the texture
## itself is letterboxed within that rect; `_displayed_rect()` reproduces
## that same fit/centering math so markers land on the real pixels they
## represent, not on the raw control bounds.

## Okabe-Ito colourblind-safe qualitative palette (Okabe & Ito, 2008) --
## 6 mutually distinct hues, matching `CIV_FACTION_COUNT` in
## `cartalith-godot` exactly (faction ids are 1-based, index with `- 1`).
## Chosen deliberately independent of the UI's own light-parchment theme:
## this is data-driven map content (which faction owns this settlement),
## not UI chrome -- the same reasoning the terrain renderer's own biome
## colours already follow (theme-independent, see CHANGELOG's UI-reskin
## entries).
##
## **This is the fallback now, not the authority** (2026-09-01). It matched
## `lib.rs`'s `FACTION_RGB` byte for byte, and the engine stopped indexing
## that table directly: `CivData::faction_rgb` consults the roster's own
## `color_override` first (`GUI_GAP_REGISTER.md` CV-21) and falls through to
## `faction_rgb_default`, whose rule past the sixth faction is
## `civ_faction_color`'s golden-angle rotation -- explicitly *not* a
## `% FACTION_RGB.len()` wrap, "which would have given faction 7 faction 1's
## exact colour". Indexing this table with `% 6` was doing exactly that, and
## ignoring every user colour edit besides, so a settlement pin disagreed
## with the territory wash under it, with the Political-control field and
## with the roster swatch -- three surfaces that all go through
## `faction_rgb`. `set_faction_colors()` pushes those same swatches in;
## these six are what is drawn before a world exists to push any.
const FACTION_COLORS: Array[Color] = [
	Color(0.902, 0.624, 0.0),   # orange
	Color(0.337, 0.706, 0.914), # sky blue
	Color(0.0, 0.620, 0.451),   # bluish green
	Color(0.941, 0.894, 0.259), # yellow
	Color(0.0, 0.447, 0.698),   # blue
	Color(0.835, 0.369, 0.0),   # vermillion
]

## Reference's `CIV_SETTLEMENT_CLASSES` (line 14674), restricted to the six
## tiers `SettlementKind` -- and so `get_settlements()`'s own `kind` field --
## can actually produce. `metropolis` (rank 5, glyph ★) joined the list on
## 2026-08-20 when `_civSelectMetropolises` was ported: a promoted imperial
## seat is a real settlement kind now, not just a manual-icon slot. The
## monastery/fortress/university/industrial special kinds are still never
## assigned to a real settlement.
## `rank` drives `_civDrawSettlementPin`'s own size formula (`(4+klass.rank)
## *sc`, reference line 15166) exactly; `glyph` is the same per-tier
## character the reference draws centred on the pin (line 15180), reused
## here now that this control draws more than a flat circle. Replaces the
## old hardcoded `TIER_RADIUS` px dict -- radius is now derived below
## (`_settlement_pin_radius`), not looked up from an arbitrary constant.
const SETTLEMENT_CLASS := {
	"hamlet":  {"rank": 0, "glyph": "⌂"},
	"village": {"rank": 1, "glyph": "●"},
	"town":    {"rank": 2, "glyph": "◉"},
	"city":    {"rank": 3, "glyph": "⬣"},
	"capital": {"rank": 4, "glyph": "✦"},
	"metropolis": {"rank": 5, "glyph": "★"},
}

## `CIV_LOD_PLACE` (reference line 15373): the minimum RAW camera zoom
## (`_camera_zoom` itself -- the same un-clamped scale `_civZoomK()`/
## `_civZoomRaw()` both start from before either clamps or inverts it,
## reference line 15003) at which a tier's full pin+glyph+label shows.
## Below it the reference does not hide the place outright -- it draws a
## small faction-tinted dot instead (reference lines 15744-15757: "render a
## small faction-tinted dot instead of hiding the place entirely -- road
## endpoints stay anchored to a visible marker at any zoom"), so a capital
## or city (threshold 0) is always full-size, town needs a little zoom-in,
## and village/hamlet need progressively more -- restricted here to the
## six tiers `SettlementKind` (and so `SETTLEMENT_CLASS` above) actually
## produces; the reference's own dict also carries monastery/fortress/
## ruin/etc. this port's settlements never have. `metropolis` is `0` in the
## reference's own dict (line 15374), the same as capital and city.
##
## This is not a port invention: it is the same population/importance-
## tiered LOD reveal real map renderers use for POI/place density --
## OpenStreetMap Carto renders cities/towns from ~z9, villages from ~z12,
## hamlets from ~z14 (github.com/gravitystorm/openstreetmap-carto), and the
## Mapbox/MapLibre style spec's `minzoom`/`maxzoom` per symbol layer is the
## same mechanism generalised (maplibre.org/maplibre-style-spec/layers/).
const SETTLEMENT_LOD := {
	"metropolis": 0.0,
	"capital": 0.0,
	"city":    0.0,
	"town":    0.4,
	"village": 0.7,
	"hamlet":  1.4,
}
## `CIV_VILLAGE_ADDON_LOD` (reference line 6446) -- the threshold for the
## *additive* village layer the `villages` generation flag produces, which the
## reference gates separately from `CIV_LOD_PLACE` and, uniquely, hides
## **outright** below it: "Below it the pin is fully hidden (not the small-dot
## fallback other kinds get) -- the whole point is to keep the low-zoom map
## uncluttered." Its own comment names the complaint it was written for
## ("waay too populated") and the previous value it was raised from (2.0).
##
## The port had no equivalent: `lib.rs` folds `civ_seed_villages`' output into
## the settlement roster as plain `Hamlet`s ("a village renders exactly like
## any other hamlet, which is what the reference's own hamlet-tier tagging for
## these already implies") and so drew every one of them under
## `SETTLEMENT_LOD.hamlet` with a dot fallback. That inference is the one place
## it does not hold -- the reference tags them `villageAddon` precisely so the
## renderer will *not* treat them as hamlets. Measured on the shell's own
## default world (the shell defaults `villages` to true, where the reference
## defaults it false): 209 addon villages against 24 real settlements, all 209
## drawn full-size with pins and names from 1.4x zoom. That is the owner's
## "minor settlements are always visible" (2026-08-24).
const VILLAGE_ADDON_LOD := 2.4
## An addon village's only signature in `get_settlements()`: `lib.rs` gives it
## an unconditional `pop: 0` (its own comment, and `VillageSettlement`'s in
## `cartalith-civ`), while every base settlement goes through
## `name_and_populate_settlements`' suitability formula, whose floor for the
## smallest tier is `round(120 * 0.7 * 0.8) = 67`. Exact, not heuristic, for
## the default pipeline.
##
## It is a proxy nonetheless, and the honest fix is to expose the flag the
## engine already keeps (`CivData::village_tids`, alongside the `tid` this
## dictionary already carries) -- registered in `GUI_GAP_REGISTER.md`. One case
## degrades until then: with the static post-collapse recovery phase enabled,
## `civ_apply_recovery` floors every population at 8, so an addon village stops
## reporting 0 and falls back to being drawn as an ordinary hamlet. That is
## today's behaviour, so the degradation is "no improvement", never a place
## wrongly hidden.
const VILLAGE_ADDON_POP := 0

## Reference's own low-zoom fallback dot (line 15752-15756):
## `dr=(isPoi?1.4:1.9)*lsc` plus a `+0.6*lsc` dark outline ring -- `lsc`
## there is this file's own per-frame `sc`.
const LOD_DOT_RADIUS_SC := 1.9
const LOD_DOT_OUTLINE_SC := 0.6
## `sc`, screen px per size-formula rank-unit. Reference: `sc=max(1,GW/512)*
## civZoomK()*civIconScale()` (line 15165) -- a canvas-resolution term times
## an inverse-CSS-zoom term times a user icon-scale, so a pin holds roughly
## the same ON-SCREEN size regardless of grid resolution or camera zoom
## (`_civZoomK`'s own comment, line 14976-14978: "shrinking in canvas-space
## as you zoom in so the on-screen size ... stays roughly constant" -- the
## exact mechanism, verified against the real function rather than trusted
## from the paraphrase, `_civZoomK()` at line 14980-14983:
## `1/clamp(viewT.scale, 0.35, 5)`).
##
## Correction (2026-08-19, the LOD/settlement-fidelity bug pass): an earlier
## version of this comment argued the inverse-zoom term could be dropped
## entirely because "this control has no separate canvas/CSS-zoom split to
## reproduce... camera zoom is a plain transform `ViewportHost` applies to
## this whole control, the same role the reference's CSS transform plays
## over its canvas." That description of the transform was correct and the
## conclusion drawn from it was backwards: the reference needs `_civZoomK()`
## *precisely because* its own CSS transform plays that exact role -- the
## whole job of `civZoomK()` is shrinking the pin in canvas-space so that
## when the CSS transform (there) / `_camera.scale` (here, `ViewportHost.
## zoom()`) multiplies the rendered size back out, the two cancel and the
## ON-SCREEN size stays constant. Without an equivalent term here,
## `_camera.scale` alone was doing the reference's zoom-transform half of
## that cancellation with nothing supplying the other half, so a pin's
## on-screen size grew linearly with `ViewportHost.zoom()` instead of
## holding steady -- confirmed numerically before this fix (a settlement
## pin at `zoom=1.0` vs. the same pin at `zoom=4.0` measured 4x the on-screen
## radius, not the same one). `_civ_zoom_k()` below is the missing term,
## keeping the reference's own `0.35` zoom-out floor. Its `5.0` zoom-*in* cap
## was kept too, on the argument that the clamp is a readability bound with no
## reason to track this port's own pan/zoom range -- **wrong, and corrected on
## 2026-08-24**: past that cap the term stops cancelling and the pin resumes
## growing linearly, which is the very defect this whole comment is about. It
## was a 1.6x overshoot while `ViewportHost` capped at 8.0 and became 32x when
## the cap became `lodMaxZoom()`. `_civ_zoom_k()` below carries the full
## reasoning and the measurement.
##
## The resolution term is still ported the same way this comment always
## described: tying `sc` to `rect.size.x` ALONE (not `rect.size.x/_gw`,
## unlike `tool_overlay.gd`'s brush-cursor radius) keeps a pin's on-screen
## size independent of grid resolution, matching what the reference's own
## `GW/512` term is actually for; a literal `radius_cells*(rect.size.x/_gw)`
## port would instead shrink pins on a bigger grid, which is right for a
## brush (a real world-space distance) but wrong for a pin's glyph size (not
## a world-space quantity at all). `PIN_SCALE_REF_PX` is a tuned constant
## this port chose (not a reference value -- there is no equivalent number
## to port), sized so a typical dock-width viewport lands close to the
## pre-formula `TIER_RADIUS` constants it replaces.
const PIN_SCALE_REF_PX := 1400.0
const CAPITAL_RING_WIDTH := 2.5

## The faith-divergence ring (see the `_faith_diverged` block further down for
## why this layer is a shape rather than a religion palette). Three 54° arcs
## with 66° gaps, outside the capital ring's own radius so a diverged capital
## draws both and neither is mistaken for the other -- a capital ring is
## solid, in the faction's own colour; this one is broken, and one fixed ink.
##
## Deliberately NOT any faction colour and not any theme token: it is a map
## *annotation*, and this control's palette independence (top of file) is
## exactly so a mark like this does not change meaning with the UI theme.
const FAITH_RING_ARCS := 3
const FAITH_RING_SPAN := 0.94          ## radians, ~54°
const FAITH_RING_PAD_SC := 2.2
const FAITH_RING_WIDTH := 1.6
const FAITH_RING_COLOR := Color(0.929, 0.921, 0.847, 0.92)
const FAITH_RING_SHADOW := Color(0.047, 0.039, 0.027, 0.75)

## Soft drop-shadow under each full-tier pin (2026-08-19, "settlement
## rendering could be made graphically more interesting" pass, explicit
## owner latitude to improve past the reference's own plain filled-circle
## baseline -- `DECISIONS.md` §7a "principled equivalence" room, same as
## this session's own Asset Library/Travel Library work). A flat faction-
## coloured fill alone reads ambiguously against some of this renderer's
## own biome colours (pale sand, snow, light grassland) without a dark halo
## under it; the reference (`_civDrawSettlementPin`) draws no shadow at
## all, so this is a genuine addition, not a port of anything.
const PIN_SHADOW_COLOR := Color(0.0, 0.0, 0.0, 0.28)
const PIN_SHADOW_OFFSET_SC := 0.6

## A small water-blue "harbour" badge at a coastal pin's lower-right --
## grounded in real, derived engine data (`get_settlements()`'s own
## `coastal` field, `civ_is_coastal` in `cartalith-civ`, re-verified against
## final placement geometry by this same pass's Part 1 fix), not invented
## decoration. River adjacency has no equivalent per-settlement flag
## anywhere in `cartalith-civ` (`NamedSettlement` carries no river bool),
## so only the coastal case gets a badge -- a river indicator would have no
## real data backing it.
const COASTAL_BADGE_COLOR := Color(0.337, 0.706, 0.914, 0.95) ## FACTION_COLORS[1]'s sky blue -- reads as "water" at a glance
const COASTAL_BADGE_OUTLINE := Color(0.051, 0.043, 0.031, 0.9)
const COASTAL_BADGE_R_SC := 0.55
## Reference's settlement-label fill (`#f6ecd4`, line 15206) -- close enough
## to `LABEL_STROKE_COLOR` below's own outline colour (`rgba(8,6,4,.85)`,
## line 15198) that this control's existing region-label palette already
## matches it; only the fill needed a name of its own.
const SETTLEMENT_LABEL_FILL := Color(0.965, 0.925, 0.831)
## Land-way styling by `way_type`, one entry per branch of the reference's own
## `drawCivLayer` §2a type ladder (reference HTML lines 15511-15534). Every
## land type there is **two strokes**: a dark underlayer first, then a lighter
## coloured overlay on top of it -- solid for the two trunk tiers, dashed for
## the three minor ones -- which is what makes a highway, a track and an
## ancient way tell apart at a glance rather than by width alone.
##
## Each entry is that branch's own literal `strokeStyle`/`lineWidth`/
## `setLineDash` values converted to `Color`/px, `rsc` factored out (widths
## here are screen px -- see `_draw_way_segment`'s own note on `_crisp_begin`):
##
## | type     | reference line | underlayer            | overlay                  | dash      |
## |----------|----------------|-----------------------|--------------------------|-----------|
## | highway  | 15515-15517    | `rgba(20,10,5,.55)` 2.3 | `rgba(210,145,55,.98)` 1.45 | solid   |
## | regional | 15519-15521    | `rgba(25,14,5,.45)` 1.8 | `rgba(178,118,52,.88)` 1.15 | solid   |
## | road     | 15531-15534    | `rgba(30,20,10,.4)` 1.2 | `rgba(160,100,60,.75)` 0.7  | `[1.8,1.3]` |
## | track    | 15523-15526    | `rgba(30,20,10,.35)` 1.1 | `rgba(100,120,60,.75)` 0.6 | `[1.3,2]`   |
## | ancient  | 15527-15530    | `rgba(20,10,5,.35)` 1.1 | `rgba(120,110,100,.65)` 0.65 | `[2.5,1.3]` |
##
## `highway`/`regional`/`road`/`track` are `cartalith_civ::WayType`'s four
## peak-corridor-usage tiers (Phase 2 milestone 14); `ancient` is not a
## `WayType` at all but the fourth `ManualWayType`, reaching this control since
## `get_roads()` began appending hand-drawn ways (IN-02). The reference gives
## it its own grey dashed branch, and now so does this.
##
## **This replaced a single flat `ROAD_COLOR` (2026-08-24.)** Every land way
## was stroked `Color(0.36, 0.29, 0.16, 0.55)` regardless of type, with only
## the width varying -- measured, not assumed: a two-background pixel probe
## recovered exactly `C=(91,75,40) a=0.549` on all five types. So a track and a
## highway differed by 1.5 px of width and nothing else, and `ancient`'s grey
## and `track`'s olive -- the two the reference deliberately colours *away*
## from the road ochre -- were indistinguishable from a trunk road.
const WAY_STYLE := {
	"highway": {
		"under": Color(0.078, 0.039, 0.020, 0.55), "under_w": 2.3,
		"over": Color(0.824, 0.569, 0.216, 0.98), "over_w": 1.45,
		"dash": 0.0, "gap": 0.0,
	},
	"regional": {
		"under": Color(0.098, 0.055, 0.020, 0.45), "under_w": 1.8,
		"over": Color(0.698, 0.463, 0.204, 0.88), "over_w": 1.15,
		"dash": 0.0, "gap": 0.0,
	},
	"road": {
		"under": Color(0.118, 0.078, 0.039, 0.4), "under_w": 1.2,
		"over": Color(0.627, 0.392, 0.235, 0.75), "over_w": 0.7,
		"dash": 1.8, "gap": 1.3,
	},
	"track": {
		"under": Color(0.118, 0.078, 0.039, 0.35), "under_w": 1.1,
		"over": Color(0.392, 0.471, 0.235, 0.75), "over_w": 0.6,
		"dash": 1.3, "gap": 2.0,
	},
	"ancient": {
		"under": Color(0.078, 0.039, 0.020, 0.35), "under_w": 1.1,
		"over": Color(0.471, 0.431, 0.392, 0.65), "over_w": 0.65,
		"dash": 2.5, "gap": 1.3,
	},
}
## The reference's own `else` arm: an unrecognised `type` falls to the `road`
## branch rather than being skipped (line 15531's comment says so outright,
## `// road (default)`).
const WAY_STYLE_DEFAULT := "road"

## `CIV_LOD_ROAD` (reference HTML line 15380, read by `_civWayLodMin` at 15012):
## the camera zoom below which a way of this type is not drawn at all --
## `GUI_GAP_REGISTER.md` **CA-18**'s zoom ladder, for the one layer the
## reference actually ships one for.
##
## Registered as unbacked ("no per-layer zoom range exists anywhere in the
## shell"), and for the layers v3 lists that is still true. For *ways* the
## reference has a real table, and this is it, ported verbatim. Its effect
## here is narrower than there and deliberately not widened: `ViewportHost`'s
## `ZOOM_MIN` is **0.4**, so `road`'s 0.35 threshold is unreachable and only
## `track` and `ancient` ever drop out -- between 0.4 and 0.7, which is the
## zoomed-right-out view where a minor track is a 1 px scratch anyway. The two
## trunk tiers are `0` there, meaning "always", not "missing".
##
## `sea-lane` is `0` in the reference too and is drawn from a different getter
## here, so it never reaches this lookup; kept in the table so the table is
## the reference's table.
const WAY_LOD_MIN := {
	"highway": 0.0, "regional": 0.0, "road": 0.35, "track": 0.7,
	"ancient": 0.7, "sea-lane": 0.0,
}
## `_civWayLodMin`'s own fallback for a type not in the table above.
const WAY_LOD_DEFAULT := 0.35
const MARKER_OUTLINE := Color(0.101, 0.070, 0.023, 0.85) ## matches PrimaryButton's ink tone
const HOVER_RADIUS_PAD := 4.0 ## extra hit-test slack (px) beyond the drawn marker radius

## Sea-lane style: the `sea-lane` arm of the same §2a ladder `WAY_STYLE` above
## covers the land types of (reference HTML lines 15511-15514) -- a dark navy
## solid underlayer plus a lighter dashed overlay, deliberately away from every
## land-road hue so a shipping lane never reads as a road. Kept as its own
## constants rather than a sixth `WAY_STYLE` row because sea lanes arrive from
## a different getter (`get_sea_routes()`, never `get_roads()`) and so never
## reach `WAY_STYLE`'s lookup.
const SEA_ROUTE_UNDERLAY := Color(0.039, 0.118, 0.235, 0.4)
const SEA_ROUTE_UNDERLAY_WIDTH := 1.5
const SEA_ROUTE_DASH_COLOR := Color(0.118, 0.510, 0.784, 0.7)
const SEA_ROUTE_DASH_WIDTH := 0.85
## `setLineDash([2.6*rsc, 2*rsc])` -- an *unequal* pair. The gap was 2.6 here
## until 2026-08-24 (it fell out of `_draw_dashed_polyline`'s "gap defaults to
## dash" convenience default), which stretched the lane's period from the
## reference's 4.6 to 5.2 and left every dash separated by a gap as long as
## itself. Measured, not assumed: the dash probe read a 26 px period at 5x
## width scale where the reference's is 23.
const SEA_ROUTE_DASH_LENGTH := 2.6
const SEA_ROUTE_DASH_GAP := 2.0

## §4.5.4's Route tool: a committed *route* is not infrastructure. It is a
## solved journey across the existing network (`route_commit` ->
## `civ_join_dijkstra_segs`, mixed land/sea), and the reference draws it as
## its own layer on top of the ways it follows (`drawCivLayer` block 2b,
## reference HTML lines 15552-15560, `civJourneys.forEach`) -- a dark
## underlayer stroke, then a dashed amber overlay. These four constants are
## that block's own unselected-journey values converted to `Color`:
## `rgba(40,25,5,.5)` at `lineWidth 3`, then `rgba(200,160,60,.85)` at
## `lineWidth 1.5` with `setLineDash([5,3])`.
##
## Reusing `ROAD_COLOR` would make a route invisible, which is exactly what
## happened before this existed: a committed 578 km route drew nothing at all
## (`GUI_GAP_REGISTER.md` IN-09).
const MANUAL_ROUTE_UNDERLAY := Color(0.157, 0.098, 0.020, 0.5)
const MANUAL_ROUTE_UNDERLAY_WIDTH := 3.0
const MANUAL_ROUTE_COLOR := Color(0.784, 0.627, 0.235, 0.85)
const MANUAL_ROUTE_WIDTH := 1.5
const MANUAL_ROUTE_DASH := 5.0
const MANUAL_ROUTE_GAP := 3.0

## Block 2b's `sel` branch, the same three values it varies and no others:
## `lineWidth (sel?5:3)`, `strokeStyle sel?'rgba(255,210,80,.98)'`,
## `lineWidth (sel?2.5:1.5)`. The dash pattern is NOT selection-dependent in
## the reference and is not made so here. Wired since IN-09's second half
## (2026-08-24), when `route_delete`/`route_set_name` gave the Routes list a
## selected row to drive it -- before that there was no route selection in
## this shell at all.
const MANUAL_ROUTE_SEL_UNDERLAY_WIDTH := 5.0
const MANUAL_ROUTE_SEL_COLOR := Color(1.0, 0.824, 0.314, 0.98)
const MANUAL_ROUTE_SEL_WIDTH := 2.5

## §4.5.5's Icon tool markers, by `icon_dict`'s `family` key
## (`cartalith_assets::manual::ManualIconFamily::key()`). No texture atlas
## from the asset pack is wired into Godot yet (`icon_bridge.rs`'s art is
## rasterised only into the baked terrain texture, never exposed as
## individually-addressable sprites here) -- these are honest placeholder
## glyphs distinguishing family and marking real placed positions, not a
## stand-in for the pack's actual per-slot art.
const ICON_FAMILY_COLORS := {
	"settlement": Color(0.835, 0.369, 0.0),
	"feature": Color(0.0, 0.620, 0.451),
	"poi": Color(0.941, 0.894, 0.259),
	"custom": Color(0.337, 0.706, 0.914),
}
## **This is not the engine's icon radius, and the two do not convert**
## (recorded 2026-09-01).
##
## Here the drawn mark is `ICON_BASE_RADIUS * max(0.2, scale)` in this
## control's own LOCAL pixels -- no `rect`/grid term and no `_civ_zoom_k()`
## term -- so the camera, which is an ancestor scale (see `_crisp_begin()`),
## multiplies it: an icon's mark grows with zoom rather than holding a
## constant on-screen size the way a settlement pin does. `_draw_landmarks`
## is written the same way, so this is a shared property of the two newer
## annotation layers and not a slip unique to this constant.
##
## The engine's `manual.rs::icon_box_at` computes
## `r = 5.0 * max(1, grid_w/512) * civ_zoom_k(zoom) * icon_scale * icon.scale`
## in GRID CELLS, and `cartography_workspace.gd` uses it that way: both the
## resize-handle hit test (`Vector2(gx, gy).distance_to(...) <= h["r"]`) and
## the drawn handle come from `icon_handles`. The two expressions share no
## term. One is grid-relative and zoom-stabilised under a clamp; the other is
## a bare local-pixel constant. They can only coincide at one accidental
## combination of grid width, viewport width and zoom, and the ratio between
## them swings as the camera moves -- which is why the resize handle does not
## sit on the mark it resizes.
##
## **Deliberately not "fixed" by editing this number.** Making the mark match
## the engine means adopting `civ_zoom_k`'s clamp, which `_civ_zoom_k()` above
## rejects on a live measurement; making the engine match the mark means the
## unclamped handle variant that same comment describes. Both are the one
## engine-side decision, and changing the drawn size here without it would add
## a third rule rather than remove the second.
const ICON_BASE_RADIUS := 5.5
const ICON_OUTLINE := Color(0.051, 0.043, 0.031, 0.9)

## **Generated landmarks are not manual icons, and must not read as them.**
##
## `design/landmark-generation/LANDMARK_UI_DESIGN.md` §9's row 23:
## `icon_place`/`icon_list` draw the *manual* Icon tool's stamps, and "a
## generated landmark is not one". They come from different places, mean
## different things, and one of them the user placed by hand — so they are drawn
## with a different mark, not a fifth colour in `ICON_FAMILY_COLORS`.
##
## Size carries CLASS (`LANDMARK_GENERATION_RESEARCH.md` §23's hierarchy:
## Continental is extremely rare and enormous, Local is common and small),
## because §23 says the hierarchy "should determine both generation frequency
## and map visibility" — this is the visibility half. Importance modulates
## within a class (§24: importance is emergent, not a rarity roll), so two
## regional landmarks are not identical dots.
##
## The mark itself is a **ring**, open rather than filled, for a reason worth
## stating: a filled mark competes with the settlement pins and the manual
## icons for the same visual weight, and a landmark is a place on the map
## rather than a thing on top of it. An open ring reads as an annotation of the
## terrain under it.
const LANDMARK_CLASS_RADIUS := {
	"continental": 9.0,
	"regional": 6.5,
	"local": 4.5,
	"cultural": 5.5,
}
## Cultural landmarks are the one class whose meaning is a civilisation's rather
## than the terrain's (§26: the same mountain is sacred to one culture and a
## border marker to another), so they carry the accent the rest of the shell
## uses for civ data, and the physical classes carry a cool neutral.
const LANDMARK_COL_PHYSICAL := Color(0.612, 0.769, 0.816, 0.95)
const LANDMARK_COL_CULTURAL := Color(0.878, 0.639, 0.290, 0.95)
const LANDMARK_OUTLINE := Color(0.051, 0.043, 0.031, 0.85)

## **Rejected candidates** -- `LARGE_ITEM_RULINGS.md`'s Landmark-funnel ruling,
## second half: "a rejected-candidate coordinate list plus a new overlay layer
## to draw it".
##
## The mark is the design's, not this file's invention.
## `design/landmark-generation/Main.dc.html` draws the viewport with two marks
## and a legend for each: a filled diamond for a placement, and for a rejection
## the **same diamond, smaller, stroked, dim, `stroke-dasharray="2 2"`** --
## legend "rejected candidate — inside a placed one's ring". `canvas.json`'s
## own `capquota` note says why it earns the space: "the viewport carries the
## fourth reading, for free … Seeing the rings overlap is the moment the concept
## lands, and it needs no vocabulary at all."
##
## A diamond, where a placement here is a **ring**: this file already diverged
## from the canvas's filled diamond for placements (see `LANDMARK_CLASS_RADIUS`
## above -- an open ring annotates the terrain instead of competing with the
## settlement pins), so diamond-versus-ring is what separates the two layers,
## and the dash then separates a rejection from anything else that might be
## drawn as one.
const LM_REJECT_RADIUS := 3.4
## Fraction of each diamond edge that is ink. The canvas's `2 2` dash on a
## ~7 px mark works out near half, and a dash needs a gap at each corner or the
## outline closes and stops reading as broken.
const LM_REJECT_DASH := 0.56
const LM_REJECT_WIDTH := 1.1

## One colour per rejection reason. The design shows one reason (spacing) and
## gives it `#5f6468`, a dim neutral; the other two are derived from this
## shell's own vocabulary rather than invented, per the standing rule for where
## no canvas exists.
##
## `cap` is the exception that matters. Those candidates passed every test and
## were turned away by the number alone -- `LandmarkFunnel::rejected_cap`'s own
## "the user got what they asked for" -- so they carry the accent the placed
## landmarks and the panel's own `at cap` rows already use, at low alpha. They
## are would-be placements, and colouring them the same dim grey as a spacing
## loss would say the opposite.
##
## `score` is unreachable at the shipped `SCORE_FLOOR` of `0.0` and is here so
## that the day a calibration pass raises the floor, the marks appear instead of
## vanishing into an unmatched key.
const LM_REJECT_COLORS := {
	"spacing": Color(0.373, 0.392, 0.408, 0.85),
	"cap": Color(0.878, 0.639, 0.290, 0.45),
	"score": Color(0.373, 0.392, 0.408, 0.55),
}
const LM_REJECT_FALLBACK := Color(0.373, 0.392, 0.408, 0.55)

## §4.5.5's Label tool. `color`/`font` are always the label's *effective*
## value (`label_dict` calls `color_or_default`/`font_or_default`), so no
## further fallback is needed here. `font` is a CSS font-family list (e.g.
## `"Georgia, serif"`) -- there is no web-font fallback chain in Godot, so
## the theme's own default font is used regardless of that string; only
## size/angle/arc/color are real per-label rendering.
const LABEL_STROKE_COLOR := Color(0.031, 0.024, 0.016, 0.8)
const LABEL_ZOOM_BASE_PX_PER_CELL := 2.0 ## tuning constant, see `_label_font_px`
const LABEL_FONT_PX_MIN := 8.0
const LABEL_FONT_PX_MAX := 96.0

## `drawArcLabel`'s three layout numbers. **`cartalith-civ/src/labels.rs` is
## the source of truth for all three** -- `ARC_STRAIGHT_THRESHOLD` (`:150`) and
## the two inside `arc_label_layout` (`:176`, the radius floor and the
## spread-over-1/2.2-of-a-circle term). They are duplicated here as named
## constants, not left as literals in `_draw_labels`, so that a change on the
## Rust side has one place to land on this one and `grep` finds the pair.
##
## Why they are duplicated at all rather than the layout being asked of
## `WorldGen.label_glyph_layout` (bound, wrapped by
## `EngineBridge.label_glyph_layout`, `engine_bridge.gd:2469 func
## label_glyph_layout`, and preferable in principle -- its doc warns that summing per-`char`
## advances instead of measuring the whole string drifts on a kerned font,
## which is exactly what the loop below does): this control is data-*pushed*.
## It holds no `EngineBridge` -- `ViewportHost.refresh_annotations()` hands it
## `label_list()` and nothing else -- so the call is not reachable from here
## without giving the overlay a bridge handle, which is `viewport_host.gd`'s
## decision and not this file's.
##
## The second reason is the one that would survive that: the binding sizes the
## label itself, from `labels.rs::label_font_size` (`grid_w / 512`, `civ_zoom_k`,
## floor 9), while this file sizes it from `_label_font_px` below (px-per-cell
## against `LABEL_ZOOM_BASE_PX_PER_CELL`, clamped to the two constants above,
## truncated to an int). Those are different numbers, and `size_px` is an input
## to the radius floor -- so swapping the loop for the binding would silently
## re-shape every arched label, not merely relocate the arithmetic. Reconciling
## the two font-size models is the real work, and it is not this row's.
##
## **What that unreconciled pair actually costs, added 2026-09-01.** The
## engine's number is not merely unused here -- it is what the user grabs.
## `label_box_at` derives its box from `label_font_size` (`side = max(meas_w,
## fsz * 1.3) * 1.25`), `label_handles` places the resize/rotate/arc handles
## on that box, and `cartography_workspace.gd` hit-tests and draws all three
## from it (`_handle_hit`, `_update_label_handles_overlay`). So the engine
## sizes the hit box and the handles against a label whose on-screen size this
## file computed differently -- the same shape of defect `ICON_BASE_RADIUS`
## and `_civ_zoom_k()` each carry a note about, and the same one fix: pick one
## model. Cheapest correct direction is to feed this file's own px-per-cell
## into `LabelViewEnv` so `label_font_size` reproduces `_label_font_px`, then
## read `fsz` off the binding and delete the local copy. That spans two crates
## and another pass's workspace file; recorded here rather than half-done.
const ARC_STRAIGHT_THRESHOLD := 0.01   ## `labels.rs:150`, the named constant there.
const ARC_RADIUS_FLOOR_K := 1.2        ## `labels.rs:176`, `size_px * 1.2`.
const ARC_SPREAD_DIVISOR := 2.2        ## `labels.rs:176`, `total_w / (2.2 * |a|)`.

## The reference's own halo, `ctx.lineWidth = max(1, sizePx * 0.16)` (ported as
## `labels.rs::arc_label_line_width` and golden-pinned there).
##
## **Still the fallback, and no longer the usual case.** Every row from
## `labels_render_list()` carries a `halo_em` from its label class's type spec
## (`LabelTypography`, `LARGE_ITEM_RULINGS.md`'s step 3), and that is what is
## used when present. This constant is what a row without one gets -- an older
## cdylib whose `label_list()` fallback is in play, per `engine_bridge.gd`'s own
## `labels_render_list()` degrade.
##
## The classes' figures are close to but not the same as this one: the
## settlement class, which every hand-placed label defaults to, is 1.5 px of
## halo on a 13 px glyph -- 0.115, against 0.16 here -- and carries 0.06 em of
## tracking where the reference has none. So a hand-placed label does render
## marginally differently once the class table reaches this file, deliberately:
## a class system that exempted the labels the user typed would leave the panel
## describing a typography half the map does not use. The user's own size, font
## and colour are untouched, which is the half the reference actually owns.
const LABEL_HALO_EM_FALLBACK := 0.16

## Synthetic oblique for the Water class, whose design spec is the only one that
## says `italic` (`parts.js:363`). Godot's theme carries one face and there is
## no italic sibling to switch to, so the glyphs are sheared, which is what a
## browser does for `font-style: oblique` when a family has no italic cut.
## `tan(12°)`, the conventional oblique angle.
const LABEL_ITALIC_SHEAR := 0.2126

## Emitted whenever the hovered settlement changes -- `null` on hover-exit.
## Lets `main.gd`'s Sample dock show the same data this overlay's own
## `_draw_hover_card` already draws on-canvas, without duplicating the
## hit-test logic in `_gui_input` below.
## `index` is the position in `get_settlements()`'s own array (`-1` on
## hover-exit) -- the same index `WorldGen.explain_settlement()` keys on, so
## the dock can ask why that settlement is there without re-running any hit
## test.
signal settlement_hovered(data: Variant, index: int)

## DCC shell milestone 1 (DCC_SHELL_SCOPE.md), click-to-pin (GUI_FEATURE_
## PARITY_SCOPE.md Category-1 item #10): emitted on a left click that hits
## a settlement, or on a click that hits nothing (`null`, `-1`) to unpin.
## Independent of `settlement_hovered` above -- the Properties dock holds
## this until the next click, unlike Sample's transient hover state.
signal settlement_selected(data: Variant, index: int)

## DCC shell milestone 1: emitted on every mouse motion with the grid-cell
## position under the cursor (`valid` false when the cursor is off the
## plate interior or nothing has been generated yet). Feeds the viewport's
## corner coordinate readout and the Sample dock -- real grid coordinates
## only, never a fabricated per-cell field (elevation/slope/biome) the
## engine doesn't expose per-cell yet.
signal cursor_sampled(gx: float, gy: float, valid: bool)

## §4.5's tool palette: the two primitives every armed tool needs, and
## nothing more specific than that -- a click-placement tool (Settlement,
## POI, Icon, Label, a Way/Route waypoint) wants `map_clicked`; a
## drag-painting tool (Biome paint, Territory, a Sculpt/Freehand stroke)
## wants `map_dragged`, which fires on every motion sample while the left
## button is held so a caller can accumulate a stroke or dab a brush
## continuously. Both fire in real grid-cell coordinates, the same values
## `cursor_sampled` already computes -- `_grid_point()` below is the one
## place that math lives now, shared by all three signals.
##
## This control stays tool-agnostic on purpose: it always emits both the
## selection signals above AND these two, on every click/drag, regardless of
## which tool is armed. The dispatch decision -- "is Inspect armed, so treat
## this as a selection, or is Settlement armed, so treat it as a placement"
## -- belongs to whoever owns the armed-tool state (`DccApp`), not here. That
## keeps this file ignorant of the tool system entirely, the same boundary
## `cursor_sampled`'s own doc comment already draws for per-cell fields.
signal map_clicked(gx: float, gy: float)
signal map_dragged(gx: float, gy: float)
signal map_released(gx: float, gy: float, valid: bool)   ## LMB release, ends a drag gesture.

## The reference's `contextmenu` handler on `view` (line 25888) ->
## `_civCtxShow` (25857). `PARITY_AUDIT.md` §5 item 2: no `MOUSE_BUTTON_RIGHT`
## handler existed anywhere under `godot-project/`, which left the reference's
## only path to Move-viewer-to and Delete-nearest-place with no counterpart.
##
## Stays as tool-agnostic as the three primitives above: this control reports
## "a right click landed at this grid cell, and the nearest settlement to it
## is `hit`", and nothing about what should happen next. The reference does
## its own nearest-place hit test inside the handler with the *same* radius
## `_civSelectPlaceAt` uses; here `_hit_test_settlement` already is that one
## shared definition, so the hit travels with the signal rather than being
## re-derived by whoever builds the menu.
##
## `screen_pos` is this control's local position, which is what a `PopupMenu`
## needs converted to global -- the receiver owns that conversion, since only
## it knows which window it is popping into.
signal map_right_clicked(gx: float, gy: float, hit: int, screen_pos: Vector2)

# ── Touch: press-and-hold IS the right click ─────────────────────────────────
#
# A finger has no second button, so on Android the context menu above had no
# route at all -- the one capability from the civ-interaction pass that phone
# could not reach by any path (`GUI_GAP_REGISTER.md`, and the phone canvas's
# own TARGETS rule says nothing about it because right click is not a phone
# gesture). Press-and-hold is the platform's answer, and this is where it
# belongs: the same control that owns the right click owns its touch twin, so
# `civilization_workspace.gd` receives one signal and never learns which
# pointer produced it.
#
# **The hard part is not the timer, it is the click that already fired.**
# `input_devices/pointing/emulate_mouse_from_touch` is on (project.godot), so a
# finger-down arrives here as a left `InputEventMouseButton` *immediately* --
# and `_gui_input`'s press branch emits `map_clicked` on press, which with the
# Settlement tool armed drops a settlement. Holding to open a menu would place
# a town first and then offer to edit a different one. So a touch press is
# **withheld** until the gesture says what it is:
#
#   drifts past `_TOUCH_SLOP`  -> it was a drag: release the press now, so the
#                                 sculpt/paint stroke starts from its real origin
#   lifts before the deadline  -> it was a tap: release the press, then release
#   reaches the deadline       -> it was a hold: discard the press entirely and
#                                 emit `map_right_clicked`; the lift is swallowed
#
# Driven off the *emulated mouse* stream rather than `InputEventScreenTouch`,
# deliberately: the emulated events are the ones this control is already known
# to receive, and `device < 0` (`InputEvent.DEVICE_ID_EMULATION`) is Godot's own
# marker for them, so a real mouse -- every desktop run -- takes none of this
# path and behaves exactly as it did.
#
# `OS.has_feature("mobile")` is ORed in as a second, coarser gate. Both were
# needed to get this working the first time and neither was individually
# provable on the handset without a build cycle per guess, so both stayed: the
# device id is the precise signal, the feature flag is the guarantee that an
# Android build takes this path even if a future engine stamps its emulated
# events differently. A physical mouse plugged into an Android device is the
# one case the flag over-claims, and it still resolves correctly -- the press
# is released on the lift or the first motion, one frame later than it would
# have been.
const _TOUCH_HOLD_MS := 500
## Physical pixels, not dp: this control is laid out in the main viewport,
## which carries no content scale. 28 px is ~10 dp on a 400 ppi handset, which
## is a finger's idle wobble and comfortably under the distance a deliberate
## drag covers in half a second.
const _TOUCH_SLOP := 28.0

var _touch_armed := false        ## a withheld press is outstanding
var _touch_swallow_up := false   ## the hold fired; the coming lift is not a release
var _touch_ms := 0
var _touch_pos := Vector2.ZERO
var _touch_press: Dictionary = {}

var _settlements: Array = []
var _roads: Array = []
var _sea_routes: Array = []
var _gw := 0
var _gh := 0
var _hover_index := -1
## Layer-granularity split (GUI_FEATURE_PARITY_SCOPE.md Category-1 item
## #9): the old shell had one checkbox hiding this whole control (and with
## it, hover input) -- these three flags let Settlements/Roads/Sea routes
## toggle independently while the control itself, and hover/click input on
## settlements, stay live regardless of which are on.
var _show_settlements := true
var _show_roads := true
var _show_sea_routes := true
## Per-class / per-way-type filters -- the reference's own
## `#explSettlementFilterList` and the by-way-type half of `#explShowRoads`
## (`design/Cartalith Menu Structure v2.dc.html`, MAP > LAYERS). Stored as
## *hidden* sets rather than shown sets so an empty dictionary means "show
## everything", which is what an untouched shell and a freshly-loaded world
## both are; a shown-set would have to be seeded from a settlement roster
## that does not exist until the first generate.
##
## Purely a draw filter: `_hit_test`/hover/click still see every settlement,
## the same independence `_show_settlements` above already keeps for the
## whole layer, because hiding a class is a cartographic choice and should
## not make a place unselectable.
var _hidden_settlement_kinds: Dictionary = {}
var _hidden_way_types: Dictionary = {}
## `state.viz.civWayScale` / `state.viz.wayOpacity` (`GUI_GAP_REGISTER.md`
## CA-16) -- see `set_way_scale()` / `set_way_opacity()`. Both at the
## reference's own default, where the layer draws exactly as it did before
## they existed: `1.0` is the multiplicative identity in both cases.
var _way_scale := 1.0
var _way_opacity := 1.0
## `GUI_GAP_REGISTER.md` **IN-13**'s map surface — per-way carried volume in
## `_roads` order, its own maximum (so the reading is relative to this world),
## and the switch. See `set_trade_load()`.
var _trade_load: PackedFloat32Array = PackedFloat32Array()
var _trade_load_max := 0.0
var _show_trade_load := false
## The busiest way draws at `1 + LOAD_WIDTH_GAIN` times its normal width.
## `1.6` is chosen against `WAY_STYLE`'s own range: a `track` at 2.6× is still
## thinner than an unloaded `highway`, so the layer re-ranks by traffic
## without ever making a track look like a trunk road.
const LOAD_WIDTH_GAIN := 1.6
## Whether `WAY_LOD_MIN`'s zoom ladder is applied (`GUI_GAP_REGISTER.md`
## CA-18). On, matching the reference, which has no switch for it at all --
## this one exists because a per-layer zoom range you cannot see the effect
## of is indistinguishable from a bug.
var _way_lod := true
## Plate-frame width as a fraction of the terrain texture's own width
## (`WorldGen.get_border_inset_frac()`, Phase 3 milestone 4). `0.0` when the
## renderer draws no frame, which makes every use of it below an exact no-op.
var _border_frac := 0.0

## The live camera zoom (`ViewportHost.zoom()`/`_zoom`), pushed in by
## `ViewportHost._zoom_at()`/`reset_view()` every time it changes --
## `PIN_SCALE_REF_PX`'s own doc comment covers why this is needed at all.
## Default `1.0` matches `ViewportHost`'s own default zoom, so a pin looks
## right even in the one frame before the first zoom/reset call ever runs.
var _camera_zoom := 1.0

func set_camera_zoom(z: float) -> void:
	if is_equal_approx(z, _camera_zoom):
		return
	_camera_zoom = z
	## Unlike `set_civ_data`/`set_show_settlements`, nothing about `_draw()`'s
	## own *content* changed here -- only how big the already-drawn geometry
	## needs to be next time, which only a fresh `_draw()` call can apply.
	## Godot does not re-run a `CanvasItem`'s draw commands just because an
	## ancestor's `scale` changed; it only rescales the cached ones, which is
	## exactly the bug `PIN_SCALE_REF_PX`'s own doc comment fixes -- without
	## this `queue_redraw()`, `_civ_zoom_k()` would compute the right radius
	## but never actually get drawn with it until some unrelated redraw
	## happened to fire.
	queue_redraw()

## The reference's own `_civZoomK()` (reference line 14980-14983), applied to
## `sc` so a settlement pin holds a roughly constant ON-SCREEN size across
## camera zoom -- see `PIN_SCALE_REF_PX`'s own doc comment for the full
## derivation of why this term is needed at all.
##
## **The reference's upper clamp is deliberately not ported (2026-08-24).**
## `_civZoomK` is `1/max(0.35,min(5,z))`, and the `min(5,…)` is free there
## because `z` is `viewT.scale`, which **stays at 1 under Tiled LOD** — the
## reference's deep zoom lives in `_lodZoom`, a different number entirely, so
## `viewT.scale` never approaches 5 at the zooms this clamp would bite at.
## Here `_camera_zoom` *is* the deep zoom. It clamped at `ZOOM_MAX = 8.0` when
## this was written (a harmless 1.6x pin overshoot at the very deepest view);
## it now runs to `lodMaxZoom()`, which is 160 on a default 800 km world, and
## the clamp turned every pin, glyph, name and label outline into 32x of
## magnified mush sitting exactly on top of the town it marks. Measured live
## (`_umreveal_shot.gd` at z=60: the pin covers the whole settlement).
##
## Dropping it restores what the comment above always claimed: the on-screen
## size is *exactly* constant, at every zoom in the range, which is what a map
## pin is. The `0.35` floor is the zoom-*out* half and is untouched — that one
## does the reference's "never dominate at extreme zoom-out" job, and this port
## reaches no zoom-out the reference does not.
##
## **The decision was taken here and only here** (recorded 2026-09-01). There
## are two more ports of `_civZoomK` in the workspace and both still clamp:
## `cartalith-civ/src/labels.rs`'s `civ_zoom_k` and
## `cartalith-assets/src/manual.rs`'s, each `1.0 / zoom_scale.clamp(0.35, 5.0)`.
## They are not dead: `label_box`/`label_handles` and `icon_box`/`icon_handles`
## are built on them, and `cartography_workspace.gd` hit-tests and draws the
## Label and Icon tools' on-canvas handles from exactly those values. So past
## `zoom = 5` this control and the engine size the same annotation by two
## different rules -- the handle a user grabs is placed by the clamped one and
## the mark they see is drawn by this one -- and the gap widens with every
## further zoom step.
##
## Restoring the clamp here is not the answer: it was removed against a live
## measurement (`_umreveal_shot.gd` at z=60), and this port's deep zoom really
## does run to `lodMaxZoom()` where the reference's never left 1. The fix is on
## the engine side -- an unclamped variant used for handle geometry while
## `civ_zoom_k` keeps the reference clamp for anything parity-pinned -- plus a
## test asserting the three implementations agree over the reachable range.
## That spans two crates and is not this pass's to make; it is written down
## here so the next reader of this function does not re-derive it.
func _civ_zoom_k() -> float:
	return 1.0 / maxf(_camera_zoom, 0.35)


## ── Rasterising at screen resolution, not control resolution ────────────────
##
## `_civ_zoom_k()` above fixes the *size* half of the camera-scale problem: a
## quantity multiplied by it comes out the same number of screen pixels at any
## zoom. It cannot fix the *resolution* half. `ViewportHost` scales this whole
## control (`_camera.scale`), and Godot does not re-run a `CanvasItem`'s draw
## commands when an ancestor's scale changes -- it rescales the geometry those
## commands already produced. A glyph is rasterised into the font atlas at the
## `font_size` passed to `draw_string`, in THIS control's own local pixels, and
## an antialiased `draw_polyline`/`draw_line` builds its feathered edge in the
## same local units. Magnify either by `_camera.scale` and you magnify the
## rasterisation with it: at zoom 8 a 9 px glyph is a 9 px bitmap stretched over
## 72 screen pixels, and a 1.5 px line's ~1 px antialiasing fringe becomes an
## 8 px translucent smear on each side. Both were reported live (2026-08-24,
## owner: settlement names "go blurry quickly", routes "slightly see-through
## and blurry") and both reproduced exactly at z=2/z=4/z=8.
##
## The fix is to generate the geometry at final screen resolution and then
## divide it back down, so the camera's own multiply lands on 1:1 pixels:
## inside `_crisp_begin()`/`_crisp_end()` every coordinate and every size is in
## **screen** pixels, and `_crisp_begin()` returns the `k` that converts this
## control's local pixels into them. A 12-screen-px label is rasterised at 12
## and drawn at 12, at every zoom in the range.
##
## Not applied to the pin discs themselves: they are a few pixels across, their
## softening is not what was reported, and `_draw()`'s settlement loop would
## have to enter and leave the transform per primitive. Deliberately scoped to
## the text and the linear layers, which is where the defect is visible.
func _crisp_begin() -> float:
	var k := maxf(_camera_zoom, 0.001)
	draw_set_transform(Vector2.ZERO, 0.0, Vector2(1.0 / k, 1.0 / k))
	return k


func _crisp_end() -> void:
	draw_set_transform(Vector2.ZERO, 0.0, Vector2.ONE)


## The camera zoom the *default* view sits at, which is what `SETTLEMENT_LOD`'s
## thresholds are calibrated against -- `1.0` there means "the zoom you get on
## opening a world", the same thing the reference's own `viewT.scale == 1` means
## for `CIV_LOD_PLACE`.
##
## `_camera_zoom` stopped being that number on 2026-08-23, when `ViewportHost.
## reset_view()` changed from plain fit (`_zoom = 1`) to the reference's **cover**
## scale (owner decision, recorded in that function). Cover is `max(1, size /
## displayed_rect_size)` and so is `>= 1` by construction and window-shaped: the
## same freshly-generated world opens at `z = 1.36` in one dock layout and above
## `1.4` in a wider one. Every threshold below `1.4` was therefore satisfied by
## the opening view alone, which is exactly the regression the owner reported --
## villages and 209 hamlets drawn full-size, with pins and names, on a map that
## had never been zoomed (reproduced live: `z=1.00 hamlet gated`, `z=2.00
## hamlet NOT gated`, and reset itself already at 1.357).
##
## Re-derived here from this control's own geometry rather than pushed in from
## `reset_view()`: same formula, no second copy of the state to go stale on a
## window resize, and no second file to edit. The ceiling is the one
## `reset_view()` clamps its own cover scale to -- `ViewportHost`'s zoom cap,
## which stopped being a flat `8.0` on 2026-08-24 when it became the
## reference's `lodMaxZoom()` (`max(64, ceil(kmW/5))`, never below 64). Only
## the *floor* of that is used here rather than the live per-world value: a
## cover scale is `max(w/rw, h/rh)` over a letterbox-fit rect and is a small
## number by construction, so no real window can reach even 64 -- reaching for
## the live cap would couple this file to a `ViewportHost` field for a bound
## that never binds.
func _lod_zoom_base() -> float:
	var rect := _displayed_rect()
	if rect.size.x <= 0.0 or rect.size.y <= 0.0 or size.x <= 0.0 or size.y <= 0.0:
		return 1.0
	return clampf(maxf(size.x / rect.size.x, size.y / rect.size.y),
		1.0, ViewportHost.ZOOM_MAX_FLOOR)

## §4.5.5's Icon and Label tools place these; `bridge.icon_list()`/
## `bridge.labels_render_list()` are this data's only source, both already bound
## (`icon_bridge.rs`/`label_bridge.rs`) and both wrapped in `engine_bridge.gd`.
## Set by `ViewportHost.refresh_annotations()`, a lighter call than the full
## `refresh()` -- placing one icon shouldn't re-fetch the terrain texture.
##
## **`_labels` is no longer only the hand-placed ones.** It is
## `labels_render_list()`: the generated labelling pass's output first, the
## Label tool's own labels over it, each row carrying its class's type spec
## (`class`, `halo_em`, `tracking_em`, `italic`) on top of `label_get`'s nine
## fields. A row's `generated` flag says which kind it is. Editing still goes
## exclusively through `label_list()`'s indices, which this array is not.
var _manual_icons: Array = []
var _labels: Array = []

## §4.5.4's Route tool. Each entry is one `route_get(i)` dictionary --
## `{points: PackedVector2Array, brks: PackedInt32Array, render_points,
## render_brks, km, mode, unreachable_legs, name}` -- so `brks` is honoured
## exactly the way `_sea_routes`'
## own breaks are: a break ends one stroke and starts the next rather than
## drawing a straight line across the gap. The draw pass uses the
## `render_*` pair (see it for why the two exist). Its own array rather than a third
## entry in `set_civ_data` because a committed route is not part of
## `get_roads()`/`get_sea_routes()` at all (`route_commit` stores into
## `InfraTools::routes`, a separate list from `InfraTools::ways` that
## `GUI_GAP_REGISTER.md` IN-02's fix appended to the two network getters).
var _manual_routes: Array = []

## The reference's `_civSelectedJourneyIdx` -- an index into `_manual_routes`,
## `-1` for none. Only ever read by the block-2b draw pass, and deliberately
## an index rather than a copy of the route: a delete renumbers the engine's
## list (`route_delete`), so anything that cached the route itself would draw
## a stale line the list no longer has a row for.
var _selected_manual_route := -1

## Generated landmarks, in `bridge.landmarks()`'s own dictionary shape:
## `{id, kind, class, x, y, elevation, score, importance, causal}`. Held as the
## bridge returned them rather than reshaped — every field the hover card or a
## future inspector wants is already there, and a reshape here would be a
## second place for the vocabulary to drift.
var _landmarks: Array = []
var _landmarks_visible := true

func set_landmarks(items: Array) -> void:
	_landmarks = items
	queue_redraw()

## The Layers popover's own on/off for this overlay. Separate from
## `_landmarks` being empty, which means "the pass has not run", so the map can
## say those two apart.
func set_landmarks_visible(on: bool) -> void:
	if on == _landmarks_visible:
		return
	_landmarks_visible = on
	queue_redraw()

## The rejected candidates of the last landmark pass, keyed by reason
## (`"spacing"` / `"cap"` / `"score"`), each a `PackedVector2Array` of grid
## cells -- `engine_bridge.gd::landmark_reject_points`'s own shape.
##
## Kept **packed and pre-grouped** rather than as the `landmark_rejects()`
## record list, which carries the same positions plus four more fields. That is
## the one decision in this layer worth stating: measured at the shipping
## 2048x1311 default, the record list is 3 216 dictionaries in 6.0 ms and this
## is the same 3 216 positions in 0.41 ms, because every `Dictionary` key write
## routes through gdext's `ensure_main_thread()` and a packed buffer is one
## allocation. This layer draws dots and needs nothing else; a surface that
## genuinely reads `score` or `needs_crowding` calls the record list instead.
var _landmark_rejects: Dictionary = {}
## **Off by default**, unlike `_landmarks_visible`. A placement is the result of
## a pass the user ran; a rejection is a diagnostic they have to ask for, and
## thousands of dim marks over every fresh world would be noise on a question
## nobody asked. `cartography_workspace.gd::LIVE_LAYERS` carries the same
## default so the popover and this file cannot disagree.
var _landmark_rejects_visible := false

func set_landmark_rejects(by_reason: Dictionary) -> void:
	_landmark_rejects = by_reason
	queue_redraw()

func set_landmark_rejects_visible(on: bool) -> void:
	if on == _landmark_rejects_visible:
		return
	_landmark_rejects_visible = on
	queue_redraw()

func set_manual_icons(icons: Array) -> void:
	_manual_icons = icons
	queue_redraw()

func set_labels(labels: Array) -> void:
	_labels = labels
	queue_redraw()

func set_manual_routes(routes: Array) -> void:
	_manual_routes = routes
	if _selected_manual_route >= routes.size():
		_selected_manual_route = -1
	queue_redraw()

## `-1` clears the selection. Out-of-range is clamped to `-1` rather than
## refused, since a delete legitimately leaves the caller holding an index
## that no longer exists.
func set_selected_manual_route(index: int) -> void:
	var idx := index if index >= 0 and index < _manual_routes.size() else -1
	if idx == _selected_manual_route:
		return
	_selected_manual_route = idx
	queue_redraw()


func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_STOP
	resized.connect(func(): queue_redraw())


## Called by `main.gd` right after a successful `generate()`/
## `generate_world_structure()`. `settlements` is `WorldGen.get_settlements()`'s
## `Array[Dictionary]`, `roads` is `get_roads()`'s `Array[Dictionary]` --
## each entry `{points: PackedVector2Array, brks: PackedInt32Array,
## way_type: String, name: String}` (Phase 2 milestone 14's own
## consolidated/smoothed/classified output, not raw grid-cell indices --
## `points` are already continuous full-resolution coordinates, drawn via
## `_point_to_screen`, distinct from `_cell_to_screen`'s settlement-marker
## `+0.5` cell-centering, which would be wrong here). `sea_routes` is
## `get_sea_routes()`'s `Array[Dictionary]` -- same `{points, brks, name}`
## shape minus `way_type` (Phase 2 milestone 13, `SeaRoute` has no
## highway/regional/road/track tier). Both also carry `km: float` and
## `manual: bool`; this control reads neither. `manual` marks a way the user
## drew with the Way tool (`GUI_GAP_REGISTER.md` IN-02) and is deliberately
## NOT consulted while drawing -- the reference keeps hand-drawn and
## generated ways in one array and styles both by `way_type` alone, so a
## hand-drawn `road` is meant to be indistinguishable from a generated one.
## It is there for lists and filters, not for this `_draw()`.
## Screen-space conversion happens every
## frame from the current control size, so this stays correct across
## window resizes without needing to be told again.
## `border_frac` is `WorldGen.get_border_inset_frac()` -- the plate frame the
## terrain raster itself now carries, as a fraction of texture width. Passed
## in rather than hardcoded here because `render.rs` owns that geometry (see
## `_interior_rect`); defaults to `0.0` so a caller that doesn't know about
## the frame simply gets the old, uninset behaviour.
func set_civ_data(settlements: Array, roads: Array, sea_routes: Array, gw: int, gh: int, border_frac: float = 0.0) -> void:
	_settlements = settlements
	_roads = roads
	_sea_routes = sea_routes
	_gw = gw
	_gh = gh
	_border_frac = border_frac
	_hover_index = -1
	## A settlement roster this control is handed afresh invalidates every
	## cached town layout by index -- see `clear_urban_layouts()`.
	clear_urban_layouts()
	queue_redraw()


## The engine's own faction swatches, index `faction - 1`, as pushed by
## `ViewportHost.refresh_faction_colors()` off `get_factions()`'s
## `color_r`/`color_g`/`color_b`. Empty until a world exists.
##
## Pushed rather than fetched, like every other array this control draws:
## `map_overlay.gd` is handed finished data and never calls the bridge
## itself (see `set_civ_data`'s own doc comment), so the one file that does
## hold a bridge handle hands these over with the rest.
var _faction_colors: Array[Color] = []

## Deliberately NOT cleared by `set_civ_data()`. A filtered settlement list
## (the Timeline's "Exist only" box) is pushed through that call several
## times a session with no faction change behind it, and wiping the swatches
## there would drop every pin back to the fallback palette until the next
## full refresh -- a flicker with no cause the user could see.
func set_faction_colors(colors: Array) -> void:
	var out: Array[Color] = []
	for c in colors:
		out.append(c as Color)
	_faction_colors = out
	queue_redraw()

## One faction's swatch. `0` is Unclaimed and has no faction colour at all;
## a `faction` past the pushed table falls back to the frozen six, which is
## the only case where the old `% size()` wrap survives -- and it is reached
## only before a world exists, since `get_factions()` enumerates every
## faction the roster has.
func _faction_color(faction: int) -> Color:
	if faction <= 0:
		return Color(0.5, 0.5, 0.5)
	if faction <= _faction_colors.size():
		return _faction_colors[faction - 1]
	return FACTION_COLORS[(faction - 1) % FACTION_COLORS.size()]

# -- Faith divergence (`RELIGION_DIFFUSION_SCOPE.md` §3 milestone 1) -----------
#
# **The one thing the map can say about religion that a panel cannot, and the
# reason this is not a religion wash.**
#
# `cartalith_civ::belief`'s own header records that the model has no stable
# mixture -- §14's `p^k` with `k > 1` is a fixation dynamic, so every
# settlement converges on one faith holding essentially all of it. And
# `belief_seed` starts every settlement wholly in its founding faction's
# state religion. Put together: at year 0 a per-settlement religion tint is a
# recolour of the faction wash `territory_view` already draws, and after
# diffusion it is *still* that wash everywhere the network has not moved
# anything. Painting it twice in two palettes is the "two pickers over one
# concept" shape `right_dock.gd`'s own layer-stack comment already names.
#
# What is genuinely new is the difference: a settlement whose plurality faith
# is no longer its ruler's. That is the paper's §27 emergent geography, it is
# invisible in the faction wash, and it is one bit per pin -- so it is drawn
# as a **broken ring**, a shape, not a hue. Nothing here relies on colour to
# carry meaning, and the pin's own faction colour keeps its one job.
#
# This control still computes nothing about the world (this file's own top
# comment): it compares two strings it was handed, the same kind of display
# test `_settlement_hidden` already makes.

## Per-faction state religion, index `faction - 1`, exactly like
## `_faction_colors` above -- `get_factions()`' own `religion` column, pushed
## by `civilization_workspace.gd`. Empty until pushed, which is why
## `_faith_diverged` below answers `false` rather than "diverged" for an
## unknown faction: an unpushed roster is a question never asked.
var _faction_religions := PackedStringArray()

## Off by default and **not** in `layer_visible()`'s match.
##
## Every arm of that match is a layer `viewport_host.gd::set_layer_visible()`
## can also write, and this one is not: its only writer is the CIVIL dock's
## own Religion category, calling this setter directly, because
## `viewport_host.gd` is not this pass's file. Registering a read-back for a
## layer the shared dispatcher cannot set would claim a wiring that does not
## exist -- see `faith_divergence_visible()`.
var _show_faith_divergence := false

func set_faction_religions(keys: PackedStringArray) -> void:
	_faction_religions = keys
	queue_redraw()

func set_faith_divergence_visible(on: bool) -> void:
	_show_faith_divergence = on
	queue_redraw()

func faith_divergence_visible() -> bool:
	return _show_faith_divergence

## Whether this settlement's plurality faith differs from its faction's state
## religion. **False in every state that is not a measured difference**, and
## each of those is a real state rather than a default:
##
## - no `religion` key -- `get_settlements()` omits it entirely until a
##   diffusion has been run (its own doc comment: omitted, not defaulted),
##   so there is nothing to differ from;
## - faction `0` (Unclaimed) or a faction past the pushed roster -- no state
##   religion exists to compare against;
## - an empty pushed key -- the roster row was not read, not "secular".
##
## `"none"` on **both** sides is agreement, not absence: `RELIGION_NONE` is
## the unaffiliated share of the population, and a secular town under a
## secular ruler has diverged from nothing.
func _faith_diverged(s: Dictionary) -> bool:
	if not _show_faith_divergence or not s.has("religion"):
		return false
	var faction := int(s.get("faction", 0))
	if faction <= 0 or faction > _faction_religions.size():
		return false
	var ruler := _faction_religions[faction - 1]
	if ruler.is_empty():
		return false
	return String(s["religion"]) != ruler

func set_show_settlements(shown: bool) -> void:
	_show_settlements = shown
	queue_redraw()


func set_show_roads(shown: bool) -> void:
	_show_roads = shown
	queue_redraw()


func set_show_sea_routes(shown: bool) -> void:
	_show_sea_routes = shown
	queue_redraw()

## The read-back half of the six `set_show_*`/`set_landmark*_visible` setters
## above -- `viewport_host.gd::layer_visible()`'s own doc comment says why this
## exists: a checkbox built once from a const default can only drift from
## whatever a second writer (`civilization_workspace.gd`'s landmark-funnel
## "Show rejected" chip, calling `set_layer_visible()` directly) last actually
## set here.
func layer_visible(layer: String) -> bool:
	match layer:
		"settlements": return _show_settlements
		"roads": return _show_roads
		"sea_routes": return _show_sea_routes
		"landmarks": return _landmarks_visible
		"landmark_rejects": return _landmark_rejects_visible
		"urban_layouts": return _show_urban_layouts
		_:
			push_error("MapOverlay: unknown layer '%s'" % layer)
			return true


## One settlement tier (`capital`/`city`/`town`/`village`/`hamlet` -- the
## engine's own five, `get_settlements()`'s `kind`). Independent of
## `set_show_settlements`, which gates the whole layer: turning the layer on
## restores whatever per-class state was last set, matching how the
## reference's own filter popover and its master Settlements checkbox behave.
func set_settlement_kind_visible(kind: String, shown: bool) -> void:
	if shown:
		_hidden_settlement_kinds.erase(kind)
	else:
		_hidden_settlement_kinds[kind] = true
	queue_redraw()


## One way type (`road`/`track`/`sea_lane`/`ancient` -- the engine's own four,
## `infra_tools_bridge::parse_way_type`). Sea lanes are drawn from
## `_sea_routes`, not `_roads`, so this filter only ever reaches the three
## land types in practice; it is keyed by the same `way_type` string
## `get_roads()` returns rather than a separate vocabulary.
func set_way_type_visible(way_type: String, shown: bool) -> void:
	if shown:
		_hidden_way_types.erase(way_type)
	else:
		_hidden_way_types[way_type] = true
	queue_redraw()


## `state.viz.civWayScale` (`#civWayScaleR`, reference line 1485) -- the user's
## own multiplier on every way, journey and route line width and on every dash
## length, `GUI_GAP_REGISTER.md` **CA-16**.
##
## The register recorded that "the reference's `#civWayScaleR` has no
## counterpart here -- so a width control would move nothing". This is that
## counterpart. It is the third term of the reference's own
## `rsc = max(1, GW/512) * _civZoomK() * _civWayScale()`; the first two are
## already in `_draw_way_segment`'s own doc comment (and the first is
## deliberately not taken, for the reason recorded there).
##
## The reference's slider is 0.20-2.50 in 0.05 steps; clamped to the same range
## here, because 0 is a hidden layer (which `set_show_roads` already is) and
## past 2.5 a highway is wider than a town.
func set_way_scale(k: float) -> void:
	_way_scale = clampf(k, 0.2, 2.5)
	queue_redraw()

func way_scale() -> float:
	return _way_scale


## `state.viz.wayOpacity` (`#wayOpacityR`, reference line 1491): one alpha
## multiplier over the whole way/journey/route layer, on top of each stroke's
## own authored alpha. The reference applies it as `globalAlpha` around each
## way's two strokes (line 15510); with no canvas-item alpha to set per stroke
## here, it multiplies each `Color`'s `a` instead, which is the same result for
## strokes that do not overlap themselves.
func set_way_opacity(a: float) -> void:
	_way_opacity = clampf(a, 0.0, 1.0)
	queue_redraw()

func way_opacity() -> float:
	return _way_opacity


## Whether the LOD ladder is applied at all (`GUI_GAP_REGISTER.md` CA-18).
func set_way_lod(on: bool) -> void:
	_way_lod = on
	queue_redraw()


## Trade load per way, in `set_civ_data`'s own `roads` order
## (`GUI_GAP_REGISTER.md` **IN-13**). Empty clears it.
##
## **Width, not hue, and that is the design.** Every faction swatch is
## already spent on the territory wash and on contested borders, and a way's
## own colour is its *type* (`WAY_STYLE`, RD-02) — a sixth colour ramp over
## the same pixels would be unreadable and would break the one thing a way's
## appearance currently tells you. So a busy way is drawn thicker in its own
## colour: the road still reads as a road.
##
## The multiplier is `1 + LOAD_WIDTH_GAIN * (load / max_load)`, which is a
## *relative* reading on purpose. An absolute scale would make every way on a
## small world hairline-thin and every way on a large one uniformly fat,
## because volume here is a population sum and populations are not comparable
## between worlds.
## `GUI_GAP_REGISTER.md` **RF-05**. `set_trade_load` is the *single* funnel for
## this data -- `infrastructure_workspace._match_trade_flows()` fills it and
## `app._refresh_world_dependent()` empties it -- so it is the one place that
## can tell the CARTO row whether there is anything to draw. Without this the
## row was RF-01 exactly: built at launch, disabled because `has_trade_load()`
## was false over an empty world, and never re-enabled by the match that made
## it true, because the match happens in a different workspace. Measured: after
## a real 624-flow match the toggle was still `disabled = true` while
## `has_trade_load()` returned true, and forcing it on moved 0.60 % of the map's
## pixels -- a working control that could not be reached.
signal trade_load_changed(available: bool)

func set_trade_load(loads: PackedFloat32Array) -> void:
	_trade_load = loads
	_trade_load_max = 0.0
	for v in loads:
		if v > _trade_load_max:
			_trade_load_max = v
	queue_redraw()
	trade_load_changed.emit(has_trade_load())

func set_show_trade_load(on: bool) -> void:
	_show_trade_load = on
	queue_redraw()

func show_trade_load() -> bool:
	return _show_trade_load

## Whether a load reading exists to draw at all — the CARTO row disables
## itself against this rather than offering a switch that does nothing.
func has_trade_load() -> bool:
	return _trade_load.size() > 0 and _trade_load_max > 0.0

## Width multiplier for one way, `1.0` when the layer is off, when no match
## has run, or when this way carries nothing.
func _trade_width_k(way_index: int) -> float:
	if not _show_trade_load or _trade_load_max <= 0.0:
		return 1.0
	if way_index < 0 or way_index >= _trade_load.size():
		return 1.0
	var v := _trade_load[way_index]
	if v <= 0.0:
		return 1.0
	return 1.0 + LOAD_WIDTH_GAIN * (v / _trade_load_max)

func way_lod() -> bool:
	return _way_lod


## Reproduces `%MapView`'s own `STRETCH_KEEP_ASPECT_CENTERED` fit math so
## grid-cell coordinates map onto the same pixels the terrain texture
## actually occupies (this control's `size` is identical to `%MapView`'s,
## both being plain siblings under one `MarginContainer` -- see this
## script's own top comment). Returns `Rect2(origin, displayed_size)`, or
## a zero rect if there's nothing to fit yet.
func _displayed_rect() -> Rect2:
	if _gw <= 0 or _gh <= 0 or size.x <= 0.0 or size.y <= 0.0:
		return Rect2()
	var scale := minf(size.x / float(_gw), size.y / float(_gh))
	var displayed_size := Vector2(_gw, _gh) * scale
	var origin := (size - displayed_size) * 0.5
	return Rect2(origin, displayed_size)

## Public alias of the above -- `ViewportHost`'s tool overlays (the Measure
## ruler, the Region marquee) need the exact same fit rect this control's own
## drawing already computes, and duplicating the letterbox math a second place
## would be the kind of drift `js_hypot`-style bugs come from. Grid-to-screen,
## not the reverse -- see `_grid_point()` for the inverse.
func displayed_rect() -> Rect2:
	return _displayed_rect()


## The plate *interior*: `_displayed_rect()` minus the frame the terrain
## raster draws over its own outermost cells (paper margin + thick and thin
## neatlines, `render.rs`'s `apply_border`, Phase 3 milestone 4). Everything
## outside this is bare paper with no map under it at all -- the terrain
## there is covered, not shown -- so nothing drawn from world data belongs
## on it.
##
## The frame is inset by the same number of *cells* on all four sides and
## `_displayed_rect()`'s fit scale is uniform, so one pixel inset serves both
## axes. `_border_frac` is a fraction of texture width rather than a cell
## count precisely so this needs no resolution knowledge.
func _interior_rect(rect: Rect2) -> Rect2:
	if _border_frac <= 0.0:
		return rect
	var inset := minf(_border_frac * rect.size.x, minf(rect.size.x, rect.size.y) * 0.45)
	return rect.grow(-inset)


func _cell_to_screen(cell: Vector2, rect: Rect2) -> Vector2:
	return rect.position + Vector2((cell.x + 0.5) / _gw, (cell.y + 0.5) / _gh) * rect.size


## Roads' own `points` are already continuous full-resolution coordinates
## (Catmull-Rom-smoothed, not raw cell indices) -- no `+0.5` centering,
## unlike `_cell_to_screen`'s settlement markers.
func _point_to_screen(p: Vector2, rect: Rect2) -> Vector2:
	return rect.position + Vector2(p.x / _gw, p.y / _gh) * rect.size


## True below `kind`'s own `SETTLEMENT_LOD` threshold -- the camera zoom, not
## `_civ_zoom_k()`'s clamped/inverted screen-size compensation, matching the
## reference's own `zoom<lodMin` test (`zoom` there is `_civZoomRaw()`, the
## un-clamped `viewT.scale`). An unrecognised kind defaults to `0.5`
## (town/village straddle), same fallback `CIV_LOD_PLACE[p.kind]!=null?...:0.5`
## uses in the reference.
## `_camera_zoom` normalised by `_lod_zoom_base()` -- see that function for why
## the raw camera scale stopped being the right number to compare.
func _settlement_below_lod(kind: String) -> bool:
	return (_camera_zoom / _lod_zoom_base()) < float(SETTLEMENT_LOD.get(kind, 0.5))


## True for an addon village still below `VILLAGE_ADDON_LOD` -- drawn as
## nothing at all, and (per the reference's `_civPlacePickVisible`, which
## excludes a still-hidden addon from picking) not hit-testable either, so a
## click cannot select a place that is not on the map.
func _settlement_hidden(s: Dictionary) -> bool:
	if s["kind"] != "hamlet" or int(s.get("population", 1)) != VILLAGE_ADDON_POP:
		return false
	return (_camera_zoom / _lod_zoom_base()) < VILLAGE_ADDON_LOD


## `(4+klass.rank)*sc` -- the reference's own settlement-pin size formula
## (`_civDrawSettlementPin`, line 15166), `sc` per `PIN_SCALE_REF_PX`'s own
## doc comment above. Shared by `_draw()`'s settlement loop and `_hit_test_
## settlement` so the drawn pin and its click/hover target never disagree.
## Below the tier's own zoom threshold this instead returns the small
## fallback-dot radius (`LOD_DOT_RADIUS_SC`) -- the pin `_draw()` actually
## renders at low zoom is the dot, so the hit target must shrink with it,
## the same "drawn size is picked size" invariant this function already
## keeps for the full pin.
func _settlement_pin_radius(kind: String, rect: Rect2) -> float:
	var sc: float = (rect.size.x / PIN_SCALE_REF_PX) * _civ_zoom_k()
	if _settlement_below_lod(kind):
		return LOD_DOT_RADIUS_SC * sc
	var klass: Dictionary = SETTLEMENT_CLASS.get(kind, SETTLEMENT_CLASS["town"])
	return (4.0 + float(klass["rank"])) * sc


## Reference lines 15716-15721 (`lblCandidates`): the four label positions a
## settlement name is tried at, above -> below -> right -> left, as screen-
## space boxes centred on the pin. `gap` matches the reference's own `2*sc`
## clearance between the pin edge and the label box.
func _settlement_label_candidates(pos: Vector2, radius: float, sc: float, w: float, h: float) -> Array[Rect2]:
	var gap := 2.0 * sc
	return [
		Rect2(pos - Vector2(w / 2.0, radius + gap + h), Vector2(w, h)),  # above
		Rect2(pos + Vector2(-w / 2.0, radius + gap), Vector2(w, h)),     # below
		Rect2(pos + Vector2(radius + gap, -h / 2.0), Vector2(w, h)),     # right
		Rect2(pos - Vector2(radius + gap + w, h / 2.0), Vector2(w, h)),  # left
	]


## Pre-seeds this frame's label-occupancy set with every manual icon's and
## user-authored label's own approximate footprint, so a settlement's
## auto-placed name never overlaps annotation the user placed deliberately
## -- the same reasoning the reference reserves region-label boxes before
## its own settlement loop runs for (line 15722-15728: "user-placed region
## names are DELIBERATE cartography -- they must not be silently suppressed
## by an auto-placed settlement label"). Approximate rather than exact for
## icons (whose real footprint depends on family/shape, drawn in
## `_draw_manual_icons`, not recomputed here) -- close enough to keep a
## label from visibly overlapping an icon, which is all this simplified
## system claims to do; see `_draw()`'s own settlement-loop comment for the
## full scope of the simplification against the reference's real occupancy
## grid (`_civLblOcc`).
##
## **The generated labelling pass rides this for free**, because `_labels` is
## now `labels_render_list()` rather than `label_list()`: a generated continent
## or lake name reserves its box here exactly as a hand-placed one does, so a
## settlement pin's auto-placed name steps around it. That is *not* the label
## collision culler `LARGE_ITEM_RULINGS.md` sequences behind the pass -- this
## only ever moves a settlement pin's own name and never suppresses a label,
## and generated labels are still never measured against each other.
func _seed_label_occupancy(rect: Rect2) -> Array[Rect2]:
	var boxes: Array[Rect2] = []
	var font := get_theme_default_font()
	for lb: Dictionary in _labels:
		var text: String = lb["text"]
		if text.is_empty():
			continue
		var pos := _point_to_screen(Vector2(lb["x"], lb["y"]), rect)
		var font_px := _label_font_px(lb, rect)
		var w := font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, font_px).x
		var h := float(font_px) * 1.3
		boxes.append(Rect2(pos - Vector2(w, h) / 2.0, Vector2(w, h)))
	for ic: Dictionary in _manual_icons:
		var pos2 := _point_to_screen(Vector2(ic["x"], ic["y"]), rect)
		var r: float = ICON_BASE_RADIUS * maxf(0.2, float(ic["scale"]))
		boxes.append(Rect2(pos2 - Vector2(r, r), Vector2(r, r) * 2.0))
	return boxes


func _draw() -> void:
	if (_settlements.is_empty() and _roads.is_empty() and _sea_routes.is_empty()
			and _manual_icons.is_empty() and _labels.is_empty()
			and _manual_routes.is_empty() and _landmarks.is_empty()
			## The rejects layer can be the ONLY thing on this control: a pass
			## that placed nothing still rejects, and that is exactly the world
			## where the diagnostic matters most. Leaving it out of this guard
			## would make the layer silently undrawable on the one map worth
			## drawing it on.
			and _landmark_rejects.is_empty()):
		return
	var rect := _displayed_rect()
	if rect.size.x <= 0.0:
		return
	var interior := _interior_rect(rect)
	## Once per frame, not once per way: the camera cannot move inside one
	## `_draw()`. See `_run_offscreen()`.
	_visible_local = _visible_local_rect()

	# Linear features (roads, sea lanes) are *clipped* at the neatline: a
	# road that runs off the plate genuinely continues past the sheet edge,
	# and cutting it there is what an atlas plate does. Point symbols are
	# handled the opposite way below -- placed or not placed, never sliced.
	#
	# One scissor rect for the whole canvas item rather than hand-clipping
	# four different primitive types. `Control` re-sets both of these from
	# its own rect on every `NOTIFICATION_DRAW`, which fires immediately
	# before `_draw()`, so this override lasts exactly one frame and needs
	# no restore.
	if _border_frac > 0.0:
		var ci := get_canvas_item()
		RenderingServer.canvas_item_set_custom_rect(ci, true, interior)
		RenderingServer.canvas_item_set_clip(ci, true)

	if _show_sea_routes:
		for route: Dictionary in _sea_routes:
			var points: PackedVector2Array = route["points"]
			if points.size() < 2:
				continue
			var brks: PackedInt32Array = route["brks"]
			var start := 0
			for cut in brks:
				_draw_sea_route_segment(points, start, cut, rect)
				start = cut
			_draw_sea_route_segment(points, start, points.size(), rect)

	if _show_roads:
		## Indexed, not `for way in _roads`: IN-13's trade load is keyed to a
		## way's position in this same array (`get_roads()` order), so the
		## index has to survive into the stroke.
		for wi in _roads.size():
			var way: Dictionary = _roads[wi]
			var points: PackedVector2Array = way["points"]
			if points.size() < 2:
				continue
			if _hidden_way_types.has(way["way_type"]):
				continue
			## `_civWayLodMin` (reference 15012) + `if(zoom<lodMin) return`
			## (15501) -- CA-18's ladder. See `WAY_LOD_MIN`.
			if _way_lod and _camera_zoom < float(WAY_LOD_MIN.get(way["way_type"], WAY_LOD_DEFAULT)):
				continue
			var style: Dictionary = WAY_STYLE.get(way["way_type"], WAY_STYLE[WAY_STYLE_DEFAULT])
			var load_k := _trade_width_k(wi)
			var brks: PackedInt32Array = way["brks"]
			# `brks` marks indices where this way's own path has a real gap
			# (two disjoint consolidated runs sharing one `Way`) -- draw each
			# run between breaks as its own stroke, not one polyline straight
			# through the gap.
			var start2 := 0
			for cut in brks:
				_draw_way_segment(points, start2, cut, rect, style, load_k)
				start2 = cut
			_draw_way_segment(points, start2, points.size(), rect, style, load_k)

	## Committed Route-tool routes, drawn after both network layers so a route
	## that runs along an existing road is still visible on top of it. Shares
	## the "Ways & routes" visibility toggle (`set_show_roads`) because that is
	## the layer row the CARTO dock actually labels "Ways & routes" -- there is
	## no separate routes checkbox to gate against.
	if _show_roads:
		for ri in _manual_routes.size():
			var r: Dictionary = _manual_routes[ri]
			## `render_points`, not `points`: a route's `points` are the
			## engine's own list, kept 1:1 with what `jp_compute` planned
			## over because `plan.stages[i].{i0,i1}` index into it.
			## `render_points` is the same curve re-sampled at render
			## density (`route_get`'s own doc comment) -- the fallback is
			## for an older GDExtension binary that predates the key.
			var rpts: PackedVector2Array = r.get("render_points", r.get("points", PackedVector2Array()))
			if rpts.size() < 2:
				continue
			var rsel := ri == _selected_manual_route
			var rbrks: PackedInt32Array = r.get("render_brks", r.get("brks", PackedInt32Array()))
			var rstart := 0
			for cut in rbrks:
				_draw_manual_route_segment(rpts, rstart, cut, rect, rsel)
				rstart = cut
			_draw_manual_route_segment(rpts, rstart, rpts.size(), rect, rsel)

	## Town layouts sit above the ways -- a town's own high street IS the
	## through-road, so it must overlay it -- and *replace* the pin of every
	## place they actually draw (`_urban_revealed`, the reference's
	## `_umRevealedSet`). See this file's "Urban layouts" block at the foot
	## for the reveal gate and why it is not `_umLayoutAlpha`'s km band.
	_urban_revealed.clear()
	if _show_urban_layouts:
		_draw_urban_layouts(rect, interior)

	if _show_settlements:
		## Same formula `_settlement_pin_radius()` uses -- kept as one inline
		## `sc` here (rather than calling that function per-settlement) since
		## `_draw()`'s loop already reuses `sc` for the glyph and label sizing
		## below it too, exactly like the reference's own `sc` feeds icon,
		## way and label sizing from one shared value (reference line 15165).
		var sc: float = (rect.size.x / PIN_SCALE_REF_PX) * _civ_zoom_k()
		## Local px -> screen px for this frame's text, hoisted out of the loop
		## because it cannot change inside one `_draw()`. See `_crisp_begin()`.
		var k := maxf(_camera_zoom, 0.001)
		var font := get_theme_default_font()
		# Trait badges (§4.5.3's own reference behaviour, `_civDrawTraitBadges`,
		# reference line 15101) are a disclosed gap, not an oversight: `get_
		# settlements()` (`lib.rs`) emits {x, y, name, population, kind,
		# faction, capital, coastal} only -- no `traits` field -- and nothing
		# in `cartalith-civ`'s own `NamedSettlement` models a trait list at
		# all (grepped: the string "traits" appears exactly once in that
		# crate, in a doc comment quoting the REFERENCE's own JS object
		# shape). There is no data to draw a badge from, so none are drawn.
		#
		# Auto-label placement below is a deliberately simplified stand-in
		# for the reference's real system (`_civLblOcc`, an occupancy GRID
		# tested/marked per label-sized bucket, reference lines 15668-15781):
		# a plain per-frame `Array[Rect2]` of already-placed boxes, tested by
		# `Rect2.intersects` rather than a spatial grid. Fine at settlement-
		# roster scale (dozens to a few hundred -- an occupancy grid exists to
		# make THOUSANDS cheap, which no generated world here produces), and
		# it reproduces the essential behaviour: higher tiers are drawn (and
		# so win label placement) first, the same four candidate positions in
		# the same above/below/right/left order, and a label that fits
		# nowhere is dropped -- but its pin is always still drawn (once past
		# its own `SETTLEMENT_LOD` threshold -- see the dot-fallback branch
		# below). This is a different LOD from `LOD_TILING_INTEGRATION_SCOPE
		# .md` milestone M1 (that one raster-tiles the TERRAIN at deep zoom;
		# this one is `CIV_LOD_PLACE`, the reference's own zoom-gated
		# settlement-pin importance tiering, owner-requested 2026-08-19).
		var occupied: Array[Rect2] = _seed_label_occupancy(rect)
		var draw_order := range(_settlements.size())
		draw_order.sort_custom(func(a, b):
			var ra: int = SETTLEMENT_CLASS.get(_settlements[a]["kind"], SETTLEMENT_CLASS["town"])["rank"]
			var rb: int = SETTLEMENT_CLASS.get(_settlements[b]["kind"], SETTLEMENT_CLASS["town"])["rank"]
			return ra > rb)

		for i in draw_order:
			var s: Dictionary = _settlements[i]
			## Per-class filter, tested before any geometry so a hidden tier
			## costs nothing and, more importantly, never reserves label
			## occupancy that a *visible* place would then be pushed out of.
			if _hidden_settlement_kinds.has(s["kind"]):
				continue
			## Tested in the same place and for the same reason as the class
			## filter above: an addon village below its own threshold draws
			## nothing, so it must not reserve label occupancy either.
			if _settlement_hidden(s):
				continue
			## The reference's `_umRevealedSet` (line 22753): a place whose own
			## generated layout was drawn *fully opaque* this frame gives up
			## its pin to it. Only at full opacity: the km band is live again as
			## of 2026-08-24, and `_draw_urban_layouts()`'s own note at the
			## handover says why the crossfade ends here rather than fading the
			## pin through it. Without it the pin -- sized to hold constant -- sits
			## squarely over the market anchor and the densest streets, which
			## is exactly what it is drawn on top of.
			if _urban_revealed.has(i):
				continue
			var pos := _cell_to_screen(Vector2(s["x"], s["y"]), rect)
			# A settlement whose cell is under the frame has no visible terrain
			# beneath it at all, so a marker there points at nothing -- it is off
			# the plate, and off-plate detail is omitted rather than trimmed to a
			# half-disc against the neatline. The clip above then trims the one
			# remaining case: a settlement just *inside* the interior whose
			# radius overhangs it (the actual defect this fixes -- markers
			# landing partly on the margin, seen in both test worlds).
			if not interior.has_point(pos):
				continue
			var faction: int = s["faction"]
			var color: Color = _faction_color(faction)
			var kind: String = s["kind"]

			# `CIV_LOD_PLACE` (reference line 15373, see `SETTLEMENT_LOD`'s own
			# doc comment above for the full derivation and real-world-mapping
			# citations): below this tier's own zoom threshold, draw a small
			# faction-tinted dot instead of the full pin -- never hide the
			# place outright (a road still needs a visible anchor at any
			# zoom), but keep a low-zoom view from drowning in city-sized
			# hamlet icons and labels. No glyph, no name label, no hover-
			# radius bump, no capital ring -- all reference behaviour for the
			# dot branch (reference lines 15747-15756 draw nothing else for
			# it either).
			if _settlement_below_lod(kind):
				var dot_r: float = LOD_DOT_RADIUS_SC * sc
				draw_circle(pos, dot_r + LOD_DOT_OUTLINE_SC * sc, Color(0.047, 0.039, 0.027, 0.65), true, -1.0, true)
				draw_circle(pos, dot_r, color, true, -1.0, true)
				continue

			var klass: Dictionary = SETTLEMENT_CLASS.get(kind, SETTLEMENT_CLASS["town"])
			var radius: float = (4.0 + float(klass["rank"])) * sc
			if i == _hover_index:
				radius += 1.5

			## `antialiased` (the 6th positional arg) defaults to `false` in
			## Godot 4 -- left implicit here would draw a visibly jagged
			## circle, worse the more a pin is magnified by camera zoom
			## (`PIN_SCALE_REF_PX`'s own doc comment covers the zoom side of
			## "not sharp"; this is the antialiasing side). `filled=true,
			## width=-1.0` spelled out are just `draw_circle`'s own defaults,
			## kept explicit because GDScript has no keyword-argument syntax
			## to set only the trailing one.
			##
			## Shadow first (drawn behind, offset straight down) so the fill
			## and outline composite over it exactly like the reference's own
			## layering, just with one extra pass underneath.
			draw_circle(pos + Vector2(0, PIN_SHADOW_OFFSET_SC * sc), radius, PIN_SHADOW_COLOR, true, -1.0, true)
			draw_circle(pos, radius, color, true, -1.0, true)
			draw_arc(pos, radius, 0, TAU, 24, MARKER_OUTLINE, 1.2, true)
			if s["capital"]:
				draw_arc(pos, radius + CAPITAL_RING_WIDTH, 0, TAU, 28, color, CAPITAL_RING_WIDTH, true)
			## Outside the capital ring, so a diverged capital shows both.
			## Under the coastal badge and the glyph for the same reason the
			## landmark rejects draw under the placements: this is context
			## about the pin, and must not win a pixel from the pin itself.
			if _faith_diverged(s):
				var ring_r: float = radius + CAPITAL_RING_WIDTH + FAITH_RING_PAD_SC * sc
				for a in FAITH_RING_ARCS:
					var from: float = TAU * float(a) / float(FAITH_RING_ARCS)
					draw_arc(pos, ring_r, from, from + FAITH_RING_SPAN, 10,
						FAITH_RING_SHADOW, FAITH_RING_WIDTH + 1.2, true)
					draw_arc(pos, ring_r, from, from + FAITH_RING_SPAN, 10,
						FAITH_RING_COLOR, FAITH_RING_WIDTH, true)
			if s.get("coastal", false):
				var badge_r: float = COASTAL_BADGE_R_SC * sc
				var badge_pos := pos + Vector2(radius, radius) * 0.62
				draw_circle(badge_pos, badge_r + 0.5, COASTAL_BADGE_OUTLINE, true, -1.0, true)
				draw_circle(badge_pos, badge_r, COASTAL_BADGE_COLOR, true, -1.0, true)

			# Glyph (reference: `ctx.fillText(klass.glyph,px,py)`, line 15178-
			# 15180) -- the one per-tier visual distinguisher beyond size,
			# centred on the pin exactly like `_draw_labels`' own straight-text
			# path centres a region label (`v_center` trick, same reasoning).
			# Godot's built-in theme font may not carry every one of these five
			# dingbat glyphs (⌂●◉⬣✦) -- a missing one falls back to whatever
			# Godot's own tofu/replacement glyph is, same disclosed limitation
			# `_draw_labels`' own doc comment already accepts for `font` (no
			# web-font fallback chain exists in Godot either).
			#
			# Sized and rasterised in SCREEN pixels (`_crisp_begin()`): the
			# reference's `max(9, ...)` floor -- and the `8` here -- are canvas
			# pixels at `viewT.scale == 1`, i.e. on-screen pixels, and applying
			# them in this control's local space instead is what made both the
			# glyph and the name below grow linearly with camera zoom while the
			# pin under them correctly held still. `radius`/`sc` are local, so
			# they convert with `* k`.
			var glyph: String = klass["glyph"]
			_crisp_begin()
			var glyph_px: int = maxi(8, int((radius + 2.0 * sc) * k))
			var glyph_w := font.get_string_size(glyph, HORIZONTAL_ALIGNMENT_LEFT, -1, glyph_px).x
			var glyph_v_center: float = (font.get_ascent(glyph_px) - font.get_descent(glyph_px)) / 2.0
			draw_string(font, pos * k + Vector2(-glyph_w / 2.0, glyph_v_center), glyph,
				HORIZONTAL_ALIGNMENT_LEFT, -1, glyph_px, Color.WHITE)
			_crisp_end()

			# Auto-placed name label -- see this block's own top comment for
			# the simplified-occupancy-set reasoning.
			var name: String = s.get("name", "")
			if not name.is_empty():
				# `label_px` and the measured `lw`/`lh` are screen pixels (see
				# the glyph's own note above); the candidate boxes and the
				# occupancy set stay in this control's local space, so both
				# come back through `/ k`.
				var label_px: int = maxi(9, int((radius + sc) * k))
				var lw := font.get_string_size(name, HORIZONTAL_ALIGNMENT_LEFT, -1, label_px).x / k
				var lh := float(label_px) * 1.3 / k
				for box in _settlement_label_candidates(pos, radius, sc, lw, lh):
					var fits := true
					for occ in occupied:
						if occ.intersects(box):
							fits = false
							break
					if not fits:
						continue
					occupied.append(box)
					var v_center: float = (font.get_ascent(label_px) - font.get_descent(label_px)) / 2.0
					var draw_pos := Vector2(box.position.x, box.position.y + box.size.y / 2.0) * k + Vector2(0.0, v_center)
					var outline_w: int = maxi(1, int(2.5 * sc * k))
					_crisp_begin()
					draw_string_outline(font, draw_pos, name, HORIZONTAL_ALIGNMENT_LEFT, -1, label_px, outline_w, LABEL_STROKE_COLOR)
					draw_string(font, draw_pos, name, HORIZONTAL_ALIGNMENT_LEFT, -1, label_px, SETTLEMENT_LABEL_FILL)
					_crisp_end()
					break

		if _hover_index >= 0 and _hover_index < _settlements.size():
			_draw_hover_card(_settlements[_hover_index], rect, interior)

	# Manual annotations (§4.5.5) are independent of the Settlements/Roads/Sea
	# routes toggles above -- they have no layer-visibility flag of their own
	# in `DCC_SHELL_SPEC.md`, so they always draw once placed, same as the
	# Measure/Region tool overlays in `tool_overlay.gd` always draw once armed.
	_draw_manual_icons(rect, interior)
	## Under the labels and over everything else. Labels are text and lose
	## legibility the moment anything crosses them; a landmark ring is a mark
	## and does not.
	##
	## Rejections draw UNDER the placements, deliberately: the layer's whole
	## claim is that a rejected candidate lost to a placed one, so the placed
	## mark must win the pixel wherever they coincide.
	_draw_landmark_rejects(rect, interior)
	_draw_landmarks(rect, interior)
	_draw_labels(rect, interior)


## §4.5.5's Icon tool: placed markers, by `family` (`icon_dict`'s
## `{x, y, family, slot, set, scale}`). Positions are continuous
## full-resolution coordinates (a placement click's own `gx, gy`, not a
## cell index) -- `_point_to_screen`, not `_cell_to_screen`, matching roads'
## own reasoning in this file's `set_civ_data` doc comment.
func _draw_manual_icons(rect: Rect2, interior: Rect2) -> void:
	for ic: Dictionary in _manual_icons:
		var pos := _point_to_screen(Vector2(ic["x"], ic["y"]), rect)
		if not interior.has_point(pos):
			continue
		var color: Color = ICON_FAMILY_COLORS.get(ic["family"], Color(0.7, 0.7, 0.7))
		var r: float = ICON_BASE_RADIUS * maxf(0.2, float(ic["scale"]))
		match ic["family"]:
			"settlement":
				var half := r * 0.85
				draw_rect(Rect2(pos - Vector2(half, half), Vector2(half, half) * 2.0), color, true)
				draw_rect(Rect2(pos - Vector2(half, half), Vector2(half, half) * 2.0), ICON_OUTLINE, false, 1.2)
			"feature":
				var pts := PackedVector2Array([
					pos + Vector2(0, -r), pos + Vector2(r * 0.87, r * 0.5), pos + Vector2(-r * 0.87, r * 0.5)])
				draw_colored_polygon(pts, color)
				draw_polyline(PackedVector2Array([pts[0], pts[1], pts[2], pts[0]]), ICON_OUTLINE, 1.2, true)
			"poi":
				var pts2 := PackedVector2Array([
					pos + Vector2(0, -r), pos + Vector2(r, 0), pos + Vector2(0, r), pos + Vector2(-r, 0)])
				draw_colored_polygon(pts2, color)
				draw_polyline(PackedVector2Array([pts2[0], pts2[1], pts2[2], pts2[3], pts2[0]]), ICON_OUTLINE, 1.2, true)
			_: ## "custom", or any future family this build doesn't recognise yet.
				draw_circle(pos, r, color, true, -1.0, true)   ## See the settlement pin's own antialiasing comment above.
				draw_arc(pos, r, 0, TAU, 20, ICON_OUTLINE, 1.2, true)
				draw_arc(pos, r * 0.4, 0, TAU, 12, ICON_OUTLINE, 1.0, true)


## Generated landmarks — `LANDMARK_GENERATION_RESEARCH.md` §23's four classes,
## drawn as open rings sized by class and modulated by importance.
##
## Positions are grid CELLS (`Landmark.x`/`.y` are `usize` cell indices, unlike
## the manual Icon tool's continuous click coordinates), so this is
## `_cell_to_screen`, not `_point_to_screen`. Getting that wrong puts every
## landmark half a cell out at every zoom, which is invisible at fit and
## obvious at deep zoom — the same distinction `set_civ_data`'s own doc comment
## draws for roads.
##
## An unknown class falls through to the Local radius rather than being skipped:
## a landmark the engine placed and this build cannot categorise is still a real
## landmark, and dropping it would be this file quietly disagreeing with the
## panel about how many exist.
func _draw_landmarks(rect: Rect2, interior: Rect2) -> void:
	if not _landmarks_visible or _landmarks.is_empty():
		return
	for lm: Dictionary in _landmarks:
		var pos := _cell_to_screen(Vector2(float(lm.get("x", 0)), float(lm.get("y", 0))), rect)
		if not interior.has_point(pos):
			continue
		var cls := String(lm.get("class", "local")).to_lower()
		var base: float = float(LANDMARK_CLASS_RADIUS.get(cls, LANDMARK_CLASS_RADIUS["local"]))
		## §24: importance is emergent, so it is worth showing. Bounded to
		## +/-25% so the class stays the dominant read — an important local
		## landmark must never out-draw a continental one, or the size stops
		## meaning class at all.
		var imp := clampf(float(lm.get("importance", 0.5)), 0.0, 1.0)
		var r: float = base * (0.75 + 0.5 * imp)
		var col: Color = LANDMARK_COL_CULTURAL if cls == "cultural" else LANDMARK_COL_PHYSICAL
		## Dark halo first so the ring survives on pale terrain, the same
		## two-pass trick the settlement labels use for their outline.
		draw_arc(pos, r, 0, TAU, 22, LANDMARK_OUTLINE, 2.4, true)
		draw_arc(pos, r, 0, TAU, 22, col, 1.3, true)
		## A centre dot only on the two rare classes. On Local, where a dense
		## world can carry hundreds, it fills the ring in and the mark stops
		## reading as open.
		if cls == "continental" or cls == "regional":
			draw_circle(pos, maxf(1.0, r * 0.22), col, true, -1.0, true)


## The candidates the landmark pass offered and did not place -- the second
## half of `LARGE_ITEM_RULINGS.md`'s Landmark-funnel ruling, drawn.
##
## One `draw_multiline` per reason, and that is the whole performance story of
## this layer. A dashed diamond is four segments; at the shipping default there
## are 3 216 marks, so the obvious loop is 12 864 draw calls per redraw, on a
## `_draw` that already carries settlements, roads, labels and icons.
## `draw_multiline` takes every segment of one colour as a single flat point
## array and issues **one** call, so the layer costs three regardless of how
## many marks it holds. The per-mark work left is arithmetic.
##
## Positions are grid CELLS, so `_cell_to_screen`, for the reason
## `_draw_landmarks` states directly above: `Landmark.x`/`.y` and a reject's
## `x`/`y` are the same `usize` cell space, unlike the Icon tool's continuous
## click coordinates.
##
## **What this layer does not draw, and why it is a narrowing rather than an
## omission.** The design's picture puts each placement's exclusion ring on the
## map too -- "rejected candidate — inside a placed one's ring" is the legend,
## and `canvas.json` calls the overlapping rings "the moment the concept
## lands". That picture draws **one highlighted type** (its own readout says
## "Waterfall highlighted · 11"). This shell has no per-type highlight, and a
## default run places 321 landmarks across all twenty kinds: 321 exclusion
## discs, several of them continental at 200 km, is not the design's picture but
## an opaque wash over the terrain the marks are annotating. The rings belong
## with the type highlight, as one piece of work, and are recorded as owed
## rather than approximated here.
func _draw_landmark_rejects(rect: Rect2, interior: Rect2) -> void:
	if not _landmark_rejects_visible or _landmark_rejects.is_empty():
		return
	var r := LM_REJECT_RADIUS
	## Where each edge's dash starts and ends, as a fraction of the edge. A
	## centred dash of `LM_REJECT_DASH` leaves an equal gap at both corners.
	var t0 := (1.0 - LM_REJECT_DASH) * 0.5
	var t1 := 1.0 - t0
	for reason: String in _landmark_rejects:
		var cells: PackedVector2Array = _landmark_rejects[reason]
		if cells.is_empty():
			continue
		var segs := PackedVector2Array()
		for cell in cells:
			var pos := _cell_to_screen(cell, rect)
			if not interior.has_point(pos):
				continue
			## The four corners of the diamond, top-first and clockwise -- the
			## same `M8 2.4 13.6 8 8 13.6 2.4 8z` the design draws.
			var up := pos + Vector2(0, -r)
			var rt := pos + Vector2(r, 0)
			var dn := pos + Vector2(0, r)
			var lf := pos + Vector2(-r, 0)
			for e in [[up, rt], [rt, dn], [dn, lf], [lf, up]]:
				var a: Vector2 = e[0]
				var b: Vector2 = e[1]
				segs.push_back(a.lerp(b, t0))
				segs.push_back(a.lerp(b, t1))
		if segs.is_empty():
			continue
		var col: Color = LM_REJECT_COLORS.get(reason, LM_REJECT_FALLBACK)
		draw_multiline(segs, col, LM_REJECT_WIDTH, true)


## §4.5.5's Label tool: user-authored region-name text, angled/arched in the
## label's own font/color. Ports the reference's `drawArcLabel` (reference
## HTML line ~15244) character-for-character -- same per-glyph placement on
## a circle of radius `R`, same `|arc| < 0.01` straight-line fast path --
## rather than approximating curved text, since a region label's curve is
## itself user-authored content (dragged into shape via the arc handle),
## not decoration.
func _draw_labels(rect: Rect2, interior: Rect2) -> void:
	if _labels.is_empty():
		return
	var font := get_theme_default_font()
	for lb: Dictionary in _labels:
		var text: String = lb["text"]
		if text.is_empty():
			continue
		var pos := _point_to_screen(Vector2(lb["x"], lb["y"]), rect)
		if not interior.has_point(pos):
			continue
		var font_px := _label_font_px(lb, rect)
		var fill: Color = Color(String(lb["color"]))
		## The class type spec, resolved against THIS file's font size rather
		## than the engine's -- see `LABEL_HALO_EM_FALLBACK` and the long note
		## on `ARC_*` above for why the two size models differ. `halo_em`/
		## `tracking_em` are multipliers precisely so the engine does not have
		## to assert a pixel size it does not own.
		var halo_em: float = float(lb.get("halo_em", LABEL_HALO_EM_FALLBACK))
		## `LabelTypography::halo_px`'s own rule, restated: floored at one
		## pixel so an outline survives rasterisation, but zero stays zero --
		## the design's halo slider starts at 0 and that end means "no halo".
		var outline_w: int = 0 if halo_em <= 0.0 else int(maxf(1.0, font_px * halo_em))
		var track_px: float = font_px * float(lb.get("tracking_em", 0.0))
		var italic: bool = bool(lb.get("italic", false))
		var v_center: float = (font.get_ascent(font_px) - font.get_descent(font_px)) / 2.0
		var th: float = deg_to_rad(float(lb["angle"]))
		var a: float = clampf(float(lb["arc"]), -1.0, 1.0)

		if absf(a) < ARC_STRAIGHT_THRESHOLD:
			## Untracked upright text keeps the single `draw_string` it always
			## had: one call, and the font's own kerning intact. Tracking or an
			## oblique forces the per-glyph path, which loses kerning -- the
			## same trade `drawArcLabel` itself makes, and unavoidable, since
			## letter-spacing is defined as extra advance between glyphs.
			if track_px == 0.0 and not italic:
				var full_w := font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, font_px).x
				var local_pos := Vector2(-full_w / 2.0, v_center)
				draw_set_transform(pos, th, Vector2.ONE)
				draw_string_outline(font, local_pos, text, HORIZONTAL_ALIGNMENT_LEFT, -1, font_px, outline_w, LABEL_STROKE_COLOR)
				draw_string(font, local_pos, text, HORIZONTAL_ALIGNMENT_LEFT, -1, font_px, fill)
				continue
			var widths := _glyph_advances(font, text, font_px, track_px)
			var run_w: float = widths[text.length()]   ## the accumulated total
			draw_set_transform_matrix(_label_xform(pos, th, italic))
			for i in text.length():
				var gp := Vector2(widths[i] - run_w / 2.0, v_center)
				var ch := text[i]
				if outline_w > 0:
					draw_string_outline(font, gp, ch, HORIZONTAL_ALIGNMENT_LEFT, -1, font_px, outline_w, LABEL_STROKE_COLOR)
				draw_string(font, gp, ch, HORIZONTAL_ALIGNMENT_LEFT, -1, font_px, fill)
			continue

		var total_w := font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, font_px).x + track_px * maxf(0.0, text.length() - 1.0)
		var radius: float = maxf(font_px * ARC_RADIUS_FLOOR_K,
			total_w / (ARC_SPREAD_DIVISOR * absf(a)))
		var dir_sign: float = 1.0 if a > 0.0 else -1.0
		var acc := -total_w / 2.0
		for ch2 in text:
			var w := font.get_string_size(ch2, HORIZONTAL_ALIGNMENT_LEFT, -1, font_px).x
			var mid := acc + w / 2.0
			var theta := mid / radius
			var glyph_local := Vector2(radius * sin(theta), dir_sign * radius * (1.0 - cos(theta)))
			var world_pt := pos + glyph_local.rotated(th)
			var local_pos2 := Vector2(-w / 2.0, v_center)
			draw_set_transform_matrix(_label_xform(world_pt, th + dir_sign * theta, italic))
			if outline_w > 0:
				draw_string_outline(font, local_pos2, ch2, HORIZONTAL_ALIGNMENT_LEFT, -1, font_px, outline_w, LABEL_STROKE_COLOR)
			draw_string(font, local_pos2, ch2, HORIZONTAL_ALIGNMENT_LEFT, -1, font_px, fill)
			acc += w + track_px
	draw_set_transform(Vector2.ZERO, 0.0, Vector2.ONE)


## Running left edges for each glyph, plus the run's own total at index `n`.
##
## One array rather than two return values so the caller can centre the run and
## place every glyph from the same pass -- and so the total is the accumulated
## sum, never a second measurement that could disagree with it by a rounding.
func _glyph_advances(font: Font, text: String, font_px: int, track_px: float) -> PackedFloat32Array:
	var out := PackedFloat32Array()
	out.resize(text.length() + 1)
	var acc := 0.0
	for i in text.length():
		out[i] = acc
		acc += font.get_string_size(text[i], HORIZONTAL_ALIGNMENT_LEFT, -1, font_px).x
		if i < text.length() - 1:
			acc += track_px
	out[text.length()] = acc
	return out


## Rotation about `pos`, optionally sheared into a synthetic oblique.
##
## The shear is applied in the label's own frame (post-multiplied), so an
## italic label that has been rotated by its angle handle leans relative to its
## own baseline rather than relative to the screen.
func _label_xform(pos: Vector2, rot: float, italic: bool) -> Transform2D:
	var xf := Transform2D(rot, pos)
	if italic:
		xf = xf * Transform2D(Vector2(1.0, 0.0), Vector2(-LABEL_ITALIC_SHEAR, 1.0), Vector2.ZERO)
	return xf


## `size_mode == "fixed"` holds a constant on-screen size regardless of
## zoom (reference: drops its own `_civZoomK()` factor); `"zoom"` grows with
## the terrain, tied to the current px-per-cell fit (`rect.size.x / _gw`) the
## same way `tool_overlay.gd`'s brush cursor radius already scales. Clamped
## to stay legible at extreme zoom in either direction.
func _label_font_px(lb: Dictionary, rect: Rect2) -> int:
	var size: float = float(lb["size"])
	var px: float
	if lb["size_mode"] == "fixed":
		px = size
	else:
		px = size * (rect.size.x / float(_gw)) / LABEL_ZOOM_BASE_PX_PER_CELL
	return int(clampf(px, LABEL_FONT_PX_MIN, LABEL_FONT_PX_MAX))


## The slice of this control's own local space that is actually on screen this
## frame, recomputed once per `_draw()` and read by `_run_offscreen()` below.
##
## Not a Rect2 constant and not `interior`: the camera is an **ancestor**
## transform (see `_crisp_begin()`), so at zoom 8 this control's local rect is
## eight times the window and only a window-sized slice of it is visible.
## `get_global_transform_with_canvas()` is the transform that knows that, and
## inverting it maps the viewport's own rect back into local coordinates.
var _visible_local := Rect2()


func _visible_local_rect() -> Rect2:
	var xf := get_global_transform_with_canvas()
	## A degenerate transform (a zero-scaled ancestor, a control not yet in a
	## viewport) has no meaningful inverse. Answer "everything is visible" --
	## culling is an optimisation and must fail towards drawing.
	if is_zero_approx(xf.determinant()):
		return Rect2(-1e9, -1e9, 2e9, 2e9)
	return (xf.affine_inverse() * get_viewport_rect()).abs()


## Is this whole run outside the window, and therefore free to skip?
##
## **The camera is an ancestor transform, so nothing above this function ever
## knew how far off screen a way was.** Every way in the world was walked,
## dashed and uploaded on every redraw, at every zoom, however far outside the
## window it lay; the viewport threw the result away afterwards. That is what
## made the drawn-object count unbounded in zoom (`MEMORY_OPTIMIZATION_SCOPE.md`
## 2026-08-25: 87 k objects at the opening view, 858 k twelve zoom notches in,
## and 93 MiB of GPU vertex buffers becoming 751 MiB with it).
##
## `pts` are screen px (`_stroke_points`), so the local rect is scaled by the
## same `k` before the test, and grown by half the widest stroke plus a pixel so
## an antialiased edge can never be clipped by this rather than by the viewport.
## A bounding box, not the polyline: it over-draws a diagonal way whose box
## clips the corner, which is the safe direction to be wrong in.
##
## **It moves no pixel by construction** -- what it skips is outside the
## viewport, which discarded it anyway. Verified rather than asserted:
## `_cullframe_probe` compares whole frames against the pre-cull script.
func _run_offscreen(pts: PackedVector2Array, k: float, pad: float) -> bool:
	if pts.is_empty():
		return true
	var box := Rect2(pts[0], Vector2.ZERO)
	for p in pts:
		box = box.expand(p)
	return not Rect2(_visible_local.position * k, _visible_local.size * k) \
		.grow(pad + 1.0).intersects(box)


## Every linear layer's own points, in **screen** pixels ready for a
## `_crisp_begin()` block: `_point_to_screen` gives this control's local space,
## and `* k` is the last step into the space the stroke must be built in. One
## place rather than three identical loops.
func _stroke_points(points: PackedVector2Array, start: int, end: int, rect: Rect2, k: float) -> PackedVector2Array:
	var out := PackedVector2Array()
	out.resize(end - start)
	for i in range(start, end):
		out[i - start] = _point_to_screen(points[i], rect) * k
	return out


## Draws `points[start:end]` (exclusive) as one stroke, converted to
## screen space. `end - start < 2` is a real, legitimate no-op (a run with
## a single point either side of a break contributes nothing to draw).
##
## Drawn inside `_crisp_begin()`, which restores the reference's `rsc` (line
## 15470, `max(1,GW/512)*_civZoomK()*_civWayScale()`) -- the factor every way
## and journey `lineWidth` in `drawCivLayer` is multiplied by, and which this
## port dropped on the way in. Every width constant in this file is therefore
## read as **screen** pixels: `WAY_STYLE.road.under_w`'s 1.2 is 1.2 px of road at
## any zoom, where before it was 1.2 px of *this control*, which the camera
## then scaled to 12.8 on screen at zoom 8 and stretched the antialiasing
## fringe with it.
##
## Two deliberate differences from `rsc`. The resolution half (`max(1,GW/512)`)
## is not taken: it exists so a bigger working canvas gets proportionally
## heavier strokes, and this port's raster is fit to the control rather than
## drawn at grid resolution, so it has no counterpart here (the same reasoning
## `PIN_SCALE_REF_PX`'s doc comment already records for pins). And the zoom
## term is unclamped, where `_civ_zoom_k()` keeps a `0.35` zoom-*out* floor --
## that floor is a readability bound for *pins*, which must not dominate the
## map when it is zoomed all the way out; a way is a line and shrinks harmlessly
## with it, and exactly constant is the simpler contract. (`_civZoomK()`'s
## zoom-*in* cap of 5.0 is no longer ported at all -- see `_civ_zoom_k()`.)
##
## `style` is one `WAY_STYLE` row -- the reference's two-stroke land way: dark
## underlayer, then the type's own colour on top, dashed for the three minor
## tiers and solid for the two trunk ones. Same structure as
## `_draw_sea_route_segment` and `_draw_manual_route_segment` below, which
## always had it; only the land types were flat.
## `load_k` is IN-13's trade-load width multiplier, `1.0` at rest — and
## `1.0` is exact in IEEE-754, so with the layer off every stroke is
## byte-identical to the version before it existed. It multiplies the two
## **widths** and deliberately not the dash lengths: a dash pattern is what
## identifies a way's type, and stretching it on a busy track would make the
## track read as a different tier.
func _draw_way_segment(points: PackedVector2Array, start: int, end: int, rect: Rect2,
		style: Dictionary, load_k: float = 1.0) -> void:
	if end - start < 2:
		return
	var k := _crisp_begin()
	var screen_points := _stroke_points(points, start, end, rect, k)
	if _run_offscreen(screen_points, k, style["under_w"] * _way_scale * load_k * 0.5):
		_crisp_end()
		return
	## `_civWayScale` scales the dash lengths too -- the reference writes
	## `setLineDash([1.8*rsc, 1.3*rsc])`, one `rsc` for both widths and dashes,
	## so a wider road gets a proportionally longer dash rather than a wide line
	## chopped into the same fine ticks.
	draw_polyline(screen_points, _way_ink(style["under"]),
		style["under_w"] * _way_scale * load_k, true)
	var dash: float = style["dash"] * _way_scale
	if dash > 0.0:
		_draw_dashed_polyline(screen_points, _way_ink(style["over"]),
			style["over_w"] * _way_scale * load_k, dash, style["gap"] * _way_scale)
	else:
		draw_polyline(screen_points, _way_ink(style["over"]),
			style["over_w"] * _way_scale * load_k, true)
	_crisp_end()


## One stroke colour with the layer's own opacity multiplier folded in
## (`state.viz.wayOpacity`, `GUI_GAP_REGISTER.md` CA-16). Identity at the
## default `1.0`, so the layer is byte-identical at rest.
func _way_ink(c: Color) -> Color:
	if _way_opacity >= 1.0:
		return c
	return Color(c.r, c.g, c.b, c.a * _way_opacity)


## Sea lane, reference's own two-pass style (reference HTML line ~15511):
## a solid dark-navy underlayer stroke first, then a lighter dashed overlay
## on top. The underlayer is one `draw_polyline` (phase doesn't matter, it's
## solid). The dash overlay is walked manually via `_draw_dashed_polyline`
## below, NOT one `draw_dashed_line` call per vertex pair -- a smoothed
## route's points are only a few px apart, shorter than one dash+gap cycle,
## so restarting the dash phase at every vertex left every segment landing
## inside the "on" portion and rendered as a solid line, not dashed (caught
## by the real-app screenshot verification this milestone requires, not
## assumed).
func _draw_sea_route_segment(points: PackedVector2Array, start: int, end: int, rect: Rect2) -> void:
	if end - start < 2:
		return
	var k := _crisp_begin()   ## Widths and dash lengths in screen px -- see `_draw_way_segment`.
	var screen_points := _stroke_points(points, start, end, rect, k)
	if _run_offscreen(screen_points, k, SEA_ROUTE_UNDERLAY_WIDTH * _way_scale * 0.5):
		_crisp_end()
		return
	draw_polyline(screen_points, _way_ink(SEA_ROUTE_UNDERLAY),
		SEA_ROUTE_UNDERLAY_WIDTH * _way_scale, true)
	_draw_dashed_polyline(screen_points, _way_ink(SEA_ROUTE_DASH_COLOR),
		SEA_ROUTE_DASH_WIDTH * _way_scale,
		SEA_ROUTE_DASH_LENGTH * _way_scale, SEA_ROUTE_DASH_GAP * _way_scale)
	_crisp_end()


## One committed Route-tool route, `points[start:end]` (exclusive). The
## reference's own two-pass journey stroke (block 2b, lines 15555-15559):
## solid dark underlayer first, dashed amber on top. Same structure as
## `_draw_sea_route_segment`, and dashed for the same reason it is -- the
## overlay walk keeps the dash phase continuous across vertices, which a
## per-vertex `draw_dashed_line` would not (see `_draw_dashed_polyline`).
func _draw_manual_route_segment(points: PackedVector2Array, start: int, end: int, rect: Rect2,
		sel: bool = false) -> void:
	if end - start < 2:
		return
	var k := _crisp_begin()   ## Widths and dash lengths in screen px -- see `_draw_way_segment`.
	var screen_points := _stroke_points(points, start, end, rect, k)
	if _run_offscreen(screen_points, k,
			(MANUAL_ROUTE_SEL_UNDERLAY_WIDTH if sel else MANUAL_ROUTE_UNDERLAY_WIDTH) * _way_scale * 0.5):
		_crisp_end()
		return
	draw_polyline(screen_points, _way_ink(MANUAL_ROUTE_UNDERLAY),
		(MANUAL_ROUTE_SEL_UNDERLAY_WIDTH if sel else MANUAL_ROUTE_UNDERLAY_WIDTH) * _way_scale, true)
	_draw_dashed_polyline(screen_points,
		_way_ink(MANUAL_ROUTE_SEL_COLOR if sel else MANUAL_ROUTE_COLOR),
		(MANUAL_ROUTE_SEL_WIDTH if sel else MANUAL_ROUTE_WIDTH) * _way_scale,
		MANUAL_ROUTE_DASH * _way_scale, MANUAL_ROUTE_GAP * _way_scale)
	_crisp_end()


## Draws `points` as a dashed line with the dash phase carried continuously
## across every vertex -- unlike `draw_dashed_line` per-segment, a dash or
## gap can span a vertex instead of always restarting "on" there.
##
## `gap_len` defaults to `dash_len`. Every caller now passes it explicitly --
## no dash pattern anywhere in `drawCivLayer` is actually equal on/off, and the
## one that relied on this default (the sea lane) was wrong because of it. The
## default is kept only so the parameter reads as optional to a future caller
## that genuinely wants a square dash.
func _draw_dashed_polyline(points: PackedVector2Array, color: Color, width: float, dash_len: float, gap_len: float = -1.0) -> void:
	if gap_len < 0.0:
		gap_len = dash_len
	var period := dash_len + gap_len
	var phase := 0.0
	for i in range(points.size() - 1):
		var p0 := points[i]
		var p1 := points[i + 1]
		var seg_vec := p1 - p0
		var seg_len := seg_vec.length()
		if seg_len <= 0.0:
			continue
		var dir := seg_vec / seg_len
		var traveled := 0.0
		while traveled < seg_len:
			var cycle_pos := fmod(phase, period)
			var on := cycle_pos < dash_len
			var remaining_in_state := (dash_len - cycle_pos) if on else (period - cycle_pos)
			# `remaining_in_state` is mathematically > 0 whenever the loop is
			# reached, but `phase` accumulates across every vertex of a long
			# route, and float drift can land `cycle_pos` close enough to a
			# `dash_len`/`period` boundary that subtraction rounds to exactly
			# 0.0 -- `step` then never advances `traveled`, spinning forever
			# and flooding the renderer's draw-command buffer until it
			# overflows (crashed a real run, not a hypothetical). Floor
			# `step` to a sub-pixel epsilon so every iteration makes forward
			# progress; the resulting overshoot is invisible.
			var step := maxf(minf(remaining_in_state, seg_len - traveled), 0.001)
			if on:
				draw_line(p0 + dir * traveled, p0 + dir * (traveled + step), color, width, true)
			traveled += step
			phase += step


func _draw_hover_card(s: Dictionary, rect: Rect2, interior: Rect2) -> void:
	var pos := _cell_to_screen(Vector2(s["x"], s["y"]), rect)
	var kind_label: String = String(s["kind"]).capitalize()
	var lines := ["%s (%s)" % [s["name"], kind_label], "Population %s" % _format_pop(s["population"])]
	lines.append_array(_faith_lines(s))
	var font := get_theme_default_font()
	var font_size := 13
	var line_h := font.get_height(font_size)
	var w := 0.0
	for line in lines:
		w = maxf(w, font.get_string_size(line, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size).x)
	var pad := 8.0
	var card_size := Vector2(w + pad * 2, line_h * lines.size() + pad * 2)
	var card_pos := pos + Vector2(10, -card_size.y - 10)
	# Clamped into the plate interior, not into this control's full bounds:
	# `_draw`'s scissor is still in force here, so a card clamped to the
	# control edge would be sliced by the neatline. Keeping it inside the
	# interior keeps it whole *and* keeps it on the map, which is where a
	# tooltip about map content belongs anyway.
	card_pos.x = clampf(card_pos.x, interior.position.x, maxf(interior.position.x, interior.end.x - card_size.x))
	card_pos.y = clampf(card_pos.y, interior.position.y, maxf(interior.position.y, interior.end.y - card_size.y))

	# Dark-viewport-compatible palette (GUI decluttering pass, 2026-08-17) --
	# was a light-parchment cream/brown card, the fourth stray light-styled
	# surface even though this control's independence from the Theme
	# resource is legitimate (map content, not chrome, see this file's own
	# top comment). Matches the shell's own surface/border/emphasis tokens
	# (`theme/dark_theme.tres`): near-black card fill, accent border,
	# emphasis-toned text.
	draw_rect(Rect2(card_pos, card_size), Color(0.051, 0.055, 0.059, 0.96), true)
	draw_rect(Rect2(card_pos, card_size), Color(0.878, 0.639, 0.290, 1.0), false, 1.5)
	for j in lines.size():
		var text_pos := card_pos + Vector2(pad, pad + line_h * (j + 1) - font.get_descent(font_size))
		draw_string(font, text_pos, lines[j], HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, Color(0.910, 0.922, 0.925))


## The hover card's religion lines, or **nothing at all**.
##
## One or two lines normally -- the plurality with its share, then every other
## faith present -- and a third only while the faith-divergence ring is on and
## this settlement carries one. See the block at the end of this function: the
## third line is the ring's caption, not a fact about religion in general.
##
## A settlement with no population has one line, and that line names the
## absence rather than the share it cannot have -- see the head line's own
## comment below.
##
## Empty is the deliberate answer when `get_settlements()` omitted the
## `religion` key, i.e. when no diffusion has been run in this world. That
## absence is a property of the whole layer, not of this settlement, and it
## has a fix the user has to be told about at length -- which is the CIVIL
## dock's Religion category's job, not a two-line map tooltip's. Printing
## "Faith — not run" over every pin in every world where the feature has
## never been touched would be noise, and would say it once per settlement.
##
## What this must never do is show one faith name as though it were the
## settlement's population. `belief.rs`'s own module doc warns that adherence
## is a distribution, and the model's fixation dynamic makes a plurality
## typically dominant but not exclusive, so the share is always printed
## beside the name and every other faith present is listed after it.
##
## **`"none"` reads "no religion", never blank and never a faith.**
## `SettlementReligionState::plurality` deliberately does not skip
## `RELIGION_NONE` -- a town that is 61 % unaffiliated has a plurality of
## `none`, and labelling it with the largest *faith* would be the same lie in
## reverse.
##
## Labels are derived from the key rather than pushed from
## `civ_religion_vocabulary()`: Godot's `String.capitalize()` reproduces
## seven of the eight `CIV_RELIGIONS` labels exactly (`sun_cult` -> "Sun
## Cult"), and the eighth is handled above by name, so a second push would
## buy one string.
func _faith_lines(s: Dictionary) -> Array:
	if not s.has("religion"):
		return []
	var pop := int(s.get("population", 0))
	var adherents: Dictionary = s.get("adherents", {})
	var plurality := String(s["religion"])
	## A settlement created at population 0 has a real share vector and an
	## empty `adherents` dictionary, so `_faith_share` correctly returns `""`
	## and the line read `Faith Sun Cult` -- indistinguishable from a plurality
	## measured over people. Measured on seed 77021 at 256x192: **158 of 173
	## settlements**, 43 of them led by a faith, every one of those cards
	## saying it without a share and without a reason. The reason is printed
	## instead, which is the same rule the panel's own row follows.
	var head := "Faith %s%s" % [_faith_label(plurality), _faith_share(adherents, plurality, pop)]
	if pop <= 0:
		head += " -- no population, so no share"
	var out: Array = [head]
	## Every other entry actually present, largest first. `adherents` omits a
	## religion with zero adherents (`lib.rs`'s own comment), so every key here
	## has at least one follower and this loop cannot print a 0. Its size is the
	## number of **rows**, not of faiths: the unaffiliated slot is one of them
	## whenever it is not the plurality, and it is not a faith.
	var rest: Array = []
	for k in adherents.keys():
		if String(k) != plurality:
			rest.append([int(adherents[k]), String(k)])
	if not rest.is_empty():
		rest.sort_custom(func(a, b): return a[0] > b[0] if a[0] != b[0] else a[1] < b[1])
		var parts := PackedStringArray()
		for r in rest:
			parts.append("%s%s" % [_faith_label(r[1]), _faith_share(adherents, r[1], pop)])
		out.append("also " + ", ".join(parts))
	## The ring's own caption, and the only legend it has.
	##
	## The broken arcs are a shape with no key drawn anywhere on the map, so a
	## reader who has switched the layer on can see *that* a settlement is
	## marked and not *what it is marked against*. This says it, and says it
	## from the same predicate that drew the arcs -- `_faith_diverged` is false
	## whenever the layer is off, so the line appears exactly when the ring
	## does and cannot describe a mark that is not on screen.
	##
	## `_faith_label` rather than the raw key, so a ruler who has set `none`
	## reads as "no religion" here exactly as the settlement's own row does.
	## Which of the two is the faction's religion is `RELIGION_DIFFUSION_
	## SCOPE.md` section 4's open fork, so neither is called the wrong one.
	if _faith_diverged(s):
		out.append("Ruler's faith %s -- this settlement's has moved"
			% _faith_label(_faction_religions[int(s.get("faction", 0)) - 1]))
	return out

func _faith_label(key: String) -> String:
	## `CIV_RELIGIONS[0]`'s own label is "None / secular", which reads as a
	## missing value in a tooltip; this says the thing it means instead.
	return "no religion" if key == "none" else key.capitalize()

## ` 94%`, ` <1%`, or `""` when the count cannot be turned into a share at all.
##
## Two absences, kept apart:
##
## - **`""`** — population 0, or a key `adherents` does not carry. Neither is
##   a share of zero; there is no denominator and no count. Printing ` 0%`
##   for either would be a fabricated measurement.
## - **` <1%`** — a real, nonzero count that rounds below one percent. Found
##   by running this over a live world, where three of the first four hover
##   cards read `Old Gods 0%` for congregations that genuinely exist:
##   `lib.rs` omits a zero adherent count from the dictionary entirely, so
##   every key here has at least one follower and a printed `0%` could only
##   ever mean "too small to round", which is exactly the reading a bare zero
##   does not give.
func _faith_share(adherents: Dictionary, key: String, pop: int) -> String:
	if pop <= 0 or not adherents.has(key):
		return ""
	var n := int(adherents[key])
	var pct := 100.0 * float(n) / float(pop)
	if n > 0 and pct < 0.5:
		return " <1%"
	## The same guard at the other end, and the live run needed it too:
	## 898 of 900 rounded to `100%` on a card whose very next line read
	## `also Old Gods <1%`. A card that contradicts itself is worse than one
	## that loses a digit.
	if n < pop and pct > 99.5:
		return " >99%"
	return " %d%%" % int(round(pct))

func _format_pop(pop: int) -> String:
	if pop >= 1000:
		return "%.1fk" % (pop / 1000.0)
	return str(pop)


## Nearest settlement whose marker is within its own hit radius of `mouse`,
## or `-1`. Shared by hover (`_gui_input`'s motion branch) and click-to-pin
## (its button branch) so both use exactly one hit-test definition.
## **A deliberate divergence from the engine's own pick, stated rather than
## left to be discovered** (2026-09-01). `lib.rs` binds
## `civ_pick_place_at(gx, gy)` -- `cartalith_civ::tools`' weighted-nearest
## rule, where a bigger settlement outcompetes a closer small one, at
## `civ_place_pick_radius`'s base radius -- and it is wrapped in
## `engine_bridge.gd` and called by no shell file. This screen-space rule is
## what the shell picks with instead, and it is the right one for a pointer:
## it tests against the marker actually on screen, at that marker's own tier
## radius plus `HOVER_RADIUS_PAD`, and it refuses the two cases where nothing
## is drawn -- an off-plate settlement and a hidden addon village. A grid-space
## pick would happily return a settlement the user cannot see, which is worse
## than losing a tie-break.
##
## What IS lost is that tie-break: two overlapping pins here resolve to the
## nearer one whatever their size, where the reference resolves to the more
## important one. Porting it means weighting `closest_dist` by settlement
## rank, which is a third rule unless it is the engine's own weighting exactly
## -- and that weighting is not exposed, only its answer is. Left as it stands,
## and named here and at `civ_pick_place_at` so neither side reads as an
## oversight.
func _hit_test_settlement(mouse: Vector2, interior: Rect2, rect: Rect2) -> int:
	var closest := -1
	var closest_dist := INF
	for i in _settlements.size():
		var s: Dictionary = _settlements[i]
		var pos := _cell_to_screen(Vector2(s["x"], s["y"]), rect)
		# Same predicate `_draw` uses: an off-plate settlement has no
		# marker, so it must not have a hit target either -- otherwise a
		# hover/click would fill in dock data from what looks like blank
		# paper.
		if not interior.has_point(pos):
			continue
		## Likewise for an addon village that is drawn as nothing -- see
		## `_settlement_hidden`.
		if _settlement_hidden(s):
			continue
		var radius: float = _settlement_pin_radius(s["kind"], rect) + HOVER_RADIUS_PAD
		var d := mouse.distance_to(pos)
		if d <= radius and d < closest_dist:
			closest = i
			closest_dist = d
	return closest


## Returns `{valid, gx, gy}` -- the inverse of `_cell_to_screen`, shared by
## `cursor_sampled`, `map_clicked` and `map_dragged` so the coordinate math
## exists in exactly one place. `valid` is false outside the plate interior
## (bare paper, no world coordinate under it) or before anything has
## generated.
func _grid_point(mouse: Vector2, rect: Rect2, interior: Rect2) -> Dictionary:
	if _gw <= 0 or _gh <= 0 or not interior.has_point(mouse):
		return {"valid": false, "gx": 0.0, "gy": 0.0}
	return {
		"valid": true,
		"gx": (mouse.x - rect.position.x) / rect.size.x * _gw,
		"gy": (mouse.y - rect.position.y) / rect.size.y * _gh,
	}

func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseMotion:
		var rect := _displayed_rect()
		if rect.size.x <= 0.0:
			cursor_sampled.emit(0.0, 0.0, false)
			return
		var interior := _interior_rect(rect)
		var mm := event as InputEventMouseMotion
		var mouse: Vector2 = mm.position

		var closest := _hit_test_settlement(mouse, interior, rect)
		if closest != _hover_index:
			_hover_index = closest
			queue_redraw()
			settlement_hovered.emit(_settlements[closest] if closest != -1 else null, closest)

		var p := _grid_point(mouse, rect, interior)
		cursor_sampled.emit(p["gx"], p["gy"], p["valid"])
		## A withheld touch press that has now travelled is a drag, not a hold:
		## let it through *before* the first `map_dragged`, so a stroke's origin
		## is the point the finger went down on rather than wherever it had got
		## to by the time the slop was exceeded.
		if _touch_armed and mouse.distance_to(_touch_pos) > _TOUCH_SLOP:
			_release_touch_press()
		if p["valid"] and (mm.button_mask & MOUSE_BUTTON_MASK_LEFT) != 0:
			map_dragged.emit(p["gx"], p["gy"])

	elif event is InputEventMouseButton:
		var mb := event as InputEventMouseButton
		if mb.button_index == MOUSE_BUTTON_RIGHT:
			## Press, not release -- the reference opens its menu from
			## `contextmenu`, which fires on press. `accept_event()` so the
			## click cannot fall through to anything behind this control,
			## matching the reference's own `e.preventDefault()` ("the canvas
			## has no useful native menu; ours only opens in civ-capable
			## tabs").
			if not mb.pressed:
				return
			var r := _displayed_rect()
			if r.size.x <= 0.0:
				return
			var inter := _interior_rect(r)
			var pt := _grid_point(mb.position, r, inter)
			if not pt["valid"]:
				return
			map_right_clicked.emit(pt["gx"], pt["gy"],
				_hit_test_settlement(mb.position, inter, r), mb.position)
			accept_event()
			return
		if mb.button_index != MOUSE_BUTTON_LEFT:
			return
		var rect := _displayed_rect()
		if rect.size.x <= 0.0:
			return
		var interior := _interior_rect(rect)
		if mb.pressed:
			var hit := _hit_test_settlement(mb.position, interior, rect)
			var p := _grid_point(mb.position, rect, interior)
			## Touch: hold the press back until the gesture identifies itself.
			## See the `_TOUCH_HOLD_MS` block above for the four outcomes.
			if mb.device < 0 or OS.has_feature("mobile"):
				_touch_armed = true
				_touch_swallow_up = false
				_touch_ms = Time.get_ticks_msec()
				_touch_pos = mb.position
				_touch_press = {"hit": hit, "point": p, "pos": mb.position}
				set_process(true)
				return
			settlement_selected.emit(_settlements[hit] if hit != -1 else null, hit)
			if p["valid"]:
				map_clicked.emit(p["gx"], p["gy"])
		else:
			if _touch_swallow_up:
				## The hold already answered this gesture; the lift is its end,
				## not a drag's. Emitting `map_released` here would hand a
				## latched tool (Region select's marquee origin) a commit it
				## never got an origin for.
				_touch_swallow_up = false
				_touch_armed = false
				set_process(false)
				return
			if _touch_armed:
				## A tap: short, and it never travelled. The press it withheld
				## is due now, immediately before the release that ends it.
				_release_touch_press()
			## §4.5.1's Region select needs the release, not the press --
			## `map_dragged` already reported every point along the way, but
			## nothing marked the gesture's *end*, which is where a marquee
			## commits. `p["valid"]` is intentionally not required here: a
			## drag that ends off-plate still has to end the drag, or a
			## caller's own latched origin (`GlobalTools._region_origin`)
			## would never clear.
			var p := _grid_point(mb.position, rect, interior)
			map_released.emit(p["gx"], p["gy"], p["valid"])


## Let a withheld touch press through, unchanged and in its original order --
## the selection first, then the click, exactly as the mouse branch emits them
## and from the point the finger actually went down on, not from where it is
## now. Clears the arming, so a second call is a no-op.
func _release_touch_press() -> void:
	if not _touch_armed:
		return
	_touch_armed = false
	set_process(false)
	var hit: int = int(_touch_press.get("hit", -1))
	var p: Dictionary = _touch_press.get("point", {})
	settlement_selected.emit(_settlements[hit] if hit != -1 else null, hit)
	if bool(p.get("valid", false)):
		map_clicked.emit(p["gx"], p["gy"])

## Runs only while a touch press is outstanding (`set_process` is turned on by
## the press and off by every one of the four exits), so this costs nothing on
## a desktop run and nothing between gestures on a phone.
func _process(_delta: float) -> void:
	if not _touch_armed:
		set_process(false)
		return
	if Time.get_ticks_msec() - _touch_ms < _TOUCH_HOLD_MS:
		return
	## The hold. The withheld press is **discarded**, never emitted: that is the
	## whole point -- opening the menu must not also fire the armed tool.
	_touch_armed = false
	_touch_swallow_up = true
	set_process(false)
	var p: Dictionary = _touch_press.get("point", {})
	if not bool(p.get("valid", false)):
		return
	map_right_clicked.emit(p["gx"], p["gy"], int(_touch_press.get("hit", -1)),
		_touch_press.get("pos", Vector2.ZERO))

func _notification(what: int) -> void:
	if what == NOTIFICATION_MOUSE_EXIT:
		cursor_sampled.emit(0.0, 0.0, false)
		if _hover_index != -1:
			_hover_index = -1
			queue_redraw()
			settlement_hovered.emit(null, -1)


# ── Urban layouts ────────────────────────────────────────────────────────────
#
# `civUrbanLayoutsChk` (`GUI_GAP_REGISTER.md` UM-01): the reference's own
# deep-zoom town-layout layer, `_umDrawLayout` called from `drawCivLayer`'s
# §2.5. This port draws the same layer from the same kind of data, restricted
# to what `URBAN_MORPHOLOGY_SCOPE.md` actually generates — a street skeleton
# on a real site, plus milestone 12's blocks and the lots platted in them.
# Buildings and the wall circuit are milestones 13 and 10 and are not
# generated; `urban_layout_draw.gd` records what it draws in their place.
#
# **The reveal gate IS the reference's `_umLayoutAlpha` again, since
# 2026-08-24.** It was not, for one stated reason: that function crossfades
# pins into layouts across a 24 km → 10 km viewport span, and while this port's
# camera clamped at a flat `ZOOM_MAX = 8.0` the closest reachable span on a
# default 800 km world was ~100 km — a ported 24 km threshold would never once
# have fired. So the gate was the town's site box measured in screen pixels
# (`URBAN_MIN_BOX_PX`) instead, which could.
#
# The cap became `lodMaxZoom()` and the same world now reaches a 5 km span,
# which retired that reason; the swap was deferred one pass and is made here.
# It is not a cosmetic tidy-up — **the pixel gate was measurably the wrong
# number.** Measured live (`_umreveal_shot.gd`, 800 km world, 440 px map area):
# `URBAN_MIN_BOX_PX = 16` first fires at a **47 km** span, where a town is a
# 16 px speck — and because a revealed town *replaces* its pin, the reveal
# swapped a legible marker for a smudge two octaves before the town was worth
# looking at. The reference's own band puts the same reveal at 24 km and
# completes it at 10 km, which on that measurement is a 31 px → 75 px box.
#
# `URBAN_MIN_BOX_PX` survives underneath the band as a floor, not as the gate:
# it is what keeps a narrow map area (a phone, or the map squeezed between two
# open docks) from drawing a sub-pixel town just because the *span* qualifies.
#
# `URBAN_FINE_BOX_PX` is, on measurement, unreachable here and that is correct:
# the deepest span is 5 km, so the box tops out near 150 px on that map area
# and a ~11 m lot is ~1 px. The per-roof ink outline would be wider than the
# roof it surrounds — the same measured finding that put the constant there.
# The fine pass belongs to the City Viewer, which is where a town is actually
# looked at; on the map a town is a mass with streets through it.
## `preload`, not the `UrbanLayoutDraw` global class name -- see
## `city_viewer_window.gd`'s own `DRAW` const for why.
const URBAN_DRAW := preload("res://shell/urban_layout_draw.gd")
## `_umLayoutAlpha`'s crossfade band (reference line 22753), verbatim, in real
## km of map-area span — `UM_FADE_FAR_KM`/`UM_FADE_NEAR_KM` there. Real km, not
## a raw zoom number, for the reference's own stated reason: a zoom's numeric
## meaning scales with map size and 24 km does not.
const UM_FADE_FAR_KM := 24.0
const UM_FADE_NEAR_KM := 10.0
const URBAN_MIN_BOX_PX := 16.0
## Above this on-screen box width, a town is drawn with its per-roof ink
## outline, ridge and drop shadow; below it the roofs are flat fills. 620 px
## across the 1.7 km box puts a ~11 m lot at ~4 px, which is about where an
## outline stops being sub-pixel. See the call in `_draw_urban_layouts()`.
const URBAN_FINE_BOX_PX := 620.0
## The site box, in km — `UME.SITE_WM`/`1000` (`urban_adapter::SITE_WM`).
const URBAN_SITE_BOX_KM := 1.7
## Requested per frame, and the number is the reference's own
## `_UM_MODEL_CACHE_MAX` (line 22684): as many towns as it was willing to hold
## generated at once. Each is a few milliseconds of real generation on the
## main thread, so the cap is what keeps a pan at deep zoom from stalling.
const URBAN_BATCH_MAX := 24

## Emitted when the layer is on, the zoom is deep enough, and these settlement
## indices have no layout yet. `ViewportHost` answers with
## `set_urban_layouts()`. Deliberately a signal rather than a direct bridge
## call: this control holds no `EngineBridge` and every other value it draws
## is pushed into it, not pulled.
signal urban_layouts_needed(indices: PackedInt32Array)

## **On by default**, where the reference's own `civUrbanLayoutsChk` is off —
## the one deliberate divergence in this block, and it is the band above that
## makes it affordable. In the reference the toggle *is* the cost control: with
## it on, every in-view settlement is a generation candidate. Here nothing is
## requested at all until the map area spans under 24 km, which is a view you
## have to deliberately zoom into. Off-by-default cost this feature its whole
## audience instead: it shipped reachable only from CARTO ▸ Layers on the rail
## (never from the map's own Layers button, which lists field rasters), and the
## owner's report was simply "I don't see the settlement rendered on the map
## itself, the dot yes. But not the place."
var _show_urban_layouts := true
## settlement index -> layout Dictionary, or `null` for an index the engine
## refused (a settlement in open water — `_umModelFor`'s own refusal). Both
## are "answered", so neither is requested twice.
var _urban_layouts: Dictionary = {}
var _urban_pending := false
var _map_width_km := 0.0
## `_umRevealedSet` (reference line 22753): the settlement indices whose layout
## was actually drawn this frame, so the pin loop can stand down for them.
## Rebuilt every `_draw()`, never persisted -- the reference rebuilds its own
## per frame for the same reason (a place mid-generation must not lose its pin
## on the strength of the toggle alone).
var _urban_revealed: Dictionary = {}


func set_show_urban_layouts(shown: bool) -> void:
	_show_urban_layouts = shown
	queue_redraw()


## The real map width, needed to size a town against the grid. Pushed from
## `ViewportHost.refresh()` alongside the civ data.
func set_map_width_km(km: float) -> void:
	if is_equal_approx(_map_width_km, km):
		return
	_map_width_km = km
	_urban_layouts.clear()
	queue_redraw()


## `requested` is the batch that was asked for; `layouts` is what came back,
## which is shorter whenever the engine refused one. Recording the whole
## requested set is what stops a refused settlement being re-requested every
## frame forever.
func set_urban_layouts(requested: PackedInt32Array, layouts: Array) -> void:
	for i in requested:
		_urban_layouts[i] = null
	for l: Dictionary in layouts:
		_urban_layouts[int(l["index"])] = l
	_urban_pending = false
	queue_redraw()


## Dropped wholesale when the world changes — `ViewportHost.refresh()` calls
## `set_civ_data`, which calls this.
func clear_urban_layouts() -> void:
	_urban_layouts.clear()
	_urban_pending = false


## Screen pixels per model metre, at the current fit and camera zoom. Widths
## drawn through this scale with the camera exactly as positions do, which is
## right for a town's streets (a real world-space width) and is the opposite
## of what `_settlement_pin_radius` wants for a pin (not a world-space
## quantity at all).
func _urban_m_scale(rect: Rect2) -> float:
	if _map_width_km <= 0.0 or rect.size.x <= 0.0:
		return 0.0
	return rect.size.x / (_map_width_km * 1000.0)


## `lodSpanKm()` (reference line 10675, `mapWidthKm / _lodZoom`): how many km of
## world the map area spans right now. `rect` is the plate in this control's own
## local space and the camera scales the whole control, so the plate covers
## `rect.size.x * _camera_zoom` screen px while the map area itself is `size.x`
## wide. The two are the same number only when the plate fills the width;
## letterboxing (a tall world in a wide map area) is why this is a ratio rather
## than `_map_width_km / _camera_zoom`.
func _urban_span_km(rect: Rect2) -> float:
	var plate_px := rect.size.x * _camera_zoom
	if _map_width_km <= 0.0 or plate_px <= 0.0 or size.x <= 0.0:
		return 0.0
	return _map_width_km * size.x / plate_px


## `_umLayoutAlpha()` (reference line 22754), ported branch for branch. 0 means
## the pins have it; 1 means the layouts do; between, both are drawn and the
## layout is the one that fades.
func _urban_layout_alpha(rect: Rect2) -> float:
	if not _show_urban_layouts:
		return 0.0
	var span := _urban_span_km(rect)
	if span <= 0.0 or span >= UM_FADE_FAR_KM:
		return 0.0
	if span <= UM_FADE_NEAR_KM:
		return 1.0
	return (UM_FADE_FAR_KM - span) / (UM_FADE_FAR_KM - UM_FADE_NEAR_KM)


func _draw_urban_layouts(rect: Rect2, interior: Rect2) -> void:
	var alpha := _urban_layout_alpha(rect)
	if alpha <= 0.0:
		return
	var m_scale := _urban_m_scale(rect)
	if m_scale <= 0.0:
		return
	## The camera scales this whole control, so a box that measures
	## `box_px` here lands `box_px * _camera_zoom` wide on screen.
	var box_px := URBAN_SITE_BOX_KM * 1000.0 * m_scale * _camera_zoom
	if box_px < URBAN_MIN_BOX_PX:
		return

	## The camera scales and offsets this whole control, so `interior` alone is
	## "on the plate", not "on screen" -- at deep zoom that is nearly the
	## entire settlement roster, and generating a town for each is real work.
	## The viewport rect pulled back through this control's own global
	## transform is the actual visible area in the space `rect` is measured in.
	var visible_area := get_global_transform().affine_inverse() * get_viewport_rect()
	visible_area = visible_area.intersection(interior)
	if visible_area.size.x <= 0.0 or visible_area.size.y <= 0.0:
		return
	## Half a box of slack, so a town whose centre is just off-screen still
	## draws the half of itself that is on-screen.
	visible_area = visible_area.grow(box_px * 0.5 / maxf(0.001, _camera_zoom))

	var need := PackedInt32Array()
	for i in _settlements.size():
		var s: Dictionary = _settlements[i]
		if _hidden_settlement_kinds.has(s["kind"]):
			continue
		var pos := _cell_to_screen(Vector2(s["x"], s["y"]), rect)
		if not visible_area.has_point(pos):
			continue
		if not _urban_layouts.has(i):
			if need.size() < URBAN_BATCH_MAX:
				need.append(i)
			continue
		var layout = _urban_layouts[i]
		if layout == null:
			continue
		## `_umDrawLayout`'s own transform: local model metres, measured from
		## the market anchor, rotated by the layout's terrain orientation, then
		## scaled into grid units and projected like any other map point — so
		## the market lands exactly on the settlement's real position and an
		## injected real road overlays the map road it came from.
		var anchor: Vector2 = layout.get("market", Vector2.ZERO)
		var rot: float = float(layout.get("orient", 0.0))
		var cth := cos(rot)
		var sth := sin(rot)
		var grid_per_meter := float(_gw) / (_map_width_km * 1000.0)
		var to_screen := func(mp: Vector2) -> Vector2:
			var l := mp - anchor
			return _point_to_screen(Vector2(
				float(s["x"]) + 0.5 + (l.x * cth - l.y * sth) * grid_per_meter,
				float(s["y"]) + 0.5 + (l.x * sth + l.y * cth) * grid_per_meter), rect)
		## `px_floor` = one screen pixel in this control's own space: the camera
		## scales this whole control by `_camera_zoom`, so a stroke floored at
		## a literal 1.0 here would land `_camera_zoom` px thick on screen.
		## The per-roof ink outline, ridge and drop shadow are three extra
		## passes over every lot in the town, and a town runs to a couple of
		## thousand. A lot is ~11 m across in a 1.7 km box, so it covers
		## `box_px / 155` pixels -- under `URBAN_FINE_BOX_PX` those passes are
		## drawing sub-pixel detail over and over. The City Viewer, which is
		## the place a town is actually looked at, always passes 1.0.
		URBAN_DRAW.draw_layout(self, layout, to_screen, m_scale,
			1.0 / maxf(0.001, _camera_zoom), alpha, false,
			1.0 if box_px >= URBAN_FINE_BOX_PX else 0.0)
		## The pin hands over only at the *end* of the crossfade. The reference
		## fades it instead (`pinAlpha = 1 - _umAlpha`, line 15778) and this
		## does not: a pin here is a disc, an outline, a glyph, a capital ring
		## and a label, each with its own colour constant, so fading it means
		## threading an alpha through five draw calls to soften two seconds of
		## transition. Holding the pin until the layout is fully opaque keeps
		## the thing you are navigating by legible for the whole fade, which is
		## the half of that behaviour that matters. Stated, not silent.
		if alpha >= 1.0:
			_urban_revealed[i] = true

	if need.size() > 0 and not _urban_pending:
		_urban_pending = true
		urban_layouts_needed.emit.call_deferred(need)
