use std::collections::HashMap;

use wayland_client::globals::GlobalListContents;
use wayland_client::protocol::{wl_compositor, wl_output, wl_registry, wl_surface};
use wayland_client::{Connection, Dispatch, Proxy, QueueHandle, WEnum, backend::ObjectId};
pub use wayland_protocols::wp::color_management::v1::client::{
    wp_color_management_output_v1 as output, wp_color_management_surface_feedback_v1 as feedback,
    wp_color_management_surface_v1 as surface, wp_color_manager_v1 as manager,
    wp_image_description_creator_icc_v1 as icc, wp_image_description_creator_params_v1 as params,
    wp_image_description_info_v1 as info, wp_image_description_v1 as image,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Image {
    Ready(u64),
    Failed(WEnum<image::Cause>),
}

#[derive(Default)]
pub struct Events {
    pub intents: Vec<WEnum<manager::RenderIntent>>,
    pub features: Vec<WEnum<manager::Feature>>,
    pub transfers: Vec<WEnum<manager::TransferFunction>>,
    pub primaries: Vec<WEnum<manager::Primaries>>,
    pub announced: usize,
    pub images: HashMap<ObjectId, Image>,
    pub info: super::info::Info,
    pub output_changed: usize,
    pub output_done: usize,
    pub preferred: Vec<u64>,
}

impl Events {
    pub fn image(&self, image: &image::WpImageDescriptionV1) -> Option<Image> {
        self.images.get(&image.id()).cloned()
    }
}

impl Dispatch<manager::WpColorManagerV1, ()> for Events {
    fn event(
        state: &mut Self,
        _: &manager::WpColorManagerV1,
        event: manager::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            manager::Event::SupportedIntent { render_intent } => state.intents.push(render_intent),
            manager::Event::SupportedFeature { feature } => state.features.push(feature),
            manager::Event::SupportedTfNamed { tf } => state.transfers.push(tf),
            manager::Event::SupportedPrimariesNamed { primaries } => {
                state.primaries.push(primaries)
            }
            manager::Event::Done => state.announced += 1,
            _ => {}
        }
    }
}

impl Dispatch<image::WpImageDescriptionV1, ()> for Events {
    fn event(
        state: &mut Self,
        object: &image::WpImageDescriptionV1,
        event: image::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let image = match event {
            image::Event::Ready { identity } => Image::Ready(u64::from(identity)),
            image::Event::Ready2 {
                identity_hi,
                identity_lo,
            } => Image::Ready((u64::from(identity_hi) << 32) | u64::from(identity_lo)),
            image::Event::Failed { cause, .. } => Image::Failed(cause),
            _ => return,
        };
        state.images.insert(object.id(), image);
    }
}

impl Dispatch<output::WpColorManagementOutputV1, ()> for Events {
    fn event(
        state: &mut Self,
        _: &output::WpColorManagementOutputV1,
        event: output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let output::Event::ImageDescriptionChanged = event {
            state.output_changed += 1;
        }
    }
}

impl Dispatch<feedback::WpColorManagementSurfaceFeedbackV1, ()> for Events {
    fn event(
        state: &mut Self,
        _: &feedback::WpColorManagementSurfaceFeedbackV1,
        event: feedback::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            feedback::Event::PreferredChanged { identity } => {
                state.preferred.push(u64::from(identity))
            }
            feedback::Event::PreferredChanged2 {
                identity_hi,
                identity_lo,
            } => state
                .preferred
                .push((u64::from(identity_hi) << 32) | u64::from(identity_lo)),
            _ => {}
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for Events {
    fn event(
        state: &mut Self,
        _: &wl_output::WlOutput,
        event: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_output::Event::Done = event {
            state.output_done += 1;
        }
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for Events {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

wayland_client::delegate_noop!(Events: ignore wl_compositor::WlCompositor);
wayland_client::delegate_noop!(Events: ignore wl_surface::WlSurface);
wayland_client::delegate_noop!(Events: ignore surface::WpColorManagementSurfaceV1);
wayland_client::delegate_noop!(Events: ignore params::WpImageDescriptionCreatorParamsV1);
wayland_client::delegate_noop!(Events: ignore icc::WpImageDescriptionCreatorIccV1);
