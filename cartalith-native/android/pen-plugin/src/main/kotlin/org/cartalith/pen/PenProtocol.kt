package org.cartalith.pen

import java.util.UUID

/**
 * The Stylo 2's wire facts and the plugin's write allow-list, as pure Kotlin (no Android types) so the JVM unit
 * tests can pin every literal. Source of the facts: the owner's integration pack (`protocol.md`, owner-measured, not
 * independently reproduced); see STYLO2_PEN_PROPOSAL.md section 2.4 for where the pack disagrees with itself.
 *
 * MUST NEVER: grow a frame that enables or disables the pen (`2C 90`), starts a vibration (`2C 92`) or sets a
 * vibration type (`2C 94`), or addresses a firmware-update service (Ruling EK keeps Ruling EE's no-haptics line).
 */
object PenProtocol {
    /** Command characteristic: commands go in, responses come back as notifications on the same one. */
    val CHR_CMD: UUID = UUID.fromString("58480001-5a6a-1122-3300-004095112200")
    /** Nordic UART TX: text events (`film <pos>`, `chg evt ...`). */
    val CHR_NUS_TX: UUID = UUID.fromString("6e400003-b5a3-f393-e0a9-e50e24dcca9e")
    /** Double-tap notification (payload `01`). */
    val CHR_DOUBLE_TAP: UUID = UUID.fromString("180938ac-2a69-11ec-8d3d-0242ac130003")
    /** Pen state echo. Subscribed (the pack does), never written. */
    val CHR_STATE: UUID = UUID.fromString("18093046-2a69-11ec-8d3d-0242ac130003")
    /** Client Characteristic Configuration descriptor (Bluetooth SIG 0x2902). */
    val CCCD: UUID = UUID.fromString("00002902-0000-1000-8000-00805f9b34fb")

    /** The notification subscriptions, in the order the queue runs them (one at a time). */
    val SUBSCRIPTIONS: List<UUID> = listOf(CHR_CMD, CHR_DOUBLE_TAP, CHR_STATE, CHR_NUS_TX)

    /** The exact bonded-device name. Exact, not "contains" (client defect 5): a third-party pen with Stylo in its name must not match. */
    const val PEN_NAME = "OnePlus Stylo 2"

    /** The three query frames, the ONLY bytes the plugin may ever write (Ruling EK). Pack: `protocol.md` command table. */
    val QUERY_BATTERY: ByteArray = byteArrayOf(0x2C, 0x08, 0x01, 0x01)
    val QUERY_FIRMWARE: ByteArray = byteArrayOf(0x2C, 0x06, 0x01, 0x05)
    val QUERY_NAME: ByteArray = byteArrayOf(0x2C, 0x26, 0x01, 0x0B)

    private val ALLOWED = listOf(QUERY_BATTERY, QUERY_FIRMWARE, QUERY_NAME)

    /** True only for one of the three query frames, byte for byte. The single gate every write passes. */
    fun isAllowedWrite(frame: ByteArray): Boolean = ALLOWED.any { it.contentEquals(frame) }

    /** Slide range in the pack: `0x05` one end to `0xFF` the other. Anything outside is dropped, never clamped into a plausible value. */
    const val SLIDE_MIN = 0x05
    const val SLIDE_MAX = 0xFF

    /**
     * Parse a Nordic-UART line. Returns the slide position for `film <hex>`, or null for anything else (including
     * `film` with a missing, non-hex or out-of-range token: an unknown frame is dropped, never guessed, proposal
     * section 7 "Firmware variance"). HEX is the pack client's reading; whether the pen sends hex or decimal is
     * proposal section 2.4 row 3, settled by one captured line in SP-3.
     */
    fun parseSlide(text: String): Int? {
        if (!text.startsWith("film ")) return null
        val v = text.removePrefix("film ").trim().toIntOrNull(16) ?: return null
        return if (v in SLIDE_MIN..SLIDE_MAX) v else null
    }

    /** True for the double-tap notification payload: first byte `01`. */
    fun isDoubleTap(value: ByteArray): Boolean = value.isNotEmpty() && value[0].toInt() == 0x01

    /**
     * Battery percent from a response notification `2C 08 01 01 <pct>`, or null if the frame is not that response
     * or the byte is not 0..100 (an absent value is null, never 0).
     */
    fun parseBattery(value: ByteArray): Int? {
        if (value.size < 5) return null
        if ((value[0].toInt() and 0xFF) != 0x2C || value[1].toInt() != 0x08) return null
        val pct = value[4].toInt() and 0xFF
        return if (pct in 0..100) pct else null
    }

    /** Firmware string from `2C 06 01 05 <b0..b4>`: the pack's example `4D 45 03 00 12` reads `4D45.03.00.12`. Null if not that response. */
    fun parseFirmware(value: ByteArray): String? {
        if (value.size < 9 || (value[0].toInt() and 0xFF) != 0x2C || value[1].toInt() != 0x06) return null
        fun h(i: Int) = "%02X".format(value[i].toInt() and 0xFF)
        return h(4) + h(5) + "." + h(6) + "." + h(7) + "." + h(8)
    }

    /** Name string from `2C 26 01 <len> <ascii...>`, or null. */
    fun parseName(value: ByteArray): String? {
        if (value.size < 5 || (value[0].toInt() and 0xFF) != 0x2C || value[1].toInt() != 0x26) return null
        return value.copyOfRange(4, value.size).toString(Charsets.US_ASCII).trim { it <= ' ' }.ifEmpty { null }
    }

    /** Reconnect back-off, proposal section 6.2: 1 s, 2 s, 5 s, then every 30 s. `attempt` counts from 0. */
    fun backoffMs(attempt: Int): Long = when {
        attempt <= 0 -> 1_000L
        attempt == 1 -> 2_000L
        attempt == 2 -> 5_000L
        else -> 30_000L
    }
}
