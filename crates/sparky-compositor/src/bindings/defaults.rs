use sparky_tiling::{Direction, Fullscreen};

use crate::apps::Apps;
use crate::bindings::action::Action;

const DIRECTIONS: [(&str, Direction); 4] = [
    ("LEFT", Direction::Left),
    ("RIGHT", Direction::Right),
    ("UP", Direction::Up),
    ("DOWN", Direction::Down),
];

pub fn sparky(apps: &Apps) -> Vec<(String, Action)> {
    let mut binds = applications(apps);
    binds.extend(windows());
    binds.extend(directions());
    binds.extend(workspaces());
    binds
}

fn applications(apps: &Apps) -> Vec<(String, Action)> {
    vec![
        bind("SUPER + RETURN", Action::Spawn(apps.terminal.clone())),
        bind("SUPER + SPACE", Action::Spawn(apps.launcher.clone())),
        bind("SUPER + SHIFT + F", Action::Spawn(apps.files.clone())),
        bind("SUPER + ALT + SPACE", Action::Spawn(apps.settings.clone())),
        bind("PRINT", Action::Spawn(format!("{} region", apps.screenshot))),
        bind("SHIFT + PRINT", Action::Spawn(format!("{} window", apps.screenshot))),
        bind("CTRL + PRINT", Action::Spawn(format!("{} output", apps.screenshot))),
        bind("SUPER + PRINT", Action::Spawn(apps.screenshot.clone())),
        bind("SUPER + SHIFT + ESCAPE", Action::Exit),
    ]
}

fn windows() -> Vec<(String, Action)> {
    vec![
        bind("SUPER + W", Action::CloseWindow),
        bind("SUPER + Q", Action::CloseWindow),
        bind("SUPER + J", Action::ToggleSplit),
        bind("SUPER + T", Action::ToggleFloating),
        bind("SUPER + F", Action::Fullscreen(Fullscreen::Monitor)),
        bind("SUPER + ALT + F", Action::Fullscreen(Fullscreen::Maximized)),
        bind("SUPER + code:20", Action::Resize(-100, 0)),
        bind("SUPER + code:21", Action::Resize(100, 0)),
        bind("SUPER + SHIFT + code:20", Action::Resize(0, -100)),
        bind("SUPER + SHIFT + code:21", Action::Resize(0, 100)),
        bind("SUPER + TAB", Action::RelativeWorkspace(1)),
        bind("SUPER + SHIFT + TAB", Action::RelativeWorkspace(-1)),
        bind("SUPER + CTRL + TAB", Action::PreviousWorkspace),
        bind("CTRL + ALT + TAB", Action::CycleMonitor(1)),
        bind("CTRL + ALT + SHIFT + TAB", Action::CycleMonitor(-1)),
    ]
}

fn directions() -> Vec<(String, Action)> {
    DIRECTIONS
        .iter()
        .flat_map(|(key, direction)| {
            [
                bind(&format!("SUPER + {key}"), Action::Focus(*direction)),
                bind(&format!("SUPER + SHIFT + {key}"), Action::Swap(*direction)),
            ]
        })
        .collect()
}

fn workspaces() -> Vec<(String, Action)> {
    (1..=10u32)
        .flat_map(|workspace| {
            let key = format!("code:{}", workspace + 9);
            [
                bind(&format!("SUPER + {key}"), Action::Workspace(workspace)),
                bind(
                    &format!("SUPER + SHIFT + {key}"),
                    Action::MoveToWorkspace { workspace, follow: true },
                ),
                bind(
                    &format!("SUPER + SHIFT + ALT + {key}"),
                    Action::MoveToWorkspace { workspace, follow: false },
                ),
            ]
        })
        .collect()
}

fn bind(keys: &str, action: Action) -> (String, Action) {
    (keys.to_owned(), action)
}

#[cfg(test)]
mod tests {
    use super::sparky;
    use crate::apps::Apps;
    use crate::bindings::action::Action;
    use crate::bindings::combo::Combo;

    fn padrao() -> Vec<(String, Action)> {
        sparky(&Apps::with(Some("kitty"), |_| true))
    }

    fn acao(keys: &str) -> Option<Action> {
        padrao().into_iter().find(|(known, _)| known == keys).map(|(_, action)| action)
    }

    #[test]
    fn todas_as_combinacoes_padrao_sao_validas() {
        for (keys, _) in padrao() {
            assert!(Combo::parse(&keys).is_ok(), "combinação inválida: {keys}");
        }
    }

    #[test]
    fn nenhuma_combinacao_padrao_se_repete() {
        let mut keys: Vec<String> = padrao().into_iter().map(|(keys, _)| keys).collect();
        let total = keys.len();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), total);
    }

    #[test]
    fn super_enter_abre_o_terminal() {
        assert_eq!(acao("SUPER + RETURN"), Some(Action::Spawn("kitty".to_owned())));
    }

    #[test]
    fn a_tecla_do_um_vai_para_a_area_um_e_a_do_zero_para_a_dez() {
        assert_eq!(acao("SUPER + code:10"), Some(Action::Workspace(1)));
        assert_eq!(acao("SUPER + code:19"), Some(Action::Workspace(10)));
    }

    #[test]
    fn mover_sem_seguir_usa_alt() {
        assert_eq!(
            acao("SUPER + SHIFT + ALT + code:12"),
            Some(Action::MoveToWorkspace { workspace: 3, follow: false })
        );
    }

    #[test]
    fn print_captura_uma_regiao() {
        assert_eq!(acao("PRINT"), Some(Action::Spawn("sparky-shot region".to_owned())));
    }
}
