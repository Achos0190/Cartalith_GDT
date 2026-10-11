extends RefCounted
## The busy overlay's message bank: one line of wit per step, the HTML's voice ported whole and extended.
##
## **What.** `LOAD_MSGS` is the reference's own bank (`Cartalith Gen1 v2.11.html`, `LOAD_MSGS`, the
## "loading messages" block just above `showBusy`), its nine categories (`init`, `height`, `tectonics`,
## `erosion`, `hydrology`, `climate`, `render`, `optimization`, `validation`) plus `rare` and `xrare`,
## copied verbatim -- including the HTML's own `×` and its references, which are the reference's
## voice and not this port's to edit. `NEW_MSGS` adds one category per step THIS shell really has:
## the ten engine generation stages (`cartalith_engine::progress::STAGE_NAMES`, mapped by
## `STAGE_CATEGORY`) and the shell's other busy actions (import, region resample, landmark pass,
## viewshed refine, save, open, export, bake, erosion pass, civilisation, journey planning).
##
## **Why.** Owner direction 2026-10-11: "use the same loading messages style as the html. Per step a
## humorous or sarcastic or sassy or know it all kind of message ... make them reflect the current
## action". The HTML hinted ONE category per busy label; the port's engine reports which of ten
## stages is running, so each stage gets its own pool instead of borrowing `height` for everything.
##
## **Never.** Never call into the engine from here (this file is pure data plus one deterministic
## picker, so a probe can test it without a world). Never draw randomness from anywhere but the
## `RandomNumberGenerator` the caller passes: `pick()` is deterministic given that RNG's state,
## which is what lets `_busyov_probe.gd` pin the rare/xrare frequencies with a fixed seed. Never add
## a `class_name` (the shell preloads this file; a new global class needs the editor import pass).
## New lines: cheeky, never crude, no real people or brands, nothing past "darn"-level, ASCII or the
## punctuation the HTML itself uses, at most `MAX_LEN` characters.

## The longest line the overlay is laid out for. A labelled judgement: the longest HTML line
## ("Calculating where the next supercontinent will eventually be.") is 62 characters, and 90 keeps
## a new line on at most two wrapped rows of the 560 px desktop card at the overlay's body size.
const MAX_LEN := 90

## The HTML's tier odds, verbatim (`pickLoadingMsg`): "4% chance of extremely rare easter egg",
## then "12% chance of generic rare message". They are SEQUENTIAL draws, exactly as the HTML wrote
## them, so the unconditional rare rate is 0.96 x 0.12 = 11.52 %, not 12 % -- kept, not "fixed".
const P_XRARE := 0.04
const P_RARE := 0.12

