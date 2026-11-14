use std::path::{Path, PathBuf};

use crate::domain::tag_color::TagColor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Local(PathBuf),
    Remote(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bookmark {
    pub target: Target,
    pub label: Option<String>,
    pub color: Option<TagColor>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Added {
    New,
    Duplicate,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Bookmarks(Vec<Bookmark>);

impl Bookmark {
    pub fn local(path: PathBuf) -> Self {
        Self {
            target: Target::Local(path),
            label: None,
            color: None,
        }
    }

    pub fn path(&self) -> Option<&Path> {
        match &self.target {
            Target::Local(path) => Some(path),
            Target::Remote(_) => None,
        }
    }

    pub fn name(&self) -> String {
        if let Some(label) = self.label.as_ref().filter(|label| !label.is_empty()) {
            return label.clone();
        }
        match &self.target {
            Target::Local(path) => path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string()),
            Target::Remote(uri) => uri.clone(),
        }
    }
}

impl Bookmarks {
    pub fn new(items: Vec<Bookmark>) -> Self {
        Self(items)
    }

    pub fn items(&self) -> &[Bookmark] {
        &self.0
    }

    pub fn get(&self, index: usize) -> Option<&Bookmark> {
        self.0.get(index)
    }

    pub fn local(&self) -> impl Iterator<Item = (usize, &Bookmark)> {
        self.0
            .iter()
            .enumerate()
            .filter(|(_, bookmark)| bookmark.path().is_some())
    }

    pub fn contains(&self, path: &Path) -> bool {
        self.find(path).is_some()
    }

    pub fn find(&self, path: &Path) -> Option<usize> {
        self.0
            .iter()
            .position(|bookmark| bookmark.path() == Some(path))
    }

    pub fn add(&mut self, path: PathBuf) -> Added {
        if self.contains(&path) {
            return Added::Duplicate;
        }
        self.0.push(Bookmark::local(path));
        Added::New
    }

    pub fn paint(&mut self, index: usize, color: Option<TagColor>) -> bool {
        match self.0.get_mut(index) {
            Some(bookmark) if bookmark.color != color => {
                bookmark.color = color;
                true
            }
            _ => false,
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<Bookmark> {
        (index < self.0.len()).then(|| self.0.remove(index))
    }

    pub fn reorder(&mut self, from: usize, to: usize) -> bool {
        if from == to || from >= self.0.len() || to >= self.0.len() {
            return false;
        }
        let moved = self.0.remove(from);
        self.0.insert(to, moved);
        true
    }
}
