use glam::{Mat4, Quat, Vec3};

const MAT4_NUM: usize = 1;
const MAT4IT_NUM: usize = 2;

pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    pub fn to_raw(&self) -> TransformRaw {
        let mat4 =
            Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation);
        TransformRaw {
            model: mat4.to_cols_array_2d(),
        }
    }

    pub fn to_rawit(&self) -> TransformRawIT {
        let mat4 =
            Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation);
        let mat4_it = mat4.inverse().transpose();

        let mut combine: [[f32; 4]; 4 * MAT4IT_NUM] = [[0.0; 4]; 4 * MAT4IT_NUM];
        combine[0..4].copy_from_slice(&mat4.to_cols_array_2d());
        combine[4..8].copy_from_slice(&mat4_it.to_cols_array_2d());

        TransformRawIT { model: combine }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformRaw {
    model: [[f32; 4]; 4 * MAT4_NUM],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformRawIT {
    model: [[f32; 4]; 4 * MAT4IT_NUM],
}
