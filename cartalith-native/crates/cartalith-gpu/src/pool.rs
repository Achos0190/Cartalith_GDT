//! The VRAM buffer pool (`HARDWARE_ACCELERATION.md` §14; `OUTSTANDING_WORK.md`
//! §2.6, built on Ruling AZ, `LARGE_ITEM_RULINGS.md`, 2026-09-28).
//!
//! Every `dispatch_gpu_*` in this crate used to create each of its buffers and
//! drop them on return. They now take them from the [`BufferPool`] of the
//! device they run on and hand them back on drop, so a buffer allocated by
//! one stage serves the next stage, and the next generation, at the same size.
//!
//! # Keys
//!
//! A buffer is keyed by its **exact byte size** and one of three canonical
//! [`Kind`]s. The kinds widen the old per-call usage flags to a common set
//! (every storage buffer carries `STORAGE | COPY_SRC | COPY_DST`) so that one
//! stage's output buffer can be another stage's input: widening a usage never
//! changes what a kernel computes, and `COPY_DST` is what lets a reused buffer
//! be rewritten with `Queue::write_buffer` instead of re-created.
//!
//! # Retention policy (conservative by default)
//!
//! 1. **Current grid size only.** Each whole-grid dispatch declares its grid
//!    ([`BufferPool::begin_grid`], in cells). A declaration naming a different
//!    cell count than the last one **releases everything** first. The coarse
//!    weather grid does not declare one: it is a function of the world grid, so
//!    it rides along under the world grid's epoch and is released with it.
//! 2. **Released with the device.** The pool lives on the device ([`crate::GpuDevice`]
//!    and every context built from it share one `Arc`). When the process-wide
//!    device cache replaces a device (preferences changed, device lost), the
//!    last handle drops and the pool drops with it. A **lost** device retains
//!    nothing and serves nothing from then on.
//! 3. **A byte cap, derived rather than chosen**: [`pool_retention_cap_bytes`]
//!    -- `GPU_GRID_BUFFERS` full grids, the figure `multi.rs` already derives as
//!    the heaviest `generate_terrain` stage's concurrent working set. Holding at
//!    most that much between generations means the pool never keeps more VRAM
//!    idle than one stage of the generation that just finished had to hold at
//!    once anyway. A buffer whose return would take the pool over the cap is
//!    simply dropped.
//! 4. **Wired through the existing budget.** When the user has set
//!    `GpuPreferences::vram_budget_bytes`, the cap becomes
//!    `min(derived, budget - working set)`, so the retained buffers plus the
//!    next generation's own working-set estimate still fit inside the budget.
//!
//! **What the cap is not.** `wgpu` 30 cannot report a device's VRAM size (see
//! `multi.rs`'s module doc), so no cap here can be a fraction of the card. The
//! derived figure is relative to what this pipeline already needed at its peak
//! on this same device a moment earlier -- it cannot promise that another
//! process has not taken that room since. That part is a guess, and it is
//! stated rather than hidden. Two further honest consequences: a pool at its
//! cap sits idle **beside** the next stage's own buffers, so the in-generation
//! peak can rise by up to the cap (at most 2 x `GPU_GRID_BUFFERS` grids); and
//! stages wider than the cap (settlement suitability's 22-plane input, resource
//! potentials' 15-plane output) are never retained, only served from it when a
//! narrower buffer of the exact same size happens to be free.
//!
//! # Correctness: which buffers are cleared, and why
//!
//! A freshly created `wgpu` buffer is zero. A reused one holds whatever its
//! last user left in it. So every pooled buffer is either **fully rewritten**
//! before a kernel reads it, or **explicitly cleared**. Derived by reading each
//! kernel's bindings and entry points (`shaders/*.wgsl`), not taken from the
//! backlog row's example:
//!
//! - Every buffer that used to be `create_buffer_init`: acquired through
//!   [`BufferPool::init`], which rewrites the whole buffer (`write_buffer` of
//!   exactly `size` bytes) on reuse.
//! - Every output / ping-pong / staging buffer that used to be a bare
//!   `create_buffer`, **except the three below**, is written in full before it
//!   is read: each kernel writes its own cell of every output unconditionally
//!   after its bounds check (warp, heterogeneity, height, resistance, stress's
//!   three planes, biome's packed words, carrying, resources' fifteen planes,
//!   suitability, both noise pilots); a ping-pong target is written in full by
//!   the first pass before the second pass reads it (blur's `box_h`, thermal's
//!   pass 0, JFA's pass 0 into `nearest_b`/`best_d2_b`); flow's `recv`/`ptr`
//!   are written by `dir_main` and `ptr_next` by `scatter_main` before any pass
//!   reads them; weather's `w2` is written by `advect_main` before
//!   `deposit_main` reads it; staging buffers are overwritten by a full-size
//!   `copy_buffer_to_buffer`.
//! - **Cleared, because the kernel reads them before writing them and relied
//!   on the zero a fresh buffer had:**
//!   - `gpu_flow`'s `delta` -- `scatter_main` `atomicAdd`s into it before
//!     `merge_main` ever exchanges it back to zero. (A clean run leaves it zero,
//!     but a buffer of the same size last used by another stage does not.)
//!   - `gpu_weather`'s `rain` -- `deposit_main` computes `rain * 0.55 + ...`,
//!     and with `iters == 0` the buffer is read back untouched.
//!   - `gpu_resources`' absent optional input planes (shear, flow, volcanic;
//!     boundary type, biome) -- the shader has no presence flags; "an absent
//!     optional input is an all-zero plane" is its own header's contract.
//!
//! The equivalence tests at the bottom of this file run every pooled kernel
//! twice with different inputs, pooled against unpooled, and then again with
//! every retained buffer deliberately filled with garbage.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use wgpu::util::DeviceExt;