## The reference's bank, verbatim. Do not edit these lines; extend `NEW_MSGS` instead.
const LOAD_MSGS := {
	"init": [
		"Waking the tectonic plates.",
		"Convincing the planet to exist.",
		"Calibrating gravity.",
		"Allocating unreasonable amounts of geology.",
		"Establishing a working relationship with continental drift.",
		"Summoning a fresh lithosphere.",
		"Checking if mountains are still pointy.",
		"Turning mathematics into landscapes.",
		"Reassuring the floating-point gods.",
		"Assembling a perfectly ordinary world.",
		"Booting the mantle.",
		"Locating the planet. Please hold.",
		"Consulting the primordial chaos.",
		"Declaring float32 bankruptcy.",
	],
	"height": [
		"Raising continents.",
		"Digging ocean basins.",
		"Stretching crust to acceptable limits.",
		"Applying suspicious amounts of uplift.",
		"Sculpting terrain one equation at a time.",
		"Distributing elevation responsibly.",
		"Converting noise into geology.",
		"Deciding where the mountains should inconvenience travel.",
		"Negotiating with plate boundaries.",
		"Ensuring islands remain mostly surrounded by water.",
		"Preventing continents from looking like spilled paint.",
		"Installing altitude.",
		"Stacking crust until something snaps.",
		"Convincing the lithosphere to develop character.",
		"Applying elevation in three increasingly ambitious coats.",
	],
	"tectonics": [
		"Moving continents very slowly, very quickly.",
		"Teaching plates to disagree.",
		"Compressing things until mountains happen.",
		"Expanding things until oceans happen.",
		"Simulating millions of years in several milliseconds.",
		"Recreating ancient catastrophes.",
		"Ensuring subduction remains unpleasant.",
		"Folding crust beyond manufacturer specifications.",
		"Calculating where the next supercontinent will eventually be.",
		"Reminding plates that collisions have consequences.",
		"Assigning blame between converging margins.",
		"Filing the collision paperwork.",
		"Stress-testing the lithosphere. Literally.",
		"Orogenic event in progress. Do not panic.",
	],
	"erosion": [
		"Letting time do the heavy lifting.",
		"Sanding mountains down.",
		"Teaching rivers where downhill is.",
		"Redistributing sediment.",
		"Applying several million years of bad weather.",
		"Converting peaks into future beaches.",
		"Making valleys the hard way.",
		"Persuading water to obey gravity.",
		"Allowing entropy to express itself.",
		"Filing rough edges.",
		"Adding geological regret.",
		"Releasing ten thousand simulated droplets into the wild.",
		"Wearing mountains down with extreme patience.",
		"Dragging sediment to the coast against its will.",
		"Discovering that rivers are very opinionated about routes.",
	],
	"hydrology": [
		"Filling depressions with optimism.",
		"Tracing river networks.",
		"Searching for the sea.",
		"Preventing rivers from flowing uphill.",
		"Connecting lakes to reality.",
		"Encouraging water to take the scenic route.",
		"Checking if the oceans are still wet.",
		"Making deltas one grain at a time.",
		"Following the path of least resistance.",
		"Resolving existential questions for inland basins.",
		"Negotiating with watersheds.",
		"Routing discharge through morally ambiguous terrain.",
		"Teaching drainage networks to make better decisions.",
	],
	"climate": [
		"Delivering weather to remote locations.",
		"Positioning rain shadows.",
		"Deciding which regions deserve forests.",
		"Handing deserts their annual precipitation budget.",
		"Assigning climates with questionable fairness.",
		"Teaching the atmosphere about topography.",
		"Placing snow where complaints are inevitable.",
		"Convincing moisture to cross mountains.",
		"Balancing heat, water, and disappointment.",
		"Ensuring tropical jungles remain inconvenient.",
		"Distributing atmospheric moisture responsibly.",
		"Arguing with the Coriolis force.",
		"Fitting a global climate into 240 × 150 cells.",
		"Bringing rain to the windward side. And sorrow to the lee.",
	],
	"render": [
		"Painting coastlines.",
		"Sharpening ridges.",
		"Colouring things according to altitude.",
		"Applying a tasteful amount of relief.",
		"Rendering dramatic mountain silhouettes.",
		"Convincing pixels to resemble geology.",
		"Enhancing cartographic beauty.",
		"Making seas look suitably sea-like.",
		"Adjusting shadows for maximum topography.",
		"Polishing continental aesthetics.",
		"Compositing the biome layer.",
		"Adding the final coat of atmosphere.",
		"Smoothing the ocean so it does not look like gravel.",
		"Illuminating the relief from the northwest, as the gods intended.",
	],
	"optimization": [
		"Compressing several eons into seconds.",
		"Appeasing the GPU.",
		"Politely asking the CPU for cooperation.",
		"Parallelizing irresponsibly.",
		"Removing unnecessary suffering.",
		"Sorting out floating-point disagreements.",
		"Organising chaos.",
		"Making mathematics run faster.",
		"Counting threads.",
		"Pretending this is easy.",
		"Negotiating with the graphics card.",
		"Transferring buffers with aggressive optimism.",
		"Wishing Workers were less complicated.",
	],
	"validation": [
		"Searching for impossible rivers.",
		"Ensuring mountains remain above sea level.",
		"Looking for misplaced oceans.",
		"Inspecting coastlines for crimes against geography.",
		"Detecting suspiciously circular continents.",
		"Asking the terrain if it makes geological sense.",
		"Running quality control on reality.",
		"Preventing catastrophic map weirdness.",
		"Making sure valleys are lower than peaks.",
		"Confirming that gravity still works.",
		"Cross-referencing with plate tectonics.",
		"Verifying that rivers exit somewhere sensible.",
	],
	"rare": [
		"Measuring continental stubbornness.",
		"Adding traces of primordial chaos.",
		"Blaming the noise function.",
		"Blaming Simplex.",
		"Blaming floating-point precision.",
		"Archaeologists are going to have questions.",
		"Your civilization will definitely build a city there.",
		"Mountains are easier to create than roads.",
		"Rivers continue to refuse straight lines.",
		"Somewhere, a geologist is nodding approvingly.",
		"Somewhere else, another geologist is frowning.",
		"Warning: terrain may contain fjords.",
		"No continents were harmed during generation.",
		"Please remain seated until the planet finishes forming.",
		"Adjusting sea level to historically acceptable values.",
		"Recalculating several hundred million years of bad decisions.",
		"Adding just one more mountain range.",
		"Almost done. Probably.",
		"Yes, this really is procedural.",
		"Loading geological consequences.",
		"The terrain has opinions.",
		"Noise functions doing their best.",
		"Cartographers will have notes.",
		"Every seed is a new planet. This one is yours.",
		"Processing continental ambitions.",
		"The coast does not negotiate.",
		"Manifesting landmass.",
		"Dispatching tectonic event. ETA: several million years.",
		"Somewhere a river just changed its mind.",
		"Watershed moment.",
		"Subducting expectations.",
	],
	"xrare": [
		"Please do not tap the supervolcano.",
		"Mountains are not a bug.",
		"Rivers have unionized.",
		"Tectonic plates are arguing again.",
		"We misplaced a continent. Found it.",
		"Asking the dinosaurs for feedback.",
		"Rotating the planet to improve feng shui.",
		"Simulating 600 million years of paperwork.",
		"Consulting ancient sedimentary records.",
		"Installing additional geology.",
		"There is no undo button for continental drift.",
		"Reality is compiling.",
		"Planet assembly in progress.",
		"Some assembly required.",
		"Wrapping infinity into finite dimensions.",
		"Teaching noise functions the meaning of beauty.",
		"Translating mathematics into mountains.",
		"The Earth object was not found. Generating a replacement.",
		"Elevating expectations.",
		"Generating tomorrow's archaeological sites.",
		"Plate tectonics: still not finished debating.",
		"This world was not meant for straight lines.",
		"The geoid is lying, but only a little.",
		"Precipitation event scheduled. Maybe.",
		"One does not simply generate a planet.",
		"The ocean has entered the chat.",
		"Technically, none of this had to exist.",
	],
}

