use smithay::reexports::calloop::EventLoop;
use smithay::reexports::wayland_server::Display;
use std::sync::Arc;

use crate::backend::{drm, nested};
use crate::cli::Options;
use crate::diagnostics::{metrics::Metrics, probe};
use crate::logging;
use crate::signals;
use crate::state::Sparky;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let options = Options::parse(std::env::args().skip(1))?;
    let diagnostics = Arc::new(Metrics::default());
    let _logs = logging::init(diagnostics.clone());
    let result = run_session(options, diagnostics);
    if let Err(error) = &result {
        tracing::error!("sessão encerrada com erro: {error}");
    }
    tracing::info!("fim da sessão");
    result
}

fn run_session(
    options: Options,
    diagnostics: Arc<Metrics>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut event_loop: EventLoop<'static, Sparky> = EventLoop::try_new()?;
    let display: Display<Sparky> = Display::new()?;
    let mut state = Sparky::with_diagnostics(&mut event_loop, display, diagnostics)?;
    match inside_another_session() {
        true => nested::init(&mut event_loop, &mut state)?,
        false => drm::init(&mut state)?,
    }
    tracing::info!("sparky ouvindo em {:?}", state.socket_name);
    state.start_ipc();
    state.start_xwayland();
    signals::stop_on_signals(&state.loop_handle)?;
    state.watch_config();
    probe::start(&state);
    let commands = options.spawn;
    event_loop.handle().insert_idle(move |state| {
        if state.is_real_session() {
            state.start_session();
        }
        for command in &commands {
            state.spawn(command);
        }
    });
    event_loop.run(None, &mut state, Sparky::after_dispatch)?;
    state.stop_xwayland();
    Ok(())
}

fn inside_another_session() -> bool {
    std::env::var_os("WAYLAND_DISPLAY").is_some() || std::env::var_os("DISPLAY").is_some()
}
