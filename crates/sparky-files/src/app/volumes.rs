use std::path::Path;

use iced::Task;
use sparky_storage::{Operation, Volume};

use super::message::Message;
use super::state::Files;

pub fn refresh(files: &mut Files) -> Task<Message> {
    request(files, Operation::List)
}

pub fn open(files: &mut Files, device: &str) -> Task<Message> {
    let Some(volume) = files.volumes.iter().find(|volume| volume.path == device) else {
        return Task::none();
    };
    if let Some(point) = &volume.mountpoint {
        files.go(point.into());
        return Task::none();
    }
    request(files, Operation::Mount(device.to_owned()))
}

pub fn unmount(files: &mut Files, device: &str) -> Task<Message> {
    if !files
        .volumes
        .iter()
        .any(|volume| volume.path == device && volume.mounted())
    {
        return Task::none();
    }
    request(files, Operation::Unmount(device.to_owned()))
}

pub fn unmount_current(files: &mut Files) -> Task<Message> {
    let device = files
        .volumes
        .iter()
        .filter(|volume| {
            volume
                .mountpoint
                .as_ref()
                .is_some_and(|point| files.location.starts_with(point))
        })
        .max_by_key(|volume| volume.mountpoint.as_ref().map(String::len))
        .map(|volume| volume.path.clone());
    device.map_or_else(Task::none, |device| unmount(files, &device))
}

fn request(files: &mut Files, operation: Operation) -> Task<Message> {
    if files.volume_work.is_some() {
        return Task::none();
    }
    files.volume_work = Some(operation.clone());
    let source = files.services.volumes.clone();
    Task::perform(
        async move {
            let work = operation.clone();
            let result = tokio::task::spawn_blocking(move || source.run(work))
                .await
                .map_err(|error| format!("consulta de dispositivos: {error}"))
                .and_then(|result| result);
            (operation, result)
        },
        |(operation, result)| Message::VolumesCompleted(operation, result),
    )
}

pub fn completed(files: &mut Files, operation: Operation, result: Result<Vec<Volume>, String>) {
    if files.volume_work.as_ref() != Some(&operation) {
        return;
    }
    files.volume_work = None;
    match result {
        Ok(volumes) => apply(files, &operation, volumes),
        Err(error) => {
            files.volume_error = Some(error.clone());
            files.fail(error);
        }
    }
}

fn apply(files: &mut Files, operation: &Operation, volumes: Vec<Volume>) {
    let departed = files
        .volumes
        .iter()
        .filter_map(|volume| volume.mountpoint.as_ref())
        .any(|point| {
            files.location.starts_with(point) && !volumes.iter().any(|volume| volume.mountpoint.as_ref() == Some(point))
        });
    files.volumes = volumes;
    files.volume_error = None;
    if departed {
        files.home();
        files.inform("O dispositivo deixou de estar montado".to_owned());
    }
    let Operation::Mount(device) = operation else {
        return;
    };
    let point = files
        .volumes
        .iter()
        .find(|volume| &volume.path == device)
        .and_then(|volume| volume.mountpoint.clone());
    match point {
        Some(point) if Path::new(&point).is_absolute() => {
            files.go(point.into());
        }
        _ => files.fail(format!("o dispositivo não está montado: {device}")),
    }
}
