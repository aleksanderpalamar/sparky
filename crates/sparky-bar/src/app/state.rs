use std::sync::Arc;

use chrono::NaiveDateTime;
use sparky_ipc::IpcError;
use sparky_theme::Skin;

use crate::app::tray_state::TrayState;
use crate::domain::clock::{self, ClockFormat};
use crate::domain::slots::{self, Slot};
use crate::ports::workspaces::Workspaces;

pub struct Bar {
    pub center: super::center_state::CenterState,
    pub panels: super::panel_state::Panels,
    skin: Skin,
    workspaces: Arc<dyn Workspaces>,
    slots: Vec<Slot>,
    clock_format: ClockFormat,
    now: NaiveDateTime,
    tray: TrayState,
}

impl Bar {
    pub fn new(skin: Skin, workspaces: Arc<dyn Workspaces>, now: NaiveDateTime) -> Self {
        let mut bar = Self {
            center: super::center_state::CenterState::new(now.date()),
            panels: super::panel_state::Panels::default(),
            skin,
            workspaces,
            slots: slots::slots(&[]),
            clock_format: ClockFormat::default(),
            now,
            tray: TrayState::default(),
        };
        bar.refresh_workspaces();
        bar
    }

    pub fn skin(&self) -> &Skin {
        &self.skin
    }

    pub fn apply_palette(&mut self, palette: sparky_theme::Palette) {
        self.skin = Skin::new(palette);
    }

    pub fn slots(&self) -> &[Slot] {
        &self.slots
    }

    pub fn tray(&self) -> &TrayState {
        &self.tray
    }

    pub fn tray_mut(&mut self) -> &mut TrayState {
        &mut self.tray
    }

    pub fn clock(&self) -> String {
        clock::render(self.clock_format, &self.now)
    }

    pub fn refresh_workspaces(&mut self) {
        let found = self.workspaces.list().unwrap_or_default();
        self.slots = slots::slots(&found);
    }

    pub fn tick(&mut self, now: NaiveDateTime) {
        self.now = now;
        self.center.calendar.update_today(now.date());
    }

    pub fn toggle_clock(&mut self) {
        self.clock_format = self.clock_format.toggled();
    }

    pub fn focus(&self, id: u32) -> Result<(), IpcError> {
        self.workspaces.focus(id)
    }
}

#[cfg(test)]
mod tests {
    use super::Bar;
    use crate::domain::slots::SlotState;
    use crate::ports::fake::FakeWorkspaces;
    use chrono::NaiveDate;
    use sparky_ipc::WorkspaceInfo;
    use sparky_theme::Skin;
    use std::sync::Arc;

    fn agora(minuto: u32) -> chrono::NaiveDateTime {
        let Some(momento) =
            NaiveDate::from_ymd_opt(2026, 10, 7).and_then(|d| d.and_hms_opt(9, minuto, 0))
        else {
            panic!("momento inválido");
        };
        momento
    }

    fn barra(workspaces: FakeWorkspaces) -> (Bar, Arc<FakeWorkspaces>) {
        let workspaces = Arc::new(workspaces);
        (
            Bar::new(Skin::default(), workspaces.clone(), agora(5)),
            workspaces,
        )
    }

    fn area_dois() -> WorkspaceInfo {
        WorkspaceInfo {
            id: 2,
            monitor: "DP-1".to_owned(),
            windows: 1,
            visible: true,
            focused: true,
        }
    }

    #[test]
    fn nasce_com_as_areas_do_sparky() {
        let (bar, _) = barra(FakeWorkspaces::with(vec![area_dois()]));
        assert_eq!(bar.slots()[1].state, SlotState::Active);
        assert_eq!(bar.slots()[0].state, SlotState::Empty);
    }

    #[test]
    fn sem_sparky_mostra_as_cinco_areas_vazias() {
        let (bar, _) = barra(FakeWorkspaces::default());
        assert_eq!(bar.slots().len(), 5);
        assert!(
            bar.slots()
                .iter()
                .all(|slot| slot.state == SlotState::Empty)
        );
    }

    #[test]
    fn o_relogio_alterna_entre_hora_e_data() {
        let (mut bar, _) = barra(FakeWorkspaces::default());
        assert_eq!(bar.clock(), "quarta 09:05");
        bar.toggle_clock();
        assert_eq!(bar.clock(), "quarta, 07 de outubro de 2026");
    }

    #[test]
    fn o_tique_atualiza_a_hora() {
        let (mut bar, _) = barra(FakeWorkspaces::default());
        bar.tick(agora(6));
        assert_eq!(bar.clock(), "quarta 09:06");
    }

    #[test]
    fn ir_para_uma_area_pede_ao_sparky() {
        let (bar, workspaces) = barra(FakeWorkspaces::default());
        assert_eq!(bar.focus(3), Ok(()));
        assert_eq!(workspaces.focused(), vec![3]);
    }
}
