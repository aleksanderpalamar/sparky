use smithay::utils::{Rectangle, Size};

use super::clip::rounding;

#[test]
fn sem_raio_o_desfoque_fica_reto() {
    let painel = Some(Rectangle::new((40, 40).into(), (408, 100).into()));
    assert!(rounding(painel, Size::from((488.0, 180.0)), 0.0).is_none());
    assert!(rounding(None, Size::from((488.0, 180.0)), 12.0).is_none());
}

#[test]
fn com_raio_o_recorte_segue_o_painel_e_nao_passa_da_metade_do_lado() {
    let painel = Some(Rectangle::new((40, 40).into(), (408, 100).into()));
    let Some(uniforms) = rounding(painel, Size::from((488.0, 180.0)), 80.0) else {
        panic!("deveria arredondar");
    };
    assert_eq!(uniforms.len(), 3);
    assert!(rounding(painel, Size::from((488.0, 180.0)), 12.0).is_some());
}