## The port's own categories: one per real step. Written 2026-10-11 for the busy overlay; every
## line follows the "Never" list in this file's header.
const NEW_MSGS := {
	# -- The ten engine stages (`progress.rs::STAGE_NAMES`, in order) --------------------------
	"stage_planet": [
		"Choosing a planet size. Medium. Always medium.",
		"Measuring the radius twice, cutting the crust once.",
		"Setting gravity to 'firm but fair'.",
		"Tilting the axis for dramatic seasonal effect.",
		"Filling the oceans to the line marked 'sea level'.",
		"Rolling the seed. No take-backs.",
		"Spinning the planet up to a respectable speed.",
		"Pouring the mantle. It is warm, thank you for asking.",
		"Establishing which way is north. Bold choice.",
		"Assigning the planet a core personality. Molten.",
		"Checking the planet is round enough for government work.",
		"Inflating the atmosphere to the recommended pressure.",
		"Agreeing on a day length. Twenty-four hours felt traditional.",
		"Laying down the first crust. It is mostly hope.",
	],
	"stage_extent": [
		"Measuring the map in kilometres, then double-checking the kilometres.",
		"Deciding how big a cell is. Bigger than a breadbox.",
		"Fitting the world inside the frame. Snugly.",
		"Converting cells to kilometres without losing any.",
		"Drawing the edges of the known world. Here be margins.",
		"Calibrating the scale bar so nobody is lied to.",
		"Allocating the grid. Every cell gets a desk.",
		"Explaining to the map that it cannot be infinite.",
		"Rounding the extent to the nearest honest number.",
		"Setting the zoom level of reality.",
		"Stretching the grid to the aspect ratio you asked for.",
		"Counting cells. Losing count. Starting over.",
		"Deciding where the world ends and the border begins.",
	],
	"stage_structure": [
		"Sketching continents on a napkin first.",
		"Deciding how many continents is too many.",
		"Arranging landmasses in an aesthetically defensible way.",
		"Seeding cratons. They grow up so fast.",
		"Drafting the continental shelf plan.",
		"Placing the old shields where the new mountains cannot reach.",
		"Laying out the plates like a very slow jigsaw.",
		"Reserving space for an ocean nobody will cross.",
		"Giving each continent a backbone and a coastline.",
		"Choosing which landmass gets the interesting peninsula.",
		"Assigning ancient basement rock. Very ancient. Very basement.",
		"Leaving room for an inland sea, just in case.",
		"Consulting the archetype. It has strong opinions.",
	],
	"stage_tectonics": [
		"Pushing plates together until someone says ouch.",
		"Subducting the evidence.",
		"Rifting gently. Then less gently.",
		"Stacking thrust sheets like unwashed dishes.",
		"Measuring compressional stress with a very firm ruler.",
		"Bending the lithosphere without voiding the warranty.",
		"Asking the transform faults to slide past each other politely.",
		"Raising a mountain range the old-fashioned way: by force.",
		"Explaining to an ocean that it is now a suture zone.",
		"Crumpling the margins. Artfully.",
		"Building ridges along every plate disagreement.",
		"Mapping the boundaries nobody agreed to.",
		"Scheduling a continental collision. Bring a helmet.",
	],
	"stage_volcanism": [
		"Locating hot spots. Do not touch the hot spots.",
		"Placing volcanoes where the crust looked tired.",
		"Scheduling eruptions at a respectful distance from the map edge.",
		"Dropping meteors. Gently, as these things go.",
		"Leaving craters as a reminder to look up.",
		"Building island arcs one lava flow at a time.",
		"Stacking basalt in tasteful layers.",
		"Venting excess enthusiasm through the nearest caldera.",
		"Calculating impact angles. Mostly 'ouch'.",
		"Pouring flood basalts. Mop on standby.",
		"Adding a supervolcano. For flavour.",
		"Cooling the lava before anyone walks on it.",
		"The crater rims have been inspected and found crater-shaped.",
	],
	"stage_erosion": [
		"Raining on every parade, systematically.",
		"Holding a grudge against mountains for a million years.",
		"Wearing down peaks one raindrop at a time. Patience is free.",
		"Rivers are carving valleys out of spite.",
		"Sending sediment downstream with a strongly worded note.",
		"Turning boulders into gravel into sand into beaches.",
		"Letting rain settle old scores with the ridgelines.",
		"Grinding cliffs into slopes, slowly and with feeling.",
		"Erosion is just gravity with a long memory.",
		"Smoothing what the tectonics roughed up.",
		"Every drop takes a little mountain with it. Every single one.",
		"The landscape is losing an argument with the weather.",
		"Polishing valleys to a fine geological finish.",
		"Undercutting riverbanks for dramatic effect.",
	],
	"stage_hydrology": [
		"Teaching every river the way to the sea.",
		"Rivers are asking for directions. Downhill, always downhill.",
		"Filling lakes to the brim, then exactly one drop more.",
		"Finding the sea. It was where we left it.",
		"Merging tributaries into one confident river.",
		"Draining swamps of their sense of mystery.",
		"Counting every drop that reaches the ocean. So far, so good.",
		"Settling a boundary dispute between two watersheds.",
		"Routing rivers around mountains they cannot be bothered to cross.",
		"Spilling lakes over their lowest rim, as nature intended.",
		"Giving each river a Strahler order and a sense of purpose.",
		"Confirming the rivers end in water and not in a mystery.",
		"Dredging the deltas into shape.",
	],
	"stage_climate": [
		"Blowing the prevailing winds in a prevailing direction.",
		"Wringing rain out of clouds on the windward slopes.",
		"Handing out rain shadows to the deserving.",
		"Sorting latitudes into hot, cold, and complaining.",
		"Making the equator warm and the poles smug about it.",
		"Raising temperatures where the sun pays attention.",
		"Measuring rainfall with a very large bucket.",
		"Explaining altitude to the thermometer.",
		"Pushing moist air uphill until it gives up its rain.",
		"Assigning seasons. No refunds.",
		"Ocean currents are carrying warmth wherever they feel like.",
		"Distributing drizzle with bureaucratic precision.",
		"Fog has been scheduled for the coasts. Mornings only.",
	],
	"stage_ecology": [
		"Planting forests where the rain said it was fine.",
		"Rolling out grassland by the hectare.",
		"Assigning deserts. They volunteered.",
		"Growing tundra. Slowly. It is tundra.",
		"Drawing the treeline and asking trees to respect it.",
		"Seeding jungles and stepping back quickly.",
		"Painting steppe where the forests lost interest.",
		"Sorting biomes by temperature, rain, and attitude.",
		"Persuading moss to colonise the awkward bits.",
		"Letting wetlands happen wherever the rivers dawdle.",
		"Mixing a temperate forest. Deciduous, with a hint of conifer.",
		"Teaching mangroves to stand in salt water and like it.",
		"Placing alpine meadows just below the inconvenient snow.",
	],
	"stage_resources": [
		"Burying ore where prospectors will complain about the walk.",
		"Mixing soils: two parts clay, one part ambition.",
		"Seeding the hills with copper and false hope.",
		"Hiding gold where only the stubborn will find it.",
		"Grading the topsoil from 'farmable' to 'try elsewhere'.",
		"Layering coal for a civilisation that does not exist yet.",
		"Leaving salt in the dried-out lake beds.",
		"Enriching the floodplains. They have earned it.",
		"Putting the iron where the mountains used to be.",
		"Composting a few million years of fallen leaves.",
		"Testing soil fertility with a very small shovel.",
		"Rationing the fertile valleys. Settlers will fight over these.",
		"Final inventory: rocks, dirt, and a surprising amount of tin.",
	],
	# -- The shell's other busy actions ----------------------------------------------------------
	"import": [
		"Reading your heightmap. Grey has never looked so meaningful.",
		"Converting pixels into altitude. Brighter means taller.",
		"Inferring tectonics from your terrain. Working backwards, as one does.",
		"Guessing which plates made these mountains. Educated guessing.",
		"Resampling your image onto the working grid.",
		"Reverse-engineering a few hundred million years of geology.",
		"Your mountains came pre-made. We are writing them a history.",
		"Reading black as the deep sea and hoping that was the plan.",
		"Matching your hills to a plausible crust.",
		"Adopting your heightmap. Filling in the paperwork.",
		"Teaching an image to have rivers.",
		"Unflattening a flat file into a round world.",
	],
	"resample": [
		"Zooming in on your selection until it becomes a whole world.",
		"Enhancing. Enhancing. Enhancing.",
		"Adding the detail the old map was too shy to show.",
		"Promoting a region to a world. Big day for it.",
		"Resampling at higher resolution. More pixels, same mountains.",
		"Inventing the small hills the coarse grid glossed over.",
		"Re-running the rivers at a scale they can be proud of.",
		"Inferring the crust beneath your chosen corner.",
		"Your selection is now the centre of its own universe.",
		"Upscaling geology. With taste, not just with arithmetic.",
		"Giving the marquee the full planetary treatment.",
	],
	"landmarks": [
		"Searching for peaks worth naming.",
		"Scouting mountain passes for future bandits.",
		"Measuring which summits have the best views.",
		"Ranking waterfalls by dramatic potential.",
		"Inspecting gorges for sufficient gorgeousness.",
		"Deciding which rock is a landmark and which is merely a rock.",
		"Spacing landmarks so they do not crowd each other's postcards.",
		"Interviewing candidate volcanoes. Most were too shy.",
		"Finding the lonely hill every map needs.",
		"Checking that every landmark has a reason to be there.",
		"Rejecting candidates politely. They took it well.",
		"Placing a natural wonder where travellers will grumble about the detour.",
	],
	"viewshed": [
		"Climbing every lookout to have a good look around.",
		"Calculating what you can see from up there. Mostly hills.",
		"Tracing sight lines across the ridges.",
		"Checking which valleys are hiding from the summits.",
		"Refining the views at full resolution. Squinting harder.",
		"Measuring the horizon. It keeps moving.",
		"Every ridge casts a blind spot. Mapping them all.",
		"Asking the mountains to stand still for the survey.",
		"Casting rays across the terrain one line at a time.",
		"Confirming the panorama is, in fact, panoramic.",
		"Polishing the view from the lookout point.",
	],
	"save": [
		"Packing the world into an archive. Mind the corners.",
		"Folding continents carefully so they do not crease.",
		"Labelling the boxes: heights, rivers, settlements, regrets.",
		"Compressing a planet. It is taking it well.",
		"Writing every cell to disk. Every single one.",
		"Zipping up the oceans. Nothing leaks.",
		"Filing the world under 'finished, for now'.",
		"Archiving your mountains for posterity.",
		"Saving. Your future self says thank you.",
		"Checking the archive closes properly. It does.",
		"Putting the world somewhere safe and well labelled.",
	],
	"load": [
		"Unpacking a world from its archive.",
		"Unfolding continents. A few creases, nothing serious.",
		"Restoring rivers to their rightful beds.",
		"Reading the archive back. It remembers everything.",
		"Putting the settlements back exactly where you left them.",
		"Checking the world survived the trip intact.",
		"Rehydrating the oceans.",
		"Re-hanging the labels on the right places.",
		"Welcome back. The world waited patiently.",
		"Restoring the map from the archive, fresh and slightly dusty.",
		"Opening the box marked 'world'. Lift with your knees.",
	],
	"export": [
		"Counting pixels. All of them.",
		"Rendering the map at a size printers fear.",
		"Arranging pixels in a pleasing order.",
		"Exporting the world one tile at a time.",
		"Convincing every pixel to agree on a colour.",
		"Writing an image large enough to wallpaper a hall.",
		"Stitching tiles together without visible seams.",
		"Tagging the colours so your viewer gets them right.",
		"Shipping the world as a picture. Handle with care.",
		"Encoding the coastline at full resolution.",
		"Proofreading the map for stray pixels.",
	],
	"bake": [
		"Baking the atlas at a gentle heat.",
		"Pre-rendering the tiles so later is faster.",
		"Setting the map so it holds its shape.",
		"Caching the expensive bits for later.",
		"Freezing the relief at its most flattering angle.",
		"Glazing the terrain with a fine layer of shading.",
		"Cooking every zoom level until golden.",
		"Building the tile pyramid. Ancient technique.",
		"Letting the map rest before slicing.",
		"Finalising. No more changes once the oven door shuts.",
		"Baking. Please do not open the oven.",
	],
	"erode": [
		"Raining on the map until the mountains apologise.",
		"Holding a grudge against high ground, efficiently.",
		"Another pass of weather. The peaks will not enjoy it.",
		"Fast-forwarding a few thousand years of drizzle.",
		"Grinding sediment downhill with renewed determination.",
		"The rivers have been told to dig deeper. They agreed.",
		"Applying weather, carefully, to exactly the spots you asked.",
		"Sanding the ridges for a smoother finish.",
		"Letting gravity have the last word, again.",
		"Washing the slopes. The rinse cycle takes ages.",
		"Turning your sharp edges into gentle complaints.",
		"Rain is settling old scores with the cliffs.",
	],
	"civilisation": [
		"Founding villages near water, as one does.",
		"Persuading settlers that the valley is worth it.",
		"Drawing borders that will cause arguments later.",
		"Connecting towns with roads that wander suspiciously.",
		"Electing capitals. The results were close.",
		"Counting heads. Settlers do not sit still.",
		"Placing a market town at the crossroads. Obviously.",
		"Negotiating trade routes over imaginary tea.",
		"Assigning farmland to the villages that asked nicely.",
		"Building ports where the coast looked friendly.",
		"Naming towns after rivers. Lazy, but traditional.",
		"Estimating how many goats the region can support.",
		"Recalculating who owns the hill. Everyone, apparently.",
	],
	"journey": [
		"Plotting a route. The mountains disagree.",
		"Packing provisions for the long way round.",
		"Calculating travel days, rest days, and grumbling days.",
		"Looking for a ford. Wet feet are a last resort.",
		"Asking the roads which way they are going.",
		"Estimating how far a cart goes before the axle complains.",
		"Avoiding the swamp. Everyone always says avoid the swamp.",
		"Routing around the pass that is closed in winter.",
		"Pricing the toll bridges. Robbery, frankly.",
		"Comparing the short way with the safe way.",
		"Checking the inns along the route for vacancies.",
		"Charting the journey one waypoint at a time.",
	],
}

