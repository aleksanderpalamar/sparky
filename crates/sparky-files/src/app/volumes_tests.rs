use std::fs;
use std::sync::{Arc, Mutex};

use iced::Task;
use iced::futures::StreamExt;
use iced_runtime::Action;
use sparky_storage::{Operation, Volume, VolumeSource};

use super::files_tests::Cena;
use super::message::Message;
use super::update::update;
use super::volumes;

fn volume(point: Option<&str>) -> Volume {
    Volume {
        path: "/dev/sda1".to_owned(),
        size: "32G".to_owned(),
        label: "USB".to_owned(),
        filesystem: "vfat".to_owned(),
        mountpoint: point.map(str::to_owned),
        removable: true,
    }
}

struct Source {
    calls: Mutex<Vec<Operation>>,
    result: Result<Vec<Volume>, String>,
}

impl VolumeSource for Source {
    fn run(&self, operation: Operation) -> Result<Vec<Volume>, String> {
        self.calls.lock().map_err(|error| error.to_string())?.push(operation);
        self.result.clone()
    }
}

fn complete(scene: &mut Cena, task: Task<Message>) {
    let Some(mut stream) = iced_runtime::task::into_stream(task) else {
        return;
    };
    let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();
    runtime.block_on(async {
        while let Some(action) = stream.next().await {
            let Action::Output(message) = action else { continue };
            let _ = update(&mut scene.files, message);
        }
    });
}

#[test]
fn volume_montado_abre_sem_disparar_comando_de_montagem() {
    let mut scene = Cena::nova();
    let point = scene.work().join("Fotos");
    scene.files.volumes = vec![volume(point.to_str())];
    let task = volumes::open(&mut scene.files, "/dev/sda1");
    assert!(iced_runtime::task::into_stream(task).is_none());
    assert_eq!(scene.files.location, point);
}

#[test]
fn montagem_assincrona_abre_o_ponto_retornado_e_serializa_pedidos() {
    let mut scene = Cena::nova();
    let point = scene.dir.path().join("USB");
    fs::create_dir(&point).unwrap();
    let source = Arc::new(Source {
        calls: Mutex::default(),
        result: Ok(vec![volume(point.to_str())]),
    });
    scene.files.services.volumes = source.clone();
    scene.files.volumes = vec![volume(None)];
    let task = volumes::open(&mut scene.files, "/dev/sda1");
    assert_eq!(scene.files.volume_work, Some(Operation::Mount("/dev/sda1".to_owned())));
    assert!(iced_runtime::task::into_stream(volumes::refresh(&mut scene.files)).is_none());
    complete(&mut scene, task);
    assert_eq!(
        source.calls.lock().unwrap().as_slice(),
        &[Operation::Mount("/dev/sda1".to_owned())]
    );
    assert_eq!(scene.files.location, point);
    assert_eq!(scene.files.volume_work, None);
}

#[test]
fn recusa_de_montagem_preserva_localizacao_e_mostra_erro() {
    let mut scene = Cena::nova();
    scene.files.services.volumes = Arc::new(Source {
        calls: Mutex::default(),
        result: Err("Not authorized".to_owned()),
    });
    scene.files.volumes = vec![volume(None)];
    let before = scene.files.location.clone();
    let task = volumes::open(&mut scene.files, "/dev/sda1");
    complete(&mut scene, task);
    assert_eq!(scene.files.location, before);
    assert_eq!(scene.files.volume_error.as_deref(), Some("Not authorized"));
    assert!(
        scene
            .files
            .notice
            .as_ref()
            .is_some_and(|notice| notice.text().contains("Not authorized"))
    );
}

#[test]
fn remover_ou_desmontar_volume_atual_volta_para_pasta_pessoal() {
    for operation in [Operation::List, Operation::Unmount("/dev/sda1".to_owned())] {
        let mut scene = Cena::nova();
        let point = scene.work().join("Fotos");
        scene.files.volumes = vec![volume(point.to_str())];
        scene.files.go(point);
        scene.files.volume_work = Some(operation.clone());
        volumes::completed(&mut scene.files, operation, Ok(Vec::new()));
        assert_eq!(scene.files.location, scene.files.home);
        assert!(scene.files.volumes.is_empty());
    }
}

#[test]
fn reconsulta_de_volumes_nao_desfaz_rolagem_manual() {
    let mut scene = Cena::nova();
    scene.files.viewport = sparky_ui::viewport::Viewport::new(1);
    scene.files.viewport.scroll(1, scene.files.listing.len());
    scene.files.volume_work = Some(Operation::List);
    let top = scene.files.viewport.top();
    volumes::completed(&mut scene.files, Operation::List, Ok(Vec::new()));
    assert_eq!(scene.files.viewport.top(), top);
}

#[test]
fn resposta_antiga_e_click_em_volume_removido_sao_ignorados() {
    let mut scene = Cena::nova();
    scene.files.volume_work = Some(Operation::List);
    volumes::completed(
        &mut scene.files,
        Operation::Mount("/dev/sda1".to_owned()),
        Ok(vec![volume(None)]),
    );
    assert_eq!(scene.files.volume_work, Some(Operation::List));
    assert!(scene.files.volumes.is_empty());
    assert!(iced_runtime::task::into_stream(volumes::open(&mut scene.files, "/dev/ausente")).is_none());
}

#[test]
fn atalho_desmonta_apenas_o_volume_em_que_esta_navegando() {
    let mut scene = Cena::nova();
    let point = scene.work().join("Fotos");
    let source = Arc::new(Source {
        calls: Mutex::default(),
        result: Ok(vec![volume(None)]),
    });
    scene.files.services.volumes = source.clone();
    scene.files.volumes = vec![volume(point.to_str())];
    scene.files.go(point);
    let task = volumes::unmount_current(&mut scene.files);
    complete(&mut scene, task);
    assert_eq!(
        source.calls.lock().unwrap().as_slice(),
        &[Operation::Unmount("/dev/sda1".to_owned())]
    );
    assert_eq!(scene.files.location, scene.files.home);
    assert!(iced_runtime::task::into_stream(volumes::unmount_current(&mut scene.files)).is_none());
}

#[test]
fn consulta_periodica_atualiza_a_lateral_sem_montar_automaticamente() {
    let mut scene = Cena::nova();
    let source = Arc::new(Source {
        calls: Mutex::default(),
        result: Ok(vec![volume(None)]),
    });
    scene.files.services.volumes = source.clone();
    let task = volumes::refresh(&mut scene.files);
    complete(&mut scene, task);
    assert_eq!(scene.files.volumes, vec![volume(None)]);
    assert_eq!(source.calls.lock().unwrap().as_slice(), &[Operation::List]);
}
