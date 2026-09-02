//! Multi-GPU: adapter enumeration, device selection, VRAM budgeting, and
//! the split-tiles dispatch (`GUI_GAP_REGISTER.md` PR-01/PR-02/PR-04/PR-05,
//! `DCC_SHELL_SPEC.md` §2.5's Performance group, `HARDWARE_ACCELERATION.md`'s
//! 2026-08-20 section).
//!
//! Everything before this module requested exactly **one** adapter --
//! `PowerPreference::HighPerformance`, no enumeration, no choice, no
//! accounting. This module adds the four things §2.5 designs, and is
//! deliberately explicit about which of them are real:
//!
//! | §2.5 row | Here |
//! |---|---|
//! | Devices | **real** — [`enumerate_devices`] lists every physical GPU with name, type, backend and limits, and [`set_preferences`] picks which one(s) dispatch runs on. |
//! | Multi-GPU mode | `single device` and `split tiles` are **real** ([`GpuDeviceSet`]); `alternate frames` is **honestly disabled** ([`MultiGpuMode::is_implemented`]) — §2.5's own note says it "only helps the 3D viewport", and this port has no 3D viewport (`DECISIONS.md` §4). |
//! | VRAM budget | **real as a cap** ([`vram_verdict`]) over a documented working-set *estimate* ([`gpu_working_set_bytes`]). §2.5's "default 75 % of the smallest active device" is **not implementable**: `wgpu` 30 exposes no VRAM size for an adapter at all (`Adapter::limits()` is an API limit, `AdapterInfo` carries none), so there is nothing to take 75 % of. The default is therefore "no cap", stated rather than faked. |
//! | Fallback when VRAM full | `CPU tile pass` is **real** (it is already what happens when the GPU path is unavailable) and `fail with error` is **real at the caller** ([`VramVerdict`] is returned, not swallowed); `reduce working res` is **honestly disabled** — nothing in this pipeline resamples a grid down and back up. |
//!
//! And the one §2.5 phrase this module deliberately does not implement as
//! written: **"live utilisation `71%`"**. `wgpu` 30 has no system-wide GPU
//! utilisation or VRAM-occupancy query on any backend, and inventing a
//! percentage would be exactly the kind of plausible-looking fiction this
//! project's menus refuse elsewhere. What *is* real, and what
//! [`device_usage`] returns, is this application's **own** allocation total
//! from `wgpu::Device::generate_allocator_report()` — measured, not
//! modelled, and labelled as ours rather than the system's.

use std::sync::RwLock;

use crate::{GpuDevice, GpuInitError, REUSED_STAGE_MAX_STORAGE_BUFFERS, RawGpuDevice};

// -- Enumeration ---------------------------------------------------------------

/// One row of `wgpu::Instance::enumerate_adapters`, flattened to owned data.
///
/// A separate type from [`GpuDeviceInfo`] on purpose: enumeration returns one
/// row **per adapter**, and a single physical GPU shows up as several
/// (on this project's own development machine the RX 7800 XT appears three
/// times -- Vulkan, Dx12 and Gl). [`group_adapters`] is the pure function
/// that collapses rows into physical devices, so it can be unit-tested with
/// no GPU present at all -- which is the headless/CI reality this crate has
/// to work under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterRow {
    pub name: String,
    pub vendor: u32,
    pub device_id: u32,
    pub device_type: wgpu::DeviceType,
    pub backend: wgpu::Backend,
    pub driver: String,
    pub driver_info: String,
    pub max_buffer_size: u64,
    pub max_storage_buffer_binding_size: u64,
    pub supports_compute: bool,
}

/// One physical GPU, with every backend that reaches it folded in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuDeviceInfo {
    /// Stable identity across sessions and across driver updates that change
    /// which backend is preferred -- `"1002:747e:AMD Radeon RX 7800 XT"`.
    /// This, not the enumeration index, is what a persisted preference
    /// stores: enumeration order is the driver's, and adding a second GPU
    /// (the entire point of this feature) renumbers it.
    pub key: String,
    pub name: String,
    pub vendor: u32,
    pub device_id: u32,
    pub device_type: wgpu::DeviceType,
    /// The backend a device request for this GPU will actually use --
    /// the lowest-ranked one available, see [`backend_rank`].
    pub backend: wgpu::Backend,
    /// The other backends that also reach this GPU, ranked order, preferred
    /// one excluded. Informational; nothing dispatches through them.
    pub alternate_backends: Vec<wgpu::Backend>,
    pub driver: String,
    pub driver_info: String,
    pub max_buffer_size: u64,
    pub max_storage_buffer_binding_size: u64,
    pub supports_compute: bool,
    /// `DeviceType::Cpu` -- a software rasterizer (Windows ships
    /// "Microsoft Basic Render Driver", which this machine's own
    /// enumeration returns). Listed rather than hidden, but never selected
    /// by default: `HARDWARE_ACCELERATION.md` §5/§31's rule is "prefer a
    /// high-performance real adapter, never a software fallback".
    ///
    /// **What enforces that rule is [`auto_pick_allows`], applied in
    /// [`pick_primary_adapter_for`] -- not `force_fallback_adapter: false`**,
    /// which is what this comment claimed until 2026-09-02 and which is
    /// wrong in the direction that matters. Read against wgpu-core 30's own
    /// `Instance::request_adapter` (`src/instance.rs`): the flag means
    /// *restrict to* fallback adapters. `true` runs
    /// `backend_adapters.retain(|a| a.info.device_type == DeviceType::Cpu)`;
    /// `false` declines to restrict and runs no filter at all. Nothing else
    /// in that function excludes a CPU adapter -- `get_order` ranks `Cpu`
    /// last (5), but the pick is `adapters.into_iter().next()`, which still
    /// returns it when it is the only thing the sort had to order. So on a
    /// machine with no usable hardware adapter (a broken or absent Vulkan
    /// ICD, a VM, a CI box, a laptop in the wrong MUX state) the automatic
    /// path opened the Basic Render Driver and ran the whole pipeline on it,
    /// silently, instead of taking the CPU path.
    pub is_software: bool,
}

impl GpuDeviceInfo {
    /// The device class in the shell's own vocabulary --
    /// `DCC_SHELL_SPEC.md` §2.5's device rows read "discrete"/"iGPU", not
    /// `wgpu` enum names. Exists so `cartalith-godot` can build its
    /// `#[func]` dictionaries without naming a `wgpu` type: nothing above
    /// this crate should have to depend on `wgpu` to describe a GPU
    /// (`ARCHITECTURE.md`'s crate ladder).
    #[must_use]
    pub const fn kind_str(&self) -> &'static str {
        match self.device_type {
            wgpu::DeviceType::DiscreteGpu => "discrete",
            wgpu::DeviceType::IntegratedGpu => "integrated",
            wgpu::DeviceType::VirtualGpu => "virtual",
            wgpu::DeviceType::Cpu => "software",
            wgpu::DeviceType::Other => "other",
        }
    }

    /// The preferred backend's name (`"vulkan"`, `"dx12"`, …).
    #[must_use]
    pub const fn backend_str(&self) -> &'static str {
        self.backend.to_str()
    }

    /// The other backends that reach this GPU, by name.
    #[must_use]
    pub fn alternate_backend_strs(&self) -> Vec<&'static str> {
        self.alternate_backends.iter().map(|b| b.to_str()).collect()
    }
}

/// Preference order when one physical GPU is reachable through several
/// backends. Vulkan first because that is what
/// `PowerPreference::HighPerformance` already resolves to on this project's
/// development machine (verified by running it, not assumed) -- so a
/// `single device` selection of the default GPU reproduces today's path
/// exactly rather than quietly switching backend.
#[must_use]
pub const fn backend_rank(b: wgpu::Backend) -> u8 {
    match b {
        wgpu::Backend::Vulkan => 0,
        wgpu::Backend::Metal => 1,
        wgpu::Backend::Dx12 => 2,
        wgpu::Backend::BrowserWebGpu => 3,
        wgpu::Backend::Gl => 4,
        wgpu::Backend::Noop => 5,
    }
}

/// Sort order for the device list the UI shows: the GPU most likely to be
/// wanted first. Not a capability judgement -- purely presentation.
const fn device_type_rank(t: wgpu::DeviceType) -> u8 {
    match t {
        wgpu::DeviceType::DiscreteGpu => 0,
        wgpu::DeviceType::IntegratedGpu => 1,
        wgpu::DeviceType::VirtualGpu => 2,
        wgpu::DeviceType::Other => 3,
        wgpu::DeviceType::Cpu => 4,
    }
}

