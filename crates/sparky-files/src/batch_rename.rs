use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::operations::Failure;
use crate::ports::Disk;

type Pair = (PathBuf, PathBuf);

pub fn rename_all(pairs: &[Pair], disk: &dyn Disk) -> Result<(), Failure> {
    let changed: Vec<&Pair> = pairs.iter().filter(|(from, to)| from != to).collect();
    let sources: BTreeSet<&Path> = changed.iter().map(|(from, _)| from.as_path()).collect();
    let chained = changed.iter().any(|(_, to)| sources.contains(to.as_path()));
    let moves: Vec<Pair> = match chained {
        false => changed.into_iter().cloned().collect(),
        true => through_temporaries(&changed, disk),
    };
    let mut done: Vec<&Pair> = Vec::new();
    for pair in &moves {
        if let Err(source) = disk.relocate(&pair.0, &pair.1) {
            undo(&done, disk);
            return Err(Failure::Io {
                name: original_name(pair, pairs),
                source,
            });
        }
        done.push(pair);
    }
    Ok(())
}

fn through_temporaries(changed: &[&Pair], disk: &dyn Disk) -> Vec<Pair> {
    let temporaries: Vec<PathBuf> = changed
        .iter()
        .enumerate()
        .map(|(index, (from, _))| temporary(from, index, disk))
        .collect();
    let out = changed
        .iter()
        .zip(&temporaries)
        .map(|((from, _), temporary)| (from.clone(), temporary.clone()));
    let back = changed
        .iter()
        .zip(&temporaries)
        .map(|((_, to), temporary)| (temporary.clone(), to.clone()));
    out.chain(back).collect()
}

fn temporary(from: &Path, index: usize, disk: &dyn Disk) -> PathBuf {
    let dir = from.parent().unwrap_or(Path::new("/"));
    let process = std::process::id();
    (0..)
        .map(|attempt| dir.join(format!(".sparky-files-rename-{process}-{index}-{attempt}")))
        .find(|candidate| !disk.exists(candidate))
        .unwrap_or_else(|| dir.join(".sparky-files-rename"))
}

fn undo(done: &[&Pair], disk: &dyn Disk) {
    for (from, to) in done.iter().rev() {
        let _ = disk.relocate(to, from);
    }
}

fn original_name(pair: &Pair, pairs: &[Pair]) -> String {
    let source = pairs
        .iter()
        .find(|(from, to)| from == &pair.0 || to == &pair.1)
        .map_or(&pair.0, |(from, _)| from);
    source
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}
