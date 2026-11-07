use std::env;
use std::io::{self, IsTerminal};

use crate::render::view::Terminal;

pub fn detect() -> Terminal {
    Terminal {
        tty: io::stdout().is_terminal(),
        columns: window_columns().or_else(environment_columns),
        no_color: env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty()),
    }
}

fn window_columns() -> Option<usize> {
    let mut size = libc::winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let result = unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut size) };
    (result == 0 && size.ws_col > 0).then_some(usize::from(size.ws_col))
}

fn environment_columns() -> Option<usize> {
    env::var("COLUMNS")
        .ok()?
        .trim()
        .parse::<usize>()
        .ok()
        .filter(|columns| *columns > 0)
}