fn device_key(name: &str, vendor: u32, device_id: u32) -> String {
    if vendor != 0 || device_id != 0 {
        format!("{vendor:04x}:{device_id:04x}:{name}")
    } else {
        format!("name:{name}")
    }
}

/// Collapse per-adapter rows into one entry per physical GPU.
///
/// Two grouping rules, and the second exists because of a real observation
/// rather than a hypothetical: on this machine the **OpenGL** adapter for the
/// RX 7800 XT reports `vendor = 0`, `device = 0` and
/// `device_type = Other`, so PCI identity cannot key it.
///
/// 1. Rows with a non-zero `(vendor, device_id)` group by that pair. This is
///    the case that matters for the canonical multi-GPU rig -- two *identical*
///    cards share a name and differ only in PCI device identity, so keying on
///    the name alone would have merged them into one and silently made the
///    whole feature unable to see the second card.
/// 2. Rows with `(0, 0)` join the single existing group of the same name if
///    there is exactly one; if there is none, or more than one, they form
///    their own group. Ambiguity is left as a separate entry rather than
///    guessed at.
#[must_use]
pub fn group_adapters(rows: Vec<AdapterRow>) -> Vec<GpuDeviceInfo> {
    let mut groups: Vec<Vec<AdapterRow>> = Vec::new();
    let mut keys: Vec<String> = Vec::new();

    for row in rows.iter().filter(|r| r.vendor != 0 || r.device_id != 0) {
        let key = device_key(&row.name, row.vendor, row.device_id);
        match keys.iter().position(|k| *k == key) {
            Some(i) => groups[i].push(row.clone()),
            None => {
                keys.push(key);
                groups.push(vec![row.clone()]);
            }
        }
    }

    for row in rows.iter().filter(|r| r.vendor == 0 && r.device_id == 0) {
        let matching: Vec<usize> =
            groups.iter().enumerate().filter(|(_, g)| g[0].name == row.name).map(|(i, _)| i).collect();
        if matching.len() == 1 {
            groups[matching[0]].push(row.clone());
        } else {
            keys.push(device_key(&row.name, 0, 0));
            groups.push(vec![row.clone()]);
        }
    }

    let mut out: Vec<GpuDeviceInfo> = groups
        .into_iter()
        .map(|mut g| {
            g.sort_by_key(|r| backend_rank(r.backend));
            let best = g[0].clone();
            GpuDeviceInfo {
                key: device_key(&best.name, best.vendor, best.device_id),
                name: best.name,
                vendor: best.vendor,
                device_id: best.device_id,
                device_type: best.device_type,
                backend: best.backend,
                alternate_backends: g[1..].iter().map(|r| r.backend).collect(),
                driver: best.driver,
                driver_info: best.driver_info,
                max_buffer_size: best.max_buffer_size,
                max_storage_buffer_binding_size: best.max_storage_buffer_binding_size,
                supports_compute: best.supports_compute,
                is_software: best.device_type == wgpu::DeviceType::Cpu,
            }
        })
        .collect();

    out.sort_by(|a, b| {
        device_type_rank(a.device_type)
            .cmp(&device_type_rank(b.device_type))
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.vendor.cmp(&b.vendor))
            .then_with(|| a.device_id.cmp(&b.device_id))
    });
    out
}

/// Every backend *except* OpenGL. This is not a preference, it is a crash fix.
///
/// This crate runs inside the Godot process, and that process's renderer is
/// GL Compatibility (`project.godot`: `renderer/rendering_method=
/// "gl_compatibility"`). Asking `wgpu` for the GL backend makes it create its
/// own OpenGL context in that same process, which leaves Godot's GLES3
/// resource caches referring to objects the now-current context does not
/// own. The symptom is not subtle and not immediate: a burst of
/// `texture_free_data`/`buffer_free_data` "Condition
/// `!*_allocs_cache.has(p_id)` is true", then
/// `update_texture_atlas: Could not create texture atlas, status: 0`, then a
/// signal-11 crash inside Godot's own GLES3 driver, with no GDScript frame
/// anywhere in the backtrace. On a shell that is not creating textures at
/// that moment there is no error burst at all -- just the signal 11, on the
/// **very next frame** after the call returns.
///
/// Reproduced on a real launch (AMD RX 7800 XT, OpenGL 3.3 Core Profile) and
/// bisected to this call: enumeration happens at startup, because
/// `menus.gd`'s Preferences ▸ Devices submenu is built during `_ready`.
///
/// **Where this mask has to be applied is the whole bug** (2026-08-23, owner
/// report "a crash when you get higher than 2k and start changing settings
/// for resources such as GPU/CPU"). The 2026-08-20 pass passed it to
/// `enumerate_adapters`, which is far too late: `wgpu::Instance::new` stands
/// up a `hal::Instance` for **every backend in its own descriptor's mask**,
/// and `InstanceDescriptor::new_without_display_handle()` leaves that mask at
/// `Backends::all()`. The GL context was therefore created the moment the
/// instance was, before a single adapter had been asked for -- so restricting
/// enumeration "did not work" (that commit says as much) and deferring
/// enumeration to the submenu's first open only moved the crash from launch
/// to the first time anyone opened Preferences ▸ Performance ▸ Devices.
/// The mask belongs on the descriptor, and [`compute_instance`] is the only
/// place in this crate that builds one.
///
/// Nothing real is lost. The GL rows this drops were duplicates of devices
/// Vulkan and DX12 already report -- and the *reason* they were duplicates
/// mattered even before this bug, since a GL row reports `vendor = device =
/// 0`, which `group_adapters` already had to work around. Compute dispatch
/// never used GL either: `init_gpu` asks for `PowerPreference::
/// HighPerformance`, which resolves to Vulkan on this hardware.
pub const COMPUTE_BACKENDS: wgpu::Backends = wgpu::Backends::VULKAN
    .union(wgpu::Backends::DX12)
    .union(wgpu::Backends::METAL)
    .union(wgpu::Backends::BROWSER_WEBGPU);

/// The **only** way this crate is allowed to create a `wgpu::Instance`.
///
/// Every call site went through `InstanceDescriptor::new_without_display_handle()`
/// before, which defaults `backends` to `Backends::all()` and so created an
/// OpenGL context inside Godot's own GL-Compatibility process. See
/// [`COMPUTE_BACKENDS`] for the crash that causes.
///
/// **`.with_env()` is why `WGPU_BACKEND` works at all** (2026-09-02).
/// `new_without_display_handle()` reads no environment variable --
/// `wgpu::Backends::with_env`, and so `WGPU_BACKEND` / `WGPU_DX12_COMPILER` /
/// `WGPU_VALIDATION`, is reached only from the `*_from_env`/`with_env`
/// constructors, which nothing here called. The escape hatch was therefore
/// inert, and the consequence was not cosmetic: nobody could run this
/// project's compute on DX12, so [`backend_rank`]'s Vulkan-first order was
/// asserted and **unmeasurable**. This does not change that order -- Vulkan
/// first is the right default and routing compute through DX12 would import
/// a DXC/FXC compiler lottery into a determinism-critical pipeline -- it only
/// makes the comparison runnable.
///
/// **The mask is intersected *after* the environment is read, and that
/// ordering is the whole safety argument.** `&=` can only clear bits, never
/// set them, so [`COMPUTE_BACKENDS`] is an upper bound no environment value
/// can lift: `WGPU_BACKEND=gl` cannot put GL back, because `try_add_hal` in
/// wgpu-core only stands up a backend's `hal::Instance` when
/// `instance_desc.backends` contains it, and after the `&=` it cannot. That
/// is the signal-11 fix in [`COMPUTE_BACKENDS`], preserved by construction
/// rather than by a second check that could drift.
///
/// **Unset, this is byte-identical to what it was.** With no variable set
/// every `with_env` in `wgpu-types` returns its input unchanged, so
/// `backends` is `Backends::all()`, and `Backends::all() & COMPUTE_BACKENDS`
/// *is* `COMPUTE_BACKENDS` -- the literal the old code assigned.
#[must_use]
pub fn compute_instance() -> wgpu::Instance {
    let mut desc = wgpu::InstanceDescriptor::new_without_display_handle().with_env();
    desc.backends &= COMPUTE_BACKENDS;
    if desc.backends.is_empty() {
        // `WGPU_BACKEND` named only backends this crate refuses (in practice
        // `gl`). Honouring it is not an option and silently substituting
        // Vulkan would ignore the request, so the instance really does get
        // nothing -- which every caller already handles as "no GPU, use the
        // CPU path". Said out loud, in `read_back`'s idiom, because an
        // unexplained total loss of the GPU path is the one outcome here
        // that is impossible to diagnose from the outside.
        eprintln!(
            "cartalith-gpu: WGPU_BACKEND selects no backend this crate allows (OpenGL is masked out \
             deliberately -- see COMPUTE_BACKENDS); running on the CPU path"
        );
    }
    wgpu::Instance::new(desc)
}

