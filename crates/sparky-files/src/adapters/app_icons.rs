use std::path::PathBuf;

use gio::prelude::*;
use sparky_xdg::icons::{IconTheme, configured_theme, icon_roots};

use crate::domain::application::Application;

const SIZE: u32 = 24;

pub struct AppIcons {
    theme: IconTheme,
}

impl AppIcons {
    pub fn current() -> Self {
        Self {
            theme: IconTheme::with_size(icon_roots(), &configured_theme(), SIZE),
        }
    }

    pub fn describe(&self, app: &gio::AppInfo) -> Option<Application> {
        Some(Application {
            id: app.id()?.to_string(),
            name: app.name().to_string(),
            icon: app.icon().and_then(|icon| self.path(&icon)),
        })
    }

    fn path(&self, icon: &gio::Icon) -> Option<PathBuf> {
        if let Some(file) = icon.downcast_ref::<gio::FileIcon>() {
            return file.file().path();
        }
        let themed = icon.downcast_ref::<gio::ThemedIcon>()?;
        let names: Vec<String> = themed.names().iter().map(|name| name.to_string()).collect();
        let names: Vec<&str> = names.iter().map(String::as_str).collect();
        self.theme.resolve(&names)
    }
}
