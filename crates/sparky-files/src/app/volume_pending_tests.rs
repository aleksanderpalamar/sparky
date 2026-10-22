use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use iced::futures::{StreamExt, channel::oneshot, join};
use iced_runtime::Action;
use sparky_storage::{Operation, Volume, VolumeSource};

use super::files_tests::Cena;
use super::message::Message;
use super::{update, volumes};

struct Waiting {
    entered: Mutex<Option<oneshot::Sender<()>>>,
    answer: Mutex<mpsc::Receiver<Result<Vec<Volume>, String>>>,
}

impl VolumeSource for Waiting {
    fn run(&self, _: Operation) -> Result<Vec<Volume>, String> {
        self.entered
            .lock()
            .map_err(|error| error.to_string())?
            .take()
            .ok_or("pedido repetido")?
            .send(())
            .map_err(|()| "observador encerrado")?;
        self.answer
            .lock()
            .map_err(|error| error.to_string())?
            .recv_timeout(Duration::from_secs(5))
            .map_err(|error| error.to_string())?
    }
}

#[test]
fn autenticar_nao_bloqueia_navegacao_e_cancelar_libera_nova_tentativa() {
    let mut scene = Cena::nova();
    let (entered, started) = oneshot::channel();
    let (answer, receiver) = mpsc::channel();
    scene.files.services.volumes = Arc::new(Waiting {
        entered: Mutex::new(Some(entered)),
        answer: Mutex::new(receiver),
    });
    scene.files.volumes = vec![Volume {
        path: "/dev/fixture".to_owned(),
        size: "299G".to_owned(),
        label: "Dados".to_owned(),
        filesystem: "ntfs".to_owned(),
        mountpoint: None,
        removable: false,
    }];
    let task = volumes::open(&mut scene.files, "/dev/fixture");
    let mut stream = iced_runtime::task::into_stream(task).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    runtime.block_on(async {
        let interacting = async {
            started.await.unwrap();
            let _ = update::update(&mut scene.files, Message::Clicked(1));
            assert_eq!(scene.files.listing.cursor(), 1);
            assert_eq!(
                scene.files.volume_work,
                Some(Operation::Mount("/dev/fixture".to_owned()))
            );
            answer
                .send(Err("Autenticação cancelada".to_owned()))
                .unwrap();
        };
        let (result, ()) = join!(stream.next(), interacting);
        let Some(Action::Output(message)) = result else {
            panic!("a operação precisa retornar o resultado");
        };
        let _ = update::update(&mut scene.files, message);
        assert_eq!(scene.files.volume_work, None);
        assert_eq!(scene.files.location, scene.work());
        assert_eq!(
            scene.files.volume_error.as_deref(),
            Some("Autenticação cancelada")
        );
        let retry = volumes::open(&mut scene.files, "/dev/fixture");
        assert!(iced_runtime::task::into_stream(retry).is_some());
    });
}
