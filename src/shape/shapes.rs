use wgpu::Color;

use crate::render::instance::Instance;

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
    /// size of the shape
    fn scale(&mut self, width: f32, height: f32) -> &mut Self;

    /// location of the shape in the frame
    fn pos(&mut self, x: f32, y: f32) -> &mut Self;

    /// rotation of the shape in radians
    fn rotation(&mut self, rotation: f32) -> &mut Self;

    /// fill color of the shape
    fn color(&mut self, red: f32, green: f32, blue: f32) -> &mut Self;

    /// fill color of the shape
    fn color_from_color(&mut self, color: Color) -> &mut Self;

    /// render layer of the shape
    ///
    /// higher layer -> foreground
    fn layer(&mut self, layer: u16) -> &mut Self;

    fn to_instance(&self) -> Instance;
}