## Engine stage name (`progress.rs::STAGE_NAMES`, verbatim) -> its category. Matched by NAME first so
## a reordered engine table still lands each stage on its own lines; `STAGE_ORDER` is the by-index
## fallback for a renamed stage. `_busyov_probe.gd` checks both tables against the live signal.
const STAGE_CATEGORY := {
	"Planet": "stage_planet",
	"Extent & scale": "stage_extent",
	"World structure": "stage_structure",
	"Tectonics": "stage_tectonics",
	"Volcanism & impacts": "stage_volcanism",
	"Erosion": "stage_erosion",
	"Hydrology": "stage_hydrology",
	"Climate": "stage_climate",
	"Ecology & biomes": "stage_ecology",
	"Resources & soils": "stage_resources",
}
const STAGE_ORDER := [
	"stage_planet", "stage_extent", "stage_structure", "stage_tectonics", "stage_volcanism",
	"stage_erosion", "stage_hydrology", "stage_climate", "stage_ecology", "stage_resources",
]

## A stage pool ALSO draws from the HTML category that covered the same ground, so the reference's
## lines keep turning up in the step they were written for (the HTML hinted `erosion` for "eroding",
## `hydrology` for "routing sediment" and so on). A labelled judgement on which HTML pool fits which
## stage; the stage's own lines are always in its pool.
const ALSO := {
	"stage_planet": ["init"],
	"stage_structure": ["height"],
	"stage_tectonics": ["tectonics"],
	"stage_erosion": ["erosion"],
	"stage_hydrology": ["hydrology"],
	"stage_climate": ["climate"],
	"stage_resources": ["validation"],
	"erode": ["erosion"],
	"bake": ["render"],
	"export": ["render"],
}

