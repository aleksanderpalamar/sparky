use crate::adapters::group_file::{GroupFile, location};
use crate::domain::groups::Group;
use crate::ports::GroupStore;

fn grupo(name: &str, apps: &[&str]) -> Group {
    Group {
        name: name.to_owned(),
        apps: apps.iter().map(|app| (*app).to_owned()).collect(),
    }
}

#[test]
fn sem_arquivo_nao_ha_grupos() {
    let dir = tempfile::tempdir().unwrap();
    let arquivo = GroupFile::new(location(dir.path()));
    assert_eq!(arquivo.load(), Ok(Vec::new()));
}

#[test]
fn salvar_e_ler_devolve_os_mesmos_grupos_na_mesma_ordem() {
    let dir = tempfile::tempdir().unwrap();
    let arquivo = GroupFile::new(location(dir.path()));
    let grupos = vec![
        grupo("Jogos", &["steam.desktop", "heroic.desktop"]),
        grupo("Vazio", &[]),
    ];
    assert_eq!(arquivo.save(&grupos), Ok(()));
    assert_eq!(arquivo.load(), Ok(grupos));
    assert!(dir.path().join("sparky/launcher-groups").is_file());
    assert!(!dir.path().join("sparky/launcher-groups.new").exists());
}

#[test]
fn tab_no_nome_nao_quebra_o_arquivo() {
    let dir = tempfile::tempdir().unwrap();
    let arquivo = GroupFile::new(location(dir.path()));
    arquivo.save(&[grupo("A\tB", &["x"])]).unwrap();
    assert_eq!(arquivo.load(), Ok(vec![grupo("A B", &["x"])]));
}

#[test]
fn linhas_vazias_ou_sem_nome_sao_ignoradas() {
    let dir = tempfile::tempdir().unwrap();
    let caminho = location(dir.path());
    std::fs::create_dir_all(caminho.parent().unwrap()).unwrap();
    std::fs::write(&caminho, "\n\tx.desktop\nJogos\nDev\t;code.desktop;;\n").unwrap();
    let lidos = GroupFile::new(caminho).load();
    assert_eq!(
        lidos,
        Ok(vec![grupo("Jogos", &[]), grupo("Dev", &["code.desktop"])])
    );
}