use crate::multi::GPU_GRID_BUFFERS;

/// The canonical usage a pooled buffer is created with.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Kind {
    /// Any storage binding, read-only or read-write.
    Storage,
    /// A `MAP_READ` readback target.
    Staging,
    /// A uniform parameter block.
    Uniform,
}

impl Kind {
    /// The fixed `wgpu::BufferUsages` for this kind -- widened past what any
    /// one call site needs (module doc, "Keys") so a buffer of a given kind
    /// and size can serve any caller that asks for that kind and size.
    const fn usage(self) -> wgpu::BufferUsages {
        match self {
            Self::Storage => wgpu::BufferUsages::STORAGE
                .union(wgpu::BufferUsages::COPY_SRC)
                .union(wgpu::BufferUsages::COPY_DST),
            // MAP_READ may only be combined with COPY_DST.
            Self::Staging => wgpu::BufferUsages::MAP_READ.union(wgpu::BufferUsages::COPY_DST),
            Self::Uniform => wgpu::BufferUsages::UNIFORM.union(wgpu::BufferUsages::COPY_DST),
        }
    }
}

/// The most bytes the pool may keep idle for a grid of `cells` cells.
///
/// `GPU_GRID_BUFFERS` full `f32` grids -- `multi.rs` derives that figure as the
/// concurrent working set of the heaviest `generate_terrain` stage, and
/// [`crate::gpu_working_set_bytes`] is the same product. With a user budget set
/// (`budget_bytes != 0`), the cap shrinks so that retained plus working set
/// stays inside it: `min(derived, budget - working set)`, and `0` when the
/// budget leaves no room.
#[must_use]
pub const fn pool_retention_cap_bytes(cells: u64, budget_bytes: u64) -> u64 {
    let derived = cells.saturating_mul(4).saturating_mul(GPU_GRID_BUFFERS);
    if budget_bytes == 0 {
        return derived;
    }
    let room = budget_bytes.saturating_sub(derived);
    if room < derived { room } else { derived }
}

/// A snapshot of one pool's counters.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BufferPoolStats {
    /// Acquisitions served from a retained buffer.
    pub hits: u64,
    /// Acquisitions that created a fresh buffer (every acquisition, when the
    /// pool is disabled).
    pub misses: u64,
    /// Bytes of those fresh buffers.
    pub fresh_bytes: u64,
    /// Bytes currently held idle.
    pub retained_bytes: u64,
    /// Buffers currently held idle.
    pub retained_buffers: u64,
    /// The cap in force ([`pool_retention_cap_bytes`]).
    pub cap_bytes: u64,
    /// The grid (in cells) the retained buffers belong to; `None` before any
    /// whole-grid dispatch, or after [`BufferPool::release_all`].
    pub grid_cells: Option<u64>,
}

/// The mutable half of a [`BufferPool`]: which grid it is retaining for, the
/// cap that grid was given, the free buffers themselves, and the running
/// counters [`BufferPoolStats`] is built from. Held behind one `Mutex` so a
/// take/give-back pair from two threads never interleaves.
#[derive(Default)]
struct State {
    grid_cells: Option<u64>,
    cap_bytes: u64,
    retained_bytes: u64,
    free: HashMap<(u64, Kind), Vec<wgpu::Buffer>>,
    hits: u64,
    misses: u64,
    fresh_bytes: u64,
}

impl State {
    /// Drop every retained buffer and zero the retained-bytes counter. Does
    /// not touch `grid_cells`/`cap_bytes` or the hit/miss counters -- callers
    /// that need those reset (`release_all`) clear them separately.
    fn flush(&mut self) {
        self.free.clear();
        self.retained_bytes = 0;
    }
}

/// One device's reusable buffers. See the module doc for the policy.
pub struct BufferPool {
    device: wgpu::Device,
    queue: wgpu::Queue,
    lost: Arc<AtomicBool>,
    enabled: AtomicBool,
    state: Mutex<State>,
}

impl BufferPool {
    /// Build an empty, enabled pool over an already-open device/queue, sharing
    /// the same `lost` flag the device itself sets so the pool can see a lost
    /// device without polling it.
    pub(crate) fn new(device: wgpu::Device, queue: wgpu::Queue, lost: Arc<AtomicBool>) -> Arc<Self> {
        Arc::new(Self { device, queue, lost, enabled: AtomicBool::new(true), state: Mutex::new(State::default()) })
    }

    /// Lock [`State`]. A poisoned lock only means another thread panicked
    /// mid-update; the map is still a valid set of buffers, and a panic must
    /// not spread here.
    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Whether reuse is both turned on and the device has not been marked lost.
    fn active(&self) -> bool {
        self.enabled.load(Ordering::Relaxed) && !self.lost.load(Ordering::Relaxed)
    }

