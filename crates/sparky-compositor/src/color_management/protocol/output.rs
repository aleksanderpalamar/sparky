use smithay::output::{Output, WeakOutput};
use smithay::reexports::wayland_protocols::wp::color_management::v1::server::wp_color_management_output_v1::{
    Request, WpColorManagementOutputV1,
};
use smithay::reexports::wayland_protocols::wp::color_management::v1::server::wp_image_description_v1::Cause;
use smithay::reexports::wayland_server::protocol::wl_output::WlOutput;
use smithay::reexports::wayland_server::{Client, DataInit, Dispatch, DisplayHandle, New, Resource, Weak};

use super::image::{self, Information};
use crate::state::Sparky;

const OUTPUT_DONE_VERSION: u32 = 2;

pub fn create(
    state: &mut Sparky,
    init: &mut DataInit<'_, Sparky>,
    id: New<WpColorManagementOutputV1>,
    output: &WlOutput,
) {
    let target = Output::from_resource(output).map(|output| output.downgrade());
    let object = init.init(id, target);
    state
        .color_management
        .output_objects
        .push(object.downgrade());
}

pub fn announce(objects: &mut Vec<Weak<WpColorManagementOutputV1>>, output: &Output) {
    objects.retain(|object| object.upgrade().is_ok());
    for object in objects.iter().filter_map(|object| object.upgrade().ok()) {
        let watches = object
            .data::<Option<WeakOutput>>()
            .and_then(Option::as_ref)
            .is_some_and(|watched| watched == output);
        if !watches {
            continue;
        }
        object.image_description_changed();
        if let Some(client) = object.client() {
            finish(output, &client);
        }
    }
}

fn finish(output: &Output, client: &Client) {
    for wl_output in output.client_outputs(client) {
        if wl_output.version() >= OUTPUT_DONE_VERSION {
            wl_output.done();
        }
    }
}

impl Dispatch<WpColorManagementOutputV1, Option<WeakOutput>> for Sparky {
    fn request(
        state: &mut Self,
        _: &Client,
        _: &WpColorManagementOutputV1,
        request: Request,
        target: &Option<WeakOutput>,
        _: &DisplayHandle,
        init: &mut DataInit<'_, Self>,
    ) {
        let Request::GetImageDescription { image_description } = request else {
            return;
        };
        match target.as_ref().and_then(WeakOutput::upgrade) {
            Some(output) => {
                let record = state.color_management.record_for(&output);
                image::ready(init, image_description, record, Information::Allowed);
            }
            None => image::failed(
                init,
                image_description,
                Cause::NoOutput,
                "o monitor não existe mais",
            ),
        }
    }
}
