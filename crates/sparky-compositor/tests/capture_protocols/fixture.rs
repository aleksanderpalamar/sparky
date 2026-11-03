use smithay::backend::allocator::{Format, Fourcc, Modifier};
use smithay::output::Mode;
use sparky_compositor::backend::nested::send_frames;
use std::{fs, path::PathBuf, time::Duration};

pub use super::server::{Error, Server};
use super::{compositor, ready, server};

pub const NAME: &str = "fixture::private_capture_server";

#[test]
fn private_capture_server() -> Result<(), Error> {
    let Some(directory) = std::env::var_os(server::DIRECTORY) else {
        return Ok(());
    };
    let (mut event_loop, mut state) = compositor::start()?;
    let output = compositor::attach_monitor(&mut state, "capture-fixture");
    state.image_capture.formats.set(
        Some(0x1234),
        [
            Format {
                code: Fourcc::Argb8888,
                modifier: Modifier::Linear,
            },
            Format {
                code: Fourcc::Argb8888,
                modifier: Modifier::Invalid,
            },
            Format {
                code: Fourcc::Abgr8888,
                modifier: Modifier::Linear,
            },
        ],
    );
    let scenario = std::env::var(server::SCENARIO);
    if scenario.as_deref() == Ok("locked") {
        state.lock.engage();
    }
    if scenario.as_deref() == Ok("no-formats") {
        state.image_capture.formats.entries.clear();
    }
    if scenario.as_deref() == Ok("ipc") {
        state.start_ipc();
    }
    let directory = PathBuf::from(directory);
    ready::publish(&directory, &state.socket_name)?;
    loop {
        if directory.join("resize").exists() {
            fs::remove_file(directory.join("resize"))?;
            output.change_current_state(
                Some(Mode {
                    size: (640, 480).into(),
                    refresh: 60_000,
                }),
                None,
                None,
                None,
            );
        }
        if directory.join("remove").exists() {
            fs::remove_file(directory.join("remove"))?;
            state.space.unmap_output(&output);
        }
        event_loop.dispatch(Duration::from_millis(5), &mut state)?;
        if scenario.as_deref() == Ok("frames") {
            send_frames(&state, &output);
        }
        super::submission::dispatch(&mut state, &directory)?;
        state.after_dispatch();
    }
}
