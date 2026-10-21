use std::env;

const MAIN_GROUP: &str = "[Desktop Entry]";

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Localized {
    default: Option<String>,
    translations: Vec<(String, String)>,
}

impl Localized {
    pub fn get(&self, locale: &str) -> Option<&str> {
        let language = locale.split('_').next().unwrap_or(locale);
        self.translation(locale)
            .or_else(|| self.translation(language))
            .or(self.default.as_deref())
    }

    fn translation(&self, locale: &str) -> Option<&str> {
        self.translations
            .iter()
            .find(|(known, _)| known == locale)
            .map(|(_, value)| value.as_str())
    }

    fn read(pairs: &[(&str, &str)], key: &str) -> Self {
        let default = pairs.iter().find(|(name, _)| *name == key).map(|(_, value)| (*value).to_owned());
        let translations = pairs
            .iter()
            .filter_map(|(name, value)| {
                let locale = name.strip_prefix(key)?.strip_prefix('[')?.strip_suffix(']')?;
                Some((locale.to_owned(), (*value).to_owned()))
            })
            .collect();
        Self { default, translations }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopEntry {
    pub name: Localized,
    pub generic_name: Localized,
    pub comment: Localized,
    pub keywords: Localized,
    pub icon: Option<String>,
    pub exec: Option<String>,
    pub try_exec: Option<String>,
    pub terminal: bool,
    pub no_display: bool,
    pub hidden: bool,
    pub enabled: bool,
    pub only_show_in: Vec<String>,
    pub not_show_in: Vec<String>,
}

impl DesktopEntry {
    pub fn parse(contents: &str) -> Option<Self> {
        let pairs = main_group(contents)?;
        let value = |key: &str| pairs.iter().find(|(name, _)| *name == key).map(|(_, value)| *value);
        let flag = |key: &str| value(key).is_some_and(|flag| flag == "true");
        if value("Type").is_some_and(|kind| kind != "Application") {
            return None;
        }
        Some(Self {
            name: Localized::read(&pairs, "Name"),
            generic_name: Localized::read(&pairs, "GenericName"),
            comment: Localized::read(&pairs, "Comment"),
            keywords: Localized::read(&pairs, "Keywords"),
            icon: value("Icon").map(str::to_owned),
            exec: value("Exec").map(str::to_owned),
            try_exec: value("TryExec").map(str::to_owned),
            terminal: flag("Terminal"),
            no_display: flag("NoDisplay"),
            hidden: flag("Hidden"),
            enabled: value("X-GNOME-Autostart-enabled").is_none_or(|flag| flag != "false"),
            only_show_in: list_of(value("OnlyShowIn")),
            not_show_in: list_of(value("NotShowIn")),
        })
    }
}

impl DesktopEntry {
    pub fn shown_in(&self, desktop: &str) -> bool {
        let allowed = self.only_show_in.is_empty() || self.only_show_in.iter().any(|name| name == desktop);
        let excluded = self.not_show_in.iter().any(|name| name == desktop);
        !self.hidden && allowed && !excluded
    }

    pub fn installed(&self, present: impl Fn(&str) -> bool) -> bool {
        self.try_exec.as_deref().is_none_or(present)
    }
}

pub fn list_of(value: Option<&str>) -> Vec<String> {
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

pub fn locale_of(value: &str) -> String {
    value.split(['.', '@']).next().unwrap_or(value).to_owned()
}

pub fn locale_from_env() -> String {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .find_map(|key| env::var(key).ok().filter(|value| !value.is_empty()))
        .map_or_else(|| "C".to_owned(), |value| locale_of(&value))
}

pub fn command_of(exec: &str) -> String {
    let mut command = String::with_capacity(exec.len());
    let mut characters = exec.chars();
    while let Some(character) = characters.next() {
        if character != '%' {
            command.push(character);
            continue;
        }
        if characters.next() == Some('%') {
            command.push('%');
        }
    }
    command.split_whitespace().collect::<Vec<_>>().join(" ")
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
