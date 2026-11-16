use crate::domain::library::Library;

pub trait LibraryStore {
    fn load(&self) -> Result<Library, String>;
    fn save(&self, library: &Library) -> Result<(), String>;
}
