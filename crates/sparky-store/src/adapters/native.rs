use std::collections::HashSet;
use std::fs;
use std::sync::Arc;

use super::command::Runner;
use super::native_detect::detect;
use super::native_parse::{parse_apt, parse_dnf, parse_pacman};
use crate::domain::{Action, Application, Manager, valid_id};

mod updates;

#[derive(Debug)]
pub struct Native {
    runner: Arc<dyn Runner>,
    manager: Option<Manager>,
}

impl Native {
    pub fn new(runner: Arc<dyn Runner>) -> Self {
        let release = fs::read_to_string("/etc/os-release")
            .or_else(|_| fs::read_to_string("/usr/lib/os-release"))
            .unwrap_or_default();
        Self::from_release(runner, &release)
    }

    pub fn from_release(runner: Arc<dyn Runner>, release: &str) -> Self {
        Self {
            runner,
            manager: detect(release),
        }
    }

    pub fn manager(&self) -> Option<Manager> {
        self.manager
    }

    pub fn search(&self, query: &str) -> Result<Vec<Application>, String> {
        let manager = self
            .manager
            .ok_or_else(|| crate::i18n::text("native-unavailable"))?;
        let output = match manager {
            Manager::Pacman => self
                .runner
                .run("pacman", &["-Ss", "--color", "never", "--", query])?,
            Manager::Apt => self.runner.run("apt-cache", &["search", "--", query])?,
            Manager::Dnf => self.runner.run(
                "dnf",
                &[
                    "repoquery",
                    "--qf",
                    "%{name}\t%{summary}\t%{version}\\n",
                    &format!("*{query}*"),
                ],
            )?,
        };
        let mut apps = match manager {
            Manager::Pacman => parse_pacman(&output),
            Manager::Apt => parse_apt(&output),
            Manager::Dnf => parse_dnf(&output),
        };
        let installed = self.installed_names(manager)?;
        for app in &mut apps {
            app.installed = installed.contains(&app.id);
        }
        Ok(apps)
    }

    fn installed_names(&self, manager: Manager) -> Result<HashSet<String>, String> {
        let output = match manager {
            Manager::Pacman => self.runner.run("pacman", &["-Qq"])?,
            Manager::Apt => self
                .runner
                .run("dpkg-query", &["-W", "-f=${Package}\t${Status}\n"])?,
            Manager::Dnf => self.runner.run("rpm", &["-qa", "--qf", "%{NAME}\n"])?,
        };
        Ok(output
            .lines()
            .filter_map(|line| match manager {
                Manager::Apt => line
                    .strip_suffix("\tinstall ok installed")
                    .map(str::to_owned),
                _ => Some(line.to_owned()),
            })
            .collect())
    }

    pub fn installed(&self) -> Result<Vec<Application>, String> {
        let manager = self
            .manager
            .ok_or_else(|| crate::i18n::text("native-unavailable"))?;
        let output = match manager {
            Manager::Pacman => self.runner.run("pacman", &["-Qqe"])?,
            Manager::Apt => self.runner.run("apt-mark", &["showmanual"])?,
            Manager::Dnf => self.runner.run(
                "dnf",
                &["repoquery", "--userinstalled", "--qf", "%{name}\\n"],
            )?,
        };
        Ok(output
            .lines()
            .filter(|name| valid_id(name))
            .map(|name| Application {
                id: name.to_owned(),
                name: name.to_owned(),
                summary: String::new(),
                version: None,
                source: crate::domain::Source::Native(manager),
                installed: true,
            })
            .collect())
    }

    pub fn perform(
        &self,
        app: &Application,
        action: Action,
        manager: Manager,
    ) -> Result<(), String> {
        if self.manager != Some(manager) || !valid_id(&app.id) {
            return Err(crate::i18n::text("invalid-package"));
        }
        let (program, args): (&str, Vec<&str>) = match (manager, action) {
            (Manager::Pacman, Action::Install) => (
                "pacman",
                vec!["-S", "--needed", "--noconfirm", "--", &app.id],
            ),
            (Manager::Pacman, Action::Update) => ("pacman", vec!["-Syu", "--noconfirm"]),
            (Manager::Pacman, Action::Remove) => {
                ("pacman", vec!["-R", "--noconfirm", "--", &app.id])
            }
            (Manager::Apt, Action::Install) => ("apt-get", vec!["install", "-y", "--", &app.id]),
            (Manager::Apt, Action::Update) => (
                "apt-get",
                vec!["install", "--only-upgrade", "-y", "--", &app.id],
            ),
            (Manager::Apt, Action::Remove) => ("apt-get", vec!["remove", "-y", "--", &app.id]),
            (Manager::Dnf, Action::Install) => ("dnf", vec!["install", "-y", "--", &app.id]),
            (Manager::Dnf, Action::Update) => ("dnf", vec!["upgrade", "-y", "--", &app.id]),
            (Manager::Dnf, Action::Remove) => ("dnf", vec!["remove", "-y", "--", &app.id]),
        };
        let mut command = vec![program];
        command.extend(args);
        self.runner.run("pkexec", &command).map(|_| ())
    }
}
