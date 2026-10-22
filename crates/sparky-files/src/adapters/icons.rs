use sparky_xdg::icons::{IconTheme, configured_theme, icon_roots};

use crate::domain::icons::{ALL, Icons};
use crate::ports::IconSource;

#[derive(Default)]
pub struct ThemeIcons {
    theme: Option<String>,
    icons: Icons,
}

impl IconSource for ThemeIcons {
    fn load(&mut self) -> Icons {
        let name = configured_theme();
        if self.theme.as_deref() == Some(&name) {
            return self.icons.clone();
        }
        let theme = IconTheme::new(icon_roots(), &name);
        self.icons = ALL
            .into_iter()
            .filter_map(|kind| theme.resolve(kind.names()).map(|path| (kind, path)))
            .collect();
        self.theme = Some(name);
        self.icons.clone()
    }
}
