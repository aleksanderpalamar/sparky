use std::io;
use std::sync::Arc;

use zbus::fdo::{RequestNameFlags, RequestNameReply};
use zbus::interface;
use zbus::object_server::ObjectServer;
use zbus::zvariant::OwnedObjectPath;

use super::dialog::ProcessDialogs;
use super::options::{Call, Options, request};
use super::results::{Results, results};
use crate::domain::choice::Request;
use crate::ports::chooser::{Closer, Dialogs, Shown};

pub const SERVE: &str = "--file-chooser";
const NAME: &str = "org.freedesktop.impl.portal.desktop.sparky";
pub const PATH: &str = "/org/freedesktop/portal/desktop";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Response {
    Success = 0,
    Cancelled = 1,
    Failed = 2,
}

pub struct FileChooser {
    dialogs: Arc<dyn Dialogs>,
}

impl FileChooser {
    pub fn new(dialogs: Arc<dyn Dialogs>) -> Self {
        Self { dialogs }
    }

    async fn ask(
        &self,
        server: &ObjectServer,
        handle: OwnedObjectPath,
        asked: Request,
    ) -> (u32, Results) {
        let closer = Closer::default();
        let pending = server
            .at(&handle, Pending(closer.clone()))
            .await
            .unwrap_or(false);
        let dialogs = Arc::clone(&self.dialogs);
        let request = asked.clone();
        let shown = tokio::task::spawn_blocking(move || dialogs.show(&request, &closer))
            .await
            .unwrap_or(Shown::Failed);
        if pending {
            let _ = server.remove::<Pending, _>(&handle).await;
        }
        match shown {
            Shown::Chosen(outcome) => (Response::Success as u32, results(&asked, &outcome)),
            Shown::Cancelled => (Response::Cancelled as u32, Results::new()),
            Shown::Failed => (Response::Failed as u32, Results::new()),
        }
    }
}

#[interface(name = "org.freedesktop.impl.portal.FileChooser")]
impl FileChooser {
    async fn open_file(
        &self,
        handle: OwnedObjectPath,
        _app_id: &str,
        _parent_window: &str,
        title: &str,
        options: Options,
        #[zbus(object_server)] server: &ObjectServer,
    ) -> (u32, Results) {
        self.ask(server, handle, request(Call::OpenFile, title, &options))
            .await
    }

    async fn save_file(
        &self,
        handle: OwnedObjectPath,
        _app_id: &str,
        _parent_window: &str,
        title: &str,
        options: Options,
        #[zbus(object_server)] server: &ObjectServer,
    ) -> (u32, Results) {
        self.ask(server, handle, request(Call::SaveFile, title, &options))
            .await
    }

    async fn save_files(
        &self,
        handle: OwnedObjectPath,
        _app_id: &str,
        _parent_window: &str,
        title: &str,
        options: Options,
        #[zbus(object_server)] server: &ObjectServer,
    ) -> (u32, Results) {
        self.ask(server, handle, request(Call::SaveFiles, title, &options))
            .await
    }
}

struct Pending(Closer);

#[interface(name = "org.freedesktop.impl.portal.Request")]
impl Pending {
    fn close(&self) {
        self.0.close();
    }
}

pub fn serve() -> io::Result<()> {
    let program = std::env::current_exe()?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        let chooser = FileChooser::new(Arc::new(ProcessDialogs::new(program)));
        let connection = zbus::connection::Builder::session()
            .and_then(|builder| builder.serve_at(PATH, chooser))
            .map_err(io::Error::other)?
            .build()
            .await
            .map_err(io::Error::other)?;
        let reply = connection
            .request_name_with_flags(NAME, RequestNameFlags::DoNotQueue.into())
            .await
            .map_err(io::Error::other)?;
        if reply != RequestNameReply::PrimaryOwner {
            return Err(io::Error::new(io::ErrorKind::AddrInUse, NAME));
        }
        std::future::pending::<io::Result<()>>().await
    })
}
