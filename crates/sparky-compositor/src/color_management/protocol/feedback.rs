use std::sync::atomic::{AtomicU64, Ordering};

use smithay::reexports::wayland_protocols::wp::color_management::v1::server::wp_color_management_surface_feedback_v1::{
    Error, Request, WpColorManagementSurfaceFeedbackV1,
};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::{Client, DataInit, Dispatch, DisplayHandle, New, Resource, Weak};

use super::image::{self, Information};
use super::names;
use crate::state::Sparky;

const CHANGED2_VERSION: u32 = 2;

pub struct FeedbackData {
    pub surface: Weak<WlSurface>,
    preferred: AtomicU64,
}

impl FeedbackData {
    pub fn replace(&self, identity: u64) -> u64 {
        self.preferred.swap(identity, Ordering::Relaxed)
    }
}

pub fn create(
    state: &mut Sparky,
    init: &mut DataInit<'_, Sparky>,
    id: New<WpColorManagementSurfaceFeedbackV1>,
    surface: &WlSurface,
) {
    let identity = state.preferred_record(surface).identity;
    let data = FeedbackData {
        surface: surface.downgrade(),
        preferred: AtomicU64::new(identity),
    };
    let object = init.init(id, data);
    state.color_management.feedbacks.push(object.downgrade());
}

pub fn notify(feedback: &WpColorManagementSurfaceFeedbackV1, identity: u64) {
    let (high, low) = names::identity_parts(identity);
    match feedback.version() >= CHANGED2_VERSION {
        true => feedback.preferred_changed2(high, low),
        false => feedback.preferred_changed(low),
    }
}

impl Dispatch<WpColorManagementSurfaceFeedbackV1, FeedbackData> for Sparky {
    fn request(
        state: &mut Self,
        _: &Client,
        feedback: &WpColorManagementSurfaceFeedbackV1,
        request: Request,
        data: &FeedbackData,
        _: &DisplayHandle,
        init: &mut DataInit<'_, Self>,
    ) {
        let image_description = match request {
            Request::GetPreferred { image_description }
            | Request::GetPreferredParametric { image_description } => image_description,
            _ => return,
        };
        let Ok(surface) = data.surface.upgrade() else {
            feedback.post_error(Error::Inert, "a superfície não existe mais");
            return;
        };
        let record = state.preferred_record(&surface);
        image::ready(init, image_description, record, Information::Allowed);
    }
}
