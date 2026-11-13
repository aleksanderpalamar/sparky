use std::io;
use std::process::{Command, Stdio};

use zbus::fdo::{RequestNameFlags, RequestNameReply};

use crate::domain::launch::{Request, windows};

pub const SERVE: &str = "--file-manager1";
const NAME: &str = "org.freedesktop.FileManager1";
const PATH: &str = "/org/freedesktop/FileManager1";

type Opener = Box<dyn Fn(Vec<String>, Option<String>) + Send + Sync>;

pub struct FileManager1 {
    open: Opener,
}

impl FileManager1 {
    pub fn new(open: impl Fn(Vec<String>, Option<String>) + Send + Sync + 'static) -> Self {
        Self {
            open: Box::new(open),
        }
    }

    fn show(&self, request: Request, uris: &[String], startup_id: String) {
        let token = (!startup_id.is_empty()).then_some(startup_id);
        for args in windows(request, uris) {
            (self.open)(args, token.clone());
        }
    }
}

#[zbus::interface(name = "org.freedesktop.FileManager1")]
impl FileManager1 {
    pub fn show_folders(&self, uris: Vec<String>, startup_id: String) {
        self.show(Request::Folders, &uris, startup_id);
    }

    pub fn show_items(&self, uris: Vec<String>, startup_id: String) {
        self.show(Request::Items, &uris, startup_id);
    }

    pub fn show_item_properties(&self, uris: Vec<String>, startup_id: String) {
        self.show(Request::Properties, &uris, startup_id);
    }
}

pub fn serve() -> io::Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        let connection = zbus::connection::Builder::session()
            .and_then(|builder| builder.serve_at(PATH, FileManager1::new(spawn)))
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

fn spawn(args: Vec<String>, token: Option<String>) {
    let Ok(program) = std::env::current_exe() else {
        return;
    };
    let mut command = Command::new(program);
    command
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(token) = token {
        command
            .env("XDG_ACTIVATION_TOKEN", &token)
            .env("DESKTOP_STARTUP_ID", token);
    }
    if let Ok(mut child) = command.spawn() {
        std::thread::spawn(move || child.wait());
    }
}
