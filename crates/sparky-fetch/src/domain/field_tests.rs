use crate::domain::field::{Field, Group};

#[test]
fn cada_campo_tem_uma_chave_estavel_e_volta_por_ela() {
    for field in Field::ALL {
        assert_eq!(Field::named(field.key()), Some(field));
    }
    assert_eq!(Field::Os.key(), "os");
    assert_eq!(Field::Packages.key(), "packages");
    assert_eq!(Field::named(" gpu "), Some(Field::Gpu));
    assert_eq!(Field::named("bateria"), None);
}

#[test]
fn as_chaves_nao_se_repetem() {
    let mut chaves: Vec<&str> = Field::ALL.iter().map(|field| field.key()).collect();
    chaves.sort_unstable();
    chaves.dedup();
    assert_eq!(chaves.len(), Field::ALL.len());
}

#[test]
fn linux_vem_antes_do_sparky_na_ordem_padrao() {
    let grupos: Vec<Group> = Field::ALL.iter().map(|field| field.group()).collect();
    let primeiro_sparky = grupos.iter().position(|group| *group == Group::Sparky);
    assert_eq!(primeiro_sparky, Some(10));
    assert!(grupos[10..].iter().all(|group| *group == Group::Sparky));
}
