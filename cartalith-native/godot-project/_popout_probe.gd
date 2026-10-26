extends Node
## PW-0 of `POPOUT_WINDOWS_SCOPE.md` -- the measurement spike for popping panels
## out to native OS windows (Ruling DK). It MEASURES; it asserts only its own
## positive controls (so a silent no-op cannot pass as "0 cost"). No product code
## changes: the "debug-only toggle" is this probe setting `Window.force_native`
## on existing shell windows from outside, before they are shown.
##
##   godot --path . --resolution 1400x900 _popout_probe.tscn -- --shots <abs dir>
##
## **WINDOWED only.** Headless has no native sub-windows and `ImageTexture.update()`
## is a no-op there (`MISTAKES.md`, "Run a pixel probe"). Mounts `app.tscn`
## directly under the window root (like `_idleredraw_probe.gd`), not in a
## `SubViewport`: only the real root window can own native sub-windows.
##
## Sections (each prints `[PW0 <tag>]` lines; the report quotes them verbatim):
##   A  embed default: project.godot key present?, ProjectSettings value, the root's
##      `gui_embed_subwindows`, `Window.is_embedded()` of an existing dialog before and
##      after `force_native`, window count at each step.
##   D  DisplayServer screen APIs: count, per-screen position/size/usable/dpi/scale/
##      refresh, `Window.current_screen` assignment.
##   B  one native roster: theme carries (3 pixel samples vs the embedded case and vs
##      the StyleBox colour), content scale, key events (do they reach `DccApp`'s
##      tree?), and whether `BusyOverlay` mutes it during a generate.
##   C  per-frame cost: 0 / 1 / 3 native windows (and the same three EMBEDDED, as the
##      control) on a generated 2048x2048 world. Frame deltas via
##      `RenderingServer.frame_post_draw` ticks, median of 300 frames, three reps;
##      vsync OFF (cost visible) and ON (what the owner sees); `Performance` video
##      memory monitors; process working/private set and the GPU process dedicated
##      usage from the Windows `GPU Process Memory` counters; shipped idle behaviour
##      (low-processor mode) as drawn frames per second and CPU % of a core.
##   O  open latency over 20 opens, embedded vs native.
##
## Never: writes the engine except through the shell's own `bridge.generate`; never
## commits or stages anything; leaves no window open (quit tears the tree down).
##
## Exit: 0 when every positive control held (`[VERDICT] CONTROLS OK`), 1 when one
## failed (`[VERDICT] CONTROLS FAILED (n)`), 2 on an unknown argument, 1 when the
## extension did not load.

const WORLD := 2048
const SEED := 483920
const REPS := 3
const FRAMES := 300

var app: Node
var _shots := ""
## `--embedded-first` runs the embedded configs BEFORE the native ones: the video-memory
## monitors only ever grow (cached canvas buffers), so a fixed order confounds "native" with
## "later". Two runs, two orders, and the per-config before/after delta, are how the report
## separates the two.
var _emb_first := false
var _fail := 0
var _t_pre := 0
var _span := 0.0

## Counts key presses that reach a node's `_unhandled_key_input`: the same path
## `DccApp._unhandled_key_input` sits on, so a hit here means a hit there would
## have been possible, and a miss means the app could never have seen it.
class KeyRec extends Node:
	var hits := 0
	func _unhandled_key_input(e: InputEvent) -> void:
		if e is InputEventKey and (e as InputEventKey).pressed:
			hits += 1

func _control(name: String, good: bool, detail := "") -> void:
	if not good:
		_fail += 1
	print("[PW0 control] ", "ok  " if good else "FAIL", " ", name, "  ", detail)

func _on_pre_draw() -> void:
	_t_pre = Time.get_ticks_usec()

func _frames(n: int) -> void:
	for i in n:
		await RenderingServer.frame_post_draw

## Median / min / max / p95 of an array of floats (ms).
func _stats(a: Array) -> Dictionary:
	var s := a.duplicate()
	s.sort()
	return {"med": s[s.size() / 2], "min": s[0], "max": s[s.size() - 1],
		"p95": s[int(s.size() * 0.95)]}

func _f(x: float) -> String:
	return "%.3f" % x

## Samples `n` consecutive frames. Returns frame-to-frame deltas (ms) and the
## pre_draw -> post_draw span (ms), the CPU-visible render-submit cost.
func _sample(n: int) -> Dictionary:
	var dts := []
	var spans := []
	await RenderingServer.frame_post_draw
	var last := Time.get_ticks_usec()
	for i in n:
		await RenderingServer.frame_post_draw
		var now := Time.get_ticks_usec()
		dts.append((now - last) / 1000.0)
		spans.append((now - _t_pre) / 1000.0)
		last = now
	return {"dt": dts, "span": spans}

