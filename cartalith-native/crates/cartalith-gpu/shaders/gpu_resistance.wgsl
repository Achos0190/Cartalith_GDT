// GPU-safe compute_resistance (GPU_LAYER_INTEGRATION_SCOPE.md milestone 4).
//
// Trivial per-cell formula, no noise, no compounding transcendental
// functions: resistance[i] = min(crustal_per_plate[plate_id[i]]*0.6 +
// age[i]*0.4, 1.0). `crustal_per_plate` is a small (num_plates-length)
// array precomputed on CPU as `plates[k].base.max(0.0)` for each plate k
// -- an O(num_plates) step, not the O(cells) per-cell workload this
// kernel accelerates, and it lets this shader avoid carrying the full
// `Plate` struct's other fields (x/y/vx/vy) this formula never reads.

struct ResistanceParams {
    width: u32,
    height: u32,
    _pad0: u32,
    _pad1: u32,
}

@group(0) @binding(0) var<uniform> params: ResistanceParams;
@group(0) @binding(1) var<storage, read> plate_id: array<u32>;
@group(0) @binding(2) var<storage, read> age: array<f32>;
@group(0) @binding(3) var<storage, read> crustal_per_plate: array<f32>;
@group(0) @binding(4) var<storage, read_write> out_resistance: array<f32>;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x >= params.width || gid.y >= params.height {
        return;
    }
    let idx = gid.y * params.width + gid.x;
    let crustal = crustal_per_plate[plate_id[idx]];
    out_resistance[idx] = min(crustal * 0.6 + age[idx] * 0.4, 1.0);
}
