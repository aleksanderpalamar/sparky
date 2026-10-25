use super::fixture::Error;
use smithay::backend::allocator::{
    Fourcc, Modifier,
    dmabuf::{Dmabuf, DmabufFlags},
};
use smithay::backend::renderer::sync::SyncPoint;
use smithay::reexports::wayland_server::Resource;
use sparky_compositor::image_capture::{
    queue::{Job, Notify},
    rules::{FRAME_INTERVAL, Gate},
    types::{SessionData, Target},
};
use sparky_compositor::state::Sparky;
use std::{path::Path, sync::Arc};

pub fn dispatch(state: &mut Sparky, directory: &Path) -> Result<(), Error> {
    transition(state, directory, "lock", true)?;
    transition(state, directory, "unlock", false)?;
    let signal = directory.join("submit-hidden");
    if !signal.exists() {
        return Ok(());
    }
    std::fs::remove_file(signal)?;
    let session = state
        .image_capture
        .sessions
        .iter()
        .filter_map(|weak| weak.upgrade().ok())
        .find(|session| {
            session
                .data::<Arc<SessionData>>()
                .is_some_and(|data| matches!(data.target, Target::Window(_)))
        })
        .ok_or("sessão de janela ausente")?;
    let data = session
        .data::<Arc<SessionData>>()
        .ok_or("dados ausentes")?
        .clone();
    let Target::Window(id) = data.target else {
        return Err("fonte incorreta".into());
    };
    let window = state.windows.get(id).ok_or("janela ausente")?;
    state.space.unmap_elem(window);
    assert!(state.space.outputs_for_element(window).is_empty());
    let (frame, size) = {
        let mut status = data.status.lock().map_err(|_| "estado inacessível")?;
        let now = state.start_time.elapsed();
        status.pace.submitted(
            now.saturating_sub(FRAME_INTERVAL),
            state.image_capture.generation,
        );
        state.image_capture.generation = state.image_capture.generation.wrapping_add(1);
        assert_eq!(
            status
                .pace
                .gate(now + FRAME_INTERVAL, state.image_capture.generation),
            Gate::Ready
        );
        (
            status.frame.as_ref().ok_or("quadro ausente")?.upgrade()?,
            status.size,
        )
    };
    let mut buffer = Dmabuf::builder(
        size,
        Fourcc::Argb8888,
        Modifier::Linear,
        DmabufFlags::empty(),
    );
    assert!(buffer.add_plane(tempfile::tempfile()?.into(), 0, 0, size.0 as u32 * 4));
    state.finish_capture(
        Job {
            notify: Notify::Image(frame),
            buffer: buffer.build().ok_or("buffer ausente")?,
            target: data.target.clone(),
            size,
            cursor: false,
            session: Some((session, data)),
            region: None,
            locked: false,
        },
        SyncPoint::signaled(),
    );
    Ok(())
}

fn transition(state: &mut Sparky, directory: &Path, name: &str, locked: bool) -> Result<(), Error> {
    let signal = directory.join(name);
    if !signal.exists() {
        return Ok(());
    }
    std::fs::remove_file(signal)?;
    let session = state
        .image_capture
        .sessions
        .first()
        .ok_or("sessão ausente")?
        .upgrade()?;
    let data = session
        .data::<Arc<SessionData>>()
        .ok_or("dados ausentes")?
        .clone();
    let (frame, size) = {
        let mut status = data.status.lock().map_err(|_| "estado inacessível")?;
        status
            .pace
            .submitted(state.start_time.elapsed(), state.image_capture.generation);
        (
            status.frame.as_ref().ok_or("quadro ausente")?.upgrade()?,
            status.size,
        )
    };
    let mut buffer = Dmabuf::builder(
        size,
        Fourcc::Argb8888,
        Modifier::Linear,
        DmabufFlags::empty(),
    );
    assert!(buffer.add_plane(tempfile::tempfile()?.into(), 0, 0, size.0 as u32 * 4));
    match locked {
        true => state.lock.engage(),
        false => state.lock.release(),
    }
    state.finish_capture(
        Job {
            notify: Notify::Image(frame),
            buffer: buffer.build().ok_or("buffer ausente")?,
            target: data.target.clone(),
            size,
            cursor: false,
            session: Some((session, data.clone())),
            region: None,
            locked: !locked,
        },
        SyncPoint::signaled(),
    );
    assert!(state.image_capture.wake.is_some());
    assert_eq!(state.image_capture.pending.len(), 1);
    assert_eq!(
        data.status
            .lock()
            .map_err(|_| "estado inacessível")?
            .pace
            .gate(state.start_time.elapsed(), state.image_capture.generation),
        Gate::Ready
    );
    let mut retry = state.image_capture.pending.pop().ok_or("quadro perdido")?;
    retry.locked = locked;
    state.finish_capture(retry, SyncPoint::signaled());
    assert!(state.image_capture.pending.is_empty());
    Ok(())
}
