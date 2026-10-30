use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Next,
    Previous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    First,
    Last,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Album {
    paths: Vec<PathBuf>,
    current: usize,
}

impl Album {
    pub fn new(mut paths: Vec<PathBuf>, start: &Path) -> Self {
        if !paths.iter().any(|path| path == start) {
            paths.push(start.to_path_buf());
        }
        paths.sort_by_cached_key(|path| order(path));
        paths.dedup();
        let current = paths.iter().position(|path| path == start).unwrap_or(0);
        Self { paths, current }
    }

    pub fn current(&self) -> &Path {
        &self.paths[self.current]
    }

    pub fn position(&self) -> (usize, usize) {
        (self.current + 1, self.paths.len())
    }

    pub fn step(&mut self, step: Step) {
        let total = self.paths.len();
        self.current = match step {
            Step::Next => (self.current + 1) % total,
            Step::Previous => (self.current + total - 1) % total,
        };
    }

    pub fn jump(&mut self, edge: Edge) {
        self.current = match edge {
            Edge::First => 0,
            Edge::Last => self.paths.len() - 1,
        };
    }
}

fn order(path: &Path) -> (String, String) {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    (name.to_lowercase(), name)
}