func _set_vsync(on: bool) -> void:
	var mode := DisplayServer.VSYNC_ENABLED if on else DisplayServer.VSYNC_DISABLED
	for wid in DisplayServer.get_window_list():
		DisplayServer.window_set_vsync_mode(mode, wid)
	Engine.max_fps = 0

## One powershell read of this process: GPU dedicated + shared usage (all adapters),
## working set and private bytes, CPU ms. `-1` for anything the counter did not give.
func _proc() -> Dictionary:
	var pid := OS.get_process_id()
	var script := "$ErrorActionPreference='SilentlyContinue';" \
		+ "$d=(Get-Counter '\\GPU Process Memory(pid_%d_*)\\Dedicated Usage').CounterSamples|Measure-Object CookedValue -Sum;" % pid \
		+ "$s=(Get-Counter '\\GPU Process Memory(pid_%d_*)\\Shared Usage').CounterSamples|Measure-Object CookedValue -Sum;" % pid \
		+ "$p=Get-Process -Id %d;" % pid \
		+ "Write-Output ('{0} {1} {2} {3} {4}' -f [int64]$d.Sum,[int64]$s.Sum,$p.WorkingSet64,$p.PrivateMemorySize64,[int64]$p.TotalProcessorTime.TotalMilliseconds)"
	var out := []
	OS.execute("powershell", ["-NoProfile", "-Command", script], out)
	var parts := String(out[0]).strip_edges().split(" ") if out.size() > 0 else PackedStringArray()
	if parts.size() < 5:
		return {"gpu_ded": -1, "gpu_shared": -1, "ws": -1, "priv": -1, "cpu_ms": -1}
	return {"gpu_ded": int(parts[0]), "gpu_shared": int(parts[1]), "ws": int(parts[2]),
		"priv": int(parts[3]), "cpu_ms": int(parts[4])}

func _mb(b: int) -> String:
	return "%.1f" % (b / 1048576.0) if b >= 0 else "n/a"

func _mem_line(tag: String) -> void:
	var p := _proc()
	print("[PW0 mem %s] gpu_dedicated=%s MB gpu_shared=%s MB ws=%s MB private=%s MB | Performance: video=%s MB tex=%s MB buf=%s MB | windows=%d" % [
		tag, _mb(p["gpu_ded"]), _mb(p["gpu_shared"]), _mb(p["ws"]), _mb(p["priv"]),
		_mb(int(Performance.get_monitor(Performance.RENDER_VIDEO_MEM_USED))),
		_mb(int(Performance.get_monitor(Performance.RENDER_TEXTURE_MEM_USED))),
		_mb(int(Performance.get_monitor(Performance.RENDER_BUFFER_MEM_USED))),
		DisplayServer.get_window_list().size()])

## The three existing long-lived shell dialogs used as "a native window" stand-ins.
func _wins() -> Array:
	return [app.faction_roster_window, app.settlement_types_window, app.generation_rules_window]

func _open_win(i: int, native: bool) -> void:
	var w: Window = _wins()[i]
	w.force_native = native
	w.transient = true
	w.exclusive = false
	if i == 0:
		app.faction_roster_window.open()
	else:
		w.open()
	await _frames(2)
	w.position = Vector2i(40 + i * 90, 60 + i * 70)
	await _frames(2)

func _close_all() -> void:
	for w in _wins():
		(w as Window).hide()
	await _frames(6)

