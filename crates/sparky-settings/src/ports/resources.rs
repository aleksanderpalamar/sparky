use sparky_resources::protocol::{Request, Status};

pub trait ResourceService {
    fn status(&self) -> Result<Status, String>;
    fn send(&self, request: Request) -> Result<(), String>;
}
