pub mod colors_file;
pub mod locations;
pub mod palette;
pub mod rgb;
pub mod skin;

pub use palette::{Mode, Palette};
pub use rgb::Rgb;
pub use skin::Skin;

pub const FONT_FAMILY: &str = "JetBrainsMono Nerd Font";

pub fn current_palette() -> Palette {
    Palette::load(
        |path| std::fs::read_to_string(path).ok(),
        &locations::color_files_from_env(),
    )
}
