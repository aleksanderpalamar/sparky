use sparky_ui::style::{HEADER_HEIGHT, STATUS_HEIGHT};

pub const APPLICATION_ID: &str = "sparky-files";
pub const WINDOW_WIDTH: f32 = 1000.0;
pub const WINDOW_HEIGHT: f32 = 640.0;
pub const ROW_HEIGHT: f32 = 28.0;
pub const SIZE_WIDTH: f32 = 90.0;
pub const DATE_WIDTH: f32 = 140.0;
const CHROME: f32 = HEADER_HEIGHT + ROW_HEIGHT + STATUS_HEIGHT;

pub fn rows_for(height: f32) -> usize {
    ((height - CHROME) / ROW_HEIGHT).floor().max(1.0) as usize
}

#[cfg(test)]
mod tests {
    use super::{CHROME, ROW_HEIGHT, rows_for};

    #[test]
    fn as_linhas_cabem_no_que_sobra_da_janela() {
        assert_eq!(rows_for(CHROME + ROW_HEIGHT * 10.0 + 5.0), 10);
    }

    #[test]
    fn janela_minuscula_ainda_mostra_uma_linha() {
        assert_eq!(rows_for(10.0), 1);
    }
}
