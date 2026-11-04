#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Crop {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub fn cover(source: (u32, u32), screen: (f32, f32)) -> Crop {
    let (width, height) = source;
    let whole = Crop {
        x: 0,
        y: 0,
        width,
        height,
    };
    let (screen_width, screen_height) = screen;
    if screen_width <= 0.0 || screen_height <= 0.0 || width == 0 || height == 0 {
        return whole;
    }
    let aspect = screen_width / screen_height;
    let fitted_width = (height as f32 * aspect).round() as u32;
    match fitted_width <= width {
        true => Crop {
            x: (width - fitted_width) / 2,
            width: fitted_width,
            ..whole
        },
        false => {
            let fitted_height = ((width as f32 / aspect).round() as u32).min(height);
            Crop {
                y: (height - fitted_height) / 2,
                height: fitted_height,
                ..whole
            }
        }
    }
}

pub fn left(crop: Crop, share: f32) -> Crop {
    Crop {
        width: (crop.width as f32 * share).round() as u32,
        ..crop
    }
}

impl Crop {
    pub fn scaled(self, factor: f32) -> Crop {
        let scale = |value: u32| (value as f32 * factor).round() as u32;
        Crop {
            x: scale(self.x),
            y: scale(self.y),
            width: scale(self.width),
            height: scale(self.height),
        }
    }
}
