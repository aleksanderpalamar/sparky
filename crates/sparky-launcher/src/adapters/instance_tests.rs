use std::path::Path;
use std::thread;

use crate::adapters::instance::{Claim, claim, socket};
use crate::domain::mode::Mode;

#[test]
fn o_socket_separa_lancador_e_menu_do_sistema_por_display() {
    let pasta = Path::new("/run/user/1000");
    assert_eq!(
        socket(pasta, "wayland-1", Mode::Apps),
        pasta.join("sparky-launcher-apps-wayland-1.sock")
    );
    assert_eq!(
        socket(pasta, "wayland-1", Mode::System),
        pasta.join("sparky-launcher-system-wayland-1.sock")
    );
}

#[test]
fn a_segunda_abertura_pede_para_a_primeira_fechar() {
    let dir = tempfile::tempdir().unwrap();
    let caminho = dir.path().join("launcher.sock");
    let Ok(Claim::Owner(primeira)) = claim(&caminho) else {
        panic!("a primeira abertura fica com o socket");
    };
    let espera = thread::spawn(move || primeira.wait());
    assert!(matches!(claim(&caminho), Ok(Claim::Signalled)));
    assert!(espera.join().unwrap().is_ok());
}

#[test]
fn socket_de_uma_abertura_que_ja_fechou_e_substituido() {
    let dir = tempfile::tempdir().unwrap();
    let caminho = dir.path().join("launcher.sock");
    {
        let Ok(Claim::Owner(_antiga)) = claim(&caminho) else {
            panic!("deveria abrir");
        };
    }
    assert!(caminho.exists());
    assert!(matches!(claim(&caminho), Ok(Claim::Owner(_))));
}
