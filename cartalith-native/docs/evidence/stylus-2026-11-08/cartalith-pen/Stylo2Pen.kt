package cartalith.pen

import android.annotation.SuppressLint
import android.bluetooth.BluetoothDevice
import android.bluetooth.BluetoothGatt
import android.bluetooth.BluetoothGattCallback
import android.bluetooth.BluetoothGattCharacteristic
import android.bluetooth.BluetoothGattDescriptor
import android.bluetooth.BluetoothManager
import android.bluetooth.BluetoothProfile
import android.content.Context
import java.util.UUID

/**
 * Minimal, non-root BLE client for the OnePlus Stylo 2, for the Cartalith project.
 *
 * Exposes the pen features reachable by an ordinary app: double-tap, the capacitive-film
 * slide position, battery/firmware/name, and vibration *configuration*. It does NOT (and cannot)
 * trigger the haptic buzz — that path is signature-walled (see protocol.md).
 *
 * Drawing input (x/y, pressure, tilt, barrel button) is NOT handled here — that is standard
 * Android `MotionEvent`; see stylus-input.md.
 *
 * Requires the runtime permission BLUETOOTH_CONNECT (API 31+). minSdk 33 (modern GATT API).
 * Callbacks fire on the Binder thread — marshal to your UI thread as needed.
 */
@SuppressLint("MissingPermission")
class Stylo2Pen(private val context: Context) {

    // ---- public callbacks ----
    /** Double-tap of the pen. */
    var onDoubleTap: (() -> Unit)? = null
    /** Capacitive-film contact position 0x00..0xFF (direction = delta over successive calls). */
    var onSlide: ((position: Int) -> Unit)? = null
    /** Raw text events from the Nordic UART channel (e.g. "chg evt ..."). */
    var onTextEvent: ((String) -> Unit)? = null
    /** Connection state. */
    var onConnected: (() -> Unit)? = null
    var onDisconnected: (() -> Unit)? = null
    /** Any command response: (cmdId, full payload bytes after the 2C header). */
    var onResponse: ((cmdId: Int, data: ByteArray) -> Unit)? = null

    private val manager = context.getSystemService(Context.BLUETOOTH_SERVICE) as BluetoothManager
    private var gatt: BluetoothGatt? = null

    /** The Stylo 2 among the system's bonded devices, or null. */
    fun bondedPen(): BluetoothDevice? =
        manager.adapter?.bondedDevices?.firstOrNull { it.name?.contains("Stylo", true) == true }

    /** Connect to the bonded pen (a connected pen does not advertise, so we connect by address). */
    fun connect(device: BluetoothDevice? = bondedPen()): Boolean {
        val dev = device ?: return false
        gatt = dev.connectGatt(context, false, callback, BluetoothDevice.TRANSPORT_LE)
        return true
    }

    fun disconnect() {
        gatt?.let { runCatching { it.disconnect(); it.close() } }
        gatt = null
    }

    // ---- commands (write to 58480001; response arrives as a notification there) ----
    fun queryFirmware() = writeCmd(byteArrayOf(0x2C, 0x06, 0x01, 0x05))
    fun queryName() = writeCmd(byteArrayOf(0x2C, 0x26, 0x01, 0x0B))
    fun queryBattery() = writeCmd(byteArrayOf(0x2C, 0x08, 0x01, 0x01))
    /** Configure the writing-feedback feel. NOTE: configures only; does not fire the motor. */
    fun setVibrationType(type: Int, level: Int) =
        writeCmd(byteArrayOf(0x2C, 0x94.toByte(), 0x02, type.toByte(), (level + 1).toByte()))

    private fun writeCmd(bytes: ByteArray) {
        val g = gatt ?: return
        val ch = findChar(g, CHR_CMD) ?: return
        g.writeCharacteristic(ch, bytes, BluetoothGattCharacteristic.WRITE_TYPE_DEFAULT)
    }

    /** Find a characteristic by UUID across all services (service grouping isn't assumed). */
    private fun findChar(g: BluetoothGatt, uuid: UUID): BluetoothGattCharacteristic? =
        g.services.firstNotNullOfOrNull { it.getCharacteristic(uuid) }

    // ---- GATT plumbing ----
    private val callback = object : BluetoothGattCallback() {
        override fun onConnectionStateChange(g: BluetoothGatt, status: Int, newState: Int) {
            when (newState) {
                BluetoothProfile.STATE_CONNECTED -> g.discoverServices()
                BluetoothProfile.STATE_DISCONNECTED -> onDisconnected?.invoke()
            }
        }

        override fun onServicesDiscovered(g: BluetoothGatt, status: Int) {
            if (status != BluetoothGatt.GATT_SUCCESS) return
            // Enable notifications on the characteristics that carry pen events.
            for (uuid in listOf(CHR_CMD, CHR_DOUBLE_TAP, CHR_STATE, CHR_NUS_TX)) enableNotify(g, uuid)
            onConnected?.invoke()
        }

        override fun onCharacteristicChanged(
            g: BluetoothGatt,
            ch: BluetoothGattCharacteristic,
            value: ByteArray,
        ) {
            when (ch.uuid) {
                CHR_DOUBLE_TAP -> if (value.isNotEmpty() && value[0].toInt() == 0x01) onDoubleTap?.invoke()
                CHR_NUS_TX -> {
                    val text = value.toString(Charsets.US_ASCII)
                    if (text.startsWith("film ")) {
                        text.removePrefix("film ").trim().toIntOrNull(16)?.let { onSlide?.invoke(it) }
                    } else {
                        onTextEvent?.invoke(text)
                    }
                }
                CHR_CMD -> if (value.size >= 2 && value[0].toInt() and 0xFF == 0x2C) {
                    onResponse?.invoke(value[1].toInt() and 0xFF, value.copyOfRange(1, value.size))
                }
            }
        }
    }

    private fun enableNotify(g: BluetoothGatt, chr: UUID) {
        val ch = findChar(g, chr) ?: return
        g.setCharacteristicNotification(ch, true)
        val cccd = ch.getDescriptor(CCCD) ?: return
        g.writeDescriptor(cccd, BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE)
    }

    companion object {
        val SVC_CMD: UUID = UUID.fromString("58480000-5a6a-1122-3300-004095112200")
        val CHR_CMD: UUID = UUID.fromString("58480001-5a6a-1122-3300-004095112200")
        val SVC_NUS: UUID = UUID.fromString("6e400001-b5a3-f393-e0a9-e50e24dcca9e")
        val CHR_NUS_TX: UUID = UUID.fromString("6e400003-b5a3-f393-e0a9-e50e24dcca9e")
        // Vendor notify characteristics (double-tap, state) sit in the 2a69-11ec group.
        val SVC_VENDOR: UUID = UUID.fromString("18093398-2a69-11ec-8d3d-0242ac130003")
        val CHR_DOUBLE_TAP: UUID = UUID.fromString("180938ac-2a69-11ec-8d3d-0242ac130003")
        val CHR_STATE: UUID = UUID.fromString("18093046-2a69-11ec-8d3d-0242ac130003")
        val CCCD: UUID = UUID.fromString("00002902-0000-1000-8000-00805f9b34fb")
    }
}
