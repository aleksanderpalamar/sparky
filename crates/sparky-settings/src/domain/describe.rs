use sparky_config::Action;
use sparky_tiling::{Direction, Fullscreen};

pub fn describe(action: &Action) -> String {
    match action {
        Action::Spawn { command } => format!("Abre {command}"),
        Action::CloseWindow => "Fecha a janela".to_owned(),
        Action::ToggleFloating => "Solta ou prende a janela no mosaico".to_owned(),
        Action::Fullscreen { mode: Fullscreen::Monitor } => "Tela cheia".to_owned(),
        Action::Fullscreen { mode: Fullscreen::Maximized } => "Maximiza mantendo a barra".to_owned(),
        Action::ToggleSplit => "Troca a direção da divisão".to_owned(),
        Action::Focus { direction } => format!("Foca a janela {}", towards(*direction)),
        Action::Swap { direction } => format!("Troca de lugar com a janela {}", towards(*direction)),
        Action::Resize { width, height } => format!("Redimensiona ({}, {})", signed(*width), signed(*height)),
        Action::Workspace { workspace } => format!("Vai para a área {workspace}"),
        Action::MoveToWorkspace { workspace, follow: true } => format!("Leva a janela para a área {workspace} e vai junto"),
        Action::MoveToWorkspace { workspace, follow: false } => format!("Manda a janela para a área {workspace}"),
        Action::RelativeWorkspace { step } if *step > 0 => "Próxima área".to_owned(),
        Action::RelativeWorkspace { .. } => "Área anterior".to_owned(),
        Action::PreviousWorkspace => "Volta para a última área usada".to_owned(),
        Action::ToggleScratchpad => "Mostra ou esconde o rascunho".to_owned(),
        Action::MoveToScratchpad => "Manda a janela para o rascunho".to_owned(),
        Action::ToggleGroup => "Agrupa a janela ou desfaz o grupo".to_owned(),
        Action::LeaveGroup => "Tira a janela do grupo".to_owned(),
        Action::MoveIntoGroup { direction } => format!("Põe a janela no grupo {}", towards(*direction)),
        Action::CycleGroup { step } if *step > 0 => "Próxima janela do grupo".to_owned(),
        Action::CycleGroup { .. } => "Janela anterior do grupo".to_owned(),
        Action::SelectInGroup { index } => format!("Mostra a janela {index} do grupo"),
        Action::CycleMonitor { step } if *step > 0 => "Próximo monitor".to_owned(),
        Action::CycleMonitor { .. } => "Monitor anterior".to_owned(),
        Action::SwitchTerminal { terminal } => format!("Vai para o terminal virtual {terminal}"),
        Action::Exit => "Sai do sparky".to_owned(),
    }
}

fn towards(direction: Direction) -> &'static str {
    match direction {
        Direction::Left => "à esquerda",
        Direction::Right => "à direita",
        Direction::Up => "de cima",
        Direction::Down => "de baixo",
    }
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
        assert_eq!(describe(&Action::Spawn { command: "kitty".to_owned() }), "Abre kitty");
        assert_eq!(describe(&Action::Focus { direction: Direction::Left }), "Foca a janela à esquerda");
        assert_eq!(describe(&Action::Resize { width: 50, height: 0 }), "Redimensiona (+50, 0)");
        assert_eq!(describe(&Action::RelativeWorkspace { step: -1 }), "Área anterior");
    }

    #[test]
    fn descreve_os_grupos() {
        assert_eq!(describe(&Action::MoveIntoGroup { direction: Direction::Right }), "Põe a janela no grupo à direita");
        assert_eq!(describe(&Action::CycleGroup { step: -1 }), "Janela anterior do grupo");
        assert_eq!(describe(&Action::SelectInGroup { index: 3 }), "Mostra a janela 3 do grupo");
    }

    #[test]
    fn descreve_o_rascunho() {
        assert_eq!(describe(&Action::ToggleScratchpad), "Mostra ou esconde o rascunho");
        assert_eq!(describe(&Action::MoveToScratchpad), "Manda a janela para o rascunho");
    }

    #[test]
    fn mover_para_a_area_diz_se_vai_junto() {
        assert_eq!(
            describe(&Action::MoveToWorkspace { workspace: 3, follow: false }),
            "Manda a janela para a área 3"
        );
        assert_eq!(
            describe(&Action::MoveToWorkspace { workspace: 3, follow: true }),
            "Leva a janela para a área 3 e vai junto"
        );
    }
}
