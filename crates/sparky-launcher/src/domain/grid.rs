#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Next,
    Previous,
    Up,
    Down,
}

pub fn moved(cursor: usize, len: usize, columns: usize, step: Move) -> usize {
    let last = len.saturating_sub(1);
    let columns = columns.max(1);
    match step {
        Move::Next => cursor.saturating_add(1).min(last),
        Move::Previous => cursor.saturating_sub(1),
        Move::Up => cursor.checked_sub(columns).unwrap_or(cursor),
        Move::Down if cursor + columns <= last => cursor + columns,
        Move::Down if row_of(cursor, columns) < row_of(last, columns) => last,
        Move::Down => cursor,
    }
}

pub fn row_of(index: usize, columns: usize) -> usize {
    index / columns.max(1)
}

pub fn rows(len: usize, columns: usize) -> usize {
    len.div_ceil(columns.max(1))
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Window {
    pub offset: f32,
    pub height: f32,
}

pub fn reveal(row: usize, step: f32, cell: f32, window: Window) -> Option<f32> {
    let top = row as f32 * step;
    let bottom = top + cell;
    match () {
        _ if top < window.offset => Some(top),
        _ if window.height > 0.0 && bottom > window.offset + window.height => {
            Some(bottom - window.height)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{Move, Window, moved, reveal, row_of, rows};

    fn janela(offset: f32) -> Window {
        Window {
            offset,
            height: 440.0,
        }
    }

    #[test]
    fn linha_visivel_nao_rola() {
        assert_eq!(reveal(2, 116.0, 108.0, janela(0.0)), None);
    }

    #[test]
    fn linha_abaixo_rola_ate_ela_aparecer_inteira() {
        assert_eq!(reveal(4, 116.0, 108.0, janela(0.0)), Some(132.0));
    }

    #[test]
    fn linha_acima_rola_ate_o_topo_dela() {
        assert_eq!(reveal(1, 116.0, 108.0, janela(300.0)), Some(116.0));
    }

    #[test]
    fn sem_altura_conhecida_so_rola_para_cima() {
        let desconhecida = Window::default();
        assert_eq!(reveal(9, 116.0, 108.0, desconhecida), None);
    }

    #[test]
    fn proximo_e_anterior_andam_uma_casa_e_param_nas_pontas() {
        assert_eq!(moved(0, 10, 6, Move::Next), 1);
        assert_eq!(moved(9, 10, 6, Move::Next), 9);
        assert_eq!(moved(0, 10, 6, Move::Previous), 0);
    }

    #[test]
    fn subir_e_descer_pulam_uma_linha_inteira() {
        assert_eq!(moved(1, 20, 6, Move::Down), 7);
        assert_eq!(moved(7, 20, 6, Move::Up), 1);
        assert_eq!(moved(2, 20, 6, Move::Up), 2);
    }

    #[test]
    fn descer_para_uma_linha_incompleta_cai_no_ultimo() {
        assert_eq!(moved(5, 8, 6, Move::Down), 7);
        assert_eq!(moved(7, 8, 6, Move::Down), 7);
    }

    #[test]
    fn lista_de_uma_coluna_anda_de_um_em_um() {
        assert_eq!(moved(0, 5, 1, Move::Down), 1);
        assert_eq!(moved(3, 5, 1, Move::Up), 2);
    }

    #[test]
    fn grade_vazia_nao_quebra() {
        assert_eq!(moved(0, 0, 6, Move::Down), 0);
        assert_eq!(moved(0, 0, 6, Move::Next), 0);
        assert_eq!(rows(0, 6), 0);
    }

    #[test]
    fn conta_linhas_e_acha_a_linha_de_cada_casa() {
        assert_eq!(rows(13, 6), 3);
        assert_eq!(row_of(12, 6), 2);
    }
}
