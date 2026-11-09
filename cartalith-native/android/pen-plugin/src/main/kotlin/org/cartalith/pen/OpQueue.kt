package org.cartalith.pen

/**
 * A strictly one-at-a-time operation queue for Android's GATT client, as a pure state machine with no Android types
 * so the JVM unit tests drive it.
 *
 * WHY (client defect 1, proposal section 2.4): Android runs one GATT operation at a time and issues made before the
 * previous one's callback are refused, so the pack's loop of four back-to-back descriptor writes typically kept only
 * the first subscription. Here an operation starts only after the previous one reports `complete`.
 *
 * Contract: [enqueue] adds work; [complete] reports the running operation's outcome and starts the next. [start]
 * is called with each operation to issue and returns whether the stack accepted it; a refusal counts as a failed
 * completion immediately. [onDrained] runs once when the queue empties after at least one success and no failure
 * (this is when the client may report "connected": defect 2). Any failure calls [onFailure] once and clears the queue.
 *
 * MUST NEVER: run two operations at once, or report drained after a failure.
 */
class OpQueue<T>(
    private val start: (T) -> Boolean,
    private val onDrained: () -> Unit,
    private val onFailure: (T) -> Unit,
) {
    private val pending = ArrayDeque<T>()
    private var running: T? = null
    private var failed = false
    private var anyDone = false

    /** True while an operation is awaiting its callback. */
    fun busy(): Boolean = running != null

    /** Operations not yet started. */
    fun waiting(): Int = pending.size

    fun enqueue(op: T) {
        if (failed) return
        pending.addLast(op)
        pump()
    }

    /** Report the running operation's result. Ignored when nothing runs (a stray callback must not advance the queue). */
    fun complete(success: Boolean) {
        val op = running ?: return
        running = null
        if (!success) { fail(op); return }
        anyDone = true
        pump()
    }

    /** Drop everything (disconnect). The next [enqueue] starts fresh. */
    fun reset() {
        pending.clear(); running = null; failed = false; anyDone = false
    }

    private fun pump() {
        if (running != null) return
        val next = pending.removeFirstOrNull()
        if (next == null) {
            if (anyDone && !failed) { anyDone = false; onDrained() }
            return
        }
        running = next
        if (!start(next)) { running = null; fail(next) }
    }

    private fun fail(op: T) {
        failed = true
        pending.clear()
        onFailure(op)
    }
}
