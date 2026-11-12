pub mod command;
mod flatpak;
pub mod icons;
mod native;
mod native_detect;
mod native_parse;
mod native_updates_parse;

#[cfg(test)]
mod flatpak_tests;
#[cfg(test)]
mod flatpak_updates_tests;
#[cfg(test)]
mod native_tests;
#[cfg(test)]
mod native_updates_tests;
#[cfg(test)]
mod updates_tests;

use std::sync::Arc;

use crate::domain::updates::{UpdateGroup, UpdateReport, UpdateSource, UpdateTarget};
use crate::domain::{Action, Application, SearchResult, SearchSource};
use crate::ports::StoreBackend;

use self::command::{Process, Runner};
use self::flatpak::Flatpak;
use self::native::Native;

#[derive(Debug)]
pub struct SystemStore {
    flatpak: Flatpak,
    native: Native,
}

impl Default for SystemStore {
    fn default() -> Self {
        let runner: Arc<dyn Runner> = Arc::new(Process);
        Self {
            flatpak: Flatpak::new(runner.clone()),
            native: Native::new(runner),
        }
    }
}

impl StoreBackend for SystemStore {
    fn browse(&self) -> Result<SearchResult, String> {
        let Some(scope) = self.flatpak.scopes().first().copied() else {
            return Ok(SearchResult {
                apps: Vec::new(),
                source: None,
                warning: Some(crate::i18n::text("flathub-unavailable")),
                flathub_missing: true,
            });
        };
        Ok(SearchResult {
            apps: self.flatpak.browse(scope)?,
            source: Some(crate::domain::Source::Flathub(scope)),
            warning: None,
            flathub_missing: false,
        })
    }

    fn search(&self, query: &str, source: SearchSource) -> Result<SearchResult, String> {
        let query = query.trim();
        if query.is_empty() || query.len() > 80 {
            return Err(crate::i18n::text("invalid-search"));
        }
        if source == SearchSource::Native {
            let apps = self.native.search(query)?;
            return Ok(SearchResult {
                source: self.native.manager().map(crate::domain::Source::Native),
                apps,
                warning: None,
                flathub_missing: self.flatpak.scopes().is_empty(),
            });
        }
        let scopes = self.flatpak.scopes();
        if scopes.is_empty() {
            return crate::domain::choose_source(
                Err(crate::i18n::text("flathub-unavailable")),
                self.native.search(query),
                true,
            );
        }
        let flatpak = self.flatpak.search(query, &scopes);
        if flatpak.as_ref().is_ok_and(|apps| !apps.is_empty()) {
            return crate::domain::choose_source(flatpak, Ok(Vec::new()), false);
        }
        crate::domain::choose_source(flatpak, self.native.search(query), false)
    }

    fn installed(&self) -> Result<Vec<Application>, String> {
        let preferred = self.flatpak.installed();
        let native = self.native.installed();
        match (preferred, native) {
            (Ok(mut flatpaks), Ok(packages)) => {
                flatpaks.extend(packages);
                Ok(flatpaks)
            }
            (Ok(flatpaks), Err(_)) => Ok(flatpaks),
            (Err(_), Ok(packages)) => Ok(packages),
            (Err(flatpak), Err(native)) => Err(format!("{flatpak}; {native}")),
        }
    }

    fn perform(&self, app: &Application, action: Action) -> Result<(), String> {
        match app.source {
            crate::domain::Source::Flathub(scope) => self.flatpak.perform(app, action, scope),
            crate::domain::Source::Native(manager) => self.native.perform(app, action, manager),
        }
    }

    fn enable_flathub(&self) -> Result<(), String> {
        self.flatpak.enable()
    }

    fn check_updates(&self, refresh: bool) -> UpdateReport {
        let mut groups = Vec::new();
        for scope in [
            crate::domain::Installation::User,
            crate::domain::Installation::System,
        ] {
            let result = self.flatpak.updates(scope);
            groups.push(update_group(UpdateSource::Flatpak(scope), result));
        }
        if let Some(manager) = self.native.manager() {
            groups.push(update_group(
                UpdateSource::Native(manager),
                self.native.updates(refresh),
            ));
        }
        UpdateReport {
            groups,
            warning: self
                .native
                .manager()
                .is_none()
                .then(|| crate::i18n::text("native-unavailable")),
        }
    }

    fn apply_update(&self, target: &UpdateTarget) -> Result<(), String> {
        match target {
            UpdateTarget::Flatpak { scope, reference } => {
                self.flatpak.apply_update(*scope, reference.as_deref())
            }
            UpdateTarget::Native { manager, package } => {
                self.native.apply_update(*manager, package.as_deref())
            }
        }
    }
}

fn update_group(
    source: UpdateSource,
    result: Result<Vec<crate::domain::updates::UpdateItem>, String>,
) -> UpdateGroup {
    match result {
        Ok(items) => UpdateGroup {
            source,
            items,
            error: None,
        },
        Err(error) => UpdateGroup {
            source,
            items: Vec::new(),
            error: Some(error),
        },
    }
}
