@group(0)@binding(0)
var<uniform> grid_size: vec2<f32>;
@group(0)@binding(1)
var<storage> cell_in: array<u32>;
@group(0)@binding(2)
var<storage, read_write> cell_out: array<u32>;

fn cell_index(cell: vec2<u32>) -> u32 {
    return (cell.y % u32(grid_size.y)) * u32(grid_size.x) + cell.x % u32(grid_size.x);
}

fn cell_active(x: u32, y: u32) -> u32 {
    return cell_in[cell_index(vec2<u32>(x, y))];
}

@compute
@workgroup_size(8, 8)
fn cp_main(@builtin(global_invocation_id) cell: vec3<u32>) {
    let max_x = u32(grid_size.x) - 1u;
    let max_y = u32(grid_size.y) - 1u;

    var cell_x_left = max_x;
    if cell.x > 0u {
        cell_x_left = cell.x - 1u;
    };

    var cell_x_right = cell.x + 1u;
    if cell.x == max_x {
        cell_x_right = 0u;
    };

    var cell_y_left = max_y;
    if cell.y > 0u {
        cell_y_left = cell.y - 1u;
    };

    var cell_y_right = cell.y + 1u;
    if cell.y == max_y {
        cell_y_right = 0u;
    };

    let active_neighbours = cell_active(cell_x_right, cell_y_right) + cell_active(cell_x_right, cell.y) + cell_active(cell_x_right, cell_y_left) + cell_active(cell.x, cell_y_right) + cell_active(cell.x, cell.y) + cell_active(cell.x, cell_y_left) + cell_active(cell_x_left, cell_y_right) + cell_active(cell_x_left, cell.y) + cell_active(cell_x_left, cell_y_left);

    let idx = cell_index(cell.xy);
    switch active_neighbours {
        case 2u: {
            cell_out[idx] = cell_in[idx];
        }
        case 3u: {
            cell_out[idx] = 1u;
        }
        default: {
            cell_out[idx] = 0u;
        }
    }
}
