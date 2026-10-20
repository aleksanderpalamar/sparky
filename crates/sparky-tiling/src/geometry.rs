#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arrangement {
    SideBySide,
    Stacked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Rect {
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    pub fn right(&self) -> i32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> i32 {
        self.y + self.height
    }

    pub fn arrangement(&self) -> Arrangement {
        match self.width > self.height {
            true => Arrangement::SideBySide,
            false => Arrangement::Stacked,
        }
    }

    pub fn split(&self, arrangement: Arrangement, ratio: f32) -> (Rect, Rect) {
        match arrangement {
            Arrangement::SideBySide => {
                let first = (self.width as f32 * ratio).round() as i32;
                (
                    Rect::new(self.x, self.y, first, self.height),
                    Rect::new(self.x + first, self.y, self.width - first, self.height),
                )
            }
            Arrangement::Stacked => {
                let first = (self.height as f32 * ratio).round() as i32;
                (
                    Rect::new(self.x, self.y, self.width, first),
                    Rect::new(self.x, self.y + first, self.width, self.height - first),
                )
            }
        }
    }

    pub fn shrink(&self, left: i32, top: i32, right: i32, bottom: i32) -> Rect {
        Rect::new(
            self.x + left,
            self.y + top,
            (self.width - left - right).max(1),
            (self.height - top - bottom).max(1),
        )
    }

    pub fn centered(&self, width: i32, height: i32) -> Rect {
        let width = width.min(self.width);
        let height = height.min(self.height);
        Rect::new(
            self.x + (self.width - width) / 2,
            self.y + (self.height - height) / 2,
            width,
            height,
        )
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }
}

#[cfg(test)]
mod tests {
    use super::{Arrangement, Rect};

    #[test]
    fn caixa_larga_divide_lado_a_lado() {
        assert_eq!(Rect::new(0, 0, 1920, 1080).arrangement(), Arrangement::SideBySide);
    }

    #[test]
    fn caixa_alta_ou_quadrada_divide_empilhado() {
        assert_eq!(Rect::new(0, 0, 960, 1080).arrangement(), Arrangement::Stacked);
        assert_eq!(Rect::new(0, 0, 500, 500).arrangement(), Arrangement::Stacked);
    }

    #[test]
    fn divide_ao_meio_lado_a_lado() {
        let (first, second) = Rect::new(10, 20, 1000, 500).split(Arrangement::SideBySide, 0.5);
        assert_eq!(first, Rect::new(10, 20, 500, 500));
        assert_eq!(second, Rect::new(510, 20, 500, 500));
    }

    #[test]
    fn divide_empilhado_com_proporcao() {
        let (first, second) = Rect::new(0, 0, 800, 1000).split(Arrangement::Stacked, 0.3);
        assert_eq!(first, Rect::new(0, 0, 800, 300));
        assert_eq!(second, Rect::new(0, 300, 800, 700));
    }

    #[test]
    fn a_divisao_nao_perde_pixel_em_tamanho_impar() {
        let (first, second) = Rect::new(0, 0, 1001, 10).split(Arrangement::SideBySide, 0.5);
        assert_eq!(first.width + second.width, 1001);
        assert_eq!(second.right(), 1001);
    }

    #[test]
    fn encolhe_cada_lado_separadamente() {
        assert_eq!(Rect::new(0, 0, 100, 100).shrink(10, 5, 10, 5), Rect::new(10, 5, 80, 90));
    }

    #[test]
    fn encolher_demais_deixa_um_pixel() {
        assert_eq!(Rect::new(0, 0, 10, 10).shrink(20, 0, 20, 0).width, 1);
    }

    #[test]
    fn centraliza_dentro_da_caixa() {
        assert_eq!(Rect::new(0, 0, 1000, 800).centered(400, 200), Rect::new(300, 300, 400, 200));
    }

    #[test]
    fn centralizar_maior_que_a_caixa_ocupa_a_caixa() {
        assert_eq!(Rect::new(0, 0, 100, 100).centered(400, 400), Rect::new(0, 0, 100, 100));
    }

    #[test]
    fn contem_o_canto_de_cima_mas_nao_o_de_baixo() {
        let rect = Rect::new(10, 10, 20, 20);
        assert!(rect.contains(10, 10));
        assert!(!rect.contains(30, 30));
    }
}
