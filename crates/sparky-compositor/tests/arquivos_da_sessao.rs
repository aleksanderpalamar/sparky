use std::fs;
use std::path::PathBuf;

use sparky_compositor::session::desktop_entry::DesktopEntry;
use sparky_compositor::session::environment::DESKTOP;

fn raiz() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn ler(caminho: &str) -> String {
    let arquivo = raiz().join(caminho);
    fs::read_to_string(&arquivo).unwrap_or_else(|erro| panic!("ler {}: {erro}", arquivo.display()))
}

fn valor(texto: &str, chave: &str) -> Option<String> {
    let marca = format!("{chave}=");
    texto
        .lines()
        .map(str::trim)
        .find_map(|linha| linha.strip_prefix(&marca))
        .map(str::to_owned)
}

#[test]
fn a_sessao_abre_o_binario_do_compositor() {
    let sessao = ler("session/sparky.desktop");
    let Some(entrada) = DesktopEntry::parse(&sessao) else {
        panic!("sparky.desktop não é uma entrada válida");
    };
    let binario = PathBuf::from(env!("CARGO_BIN_EXE_sparky"));
    let nome = binario.file_name().map(|nome| nome.to_string_lossy().into_owned());
    assert_eq!(entrada.exec, nome);
}

#[test]
fn a_sessao_se_anuncia_com_o_nome_do_desktop() {
    assert_eq!(valor(&ler("session/sparky.desktop"), "DesktopNames").as_deref(), Some(DESKTOP));
}

#[test]
fn os_portais_seguem_o_nome_do_desktop() {
    let portais = raiz().join(format!("portals/{DESKTOP}-portals.conf"));
    assert!(portais.is_file(), "falta {}", portais.display());
    assert_eq!(valor(&ler(&format!("portals/{DESKTOP}-portals.conf")), "default").as_deref(), Some("gtk"));
}

#[test]
fn compartilhar_e_capturar_a_tela_vao_pelo_portal_wlr() {
    let portais = ler(&format!("portals/{DESKTOP}-portals.conf"));
    assert_eq!(valor(&portais, "org.freedesktop.impl.portal.ScreenCast").as_deref(), Some("wlr"));
    assert_eq!(valor(&portais, "org.freedesktop.impl.portal.Screenshot").as_deref(), Some("wlr"));
}
