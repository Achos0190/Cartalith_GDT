package org.cartalith.pen

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Test

/** Protects Ruling EK's write allow-list, the parsers' refusal to guess, and the one-at-a-time queue (client defects 1, 2, 4, 6). */
class PenProtocolTest {
    private fun b(vararg v: Int) = ByteArray(v.size) { v[it].toByte() }

    @Test fun onlyTheThreeQueriesMayBeWritten() {
        assertTrue(PenProtocol.isAllowedWrite(b(0x2C, 0x08, 0x01, 0x01)))
        assertTrue(PenProtocol.isAllowedWrite(b(0x2C, 0x06, 0x01, 0x05)))
        assertTrue(PenProtocol.isAllowedWrite(b(0x2C, 0x26, 0x01, 0x0B)))
        // enable/disable, start vibration, set vibration type, status, protocol version, empty, extended and altered frames
        assertFalse(PenProtocol.isAllowedWrite(b(0x2C, 0x90, 0x01, 0x01, 0x01)))
        assertFalse(PenProtocol.isAllowedWrite(b(0x2C, 0x92, 0x01, 0x02, 0xFF, 0x02)))
        assertFalse(PenProtocol.isAllowedWrite(b(0x2C, 0x94, 0x02, 0x00, 0x01)))
        assertFalse(PenProtocol.isAllowedWrite(b(0x2C, 0x01, 0x01, 0x01)))
        assertFalse(PenProtocol.isAllowedWrite(b(0x2C, 0x25, 0x01, 0x03)))
        assertFalse(PenProtocol.isAllowedWrite(b()))
        assertFalse(PenProtocol.isAllowedWrite(b(0x2C, 0x08, 0x01, 0x01, 0x00)))
        assertFalse(PenProtocol.isAllowedWrite(b(0x2C, 0x08, 0x01, 0x02)))
    }

    @Test fun slideParsesHexInRangeAndRefusesTheRest() {
        assertEquals(0x05, PenProtocol.parseSlide("film 05"))
        assertEquals(0xFF, PenProtocol.parseSlide("film FF"))
        assertEquals(0x80, PenProtocol.parseSlide("film 80\n"))
        assertNull(PenProtocol.parseSlide("film 04"))
        assertNull(PenProtocol.parseSlide("film 100"))
        assertNull(PenProtocol.parseSlide("film"))
        assertNull(PenProtocol.parseSlide("film zz"))
        assertNull(PenProtocol.parseSlide("chg evt 1"))
    }

    @Test fun doubleTapIsFirstByteOne() {
        assertTrue(PenProtocol.isDoubleTap(b(0x01)))
        assertFalse(PenProtocol.isDoubleTap(b(0x00)))
        assertFalse(PenProtocol.isDoubleTap(b()))
    }

    @Test fun responsesParseAndAbsentIsNull() {
        assertEquals(100, PenProtocol.parseBattery(b(0x2C, 0x08, 0x01, 0x01, 0x64)))
        assertEquals(0, PenProtocol.parseBattery(b(0x2C, 0x08, 0x01, 0x01, 0x00)))
        assertNull(PenProtocol.parseBattery(b(0x2C, 0x08, 0x01, 0x01, 0x65)))
        assertNull(PenProtocol.parseBattery(b(0x2C, 0x08, 0x01, 0x01)))
        assertNull(PenProtocol.parseBattery(b(0x2C, 0x06, 0x01, 0x05, 0x64)))
        assertEquals("4D45.03.00.12", PenProtocol.parseFirmware(b(0x2C, 0x06, 0x01, 0x05, 0x4D, 0x45, 0x03, 0x00, 0x12)))
        assertNull(PenProtocol.parseFirmware(b(0x2C, 0x06, 0x01, 0x05)))
        val n = b(0x2C, 0x26, 0x01, 0x0F) + "OnePlus Stylo 2".toByteArray()
        assertEquals("OnePlus Stylo 2", PenProtocol.parseName(n))
    }

    @Test fun backoffIs1_2_5_then30() {
        assertEquals(listOf(1000L, 2000L, 5000L, 30000L, 30000L), (0..4).map { PenProtocol.backoffMs(it) })
    }

    @Test fun exactNameAndFourSubscriptions() {
        assertEquals("OnePlus Stylo 2", PenProtocol.PEN_NAME)
        assertEquals(4, PenProtocol.SUBSCRIPTIONS.distinct().size)
    }

    @Test fun queueRunsOneAtATimeAndDrainsOnlyAfterAllConfirmed() {
        val started = mutableListOf<Int>(); var drained = 0; val failed = mutableListOf<Int>()
        val q = OpQueue<Int>({ started += it; true }, { drained++ }, { failed += it })
        listOf(1, 2, 3, 4).forEach { q.enqueue(it) }
        assertEquals(listOf(1), started)
        assertEquals(0, drained)
        q.complete(true); assertEquals(listOf(1, 2), started)
        q.complete(true); q.complete(true); assertEquals(0, drained)
        q.complete(true); assertEquals(1, drained); assertTrue(failed.isEmpty())
    }

    @Test fun queueFailureClearsAndNeverDrains() {
        val started = mutableListOf<Int>(); var drained = 0; val failed = mutableListOf<Int>()
        val q = OpQueue<Int>({ started += it; true }, { drained++ }, { failed += it })
        listOf(1, 2, 3).forEach { q.enqueue(it) }
        q.complete(true); q.complete(false)
        assertEquals(listOf(2), failed); assertEquals(0, drained); assertEquals(0, q.waiting())
        q.complete(true)
        assertEquals(listOf(1, 2), started)
    }

    @Test fun refusedStartIsAFailure() {
        var failed = 0
        val q = OpQueue<Int>({ false }, { fail("drained") }, { failed++ })
        q.enqueue(1)
        assertEquals(1, failed)
    }
}
