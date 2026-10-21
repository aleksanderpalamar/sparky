use std::rc::Rc;

use sparky_ui::notice::Notice;

use crate::app::message::Message;
use crate::app::test_support::{Comandos, Sparky, enter, painel_com};
use crate::app::update::update;
use crate::domain::section::Section;
use crate::ports::fake::FakeCommands;

const CALL: &str = "lsblk -J -o NAME,PATH,SIZE,TYPE,MOUNTPOINT,LABEL,RM,HOTPLUG,FSTYPE";
const USB: &str = r#"{"blockdevices":[{"name":"sdb1","path":"/dev/sdb1","type":"part","size":"8G","label":"USB","rm":true,"fstype":"vfat"}]}"#;

#[test]
fn montar_rele_os_volumes_sem_gravar_a_configuracao_do_sparky() {
    let commands = Comandos(Rc::new(FakeCommands::new().with(CALL, USB).accepting("udisksctl mount -b /dev/sdb1")));
    let (mut settings, arquivo) = painel_com(None, Sparky::default(), commands.clone());
    let _ = update(&mut settings, Message::Section(Section::Media));
    assert_eq!(settings.count(), 1);
    enter(&mut settings);
    assert!(commands.0.calls().contains(&"udisksctl mount -b /dev/sdb1".to_owned()));
    assert_eq!(commands.0.calls().iter().filter(|call| *call == CALL).count(), 2);
    assert_eq!(*arquivo.0.borrow(), None);
}

#[test]
fn recusas_de_montagem_aparecem_no_aviso() {
    let commands = Comandos(Rc::new(FakeCommands::new().with(CALL, USB)));
    let (mut settings, _) = painel_com(None, Sparky::default(), commands);
    let _ = update(&mut settings, Message::Section(Section::Media));
    enter(&mut settings);
    assert!(matches!(&settings.notice, Some(Notice::Error(text)) if text.contains("programa não encontrado")));
}
