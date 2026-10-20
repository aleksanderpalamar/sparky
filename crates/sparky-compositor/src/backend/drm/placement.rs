use sparky_tiling::Rect;

pub fn next_position(existing: &[Rect]) -> (i32, i32) {
    let right = existing.iter().map(Rect::right).max().unwrap_or(0);
    (right, 0)
}

#[cfg(test)]
mod tests {
    use super::next_position;
    use sparky_tiling::Rect;

    #[test]
    fn o_primeiro_monitor_fica_na_origem() {
        assert_eq!(next_position(&[]), (0, 0));
    }

    #[test]
    fn o_monitor_novo_fica_a_direita_dos_outros() {
        let existentes = [Rect::new(0, 0, 1920, 1080), Rect::new(1920, 0, 2560, 1440)];
        assert_eq!(next_position(&existentes), (4480, 0));
    }
}
