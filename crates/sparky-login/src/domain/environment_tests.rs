use std::path::PathBuf;

use sparky_login_core::sessions::Session;

use crate::domain::environment::{Account, Class, environment, login_command, pam_variables, tty};

fn par(name: &str, value: &str) -> (String, String) {
    (name.to_owned(), value.to_owned())
}

fn conta(shell: &str) -> Account {
    Account {
        name: "palamar".into(),
        uid: 1000,
        gid: 1000,
        home: PathBuf::from("/home/palamar"),
        shell: PathBuf::from(shell),
    }
}

fn sessao(id: &str, names: &[&str]) -> Session {
    Session {
        id: id.into(),
        name: id.into(),
        exec: id.into(),
        desktop_names: names.iter().map(|name| (*name).to_owned()).collect(),
    }
}

#[test]
fn a_sessao_do_usuario_se_registra_no_logind_como_wayland_no_terminal_certo() {
    let variaveis = pam_variables(Class::User, 2, Some(&sessao("hyprland", &["Hyprland"])));
    for esperado in [
        par("XDG_SESSION_TYPE", "wayland"),
        par("XDG_SESSION_CLASS", "user"),
        par("XDG_SEAT", "seat0"),
        par("XDG_VTNR", "2"),
        par("XDG_SESSION_DESKTOP", "hyprland"),
        par("XDG_CURRENT_DESKTOP", "Hyprland"),
    ] {
        assert!(variaveis.contains(&esperado), "{esperado:?}");
    }
}

#[test]
fn a_tela_de_login_entra_como_greeter_sem_desktop() {
    let variaveis = pam_variables(Class::Greeter, 1, None);
    assert!(variaveis.contains(&par("XDG_SESSION_CLASS", "greeter")));
    assert!(
        !variaveis
            .iter()
            .any(|(name, _)| name.starts_with("XDG_SESSION_DESKTOP")
                || name == "XDG_CURRENT_DESKTOP")
    );
}

#[test]
fn varios_desktop_names_viram_lista_com_dois_pontos_e_nenhum_nao_define_nada() {
    let varios = pam_variables(
        Class::User,
        1,
        Some(&sessao("ubuntu", &["ubuntu", "GNOME"])),
    );
    assert!(varios.contains(&par("XDG_CURRENT_DESKTOP", "ubuntu:GNOME")));
    let nenhum = pam_variables(Class::User, 1, Some(&sessao("sparky", &[])));
    assert!(!nenhum.iter().any(|(name, _)| name == "XDG_CURRENT_DESKTOP"));
}

#[test]
fn o_terminal_e_o_do_numero_pedido() {
    assert_eq!(tty(1), PathBuf::from("/dev/tty1"));
    assert_eq!(tty(7), PathBuf::from("/dev/tty7"));
}

#[test]
fn o_ambiente_tem_a_conta_e_o_que_o_pam_definiu_por_cima() {
    let pam = vec![
        par("PATH", "/opt/bin:/usr/bin"),
        par("XDG_RUNTIME_DIR", "/run/user/1000"),
    ];
    let ambiente = environment(&conta("/usr/bin/zsh"), pam);
    for esperado in [
        par("HOME", "/home/palamar"),
        par("USER", "palamar"),
        par("LOGNAME", "palamar"),
        par("SHELL", "/usr/bin/zsh"),
        par("PATH", "/opt/bin:/usr/bin"),
        par("XDG_RUNTIME_DIR", "/run/user/1000"),
    ] {
        assert!(ambiente.contains(&esperado), "{esperado:?}");
    }
    assert_eq!(
        ambiente.iter().filter(|(name, _)| name == "PATH").count(),
        1
    );
}

#[test]
fn sem_path_do_pam_vale_o_padrao() {
    let ambiente = environment(&conta("/bin/bash"), Vec::new());
    assert!(ambiente.contains(&par("PATH", "/usr/local/bin:/usr/bin:/bin")));
}

#[test]
fn a_sessao_roda_pelo_shell_de_login_do_usuario() {
    assert_eq!(
        login_command(&conta("/usr/bin/zsh"), "Hyprland --config x"),
        vec!["/usr/bin/zsh", "-l", "-c", "exec Hyprland --config x"]
    );
    assert_eq!(
        login_command(&conta(""), "sparky"),
        vec!["/bin/sh", "-l", "-c", "exec sparky"]
    );
}