fn adapter_rows() -> Vec<AdapterRow> {
    let instance = compute_instance();
    pollster::block_on(instance.enumerate_adapters(COMPUTE_BACKENDS))
        .iter()
        .map(describe_adapter)
        .collect()
}

fn describe_adapter(a: &wgpu::Adapter) -> AdapterRow {
    let info = a.get_info();
    let limits = a.limits();
    AdapterRow {
        name: info.name,
        vendor: info.vendor,
        device_id: info.device,
        device_type: info.device_type,
        backend: info.backend,
        driver: info.driver,
        driver_info: info.driver_info,
        max_buffer_size: limits.max_buffer_size,
        max_storage_buffer_binding_size: limits.max_storage_buffer_binding_size,
        supports_compute: a.get_downlevel_capabilities().flags.contains(wgpu::DownlevelFlags::COMPUTE_SHADERS),
    }
}

/// Every physical GPU this machine exposes, one entry each.
///
/// Returns an **empty vec** rather than an error when there is nothing --
/// a headless machine, a container with no ICD, a CI runner. That is a
/// normal state here, not a failure: every caller in this workspace already
/// treats "no GPU" as "use the CPU path" (`HARDWARE_ACCELERATION.md` §27).
#[must_use]
pub fn enumerate_devices() -> Vec<GpuDeviceInfo> {
    group_adapters(adapter_rows())
}

// -- Preferences ---------------------------------------------------------------

/// `DCC_SHELL_SPEC.md` §2.5's "Multi-GPU mode".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum MultiGpuMode {
    /// Everything on one device -- today's behaviour, plus a chooser.
    #[default]
    SingleDevice,
    /// Partition the working grid across the selected devices. Real, and
    /// implemented for the one pipeline stage where a partition is exact
    /// rather than approximate: see [`crate::warp_grid_gpu_split`].
    SplitTiles,
    /// **Not implemented.** §2.5's own note: "alternate frames only helps
    /// the 3D viewport". There is no 3D viewport (`DECISIONS.md` §4,
    /// `ROADMAP.md` Phase 3), so there are no frames to alternate; selecting
    /// it would be indistinguishable from [`Self::SingleDevice`] while
    /// implying otherwise. Kept as a variant so the setting round-trips and
    /// the UI can disable the row with a reason rather than omit it.
    AlternateFrames,
}

impl MultiGpuMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SingleDevice => "single_device",
            Self::SplitTiles => "split_tiles",
            Self::AlternateFrames => "alternate_frames",
        }
    }

    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "single_device" => Some(Self::SingleDevice),
            "split_tiles" => Some(Self::SplitTiles),
            "alternate_frames" => Some(Self::AlternateFrames),
            _ => None,
        }
    }

    /// Whether selecting this mode actually changes dispatch.
    #[must_use]
    pub const fn is_implemented(self) -> bool {
        !matches!(self, Self::AlternateFrames)
    }
}

/// `DCC_SHELL_SPEC.md` §2.5's "Fallback when VRAM full".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum VramFallback {
    /// Run the stage on the CPU instead. Real -- and already exactly what
    /// happens today whenever GPU init or a dispatch is unavailable, so
    /// wiring this option is disclosure of existing behaviour rather than
    /// new behaviour.
    #[default]
    CpuTilePass,
    /// **Not implemented.** Nothing in this pipeline computes a stage at a
    /// reduced grid and resamples back up; `lod_synthesize_tile` resamples
    /// an *existing* field, which is a different operation. Kept as a
    /// variant for the same round-trip reason as
    /// [`MultiGpuMode::AlternateFrames`].
    ReduceWorkingRes,
    /// Surface the over-budget condition to the caller instead of silently
    /// degrading. Real: [`vram_verdict`] returns it, and the caller decides.
    FailWithError,
}

impl VramFallback {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CpuTilePass => "cpu_tile_pass",
            Self::ReduceWorkingRes => "reduce_working_res",
            Self::FailWithError => "fail_with_error",
        }
    }

    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "cpu_tile_pass" => Some(Self::CpuTilePass),
            "reduce_working_res" => Some(Self::ReduceWorkingRes),
            "fail_with_error" => Some(Self::FailWithError),
            _ => None,
        }
    }

    #[must_use]
    pub const fn is_implemented(self) -> bool {
        !matches!(self, Self::ReduceWorkingRes)
    }
}

/// The four §2.5 Performance settings this module owns.
///
/// Process-global rather than threaded through `WorldParams`, deliberately.
/// These describe **the machine**, not the world: two worlds generated with
/// the same seed on the same devices must be identical, and a device choice
/// stored per-world would travel in a `.zip` to a machine where the key
/// names nothing. It also keeps `use_gpu = false`'s golden path untouched --
/// nothing here is read at all unless `use_gpu` is on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuPreferences {
    /// Device keys ([`GpuDeviceInfo::key`]) in dispatch order. **Empty means
    /// "auto"** -- exactly today's `PowerPreference::HighPerformance`
    /// request, so an untouched install behaves as it did before this module
    /// existed.
    pub selected_keys: Vec<String>,
    pub mode: MultiGpuMode,
    /// `0` means no cap. See the module doc for why this cannot default to
    /// §2.5's "75 % of the smallest active device".
    pub vram_budget_bytes: u64,
    pub fallback: VramFallback,
}

impl GpuPreferences {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            selected_keys: Vec::new(),
            mode: MultiGpuMode::SingleDevice,
            vram_budget_bytes: 0,
            fallback: VramFallback::CpuTilePass,
        }
    }
}

impl Default for GpuPreferences {
    fn default() -> Self {
        Self::new()
    }
}

static PREFS: RwLock<GpuPreferences> = RwLock::new(GpuPreferences::new());

/// A snapshot of the current preferences.
#[must_use]
pub fn preferences() -> GpuPreferences {
    PREFS.read().map(|p| p.clone()).unwrap_or_default()
}

/// Replace the preferences. Takes effect on the next device request --
/// nothing already-created is torn down, matching the GPU toggle's own
/// "takes effect on the next generate" contract.
pub fn set_preferences(p: GpuPreferences) {
    if let Ok(mut w) = PREFS.write() {
        *w = p;
    }
}

// -- VRAM budget ---------------------------------------------------------------

/// Concurrent `f32` buffers the heaviest GPU stage in `generate_terrain`
/// holds live for one grid.
///
/// Derived, not guessed: plate assignment's `JFA_LAYOUT` binds **8** storage
/// buffers over the full grid (`nearest_in`/`nearest_out`,
/// `best_d2_in`/`best_d2_out`, `warp_x`/`warp_y`, plus the two plate-seed
/// arrays, which are `plates`-sized rather than grid-sized and so are
/// over-counted here), and every dispatch in this crate additionally
/// allocates a `COPY_DST | MAP_READ` staging buffer per output. 8 + 2 is the
/// resulting upper bound, and it is deliberately an upper bound: a budget
/// that under-estimates is worse than useless.
///
/// This is an **estimate of what this pipeline allocates**, not a
/// measurement and not a claim about total VRAM occupancy -- for a real
/// measured number see [`device_usage`].
pub const GPU_GRID_BUFFERS: u64 = 10;

/// Upper-bound bytes the GPU substrate path allocates for a `width` x
/// `height` grid. See [`GPU_GRID_BUFFERS`] for where the multiplier comes
/// from.
#[must_use]
pub fn gpu_working_set_bytes(width: usize, height: usize) -> u64 {
    (width as u64) * (height as u64) * (size_of::<f32>() as u64) * GPU_GRID_BUFFERS
}

/// What the budget says about a grid, and what the caller should do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VramVerdict {
    /// Within budget (or no budget set) -- dispatch normally.
    Ok,
    /// Over budget; the configured fallback is [`VramFallback::CpuTilePass`]
    /// (or the un-implemented [`VramFallback::ReduceWorkingRes`], which
    /// degrades to the same thing rather than pretending). Run on CPU.
    FallBackToCpu,
    /// Over budget and the caller asked to be told rather than degraded.
    Fail,
}

