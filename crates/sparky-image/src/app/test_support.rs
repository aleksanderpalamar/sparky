use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use iced::futures::StreamExt;
use iced::{Size, Task};
use iced_runtime::Action;
use sparky_theme::Skin;

use super::message::Message;
use super::state::{Services, Viewer};
use super::update::update;
use crate::domain::picture::Picture;
use crate::ports::{DecodeError, Decoder, Gallery};

pub const JANELA: Size = Size {
    width: 1000.0,
    height: 836.0,
};

#[derive(Default)]
pub struct Decodificador {
    pub decodificadas: AtomicUsize,
}

impl Decoder for Decodificador {
    fn decode(&self, path: &Path) -> Result<Picture, DecodeError> {
        self.decodificadas.fetch_add(1, Ordering::SeqCst);
        let name = path.to_string_lossy();
        let (width, height) = match () {
            _ if name.contains("quebrada") => return Err(DecodeError::Corrupted),
            _ if name.contains("grande") => (2000, 1000),
            _ => (200, 100),
        };
        Ok(Picture {
            width,
            height,
            rgba: [200, 30, 30, 255].repeat((width * height) as usize),
        })
    }
}

pub struct Pasta(pub Vec<&'static str>);

impl Gallery for Pasta {
    fn images(&self, folder: &Path) -> Vec<PathBuf> {
        self.0.iter().map(|name| folder.join(name)).collect()
    }
}

pub fn abrir(pasta: Vec<&'static str>, alvo: Option<&str>) -> (Viewer, Task<Message>) {
    abrir_com(pasta, alvo, Arc::new(Decodificador::default()))
}

pub fn abrir_com(
    pasta: Vec<&'static str>,
    alvo: Option<&str>,
    decoder: Arc<Decodificador>,
) -> (Viewer, Task<Message>) {
    let services = Services {
        decoder,
        gallery: Box::new(Pasta(pasta)),
    };
    Viewer::open(services, Skin::default(), alvo.map(PathBuf::from), JANELA)
}

pub fn concluir(viewer: &mut Viewer, task: Task<Message>) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let mut fila = vec![task];
    while let Some(task) = fila.pop() {
        let Some(mut stream) = iced_runtime::task::into_stream(task) else {
            continue;
        };
        let mensagens = runtime.block_on(async {
            let mut mensagens = Vec::new();
            while let Some(action) = stream.next().await {
                if let Action::Output(message) = action {
                    mensagens.push(message);
                }
            }
            mensagens
        });
        for message in mensagens {
            fila.push(update(viewer, message));
        }
    }
}
