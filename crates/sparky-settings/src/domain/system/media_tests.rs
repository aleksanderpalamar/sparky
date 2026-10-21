use crate::domain::system::media::Media;
use crate::domain::system::volumes;
use crate::domain::system::{Change, Control, Request, RowId};
use crate::domain::value::Step;
use crate::ports::fake::FakeCommands;

pub const CALL: &str = "lsblk -J -o NAME,PATH,SIZE,TYPE,MOUNTPOINT,LABEL,RM,HOTPLUG,FSTYPE";
pub const DEVICES: &str = r#"{"blockdevices":[
 {"name":"sda","path":"/dev/sda","type":"disk","rm":true,"children":[
   {"name":"sda1","path":"/dev/sda1","type":"part","size":"32G","fstype":"vfat","label":"USB","mountpoint":null},
   {"name":"sda2","path":"/dev/sda2","type":"part","fstype":"swap"}]},
 {"name":"nvme0n1p1","path":"/dev/nvme0n1p1","type":"part","fstype":"ext4","mountpoint":"/","hotplug":true},
 {"name":"sdb1","path":"/dev/sdb1","type":"part","fstype":"ext4","size":"64G","label":"BACKUP","mountpoint":"/run/media/ana/BACKUP"}
 ]}"#;

#[test]
fn herda_removivel_e_deixa_o_sistema_e_a_swap_de_fora() {
    let volumes = volumes::parse(DEVICES);
    assert!(volumes.is_ok());
    let volumes = volumes.unwrap_or_default();
    assert_eq!(volumes.len(), 2);
    assert_eq!(volumes[0].path, "/dev/sda1");
    assert!(volumes[0].removable);
    assert_eq!(volumes[1].name(), "BACKUP");
    assert!(volumes[1].mounted());
}

#[test]
fn linhas_identificam_o_volume_e_a_montagem() {
    let media = Media::load(&FakeCommands::new().with(CALL, DEVICES));
    let rows = media.rows();
    assert!(rows[0].label.contains("USB"));
    assert!(rows[0].label.contains("32G"));
    assert_eq!(rows[0].control, Control::Toggle(false));
    assert_eq!(rows[1].control, Control::Toggle(true));
    assert!(rows[1].value.contains("/run/media/ana/BACKUP"));
}

#[test]
fn enter_alterna_e_setas_pedem_um_estado_especifico() {
    let media = Media::load(&FakeCommands::new().with(CALL, DEVICES));
    assert_eq!(media.request(RowId::Volume(0), Change::Activate), Some(Request::MountVolume("/dev/sda1".to_owned())));
    assert_eq!(media.request(RowId::Volume(1), Change::Activate), Some(Request::UnmountVolume("/dev/sdb1".to_owned())));
    assert_eq!(media.request(RowId::Volume(0), Change::Step(Step::Less)), None);
    assert_eq!(media.request(RowId::Volume(1), Change::Step(Step::More)), None);
    assert_eq!(media.request(RowId::Volume(99), Change::Activate), None);
    assert_eq!(media.request(RowId::Volume(0), Change::Text("/dev/outro".to_owned())), None);
}

#[test]
fn falhas_de_consulta_e_json_invalido_ficam_visiveis() {
    assert_eq!(Media::load(&FakeCommands::new()).rows()[0].control, Control::Reading);
    let media = Media::load(&FakeCommands::new().with(CALL, "invalido"));
    assert!(media.rows()[0].value.contains("resposta inválida"));
    assert!(volumes::parse("invalido").is_err());
}

#[test]
fn pedidos_usam_udisksctl_sem_interpretador() {
    assert_eq!(Request::MountVolume("/dev/sda1".to_owned()).command(), ("udisksctl", vec!["mount".to_owned(), "-b".to_owned(), "/dev/sda1".to_owned()]));
    assert_eq!(Request::UnmountVolume("/dev/sda1".to_owned()).command(), ("udisksctl", vec!["unmount".to_owned(), "-b".to_owned(), "/dev/sda1".to_owned()]));
}

#[test]
fn ignora_dispositivos_sem_caminho_valido_ou_sem_sistema_de_arquivos() {
    let output = r#"{"blockdevices":[{"name":"zram0","path":"/dev/zram0","type":"disk","rm":true,"fstype":"swap"},{"name":"usb","path":"--help","type":"disk","rm":true,"fstype":"vfat"},{"name":"sdc","path":"/dev/sdc","type":"disk","rm":true}]}"#;
    assert_eq!(volumes::parse(output).unwrap_or_default().len(), 0);
}
