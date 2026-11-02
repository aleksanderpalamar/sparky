use smithay::reexports::wayland_protocols::wp::color_management::v1::server::wp_image_description_info_v1::WpImageDescriptionInfoV1;

use super::names;
use crate::color_management::description::{Chromaticities, ImageDescription};

pub fn send(info: &WpImageDescriptionInfoV1, description: &ImageDescription) {
    let [rx, ry, gx, gy, bx, by, wx, wy] = coordinates(description.primaries.chromaticities());
    info.primaries(rx, ry, gx, gy, bx, by, wx, wy);
    info.primaries_named(names::primaries(description.primaries));
    info.tf_named(names::transfer(description.transfer));
    let luminances = description.luminances;
    info.luminances(luminances.min, luminances.max, luminances.reference);
    let [rx, ry, gx, gy, bx, by, wx, wy] = coordinates(description.target.primaries);
    info.target_primaries(rx, ry, gx, gy, bx, by, wx, wy);
    let target = description.target.luminance;
    info.target_luminance(target.min, target.max);
    if let Some(max_cll) = description.light.max_cll {
        info.target_max_cll(max_cll);
    }
    if let Some(max_fall) = description.light.max_fall {
        info.target_max_fall(max_fall);
    }
    info.done();
}

const fn coordinates(gamut: Chromaticities) -> [i32; 8] {
    [
        gamut.red.x,
        gamut.red.y,
        gamut.green.x,
        gamut.green.y,
        gamut.blue.x,
        gamut.blue.y,
        gamut.white.x,
        gamut.white.y,
    ]
}
