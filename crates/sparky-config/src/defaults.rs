use sparky_tiling::{Direction, Fullscreen};

use crate::action::{Action, Bind};
use crate::schema::Apps;

const DIRECTIONS: [(&str, Direction); 4] = [
    ("LEFT", Direction::Left),
    ("RIGHT", Direction::Right),
    ("UP", Direction::Up),
    ("DOWN", Direction::Down),
];

pub fn sparky(apps: &Apps, terminal: &str) -> Vec<Bind> {
    let mut binds = applications(apps, terminal);
    binds.extend(windows());
    binds.extend(directions());
    binds.extend(workspaces());
    binds
}

fn applications(apps: &Apps, terminal: &str) -> Vec<Bind> {
    vec![
        bind("SUPER + RETURN", spawn(terminal)),
        bind("SUPER + SPACE", spawn(&apps.launcher)),
        bind("SUPER + SHIFT + F", spawn(&apps.files)),
        bind("SUPER + ALT + SPACE", spawn(&apps.settings)),
        bind("PRINT", spawn(&format!("{} region", apps.screenshot))),
        bind("SHIFT + PRINT", spawn(&format!("{} window", apps.screenshot))),
        bind("CTRL + PRINT", spawn(&format!("{} output", apps.screenshot))),
        bind("SUPER + PRINT", spawn(&apps.screenshot)),
        bind("SUPER + SHIFT + ESCAPE", Action::Exit),
    ]
}

fn windows() -> Vec<Bind> {
    vec![
        bind("SUPER + W", Action::CloseWindow),
        bind("SUPER + Q", Action::CloseWindow),
        bind("SUPER + J", Action::ToggleSplit),
        bind("SUPER + T", Action::ToggleFloating),
        bind("SUPER + F", Action::Fullscreen { mode: Fullscreen::Monitor }),
        bind("SUPER + ALT + F", Action::Fullscreen { mode: Fullscreen::Maximized }),
        bind("SUPER + code:20", Action::Resize { width: -100, height: 0 }),
        bind("SUPER + code:21", Action::Resize { width: 100, height: 0 }),
        bind("SUPER + SHIFT + code:20", Action::Resize { width: 0, height: -100 }),
        bind("SUPER + SHIFT + code:21", Action::Resize { width: 0, height: 100 }),
        bind("SUPER + TAB", Action::RelativeWorkspace { step: 1 }),
        bind("SUPER + SHIFT + TAB", Action::RelativeWorkspace { step: -1 }),
        bind("SUPER + CTRL + TAB", Action::PreviousWorkspace),
        bind("SUPER + S", Action::ToggleScratchpad),
        bind("SUPER + code:49", Action::ToggleScratchpad),
        bind("SUPER + ALT + S", Action::MoveToScratchpad),
        bind("SUPER + SHIFT + code:49", Action::MoveToScratchpad),
        bind("CTRL + ALT + TAB", Action::CycleMonitor { step: 1 }),
        bind("CTRL + ALT + SHIFT + TAB", Action::CycleMonitor { step: -1 }),
    ]
}

fn directions() -> Vec<Bind> {
    DIRECTIONS
        .iter()
        .flat_map(|(key, direction)| {
            [
                bind(&format!("SUPER + {key}"), Action::Focus { direction: *direction }),
                bind(&format!("SUPER + SHIFT + {key}"), Action::Swap { direction: *direction }),
            ]
        })
        .collect()
}

fn workspaces() -> Vec<Bind> {
    (1..=10u32)
        .flat_map(|workspace| {
            let key = format!("code:{}", workspace + 9);
            [
                bind(&format!("SUPER + {key}"), Action::Workspace { workspace }),
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

fn bind(keys: &str, action: Action) -> Bind {
    Bind::new(keys, action)
}

fn spawn(command: &str) -> Action {
    Action::Spawn {
        command: command.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::sparky;
    use crate::action::{Action, Bind};
    use crate::combo::Combo;
    use crate::schema::Apps;

    fn padrao() -> Vec<Bind> {
        sparky(&Apps::default(), "kitty")
    }

    fn acao(keys: &str) -> Option<Action> {
        padrao().into_iter().find(|bind| bind.keys == keys).map(|bind| bind.action)
    }

    #[test]
    fn todas_as_combinacoes_padrao_sao_validas() {
        for bind in padrao() {
            assert!(Combo::parse(&bind.keys).is_ok(), "combinação inválida: {}", bind.keys);
        }
    }

    #[test]
    fn nenhuma_combinacao_padrao_se_repete() {
        let mut keys: Vec<String> = padrao().into_iter().map(|bind| bind.keys).collect();
        let total = keys.len();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), total);
    }

    #[test]
    fn super_enter_abre_o_terminal() {
        assert_eq!(acao("SUPER + RETURN"), Some(Action::Spawn { command: "kitty".to_owned() }));
    }

    #[test]
    fn a_tecla_do_um_vai_para_a_area_um_e_a_do_zero_para_a_dez() {
        assert_eq!(acao("SUPER + code:10"), Some(Action::Workspace { workspace: 1 }));
        assert_eq!(acao("SUPER + code:19"), Some(Action::Workspace { workspace: 10 }));
    }

    #[test]
    fn mover_sem_seguir_usa_alt() {
        assert_eq!(
            acao("SUPER + SHIFT + ALT + code:12"),
            Some(Action::MoveToWorkspace { workspace: 3, follow: false })
        );
    }

    #[test]
    fn super_s_e_super_crase_alternam_o_rascunho() {
        assert_eq!(acao("SUPER + S"), Some(Action::ToggleScratchpad));
        assert_eq!(acao("SUPER + code:49"), Some(Action::ToggleScratchpad));
    }

    #[test]
    fn super_alt_s_e_super_shift_crase_mandam_para_o_rascunho() {
        assert_eq!(acao("SUPER + ALT + S"), Some(Action::MoveToScratchpad));
        assert_eq!(acao("SUPER + SHIFT + code:49"), Some(Action::MoveToScratchpad));
    }

    #[test]
    fn print_captura_uma_regiao() {
        assert_eq!(acao("PRINT"), Some(Action::Spawn { command: "sparky-shot region".to_owned() }));
    }
}
