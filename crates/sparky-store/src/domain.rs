#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Manager {
    Pacman,
    Apt,
    Dnf,
}

impl Manager {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pacman => "pacman",
            Self::Apt => "APT",
            Self::Dnf => "DNF",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Installation {
    User,
    System,
}

impl Installation {
    pub fn flag(self) -> &'static str {
        match self {
            Self::User => "--user",
            Self::System => "--system",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Flathub(Installation),
    Native(Manager),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Application {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub version: Option<String>,
    pub source: Source,
    pub installed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Install,
    Update,
    Remove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchSource {
    Preferred,
    Native,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    pub apps: Vec<Application>,
    pub source: Option<Source>,
    pub warning: Option<String>,
    pub flathub_missing: bool,
}

pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && !id.starts_with('-')
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._+-".contains(&byte))
}

pub fn choose_source(
    preferred: Result<Vec<Application>, String>,
    native: Result<Vec<Application>, String>,
    flathub_missing: bool,
) -> Result<SearchResult, String> {
    match preferred {
        Ok(apps) if !apps.is_empty() => Ok(SearchResult {
            source: apps.first().map(|app| app.source),
            apps,
            warning: None,
            flathub_missing,
        }),
        other => {
            let (apps, warning) = match (other.err(), native) {
                (None, Ok(apps)) => (apps, None),
                (Some(warning), Ok(apps)) => (apps, Some(warning)),
                (None, Err(warning)) => (Vec::new(), Some(warning)),
                (Some(primary), Err(fallback)) => return Err(format!("{primary}; {fallback}")),
            };
            let source = apps.first().map(|app| app.source);
            Ok(SearchResult {
                apps,
                source,
                warning,
                flathub_missing,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Application, Installation, Manager, Source, choose_source, valid_id};

    fn app(source: Source) -> Application {
        Application {
            id: "org.example.App".into(),
            name: "Example".into(),
            summary: String::new(),
            version: None,
            source,
            installed: false,
        }
    }

    #[test]
    fn prioriza_flathub_e_usa_nativo_se_ausente() {
        let flatpak = app(Source::Flathub(Installation::System));
        let native = app(Source::Native(Manager::Pacman));
        assert_eq!(
            choose_source(Ok(vec![flatpak.clone()]), Ok(vec![native.clone()]), false)
                .unwrap()
                .apps,
            vec![flatpak]
        );
        assert_eq!(
            choose_source(Ok(Vec::new()), Ok(vec![native.clone()]), false)
                .unwrap()
                .apps,
            vec![native]
        );
    }

    #[test]
    fn nomes_de_pacotes_nao_podem_virar_opcoes() {
        assert!(valid_id("org.gimp.GIMP"));
        assert!(valid_id("libfoo-1.0+meta"));
        assert!(!valid_id("--root"));
        assert!(!valid_id("foo/bar"));
        assert!(!valid_id(""));
    }
}
pub mod updates;
