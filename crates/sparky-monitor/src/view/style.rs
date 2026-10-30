use iced::Size;
use sparky_ui::style::{HEADER_HEIGHT, PADDING, STATUS_HEIGHT};

pub const APPLICATION_ID: &str = "sparky-monitor";
pub const WINDOW: Size = Size {
    width: 1280.0,
    height: 800.0,
};
pub const ROW_HEIGHT: f32 = 30.0;
pub const VALUE_WIDTH: f32 = 96.0;
pub const ICON_SIZE: f32 = 18.0;
pub const SECTION_TITLE: f32 = 18.0;
pub const CARD_CHART: f32 = 150.0;
pub const DETAIL_CHART: f32 = 220.0;
pub const DASHBOARD_APP_ROWS: usize = 8;
pub const DETAIL_ROWS: usize = 8;
const TITLE_SPACE: f32 = 40.0;
const LINK_SPACE: f32 = 34.0;

pub fn table_rows(height: f32) -> usize {
    let free = height - HEADER_HEIGHT - STATUS_HEIGHT - PADDING * 2.0 - TITLE_SPACE - ROW_HEIGHT;
    (free / ROW_HEIGHT).floor().max(1.0) as usize
}

pub fn dashboard_process_rows(height: f32) -> usize {
    let used =
        (TITLE_SPACE + ROW_HEIGHT + LINK_SPACE) * 2.0 + DASHBOARD_APP_ROWS as f32 * ROW_HEIGHT;
    let free = height - HEADER_HEIGHT - STATUS_HEIGHT - PADDING * 2.0 - used;
    (free / ROW_HEIGHT).floor().max(3.0) as usize
}

#[cfg(test)]
mod tests {
    use super::{WINDOW, dashboard_process_rows, table_rows};

    #[test]
    fn linhas_acompanham_a_altura_da_janela() {
        assert_eq!(table_rows(WINDOW.height), 20);
        assert!(table_rows(WINDOW.height + 300.0) == 30);
        assert_eq!(table_rows(10.0), 1);
        assert_eq!(dashboard_process_rows(WINDOW.height), 8);
        assert_eq!(dashboard_process_rows(100.0), 3);
    }
}