## Config: `n` windows open, native or embedded. Prints the frame-time rows.
func _perf_cfg(tag: String, n: int, native: bool) -> void:
	await _close_all()
	await _frames(30)
	var v0 := float(Performance.get_monitor(Performance.RENDER_VIDEO_MEM_USED))
	var t0m := float(Performance.get_monitor(Performance.RENDER_TEXTURE_MEM_USED))
	var b0m := float(Performance.get_monitor(Performance.RENDER_BUFFER_MEM_USED))
	for i in n:
		await _open_win(i, native)
	await _frames(30)
	OS.low_processor_usage_mode = false
	var meds := []
	var spans := []
	var mins := 1e9
	var maxs := 0.0
	var p95s := []
	_set_vsync(false)
	await _frames(20)
	for r in REPS:
		var s: Dictionary = await _sample(FRAMES)
		var st := _stats(s["dt"])
		meds.append(st["med"]); p95s.append(st["p95"])
		spans.append(_stats(s["span"])["med"])
		mins = min(mins, st["min"]); maxs = max(maxs, st["max"])
	var m := _stats(meds)
	var sp := _stats(spans)
	print("[PW0 frame %s vsync=off] windows=%d native=%s | frame ms median-of-%d-medians=%s (reps %s..%s) min=%s max=%s p95(med)=%s | draw-span ms median=%s (%s..%s) | n=%dx%d" % [
		tag, n, str(native), REPS, _f(m["med"]), _f(m["min"]), _f(m["max"]), _f(mins), _f(maxs),
		_f(_stats(p95s)["med"]), _f(sp["med"]), _f(sp["min"]), _f(sp["max"]), REPS, FRAMES])
	_set_vsync(true)
	await _frames(20)
	var v: Dictionary = await _sample(FRAMES)
	var vs := _stats(v["dt"])
	print("[PW0 frame %s vsync=on ] windows=%d native=%s | frame ms median=%s min=%s max=%s p95=%s | n=%d" % [
		tag, n, str(native), _f(vs["med"]), _f(vs["min"]), _f(vs["max"]), _f(vs["p95"]), FRAMES])
	await _mem_line(tag)
	print("[PW0 memdelta %s] windows=%d native=%s | Performance video %+.1f MB (texture %+.1f, buffer %+.1f) vs the same run's closed state just before opening" % [
		tag, n, str(native),
		(float(Performance.get_monitor(Performance.RENDER_VIDEO_MEM_USED)) - v0) / 1048576.0,
		(float(Performance.get_monitor(Performance.RENDER_TEXTURE_MEM_USED)) - t0m) / 1048576.0,
		(float(Performance.get_monitor(Performance.RENDER_BUFFER_MEM_USED)) - b0m) / 1048576.0])

## Shipped idle behaviour with `n` native windows: low-processor mode ON, no input.
func _idle_cfg(tag: String, n: int) -> void:
	## Low-processor mode stalls `frame_post_draw` (no change, no draw), so every
	## `_frames()` wait in this probe runs with it OFF; only this idle window turns it on.
	OS.low_processor_usage_mode = false
	await _close_all()
	for i in n:
		await _open_win(i, true)
	await get_tree().create_timer(1.0).timeout
	OS.low_processor_usage_mode = true
	_set_vsync(true)
	await get_tree().create_timer(3.0).timeout   ## settle: the previous config's work and the first powershell read
	var a := _proc()
	var lowp := OS.low_processor_usage_mode
	var f0 := Engine.get_frames_drawn()
	var t0 := Time.get_ticks_usec()
	await get_tree().create_timer(5.0).timeout
	var dt := (Time.get_ticks_usec() - t0) / 1e6
	var drawn := Engine.get_frames_drawn() - f0
	var b := _proc()
	var cpu_ms := float(b["cpu_ms"] - a["cpu_ms"])
	OS.low_processor_usage_mode = false
	print("[PW0 idle %s] native windows=%d low_processor=%s drawn=%d (%.1f/s) cpu=%.0f ms over %.2fs (%.1f%% of a core)" % [
		tag, n, str(lowp), drawn, drawn / dt, cpu_ms, dt,
		cpu_ms / (dt * 10.0)])

## Median/min/max open latency (ms) of `w.open()` -> second frame drawn, 20 opens.
func _latency(tag: String, native: bool) -> void:
	await _close_all()
	var w: Window = app.faction_roster_window
	w.force_native = native
	w.transient = true
	w.exclusive = false
	OS.low_processor_usage_mode = false
	_set_vsync(true)
	var ms := []
	for i in 21:
		w.hide()
		await _frames(4)
		var t0 := Time.get_ticks_usec()
		app.faction_roster_window.open()
		await RenderingServer.frame_post_draw
		await RenderingServer.frame_post_draw
		var d := (Time.get_ticks_usec() - t0) / 1000.0
		if i > 0:
			ms.append(d)   ## the first open of each mode is a warm-up, dropped (20 kept)
	var st := _stats(ms)
	print("[PW0 open-latency %s] native=%s n=%d median=%s ms (min %s .. max %s)  [vsync on: ~2 frames of floor]" % [
		tag, str(native), ms.size(), _f(st["med"]), _f(st["min"]), _f(st["max"])])
	w.hide()
	await _frames(4)

