use wayland_client::globals::GlobalListContents;
use wayland_client::protocol::{
    wl_buffer, wl_compositor, wl_keyboard, wl_output, wl_pointer, wl_region, wl_registry, wl_seat,
    wl_shm, wl_shm_pool, wl_surface,
};
use wayland_client::{Connection, Dispatch, QueueHandle, WEnum, delegate_noop};
pub use wayland_protocols::wp::pointer_constraints::zv1::client::{
    zwp_confined_pointer_v1 as confined, zwp_pointer_constraints_v1 as constraints,
};
pub use wayland_protocols::xdg::shell::client::{
    xdg_popup, xdg_positioner, xdg_surface, xdg_toplevel, xdg_wm_base,
};
pub use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1 as layer_shell, zwlr_layer_surface_v1 as layer_surface,
};

#[derive(Default)]
pub struct Events {
    pub focus: Option<wl_surface::WlSurface>,
    pub presses: Vec<(Option<wl_surface::WlSurface>, wl_pointer::ButtonState)>,
    pub last_y: Option<f64>,
    pub configured: usize,
    pub layer_sizes: Vec<(u32, u32)>,
    pub confined: bool,
    pub keyboard: Option<wl_surface::WlSurface>,
    pub serial: Option<u32>,
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

impl Dispatch<wl_pointer::WlPointer, ()> for Events {
    fn event(
        state: &mut Self,
        _: &wl_pointer::WlPointer,
        event: wl_pointer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_pointer::Event::Enter {
                surface, surface_y, ..
            } => {
                state.focus = Some(surface);
                state.last_y = Some(surface_y);
            }
            wl_pointer::Event::Leave { .. } => state.focus = None,
            wl_pointer::Event::Motion { surface_y, .. } => state.last_y = Some(surface_y),
            wl_pointer::Event::Button {
                serial,
                state: WEnum::Value(button),
                ..
            } => {
                state.serial = Some(serial);
                state.presses.push((state.focus.clone(), button));
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for Events {
    fn event(
        state: &mut Self,
        _: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_keyboard::Event::Enter { surface, .. } => state.keyboard = Some(surface),
            wl_keyboard::Event::Leave { .. } => state.keyboard = None,
            _ => {}
        }
    }
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for Events {
    fn event(
        _: &mut Self,
        base: &xdg_wm_base::XdgWmBase,
        event: xdg_wm_base::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            base.pong(serial);
        }
    }
}

impl Dispatch<xdg_surface::XdgSurface, ()> for Events {
    fn event(
        state: &mut Self,
        surface: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial } = event {
            surface.ack_configure(serial);
            state.configured += 1;
        }
    }
}

impl Dispatch<layer_surface::ZwlrLayerSurfaceV1, ()> for Events {
    fn event(
        state: &mut Self,
        surface: &layer_surface::ZwlrLayerSurfaceV1,
        event: layer_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let layer_surface::Event::Configure {
            serial,
            width,
            height,
        } = event
        {
            surface.ack_configure(serial);
            state.layer_sizes.push((width, height));
        }
    }
}

impl Dispatch<confined::ZwpConfinedPointerV1, ()> for Events {
    fn event(
        state: &mut Self,
        _: &confined::ZwpConfinedPointerV1,
        event: confined::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        state.confined = matches!(event, confined::Event::Confined);
    }
}

delegate_noop!(Events: wl_compositor::WlCompositor);
delegate_noop!(Events: wl_shm_pool::WlShmPool);
delegate_noop!(Events: wl_region::WlRegion);
delegate_noop!(Events: layer_shell::ZwlrLayerShellV1);
delegate_noop!(Events: constraints::ZwpPointerConstraintsV1);
delegate_noop!(Events: ignore wl_shm::WlShm);
delegate_noop!(Events: ignore wl_buffer::WlBuffer);
delegate_noop!(Events: ignore wl_surface::WlSurface);
delegate_noop!(Events: ignore wl_output::WlOutput);
delegate_noop!(Events: ignore wl_seat::WlSeat);
delegate_noop!(Events: ignore xdg_toplevel::XdgToplevel);
delegate_noop!(Events: ignore xdg_popup::XdgPopup);
delegate_noop!(Events: xdg_positioner::XdgPositioner);
