use crate::desktop_entry::{DesktopEntry, command_of};

const NM_APPLET: &str = "[Desktop Entry]
Name=Network
Name[pt_BR]=Rede
Exec=nm-applet --indicator
TryExec=nm-applet
NotShowIn=KDE;GNOME;
Type=Application

[Desktop Action Nova]
Exec=nm-connection-editor
";

#[test]
fn le_os_campos_do_grupo_principal() {
    let Some(entry) = DesktopEntry::parse(NM_APPLET) else {
        panic!("deveria ler a entrada");
    };
    assert_eq!(entry.exec.as_deref(), Some("nm-applet --indicator"));
    assert_eq!(entry.try_exec.as_deref(), Some("nm-applet"));
    assert_eq!(entry.not_show_in, vec!["KDE", "GNOME"]);
    assert!(entry.only_show_in.is_empty());
    assert!(!entry.hidden);
    assert!(entry.enabled);
}

#[test]
fn ignora_as_chaves_de_outros_grupos() {
    let Some(entry) = DesktopEntry::parse(NM_APPLET) else {
        panic!("deveria ler a entrada");
    };
    assert_eq!(entry.exec.as_deref(), Some("nm-applet --indicator"));
}

#[test]
fn le_oculta_e_desligada_pelo_gnome() {
    let contents = "[Desktop Entry]\nExec=x\nHidden=true\nX-GNOME-Autostart-enabled=false\nType=Application\n";
    let Some(entry) = DesktopEntry::parse(contents) else {
        panic!("deveria ler a entrada");
    };
    assert!(entry.hidden);
    assert!(!entry.enabled);
}

#[test]
fn so_aceita_entrada_do_tipo_aplicativo() {
    assert!(DesktopEntry::parse("[Desktop Entry]\nExec=x\nType=Link\n").is_none());
    assert!(DesktopEntry::parse("[Desktop Entry]\nExec=x\n").is_some());
}

#[test]
fn arquivo_sem_grupo_principal_nao_e_entrada() {
    assert!(DesktopEntry::parse("Exec=x\n").is_none());
    assert!(DesktopEntry::parse("").is_none());
}

#[test]
fn ignora_comentarios_e_linhas_soltas() {
    let contents = "# comentário\n[Desktop Entry]\n\nlixo\nExec=x\n";
    let Some(entry) = DesktopEntry::parse(contents) else {
        panic!("deveria ler a entrada");
    };
    assert_eq!(entry.exec.as_deref(), Some("x"));
}

#[test]
fn tira_os_codigos_de_campo_do_exec() {
    assert_eq!(command_of("firefox %U"), "firefox");
    assert_eq!(command_of("app --file=%f --name %c"), "app --file= --name");
    assert_eq!(command_of("echo 100%%"), "echo 100%");
}