/// Apply the configured budget to a grid size.
///
/// Pure with respect to everything except [`preferences`], and the two are
/// separated ([`vram_verdict_for`]) so the decision itself is testable with
/// no global state and no GPU.
#[must_use]
pub fn vram_verdict(width: usize, height: usize) -> VramVerdict {
    let p = preferences();
    vram_verdict_for(gpu_working_set_bytes(width, height), p.vram_budget_bytes, p.fallback)
}

/// The budget decision as a pure function.
#[must_use]
pub const fn vram_verdict_for(need_bytes: u64, budget_bytes: u64, fallback: VramFallback) -> VramVerdict {
    if budget_bytes == 0 || need_bytes <= budget_bytes {
        return VramVerdict::Ok;
    }
    match fallback {
        VramFallback::FailWithError => VramVerdict::Fail,
        // `ReduceWorkingRes` has no implementation (see its own doc
        // comment); it does the *safe* thing rather than the promised
        // thing, and the UI says so rather than this silently differing.
        VramFallback::CpuTilePass | VramFallback::ReduceWorkingRes => VramVerdict::FallBackToCpu,
    }
}

/// Whether the GPU path may be used for this grid at all. The one call
/// `cartalith-engine` makes -- `true` whenever no budget is set, so the
/// default install's behaviour is byte-for-byte what it was.
#[must_use]
pub fn gpu_allowed_for_grid(width: usize, height: usize) -> bool {
    matches!(vram_verdict(width, height), VramVerdict::Ok)
}

/// Bytes one full-grid `f32` buffer occupies. Every storage binding and every
/// `MAP_READ` staging buffer in this crate's whole-grid dispatches is exactly
/// this size, which is why one number answers for all of them.
#[must_use]
pub const fn grid_buffer_bytes(width: usize, height: usize) -> u64 {
    (width as u64) * (height as u64) * (size_of::<f32>() as u64)
}

/// Whether `gpu` can actually **bind** one full-grid buffer for this size.
///
/// A hard device limit, distinct from [`vram_verdict`]'s user-set budget: the
/// budget is a policy the owner chooses, this is arithmetic the driver
/// enforces. Both `max_storage_buffer_binding_size` (what a bind group may
/// reference) and `max_buffer_size` (what may be allocated at all) are checked,
/// because a whole-grid dispatch needs a buffer of this size on both counts.
///
/// **Why this exists as a check rather than as trust in the request**: over the
/// limit, `wgpu` does not return an error a caller can act on -- it raises a
/// validation error on the device's uncaptured-error path, which panics, and a
/// panic inside a loaded GDExtension takes the whole Godot process with it
/// (`cartalith-rust-conventions`). `request_gpu_device_from` now asks for the
/// adapter's own ceilings so this is satisfied at every size
/// `new_world_dialog.gd` offers on this project's hardware; this function is
/// what makes an adapter that genuinely cannot reach a size degrade to the CPU
/// path (`HARDWARE_ACCELERATION.md` §27) instead of crashing.
///
/// **Two grounds, not one.** The binding arithmetic above is what the device
/// *promises*; [`note_readback_failure`] is what it actually *did*. An adapter
/// can report limits that cover a size and still fail to complete a dispatch at
/// it -- this machine's integrated Radeon reports 2047 MiB and reaches
/// `create_bind_group` fine at 8192², then returns `BufferAsyncError` from the
/// `MAP_READ` staging map. There is no query that predicts that; the only
/// honest signal is having tried. So a device that has failed a readback at a
/// size is treated as not supporting that size or any larger one for the rest
/// of the session, and the caller takes the CPU path the same way it does for a
/// limits failure.
#[must_use]
pub fn device_supports_grid(gpu: &GpuDevice, width: usize, height: usize) -> bool {
    if let Some(failed_at) = readback_failure_cells(&gpu.adapter_name, gpu.adapter_vendor, gpu.adapter_backend)
        && (width as u64) * (height as u64) >= failed_at
    {
        return false;
    }
    grid_buffer_bytes(width, height) <= device_grid_limit_bytes(gpu)
}

/// Session-wide record of devices that failed a buffer readback, and the
/// smallest grid (in **cells**, not bytes) each failed at.
///
/// Keyed by adapter identity rather than by a live handle on purpose: the
/// device set is re-opened per `generate_terrain` call, and what was learnt
/// about the hardware should outlive the handle that learnt it.
static READBACK_FAILURES: RwLock<Vec<(String, u64)>> = RwLock::new(Vec::new());

/// The identity a readback failure is recorded against. Not [`device_key`]:
/// that one needs a `device_id`, which the live [`GpuDevice`] and the per-stage
/// contexts do not carry -- these three fields are what all of them do.
fn readback_key(name: &str, vendor: u32, backend: wgpu::Backend) -> String {
    format!("{vendor:04x}:{}:{name}", backend.to_str())
}

/// Record that this device could not complete a readback for a `cells`-cell
/// grid. Keeps the *smallest* failing size, so the ban is monotone: anything
/// at or above the size that failed is refused, anything below is still tried.
pub fn note_readback_failure(name: &str, vendor: u32, backend: wgpu::Backend, cells: u64) {
    let key = readback_key(name, vendor, backend);
    if let Ok(mut w) = READBACK_FAILURES.write() {
        match w.iter_mut().find(|(k, _)| *k == key) {
            Some((_, at)) => *at = (*at).min(cells),
            None => w.push((key, cells)),
        }
    }
}

/// The smallest grid this device has failed a readback at this session, if any.
#[must_use]
pub fn readback_failure_cells(name: &str, vendor: u32, backend: wgpu::Backend) -> Option<u64> {
    let key = readback_key(name, vendor, backend);
    READBACK_FAILURES.read().ok()?.iter().find(|(k, _)| *k == key).map(|(_, at)| *at)
}

/// Whether *any* readback failure is on record for this session, on any
/// adapter.
///
/// [`readback_failure_cells`] answers for one named adapter; a UI offering
/// "try the GPU again" has no adapter in hand and only needs to know whether
/// there is anything to clear. It is the exact predicate for enabling that
/// affordance: true iff a later [`clear_readback_failures`] would change
/// something.
///
/// **What it does not cover**: the per-device lost flag
/// (`lib.rs`'s `device_is_unusable`, which checks `ctx.lost()` first). A lost
/// device is already forgotten when the next `generate_terrain` opens its own
/// device, so there is nothing session-wide for a user to clear there --
/// this record is the only thing that outlives a device handle, and so the
/// only thing that ban-clearing acts on.
#[must_use]
pub fn any_readback_failure() -> bool {
    READBACK_FAILURES.read().is_ok_and(|r| !r.is_empty())
}

/// Forget every recorded readback failure. For tests that need a clean slate,
/// and for a "try the GPU again" affordance after the user changes something
/// (a driver update, a smaller world) that might make it work.
pub fn clear_readback_failures() {
    if let Ok(mut w) = READBACK_FAILURES.write() {
        w.clear();
    }
}

/// The largest single buffer `gpu` was actually opened for -- the binding of
/// [`device_supports_grid`]'s two limits. Public because a failure here is only
/// diagnosable if the number is quotable: "this device tops out at N MiB" is a
/// different report from "the GPU path is off".
#[must_use]
pub fn device_grid_limit_bytes(gpu: &GpuDevice) -> u64 {
    let l = gpu.device.limits();
    l.max_storage_buffer_binding_size.min(l.max_buffer_size)
}

/// A real, measured memory reading for one live device.
///
/// Read the two numbers together. Every dispatch in this crate frees its
/// buffers as it returns, so a reading taken *after* a generation shows
/// `allocated_bytes` back near the device's idle baseline while
/// `reserved_bytes` shows what the allocator actually took from the driver
/// and still holds — measured on this machine: 524 KB allocated against
/// 256 MB reserved after a full 256-grid generation. The reserved figure is
/// the one that answers "how much of this card is this app holding".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuMemoryUse {
    /// Bytes this process has live in GPU allocations at the moment of the
    /// reading, summed by `wgpu`'s own allocator. Ours, not the system's.
    pub allocated_bytes: u64,
    /// Bytes reserved by the allocator's memory blocks, including regions it
    /// holds but has not sub-allocated. Always >= `allocated_bytes`.
    pub reserved_bytes: u64,
}

/// This application's own GPU memory on `gpu`, from
/// `wgpu::Device::generate_allocator_report()`.
///
/// `None` when the backend does not implement the report (it is
/// `Option`-returning in `wgpu` itself). Verified present on this project's
/// development machine's Vulkan backend by running it -- a 64 MB buffer
/// moved the reported total by 64 MB -- rather than assumed from the type
/// signature.
#[must_use]
pub fn device_usage(gpu: &GpuDevice) -> Option<GpuMemoryUse> {
    // A device that lost a readback is invalid; asking it anything is how the
    // 8192² integrated-GPU run turned a graceful fallback back into a panic.
    // No reading is the honest answer here, and `None` already means that.
    if gpu.lost.load(std::sync::atomic::Ordering::Relaxed) {
        return None;
    }
    gpu.device.generate_allocator_report().map(|r| GpuMemoryUse {
        allocated_bytes: r.total_allocated_bytes,
        reserved_bytes: r.total_reserved_bytes,
    })
}