    /// Turn reuse on or off for this device. Off releases everything held and
    /// makes every later acquisition a fresh allocation -- the pre-pool
    /// behaviour, kept reachable for measurement and for the equivalence tests.
    pub fn set_enabled(&self, on: bool) {
        self.enabled.store(on, Ordering::Relaxed);
        if !on {
            self.state().flush();
        }
    }

    /// Whether reuse is currently turned on (irrespective of device loss).
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Drop every retained buffer and forget the current grid.
    pub fn release_all(&self) {
        let mut s = self.state();
        s.flush();
        s.grid_cells = None;
        s.cap_bytes = 0;
    }

    /// A snapshot of this pool's counters, for diagnostics and for the
    /// equivalence tests to assert reuse actually happened.
    #[must_use]
    pub fn stats(&self) -> BufferPoolStats {
        let s = self.state();
        BufferPoolStats {
            hits: s.hits,
            misses: s.misses,
            fresh_bytes: s.fresh_bytes,
            retained_bytes: s.retained_bytes,
            retained_buffers: s.free.values().map(|v| v.len() as u64).sum(),
            cap_bytes: s.cap_bytes,
            grid_cells: s.grid_cells,
        }
    }

    /// Declare the whole grid the next dispatch runs over. A different cell
    /// count than the last declaration releases everything first (policy 1).
    pub(crate) fn begin_grid(&self, cells: u64) {
        let budget = crate::preferences().vram_budget_bytes;
        let active = self.active();
        let mut s = self.state();
        if s.grid_cells != Some(cells) || !active {
            s.flush();
        }
        s.grid_cells = Some(cells);
        s.cap_bytes = pool_retention_cap_bytes(cells, budget);
        if s.retained_bytes > s.cap_bytes {
            s.flush();
        }
    }

    /// Pop a matching retained buffer if one exists and the pool is active,
    /// else record a miss (and the bytes a fresh allocation will cost) and
    /// return `None`.
    fn take(&self, size: u64, kind: Kind) -> Option<wgpu::Buffer> {
        let mut s = self.state();
        if !self.active() {
            s.misses += 1;
            s.fresh_bytes += size;
            return None;
        }
        match s.free.get_mut(&(size, kind)).and_then(Vec::pop) {
            Some(b) => {
                s.retained_bytes -= size;
                s.hits += 1;
                Some(b)
            }
            None => {
                s.misses += 1;
                s.fresh_bytes += size;
                None
            }
        }
    }

    /// [`PooledBuffer::drop`]'s call: retain `buf` if the pool is active,
    /// a grid is currently declared, and doing so would not exceed the cap
    /// (retention policies 1 and 3); otherwise let it drop and be freed.
    fn give_back(&self, buf: wgpu::Buffer, size: u64, kind: Kind) {
        if !self.active() {
            return;
        }
        let mut s = self.state();
        if s.grid_cells.is_none() || s.retained_bytes + size > s.cap_bytes {
            return; // dropped: freed once the GPU is done with it
        }
        s.retained_bytes += size;
        s.free.entry((size, kind)).or_default().push(buf);
    }

    /// A buffer of `size` bytes whose **contents are unspecified** -- only for
    /// a buffer the caller's kernel writes in full before anything reads it,
    /// or one the caller clears itself (see the module doc's list).
    pub(crate) fn empty(self: &Arc<Self>, label: &str, size: u64, kind: Kind) -> PooledBuffer {
        let buf = self.take(size, kind).unwrap_or_else(|| {
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage: kind.usage(),
                mapped_at_creation: false,
            })
        });
        PooledBuffer { buf: Some(buf), size, kind, pool: Some(Arc::clone(self)) }
    }

    /// A buffer holding exactly `contents` -- the pooled `create_buffer_init`.
    /// A reused buffer is rewritten in full, so nothing of its last user
    /// survives.
    pub(crate) fn init(self: &Arc<Self>, label: &str, contents: &[u8], kind: Kind) -> PooledBuffer {
        let size = contents.len() as u64;
        // `create_buffer_init` pads a size that is not a multiple of 4, which
        // would break exact-size keying; nothing here uploads one, and a zero
        // size is never reused either.
        if size == 0 || !size.is_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT) {
            let buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents,
                usage: kind.usage(),
            });
            return PooledBuffer { buf: Some(buf), size, kind, pool: None };
        }
        let buf = match self.take(size, kind) {
            Some(b) => {
                self.queue.write_buffer(&b, 0, contents);
                b
            }
            None => self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents,
                usage: kind.usage(),
            }),
        };
        PooledBuffer { buf: Some(buf), size, kind, pool: Some(Arc::clone(self)) }
    }

    /// Test-only: overwrite every retained buffer with `byte`, so a kernel
    /// that reads a reused buffer before writing it reads garbage rather than
    /// a plausible leftover.
    #[cfg(test)]
    pub(crate) fn poison_retained(&self, byte: u8) {
        let s = self.state();
        for ((size, _), bufs) in &s.free {
            let fill = vec![byte; *size as usize];
            for b in bufs {
                self.queue.write_buffer(b, 0, &fill);
            }
        }
        drop(s);
        self.queue.submit(None);
        let _ = self.device.poll(wgpu::PollType::wait_indefinitely());
    }
}

