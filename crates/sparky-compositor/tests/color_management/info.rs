use wayland_client::{Connection, Dispatch, QueueHandle, WEnum};

use super::events::{Events, info, manager};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Info {
    pub primaries: Option<[i32; 8]>,
    pub primaries_named: Option<WEnum<manager::Primaries>>,
    pub transfer: Option<WEnum<manager::TransferFunction>>,
    pub luminances: Option<(u32, u32, u32)>,
    pub target_primaries: Option<[i32; 8]>,
    pub target_luminance: Option<(u32, u32)>,
    pub done: bool,
}

impl Dispatch<info::WpImageDescriptionInfoV1, ()> for Events {
    fn event(
        state: &mut Self,
        _: &info::WpImageDescriptionInfoV1,
        event: info::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let info = &mut state.info;
        match event {
            info::Event::Primaries {
                r_x,
                r_y,
                g_x,
                g_y,
                b_x,
                b_y,
                w_x,
                w_y,
            } => {
                info.primaries = Some([r_x, r_y, g_x, g_y, b_x, b_y, w_x, w_y]);
            }
            info::Event::PrimariesNamed { primaries } => info.primaries_named = Some(primaries),
            info::Event::TfNamed { tf } => info.transfer = Some(tf),
            info::Event::Luminances {
                min_lum,
                max_lum,
                reference_lum,
            } => {
                info.luminances = Some((min_lum, max_lum, reference_lum));
            }
            info::Event::TargetPrimaries {
                r_x,
                r_y,
                g_x,
                g_y,
                b_x,
                b_y,
                w_x,
                w_y,
            } => {
                info.target_primaries = Some([r_x, r_y, g_x, g_y, b_x, b_y, w_x, w_y]);
            }
            info::Event::TargetLuminance { min_lum, max_lum } => {
                info.target_luminance = Some((min_lum, max_lum));
            }
            info::Event::Done => info.done = true,
            _ => {}
        }
    }
}
