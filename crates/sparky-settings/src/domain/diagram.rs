use sparky_ipc::OutputInfo;

const MARGIN: f32 = 12.0;
const GAP: f32 = 2.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Placement {
    pub number: usize,
    pub focused: bool,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

struct Extent {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
}

pub fn map(outputs: &[OutputInfo], width: f32, height: f32) -> Vec<Placement> {
    let Some(extent) = extent(outputs) else {
        return Vec::new();
    };
    let usable = (width - MARGIN * 2.0).max(1.0);
    let room = (height - MARGIN * 2.0).max(1.0);
    let factor = (usable / extent.width).min(room / extent.height);
    let left = MARGIN + (usable - extent.width * factor) / 2.0;
    let top = MARGIN + (room - extent.height * factor) / 2.0;
    outputs
        .iter()
        .enumerate()
        .map(|(index, output)| Placement {
            number: index + 1,
            focused: output.focused,
            x: left + (output.x as f32 - extent.left) * factor,
            y: top + (output.y as f32 - extent.top) * factor,
            width: (output.width as f32 * factor - GAP).max(1.0),
            height: (output.height as f32 * factor - GAP).max(1.0),
        })
        .collect()
}

fn extent(outputs: &[OutputInfo]) -> Option<Extent> {
    let left = outputs.iter().map(|output| output.x).min()? as f32;
    let top = outputs.iter().map(|output| output.y).min()? as f32;
    let right = outputs.iter().map(|output| output.x + output.width).max()? as f32;
    let bottom = outputs
        .iter()
        .map(|output| output.y + output.height)
        .max()? as f32;
    Some(Extent {
        left,
        top,
        width: (right - left).max(1.0),
        height: (bottom - top).max(1.0),
    })
}
