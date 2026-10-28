use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, SystemTime};

use iced::futures::StreamExt;
use iced::{Size, Task};
use iced_runtime::Action as TaskAction;

use super::message::Message;
use super::state::Files;
use super::test_scene::Cena;
use super::update::update;
use crate::domain::thumbnail::Thumbnail;
use crate::ports::ThumbnailSource;

#[derive(Default)]
struct Contador(AtomicUsize);

impl ThumbnailSource for Contador {
    fn load(&self, _path: &Path) -> Option<Thumbnail> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Some(Thumbnail {
            width: 1,
            height: 1,
            rgba: vec![0; 4],
        })
    }
}

fn cena_com_imagens(size: Size, extra: usize) -> (Cena, Arc<Contador>, Vec<PathBuf>) {
    let mut scene = Cena::nova();
    let contador = Arc::new(Contador::default());
    scene.files.services.thumbnails = contador.clone();
    scene.files.resize(size);
    let total = scene.files.layout.capacity() + extra;
    let paths: Vec<_> = (0..total)
        .map(|index| scene.work().join(format!("foto-{index:04}.png")))
        .collect();
    for path in &paths {
        fs::write(path, "png").unwrap();
    }
    scene.files.refresh();
    (scene, contador, paths)
}

fn imagens_visiveis(files: &Files) -> Vec<PathBuf> {
    files
        .layout
        .range(files.viewport.top(), files.listing.len())
        .filter_map(|index| files.listing.get(index))
        .filter(|entry| entry.name.ends_with(".png"))
        .map(|entry| files.location.join(&entry.name))
        .collect()
}

fn concluir(files: &mut Files, task: Task<Message>, limite: usize) -> usize {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let mut queue = vec![task];
    let mut processadas = 0;
    while let Some(task) = queue.pop() {
        let Some(mut stream) = iced_runtime::task::into_stream(task) else {
            continue;
        };
        let messages = runtime.block_on(async {
            let mut messages = Vec::new();
            while let Some(action) = stream.next().await {
                if let TaskAction::Output(message) = action {
                    messages.push(message);
                }
            }
            messages
        });
        for message in messages {
            processadas += 1;
            if processadas > limite {
                return processadas;
            }
            queue.push(update(files, message));
        }
    }
    processadas
}

#[test]
fn miniaturas_sao_geradas_somente_para_itens_visiveis() {
    let (mut scene, contador, paths) = cena_com_imagens(Size::new(900.0, 560.0), 5);
    let visiveis = imagens_visiveis(&scene.files);
    let task = scene.files.request_previews();
    concluir(&mut scene.files, task, 1000);
    let geradas: Vec<_> = paths
        .iter()
        .filter(|path| scene.files.previews.handle(path).is_some())
        .cloned()
        .collect();
    assert_eq!(geradas, visiveis);
    assert_eq!(contador.0.load(Ordering::SeqCst), visiveis.len());
    assert!(
        scene
            .files
            .previews
            .handle(&paths[paths.len() - 1])
            .is_none()
    );
}

#[test]
fn janela_grande_com_muitas_imagens_nao_redecodifica_sem_fim() {
    let (mut scene, contador, _) = cena_com_imagens(Size::new(2560.0, 1440.0), 0);
    let visiveis = imagens_visiveis(&scene.files);
    assert!(visiveis.len() > 64);
    let task = scene.files.request_previews();
    let processadas = concluir(&mut scene.files, task, visiveis.len() * 3);
    assert_eq!(processadas, visiveis.len());
    assert_eq!(contador.0.load(Ordering::SeqCst), visiveis.len());
    assert!(
        visiveis
            .iter()
            .all(|path| scene.files.previews.handle(path).is_some())
    );
}

#[test]
fn conclusao_de_arquivo_modificado_e_descartada_e_refeita() {
    let (mut scene, contador, paths) = cena_com_imagens(Size::new(900.0, 560.0), 0);
    let path = paths[0].clone();
    let antigo = scene.files.services.disk.stamp(&path);
    let _ = scene.files.request_previews();
    let novo = SystemTime::now() + Duration::from_secs(60);
    File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_modified(novo)
        .unwrap();
    let thumbnail = Thumbnail {
        width: 1,
        height: 1,
        rgba: vec![0; 4],
    };
    let task = update(
        &mut scene.files,
        Message::PreviewReady(path.clone(), antigo, Some(thumbnail)),
    );
    assert!(scene.files.previews.handle(&path).is_none());
    concluir(&mut scene.files, task, 1000);
    assert!(scene.files.previews.handle(&path).is_some());
    assert!(contador.0.load(Ordering::SeqCst) >= 1);
}
