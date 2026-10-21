use sparky_xdg::desktop_entry::DesktopEntry;

use crate::domain::app::{App, Runs, app_of, command_line};

fn entrada(contents: &str) -> DesktopEntry {
    match DesktopEntry::parse(contents) {
        Some(entry) => entry,
        None => panic!("entrada inválida"),
    }
}

#[test]
fn le_nome_detalhe_palavras_e_comando_no_idioma() {
    let entry = entrada("[Desktop Entry]\nName=Files\nName[pt_BR]=Arquivos\nGenericName[pt_BR]=Gerenciador\nKeywords=pastas;\nExec=nautilus %U\n");
    let Some(app) = app_of("org.gnome.Nautilus", &entry, "pt_BR", "sparky") else { panic!("deveria virar app") };
    assert_eq!(app.name, "Arquivos");
    assert_eq!(app.detail.as_deref(), Some("Gerenciador"));
    assert_eq!(app.keywords, ["pastas"]);
    assert_eq!(app.command, "nautilus");
    assert_eq!(app.runs, Runs::Directly);
}

#[test]
fn escondidas_e_de_outros_desktops_ficam_de_fora() {
    assert!(app_of("a", &entrada("[Desktop Entry]\nName=A\nExec=a\nNoDisplay=true\n"), "pt_BR", "sparky").is_none());
    assert!(app_of("b", &entrada("[Desktop Entry]\nName=B\nExec=b\nOnlyShowIn=GNOME;\n"), "pt_BR", "sparky").is_none());
    assert!(app_of("c", &entrada("[Desktop Entry]\nName=C\n"), "pt_BR", "sparky").is_none());
}

#[test]
fn o_detalhe_que_repete_o_nome_some() {
    let entry = entrada("[Desktop Entry]\nName=Firefox\nGenericName=Firefox\nComment=Navegue\nExec=firefox\n");
    assert_eq!(app_of("firefox", &entry, "C", "sparky").and_then(|app| app.detail), None);
}

#[test]
fn app_de_terminal_abre_no_terminal_escolhido() {
    let app = App {
        id: "htop".to_owned(),
        name: "htop".to_owned(),
        detail: None,
        keywords: Vec::new(),
        command: "htop".to_owned(),
        runs: Runs::InTerminal,
    };
    assert_eq!(command_line(&app, "foot"), "foot -e htop");
    assert_eq!(command_line(&App { runs: Runs::Directly, ..app }, "foot"), "htop");
}
