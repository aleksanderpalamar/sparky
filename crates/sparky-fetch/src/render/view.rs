use crate::config::settings::{ColorMode, Colors, Format, Layout, Logo, Settings};

const WIDTH: usize = 80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Terminal {
    pub tty: bool,
    pub columns: Option<usize>,
    pub no_color: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct View {
    pub format: Format,
    pub color: bool,
    pub logo: Logo,
    pub layout: Layout,
    pub width: usize,
    pub colors: Colors,
}

impl View {
    pub fn resolve(settings: &Settings, terminal: &Terminal) -> View {
        let format = settings.format.unwrap_or(match terminal.tty {
            true => Format::Visual,
            false => Format::Plain,
        });
        let color = match settings.color {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => terminal.tty && !terminal.no_color && format == Format::Visual,
        };
        View {
            format,
            color,
            logo: settings.logo,
            layout: settings.layout,
            width: settings.width.or(terminal.columns).unwrap_or(WIDTH),
            colors: settings.colors,
        }
    }
}
