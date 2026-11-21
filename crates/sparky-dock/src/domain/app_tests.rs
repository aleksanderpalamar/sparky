use sparky_xdg::desktop_entry::DesktopEntry;

use super::app::App;

fn entry(body: &str) -> DesktopEntry {
    let contents = format!("[Desktop Entry]\nType=Application\n{body}");
    match DesktopEntry::parse(&contents) {
        Some(entry) => entry,
        None => panic!("entrada inválida no teste"),
    }
}

#[test]
fn le_nome_traduzido_icone_comando_e_classe() {
    let found = App::of(
        "sparky-files.desktop",
        &entry(
            "Name=Files\nName[pt_BR]=Arquivos\nIcon=folder\nExec=sparky-files %U\nStartupWMClass=Files\n",
        ),
        "pt_BR",
    );
    let Some(app) = found else {
        panic!("deveria virar aplicativo");
    };
    assert_eq!(app.id, "sparky-files");
    assert_eq!(app.name, "Arquivos");
    assert_eq!(app.icon.as_deref(), Some("folder"));
    assert_eq!(app.launch.arguments, vec!["sparky-files"]);
    assert_eq!(app.window_class.as_deref(), Some("Files"));
}

#[test]
fn ignora_entradas_ocultas_de_outro_desktop_ou_sem_comando() {
    assert!(App::of("a", &entry("Name=A\nExec=a\nNoDisplay=true\n"), "en").is_none());
    assert!(App::of("a", &entry("Name=A\nExec=a\nOnlyShowIn=KDE;\n"), "en").is_none());
    assert!(App::of("a", &entry("Name=A\n"), "en").is_none());
    assert!(App::of("a", &entry("Name= \nExec=a\n"), "en").is_none());
    assert!(App::of("a", &entry("Name=A\nExec=a\nOnlyShowIn=sparky;\n"), "en").is_some());
}
