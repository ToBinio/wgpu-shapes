pub struct Rect {
    pub width: f32,
    pub height: f32,
    //todo rename to location
    pub pos: (f32, f32),
    pub rotation: f32,
    pub color: (f32, f32, f32, f32),
}

impl Rect {
    pub fn width(&mut self, width: f32) -> &mut Self {
        self.width = width;
        self
    }

    pub fn height(&mut self, height: f32) -> &mut Self {
        self.height = height;
        self
    }

    pub fn color(&mut self, color: (f32, f32, f32, f32)) -> &mut Self {
        self.color = color;
        self
    }

    pub fn rotation(&mut self, rotation: f32) -> &mut Self {
        self.rotation = rotation;
        self
    }

    pub fn pos(&mut self, pos: (f32, f32)) -> &mut Self {
        self.pos = pos;
        self
    }
}

impl Default for Rect {
    fn default() -> Self {
        Rect {
            width: 20.0,
            height: 20.0,
            pos: (0.0, 0.0),
            rotation: 0.0,
            color: (1.0, 1.0, 1.0, 1.0),
        }
    }
}