## A pixel (as Color) out of an image, clamped; transparent black on a bad point.
func _px(img: Image, p: Vector2i) -> Color:
	if img == null or p.x < 0 or p.y < 0 or p.x >= img.get_width() or p.y >= img.get_height():
		return Color(0, 0, 0, 0)
	return img.get_pixel(p.x, p.y)

func _first_button(n: Node, skip: Node) -> Control:
	for c in n.get_children():
		if c is Button and c != skip and (c as Button).is_visible_in_tree() and (c as Control).size.x > 8:
			return c
		var r := _first_button(c, skip)
		if r != null:
			return r
	return null

func _hex(c: Color) -> String:
	return "#" + c.to_html(false)

func _theme_and_scale() -> void:
	var w: AcceptDialog = app.faction_roster_window
	await _close_all()
	var root_img_path := _shots + "/embedded_main.png"
	## --- embedded reference ---
	w.force_native = false
	app.faction_roster_window.open()
	await _frames(6)
	var ok: Button = w.get_ok_button()
	var other := _first_button(w, ok)
	print("[PW0 theme] embedded: is_embedded=%s ok=%s other=%s" % [str(w.is_embedded()), str(ok.get_global_rect()), str(other.get_global_rect() if other else "none")])
	var main_img := get_tree().root.get_texture().get_image()
	main_img.save_png(root_img_path)
	var pts_emb := {}
	var rel := {
		"bg_left_of_ok": ok.get_global_rect().position + Vector2(-12, 4),
		"ok_centre": ok.get_global_rect().get_center(),
		"other_btn_centre": other.get_global_rect().get_center() if other != null else Vector2.ZERO,
	}
	## Embedder coordinates = the control's screen transform applied to its local point.
	for k in rel:
		var local: Vector2 = rel[k]
		var ctl: Control = ok if k != "other_btn_centre" else other
		var off: Vector2 = ctl.get_screen_transform().origin - ctl.get_global_rect().position
		pts_emb[k] = _px(main_img, Vector2i(local + off))
	var emb_ok_size := ok.size
	var emb_visible := w.get_visible_rect().size
	await _close_all()
	## --- native ---
	w.force_native = true
	w.transient = true
	w.exclusive = false
	app.faction_roster_window.open()
	await _frames(8)
	var ok2: Button = w.get_ok_button()
	var other2 := _first_button(w, ok2)
	print("[PW0 theme] native:   is_embedded=%s  window_list=%d  size=%s  content_scale_factor=%s mode=%d  theme_set=%s" % [
		str(w.is_embedded()), DisplayServer.get_window_list().size(), str(w.size),
		str(w.content_scale_factor), w.content_scale_mode, str(w.theme != null)])
	var tex_img := w.get_texture().get_image() if w.get_texture() != null else null
	if tex_img != null:
		tex_img.save_png(_shots + "/native_window_texture.png")
	## OS-level capture of THIS window including its title bar. `screen_get_image_rect` was
	## tried first and returned another application's pixels (it grabs the screen region, not
	## the window), so it is not used; `PrintWindow` on the window's own HWND is.
	var hwnd := DisplayServer.window_get_native_handle(DisplayServer.WINDOW_HANDLE, w.get_window_id())
	var png := _shots + "/native_os_chrome.png"
	var cap := "$ProgressPreference='SilentlyContinue'; $ErrorActionPreference='Continue'; Add-Type -AssemblyName System.Drawing; Add-Type -Namespace W -Name P -MemberDefinition '[DllImport(\"user32.dll\")] public static extern bool PrintWindow(System.IntPtr h, System.IntPtr dc, uint f); [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L,T,R,B; } [DllImport(\"user32.dll\")] public static extern bool GetWindowRect(System.IntPtr h, out RECT r);'; $h=[System.IntPtr]%d; $r=New-Object W.P+RECT; [void][W.P]::GetWindowRect($h,[ref]$r); $bmp=New-Object System.Drawing.Bitmap ($r.R-$r.L),($r.B-$r.T); $g=[System.Drawing.Graphics]::FromImage($bmp); $dc=$g.GetHdc(); [void][W.P]::PrintWindow($h,$dc,2); $g.ReleaseHdc($dc); $bmp.Save('%s'); Write-Output ('{0}x{1}' -f $bmp.Width,$bmp.Height)" % [hwnd, png]
	## `create_process`, not `execute`: PrintWindow sends the window's own thread a message, and a
	## blocking `OS.execute` on that same thread deadlocked it (the first attempt hung).
	DirAccess.remove_absolute(png) if FileAccess.file_exists(png) else null
	var pid := OS.create_process("powershell", ["-NoProfile", "-WindowStyle", "Hidden", "-EncodedCommand", Marshalls.raw_to_base64(cap.to_utf16_buffer())])
	await get_tree().create_timer(5.0).timeout
	print("[PW0 theme] PrintWindow capture of the native window incl. its OS frame: pid=%d file written=%s" % [pid, str(FileAccess.file_exists(png))])
	print("[PW0 theme] native texture image=%s" % [str(tex_img.get_size()) if tex_img != null else "null"])
	var rel2 := {
		"bg_left_of_ok": ok2.get_global_rect().position + Vector2(-12, 4),
		"ok_centre": ok2.get_global_rect().get_center(),
		"other_btn_centre": other2.get_global_rect().get_center() if other2 != null else Vector2.ZERO,
	}
	var panel_sb := w.get_theme_stylebox("panel", "AcceptDialog")
	var panel_col: Color = (panel_sb as StyleBoxFlat).bg_color if panel_sb is StyleBoxFlat else Color(0, 0, 0, 0)
	var btn_sb := ok2.get_theme_stylebox("normal")
	var btn_col: Color = (btn_sb as StyleBoxFlat).bg_color if btn_sb is StyleBoxFlat else Color(0, 0, 0, 0)
	var same := 0
	for k in rel2:
		var pn := _px(tex_img, Vector2i(rel2[k]))
		var pe: Color = pts_emb[k]
		var eq := pn.is_equal_approx(pe) or (absf(pn.r - pe.r) < 0.02 and absf(pn.g - pe.g) < 0.02 and absf(pn.b - pe.b) < 0.02)
		if eq:
			same += 1
		print("[PW0 theme] sample %-17s native=%s embedded=%s equal=%s" % [k, _hex(pn), _hex(pe), str(eq)])
	print("[PW0 theme] StyleBox colours: AcceptDialog/panel=%s  OK normal=%s ; project theme custom=%s" % [
		_hex(panel_col), _hex(btn_col), str(ProjectSettings.get_setting("gui/theme/custom", ""))])
	print("[PW0 theme] samples equal native vs embedded: %d of 3" % same)
	_control("theme: native pixels sampled are not all one flat colour", _px(tex_img, Vector2i(rel2["bg_left_of_ok"])) != _px(tex_img, Vector2i(rel2["ok_centre"])) or btn_col == panel_col)
	print("[PW0 scale] ok button size native=%s embedded=%s ; visible_rect native=%s embedded=%s ; screen scale=%s dpi=%s" % [
		str(ok2.size), str(emb_ok_size), str(w.get_visible_rect().size), str(emb_visible),
		str(DisplayServer.screen_get_scale(w.current_screen)), str(DisplayServer.screen_get_dpi(w.current_screen))])
	## Does a per-window content scale carry? CANVAS_ITEMS 1.5: the visible (virtual) rect shrinks by 1.5.
	w.content_scale_mode = Window.CONTENT_SCALE_MODE_CANVAS_ITEMS
	w.content_scale_factor = 1.5
	await _frames(6)
	print("[PW0 scale] native content_scale_factor=1.5/CANVAS_ITEMS: size=%s visible_rect=%s (expect visible ~ size/1.5)" % [str(w.size), str(w.get_visible_rect().size)])
	var s_img := w.get_texture().get_image()
	if s_img != null:
		s_img.save_png(_shots + "/native_scale_1_5.png")
	w.content_scale_factor = 1.0
	w.content_scale_mode = Window.CONTENT_SCALE_MODE_DISABLED
	await _frames(3)

