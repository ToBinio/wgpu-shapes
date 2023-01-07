use wgpu::Color;

use crate::instance::Instance;

pub struct BasicShapeData {
    pub scale: (f32, f32),
    pub pos: (f32, f32),
    pub rotation: f32,
    pub color: (f32, f32, f32),
    pub layer: u16,
}

impl From<&BasicShapeData> for Instance {
    fn from(data: &BasicShapeData) -> Self {
        Instance {
            position: [data.pos.0, data.pos.1],
            scale: [data.scale.0, data.scale.1],
            rotation: data.rotation,
            color: [data.color.0, data.color.1, data.color.2],
            layer: data.layer as u32,
        }
    }
}

impl Default for BasicShapeData {
    fn default() -> Self {
        BasicShapeData {
            scale: (20.0, 20.0),
            pos: (0.0, 0.0),
            rotation: 0.0,
            color: (0.0, 0.0, 1.0),
            layer: 0,
        }
    }
}

pub trait BasicShape {
    fn scale(&mut self, width: f32, height: f32) -> &mut Self;

    fn pos(&mut self, x: f32, y: f32) -> &mut Self;

    fn rotation(&mut self, rotation: f32) -> &mut Self;

    fn color(&mut self, red: f32, green: f32, blue: f32) -> &mut Self;

    fn color_from_color(&mut self, color: Color) -> &mut Self;

    fn layer(&mut self, layer: u16) -> &mut Self;

    fn to_instance(&self) -> Instance;
}