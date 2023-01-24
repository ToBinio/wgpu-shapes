use crate::render::instance::TextureInstance;

pub struct Image {
    pub(crate) scale: (f32, f32),
    pub(crate) pos: (f32, f32),
    pub(crate) rotation: f32,
    pub(crate) layer: u16,
    pub(crate) texture_pos: (f32, f32),
    pub(crate) texture_scale: (f32, f32),
}

impl Image {
    pub fn scale(&mut self, width: f32, height: f32) -> &mut Self {
        self.scale = (width, height);
        self
    }

    pub fn pos(&mut self, x: f32, y: f32) -> &mut Self {
        self.pos = (x, y);
        self
    }

    pub fn rotation(&mut self, rotation: f32) -> &mut Self {
        self.rotation = rotation;
        self
    }

    pub fn layer(&mut self, layer: u16) -> &mut Self {
        self.layer = layer;
        self
    }

    pub fn to_instance(&self) -> TextureInstance {
        TextureInstance {
            position: [self.pos.0, self.pos.1],
            scale: [self.scale.0, self.scale.1],
            rotation: self.rotation,
            layer: self.layer as u32,
            texture_position: [self.texture_pos.0, self.texture_pos.1],
            texture_scale: [self.texture_scale.0, self.texture_scale.1],
        }
    }
}

impl Default for Image {
    fn default() -> Self {
        Image {
            scale: (20.0, 20.0),
            pos: (0.0, 0.0),
            rotation: 0.0,
            layer: 0,
            texture_pos: (0.0, 0.0),
            texture_scale: (1.0, 1.0),
        }
    }
}