func _key_test() -> void:
	var w: AcceptDialog = app.faction_roster_window
	var rec_main := KeyRec.new()
	get_tree().root.add_child(rec_main)
	var rec_win := KeyRec.new()
	w.add_child(rec_win)
	await _frames(2)
	var ev := InputEventKey.new()
	ev.keycode = KEY_F13
	ev.physical_keycode = KEY_F13
	ev.pressed = true
	## Positive control 1: an event pushed into the main window's viewport reaches the main tree.
	get_tree().root.push_input(ev.duplicate())
	_control("keys: control -- event pushed to the MAIN window reaches the main tree's _unhandled_key_input", rec_main.hits == 1 and rec_win.hits == 0, "main=%d win=%d" % [rec_main.hits, rec_win.hits])
	rec_main.hits = 0
	rec_win.hits = 0
	## The measurement: an event delivered to the NATIVE window's own viewport.
	w.push_input(ev.duplicate())
	_control("keys: control -- event pushed to the NATIVE window reaches that window's tree", rec_win.hits == 1, "win=%d" % rec_win.hits)
	print("[PW0 keys] pushed into native window: main-tree _unhandled_key_input hits=%d (the DccApp path), window-tree hits=%d" % [rec_main.hits, rec_win.hits])
	var pushed_main_hits := rec_main.hits
	rec_main.hits = 0
	rec_win.hits = 0
	## A real OS key (Left Shift, scancode 0x2A: inert in every other application, unlike Ctrl+Z; F13 never arrived) when the
	## native window truly has OS focus. SendKeys goes to the foreground window only.
	w.grab_focus()
	await get_tree().create_timer(0.6).timeout
	var wid := w.get_window_id()
	var focused := DisplayServer.window_is_focused(wid)
	print("[PW0 keys] native window OS-focused=%s (wid %d)" % [str(focused), wid])
	if focused:
		var out := []
		## keybd_event (no SendKeys: its host console can take the foreground before the key goes out).
		var ps := "$ProgressPreference='SilentlyContinue'; Add-Type -Namespace W -Name K -MemberDefinition '[DllImport(\"user32.dll\")] public static extern void keybd_event(byte v, byte s, uint f, System.UIntPtr e);'; [W.K]::keybd_event(0xA0,0x2A,0,[System.UIntPtr]::Zero); Start-Sleep -Milliseconds 80; [W.K]::keybd_event(0xA0,0x2A,2,[System.UIntPtr]::Zero)"
		## -EncodedCommand: the embedded double quotes survive no command-line quoting layer.
		var enc := Marshalls.raw_to_base64(ps.to_utf16_buffer())
		OS.execute("powershell", ["-NoProfile", "-WindowStyle", "Hidden", "-EncodedCommand", enc], out, true)
		print("[PW0 keys] helper output: ", String(out[0]).strip_edges() if out.size() > 0 else "<none>")
		print("[PW0 keys] after the helper process: native window still OS-focused=%s" % str(DisplayServer.window_is_focused(wid)))
		await get_tree().create_timer(0.6).timeout
		print("[PW0 keys] real OS Left-Shift press while native window focused: window-tree hits=%d, MAIN-tree hits=%d" % [rec_win.hits, rec_main.hits])
		if rec_win.hits == 0:
			print("[PW0 keys] (OS key did not arrive in the window at all: inconclusive; the push_input result above stands)")
	else:
		print("[PW0 keys] OS key leg SKIPPED: window not OS-focused (not sending a key to whatever is in front)")
	rec_main.queue_free()
	rec_win.queue_free()
	print("[PW0 keys] summary: key events typed in a native window reach the main tree = %s" % str(pushed_main_hits > 0))

