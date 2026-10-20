use crate::geometry::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gaps {
    pub inner: i32,
    pub outer: i32,
}

impl Gaps {
    pub const SPARKY: Self = Self { inner: 5, outer: 10 };

    pub fn apply(&self, tile: Rect, area: Rect) -> Rect {
        let side = |at_edge: bool| match at_edge {
            true => self.outer,
            false => self.inner,
        };
        tile.shrink(
            side(tile.x == area.x),
            side(tile.y == area.y),
            side(tile.right() == area.right()),
            side(tile.bottom() == area.bottom()),
        )
    }

    pub fn outer_area(&self, area: Rect) -> Rect {
        area.shrink(self.outer, self.outer, self.outer, self.outer)
    }
}

#[cfg(test)]
mod tests {
    use super::Gaps;
    use crate::geometry::Rect;

    const AREA: Rect = Rect::new(0, 26, 1920, 1054);

    #[test]
    fn janela_sozinha_fica_com_a_folga_de_fora_em_volta() {
        assert_eq!(Gaps::SPARKY.apply(AREA, AREA), Rect::new(10, 36, 1900, 1034));
    }

    #[test]
    fn entre_duas_janelas_cada_lado_recebe_a_folga_de_dentro() {
        let left = Gaps::SPARKY.apply(Rect::new(0, 26, 960, 1054), AREA);
        let right = Gaps::SPARKY.apply(Rect::new(960, 26, 960, 1054), AREA);
        assert_eq!(left, Rect::new(10, 36, 945, 1034));
        assert_eq!(right, Rect::new(965, 36, 945, 1034));
        assert_eq!(right.x - left.right(), 10);
    }

    #[test]
    fn a_area_de_fora_tira_a_folga_de_todos_os_lados() {
        assert_eq!(Gaps::SPARKY.outer_area(AREA), Rect::new(10, 36, 1900, 1034));
    }
}
