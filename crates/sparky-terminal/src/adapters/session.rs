use std::env;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use alacritty_terminal::event::Notify;
use alacritty_terminal::event::{OnResize, WindowSize};
use alacritty_terminal::event_loop::{EventLoop, Notifier};
use alacritty_terminal::grid::{Dimensions, Scroll};
use alacritty_terminal::index::{Column, Line, Point, Side};
use alacritty_terminal::selection::{Selection, SelectionType};
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::{Config, Osc52, Term, TermMode};
use alacritty_terminal::tty::{self, Options, Shell};
use sparky_theme::TerminalPalette;
use sparky_xdg::programs::installed;

use crate::adapters::listener::Listener;
use crate::adapters::snapshot::snapshot;
use crate::domain::grid::{Cell, Half, Spot};
use crate::domain::launch::{Availability, Launch, plan};
use crate::domain::screen::Screen;
use crate::domain::zsh::{ZSH, chosen, dirs};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub columns: usize,
    pub lines: usize,
    pub cell: Cell,
    pub scale: f32,
}

impl Dimensions for Size {
    fn total_lines(&self) -> usize {
        self.lines
    }

    fn screen_lines(&self) -> usize {
        self.lines
    }

    fn columns(&self) -> usize {
        self.columns
    }
}

impl Size {
    pub fn window(&self) -> WindowSize {
        let narrow = |value: usize| u16::try_from(value).unwrap_or(u16::MAX);
        WindowSize {
            num_lines: narrow(self.lines),
            num_cols: narrow(self.columns),
            cell_width: (self.cell.width * self.scale).round() as u16,
            cell_height: (self.cell.height * self.scale).round() as u16,
        }
    }
}

pub struct Session {
    term: Arc<FairMutex<Term<Listener>>>,
    notifier: Notifier,
}

impl Session {
    pub fn start(launch: &Launch, size: Size, listener: Listener) -> io::Result<Self> {
        let plan = plan(launch, zsh_availability(), &zdotdir());
        let options = Options {
            shell: plan
                .program
                .map(|(program, args)| Shell::new(program, args)),
            working_directory: launch.directory.clone(),
            drain_on_exit: false,
            env: plan.environment.into_iter().collect(),
        };
        let pty = tty::new(&options, size.window(), 0)?;
        let term = Arc::new(FairMutex::new(Term::new(
            terminal_config(),
            &size,
            listener.clone(),
        )));
        let event_loop = EventLoop::new(Arc::clone(&term), listener, pty, false, false)?;
        let notifier = Notifier(event_loop.channel());
        event_loop.spawn();
        Ok(Self { term, notifier })
    }

    pub fn write(&self, bytes: Vec<u8>) {
        self.notifier.notify(bytes);
    }

    pub fn resize(&mut self, size: Size) {
        self.term.lock().resize(size);
        self.notifier.on_resize(size.window());
    }

    pub fn scroll(&self, scroll: Scroll) {
        self.term.lock().scroll_display(scroll);
    }

    pub fn mode(&self) -> TermMode {
        *self.term.lock().mode()
    }

    pub fn copy(&self) -> Option<String> {
        self.term.lock().selection_to_string()
    }

    pub fn select(&self, spot: Spot) {
        let mut term = self.term.lock();
        let point = point_of(&term, spot);
        term.selection = Some(Selection::new(
            SelectionType::Simple,
            point,
            side_of(spot.half),
        ));
    }

    pub fn extend(&self, spot: Spot) {
        let mut term = self.term.lock();
        let point = point_of(&term, spot);
        if let Some(selection) = term.selection.as_mut() {
            selection.update(point, side_of(spot.half));
        }
    }

    pub fn clear_selection(&self) {
        self.term.lock().selection = None;
    }

    pub fn screen(&self, palette: &TerminalPalette) -> Screen {
        snapshot(&self.term.lock(), palette)
    }
}

pub fn terminal_config() -> Config {
    Config {
        osc52: Osc52::OnlyCopy,
        ..Config::default()
    }
}

fn zsh_availability() -> Availability {
    match installed(ZSH) {
        true => Availability::Installed,
        false => Availability::Missing,
    }
}

fn zdotdir() -> PathBuf {
    let home = env::var("HOME").unwrap_or_default();
    let config_home = env::var("XDG_CONFIG_HOME").ok();
    let sparky_path = env::var("SPARKY_PATH").ok();
    chosen(
        &dirs(config_home.as_deref(), &home, sparky_path.as_deref()),
        Path::is_file,
    )
}

fn point_of(term: &Term<Listener>, spot: Spot) -> Point {
    let offset = term.grid().display_offset() as i32;
    Point::new(Line(spot.line as i32 - offset), Column(spot.column))
}

fn side_of(half: Half) -> Side {
    match half {
        Half::Left => Side::Left,
        Half::Right => Side::Right,
    }
}
