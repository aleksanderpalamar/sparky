use crate::domain::app::App;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Office,
    Development,
    System,
    Games,
    Utilities,
}

pub const CATEGORIES: [Category; 5] = [
    Category::Office,
    Category::Development,
    Category::System,
    Category::Games,
    Category::Utilities,
];

impl Category {
    pub fn from_key(key: &str) -> Option<Self> {
        CATEGORIES
            .into_iter()
            .find(|category| category.key() == key)
    }

    pub fn key(self) -> &'static str {
        match self {
            Category::Office => "Office",
            Category::Development => "Development",
            Category::System => "System",
            Category::Games => "Game",
            Category::Utilities => "Utility",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Category::Office => "shelf-office",
            Category::Development => "shelf-development",
            Category::System => "shelf-system",
            Category::Games => "shelf-games",
            Category::Utilities => "shelf-utilities",
        }
    }

    pub fn icon(self) -> &'static [&'static str] {
        match self {
            Category::Office => &["applications-office-symbolic", "x-office-document-symbolic"],
            Category::Development => &[
                "applications-engineering-symbolic",
                "applications-development-symbolic",
                "utilities-terminal-symbolic",
            ],
            Category::System => &[
                "applications-system-symbolic",
                "preferences-system-symbolic",
            ],
            Category::Games => &["applications-games-symbolic", "input-gaming-symbolic"],
            Category::Utilities => &[
                "applications-utilities-symbolic",
                "applications-accessories-symbolic",
            ],
        }
    }

    pub fn holds(self, app: &App) -> bool {
        app.categories.iter().any(|category| category == self.key())
    }
}

#[cfg(test)]
mod tests {
    use super::{CATEGORIES, Category};
    use crate::domain::app::{App, Runs};

    fn app(categorias: &[&str]) -> App {
        App {
            id: "x.desktop".to_owned(),
            name: "X".to_owned(),
            detail: None,
            keywords: Vec::new(),
            command: "x".to_owned(),
            runs: Runs::Directly,
            icon: None,
            categories: categorias.iter().map(|name| (*name).to_owned()).collect(),
        }
    }

    #[test]
    fn cada_categoria_usa_a_categoria_principal_do_freedesktop() {
        assert!(Category::Games.holds(&app(&["Game", "ActionGame"])));
        assert!(Category::Utilities.holds(&app(&["Utility"])));
        assert!(Category::Development.holds(&app(&["Development", "IDE"])));
        assert!(Category::Office.holds(&app(&["Office"])));
        assert!(Category::System.holds(&app(&["System", "Monitor"])));
    }

    #[test]
    fn a_chave_do_freedesktop_volta_para_a_categoria() {
        assert!(
            CATEGORIES
                .iter()
                .all(|category| Category::from_key(category.key()) == Some(*category))
        );
        assert_eq!(Category::from_key("Network"), None);
    }

    #[test]
    fn aplicativo_sem_a_categoria_fica_de_fora() {
        assert!(!Category::Games.holds(&app(&["ActionGame"])));
        assert!(CATEGORIES.iter().all(|category| !category.holds(&app(&[]))));
    }
}
