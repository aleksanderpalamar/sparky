use crate::domain::groups::Groups;
use crate::domain::shelf::Shelf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    App(String),
    Group(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    AddTo { group: usize, app: String },
    TakeFrom { group: usize, app: String },
    NewGroupWith(String),
    Rename(usize),
    Remove(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub action: Action,
    pub label: &'static str,
    pub group: Option<String>,
}

pub fn items(target: &Target, groups: &Groups, shelf: Shelf) -> Vec<Item> {
    match target {
        Target::App(app) => app_items(app, groups, shelf),
        Target::Group(index) => group_items(*index, groups),
    }
}

fn app_items(app: &str, groups: &Groups, shelf: Shelf) -> Vec<Item> {
    let named = |index: usize| groups.get(index).map(|group| group.name.clone());
    let take = match shelf {
        Shelf::Group(index) if groups.holds(index, app) => Some(Item {
            action: Action::TakeFrom {
                group: index,
                app: app.to_owned(),
            },
            label: "menu-take-from",
            group: named(index),
        }),
        _ => None,
    };
    let add = (0..groups.list().len())
        .filter(|index| !groups.holds(*index, app))
        .map(|index| Item {
            action: Action::AddTo {
                group: index,
                app: app.to_owned(),
            },
            label: "menu-add-to",
            group: named(index),
        });
    let create = Item {
        action: Action::NewGroupWith(app.to_owned()),
        label: "menu-new-group-with",
        group: None,
    };
    take.into_iter()
        .chain(add)
        .chain(std::iter::once(create))
        .collect()
}

fn group_items(index: usize, groups: &Groups) -> Vec<Item> {
    if groups.get(index).is_none() {
        return Vec::new();
    }
    vec![
        Item {
            action: Action::Rename(index),
            label: "menu-rename-group",
            group: None,
        },
        Item {
            action: Action::Remove(index),
            label: "menu-remove-group",
            group: None,
        },
    ]
}

pub fn placed(point: (f32, f32), area: (f32, f32), menu: (f32, f32)) -> (f32, f32) {
    let fit = |start: f32, room: f32, size: f32| start.min(room - size).max(0.0);
    (fit(point.0, area.0, menu.0), fit(point.1, area.1, menu.1))
}

#[cfg(test)]
mod tests {
    use super::{Action, Target, items, placed};
    use crate::domain::groups::{Group, Groups};
    use crate::domain::shelf::Shelf;

    fn grupos() -> Groups {
        Groups::new(vec![
            Group {
                name: "Jogos".to_owned(),
                apps: vec!["steam".to_owned()],
            },
            Group {
                name: "Trabalho".to_owned(),
                apps: Vec::new(),
            },
        ])
    }

    fn acoes(target: Target, shelf: Shelf) -> Vec<Action> {
        items(&target, &grupos(), shelf)
            .into_iter()
            .map(|item| item.action)
            .collect()
    }

    #[test]
    fn no_aplicativo_oferece_os_grupos_onde_ele_ainda_nao_esta() {
        assert_eq!(
            acoes(Target::App("steam".to_owned()), Shelf::Home),
            vec![
                Action::AddTo {
                    group: 1,
                    app: "steam".to_owned()
                },
                Action::NewGroupWith("steam".to_owned())
            ]
        );
    }

    #[test]
    fn dentro_do_grupo_oferece_tirar_o_aplicativo_dele() {
        assert_eq!(
            acoes(Target::App("steam".to_owned()), Shelf::Group(0))[0],
            Action::TakeFrom {
                group: 0,
                app: "steam".to_owned()
            }
        );
    }

    #[test]
    fn os_itens_levam_o_nome_do_grupo() {
        let lista = items(&Target::App("kitty".to_owned()), &grupos(), Shelf::Home);
        assert_eq!(lista[1].group.as_deref(), Some("Trabalho"));
    }

    #[test]
    fn no_grupo_oferece_renomear_e_apagar() {
        assert_eq!(
            acoes(Target::Group(1), Shelf::Home),
            vec![Action::Rename(1), Action::Remove(1)]
        );
        assert!(acoes(Target::Group(9), Shelf::Home).is_empty());
    }

    #[test]
    fn o_menu_nao_sai_da_janela() {
        assert_eq!(
            placed((10.0, 20.0), (800.0, 600.0), (200.0, 100.0)),
            (10.0, 20.0)
        );
        assert_eq!(
            placed((750.0, 580.0), (800.0, 600.0), (200.0, 100.0)),
            (600.0, 500.0)
        );
        assert_eq!(
            placed((50.0, 50.0), (100.0, 80.0), (200.0, 100.0)),
            (0.0, 0.0)
        );
    }
}
