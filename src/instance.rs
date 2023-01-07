#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub position: [f32; 2],
    pub scale: [f32; 2],
    pub rotation: f32,
    //todo transparency
    //problem: transparency only works in the same drawCall
    pub color: [f32; 3],
    pub layer: u32,
}

impl Instance {
    const ATTRIBS: [wgpu::VertexAttribute; 5] =
        wgpu::vertex_attr_array![1 => Float32x2,2 => Float32x2,3 => Float32,4 => Float32x3, 5 => Uint32];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}