use sparky_theme::Rgb;

use crate::config::settings::{Colors, Layout, Logo};
use crate::domain::field::Field;
use crate::domain::report::{Entry, Report};
use crate::domain::status::{Status, Value};
use crate::render::view::View;
use crate::render::visual::fixture::{ACCENT, monitors, report, view};
use crate::render::visual::{ansi, render};

fn visivel(texto: &str) -> String {
    texto.replace('\x1b', "\\e")
}

fn com(layout: Layout, logo: Logo, width: usize) -> View {
    View {
        layout,
        logo,
        ..view(width, false)
    }
}

#[test]
fn classico_sem_cor_em_80_colunas() {
    assert_eq!(
        render(&report(), &view(80, false)),
        include_str!("snapshots/classic.txt")
    );
}

#[test]
fn classico_com_cor_pinta_logotipo_rotulos_e_amostras() {
    assert_eq!(
        visivel(&render(&report(), &view(80, true))),
        include_str!("snapshots/classic_color.txt")
    );
}

#[test]
fn empilhado_poe_o_logotipo_acima() {
    assert_eq!(
        render(&report(), &com(Layout::Stacked, Logo::Show, 80)),
        include_str!("snapshots/stacked.txt")
    );
}

#[test]
fn largura_estreita_cai_para_o_empilhado_e_corta_valores() {
    assert_eq!(
        render(&report(), &view(40, false)),
        include_str!("snapshots/narrow.txt")
    );
}

#[test]
fn largura_minima_esconde_o_logotipo_e_corta_tudo() {
    assert_eq!(
        render(&report(), &view(20, false)),
        include_str!("snapshots/tiny.txt")
    );
}

#[test]
fn sem_logotipo_as_linhas_continuam_alinhadas() {
    let relatorio = Report {
        entries: vec![
            Entry::new(Field::Os, Status::text("Arch Linux")),
            Entry::new(Field::Monitors, monitors()),
            Entry::new(
                Field::Gpu,
                Status::lines(vec!["Radeon".into(), "Intel".into()]),
            ),
        ],
    };
    assert_eq!(
        render(&relatorio, &com(Layout::Classic, Logo::Hide, 80)),
        include_str!("snapshots/lines.txt")
    );
}

#[test]
fn cores_pedidas_trocam_destaque_rotulo_e_valor() {
    let relatorio = Report {
        entries: vec![
            Entry::new(Field::Os, Status::text("Arch Linux")),
            Entry::new(
                Field::Colors,
                Status::ready(Value::Colors(vec![ACCENT, Rgb::new(1, 2, 3)])),
            ),
        ],
    };
    let visao = View {
        colors: Colors {
            accent: Some(Rgb::new(255, 0, 128)),
            label: Some(Rgb::new(0, 255, 0)),
            value: Some(Rgb::new(9, 9, 9)),
        },
        ..view(80, true)
    };
    assert_eq!(
        visivel(&render(&relatorio, &visao)),
        include_str!("snapshots/overrides.txt")
    );
}

#[test]
fn nenhuma_linha_passa_da_largura_e_sem_cor_nao_ha_escapes() {
    for width in 0..=100 {
        let colorida = render(&report(), &view(width, true));
        assert!(colorida.lines().all(|linha| ansi::width(linha) <= width));
        let simples = render(&report(), &view(width, false));
        assert!(simples.lines().all(|linha| ansi::width(linha) <= width));
        assert!(!simples.contains('\x1b'));
    }
}

#[test]
fn nenhuma_linha_termina_com_espaco() {
    let saida = render(&report(), &view(80, true));
    assert!(saida.lines().all(|linha| linha == linha.trim_end()));
}