/// A buffer on loan from a [`BufferPool`]; returned to it on drop.
pub(crate) struct PooledBuffer {
    buf: Option<wgpu::Buffer>,
    size: u64,
    kind: Kind,
    pool: Option<Arc<BufferPool>>,
}

impl std::ops::Deref for PooledBuffer {
    type Target = wgpu::Buffer;
    /// Borrow the wrapped buffer. Only `drop` ever takes ownership of it, so
    /// `self.buf` is always `Some` here.
    fn deref(&self) -> &wgpu::Buffer {
        self.buf.as_ref().expect("pooled buffer present until drop")
    }
}

impl Drop for PooledBuffer {
    /// Return the buffer to its pool ([`BufferPool::give_back`]), or simply
    /// let it drop if it was never pool-backed (`init`'s misaligned-size path).
    fn drop(&mut self) {
        if let (Some(buf), Some(pool)) = (self.buf.take(), self.pool.as_ref()) {
            pool.give_back(buf, self.size, self.kind);
        }
    }
}

/// The pool's own retention-cap arithmetic, plus the equivalence contract
/// ([`check_equivalence`]) every pooled GPU kernel in this crate must meet:
/// pooled output bit-identical to unpooled, a genuine reuse (not two fresh
/// runs) between the two calls, and no leak from a retained buffer another
/// kernel dirtied. Kernel tests skip cleanly with no GPU; the cap-arithmetic
/// test needs none.
#[cfg(test)]
mod tests {
    use super::*;

    /// Literals, not the constant against itself: 10 grids of 4-byte cells.
    #[test]
    fn retention_cap_is_ten_f32_grids_and_respects_the_budget() {
        assert_eq!(pool_retention_cap_bytes(1024 * 1024, 0), 40 * 1024 * 1024);
        assert_eq!(pool_retention_cap_bytes(4096 * 4096, 0), 640 * 1024 * 1024);
        // A budget of 1 GiB at 4096²: working set 640 MiB, room 384 MiB.
        assert_eq!(pool_retention_cap_bytes(4096 * 4096, 1024 * 1024 * 1024), 384 * 1024 * 1024);
        // A generous budget does not raise the cap above the derived figure.
        assert_eq!(pool_retention_cap_bytes(1024 * 1024, 1 << 40), 40 * 1024 * 1024);
        // A budget the working set alone fills leaves nothing to retain.
        assert_eq!(pool_retention_cap_bytes(4096 * 4096, 600 * 1024 * 1024), 0);
    }

    /// Bits of every output value, so NaN compares equal to itself and -0.0
    /// does not compare equal to 0.0.
    fn bits(v: &[f32]) -> Vec<u32> {
        v.iter().map(|x| x.to_bits()).collect()
    }

    /// A small deterministic generator, so inputs differ between runs without
    /// a dependency.
    fn field(n: usize, seed: u32, lo: f32, hi: f32) -> Vec<f32> {
        let mut s = seed.wrapping_mul(2_654_435_761).wrapping_add(1);
        (0..n)
            .map(|_| {
                s ^= s << 13;
                s ^= s >> 17;
                s ^= s << 5;
                lo + (hi - lo) * ((s >> 8) as f32 / (1u32 << 24) as f32)
            })
            .collect()
    }

    /// A deterministic u8 field in `0..=max`, built by truncating [`field`]'s
    /// f32 output -- for the packed class-id/mask inputs the affordance and
    /// stress kernels take.
    fn bytes_field(n: usize, seed: u32, max: u8) -> Vec<u8> {
        field(n, seed, 0.0, f32::from(max) + 0.999).iter().map(|&v| v as u8).collect()
    }

    /// The equivalence contract every pooled kernel must meet:
    ///
    /// 1. unpooled, input A then input B -- the references;
    /// 2. pooled from empty, A then B -- bit-identical to the references, and
    ///    B must actually have been served from the pool (so this is a reuse
    ///    test, not two fresh runs): **run 2 must not see run 1's data**;
    /// 3. every retained buffer overwritten with garbage, then B again --
    ///    still bit-identical. This is the case that catches a buffer reused
    ///    across *different* kernels, which (2) cannot: flow's `delta` ends a
    ///    clean flow run at zero, so only a buffer another stage left dirty
    ///    exposes a missing clear.
    fn check_equivalence<I>(name: &str, pool: &BufferPool, grid_cells: u64, a: &I, b: &I, run: impl Fn(&I) -> Vec<u32>) {
        pool.set_enabled(false);
        let ref_a = run(a);
        let ref_b = run(b);
        assert!(!ref_a.is_empty(), "{name}: empty output proves nothing");
        assert_ne!(ref_a, ref_b, "{name}: the two inputs must give different outputs, or the leak case proves nothing");

        pool.set_enabled(true);
        pool.begin_grid(grid_cells);
        assert_eq!(pool.stats().retained_bytes, 0, "{name}: enabling starts from an empty pool");
        assert_eq!(run(a), ref_a, "{name}: pooled run 1 differs from unpooled");
        assert!(pool.stats().retained_bytes > 0, "{name}: run 1 retained nothing, so run 2 cannot test reuse");
        let hits_before = pool.stats().hits;
        assert_eq!(run(b), ref_b, "{name}: pooled run 2 saw run 1's data");
        assert!(pool.stats().hits > hits_before, "{name}: run 2 was not served from the pool");

        pool.poison_retained(0xA5);
        let hits_before = pool.stats().hits;
        assert_eq!(run(b), ref_b, "{name}: a reused buffer's garbage reached the output");
        assert!(pool.stats().hits > hits_before, "{name}: the poisoned run was not served from the pool");
    }

