use super::sidebar::{device_detail, ordered_places, work_label};
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
fn telefone_indica_mtp_sem_inventar_capacidade() {
    let mut volume = Volume {
        path: "mtp://Android_Test/".to_owned(),
        label: "Android".to_owned(),
        filesystem: "mtp".to_owned(),
        size: String::new(),
        mountpoint: None,
        removable: true,
    };
    assert_eq!(device_detail(&volume), "MTP · não montado");
    volume.mountpoint = Some("/run/user/1000/gvfs/mtp:host=Android_Test".to_owned());
    assert_eq!(device_detail(&volume), "MTP · montado");
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
