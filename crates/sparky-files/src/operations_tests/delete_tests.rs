use std::path::Path;

use super::FakeDisk;
use crate::operations::delete;

#[test]
fn excluir_de_vez_conta_os_itens_e_para_na_primeira_falha() {
    let disk = FakeDisk::default();
    let names = ["a.txt".to_owned(), "Fotos".to_owned()];
    assert_eq!(delete(Path::new("/h"), &names, &disk).0, 2);
    assert_eq!(disk.calls(), ["excluir /h/a.txt", "excluir /h/Fotos"]);
    let quebrado = FakeDisk {
        broken: true,
        ..FakeDisk::default()
    };
    let (count, result) = delete(Path::new("/h"), &names, &quebrado);
    assert_eq!(count, 0);
    assert_eq!(
        result.err().map(|f| f.to_string()).as_deref(),
        Some("a.txt: disco cheio")
    );
}