    /// Open the shared device, or `None` (printing why) on a machine with no
    /// GPU -- every equivalence test's uniform skip condition.
    fn device() -> Option<crate::GpuDevice> {
        let d = crate::init_gpu_shared_device().ok();
        if d.is_none() {
            eprintln!("no GPU adapter on this machine -- pooled-equivalence test skipped");
        }
        d
    }

    const W: u32 = 61; // deliberately not a multiple of the 8x8 workgroup, so every kernel's bounds check is on the path
    const H: u32 = 37; // co-prime-ish with W for the same reason: an x/y transpose bug would still be caught
    const N: usize = (W * H) as usize; // total cells at the fixture grid above

    /// [`check_equivalence`] over `dispatch_gpu_warp` -- the simplest kernel
    /// here (no upstream fields, just `(x, y, seed)`).
    #[test]
    fn pooled_warp_matches_unpooled() {
        // Protects: pooled warp output is bit-identical to unpooled, run 2
        // is genuinely served from the pool rather than freshly computed, and
        // a poisoned retained buffer does not leak into the result.
        let Some(gpu) = device() else { return };
        let ctx = crate::init_gpu_warp_with(&gpu);
        check_equivalence("warp", &gpu.pool, N as u64, &1i32, &2i32, |&seed| {
            let (x, y) = crate::dispatch_gpu_warp(&ctx, W, H, seed, 0.05, 3.0, false).expect("readback");
            [bits(&x), bits(&y)].concat()
        });
    }

    /// [`check_equivalence`] over `dispatch_gpu_heterogeneity`.
    #[test]
    fn pooled_heterogeneity_matches_unpooled() {
        // Protects: pooled heterogeneity output is bit-identical to unpooled
        // across a genuine pool reuse, and survives a poisoned retained buffer.
        let Some(gpu) = device() else { return };
        let ctx = crate::init_gpu_heterogeneity_with(&gpu);
        let input = |s: u32| (field(N, s, 0.0, 1.0), field(N, s + 1, -3.0, 3.0), field(N, s + 2, -3.0, 3.0));
        check_equivalence("heterogeneity", &gpu.pool, N as u64, &input(10), &input(20), |(age, wx, wy)| {
            bits(&crate::dispatch_gpu_heterogeneity(&ctx, W, H, 7, 0.04, false, 3, age, wx, wy).expect("readback"))
        });
    }

    /// [`check_equivalence`] over `dispatch_gpu_gauss_blur`, whose ping-pong
    /// pass is one of the buffers the module doc names as written-before-read
    /// rather than cleared.
    #[test]
    fn pooled_gauss_blur_matches_unpooled() {
        // Protects: pooled blur output is bit-identical to unpooled across a
        // genuine pool reuse, and survives a poisoned retained ping-pong buffer.
        let Some(gpu) = device() else { return };
        let ctx = crate::init_gpu_gauss_blur_with(&gpu);
        check_equivalence("gauss_blur", &gpu.pool, N as u64, &field(N, 3, 0.0, 1.0), &field(N, 4, 0.0, 1.0), |src| {
            bits(&crate::dispatch_gpu_gauss_blur(&ctx, src, 5.0, W, H, true).expect("readback"))
        });
    }

    /// [`check_equivalence`] over `dispatch_gpu_thermal`'s three-pass
    /// ping-pong, chosen with an odd pass count so the result ends up in the
    /// buffer that started the run empty (once-written, never pre-cleared).
    #[test]
    fn pooled_thermal_matches_unpooled() {
        // Protects: pooled thermal output is bit-identical to unpooled across
        // a genuine pool reuse, including the odd-pass-count ping-pong buffer,
        // and survives a poisoned retained buffer.
        let Some(gpu) = device() else { return };
        let ctx = crate::build_pipeline_shared(&gpu, crate::SHADER_SRC_GPU_THERMAL, "gpu_thermal", &crate::BLUR_LAYOUT);
        check_equivalence("thermal", &gpu.pool, N as u64, &field(N, 5, 0.0, 1.0), &field(N, 6, 0.0, 1.0), |src| {
            bits(&crate::dispatch_gpu_thermal(&ctx, src, W, H, 3, 0.01).expect("readback"))
        });
    }

    /// [`check_equivalence`] over `dispatch_gpu_assign_plates` (the JFA plate
    /// assignment), whose pass-0 ping-pong buffers are among the
    /// written-before-read cases in the module doc.
    #[test]
    fn pooled_jfa_plates_matches_unpooled() {
        // Protects: pooled JFA plate assignment is bit-identical to unpooled
        // across a genuine pool reuse, and survives a poisoned retained buffer.
        let Some(gpu) = device() else { return };
        let ctx = crate::init_gpu_jfa_plates_with(&gpu);
        let input = |s: u32| (field(9, s, 0.0, W as f32), field(9, s + 1, 0.0, H as f32), field(N, s + 2, -2.0, 2.0));
        check_equivalence("jfa_plates", &gpu.pool, N as u64, &input(30), &input(40), |(px, py, warp)| {
            let out = crate::dispatch_gpu_assign_plates(&ctx, W, H, px, py, Some(warp), Some(warp), true).expect("readback");
            out.iter().map(|&v| v as u32).collect()
        });
    }

