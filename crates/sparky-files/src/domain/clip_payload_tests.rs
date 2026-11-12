use std::path::PathBuf;

use super::clip::Clip;
use super::clip_payload::{gnome, parse_gnome, parse_uri_list, plain, uri_list};

fn caminhos(textos: &[&str]) -> Vec<PathBuf> {
    textos.iter().map(PathBuf::from).collect()
}

#[test]
fn formato_do_gnome_leva_a_operacao_e_os_uris() {
    let clip = Clip::Cut(caminhos(&[
        "/home/ana/Fotos/praia 1.png",
        "/home/ana/nota.txt",
    ]));
    assert_eq!(
        gnome(&clip),
        "cut\nfile:///home/ana/Fotos/praia%201.png\nfile:///home/ana/nota.txt"
    );
    assert_eq!(parse_gnome(&gnome(&clip)), Some(clip));
}

#[test]
fn lista_de_uris_usa_crlf_e_volta_igual() {
    let clip = Clip::Copy(caminhos(&["/tmp/São Paulo.txt"]));
    assert_eq!(uri_list(&clip), "file:///tmp/S%C3%A3o%20Paulo.txt\r\n");
    assert_eq!(parse_uri_list(&uri_list(&clip), false), Some(clip));
}

#[test]
fn lista_do_dolphin_com_marca_de_recorte_vira_recorte() {
    let texto = "# comentario\r\nfile:///home/ana/a.txt\r\nhttps://exemplo.com/x\r\n";
    assert_eq!(
        parse_uri_list(texto, true),
        Some(Clip::Cut(caminhos(&["/home/ana/a.txt"])))
    );
}

#[test]
fn uri_com_localhost_e_aceito_e_sem_arquivos_e_ignorado() {
    assert_eq!(
        parse_uri_list("file://localhost/etc/hosts", false),
        Some(Clip::Copy(caminhos(&["/etc/hosts"])))
    );
    assert_eq!(parse_uri_list("https://exemplo.com", false), None);
    assert_eq!(parse_gnome("copy\n"), None);
    assert_eq!(parse_gnome("abrir\nfile:///a"), None);
    assert_eq!(parse_uri_list("file:///a%00b", false), None);
}

#[test]
fn texto_simples_lista_um_caminho_por_linha() {
    let clip = Clip::Copy(caminhos(&["/a/b.txt", "/c"]));
    assert_eq!(plain(&clip), "/a/b.txt\n/c");
}
