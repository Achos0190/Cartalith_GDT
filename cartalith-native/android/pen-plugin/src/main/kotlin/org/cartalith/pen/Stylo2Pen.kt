package org.cartalith.pen

import android.Manifest
import android.annotation.SuppressLint
import android.bluetooth.BluetoothDevice
import android.bluetooth.BluetoothGatt
import android.bluetooth.BluetoothGattCallback
import android.bluetooth.BluetoothGattCharacteristic
import android.bluetooth.BluetoothGattDescriptor
import android.bluetooth.BluetoothManager
import android.bluetooth.BluetoothProfile
import android.bluetooth.BluetoothStatusCodes
import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import android.os.Handler
import android.os.Looper
import java.util.UUID

/**
 * The FIXED read-only BLE client for the OnePlus Stylo 2 (Ruling EK). Derived from the owner's pack file
 * `docs/evidence/stylus-2026-11-08/cartalith-pen/Stylo2Pen.kt` (kept verbatim there; package `cartalith.pen`).
 *
 * WHAT WAS CHANGED, and why (proposal section 2.4/2.5, defects numbered as there):
 *  1. Four CCCD writes back to back: now one operation at a time through [OpQueue]; the next starts in `onDescriptorWrite`.
 *  2. `onConnected` fired before any subscription: now "connected" is reported only when the queue drains with every subscription confirmed.
 *  3. No `close()` on an unexpected disconnect: every disconnect and every failed operation closes the `BluetoothGatt`, then reconnects with [PenProtocol.backoffMs].
 *  4. No status handling: `status != GATT_SUCCESS` on connect, service discovery, descriptor and characteristic writes is a failure; the int returned by `writeCharacteristic`/`writeDescriptor` is checked.
 *  5. `name.contains("Stylo")`: now the exact [PenProtocol.PEN_NAME]; reading the name needs BLUETOOTH_CONNECT, checked first.
 *  6. Every command writable (`setVibrationType` public): REMOVED. The only writes are the three queries, gated by [PenProtocol.isAllowedWrite]. No vibration, enable or firmware frame exists in this file.
 *  7. Binder-thread callbacks: [Listener] methods run on the Binder or main thread; [CartalithPenPlugin] copies and posts via `emitSignal`.
 *
 * MUST NEVER: scan, request a permission itself, write a frame outside the allow-list, or throw into the caller.
 * Needs BLUETOOTH_CONNECT (API 31+) and the API 33 GATT calls; [supported] says so, and the app's minSdk is not raised.
 */
@SuppressLint("MissingPermission")
class Stylo2Pen(private val context: Context, private val listener: Listener) {

    /** Events out. Called on a Binder or main thread; implementers must not touch Godot directly. */
    interface Listener {
        fun onState(state: String)
        fun onDoubleTap()
        fun onSlide(position: Int)
        fun onBattery(percent: Int)
        fun onFirmware(version: String)
        fun onName(name: String)
        /** A frame or failure the client could not classify: logged by the plugin, never acted on. */
        fun onUnknown(text: String)
    }

    private val main = Handler(Looper.getMainLooper())
    private var gatt: BluetoothGatt? = null
    private var wanted = false
    private var connected = false
    private var attempt = 0
    private var retry: Runnable? = null

    /** An operation in the single GATT queue. */
    private sealed class Op {
        data class Subscribe(val chr: UUID) : Op()
        class Write(val frame: ByteArray) : Op()
    }

    private val queue = OpQueue<Op>(
        start = { startOp(it) },
        onDrained = { onQueueDrained() },
        onFailure = { failAndReconnect("gatt operation failed: " + it::class.simpleName) },
    )

    private fun adapter() = (context.getSystemService(Context.BLUETOOTH_SERVICE) as? BluetoothManager)?.adapter

    /** Whether this device can run the link at all: API 33 and BLE present. Pairing and permission are separate states. */
    fun supported(): Boolean =
        Build.VERSION.SDK_INT >= 33 &&
            context.packageManager.hasSystemFeature(PackageManager.FEATURE_BLUETOOTH_LE)

    fun hasPermission(): Boolean =
        Build.VERSION.SDK_INT >= 31 &&
            context.checkSelfPermission(Manifest.permission.BLUETOOTH_CONNECT) == PackageManager.PERMISSION_GRANTED

    /** The bonded pen by EXACT name, or null. Needs the permission (the name read does). */
    fun bondedPen(): BluetoothDevice? {
        if (!hasPermission()) return null
        return adapter()?.bondedDevices?.firstOrNull { it.name == PenProtocol.PEN_NAME }
    }

    /** Begin the link. Returns the state reached immediately; later states arrive through [Listener.onState]. */
    fun start(): String {
        wanted = true
        attempt = 0
        return connectNow()
    }

    /** Stop for good: no reconnect, GATT closed. */
    fun stop() {
        wanted = false
        cancelRetry()
        teardown()
        listener.onState("off")
    }

    /** Pause without forgetting that the link is wanted (app in the background). */
    fun suspend() { cancelRetry(); teardown() }

    /** Resume after [suspend] if the link is still wanted. */
    fun resume() { if (wanted && gatt == null) connectNow() }

