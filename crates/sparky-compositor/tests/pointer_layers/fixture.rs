use std::{env, fs, path::PathBuf, time::Duration};

use super::{commands, compositor, ready, server};

pub use super::server::{Error, Server};

pub const NAME: &str = "fixture::pointer_layers_server";
pub const COMMAND: &str = "command";

#[test]
fn pointer_layers_server() -> Result<(), Error> {
    let Some(directory) = env::var_os(server::DIRECTORY) else {
        return Ok(());
    };
    let (mut event_loop, mut state) = compositor::start()?;
    compositor::attach_monitor(&mut state, "A", 0);
    if env::var(server::SCENARIO).as_deref() == Ok("two") {
        compositor::attach_monitor(&mut state, "B", 800);
    }
    let directory = PathBuf::from(directory);
    ready::publish(&directory, &state.socket_name)?;
    loop {
        if let Ok(text) = fs::read_to_string(directory.join(COMMAND)) {
            commands::run(&mut state, text.trim());
            fs::remove_file(directory.join(COMMAND))?;
        }
        event_loop.dispatch(Duration::from_millis(5), &mut state)?;
        state.after_dispatch();
    }
}
