use super::message::{Message, TrashEvent};
use super::state::Mode;
use super::test_scene::Cena;
use crate::adapters::home_trash::HomeTrash;
use crate::domain::sidebar_location::SidebarLocation;

#[test]
fn paginas_virtuais_nao_executam_acoes_da_lixeira_anterior() {
    for page in [
        SidebarLocation::Recent,
        SidebarLocation::Favorites,
        SidebarLocation::Network,
    ] {
        let mut scene = Cena::nova();
        let file = scene.work().join("a.txt");
        let trash = HomeTrash::new(scene.dir.path().join("Trash"));
        trash.put(&file).unwrap();
        scene.mandar(Message::Trash(TrashEvent::Open));
        assert!(scene.files.in_trash());
        scene.files.listing.select_name("a.txt");
        scene.files.open_sidebar(page);
        assert_eq!(scene.files.sidebar_page, Some(page));
        assert!(!scene.files.in_trash());
        scene.mandar(Message::Trash(TrashEvent::RestoreSelected));
        scene.mandar(Message::Trash(TrashEvent::RestoreAll));
        scene.mandar(Message::Trash(TrashEvent::Empty));
        assert!(!file.exists());
        assert!(scene.dir.path().join("Trash/files/a.txt").exists());
        assert!(matches!(scene.files.mode, Mode::Browse));
        scene.mandar(Message::Trash(TrashEvent::Open));
        assert!(scene.files.in_trash());
        scene.mandar(Message::Trash(TrashEvent::RestoreSelected));
        assert!(file.exists());
    }
}