static LAST_USAGE: RwLock<Vec<(String, GpuMemoryUse)>> = RwLock::new(Vec::new());

/// Record every device in `set`'s current memory use, so a UI can show a
/// real number without paying an adapter/device handshake of its own (~1.3 s,
/// measured in `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 6) just to ask.
pub fn record_usage(set: &GpuDeviceSet) {
    let snapshot: Vec<(String, GpuMemoryUse)> =
        set.devices.iter().filter_map(|d| device_usage(d).map(|u| (d.adapter_name.clone(), u))).collect();
    if let Ok(mut w) = LAST_USAGE.write() {
        *w = snapshot;
    }
}

/// The last recording [`record_usage`] made. Empty before the first GPU
/// generation of the session -- which the UI must say, rather than showing a
/// zero that looks like a measurement.
#[must_use]
pub fn last_usage() -> Vec<(String, GpuMemoryUse)> {
    LAST_USAGE.read().map(|v| v.clone()).unwrap_or_default()
}

/// The backend the last generation actually opened a device on, or `None`
/// when it opened none.
///
/// Recorded rather than inferred, and that is the whole point of it. The
/// shell's existing readout (`menus.gd::_active_backend`) picks a backend out
/// of [`enumerate_devices`], which answers "what a device request *would*
/// prefer" -- it cannot notice that the request landed on something else, or
/// that it opened nothing at all and the run went to the CPU. On Android that
/// is the entire question: `wgpu`, `wgpu-hal` and `ash` are compiled into the
/// shipped arm64 `.so`, no `cfg(target_os = "android")` gates the GPU crate
/// off, and `engine_bridge.gd::_ready` turns `use_gpu` on at boot -- so "the
/// handset runs the CPU pipeline" has to be a reading, not an assumption.
static LAST_BACKEND: RwLock<Option<&'static str>> = RwLock::new(None);

/// Record which backend `set` opened -- or, for `None`, that this generation
/// opened no device at all.
///
/// Called on **every** generation, not only the ones that reach the GPU: a
/// reading left over from an earlier run is exactly the stale claim this
/// record exists to replace, so a CPU-only run must overwrite it.
///
/// [`GpuDeviceSet::primary`]'s backend, since that is the device every
/// non-split stage runs on.
pub fn record_opened_backend(set: Option<&GpuDeviceSet>) {
    if let Ok(mut w) = LAST_BACKEND.write() {
        *w = set.map(|s| s.primary().adapter_backend.to_str());
    }
}

/// The last recording [`record_opened_backend`] made -- `"vulkan"`, `"dx12"`,
/// `"metal"`, `"gl"`, … `None` before the first generation of the session as
/// well as after a CPU-only one.
///
/// Read it beside `WorldState::gpu_stages_used`: together they separate three
/// cases a UI must not conflate -- no device opened, a device opened but every
/// stage still fell back, and real GPU work.
#[must_use]
pub fn last_backend() -> Option<&'static str> {
    LAST_BACKEND.read().ok().and_then(|v| *v)
}

// -- Device set ----------------------------------------------------------------

/// One or more live devices plus the mode they were opened for.
pub struct GpuDeviceSet {
    devices: Vec<GpuDevice>,
    mode: MultiGpuMode,
}

impl GpuDeviceSet {
    /// The device every non-split stage runs on.
    #[must_use]
    pub fn primary(&self) -> &GpuDevice {
        &self.devices[0]
    }

    #[must_use]
    pub fn devices(&self) -> &[GpuDevice] {
        &self.devices
    }

    #[must_use]
    pub const fn mode(&self) -> MultiGpuMode {
        self.mode
    }

    /// Whether a split-tiles dispatch would actually split. False for a
    /// one-device set even in `split_tiles` mode -- there is nothing to
    /// split across -- and false for `alternate_frames`, which is not
    /// implemented.
    #[must_use]
    pub fn is_split(&self) -> bool {
        self.mode == MultiGpuMode::SplitTiles && self.devices.len() >= 2
    }

    /// Whether **every** device in the set can bind a full-grid buffer at this
    /// size -- see [`device_supports_grid`]. All, not any: a split dispatch
    /// gives each device a band of the same grid, and any stage outside the
    /// split runs whole-grid on [`Self::primary`], so one device that cannot
    /// reach the size makes the whole set unusable for it.
    #[must_use]
    pub fn supports_grid(&self, width: usize, height: usize) -> bool {
        self.devices.iter().all(|d| device_supports_grid(d, width, height))
    }
}

/// Resolve one selected key to a live adapter, or `None` if this session's
/// enumeration no longer contains it (a GPU was removed, a driver changed,
/// the preference came from another machine).
fn adapter_for_key(instance: &wgpu::Instance, key: &str) -> Option<wgpu::Adapter> {
    let mut matches: Vec<wgpu::Adapter> = pollster::block_on(instance.enumerate_adapters(COMPUTE_BACKENDS))
        .into_iter()
        .filter(|a| {
            let row = describe_adapter(a);
            device_key(&row.name, row.vendor, row.device_id) == key
        })
        .collect();
    matches.sort_by_key(|a| backend_rank(a.get_info().backend));
    matches.into_iter().next()
}

/// Whether the **automatic** adapter pick may return this device class.
///
/// One line, but it is the entire enforcement of
/// `HARDWARE_ACCELERATION.md` §5/§31's "never a software fallback" -- see
/// [`GpuDeviceInfo::is_software`] for why `force_fallback_adapter: false`
/// never enforced it. Named and separate so the rule has a check that runs
/// with no GPU present, which is the only place it can be tested on a
/// machine that *has* a real GPU.
///
/// Deliberately not applied to [`adapter_for_key`]: an explicitly selected
/// device is the user's call, and `selected_keys` is that surface.
const fn auto_pick_allows(t: wgpu::DeviceType) -> bool {
    !matches!(t, wgpu::DeviceType::Cpu)
}

/// Adapter for the *primary* device, from an explicit selection: the first
/// key that still resolves, otherwise the same
/// `PowerPreference::HighPerformance` request every version of this crate
/// before this module made. An unresolvable preference degrades to auto
/// rather than to no GPU.
///
/// **`None` when the only adapter left is a software rasterizer.** wgpu's
/// own request does not exclude one (again, see
/// [`GpuDeviceInfo::is_software`]), and one `.filter` is enough: the pick is
/// the minimum of `get_order`, which orders `Cpu` *after* every hardware
/// class, so a `Cpu` result **proves** no hardware adapter was a candidate.
/// `None` here becomes [`GpuInitError::NoAdapter`] at [`open_primary`], which
/// every caller already turns into the CPU pipeline
/// (`cartalith-engine`'s `init_gpu_device_set().ok()`,
/// `HARDWARE_ACCELERATION.md` §27) -- the correct behaviour when no real GPU
/// exists, and the one this used to bypass.
///
/// Takes the keys as an argument rather than reading [`preferences`] itself,
/// and that is a correctness requirement rather than a style choice: see
/// [`init_gpu_device_set_with`] for the bug that came of the ambient read.
/// One logical "open the selected device" operation used to consult the
/// process-global preferences **twice**, and could act on two different
/// snapshots of it.
pub(crate) fn pick_primary_adapter_for(instance: &wgpu::Instance, selected_keys: &[String]) -> Option<wgpu::Adapter> {
    if let Some(key) = selected_keys.first()
        && let Some(a) = adapter_for_key(instance, key)
    {
        return Some(a);
    }
    pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .ok()
    .filter(|a| auto_pick_allows(a.get_info().device_type))
}

/// [`pick_primary_adapter_for`] against the current ambient preferences, for
/// the callers that have no snapshot of their own ([`crate::init_gpu_shared_device`]
/// and the single-use `init_gpu_*` pipeline builders).
pub(crate) fn pick_primary_adapter(instance: &wgpu::Instance) -> Option<wgpu::Adapter> {
    pick_primary_adapter_for(instance, &preferences().selected_keys)
}

