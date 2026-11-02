use std::sync::Mutex;

use smithay::reexports::wayland_protocols::wp::color_management::v1::server::wp_color_manager_v1::{
    Error, Feature, RenderIntent, Request, WpColorManagerV1,
};
use smithay::reexports::wayland_protocols::wp::color_management::v1::server::wp_image_description_v1::Cause;
use smithay::reexports::wayland_server::{
    Client, DataInit, Dispatch, DisplayHandle, GlobalDispatch, New, Resource,
};

use super::image::{self, Information};
use super::{feedback, names, output, surface};
use crate::color_management::description::{NamedPrimaries, TransferFunction};
use crate::color_management::params::Params;
use crate::color_management::presets;
use crate::state::Sparky;

pub const VERSION: u32 = 2;
const FEATURES: [Feature; 4] = [
    Feature::Parametric,
    Feature::SetLuminances,
    Feature::SetMasteringDisplayPrimaries,
    Feature::WindowsScrgb,
];

pub fn register(display: &DisplayHandle) {
    display.create_global::<Sparky, WpColorManagerV1, _>(VERSION, ());
}

fn announce(manager: &WpColorManagerV1) {
    manager.supported_intent(RenderIntent::Perceptual);
    for feature in FEATURES {
        manager.supported_feature(feature);
    }
    for transfer in TransferFunction::ALL {
        manager.supported_tf_named(names::transfer(transfer));
    }
    for primaries in NamedPrimaries::ALL {
        manager.supported_primaries_named(names::primaries(primaries));
    }
    manager.done();
}

impl GlobalDispatch<WpColorManagerV1, ()> for Sparky {
    fn bind(
        _: &mut Self,
        _: &DisplayHandle,
        _: &Client,
        resource: New<WpColorManagerV1>,
        _: &(),
        init: &mut DataInit<'_, Self>,
    ) {
        announce(&init.init(resource, ()));
    }
}

impl Dispatch<WpColorManagerV1, ()> for Sparky {
    fn request(
        state: &mut Self,
        _: &Client,
        manager: &WpColorManagerV1,
        request: Request,
        _: &(),
        _: &DisplayHandle,
        init: &mut DataInit<'_, Self>,
    ) {
        match request {
            Request::GetOutput { id, output } => output::create(state, init, id, &output),
            Request::GetSurface { id, surface } => surface::create(manager, init, id, &surface),
            Request::GetSurfaceFeedback { id, surface } => {
                feedback::create(state, init, id, &surface);
            }
            Request::CreateParametricCreator { obj } => {
                init.init(obj, Mutex::new(Params::default()));
            }
            Request::CreateWindowsScrgb { image_description } => {
                let record = state
                    .color_management
                    .records
                    .record(presets::windows_scrgb());
                image::ready(init, image_description, record, Information::Denied);
            }
            Request::GetImageDescription {
                image_description, ..
            } => image::failed(
                init,
                image_description,
                Cause::Unsupported,
                "referência desconhecida",
            ),
            Request::CreateIccCreator { .. } => {
                manager.post_error(Error::UnsupportedFeature, "perfis ICC não são aceitos");
            }
            _ => {}
        }
    }
}
