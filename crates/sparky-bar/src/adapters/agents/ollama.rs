use std::collections::HashSet;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
    pub name: String,
    pub running: bool,
}

pub fn models(binary: &Path) -> Result<Vec<Model>, String> {
    let installed = names(&run(binary, &["list"])?.stdout);
    let running: HashSet<_> = names(&run(binary, &["ps"])?.stdout).into_iter().collect();
    Ok(installed
        .into_iter()
        .map(|name| Model {
            running: running.contains(&name),
            name,
        })
        .collect())
}

pub fn stop(binary: &Path, model: &str) -> Result<(), String> {
    run(binary, &["stop", model]).map(|_| ())
}

fn names(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .skip(1)
        .filter_map(|line| line.split_whitespace().next())
        .map(str::to_owned)
        .collect()
}

fn run(binary: &Path, args: &[&str]) -> Result<std::process::Output, String> {
    let mut child = Command::new(binary)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                let output = child
                    .wait_with_output()
                    .map_err(|error| error.to_string())?;
                if output.status.success() {
                    return Ok(output);
                }
                let error = String::from_utf8_lossy(&output.stderr).trim().to_owned();
                return Err(if error.is_empty() {
                    crate::i18n::text("agents-local-command-failed")
                } else {
                    error
                });
            }
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(25)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(crate::i18n::text("agents-local-timeout"));
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error.to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::names;

    #[test]
    fn extrai_modelos_sem_confundir_cabecalho_ou_colunas() {
        let table = b"NAME    ID    SIZE    MODIFIED\nqwen3:8b abc 5 GB today\nllama3:latest def 4 GB yesterday\n";
        assert_eq!(names(table), vec!["qwen3:8b", "llama3:latest"]);
    }
}
