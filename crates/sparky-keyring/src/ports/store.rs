use std::io;

pub trait VaultStore: Send {
    fn load(&self) -> io::Result<Option<String>>;
    fn save(&self, contents: &str) -> io::Result<()>;
}
