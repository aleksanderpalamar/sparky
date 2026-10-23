use super::entry::{Kind, entry};
use super::icons::IconKind;

#[test]
fn pastas_especiais_e_arquivos_sao_identificados_sem_abrir_o_conteudo() {
    assert_eq!(
        IconKind::for_entry(&entry("Fotos", Kind::Directory), None),
        IconKind::Folder
    );
    assert_eq!(
        IconKind::for_entry(&entry("Fotos", Kind::Directory), Some("Imagens")),
        IconKind::Pictures
    );
    assert_eq!(
        IconKind::for_entry(&entry("foto.PNG", Kind::File), None),
        IconKind::Image
    );
    assert_eq!(
        IconKind::for_entry(&entry("relatorio.pdf", Kind::File), None),
        IconKind::Pdf
    );
    assert_eq!(
        IconKind::for_entry(&entry("arquivo.zip", Kind::File), None),
        IconKind::Archive
    );
    assert_eq!(
        IconKind::for_entry(&entry("codigo.rs", Kind::File), None),
        IconKind::Code
    );
    assert_eq!(
        IconKind::for_entry(&entry("desconhecido", Kind::File), None),
        IconKind::File
    );
}

#[test]
fn cada_tipo_tem_icone_de_tema_e_fallback_generico() {
    assert_eq!(
        IconKind::Pdf.names(),
        &["application-pdf", "x-office-document", "text-x-generic"]
    );
    assert_eq!(IconKind::Folder.names()[0], "folder");
    assert_eq!(IconKind::Removable.names()[0], "drive-removable-media");
    assert_eq!(IconKind::Phone.names()[0], "phone");
    assert!(super::icons::ALL.contains(&IconKind::Phone));
}
