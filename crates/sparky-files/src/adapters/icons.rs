use sparky_xdg::icons::{IconTheme, configured_theme, icon_roots};

use crate::domain::icons::{ALL, Icons, SIDEBAR};
use crate::ports::IconSource;

#[derive(Default)]
pub struct ThemeIcons {
    theme: Option<String>,
    icons: Icons,
    sidebar: Icons,
}

impl IconSource for ThemeIcons {
    fn load(&mut self) -> Icons {
        let name = configured_theme();
        if self.theme.as_deref() == Some(&name) {
            return self.icons.clone();
        }
        let theme = IconTheme::with_size(icon_roots(), &name, 96);
        self.icons = ALL
            .into_iter()
            .filter_map(|kind| theme.resolve(kind.names()).map(|path| (kind, path)))
            .collect();
        let sidebar_theme = IconTheme::with_size(icon_roots(), &name, 20);
        self.sidebar = SIDEBAR
            .into_iter()
            .filter_map(|kind| {
                let names = kind.sidebar_names();
                let names: Vec<_> = names.iter().map(String::as_str).collect();
                sidebar_theme.resolve(&names).map(|path| (kind, path))
            })
            .collect();
        self.theme = Some(name);
        self.icons.clone()
    }

    fn load_sidebar(&mut self) -> Icons {
        self.load();
        self.sidebar.clone()
    }
}
