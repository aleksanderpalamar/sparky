use crate::domain::field::Field;
use crate::domain::report::{Entry, Report};
use crate::domain::status::{Reason, Status};
use crate::render::plain;

fn relatorio() -> Report {
    Report {
        entries: vec![
            Entry::new(Field::Os, Status::text("Arch Linux")),
            Entry::new(Field::Gpu, Status::unavailable(Reason::Missing)),
            Entry::new(Field::Packages, Status::TimedOut),
            Entry::new(
                Field::Monitors,
                Status::lines(vec!["DP-1 2560x1440".into(), "HDMI-A-1 1920x1080".into()]),
            ),
        ],
    }
}

#[test]
fn texto_simples_usa_rotulo_traduzido_e_na_para_falhas() {
    let texto = plain::render(&relatorio());
    assert_eq!(
        texto,
        "Sistema: Arch Linux\nGPU: N/A\nPacotes: N/A\nMonitores: DP-1 2560x1440, HDMI-A-1 1920x1080\n"
    );
}

use crate::config::settings::{ColorMode, Format, Settings};
use crate::render::view::{Terminal, View};

fn terminal(tty: bool, no_color: bool) -> Terminal {
    Terminal {
        tty,
        columns: Some(120),
        no_color,
    }
}

#[test]
fn no_terminal_sai_visual_e_colorido_e_no_pipe_sai_texto_simples() {
    let padrao = Settings::default();
    let tela = View::resolve(&padrao, &terminal(true, false));
    assert_eq!(
        (tela.format, tela.color, tela.width),
        (Format::Visual, true, 120)
    );
    let pipe = View::resolve(&padrao, &terminal(false, false));
    assert_eq!((pipe.format, pipe.color), (Format::Plain, false));
}

#[test]
fn no_color_e_a_escolha_explicita_mandam_na_cor() {
    let padrao = Settings::default();
    assert!(!View::resolve(&padrao, &terminal(true, true)).color);
    let sempre = Settings {
        color: ColorMode::Always,
        ..Settings::default()
    };
    assert!(View::resolve(&sempre, &terminal(false, true)).color);
}

#[test]
fn formato_e_largura_pedidos_valem_mesmo_no_pipe() {
    let pedido = Settings {
        format: Some(Format::Json),
        width: Some(50),
        ..Settings::default()
    };
    let visao = View::resolve(&pedido, &terminal(false, false));
    assert_eq!((visao.format, visao.width), (Format::Json, 50));
}
