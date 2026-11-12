use gio::prelude::*;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;

use crate::adapters::app_icons::AppIcons;
use crate::domain::application::{AppChoices, Remember};
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

    fn applications(&self, path: &Path) -> AppChoices {
        let kind = content_type(path);
        let icons = AppIcons::current();
        let usable = |app: &gio::AppInfo| app.should_show() && can_open_local(app);
        let mut recommended: Vec<gio::AppInfo> = gio::AppInfo::default_for_type(&kind, false)
            .into_iter()
            .chain(gio::AppInfo::recommended_for_type(&kind))
            .filter(usable)
            .collect();
        dedup(&mut recommended);
        let mut others: Vec<gio::AppInfo> = gio::AppInfo::all()
            .into_iter()
            .filter(usable)
            .filter(|app| !recommended.iter().any(|known| known.id() == app.id()))
            .collect();
        dedup(&mut others);
        others.sort_by_key(|app| app.name().to_lowercase());
        AppChoices {
            kind: gio::content_type_get_description(&kind).to_string(),
            recommended: recommended
                .iter()
                .filter_map(|app| icons.describe(app))
                .collect(),
            others: others
                .iter()
                .filter_map(|app| icons.describe(app))
                .collect(),
        }
    }

    fn open_with(&self, path: &Path, id: &str, remember: Remember) -> io::Result<()> {
        let app = gio::AppInfo::all()
            .into_iter()
            .find(|app| app.id().as_deref() == Some(id))
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    crate::i18n::text("application-unavailable"),
                )
            })?;
        if remember == Remember::Always {
            app.set_as_default_for_type(&content_type(path))
                .map_err(io::Error::other)?;
        }
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

fn can_open_local(app: &gio::AppInfo) -> bool {
    app.supports_files() || app.supports_uris()
}

fn content_type(path: &Path) -> String {
    if path.is_dir() {
        return "inode/directory".to_owned();
    }
    gio::File::for_path(path)
        .query_info(
            gio::FILE_ATTRIBUTE_STANDARD_CONTENT_TYPE,
            gio::FileQueryInfoFlags::NONE,
            None::<&gio::Cancellable>,
        )
        .ok()
        .and_then(|info| info.content_type())
        .map(|kind| kind.to_string())
        .unwrap_or_else(|| gio::content_type_guess(Some(path), None).0.to_string())
}

fn dedup(apps: &mut Vec<gio::AppInfo>) {
    let mut seen = std::collections::BTreeSet::new();
    apps.retain(|app| seen.insert(app.id().map(|id| id.to_string())));
}

#[cfg(test)]
mod tests {
    use super::content_type;

    #[test]
    fn ini_com_texto_e_reconhecido_pelo_conteudo_como_texto() {
        let Ok(dir) = tempfile::tempdir() else {
            panic!("sem pasta temporária")
        };
        let ini = dir.path().join("neuralfx.ini");
        assert!(std::fs::write(&ini, "[Geral]\nbrilho=0.8\n").is_ok());
        assert_eq!(content_type(&ini), "text/plain");
        assert_eq!(content_type(dir.path()), "inode/directory");
    }
}
