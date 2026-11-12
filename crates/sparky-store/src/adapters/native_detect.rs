use crate::domain::Manager;

pub fn detect(release: &str) -> Option<Manager> {
    let id = value(release, "ID")?;
    let family = value(release, "ID_LIKE").unwrap_or_default();
    let names = std::iter::once(id).chain(family.split_whitespace().map(str::to_owned));
    for name in names {
        match name.as_str() {
            "arch" | "manjaro" | "endeavouros" => return Some(Manager::Pacman),
            "debian" | "ubuntu" => return Some(Manager::Apt),
            "fedora" | "rhel" | "centos" | "rocky" => return Some(Manager::Dnf),
            _ => {}
        }
    }
    None
}

fn value(release: &str, key: &str) -> Option<String> {
    release
        .lines()
        .filter_map(|line| line.split_once('='))
        .find(|(name, _)| *name == key)
        .map(|(_, value)| value.trim_matches(['"', '\'']).to_owned())
}

#[cfg(test)]
mod tests {
    use super::detect;
    use crate::domain::Manager;

    #[test]
    fn identifica_familia_sem_escolher_gerenciador_errado() {
        assert_eq!(
            detect("ID=endeavouros\nID_LIKE=arch\n"),
            Some(Manager::Pacman)
        );
        assert_eq!(
            detect("ID=linuxmint\nID_LIKE=ubuntu debian\n"),
            Some(Manager::Apt)
        );
        assert_eq!(
            detect("ID=rocky\nID_LIKE=\"rhel fedora\"\n"),
            Some(Manager::Dnf)
        );
        assert_eq!(detect("ID=unknown\n"), None);
    }
}
