use sparky_config::Action;
use sparky_tiling::{Direction, Fullscreen};

pub fn describe(action: &Action) -> String {
    match action {
        Action::Spawn { command } => crate::i18n::with("action-spawn", "value", command),
        Action::CloseWindow => crate::i18n::text("action-close"),
        Action::ToggleFloating => crate::i18n::text("action-floating"),
        Action::Fullscreen {
            mode: Fullscreen::Monitor,
        } => crate::i18n::text("action-fullscreen"),
        Action::Fullscreen {
            mode: Fullscreen::Maximized,
        } => crate::i18n::text("action-maximized"),
        Action::ToggleSplit => crate::i18n::text("action-split"),
        Action::Focus { direction } => {
            crate::i18n::with("action-focus", "value", &towards(*direction))
        }
        Action::Swap { direction } => {
            crate::i18n::with("action-swap", "value", &towards(*direction))
        }
        Action::Resize { width, height } => {
            let mut args = fluent_bundle::FluentArgs::new();
            args.set("width", signed(*width));
            args.set("height", signed(*height));
            crate::i18n::message("action-resize", Some(&args))
        }
        Action::Workspace { workspace } => number("action-workspace", *workspace),
        Action::MoveToWorkspace {
            workspace,
            follow: true,
        } => number("action-move-follow", *workspace),
        Action::MoveToWorkspace {
            workspace,
            follow: false,
        } => number("action-move", *workspace),
        Action::RelativeWorkspace { step } if *step > 0 => {
            crate::i18n::text("action-next-workspace")
        }
        Action::RelativeWorkspace { .. } => crate::i18n::text("action-previous-workspace"),
        Action::PreviousWorkspace => crate::i18n::text("action-last-workspace"),
        Action::ToggleScratchpad => crate::i18n::text("action-toggle-scratchpad"),
        Action::MoveToScratchpad => crate::i18n::text("action-move-scratchpad"),
        Action::ToggleGroup => crate::i18n::text("action-toggle-group"),
        Action::LeaveGroup => crate::i18n::text("action-leave-group"),
        Action::MoveIntoGroup { direction } => {
            crate::i18n::with("action-move-group", "value", &towards(*direction))
        }
        Action::CycleGroup { step } if *step > 0 => crate::i18n::text("action-next-group"),
        Action::CycleGroup { .. } => crate::i18n::text("action-previous-group"),
        Action::SelectInGroup { index } => number("action-select-group", *index),
        Action::CycleMonitor { step } if *step > 0 => crate::i18n::text("action-next-monitor"),
        Action::CycleMonitor { .. } => crate::i18n::text("action-previous-monitor"),
        Action::SwitchTerminal { terminal } => number("action-terminal", *terminal),
        Action::Exit => crate::i18n::text("action-exit"),
    }
}

fn towards(direction: Direction) -> String {
    let key = match direction {
        Direction::Left => "direction-left",
        Direction::Right => "direction-right",
        Direction::Up => "direction-up",
        Direction::Down => "direction-down",
    };
    crate::i18n::text(key)
}

fn number(key: &str, value: impl ToString) -> String {
    crate::i18n::with(key, "value", &value.to_string())
}

fn signed(number: i32) -> String {
    match number > 0 {
        true => format!("+{number}"),
        false => number.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::describe;
    use sparky_config::Action;
    use sparky_tiling::Direction;

    #[test]
    fn descreve_as_acoes_em_portugues() {
        assert_eq!(
            describe(&Action::Spawn {
                command: "kitty".to_owned()
            }),
            "Abre kitty"
        );
        assert_eq!(
            describe(&Action::Focus {
                direction: Direction::Left
            }),
            "Foca a janela à esquerda"
        );
        assert_eq!(
            describe(&Action::Resize {
                width: 50,
                height: 0
            }),
            "Redimensiona (+50, 0)"
        );
        assert_eq!(
            describe(&Action::RelativeWorkspace { step: -1 }),
            "Área anterior"
        );
    }

    #[test]
    fn descreve_os_grupos() {
        assert_eq!(
            describe(&Action::MoveIntoGroup {
                direction: Direction::Right
            }),
            "Põe a janela no grupo à direita"
        );
        assert_eq!(
            describe(&Action::CycleGroup { step: -1 }),
            "Janela anterior do grupo"
        );
        assert_eq!(
            describe(&Action::SelectInGroup { index: 3 }),
            "Mostra a janela 3 do grupo"
        );
    }

    #[test]
    fn descreve_o_rascunho() {
        assert_eq!(
            describe(&Action::ToggleScratchpad),
            "Mostra ou esconde o rascunho"
        );
        assert_eq!(
            describe(&Action::MoveToScratchpad),
            "Manda a janela para o rascunho"
        );
    }

    #[test]
    fn mover_para_a_area_diz_se_vai_junto() {
        assert_eq!(
            describe(&Action::MoveToWorkspace {
                workspace: 3,
                follow: false
            }),
            "Manda a janela para a área 3"
        );
        assert_eq!(
            describe(&Action::MoveToWorkspace {
                workspace: 3,
                follow: true
            }),
            "Leva a janela para a área 3 e vai junto"
        );
    }
}
