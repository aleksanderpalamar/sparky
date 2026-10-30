use super::turn::{Spin, Turn};
use super::viewport::{Area, Direction, Placement, Viewport};

const JANELA: Area = Area {
    width: 1000.0,
    height: 800.0,
};

fn posicao(placement: Placement) -> (f32, f32, f32, f32) {
    (placement.x, placement.y, placement.width, placement.height)
}

#[test]
fn ajustar_reduz_imagem_grande_e_mantem_pequena_em_tamanho_real() {
    let grande = Viewport::new(2000, 1000);
    assert_eq!(
        posicao(grande.placement(JANELA)),
        (0.0, 150.0, 1000.0, 500.0)
    );
    assert_eq!(grande.percent(JANELA), 50);
    let pequena = Viewport::new(200, 100);
    assert_eq!(
        posicao(pequena.placement(JANELA)),
        (400.0, 350.0, 200.0, 100.0)
    );
    assert_eq!(pequena.percent(JANELA), 100);
}

#[test]
fn girar_troca_largura_e_altura_no_ajuste() {
    let mut viewport = Viewport::new(2000, 1000);
    viewport.rotate(Spin::Clockwise);
    let placement = viewport.placement(JANELA);
    assert_eq!(placement.turn, Turn::Right);
    assert_eq!(posicao(placement), (100.0, 200.0, 800.0, 400.0));
    viewport.rotate(Spin::CounterClockwise);
    viewport.rotate(Spin::CounterClockwise);
    assert_eq!(viewport.placement(JANELA).turn, Turn::Left);
    for _ in 0..3 {
        viewport.rotate(Spin::Clockwise);
    }
    assert_eq!(viewport.placement(JANELA).turn, Turn::Half);
}

#[test]
fn zoom_parte_do_ajuste_e_volta_a_ajustar_ao_diminuir_demais() {
    let mut viewport = Viewport::new(2000, 1000);
    viewport.zoom_in(JANELA);
    assert_eq!(viewport.percent(JANELA), 63);
    viewport.zoom_out(JANELA);
    viewport.zoom_out(JANELA);
    assert!(viewport.is_fit());
    assert_eq!(viewport.percent(JANELA), 50);
    viewport.actual_size();
    assert_eq!(viewport.percent(JANELA), 100);
    for _ in 0..40 {
        viewport.zoom_in(JANELA);
    }
    assert_eq!(viewport.percent(JANELA), 3200);
    viewport.fit();
    assert!(viewport.is_fit());
}

#[test]
fn deslocar_so_anda_quando_a_imagem_passa_da_janela_e_para_nas_bordas() {
    let mut viewport = Viewport::new(2000, 1000);
    assert!(!viewport.overflows(JANELA));
    viewport.look(Direction::Right, JANELA);
    assert_eq!(viewport.placement(JANELA).x, 0.0);
    viewport.actual_size();
    assert!(viewport.overflows(JANELA));
    viewport.look(Direction::Right, JANELA);
    assert_eq!(viewport.placement(JANELA).x, -580.0);
    for _ in 0..20 {
        viewport.look(Direction::Right, JANELA);
        viewport.look(Direction::Down, JANELA);
    }
    assert_eq!(
        posicao(viewport.placement(JANELA)),
        (-1000.0, -200.0, 2000.0, 1000.0)
    );
    viewport.drag(30.0, 500.0, JANELA);
    assert_eq!(
        posicao(viewport.placement(JANELA)),
        (-970.0, 0.0, 2000.0, 1000.0)
    );
}

#[test]
fn zoom_mantem_o_ponto_central_dentro_dos_limites() {
    let mut viewport = Viewport::new(2000, 1000);
    viewport.actual_size();
    for _ in 0..10 {
        viewport.look(Direction::Right, JANELA);
    }
    viewport.zoom_out(JANELA);
    let placement = viewport.placement(JANELA);
    assert_eq!(placement.width, 1600.0);
    assert_eq!(placement.x, -600.0);
}

#[test]
fn janela_sem_area_nao_gera_valores_invalidos() {
    let viewport = Viewport::new(2000, 1000);
    let placement = viewport.placement(Area {
        width: 0.0,
        height: 0.0,
    });
    assert!(
        [placement.x, placement.y, placement.width, placement.height]
            .iter()
            .all(|value| value.is_finite())
    );
}

#[test]
fn arredondamento_do_ajuste_nao_conta_como_imagem_maior_que_a_janela() {
    let area = Area {
        width: 1000.0,
        height: 664.0,
    };
    let mut viewport = Viewport::new(1578, 1052);
    assert!(!viewport.overflows(area));
    let antes = viewport.placement(area);
    viewport.look(Direction::Down, area);
    assert_eq!(viewport.placement(area), antes);
    viewport.zoom_in(area);
    assert!(viewport.overflows(area));
}
