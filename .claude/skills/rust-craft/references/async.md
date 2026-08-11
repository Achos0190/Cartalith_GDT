# Async and concurrency

## Ask first whether you need async

Async buys concurrency over *waiting* — sockets, files, timers, thousands of
idle connections. It buys nothing for CPU-bound work, and it costs you a
runtime, `Send + 'static` bounds spreading through your signatures, and a harder
debugging story.

CPU-bound work parallelises with threads: `rayon` for data parallelism,
`std::thread::scope` for a handful of tasks that borrow from the stack. Reach for
async when the program is I/O-bound, not because it sounds faster.

## Never hold a lock across `.await`

This is the defining async bug. A `std::sync::MutexGuard` held across a suspend
point can deadlock the executor: the task parks while holding the lock, and the
thread that would release it is now blocked behind the same lock.

```rust
// wrong — guard lives across the await
let mut state = mutex.lock().unwrap();
state.value = fetch().await;

// right — the lock scope closes before the await
let value = fetch().await;
mutex.lock().unwrap().value = value;
```

When state genuinely must be held across a suspend point, use the runtime's own
async mutex (`tokio::sync::Mutex`), which yields instead of blocking. It is
slower than `std::sync::Mutex`, so use it only for that case.

## Never block the executor

`std::thread::sleep`, blocking file I/O, and long CPU loops stall every task on
that worker thread. Hand blocking work to `tokio::task::spawn_blocking`, and CPU
work to a thread pool such as rayon.

## Cancellation is silent

Dropping a future stops it wherever it was suspended — no unwinding, no cleanup
code, no error. Anything that must complete (flushing a buffer, releasing a
remote lease) belongs in a `Drop` impl or in a task you own, not in the tail of a
future somebody else might drop. `tokio::select!` drops the losing branch, which
makes it the most common place to meet this.

## Picking a channel

| Channel | Shape | Use |
|---|---|---|
| `mpsc` bounded | many senders, one receiver | work queues — the bound is your backpressure |
| `mpsc` unbounded | same, no bound | only when you can prove the producer is slower than the consumer |
| `oneshot` | one value, once | returning a result to a caller who awaits it |
| `broadcast` | every receiver gets every message | events, shutdown signals |
| `watch` | receivers see only the latest | config reloads, status — where stale intermediate values are noise |

Prefer bounded channels. An unbounded channel converts backpressure into memory
growth, which surfaces as an out-of-memory kill far from the cause.

## Spawn vs await

`await` runs the future on the current task, in order. `spawn` hands it to the
runtime to run concurrently and gives you a `JoinHandle`. Spawning requires
`Send + 'static`, which is why a spawned task cannot borrow from its caller —
clone or move what it needs, or restructure so it does not need the borrow.

To run several futures concurrently *without* spawning, join them:

```rust
let (a, b) = tokio::join!(fetch_user(id), fetch_orders(id));
```

## Streams

A `Stream` is an async iterator. The combinators mirror `Iterator`, with one
addition worth knowing: `buffer_unordered(n)` runs up to `n` futures at once and
yields them as they finish, which is the standard way to bound concurrent
requests.