## BusyOverlay vs a native window during a real generate.
func _busy_test() -> void:
	var nat: Window = app.faction_roster_window
	var emb: Window = app.settlement_types_window
	await _close_all()
	nat.force_native = true
	nat.transient = true
	nat.exclusive = false
	emb.force_native = false
	app.faction_roster_window.open()
	app.settlement_types_window.open()
	await _frames(6)
	nat.position = Vector2i(60, 80)
	emb.position = Vector2i(500, 200)
	var counts := {"nat": 0, "emb": 0}
	var bn := Button.new()
	bn.text = "probe-native"
	bn.size = Vector2(120, 40)
	bn.position = Vector2(10, 10)
	bn.pressed.connect(func(): counts["nat"] += 1)
	nat.add_child(bn)
	var be := Button.new()
	be.text = "probe-embedded"
	be.size = Vector2(120, 40)
	be.position = Vector2(10, 10)
	be.pressed.connect(func(): counts["emb"] += 1)
	emb.add_child(be)
	await _frames(4)
	var click := func(win: Window, pos: Vector2, via_root: bool) -> void:
		for down in [true, false]:
			var m := InputEventMouseButton.new()
			m.button_index = MOUSE_BUTTON_LEFT
			m.pressed = down
			m.position = pos
			m.global_position = pos
			if via_root:
				get_tree().root.push_input(m)
			else:
				win.push_input(m)
			await _frames(1)
	## Control, idle: both clicks register.
	await click.call(nat, bn.get_global_rect().get_center(), false)
	var emb_pt: Vector2 = be.get_screen_transform() * (be.size * 0.5)
	await click.call(emb, emb_pt, true)
	_control("busy: control (no generate) -- click lands in the native window's button", counts["nat"] == 1, "native=%d" % counts["nat"])
	_control("busy: control (no generate) -- click lands in the embedded window's button", counts["emb"] == 1, "embedded=%d" % counts["emb"])
	counts["nat"] = 0
	counts["emb"] = 0
	app.bridge.generate({"seed": SEED + 7, "width_km": 800.0, "grid_w": WORLD, "grid_h": WORLD,
		"archetype": "", "villages": true, "sea_level": 0.45})
	await get_tree().create_timer(0.8).timeout
	var ov = app.busy_overlay
	var blocking: bool = ov.is_blocking()
	_control("busy: the overlay is blocking when sampled", blocking and app.bridge.generating, "blocking=%s generating=%s" % [str(blocking), str(app.bridge.generating)])
	var vp_embedded: Array = get_tree().root.get_embedded_subwindows()
	print("[PW0 busy] during generate: native roster in get_embedded_subwindows=%s  embedded settlement-types in it=%s  mirror_count=%d  native.gui_disable_input=%s  embedded.gui_disable_input=%s" % [
		str(vp_embedded.has(nat)), str(vp_embedded.has(emb)), ov.mirror_count(),
		str(nat.gui_disable_input), str(emb.gui_disable_input)])
	await click.call(nat, bn.get_global_rect().get_center(), false)
	emb_pt = be.get_screen_transform() * (be.size * 0.5)
	await click.call(emb, emb_pt, true)
	print("[PW0 busy] click DURING generate: native window button presses=%d (1 = the overlay did NOT mute it), embedded window button presses=%d (0 = muted)" % [counts["nat"], counts["emb"]])
	_control("busy: control -- the embedded window IS muted during the run", counts["emb"] == 0 and emb.gui_disable_input)
	var nat_img := nat.get_texture().get_image() if nat.get_texture() != null else null
	if nat_img != null:
		nat_img.save_png(_shots + "/native_during_generate.png")
	app.get_viewport().get_texture().get_image().save_png(_shots + "/main_during_generate.png")
	while app.bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(10)
	bn.queue_free()
	be.queue_free()
	await _close_all()

