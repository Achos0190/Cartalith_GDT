extends AcceptDialog
## Credits & academic principles (Phase 1 closeout item, `ROADMAP.md`/
## `PROVENANCE.md`: "the app ships a credits screen... dropping it in the
## rewrite would quietly withdraw attribution the HTML app has always
## given"). Reachable via the header "ⓘ" button (`main.tscn`), mirroring
## the reference HTML's own `#creditsModal` (header ⓘ, `creditsBtn`).
##
## First section is the reference's own attribution, condensed from
## `reference/Cartalith Gen1 v2.10.html` (`#creditsModal`, line ~2043) --
## the port inherits this standing, since it re-implements the same
## published methods in Rust. Second section is this port's own layer:
## the real crate license audit (`cargo license --all-features`,
## 2026-08-17) that `PROVENANCE.md`'s "Licence position" section requires
## before Phase 1 is considered done.

func _ready() -> void:
	get_ok_button().text = "Close"
	%CreditsText.text = _bbcode()


func _bbcode() -> String:
	var s := ""
	s += "[b]Cartalith Terrain Generator[/b] is a native Rust/Godot port of [i]Cartalith Gen1[/i], a single self-contained HTML/Canvas/WebGL procedural world generator. The original app has zero runtime dependencies -- every solver, renderer and UI control is original code. The algorithms below were [b]studied, not copied[/b]: each is an independent implementation of a published method, and this port carries that same standing forward. Full inline attribution lives in the reference source and in its docs/research/.\n\n"

	s += "[b]Programming & code sources studied[/b]\n"
	s += "• Hydraulic erosion -- virtual-pipes shallow-water & droplet methods, informed by LanLou123/Webgl-Erosion, SebLague/Hydraulic-Erosion, weigert/SimpleHydrology, and Beyer (2015).\n"
	s += "• River geometry / drainage synthesis -- Pasternack Lab / RiverBuilder (UC Davis), Genevaux et al. (2013), Galin et al. (2019).\n"
	s += "• Optical water shading -- Beer-Lambert depth + flow-map UV, after Premože & Ashikhmin, \"Rendering Natural Waters\" (Stanford).\n"
	s += "• Procedural noise -- fractal Brownian motion / gradient (Perlin-style) noise, original implementation; this port's GPU noise path additionally uses the PCG3D hash (Jarzynski & Olano, JCGT 2020).\n"
	s += "• Cartalith V1.915 editor -- the routes / settlements / territory / journey-planner layer is ported from the original author's own earlier cartographic editor.\n\n"

	s += "[b]Academic principles -- terrain, tectonics & climate[/b]\n"
	s += "• Plate tectonics -- plate partition, boundary classification (collision / subduction / island-arc / rift / transform) and stress fields drive orogeny and crustal age.\n"
	s += "• Lithospheric flexure & isostasy -- broad flexural response to boundary loads and erosional-unloading isostatic rebound.\n"
	s += "• Stream-power incision -- implicit fluvial incision, Braun & Willett (2013), with multiple-flow-direction drainage.\n"
	s += "• Drainage networks & hydraulic geometry -- Strahler (1957) stream ordering; downstream hydraulic geometry, Leopold & Maddock (1953).\n"
	s += "• Velocity-field hydraulic erosion -- virtual-pipes shallow water with semi-Lagrangian momentum advection, Mei et al. (2007).\n"
	s += "• Climate -- latitude/insolation + altitude lapse-rate temperature, orographic moisture advection, Köppen-Geiger classification with axial-tilt seasons; NPP via the Miami model.\n\n"

	s += "[b]Academic principles -- civilization & population[/b]\n"
	s += "• Carrying capacity & population density -- NPP-anchored forager floor (Tallavaara et al. 2018; Zhu et al. 2021) and a water-gated agrarian ceiling.\n"
	s += "• Central-place theory -- settlement spacing by threshold & range (Christaller 1933; Lösch 1940).\n"
	s += "• Network centrality & robustness -- Brandes (2001) betweenness; scale-free robustness/fragility (Albert, Jeong & Barabási 2000).\n"
	s += "• Gravity model of migration -- flow proportional to attractiveness / distance^β (Zipf 1946; Ravenstein 1885).\n"
	s += "• Logistic population growth -- Verhulst (1838) regrowth toward a catchment ceiling.\n\n"

	s += "[b]This port's own dependencies -- crate license audit[/b] (cargo license, 2026-08-17)\n"
	s += "Unlike the original HTML app, this native port is a Rust/Godot binary and depends on real third-party crates. Every dependency in the workspace was enumerated and checked:\n"
	s += "• The overwhelming majority ([b]~190 of ~200[/b] dependencies) are permissively licensed -- MIT, Apache-2.0, BSD-2-Clause, Zlib, ISC, Unlicense, CC0-1.0, or 0BSD, individually or dual/tri-licensed. This covers every core dependency: rayon (CPU parallelism), wgpu/naga (GPU compute), serde/serde_json (save format), zip/flate2/crc32fast (the .zip save format), glam (vector math), and this project's own nine crates.\n"
	s += "• [b]godot / gdext[/b] (godot, godot-core, godot-ffi, godot-macros, godot-codegen, godot-cell, godot-bindings, gdextension-api) -- [b]MPL-2.0[/b] (Mozilla Public License 2.0), a file-level weak-copyleft license. Used here as an unmodified upstream dependency (the Rust-Godot binding this whole port is built on) -- MPL-2.0's copyleft applies to modifications of MPL-licensed files themselves, not to separate code that merely links against them.\n"
	s += "• [b]libbz2-rs-sys[/b] -- the original bzip2 license (Julian Seward), a permissive BSD-style license, pulled in transitively via the zip crate's optional bzip2 support.\n"
	s += "No GPL, LGPL, AGPL, or other strong-copyleft dependency was found anywhere in the workspace (all features, all platforms).\n\n"

	s += "[i]Detailed, per-method citations and derivations: docs/research/ in the reference project. Algorithms studied, not copied -- original implementations throughout, in both the original HTML app and this port.[/i]"
	return s
