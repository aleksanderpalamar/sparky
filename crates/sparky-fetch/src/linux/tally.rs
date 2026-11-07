use crate::domain::status::Status;
use crate::i18n;
use crate::linux::files;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tally {
    Present(usize),
    Absent,
    Broken(Status),
}

impl From<Status> for Tally {
    fn from(status: Status) -> Self {
        match status {
            Status::Unavailable { .. } => Tally::Absent,
            other => Tally::Broken(other),
        }
    }
}

impl Tally {
    pub fn merge(self, other: Tally) -> Tally {
        match (self, other) {
            (Tally::Present(left), Tally::Present(right)) => Tally::Present(left + right),
            (Tally::Present(count), _) | (_, Tally::Present(count)) => Tally::Present(count),
            (Tally::Broken(status), _) | (_, Tally::Broken(status)) => Tally::Broken(status),
            (Tally::Absent, Tally::Absent) => Tally::Absent,
        }
    }
}

pub fn summary(tallies: &[(&str, Tally)]) -> Result<String, Status> {
    let counted: Vec<String> = tallies
        .iter()
        .filter_map(|(manager, tally)| match tally {
            Tally::Present(count) if *count > 0 => Some(format!("{count} ({manager})")),
            Tally::Present(_) | Tally::Absent | Tally::Broken(_) => None,
        })
        .collect();
    if !counted.is_empty() {
        return Ok(counted.join(", "));
    }
    let broken = tallies.iter().find_map(|(_, tally)| match tally {
        Tally::Broken(status) => Some(status.clone()),
        Tally::Present(_) | Tally::Absent => None,
    });
    Err(broken.unwrap_or_else(files::missing))
}

pub fn timed_out(command: &str) -> Status {
    Status::failed(i18n::with("error-command-timeout", "command", command))
}

pub fn command_failed(command: &str, error: &str) -> Status {
    Status::failed(i18n::pairs(
        "error-command",
        &[("command", command), ("error", error)],
    ))
}
