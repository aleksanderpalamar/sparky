use std::sync::{Arc, Mutex};

use smithay::reexports::wayland_protocols::wp::color_management::v1::server::wp_color_management_surface_v1::{
    Error, Request, WpColorManagementSurfaceV1,
};
use smithay::reexports::wayland_protocols::wp::color_management::v1::server::wp_color_manager_v1::{
    self, RenderIntent, WpColorManagerV1,
};
use smithay::reexports::wayland_protocols::wp::color_management::v1::server::wp_image_description_v1::WpImageDescriptionV1;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::{Client, DataInit, Dispatch, DisplayHandle, New, Resource, WEnum, Weak};
use smithay::wayland::compositor::with_states;

use super::image;
use crate::color_management::records::Record;
use crate::color_management::surface_color::SurfaceColor;
use crate::state::Sparky;

#[derive(Default)]
struct Owner(Mutex<Option<Weak<WpColorManagementSurfaceV1>>>);

pub fn create(
    state: &mut Sparky,
    manager: &WpColorManagerV1,
    init: &mut DataInit<'_, Sparky>,
    id: New<WpColorManagementSurfaceV1>,
    surface: &WlSurface,
) {
    if owned(surface) {
        manager.post_error(
            wp_color_manager_v1::Error::SurfaceExists,
            "a superfície já tem gerenciamento de cor",
        );
        return;
    }
    let object = init.init(id, surface.downgrade());
    state.color_management.surfaces.add(surface);
    with_states(surface, |states| {
        if let Some(owner) = states.data_map.get::<Owner>()
            && let Ok(mut slot) = owner.0.lock()
        {
            *slot = Some(object.downgrade());
        }
    });
}

fn owned(surface: &WlSurface) -> bool {
    with_states(surface, |states| {
        states.data_map.insert_if_missing_threadsafe(Owner::default);
        states
            .data_map
            .get::<Owner>()
            .and_then(|owner| owner.0.lock().ok().and_then(|slot| slot.clone()))
            .is_some_and(|object| object.upgrade().is_ok())
    })
}

fn assign(surface: &WlSurface, description: Option<Arc<Record>>) {
    with_states(surface, |states| {
        states
            .cached_state
            .get::<SurfaceColor>()
            .pending()
            .description = description;
    });
}

fn set(
    object: &WpColorManagementSurfaceV1,
    surface: &WlSurface,
    image: &WpImageDescriptionV1,
    intent: WEnum<RenderIntent>,
) {
    if intent != WEnum::Value(RenderIntent::Perceptual) {
        object.post_error(Error::RenderIntent, "intenção de cor não anunciada");
        return;
    }
    let Some(record) = image::record_of(image) else {
        object.post_error(
            Error::ImageDescription,
            "a descrição de cor não está pronta",
        );
        return;
    };
    assign(surface, Some(record));
}

impl Dispatch<WpColorManagementSurfaceV1, Weak<WlSurface>> for Sparky {
    fn request(
        _: &mut Self,
        _: &Client,
        object: &WpColorManagementSurfaceV1,
        request: Request,
        surface: &Weak<WlSurface>,
        _: &DisplayHandle,
        _: &mut DataInit<'_, Self>,
    ) {
        let Ok(surface) = surface.upgrade() else {
            if !matches!(request, Request::Destroy) {
                object.post_error(Error::Inert, "a superfície não existe mais");
            }
            return;
        };
        match request {
            Request::SetImageDescription {
                image_description,
                render_intent,
            } => set(object, &surface, &image_description, render_intent),
            Request::UnsetImageDescription | Request::Destroy => assign(&surface, None),
            _ => {}
        }
    }
}
