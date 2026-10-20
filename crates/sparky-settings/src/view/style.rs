use sparky_ui::style::{PADDING, STATUS_HEIGHT};

pub const APPLICATION_ID: &str = "sparky-settings";
pub const WINDOW_WIDTH: f32 = 960.0;
pub const WINDOW_HEIGHT: f32 = 640.0;
pub const LINE_HEIGHT: f32 = 30.0;
pub const VALUE_WIDTH: f32 = 280.0;
pub const NUMBER_WIDTH: f32 = 64.0;
pub const KEYS_WIDTH: f32 = 280.0;
pub const SECTION_HEADER: f32 = 56.0;
const CHROME: f32 = SECTION_HEADER + STATUS_HEIGHT + PADDING * 3.0;

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
