use super::{client::Client, events::Events, fixture::Error};
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols_wlr::screencopy::v1::client::{
    zwlr_screencopy_frame_v1 as frame, zwlr_screencopy_manager_v1 as manager,
};

impl Dispatch<frame::ZwlrScreencopyFrameV1, ()> for Events {
    fn event(
        state: &mut Self,
        _: &frame::ZwlrScreencopyFrameV1,
        event: frame::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            frame::Event::LinuxDmabuf {
                format,
                width,
                height,
            } => state.legacy_dma = Some((format, width, height)),
            frame::Event::Buffer {
                width,
                height,
                stride,
                ..
            } => state.legacy_shm = Some((width, height, stride)),
            frame::Event::BufferDone => state.legacy_done = true,
            _ => {}
        }
    }
}
wayland_client::delegate_noop!(Events: ignore manager::ZwlrScreencopyManagerV1);

#[test]
fn protocolo_antigo_oferece_dma_buf_e_preserva_shm_para_screenshots() -> Result<(), Error> {
    let mut client = Client::connect("")?;
    let qh = client.queue.handle();
    let manager: manager::ZwlrScreencopyManagerV1 = client.bindings.bind(&qh, 3..=3, ())?;
    let _frame = manager.capture_output(1, &client.output, &qh, ());
    client.roundtrip()?;
    assert_eq!(
        client.events.legacy_dma,
        Some((
            smithay::backend::allocator::Fourcc::Argb8888 as u32,
            800,
            600
        ))
    );
    assert_eq!(client.events.legacy_shm, Some((800, 600, 3200)));
    assert!(client.events.legacy_done);
    Ok(())
}
