package org.cartalith.pen

import android.util.Log
import org.godotengine.godot.Godot
import org.godotengine.godot.plugin.GodotPlugin
import org.godotengine.godot.plugin.SignalInfo
import org.godotengine.godot.plugin.UsedByGodot

/**
 * The Godot v2 Android plugin singleton `CartalithPen` (Ruling EK; STYLO2_PEN_PROPOSAL.md section 6.2). It knows
 * bytes and the pen and NOTHING about tools: every mapping, setting and guard is the GDScript facade `PenLink`.
 *
 * Signals (copied, then posted by `emitSignal`, so no GDScript runs on a Binder thread):
 *   pen_state(String)  off, unsupported, no_permission, not_paired, connecting, connected, lost
 *   pen_double_tap()
 *   pen_slide(Int)
 *   pen_info(Int battery, String firmware, String name)  with -1 or "" for a field not yet known; the facade turns
 *       those into "absent" and never shows them as values.
 *
 * MUST NEVER: request a permission (the facade does, when the user switches the link on), scan, write outside the
 * allow-list, or start by itself: nothing connects until `start()`.
 */
class CartalithPenPlugin(godot: Godot) : GodotPlugin(godot), Stylo2Pen.Listener {
    private var pen: Stylo2Pen? = null
    private var battery = -1
    private var firmware = ""
    private var name = ""

    override fun getPluginName(): String = "CartalithPen"

    override fun getPluginSignals(): Set<SignalInfo> = setOf(
        SignalInfo("pen_state", String::class.java),
        SignalInfo("pen_double_tap"),
        SignalInfo("pen_slide", Integer::class.java),
        SignalInfo("pen_info", Integer::class.java, String::class.java, String::class.java),
    )

    private fun client(): Stylo2Pen? {
        if (pen == null) pen = activity?.applicationContext?.let { Stylo2Pen(it, this) }
        return pen
    }

    /** API 33 and BLE present. Cheap and side-effect free; false (never an error) elsewhere. */
    @UsedByGodot
    fun is_supported(): Boolean = runCatching { client()?.supported() ?: false }.getOrDefault(false)

    @UsedByGodot
    fun has_permission(): Boolean = runCatching { client()?.hasPermission() ?: false }.getOrDefault(false)

    /** Begin the link; returns the immediate state. Never throws into Godot. */
    @UsedByGodot
    fun start(): String = runCatching { client()?.start() ?: "unsupported" }.getOrElse { "lost" }

    @UsedByGodot
    fun stop() { runCatching { pen?.stop() } }

    @UsedByGodot
    fun query_battery() { runCatching { pen?.queryBattery() } }

    @UsedByGodot
    fun query_info() { runCatching { pen?.queryInfo() } }

    override fun onMainPause() { runCatching { pen?.suspend() } }
    override fun onMainResume() { runCatching { pen?.resume() } }
    override fun onMainDestroy() { runCatching { pen?.stop() } }

    // ---- Stylo2Pen.Listener: Binder-thread callbacks; emitSignal posts to Godot's thread ----
    override fun onState(state: String) {
        if (state != "connected") { battery = -1; firmware = ""; name = "" }
        emitSignal("pen_state", state)
    }
    override fun onDoubleTap() { emitSignal("pen_double_tap") }
    override fun onSlide(position: Int) { emitSignal("pen_slide", position) }
    override fun onBattery(percent: Int) { battery = percent; emitInfo() }
    override fun onFirmware(version: String) { firmware = version; emitInfo() }
    override fun onName(name: String) { this.name = name; emitInfo() }
    override fun onUnknown(text: String) { Log.i("CartalithPen", "dropped: " + text.take(80)) }

    private fun emitInfo() { emitSignal("pen_info", battery, firmware, name) }
}
