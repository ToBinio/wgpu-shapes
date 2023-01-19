use crate::instance::TextureInstance;

pub struct Imgage {
    pub scale: (f32, f32),
    pub pos: (f32, f32),
    pub rotation: f32,
    pub layer: u16,
}

impl Imgage {
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
        }
    }
}

impl Default for Imgage {
    fn default() -> Self {
        Imgage {
            scale: (20.0, 20.0),
            pos: (0.0, 0.0),
            rotation: 0.0,
            layer: 0,
        }
    }
}