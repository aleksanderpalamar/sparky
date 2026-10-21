use sparky_tiling::Rect;

pub const GAP: i32 = 5;

pub fn slots(bar: Rect, count: usize) -> Vec<Rect> {
    let count = count as i32;
    if count == 0 {
        return Vec::new();
    }
    let usable = (bar.width - GAP * (count - 1)).max(count);
    (0..count)
        .map(|index| {
            let left = bar.x + index * GAP + usable * index / count;
            let right = bar.x + index * GAP + usable * (index + 1) / count;
            Rect::new(left, bar.y, right - left, bar.height)
        })
        .collect()
}

pub fn indicator(slot: Rect) -> Rect {
    Rect::new(slot.x, slot.y + slot.height - 1, slot.width, 1)
}

#[cfg(test)]
mod tests {
    use super::{indicator, slots};
    use sparky_tiling::Rect;

    #[test]
    fn as_abas_dividem_a_barra_com_folga_entre_elas() {
        assert_eq!(
            slots(Rect::new(10, 20, 310, 22), 3),
            vec![Rect::new(10, 20, 100, 22), Rect::new(115, 20, 100, 22), Rect::new(220, 20, 100, 22)]
        );
    }

    #[test]
    fn a_ultima_aba_termina_na_borda_mesmo_com_resto() {
        let abas = slots(Rect::new(0, 0, 101, 22), 2);
        assert_eq!(abas.last().map(|aba| aba.x + aba.width), Some(101));
    }

    #[test]
    fn sem_janelas_nao_ha_abas() {
        assert!(slots(Rect::new(0, 0, 100, 22), 0).is_empty());
    }

    #[test]
    fn o_indicador_e_a_ultima_linha_da_aba() {
        assert_eq!(indicator(Rect::new(5, 10, 80, 22)), Rect::new(5, 31, 80, 1));
    }
}
