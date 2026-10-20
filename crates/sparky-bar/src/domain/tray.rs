use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayIcon {
    File(PathBuf),
    Pixels { width: u32, height: u32, rgba: Vec<u8> },
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayRequest {
    Activate { address: String },
    Select { address: String, menu_path: String, entry: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimaryAction {
    Activate,
    Menu,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuEntry {
    pub id: i32,
    pub label: String,
    pub enabled: bool,
    pub separator: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayItem {
    pub address: String,
    pub title: String,
    pub icon: TrayIcon,
    pub primary_action: PrimaryAction,
    pub menu_path: Option<String>,
    pub menu: Vec<MenuEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayChange {
    Added(TrayItem),
    Icon { address: String, icon: TrayIcon },
    Title { address: String, title: String },
    Menu { address: String, menu: Vec<MenuEntry> },
    Removed(String),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tray {
    items: Vec<TrayItem>,
}

impl Tray {
    pub fn items(&self) -> &[TrayItem] {
        &self.items
    }

    pub fn item(&self, address: &str) -> Option<&TrayItem> {
        self.items.iter().find(|item| item.address == address)
    }

    pub fn apply(&mut self, change: TrayChange) {
        match change {
            TrayChange::Added(item) => self.add(item),
            TrayChange::Icon { address, icon } => self.edit(&address, |item| item.icon = icon),
            TrayChange::Title { address, title } => self.edit(&address, |item| item.title = title),
            TrayChange::Menu { address, menu } => self.edit(&address, |item| item.menu = menu),
            TrayChange::Removed(address) => self.items.retain(|item| item.address != address),
        }
    }

    fn add(&mut self, item: TrayItem) {
        match self.items.iter_mut().find(|known| known.address == item.address) {
            Some(known) => *known = item,
            None => self.items.push(item),
        }
    }

    fn edit(&mut self, address: &str, change: impl FnOnce(&mut TrayItem)) {
        if let Some(item) = self.items.iter_mut().find(|item| item.address == address) {
            change(item);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MenuEntry, PrimaryAction, Tray, TrayChange, TrayIcon, TrayItem};

    fn item(address: &str) -> TrayItem {
        TrayItem {
            address: address.to_owned(),
            title: address.to_owned(),
            icon: TrayIcon::Missing,
            primary_action: PrimaryAction::Activate,
            menu_path: None,
            menu: Vec::new(),
        }
    }

    #[test]
    fn os_itens_ficam_na_ordem_em_que_chegam() {
        let mut tray = Tray::default();
        tray.apply(TrayChange::Added(item(":1.10")));
        tray.apply(TrayChange::Added(item(":1.20")));
        let enderecos: Vec<&str> = tray.items().iter().map(|item| item.address.as_str()).collect();
        assert_eq!(enderecos, vec![":1.10", ":1.20"]);
    }

    #[test]
    fn item_repetido_substitui_o_antigo() {
        let mut tray = Tray::default();
        tray.apply(TrayChange::Added(item(":1.10")));
        let mut novo = item(":1.10");
        novo.title = "Discord".to_owned();
        tray.apply(TrayChange::Added(novo));
        assert_eq!(tray.items().len(), 1);
        assert_eq!(tray.items()[0].title, "Discord");
    }

    #[test]
    fn atualiza_icone_titulo_e_menu() {
        let mut tray = Tray::default();
        tray.apply(TrayChange::Added(item(":1.10")));
        let pixels = TrayIcon::Pixels { width: 1, height: 1, rgba: vec![1, 2, 3, 4] };
        tray.apply(TrayChange::Icon { address: ":1.10".to_owned(), icon: pixels.clone() });
        tray.apply(TrayChange::Title { address: ":1.10".to_owned(), title: "Steam".to_owned() });
        let entrada = MenuEntry { id: 3, label: "Sair".to_owned(), enabled: true, separator: false };
        tray.apply(TrayChange::Menu { address: ":1.10".to_owned(), menu: vec![entrada.clone()] });
        let Some(found) = tray.item(":1.10") else {
            panic!("item sumiu");
        };
        assert_eq!(found.icon, pixels);
        assert_eq!(found.title, "Steam");
        assert_eq!(found.menu, vec![entrada]);
    }

    #[test]
    fn mudanca_em_item_desconhecido_e_ignorada() {
        let mut tray = Tray::default();
        tray.apply(TrayChange::Title { address: ":9.9".to_owned(), title: "x".to_owned() });
        assert!(tray.items().is_empty());
    }

    #[test]
    fn remover_tira_o_item() {
        let mut tray = Tray::default();
        tray.apply(TrayChange::Added(item(":1.10")));
        tray.apply(TrayChange::Removed(":1.10".to_owned()));
        assert_eq!(tray.item(":1.10"), None);
    }
}
