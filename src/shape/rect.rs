use crate::render::instance::Instance;
use wgpu::Color;

use crate::shape::shapes::{BasicShape, BasicShapeData};

/// Shape which can be render and created which though the [rect](shape_renderer::ShapeRenderer::rect)
#[derive(Default)]
pub struct Rect {
    pub(crate) data: BasicShapeData,
}

impl BasicShape for Rect {
    fn scale(&mut self, width: f32, height: f32) -> &mut Self {
        self.data.scale = (width, height);
        self
    }

    fn pos(&mut self, x: f32, y: f32) -> &mut Self {
        self.data.pos = (x, y);
        self
    }

    fn rotation(&mut self, rotation: f32) -> &mut Self {
        self.data.rotation = rotation;
        self
    }

    fn color(&mut self, red: f32, green: f32, blue: f32) -> &mut Self {
        self.data.color = (red, green, blue);
        self
    }

    fn color_from_color(&mut self, color: Color) -> &mut Self {
        self.data.color = (color.r as f32, color.g as f32, color.b as f32);
        self
    }

    fn layer(&mut self, layer: u16) -> &mut Self {
        self.data.layer = layer;
        self
    }

    fn to_instance(&self) -> Instance {
        (&self.data).into()
    }
}
