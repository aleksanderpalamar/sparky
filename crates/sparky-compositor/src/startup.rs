use smithay::reexports::calloop::EventLoop;
use smithay::reexports::wayland_server::Display;
use tracing_subscriber::EnvFilter;

use crate::backend::{drm, nested};
use crate::cli::Options;
use crate::spawn;
use crate::state::Sparky;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let options = Options::parse(std::env::args().skip(1))?;
    init_logging();
    let mut event_loop: EventLoop<'static, Sparky> = EventLoop::try_new()?;
    let display: Display<Sparky> = Display::new()?;
    let mut state = Sparky::new(&mut event_loop, display)?;
    match inside_another_session() {
        true => nested::init(&mut event_loop, &mut state)?,
        false => drm::init(&mut state)?,
    }
    tracing::info!("sparky ouvindo em {:?}", state.socket_name);
    for command in &options.spawn {
        spawn::spawn(command, &state.socket_name);
    }
    event_loop.run(None, &mut state, Sparky::after_dispatch)?;
    Ok(())
}

fn inside_another_session() -> bool {
    std::env::var_os("WAYLAND_DISPLAY").is_some() || std::env::var_os("DISPLAY").is_some()
}

fn init_logging() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
