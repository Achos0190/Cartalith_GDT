Measure and export to journey made a 897km measure into a 206km Journey

Civil -> Factions; Unable to remove the last faction after removing other 5.

Labels: only able to remove and edit user placed labels, not possible to adjust the Auto-Placed labels 

Labels: Not able to drag labels around when in this mode

Name Generation: not available on pc

Journey Planner module: slow in response, seems like it's eating a lot of processing time. 

GPU support seems to be completely removed. 

Journey Planner: no overview with all plotted journeys nor a way to delete plotted journeys. This is in a separate spot. 

Assistant: Search function for/external models does not work in pc. 

Assistant setup: External model on PC seems to only accept Key source OS credential store. 

Timeline and collapse simulation: End Date isnt maintained as end date for the timeline/scale. Fort example putting a Start at 4200 and an end at 4700 makes the timeline overflow to 8064

Editing Terrain with local model (Qwen 3) seems to be underwhelming, what information is exactly sent and how is a model able to make sense of it? This might feed into a custom LLM.

Left Control button seems to bring up the Vault menu. Especially frustrating when using cntrl+s or cntrl+z.

When hovering over a settlement the base information card stays a fixed size

Links made by the assistant do not follow the obsidian database links drive "V" does not exsist: > Where Druanvi once stood, there is now only a crater, its scar worn down by four centuries of erosion [V:Echoes Between Sky and Stone/2. Stories/Novels/04. 302 Years after the Fall/Story Building/4 Drafts & Snippets/Snippets/Prose.md]. The once-great city has faded into myth, yet what happened there has not truly ended: its horrors lie buried deep beneath the earth, festering like a wound that will not heal [V:Echoes Between Sky and Stone/2. Stories/Novels/04. 302 Years after the Fall/Story Building/4 Drafts & Snippets/Snippets/Prose.md]. Even the wardens did not escape unchanged. Their power was diminished, and their once-unshakeable faith in their own neutrality was shattered by what took place at Druanvi [V:Echoes Between Sky and Stone/2. Stories/Novels/04. 302 Years after the Fall/Story Building/4 Drafts & Snippets/Snippets/Prose.md]. 

Ways over land are no longer the spaghetti/bundle effect since last upgrades. This should be done for the sea-routes as well.

In Sculpting -> Biome Paint should be Renamed: Biome & Terrain Paint

When Sculpting -> Biome & Terrain Paint; Target Field should also change the visible map layer to match the tool

Android/Mobile only options such as Handedness should not be shown on PC.

In 3D view the embossing effect at deeper zooms should be turned of, makes the map look unnatural and spikey. 

3D forced Full works fine, though heavy - expected due to the embossing and added vertices this adds. (see previous finding)

Animate  Water - Remove this feature entirely

Rivers seem slightly see trough when zooming in. Rivers should be solid. Also the toggle from layers can be removed. 

Option for village seeding isnt available beyond initial generation.

There is continental influence and borders drawn rather in straight lines, borders often stop at geological features research this and adapt the drawing of borders. 

When there are no nations established I want city-states, they should also have a sphere of influence/ability to align other settlements to them as a faction. (when they grow larg enough a cluster the visual naturally will look like a nation) )

When Roads Converge often a settlement/waypoint arises/is founded. The chance is even higher when this happens with a river in its proximity and in between larger settlements. 

Plates and Drift cant be modified after initial (new world) generation. This seems to be when a world type is selected: if this holds and does not happen on the original setting - keep the logic. 

