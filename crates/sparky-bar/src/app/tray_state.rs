use std::sync::Arc;

use iced::window;

use crate::domain::tray::{Tray, TrayChange};
use crate::ports::tray::TrayCommands;

#[derive(Debug, Clone)]
pub struct TrayClient(pub Arc<dyn TrayCommands>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenMenu {
    pub window: window::Id,
    pub address: String,
}

#[derive(Debug, Default)]
pub struct TrayState {
    tray: Tray,
    client: Option<TrayClient>,
    menu: Option<OpenMenu>,
}

impl TrayState {
    pub fn tray(&self) -> &Tray {
        &self.tray
    }

    pub fn apply(&mut self, change: TrayChange) -> Option<OpenMenu> {
        self.tray.apply(change);
        let invalid = self
            .menu
            .as_ref()
            .is_some_and(|menu| self.tray.item(&menu.address).is_none_or(|item| item.menu.is_empty()));
        match invalid {
            true => self.menu.take(),
            false => None,
        }
    }

    pub fn client(&self) -> Option<TrayClient> {
        self.client.clone()
    }

    pub fn connect(&mut self, client: TrayClient) {
        self.client = Some(client);
    }

    pub fn menu(&self) -> Option<&OpenMenu> {
        self.menu.as_ref()
    }

    pub fn open_menu(&mut self, menu: OpenMenu) -> Option<OpenMenu> {
        self.menu.replace(menu)
    }

    pub fn close_menu(&mut self) -> Option<OpenMenu> {
        self.menu.take()
    }
}

#[cfg(test)]
mod tests {
    use super::{OpenMenu, TrayState};
    use crate::domain::tray::{PrimaryAction, TrayChange, TrayIcon, TrayItem};
    use iced::window;

    fn item(address: &str) -> TrayItem {
        TrayItem {
            address: address.to_owned(),
            title: String::new(),
            icon: TrayIcon::Missing,
            primary_action: PrimaryAction::Activate,
            menu_path: None,
            menu: Vec::new(),
        }
    }

    #[test]
    fn abrir_outro_menu_devolve_o_anterior_para_fechar() {
        let mut state = TrayState::default();
        let primeiro = OpenMenu { window: window::Id::unique(), address: ":1.1".to_owned() };
        assert_eq!(state.open_menu(primeiro.clone()), None);
        let segundo = OpenMenu { window: window::Id::unique(), address: ":1.2".to_owned() };
        assert_eq!(state.open_menu(segundo), Some(primeiro));
    }

    #[test]
    fn o_item_que_sai_leva_o_menu_junto() {
        let mut state = TrayState::default();
        state.apply(TrayChange::Added(item(":1.1")));
        state.open_menu(OpenMenu { window: window::Id::unique(), address: ":1.1".to_owned() });
        state.apply(TrayChange::Removed(":1.1".to_owned()));
        assert_eq!(state.menu(), None);
        assert!(state.tray().items().is_empty());
    }
}
