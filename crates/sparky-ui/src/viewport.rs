use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    top: usize,
    rows: usize,
}

impl Viewport {
    pub fn new(rows: usize) -> Self {
        Self { top: 0, rows: rows.max(1) }
    }

    pub fn top(&self) -> usize {
        self.top
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn resize(&mut self, rows: usize, cursor: usize, len: usize) {
        self.rows = rows.max(1);
        self.follow(cursor, len);
    }

    pub fn follow(&mut self, cursor: usize, len: usize) {
        if cursor < self.top {
            self.top = cursor;
        }
        if cursor >= self.top + self.rows {
            self.top = cursor + 1 - self.rows;
        }
        self.top = self.top.min(len.saturating_sub(self.rows));
    }

    pub fn scroll(&mut self, delta: isize, len: usize) {
        self.top = self.top.saturating_add_signed(delta).min(len.saturating_sub(self.rows));
    }

    pub fn range(&self, len: usize) -> Range<usize> {
        self.top.min(len)..(self.top + self.rows).min(len)
    }
}

#[cfg(test)]
mod tests {
    use super::Viewport;

    #[test]
    fn o_cursor_abaixo_da_janela_empurra_ela_para_baixo() {
        let mut viewport = Viewport::new(10);
        viewport.follow(14, 100);
        assert_eq!(viewport.range(100), 5..15);
    }

    #[test]
    fn o_cursor_acima_da_janela_puxa_ela_para_cima() {
        let mut viewport = Viewport::new(10);
        viewport.follow(50, 100);
        viewport.follow(20, 100);
        assert_eq!(viewport.top(), 20);
    }

    #[test]
    fn a_janela_nao_passa_do_fim_da_lista() {
        let mut viewport = Viewport::new(10);
        viewport.scroll(500, 25);
        assert_eq!(viewport.range(25), 15..25);
        viewport.scroll(-500, 25);
        assert_eq!(viewport.top(), 0);
    }

    #[test]
    fn lista_menor_que_a_janela_mostra_tudo() {
        let mut viewport = Viewport::new(10);
        viewport.follow(3, 4);
        assert_eq!(viewport.range(4), 0..4);
    }

    #[test]
    fn mudar_o_tamanho_mantem_o_cursor_visivel() {
        let mut viewport = Viewport::new(20);
        viewport.follow(15, 100);
        viewport.resize(5, 15, 100);
        assert_eq!(viewport.range(100), 11..16);
        assert_eq!(viewport.rows(), 5);
    }
}
