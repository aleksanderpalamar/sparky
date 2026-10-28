use gio::prelude::*;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;

use crate::domain::application::Application;
use crate::ports::Opener;

pub struct XdgOpen;

impl Opener for XdgOpen {
    fn open(&self, path: &Path) -> io::Result<()> {
        let mut child = Command::new("xdg-open")
            .arg(path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        thread::spawn(move || child.wait());
        Ok(())
    }

    fn applications(&self, path: &Path) -> Vec<Application> {
        let mut applications: Vec<_> = gio::AppInfo::all_for_type(&content_type(path))
            .into_iter()
            .filter(|app| app.should_show() && app.supports_files())
            .filter_map(|app| {
                Some(Application {
                    id: app.id()?.to_string(),
                    name: app.name().to_string(),
                })
            })
            .collect();
        applications.sort_by(|left, right| left.name.cmp(&right.name));
        applications.dedup_by(|left, right| left.id == right.id);
        applications
    }

    fn open_with(&self, path: &Path, id: &str) -> io::Result<()> {
        let app = gio::AppInfo::all_for_type(&content_type(path))
            .into_iter()
            .find(|app| app.id().as_deref() == Some(id))
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "aplicativo indisponível"))?;
        app.launch(&[gio::File::for_path(path)], None::<&gio::AppLaunchContext>)
            .map_err(io::Error::other)
    }

    fn email(&self, paths: &[PathBuf]) -> io::Result<()> {
        let mut command = Command::new("xdg-email");
        for path in paths {
            command.arg("--attach").arg(path);
        }
        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        thread::spawn(move || child.wait());
        Ok(())
    }
}

fn content_type(path: &Path) -> String {
    if path.is_dir() {
        return "inode/directory".to_owned();
    }
    gio::content_type_guess(Some(path), None).0.to_string()
}

#[cfg(test)]
mod tests {
    use super::content_type;

    #[test]
    fn pasta_usa_o_tipo_de_diretorio_para_listar_aplicativos() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("Fotos");
        std::fs::create_dir(&folder).unwrap();
        assert_eq!(content_type(&folder), "inode/directory");
    }
}
