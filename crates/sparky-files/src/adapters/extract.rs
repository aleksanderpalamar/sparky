use std::fs::{self, Permissions};
use std::io::{self, BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::Duration;

use tempfile::TempDir;

use crate::adapters::staged::rename_noreplace;
use crate::domain::archive::{Extraction, Placement, extractable, placement};
use crate::domain::naming::free_name;
use crate::domain::transfer::Progress;
use crate::i18n;

const TOOL: &str = "bsdtar";
const POLL: Duration = Duration::from_millis(100);

enum Followed {
    Finished(Option<String>),
    Cancelled,
}

pub fn extract(
    archive: &Path,
    into: &Path,
    advance: &mut dyn FnMut(Progress) -> bool,
) -> io::Result<Extraction> {
    let mut progress = Progress {
        items_total: count(archive)?,
        ..Progress::default()
    };
    if !advance(progress) {
        return Ok(Extraction::Cancelled);
    }
    let staging = tempfile::Builder::new()
        .prefix(".sparky-files-")
        .tempdir_in(into)?;
    let mut child = Command::new(TOOL)
        .arg("-xvf")
        .arg(archive)
        .arg("-C")
        .arg(staging.path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;
    let Followed::Finished(complaint) = follow(&mut child, &mut progress, advance)? else {
        let _ = child.kill();
        let _ = child.wait();
        return Ok(Extraction::Cancelled);
    };
    if !child.wait()?.success() {
        return Err(io::Error::other(
            complaint.unwrap_or_else(|| i18n::text("extract-failed")),
        ));
    }
    if !advance(progress) {
        return Ok(Extraction::Cancelled);
    }
    place(staging, into, &stem(archive)).map(Extraction::Done)
}

fn count(archive: &Path) -> io::Result<usize> {
    let listed = Command::new(TOOL)
        .arg("-tf")
        .arg(archive)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()?;
    if !listed.status.success() {
        return Err(io::Error::other(i18n::text("extract-failed")));
    }
    Ok(listed
        .stdout
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .count())
}

fn follow(
    child: &mut Child,
    progress: &mut Progress,
    advance: &mut dyn FnMut(Progress) -> bool,
) -> io::Result<Followed> {
    let Some(stderr) = child.stderr.take() else {
        return Ok(Followed::Finished(None));
    };
    let lines = read_lines(stderr);
    let mut complaint = None;
    loop {
        match lines.recv_timeout(POLL) {
            Ok(line) => {
                let line = line?;
                if !line.starts_with("x ") {
                    complaint = Some(line);
                    continue;
                }
                progress.items_done += 1;
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return Ok(Followed::Finished(complaint)),
        }
        if !advance(*progress) {
            return Ok(Followed::Cancelled);
        }
    }
}

fn read_lines(stderr: ChildStderr) -> Receiver<io::Result<String>> {
    let (sender, lines) = mpsc::channel();
    thread::spawn(move || {
        for line in BufReader::new(stderr).lines() {
            if sender.send(line).is_err() {
                return;
            }
        }
    });
    lines
}

fn place(staging: TempDir, into: &Path, stem: &str) -> io::Result<PathBuf> {
    let tops: Vec<_> = fs::read_dir(staging.path())?
        .filter_map(Result::ok)
        .map(|item| item.file_name())
        .collect();
    match placement(tops) {
        Placement::Single(name) => {
            let target = free(into, &name.to_string_lossy());
            rename_noreplace(&staging.path().join(&name), &target)?;
            Ok(target)
        }
        Placement::Folder => {
            let target = free(into, stem);
            fs::set_permissions(staging.path(), Permissions::from_mode(0o755))?;
            rename_noreplace(staging.path(), &target)?;
            let _ = staging.keep();
            Ok(target)
        }
    }
}

fn free(into: &Path, name: &str) -> PathBuf {
    into.join(free_name(name, |candidate| {
        into.join(candidate).symlink_metadata().is_ok()
    }))
}

fn stem(archive: &Path) -> String {
    let name = archive
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    extractable(&name).unwrap_or(&name).to_owned()
}
