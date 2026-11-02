use smithay::output::{Mode, Output, PhysicalProperties, Subpixel};
use smithay::reexports::{calloop::EventLoop, wayland_server::Display};
use smithay::utils::Transform;
use sparky_compositor::state::Sparky;
use std::{thread, time::Duration};

use super::server::Error;

pub fn start() -> Result<(EventLoop<'static, Sparky>, Sparky), Error> {
    thread::spawn(|| {
        thread::sleep(Duration::from_secs(30));
        std::process::exit(2);
    });
    let mut event_loop = EventLoop::try_new()?;
    let display = Display::new()?;
    let state = Sparky::new(&mut event_loop, display)?;
    Ok((event_loop, state))
}

pub fn attach_monitor(state: &mut Sparky, name: &str) -> Output {
    let output = Output::new(
        name.into(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "fixture".into(),
            model: "fixture".into(),
        },
    );
    output.create_global::<Sparky>(&state.display_handle);
    let mode = Mode {
        size: (800, 600).into(),
        refresh: 60_000,
    };
    output.change_current_state(
        Some(mode),
        Some(Transform::Normal),
        None,
        Some((0, 0).into()),
    );
    state.space.map_output(&output, (0, 0));
    state
        .desktop
        .add_monitor(name, sparky_tiling::Rect::new(0, 0, 800, 600));
    output
}
