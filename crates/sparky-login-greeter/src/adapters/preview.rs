use crate::domain::form::{Credentials, Refusal};
use crate::domain::power::{Power, key};
use crate::ports::backend::Backend;
use fluent_bundle::FluentArgs;
use sparky_login_look::i18n;

pub struct Preview;

impl Backend for Preview {
    fn login(&self, _credentials: Credentials, _session: &str) -> Result<(), Refusal> {
        Err(Refusal::Denied)
    }

    fn power(&self, action: Power) -> Result<(), String> {
        let mut args = FluentArgs::new();
        args.set("action", i18n::text(key(action)));
        Err(i18n::message("preview-power", Some(&args)))
    }
}
