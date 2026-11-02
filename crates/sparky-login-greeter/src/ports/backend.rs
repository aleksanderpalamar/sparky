use crate::domain::form::{Credentials, Refusal};
use crate::domain::power::Power;

pub trait Backend: Send + Sync {
    fn login(&self, credentials: Credentials, session: &str) -> Result<(), Refusal>;

    fn power(&self, action: Power) -> Result<(), String>;
}
