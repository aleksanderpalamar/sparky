use crate::domain::form::{Credentials, Refusal};
use crate::domain::power::{Power, label};
use crate::ports::backend::Backend;

pub struct Preview;

impl Backend for Preview {
    fn login(&self, _credentials: Credentials, _session: &str) -> Result<(), Refusal> {
        Err(Refusal::Denied)
    }

    fn power(&self, action: Power) -> Result<(), String> {
        Err(format!("na prévia, {} não faz nada", label(action)))
    }
}
