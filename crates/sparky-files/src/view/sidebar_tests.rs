use super::sidebar::ordered_places;
use super::sidebar_device::{DeviceAction, device_action, display_name, work_label};
use sparky_storage::{Operation, Volume};

#[test]
fn ordem_visual_preserva_indices_dos_atalhos() {
    let dirs = sparky_xdg::UserDirs::new("/home/p", None);
    let places = crate::domain::places::places(&dirs);
    let ordered = ordered_places(&places);
    let indices: Vec<_> = ordered.iter().map(|(index, _)| *index).collect();
    assert_eq!(indices, vec![0, 2, 5, 4, 6, 3]);
    assert!(
        ordered
            .iter()
            .all(|(_, place)| place.key != "place-desktop")
    );
}

#[test]
fn telefone_mostra_nome_sem_tipo_ou_estado() {
    let mut volume = Volume {
        path: "mtp://Android_Test/".to_owned(),
        label: "Samsung Galaxy S22 Ultra".to_owned(),
        filesystem: "mtp".to_owned(),
        size: String::new(),
        mountpoint: None,
        removable: true,
    };
    assert_eq!(display_name(&volume), "Samsung Galaxy S22 Ultra");
    volume.mountpoint = Some("/run/user/1000/gvfs/mtp:host=Android_Test".to_owned());
    assert_eq!(display_name(&volume), "Samsung Galaxy S22 Ultra");
}

#[test]
fn disco_prioriza_o_nome_e_usa_capacidade_quando_sem_descricao() {
    let mut volume = Volume {
        path: "/dev/sdb1".to_owned(),
        label: "Dados".to_owned(),
        filesystem: "ext4".to_owned(),
        size: "299,1G".to_owned(),
        mountpoint: None,
        removable: false,
    };
    assert_eq!(display_name(&volume), "Dados");
    volume.mountpoint = Some("/run/media/Dados".to_owned());
    assert_eq!(display_name(&volume), "Dados");
    volume.label.clear();
    assert_eq!(display_name(&volume), "Volume de 299,1G");
}

#[test]
fn consulta_periodica_nao_altera_a_indicacao_de_atividade() {
    assert_eq!(work_label(Some(&Operation::List)), work_label(None));
}

#[test]
fn montar_e_desmontar_continuam_com_indicacao_de_progresso() {
    let device = "/dev/fixture".to_owned();
    assert_eq!(
        work_label(Some(&Operation::Mount(device.clone()))),
        Some(crate::i18n::text("mounting"))
    );
    assert_eq!(
        work_label(Some(&Operation::Unmount(device))),
        Some(crate::i18n::text("unmounting"))
    );
}

#[test]
fn ejetar_so_e_oferecido_para_unidade_removivel_de_bloco() {
    let mut volume = Volume {
        path: "/dev/sdb1".to_owned(),
        label: "USB".to_owned(),
        filesystem: "vfat".to_owned(),
        size: "32G".to_owned(),
        mountpoint: None,
        removable: true,
    };
    assert_eq!(device_action(&volume), Some(DeviceAction::Eject));
    volume.mountpoint = Some("/run/media/USB".to_owned());
    assert_eq!(device_action(&volume), Some(DeviceAction::Unmount));
    volume.removable = false;
    assert_eq!(device_action(&volume), Some(DeviceAction::Unmount));
    volume.path = "mtp://Android/".to_owned();
    assert_eq!(device_action(&volume), Some(DeviceAction::Unmount));
    volume.mountpoint = None;
    assert_eq!(device_action(&volume), None);
}

#[test]
fn ejecao_tem_indicacao_distinta_de_montagem() {
    assert_eq!(
        work_label(Some(&Operation::Eject("/dev/sdb1".to_owned()))),
        Some(crate::i18n::text("ejecting"))
    );
}
