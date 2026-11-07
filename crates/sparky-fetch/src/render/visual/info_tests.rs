use crate::domain::field::Field;
use crate::domain::report::{Entry, Report};
use crate::domain::status::{Reason, Status};
use crate::render::visual::fixture::{monitors, report};
use crate::render::visual::info;
use crate::render::visual::segment::{Line, Segment, Tone};
use crate::render::visual::style::Paint;

fn texto(linhas: &[Line]) -> Vec<String> {
    linhas
        .iter()
        .map(|linha| Paint::Plain.line(linha))
        .collect()
}

#[test]
fn alinha_os_valores_numa_coluna_e_mostra_na_nas_falhas() {
    let relatorio = Report {
        entries: vec![
            Entry::new(Field::Os, Status::text("Arch Linux")),
            Entry::new(Field::Uptime, Status::text("3 h")),
            Entry::new(Field::Gpu, Status::unavailable(Reason::Missing)),
            Entry::new(Field::Disk, Status::failed("negado")),
        ],
    };
    assert_eq!(
        texto(&info::rows(&relatorio)),
        [
            "Sistema:      Arch Linux",
            "Tempo ligado: 3 h",
            "GPU:          N/A",
            "Disco:        N/A",
        ]
    );
}

#[test]
fn varias_linhas_continuam_alinhadas_sob_o_primeiro_valor() {
    let relatorio = Report {
        entries: vec![Entry::new(Field::Monitors, monitors())],
    };
    let linhas = info::rows(&relatorio);
    assert_eq!(
        texto(&linhas),
        [
            "Monitores: DP-1 2560x1440 @ 165 Hz",
            "           HDMI-A-1 1920x1080 @ 60 Hz",
        ]
    );
    assert_eq!(
        linhas[0].segments[0],
        Segment::new("Monitores:", Tone::Label)
    );
    assert_eq!(linhas[1].segments[0].tone, Tone::Plain);
}

#[test]
fn valores_perdem_escapes_e_controles() {
    let relatorio = Report {
        entries: vec![Entry::new(Field::Shell, Status::text("\x1b[31mzsh\n"))],
    };
    assert_eq!(texto(&info::rows(&relatorio)), ["Shell: zsh"]);
}

#[test]
fn as_amostras_vem_depois_de_uma_linha_em_branco() {
    let bloco = texto(&info::block(&report(), &Paint::Plain, 80));
    assert_eq!(bloco.len(), 13);
    assert_eq!(bloco[11], "");
    assert_eq!(bloco[12], "#f7768e #9ece6a #e0af68 #7aa2f7");
    assert!(!bloco.iter().any(|linha| linha.starts_with("Cores")));
}

#[test]
fn sem_espaco_para_amostras_nao_sobram_linhas_em_branco() {
    let bloco = texto(&info::block(&report(), &Paint::Plain, 6));
    assert_eq!(bloco.len(), 11);
    assert_eq!(
        bloco.last().map(String::as_str),
        Some("Tema:         Tokyo Night")
    );
}

#[test]
fn cores_indisponiveis_viram_uma_linha_comum() {
    let relatorio = Report {
        entries: vec![Entry::new(Field::Colors, Status::TimedOut)],
    };
    assert_eq!(
        texto(&info::block(&relatorio, &Paint::Plain, 80)),
        ["Cores: N/A"]
    );
}
