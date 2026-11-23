use smithay::utils::{Rectangle, Size};

use super::clip::{pieces, rounding};

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

#[test]
fn cada_retangulo_desenha_com_o_proprio_arredondamento() {
    let region = [
        Rectangle::new((40, 40).into(), (100, 60).into()),
        Rectangle::new((300, 40).into(), (100, 60).into()),
    ];
    let visible = Rectangle::new((0.0, 0.0).into(), (440.0, 140.0).into());
    let dst = Rectangle::new((0, 0).into(), (440, 140).into());
    let damage = [dst];
    let pecas = pieces(
        &region,
        visible,
        dst,
        &damage,
        Size::from((440.0, 140.0)),
        8.0,
    );
    assert_eq!(pecas.len(), 2);
    assert_eq!(
        pecas[0].clip,
        vec![Rectangle::new((40, 40).into(), (100, 60).into())]
    );
    assert_eq!(
        pecas[1].clip,
        vec![Rectangle::new((300, 40).into(), (100, 60).into())]
    );
    assert!(pecas.iter().all(|peca| peca.rounding.is_some()));
}

#[test]
fn retangulo_fora_do_dano_nao_desenha() {
    let region = [
        Rectangle::new((40, 40).into(), (100, 60).into()),
        Rectangle::new((300, 40).into(), (100, 60).into()),
    ];
    let visible = Rectangle::new((0.0, 0.0).into(), (440.0, 140.0).into());
    let dst = Rectangle::new((0, 0).into(), (440, 140).into());
    let damage = [Rectangle::new((0, 0).into(), (200, 140).into())];
    let pecas = pieces(
        &region,
        visible,
        dst,
        &damage,
        Size::from((440.0, 140.0)),
        8.0,
    );
    assert_eq!(pecas.len(), 1);
}
