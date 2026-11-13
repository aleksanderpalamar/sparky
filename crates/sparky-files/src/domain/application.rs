use std::path::PathBuf;

use sparky_ui::natural::fold;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Application {
    pub id: String,
    pub name: String,
    pub icon: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Remember {
    #[default]
    Once,
    Always,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Recommended,
    Other,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppChoices {
    pub kind: String,
    pub recommended: Vec<Application>,
    pub others: Vec<Application>,
}

impl AppChoices {
    pub fn is_empty(&self) -> bool {
        self.recommended.is_empty() && self.others.is_empty()
    }

    pub fn find(&self, id: &str) -> Option<&Application> {
        self.recommended
            .iter()
            .chain(&self.others)
            .find(|app| app.id == id)
    }

    pub fn visible(&self, query: &str) -> Vec<(Section, &Application)> {
        let wanted = fold(query.trim());
        let matches = |app: &&Application| fold(&app.name).contains(&wanted);
        let recommended = self.recommended.iter().filter(matches);
        let others = self.others.iter().filter(matches);
        recommended
            .map(|app| (Section::Recommended, app))
            .chain(others.map(|app| (Section::Other, app)))
            .collect()
    }

    pub fn first(&self, query: &str) -> Option<String> {
        self.visible(query).first().map(|(_, app)| app.id.clone())
    }

    pub fn step(&self, query: &str, current: Option<&str>, forward: bool) -> Option<String> {
        let visible = self.visible(query);
        let position = visible
            .iter()
            .position(|(_, app)| Some(app.id.as_str()) == current);
        let next = match (position, forward) {
            (None, _) => 0,
            (Some(at), true) => (at + 1).min(visible.len().saturating_sub(1)),
            (Some(at), false) => at.saturating_sub(1),
        };
        visible.get(next).map(|(_, app)| app.id.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::{AppChoices, Application, Section};

    fn app(id: &str, name: &str) -> Application {
        Application {
            id: id.to_owned(),
            name: name.to_owned(),
            icon: None,
        }
    }

    fn escolhas() -> AppChoices {
        AppChoices {
            kind: "Documento de texto simples".to_owned(),
            recommended: vec![app("vim.desktop", "Vim")],
            others: vec![
                app("gedit.desktop", "Editor de Texto"),
                app("vlc.desktop", "VLC"),
            ],
        }
    }

    fn ids(lista: &[(Section, &Application)]) -> Vec<(Section, String)> {
        lista
            .iter()
            .map(|(secao, app)| (*secao, app.id.clone()))
            .collect()
    }

    #[test]
    fn recomendados_vem_antes_dos_outros() {
        assert_eq!(
            ids(&escolhas().visible("")),
            [
                (Section::Recommended, "vim.desktop".to_owned()),
                (Section::Other, "gedit.desktop".to_owned()),
                (Section::Other, "vlc.desktop".to_owned()),
            ]
        );
    }

    #[test]
    fn a_busca_ignora_caixa_e_acentos() {
        assert_eq!(
            ids(&escolhas().visible("TEXTO")),
            [(Section::Other, "gedit.desktop".to_owned())]
        );
        assert!(escolhas().visible("xyz").is_empty());
        assert_eq!(escolhas().first("vl").as_deref(), Some("vlc.desktop"));
    }

    #[test]
    fn as_setas_andam_pela_lista_visivel_sem_passar_das_pontas() {
        let escolhas = escolhas();
        let descer = |atual: Option<&str>| escolhas.step("", atual, true);
        assert_eq!(
            descer(Some("vim.desktop")).as_deref(),
            Some("gedit.desktop")
        );
        assert_eq!(descer(Some("vlc.desktop")).as_deref(), Some("vlc.desktop"));
        assert_eq!(
            escolhas.step("", Some("vim.desktop"), false).as_deref(),
            Some("vim.desktop")
        );
        assert_eq!(descer(None).as_deref(), Some("vim.desktop"));
        assert!(escolhas.find("vlc.desktop").is_some());
        assert!(!escolhas.is_empty());
    }
}