## The HTML's `_LOAD_CATS`: where an unknown hint falls back to, verbatim.
const LOAD_CATS := ["init", "height", "tectonics", "erosion", "hydrology", "climate", "render",
	"optimization", "validation"]

## The HTML's `_LABEL_HINT` (busy label -> category), verbatim, plus this shell's own labels below
## the blank line. `category_for_label` resolves a label the way `showBusy` did.
const LABEL_HINT := {
	"generating…": "height", "generating": "height",
	"eroding…": "erosion", "eroding": "erosion",
	"evolving…": "erosion", "evolving": "erosion",
	"finishing…": "validation",
	"routing sediment…": "hydrology",
	"diffusing slopes…": "erosion",
	"simulating weather…": "climate",
	"updating climate…": "climate",
	"computing tides…": "climate",
	"computing seasons…": "climate",
	"shaping coastline…": "erosion",
	"updating sea level…": "hydrology",
	"refining tiles…": "render",
	"baking to atlas…": "render",
	"baking…": "render",
	"loading…": "init",
	"applying feature…": "height",
	"refining…": "render",
	"accreting tidal flats…": "hydrology",

	"importing heightmap…": "import",
	"resampling region…": "resample",
	"placing landmarks…": "landmarks",
	"refining viewshed…": "viewshed",
	"saving project…": "save",
	"opening project…": "load",
	"exporting…": "export",
	"baking atlas…": "bake",
	"erosion pass…": "erode",
	"settling the land…": "civilisation",
	"planning journey…": "journey",
}

