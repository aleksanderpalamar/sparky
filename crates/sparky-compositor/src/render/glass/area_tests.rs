use smithay::utils::Rectangle;
use sparky_ipc::{GlassArea, GlassRect};

use super::area::{radius, rects};

fn surface() -> Rectangle<i32, smithay::utils::Logical> {
    Rectangle::new((100, 50).into(), (1000, 600).into())
}

#[test]
fn superficie_inteira_vira_um_retangulo_so() {
    assert_eq!(rects(&GlassArea::Whole, surface()), vec![surface()]);
}

#[test]
fn painel_centralizado_fica_no_meio_da_superficie() {
    let area = GlassArea::Centered {
        width: 400,
        height: 200,
    };
    assert_eq!(
        rects(&area, surface()),
        vec![Rectangle::new((400, 250).into(), (400, 200).into())]
    );
}

#[test]
fn retangulos_sao_relativos_a_superficie_e_recortados_nela() {
    let area = GlassArea::Rects {
        rects: vec![
            GlassRect {
                x: 10,
                y: 20,
                width: 50,
                height: 40,
            },
            GlassRect {
                x: 980,
                y: 0,
                width: 100,
                height: 10,
            },
            GlassRect {
                x: 0,
                y: 0,
                width: 0,
                height: 10,
            },
        ],
        radius: 0,
    };
    assert_eq!(
        rects(&area, surface()),
        vec![
            Rectangle::new((110, 70).into(), (50, 40).into()),
            Rectangle::new((1080, 50).into(), (20, 10).into())
        ]
    );
}

#[test]
fn painel_maior_que_a_superficie_fica_limitado_a_ela() {
    let area = GlassArea::Centered {
        width: 5000,
        height: 100,
    };
    assert_eq!(
        rects(&area, surface()),
        vec![Rectangle::new((100, 300).into(), (1000, 100).into())]
    );
}

#[test]
fn so_retangulos_informados_trazem_raio_e_nunca_negativo() {
    let arredondado = GlassArea::Rects {
        rects: Vec::new(),
        radius: 12,
    };
    assert_eq!(radius(&arredondado), 12);
    assert_eq!(
        radius(&GlassArea::Rects {
            rects: Vec::new(),
            radius: -3
        }),
        0
    );
    assert_eq!(radius(&GlassArea::Whole), 0);
}
