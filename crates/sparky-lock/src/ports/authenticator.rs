use sparky_login_core::protocol::Refusal;
use zeroize::Zeroizing;

pub trait Authenticator: Send + Sync {
    fn unlock(&self, password: Zeroizing<String>) -> Result<(), Refusal>;
}
