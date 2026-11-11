use std::collections::HashSet;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::thread::{self, JoinHandle};
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

fn run(binary: &Path, args: &[&str]) -> Result<Output, String> {
    let mut child = Command::new(binary)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;
    let stdout = child.stdout.take().ok_or_else(command_failed)?;
    let stderr = child.stderr.take().ok_or_else(command_failed)?;
    let stdout_reader = thread::spawn(move || read_all(stdout));
    let stderr_reader = thread::spawn(move || read_all(stderr));
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let output = Output {
                    status,
                    stdout: collect(stdout_reader)?,
                    stderr: collect(stderr_reader)?,
                };
                if output.status.success() {
                    return Ok(output);
                }
                let error = String::from_utf8_lossy(&output.stderr).trim().to_owned();
                return Err(if error.is_empty() {
                    command_failed()
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

fn read_all(mut pipe: impl Read) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    pipe.read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    Ok(bytes)
}

fn collect(reader: JoinHandle<Result<Vec<u8>, String>>) -> Result<Vec<u8>, String> {
    reader.join().map_err(|_| command_failed())?
}

fn command_failed() -> String {
    crate::i18n::text("agents-local-command-failed")
}

#[cfg(test)]
mod tests {
    use super::{models, names};
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn extrai_modelos_sem_confundir_cabecalho_ou_colunas() {
        let table = b"NAME    ID    SIZE    MODIFIED\nqwen3:8b abc 5 GB today\nllama3:latest def 4 GB yesterday\n";
        assert_eq!(names(table), vec!["qwen3:8b", "llama3:latest"]);
    }

    #[test]
    fn lista_grande_nao_bloqueia_o_pipe() {
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join("ollama");
        let script = "#!/bin/sh\nprintf 'NAME ID SIZE MODIFIED\\n'\ni=0\nwhile [ \"$i\" -lt 2000 ]; do\n  printf 'modelo-%04d:latest abc 1GB today\\n' \"$i\"\n  i=$((i + 1))\ndone\n";
        fs::write(&binary, script).unwrap();
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).unwrap();

        let found = models(&binary).unwrap();
        assert_eq!(found.len(), 2000);
        assert!(found.iter().all(|model| model.running));
    }
}
