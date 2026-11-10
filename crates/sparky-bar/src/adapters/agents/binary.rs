use std::env;
use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub fn find(name: &str, home: &Path) -> Option<PathBuf> {
    let path = env::var_os("PATH");
    find_in(name, home, path.as_deref())
}

fn find_in(name: &str, home: &Path, path: Option<&OsStr>) -> Option<PathBuf> {
    let user_paths = [
        home.join(".local/bin"),
        home.join(".npm-global/bin"),
        home.join(".local/share/mise/shims"),
    ];
    path.into_iter()
        .flat_map(env::split_paths)
        .chain(user_paths)
        .map(|directory| directory.join(name))
        .find(|candidate| {
            candidate
                .metadata()
                .is_ok_and(|file| file.is_file() && file.permissions().mode() & 0o111 != 0)
        })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use super::find_in;

    #[test]
    fn encontra_cli_do_usuario_fora_do_path_da_sessao() {
        let home = tempfile::tempdir().unwrap();
        let directory = home.path().join(".local/bin");
        fs::create_dir_all(&directory).unwrap();
        let binary = directory.join("codex");
        fs::write(&binary, "#!/bin/sh\n").unwrap();
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(find_in("codex", home.path(), None), Some(binary));
    }

    #[test]
    fn ignora_arquivo_sem_permissao_de_execucao() {
        let home = tempfile::tempdir().unwrap();
        let directory = home.path().join(".local/bin");
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("claude"), "not executable").unwrap();
        assert_eq!(find_in("claude", home.path(), None), None);
    }
}
