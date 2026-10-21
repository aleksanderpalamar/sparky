use smithay::backend::allocator::format::FormatSet;
use smithay::backend::egl::EGLDevice;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::reexports::wayland_protocols::wp::linux_dmabuf::zv1::server::zwp_linux_dmabuf_v1::ZwpLinuxDmabufV1;
use smithay::reexports::wayland_server::{DisplayHandle, GlobalDispatch};
use smithay::wayland::buffer::BufferHandler;
use smithay::wayland::dmabuf::{DmabufFeedbackBuilder, DmabufGlobal, DmabufGlobalData, DmabufHandler, DmabufState};

pub(super) fn publish<D>(
    display: &DisplayHandle,
    state: &mut DmabufState,
    device: Option<libc::dev_t>,
    formats: FormatSet,
) -> std::io::Result<DmabufGlobal>
where
    D: GlobalDispatch<ZwpLinuxDmabufV1, DmabufGlobalData> + BufferHandler + DmabufHandler + 'static,
{
    let Some(device) = device else {
        return Ok(state.create_global::<D>(display, formats));
    };
    let feedback = DmabufFeedbackBuilder::new(device, formats).build()?;
    Ok(state.create_global_with_default_feedback::<D>(display, &feedback))
}

pub(super) fn render_device(renderer: &GlesRenderer) -> Option<libc::dev_t> {
    let device =
        EGLDevice::device_for_display(renderer.egl_context().display()).and_then(|device| device.try_get_render_node());
    match device {
        Ok(node) => node.map(|node| node.dev_id()),
        Err(error) => {
            tracing::warn!("sem dispositivo EGL para o feedback de DMA-BUF: {error}");
            None
        }
    }
}

#[cfg(test)]
#[path = "dmabuf_test_server.rs"]
mod test_server;

#[cfg(test)]
#[path = "dmabuf_test_client.rs"]
mod test_client;

#[cfg(test)]
#[path = "dmabuf_tests.rs"]
mod tests;
