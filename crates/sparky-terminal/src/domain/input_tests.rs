use crate::domain::input::{Bracketed, Buffer, Wheel, paste, wheel};

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
    assert_eq!(wheel(3.0, Buffer::Main), Wheel::History(3));
    assert_eq!(wheel(-2.0, Buffer::Main), Wheel::History(-2));
    assert_eq!(wheel(0.2, Buffer::Main), Wheel::History(0));
}

#[test]
fn em_tela_alternativa_a_roda_vira_setas() {
    assert_eq!(
        wheel(2.0, Buffer::Alternate),
        Wheel::Keys(b"\x1b[A\x1b[A".to_vec())
    );
    assert_eq!(
        wheel(-1.0, Buffer::Alternate),
        Wheel::Keys(b"\x1b[B".to_vec())
    );
}
