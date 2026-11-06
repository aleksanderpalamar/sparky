use std::fs;
use std::path::{Path, PathBuf};

use gio::prelude::*;

use crate::domain::sidebar_location::{SidebarItem, SidebarLocation, SidebarTarget};
use crate::ports::SidebarSource;

pub struct DesktopSidebar {
    data: PathBuf,
}

impl DesktopSidebar {
    pub fn new(data: PathBuf) -> Self {
        Self { data }
    }

    pub fn from_env() -> Self {
        let data = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
            })
            .unwrap_or_default();
        Self { data }
    }

    fn favorites_path(&self) -> PathBuf {
        self.data.join("sparky/files-favorites.json")
    }

    fn favorites(&self) -> Result<Vec<PathBuf>, String> {
        let contents = match fs::read_to_string(self.favorites_path()) {
            Ok(contents) => contents,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error.to_string()),
        };
        let uris: Vec<String> =
            serde_json::from_str(&contents).map_err(|error| error.to_string())?;
        Ok(uris
            .into_iter()
            .filter_map(|uri| gio::File::for_uri(&uri).path())
            .collect())
    }

    fn save_favorites(&self, paths: &[PathBuf]) -> Result<(), String> {
        let uris: Vec<_> = paths
            .iter()
            .map(|path| gio::File::for_path(path).uri().to_string())
            .collect();
        let destination = self.favorites_path();
        let parent = destination.parent().ok_or("invalid favorites directory")?;
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        let mut temporary =
            tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
        serde_json::to_writer(&mut temporary, &uris).map_err(|error| error.to_string())?;
        temporary
            .persist(destination)
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn recent(&self) -> Result<Vec<SidebarItem>, String> {
        super::sidebar_recent::read_recent(&self.data)
    }

    fn network(&self) -> Vec<SidebarItem> {
        let mut items = vec![SidebarItem {
            target: SidebarTarget::Remote {
                uri: "network:///".to_owned(),
                directory: true,
            },
            name: crate::i18n::text("network-browse"),
        }];
        items.extend(
            gio::VolumeMonitor::get()
                .mounts()
                .into_iter()
                .filter_map(|mount| {
                    let root = mount.root();
                    let uri = root.uri();
                    let supported = ["smb:", "sftp:", "ftp:", "dav:", "davs:", "nfs:"];
                    if !supported.iter().any(|scheme| uri.starts_with(scheme)) {
                        return None;
                    }
                    Some(SidebarItem {
                        target: SidebarTarget::Remote {
                            uri: uri.to_string(),
                            directory: true,
                        },
                        name: mount.name().to_string(),
                    })
                }),
        );
        items
    }
}

impl SidebarSource for DesktopSidebar {
    fn list(&self, location: SidebarLocation) -> Result<Vec<SidebarItem>, String> {
        match location {
            SidebarLocation::Recent => self.recent(),
            SidebarLocation::Favorites => Ok(self
                .favorites()?
                .into_iter()
                .filter(|path| path.exists())
                .map(item)
                .collect()),
            SidebarLocation::Network => Ok(self.network()),
        }
    }

    fn network_children(&self, uri: &str) -> Result<Vec<SidebarItem>, String> {
        let root = gio::File::for_uri(uri);
        let children = root
            .enumerate_children(
                "standard::name,standard::display-name,standard::type,standard::target-uri",
                gio::FileQueryInfoFlags::NONE,
                gio::Cancellable::NONE,
            )
            .map_err(|error| error.to_string())?;
        children
            .map(|info| {
                let info = info.map_err(|error| error.to_string())?;
                Ok(super::sidebar_network::item(&root, &info))
            })
            .collect()
    }

    fn open_remote(&self, uri: &str) -> Result<(), String> {
        gio::AppInfo::launch_default_for_uri(uri, gio::AppLaunchContext::NONE)
            .map_err(|error| error.to_string())
    }

    fn favorite(&self, path: &Path, enabled: bool) -> Result<(), String> {
        let mut paths = self.favorites()?;
        paths.retain(|saved| saved != path && saved.exists());
        if enabled && path.exists() {
            paths.push(path.to_path_buf());
        }
        self.save_favorites(&paths)
    }

    fn is_favorite(&self, path: &Path) -> bool {
        self.favorites()
            .is_ok_and(|paths| paths.iter().any(|saved| saved == path))
    }

    fn relocated(&self, from: &Path, to: &Path) -> Result<(), String> {
        let paths = self.favorites()?;
        let changed: Vec<_> = paths
            .iter()
            .map(|path| {
                path.strip_prefix(from)
                    .map_or_else(|_| path.clone(), |suffix| to.join(suffix))
            })
            .collect();
        if changed == paths {
            return Ok(());
        }
        self.save_favorites(&changed)
    }
}

pub(super) fn item(path: PathBuf) -> SidebarItem {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());
    SidebarItem {
        target: SidebarTarget::Local(path),
        name,
    }
}
