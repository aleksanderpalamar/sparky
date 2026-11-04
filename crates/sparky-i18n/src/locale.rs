#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locale(String);

impl Locale {
    pub fn parse(value: &str) -> Self {
        let value = value.split(['.', '@']).next().unwrap_or_default();
        let value = value.replace('_', "-");
        match value.as_str() {
            "" | "C" | "POSIX" => Self("pt-BR".to_owned()),
            _ => Self(value),
        }
    }

    pub fn language(&self) -> &str {
        self.0.split('-').next().unwrap_or("pt")
    }

    pub fn candidates(&self) -> Vec<String> {
        let language = self.language().to_owned();
        if self.0 == language {
            return vec![language];
        }
        vec![self.0.clone(), language]
    }
}

pub fn locale_from_environment(get: impl Fn(&str) -> Option<String>) -> Locale {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .into_iter()
        .find_map(get)
        .map_or_else(|| Locale::parse("pt-BR"), |value| Locale::parse(&value))
}
