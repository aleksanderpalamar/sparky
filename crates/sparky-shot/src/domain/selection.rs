use sparky_tiling::Rect;

const MINIMUM: i32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    bounds: Rect,
    rect: Rect,
    anchor: Option<(i32, i32)>,
}

impl Selection {
    pub fn new(width: i32, height: i32) -> Self {
        let bounds = Rect::new(0, 0, width.max(MINIMUM), height.max(MINIMUM));
        Self {
            bounds,
            rect: bounds.centered(bounds.width / 2, bounds.height / 2),
            anchor: None,
        }
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn press(&mut self, x: i32, y: i32) {
        self.anchor = Some(self.inside(x, y));
    }

    pub fn drag(&mut self, x: i32, y: i32) {
        if let Some(anchor) = self.anchor {
            let span = span(anchor, self.inside(x, y));
            if span.width >= MINIMUM && span.height >= MINIMUM {
                self.rect = span;
            }
        }
    }

    pub fn release(&mut self, x: i32, y: i32) {
        self.drag(x, y);
        self.anchor = None;
    }

    pub fn nudge(&mut self, dx: i32, dy: i32) {
        let x = (self.rect.x + dx).clamp(self.bounds.x, self.bounds.right() - self.rect.width);
        let y = (self.rect.y + dy).clamp(self.bounds.y, self.bounds.bottom() - self.rect.height);
        self.rect = Rect::new(x, y, self.rect.width, self.rect.height);
    }

    pub fn grow(&mut self, dw: i32, dh: i32) {
        let width = (self.rect.width + dw).clamp(MINIMUM, self.bounds.right() - self.rect.x);
        let height = (self.rect.height + dh).clamp(MINIMUM, self.bounds.bottom() - self.rect.y);
        self.rect = Rect::new(self.rect.x, self.rect.y, width, height);
    }

    pub fn whole(&mut self) {
        self.rect = self.bounds;
    }

    fn inside(&self, x: i32, y: i32) -> (i32, i32) {
        (
            x.clamp(self.bounds.x, self.bounds.right()),
            y.clamp(self.bounds.y, self.bounds.bottom()),
        )
    }
}

fn span(first: (i32, i32), second: (i32, i32)) -> Rect {
    let x = first.0.min(second.0);
    let y = first.1.min(second.1);
    Rect::new(x, y, (first.0 - second.0).abs(), (first.1 - second.1).abs())
}

#[cfg(test)]
mod tests {
    use super::Selection;
    use sparky_tiling::Rect;

    fn tela() -> Selection {
        Selection::new(1920, 1080)
    }

    #[test]
    fn comeca_com_metade_da_tela_no_centro() {
        assert_eq!(tela().rect(), Rect::new(480, 270, 960, 540));
    }

    #[test]
    fn arrastar_desenha_a_selecao_em_qualquer_sentido() {
        let mut selection = tela();
        selection.press(500, 400);
        selection.drag(100, 100);
        assert_eq!(selection.rect(), Rect::new(100, 100, 400, 300));
        selection.release(700, 600);
        assert_eq!(selection.rect(), Rect::new(500, 400, 200, 200));
    }

    #[test]
    fn um_clique_sem_arrastar_mantem_a_selecao_anterior() {
        let mut selection = tela();
        selection.press(10, 10);
        selection.release(10, 10);
        assert_eq!(selection.rect(), Rect::new(480, 270, 960, 540));
    }

    #[test]
    fn mexer_o_mouse_sem_apertar_nao_muda_nada() {
        let mut selection = tela();
        selection.drag(10, 10);
        assert_eq!(selection.rect(), Rect::new(480, 270, 960, 540));
    }

    #[test]
    fn arrastar_para_fora_para_na_borda() {
        let mut selection = tela();
        selection.press(1800, 1000);
        selection.release(5000, 5000);
        assert_eq!(selection.rect(), Rect::new(1800, 1000, 120, 80));
    }

    #[test]
    fn as_setas_movem_sem_sair_da_tela() {
        let mut selection = tela();
        selection.nudge(10, -10);
        assert_eq!(selection.rect(), Rect::new(490, 260, 960, 540));
        selection.nudge(-5000, 5000);
        assert_eq!(selection.rect(), Rect::new(0, 540, 960, 540));
    }

    #[test]
    fn redimensionar_tem_minimo_e_para_na_borda() {
        let mut selection = tela();
        selection.grow(-5000, 0);
        assert_eq!(selection.rect().width, 1);
        selection.grow(5000, 5000);
        assert_eq!(selection.rect(), Rect::new(480, 270, 1440, 810));
    }

    #[test]
    fn a_tela_toda_e_um_atalho() {
        let mut selection = tela();
        selection.whole();
        assert_eq!(selection.rect(), Rect::new(0, 0, 1920, 1080));
    }
}
