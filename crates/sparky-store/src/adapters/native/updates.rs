use super::Native;
use crate::adapters::command::CommandOutput;
use crate::adapters::native_updates_parse::{apt, dnf, pacman, rpm_versions};
use crate::domain::Manager;
use crate::domain::updates::{UpdateItem, valid_native_package};

fn output_error(program: &str, output: CommandOutput) -> String {
    let detail = output.stderr.trim();
    if detail.is_empty() {
        return format!("{program} exited with {:?}", output.code);
    }
    detail.to_owned()
}

impl Native {
    pub fn updates(&self, refresh: bool) -> Result<Vec<UpdateItem>, String> {
        let manager = self
            .manager
            .ok_or_else(|| crate::i18n::text("native-unavailable"))?;
        match manager {
            Manager::Pacman => self.pacman_updates(),
            Manager::Apt => self.apt_updates(refresh),
            Manager::Dnf => self.dnf_updates(),
        }
    }

    fn pacman_updates(&self) -> Result<Vec<UpdateItem>, String> {
        let output = self.runner.run_status("checkupdates", &["--nocolor"])?;
        match output.code {
            Some(0) => pacman(&output.stdout),
            Some(2) => Ok(Vec::new()),
            _ => Err(output_error("checkupdates", output)),
        }
    }

    fn apt_updates(&self, refresh: bool) -> Result<Vec<UpdateItem>, String> {
        if refresh {
            self.runner.run("pkexec", &["apt-get", "update"])?;
        }
        let output = self
            .runner
            .run("apt-get", &["-s", "--with-new-pkgs", "upgrade"])?;
        apt(&output)
    }

    fn dnf_updates(&self) -> Result<Vec<UpdateItem>, String> {
        let installed = self.runner.run(
            "rpm",
            &["-qa", "--qf", "%{NAME}.%{ARCH}\t%{VERSION}-%{RELEASE}\\n"],
        )?;
        let output = self
            .runner
            .run_status("dnf", &["--quiet", "check-update"])?;
        match output.code {
            Some(0) => Ok(Vec::new()),
            Some(100) => dnf(&output.stdout, &rpm_versions(&installed)),
            _ => Err(output_error("dnf", output)),
        }
    }

    pub fn apply_update(&self, manager: Manager, package: Option<&str>) -> Result<(), String> {
        if self.manager != Some(manager) || package.is_some_and(|id| !valid_native_package(id)) {
            return Err(crate::i18n::text("invalid-package"));
        }
        let args = match (manager, package) {
            (Manager::Pacman, _) => vec!["pacman", "-Syu", "--noconfirm"],
            (Manager::Apt, Some(package)) => {
                vec!["apt-get", "install", "--only-upgrade", "-y", "--", package]
            }
            (Manager::Apt, None) => vec!["apt-get", "upgrade", "--with-new-pkgs", "-y"],
            (Manager::Dnf, Some(package)) => vec!["dnf", "upgrade", "-y", "--", package],
            (Manager::Dnf, None) => vec!["dnf", "upgrade", "-y"],
        };
        if manager == Manager::Apt && package.is_none() {
            self.runner.run("pkexec", &["apt-get", "update"])?;
        }
        self.runner.run("pkexec", &args).map(|_| ())
    }
}
