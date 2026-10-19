# Volcano colour and form — a note for later (owner, 2026-10-19)

**Status of this file:** an owner request filed for another time. It is a note, not a scope; it carries no status column (progress lives in `cartalith-native/docs/STATUS.md`; the routed row is in `OUTSTANDING_WORK.md`).

## The request, in the owner's words

> "Make a note for another time, it's for coloring volcanos: volcanic rock and volcanos depending the type shape and age have different colours. Some are green and overgrown, others are still with an active caldera. Do research into this and what determines shape, size and visual aspects of volcanos."

## What to research (to become a research file, then a scope with owner questions)

1. **What determines a volcano's shape and size:** magma composition and viscosity (basaltic shields, andesitic stratovolcanoes, rhyolitic domes and calderas), eruption style (effusive, explosive), tectonic setting (arc, hotspot, rift), eruption frequency and lifespan, edifice collapse and erosion; typical height, base width, flank slope and caldera size per type, with citations.
2. **What determines its visual aspects:** fresh versus weathered rock colours by lithology (basalt, andesite, rhyolite, scoria, ash, pumice, obsidian), how colour changes with age (oxidation, soil formation, vegetation succession from bare lava to forest), snow and ice caps, fumarole and sulphur staining, active crater glow and lava lakes, ash plumes and ash fall, lava flows by age.
3. **A classification that the engine can drive:** type × age × activity × climate → a palette and texture rule set (for example: young active shield: near-black basalt, glowing caldera; old dormant stratovolcano in a wet climate: green, overgrown, eroded gullies; extinct dome: grey rock with lichen or scrub).
4. **What this engine already has to build on (re-verify at the symbols before relying on it):** the volcano edifice model (`volc.edifice_model`, `cartalith_terrain::volcanic_field`, `edifice_pits`, `edifice_furrows`), the lithology and rock model (`GEOLOGY_FIRST_SCOPE.md`, GF-0…GF-8, `rock_painted`), the Satellite relief slope-rock rule (which currently greys volcano flanks), the colour-paint layer's lava material (`COLOUR_PAINT_SCOPE.md` §18–§21; Ruling CN, CS), and the landmark system's volcano kinds.
5. **Measure before building, and judge at ×1 / ×4 / ×16 / ×64** (the owner's standing hard requirement that zoom gains detail at every level).

## Where it connects

- Related open items: the Satellite relief slope-rock rule keeping volcano flanks grey (`OUTSTANDING_WORK.md`, render-target row), type-dependent gully spacing (the edifice rows), and the forest texture rows.
- Owner rulings that apply: CI, CM, CN, CO, CU, CV in `LARGE_ITEM_RULINGS.md`.
