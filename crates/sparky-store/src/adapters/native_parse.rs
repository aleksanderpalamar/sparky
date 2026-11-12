use crate::domain::{Application, Manager, Source, valid_id};

fn app(id: &str, summary: &str, version: Option<&str>, manager: Manager) -> Option<Application> {
    if !valid_id(id) {
        return None;
    }
    Some(Application {
        id: id.to_owned(),
        name: id.to_owned(),
        summary: summary.to_owned(),
        version: version.map(str::to_owned),
        source: Source::Native(manager),
        installed: false,
    })
}

pub fn parse_pacman(output: &str) -> Vec<Application> {
    output
        .lines()
        .collect::<Vec<_>>()
        .chunks(2)
        .filter_map(|pair| {
            let [first, description] = pair else {
                return None;
            };
            let (repo_name, version) = first.split_once(' ')?;
            let (_, name) = repo_name.split_once('/')?;
            let version = version.split_whitespace().next();
            app(name, description.trim(), version, Manager::Pacman)
        })
        .collect()
}

pub fn parse_apt(output: &str) -> Vec<Application> {
    output
        .lines()
        .filter_map(|line| {
            let (name, summary) = line.split_once(" - ")?;
            app(name, summary, None, Manager::Apt)
        })
        .collect()
}

pub fn parse_dnf(output: &str) -> Vec<Application> {
    output
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(3, '\t');
            app(fields.next()?, fields.next()?, fields.next(), Manager::Dnf)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{parse_apt, parse_dnf, parse_pacman};

    #[test]
    fn interpreta_resultados_nativos() {
        assert_eq!(
            parse_pacman("extra/gimp 3.0-1\n    Image editor\n")[0].id,
            "gimp"
        );
        assert_eq!(
            parse_apt("gimp - Image editor\n")[0].summary,
            "Image editor"
        );
        assert_eq!(
            parse_dnf("gimp\tImage editor\t3.0\n")[0].version.as_deref(),
            Some("3.0")
        );
    }
}