Peak Altitude seems to be Default at 4000m instead of having a base parameter that is relative to map size or what realistically can be expected from tectonic events. (4km shouldn't be a hard-cap)

Screens that have been popped out should be able to auto-fit to screen size, for example when dragged to a second monitor it should be able to use the same windows drag logic to make it take the full screen.

Town Look and Marker Pins should be under CARTO instead of Preferences Menu

After having placed 1 Route the tool becomes inaccessible until other menu's have been selected in between. Same seems to happen when trying to drop multiple settlements, top tool bar should be inspected with its tools and how their functioning is.

Planner misidentified Biome for desert when no such Biome was crossed

Orogeny layer isnt selectable, Does the function even work?

Dropping Pins automatically is not evident on PC, seems to be built on android.

Generation of settlements should be reviewed, not all map information seems to be accounted for when generating a settlement (Location with multple rivers confluencing for example and near a coast.)

Snap to water option for settlement generation doesnt seem to work properly when manually placing settlements.

Buttons seem to overlap/fall behind window edges. 

In the following map there is a sea-route that skips coastal settlements and goes way of the coast: 
Cartalith native port
Grid 2048 x 1024  ·  200 x 100 km
Seed 83744
Quality tier: ultra
GPU: on
Project format version: 2
Bindings missing: ns_sections

Full generation parameters (for reproducing this exact world):
basin_routing: true
carve_rivers: true
civ.base_ag_tech: 5
civ.biome_k: false
civ.factions: 2
civ.fixed_counts: false
civ.founding_year: true
civ.garrison_exposure_scale: 1.0
civ.harbour_box: true
civ.map_crossings: true
civ.map_rivers: true
civ.merge_ways: true
civ.metropolis: true
civ.n_capital: 0
civ.n_city: 0
civ.n_hamlet: 0
civ.n_town: 0
civ.n_village: 0
civ.port_reach: true
civ.port_shore: true
civ.recovery_phase: 3
civ.river_front: true
civ.seed_suppress_div: 22.0
civ.seed_thresh: 0.51
civ.villages: true
climate.albedo_k: 0.0
climate.bulk_evap: true
climate.current_k: 1.0
climate.currents: true
climate.equator_temp: 32.0
climate.evap: 0.2
climate.lapse_rate: 5.0
climate.lat_n: 55.0
climate.lat_s: 5.0
climate.ocean: 1.5
climate.ocean_hum: 1.8
climate.pole_temp: -27.0
climate.press_k: 0.6
climate.rain_bspline: true
climate.rain_dep: 0.38
climate.rain_k: 1.5
climate.terrain_wind_deflection: true
climate.w_iters: 90
climate.wind_dir_deg: 0.0
climate.wind_manual: false
climate.zonal_k: 0.5
crater.age: 0.5
crater.count: 40
crater.physical_model: true
crater.surface_age_myr: 100.0
geo.age: 1.0
geology_carve: false
geology_karst: false
geology_model: true
geology_processes: false
integrate_drainage: true
lake_balance: true
passes.coastal: false
passes.coastal_passes: 4
passes.diffuse_d: 0.15
passes.diffuse_passes: 6
passes.estuary_depth: 0.08
passes.evolve_cycles: 0
passes.glacial: true
passes.glacial_kg: 0.15
passes.glacial_mg: 0.4
passes.glacial_passes: 9
passes.glacial_snowline: 0.65
passes.glacial_u_factor: 0.6
passes.hillslope: false
passes.marsh_band: 0.03
passes.sediment_capacity: 6.0
passes.sediment_fill: false
passes.tidal_flats: false
passes.tidal_k: 0.45
passes.velo_iters: 60
passes.velo_meander: 0.6
passes.velo_strength: 0.51
passes.velocity: false
passes.wave_str: 0.5
peak_m: 4000.0
planet.axial_tilt_deg: 23.4
planet.g: 1.0
planet.rotation_hours: 24.0
relief_model: false
river_density: 2.5
rough_routing: true
sea_level: 0.42
stream.climate_k: 0.5
stream.deposit: 0.3
stream.iters: 15
stream.k: 0.012
stream.uplift: 0.0
tect.age_inf: 0.6
tect.alpha: 1.1
tect.beta: 0.58
tect.blur_r: 18.0
tect.dynamic_lithology: true
tect.flexure: 0.32
tect.hetero: 0.14
tect.high_ceiling: true
tect.lloyd: 2
tect.narrow_plate_base_blur: true
tect.orogen_relief: true
tect.plates: 4
tect.resist: 0.7
tect.ridged: true
tect.vel: 1.8
tect.warp: 0.95
urban_burgage: false
urban_harbour_front: true
urban_market_scale: true
urban_river_kind_map: true
urban_river_stretch: true
urban_riverbank: true
urban_street_village: true
urban_wall_hug: true
use_gpu: true
volc.age: 0.4
volc.cinder_grid_law: true
volc.count: 13
volc.dry_active_craters: true
volc.edifice_model: true
volc.exclude_transform: true
volc.fill_edifice_pits: true
volc.kind_caldera: true
volc.kind_dome: true
volc.kind_flood_basalt: true
volc.kind_maar: true
volc.kind_supervolcano: true
volc.lava_bench: true
volc.morphometry: true
volc.provinces: true
world: false
world_structure.continentality: 0.3
world_structure.enabled: false
world_structure.fragmentation: 0.5
world_structure.hotspot_density: 0.2
world_structure.ocean_depth: 0.6
world_structure.tectonic_energy: 0.6

Based on these findings so far:
Finish the current Outstanding Work and add these point, this work is to be done BEFORE we walk all the menu's and judge if they are correctly categorized/designed/working. And before we Refractor the codebase.
Create a crate to Log what functions do and how they behave, basically i want you to check out what happens on a deep level.
Create a MCP tool you can acces/use to manipulate cartalith as if youre human
Find a skill to playtest/develop/bughunt the application