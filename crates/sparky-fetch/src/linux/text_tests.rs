use crate::domain::status::{Reason, Status, Value};
use crate::linux::fake::Fake;
use crate::linux::{gtk, kernel, os, shell, terminal, uptime};

fn text(value: &str) -> Result<Value, Status> {
    Ok(Value::Text(value.to_owned()))
}

fn missing() -> Result<Value, Status> {
    Err(Status::unavailable(Reason::Missing))
}

fn failed(result: Result<Value, Status>) -> bool {
    matches!(result, Err(Status::Failed { .. }))
}

#[test]
fn sistema_usa_o_nome_bonito_com_aspas_e_escapes() {
    let release = "NAME=Arch\nPRETTY_NAME=\"Arch \\\"Linux\\\"\"\nID=arch\n";
    assert_eq!(os::name(release), "Arch \"Linux\"");
    assert_eq!(os::name("NAME='Debian'\n"), "Debian");
    assert_eq!(os::name("ID=x\nPRETTY_NAME=\n"), "Linux");
}

#[test]
fn sistema_cai_para_usr_lib_e_falta_quando_nenhum_existe() {
    let fallback = Fake::default().file("/usr/lib/os-release", "PRETTY_NAME=Fedora\n");
    assert_eq!(os::collect(&fallback), text("Fedora"));
    assert_eq!(os::collect(&Fake::default()), missing());
    assert!(failed(os::collect(
        &Fake::default().broken("/etc/os-release")
    )));
}

#[test]
fn kernel_junta_tipo_e_versao() {
    let host = Fake::default()
        .file("/proc/sys/kernel/ostype", "Linux\n")
        .file("/proc/sys/kernel/osrelease", "6.17.1-arch1-1\n");
    assert_eq!(kernel::collect(&host), text("Linux 6.17.1-arch1-1"));
    assert_eq!(kernel::collect(&Fake::default()), missing());
}

#[test]
fn tempo_ligado_usa_a_parte_inteira_dos_segundos() {
    assert_eq!(uptime::seconds("187440.93 1234.00\n"), Some(187_440));
    assert_eq!(uptime::seconds("42 1"), Some(42));
    assert_eq!(uptime::seconds("x 1"), None);
    let host = Fake::default().file("/proc/uptime", "11040.5 9.0\n");
    assert_eq!(uptime::collect(&host), text("3h 4m"));
    assert!(failed(uptime::collect(
        &Fake::default().file("/proc/uptime", "")
    )));
}

#[test]
fn shell_e_o_nome_do_executavel() {
    assert_eq!(shell::name("/usr/bin/zsh").as_deref(), Some("zsh"));
    assert_eq!(shell::name("/"), None);
    let host = Fake::default().var("SHELL", "/bin/fish");
    assert_eq!(shell::collect(&host), text("fish"));
    assert_eq!(shell::collect(&Fake::default()), missing());
}

#[test]
fn processo_pai_e_lido_depois_do_nome_entre_parenteses() {
    assert_eq!(terminal::parent("42 (a (b) c) S 7 42 42"), Some(7));
    assert_eq!(terminal::parent("42 (zsh) S"), None);
}

#[test]
fn terminal_vem_da_variavel_ou_do_primeiro_ancestral_que_nao_e_shell() {
    let named = Fake::default().var("TERM_PROGRAM", "WezTerm");
    assert_eq!(terminal::collect(&named), text("WezTerm"));
    let chain = Fake::default()
        .file("/proc/self/stat", "90 (sparky-fetch) R 80 1 1")
        .file("/proc/80/comm", "-zsh\n")
        .file("/proc/80/stat", "80 (zsh) S 70 1 1")
        .file("/proc/70/comm", "sparky-terminal\n");
    assert_eq!(terminal::collect(&chain), text("sparky-terminal"));
}

#[test]
fn terminal_falta_quando_a_cadeia_chega_ao_init() {
    let chain = Fake::default()
        .file("/proc/self/stat", "90 (sparky-fetch) R 80 1 1")
        .file("/proc/80/comm", "bash\n")
        .file("/proc/80/stat", "80 (bash) S 1 1 1");
    assert_eq!(terminal::collect(&chain), missing());
    let broken = Fake::default().file("/proc/self/stat", "lixo");
    assert!(failed(terminal::collect(&broken)));
}

#[test]
fn configuracao_gtk_le_apenas_a_secao_settings() {
    let ini = "[Outra]\ngtk-icon-theme-name=X\n[Settings]\ngtk-icon-theme-name = Papirus \ngtk-font-name=\"Cantarell 11\"\n";
    assert_eq!(
        gtk::setting(ini, "gtk-icon-theme-name").as_deref(),
        Some("Papirus")
    );
    assert_eq!(
        gtk::setting(ini, "gtk-font-name").as_deref(),
        Some("Cantarell 11")
    );
    assert_eq!(gtk::setting("gtk-font-name=Sans\n", "gtk-font-name"), None);
}

#[test]
fn icones_usam_xdg_config_home_e_depois_o_gtk4() {
    let xdg = Fake::default().var("XDG_CONFIG_HOME", "/cfg").file(
        "/cfg/gtk-3.0/settings.ini",
        "[Settings]\ngtk-icon-theme-name=Yaru-blue\n",
    );
    assert_eq!(gtk::icons(&xdg), text("Yaru-blue"));
    let gtk4 = Fake::default()
        .var("HOME", "/home/u")
        .file("/home/u/.config/gtk-3.0/settings.ini", "[Settings]\n")
        .file(
            "/home/u/.config/gtk-4.0/settings.ini",
            "[Settings]\ngtk-icon-theme-name=Papirus\n",
        );
    assert_eq!(gtk::icons(&gtk4), text("Papirus"));
    assert_eq!(gtk::icons(&Fake::default().var("HOME", "/h")), missing());
}

#[test]
fn fonte_mostra_a_da_interface_e_a_do_gtk_quando_existe() {
    assert_eq!(
        gtk::fonts("Mono", Some("Cantarell 11")),
        "Mono · Cantarell 11"
    );
    assert_eq!(gtk::fonts("Mono", Some("Mono")), "Mono");
    assert_eq!(gtk::fonts("Mono", None), "Mono");
    let host = Fake::default().var("HOME", "/h");
    assert_eq!(gtk::font(&host), text(sparky_theme::FONT_FAMILY));
    let broken = Fake::default()
        .var("HOME", "/h")
        .broken("/h/.config/gtk-3.0/settings.ini");
    assert!(failed(gtk::font(&broken)));
}
