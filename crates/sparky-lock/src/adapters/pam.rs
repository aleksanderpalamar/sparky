use sparky_login_core::protocol::Refusal;
use sparky_pam::{LinuxPam, Pam};
use zeroize::Zeroizing;

use crate::ports::authenticator::Authenticator;

pub const SERVICE: &str = "sparky-lock";

pub struct PamAuthenticator {
    pam: LinuxPam,
    user: String,
}

impl PamAuthenticator {
    pub fn new(pam: LinuxPam, user: String) -> Self {
        Self { pam, user }
    }
}

impl Authenticator for PamAuthenticator {
    fn unlock(&self, password: Zeroizing<String>) -> Result<(), Refusal> {
        let mut transaction = self
            .pam
            .begin(SERVICE, &self.user, Some(password))
            .map_err(Refusal::Unavailable)?;
        transaction.authenticate()
    }
}