/// Open every device the current preferences call for.
///
/// Takes **one** snapshot of the process-global preferences and hands it to
/// [`init_gpu_device_set_with`], which does the actual work. Callers that
/// already hold a [`GpuPreferences`] should call that directly.
///
/// # Errors
/// [`GpuInitError::NoAdapter`] when no device could be opened at all.
pub fn init_gpu_device_set() -> Result<GpuDeviceSet, GpuInitError> {
    init_gpu_device_set_with(&preferences())
}

/// Open every device `prefs` calls for, touching no global state.
///
/// In `single_device` mode (the default) this is exactly one device and is
/// indistinguishable from [`crate::init_gpu_shared_device`]. In `split_tiles`
/// mode it opens each selected key in order; keys that fail to open are
/// skipped rather than fatal, so a machine that has lost its second GPU
/// still generates on the first.
///
/// **Why this takes `prefs` rather than reading them** (2026-08-24). The
/// previous version read [`preferences`] here *and* again, one call deeper,
/// inside `pick_primary_adapter` -- so a single "open the selected device"
/// operation consulted the process-global twice and could straddle a
/// concurrent [`set_preferences`]. Deciding `single_device` from a snapshot
/// naming the integrated GPU and then resolving the adapter from a snapshot
/// whose `selected_keys` had since been emptied takes the *auto* branch, and
/// auto is `PowerPreference::HighPerformance` -- the discrete card. The
/// caller asked for one GPU by key and silently got the other, with no error
/// anywhere.
///
/// This is how it was found: `every_enumerated_device_can_be_selected_and_opened`
/// failed on roughly one run in six, always on the integrated device and
/// never in isolation, because seven tests in `tests/multi_gpu.rs` shared
/// that one global and `cargo test` runs them in parallel. The discrete
/// iteration could not expose it -- losing the race there yields the discrete
/// GPU anyway, which is indistinguishable from success.
///
/// The single snapshot fixes the crate's half. Callers that set a preference
/// and then act on it are still two operations; the tests pass their
/// preferences here explicitly instead, which is race-free by construction.
///
/// # Errors
/// [`GpuInitError::NoAdapter`] when no device could be opened at all.
pub fn init_gpu_device_set_with(prefs: &GpuPreferences) -> Result<GpuDeviceSet, GpuInitError> {
    let instance = compute_instance();

    if prefs.mode != MultiGpuMode::SplitTiles || prefs.selected_keys.len() < 2 {
        let device = open_primary(&instance, &prefs.selected_keys)?;
        return Ok(GpuDeviceSet { devices: vec![device], mode: prefs.mode });
    }

    let devices: Vec<GpuDevice> = prefs
        .selected_keys
        .iter()
        .filter_map(|k| adapter_for_key(&instance, k))
        .filter_map(|a| {
            crate::request_gpu_device_from(
                a,
                wgpu::Features::empty(),
                REUSED_STAGE_MAX_STORAGE_BUFFERS,
                "cartalith-gpu split-tiles device",
            )
            .ok()
        })
        .map(RawGpuDevice::into_shared)
        .collect();

    if devices.is_empty() {
        // Every selected key failed -- fall back to the auto path rather
        // than to no GPU at all.
        return Ok(GpuDeviceSet { devices: vec![open_primary(&instance, &[])?], mode: MultiGpuMode::SingleDevice });
    }
    Ok(GpuDeviceSet { devices, mode: prefs.mode })
}

/// The single shared device [`crate::init_gpu_shared_device`] opens, but for
/// an explicit key list instead of the ambient one. Same features, same
/// storage-buffer floor, same label -- only the adapter choice differs.
fn open_primary(instance: &wgpu::Instance, selected_keys: &[String]) -> Result<GpuDevice, GpuInitError> {
    let adapter = pick_primary_adapter_for(instance, selected_keys).ok_or(GpuInitError::NoAdapter)?;
    Ok(crate::request_gpu_device_from(
        adapter,
        wgpu::Features::empty(),
        REUSED_STAGE_MAX_STORAGE_BUFFERS,
        "cartalith-gpu shared device",
    )?
    .into_shared())
}

// -- Split-tiles partitioning --------------------------------------------------

/// Relative throughput weight per device class, used to size each device's
/// row band.
///
/// **Measured, not assumed.** `per_device_warp_throughput_measured` in
/// `tests/multi_gpu.rs` times the whole-grid warp kernel on each device
/// alone; on this project's development machine (AMD Radeon RX 7800 XT
/// discrete + AMD Radeon integrated, both Vulkan) it reports:
///
/// | grid | discrete | integrated | ratio |
/// |---|---|---|---|
/// | 1024² | 3.5 ms | 16.1 ms | 0.217 |
/// | 2048² | 8.0 ms | 60.3 ms | 0.133 |
/// | 4096² | 48.1 ms | 283.3 ms | 0.170 |
///
/// `0.17` is the 4096² figure, chosen over the other two because that is
/// the size at which splitting is not already lost to fixed overhead
/// (~1.8 ms, also measured) — sizing the bands correctly at a size where
/// the split cannot win regardless would be optimising the wrong point.
/// The full numbers and what they mean are in `HARDWARE_ACCELERATION.md`'s
/// 2026-08-20 section.
///
/// A fixed table rather than a running self-calibration on purpose: the band
/// boundaries are part of what the result depends on, so weights that
/// drifted with measurement noise would make the same seed on the same
/// devices produce different numbers between runs
/// (`DECISIONS.md` §7a requires determinism *within* a path). The cost is
/// that these are one machine's ratios; the test that produced them is
/// shipped so another machine can re-measure rather than inherit them.
#[must_use]
pub const fn device_weight(t: wgpu::DeviceType) -> f64 {
    match t {
        wgpu::DeviceType::DiscreteGpu => 1.0,
        wgpu::DeviceType::IntegratedGpu => 0.17,
        wgpu::DeviceType::VirtualGpu | wgpu::DeviceType::Other => 0.5,
        // A software rasterizer is never given work in a split.
        wgpu::DeviceType::Cpu => 0.0,
    }
}

/// Partition `height` rows across devices in proportion to `weights`.
///
/// Returns `(y_offset, rows)` per device, contiguous, covering exactly
/// `0..height`, in the same order as `weights`. A device may get zero rows
/// (weight 0, or a grid with fewer rows than devices) -- callers skip those
/// rather than dispatching an empty band.
///
/// Deterministic by construction: the boundaries come from a cumulative-sum
/// `floor`, so the same `(height, weights)` always yields the same split, on
/// any machine, in any order.
#[must_use]
pub fn split_rows(height: u32, weights: &[f64]) -> Vec<(u32, u32)> {
    let total: f64 = weights.iter().filter(|w| **w > 0.0).sum();
    if weights.is_empty() || total <= 0.0 {
        // No usable weight anywhere: give everything to the first device
        // rather than dispatching nothing.
        let mut out = vec![(0u32, 0u32); weights.len()];
        if let Some(first) = out.first_mut() {
            *first = (0, height);
        }
        return out;
    }

    let mut out = Vec::with_capacity(weights.len());
    let mut cum = 0.0f64;
    let mut prev_edge = 0u32;
    for (i, w) in weights.iter().enumerate() {
        cum += w.max(0.0);
        let edge = if i + 1 == weights.len() {
            height
        } else {
            ((cum / total) * f64::from(height)).floor().min(f64::from(height)) as u32
        };
        let edge = edge.max(prev_edge);
        out.push((prev_edge, edge - prev_edge));
        prev_edge = edge;
    }
    out
}

/// The per-device weights [`split_rows`] should be called with for `set`.
#[must_use]
pub fn set_weights(set: &GpuDeviceSet) -> Vec<f64> {
    set.devices.iter().map(|d| device_weight(d.device_type)).collect()
}

// -- RawGpuDevice -> GpuDevice -------------------------------------------------