    /// [`check_equivalence`] over `dispatch_gpu_flow`, whose `delta` buffer is
    /// the module doc's own example of a buffer that must be cleared rather
    /// than relied on to already be zero.
    #[test]
    fn pooled_flow_matches_unpooled() {
        // Protects: pooled flow accumulation is bit-identical to unpooled
        // across a genuine pool reuse, and a poisoned `delta` (or any other
        // retained buffer) does not leak into the accumulation.
        let Some(gpu) = device() else { return };
        let ctx = crate::init_gpu_flow_with(&gpu);
        let input = |s: u32| (field(N, s, 0.0, 1.0), field(N, s + 1, 0.0, 1.0));
        check_equivalence("flow", &gpu.pool, N as u64, &input(50), &input(60), |(f, rain)| {
            let r = crate::dispatch_gpu_flow(&ctx, W as usize, H as usize, f, Some(rain), true, false).expect("readback");
            [bits(&r.acc), r.recv.iter().map(|&v| v as u32).collect()].concat()
        });
    }

    /// [`check_equivalence`] over `stress_gather_grid_gpu_with`.
    #[test]
    fn pooled_stress_gather_matches_unpooled() {
        // Protects: pooled plate-stress gathering is bit-identical to
        // unpooled across a genuine pool reuse, and survives a poisoned
        // retained buffer.
        let Some(gpu) = device() else { return };
        let plates = 6usize;
        let input = |s: u32| {
            let ids: Vec<u16> = bytes_field(N, s, plates as u8 - 1).iter().map(|&b| u16::from(b)).collect();
            let vals = field(plates * plates * 3, s + 1, -1.0, 1.0);
            let pairs: Vec<(f64, f64, f64, u8)> = (0..plates * plates)
                .map(|k| (f64::from(vals[3 * k]), f64::from(vals[3 * k + 1]), f64::from(vals[3 * k + 2]).abs(), (k % 5) as u8))
                .collect();
            (ids, pairs)
        };
        check_equivalence("stress", &gpu.pool, N as u64, &input(70), &input(80), |(ids, pairs)| {
            let g = crate::stress_gather_grid_gpu_with(&gpu, W, H, true, ids, pairs).expect("readback");
            let meta: Vec<u32> = g.boundary_mask.iter().zip(&g.boundary_type).map(|(&m, &t)| u32::from(m) | u32::from(t) << 8).collect();
            [bits(&g.raw), bits(&g.raw_s), meta].concat()
        });
    }

    /// [`check_equivalence`] over `dispatch_gpu_weather`, run over its own
    /// smaller weather grid (weather declares no grid of its own, per policy
    /// 1, so this stands in for the world-grid declaration a real generation
    /// always makes first).
    #[test]
    fn pooled_weather_matches_unpooled() {
        // Protects: pooled weather output is bit-identical to unpooled across
        // a genuine pool reuse, and survives a poisoned retained buffer.
        let Some(gpu) = device() else { return };
        let ctx = crate::init_gpu_weather_with(&gpu);
        let (ww, wh) = (29u32, 19u32);
        let n = (ww * wh) as usize;
        let input = |s: u32| {
            (
                field(n, s, 0.0, 1.0),
                field(n, s + 1, -5.0, 30.0),
                field(n, s + 2, 0.0, 1.0),
                field(n, s + 3, -1.0, 1.0),
                field(n, s + 4, -1.0, 1.0),
                field(n, s + 5, 0.0, 0.5),
            )
        };
        // Weather declares no grid of its own (policy 1); a generation's world
        // grid is always declared before it runs, which this stands in for.
        check_equivalence("weather", &gpu.pool, n as u64, &input(90), &input(100), |(eh, tc, sst, wx, wy, w0)| {
            let (w, rain) = crate::dispatch_gpu_weather(
                &ctx, eh, tc, sst, wx, wy, w0, ww, wh, 6, 0.4, 0.6, 0.3, 1.0, 0.4, 0.8, 1.0, true, false,
            )
            .expect("readback");
            [bits(&w), bits(&rain)].concat()
        });
    }

    /// `iters == 0` reads the rain buffer back untouched: the one path where
    /// the clear is the whole answer.
    #[test]
    fn pooled_weather_with_no_iterations_returns_zero_rain() {
        let Some(gpu) = device() else { return };
        let ctx = crate::init_gpu_weather_with(&gpu);
        let (ww, wh) = (29u32, 19u32);
        let n = (ww * wh) as usize;
        let f = field(n, 7, 0.0, 1.0);
        gpu.pool.begin_grid(n as u64);
        let run = |iters| {
            crate::dispatch_gpu_weather(&ctx, &f, &f, &f, &f, &f, &f, ww, wh, iters, 0.4, 0.6, 0.3, 1.0, 0.4, 0.8, 1.0, true, false)
                .expect("readback")
        };
        let (_, rain) = run(6);
        assert!(rain.iter().any(|&r| r != 0.0), "the first run must leave rain behind to leak");
        gpu.pool.poison_retained(0xA5);
        let (_, rain) = run(0);
        assert!(rain.iter().all(|&r| r.to_bits() == 0), "rain read back uninitialised");
    }

