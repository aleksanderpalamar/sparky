use super::types::{OutputManager, Source, Target, WindowManager};
use crate::state::Sparky;
use smithay::output::Output;
use smithay::reexports::wayland_protocols::ext::image_capture_source::v1::server::{
    ext_foreign_toplevel_image_capture_source_manager_v1 as window,
    ext_output_image_capture_source_manager_v1 as output,
};
use smithay::reexports::wayland_server::{
    Client, DataInit, Dispatch, DisplayHandle, GlobalDispatch, New,
};
use smithay::wayland::foreign_toplevel_list::ForeignToplevelHandle;
use sparky_tiling::WindowId;

impl GlobalDispatch<OutputManager, ()> for Sparky {
    fn bind(
        _: &mut Self,
        _: &DisplayHandle,
        _: &Client,
        resource: New<OutputManager>,
        _: &(),
        init: &mut DataInit<'_, Self>,
    ) {
        init.init(resource, ());
    }
}
impl GlobalDispatch<WindowManager, ()> for Sparky {
    fn bind(
        _: &mut Self,
        _: &DisplayHandle,
        _: &Client,
        resource: New<WindowManager>,
        _: &(),
        init: &mut DataInit<'_, Self>,
    ) {
        init.init(resource, ());
    }
}
impl Dispatch<OutputManager, ()> for Sparky {
    fn request(
        _: &mut Self,
        _: &Client,
        _: &OutputManager,
        request: output::Request,
        _: &(),
        _: &DisplayHandle,
        init: &mut DataInit<'_, Self>,
    ) {
        if let output::Request::CreateSource { source, output } = request {
            let target = Output::from_resource(&output)
                .map(|o| Target::Output(o.downgrade()))
                .unwrap_or(Target::Unavailable);
            init.init(source, target);
        }
    }
}
impl Dispatch<WindowManager, ()> for Sparky {
    fn request(
        _: &mut Self,
        _: &Client,
        _: &WindowManager,
        request: window::Request,
        _: &(),
        _: &DisplayHandle,
        init: &mut DataInit<'_, Self>,
    ) {
        if let window::Request::CreateSource {
            source,
            toplevel_handle,
        } = request
        {
            let target = ForeignToplevelHandle::from_resource(&toplevel_handle)
                .and_then(|handle| handle.user_data().get::<WindowId>().copied())
                .map(Target::Window)
                .unwrap_or(Target::Unavailable);
            init.init(source, target);
        }
    }
}
impl Dispatch<Source, Target> for Sparky {
    fn request(
        _: &mut Self,
        _: &Client,
        _: &Source,
        _: smithay::reexports::wayland_protocols::ext::image_capture_source::v1::server::ext_image_capture_source_v1::Request,
        _: &Target,
        _: &DisplayHandle,
        _: &mut DataInit<'_, Self>,
    ) {
    }
}
