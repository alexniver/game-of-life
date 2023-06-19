use wgpu::VertexAttribute;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: [f32; 2],
}

impl Vertex {
    pub fn new(pos: [f32; 2]) -> Self {
        Self { pos }
    }

    const ATTRS: [VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];
    pub fn vertex_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as _,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRS,
        }
    }

    pub fn triangle() -> [Vertex; 3] {
        [
            Vertex::new([0.1, 0.1]),
            Vertex::new([0.9, 0.1]),
            Vertex::new([0.1, 0.9]),
        ]
    }

    pub fn rect() -> [Vertex; 6] {
        [
            Vertex::new([0.1, 0.1]),
            Vertex::new([0.9, 0.1]),
            Vertex::new([0.1, 0.9]),
            Vertex::new([0.9, 0.1]),
            Vertex::new([0.9, 0.9]),
            Vertex::new([0.1, 0.9]),
        ]
    }
}
