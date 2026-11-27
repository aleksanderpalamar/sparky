use sparky_tiling::{Fullscreen, Layer, Placement, Rect, WindowId};

use super::{Cover, Stacking, Stratum, strata};

fn placement(layer: Layer) -> Placement {
    Placement {
        window: WindowId(1),
        frame: Rect::new(0, 0, 800, 600),
        layer,
        focused: true,
    }
}

fn stacking(layers: &[(&str, Layer)]) -> Stacking {
    let arrangement: Vec<(String, Vec<Placement>)> = layers
        .iter()
        .map(|(name, layer)| (name.to_string(), vec![placement(*layer)]))
        .collect();
    let mut stacking = Stacking::default();
    stacking.refresh(&arrangement);
    stacking
}

#[test]
fn tela_cheia_tira_a_barra_do_caminho_mas_nao_o_overlay() {
    let order = strata(Cover::Fullscreen);
    assert!(!order.contains(&Stratum::Top));
    assert_eq!(order.first(), Some(&Stratum::Overlay));
}

#[test]
fn sem_tela_cheia_a_barra_fica_acima_das_janelas() {
    let order = strata(Cover::Open);
    let top = order.iter().position(|stratum| *stratum == Stratum::Top);
    let windows = order
        .iter()
        .position(|stratum| *stratum == Stratum::Windows);
    assert!(top < windows);
}

#[test]
fn so_o_monitor_com_tela_cheia_fica_coberto() {
    let stacking = stacking(&[
        ("A", Layer::Fullscreen(Fullscreen::Monitor)),
        ("B", Layer::Tiled),
    ]);
    assert_eq!(stacking.cover("A"), Cover::Fullscreen);
    assert_eq!(stacking.cover("B"), Cover::Open);
}

#[test]
fn maximizado_nao_cobre_a_barra() {
    let stacking = stacking(&[("A", Layer::Fullscreen(Fullscreen::Maximized))]);
    assert_eq!(stacking.cover("A"), Cover::Open);
}

#[test]
fn saida_da_tela_cheia_descobre_o_monitor() {
    let mut stacking = stacking(&[("A", Layer::Fullscreen(Fullscreen::Monitor))]);
    stacking.refresh(&[("A".to_string(), vec![placement(Layer::Tiled)])]);
    assert_eq!(stacking.cover("A"), Cover::Open);
}
