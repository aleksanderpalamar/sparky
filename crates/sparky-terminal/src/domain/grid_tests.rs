use crate::domain::grid::{Cell, Half, Metrics, Spot};

const METRICAS: Metrics = Metrics {
    cell: Cell {
        width: 8.0,
        height: 16.0,
    },
    padding: 18.0,
};

#[test]
fn a_grade_cabe_na_janela_descontando_a_margem() {
    assert_eq!(METRICAS.grid(900.0, 560.0), (108, 32));
}

#[test]
fn janela_minuscula_ainda_tem_duas_colunas_e_uma_linha() {
    assert_eq!(METRICAS.grid(10.0, 5.0), (2, 1));
}

#[test]
fn o_mouse_aponta_para_a_celula_e_o_lado_dela() {
    assert_eq!(
        METRICAS.spot(18.0 + 8.0 * 3.0 + 1.0, 18.0 + 16.0 * 2.0 + 4.0, (108, 32)),
        Spot {
            line: 2,
            column: 3,
            half: Half::Left
        }
    );
    assert_eq!(
        METRICAS.spot(18.0 + 8.0 * 3.0 + 7.0, 18.0, (108, 32)),
        Spot {
            line: 0,
            column: 3,
            half: Half::Right
        }
    );
}

#[test]
fn fora_da_grade_o_mouse_fica_na_borda() {
    assert_eq!(
        METRICAS.spot(0.0, 0.0, (108, 32)),
        Spot {
            line: 0,
            column: 0,
            half: Half::Left
        }
    );
    assert_eq!(
        METRICAS.spot(5000.0, 5000.0, (108, 32)),
        Spot {
            line: 31,
            column: 107,
            half: Half::Right
        }
    );
}
