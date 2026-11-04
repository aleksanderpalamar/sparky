mod locale;

use std::path::{Path, PathBuf};

use fluent_bundle::{FluentArgs, FluentResource, concurrent::FluentBundle};
use unic_langid::LanguageIdentifier;

pub use locale::{Locale, locale_from_environment};

pub struct Catalog {
    installed: Option<FluentBundle<FluentResource>>,
    selected: FluentBundle<FluentResource>,
    fallback: FluentBundle<FluentResource>,
}

impl Catalog {
    pub fn for_domain(domain: &str, portuguese: &str, english: &str) -> Result<Self, String> {
        let locale = locale_from_environment(|key| std::env::var(key).ok());
        let paths = std::env::var_os("XDG_DATA_DIRS")
            .map(|value| std::env::split_paths(&value).collect::<Vec<_>>())
            .unwrap_or_else(|| {
                vec![
                    PathBuf::from("/usr/local/share"),
                    PathBuf::from("/usr/share"),
                ]
            });
        let directories = paths.iter().map(PathBuf::as_path).collect::<Vec<_>>();
        Self::with_installed(&locale, domain, &directories, portuguese, english)
    }

    pub fn new(locale: &Locale, portuguese: &str, english: &str) -> Result<Self, String> {
        let selected_text = match locale.language() {
            "en" => english,
            _ => portuguese,
        };
        Ok(Self {
            installed: None,
            selected: bundle(locale.language(), selected_text)?,
            fallback: bundle("pt-BR", portuguese)?,
        })
    }

    pub fn with_installed(
        locale: &Locale,
        domain: &str,
        directories: &[&Path],
        portuguese: &str,
        english: &str,
    ) -> Result<Self, String> {
        let selected = locale
            .candidates()
            .into_iter()
            .find_map(|language| installed(directories, &language, domain));
        let mut catalog = Self::new(locale, portuguese, english)?;
        if let Some(text) = selected {
            catalog.installed = bundle(locale.language(), &text).ok();
        }
        Ok(catalog)
    }

    pub fn text(&self, key: &str) -> Option<String> {
        self.message(key, None)
    }

    pub fn message(&self, key: &str, args: Option<&FluentArgs<'_>>) -> Option<String> {
        self.installed
            .as_ref()
            .and_then(|installed| render(installed, key, args))
            .or_else(|| render(&self.selected, key, args))
            .or_else(|| render(&self.fallback, key, args))
    }
}

fn installed(directories: &[&Path], language: &str, domain: &str) -> Option<String> {
    directories.iter().find_map(|directory| {
        let file = directory
            .join("sparky/i18n")
            .join(language)
            .join(format!("{domain}.ftl"));
        std::fs::read_to_string(file).ok()
    })
}

fn bundle(locale: &str, text: &str) -> Result<FluentBundle<FluentResource>, String> {
    let language: LanguageIdentifier = locale.parse().map_err(|error| format!("{error}"))?;
    let resource =
        FluentResource::try_new(text.to_owned()).map_err(|(_, errors)| format!("{errors:?}"))?;
    let mut bundle = FluentBundle::new_concurrent(vec![language]);
    bundle.set_use_isolating(false);
    bundle
        .add_resource(resource)
        .map_err(|errors| format!("{errors:?}"))?;
    Ok(bundle)
}

fn render(
    bundle: &FluentBundle<FluentResource>,
    key: &str,
    args: Option<&FluentArgs<'_>>,
) -> Option<String> {
    let pattern = bundle.get_message(key)?.value()?;
    let mut errors = Vec::new();
    let value = bundle.format_pattern(pattern, args, &mut errors);
    errors.is_empty().then(|| value.into_owned())
}

#[cfg(test)]
mod tests;
