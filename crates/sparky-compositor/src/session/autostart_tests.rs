use crate::session::autostart::{Candidate, commands};

fn arquivo(name: &str, contents: &str) -> Candidate {
    Candidate {
        name: name.to_owned(),
        contents: contents.to_owned(),
    }
}

fn entrada(exec: &str, extra: &str) -> String {
    format!("[Desktop Entry]\nType=Application\nExec={exec}\n{extra}")
}

fn tudo_instalado(_: &str) -> bool {
    true
}

#[test]
fn o_arquivo_do_usuario_vence_o_do_sistema_com_o_mesmo_nome() {
    let usuario = vec![arquivo("nm-applet.desktop", &entrada("nm-applet --indicator", ""))];
    let sistema = vec![arquivo("nm-applet.desktop", &entrada("nm-applet", ""))];
    assert_eq!(commands(&[usuario, sistema], "sparky", tudo_instalado), vec!["nm-applet --indicator"]);
}

#[test]
fn o_usuario_desliga_uma_entrada_do_sistema_com_hidden() {
    let usuario = vec![arquivo("nm-applet.desktop", &entrada("nm-applet", "Hidden=true\n"))];
    let sistema = vec![arquivo("nm-applet.desktop", &entrada("nm-applet", ""))];
    assert!(commands(&[usuario, sistema], "sparky", tudo_instalado).is_empty());
}

#[test]
fn respeita_only_show_in_e_not_show_in() {
    let sistema = vec![
        arquivo("a.desktop", &entrada("so-gnome", "OnlyShowIn=GNOME;\n")),
        arquivo("b.desktop", &entrada("com-sparky", "OnlyShowIn=GNOME;sparky;\n")),
        arquivo("c.desktop", &entrada("menos-sparky", "NotShowIn=sparky;\n")),
        arquivo("d.desktop", &entrada("em-todos", "NotShowIn=KDE;\n")),
    ];
    assert_eq!(commands(&[sistema], "sparky", tudo_instalado), vec!["com-sparky", "em-todos"]);
}

#[test]
fn try_exec_ausente_tira_a_entrada() {
    let sistema = vec![
        arquivo("a.desktop", &entrada("blueman-applet", "TryExec=blueman-applet\n")),
        arquivo("b.desktop", &entrada("nm-applet", "TryExec=nm-applet\n")),
    ];
    let instalado = |programa: &str| programa == "nm-applet";
    assert_eq!(commands(&[sistema], "sparky", instalado), vec!["nm-applet"]);
}

#[test]
fn desligada_pelo_gnome_ou_sem_exec_fica_de_fora() {
    let sistema = vec![
        arquivo("a.desktop", &entrada("tracker", "X-GNOME-Autostart-enabled=false\n")),
        arquivo("b.desktop", "[Desktop Entry]\nType=Application\n"),
        arquivo("c.desktop", "nem é uma entrada"),
    ];
    assert!(commands(&[sistema], "sparky", tudo_instalado).is_empty());
}

#[test]
fn a_ordem_segue_o_nome_do_arquivo() {
    let usuario = vec![arquivo("z.desktop", &entrada("zeta", ""))];
    let sistema = vec![arquivo("a.desktop", &entrada("alfa", ""))];
    assert_eq!(commands(&[usuario, sistema], "sparky", tudo_instalado), vec!["alfa", "zeta"]);
}
