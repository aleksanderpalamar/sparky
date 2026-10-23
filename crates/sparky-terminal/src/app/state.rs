use iced::Rectangle;
use iced::widget::canvas;
use sparky_theme::{Palette, Skin, TerminalPalette};

use crate::adapters::session::{Session, Size};
use crate::app::paint::{origin, span};
use crate::domain::grid::{Cell, Metrics};
use crate::domain::launch::Launch;
use crate::domain::screen::Screen;

pub const PADDING: f32 = 18.0;
pub const WIDTH: f32 = 900.0;
pub const HEIGHT: f32 = 560.0;
const TITLE: &str = "Terminal";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Changed {
    Yes,
    No,
}

pub struct Terminal {
    pub launch: Launch,
    pub session: Option<Session>,
    pub skin: Skin,
    pub palette: TerminalPalette,
    pub metrics: Metrics,
    pub size: Size,
    pub screen: Screen,
    pub cache: canvas::Cache,
    pub title: String,
    pub wheel_rest: f32,
}

impl Terminal {
    pub fn new(launch: Launch, palette: Palette, colors: TerminalPalette, cell: Cell) -> Self {
        let metrics = Metrics {
            cell,
            padding: PADDING,
        };
        let (columns, lines) = metrics.grid(WIDTH, HEIGHT);
        Self {
            launch,
            session: None,
            skin: Skin::new(palette),
            palette: colors,
            metrics,
            size: Size {
                columns,
                lines,
                cell,
                scale: 1.0,
            },
            screen: Screen::default(),
            cache: canvas::Cache::new(),
            title: TITLE.to_owned(),
            wheel_rest: 0.0,
        }
    }

    pub fn observe_colors(&mut self, palette: Palette, colors: TerminalPalette) -> Changed {
        let skin = Skin::new(palette);
        if skin == self.skin && colors == self.palette {
            return Changed::No;
        }
        self.skin = skin;
        self.palette = colors;
        self.refresh();
        Changed::Yes
    }

    pub fn fit(&mut self, width: f32, height: f32) {
        let (columns, lines) = self.metrics.grid(width, height);
        if (columns, lines) == (self.size.columns, self.size.lines) {
            return;
        }
        self.size = Size {
            columns,
            lines,
            ..self.size
        };
        self.resize_session();
    }

    pub fn rescale(&mut self, scale: f32) {
        if scale == self.size.scale {
            return;
        }
        self.size.scale = scale;
        self.resize_session();
    }

    pub fn ime_area(&self) -> Rectangle {
        let (line, column) = self
            .screen
            .cursor
            .map_or((0, 0), |cursor| (cursor.line, cursor.column));
        Rectangle::new(origin(self.metrics, line, column), span(self.metrics, 1))
    }

    fn resize_session(&mut self) {
        if let Some(session) = self.session.as_mut() {
            session.resize(self.size);
        }
        self.refresh();
    }

    pub fn refresh(&mut self) {
        if let Some(session) = &self.session {
            self.screen = session.screen(&self.palette);
        }
        self.cache.clear();
    }

    pub fn reset_title(&mut self) {
        TITLE.clone_into(&mut self.title);
    }
}
