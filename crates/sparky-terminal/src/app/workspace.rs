use iced::Size;
use sparky_theme::{Palette, Skin, TerminalPalette};

use super::state::{HEIGHT, Terminal, WIDTH};
use crate::domain::grid::Cell;
use crate::domain::input::Focus;
use crate::domain::launch::Launch;
use crate::domain::tabs::Tabs;

pub const TAB_HEIGHT: f32 = 32.0;

pub struct Workspace {
    pub tabs: Tabs<Terminal>,
    pub launch: Launch,
    pub skin: Skin,
    pub palette: TerminalPalette,
    pub colors: Palette,
    pub cell: Cell,
    pub size: Size,
    pub scale: f32,
    pub focus: Focus,
}

impl Workspace {
    pub fn new(launch: Launch, palette: Palette, colors: TerminalPalette, cell: Cell) -> Self {
        let mut first = Terminal::new(launch.clone(), palette, colors, cell);
        first.fit(WIDTH, HEIGHT - TAB_HEIGHT);
        Self {
            tabs: Tabs::new(first),
            launch,
            skin: Skin::new(palette),
            palette: colors,
            colors: palette,
            cell,
            size: Size::new(WIDTH, HEIGHT),
            scale: 1.0,
            focus: Focus::Gained,
        }
    }

    pub fn title(&self) -> String {
        self.tabs
            .active()
            .map_or_else(|| "Terminal".into(), |tab| tab.value.title.clone())
    }

    pub fn fit(&mut self, size: Size) {
        self.size = size;
        for tab in self.tabs.iter_mut() {
            tab.value
                .fit(size.width, (size.height - TAB_HEIGHT).max(0.0));
        }
    }

    pub fn rescale(&mut self, scale: f32) {
        self.scale = scale;
        for tab in self.tabs.iter_mut() {
            tab.value.rescale(scale);
        }
    }

    pub fn observe_colors(&mut self, palette: Palette, colors: TerminalPalette) {
        self.skin = Skin::new(palette);
        self.colors = palette;
        self.palette = colors;
        for tab in self.tabs.iter_mut() {
            tab.value.observe_colors(palette, colors);
        }
    }

    pub fn focus(&mut self, change: Focus) {
        self.focus = change;
        self.report_active(change);
    }

    pub(super) fn report_active(&mut self, change: Focus) {
        if let Some(tab) = self.tabs.active_mut() {
            super::input::focus(&mut tab.value, change);
        }
    }
}
