const MAIN_GROUP: &str = "[Desktop Entry]";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopEntry {
    pub exec: Option<String>,
    pub try_exec: Option<String>,
    pub hidden: bool,
    pub enabled: bool,
    pub only_show_in: Vec<String>,
    pub not_show_in: Vec<String>,
}

impl DesktopEntry {
    pub fn parse(contents: &str) -> Option<Self> {
        let pairs = main_group(contents)?;
        let value = |key: &str| {
            pairs
                .iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| value.to_string())
        };
        if value("Type").is_some_and(|kind| kind != "Application") {
            return None;
        }
        Some(Self {
            exec: value("Exec"),
            try_exec: value("TryExec"),
            hidden: value("Hidden").is_some_and(|flag| flag == "true"),
            enabled: value("X-GNOME-Autostart-enabled").is_none_or(|flag| flag != "false"),
            only_show_in: list(value("OnlyShowIn")),
            not_show_in: list(value("NotShowIn")),
        })
    }
}

fn main_group(contents: &str) -> Option<Vec<(&str, &str)>> {
    let mut lines = contents.lines().map(str::trim);
    lines.find(|line| *line == MAIN_GROUP)?;
    let pairs = lines
        .take_while(|line| !line.starts_with('['))
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.trim(), value.trim()))
        .collect();
    Some(pairs)
}

fn list(value: Option<String>) -> Vec<String> {
    value
        .map(|value| {
            value
                .split(';')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}
