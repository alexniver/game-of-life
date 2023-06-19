struct VertexIn {
    @location(0) pos: vec2<f32>,
}

struct VertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@group(0)@binding(0)
var<uniform> proj: mat4x4<f32>;

@group(1)@binding(0)
var<uniform> grid_size: vec2<f32>;
@group(1)@binding(1)
var<uniform> grid_pixel_size: vec2<f32>;
@group(1)@binding(2)
var<storage> grid_cell_arr: array<u32>;

@vertex
fn vs_main(in: VertexIn, @builtin(instance_index) instance_idx: u32) -> VertexOut {
    let idx = f32(instance_idx);

    let origin_pos = grid_pixel_size / 2.0 * -1.0;
    let cell_size = grid_pixel_size / grid_size;

    let cell_idx = vec2<f32>(idx % grid_size.x, floor(idx / grid_size.x));

    var out: VertexOut;

    let cell_val = grid_cell_arr[instance_idx];
    if cell_val == 0u {
        out.clip_pos = vec4<f32>(0.0);
        out.color = vec3<f32>(0.0);
    } else {
        let cell_pos = cell_idx * cell_size * f32(grid_cell_arr[instance_idx]);
        out.clip_pos = proj * vec4<f32>(in.pos / grid_size * grid_pixel_size + origin_pos + cell_pos, 0.0, 1.0);
        let c = cell_idx / grid_size;
        out.color = vec3<f32>(c, 1.0 - c.x);
    }
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
