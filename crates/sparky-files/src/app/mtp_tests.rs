use std::sync::{Arc, Mutex};

use iced::futures::StreamExt;
use iced_runtime::Action;
use sparky_storage::{Operation, Volume, VolumeSource};

use super::files_tests::Cena;
use super::message::Message;
use super::{update, volumes};

const URI: &str = "mtp://Android_Test/";

fn phone(point: Option<&str>) -> Volume {
    Volume {
        path: URI.to_owned(),
        label: "Android".to_owned(),
        filesystem: "mtp".to_owned(),
        size: String::new(),
        mountpoint: point.map(str::to_owned),
        removable: true,
    }
}

struct Source {
    volumes: Vec<Volume>,
    calls: Mutex<Vec<Operation>>,
}

impl VolumeSource for Source {
    fn run(&self, operation: Operation) -> Result<Vec<Volume>, String> {
        self.calls.lock().unwrap().push(operation);
        Ok(self.volumes.clone())
    }
}

#[test]
fn clicar_no_android_abre_o_ponto_local_retornado_pelo_backend() {
    let mut scene = Cena::nova();
    let point = scene.work().join("Fotos");
    let source = Arc::new(Source {
        volumes: vec![phone(point.to_str())],
        calls: Mutex::default(),
    });
    scene.files.services.volumes = source.clone();
    scene.files.volumes = vec![phone(None)];
    let task = volumes::open(&mut scene.files, URI);
    let mut stream = iced_runtime::task::into_stream(task).unwrap();
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(async {
            while let Some(Action::Output(message)) = stream.next().await {
                let _ = update::update(&mut scene.files, message);
            }
        });
    assert_eq!(scene.files.location, point);
    assert_eq!(
        *source.calls.lock().unwrap(),
        [Operation::Mount(URI.to_owned())]
    );
    assert!(scene.files.volume_work.is_none());
}

#[test]
fn retirar_o_android_aberto_volta_para_a_pasta_pessoal() {
    let mut scene = Cena::nova();
    let point = scene.work().join("Fotos");
    scene.files.volumes = vec![phone(point.to_str())];
    scene.files.go(point);
    scene.files.volume_work = Some(Operation::List);
    let _ = update::update(
        &mut scene.files,
        Message::VolumesCompleted(Operation::List, Ok(Vec::new())),
    );
    assert_eq!(scene.files.location, scene.files.home);
    assert!(scene.files.volumes.is_empty());
}
