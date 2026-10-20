use sparky_ipc::WorkspaceInfo;

const ALWAYS_SHOWN: u32 = 5;
const LAST_NUMBERED: u32 = 10;
const ACTIVE_MARK: &str = "●";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotState {
    Active,
    Occupied,
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot {
    pub id: u32,
    pub state: SlotState,
}

impl Slot {
    pub fn label(&self) -> String {
        match (self.state, self.id) {
            (SlotState::Active, _) => ACTIVE_MARK.to_owned(),
            (_, LAST_NUMBERED) => "0".to_owned(),
            (_, id) => id.to_string(),
        }
    }
}

pub fn slots(workspaces: &[WorkspaceInfo]) -> Vec<Slot> {
    let mut ids: Vec<u32> = (1..=ALWAYS_SHOWN).collect();
    ids.extend(workspaces.iter().map(|workspace| workspace.id).filter(|id| numbered(*id)));
    ids.sort_unstable();
    ids.dedup();
    ids.into_iter()
        .map(|id| Slot {
            id,
            state: state_of(id, workspaces),
        })
        .collect()
}

fn numbered(id: u32) -> bool {
    (1..=LAST_NUMBERED).contains(&id)
}

fn state_of(id: u32, workspaces: &[WorkspaceInfo]) -> SlotState {
    let Some(workspace) = workspaces.iter().find(|workspace| workspace.id == id) else {
        return SlotState::Empty;
    };
    match (workspace.focused, workspace.windows > 0) {
        (true, _) => SlotState::Active,
        (false, true) => SlotState::Occupied,
        (false, false) => SlotState::Empty,
    }
}

#[cfg(test)]
mod tests {
    use super::{Slot, SlotState, slots};
    use sparky_ipc::WorkspaceInfo;

    fn area(id: u32, windows: usize, focused: bool) -> WorkspaceInfo {
        WorkspaceInfo {
            id,
            monitor: "HDMI-A-1".to_owned(),
            windows,
            visible: focused,
            focused,
        }
    }

    fn ids(found: &[Slot]) -> Vec<u32> {
        found.iter().map(|slot| slot.id).collect()
    }

    #[test]
    fn as_cinco_primeiras_aparecem_sempre() {
        assert_eq!(ids(&slots(&[])), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn areas_alem_da_quinta_aparecem_quando_existem() {
        assert_eq!(ids(&slots(&[area(1, 1, false), area(8, 0, true)])), vec![1, 2, 3, 4, 5, 8]);
    }

    #[test]
    fn areas_acima_de_dez_ficam_de_fora() {
        assert_eq!(ids(&slots(&[area(11, 1, true)])), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn distingue_ativa_ocupada_e_vazia() {
        let found = slots(&[area(1, 0, true), area(2, 3, false)]);
        assert_eq!(found[0].state, SlotState::Active);
        assert_eq!(found[1].state, SlotState::Occupied);
        assert_eq!(found[2].state, SlotState::Empty);
    }

    #[test]
    fn a_area_ativa_vira_um_ponto_e_a_dez_vira_zero() {
        let active = Slot { id: 3, state: SlotState::Active };
        let ten = Slot { id: 10, state: SlotState::Occupied };
        let four = Slot { id: 4, state: SlotState::Empty };
        assert_eq!(active.label(), "●");
        assert_eq!(ten.label(), "0");
        assert_eq!(four.label(), "4");
    }
}