    fun queryBattery() = write(PenProtocol.QUERY_BATTERY)
    fun queryInfo() { write(PenProtocol.QUERY_FIRMWARE); write(PenProtocol.QUERY_NAME) }

    private fun write(frame: ByteArray) {
        // The single gate: anything not on the allow-list is dropped here, whoever called.
        if (!PenProtocol.isAllowedWrite(frame) || !connected) return
        queue.enqueue(Op.Write(frame))
    }

    private fun connectNow(): String {
        if (!supported()) { listener.onState("unsupported"); return "unsupported" }
        if (!hasPermission()) { listener.onState("no_permission"); return "no_permission" }
        val dev = bondedPen()
        if (dev == null) { listener.onState("not_paired"); return "not_paired" }
        teardown()
        listener.onState("connecting")
        gatt = dev.connectGatt(context, false, callback, BluetoothDevice.TRANSPORT_LE)
        return "connecting"
    }

    private fun teardown() {
        connected = false
        queue.reset()
        gatt?.let { runCatching { it.disconnect() }; runCatching { it.close() } }
        gatt = null
    }

    private fun failAndReconnect(why: String) {
        listener.onUnknown(why)
        teardown()
        listener.onState("lost")
        if (!wanted) return
        val delay = PenProtocol.backoffMs(attempt++)
        cancelRetry()
        retry = Runnable { retry = null; if (wanted) connectNow() }.also { main.postDelayed(it, delay) }
    }

    private fun cancelRetry() { retry?.let { main.removeCallbacks(it) }; retry = null }

    private fun startOp(op: Op): Boolean {
        val g = gatt ?: return false
        return when (op) {
            is Op.Subscribe -> {
                val ch = findChar(g, op.chr) ?: return false
                val cccd = ch.getDescriptor(PenProtocol.CCCD) ?: return false
                g.setCharacteristicNotification(ch, true) &&
                    g.writeDescriptor(cccd, BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE) == BluetoothStatusCodes.SUCCESS
            }
            is Op.Write -> {
                if (!PenProtocol.isAllowedWrite(op.frame)) return false
                val ch = findChar(g, PenProtocol.CHR_CMD) ?: return false
                g.writeCharacteristic(ch, op.frame, BluetoothGattCharacteristic.WRITE_TYPE_DEFAULT) == BluetoothStatusCodes.SUCCESS
            }
        }
    }

    private fun onQueueDrained() {
        if (connected) return // a later query write also drains the queue; "connected" is reported once
        connected = true
        attempt = 0
        listener.onState("connected")
        write(PenProtocol.QUERY_BATTERY)
    }

    private fun findChar(g: BluetoothGatt, uuid: UUID): BluetoothGattCharacteristic? =
        g.services.firstNotNullOfOrNull { it.getCharacteristic(uuid) }

    private val callback = object : BluetoothGattCallback() {
        override fun onConnectionStateChange(g: BluetoothGatt, status: Int, newState: Int) {
            if (g !== gatt) { runCatching { g.close() }; return } // a stale client from before a reconnect
            if (status != BluetoothGatt.GATT_SUCCESS || newState == BluetoothProfile.STATE_DISCONNECTED) {
                failAndReconnect("connection state status=$status newState=$newState")
                return
            }
            if (newState == BluetoothProfile.STATE_CONNECTED) g.discoverServices()
        }

        override fun onServicesDiscovered(g: BluetoothGatt, status: Int) {
            if (g !== gatt) return
            if (status != BluetoothGatt.GATT_SUCCESS) { failAndReconnect("service discovery status=$status"); return }
            for (u in PenProtocol.SUBSCRIPTIONS) queue.enqueue(Op.Subscribe(u))
        }

        override fun onDescriptorWrite(g: BluetoothGatt, d: BluetoothGattDescriptor, status: Int) {
            if (g !== gatt) return
            queue.complete(status == BluetoothGatt.GATT_SUCCESS)
        }

        override fun onCharacteristicWrite(g: BluetoothGatt, ch: BluetoothGattCharacteristic, status: Int) {
            if (g !== gatt) return
            queue.complete(status == BluetoothGatt.GATT_SUCCESS)
        }

        override fun onCharacteristicChanged(g: BluetoothGatt, ch: BluetoothGattCharacteristic, value: ByteArray) {
            if (g !== gatt) return
            when (ch.uuid) {
                PenProtocol.CHR_DOUBLE_TAP -> if (PenProtocol.isDoubleTap(value)) listener.onDoubleTap()
                PenProtocol.CHR_NUS_TX -> {
                    val text = value.toString(Charsets.US_ASCII)
                    val pos = PenProtocol.parseSlide(text)
                    if (pos != null) listener.onSlide(pos) else listener.onUnknown(text)
                }
                PenProtocol.CHR_CMD -> {
                    PenProtocol.parseBattery(value)?.let { listener.onBattery(it); return }
                    PenProtocol.parseFirmware(value)?.let { listener.onFirmware(it); return }
                    PenProtocol.parseName(value)?.let { listener.onName(it); return }
                }
            }
        }
    }
}
