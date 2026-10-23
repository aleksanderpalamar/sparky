use std::future::Future;
use std::task::Poll;
use std::time::Duration;

use gio::glib;
use gio::prelude::*;

use super::Mounts;

pub struct GioMounts;

impl Mounts for GioMounts {
    fn mountpoint(&self, uri: &str) -> Result<Option<String>, String> {
        let Some(path) = gio::File::for_uri(uri).path() else {
            return Ok(None);
        };
        if !path.is_absolute() {
            return Err("O backend MTP retornou um caminho inválido.".to_owned());
        }
        path.into_os_string()
            .into_string()
            .map(Some)
            .map_err(|_| "O ponto de acesso MTP não possui um nome UTF-8 válido.".to_owned())
    }

    fn mount(&self, uri: &str) -> Result<(), String> {
        let file = gio::File::for_uri(uri);
        let result = wait(file.mount_enclosing_volume_future(
            gio::MountMountFlags::NONE,
            None::<&gio::MountOperation>,
        ));
        match result {
            Ok(()) => {}
            Err(error) if error.matches(gio::IOErrorEnum::AlreadyMounted) => {}
            Err(error) => return Err(failure(error)),
        }
        self.mountpoint(uri)?.ok_or_else(|| {
            "O telefone foi conectado, mas falta o acesso FUSE do GVfs. Verifique gvfs-fuse (gvfs no Arch).".to_owned()
        })?;
        Ok(())
    }

    fn unmount(&self, uri: &str) -> Result<(), String> {
        let mount = gio::File::for_uri(uri)
            .find_enclosing_mount(gio::Cancellable::NONE)
            .map_err(failure)?;
        wait(mount.unmount_with_operation_future(
            gio::MountUnmountFlags::NONE,
            None::<&gio::MountOperation>,
        ))
        .map_err(failure)
    }
}

fn wait<F: Future<Output = Result<(), glib::Error>>>(future: F) -> Result<(), glib::Error> {
    glib::MainContext::new().block_on(bounded(future, Duration::from_secs(30)))
}

async fn bounded<F, T>(future: F, limit: Duration) -> Result<T, glib::Error>
where
    F: Future<Output = Result<T, glib::Error>>,
{
    let mut future = std::pin::pin!(future);
    let mut timeout = glib::timeout_future(limit);
    std::future::poll_fn(|context| {
        if let Poll::Ready(result) = future.as_mut().poll(context) {
            return Poll::Ready(result);
        }
        if timeout.as_mut().poll(context).is_ready() {
            return Poll::Ready(Err(glib::Error::new(
                gio::IOErrorEnum::TimedOut,
                "O telefone não respondeu em 30 segundos. Desbloqueie-o e tente novamente.",
            )));
        }
        Poll::Pending
    })
    .await
}

fn failure(error: glib::Error) -> String {
    match error.kind::<gio::IOErrorEnum>() {
        Some(gio::IOErrorEnum::NotSupported) =>
            "O backend MTP não está disponível. Verifique gvfs-mtp (gvfs-backends no Debian/Ubuntu).".to_owned(),
        Some(gio::IOErrorEnum::Busy) =>
            "O telefone está em uso por outro aplicativo. Feche esse acesso e tente novamente.".to_owned(),
        Some(gio::IOErrorEnum::Cancelled) => "Acesso ao telefone cancelado.".to_owned(),
        _ => format!("Não foi possível acessar o telefone MTP: {error}. Confirme o desbloqueio e a transferência de arquivos no Android."),
    }
}

#[cfg(test)]
mod tests;
