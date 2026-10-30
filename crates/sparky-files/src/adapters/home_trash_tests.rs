use std::ffi::OsStr;
use std::fs;
use std::io;
use std::os::unix::fs::symlink;

use super::home_trash::HomeTrash;
use crate::ports::TrashStore;

#[test]
fn restaura_arquivo_e_pasta_para_o_caminho_original() {
    let dir = tempfile::tempdir().unwrap();
    let trash = HomeTrash::new(dir.path().join("Trash"));
    let file = dir.path().join("Álbum novo.png");
    let folder = dir.path().join("Pasta");
    fs::write(&file, "imagem").unwrap();
    fs::create_dir(&folder).unwrap();
    trash.put(&file).unwrap();
    trash.put(&folder).unwrap();
    assert_eq!(trash.names().unwrap().len(), 2);
    assert_eq!(trash.restore(OsStr::new("Álbum novo.png")).unwrap(), file);
    assert_eq!(trash.restore(OsStr::new("Pasta")).unwrap(), folder);
    assert_eq!(fs::read_to_string(&file).unwrap(), "imagem");
    assert!(folder.is_dir());
    assert!(trash.names().unwrap().is_empty());
    assert!(
        !dir.path()
            .join("Trash/info/Álbum novo.png.trashinfo")
            .exists()
    );
}

#[test]
fn conflito_no_destino_preserva_o_conteudo_e_o_registro_na_lixeira() {
    let dir = tempfile::tempdir().unwrap();
    let trash = HomeTrash::new(dir.path().join("Trash"));
    let file = dir.path().join("nota.txt");
    fs::write(&file, "antigo").unwrap();
    trash.put(&file).unwrap();
    fs::write(&file, "novo").unwrap();
    assert_eq!(
        trash.restore(OsStr::new("nota.txt")).unwrap_err().kind(),
        io::ErrorKind::AlreadyExists
    );
    assert_eq!(fs::read_to_string(&file).unwrap(), "novo");
    assert_eq!(
        fs::read_to_string(dir.path().join("Trash/files/nota.txt")).unwrap(),
        "antigo"
    );
    assert!(dir.path().join("Trash/info/nota.txt.trashinfo").exists());
}

#[test]
fn recusa_item_sem_registro_e_nome_que_tenta_sair_da_lixeira() {
    let dir = tempfile::tempdir().unwrap();
    let trash = HomeTrash::new(dir.path().join("Trash"));
    trash.open().unwrap();
    fs::write(dir.path().join("Trash/files/orfao.txt"), "dado").unwrap();
    assert!(trash.restore(OsStr::new("orfao.txt")).is_err());
    assert!(trash.restore(OsStr::new("../orfao.txt")).is_err());
    assert!(dir.path().join("Trash/files/orfao.txt").exists());
}

#[test]
fn esvazia_arquivos_pastas_e_links_sem_seguir_o_destino() {
    let dir = tempfile::tempdir().unwrap();
    let trash = HomeTrash::new(dir.path().join("Trash"));
    let file = dir.path().join("foto.png");
    let folder = dir.path().join("Fotos");
    let outside = dir.path().join("fora.txt");
    fs::write(&file, "imagem").unwrap();
    fs::create_dir(&folder).unwrap();
    fs::write(folder.join("dentro.txt"), "dentro").unwrap();
    fs::write(&outside, "preservado").unwrap();
    trash.put(&file).unwrap();
    trash.put(&folder).unwrap();
    symlink(&outside, trash.directory().join("atalho")).unwrap();

    let result = trash.empty().unwrap();
    assert_eq!(result.removed, 3);
    assert!(result.errors.is_empty());
    assert!(trash.names().unwrap().is_empty());
    assert!(!dir.path().join("Trash/info/foto.png.trashinfo").exists());
    assert!(!dir.path().join("Trash/info/Fotos.trashinfo").exists());
    assert_eq!(fs::read_to_string(outside).unwrap(), "preservado");
}

#[test]
fn esvaziar_recusa_pasta_de_arquivos_redirecionada_por_link() {
    let dir = tempfile::tempdir().unwrap();
    let outside = dir.path().join("fora");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("intocado.txt"), "preservado").unwrap();
    let root = dir.path().join("Trash");
    fs::create_dir(&root).unwrap();
    symlink(&outside, root.join("files")).unwrap();

    let error = HomeTrash::new(root).empty().unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert_eq!(
        fs::read_to_string(outside.join("intocado.txt")).unwrap(),
        "preservado"
    );
}
