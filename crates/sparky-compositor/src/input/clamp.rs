use sparky_tiling::Rect;

pub fn clamp(x: f64, y: f64, screens: &[Rect]) -> Option<(f64, f64)> {
    screens
        .iter()
        .map(|screen| inside(x, y, screen))
        .min_by(|first, second| distance(x, y, *first).total_cmp(&distance(x, y, *second)))
}

fn inside(x: f64, y: f64, screen: &Rect) -> (f64, f64) {
    let right = (screen.right() - 1).max(screen.x) as f64;
    let bottom = (screen.bottom() - 1).max(screen.y) as f64;
    (x.clamp(screen.x as f64, right), y.clamp(screen.y as f64, bottom))
}

fn distance(x: f64, y: f64, point: (f64, f64)) -> f64 {
    (x - point.0).powi(2) + (y - point.1).powi(2)
}

#[cfg(test)]
mod tests {
    use super::clamp;
    use sparky_tiling::Rect;

    const TELAS: [Rect; 2] = [Rect::new(0, 0, 1920, 1080), Rect::new(1920, 0, 2560, 1440)];

    #[test]
    fn dentro_de_uma_tela_nao_muda() {
        assert_eq!(clamp(100.5, 200.0, &TELAS), Some((100.5, 200.0)));
        assert_eq!(clamp(3000.0, 1300.0, &TELAS), Some((3000.0, 1300.0)));
    }

    #[test]
    fn fora_de_tudo_para_na_borda_mais_proxima() {
        assert_eq!(clamp(-50.0, 500.0, &TELAS), Some((0.0, 500.0)));
        assert_eq!(clamp(5000.0, 2000.0, &TELAS), Some((4479.0, 1439.0)));
    }

    #[test]
    fn abaixo_da_tela_menor_vai_para_a_borda_dela_ou_da_vizinha() {
        assert_eq!(clamp(1000.0, 1200.0, &TELAS), Some((1000.0, 1079.0)));
    }

    #[test]
    fn sem_tela_nao_ha_posicao() {
        assert_eq!(clamp(1.0, 1.0, &[]), None);
    }
}
