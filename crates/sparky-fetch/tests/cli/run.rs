use std::path::Path;
use std::process::{Command, Output};

pub struct Run {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

pub fn fetch(arguments: &[&str], home: &Path, socket: Option<&Path>, locale: &str) -> Run {
    let mut command = Command::new(env!("CARGO_BIN_EXE_sparky-fetch"));
    command
        .args(arguments)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_DATA_DIRS", home.join("data"))
        .env("LC_ALL", locale)
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("SPARKY_SOCKET")
        .env_remove("NO_COLOR");
    if let Some(socket) = socket {
        command.env("SPARKY_SOCKET", socket);
    }
    finished(command.output())
}

fn finished(output: std::io::Result<Output>) -> Run {
    let output = output.unwrap_or_else(|error| panic!("sparky-fetch não rodou: {error}"));
    Run {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

pub fn json(run: &Run) -> serde_json::Value {
    serde_json::from_str(&run.stdout).unwrap_or_else(|error| panic!("{error}: {}", run.stdout))
}

pub fn entry<'a>(report: &'a serde_json::Value, field: &str) -> &'a serde_json::Value {
    report["entries"]
        .as_array()
        .and_then(|entries| entries.iter().find(|entry| entry["field"] == field))
        .unwrap_or_else(|| panic!("campo {field} ausente: {report}"))
}
