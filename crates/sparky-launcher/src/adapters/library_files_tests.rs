use crate::adapters::library_files::LibraryFiles;
use crate::domain::category::Category;
use crate::domain::groups::{Group, Groups};
use crate::domain::library::Library;
use crate::domain::placements::{Choice, Placement, Placements};
use crate::ports::LibraryStore;

fn grupo(name: &str, apps: &[&str]) -> Group {
    Group {
        name: name.to_owned(),
        apps: apps.iter().map(|app| (*app).to_owned()).collect(),
    }
}

fn escolha(category: Category, app: &str, choice: Choice) -> Placement {
    Placement {
        category,
        app: app.to_owned(),
        choice,
    }
}

#[test]
fn sem_arquivos_a_biblioteca_comeca_vazia() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(LibraryFiles::new(dir.path()).load(), Ok(Library::default()));
}

#[test]
fn salvar_e_ler_devolve_grupos_e_escolhas_de_categoria() {
    let dir = tempfile::tempdir().unwrap();
    let arquivos = LibraryFiles::new(dir.path());
    let library = Library {
        groups: Groups::new(vec![
            grupo("Jogos", &["steam.desktop", "heroic.desktop"]),
            grupo("Vazio", &[]),
        ]),
        placements: Placements::new(vec![
            escolha(Category::Utilities, "emojify.desktop", Choice::Added),
            escolha(Category::Games, "steam.desktop", Choice::Removed),
        ]),
    };
    assert_eq!(arquivos.save(&library), Ok(()));
    assert_eq!(arquivos.load(), Ok(library));
    let categorias =
        std::fs::read_to_string(dir.path().join("sparky/launcher-categories")).unwrap();
    assert_eq!(
        categorias,
        "+\tUtility\temojify.desktop\n-\tGame\tsteam.desktop\n"
    );
    assert!(!dir.path().join("sparky/launcher-groups.new").exists());
}

#[test]
fn o_arquivo_de_grupos_continua_no_formato_antigo() {
    let dir = tempfile::tempdir().unwrap();
    let pasta = dir.path().join("sparky");
    std::fs::create_dir_all(&pasta).unwrap();
    std::fs::write(pasta.join("launcher-groups"), "Teste\tkitty.desktop\n").unwrap();
    let lida = LibraryFiles::new(dir.path()).load().unwrap();
    assert_eq!(lida.groups.list(), &[grupo("Teste", &["kitty.desktop"])]);
}

#[test]
fn tab_no_nome_e_linhas_invalidas_nao_quebram_os_arquivos() {
    let dir = tempfile::tempdir().unwrap();
    let pasta = dir.path().join("sparky");
    std::fs::create_dir_all(&pasta).unwrap();
    std::fs::write(
        pasta.join("launcher-categories"),
        "+\tGame\tx\n?\tGame\ty\n+\tNetwork\tz\n-\tOffice\t\n",
    )
    .unwrap();
    let arquivos = LibraryFiles::new(dir.path());
    let lida = arquivos.load().unwrap();
    assert_eq!(
        lida.placements.list(),
        &[escolha(Category::Games, "x", Choice::Added)]
    );
    let library = Library {
        groups: Groups::new(vec![grupo("A\tB", &["x"])]),
        ..Library::default()
    };
    arquivos.save(&library).unwrap();
    assert_eq!(
        arquivos.load().unwrap().groups.list(),
        &[grupo("A B", &["x"])]
    );
}
