use crate::view::kit::chrome::FOOTER_HEIGHT;
use crate::view::kit::tokens::space;

pub const APPLICATION_ID: &str = "sparky-settings";
pub const WINDOW_WIDTH: f32 = 960.0;
pub const WINDOW_HEIGHT: f32 = 640.0;
pub const LINE_HEIGHT: f32 = space::ROW_HEIGHT + space::XXS;
pub const KEYS_WIDTH: f32 = 280.0;
const HEADER: f32 = 48.0;
const CAPTION: f32 = 22.0;
const CHROME: f32 = HEADER + FOOTER_HEIGHT + 2.0 + space::HUGE * 2.0 + CAPTION;

pub fn rows_for(height: f32) -> usize {
    ((height - CHROME) / LINE_HEIGHT).floor().max(1.0) as usize
}

#[cfg(test)]
mod tests {
    use super::{CHROME, LINE_HEIGHT, rows_for};

    #[test]
    fn as_linhas_da_lista_cabem_na_janela() {
        assert_eq!(rows_for(CHROME + LINE_HEIGHT * 12.0), 12);
        assert_eq!(rows_for(0.0), 1);
    }
}
