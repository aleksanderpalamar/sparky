pub mod colors_file;
pub mod glass;
pub mod locations;
pub mod names;
pub mod palette;
pub mod rgb;
pub mod skin;
pub mod terminal;
pub mod wallpapers;

pub use palette::{Mode, Palette};
pub use rgb::Rgb;
pub use skin::Skin;
pub use terminal::TerminalPalette;

#[cfg(test)]
mod glass_tests;
#[cfg(test)]
mod names_tests;
#[cfg(test)]
mod terminal_tests;

pub const FONT_FAMILY: &str = "JetBrainsMono Nerd Font";

pub fn current_palette() -> Palette {
    let palette = Palette::load(
        |path| std::fs::read_to_string(path).ok(),
        &locations::color_files_from_env(),
    );
    Palette {
        glass: glass::glass_from_env(),
        ..palette
    }
}

pub fn current_terminal_palette() -> TerminalPalette {
    TerminalPalette::load(
        |path| std::fs::read_to_string(path).ok(),
        &locations::color_files_from_env(),
    )
}
