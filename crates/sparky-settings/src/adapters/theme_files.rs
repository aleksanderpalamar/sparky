use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use sparky_theme::{Palette, locations};

use crate::domain::system::icons::{icon_theme_of, with_icon_theme};
use crate::domain::system::themes::{Catalog, Theme};
use crate::ports::themes::ThemeStore;

pub struct ThemeFiles {
    roots: Vec<PathBuf>,
    target: PathBuf,
    default_colors: PathBuf,
    gtk: Vec<PathBuf>,
}

impl ThemeFiles {
    pub fn new(roots: Vec<PathBuf>, target: PathBuf, default_colors: PathBuf) -> Self {
        Self { roots, target, default_colors, gtk: Vec::new() }
    }

    pub fn with_gtk(mut self, gtk: Vec<PathBuf>) -> Self {
        self.gtk = gtk;
        self
    }

    pub fn from_env() -> Self {
        let home = env::var("HOME").unwrap_or_default();
        let config = env::var("XDG_CONFIG_HOME").ok();
        let data = env::var("XDG_DATA_HOME").ok().filter(|path| !path.trim().is_empty())
            .map(PathBuf::from).unwrap_or_else(|| PathBuf::from(&home).join(".local/share"));
        let system = env::var("SPARKY_PATH").ok();
        let system = locations::sparky_path(system.as_deref()).join("themes");
        let config_home = config.filter(|path| !path.trim().is_empty()).map(PathBuf::from).unwrap_or_else(|| PathBuf::from(&home).join(".config"));
        Self::new(
            vec![data.join("sparky/themes"), system.clone()],
            locations::config_dir(config_home.to_str(), &home).join("colors.toml"),
            system.join("tokyo-night/colors.toml"),
        )
        .with_gtk(vec![config_home.join("gtk-3.0"), config_home.join("gtk-4.0")])
    }

    fn catalog(&self) -> io::Result<BTreeMap<String, Theme>> {
        let mut themes = BTreeMap::new();
        for root in &self.roots {
            let entries = match fs::read_dir(root) {
                Ok(entries) => entries,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error),
            };
            for entry in entries {
                let entry = entry?;
                let Ok(contents) = fs::read_to_string(entry.path().join("colors.toml")) else { continue };
                let name = entry.file_name().to_string_lossy().into_owned();
                let Ok(theme) = Theme::parse(&name, &contents) else { continue };
                let icons = fs::read_to_string(entry.path().join("icons.theme")).ok().as_deref().and_then(icon_theme_of);
                themes.entry(name).or_insert(theme.with_icons(icons));
            }
        }
        themes.entry("tokyo-night".to_owned()).or_insert_with(Theme::builtin);
        Ok(themes)
    }
}

impl ThemeStore for ThemeFiles {
    fn load(&self) -> io::Result<Catalog> {
        let themes = self.catalog()?;
        let current = Palette::load(|path| fs::read_to_string(path).ok(), &[self.target.clone(), self.default_colors.clone()]);
        Ok(Catalog { themes: themes.into_values().collect(), current })
    }

    fn apply(&self, theme: &Theme) -> io::Result<()> {
        let parent = self.target.parent().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "caminho de cores sem diretório"))?;
        fs::create_dir_all(parent)?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        temporary.write_all(theme.contents().as_bytes())?;
        temporary.flush()?;
        temporary.persist(&self.target).map_err(|error| error.error)?;
        match theme.icons() {
            Some(icons) => self.gtk.iter().try_for_each(|folder| write_icons(folder, icons)),
            None => Ok(()),
        }
    }
}

fn write_icons(folder: &Path, icons: &str) -> io::Result<()> {
    let path = folder.join("settings.ini");
    let current = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error),
    };
    fs::create_dir_all(folder)?;
    let mut temporary = tempfile::NamedTempFile::new_in(folder)?;
    temporary.write_all(with_icon_theme(&current, icons).as_bytes())?;
    temporary.flush()?;
    temporary.persist(&path).map_err(|error| error.error).map(drop)
}
