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
    /// high-performance real adapter, never a software fallback", and every
    /// `request_adapter` in this crate already passes
    /// `force_fallback_adapter: false`.
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

fn adapter_rows() -> Vec<AdapterRow> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    pollster::block_on(instance.enumerate_adapters(wgpu::Backends::all()))
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
}

/// Resolve one selected key to a live adapter, or `None` if this session's
/// enumeration no longer contains it (a GPU was removed, a driver changed,
/// the preference came from another machine).
fn adapter_for_key(instance: &wgpu::Instance, key: &str) -> Option<wgpu::Adapter> {
    let mut matches: Vec<wgpu::Adapter> = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::all()))
        .into_iter()
        .filter(|a| {
            let row = describe_adapter(a);
            device_key(&row.name, row.vendor, row.device_id) == key
        })
        .collect();
    matches.sort_by_key(|a| backend_rank(a.get_info().backend));
    matches.into_iter().next()
}

/// Adapter for the *primary* device: the first selected key if it still
/// resolves, otherwise the same `PowerPreference::HighPerformance` request
/// every version of this crate before this module made. An unresolvable
/// preference degrades to auto rather than to no GPU.
pub(crate) fn pick_primary_adapter(instance: &wgpu::Instance) -> Option<wgpu::Adapter> {
    if let Some(key) = preferences().selected_keys.first()
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
}

/// Open every device the current preferences call for.
///
/// In `single_device` mode (the default) this is exactly one device and is
/// indistinguishable from [`crate::init_gpu_shared_device`]. In `split_tiles`
/// mode it opens each selected key in order; keys that fail to open are
/// skipped rather than fatal, so a machine that has lost its second GPU
/// still generates on the first.
///
/// # Errors
/// [`GpuInitError::NoAdapter`] when no device could be opened at all.
pub fn init_gpu_device_set() -> Result<GpuDeviceSet, GpuInitError> {
    let prefs = preferences();
    if prefs.mode != MultiGpuMode::SplitTiles || prefs.selected_keys.len() < 2 {
        return Ok(GpuDeviceSet { devices: vec![crate::init_gpu_shared_device()?], mode: prefs.mode });
    }

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
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
        return Ok(GpuDeviceSet { devices: vec![crate::init_gpu_shared_device()?], mode: MultiGpuMode::SingleDevice });
    }
    Ok(GpuDeviceSet { devices, mode: prefs.mode })
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
