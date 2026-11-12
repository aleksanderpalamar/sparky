use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::command::Runner;
use crate::domain::{Action, Application, Installation, Source, valid_id};

mod catalog;
mod updates;

#[derive(Debug)]
pub struct Flatpak {
    runner: Arc<dyn Runner>,
}

impl Flatpak {
    pub fn new(runner: Arc<dyn Runner>) -> Self {
        Self { runner }
    }

    pub fn scopes(&self) -> Vec<Installation> {
        [Installation::System, Installation::User]
            .into_iter()
            .filter(|scope| self.has_flathub(*scope))
            .collect()
    }

    fn has_flathub(&self, scope: Installation) -> bool {
        let Ok(output) = self
            .runner
            .run("flatpak", &[scope.flag(), "remotes", "--columns=name"])
        else {
            return false;
        };
        output.lines().any(|name| name == "flathub")
    }

    pub fn browse(&self, scope: Installation) -> Result<Vec<Application>, String> {
        let output = self.runner.run(
            "flatpak",
            &[
                scope.flag(),
                "remote-ls",
                "--app",
                "--columns=application:full,name:full,description:full,version:full,branch:full",
                "flathub",
            ],
        )?;
        let installed: HashMap<_, _> = self
            .installed()?
            .into_iter()
            .map(|app| (app.id, app.source))
            .collect();
        catalog::parse(&output, scope, &installed)
    }

    pub fn search(&self, query: &str, scopes: &[Installation]) -> Result<Vec<Application>, String> {
        let installed: HashMap<_, _> = self
            .installed()?
            .into_iter()
            .map(|app| (app.id.clone(), app.source))
            .collect();
        let mut found = Vec::new();
        let mut seen = HashSet::new();
        for scope in scopes {
            let output = self
                .runner
                .run(
                    "flatpak",
                    &[
                        scope.flag(),
                        "search",
                        "--columns=application:full,name:full,description:full,version:full,remotes:full,branch:full",
                        query,
                    ],
                )?;
            let rows = catalog::parse_search(&output)?;
            for row in rows {
                if !row
                    .remotes
                    .split(',')
                    .any(|remote| remote.trim() == "flathub")
                {
                    continue;
                }
                if !seen.insert(row.application_id.clone()) {
                    continue;
                }
                let source = installed
                    .get(&row.application_id)
                    .copied()
                    .unwrap_or(Source::Flathub(*scope));
                found.push(Application {
                    id: row.application_id.clone(),
                    name: row.name,
                    summary: row.description.unwrap_or_default(),
                    version: row.version,
                    source,
                    installed: installed.contains_key(&row.application_id),
                });
            }
        }
        Ok(found)
    }

    pub fn installed(&self) -> Result<Vec<Application>, String> {
        let mut apps = Vec::new();
        for scope in [Installation::System, Installation::User] {
            let output = self.runner.run(
                "flatpak",
                &[
                    scope.flag(),
                    "list",
                    "--app",
                    "--columns=name:full,description:full,application:full,version:full,origin:full",
                ],
            )?;
            apps.extend(catalog::parse_installed(&output, scope)?);
        }
        Ok(apps)
    }

    pub fn perform(
        &self,
        app: &Application,
        action: Action,
        scope: Installation,
    ) -> Result<(), String> {
        if !valid_id(&app.id) {
            return Err(crate::i18n::text("invalid-package"));
        }
        let args = match action {
            Action::Install => vec![
                scope.flag(),
                "install",
                "--app",
                "-y",
                "--noninteractive",
                "flathub",
                &app.id,
            ],
            Action::Update => vec![
                scope.flag(),
                "update",
                "--app",
                "-y",
                "--noninteractive",
                &app.id,
            ],
            Action::Remove => vec![
                scope.flag(),
                "uninstall",
                "--app",
                "-y",
                "--noninteractive",
                &app.id,
            ],
        };
        self.runner.run("flatpak", &args).map(|_| ())
    }

    pub fn enable(&self) -> Result<(), String> {
        self.runner
            .run(
                "flatpak",
                &[
                    "--user",
                    "remote-add",
                    "--if-not-exists",
                    "flathub",
                    "https://dl.flathub.org/repo/flathub.flatpakrepo",
                ],
            )
            .map(|_| ())
    }
}
