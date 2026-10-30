use std::fs;

use crate::ports::SystemFiles;

pub struct RootFs;

impl SystemFiles for RootFs {
    fn read(&self, path: &str) -> Option<String> {
        fs::read(path)
            .ok()
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
    }

    fn list(&self, path: &str) -> Vec<String> {
        let Ok(entries) = fs::read_dir(path) else {
            return Vec::new();
        };
        entries
            .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
            .collect()
    }

    fn link(&self, path: &str) -> Option<String> {
        fs::read_link(path)
            .ok()
            .map(|target| target.to_string_lossy().into_owned())
    }
}