impl RawGpuDevice {
    pub(crate) fn into_shared(self) -> GpuDevice {
        GpuDevice {
            adapter_name: self.adapter_name,
            adapter_vendor: self.adapter_vendor,
            adapter_backend: self.adapter_backend,
            device_type: self.device_type,
            device: self.device,
            queue: self.queue,
            lost: self.lost,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `READBACK_FAILURES` is one process-wide static, and `cargo test` runs
    /// this module's tests on many threads at once. Every test that writes
    /// it -- and in particular every test that *clears* it -- takes this
    /// first, so a clear cannot wipe a sibling's record mid-assertion.
    /// Recovered from poisoning on purpose: a panicking test has already
    /// failed, and it must not turn every other readback test into a
    /// secondary failure.
    static READBACK_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn readback_test_guard() -> std::sync::MutexGuard<'static, ()> {
        READBACK_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// The one behaviour that makes `last_backend()` a measurement rather
    /// than a souvenir: a generation that opened no device must **overwrite**
    /// the previous reading, not inherit it. A CPU-only run still reporting
    /// `"vulkan"` from the run before it is precisely the unfalsifiable claim
    /// this record was added to replace.
    ///
    /// The `Some` half needs a real adapter, so it is driven through the
    /// static directly here and left to the device passes on hardware.
    ///
    /// No test lock, unlike `READBACK_FAILURES` above: this is the only test
    /// in the crate that touches `LAST_BACKEND`, and `record_opened_backend`
    /// is never reached from one, so there is no sibling to race.
    #[test]
    fn a_generation_that_opens_nothing_clears_the_last_backend() {
        *LAST_BACKEND.write().unwrap_or_else(std::sync::PoisonError::into_inner) = Some("vulkan");
        assert_eq!(last_backend(), Some("vulkan"), "the static is readable through the accessor at all");

        record_opened_backend(None);
        assert_eq!(last_backend(), None, "a CPU-only generation must not inherit the last GPU one's backend");
    }

    fn row(name: &str, vendor: u32, device_id: u32, t: wgpu::DeviceType, b: wgpu::Backend) -> AdapterRow {
        AdapterRow {
            name: name.to_string(),
            vendor,
            device_id,
            device_type: t,
            backend: b,
            driver: String::new(),
            driver_info: String::new(),
            max_buffer_size: 1 << 31,
            max_storage_buffer_binding_size: u64::from(u32::MAX) - 3,
            supports_compute: true,
        }
    }

    /// The exact six rows this project's development machine really
    /// enumerates (captured from a run, not invented): one discrete and one
    /// integrated AMD GPU each visible over Vulkan and Dx12, the Windows
    /// software rasterizer over Dx12, and the discrete GPU again over
    /// OpenGL with **zero** vendor/device ids.
    fn real_machine_rows() -> Vec<AdapterRow> {
        vec![
            row("AMD Radeon RX 7800 XT", 0x1002, 0x747e, wgpu::DeviceType::DiscreteGpu, wgpu::Backend::Vulkan),
            row("AMD Radeon(TM) Graphics", 0x1002, 0x13c0, wgpu::DeviceType::IntegratedGpu, wgpu::Backend::Vulkan),
            row("AMD Radeon RX 7800 XT", 0x1002, 0x747e, wgpu::DeviceType::DiscreteGpu, wgpu::Backend::Dx12),
            row("AMD Radeon(TM) Graphics", 0x1002, 0x13c0, wgpu::DeviceType::IntegratedGpu, wgpu::Backend::Dx12),
            row("Microsoft Basic Render Driver", 0x1414, 0x008c, wgpu::DeviceType::Cpu, wgpu::Backend::Dx12),
            row("AMD Radeon RX 7800 XT", 0, 0, wgpu::DeviceType::Other, wgpu::Backend::Gl),
        ]
    }

    #[test]
    fn group_adapters_collapses_the_real_machines_six_rows_to_three_devices() {
        let devs = group_adapters(real_machine_rows());
        assert_eq!(devs.len(), 3, "six adapter rows, three physical devices");

        assert_eq!(devs[0].name, "AMD Radeon RX 7800 XT");
        assert_eq!(devs[0].device_type, wgpu::DeviceType::DiscreteGpu);
        assert_eq!(devs[0].backend, wgpu::Backend::Vulkan, "Vulkan outranks Dx12 and Gl");
        assert_eq!(devs[0].alternate_backends, vec![wgpu::Backend::Dx12, wgpu::Backend::Gl]);
        assert!(!devs[0].is_software);
        assert_eq!(devs[0].key, "1002:747e:AMD Radeon RX 7800 XT");

        assert_eq!(devs[1].device_type, wgpu::DeviceType::IntegratedGpu);
        assert_eq!(devs[1].alternate_backends, vec![wgpu::Backend::Dx12]);

        assert!(devs[2].is_software, "the Basic Render Driver is sorted last and flagged");
    }

    /// The case that would be silently broken by keying on the name: two
    /// identical cards, which is the canonical multi-GPU rig.
    #[test]
    fn group_adapters_keeps_two_identical_cards_apart() {
        let devs = group_adapters(vec![
            row("AMD Radeon RX 7800 XT", 0x1002, 0x747e, wgpu::DeviceType::DiscreteGpu, wgpu::Backend::Vulkan),
            row("AMD Radeon RX 7800 XT", 0x1002, 0x747f, wgpu::DeviceType::DiscreteGpu, wgpu::Backend::Vulkan),
        ]);
        assert_eq!(devs.len(), 2);
        assert_ne!(devs[0].key, devs[1].key);
    }

    /// A zero-id row with two same-named candidates is ambiguous, so it
    /// stays separate instead of being attached to an arbitrary one.
    #[test]
    fn group_adapters_leaves_an_ambiguous_zero_id_row_alone() {
        let devs = group_adapters(vec![
            row("Card", 0x1002, 0x0001, wgpu::DeviceType::DiscreteGpu, wgpu::Backend::Vulkan),
            row("Card", 0x1002, 0x0002, wgpu::DeviceType::DiscreteGpu, wgpu::Backend::Vulkan),
            row("Card", 0, 0, wgpu::DeviceType::Other, wgpu::Backend::Gl),
        ]);
        assert_eq!(devs.len(), 3);
        assert_eq!(devs[2].key, "name:Card");
    }

    #[test]
    fn group_adapters_on_a_headless_machine_returns_nothing() {
        assert!(group_adapters(Vec::new()).is_empty());
    }

    #[test]
    fn split_rows_covers_every_row_exactly_once() {
        for h in [1u32, 2, 7, 8, 64, 511, 512, 1024] {
            for weights in [vec![1.0], vec![1.0, 1.0], vec![1.0, 0.2], vec![1.0, 0.2, 0.5]] {
                let bands = split_rows(h, &weights);
                assert_eq!(bands.len(), weights.len());
                let mut expect = 0u32;
                for (y, rows) in &bands {
                    assert_eq!(*y, expect, "bands must be contiguous (h={h}, w={weights:?})");
                    expect += rows;
                }
                assert_eq!(expect, h, "bands must cover the grid exactly (h={h}, w={weights:?})");
            }
        }
    }

    #[test]
    fn split_rows_is_proportional_to_the_weights() {
        let bands = split_rows(1000, &[1.0, 0.2]);
        assert_eq!(bands[0], (0, 833));
        assert_eq!(bands[1], (833, 167));
    }

    #[test]
    fn split_rows_gives_a_zero_weight_device_nothing() {
        let bands = split_rows(100, &[1.0, 0.0]);
        assert_eq!(bands[0], (0, 100));
        assert_eq!(bands[1], (100, 0));
    }

    #[test]
    fn split_rows_with_no_usable_weight_still_covers_the_grid() {
        let bands = split_rows(100, &[0.0, 0.0]);
        assert_eq!(bands[0], (0, 100));
        assert_eq!(bands[1], (0, 0));
    }

    #[test]
    fn mode_and_fallback_round_trip_through_their_string_names() {
        for m in [MultiGpuMode::SingleDevice, MultiGpuMode::SplitTiles, MultiGpuMode::AlternateFrames] {
            assert_eq!(MultiGpuMode::parse(m.as_str()), Some(m));
        }
        for f in [VramFallback::CpuTilePass, VramFallback::ReduceWorkingRes, VramFallback::FailWithError] {
            assert_eq!(VramFallback::parse(f.as_str()), Some(f));
        }
        assert_eq!(MultiGpuMode::parse("nonsense"), None);
        assert_eq!(VramFallback::parse("nonsense"), None);
    }

    #[test]
    fn the_two_unimplemented_choices_say_so() {
        assert!(!MultiGpuMode::AlternateFrames.is_implemented());
        assert!(MultiGpuMode::SplitTiles.is_implemented());
        assert!(!VramFallback::ReduceWorkingRes.is_implemented());
        assert!(VramFallback::CpuTilePass.is_implemented());
    }

    #[test]
    fn working_set_estimate_is_ten_f32_grids() {
        assert_eq!(gpu_working_set_bytes(512, 256), 512 * 256 * 4 * 10);
        // The two sizes that bracket a plausible cap, spelled out rather
        // than asserted loosely: 4096² is 640 MB and 8192² is 2.5 GB, so a
        // 1 GB budget is exactly the setting that admits the first and
        // refuses the second. (A first pass asserted 4096² was already past
        // 2 GB -- it is not, and the test caught the arithmetic.)
        assert_eq!(gpu_working_set_bytes(4096, 4096), 640 * 1024 * 1024);
        assert_eq!(gpu_working_set_bytes(8192, 8192), 2560 * 1024 * 1024);
        let one_gb = 1024 * 1024 * 1024;
        assert_eq!(vram_verdict_for(gpu_working_set_bytes(4096, 4096), one_gb, VramFallback::FailWithError), VramVerdict::Ok);
        assert_eq!(
            vram_verdict_for(gpu_working_set_bytes(8192, 8192), one_gb, VramFallback::FailWithError),
            VramVerdict::Fail
        );
    }

    #[test]
    fn no_budget_never_denies() {
        assert_eq!(vram_verdict_for(u64::MAX, 0, VramFallback::FailWithError), VramVerdict::Ok);
    }

    #[test]
    fn budget_denies_only_above_the_cap_and_honours_the_fallback() {
        let gb = 1024 * 1024 * 1024;
        assert_eq!(vram_verdict_for(gb, gb, VramFallback::FailWithError), VramVerdict::Ok, "equal fits");
        assert_eq!(vram_verdict_for(gb + 1, gb, VramFallback::FailWithError), VramVerdict::Fail);
        assert_eq!(vram_verdict_for(gb + 1, gb, VramFallback::CpuTilePass), VramVerdict::FallBackToCpu);
        assert_eq!(
            vram_verdict_for(gb + 1, gb, VramFallback::ReduceWorkingRes),
            VramVerdict::FallBackToCpu,
            "the un-implemented choice degrades safely rather than pretending"
        );
    }

    /// The readback-failure record's own arithmetic, with no hardware:
    /// smallest-wins, at-or-above is banned, below still allowed.
    ///
    /// Uses a name no real adapter has, so it cannot collide with a
    /// concurrently-running device test's own record.
    #[test]
    fn a_recorded_readback_failure_bans_that_size_and_larger_only() {
        let _guard = readback_test_guard();
        const NAME: &str = "cartalith test pseudo-adapter";
        const VENDOR: u32 = 0xdead;
        let backend = wgpu::Backend::Noop;

        assert_eq!(readback_failure_cells(NAME, VENDOR, backend), None, "nothing recorded yet");
        note_readback_failure(NAME, VENDOR, backend, 8192 * 8192);
        assert_eq!(readback_failure_cells(NAME, VENDOR, backend), Some(8192 * 8192));

        // A later, larger failure must not raise the ceiling back up.
        note_readback_failure(NAME, VENDOR, backend, 16384 * 16384);
        assert_eq!(readback_failure_cells(NAME, VENDOR, backend), Some(8192 * 8192), "smallest failure wins");

        // A smaller one does lower it.
        note_readback_failure(NAME, VENDOR, backend, 4096 * 4096);
        assert_eq!(readback_failure_cells(NAME, VENDOR, backend), Some(4096 * 4096));

        // A different adapter is untouched by any of it.
        assert_eq!(readback_failure_cells(NAME, VENDOR + 1, backend), None);
        assert_eq!(readback_failure_cells(NAME, VENDOR, wgpu::Backend::Vulkan), None);
    }

    /// The predicate `menus.gd`'s `Preferences > Performance > Try the GPU
    /// again` row is enabled by, and the clearer that row calls, over the
    /// three states the row can be in.
    ///
    /// The third case is the one worth a test: the row is drawn disabled when
    /// nothing is banned, so clearing on an empty record should never be
    /// reached -- but a menu accelerator, a replayed command or a second click
    /// can reach a disabled row's handler anyway, and it must be inert rather
    /// than an error or a panic.
    #[test]
    fn any_readback_failure_tracks_the_record_and_clearing_is_idempotent() {
        let _guard = readback_test_guard();
        const NAME: &str = "cartalith clear-path pseudo-adapter";
        const VENDOR: u32 = 0xbeef;
        let backend = wgpu::Backend::Noop;

        // Clearing with nothing recorded: a harmless no-op, twice over.
        clear_readback_failures();
        assert!(!any_readback_failure(), "nothing recorded, so nothing to clear");
        clear_readback_failures();
        assert!(!any_readback_failure(), "clearing an empty record stays empty");

        // A recorded failure is what turns the row on.
        note_readback_failure(NAME, VENDOR, backend, 8192 * 8192);
        assert!(any_readback_failure(), "a recorded failure is visible without naming the adapter");

        // And clearing turns it back off, per-adapter record and all.
        clear_readback_failures();
        assert!(!any_readback_failure());
        assert_eq!(
            readback_failure_cells(NAME, VENDOR, backend),
            None,
            "the per-adapter ban is gone too, not merely the summary"
        );
    }

    /// [`compute_instance`] now reads `WGPU_BACKEND`, and the signal-11 fix
    /// in [`COMPUTE_BACKENDS`] survives that only because the mask is applied
    /// as an **intersection afterwards**. This is that guarantee, checked
    /// against the real parser `wgpu::Backends::from_env` uses, without
    /// touching the process environment (which `cargo test`'s threads share).
    ///
    /// The first assertion is the other half: with nothing set, the
    /// descriptor's `Backends::all()` intersects to exactly the literal the
    /// old code assigned, so the default path did not move.
    #[test]
    fn no_environment_value_can_put_opengl_back() {
        assert_eq!(
            wgpu::Backends::all() & COMPUTE_BACKENDS,
            COMPUTE_BACKENDS,
            "unset WGPU_BACKEND must leave the mask exactly as it was"
        );
        assert!(!COMPUTE_BACKENDS.contains(wgpu::Backends::GL));

        for s in ["gl", "opengl", "gles", "gl,vulkan", "vulkan", "dx12", "d3d12", "noop", "", "nonsense"] {
            let masked = wgpu::Backends::from_comma_list(s) & COMPUTE_BACKENDS;
            assert!(!masked.contains(wgpu::Backends::GL), "WGPU_BACKEND={s:?} must not reach the GL backend");
            assert!(
                COMPUTE_BACKENDS.contains(masked),
                "the mask is an upper bound: WGPU_BACKEND={s:?} must not add a backend to it"
            );
        }

        // And the escape hatch is not merely safe, it works: dx12 is the
        // comparison `backend_rank`'s Vulkan-first order could not be
        // measured against before.
        assert_eq!(wgpu::Backends::from_comma_list("dx12") & COMPUTE_BACKENDS, wgpu::Backends::DX12);
    }

    /// `HARDWARE_ACCELERATION.md` §5/§31's "never a software fallback", as a
    /// check that runs with no GPU -- which is the only way to test it on a
    /// machine that has one, since there the automatic pick would return the
    /// real card either way.
    ///
    /// Every variant is listed rather than only `Cpu`, so adding a device
    /// class to `wgpu` cannot silently join the banned set (or the allowed
    /// one) without this failing to compile or failing here.
    #[test]
    fn only_a_software_rasterizer_is_barred_from_the_automatic_pick() {
        assert!(!auto_pick_allows(wgpu::DeviceType::Cpu), "the whole point: software is never picked for you");
        assert!(auto_pick_allows(wgpu::DeviceType::DiscreteGpu));
        assert!(auto_pick_allows(wgpu::DeviceType::IntegratedGpu));
        assert!(auto_pick_allows(wgpu::DeviceType::VirtualGpu));
        // `Other` is what an OpenGL adapter reports (see `group_adapters`),
        // and a real GPU behind a driver that will not say so must not be
        // refused -- barring it would turn "the driver is vague" into "no
        // GPU".
        assert!(auto_pick_allows(wgpu::DeviceType::Other));

        // The class the ban is about is exactly the one `is_software` flags,
        // and exactly the one a split gives no rows to. Three places, one
        // rule; this is what keeps them from drifting apart.
        let sw = group_adapters(vec![row(
            "Microsoft Basic Render Driver",
            0x1414,
            0x008c,
            wgpu::DeviceType::Cpu,
            wgpu::Backend::Dx12,
        )]);
        assert!(sw[0].is_software);
        assert!(!auto_pick_allows(sw[0].device_type));
        assert_eq!(device_weight(sw[0].device_type), 0.0);
    }

    /// The default install must behave exactly as it did before this
    /// module: auto device, one device, no cap.
    #[test]
    fn default_preferences_change_nothing() {
        let p = GpuPreferences::default();
        assert!(p.selected_keys.is_empty());
        assert_eq!(p.mode, MultiGpuMode::SingleDevice);
        assert_eq!(p.vram_budget_bytes, 0);
        assert_eq!(p.fallback, VramFallback::CpuTilePass);
        assert!(gpu_allowed_for_grid(8192, 8192), "no cap set => never refused");
    }
}
