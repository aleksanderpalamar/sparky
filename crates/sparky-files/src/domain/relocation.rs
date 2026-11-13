use std::path::{Path, PathBuf};

pub fn relocate(path: &Path, moves: &[(PathBuf, PathBuf)]) -> PathBuf {
    moves
        .iter()
        .filter_map(|(from, to)| {
            let suffix = path.strip_prefix(from).ok()?;
            Some((from.components().count(), moved(to, suffix)))
        })
        .max_by_key(|(depth, _)| *depth)
        .map_or_else(|| path.to_path_buf(), |(_, moved)| moved)
}

fn moved(to: &Path, suffix: &Path) -> PathBuf {
    match suffix.as_os_str().is_empty() {
        true => to.to_path_buf(),
        false => to.join(suffix),
    }
}
