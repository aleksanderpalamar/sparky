use crate::domain::app::App;
use crate::domain::category::{CATEGORIES, Category};
use crate::domain::groups::Groups;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Shelf {
    #[default]
    Home,
    Category(Category),
    Group(usize),
}

impl Shelf {
    pub fn holds(self, app: &App, groups: &Groups) -> bool {
        match self {
            Shelf::Home => !groups.grouped(&app.id),
            Shelf::Category(category) => category.holds(app),
            Shelf::Group(index) => groups.holds(index, &app.id),
        }
    }
}

pub fn shelves(apps: &[App], groups: &Groups) -> Vec<Shelf> {
    let categories = CATEGORIES
        .into_iter()
        .filter(|category| apps.iter().any(|app| category.holds(app)))
        .map(Shelf::Category);
    let custom = (0..groups.list().len()).map(Shelf::Group);
    std::iter::once(Shelf::Home)
        .chain(categories)
        .chain(custom)
        .collect()
}

pub fn neighbour(shelves: &[Shelf], current: Shelf, delta: isize) -> Shelf {
    let Some(position) = shelves.iter().position(|shelf| *shelf == current) else {
        return Shelf::Home;
    };
    let count = shelves.len() as isize;
    let next = (position as isize + delta).rem_euclid(count.max(1)) as usize;
    shelves.get(next).copied().unwrap_or_default()
}

pub fn after_removal(current: Shelf, removed: usize) -> Shelf {
    match current {
        Shelf::Group(index) if index == removed => Shelf::Home,
        Shelf::Group(index) if index > removed => Shelf::Group(index - 1),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::{Shelf, after_removal, neighbour, shelves};
    use crate::domain::app::{App, Runs};
    use crate::domain::category::Category;
    use crate::domain::groups::{Group, Groups};

    fn app(id: &str, categorias: &[&str]) -> App {
        App {
            id: id.to_owned(),
            name: id.to_owned(),
            detail: None,
            keywords: Vec::new(),
            command: id.to_owned(),
            runs: Runs::Directly,
            icon: None,
            categories: categorias.iter().map(|name| (*name).to_owned()).collect(),
        }
    }

    fn grupos() -> Groups {
        Groups::new(vec![Group {
            name: "Favoritos".to_owned(),
            apps: vec!["kitty".to_owned()],
        }])
    }

    #[test]
    fn a_barra_mostra_inicio_categorias_com_apps_e_os_grupos() {
        let apps = [app("steam", &["Game"]), app("kitty", &["System"])];
        assert_eq!(
            shelves(&apps, &grupos()),
            vec![
                Shelf::Home,
                Shelf::Category(Category::System),
                Shelf::Category(Category::Games),
                Shelf::Group(0),
            ]
        );
    }

    #[test]
    fn cada_prateleira_mostra_so_os_seus_aplicativos() {
        let steam = app("steam", &["Game"]);
        let kitty = app("kitty", &["System"]);
        assert!(Shelf::Home.holds(&steam, &grupos()));
        assert!(Shelf::Category(Category::Games).holds(&steam, &grupos()));
        assert!(!Shelf::Category(Category::Games).holds(&kitty, &grupos()));
        assert!(Shelf::Group(0).holds(&kitty, &grupos()));
        assert!(!Shelf::Group(0).holds(&steam, &grupos()));
    }

    #[test]
    fn aplicativo_num_grupo_sai_do_inicio_mas_fica_na_categoria() {
        let kitty = app("kitty", &["System"]);
        assert!(!Shelf::Home.holds(&kitty, &grupos()));
        assert!(Shelf::Category(Category::System).holds(&kitty, &grupos()));
        assert!(Shelf::Home.holds(&kitty, &Groups::default()));
    }

    #[test]
    fn trocar_de_prateleira_da_a_volta_nas_pontas() {
        let barra = [
            Shelf::Home,
            Shelf::Category(Category::Games),
            Shelf::Group(0),
        ];
        assert_eq!(
            neighbour(&barra, Shelf::Home, 1),
            Shelf::Category(Category::Games)
        );
        assert_eq!(neighbour(&barra, Shelf::Home, -1), Shelf::Group(0));
        assert_eq!(neighbour(&barra, Shelf::Group(0), 1), Shelf::Home);
        assert_eq!(neighbour(&barra, Shelf::Group(4), 1), Shelf::Home);
    }

    #[test]
    fn apagar_um_grupo_ajusta_a_prateleira_aberta() {
        assert_eq!(after_removal(Shelf::Group(1), 1), Shelf::Home);
        assert_eq!(after_removal(Shelf::Group(2), 0), Shelf::Group(1));
        assert_eq!(after_removal(Shelf::Group(0), 3), Shelf::Group(0));
        assert_eq!(after_removal(Shelf::Home, 0), Shelf::Home);
    }
}
