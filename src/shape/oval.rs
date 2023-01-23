use wgpu::Color;

use crate::render::instance::Instance;
use crate::shape::shapes::{BasicShape, BasicShapeData};

/// Shape which can be render and created which though the [oval](shape_renderer::ShapeRenderer::oval)
pub struct Oval {
    pub(crate) data: BasicShapeData,
    pub(crate) detail: u32,
}

impl BasicShape for Oval {
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

impl Oval {
    ///segment count of the oval
    ///
    /// higher -> smoother circle
    pub fn segment_count(&mut self, segment_count: u32) -> &mut Self {
        self.detail = segment_count;
        self
    }
}

impl Default for Oval {
    fn default() -> Self {
        Oval {
            data: Default::default(),
            detail: 128,
        }
    }
}
