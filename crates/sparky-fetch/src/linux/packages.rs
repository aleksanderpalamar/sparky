use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::domain::status::{Status, Value};
use crate::linux::files;
use crate::linux::host::{Host, Ran};
use crate::linux::tally::{self, Tally};

const PACMAN: &str = "/var/lib/pacman/local";
const PACMAN_VERSION: &str = "ALPM_DB_VERSION";
const DPKG: &str = "/var/lib/dpkg/status";
const INSTALLED: &str = "Status: install ok installed";
const FLATPAK: &str = "/var/lib/flatpak/app";
const RPM_TIMEOUT: Duration = Duration::from_millis(300);

pub fn collect(host: &dyn Host) -> Result<Value, Status> {
    let tallies = [
        ("pacman", pacman(host)),
        ("dpkg", dpkg(host)),
        ("rpm", rpm(host)),
        ("flatpak", flatpak(host)),
    ];
    tally::summary(&tallies).map(Value::Text)
}

fn pacman(host: &dyn Host) -> Tally {
    files::list(host, Path::new(PACMAN)).map_or_else(Tally::from, |entries| {
        let packages = entries
            .iter()
            .filter(|entry| entry.file_name().is_some_and(|name| name != PACMAN_VERSION))
            .count();
        Tally::Present(packages)
    })
}

fn dpkg(host: &dyn Host) -> Tally {
    match files::read(host, Path::new(DPKG)) {
        Ok(text) => Tally::Present(
            text.lines()
                .filter(|line| line.trim_end() == INSTALLED)
                .count(),
        ),
        Err(status) => Tally::from(status),
    }
}

fn rpm(host: &dyn Host) -> Tally {
    match host.run("rpm", &["-qa"], RPM_TIMEOUT) {
        Ran::Output(text) => {
            Tally::Present(text.lines().filter(|line| !line.trim().is_empty()).count())
        }
        Ran::Missing => Tally::Absent,
        Ran::TimedOut => Tally::Broken(tally::timed_out("rpm")),
        Ran::Failed(error) => Tally::Broken(tally::command_failed("rpm", &error)),
    }
}

fn flatpak(host: &dyn Host) -> Tally {
    let user =
        files::home(host, "XDG_DATA_HOME", ".local/share").map(|data| data.join("flatpak/app"));
    let roots: Vec<PathBuf> = [Some(PathBuf::from(FLATPAK)), user]
        .into_iter()
        .flatten()
        .collect();
    roots
        .iter()
        .map(|root| {
            files::list(host, root).map_or_else(Tally::from, |apps| Tally::Present(apps.len()))
        })
        .fold(Tally::Absent, Tally::merge)
}
