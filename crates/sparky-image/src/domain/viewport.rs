use super::turn::{Spin, Turn};

const ZOOM_STEP: f32 = 1.25;
const MAX_SCALE: f32 = 32.0;
const PAN_STEP: f32 = 80.0;
const ROUNDING: f32 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Area {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Zoom {
    Fit,
    Scale(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub turn: Turn,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    width: f32,
    height: f32,
    zoom: Zoom,
    turn: Turn,
    offset_x: f32,
    offset_y: f32,
}

impl Viewport {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width: width as f32,
            height: height as f32,
            zoom: Zoom::Fit,
            turn: Turn::Upright,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }

    pub fn is_fit(&self) -> bool {
        self.zoom == Zoom::Fit
    }

    pub fn percent(&self, area: Area) -> u32 {
        (self.scale(area) * 100.0).round() as u32
    }

    pub fn zoom_in(&mut self, area: Area) {
        self.rescale(self.scale(area) * ZOOM_STEP, area);
    }

    pub fn zoom_out(&mut self, area: Area) {
        let scale = self.scale(area) / ZOOM_STEP;
        if scale <= self.fit_scale(area) + f32::EPSILON {
            return self.fit();
        }
        self.rescale(scale, area);
    }

    pub fn actual_size(&mut self) {
        self.reset(Zoom::Scale(1.0));
    }

    pub fn fit(&mut self) {
        self.reset(Zoom::Fit);
    }

    pub fn rotate(&mut self, spin: Spin) {
        self.turn = self.turn.spin(spin);
        self.reset(self.zoom);
    }

    pub fn look(&mut self, direction: Direction, area: Area) {
        let (x, y) = match direction {
            Direction::Left => (PAN_STEP, 0.0),
            Direction::Right => (-PAN_STEP, 0.0),
            Direction::Up => (0.0, PAN_STEP),
            Direction::Down => (0.0, -PAN_STEP),
        };
        self.drag(x, y, area);
    }

    pub fn drag(&mut self, x: f32, y: f32, area: Area) {
        self.offset_x += x;
        self.offset_y += y;
        self.clamp(area);
    }

    pub fn overflows(&self, area: Area) -> bool {
        let (x, y) = self.limits(area);
        x > 0.0 || y > 0.0
    }

    pub fn placement(&self, area: Area) -> Placement {
        let scale = self.scale(area);
        let (limit_x, limit_y) = self.limits(area);
        let width = self.width * scale;
        let height = self.height * scale;
        let center_x = area.width / 2.0 + self.offset_x.clamp(-limit_x, limit_x);
        let center_y = area.height / 2.0 + self.offset_y.clamp(-limit_y, limit_y);
        Placement {
            x: center_x - width / 2.0,
            y: center_y - height / 2.0,
            width,
            height,
            turn: self.turn,
        }
    }

    fn scale(&self, area: Area) -> f32 {
        match self.zoom {
            Zoom::Fit => self.fit_scale(area),
            Zoom::Scale(scale) => scale,
        }
    }

    fn fit_scale(&self, area: Area) -> f32 {
        let (width, height) = self.footprint();
        (area.width / width).min(area.height / height).min(1.0)
    }

    fn footprint(&self) -> (f32, f32) {
        match self.turn.sideways() {
            true => (self.height, self.width),
            false => (self.width, self.height),
        }
    }

    fn limits(&self, area: Area) -> (f32, f32) {
        let scale = self.scale(area);
        let (width, height) = self.footprint();
        (
            excess(width * scale, area.width),
            excess(height * scale, area.height),
        )
    }

    fn rescale(&mut self, scale: f32, area: Area) {
        let previous = self.scale(area);
        let scale = scale.min(MAX_SCALE);
        let ratio = if previous > 0.0 {
            scale / previous
        } else {
            1.0
        };
        self.zoom = Zoom::Scale(scale);
        self.offset_x *= ratio;
        self.offset_y *= ratio;
        self.clamp(area);
    }

    fn clamp(&mut self, area: Area) {
        let (limit_x, limit_y) = self.limits(area);
        self.offset_x = self.offset_x.clamp(-limit_x, limit_x);
        self.offset_y = self.offset_y.clamp(-limit_y, limit_y);
    }

    fn reset(&mut self, zoom: Zoom) {
        self.zoom = zoom;
        self.offset_x = 0.0;
        self.offset_y = 0.0;
    }
}

fn excess(size: f32, room: f32) -> f32 {
    match (size - room) / 2.0 {
        excess if excess > ROUNDING => excess,
        _ => 0.0,
    }
}
