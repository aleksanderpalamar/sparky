use std::path::PathBuf;

use crate::wallpaper::choice::Choice;
use crate::wallpaper::{Decoded, Settled, Wallpaper, Wanted};

fn lago() -> PathBuf {
    PathBuf::from("/fundos/lago.webp")
}

fn monitores() -> Vec<(String, (u32, u32))> {
    vec![("DP-1".to_owned(), (4, 2)), ("HDMI-A-1".to_owned(), (2, 2))]
}

fn pronto(wanted: Wanted) -> Decoded {
    let (largura, altura) = wanted.size;
    Decoded { pixels: Ok(vec![255; (largura * altura * 4) as usize]), wanted }
}

#[test]
fn cada_monitor_pede_a_imagem_no_proprio_tamanho_uma_vez() {
    let mut fundo = Wallpaper::default();
    let pedidos = fundo.plan(Choice::File(lago()), &monitores());
    assert_eq!(pedidos.iter().map(|pedido| (pedido.output.as_str(), pedido.size)).collect::<Vec<_>>(), [("DP-1", (4, 2)), ("HDMI-A-1", (2, 2))]);
    assert!(fundo.plan(Choice::File(lago()), &monitores()).is_empty());
    for pedido in pedidos {
        assert_eq!(fundo.settle(pronto(pedido)), Settled::Painted);
    }
    assert!(fundo.buffer("DP-1").is_some());
    assert!(fundo.plan(Choice::File(lago()), &monitores()).is_empty());
}

#[test]
fn mudar_o_tamanho_do_monitor_pede_de_novo_so_para_ele() {
    let mut fundo = Wallpaper::default();
    for pedido in fundo.plan(Choice::File(lago()), &monitores()) {
        fundo.settle(pronto(pedido));
    }
    let maior = vec![("DP-1".to_owned(), (8, 4)), ("HDMI-A-1".to_owned(), (2, 2))];
    let pedidos = fundo.plan(Choice::File(lago()), &maior);
    assert_eq!(pedidos.len(), 1);
    assert!(fundo.buffer("DP-1").is_some());
}

#[test]
fn imagem_de_uma_escolha_antiga_e_descartada() {
    let mut fundo = Wallpaper::default();
    let pedidos = fundo.plan(Choice::File(lago()), &monitores());
    fundo.plan(Choice::File(PathBuf::from("/fundos/outro.jpg")), &monitores());
    let Some(antigo) = pedidos.into_iter().next() else { panic!("sem pedido") };
    assert_eq!(fundo.settle(pronto(antigo)), Settled::Stale);
    assert!(fundo.buffer("DP-1").is_none());
}

#[test]
fn desligar_tira_o_fundo_e_falha_limpa_o_monitor() {
    let mut fundo = Wallpaper::default();
    let pedidos = fundo.plan(Choice::File(lago()), &monitores());
    let Some(primeiro) = pedidos.first().cloned() else { panic!("sem pedido") };
    fundo.settle(pronto(primeiro.clone()));
    assert!(fundo.plan(Choice::Off, &monitores()).is_empty());
    assert!(fundo.buffer("DP-1").is_none());
    fundo.plan(Choice::File(lago()), &monitores());
    let falha = fundo.settle(Decoded { wanted: primeiro, pixels: Err("formato desconhecido".to_owned()) });
    assert_eq!(falha, Settled::Failed("formato desconhecido".to_owned()));
}

#[test]
fn monitor_desligado_deixa_de_ter_fundo() {
    let mut fundo = Wallpaper::default();
    for pedido in fundo.plan(Choice::File(lago()), &monitores()) {
        fundo.settle(pronto(pedido));
    }
    fundo.plan(Choice::File(lago()), &monitores()[..1]);
    assert!(fundo.buffer("HDMI-A-1").is_none());
    assert!(fundo.buffer("DP-1").is_some());
}