## Every category `pick()` can be asked for (the HTML's nine, then the port's). `rare`/`xrare` are
## tiers, not categories, and are deliberately absent.
static func categories() -> PackedStringArray:
	var out := PackedStringArray(LOAD_CATS)
	for k in NEW_MSGS.keys():
		out.append(String(k))
	return out

## The category's OWN lines (no `ALSO` merge), or an empty array for an unknown name. What the
## per-category line counts in the probe and the report are taken from.
static func own_lines(category: String) -> Array:
	if NEW_MSGS.has(category):
		return NEW_MSGS[category]
	if LOAD_MSGS.has(category):
		return LOAD_MSGS[category]
	return []

## The pool a category draws from: its own lines followed by every `ALSO` category's, in that order.
## Empty for an unknown name (the caller falls back, as `pickLoadingMsg` did).
static func pool(category: String) -> Array:
	var out: Array = own_lines(category).duplicate()
	for extra in ALSO.get(category, []):
		out.append_array(own_lines(String(extra)))
	return out

## The category for an engine stage: by `name` when the engine's name is known, else by `index`,
## else `init` (the HTML's own default hint). Never returns an empty string.
static func stage_category(index: int, name: String) -> String:
	if STAGE_CATEGORY.has(name):
		return String(STAGE_CATEGORY[name])
	if index >= 0 and index < STAGE_ORDER.size():
		return String(STAGE_ORDER[index])
	return "init"

