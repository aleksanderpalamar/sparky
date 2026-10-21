use crate::desktop_entry::{DesktopEntry, command_of, list_of, locale_of};

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

const FIREFOX: &str = "[Desktop Entry]
Name=Firefox
Name[pt_BR]=Navegador Firefox
Name[pt]=Firefox PT
GenericName=Web Browser
GenericName[pt]=Navegador
Comment=Browse the Web
Keywords=internet;www;
Keywords[pt_BR]=internet;navegador;web;
Icon=firefox
Exec=firefox %u
Terminal=false
Type=Application
";

#[test]
fn o_nome_segue_o_idioma_e_cai_para_a_lingua_e_depois_para_o_padrao() {
    let Some(entry) = DesktopEntry::parse(FIREFOX) else { panic!("deveria ler") };
    assert_eq!(entry.name.get("pt_BR"), Some("Navegador Firefox"));
    assert_eq!(entry.name.get("pt_PT"), Some("Firefox PT"));
    assert_eq!(entry.name.get("de_DE"), Some("Firefox"));
    assert_eq!(entry.generic_name.get("pt_BR"), Some("Navegador"));
    assert_eq!(entry.comment.get("pt_BR"), Some("Browse the Web"));
}

#[test]
fn le_icone_palavras_chave_e_as_marcas_de_exibicao() {
    let Some(entry) = DesktopEntry::parse(FIREFOX) else { panic!("deveria ler") };
    assert_eq!(entry.icon.as_deref(), Some("firefox"));
    assert_eq!(list_of(entry.keywords.get("pt_BR")), vec!["internet", "navegador", "web"]);
    assert!(!entry.no_display);
    assert!(!entry.terminal);
    let Some(htop) = DesktopEntry::parse("[Desktop Entry]\nName=htop\nExec=htop\nTerminal=true\nNoDisplay=true\n") else {
        panic!("deveria ler");
    };
    assert!(htop.terminal);
    assert!(htop.no_display);
}

#[test]
fn o_idioma_vem_do_ambiente_sem_a_codificacao() {
    assert_eq!(locale_of("pt_BR.UTF-8"), "pt_BR");
    assert_eq!(locale_of("pt_BR.UTF-8@euro"), "pt_BR");
    assert_eq!(locale_of("C"), "C");
}

#[test]
fn a_entrada_aparece_conforme_only_e_not_show_in() {
    let Some(entry) = DesktopEntry::parse(NM_APPLET) else { panic!("deveria ler") };
    assert!(entry.shown_in("sparky"));
    assert!(!entry.shown_in("GNOME"));
    let Some(so_sparky) = DesktopEntry::parse("[Desktop Entry]\nExec=x\nOnlyShowIn=sparky;\n") else { panic!("deveria ler") };
    assert!(so_sparky.shown_in("sparky"));
    assert!(!so_sparky.shown_in("KDE"));
    let Some(escondida) = DesktopEntry::parse("[Desktop Entry]\nExec=x\nHidden=true\n") else { panic!("deveria ler") };
    assert!(!escondida.shown_in("sparky"));
}

#[test]
fn try_exec_ausente_tira_a_entrada() {
    let Some(entry) = DesktopEntry::parse(NM_APPLET) else { panic!("deveria ler") };
    assert!(entry.installed(|program| program == "nm-applet"));
    assert!(!entry.installed(|_| false));
}