    /// [`check_equivalence`] over the three noise-pilot-era kernels
    /// (`dispatch_gpu`, `dispatch_gpu_resistance`, `dispatch_gpu_height`),
    /// each on its own standalone device and so its own pool.
    #[test]
    fn pooled_noise_resistance_and_height_match_unpooled() {
        // Protects: pooled noise, resistance and height output are each
        // bit-identical to unpooled across a genuine pool reuse, and survive
        // a poisoned retained buffer.
        let Ok(noise) = crate::init_gpu() else {
            eprintln!("no GPU adapter on this machine -- pooled-equivalence test skipped");
            return;
        };
        // The pilot's f32 hash barely moves with the seed at this size (see
        // `f32_hash_diverges_from_cpu_reference`), so the scale is varied too.
        check_equivalence("vnoise", &noise.pool, N as u64, &(11i32, 0.13f32), &(12i32, 0.29f32), |&(seed, scale)| {
            bits(&crate::dispatch_gpu(&noise, W, H, seed, scale).expect("readback"))
        });

        let res = crate::init_gpu_resistance().expect("adapter present above");
        let input = |s: u32| {
            let ids: Vec<u32> = bytes_field(N, s, 5).iter().map(|&b| u32::from(b)).collect();
            (ids, field(N, s + 1, 0.0, 1.0), field(6, s + 2, 0.0, 1.0))
        };
        check_equivalence("resistance", &res.pool, N as u64, &input(110), &input(120), |(ids, age, crust)| {
            bits(&crate::dispatch_gpu_resistance(&res, W, H, ids, age, crust).expect("readback"))
        });

        let height = crate::init_gpu_height().expect("adapter present above");
        let input = |s: u32| (0..8).map(|k| field(N, s + k, 0.0, 1.0)).collect::<Vec<_>>();
        check_equivalence("height", &height.pool, N as u64, &input(130), &input(140), |f| {
            bits(
                &crate::dispatch_gpu_height(
                    &height, W, H, 5, 0.02, 0.6, 0.4, 0.3, 0.2, 0.25, true, true, &f[0], &f[1], &f[2], &f[3], &f[4],
                    &f[5], &f[6], &f[7],
                )
                .expect("readback"),
            )
        });
    }

    /// [`check_equivalence`] over the four Phase 2 affordance kernels
    /// (`src/affordance.rs`): biome, carrying capacity, resources and
    /// suitability.
    #[test]
    fn pooled_affordance_kernels_match_unpooled() {
        // Protects: each of the four affordance kernels' pooled output is
        // bit-identical to unpooled across a genuine pool reuse; resources'
        // fixture additionally runs an all-optional-planes-present pass then
        // an all-absent pass, so a plane run A filled cannot leak into run B
        // through a reused buffer the absent-plane clear should have cleared.
        let Some(gpu) = device() else { return };

        let input = |s: u32| (bytes_field(N, s, 2), field(N, s + 1, -10.0, 35.0), field(N, s + 2, 0.0, 1.0));
        check_equivalence("biome", &gpu.pool, N as u64, &input(150), &input(160), |(wb, t, r)| {
            crate::biome_raster_grid_gpu_with(&gpu, wb, t, r).expect("readback").iter().map(|&b| u32::from(b)).collect()
        });

        let input = |s: u32| {
            (
                (0..4).map(|k| field(N, s + k, 0.0, 1.0)).collect::<Vec<_>>(),
                bytes_field(N, s + 9, 13),
                bytes_field(N, s + 10, 1),
            )
        };
        check_equivalence("carrying", &gpu.pool, N as u64, &input(170), &input(180), |(f, biome, wet)| {
            bits(
                &crate::carrying_capacity_grid_gpu_with(&gpu, &f[0], &f[1], biome, &f[2], &f[3], 0.3, 0.5, Some(wet))
                    .expect("readback"),
            )
        });

        /// Run A has every optional plane; run B has none of them. So run B
        /// reads planes run A filled -- the leak the absent-plane clear exists
        /// for -- and the poisoned pass catches it whatever A left behind.
        struct Res {
            lith: Vec<u8>,
            bt: Vec<u8>,
            biome: Vec<u8>,
            f: Vec<Vec<f32>>,
            optional: bool,
        }
        let input = |s: u32, optional: bool| Res {
            lith: bytes_field(N, s, 6),
            bt: bytes_field(N, s + 1, 5),
            biome: bytes_field(N, s + 2, 12),
            f: (0..7).map(|k| field(N, s + 3 + k, 0.0, 1.0)).collect(),
            optional,
        };
        check_equivalence("resources", &gpu.pool, N as u64, &input(190, true), &input(200, false), |r| {
            /// `Some(v)` when `on`, else `None` -- shorthand for building the
            /// optional planes below from the fixture's single `optional` flag.
            fn present(on: bool, v: &[f32]) -> Option<&[f32]> {
                on.then_some(v)
            }
            let opt = |v| present(r.optional, v);
            let inp = crate::ResourceGpuInputs {
                lith: &r.lith,
                boundary_type: r.optional.then_some(&r.bt[..]),
                shear_field: opt(&r.f[0]),
                flow: opt(&r.f[1]),
                biome: r.optional.then_some(&r.biome[..]),
                field: &r.f[2],
                rain: &r.f[3],
                age: &r.f[4],
                volcanic: opt(&r.f[5]),
                cu_dist: &r.f[6],
                flow_max: 3.0,
                gw: W as usize,
                sea: 0.3,
            };
            crate::resource_potentials_grid_gpu_with(&gpu, &inp).expect("readback").iter().flat_map(|p| bits(p)).collect()
        });

        let input = |s: u32| ((0..22).map(|k| field(N, s + k, 0.0, 1.0)).collect::<Vec<_>>(), bytes_field(N, s + 30, 1));
        check_equivalence("suitability", &gpu.pool, N as u64, &input(210), &input(240), |(f, wb)| {
            let inp = crate::SuitabilityGpuInputs {
                soil: &f[0],
                water: &f[1],
                carrying_cap: &f[2],
                field: &f[3],
                slope_n: &f[4],
                water_bodies: wb,
                corridor: &f[5],
                landmass: &f[6],
                flow: &f[7],
                river_reach: &f[8],
                coast_reach: &f[9],
                resources: [&f[10], &f[11], &f[12], &f[13], &f[14], &f[15], &f[16], &f[17], &f[18]],
                rain: &f[19],
                flood: &f[20],
                slope_raw: &f[21],
                flow_thresh: 0.5,
                gw: W as usize,
                gh: H as usize,
                sea: 0.3,
            };
            let wt = crate::SuitabilityWeights {
                k: 0.2,
                w: 0.2,
                a: 0.1,
                d: 0.1,
                agri: 0.1,
                build: 0.05,
                coast: 0.1,
                river: 0.1,
                lake: 0.05,
                mineral: 0.05,
                corridor: 0.05,
                flood: 0.1,
                islet: 0.1,
                islet_knee: 0.3,
            };
            bits(&crate::settlement_suitability_grid_gpu_with(&gpu, &inp, &wt).expect("readback"))
        });
    }

