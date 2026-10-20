use iced::mouse::ScrollDelta;

const LINES_PER_NOTCH: f32 = 3.0;

pub fn rows(delta: ScrollDelta, row_height: f32) -> isize {
    let rows = match delta {
        ScrollDelta::Lines { y, .. } => y * LINES_PER_NOTCH,
        ScrollDelta::Pixels { y, .. } => y / row_height,
    };
    -rows.round() as isize
}

#[cfg(test)]
mod tests {
    use super::rows;
    use iced::mouse::ScrollDelta;

    #[test]
    fn um_degrau_da_roda_para_baixo_desce_tres_linhas() {
        assert_eq!(rows(ScrollDelta::Lines { x: 0.0, y: -1.0 }, 28.0), 3);
        assert_eq!(rows(ScrollDelta::Lines { x: 0.0, y: 1.0 }, 28.0), -3);
    }

    #[test]
    fn o_touchpad_rola_pelos_pixels() {
        assert_eq!(rows(ScrollDelta::Pixels { x: 0.0, y: -56.0 }, 28.0), 2);
    }
}
