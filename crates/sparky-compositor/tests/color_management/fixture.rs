use std::{fs, path::Path, path::PathBuf, time::Duration};

use sparky_compositor::color_management::description::Range;
use sparky_compositor::color_management::presets::OutputColor;
use sparky_compositor::state::Sparky;

pub use super::server::{Error, Server};
use super::{compositor, ready, server};

pub const NAME: &str = "fixture::private_color_server";
pub const HDR: OutputColor = OutputColor::Hdr(Range { min: 50, max: 600 });

#[test]
fn private_color_server() -> Result<(), Error> {
    let Some(directory) = std::env::var_os(server::DIRECTORY) else {
        return Ok(());
    };
    let (mut event_loop, mut state) = compositor::start()?;
    let output = compositor::attach_monitor(&mut state, "color-fixture");
    let directory = PathBuf::from(directory);
    ready::publish(&directory, &state.socket_name)?;
    loop {
        if take(&directory, "hdr")? {
            state.apply_output_colors(vec![(output.clone(), HDR)]);
        }
        if take(&directory, "sdr")? {
            state.apply_output_colors(vec![(output.clone(), OutputColor::Sdr)]);
        }
        if take(&directory, "probe")? {
            probe(&state, &directory)?;
        }
        event_loop.dispatch(Duration::from_millis(5), &mut state)?;
        state.after_dispatch();
    }
}

fn probe(state: &Sparky, directory: &Path) -> std::io::Result<()> {
    let decodes: Vec<String> = state
        .color_management
        .surfaces
        .conversions()
        .iter()
        .map(|(_, conversion)| format!("{:?}", conversion.decode))
        .collect();
    let pending = directory.join("conversions.tmp");
    fs::write(&pending, decodes.join("\n"))?;
    fs::rename(pending, directory.join("conversions"))
}

fn take(directory: &Path, signal: &str) -> std::io::Result<bool> {
    let path = directory.join(signal);
    if !path.exists() {
        return Ok(false);
    }
    fs::remove_file(path)?;
    Ok(true)
}
