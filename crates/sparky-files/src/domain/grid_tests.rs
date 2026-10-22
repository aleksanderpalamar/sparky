use super::grid::{Direction, Layout, ViewMode};

#[test]
fn grade_e_o_padrao_e_o_intervalo_segue_linhas_inteiras() {
    assert_eq!(ViewMode::default(), ViewMode::Grid);
    let grid = Layout::new(4, 3);
    assert_eq!(grid.row_count(19), 5);
    assert_eq!(grid.range(2, 19), 8..19);
    assert_eq!(grid.range(0, 2), 0..2);
}

#[test]
fn setas_respeitam_colunas_e_nao_atravessam_as_bordas_da_linha() {
    let grid = Layout::new(4, 3);
    assert_eq!(grid.neighbor(2, 11, Direction::Down), 6);
    assert_eq!(grid.neighbor(6, 11, Direction::Up), 2);
    assert_eq!(grid.neighbor(3, 11, Direction::Right), 3);
    assert_eq!(grid.neighbor(4, 11, Direction::Left), 4);
    assert_eq!(grid.neighbor(7, 11, Direction::Down), 10);
    assert_eq!(grid.neighbor(0, 0, Direction::Down), 0);
    assert_eq!(grid.neighbor(2, 11, Direction::Up), 2);
    assert_eq!(grid.neighbor(9, 11, Direction::Down), 9);
}

#[test]
fn pagina_respeita_a_capacidade_e_janelas_pequenas_preservam_uma_celula() {
    let grid = Layout::new(4, 3);
    assert_eq!(grid.capacity(), 12);
    let small = Layout::new(0, 0);
    assert_eq!(small.capacity(), 1);
    assert_eq!(small.row_count(0), 0);
}
