use crate::domain::input::{
    Bracketed, Buffer, Focus, Reporting, Wheel, focus_report, paste, wheel,
};

#[test]
fn colar_troca_quebras_de_linha_por_enter() {
    assert_eq!(paste("a\nb", Bracketed::No), b"a\rb".to_vec());
    assert_eq!(paste("a\r\nb", Bracketed::No), b"a\rb".to_vec());
}

#[test]
fn colar_entre_marcas_sem_deixar_o_texto_fechar_o_bloco() {
    assert_eq!(
        paste("ls", Bracketed::Yes),
        b"\x1b[200~ls\x1b[201~".to_vec()
    );
    assert_eq!(
        paste("x\x1b[201~rm -rf ~\n", Bracketed::Yes),
        b"\x1b[200~x[201~rm -rf ~\r\x1b[201~".to_vec()
    );
}

#[test]
fn a_roda_rola_o_historico_no_shell() {
    assert_eq!(wheel(3.0, 0.0, Buffer::Main).0, Wheel::History(3));
    assert_eq!(wheel(-2.0, 0.0, Buffer::Main).0, Wheel::History(-2));
}

#[test]
fn o_touchpad_acumula_as_fracoes_ate_dar_uma_linha() {
    let (primeira, resto) = wheel(0.4, 0.0, Buffer::Main);
    assert_eq!(primeira, Wheel::History(0));
    let (segunda, resto) = wheel(0.4, resto, Buffer::Main);
    assert_eq!(segunda, Wheel::History(0));
    let (terceira, resto) = wheel(0.4, resto, Buffer::Main);
    assert_eq!(terceira, Wheel::History(1));
    assert!((resto - 0.2).abs() < 0.001);
    let (descendo, resto) = wheel(-0.7, 0.0, Buffer::Main);
    assert_eq!(descendo, Wheel::History(0));
    assert_eq!(wheel(-0.7, resto, Buffer::Main).0, Wheel::History(-1));
}

#[test]
fn em_tela_alternativa_a_roda_vira_setas() {
    assert_eq!(
        wheel(2.0, 0.0, Buffer::Alternate).0,
        Wheel::Keys(b"\x1b[A\x1b[A".to_vec())
    );
    assert_eq!(
        wheel(-1.0, 0.0, Buffer::Alternate).0,
        Wheel::Keys(b"\x1b[B".to_vec())
    );
}

#[test]
fn o_foco_so_e_avisado_quando_o_programa_pede() {
    assert_eq!(
        focus_report(Focus::Gained, Reporting::On),
        Some(b"\x1b[I".as_slice())
    );
    assert_eq!(
        focus_report(Focus::Lost, Reporting::On),
        Some(b"\x1b[O".as_slice())
    );
    assert_eq!(focus_report(Focus::Gained, Reporting::Off), None);
}
