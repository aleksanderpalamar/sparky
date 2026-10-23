use std::path::{Path, PathBuf};

use crate::domain::zsh::{Dirs, chosen, dirs};

#[test]
fn a_configuracao_do_sparky_fica_em_usr_share_e_a_do_usuario_em_config() {
    assert_eq!(
        dirs(None, "/home/ana", None),
        Dirs {
            user: PathBuf::from("/home/ana/.config/sparky/zsh"),
            system: PathBuf::from("/usr/share/sparky/zsh")
        }
    );
    assert_eq!(
        dirs(Some("/cfg"), "/home/ana", Some("/opt/sparky")).system,
        PathBuf::from("/opt/sparky/zsh")
    );
    assert_eq!(
        dirs(Some("/cfg"), "/home/ana", None).user,
        PathBuf::from("/cfg/sparky/zsh")
    );
}

#[test]
fn o_zshrc_do_usuario_no_sparky_substitui_o_do_sistema() {
    let pastas = dirs(None, "/home/ana", None);
    let existe = |caminho: &Path| caminho == Path::new("/home/ana/.config/sparky/zsh/.zshrc");
    assert_eq!(
        chosen(&pastas, existe),
        PathBuf::from("/home/ana/.config/sparky/zsh")
    );
    assert_eq!(
        chosen(&pastas, |_| false),
        PathBuf::from("/usr/share/sparky/zsh")
    );
}
