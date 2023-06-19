use rand::Rng;

pub const GRID_SIZE: u32 = 40;
pub const GRID_PIXEL_SIZE: u32 = 800;

pub struct Grid {
    pub cell_arr: [u32; (GRID_SIZE * GRID_SIZE) as _],
    pub step: u32,
}

impl Grid {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut cell_arr = [0; (GRID_SIZE * GRID_SIZE) as _];
        for i in 0..(GRID_SIZE * GRID_SIZE) {
            let r = rng.gen::<f32>() > 0.7;
            let v = if r { 1 } else { 0 };
            cell_arr[i as usize] = v;
        }
        Self { cell_arr, step: 0 }
    }
}