func _screens() -> void:
	var n := DisplayServer.get_screen_count()
	print("[PW0 screens] count=%d primary=%d  FEATURE_SUBWINDOWS=%s  display_server=%s" % [
		n, DisplayServer.get_primary_screen(), str(DisplayServer.has_feature(DisplayServer.FEATURE_SUBWINDOWS)), DisplayServer.get_name()])
	for i in n:
		print("[PW0 screens] #%d position=%s size=%s usable=%s dpi=%d scale=%.3f refresh=%.2f Hz" % [
			i, str(DisplayServer.screen_get_position(i)), str(DisplayServer.screen_get_size(i)),
			str(DisplayServer.screen_get_usable_rect(i)), DisplayServer.screen_get_dpi(i),
			DisplayServer.screen_get_scale(i), DisplayServer.screen_get_refresh_rate(i)])
	var w: Window = app.faction_roster_window
	w.force_native = true
	w.transient = true
	w.exclusive = false
	app.faction_roster_window.open()
	await _frames(6)
	var before := w.current_screen
	var pos_before := w.position
	var main_screen := get_tree().root.current_screen
	## Assign the screen it is already on, then (when it exists) every other screen.
	for target in range(n):
		w.current_screen = target
		await _frames(4)
		print("[PW0 screens] assigned current_screen=%d -> readback=%d position=%s (before: screen %d at %s; main window on screen %d)" % [
			target, w.current_screen, str(w.position), before, str(pos_before), main_screen])
		_control("screens: current_screen=%d reads back %d" % [target, target], w.current_screen == target)
		if DisplayServer.screen_get_usable_rect(target).has_point(w.position):
			print("[PW0 screens] ...and the window position lies inside screen %d's usable rect" % target)
	if n < 2:
		print("[PW0 screens] ONLY ONE SCREEN attached: the second-monitor bars (a window dragged to a monitor of different scale, restore-onto-monitor-2, screen-absent clamp with a real second screen) are NOT measured and need the owner's two monitors")
	w.hide()
	await _frames(4)

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--shots" and i + 1 < args.size():
			_shots = args[i + 1]
			i += 2
		elif args[i] == "--embedded-first":
			_emb_first = true
			i += 1
		else:
			print("[FATAL] unknown argument ", args[i]); get_tree().quit(2); return
	if _shots == "":
		print("[FATAL] --shots <abs dir> required"); get_tree().quit(2); return
	DirAccess.make_dir_recursive_absolute(_shots)
	RenderingServer.frame_pre_draw.connect(_on_pre_draw)
	app = load("res://shell/app.tscn").instantiate()
	get_tree().root.add_child.call_deferred(app)
	await get_tree().create_timer(4.0).timeout
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	print("[PW0 env] godot=%s renderer=%s adapter=%s %s | main window %s on screen %d | OS=%s" % [
		Engine.get_version_info()["string"], str(ProjectSettings.get_setting("rendering/renderer/rendering_method")),
		RenderingServer.get_video_adapter_name(), RenderingServer.get_video_adapter_vendor(),
		str(get_tree().root.size), get_tree().root.current_screen, OS.get_name()])
	OS.low_processor_usage_mode = false   ## see `_idle_cfg`: frame_post_draw needs continuous drawing
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if "phone_project_picker" in app and app.phone_project_picker != null:
		app.phone_project_picker.hide()

	# ---- A: the embed default -------------------------------------------------
	var pj := FileAccess.get_file_as_string("res://project.godot")
	var key := "display/window/subwindows/embed_subwindows"
	var w0: Window = app.faction_roster_window
	print("[PW0 embed] project.godot text mentions embed_subwindows=%s (keys set under [display]: window/handheld/orientation only)" % str(pj.contains("embed_subwindows")))
	print("[PW0 embed] ProjectSettings.has_setting=%s value=%s ; root.gui_embed_subwindows=%s ; FEATURE_SUBWINDOWS=%s" % [
		str(ProjectSettings.has_setting(key)), str(ProjectSettings.get_setting(key, "<absent>")),
		str(get_tree().root.gui_embed_subwindows), str(DisplayServer.has_feature(DisplayServer.FEATURE_SUBWINDOWS))])
	print("[PW0 embed] windows before any dialog: %d" % DisplayServer.get_window_list().size())
	app.faction_roster_window.open()
	await _frames(6)
	print("[PW0 embed] roster (no force_native) open: is_embedded=%s force_native=%s visible=%s windows=%d" % [
		str(w0.is_embedded()), str(w0.force_native), str(w0.visible), DisplayServer.get_window_list().size()])
	_control("embed: the engine default embeds an unmodified shell dialog", w0.is_embedded() and DisplayServer.get_window_list().size() == 1)
	w0.hide()
	await _frames(4)

	# ---- a world to measure against -------------------------------------------
	app.bridge.generate({"seed": SEED, "width_km": 800.0, "grid_w": WORLD, "grid_h": WORLD,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var tg := Time.get_ticks_msec()
	while app.bridge.generating:
		await get_tree().create_timer(0.25).timeout
	print("[PW0 world] generated %dx%d seed %d in %.1f s" % [WORLD, WORLD, SEED, (Time.get_ticks_msec() - tg) / 1000.0])
	await get_tree().create_timer(3.0).timeout
	_control("world: a factions list exists to open the roster on", app.bridge.get_factions().size() >= 1, "factions=%d" % app.bridge.get_factions().size())

	await _screens()

	# ---- B ---------------------------------------------------------------------
	await _theme_and_scale()
	await _key_test()
	await _close_all()
	await _busy_test()
	await get_tree().create_timer(3.0).timeout

	# ---- O ---------------------------------------------------------------------
	await _latency("embedded", false)
	await _latency("native", true)
	await _latency("embedded-again", false)
	await _latency("native-again", true)

	# ---- C ---------------------------------------------------------------------
	await _mem_line("baseline-0")
	if _emb_first:
		await _perf_cfg("embedded0-a", 0, false)
		await _perf_cfg("embedded1", 1, false)
		await _perf_cfg("embedded3", 3, false)
		await _perf_cfg("native0-a", 0, true)
		await _perf_cfg("native1", 1, true)
		await _perf_cfg("native3", 3, true)
		await _perf_cfg("native0-after", 0, true)
	else:
		await _perf_cfg("native0-a", 0, true)
		await _perf_cfg("native1", 1, true)
		await _perf_cfg("native3", 3, true)
		await _perf_cfg("native0-after", 0, true)
		await _perf_cfg("embedded1", 1, false)
		await _perf_cfg("embedded3", 3, false)
		await _perf_cfg("embedded0-after", 0, false)
	await _idle_cfg("idle-0", 0)
	await _idle_cfg("idle-1-native", 1)
	await _idle_cfg("idle-3-native", 3)
	await _idle_cfg("idle-0-after", 0)
	await _close_all()
	print("[VERDICT] ", "CONTROLS OK" if _fail == 0 else "CONTROLS FAILED (%d)" % _fail)
	get_tree().quit(0 if _fail == 0 else 1)
