use sparky_login_core::protocol::Refusal;
use zeroize::Zeroizing;

pub trait Transaction {
    fn authenticate(&mut self) -> Result<(), Refusal>;
    fn authorize(&mut self) -> Result<(), Refusal>;
    fn set_tty(&mut self, tty: &str) -> Result<(), String>;
    fn put(&mut self, name: &str, value: &str) -> Result<(), String>;
    fn establish(&mut self) -> Result<(), String>;
    fn open(&mut self) -> Result<(), String>;
    fn environment(&self) -> Vec<(String, String)>;
    fn close(&mut self);
}

pub trait Pam {
    fn begin(
        &self,
        service: &str,
        user: &str,
        password: Option<Zeroizing<String>>,
    ) -> Result<Box<dyn Transaction>, String>;
}
