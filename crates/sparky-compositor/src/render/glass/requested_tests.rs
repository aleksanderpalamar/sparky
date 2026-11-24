use smithay::utils::{Logical, Point, Rectangle, Size};
use smithay::wayland::compositor::{RectangleKind, RegionAttributes};

use super::requested::rects;

fn retangulo(x: i32, y: i32, w: i32, h: i32) -> Rectangle<i32, Logical> {
    Rectangle::new((x, y).into(), (w, h).into())
}

fn regiao(partes: &[(RectangleKind, Rectangle<i32, Logical>)]) -> RegionAttributes {
    RegionAttributes {
        rects: partes.to_vec(),
    }
}

fn origem() -> Point<i32, Logical> {
    Point::new(100, 50)
}

fn tamanho() -> Size<i32, Logical> {
    Size::new(300, 200)
}

#[test]
fn regiao_infinita_cobre_a_superficie_inteira_na_tela() {
    let pedido = regiao(&[(RectangleKind::Add, retangulo(0, 0, i32::MAX, i32::MAX))]);
    assert_eq!(
        rects(&pedido, origem(), tamanho()),
        vec![retangulo(100, 50, 300, 200)]
    );
}

#[test]
fn a_parte_subtraida_fica_sem_desfoque() {
    let pedido = regiao(&[
        (RectangleKind::Add, retangulo(0, 0, 300, 200)),
        (RectangleKind::Subtract, retangulo(0, 100, 300, 100)),
    ]);
    assert_eq!(
        rects(&pedido, origem(), tamanho()),
        vec![retangulo(100, 50, 300, 100)]
    );
}

#[test]
fn as_partes_valem_na_ordem_em_que_foram_pedidas() {
    let pedido = regiao(&[
        (RectangleKind::Subtract, retangulo(0, 0, 300, 200)),
        (RectangleKind::Add, retangulo(10, 10, 50, 50)),
    ]);
    assert_eq!(
        rects(&pedido, origem(), tamanho()),
        vec![retangulo(110, 60, 50, 50)]
    );
}

#[test]
fn regiao_vazia_ou_fora_da_superficie_nao_desfoca() {
    assert!(rects(&regiao(&[]), origem(), tamanho()).is_empty());
    let fora = regiao(&[(RectangleKind::Add, retangulo(400, 0, 50, 50))]);
    assert!(rects(&fora, origem(), tamanho()).is_empty());
}
