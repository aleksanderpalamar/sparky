const REFERENCE: f32 = 1080.0;
const POINTS: f32 = 13.0;
const PIXELS_PER_POINT: f32 = 96.0 / 72.0;
pub const FORM_SHARE: f32 = 0.4;
const FIELD_SHARE: f32 = 0.2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    Time,
    Date,
    Warning,
    Username,
    Password,
    Login,
    PowerIcons,
    PowerLabels,
    Session,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Band {
    pub part: Part,
    pub top: f32,
    pub height: f32,
}

const PLACES: [(Part, f32, f32); 9] = [
    (Part::Time, 0.247, 190.0),
    (Part::Date, 0.362, 46.0),
    (Part::Warning, 0.513, 26.0),
    (Part::Username, 0.552, 39.0),
    (Part::Password, 0.605, 39.0),
    (Part::Login, 0.687, 39.0),
    (Part::PowerIcons, 0.838, 40.0),
    (Part::PowerLabels, 0.872, 20.0),
    (Part::Session, 0.936, 20.0),
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    pub form_width: f32,
    pub field_width: f32,
    pub field_height: f32,
    pub field_icon: f32,
    pub time: f32,
    pub date: f32,
    pub text: f32,
    pub small: f32,
    pub power_icon: f32,
    pub power_width: f32,
    pub power_spacing: f32,
    pub blur: f32,
}

impl Layout {
    pub fn new(width: f32, height: f32) -> Self {
        let scale = height / REFERENCE;
        let point = POINTS * scale;
        Self {
            form_width: width * FORM_SHARE,
            field_width: width * FIELD_SHARE,
            field_height: point * 3.0,
            field_icon: point * 4.5 * 0.25,
            time: point * 9.0 * PIXELS_PER_POINT,
            date: point * 2.0 * PIXELS_PER_POINT,
            text: point * PIXELS_PER_POINT,
            small: point * 0.8 * PIXELS_PER_POINT,
            power_icon: 2.0 * (point * 3.0 / 2.0).round(),
            power_width: 70.0 * scale,
            power_spacing: point,
            blur: 32.0 * scale,
        }
    }
}

pub fn bands(height: f32) -> Vec<Band> {
    let scale = height / REFERENCE;
    PLACES
        .iter()
        .map(|&(part, center, size)| {
            let size = size * scale;
            Band {
                part,
                top: center * height - size / 2.0,
                height: size,
            }
        })
        .collect()
}
