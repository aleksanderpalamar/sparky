use crate::domain::status::{Status, Value};
use crate::linux::files;
use crate::linux::host::Host;

const VERSIONS: [&str; 2] = ["gtk-3.0", "gtk-4.0"];
const SECTION: &str = "[Settings]";
const ICONS: &str = "gtk-icon-theme-name";
const FONT: &str = "gtk-font-name";
const SEPARATOR: &str = " · ";

pub fn icons(host: &dyn Host) -> Result<Value, Status> {
    lookup(host, ICONS)?
        .map(Value::Text)
        .ok_or_else(files::missing)
}

pub fn font(host: &dyn Host) -> Result<Value, Status> {
    let desktop = lookup(host, FONT)?;
    Ok(Value::Text(fonts(
        sparky_theme::FONT_FAMILY,
        desktop.as_deref(),
    )))
}

pub fn fonts(interface: &str, desktop: Option<&str>) -> String {
    match desktop {
        Some(desktop) if desktop != interface => format!("{interface}{SEPARATOR}{desktop}"),
        Some(_) | None => interface.to_owned(),
    }
}

pub fn setting(ini: &str, key: &str) -> Option<String> {
    ini.lines()
        .map(str::trim)
        .skip_while(|line| *line != SECTION)
        .skip(1)
        .take_while(|line| !line.starts_with('['))
        .filter_map(|line| line.split_once('='))
        .find(|(name, _)| name.trim() == key)
        .map(|(_, value)| value.trim().trim_matches('"').trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn lookup(host: &dyn Host, key: &str) -> Result<Option<String>, Status> {
    let Some(config) = files::home(host, "XDG_CONFIG_HOME", ".config") else {
        return Ok(None);
    };
    for version in VERSIONS {
        let path = config.join(version).join("settings.ini");
        let text = match files::read(host, &path) {
            Ok(text) => text,
            Err(Status::Unavailable { .. }) => continue,
            Err(status) => return Err(status),
        };
        if let Some(value) = setting(&text, key) {
            return Ok(Some(value));
        }
    }
    Ok(None)
}