## `showBusy`'s hint lookup: the exact label, then the label cut after its first `…` (the HTML's
## `label.replace(/….*$/,'…')`), then `init`. Never returns an empty string.
static func category_for_label(label: String) -> String:
	if LABEL_HINT.has(label):
		return String(LABEL_HINT[label])
	var cut := label.find("…")
	if cut >= 0:
		var head := label.substr(0, cut + 1)
		if LABEL_HINT.has(head):
			return String(LABEL_HINT[head])
	return "init"

## `pickLoadingMsg(hint)`, deterministic given `rng`: returns `{text, tier, category}` where `tier` is
## `"xrare"`, `"rare"` or `"cat"` and `category` is the pool actually drawn from.
##
## The tier draw is the HTML's, in its order and with its odds (`P_XRARE` first, then `P_RARE`);
## an unknown `category` falls back to a random one of `LOAD_CATS`, as the HTML's did.
##
## **One deliberate difference: a line never repeats the previous one, in any tier.** The HTML
## re-drew at most six times and only inside the hinted category, so a rare line could show twice
## running and a two-line pool could too. Here a draw that equals `last` steps to the next line of
## the same pool, which keeps the tier odds untouched and makes the no-repeat rule unconditional
## for every pool longer than one line (every pool in this file).
static func pick_entry(category: String, rng: RandomNumberGenerator, last: String = "") -> Dictionary:
	var tier := "cat"
	var key := category
	if rng.randf() < P_XRARE:
		tier = "xrare"
		key = "xrare"
	elif rng.randf() < P_RARE:
		tier = "rare"
		key = "rare"
	var arr: Array = LOAD_MSGS[key] if tier != "cat" else pool(key)
	if arr.is_empty():
		key = String(LOAD_CATS[rng.randi_range(0, LOAD_CATS.size() - 1)])
		arr = pool(key)
	var i := rng.randi_range(0, arr.size() - 1)
	if String(arr[i]) == last and arr.size() > 1:
		i = (i + 1) % arr.size()
	return {"text": String(arr[i]), "tier": tier, "category": key}

## The line alone -- `pick_entry(...)["text"]`. See `pick_entry` for the contract.
static func pick(category: String, rng: RandomNumberGenerator, last: String = "") -> String:
	return String(pick_entry(category, rng, last)["text"])