    /// Policy 1 and 2: a different grid releases everything; disabling
    /// releases everything; a lost device retains nothing.
    #[test]
    fn grid_change_disable_and_loss_release_the_pool() {
        let Some(gpu) = device() else { return };
        let ctx = crate::init_gpu_warp_with(&gpu);
        crate::dispatch_gpu_warp(&ctx, W, H, 1, 0.05, 3.0, false).expect("readback");
        let s = gpu.pool.stats();
        assert_eq!(s.grid_cells, Some(N as u64));
        assert!(s.retained_bytes > 0 && s.retained_bytes <= s.cap_bytes);

        gpu.pool.begin_grid(N as u64 + 1);
        assert_eq!(gpu.pool.stats().retained_bytes, 0, "a grid change must release every retained buffer");

        crate::dispatch_gpu_warp(&ctx, W, H, 1, 0.05, 3.0, false).expect("readback");
        assert!(gpu.pool.stats().retained_bytes > 0);
        gpu.pool.set_enabled(false);
        assert_eq!(gpu.pool.stats().retained_bytes, 0, "disabling must release every retained buffer");
        gpu.pool.set_enabled(true);

        crate::dispatch_gpu_warp(&ctx, W, H, 1, 0.05, 3.0, false).expect("readback");
        assert!(gpu.pool.stats().retained_bytes > 0);
        gpu.lost.store(true, Ordering::Relaxed);
        gpu.pool.begin_grid(N as u64);
        assert_eq!(gpu.pool.stats().retained_bytes, 0, "a lost device must retain nothing");
    }

    /// The cap is honoured: a grid whose buffers exceed it keeps no more than
    /// the cap, and a buffer larger than the cap is never kept.
    #[test]
    fn retained_bytes_never_exceed_the_cap() {
        let Some(gpu) = device() else { return };
        // Suitability binds a 22-plane input: wider than the 10-grid cap.
        let f = field(N, 1, 0.0, 1.0);
        let wb = bytes_field(N, 2, 1);
        let inp = crate::SuitabilityGpuInputs {
            soil: &f,
            water: &f,
            carrying_cap: &f,
            field: &f,
            slope_n: &f,
            water_bodies: &wb,
            corridor: &f,
            landmass: &f,
            flow: &f,
            river_reach: &f,
            coast_reach: &f,
            resources: [&f; 9],
            rain: &f,
            flood: &f,
            slope_raw: &f,
            flow_thresh: 0.5,
            gw: W as usize,
            gh: H as usize,
            sea: 0.3,
        };
        let wt = crate::SuitabilityWeights {
            k: 0.2,
            w: 0.2,
            a: 0.1,
            d: 0.1,
            agri: 0.1,
            build: 0.05,
            coast: 0.1,
            river: 0.1,
            lake: 0.05,
            mineral: 0.05,
            corridor: 0.05,
            flood: 0.1,
            islet: 0.1,
            islet_knee: 0.3,
        };
        crate::settlement_suitability_grid_gpu_with(&gpu, &inp, &wt).expect("readback");
        let s = gpu.pool.stats();
        assert_eq!(s.cap_bytes, N as u64 * 4 * 10);
        assert!(s.retained_bytes <= s.cap_bytes);
        let plane = N as u64 * 4;
        assert!(
            !gpu.pool.state().free.contains_key(&(plane * 22, Kind::Storage)),
            "a buffer wider than the cap was retained"
        );
    }
}
