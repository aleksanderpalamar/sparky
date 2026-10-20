use crate::geometry::{Direction, Rect};

pub fn neighbor<K: Copy + PartialEq>(boxes: &[(K, Rect)], from: K, direction: Direction) -> Option<K> {
    let (_, origin) = boxes.iter().find(|(key, _)| *key == from)?;
    boxes
        .iter()
        .filter(|(key, _)| *key != from)
        .filter_map(|(key, rect)| score(origin, rect, direction).map(|score| (*key, score)))
        .min_by_key(|(_, (distance, overlap))| (*distance, -*overlap))
        .map(|(key, _)| key)
}

fn score(origin: &Rect, candidate: &Rect, direction: Direction) -> Option<(i32, i32)> {
    let (distance, overlap) = match direction {
        Direction::Left => (origin.x - candidate.right(), vertical_overlap(origin, candidate)),
        Direction::Right => (candidate.x - origin.right(), vertical_overlap(origin, candidate)),
        Direction::Up => (origin.y - candidate.bottom(), horizontal_overlap(origin, candidate)),
        Direction::Down => (candidate.y - origin.bottom(), horizontal_overlap(origin, candidate)),
    };
    (distance >= 0 && overlap > 0).then_some((distance, overlap))
}

fn vertical_overlap(first: &Rect, second: &Rect) -> i32 {
    first.bottom().min(second.bottom()) - first.y.max(second.y)
}

fn horizontal_overlap(first: &Rect, second: &Rect) -> i32 {
    first.right().min(second.right()) - first.x.max(second.x)
}

#[cfg(test)]
mod tests {
    use super::neighbor;
    use crate::geometry::{Direction, Rect};
    use crate::ids::WindowId;

    fn tres() -> Vec<(WindowId, Rect)> {
        vec![
            (WindowId(1), Rect::new(10, 10, 945, 1060)),
            (WindowId(2), Rect::new(965, 10, 945, 525)),
            (WindowId(3), Rect::new(965, 545, 945, 525)),
        ]
    }

    #[test]
    fn a_direita_da_grande_esta_a_de_cima() {
        assert_eq!(neighbor(&tres(), WindowId(1), Direction::Right), Some(WindowId(2)));
    }

    #[test]
    fn a_esquerda_das_empilhadas_esta_a_grande() {
        assert_eq!(neighbor(&tres(), WindowId(3), Direction::Left), Some(WindowId(1)));
    }

    #[test]
    fn abaixo_da_de_cima_esta_a_de_baixo() {
        assert_eq!(neighbor(&tres(), WindowId(2), Direction::Down), Some(WindowId(3)));
        assert_eq!(neighbor(&tres(), WindowId(3), Direction::Up), Some(WindowId(2)));
    }

    #[test]
    fn na_borda_nao_ha_vizinho() {
        assert_eq!(neighbor(&tres(), WindowId(1), Direction::Left), None);
        assert_eq!(neighbor(&tres(), WindowId(2), Direction::Up), None);
    }

    #[test]
    fn janela_desconhecida_nao_tem_vizinho() {
        assert_eq!(neighbor(&tres(), WindowId(9), Direction::Right), None);
    }

    #[test]
    fn empate_de_distancia_fica_com_a_maior_sobreposicao() {
        let janelas = vec![
            (WindowId(1), Rect::new(0, 0, 100, 300)),
            (WindowId(2), Rect::new(110, 0, 100, 50)),
            (WindowId(3), Rect::new(110, 50, 100, 250)),
        ];
        assert_eq!(neighbor(&janelas, WindowId(1), Direction::Right), Some(WindowId(3)));
    }
}
