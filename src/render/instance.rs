use wgpu::VertexStepMode;
use wgpu_noboiler::vertex::Vertex;

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

impl Vertex<5> for Instance {
    const STEP_MODE: VertexStepMode = VertexStepMode::Instance;

    const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![1 => Float32x2,2 => Float32x2,3 => Float32,4 => Float32x3, 5 => Uint32];
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextureInstance {
    pub position: [f32; 2],
    pub scale: [f32; 2],
    pub rotation: f32,
    pub layer: u32,
    pub texture_position: [f32; 2],
    pub texture_scale: [f32; 2],
}

impl Vertex<6> for TextureInstance {
    const STEP_MODE: VertexStepMode = VertexStepMode::Instance;

    const ATTRIBS: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![1 => Float32x2,2 => Float32x2,3 => Float32, 4 => Uint32, 5 => Float32x2,6 => Float32x2];
}
