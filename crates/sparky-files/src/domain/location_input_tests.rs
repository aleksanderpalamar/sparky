use std::path::{Path, PathBuf};

use super::location_input::{Typed, matches, normalize, pattern, typed};

fn nomes(lista: &[&str]) -> Vec<String> {
    lista.iter().map(|nome| (*nome).to_owned()).collect()
}

#[test]
fn caminho_digitado_aceita_absoluto_relativo_til_e_file() {
    let atual = Path::new("/dados/fotos");
    let casa = Path::new("/home/ana");
    let pasta = |texto| typed(texto, atual, casa);
    assert_eq!(pasta("/mnt"), Some(Typed::Folder(PathBuf::from("/mnt"))));
    assert_eq!(
        pasta("../videos"),
        Some(Typed::Folder(PathBuf::from("/dados/videos")))
    );
    assert_eq!(
        pasta(" ~/Documentos "),
        Some(Typed::Folder(PathBuf::from("/home/ana/Documentos")))
    );
    assert_eq!(
        pasta("file:///tmp/x"),
        Some(Typed::Folder(PathBuf::from("/tmp/x")))
    );
    assert_eq!(pasta("   "), None);
}

#[test]
fn uri_com_servidor_vai_para_a_rede() {
    let atual = Path::new("/");
    let uri = typed("smb://nas/fotos", atual, atual);
    assert_eq!(uri, Some(Typed::Remote("smb://nas/fotos".to_owned())));
    assert_eq!(
        typed("sftp://ana@servidor:22/", atual, atual),
        Some(Typed::Remote("sftp://ana@servidor:22/".to_owned()))
    );
    assert!(matches!(
        typed("smb:///", atual, atual),
        Some(Typed::Folder(_))
    ));
}

#[test]
fn normalizar_resolve_ponto_e_ponto_ponto() {
    assert_eq!(normalize(Path::new("/a/./b/../c")), PathBuf::from("/a/c"));
    assert_eq!(normalize(Path::new("/../a")), PathBuf::from("/a"));
}

#[test]
fn padrao_completa_dentro_da_pasta_quando_termina_em_barra() {
    let dentro = pattern("/dados/", Path::new("/dados"));
    assert_eq!(
        dentro.map(|p| (p.folder, p.prefix)),
        Some((PathBuf::from("/dados"), String::new()))
    );
    let irmao = pattern("~/Do", Path::new("/home/ana/Do"));
    let esperado = (PathBuf::from("/home/ana"), "Do".to_owned(), "~/".to_owned());
    assert_eq!(irmao.map(|p| (p.folder, p.prefix, p.kept)), Some(esperado));
}

#[test]
fn completar_filtra_pelo_prefixo_sem_diferenciar_maiusculas_e_ordena() {
    let achados = matches(
        "do",
        nomes(&["Downloads", "documentos", "Fotos", "doc 10", "doc 9"]),
    );
    assert_eq!(achados, ["doc 9", "doc 10", "documentos", "Downloads"]);
}

#[test]
fn completar_sem_prefixo_esconde_ocultos_e_limita_a_oito() {
    let muitos: Vec<String> = (0..12)
        .map(|n| format!("p{n:02}"))
        .chain([".oculta".to_owned()])
        .collect();
    let achados = matches("", muitos.clone());
    assert_eq!(achados.len(), 8);
    assert!(achados.iter().all(|nome| !nome.starts_with('.')));
    assert_eq!(matches(".o", muitos), [".oculta"]);
}
